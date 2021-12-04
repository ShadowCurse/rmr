pub mod rmr_grpc {
    tonic::include_proto!("rmr_grpc");
}
use rmr_grpc::coordinator_service_server::{CoordinatorService, CoordinatorServiceServer};
use rmr_grpc::{Acknowledge, CurrentTask, TaskDescription, TaskType, WorkerDescription};

use tonic::{transport::Server, Request, Response, Status};

use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug)]
pub struct Coordinator {
    data_path: String,
    reduce_tasks: u32,
    dead_task_delta: u32,
    addr: SocketAddr,
}

impl Coordinator {
    pub fn new(
        data_path: String,
        reduce_tasks: u32,
        dead_task_delta: u32,
        addr: SocketAddr,
    ) -> Self {
        Coordinator {
            data_path,
            reduce_tasks,
            dead_task_delta,
            addr,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = MRCoordinator::new(
            self.data_path.clone(),
            self.reduce_tasks,
            self.dead_task_delta,
        )?;
        Server::builder()
            .add_service(CoordinatorServiceServer::new(coordinator))
            .serve(self.addr)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum TaskState {
    NotStarted,
    InProgress(Instant),
    Finished,
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::NotStarted
    }
}

#[derive(Debug, Default, Clone)]
struct MapTask {
    file: String,
    worker: Option<Uuid>,
    state: TaskState,
    results: Vec<String>,
}

#[derive(Debug, Default, Clone)]
struct ReduceTask {
    worker: Option<Uuid>,
    state: TaskState,
}

#[derive(Debug, Default)]
pub struct CoordinatorData {
    map_tasks: Vec<MapTask>,
    reduce_tasks: Vec<ReduceTask>,
}

#[derive(Debug, Default)]
struct MRCoordinator {
    data: Mutex<CoordinatorData>,
    reduce_tasks: u32,
    dead_task_delta: u32,
}

impl MRCoordinator {
    pub(crate) fn new(
        data_path: String,
        reduce_tasks: u32,
        dead_task_delta: u32,
    ) -> Result<MRCoordinator, Box<dyn std::error::Error>> {
        let paths = std::fs::read_dir(&data_path)?;
        let map_tasks = paths
            .map(|path| MapTask {
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
            data: Mutex::new(data),
            reduce_tasks,
            dead_task_delta,
        })
    }

    fn map_worker_uuid_by_id(&self, id: usize) -> Option<Uuid> {
        self.data.lock().unwrap().map_tasks.get(id)?.worker
    }

    fn reduce_worker_uuid_by_id(&self, id: usize) -> Option<Uuid> {
        self.data.lock().unwrap().reduce_tasks.get(id)?.worker
    }

    fn update_task_status(&self, task_description: CurrentTask, state: TaskState) {
        let worker_uuid = Uuid::from_bytes(&task_description.uuid).unwrap();
        let mut data = self.data.lock().unwrap();
        match task_description.task_type {
            0 => {
                if let Some(task) = data.map_tasks.get_mut(task_description.id as usize) {
                    if task.worker.unwrap() == worker_uuid {
                        task.state = state
                    }
                };
            }
            _ => {
                if let Some(task) = data.reduce_tasks.get_mut(task_description.id as usize) {
                    if task.worker.unwrap() == worker_uuid {
                        task.state = state
                    }
                };
            }
        }
    }

    fn assign_map_task(&self, worker: &Uuid) -> Option<TaskDescription> {
        let mut data = self.data.lock().unwrap();
        let task_pos = data.map_tasks.iter().position(|mt| {
            // if not started or last timestamp was too long ago
            match mt.state {
                TaskState::NotStarted => true,
                TaskState::InProgress(last_timestamp) => {
                    last_timestamp.elapsed().as_secs() > self.dead_task_delta as u64
                }
                TaskState::Finished => false,
            }
        })?;
        data.map_tasks[task_pos].worker = Some(*worker);
        data.map_tasks[task_pos].state = TaskState::InProgress(Instant::now());
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
        data.map_tasks[task.id as usize].state = TaskState::Finished;
    }

    fn assign_reduce_task(&self, worker: &Uuid) -> Option<TaskDescription> {
        let mut data = self.data.lock().unwrap();
        let task_pos = data.reduce_tasks.iter().position(|rt| {
            // if not started or last timestamp was too long ago
            match rt.state {
                TaskState::NotStarted => true,
                TaskState::InProgress(last_timestamp) => {
                    last_timestamp.elapsed().as_secs() > self.dead_task_delta as u64
                }
                TaskState::Finished => false,
            }
        })?;
        data.reduce_tasks[task_pos].worker = Some(*worker);
        data.reduce_tasks[task_pos].state = TaskState::InProgress(Instant::now());

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

        // check for unfinished map or reduce tasks (or those which take too long)
        let reply = self
            .assign_map_task(&worker_uuid)
            .or_else(|| self.assign_reduce_task(&worker_uuid))
            .unwrap_or_default();

        Ok(Response::new(reply))
    }

    async fn task_done(
        &self,
        request: Request<TaskDescription>,
    ) -> Result<Response<TaskDescription>, Status> {
        let finished_task = request.into_inner();

        let reply = match finished_task.task_type {
            // Map
            0 => {
                if let Some(worker_uuid) = self.map_worker_uuid_by_id(finished_task.id as usize) {
                    println!("Map task finished. Worker: {:#?}", worker_uuid);
                    self.record_map_task(finished_task);
                    self.assign_map_task(&worker_uuid)
                        .or_else(|| self.assign_reduce_task(&worker_uuid))
                        .unwrap_or_default()
                } else {
                    Default::default()
                }
            }
            // Reduce
            _ => {
                if let Some(worker_uuid) = self.reduce_worker_uuid_by_id(finished_task.id as usize)
                {
                    println!("Reduce task finished. Worker: {:#?}", worker_uuid);
                    self.assign_reduce_task(&worker_uuid).unwrap_or_default()
                } else {
                    Default::default()
                }
            }
        };

        Ok(Response::new(reply))
    }

    async fn task_failed(
        &self,
        request: Request<CurrentTask>,
    ) -> Result<Response<Acknowledge>, Status> {
        let current_task = request.into_inner();
        println!("Got failed task: {:#?}", current_task);
        self.update_task_status(current_task, TaskState::NotStarted);
        Ok(Response::new(Acknowledge::default()))
    }

    async fn notify_working(
        &self,
        request: Request<CurrentTask>,
    ) -> Result<Response<Acknowledge>, Status> {
        let current_task = request.into_inner();
        println!("Got notification: {:#?}", current_task);
        self.update_task_status(current_task, TaskState::InProgress(Instant::now()));
        Ok(Response::new(Acknowledge::default()))
    }
}
