# Database & Authentication

This guide explains how we handle data and keep users logged in.

## 🗄️ 1. Database with SQLx

The `db/mod.rs` file handles everything database-related.

### 🏊 The Connection Pool

Creating a new connection for every request is expensive. Instead, we use a **Connection Pool** (`AnyPool`). It keeps a set of open connections and hands them out to requests as needed.

### 📜 Automatic Migrations

When the app starts, it checks the database and runs migrations if necessary. This creates the `users` and `tower_sessions` tables if they don't exist. This is done with raw SQL to ensure it works on both SQLite and Postgres.

## 🔐 2. Cookie-based Authentication

Instead of JWTs (JSON Web Tokens), this project uses **Session Cookies**.

### 🍪 How it works:

1.  **Login/Register:**
    *   The user sends their credentials to the `/api/auth/login` or `/api/auth/register` endpoint.
    *   The backend validates the credentials and hashes the password with Argon2.
    *   If correct, a new "session" is created in the database (`tower_sessions` table).
    *   A unique session ID is sent to the browser in a `Set-Cookie` header.

2.  **Every subsequent request:**
    *   The browser automatically sends the session ID cookie back to the backend.
    *   The `tower-sessions` middleware reads the cookie and fetches the session data from the database.
    *   Our `AuthUser` extractor (in `middleware/auth.rs`) then finds the "user" key in that session.

3.  **Logout:**
    *   The user calls `/api/auth/logout`.
    *   The backend clears the session from the database and tells the browser to delete the cookie.

### 🛡️ Why use sessions?

*   **Security:** If a user's account is compromised, you can instantly revoke their session by deleting it from the database.
*   **Convenience:** The browser handles the cookie automatically, so you don't need to write manual logic to store tokens in `localStorage`.
*   **Privacy:** Sensitive data (like user IDs or permissions) is stored on the server, not in the token itself.
