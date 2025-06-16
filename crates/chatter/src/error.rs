use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatterError {
    #[error(transparent)]
    OpenAIError(#[from] async_openai::error::OpenAIError),
    #[error(transparent)]
    PostgresError(#[from] tokio_postgres::Error),
    #[error(transparent)]
    EnvError(#[from] std::env::VarError),
    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
    #[error(transparent)]
    DeadpoolPostgresPoolError(#[from] deadpool_postgres::PoolError),
    #[error(transparent)]
    DeadpoolPostgresCreatePoolError(#[from] deadpool_postgres::CreatePoolError),

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
    #[error("Data request creation error: {0}")]
    DataRequestCreationError(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

pub type Result<T> = std::result::Result<T, ChatterError>;
