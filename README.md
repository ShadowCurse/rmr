# RMR - rust implementation of MapReduce

Simple implementation of MapReduce algorithm. 
(Lab 1 from [MIT 6.824: Distributed Systems](https://pdos.csail.mit.edu/6.824/))

## Usage
To make this work you need to define a worker struct and implement `WorkerTrait` on it like this:
```rust
use rmr::{map_reduce, worker::WorkerTrait};
use std::collections::HashMap;

struct MyWorker;
impl WorkerTrait for MyWorker {
    fn map(_key: String, value: String) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let words = value
            .split_ascii_whitespace()
            .filter_map(|word| {
                let filtered = word
                    .chars()
                    .filter(|c| c.is_alphabetic())
                    .map(|c| c.to_lowercase().collect::<String>())
                    .collect::<String>();
                if !filtered.is_empty() {
                    Some(filtered)
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

    fn reduce(_key: String, values: Vec<String>) -> String {
        values.len().to_string()
    }
}
```

and then just use macro `map_reduce!` on your struct
```rust
map_reduce!(MyWorker);
```
This will create a simple cli app that can launch both coordinator and worker processes.

## Example
To run example word count MR you need to build it first:

```
cargo build --release
```

Then to launch coordinator server run:

```
cargo run --release -- server --addr 0.0.0.0:9999 --dead-task-delta 1 --path PATH_TO_FILES --reduce-tasks 10
```

To launch worker run:

```
cargo run --release -- worker --coordinator-addr 0.0.0.0:999
```
