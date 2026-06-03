use lara_router::FormRequest;
use lara_validator::Rule;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl FormRequest for LoginRequest {
    fn rules() -> Vec<(&'static str, Vec<Rule>)> {
        vec![
            ("email",    vec![Rule::Required, Rule::Email]),
            ("password", vec![Rule::Required, Rule::MinLength(6)]),
        ]
    }
}
