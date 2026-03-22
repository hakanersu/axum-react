# RustStack Project Documentation

Welcome! This documentation is designed to help you understand how the **RustStack** project works, specifically the Rust backend. Since you're learning Rust, these guides will explain not just *what* the code does, but *why* it's written that way, often comparing it to patterns you might know from JavaScript/TypeScript.

## 📚 Table of Contents

1.  **[Architecture Overview](./architecture.md)**
    *   Learn how the pieces (Axum, SQLx, Tower-sessions) fit together.
2.  **[Backend Walkthrough](./backend-walkthrough.md)**
    *   A step-by-step guide through the codebase, from `main.rs` to the database.
3.  **[Core Rust Concepts](./rust-concepts.md)**
    *   Deep dive into the Rust features used here: `Arc`, `Traits`, `Async`, and `Error Handling`.
4.  **[Database & Authentication](./db-and-auth.md)**
    *   How we manage data and keep users logged in using session cookies.

## 🚀 Getting Started

To run the backend:

1.  Navigate to the `backend/` folder.
2.  Install dependencies and build: `cargo build`.
3.  Run the migrations and seeds: The app does this automatically on startup.
4.  Start the server: `cargo run`.

The server will start on `http://0.0.0.0:3000` by default.
