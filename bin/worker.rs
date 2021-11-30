use rmr::worker;
use std::collections::HashMap;

struct MyWorker;

impl worker::WorkerTrait for MyWorker {
    fn map(_key: &str, value: &str) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let words = value
            .split_ascii_whitespace()
            .filter_map(|word| {
                if !word.is_empty() {
                    Some(
                        word.chars()
                            .filter(|c| c.is_alphabetic())
                            .map(|c| c.to_lowercase().collect::<String>())
                            .collect(),
                    )
                } else {
                    None
                }
            })
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

    fn reduce(_key: &str, values: Vec<&str>) -> String {
        values.len().to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut worker = worker::MRWorker::<MyWorker>::new("http://[::1]:50051").await?;
    worker.run().await?;
    Ok(())
}
