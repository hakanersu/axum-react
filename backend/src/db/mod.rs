use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};
use sqlx::{AnyPool, any::AnyPoolOptions};
use crate::config::AppConfig;

/// Database connection pool wrapper.
///
/// A "pool" maintains multiple open database connections that can be
/// reused across requests. This is crucial for performance - creating
/// a new connection per request would be very slow.
///
/// `AnyPool` is sqlx's database-agnostic pool - it works with SQLite,
/// PostgreSQL, MySQL, etc. based on the connection URL.
#[derive(Clone)]
pub struct Database {
    pub pool: AnyPool,
}

impl Database {
    /// Creates a new database connection pool.
    ///
    /// `async fn` means this function returns a Future that must be `.await`ed.
    /// Unlike JavaScript where async is transparent, Rust futures are lazy -
    /// they don't do anything until you await them.
    pub async fn connect(config: &AppConfig) -> Result<Self, sqlx::Error> {
        // Install drivers for all database types we might use.
        // This is needed because sqlx's "any" driver needs to know
        // which concrete drivers are available at runtime.
        sqlx::any::install_default_drivers();

        let pool = AnyPoolOptions::new()
            // Max 10 simultaneous connections. For SQLite this matters less
            // (it's a single file), but for PostgreSQL this prevents
            // overwhelming the database server.
            .max_connections(10)
            // If all connections are in use, wait up to 30s before giving up
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&config.database_url)
            .await?; // `?` propagates the error if connection fails

        tracing::info!("Connected to database: {}", &config.database_url);

        Ok(Self { pool })
    }

    /// Run initial migrations to set up the database schema.
    /// This creates the `users` table and `sessions` table if they don't exist.
    ///
    /// We use raw SQL here instead of sqlx migrations because we need
    /// database-agnostic SQL that works on both SQLite and PostgreSQL.
    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        // Create users table
        // Note: SQLite and PostgreSQL have slightly different syntax,
        // but this subset works on both.
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create session storage table for tower-sessions
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tower_sessions (
                id TEXT PRIMARY KEY NOT NULL,
                data BLOB NOT NULL,
                expiry_date INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Database migrations completed");
        Ok(())
    }

    /// Seed the database with initial users for development/testing.
    /// This is idempotent — users that already exist are skipped.
    pub async fn run_seeds(&self) -> Result<(), Box<dyn std::error::Error>> {
        let seed_users = [
            ("admin", "admin@example.com", "password123"),
            ("user",  "user@example.com",  "password123"),
        ];

        let argon2 = Argon2::default();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        for (username, email, password) in seed_users {
            // Skip if this email already exists
            let exists: Option<sqlx::any::AnyRow> =
                sqlx::query("SELECT id FROM users WHERE email = $1")
                    .bind(email)
                    .fetch_optional(&self.pool)
                    .await?;

            if exists.is_some() {
                tracing::info!("Seed user '{}' already exists, skipping", email);
                continue;
            }

            let salt = SaltString::generate(&mut OsRng);
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| format!("Password hashing failed: {e}"))?
                .to_string();

            let id = uuid::Uuid::new_v4().to_string();

            sqlx::query(
                "INSERT INTO users (id, email, username, password_hash, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&id)
            .bind(email)
            .bind(username)
            .bind(&password_hash)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await?;

            tracing::info!("Seeded user: {} ({})", username, email);
        }

        Ok(())
    }
}
