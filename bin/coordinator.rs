use rmr::coordinator::Coordinator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let dir = "./data".to_string();
    let dead_task_delta = 5;

    Coordinator::new(dir, 10, dead_task_delta, addr).run().await
}
