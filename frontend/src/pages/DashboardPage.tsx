import { useNavigate } from "react-router-dom";
import { useAuthStore } from "@/stores/auth-store";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

/**
 * Dashboard page - only accessible to authenticated users.
 *
 * The `ProtectedRoute` component (in App.tsx) ensures this page
 * never renders unless `user` is non-null.
 */
export function DashboardPage() {
  const { user, logout } = useAuthStore();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate("/login");
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Navigation bar */}
      <header className="border-b">
        <div className="container mx-auto flex items-center justify-between h-16 px-4">
          <h1 className="text-xl font-bold">RustStack</h1>
          <div className="flex items-center gap-4">
            <span className="text-sm text-muted-foreground">
              {user?.email}
            </span>
            <Button variant="outline" size="sm" onClick={handleLogout}>
              Sign out
            </Button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="container mx-auto p-4 mt-8">
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          <Card>
            <CardHeader>
              <CardTitle>Welcome, {user?.username}!</CardTitle>
              <CardDescription>
                You're logged in with session-based auth. No JWT tokens here!
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">User ID</span>
                  <span className="font-mono text-xs">{user?.id}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Email</span>
                  <span>{user?.email}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Joined</span>
                  <span>{user?.created_at}</span>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>CLI Generator</CardTitle>
              <CardDescription>
                Generate models and controllers from the terminal
              </CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                <code>{`# Generate a model
ruststack generate model post \\
  title:string content:text \\
  published:bool

# Generate a controller
ruststack generate controller post

# Scaffold both
ruststack g s post title:string \\
  content:text published:bool`}</code>
              </pre>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Stack Info</CardTitle>
              <CardDescription>
                Your full-stack framework overview
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2 text-sm">
                <li className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-orange-500 rounded-full" />
                  Rust + Axum backend
                </li>
                <li className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-blue-500 rounded-full" />
                  React + TypeScript frontend
                </li>
                <li className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-green-500 rounded-full" />
                  SQLite / PostgreSQL
                </li>
                <li className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-purple-500 rounded-full" />
                  Cookie-based sessions
                </li>
                <li className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-pink-500 rounded-full" />
                  shadcn/ui components
                </li>
              </ul>
            </CardContent>
          </Card>
        </div>
      </main>
    </div>
  );
}
