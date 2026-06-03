use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use lara_validator::{Rule, Validator};

/// Implement `FormRequest` to declare validation rules + authorization for an
/// incoming request body. Use together with the [`Validated<T>`] extractor.
///
/// # Example
/// ```rust
/// #[derive(Deserialize)]
/// pub struct StoreUserRequest {
///     pub name: String,
///     pub email: String,
///     pub password: String,
/// }
///
/// impl FormRequest for StoreUserRequest {
///     fn rules() -> Vec<(&'static str, Vec<Rule>)> {
///         vec![
///             ("name",     vec![Rule::Required, Rule::MinLength(2)]),
///             ("email",    vec![Rule::Required, Rule::Email]),
///             ("password", vec![Rule::Required, Rule::MinLength(8)]),
///         ]
///     }
/// }
/// ```
pub trait FormRequest: DeserializeOwned + Send {
    /// Validation rules keyed by field name.
    fn rules() -> Vec<(&'static str, Vec<Rule>)>;

    /// Authorization gate — return `false` to reject with 403.
    /// Note: runs on the raw JSON before deserialization.
    fn authorize(_data: &Value) -> bool {
        true
    }
}

/// Axum extractor that runs `FormRequest` validation before handing you the typed struct.
///
/// On validation failure returns `422 Unprocessable Entity` with a JSON error body.
/// On authorization failure returns `403 Forbidden`.
pub struct Validated<T>(pub T);

/// Error response produced by the `Validated` extractor.
pub enum ValidationRejection {
    InvalidJson(String),
    Unauthorized,
    Failed(lara_validator::Errors),
}

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        match self {
            ValidationRejection::InvalidJson(msg) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "message": "Invalid JSON body", "error": msg })),
            ).into_response(),
            ValidationRejection::Unauthorized => (
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "This action is unauthorized." })),
            ).into_response(),
            ValidationRejection::Failed(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "message": "The given data was invalid.", "errors": errors.errors })),
            ).into_response(),
        }
    }
}

impl<T, S> FromRequest<S> for Validated<T>
where
    T: FormRequest,
    S: Send + Sync,
{
    type Rejection = ValidationRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Read the raw body
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| ValidationRejection::InvalidJson(e.to_string()))?;

        // Parse to JSON (empty body → empty object)
        let value: Value = if bytes.is_empty() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_slice(&bytes)
                .map_err(|e| ValidationRejection::InvalidJson(e.to_string()))?
        };

        // Authorization gate
        if !T::authorize(&value) {
            return Err(ValidationRejection::Unauthorized);
        }

        // Validation
        let map = value
            .as_object()
            .cloned()
            .unwrap_or_default();

        let mut validator = Validator::new();
        for (field, rules) in T::rules() {
            validator = validator.field(field, rules);
        }

        if let Err(lara_validator::ValidatorError::ValidationFailed(errors)) = validator.validate(&map) {
            return Err(ValidationRejection::Failed(errors));
        }

        // Deserialize into the typed struct
        let typed: T = serde_json::from_value(value)
            .map_err(|e| ValidationRejection::InvalidJson(e.to_string()))?;

        Ok(Validated(typed))
    }
}
