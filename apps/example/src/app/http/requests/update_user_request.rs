use lara_router::FormRequest;
use lara_validator::Rule;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
}

impl FormRequest for UpdateUserRequest {
    fn rules() -> Vec<(&'static str, Vec<Rule>)> {
        vec![
            // `Sometimes` skips the field entirely when it is absent from the body.
            ("name",   vec![Rule::Sometimes, Rule::MinLength(2), Rule::MaxLength(100)]),
            ("email",  vec![Rule::Sometimes, Rule::Email]),
            ("status", vec![Rule::Sometimes, Rule::In(vec!["active".into(), "inactive".into(), "banned".into()])]),
        ]
    }
}
