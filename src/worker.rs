use coordinator::coordinator_client::CoordinatorClient;
use coordinator::WorkerDescription;

pub mod coordinator {
    tonic::include_proto!("coordinator");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CoordinatorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(WorkerDescription {
        name: "worker1".into(),
    });

    let response = client.request_job(request).await?;

    println!("RESPONSE={:?}", response);

    let mut r = response.into_inner();

    while !r.file_path.is_empty() {
        let request = tonic::Request::new(r);
        let response = client.job_done(request).await?;
        println!("RESPONSE={:?}", response);
        r = response.into_inner();
    }

    Ok(())
}
