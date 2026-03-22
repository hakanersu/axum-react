use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Custom error type for our application.
///
/// `#[derive(Debug, thiserror::Error)]` auto-generates:
/// - Debug trait for debugging output
/// - std::error::Error trait with proper Display implementation
///
/// Each variant represents a different kind of error that can occur.
/// The `#[error("...")]` attribute defines what gets shown when you print the error.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error), // `#[from]` lets us use `?` with sqlx errors

    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// This implementation tells axum how to convert our AppError into an HTTP response.
///
/// `IntoResponse` is a trait (like an interface in TypeScript) that axum uses
/// to convert any type into an HTTP response. By implementing it for AppError,
/// we can return `Result<T, AppError>` from any handler and axum will
/// automatically convert errors into proper JSON error responses.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // `match` is like a switch statement but MUCH more powerful:
        // 1. The compiler ensures you handle every variant (exhaustive)
        // 2. You can destructure values inside variants
        // 3. It returns a value (it's an expression, not a statement)
        let (status, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        // Log server errors for debugging
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("Internal error: {}", message);
        }

        // Build JSON response body
        // `json!` macro creates a serde_json::Value, like a JS object literal
        let body = axum::Json(json!({
            "error": {
                "status": status.as_u16(),
                "message": message,
            }
        }));

        // Combine status code + JSON body into a full response
        // The tuple `(StatusCode, Json<Value>)` implements IntoResponse
        (status, body).into_response()
    }
}
