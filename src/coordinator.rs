pub mod rmr_grpc {
    tonic::include_proto!("rmr_grpc");
}
use rmr_grpc::coordinator_service_server::{CoordinatorService, CoordinatorServiceServer};
use rmr_grpc::{Acknowledge, CurrentTask, TaskDescription, TaskType, WorkerDescription};

use tonic::{transport::Server, Request, Response, Status};

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug)]
pub struct Coordinator {
    data_path: String,
    reduce_tasks: u32,
    addr: SocketAddr,
}

impl Coordinator {
    pub fn new(data_path: String, reduce_tasks: u32, addr: SocketAddr) -> Self {
        Coordinator {
            data_path,
            reduce_tasks,
            addr,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = MRCoordinator::new(self.data_path.clone(), self.reduce_tasks)?;
        Server::builder()
            .add_service(CoordinatorServiceServer::new(coordinator))
            .serve(self.addr)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct WorkerJob {
    file: String,
    worker: Option<Uuid>,
    results: Vec<String>,
}

#[derive(Debug, Default, Clone)]
struct ReduceTask {
    worker: Option<Uuid>,
}

#[derive(Debug, Default)]
pub struct CoordinatorData {
    map_tasks: Vec<WorkerJob>,
    reduce_tasks: Vec<ReduceTask>,
}

#[derive(Debug, Default)]
struct MRCoordinator {
    data: Arc<Mutex<CoordinatorData>>,
    reduce_tasks: u32,
}

impl MRCoordinator {
    pub(crate) fn new(
        data_path: String,
        reduce_tasks: u32,
    ) -> Result<MRCoordinator, Box<dyn std::error::Error>> {
        let paths = std::fs::read_dir(&data_path)?;
        let map_tasks = paths
            .map(|path| WorkerJob {
                file: path.unwrap().path().to_str().unwrap().to_string(),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        println!("Found files: {:#?}", map_tasks);

        let data = CoordinatorData {
            map_tasks,
            reduce_tasks: vec![Default::default(); reduce_tasks as usize],
        };

        Ok(MRCoordinator {
            data: Arc::new(Mutex::new(data)),
            reduce_tasks,
        })
    }

    fn get_worker_uuid(&self, id: usize) -> Option<Uuid> {
        self.data.lock().unwrap().map_tasks.get(id)?.worker
    }

    fn assign_map_task(&self, worker: &Uuid) -> Option<TaskDescription> {
        let mut data = self.data.lock().unwrap();
        let task_pos = data.map_tasks.iter().position(|mt| mt.worker.is_none())?;
        data.map_tasks[task_pos].worker = Some(*worker);
        Some(TaskDescription {
            id: task_pos as u32,
            task_type: TaskType::Map as i32,
            n: self.reduce_tasks,
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
        data.reduce_tasks[task_pos].worker = Some(*worker);

        let files = data
            .map_tasks
            .iter()
            .map(|task| task.results[task_pos].clone())
            .collect();

        Some(TaskDescription {
            id: task_pos as u32,
            task_type: TaskType::Reduce as i32,
            n: self.reduce_tasks,
            files,
        })
    }
}

#[tonic::async_trait]
impl CoordinatorService for MRCoordinator {
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
            .unwrap_or_default();
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
            .unwrap_or_default();
        println!("Worker {} finished task {:#?}", worker_uuid, finished_task);

        let reply = match finished_task.task_type {
            0 => {
                self.record_map_task(finished_task);
                self.assign_map_task(&worker_uuid)
                    .or_else(|| self.assign_reduce_task(&worker_uuid))
                    .unwrap_or_default()
            }
            _ => self.assign_reduce_task(&worker_uuid).unwrap_or_default(),
        };
        println!("Responding with: {:#?}", reply);

        Ok(Response::new(reply))
    }

    async fn task_failed(
        &self,
        request: Request<CurrentTask>,
    ) -> Result<Response<Acknowledge>, Status> {
        println!("Got failed task: {:#?}", request.into_inner());
        Ok(Response::new(Acknowledge::default()))
    }

    async fn notify_working(
        &self,
        request: Request<CurrentTask>,
    ) -> Result<Response<Acknowledge>, Status> {
        println!("Got notification: {:#?}", request.into_inner());
        Ok(Response::new(Acknowledge::default()))
    }
}
