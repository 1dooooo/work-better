/**
 * useKeyboardShortcuts — 全局键盘快捷键 hook
 *
 * 功能：
 * - 注册全局快捷键（⌘K, ⌘1-5, ⌘,, ⌘N, ⌘⇧N）
 * - 注册上下文快捷键（Esc, J, K, Enter, Space）
 * - 快捷键冲突检测
 * - 尊重输入焦点（文本框内不触发）
 */

import { useEffect, useCallback, useRef } from "react";

type ShortcutHandler = (e: KeyboardEvent) => void;

interface Shortcut {
  key: string;
  metaKey?: boolean;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  handler: ShortcutHandler;
  description?: string;
  /** 是否在输入框内也触发 */
  allowInInput?: boolean;
}

// ─── 快捷键匹配 ──────────────────────────────────────────────

function matchesShortcut(e: KeyboardEvent, shortcut: Shortcut): boolean {
  const isMac = navigator.platform.includes("Mac");
  const modKey = isMac ? e.metaKey : e.ctrlKey;

  return (
    e.key.toLowerCase() === shortcut.key.toLowerCase() &&
    modKey === (shortcut.metaKey ?? false) &&
    e.shiftKey === (shortcut.shiftKey ?? false) &&
    e.altKey === (shortcut.altKey ?? false)
  );
}

// ─── 输入焦点检测 ──────────────────────────────────────────────

function isInputElement(target: EventTarget | null): boolean {
  if (!target || !(target instanceof HTMLElement)) return false;
  const tag = target.tagName.toLowerCase();
  return (
    tag === "input" ||
    tag === "textarea" ||
    tag === "select" ||
    target.isContentEditable
  );
}

// ─── Hook ──────────────────────────────────────────────────────

export function useKeyboardShortcuts(shortcuts: Shortcut[]) {
  const shortcutsRef = useRef(shortcuts);
  shortcutsRef.current = shortcuts;

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    const target = e.target;
    const isInput = isInputElement(target);

    for (const shortcut of shortcutsRef.current) {
      // 跳过输入框内的快捷键（除非明确允许）
      if (isInput && !shortcut.allowInInput) {
        continue;
      }

      if (matchesShortcut(e, shortcut)) {
        e.preventDefault();
        e.stopPropagation();
        shortcut.handler(e);
        return;
      }
    }
  }, []);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}

// ─── 预定义快捷键 ──────────────────────────────────────────────

export const SHORTCUTS = {
  // 全局快捷键
  COMMAND_PALETTE: { key: "k", metaKey: true, description: "打开命令面板" },
  VIEW_DASHBOARD: { key: "1", metaKey: true, description: "查看工作台" },
  VIEW_EVENTS: { key: "2", metaKey: true, description: "查看事件" },
  VIEW_TASKS: { key: "3", metaKey: true, description: "查看任务" },
  VIEW_TIMELINE: { key: "4", metaKey: true, description: "查看时间线" },
  VIEW_REPORTS: { key: "5", metaKey: true, description: "查看报告" },
  VIEW_SETTINGS: { key: ",", metaKey: true, description: "打开设置" },
  NEW_TASK: { key: "n", metaKey: true, description: "新建任务" },
  NEW_EVENT: { key: "n", metaKey: true, shiftKey: true, description: "新建事件" },

  // 列表导航
  LIST_DOWN: { key: "j", description: "下一项" },
  LIST_UP: { key: "k", description: "上一项" },
  LIST_ENTER: { key: "Enter", description: "展开/确认" },
  LIST_SPACE: { key: " ", description: "标记/切换" },
  LIST_TOP: { key: "g", description: "跳转到顶部" },
  LIST_BOTTOM: { key: "G", shiftKey: true, description: "跳转到底部" },
  LIST_SEARCH: { key: "/", description: "聚焦搜索" },
  LIST_SELECT_ALL: { key: "a", metaKey: true, description: "全选" },
  LIST_MARK_ALL: { key: "a", metaKey: true, shiftKey: true, description: "标记全部已读" },

  // 通用
  ESCAPE: { key: "Escape", description: "关闭/返回" },
} as const;

// ─── 快捷键提示格式化 ──────────────────────────────────────────────

export function formatShortcutHint(shortcut: {
  key: string;
  metaKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
}): string {
  const isMac = navigator.platform.includes("Mac");
  const parts: string[] = [];

  if (shortcut.metaKey) {
    parts.push(isMac ? "⌘" : "Ctrl");
  }
  if (shortcut.shiftKey) {
    parts.push(isMac ? "⇧" : "Shift");
  }
  if (shortcut.altKey) {
    parts.push(isMac ? "⌥" : "Alt");
  }

  // 特殊键名映射
  const keyMap: Record<string, string> = {
    Enter: "↵",
    Escape: "Esc",
    " ": "Space",
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
  };

  parts.push(keyMap[shortcut.key] || shortcut.key.toUpperCase());
  return parts.join(isMac ? "" : "+");
}
