use serde::{Deserialize, Serialize};
use std::fmt;

/// A database-agnostic value type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Int(n) => Some(*n != 0),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(d) => write!(f, "{}", d),
            Value::Text(s) => write!(f, "{}", s),
            Value::Bytes(b) => write!(f, "<bytes:{}>", b.len()),
            Value::Json(v) => write!(f, "{}", v),
        }
    }
}

// From impls
impl From<i64> for Value { fn from(v: i64) -> Self { Value::Int(v) } }
impl From<i32> for Value { fn from(v: i32) -> Self { Value::Int(v as i64) } }
impl From<u64> for Value { fn from(v: u64) -> Self { Value::Int(v as i64) } }
impl From<u32> for Value { fn from(v: u32) -> Self { Value::Int(v as i64) } }
impl From<f64> for Value { fn from(v: f64) -> Self { Value::Float(v) } }
impl From<f32> for Value { fn from(v: f32) -> Self { Value::Float(v as f64) } }
impl From<bool> for Value { fn from(v: bool) -> Self { Value::Bool(v) } }
impl From<String> for Value { fn from(v: String) -> Self { Value::Text(v) } }
impl From<&str> for Value { fn from(v: &str) -> Self { Value::Text(v.to_string()) } }
impl From<Vec<u8>> for Value { fn from(v: Vec<u8>) -> Self { Value::Bytes(v) } }
impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self { Value::Json(v) }
}
impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(t) => t.into(),
            None => Value::Null,
        }
    }
}

/// Convert a `Value` into a `serde_json::Value` (used for row→struct deserialization).
impl From<Value> for serde_json::Value {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(b),
            Value::Int(n) => serde_json::Value::Number(n.into()),
            Value::Float(f) => serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::Text(s) => serde_json::Value::String(s),
            Value::Bytes(b) => serde_json::Value::String(base64_encode(&b)),
            Value::Json(v) => v,
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(TABLE[(b0 >> 2)] as char);
        out.push(TABLE[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(if chunk.len() > 1 { TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[b2 & 0x3f] as char } else { '=' });
    }
    out
}

/// A database row as an ordered map of column→Value.
pub type Row = indexmap::IndexMap<String, Value>;

/// Convert a `serde_json::Map` to a `Row`.
pub fn json_map_to_row(map: serde_json::Map<String, serde_json::Value>) -> Row {
    map.into_iter()
        .map(|(k, v)| (k, json_value_to_value(v)))
        .collect()
}

pub fn json_value_to_value(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { Value::Int(i) }
            else if let Some(f) = n.as_f64() { Value::Float(f) }
            else { Value::Text(n.to_string()) }
        }
        serde_json::Value::String(s) => Value::Text(s),
        other => Value::Json(other),
    }
}
