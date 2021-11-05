use tonic::{transport::Server, Request, Response, Status};

use coordinator::coordinator_server::{Coordinator, CoordinatorServer};
use coordinator::{HelloReply, HelloRequest};

pub mod coordinator {
    tonic::include_proto!("coordinator");
}

#[derive(Debug, Default)]
pub struct MyCoordinator {}

#[tonic::async_trait]
impl Coordinator for MyCoordinator {
    async fn say_hello(&self, request: Request<HelloRequest>,) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = coordinator::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into()
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let addr = "[::1]:50051".parse()?;
    let coordinator = MyCoordinator::default();

    Server::builder()
        .add_service(CoordinatorServer::new(coordinator))
        .serve(addr)
        .await?;

    Ok(())
}
