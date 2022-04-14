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

map_reduce!(MyWorker);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_test() {
        let data = vec![
            ("algernon", vec!["1"]),
            ("and", vec!["1"]),
            ("who", vec!["1"]),
            ("are", vec!["1"]),
            ("the", vec!["1"]),
            ("people", vec!["1"]),
            ("you", vec!["1"]),
            ("amuse", vec!["1"]),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        let res = MyWorker::map(
            "".to_string(),
            "Algernon.  And who are the people you amuse?".to_string(),
        );
        for (k, v) in data.iter() {
            assert!(res.contains_key(*k));
            assert!(res[*k].len() == v.len());
            assert!(res[*k][0] == v[0]);
        }

        for (k, v) in res.into_iter() {
            let reduce_res = MyWorker::reduce(k, v);
            assert!(reduce_res == 1.to_string());
        }
    }
}
