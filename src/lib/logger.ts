/**
 * 前端日志工具
 *
 * 将前端日志输出到Tauri终端，实现日志统一观测
 * 支持日志级别：debug, info, warn, error
 */

import { invoke } from "@tauri-apps/api/core";

type LogLevel = "debug" | "info" | "warn" | "error";

interface LogOptions {
  /** 日志标签 */
  tag?: string;
  /** 日志级别 */
  level?: LogLevel;
}

/**
 * 输出日志到Tauri终端
 *
 * @param message 日志消息
 * @param options 日志选项
 */
export async function log(
  message: string,
  options: LogOptions = {},
): Promise<void> {
  const { tag = "Frontend", level = "info" } = options;

  try {
    await invoke("log_message", { level, message, tag });
  } catch {
    // 如果invoke失败，降级到console
    const consoleMethod =
      level === "error"
        ? console.error
        : level === "warn"
          ? console.warn
          : level === "debug"
            ? console.debug
            : console.log;
    consoleMethod(`[${tag}] ${message}`);
  }
}

/**
 * 创建带标签的日志器
 *
 * @param tag 日志标签
 * @returns 日志器对象
 */
export function createLogger(tag: string) {
  return {
    debug: (message: string) => log(message, { tag, level: "debug" }),
    info: (message: string) => log(message, { tag, level: "info" }),
    warn: (message: string) => log(message, { tag, level: "warn" }),
    error: (message: string) => log(message, { tag, level: "error" }),
  };
}
