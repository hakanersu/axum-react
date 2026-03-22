use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::Row;
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

use crate::db::Database;
use crate::errors::AppError;
use crate::middleware::auth::{clear_user_session, set_user_session, AuthUser};
use crate::models::{CreateUserDto, LoginDto, User, UserResponse};

use std::sync::Arc;

/// Application state shared across all request handlers.
///
/// `Arc` (Atomic Reference Count) is Rust's thread-safe smart pointer.
/// It lets multiple threads hold a reference to the same data without copying it.
/// When the last reference is dropped, the data is freed.
///
/// In JavaScript, this happens automatically because everything is garbage collected
/// and single-threaded. In Rust, you need to be explicit about shared ownership.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

/// Register a new user.
///
/// `State(state)`: Extracts shared AppState from the request (via Arc)
/// `Json(body)`: Extracts and deserializes the JSON request body into CreateUserDto
///
/// The return type `Result<Json<Value>, AppError>` means:
/// - On success: returns JSON response
/// - On failure: returns our custom AppError (which becomes an HTTP error response)
pub async fn register(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(body): Json<CreateUserDto>,
) -> Result<Json<Value>, AppError> {
    // Run validation rules defined on the DTO struct (#[validate(...)] attributes)
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if email already exists.
    // `sqlx::query()` creates a raw SQL query.
    // `.bind()` safely inserts parameters (prevents SQL injection).
    // `.fetch_optional()` returns Option<Row> - None if no rows match.
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.db.pool)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".into()));
    }

    // Check if username already exists
    let existing = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(&body.username)
        .fetch_optional(&state.db.pool)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Username already taken".into()));
    }

    // Hash the password using Argon2.
    //
    // Argon2 is the winner of the Password Hashing Competition (2015).
    // It's memory-hard, meaning attackers can't just throw GPUs at it.
    //
    // `SaltString::generate` creates a random salt (prevents rainbow table attacks).
    // The salt is embedded in the hash output, so you don't store it separately.
    let salt = argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = argon2::Argon2::default();
    let password_hash = argon2::PasswordHasher::hash_password(&argon2, body.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
        .to_string();

    // Generate a UUID for the new user
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Insert the new user into the database
    sqlx::query(
        "INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
        .bind(&id)
        .bind(&body.email)
        .bind(&body.username)
        .bind(&password_hash)
        .bind(&now)
        .bind(&now)
        .execute(&state.db.pool)
        .await?;

    // Build the response
    let user_response = UserResponse {
        id,
        email: body.email,
        username: body.username,
        created_at: now.clone(),
        updated_at: now,
    };

    // Set the session cookie - this logs the user in immediately after registration
    set_user_session(&session, &user_response).await?;

    Ok(Json(json!({
        "user": user_response,
        "message": "Registration successful"
    })))
}

/// Login with email and password.
pub async fn login(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(body): Json<LoginDto>,
) -> Result<Json<Value>, AppError> {
    // Find user by email
    let row = sqlx::query("SELECT * FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let user = User::from_row(&row)?;

    // Verify password against stored hash.
    //
    // `PasswordHash::new` parses the stored hash string back into its components.
    // `verify_password` checks if the provided password matches.
    // This is constant-time to prevent timing attacks.
    let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("Hash parse error: {}", e)))?;

    argon2::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        body.password.as_bytes(),
        &parsed_hash,
    )
    .map_err(|_| AppError::InvalidCredentials)?;

    let user_response = UserResponse::from(user);
    set_user_session(&session, &user_response).await?;

    Ok(Json(json!({
        "user": user_response,
        "message": "Login successful"
    })))
}

/// Logout - clears the session.
pub async fn logout(session: Session) -> Result<Json<Value>, AppError> {
    clear_user_session(&session).await?;
    Ok(Json(json!({ "message": "Logged out successfully" })))
}

/// Get current user info - requires authentication.
///
/// `AuthUser` is our custom extractor. If the user isn't logged in,
/// this handler never runs - axum returns 401 automatically.
pub async fn me(auth: AuthUser) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "user": auth.user })))
}
