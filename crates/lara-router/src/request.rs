use axum::{
    body::Body,
    extract::{FromRequest, Path, Query, Request},
    http::{header, HeaderMap, Method, StatusCode, Uri},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

/// Enriched request wrapper.
pub struct LaraRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub body: Value,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl LaraRequest {
    pub fn input(&self, key: &str) -> Option<&Value> {
        self.body.get(key)
    }

    pub fn input_str(&self, key: &str) -> Option<&str> {
        self.body.get(key).and_then(|v| v.as_str())
    }

    pub fn input_i64(&self, key: &str) -> Option<i64> {
        self.body.get(key).and_then(|v| v.as_i64())
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.query_params.get(key).map(|s| s.as_str())
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.path_params.get(key).map(|s| s.as_str())
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    pub fn bearer_token(&self) -> Option<&str> {
        self.header(header::AUTHORIZATION.as_str())
            .and_then(|v| v.strip_prefix("Bearer "))
    }

    pub fn ip(&self) -> Option<&str> {
        self.header("x-forwarded-for")
            .or_else(|| self.header("x-real-ip"))
    }

    pub fn is_json(&self) -> bool {
        self.header("content-type")
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    pub fn path(&self) -> &str {
        self.uri.path()
    }

    pub fn all(&self) -> &Value {
        &self.body
    }
}
