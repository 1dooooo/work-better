import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/ts/e2e",
  timeout: 120000, // Tauri 启动需要更多时间
  projects: [
    {
      name: "tauri",
      use: {
        // macOS 使用 Safari WebDriver
        browserName: "webkit",
        // 连接到系统 WebDriver
        connectOptions: {
          wsEndpoint: "http://localhost:4444/session",
        },
      },
    },
  ],
  use: {
    // Tauri WebView 的基础 URL
    baseURL: "http://localhost:1420",
  },
  // 启动 Tauri app
  webServer: [
    {
      // 启动 Tauri 开发服务器
      command: "source ~/.cargo/env && cargo tauri dev",
      port: 1420,
      reuseExistingServer: !process.env.CI,
      timeout: 120000,
    },
  ],
});
