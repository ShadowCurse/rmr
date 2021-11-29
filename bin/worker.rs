use rmr::worker;
use std::collections::HashMap;

struct MyWorker;

impl worker::WorkerTrait for MyWorker {
    fn map(_key: &str, value: &str) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let words = value
            .split_ascii_whitespace()
            .map(|word| word.chars().filter(|c| c.is_alphabetic()).collect())
            .collect::<Vec<String>>();
        for word in words {
            if !map.contains_key(&*word) {
                map.insert(word.to_string(), vec!["1".to_string()]);
            } else {
                map.get_mut(&*word).unwrap().push("1".to_string());
            }
        }
        map
    }

    fn reduce(key: &str, values: Vec<&str>) -> String {
        "".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut worker = worker::MRWorker::<MyWorker>::new("http://[::1]:50051").await?;
    worker.run().await?;
    Ok(())
}
