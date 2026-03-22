import { useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useAuthStore } from "@/stores/auth-store";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

/**
 * Login page component.
 *
 * This demonstrates several important React patterns:
 * 1. Controlled inputs: Input values are stored in state and updated via onChange
 * 2. Form submission with preventDefault (no page reload)
 * 3. Global state management with Zustand
 * 4. Programmatic navigation after successful action
 * 5. Loading states to prevent double-submission
 */
export function LoginPage() {
  // Local state for form inputs.
  // These are "controlled components" - React state is the source of truth,
  // not the DOM. The input's value is always what's in state.
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  // Pull actions and state from our Zustand store.
  // Zustand uses selector functions to subscribe to specific state slices.
  // This component only re-renders when these specific values change.
  const { login, isLoading, error, clearError } = useAuthStore();

  // react-router's navigation hook - like window.location but for SPA routing
  const navigate = useNavigate();

  /**
   * Handle form submission.
   *
   * `FormEvent` is the TypeScript type for form submit events.
   * `preventDefault()` stops the browser's default behavior of
   * sending a GET/POST request and reloading the page.
   */
  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    try {
      await login(email, password);
      // If login succeeds (no error thrown), redirect to dashboard
      navigate("/dashboard");
    } catch {
      // Error is already set in the store by the login action.
      // We catch here just to prevent an unhandled promise rejection.
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl font-bold">Welcome back</CardTitle>
          <CardDescription>
            Enter your credentials to access your account
          </CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-4">
            {/* Error message - only renders when error is truthy */}
            {error && (
              <div className="rounded-md bg-destructive/15 p-3 text-sm text-destructive">
                {error}
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="you@example.com"
                value={email}
                onChange={(e) => {
                  setEmail(e.target.value);
                  clearError(); // Clear any previous error when user types
                }}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                placeholder="••••••••"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  clearError();
                }}
                required
              />
            </div>
          </CardContent>
          <CardFooter className="flex flex-col space-y-4">
            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading ? "Signing in..." : "Sign in"}
            </Button>
            <p className="text-sm text-muted-foreground text-center">
              Don't have an account?{" "}
              <Link
                to="/register"
                className="text-primary underline-offset-4 hover:underline"
              >
                Sign up
              </Link>
            </p>
          </CardFooter>
        </form>
      </Card>
    </div>
  );
}
