use tonic::{transport::Server, Request, Response, Status};

use coordinator::coordinator_client::CoordinatorClient;
use coordinator::{task_description::TaskType, TaskDescription, WorkerDescription};

pub mod coordinator {
    tonic::include_proto!("coordinator");
}

use std::sync::{Arc, Mutex};
use uuid::Uuid;

trait MRCoordinator {
    // Register worker to track it's execution
    fn register_worker(&self, worker_id: Uuid);
    fn assign_map_task(&self, worker_id: Uuid);
    fn record_map_task(&self);
    fn assign_reduce_task(&self, worker_id: Uuid);
}
