/**
 * MenuBarActions — 菜单栏底部操作栏组件
 *
 * 显示快捷操作按钮：主窗口、速记、截图、处理
 */

import { cn } from "@/lib/utils";
import {
  Monitor,
  PenLine,
  Camera,
  Play,
} from "lucide-react";

// ─── 子组件：快捷操作按钮 ──────────────────────────────────────

function ActionButton({
  icon: Icon,
  label,
  onClick,
  disabled,
  spinning,
  accent,
}: {
  icon: typeof Monitor;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  spinning?: boolean;
  accent?: boolean;
}) {
  return (
    <button
      className={cn(
        "flex items-center gap-1.5 rounded-lg px-3 py-1.5 transition-all duration-150",
        "active:scale-[0.97] active:opacity-80",
        accent
          ? "bg-macos-blue/20 text-macos-blue hover:bg-macos-blue/30"
          : "text-muted-foreground hover:text-foreground hover:bg-accent",
        disabled && "opacity-40 cursor-not-allowed",
      )}
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
    >
      <Icon
        className={cn("h-3.5 w-3.5", spinning && "animate-spin")}
        strokeWidth={1.8}
      />
      <span className="text-[10px] font-medium leading-none">{label}</span>
    </button>
  );
}

// ─── 类型定义 ─────────────────────────────────────────────────

interface MenuBarActionsProps {
  /** 是否正在处理 */
  processing: boolean;
  /** 打开主窗口回调 */
  onOpenMainWindow: () => void;
  /** 打开速记窗口回调 */
  onOpenCapture: () => void;
  /** 截图回调 */
  onTakeScreenshot: () => void;
  /** 触发处理回调 */
  onTriggerProcess: () => void;
}

// ─── 组件 ─────────────────────────────────────────────────────

export function MenuBarActions({
  processing,
  onOpenMainWindow,
  onOpenCapture,
  onTakeScreenshot,
  onTriggerProcess,
}: MenuBarActionsProps) {
  return (
    <div className="flex items-center justify-between border-t border-border px-2 pt-1.5 pb-0.5" data-testid="menubar-actions">
      <div className="flex items-center gap-0.5">
        <ActionButton
          icon={Monitor}
          label="主窗口"
          onClick={onOpenMainWindow}
        />
        <ActionButton
          icon={PenLine}
          label="速记"
          onClick={onOpenCapture}
        />
        <ActionButton
          icon={Camera}
          label="截图"
          onClick={onTakeScreenshot}
        />
      </div>
      <ActionButton
        icon={Play}
        label={processing ? "处理中..." : "处理"}
        onClick={onTriggerProcess}
        disabled={processing}
        spinning={processing}
        accent
      />
    </div>
  );
}
