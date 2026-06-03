use regex::Regex;
use serde_json::Value;

/// Returns `Some(error_message)` if invalid, `None` if valid.
pub type RuleFn = Box<dyn Fn(&str, &Value, &serde_json::Map<String, Value>) -> Option<String> + Send + Sync>;

/// Built-in rule registry.
pub fn required(field: &str, val: &Value, _data: &serde_json::Map<String, Value>) -> Option<String> {
    if val.is_null() || (val.is_string() && val.as_str().unwrap().is_empty()) {
        Some(format!("The {} field is required.", field))
    } else {
        None
    }
}

pub fn min_length(min: usize) -> RuleFn {
    Box::new(move |field, val, _| {
        let len = val.as_str().map(|s| s.len()).unwrap_or(0);
        if len < min {
            Some(format!("The {} must be at least {} characters.", field, min))
        } else {
            None
        }
    })
}

pub fn max_length(max: usize) -> RuleFn {
    Box::new(move |field, val, _| {
        let len = val.as_str().map(|s| s.len()).unwrap_or(usize::MAX);
        if len > max {
            Some(format!("The {} must not exceed {} characters.", field, max))
        } else {
            None
        }
    })
}

pub fn email(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    let s = val.as_str().unwrap_or("");
    let re = Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(s) {
        Some(format!("The {} must be a valid email address.", field))
    } else {
        None
    }
}

pub fn numeric(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    match val {
        Value::Number(_) => None,
        Value::String(s) if s.parse::<f64>().is_ok() => None,
        _ => Some(format!("The {} must be a number.", field)),
    }
}

pub fn integer(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    match val {
        Value::Number(n) if n.is_i64() || n.is_u64() => None,
        Value::String(s) if s.parse::<i64>().is_ok() => None,
        _ => Some(format!("The {} must be an integer.", field)),
    }
}

pub fn boolean_rule(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    match val {
        Value::Bool(_) => None,
        Value::Number(n) => {
            if n.as_i64() == Some(0) || n.as_i64() == Some(1) { None }
            else { Some(format!("The {} must be a boolean.", field)) }
        }
        Value::String(s) => {
            if matches!(s.as_str(), "true" | "false" | "1" | "0" | "yes" | "no") { None }
            else { Some(format!("The {} must be a boolean.", field)) }
        }
        _ => Some(format!("The {} must be a boolean.", field)),
    }
}

pub fn min_value(min: f64) -> RuleFn {
    Box::new(move |field, val, _| {
        let n = val.as_f64().or_else(|| val.as_str().and_then(|s| s.parse().ok())).unwrap_or(f64::MIN);
        if n < min { Some(format!("The {} must be at least {}.", field, min)) }
        else { None }
    })
}

pub fn max_value(max: f64) -> RuleFn {
    Box::new(move |field, val, _| {
        let n = val.as_f64().or_else(|| val.as_str().and_then(|s| s.parse().ok())).unwrap_or(f64::MAX);
        if n > max { Some(format!("The {} must not be greater than {}.", field, max)) }
        else { None }
    })
}

pub fn between(min: f64, max: f64) -> RuleFn {
    Box::new(move |field, val, _| {
        let n = val.as_f64().or_else(|| val.as_str().and_then(|s| s.parse().ok())).unwrap_or(f64::MIN);
        if n < min || n > max {
            Some(format!("The {} must be between {} and {}.", field, min, max))
        } else {
            None
        }
    })
}

pub fn in_list(options: Vec<String>) -> RuleFn {
    Box::new(move |field, val, _| {
        let s = match val {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => return Some(format!("The {} field has an invalid value.", field)),
        };
        if options.contains(&s) { None }
        else { Some(format!("The selected {} is invalid.", field)) }
    })
}

pub fn not_in(options: Vec<String>) -> RuleFn {
    Box::new(move |field, val, _| {
        let s = match val {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => return None,
        };
        if options.contains(&s) { Some(format!("The selected {} is invalid.", field)) }
        else { None }
    })
}

pub fn regex_rule(pattern: &str) -> RuleFn {
    let re = Regex::new(pattern).expect("Invalid regex");
    Box::new(move |field, val, _| {
        let s = val.as_str().unwrap_or("");
        if re.is_match(s) { None }
        else { Some(format!("The {} format is invalid.", field)) }
    })
}

