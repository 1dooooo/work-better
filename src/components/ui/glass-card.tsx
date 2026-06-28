/**
 * GlassCard — 玻璃态卡片组件
 *
 * 用于事件、任务、通知等列表项
 * 支持明暗主题，使用共享的设计 token
 */

import { forwardRef, type HTMLAttributes, type ReactNode } from "react";
import { cn } from "@/lib/utils";

// ─── 类型定义 ─────────────────────────────────────────────────

type GlassCardVariant = "default" | "interactive" | "highlight";
type GlassCardSize = "sm" | "md" | "lg";

interface GlassCardProps extends HTMLAttributes<HTMLDivElement> {
  /** 卡片变体 */
  variant?: GlassCardVariant;
  /** 卡片尺寸 */
  size?: GlassCardSize;
  /** 左侧装饰色（用于状态指示） */
  accentColor?: string;
  /** 是否显示悬停效果 */
  hoverable?: boolean;
  /** 子元素 */
  children: ReactNode;
}

// ─── 变体样式映射 ─────────────────────────────────────────────

const variantStyles: Record<GlassCardVariant, string> = {
  default: "bg-[var(--color-glass-bg)]",
  interactive: cn(
    "bg-[var(--color-glass-bg)]",
    "hover:bg-[var(--color-glass-hover)]",
    "cursor-pointer transition-colors duration-150",
  ),
  highlight: cn(
    "bg-[var(--color-glass-hover)]",
    "border-l-2 border-l-[var(--color-glass-accent)]",
  ),
};

const sizeStyles: Record<GlassCardSize, string> = {
  sm: "px-2 py-1 gap-1.5",
  md: "px-3 py-1.5 gap-2",
  lg: "px-4 py-2 gap-2.5",
};

// ─── 组件 ─────────────────────────────────────────────────────

const GlassCard = forwardRef<HTMLDivElement, GlassCardProps>(
  (
    {
      variant = "default",
      size = "md",
      accentColor,
      hoverable = false,
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
          "flex items-center rounded-lg",
          "border border-[var(--color-glass-border)]",
          "transition-colors duration-150",
          // 变体样式
          variantStyles[variant],
          // 尺寸样式
          sizeStyles[size],
          // 可选悬停效果
          hoverable && "hover:bg-[var(--color-glass-hover)] cursor-pointer",
          className,
        )}
        style={
          accentColor
            ? { borderLeftColor: accentColor, borderLeftWidth: "2px" }
            : undefined
        }
        {...props}
      >
        {children}
      </div>
    );
  },
);

GlassCard.displayName = "GlassCard";

// ─── 导出 ─────────────────────────────────────────────────────

export { GlassCard, type GlassCardProps, type GlassCardVariant, type GlassCardSize };
