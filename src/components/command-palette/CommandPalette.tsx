/**
 * CommandPalette — 命令面板组件
 *
 * 集成 Variant B 设计（分类图标布局）
 * 支持：导航、操作、搜索
 * 视觉定制：色标图标容器、分组描述、紧凑布局
 */

import { useState, useCallback } from "react";
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
import { cn } from "@/lib/utils";
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
} from "lucide-react";

/** 搜索结果中每类最多显示的条目数 */
const MAX_SEARCH_RESULTS = 5;
/** 搜索结果内容预览最大字符数 */
const CONTENT_PREVIEW_LENGTH = 50;

interface CommandPaletteProps {
  onNavigate: (view: ViewId) => void;
  onAction: (action: string) => void;
}

// 色标图标容器 — 区分不同类型的命令
function IconBox({
  children,
  variant = "default",
}: {
  children: React.ReactNode;
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
          <CommandItem onSelect={() => handleSelect("navigate:dashboard")}>
            <IconBox variant="accent">
              <LayoutDashboard className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>工作台</div>
              <div className="text-xs text-muted-foreground">首页概览</div>
            </div>
            <CommandShortcut>{shortcuts.dashboard}</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => handleSelect("navigate:events")}>
            <IconBox>
              <CalendarDays className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>事件</div>
              <div className="text-xs text-muted-foreground">查看所有事件</div>
            </div>
            <CommandShortcut>{shortcuts.events}</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => handleSelect("navigate:tasks")}>
            <IconBox>
              <CheckSquare className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>任务</div>
              <div className="text-xs text-muted-foreground">管理任务</div>
            </div>
            <CommandShortcut>{shortcuts.tasks}</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => handleSelect("navigate:timeline")}>
            <IconBox>
              <Clock className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>时间线</div>
              <div className="text-xs text-muted-foreground">时间轴视图</div>
            </div>
            <CommandShortcut>{shortcuts.timeline}</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => handleSelect("navigate:reports")}>
            <IconBox>
              <BarChart3 className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>报告</div>
              <div className="text-xs text-muted-foreground">数据报告</div>
            </div>
            <CommandShortcut>{shortcuts.reports}</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => handleSelect("navigate:settings")}>
            <IconBox>
              <Settings className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>设置</div>
              <div className="text-xs text-muted-foreground">应用设置</div>
            </div>
            <CommandShortcut>{shortcuts.settings}</CommandShortcut>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator />

        {/* 常用操作 */}
        <CommandGroup heading="常用操作">
          <CommandItem onSelect={() => handleSelect("new-task")}>
            <IconBox variant="primary">
              <Plus className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>新建任务</div>
              <div className="text-xs text-muted-foreground">创建新的工作任务</div>
            </div>
            <CommandShortcut>{shortcuts.newTask}</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => handleSelect("trigger-collect")}>
            <IconBox variant="accent">
              <Download className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>触发采集</div>
              <div className="text-xs text-muted-foreground">从外部系统获取数据</div>
            </div>
          </CommandItem>
          <CommandItem onSelect={() => handleSelect("mark-processed")}>
            <IconBox>
              <CheckCircle className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>标记事件已处理</div>
              <div className="text-xs text-muted-foreground">批量处理事件</div>
            </div>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator />

        {/* AI 推荐 */}
        <CommandGroup heading="AI 推荐">
          <CommandItem onSelect={() => handleSelect("ai-generate-report")}>
            <IconBox variant="warning">
              <Zap className="size-4" />
            </IconBox>
            <div className="flex-1">
              <div>生成今日报告</div>
              <div className="text-xs text-muted-foreground">基于今日工作生成报告</div>
            </div>
          </CommandItem>
        </CommandGroup>

        {/* 搜索结果：事件 */}
        {!loading && events.length > 0 && (
          <>
            <CommandSeparator />
            <CommandGroup heading={`事件 (${events.length})`}>
              {events.slice(0, MAX_SEARCH_RESULTS).map((event) => {
                const content =
                  typeof event.content === "string"
                    ? event.content
                    : JSON.stringify(event.content);
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
