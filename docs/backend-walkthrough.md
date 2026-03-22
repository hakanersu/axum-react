# Backend Walkthrough

This document takes you through the key parts of the backend code.

## 🏁 1. The Entry Point: `main.rs`

The `main` function is the heartbeat of the app. It does several things before starting the server:

*   **`#[tokio::main]`:** This macro turns the `main` function into an async function and starts the Tokio runtime.
*   **Structured Logging:** Uses `tracing-subscriber` to print colored logs to your terminal.
*   **Loading Config:** `AppConfig::from_env()` loads values from `.env`.
*   **Database Setup:** Connects to the database and runs migrations.
*   **Middleware Layers:**
    *   `session_layer`: Manages the "user" cookie.
    *   `cors`: Allows the frontend (on port 5173) to talk to the backend (on port 3000).
    *   `TraceLayer`: Logs every incoming HTTP request.

## 🛣️ 2. The Router: `routes/mod.rs`

The router defines the API endpoints. Unlike Express, where you call `app.get()` imperatively, Axum uses a more functional, declarative style.

```rust
Router::new()
    .nest("/api/auth", auth_routes) // Prefix routes with /api/auth
    .with_state(state) // Inject the database pool into all handlers
```

## 🎮 3. Controllers: `controllers/auth_controller.rs`

This is where the business logic lives. Let's look at `register`:

```rust
pub async fn register(
    State(state): State<Arc<AppState>>, // Extractor: Get shared state
    session: Session,                   // Extractor: Get current session
    Json(body): Json<CreateUserDto>,    // Extractor: Parse JSON body
) -> Result<Json<Value>, AppError> {
    // 1. Validate input
    body.validate()?;

    // 2. Hash password with Argon2
    let password_hash = hash_password(&body.password);

    // 3. Save to database
    sqlx::query("INSERT INTO users...").execute(&state.db.pool).await?;

    // 4. Set session cookie
    set_user_session(&session, &user_response).await?;

    // 5. Return success
    Ok(Json(json!({ "message": "Success!" })))
}
```

## 📦 4. Models & Data: `models/user.rs`

We use **DTOs (Data Transfer Objects)** like `CreateUserDto` for incoming data and `UserResponse` for outgoing data.

*   **`#[derive(Serialize, Deserialize)]`**: This tells Rust how to convert the struct to and from JSON using the `serde` crate.
*   **`#[serde(skip_serializing)]`**: We use this on the `password_hash` field to ensure it NEVER leaves the server.
*   **Validation**: The `#[validate(email)]` attributes on `CreateUserDto` make it easy to check user input without writing complex regex yourself.
