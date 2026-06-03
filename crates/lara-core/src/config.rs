use serde_json::Value;
use std::collections::HashMap;

/// Hierarchical configuration store.
/// Keys use dot-notation: `"database.connections.postgres.host"`.
#[derive(Debug, Default, Clone)]
pub struct Config {
    data: HashMap<String, Value>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value at the given dot-notation key.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.data.insert(key.into(), value.into());
    }

    /// Get a value by dot-notation key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Some(v) = self.data.get(key) {
            return Some(v);
        }
        // Walk nested segments
        let mut parts = key.splitn(2, '.');
        let head = parts.next()?;
        let tail = parts.next()?;
        match self.data.get(head)? {
            Value::Object(map) => map.get(tail),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key)?.as_u64()
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key)?.as_i64()
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_f64()
    }

    /// Merge another config on top (values from `other` win).
    pub fn merge(&mut self, other: Config) {
        for (k, v) in other.data {
            self.data.insert(k, v);
        }
    }

    /// Load from a JSON file.
    pub fn from_json_file(path: &str) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::CoreError::Config(e.to_string()))?;
        let map: HashMap<String, Value> = serde_json::from_str(&content)
            .map_err(|e| crate::error::CoreError::Config(e.to_string()))?;
        Ok(Self { data: map })
    }
}
