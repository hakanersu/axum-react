use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::controllers;
use crate::controllers::AppState;

/// Build all application routes.
///
/// In axum, routes are built as a tree of `Router` objects that you compose together.
/// This is different from Express where you call app.get(), app.post() imperatively.
///
/// The `Arc<AppState>` type parameter means "this router expects shared state of type AppState".
/// `Arc` (Atomic Reference Count) lets multiple handler threads share the same state safely.
pub fn build_routes(state: Arc<AppState>) -> Router {
    // Auth routes - these handle registration, login, logout
    let auth_routes = Router::new()
        .route("/register", post(controllers::register))
        .route("/login", post(controllers::login))
        .route("/logout", post(controllers::logout))
        .route("/me", get(controllers::me));

    // Health check route - useful for monitoring and load balancers
    let health_routes = Router::new()
        .route("/health", get(health_check));

    // Combine all route groups under /api prefix
    // `.merge()` combines two routers like Object.assign() merges objects
    // `.nest()` adds a path prefix to all routes in a router
    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api", health_routes)
        .with_state(state) // Make AppState available to all handlers
}

/// Simple health check handler.
/// Returns 200 OK with a JSON body - used by load balancers and monitoring.
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION") // Read from Cargo.toml at compile time!
    }))
}
