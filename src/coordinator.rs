use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

use coordinator::coordinator_server::{Coordinator, CoordinatorServer};
use coordinator::{HelloReply, HelloRequest, JobDescription, WorkerDescription};

pub mod coordinator {
    tonic::include_proto!("coordinator");
}

#[derive(Debug)]
struct WorkerJob {
    file: String,
    worker: Option<String>,
}

#[derive(Debug, Default)]
pub struct CoordinatorData {
    maps_jobs: Vec<WorkerJob>,
    reduce_jobs: Vec<WorkerJob>,
}

#[derive(Debug, Default)]
pub struct MyCoordinator {
    data: Arc<Mutex<CoordinatorData>>,
}

impl MyCoordinator {
    pub fn new(data: String) -> MyCoordinator {
        let paths = std::fs::read_dir(&data).unwrap();
        let maps_jobs = paths
            .map(|path| WorkerJob {
                file: path.unwrap().path().to_str().unwrap().to_string(),
                worker: None,
            })
            .collect::<Vec<_>>();
        for wj in maps_jobs.iter() {
            println!("Found file: {}", wj.file);
        }
        let data = CoordinatorData {
            maps_jobs,
            reduce_jobs: Default::default(),
        };
        MyCoordinator {
            data: Arc::new(Mutex::new(data)),
        }
    }

    fn get_worker_name(&self, id: usize) -> Option<String> {
        Some(
            self.data
                .lock()
                .unwrap()
                .maps_jobs
                .get(id)?
                .worker
                .as_ref()?
                .clone(),
        )
    }

    fn get_and_assign_map_task(&self, worker: &str) -> Option<JobDescription> {
        let mut data = self.data.lock().unwrap();
        let job_pos = data.maps_jobs.iter().position(|mj| mj.worker.is_none())?;
        data.maps_jobs[job_pos].worker = Some(worker.to_string());
        Some(JobDescription {
            id: job_pos as i32,
            file_path: data.maps_jobs[job_pos].file.clone(),
        })
    }
}

#[tonic::async_trait]
impl Coordinator for MyCoordinator {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = coordinator::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }

    async fn request_job(
        &self,
        request: Request<WorkerDescription>,
    ) -> Result<Response<JobDescription>, Status> {
        let worker_name = request.into_inner().name;
        println!("Got a request for a job from: {:?}", worker_name);

        let reply = self
            .get_and_assign_map_task(&worker_name)
            .unwrap_or(Default::default());

        Ok(Response::new(reply))
    }

    async fn job_done(
        &self,
        request: Request<JobDescription>,
    ) -> Result<Response<JobDescription>, Status> {
        let worker_name = self
            .get_worker_name(request.into_inner().id as usize)
            .unwrap_or(Default::default());
        println!("Worker {} finished task", worker_name);

        let reply = self
            .get_and_assign_map_task(&worker_name)
            .unwrap_or(Default::default());

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let dir = "data".to_string();
    let coordinator = MyCoordinator::new(dir);

    Server::builder()
        .add_service(CoordinatorServer::new(coordinator))
        .serve(addr)
        .await?;

    Ok(())
}
