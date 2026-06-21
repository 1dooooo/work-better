/**
 * CommandPalette — 命令面板组件
 *
 * 集成 Variant B 设计（分类图标布局）
 * 支持：导航、操作、搜索
 * 视觉定制：色标图标容器、分组描述、紧凑布局
 *
 * 使用数据驱动渲染，减少重复 JSX
 */

import { useState, useCallback, type ReactNode } from "react";
import {
  CommandDialog,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandShortcut,
  CommandSeparator,
} from "@/components/ui/command";
import { useKeyboardShortcuts, SHORTCUTS, formatShortcutHint } from "@/hooks/useKeyboardShortcuts";
import { useCommandData } from "@/hooks/useCommandData";
import { cn, getContentString } from "@/lib/utils";
import type { ViewId } from "@/components/layout/Sidebar";
import {
  LayoutDashboard,
  CalendarDays,
  CheckSquare,
  Clock,
  BarChart3,
  Settings,
  Plus,
  Download,
  CheckCircle,
  Zap,
  FileText,
  type LucideIcon,
} from "lucide-react";

/** 搜索结果中每类最多显示的条目数 */
const MAX_SEARCH_RESULTS = 5;
/** 搜索结果内容预览最大字符数 */
const CONTENT_PREVIEW_LENGTH = 50;

// ─── 数据定义 ────────────────────────────────────────────────────────

interface NavigationItem {
  view: ViewId;
  icon: LucideIcon;
  label: string;
  description: string;
  shortcutKey: keyof typeof SHORTCUTS;
  iconVariant?: "default" | "primary" | "accent" | "warning";
}

interface ActionItem {
  action: string;
  icon: LucideIcon;
  label: string;
  description: string;
  shortcutKey?: keyof typeof SHORTCUTS;
  iconVariant?: "default" | "primary" | "accent" | "warning";
}

const NAVIGATION_ITEMS: NavigationItem[] = [
  { view: "dashboard", icon: LayoutDashboard, label: "工作台", description: "首页概览", shortcutKey: "VIEW_DASHBOARD", iconVariant: "accent" },
  { view: "events", icon: CalendarDays, label: "事件", description: "查看所有事件", shortcutKey: "VIEW_EVENTS" },
  { view: "tasks", icon: CheckSquare, label: "任务", description: "管理任务", shortcutKey: "VIEW_TASKS" },
  { view: "timeline", icon: Clock, label: "时间线", description: "时间轴视图", shortcutKey: "VIEW_TIMELINE" },
  { view: "reports", icon: BarChart3, label: "报告", description: "数据报告", shortcutKey: "VIEW_REPORTS" },
  { view: "settings", icon: Settings, label: "设置", description: "应用设置", shortcutKey: "VIEW_SETTINGS" },
];

const ACTION_ITEMS: ActionItem[] = [
  { action: "new-task", icon: Plus, label: "新建任务", description: "创建新的工作任务", shortcutKey: "NEW_TASK", iconVariant: "primary" },
  { action: "trigger-collect", icon: Download, label: "触发采集", description: "从外部系统获取数据", iconVariant: "accent" },
  { action: "mark-processed", icon: CheckCircle, label: "标记事件已处理", description: "批量处理事件" },
];

const AI_ITEMS: ActionItem[] = [
  { action: "ai-generate-report", icon: Zap, label: "生成今日报告", description: "基于今日工作生成报告", iconVariant: "warning" },
];

// ─── 子组件 ──────────────────────────────────────────────────────────

interface CommandPaletteProps {
  onNavigate: (view: ViewId) => void;
  onAction: (action: string) => void;
}

// 色标图标容器 — 区分不同类型的命令
function IconBox({
  children,
  variant = "default",
}: {
  children: ReactNode;
  variant?: "default" | "primary" | "accent" | "warning";
}) {
  return (
    <div
      className={cn(
        "flex size-8 items-center justify-center rounded-lg",
        variant === "default" && "border border-border bg-muted/50",
        variant === "primary" && "bg-primary text-primary-foreground",
        variant === "accent" && "bg-info/10 text-info",
        variant === "warning" && "bg-warning/15 text-warning"
      )}
    >
      {children}
    </div>
  );
}

/** 渲染单个命令项（导航或操作） */
function CommandItemRow({
  icon: Icon,
  label,
  description,
  shortcut,
  iconVariant = "default",
  onSelect,
}: {
  icon: LucideIcon;
  label: string;
  description: string;
  shortcut?: string;
  iconVariant?: "default" | "primary" | "accent" | "warning";
  onSelect: () => void;
}) {
  return (
    <CommandItem onSelect={onSelect}>
      <IconBox variant={iconVariant}>
        <Icon className="size-4" />
      </IconBox>
      <div className="flex-1">
        <div>{label}</div>
        <div className="text-xs text-muted-foreground">{description}</div>
      </div>
      {shortcut && <CommandShortcut>{shortcut}</CommandShortcut>}
    </CommandItem>
  );
}

// ─── 主组件 ──────────────────────────────────────────────────────────

