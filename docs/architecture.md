# Architecture Overview

This project uses a modern Rust web stack. Each component is chosen for its performance, safety, and ecosystem support.

## 🏗️ The Stack

1.  **Axum (Web Framework):**
    *   Built by the Tokio team.
    *   Highly modular and uses standard `tower` middleware.
    *   Uses **Extractors** to parse requests (like `Json(body)` or `State(state)`).
2.  **Tokio (Async Runtime):**
    *   The engine that drives the async code.
    *   Handles the event loop, thread pool, and I/O.
3.  **SQLx (Database Access):**
    *   Fully async and safe.
    *   Supports compile-time SQL verification (though we use the dynamic `AnyPool` for database-agnosticism).
4.  **Tower-sessions (Authentication):**
    *   Handles cookie-based session management.
    *   Stores session data in the database (SQLite or Postgres).

## 🗄️ Module Structure

The project follows a standard Rust module pattern:

```text
src/
├── main.rs          # Application entry point & setup
├── config/          # Environment variable loading
├── controllers/     # Request handlers (logic)
├── db/              # Database connection & migrations
├── errors/          # Custom error handling (AppError)
├── middleware/      # Custom middleware (like Auth)
├── models/          # Data structures (User, DTOs)
└── routes/          # API route definitions
```

### 🧩 How it fits together

1.  `main.rs` initializes the logger, config, and database.
2.  It wraps the database pool in an `Arc` (thread-safe pointer) and puts it in `AppState`.
3.  `routes/mod.rs` builds the router and attaches middleware (CORS, Sessions).
4.  When a request hits a route, `axum` calls the corresponding function in `controllers/`.
5.  The controller interacts with the database (`db/`) and returns a `Result`.
6.  If it returns an `Err(AppError)`, Axum converts it into a JSON response via the `IntoResponse` trait in `errors/mod.rs`.
