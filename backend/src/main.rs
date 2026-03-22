// Module declarations - these tell Rust's compiler about our code structure.
// Each `mod` statement corresponds to a file or directory with a mod.rs.
// This is different from JS/TS where you import files directly.
mod config;
mod controllers;
mod db;
mod errors;
mod middleware;
mod models;
mod routes;

use std::sync::Arc;
use axum::http::{HeaderName, Method};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions::cookie::{time::Duration, SameSite};

use config::AppConfig;
use controllers::AppState;
use db::Database;

/// Application entry point.
///
/// `#[tokio::main]` is a macro that:
/// 1. Creates a tokio async runtime
/// 2. Runs our async main function on it
///
/// Without this, you'd have to manually create a Runtime and call block_on().
/// Tokio is Rust's most popular async runtime - it manages thread pools,
/// task scheduling, and I/O efficiently.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging.
    // RUST_LOG env var controls verbosity: error, warn, info, debug, trace
    // Default to "info" level if not set.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    // Load configuration from .env file and environment variables
    let config = AppConfig::from_env();

    // Connect to database and run migrations
    let database = Database::connect(&config).await?;
    database.run_migrations().await?;

    // Set up session store backed by our database.
    //
    // This is where cookie-based auth happens:
    // 1. On login, we store user data in the session store (database)
    // 2. A session ID cookie is sent to the browser
    // 3. On each request, the cookie is read, session is loaded from DB
    // 4. The handler can access the session data
    //
    // This is more secure than JWT because:
    // - Sessions can be revoked instantly (delete from DB)
    // - No sensitive data in the cookie (just a random session ID)
    // - Session data lives server-side, not client-side
    let session_store = tower_sessions_sqlx_store::SqliteStore::new(
        // We need a sqlx::SqlitePool specifically for the session store
        // This is a limitation of the library - it can't use AnyPool
        sqlx::SqlitePool::connect(&config.database_url).await?,
    );

    // Create the session management layer (middleware).
    // This intercepts every request/response to manage cookies.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.cookie_secure)     // HTTPS only in production
        .with_same_site(SameSite::Lax)          // CSRF protection
        .with_http_only(true)                    // JavaScript can't access the cookie
        .with_expiry(Expiry::OnInactivity(Duration::days(7))); // Auto-expire after 7 days idle

    // CORS configuration - controls which domains can call our API.
    // In development, the React dev server runs on a different port (5173),
    // so we need to allow cross-origin requests from it.
    let cors = CorsLayer::new()
        .allow_origin(config.frontend_url.parse::<axum::http::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-requested-with"),
        ])
        .allow_credentials(true); // Required for cookies to work cross-origin!

    // Build shared application state.
    // `Arc::new()` wraps it in an atomic reference counter so it can be
    // safely shared across all handler threads.
    let state = Arc::new(AppState { db: database });

    // Build the full application by combining routes + middleware layers.
    // Layers are applied bottom-to-top (last added = runs first).
    let app = routes::build_routes(state)
        .layer(session_layer) // Session management
        .layer(cors)          // CORS headers
        .layer(TraceLayer::new_for_http()); // Request logging

    // Start the HTTP server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
