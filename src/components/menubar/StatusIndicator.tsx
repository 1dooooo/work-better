/**
 * StatusIndicator — 系统状态指示器组件
 *
 * 显示采集器健康状态、调度状态和今日处理数
 */

import type { SystemStatus } from "@/lib/tauri";
import { cn } from "@/lib/utils";

// ─── 类型定义 ─────────────────────────────────────────────────

interface StatusIndicatorProps {
  /** 系统状态 */
  status: SystemStatus;
}

// ─── 组件 ─────────────────────────────────────────────────────

export function StatusIndicator({ status }: StatusIndicatorProps) {
  return (
    <div className="flex items-center gap-1.5">
      {/* 采集器健康状态 */}
      <div className="flex items-center gap-1">
        <span
          className={cn(
            "h-1.5 w-1.5 rounded-full",
            status.collectors_healthy > 0
              ? "bg-[var(--color-success)]"
              : "bg-[var(--color-muted-foreground)]",
          )}
        />
        <span className="text-[9px] text-[var(--color-text-hint)] tabular-nums">
          {status.collectors_healthy}/{status.collectors_total}
        </span>
      </div>

      {/* 分隔符 */}
      <span className="text-[9px] text-[var(--color-glass-muted)]">·</span>

      {/* 调度状态 */}
      <span className="text-[9px] text-[var(--color-text-hint)]">
        {status.scheduler_running ? "运行中" : "已暂停"}
      </span>

      {/* 今日处理数 */}
      {status.today_processed_count > 0 && (
        <>
          <span className="text-[9px] text-[var(--color-glass-muted)]">·</span>
          <span className="text-[9px] text-[var(--color-text-hint)] tabular-nums">
            今日 {status.today_processed_count}
          </span>
        </>
      )}
    </div>
  );
}
