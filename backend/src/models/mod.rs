pub mod user;

// Re-export commonly used types so other modules can do:
// `use crate::models::User` instead of `use crate::models::user::User`
pub use user::*;
