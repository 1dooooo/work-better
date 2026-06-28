/**
 * MenuBarHeader — 菜单栏头部组件
 *
 * 显示应用名称、系统状态指示器和未处理数
 */

import type { SystemStatus } from "@/lib/tauri";
import { Zap } from "lucide-react";
import { StatusIndicator } from "./StatusIndicator";

// ─── 类型定义 ─────────────────────────────────────────────────

interface MenuBarHeaderProps {
  /** 系统状态 */
  systemStatus: SystemStatus | null;
  /** 未处理事件数 */
  unprocessedCount: number;
}

// ─── 组件 ─────────────────────────────────────────────────────

export function MenuBarHeader({
  systemStatus,
  unprocessedCount,
}: MenuBarHeaderProps) {
  return (
    <header className="flex items-center justify-between px-3.5 py-2 border-b border-[var(--color-glass-border)]">
      {/* 左侧：应用图标和名称 */}
      <div className="flex items-center gap-2">
        <div className="flex h-5 w-5 items-center justify-center rounded-md bg-[var(--color-glass-accent)]/20">
          <Zap className="h-3 w-3 text-[var(--color-glass-accent)]" strokeWidth={2.2} />
        </div>
        <span className="text-[12px] font-semibold text-foreground/90 tracking-tight">
          Work Better
        </span>
      </div>

      {/* 右侧：状态和计数 */}
      <div className="flex items-center gap-2">
        {systemStatus && <StatusIndicator status={systemStatus} />}

        {/* 未处理数 */}
        {unprocessedCount > 0 && (
          <span className="flex h-4 min-w-4 items-center justify-center rounded-full bg-[var(--color-glass-accent)] px-1 text-[9px] font-semibold text-white tabular-nums">
            {unprocessedCount}
          </span>
        )}
      </div>
    </header>
  );
}
