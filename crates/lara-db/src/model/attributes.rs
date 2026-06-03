use serde_json::Value as JsonValue;

/// Cast types that can be declared on model fields.
#[derive(Debug, Clone, PartialEq)]
pub enum CastType {
    Integer,
    Float,
    Boolean,
    String,
    Json,
    Date,       // "YYYY-MM-DD"
    DateTime,   // ISO 8601
    Timestamp,  // Unix epoch i64
    Uuid,
    Hashed,     // bcrypt on write, verify-only on read
    Encrypted,  // AES-GCM encrypt/decrypt
    Array,
    Custom(String),
}

impl CastType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "int" | "integer" => Self::Integer,
            "float" | "double" => Self::Float,
            "bool" | "boolean" => Self::Boolean,
            "string" => Self::String,
            "json" | "array" | "object" => Self::Json,
            "date" => Self::Date,
            "datetime" | "timestamp" => Self::DateTime,
            "unix" => Self::Timestamp,
            "uuid" => Self::Uuid,
            "hashed" => Self::Hashed,
            "encrypted" => Self::Encrypted,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Apply a cast to an incoming JSON value (from the database).
pub fn cast_get(value: JsonValue, cast: &CastType) -> JsonValue {
    match cast {
        CastType::Integer => match &value {
            JsonValue::Number(n) => JsonValue::Number(n.clone()),
            JsonValue::String(s) => s.parse::<i64>()
                .map(|n| JsonValue::Number(n.into()))
                .unwrap_or(value),
            JsonValue::Bool(b) => JsonValue::Number((*b as i64).into()),
            _ => value,
        },
        CastType::Float => match &value {
            JsonValue::Number(n) => JsonValue::Number(n.clone()),
            JsonValue::String(s) => s.parse::<f64>()
                .ok()
                .and_then(|f| serde_json::Number::from_f64(f).map(JsonValue::Number))
                .unwrap_or(value),
            _ => value,
        },
        CastType::Boolean => match &value {
            JsonValue::Bool(b) => JsonValue::Bool(*b),
            JsonValue::Number(n) => JsonValue::Bool(n.as_i64().unwrap_or(0) != 0),
            JsonValue::String(s) => JsonValue::Bool(
                matches!(s.to_lowercase().as_str(), "true" | "1" | "yes" | "on"),
            ),
            _ => value,
        },
        CastType::String => match value {
            JsonValue::String(_) => value,
            other => JsonValue::String(other.to_string()),
        },
        CastType::Json => match &value {
            JsonValue::String(s) => serde_json::from_str(s).unwrap_or(value),
            _ => value,
        },
        _ => value,
    }
}

/// Apply a cast to an outgoing value (before writing to the database).
pub fn cast_set(value: JsonValue, cast: &CastType) -> JsonValue {
    match cast {
        CastType::Json => match value {
            JsonValue::Object(_) | JsonValue::Array(_) => {
                JsonValue::String(value.to_string())
            }
            other => other,
        },
        CastType::Boolean => match value {
            JsonValue::Bool(b) => JsonValue::Number((b as i64).into()),
            other => other,
        },
        _ => value,
    }
}
