pub mod rmr_grpc {
    tonic::include_proto!("rmr_grpc");
}
use rmr_grpc::coordinator_service_client::CoordinatorServiceClient;
use rmr_grpc::{CurrentTask, TaskDescription, WorkerDescription};

use futures::{future::select, future::Either, pin_mut};
use tokio::time::sleep;
use tonic::Request;
use uuid::Uuid;

use std::collections::HashMap;
use std::io::Write;
use std::marker::PhantomData;
use std::time::Duration;

pub trait WorkerTrait {
    fn map(key: &str, value: &str) -> HashMap<String, Vec<String>>;
    fn reduce(key: &str, values: Vec<&str>) -> String;
}

pub struct MRWorker<T: WorkerTrait> {
    uuid: Uuid,
    client: CoordinatorServiceClient<tonic::transport::Channel>,
    current_task: CurrentTask,
    _phantom: PhantomData<fn() -> T>,
}

impl<T: WorkerTrait> MRWorker<T> {
    pub async fn new(client: String) -> Result<MRWorker<T>, Box<dyn std::error::Error>> {
        let client = CoordinatorServiceClient::connect(client).await?;
        let uuid = Uuid::new_v4();
        Ok(MRWorker {
            uuid,
            client,
            current_task: CurrentTask {
                uuid: uuid.as_bytes().to_vec(),
                ..Default::default()
            },
            _phantom: PhantomData,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(WorkerDescription {
            uuid: self.uuid.as_bytes().to_vec(),
        });
        let response = self.client.request_task(request).await?;
        let mut task = response.into_inner();
        while !task.files.is_empty() {
            self.current_task.id = task.id;
            self.current_task.task_type = task.task_type;

            // spawning notification task
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let current_task = self.current_task.clone();
            let client = self.client.clone();
            let notify_handle =
                tokio::spawn(
                    async move { Self::notify_coordinator(current_task, client, rx).await },
                );

            println!("Received task: {:?}", task);
            let result = match task.task_type {
                0 => Self::run_map(task),
                _ => Self::run_reduce(task),
            };

            // stoping notification task
            tx.send(()).await?;
            notify_handle.await?;

            println!("Task done");

            let request = tonic::Request::new(result);
            let response = self.client.task_done(request).await?;
            task = response.into_inner();
        }
        Ok(())
    }

    async fn notify_coordinator(
        current_task: CurrentTask,
        mut client: CoordinatorServiceClient<tonic::transport::Channel>,
        mut rx: tokio::sync::mpsc::Receiver<()>,
    ) {
        loop {
            let rx = rx.recv();
            pin_mut!(rx);
            let sleep = sleep(Duration::from_secs(1));
            pin_mut!(sleep);
            match select(rx, sleep).await {
                Either::Left((_, _)) => break,
                Either::Right((_, _)) => {
                    let notify_msg = Request::new(current_task.clone());
                    let _ = client.notify_working(notify_msg).await.unwrap();
                }
            }
        }
    }

    fn run_map(mut task: TaskDescription) -> TaskDescription {
        assert_eq!(task.files.len(), 1);

        let content = std::fs::read_to_string(&task.files[0]).unwrap();

        let map_result = T::map(&task.files[0], &content);
        let shuffle = Self::shuffle(&map_result, task.n as u64);

        let mut files = Vec::with_capacity(task.n as usize);
        files.resize(task.n as usize, "".to_string());
        for (i, keys) in shuffle.iter().enumerate() {
            let file_name = format!("./tmp/tmp-{}-{}", task.id, i);
            let mut file = std::fs::File::create(&file_name).unwrap();
            for key in keys.iter() {
                for val in map_result[*key].iter() {
                    file.write_all(format!("{} {}\n", key, val).as_bytes())
                        .unwrap();
                }
            }
            files[i] = file_name;
        }
        task.files = files;
        task
    }

    fn run_reduce(task: TaskDescription) -> TaskDescription {
        let content = std::fs::read_to_string(&task.files[0]).unwrap();
        let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
        for line in content.split('\n').into_iter() {
            let pair = line.split_ascii_whitespace().collect::<Vec<_>>();

            // TODO find out why some lines are incorrect
            if pair.len() != 2 {
                println!("line: {}", line);
                continue;
            }
            match map.get_mut(pair[0]) {
                Some(vals) => vals.push(pair[1]),
                None => {
                    let _ = map.insert(pair[0], vec![pair[1]]);
                }
            }
        }
        let reduce_results = map
            .into_iter()
            .map(|(key, values)| (key, T::reduce(key, values)))
            .collect::<Vec<_>>();
        for (i, (key, val)) in reduce_results.iter().enumerate() {
            let file_name = format!("./tmp/reduce-{}-{}", task.id, i);
            let mut file = std::fs::File::create(&file_name).unwrap();
            file.write_all(format!("{} {}\n", key, val).as_bytes())
                .unwrap();
        }

        task
    }

    fn shuffle(data: &HashMap<String, Vec<String>>, n: u64) -> Vec<Vec<&str>> {
        let mut suffled = Vec::<Vec<&str>>::with_capacity(n as usize);
        suffled.resize(n as usize, vec![]);
        for (key, _) in data.iter() {
            suffled[(Self::hash(key) % n) as usize].push(key);
        }
        suffled
    }

    fn hash(str: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        str.hash(&mut hasher);
        hasher.finish()
    }
}
