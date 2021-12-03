use rmr::coordinator::Coordinator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let dir = "./data".to_string();

    Coordinator::new(dir, 10, addr).run().await
}
