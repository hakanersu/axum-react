import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./globals.css";

/**
 * React 18+ entry point using createRoot.
 *
 * `document.getElementById("root")!` — the `!` is TypeScript's
 * non-null assertion. We know the element exists (it's in index.html),
 * so we tell TypeScript "trust me, this isn't null".
 *
 * `<React.StrictMode>` enables extra development checks:
 * - Components render twice to catch side effects
 * - Deprecated APIs trigger warnings
 * - This has NO effect in production builds
 */
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
