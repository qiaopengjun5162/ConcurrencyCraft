// metrics data structure
// 基本功能： inc/dec/snapshot

use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::{Arc, RwLock},
};
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    data: Arc<RwLock<HashMap<String, i64>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow!(e.to_string()))?;
        let count = data.entry(key.into()).or_insert(0);
        *count += 1;
        Ok(())
    }

    pub fn dec(&self, key: impl Into<String>) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow!(e.to_string()))?;
        let count = data.entry(key.into()).or_insert(0);
        *count -= 1;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        let data = self.data.read().map_err(|e| anyhow!(e.to_string()))?;
        Ok(data.clone())
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read().map_err(|_e| fmt::Error {})?;
        for (k, v) in data.iter() {
            writeln!(f, "{}: {}", k, v)?;
        }
        Ok(())
    }
}
