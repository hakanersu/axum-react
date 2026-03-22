/**
 * TypeScript types that mirror the backend Rust models.
 *
 * These must stay in sync with the backend DTOs (Data Transfer Objects).
 * In a larger project, you'd auto-generate these from the Rust types
 * using a tool like `ts-rs` or `specta`.
 */

/** Matches backend's UserResponse struct */
export interface User {
  id: string;
  email: string;
  username: string;
  created_at: string;
  updated_at: string;
}

/** Matches backend's CreateUserDto struct */
export interface RegisterRequest {
  email: string;
  username: string;
  password: string;
}

/** Matches backend's LoginDto struct */
export interface LoginRequest {
  email: string;
  password: string;
}

/** Standard API error response shape from our backend */
export interface ApiError {
  error: {
    status: number;
    message: string;
  };
}

/** Generic API response wrapper */
export interface ApiResponse<T> {
  data?: T;
  error?: ApiError;
}
