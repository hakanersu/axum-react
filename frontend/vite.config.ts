import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// Vite config - think of it like a webpack.config.js but much simpler.
// Vite uses ES modules natively in dev (no bundling needed = instant startup),
// and Rollup for production builds (optimized output).
export default defineConfig({
  plugins: [react()],
  resolve: {
    // This lets us use `@/components/Button` instead of `../../components/Button`.
    // The `@` maps to the `src/` directory. This is a common convention
    // in React projects, especially with shadcn/ui.
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 5173,
    // Proxy API requests to our Rust backend during development.
    // When the frontend calls `/api/auth/login`, Vite forwards it to localhost:3000.
    // This avoids CORS issues in development and mimics production setup.
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
});
