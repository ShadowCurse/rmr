pub mod rmr_grpc {
    tonic::include_proto!("rmr_grpc");
}
use rmr_grpc::coordinator_server::{Coordinator, CoordinatorServer};
use rmr_grpc::{TaskType, TaskDescription, WorkerDescription};

use tonic::{transport::Server, Request, Response, Status};

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
struct WorkerJob {
    file: String,
    worker: Option<Uuid>,
    results: Vec<String>,
}

#[derive(Debug, Default)]
pub struct CoordinatorData {
    map_tasks: Vec<WorkerJob>,
    reduce_tasks: Vec<WorkerJob>,
}

#[derive(Debug, Default)]
pub struct MRCoordinator {
    data: Arc<Mutex<CoordinatorData>>,
    n: u32,
}

impl MRCoordinator {
    pub fn new(data_path: String, n: u32) -> MRCoordinator {
        let paths = std::fs::read_dir(&data_path).unwrap();
        let map_tasks = paths
            .map(|path| WorkerJob {
                file: path.unwrap().path().to_str().unwrap().to_string(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        for wt in map_tasks.iter() {
            println!("Found file: {}", wt.file);
        }
        let data = CoordinatorData {
            map_tasks,
            reduce_tasks: vec![Default::default(); n as usize],
        };
        MRCoordinator {
            data: Arc::new(Mutex::new(data)),
            n,
        }
    }

    pub async fn run(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        Server::builder()
            .add_service(CoordinatorServer::new(self))
            .serve(addr)
            .await?;
        Ok(())
    }

    fn get_worker_uuid(&self, id: usize) -> Option<Uuid> {
        Some(
            self.data
                .lock()
                .unwrap()
                .map_tasks
                .get(id)?
                .worker
                .as_ref()?
                .clone(),
        )
    }

    fn assign_map_task(&self, worker: &Uuid) -> Option<TaskDescription> {
        let mut data = self.data.lock().unwrap();
        let task_pos = data.map_tasks.iter().position(|mt| mt.worker.is_none())?;
        data.map_tasks[task_pos].worker = Some(worker.clone());
        Some(TaskDescription {
            id: task_pos as u32,
            task_type: TaskType::Map as i32,
            n: self.n,
            files: vec![data.map_tasks[task_pos].file.clone()],
        })
    }

    fn record_map_task(&self, task: TaskDescription) {
        let mut data = self.data.lock().unwrap();
        data.map_tasks[task.id as usize].results = task.files;
    }

    fn assign_reduce_task(&self, worker: &Uuid) -> Option<TaskDescription> {
        let mut data = self.data.lock().unwrap();
        let task_pos = data
            .reduce_tasks
            .iter()
            .position(|mt| mt.worker.is_none())?;
        data.reduce_tasks[task_pos].worker = Some(worker.clone());

        let files = data
            .map_tasks
            .iter()
            .map(|task| task.results[task_pos].clone())
            .collect();

        Some(TaskDescription {
            id: task_pos as u32,
            task_type: TaskType::Reduce as i32,
            n: self.n,
            files,
        })
    }
}

#[tonic::async_trait]
impl Coordinator for MRCoordinator {
    async fn request_task(
        &self,
        request: Request<WorkerDescription>,
    ) -> Result<Response<TaskDescription>, Status> {
        let worker_uuid = Uuid::from_bytes(&request.into_inner().uuid).unwrap();
        println!("Got a request for a task from worker: {:?}", worker_uuid);

        // check for unfinished map tasks (or those which take too long)
        let reply = self
            .assign_map_task(&worker_uuid)
            .or_else(|| self.assign_reduce_task(&worker_uuid))
            .unwrap_or(Default::default());
        // check for reduce tasks

        Ok(Response::new(reply))
    }

    async fn task_done(
        &self,
        request: Request<TaskDescription>,
    ) -> Result<Response<TaskDescription>, Status> {
        let finished_task = request.into_inner();
        let worker_uuid = self
            .get_worker_uuid(finished_task.id as usize)
            .unwrap_or(Default::default());
        println!("Worker {} finished task {:#?}", worker_uuid, finished_task);

        let reply = match finished_task.task_type {
            0 => {
                self.record_map_task(finished_task);
                self.assign_map_task(&worker_uuid)
                    .or_else(|| self.assign_reduce_task(&worker_uuid))
                    .unwrap_or(Default::default())
            }
            _ => self
                .assign_reduce_task(&worker_uuid)
                .unwrap_or(Default::default()),
        };
        println!("Responding with: {:#?}", reply);

        Ok(Response::new(reply))
    }
}
