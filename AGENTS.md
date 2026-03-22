# Sekizgen – AI Agent Guide

This file is the single source of truth for AI assistants working on this repository.
It covers architecture, conventions, gotchas, and step-by-step guides for common tasks.

---

## What Is Sekizgen?

Sekizgen is a full-stack web framework starter built with:

- **Backend** – Rust · [Axum 0.8](https://github.com/tokio-rs/axum) · SQLx · SQLite (default) / PostgreSQL
- **Frontend** – React 18 · Vite · TypeScript · Tailwind CSS · Zustand · Radix UI
- **CLI** – `sekizgen` binary (in `cli/`) for scaffolding new projects and generating code

The CLI works like Laravel's artisan: `sekizgen new blog` clones this repo as a template and renames everything to the project name.

---

## Repository Layout

```
/
├── backend/                  # Axum REST API
│   └── src/
│       ├── config/mod.rs     # Loads env vars → AppConfig struct
│       ├── controllers/      # HTTP handlers (auth_controller.rs, …)
│       ├── db/mod.rs         # Connection pool, migrations, seeds
│       ├── errors/mod.rs     # AppError enum → JSON HTTP responses
│       ├── middleware/auth.rs # AuthUser extractor (session-based auth)
│       ├── models/user.rs    # User, UserResponse, DTOs
│       ├── routes/mod.rs     # Route tree, /api/health
│       └── main.rs           # Entry point, CORS, session layer
├── frontend/                 # React + Vite SPA
│   └── src/
│       ├── lib/api.ts        # apiFetch wrapper + authApi methods
│       ├── stores/auth-store.ts  # Zustand auth state
│       ├── types/index.ts    # TypeScript interfaces (mirror Rust structs)
│       ├── pages/            # LoginPage, RegisterPage, DashboardPage
│       └── components/ui/    # Button, Card, Input, Label (shadcn-style)
├── cli/src/main.rs           # sekizgen CLI (new, generate model/controller/scaffold)
├── Makefile                  # Dev runner
├── Cargo.toml                # Workspace (members: backend, cli)
└── .env                      # Environment variables (loaded by dotenvy)
```

---

## Running the Project

### Prerequisites

- Rust (stable, ≥ 1.75)
- Node.js + npm
- `make`

### Commands

```bash
make dev        # Start backend + frontend concurrently (Ctrl-C kills both)
make backend    # Backend only  → http://localhost:3000
make frontend   # Frontend only → http://localhost:5173
```

### Environment Variables (`.env`)

All variables have sensible defaults; the file is optional in development.

| Variable        | Default                               | Notes                                     |
|-----------------|---------------------------------------|-------------------------------------------|
| `DATABASE_URL`  | `sqlite:data.db?mode=rwc`             | Prefix with `postgres://` to use Postgres |
| `HOST`          | `0.0.0.0`                             |                                           |
| `PORT`          | `3000`                                |                                           |
| `SESSION_SECRET`| *(hardcoded dev value)*               | **Change in production**                  |
| `COOKIE_SECURE` | `false`                               | Set `true` in production (HTTPS only)     |
| `FRONTEND_URL`  | `http://localhost:5173`               | Used for CORS `allow_origin`              |
| `RUST_LOG`      | `info,sqlx=warn`                      | Tracing filter                            |

---

## Backend Architecture

### Request Lifecycle

```
HTTP request
  → TraceLayer (logging)
  → CorsLayer (CORS headers)
  → SessionManagerLayer (reads/writes session cookie)
  → Router (route matching)
  → Extractor (State, Session, Json, AuthUser)
  → Handler function
  → Result<Json<Value>, AppError>
  → AppError::into_response() if Err
```

### AppState

Shared across all handlers via `Arc<AppState>`. Contains a single `Database` field with an `AnyPool` connection pool. Add new shared resources here.

```rust
pub struct AppState {
    pub db: Database,
}
```

### Database

- `Database::connect()` – creates the `AnyPool` and calls `sqlx::any::install_default_drivers()`
- `Database::run_migrations()` – creates `users` and `tower_sessions` tables on startup (idempotent)
- `Database::run_seeds()` – inserts dev users on startup (idempotent, skips existing)

Default seed users:

| username | email               | password    |
|----------|---------------------|-------------|
| admin    | admin@example.com   | password123 |
| user     | user@example.com    | password123 |

### Authentication

Cookie-based session auth via `tower-sessions` + `tower-sessions-sqlx-store`.

- **Login** → session store saves `UserResponse` under key `"user"` → browser gets a `Set-Cookie`
- **Subsequent requests** → browser sends cookie → session middleware loads user → `AuthUser` extractor makes it available to handlers
- **Logout** → `session.flush()` destroys the session

The `AuthUser` extractor in `middleware/auth.rs` implements `FromRequestParts`. Add it as a parameter to any handler that requires authentication:

```rust
pub async fn my_protected_handler(auth: AuthUser) -> impl IntoResponse {
    Json(json!({ "user": auth.user }))
}
```

### Error Handling

Return `Result<Json<Value>, AppError>` from handlers. `AppError` variants and their HTTP status codes:

| Variant              | Status |
|----------------------|--------|
| `Database(_)`        | 500    |
| `Unauthorized`       | 401    |
| `InvalidCredentials` | 401    |
| `Validation(msg)`    | 400    |
| `NotFound(msg)`      | 404    |
| `Conflict(msg)`      | 409    |
| `Internal(msg)`      | 500    |

### API Endpoints

| Method | Path                  | Auth required | Description              |
|--------|-----------------------|---------------|--------------------------|
| GET    | `/api/health`         | No            | Health check             |
| POST   | `/api/auth/register`  | No            | Register new user        |
| POST   | `/api/auth/login`     | No            | Login                    |
| POST   | `/api/auth/logout`    | No            | Logout (clears session)  |
| GET    | `/api/auth/me`        | Yes           | Get current user         |

---

## Frontend Architecture

- **Vite proxy** – all `/api/*` requests in dev are proxied to `http://localhost:3000`, so the frontend never makes cross-origin requests in development.
- **`@/` alias** – maps to `src/`, e.g. `import { Button } from "@/components/ui/button"`
- **`apiFetch`** – always sends `credentials: "include"` so session cookies are included
- **Zustand store** (`useAuthStore`) – holds `user`, `isLoading`, `error` and actions `login`, `register`, `logout`, `checkAuth`
- **Types in `src/types/index.ts`** must stay in sync with backend Rust structs

---

## CLI – sekizgen

Install globally:

```bash
cargo install --git https://github.com/hakanersu/axum-react sekizgen
```

### Commands

```bash
sekizgen new <name>                                  # Scaffold a new project
sekizgen generate model <name> [field:type …]        # Generate model + migration SQL
sekizgen generate controller <name> [actions …]      # Generate controller with handlers
sekizgen generate scaffold <name> [field:type …]     # Model + controller together
sekizgen g m post title:string body:text             # Short aliases (g, m, c, s)
```

Field types: `string`, `text`, `int`, `integer`, `float`, `double`, `bool`, `boolean`, `date`, `datetime`, `uuid`

### What `sekizgen new <name>` Does

1. `git clone --depth=1 https://github.com/hakanersu/axum-react <name>`
2. Removes `.git/` and `data.db`
3. Replaces all occurrences of `sekizgen` with `<name>` in `backend/Cargo.toml`, `cli/Cargo.toml`, `Cargo.toml`, `Makefile`, `frontend/package.json`, `frontend/package-lock.json`
4. Runs `npm install` in `frontend/`
5. Runs `git init && git add . && git commit -m "Initial commit"`

---

## How to Add a New Resource (Step-by-Step)

### 1. Generate the files

```bash
sekizgen g scaffold post title:string body:text published:bool
```

This creates:
- `backend/src/models/post.rs`
- `backend/src/controllers/post_controller.rs`
- `backend/migrations/create_posts.sql`

### 2. Register the model

In `backend/src/models/mod.rs`, add:

```rust
pub mod post;
pub use post::*;
```

### 3. Register the controller

In `backend/src/controllers/mod.rs`, add:

```rust
pub mod post_controller;
```

### 4. Add the migration

In `backend/src/db/mod.rs`, inside `run_migrations()`, add the SQL from the generated migration file.

### 5. Mount the routes

In `backend/src/routes/mod.rs`:

```rust
use crate::controllers::post_controller;

// Inside build_routes():
Router::new()
    .nest("/api/auth", auth_routes)
    .nest("/api", health_routes)
    .nest("/api", post_controller::routes(state.clone()))
    .with_state(state)
```

---

## Critical Patterns & Gotchas

### sqlx AnyRow type annotations

When using `AnyPool`, the compiler cannot infer the row type. Always annotate:

```rust
// CORRECT
let row: Option<sqlx::any::AnyRow> = sqlx::query("SELECT id FROM users WHERE email = $1")
    .bind(&email)
    .fetch_optional(&pool)
    .await?;

// WRONG – causes E0282 type annotations needed
let row = sqlx::query(…).fetch_optional(&pool).await?;
```

### UserResponse must derive Deserialize

`session.get::<T>()` requires `T: DeserializeOwned`. Any struct stored in the session needs both `Serialize` and `Deserialize`.

```rust
#[derive(Debug, Serialize, Deserialize)]  // Both required
pub struct UserResponse { … }
```

### CORS with credentials

`allow_credentials(true)` is incompatible with wildcard `Any` for methods or headers. Always use explicit lists:

```rust
// CORRECT
.allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
.allow_headers([HeaderName::from_static("content-type"), …])
.allow_credentials(true)

// WRONG – panics at startup
.allow_methods(Any)
.allow_headers(Any)
.allow_credentials(true)
```

### axum 0.8 – no async_trait

`axum 0.8` removed the `async_trait` re-export. `async fn` in traits is stable since Rust 1.75. Do not use `#[axum::async_trait]` or `#[async_trait::async_trait]`.

```rust
// CORRECT
impl<S> FromRequestParts<S> for AuthUser where S: Send + Sync {
    type Rejection = AppError;
    async fn from_request_parts(…) -> Result<Self, Self::Rejection> { … }
}

// WRONG
#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser …
```

### argon2 error handling

`argon2::password_hash::Error` does not implement `std::error::Error`, so `?` won't work when returning `Box<dyn std::error::Error>`. Map it explicitly:

```rust
argon2.hash_password(bytes, &salt)
    .map_err(|e| format!("Hashing failed: {e}"))?
    .to_string()
```

### tower-sessions + sqlx-store version pinning

The session crates have a lagging version relationship. The only compatible combination as of this writing:

```toml
tower-sessions = "0.14"
tower-sessions-sqlx-store = { version = "0.15", features = ["sqlite", "postgres"] }
```

Both resolve to `tower-sessions-core 0.14`. Do not upgrade either independently without checking that they share the same `tower-sessions-core` version (`cargo tree | grep tower-sessions-core`).

### sqlx AnyPool import path

```rust
// CORRECT (sqlx 0.8)
use sqlx::{AnyPool, any::AnyPoolOptions};

// WRONG
use sqlx::any::{AnyPool, AnyPoolOptions};
```

---

## Naming Conventions

| Context            | Convention         | Example                    |
|--------------------|--------------------|----------------------------|
| Rust files/modules | `snake_case`       | `auth_controller.rs`       |
| Rust types/structs | `PascalCase`       | `UserResponse`             |
| Rust functions     | `snake_case`       | `run_migrations()`         |
| DB tables          | `snake_case plural`| `users`, `blog_posts`      |
| DB columns         | `snake_case`       | `created_at`               |
| API routes         | `kebab-case`       | `/api/auth/me`             |
| TypeScript         | `camelCase`        | `authStore`, `apiFetch`    |
| React components   | `PascalCase`       | `LoginPage`, `AuthUser`    |
| Env variables      | `SCREAMING_SNAKE`  | `DATABASE_URL`             |

The project binary/package is named `sekizgen`. Do not reintroduce `ruststack` references.

---

## Production Checklist

- [ ] Set `SESSION_SECRET` to a random 64-character string
- [ ] Set `COOKIE_SECURE=true` (requires HTTPS)
- [ ] Switch `DATABASE_URL` to a Postgres connection string
- [ ] Set `FRONTEND_URL` to the production frontend domain
- [ ] Set `RUST_LOG=warn` or `error`
