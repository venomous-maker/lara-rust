use lara_router::FormRequest;
use lara_validator::Rule;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl FormRequest for RegisterRequest {
    fn rules() -> Vec<(&'static str, Vec<Rule>)> {
        vec![
            ("name",     vec![Rule::Required, Rule::MinLength(2), Rule::MaxLength(100)]),
            ("email",    vec![Rule::Required, Rule::Email]),
            ("password", vec![Rule::Required, Rule::MinLength(8)]),
        ]
    }
}
