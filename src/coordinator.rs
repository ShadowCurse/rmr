pub mod rmr_grpc {
    tonic::include_proto!("rmr_grpc");
}
use rmr_grpc::coordinator_service_server::{CoordinatorService, CoordinatorServiceServer};
use rmr_grpc::{Acknowledge, CurrentTask, TaskDescription, TaskType, WorkerDescription};

use tonic::{transport::Server, Request, Response, Status};

use std::fs::read_dir;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug)]
pub struct MRCoordinator {
    data_path: String,
    reduce_tasks: u32,
    dead_task_delta: u32,
    addr: SocketAddr,
}

impl MRCoordinator {
    pub fn new(
        data_path: String,
        reduce_tasks: u32,
        dead_task_delta: u32,
        addr: SocketAddr,
    ) -> Self {
        MRCoordinator {
            data_path,
            reduce_tasks,
            dead_task_delta,
            addr,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Stating coordinator with parameters:\n{:#?}", self);
        let coordinator = MRCoordinatorImpl::new(
            self.data_path.clone(),
            self.reduce_tasks,
            self.dead_task_delta,
        )?;
        Server::builder()
            .add_service(CoordinatorServiceServer::new(coordinator))
            // TODO use serve_with_shutdown to stop coordinator when all tasks are done
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

trait Task {
    fn worker(&self) -> &Option<Uuid>;
    fn state(&self) -> &TaskState;
    fn set_worker(&mut self, worker: Uuid);
    fn set_state(&mut self, state: TaskState);
}

#[derive(Debug, Default, Clone)]
struct MapTask {
    file: String,
    worker: Option<Uuid>,
    state: TaskState,
    results: Vec<String>,
}

impl Task for MapTask {
    fn worker(&self) -> &Option<Uuid> {
        &self.worker
    }
    fn state(&self) -> &TaskState {
        &self.state
    }
    fn set_worker(&mut self, worker: Uuid) {
        self.worker = Some(worker);
    }
    fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }
}

#[derive(Debug, Default, Clone)]
struct ReduceTask {
    worker: Option<Uuid>,
    state: TaskState,
}

impl Task for ReduceTask {
    fn worker(&self) -> &Option<Uuid> {
        &self.worker
    }
    fn state(&self) -> &TaskState {
        &self.state
    }
    fn set_worker(&mut self, worker: Uuid) {
        self.worker = Some(worker);
    }
    fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }
}

#[derive(Debug, Default)]
pub struct CoordinatorData {
    map_tasks: Vec<MapTask>,
    reduce_tasks: Vec<ReduceTask>,
}

#[derive(Debug, Default)]
struct MRCoordinatorImpl {
    data: Mutex<CoordinatorData>,
    reduce_tasks: u32,
    dead_task_delta: u32,
}

impl MRCoordinatorImpl {
    pub fn new(
        data_path: String,
        reduce_tasks: u32,
        dead_task_delta: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let paths = read_dir(&data_path)?;

        let paths = paths
            .map(|path| path.map(|p| p.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        let map_tasks = paths
            .iter()
            .filter_map(|path| {
                if path.is_file() {
                    let file = path.to_str().unwrap().to_string();
                    println!("Found file: {:#?}", file);
                    Some(MapTask {
                        file,
                        ..Default::default()
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let data = CoordinatorData {
            map_tasks,
            reduce_tasks: vec![Default::default(); reduce_tasks as usize],
        };

        Ok(Self {
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

    fn assign_map_task(&self, worker: &Uuid) -> Option<TaskDescription> {
        let mut data = self.data.lock().unwrap();
        let task_pos = self.update_task_data(&mut data.map_tasks, worker)?;
        Some(TaskDescription {
            id: task_pos as u32,
            task_type: TaskType::Map as i32,
            n: self.reduce_tasks,
            files: vec![data.map_tasks[task_pos].file.clone()],
        })
    }

    fn assign_reduce_task(&self, worker: &Uuid) -> Option<TaskDescription> {
        let mut data = self.data.lock().unwrap();
        let task_pos = self.update_task_data(&mut data.reduce_tasks, worker)?;
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

    fn update_task_data(&self, tasks: &mut Vec<impl Task>, worker: &Uuid) -> Option<usize> {
        let task_pos = tasks.iter().position(|t| match t.state() {
            TaskState::NotStarted => true,
            TaskState::InProgress(last_timestamp) => {
                last_timestamp.elapsed().as_secs() > self.dead_task_delta as u64
            }
            TaskState::Finished => false,
        })?;
        tasks[task_pos].set_worker(*worker);
        tasks[task_pos].set_state(TaskState::InProgress(Instant::now()));
        Some(task_pos)
    }

    fn record_map_task(&self, task: TaskDescription) {
        let mut data = self.data.lock().unwrap();
        data.map_tasks[task.id as usize].results = task.files;
        data.map_tasks[task.id as usize].state = TaskState::Finished;
    }
}

#[tonic::async_trait]
impl CoordinatorService for MRCoordinatorImpl {
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
