import type { ApiError } from "@/types";

/**
 * Base URL for API requests.
 *
 * Empty string means "same origin" — requests go to the same host.
 * In development, Vite's proxy (configured in vite.config.ts) intercepts
 * `/api/*` requests and forwards them to the Rust backend at localhost:3000.
 * In production, both frontend and API would be served from the same domain.
 */
const BASE_URL = "";

/**
 * Custom error class for API errors.
 *
 * Extends the built-in Error class to include the status code and
 * structured error data from our backend.
 */
export class ApiRequestError extends Error {
  status: number;
  data: ApiError;

  constructor(status: number, data: ApiError) {
    super(data.error?.message || "An error occurred");
    this.status = status;
    this.data = data;
  }
}

/**
 * Generic fetch wrapper with proper error handling and cookie support.
 *
 * The generic type `T` represents the expected response shape.
 * For example: `apiFetch<{ user: User }>("/api/auth/me")` tells TypeScript
 * that the response will have a `user` property of type `User`.
 *
 * `credentials: "include"` is CRITICAL — without it, the browser won't
 * send our session cookie with the request, and the backend won't know
 * who we are. This is the key difference from JWT auth where you'd
 * manually attach an Authorization header.
 */
async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const response = await fetch(`${BASE_URL}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
    // This tells the browser to include cookies in cross-origin requests.
    // For session-based auth, this is essential — it's what sends our
    // session ID cookie to the backend on every request.
    credentials: "include",
  });

  // Handle non-OK responses (status 400+)
  if (!response.ok) {
    const errorData = (await response.json()) as ApiError;
    throw new ApiRequestError(response.status, errorData);
  }

  return response.json() as Promise<T>;
}

/**
 * Auth API methods.
 *
 * Each method returns a typed Promise. The type matches what our
 * Rust backend returns (defined in the controller's Json(json!({...}))).
 */
export const authApi = {
  register: (data: { email: string; username: string; password: string }) =>
    apiFetch<{ user: import("@/types").User; message: string }>(
      "/api/auth/register",
      {
        method: "POST",
        body: JSON.stringify(data),
      }
    ),

  login: (data: { email: string; password: string }) =>
    apiFetch<{ user: import("@/types").User; message: string }>(
      "/api/auth/login",
      {
        method: "POST",
        body: JSON.stringify(data),
      }
    ),

  logout: () =>
    apiFetch<{ message: string }>("/api/auth/logout", {
      method: "POST",
    }),

  me: () =>
    apiFetch<{ user: import("@/types").User }>("/api/auth/me"),
};
