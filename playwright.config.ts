import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/ts/e2e",
  timeout: 60000,
  expect: {
    timeout: 10000,
  },
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
    // 自动收集失败测试的截图和视频
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "source ~/.cargo/env && cargo tauri dev",
    port: 1420,
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
  // 报告器配置
  reporter: [
    ["html", { open: "never" }],
    ["list"],
  ],
});
