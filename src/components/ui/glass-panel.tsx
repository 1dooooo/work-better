/**
 * GlassPanel — 可复用的玻璃态容器组件
 *
 * 支持多种变体：tray（托盘）、sidebar（侧边栏）、card（卡片）、popover（弹窗）
 * 使用主题系统中的玻璃态 token，支持明暗模式切换
 */

import { forwardRef, type HTMLAttributes, type ReactNode } from "react";
import { cn } from "@/lib/utils";

// ─── 类型定义 ─────────────────────────────────────────────────

type GlassVariant = "tray" | "sidebar" | "card" | "popover";

interface GlassPanelProps extends HTMLAttributes<HTMLDivElement> {
  /** 玻璃态变体 */
  variant?: GlassVariant;
  /** 是否启用模糊效果 */
  blur?: boolean;
  /** 自定义背景色（覆盖 token） */
  bgClassName?: string;
  /** 自定义边框色（覆盖 token） */
  borderClassName?: string;
  /** 子元素 */
  children: ReactNode;
}

// ─── 变体样式映射 ─────────────────────────────────────────────

const variantStyles: Record<GlassVariant, string> = {
  tray: cn(
    "rounded-xl",
    "bg-[var(--color-glass-bg)] backdrop-blur-[var(--glass-blur)]",
    "border border-[var(--color-glass-border)]",
    "shadow-lg",
  ),
  sidebar: cn(
    "rounded-lg",
    "bg-[var(--color-glass-bg)] backdrop-blur-[var(--glass-blur)]",
    "border border-[var(--color-glass-border)]",
    "shadow-md",
  ),
  card: cn(
    "rounded-lg",
    "bg-[var(--color-glass-bg)] backdrop-blur-[var(--glass-blur)]",
    "border border-[var(--color-glass-border)]",
    "shadow-sm",
  ),
  popover: cn(
    "rounded-xl",
    "bg-[var(--color-glass-bg)] backdrop-blur-[var(--glass-blur)]",
    "border border-[var(--color-glass-border)]",
    "shadow-lg",
  ),
};

// ─── 组件 ─────────────────────────────────────────────────────

const GlassPanel = forwardRef<HTMLDivElement, GlassPanelProps>(
  (
    {
      variant = "card",
      blur = true,
      bgClassName,
      borderClassName,
      className,
      children,
      ...props
    },
    ref,
  ) => {
    return (
      <div
        ref={ref}
        className={cn(
          // 基础样式
          "text-foreground font-sans",
          // 变体样式
          variantStyles[variant],
          // 可选覆盖
          !blur && "backdrop-blur-none",
          bgClassName,
          borderClassName,
          className,
        )}
        {...props}
      >
        {children}
      </div>
    );
  },
);

GlassPanel.displayName = "GlassPanel";

// ─── 导出 ─────────────────────────────────────────────────────

export { GlassPanel, type GlassPanelProps, type GlassVariant };
