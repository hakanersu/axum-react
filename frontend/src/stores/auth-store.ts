import { create } from "zustand";
import type { User } from "@/types";
import { authApi, ApiRequestError } from "@/lib/api";

/**
 * Shape of our auth store.
 *
 * In Zustand, the store is defined as a single interface containing
 * both STATE (data) and ACTIONS (functions that modify the state).
 * This is different from Redux where actions and reducers are separate.
 */
interface AuthState {
  // State
  user: User | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  register: (email: string, username: string, password: string) => Promise<void>;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => Promise<void>;
  clearError: () => void;
}

/**
 * Create the auth store.
 *
 * `create<AuthState>()` returns a React hook (`useAuthStore`).
 * The function receives `set` and `get`:
 * - `set(partial)`: Merges partial state into current state (like Object.assign)
 * - `get()`: Returns current state (useful inside actions)
 *
 * Unlike Redux, there's no boilerplate — no action types, no dispatching,
 * no reducers, no providers. Just define state and functions.
 */
export const useAuthStore = create<AuthState>((set) => ({
  // Initial state
  user: null,
  isLoading: true, // Start as true so we show a loading state while checking session
  error: null,

  /**
   * Register a new user.
   * On success, the backend sets a session cookie AND returns the user data.
   * We store the user in our Zustand state for immediate use.
   */
  register: async (email, username, password) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authApi.register({ email, username, password });
      set({ user: response.user, isLoading: false });
    } catch (err) {
      const message =
        err instanceof ApiRequestError
          ? err.message
          : "Registration failed";
      set({ error: message, isLoading: false });
      throw err; // Re-throw so the component can handle it too
    }
  },

  /**
   * Log in with email/password.
   * The backend validates credentials, creates a session, and sets a cookie.
   */
  login: async (email, password) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authApi.login({ email, password });
      set({ user: response.user, isLoading: false });
    } catch (err) {
      const message =
        err instanceof ApiRequestError
          ? err.message
          : "Login failed";
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  /**
   * Log out - tells the backend to destroy the session.
   * The backend clears the session from the database and invalidates the cookie.
   */
  logout: async () => {
    try {
      await authApi.logout();
    } finally {
      // Always clear local state, even if the API call fails
      set({ user: null, isLoading: false });
    }
  },

  /**
   * Check if the user has an existing session.
   *
   * Called on app startup. The browser automatically sends the session cookie
   * (if one exists) with the request. If the session is valid, the backend
   * returns the user data. If not, we get a 401 and set user to null.
   *
   * This is how "remember me" works with cookie-based auth — the cookie
   * persists across browser restarts, so the session survives.
   */
  checkAuth: async () => {
    set({ isLoading: true });
    try {
      const response = await authApi.me();
      set({ user: response.user, isLoading: false });
    } catch {
      // 401 means no valid session — that's expected, not an error
      set({ user: null, isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
