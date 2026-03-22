use serde::{Deserialize, Serialize};
use sqlx::any::AnyRow;
use sqlx::Row;
use validator::Validate;

/// The User model represents a row in the `users` table.
///
/// Derive macros explained:
/// - `Debug`: enables {:?} formatting for logging
/// - `Clone`: allows duplicating this struct
/// - `Serialize`: converts struct → JSON (for API responses)
/// - `Deserialize`: converts JSON → struct (for reading from DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    /// `#[serde(skip_serializing)]` ensures the password hash
    /// is NEVER included in JSON responses - critical for security!
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Manual implementation of FromRow for AnyRow.
///
/// sqlx's `FromRow` derive macro doesn't work with `AnyRow` (the database-agnostic row type),
/// so we manually map columns to struct fields.
/// `row.get("column_name")` extracts a value from the row by column name.
impl User {
    pub fn from_row(row: &AnyRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            username: row.try_get("username")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// DTO (Data Transfer Object) for user registration.
///
/// DTOs are separate structs for incoming data. This is important because:
/// 1. The incoming data has different fields than the DB model (no id, no timestamps)
/// 2. We can add validation rules specific to registration
/// 3. We never accidentally expose internal fields
///
/// `Validate` derive enables the `validate()` method which checks
/// the `#[validate(...)]` attributes on each field.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// DTO for login - simpler than registration
#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

/// What we return to the frontend after login/registration.
/// Notice: NO password_hash field here - this is a public-facing struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Convert a full User into a safe UserResponse.
///
/// `impl From<A> for B` is a Rust pattern for type conversion.
/// It enables: `let response: UserResponse = user.into();`
/// or: `UserResponse::from(user)`
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
