import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./tests/ts/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}", "tests/ts/**/*.test.{ts,tsx}"],
    reporters: ["json", "default"],
    outputFile: "./test-results/vitest-results.json",
  },
});