pub fn url_rule(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    let s = val.as_str().unwrap_or("");
    if s.starts_with("http://") || s.starts_with("https://") {
        None
    } else {
        Some(format!("The {} must be a valid URL.", field))
    }
}

pub fn uuid_rule(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    let s = val.as_str().unwrap_or("");
    let re = Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
    ).unwrap();
    if re.is_match(s) { None }
    else { Some(format!("The {} must be a valid UUID.", field)) }
}

pub fn ip_rule(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    let s = val.as_str().unwrap_or("");
    let is_ipv4 = s.parse::<std::net::Ipv4Addr>().is_ok();
    let is_ipv6 = s.parse::<std::net::Ipv6Addr>().is_ok();
    if is_ipv4 || is_ipv6 { None }
    else { Some(format!("The {} must be a valid IP address.", field)) }
}

pub fn date_rule(field: &str, val: &Value, _: &serde_json::Map<String, Value>) -> Option<String> {
    let s = val.as_str().unwrap_or("");
    let formats = ["%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M:%SZ"];
    for fmt in &formats {
        if chrono::NaiveDate::parse_from_str(s, fmt).is_ok()
            || chrono::DateTime::parse_from_rfc3339(s).is_ok()
        {
            return None;
        }
    }
    Some(format!("The {} is not a valid date.", field))
}

pub fn same(other_field: &str) -> RuleFn {
    let other = other_field.to_string();
    Box::new(move |field, val, data| {
        let other_val = data.get(&other).cloned().unwrap_or(Value::Null);
        if val == &other_val { None }
        else { Some(format!("The {} and {} must match.", field, other)) }
    })
}

pub fn different(other_field: &str) -> RuleFn {
    let other = other_field.to_string();
    Box::new(move |field, val, data| {
        let other_val = data.get(&other).cloned().unwrap_or(Value::Null);
        if val != &other_val { None }
        else { Some(format!("The {} and {} must be different.", field, other)) }
    })
}

pub fn confirmed(field: &str, val: &Value, data: &serde_json::Map<String, Value>) -> Option<String> {
    let confirm_key = format!("{}_confirmation", field);
    let other = data.get(&confirm_key).cloned().unwrap_or(Value::Null);
    if val == &other { None }
    else { Some(format!("The {} confirmation does not match.", field)) }
}

pub fn starts_with_rule(prefix: &str) -> RuleFn {
    let p = prefix.to_string();
    Box::new(move |field, val, _| {
        if val.as_str().map(|s| s.starts_with(&p)).unwrap_or(false) { None }
        else { Some(format!("The {} must start with '{}'.", field, p)) }
    })
}

pub fn ends_with_rule(suffix: &str) -> RuleFn {
    let s = suffix.to_string();
    Box::new(move |field, val, _| {
        if val.as_str().map(|st| st.ends_with(&s)).unwrap_or(false) { None }
        else { Some(format!("The {} must end with '{}'.", field, s)) }
    })
}

pub fn contains_rule(needle: &str) -> RuleFn {
    let n = needle.to_string();
    Box::new(move |field, val, _| {
        if val.as_str().map(|s| s.contains(&n)).unwrap_or(false) { None }
        else { Some(format!("The {} must contain '{}'.", field, n)) }
    })
}

pub fn required_if(other_field: &str, other_value: &str) -> RuleFn {
    let of = other_field.to_string();
    let ov = other_value.to_string();
    Box::new(move |field, val, data| {
        let other = data.get(&of).and_then(|v| v.as_str()).unwrap_or("");
        if other == ov && (val.is_null() || val.as_str().map(|s| s.is_empty()).unwrap_or(false)) {
            Some(format!("The {} field is required when {} is {}.", field, of, ov))
        } else {
            None
        }
    })
}

pub fn prohibited_if(other_field: &str, other_value: &str) -> RuleFn {
    let of = other_field.to_string();
    let ov = other_value.to_string();
    Box::new(move |field, val, data| {
        let other = data.get(&of).and_then(|v| v.as_str()).unwrap_or("");
        if other == ov && !val.is_null() {
            Some(format!("The {} field is prohibited when {} is {}.", field, of, ov))
        } else {
            None
        }
    })
}
