/**
 * useListNavigation — vim 风格列表导航 hook
 *
 * 功能：
 * - J/K 上下移动焦点
 * - Enter 展开/折叠
 * - Space 标记/切换
 * - gg/G 跳转到顶/底
 * - / 聚焦搜索框
 * - 焦点项自动滚动到可见区域
 */

import { useState, useCallback, useRef, useEffect } from "react";

interface UseListNavigationOptions {
  /** 列表项数量 */
  itemCount: number;
  /** 是否启用 vim 导航 */
  enabled?: boolean;
  /** Enter 回调 */
  onEnter?: (index: number) => void;
  /** Space 回调 */
  onSpace?: (index: number) => void;
  /** 搜索框选择器 */
  searchSelector?: string;
  /** 焦点变化回调 */
  onFocusChange?: (index: number) => void;
}

export function useListNavigation({
  itemCount,
  enabled = true,
  onEnter,
  onSpace,
  searchSelector = 'input[type="search"], input[placeholder*="搜索"]',
  onFocusChange,
}: UseListNavigationOptions) {
  const [focusedIndex, setFocusedIndex] = useState<number>(-1);
  const [isNavigating, setIsNavigating] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);
  const gPressedRef = useRef(false);

  // 焦点变化时滚动到可见区域
  useEffect(() => {
    if (focusedIndex < 0 || !listRef.current) return;

    const items = listRef.current.querySelectorAll("[data-list-item]");
    const focusedItem = items[focusedIndex];
    if (focusedItem) {
      focusedItem.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }

    onFocusChange?.(focusedIndex);
  }, [focusedIndex, onFocusChange]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!enabled || itemCount === 0) return;

      const target = e.target;
      const isInput =
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement;

      // 在输入框内不处理（除非是 Escape）
      if (isInput && e.key !== "Escape") return;

      switch (e.key) {
        case "j":
        case "J":
          if (!isInput) {
            e.preventDefault();
            setIsNavigating(true);
            setFocusedIndex((prev) => Math.min(prev + 1, itemCount - 1));
          }
          break;

        case "k":
        case "K":
          if (!isInput) {
            e.preventDefault();
            setIsNavigating(true);
            setFocusedIndex((prev) => Math.max(prev - 1, 0));
          }
          break;

        case "Enter":
          if (!isInput && focusedIndex >= 0) {
            e.preventDefault();
            onEnter?.(focusedIndex);
          }
          break;

        case " ":
          if (!isInput && focusedIndex >= 0) {
            e.preventDefault();
            onSpace?.(focusedIndex);
          }
          break;

        case "g":
          if (!isInput) {
            if (gPressedRef.current) {
              // gg - 跳转到顶部
              e.preventDefault();
              setFocusedIndex(0);
              gPressedRef.current = false;
            } else {
              gPressedRef.current = true;
              // 500ms 后重置
              setTimeout(() => {
                gPressedRef.current = false;
              }, 500);
            }
          }
          break;

        case "G":
          if (!isInput) {
            e.preventDefault();
            setFocusedIndex(itemCount - 1);
          }
          break;

        case "/":
          if (!isInput) {
            e.preventDefault();
            const searchInput = document.querySelector(searchSelector);
            if (searchInput instanceof HTMLInputElement) {
              searchInput.focus();
            }
          }
          break;

        case "Escape":
          if (isNavigating) {
            e.preventDefault();
            setIsNavigating(false);
            setFocusedIndex(-1);
          }
          break;
      }
    },
    [enabled, itemCount, focusedIndex, onEnter, onSpace, searchSelector, isNavigating]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return {
    focusedIndex,
    isNavigating,
    listRef,
    setFocusedIndex,
    setIsNavigating,
  };
}
