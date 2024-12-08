// metrics data structure
// 基本功能： inc/dec/snapshot

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Metrics {
    data: HashMap<String, i64>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            data: HashMap::new(),
        }
    }

    pub fn inc(&mut self, key: impl Into<String>) {
        let count = self.data.entry(key.into()).or_insert(0);
        *count += 1;
    }

    pub fn dec(&mut self, key: impl Into<String>) {
        let count = self.data.entry(key.into()).or_insert(0);
        *count -= 1;
    }

    pub fn snapshot(&self) -> HashMap<String, i64> {
        self.data.clone()
    }
}
