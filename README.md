# RustStack

A full-stack web framework using **Rust + Axum** (backend) and **React + TypeScript + shadcn/ui** (frontend), with cookie-based session authentication and multi-database support.

## Architecture Overview

```
ruststack/
├── backend/                  # Rust Axum server
│   ├── src/
│   │   ├── main.rs           # Entry point: server setup, middleware, startup
│   │   ├── config/           # Environment-based configuration
│   │   ├── controllers/      # Request handlers (like Express route handlers)
│   │   ├── models/           # Data structures + DTOs (like TypeScript interfaces)
│   │   ├── middleware/       # Auth middleware (session extraction)
│   │   ├── db/               # Database connection pool + migrations
│   │   ├── errors/           # Custom error types → HTTP responses
│   │   └── routes/           # URL → handler mapping
│   └── migrations/           # SQL migration files
├── cli/                      # Code generator CLI tool
│   └── src/main.rs           # Generates models + controllers
├── frontend/                 # React SPA
│   ├── src/
│   │   ├── App.tsx           # Routing + auth guards
│   │   ├── components/ui/    # shadcn/ui components
│   │   ├── pages/            # Page components
│   │   ├── stores/           # Zustand state management
│   │   ├── lib/              # API client + utilities
│   │   └── types/            # TypeScript type definitions
│   └── index.html            # Vite entry HTML
├── Cargo.toml                # Rust workspace manifest
└── .env                      # Environment variables
```

## Prerequisites

- **Rust** (1.75+): Install from https://rustup.rs
- **Node.js** (18+): Install from https://nodejs.org
- **SQLite** (default) or **PostgreSQL** (optional)

## Quick Start

### 1. Clone and setup

```bash
cd ruststack

# Install frontend dependencies
cd frontend && npm install && cd ..
```

### 2. Configure environment

Edit `.env` to customize settings. The defaults work out of the box with SQLite:

```env
# SQLite (zero setup needed)
DATABASE_URL=sqlite:data.db?mode=rwc

# PostgreSQL (requires running PostgreSQL server)
# DATABASE_URL=postgres://user:password@localhost:5432/ruststack
```

### 3. Start the backend

```bash
# From the project root
cargo run -p ruststack-backend

# For PostgreSQL, enable the postgres feature:
# cargo run -p ruststack-backend --no-default-features --features postgres
```

The server starts at `http://localhost:3000`.

### 4. Start the frontend

```bash
cd frontend
npm run dev
```

The React app starts at `http://localhost:5173`.

### 5. Open your browser

Navigate to `http://localhost:5173` — you'll see the login page.

## Authentication Flow

This framework uses **secure cookie-based sessions** instead of JWT. Here's how it works:

```
┌──────────┐       POST /api/auth/login        ┌──────────┐
│          │ ──────────────────────────────────▶ │          │
│  Browser │       { email, password }           │  Server  │
│          │ ◀────────────────────────────────── │          │
│          │   Set-Cookie: session_id=abc123     │          │
│          │   { user: { id, email, ... } }      │          │
└──────────┘                                     └──────────┘

┌──────────┐       GET /api/auth/me             ┌──────────┐
│          │ ──────────────────────────────────▶ │          │
│  Browser │   Cookie: session_id=abc123         │  Server  │
│          │ ◀────────────────────────────────── │  (looks  │
│          │   { user: { id, email, ... } }      │  up DB)  │
└──────────┘                                     └──────────┘
```

**Why cookies over JWT?**

| Feature | Cookies (this framework) | JWT |
|---------|-------------------------|-----|
| Revocation | Instant (delete from DB) | Must wait for expiry |
| Data exposure | Session ID only (opaque) | User data visible in token |
| Storage | Server-side (database) | Client-side (localStorage) |
| XSS risk | HttpOnly flag prevents JS access | Token in localStorage is vulnerable |
| Size | Tiny cookie (~50 bytes) | Large token (~500+ bytes) |

## CLI Code Generator

Build and install the CLI tool:

```bash
cargo build -p ruststack-cli
# The binary is at: target/debug/ruststack
```

### Generate a Model

Creates a Rust struct, DTOs, and SQL migration:

```bash
ruststack generate model post title:string content:text published:bool

# Shorthand:
ruststack g m post title:string content:text published:bool
```

**Generated files:**
- `backend/src/models/post.rs` — Struct, CreateDto, UpdateDto, Response type
- `backend/migrations/create_posts.sql` — CREATE TABLE statement

