use std::collections::HashMap;

#[derive(Default)]
/// key-value storage model
pub struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    /// default factory
    pub fn new() -> Self {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// get value by key
    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).cloned()
    }

    /// insert value at key
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// remove value at key
    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
