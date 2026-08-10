import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// Resolve the `@/*` path alias (tsconfig paths) so Vitest can import app modules
// the same way Next does.
export default defineConfig({
  // Use the automatic JSX runtime so components render without an explicit
  // `import React` (matches Next's compiler).
  esbuild: {
    jsx: "automatic",
    jsxImportSource: "react",
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
});
