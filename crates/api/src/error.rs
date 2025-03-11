use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use lambda_http::tracing;
use serde_json::json;
use std::fmt;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    InternalServerError(anyhow::Error),
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InternalServerError(error) => {
                tracing::error!("Unhandled error: {:?}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Something went wrong. Please try again later."),
                )
                    .into_response()
            }
            AppError::Conflict(code) => (
                StatusCode::CONFLICT,
                Json(json!({
                    "error_code": code,
                })),
            )
                .into_response(),
        }
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::InternalServerError(err.into())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InternalServerError(error) => write!(f, "Internal server error: {}", error),
            AppError::Conflict(code) => write!(f, "Conflict: {}", code),
        }
    }
}
