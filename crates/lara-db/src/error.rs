use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("MongoDB error: {0}")]
    #[cfg(feature = "mongodb")]
    Mongo(#[from] mongodb::error::Error),

    #[error("BSON serialization error: {0}")]
    #[cfg(feature = "mongodb")]
    Bson(#[from] bson::ser::Error),

    #[error("BSON deserialization error: {0}")]
    #[cfg(feature = "mongodb")]
    BsonDe(#[from] bson::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Record not found")]
    NotFound,

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("Foreign key violation: {0}")]
    ForeignKeyViolation(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Unsupported operation for this driver: {0}")]
    UnsupportedOperation(String),

    #[error("Mass-assignment violation — `{0}` is not fillable")]
    MassAssignment(String),

    #[error("Relation `{0}` is not defined on `{1}`")]
    RelationNotDefined(String, String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
