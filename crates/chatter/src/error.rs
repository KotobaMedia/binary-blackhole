use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatterError {
    #[error("OpenAI error: {0}")]
    OpenAIError(#[from] async_openai::error::OpenAIError),
    #[error("Postgres error: {0}")]
    PostgresError(#[from] tokio_postgres::Error),
    #[error("Environment error: {0}")]
    EnvError(#[from] std::env::VarError),
    #[error("JSON Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to create function execution context failed: {0}")]
    ExecutionContextBuilderError(#[from] crate::functions::ExecutionContextBuilderError),

    #[error("Unknown Tool Call: {0}")]
    UnknownToolCall(String),
    #[error("Unknown Role: {0}")]
    UnknownRole(String),

    #[error("Geometry was not found in the query result")]
    GeometryNotFound,

    #[error("ToSQL Error: {0}")]
    ToSQLError(#[from] km_to_sql::error::Error),

    #[error("SQL Query Error: {0}")]
    QueryError(String),
    #[error("SQL query creation error: {0}")]
    SqlQueryCreationError(String),
}

pub type Result<T> = std::result::Result<T, ChatterError>;
