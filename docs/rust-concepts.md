# Core Rust Concepts

Since you're learning Rust, here are some key concepts we've used in this project and how they relate to other languages.

## 🧵 1. `Arc<T>` (Atomic Reference Counting)

In JavaScript, objects are garbage-collected and single-threaded, so sharing data is easy. In Rust, we have to be explicit about how multiple threads can own and access the same data.

*   `Arc<AppState>` is like a smart pointer that keeps track of how many things are using `AppState`.
*   When you clone an `Arc`, you're not cloning the `AppState` itself—just the pointer!
*   When the last `Arc` is dropped, the `AppState` is automatically cleaned up.

## 🧬 2. Traits (Interfaces on Steroids)

Traits are like Interfaces in TypeScript, but much more powerful. We use them for:

*   **`IntoResponse`**: We implemented this for our `AppError`. This tells Axum: "Whenever you see an `AppError`, use this logic to turn it into an HTTP response."
*   **`FromRequestParts`**: This is how we made the `AuthUser` extractor. It allows us to simply add `auth: AuthUser` to any controller function to enforce login!
*   **`Serialize`/`Deserialize`**: These traits from `serde` allow our structs to be converted to and from JSON.

## ⚠️ 3. Error Handling: `Result<T, E>`

Rust doesn't have `try/catch` or `throw`. Instead, functions return a `Result` enum which can be either `Ok(value)` or `Err(error)`.

The `?` operator is a shortcut:
*   If the result is `Ok`, it unwraps the value.
*   If it's `Err`, it immediately returns that error from the current function.

In this project, we created a custom `AppError` enum to centralize all our error types (database errors, validation errors, etc.) and convert them to consistent HTTP responses.

## ⚡ 4. Async/Await

Like in JavaScript, Rust uses `async` and `await` for non-blocking I/O.

*   **Difference:** Rust's futures are **lazy**. They don't do anything until you `.await` them. In JS, a Promise starts running as soon as it's created.
*   **Runtime:** Rust's standard library doesn't include an async runtime. We use `Tokio` for this.
