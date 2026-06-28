import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/ts/e2e",
  timeout: 60000, // Tauri 启动需要更多时间
  projects: [
    {
      name: "chromium",
      use: {
        browserName: "chromium",
      },
    },
  ],
  use: {
    baseURL: "http://localhost:1420", // Tauri dev server 默认端口
  },
  webServer: {
    command: "source ~/.cargo/env && cargo tauri dev", // 启动真实 Tauri app
    port: 1420,
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // Tauri 编译需要较长时间
  },
});
