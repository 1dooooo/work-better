import "@testing-library/jest-dom";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// 每个测试后自动清理 DOM，防止 singleFork 模式下的状态污染
afterEach(() => {
  cleanup();
});
