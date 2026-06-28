import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/ts/e2e",
  timeout: 60000,
  projects: [
    {
      name: "tauri",
      use: {
        browserName: "chromium",
      },
    },
  ],
  use: {
    baseURL: "http://localhost:1420",
  },
  webServer: {
    command: "source ~/.cargo/env && cargo tauri dev",
    port: 1420,
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
