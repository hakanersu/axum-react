import { useEffect } from "react";
import {
  BrowserRouter,
  Routes,
  Route,
  Navigate,
  Outlet,
} from "react-router-dom";
import { useAuthStore } from "@/stores/auth-store";
import { LoginPage } from "@/pages/LoginPage";
import { RegisterPage } from "@/pages/RegisterPage";
import { DashboardPage } from "@/pages/DashboardPage";

/**
 * Protected Route wrapper component.
 *
 * This is a common React pattern for auth-gated routes.
 * It checks if the user is authenticated:
 * - If yes: renders the child routes via `<Outlet />`
 * - If no: redirects to the login page
 * - While checking: shows a loading spinner
 *
 * `<Outlet />` is a react-router concept — it renders whichever
 * child <Route> matches the current URL.
 */
function ProtectedRoute() {
  const { user, isLoading } = useAuthStore();

  // While checking if the user has a valid session, show loading
  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>
    );
  }

  // Not logged in — redirect to login page.
  // `replace` means this redirect won't be added to browser history,
  // so clicking "back" won't go to the protected page again.
  if (!user) {
    return <Navigate to="/login" replace />;
  }

  // User is authenticated — render the child routes
  return <Outlet />;
}

/**
 * Guest Route wrapper — only accessible when NOT logged in.
 * If the user is already logged in, redirect to dashboard.
 */
function GuestRoute() {
  const { user, isLoading } = useAuthStore();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>
    );
  }

  if (user) {
    return <Navigate to="/dashboard" replace />;
  }

  return <Outlet />;
}

/**
 * Main App component.
 *
 * `useEffect` with an empty dependency array `[]` runs exactly once
 * when the component first mounts. This is where we check if the user
 * has an existing session (the browser sends the session cookie automatically).
 *
 * The route structure:
 * - `/login` and `/register` are guest-only (redirect if already logged in)
 * - `/dashboard` is protected (redirect to login if not authenticated)
 * - `/` redirects to dashboard
 */
export default function App() {
  const { checkAuth } = useAuthStore();

  // On app startup, check if the user has a valid session.
  // This is what makes "remember me" work — the session cookie
  // persists across browser restarts (set by the Rust backend).
  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  return (
    <BrowserRouter>
      <Routes>
        {/* Guest-only routes (login/register) */}
        <Route element={<GuestRoute />}>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />
        </Route>

        {/* Protected routes (require auth) */}
        <Route element={<ProtectedRoute />}>
          <Route path="/dashboard" element={<DashboardPage />} />
        </Route>

        {/* Default redirect */}
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
