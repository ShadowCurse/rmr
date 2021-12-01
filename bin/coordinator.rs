use rmr::coordinator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let dir = "./data".to_string();
    let coordinator = coordinator::MRCoordinator::new(dir, 10);

    coordinator.run(addr).await
}
