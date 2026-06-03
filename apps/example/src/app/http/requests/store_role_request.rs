use lara_router::FormRequest;
use lara_validator::Rule;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StoreRoleRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

impl FormRequest for StoreRoleRequest {
    fn rules() -> Vec<(&'static str, Vec<Rule>)> {
        vec![
            ("name", vec![Rule::Required, Rule::MinLength(2)]),
            ("slug", vec![Rule::Required, Rule::MinLength(2), Rule::Regex(r"^[a-z0-9\-]+$".into())]),
            ("description", vec![Rule::Sometimes, Rule::MaxLength(255)]),
        ]
    }
}