export default function CommandPalette({ onNavigate, onAction }: CommandPaletteProps) {
  const [open, setOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const { events, tasks, loading } = useCommandData(searchQuery);

  // 注册 ⌘K 快捷键
  useKeyboardShortcuts([
    {
      ...SHORTCUTS.COMMAND_PALETTE,
      handler: () => setOpen((prev) => !prev),
    },
  ]);

  // 处理命令选择
  const handleSelect = useCallback(
    (command: string) => {
      setOpen(false);

      // 导航命令
      if (command.startsWith("navigate:")) {
        const view = command.replace("navigate:", "") as ViewId;
        onNavigate(view);
        return;
      }

      // 操作命令
      onAction(command);
    },
    [onNavigate, onAction]
  );

  // 键盘快捷键提示（与 SHORTCUTS 常量对齐）
  const shortcuts = {
    dashboard: formatShortcutHint(SHORTCUTS.VIEW_DASHBOARD),
    events: formatShortcutHint(SHORTCUTS.VIEW_EVENTS),
    tasks: formatShortcutHint(SHORTCUTS.VIEW_TASKS),
    timeline: formatShortcutHint(SHORTCUTS.VIEW_TIMELINE),
    reports: formatShortcutHint(SHORTCUTS.VIEW_REPORTS),
    settings: formatShortcutHint(SHORTCUTS.VIEW_SETTINGS),
    newTask: formatShortcutHint(SHORTCUTS.NEW_TASK),
  };

  // 快捷键映射
  const shortcutMap: Record<string, string> = {
    VIEW_DASHBOARD: shortcuts.dashboard,
    VIEW_EVENTS: shortcuts.events,
    VIEW_TASKS: shortcuts.tasks,
    VIEW_TIMELINE: shortcuts.timeline,
    VIEW_REPORTS: shortcuts.reports,
    VIEW_SETTINGS: shortcuts.settings,
    NEW_TASK: shortcuts.newTask,
  };

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput
        placeholder="搜索命令、事件、任务..."
        value={searchQuery}
        onValueChange={setSearchQuery}
      />
      <CommandList>
        <CommandEmpty>
          <div className="flex flex-col items-center gap-1 py-4">
            <span className="text-sm text-muted-foreground">未找到结果</span>
            <span className="text-xs text-muted-foreground/60">尝试其他关键词</span>
          </div>
        </CommandEmpty>

        {/* 快速导航 */}
        <CommandGroup heading="快速导航">
          {NAVIGATION_ITEMS.map((item) => (
            <CommandItemRow
              key={item.view}
              icon={item.icon}
              label={item.label}
              description={item.description}
              shortcut={shortcutMap[item.shortcutKey]}
              iconVariant={item.iconVariant}
              onSelect={() => handleSelect(`navigate:${item.view}`)}
            />
          ))}
        </CommandGroup>

        <CommandSeparator />

        {/* 常用操作 */}
        <CommandGroup heading="常用操作">
          {ACTION_ITEMS.map((item) => (
            <CommandItemRow
              key={item.action}
              icon={item.icon}
              label={item.label}
              description={item.description}
              shortcut={item.shortcutKey ? shortcutMap[item.shortcutKey] : undefined}
              iconVariant={item.iconVariant}
              onSelect={() => handleSelect(item.action)}
            />
          ))}
        </CommandGroup>

        <CommandSeparator />

        {/* AI 推荐 */}
        <CommandGroup heading="AI 推荐">
          {AI_ITEMS.map((item) => (
            <CommandItemRow
              key={item.action}
              icon={item.icon}
              label={item.label}
              description={item.description}
              iconVariant={item.iconVariant}
              onSelect={() => handleSelect(item.action)}
            />
          ))}
        </CommandGroup>

        {/* 搜索结果：事件 */}
        {!loading && events.length > 0 && (
          <>
            <CommandSeparator />
            <CommandGroup heading={`事件 (${events.length})`}>
              {events.slice(0, MAX_SEARCH_RESULTS).map((event) => {
                const content = getContentString(event.content);
                return (
                  <CommandItem
                    key={event.id}
                    onSelect={() => handleSelect(`event-${event.id}`)}
                  >
                    <IconBox>
                      <FileText className="size-4" />
                    </IconBox>
                    <div className="flex-1 min-w-0">
                      <div className="truncate">{content.slice(0, CONTENT_PREVIEW_LENGTH)}</div>
                      <div className="text-xs text-muted-foreground">
                        {event.source} · {event.type}
                      </div>
                    </div>
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </>
        )}

        {/* 搜索结果：任务 */}
        {!loading && tasks.length > 0 && (
          <>
            <CommandSeparator />
            <CommandGroup heading={`任务 (${tasks.length})`}>
              {tasks.slice(0, MAX_SEARCH_RESULTS).map((task) => (
                <CommandItem
                  key={task.id}
                  onSelect={() => handleSelect(`task-${task.id}`)}
                >
                  <IconBox>
                    <CheckSquare className="size-4" />
                  </IconBox>
                  <div className="flex-1 min-w-0">
                    <div className="truncate">{task.title}</div>
                    <div className="text-xs text-muted-foreground">
                      {task.status} · {task.priority}优先级
                    </div>
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>
          </>
        )}
      </CommandList>
    </CommandDialog>
  );
}
