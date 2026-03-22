use std::env;

/// Represents which database backend we're using.
/// Rust enums are much more powerful than TypeScript enums -
/// each variant can hold different data (called "algebraic data types").
#[derive(Debug, Clone)]
pub enum DbType {
    Sqlite,
    Postgres,
}

/// Application configuration, loaded from environment variables.
/// `#[derive(Debug, Clone)]` auto-generates implementations:
/// - Debug: lets us print with {:?} for debugging
/// - Clone: lets us duplicate the struct (needed to share across threads)
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub session_secret: String,
    pub cookie_domain: String,
    pub cookie_secure: bool,
    pub frontend_url: String,
}

impl AppConfig {
    /// Loads config from environment variables with sensible defaults.
    ///
    /// In Rust, there's no constructor keyword - by convention we use
    /// `fn new()` or `fn from_env()` as associated functions (like static methods).
    pub fn from_env() -> Self {
        // dotenvy loads .env file into environment variables
        dotenvy::dotenv().ok(); // .ok() ignores errors (no .env file is fine)

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data.db?mode=rwc".to_string());

        // Determine DB type from the URL prefix
        let db_type = if database_url.starts_with("postgres") {
            DbType::Postgres
        } else {
            DbType::Sqlite
        };

        Self {
            database_url,
            db_type,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse::<u16>()
                .expect("PORT must be a valid number"),
            session_secret: env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "change-me-to-a-64-char-secret-key-in-production-please!-really!".to_string()),
            cookie_domain: env::var("COOKIE_DOMAIN")
                .unwrap_or_else(|_| "localhost".to_string()),
            cookie_secure: env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
        }
    }
}
