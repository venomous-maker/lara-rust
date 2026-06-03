use lara_router::FormRequest;
use lara_validator::Rule;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SyncPermissionsRequest {
    pub permission_ids: Vec<i64>,
}

impl FormRequest for SyncPermissionsRequest {
    fn rules() -> Vec<(&'static str, Vec<Rule>)> {
        vec![
            ("permission_ids", vec![Rule::Required]),
        ]
    }
}
