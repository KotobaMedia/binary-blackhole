use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[cfg(feature = "streaming")]
use futures::stream;
use lambda_http::tracing;
use sentry::integrations::anyhow::capture_anyhow;
use serde_json::json;
use std::{convert::Infallible, fmt};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    InternalServerError(anyhow::Error),
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Create a status and a body message from the error variant.
        let (status, message) = match self {
            AppError::InternalServerError(error) => {
                tracing::error!("Unhandled error: {:?}", error);

                // Capture error in Sentry
                capture_anyhow(&error);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong. Please try again later.".to_string(),
                )
            }
            AppError::Conflict(code) => {
                let json_body = json!({ "error_code": code });
                let message =
                    serde_json::to_string(&json_body).unwrap_or_else(|_| "Conflict".into());
                (StatusCode::CONFLICT, message)
            }
        };

        // If streaming is enabled, wrap the string in a stream body;
        // otherwise, use the standard response.
        #[cfg(feature = "streaming")]
        {
            let stream = stream::once(async move { Ok::<_, Infallible>(message) });
            Response::builder()
                .status(status)
                .body(Body::from_stream(stream))
                .unwrap()
        }

        #[cfg(not(feature = "streaming"))]
        {
            (status, message).into_response()
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
