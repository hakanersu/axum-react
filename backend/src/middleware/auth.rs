use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use tower_sessions::Session;

use crate::errors::AppError;
use crate::models::UserResponse;

/// Key used to store user data in the session.
/// Think of sessions like a server-side HashMap<String, Value>.
/// This constant is the key we use to store/retrieve the user.
const USER_SESSION_KEY: &str = "user";

/// Middleware extractor that requires authentication.
///
/// In axum, "extractors" are types that implement `FromRequestParts` or `FromRequest`.
/// When you put them as function parameters in a handler, axum automatically
/// runs the extraction logic before your handler code.
///
/// Example usage in a handler:
/// ```
/// async fn profile(auth: AuthUser) -> impl IntoResponse {
///     // auth.user is guaranteed to be a valid, logged-in user
///     Json(auth.user)
/// }
/// ```
///
/// If the user isn't logged in, the handler never runs - axum returns 401 instead.
pub struct AuthUser {
    pub user: UserResponse,
}

/// Implement the `FromRequestParts` trait to make `AuthUser` an extractor.
///
/// `#[async_trait]` is a macro that makes async functions work in traits.
/// Rust doesn't support `async fn` in traits natively yet (it's coming in future versions),
/// so this macro transforms it into something the compiler accepts.
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync, // Required bounds - S is the application state type
{
    type Rejection = AppError; // What to return if extraction fails

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First, extract the Session from the request.
        // Sessions are identified by a cookie that axum-sessions manages automatically.
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        // Try to get the user data from the session store.
        // `session.get::<UserResponse>(key)` deserializes the stored JSON back into our struct.
        // It returns Option<Option<T>>... the outer Option is for errors, inner for "key not found"
        let user: UserResponse = session
            .get(USER_SESSION_KEY)
            .await
            .map_err(|_| AppError::Unauthorized)?  // Session store error
            .ok_or(AppError::Unauthorized)?;        // No user in session = not logged in

        Ok(AuthUser { user })
    }
}

/// Helper functions for session management.
/// These are standalone functions, not methods on a struct.
pub async fn set_user_session(session: &Session, user: &UserResponse) -> Result<(), AppError> {
    session
        .insert(USER_SESSION_KEY, user)
        .await
        .map_err(|e| AppError::Internal(format!("Session error: {}", e)))?;
    Ok(())
}

pub async fn clear_user_session(session: &Session) -> Result<(), AppError> {
    session.flush().await.map_err(|e| AppError::Internal(format!("Session error: {}", e)))?;
    Ok(())
}