**Available field types:**
| CLI Type | Rust Type | SQL Type |
|----------|-----------|----------|
| `string` | `String` | `TEXT NOT NULL` |
| `text` | `String` | `TEXT NOT NULL` |
| `int` / `integer` | `i64` | `INTEGER NOT NULL` |
| `float` / `double` | `f64` | `REAL NOT NULL` |
| `bool` / `boolean` | `bool` | `BOOLEAN NOT NULL DEFAULT FALSE` |
| `date` / `datetime` | `String` | `TEXT NOT NULL` |
| `uuid` | `String` | `TEXT NOT NULL` |

### Generate a Controller

Creates an Axum handler file with CRUD operations:

```bash
ruststack generate controller post
# Or specific actions only:
ruststack generate controller post index show create

# Shorthand:
ruststack g c post
```

**Generated actions:** `index`, `show`, `create`, `update`, `delete`

### Scaffold (Model + Controller)

Generate both at once:

```bash
ruststack generate scaffold post title:string content:text published:bool

# Shorthand:
ruststack g s post title:string content:text published:bool
```

### After Generating

1. **Register the model** in `backend/src/models/mod.rs`:
   ```rust
   pub mod post;
   pub use post::*;
   ```

2. **Register the controller** in `backend/src/controllers/mod.rs`:
   ```rust
   pub mod post_controller;
   ```

3. **Add routes** in `backend/src/routes/mod.rs`:
   ```rust
   use crate::controllers::post_controller;

   // Inside build_routes():
   .nest("/api", post_controller::routes(state.clone()))
   ```

4. **Run the migration** (add to `db/mod.rs` `run_migrations()` or use a migration tool)

## API Endpoints

### Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/auth/register` | No | Create new account |
| `POST` | `/api/auth/login` | No | Log in (sets session cookie) |
| `POST` | `/api/auth/logout` | Yes | Log out (destroys session) |
| `GET` | `/api/auth/me` | Yes | Get current user info |
| `GET` | `/api/health` | No | Health check |

### Example requests

```bash
# Register
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","username":"testuser","password":"password123"}' \
  -c cookies.txt

# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}' \
  -c cookies.txt

# Get current user (uses saved cookie)
curl http://localhost:3000/api/auth/me -b cookies.txt

# Logout
curl -X POST http://localhost:3000/api/auth/logout -b cookies.txt
```

## Switching Databases

### SQLite (default)

Zero configuration. The database file is created automatically:

```env
DATABASE_URL=sqlite:data.db?mode=rwc
```

```bash
cargo run -p ruststack-backend
```

### PostgreSQL

1. Create a database:
   ```sql
   CREATE DATABASE ruststack;
   ```

2. Update `.env`:
   ```env
   DATABASE_URL=postgres://user:password@localhost:5432/ruststack
   ```

3. Run with postgres feature:
   ```bash
   cargo run -p ruststack-backend --no-default-features --features postgres
   ```

## Key Concepts for Learners

### Rust Concepts Used

- **Ownership & Borrowing**: `&self`, `&str` vs `String`, `Arc<T>` for shared ownership
- **Error Handling**: `Result<T, E>`, `?` operator, custom error types with `thiserror`
- **Traits**: `IntoResponse`, `FromRequestParts`, `From<T>`, `Serialize/Deserialize`
- **Async/Await**: `async fn`, `.await`, `tokio` runtime
- **Derive Macros**: `#[derive(Debug, Clone, Serialize)]` auto-generates implementations
- **Pattern Matching**: `match` expressions for exhaustive error handling
- **Modules**: `mod`, `pub`, `use`, `pub use` for re-exports

### React Concepts Used

- **Hooks**: `useState`, `useEffect`, `useNavigate`
- **Component Composition**: Compound components (Card + CardHeader + CardContent)
- **State Management**: Zustand store with actions
- **Protected Routes**: Auth guards using route wrappers
- **Controlled Components**: Input values managed by React state
- **TypeScript Generics**: `apiFetch<T>()` for typed API responses

## Production Checklist

- [ ] Change `SESSION_SECRET` to a random 64-character string
- [ ] Set `COOKIE_SECURE=true` (requires HTTPS)
- [ ] Set `COOKIE_DOMAIN` to your production domain
- [ ] Update `FRONTEND_URL` for CORS
- [ ] Switch to PostgreSQL for production workloads
- [ ] Add rate limiting middleware
- [ ] Set up HTTPS (via reverse proxy like nginx or Caddy)
- [ ] Run `npm run build` and serve frontend as static files
- [ ] Compile backend with `cargo build --release`

## License

MIT
