/**
 * Command Palette Prototype — throwaway code to validate design
 *
 * Question: What should the command palette look like and how should it work?
 *
 * This prototype demonstrates:
 * 1. Command palette UI - ⌘K triggered global action interface
 * 2. Navigation functionality - switch views (dashboard, events, tasks, timeline, reports, settings)
 * 3. Operation functionality - new task, trigger collection, mark event processed
 * 4. Search functionality - search events, tasks, settings
 * 5. Shortcut hints - show numbers next to sidebar icons (⌘1-5)
 */

import { useState, useEffect, useCallback } from "react";
import {
  Command,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandShortcut,
  CommandSeparator,
} from "@/components/ui/command";
import { useKeyboardShortcuts, SHORTCUTS, formatShortcutHint } from "@/hooks/useKeyboardShortcuts";
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
  FileText,
  Zap,
} from "lucide-react";

// ─── Mock Data ──────────────────────────────────────────────────

const MOCK_EVENTS = [
  { id: 1, title: "飞书消息：项目进度更新", type: "feishu", time: "10分钟前" },
  { id: 2, title: "GitHub PR #123: 修复登录bug", type: "github", time: "30分钟前" },
  { id: 3, title: "飞书文档：需求评审会议纪要", type: "feishu", time: "1小时前" },
];

const MOCK_TASKS = [
  { id: 1, title: "完成用户认证模块", status: "进行中", priority: "高" },
  { id: 2, title: "编写API文档", status: "待处理", priority: "中" },
  { id: 3, title: "修复首页加载慢的问题", status: "已完成", priority: "高" },
];

// ─── Variant A: Compact List ────────────────────────────────────

function VariantA() {
  const [currentView, setCurrentView] = useState("dashboard");

  useKeyboardShortcuts([
    { ...SHORTCUTS.VIEW_EVENTS, handler: () => setCurrentView("events") },
    { ...SHORTCUTS.VIEW_TASKS, handler: () => setCurrentView("tasks") },
    { ...SHORTCUTS.VIEW_TIMELINE, handler: () => setCurrentView("timeline") },
    { ...SHORTCUTS.VIEW_REPORTS, handler: () => setCurrentView("reports") },
    { ...SHORTCUTS.VIEW_SETTINGS, handler: () => setCurrentView("settings") },
  ]);

  const handleSelect = useCallback((action: string) => {
    console.log("Selected:", action);
  }, []);

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold mb-2">Variant A: Compact List</h1>
        <p className="text-muted-foreground">
          紧凑列表布局，所有功能平铺展示。
        </p>
      </div>

      <div className="flex gap-4 mb-8">
        <div className="flex-1 p-4 border rounded-lg">
          <h3 className="font-medium mb-2">当前视图: {currentView}</h3>
          <p className="text-sm text-muted-foreground">
            使用 ⌘1-5 切换视图
          </p>
        </div>
      </div>

      <div className="max-w-2xl mx-auto">
        <Command className="rounded-xl border shadow-lg">
          <CommandInput placeholder="输入命令或搜索..." />
          <CommandList>
            <CommandEmpty>未找到结果</CommandEmpty>

            <CommandGroup heading="导航">
              <CommandItem onSelect={() => handleSelect("dashboard")}>
                <LayoutDashboard className="mr-2 size-4" />
                <span>工作台</span>
                <CommandShortcut>{formatShortcutHint(SHORTCUTS.VIEW_EVENTS)}</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("events")}>
                <CalendarDays className="mr-2 size-4" />
                <span>事件</span>
                <CommandShortcut>{formatShortcutHint(SHORTCUTS.VIEW_EVENTS)}</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("tasks")}>
                <CheckSquare className="mr-2 size-4" />
                <span>任务</span>
                <CommandShortcut>{formatShortcutHint(SHORTCUTS.VIEW_TASKS)}</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("timeline")}>
                <Clock className="mr-2 size-4" />
                <span>时间线</span>
                <CommandShortcut>{formatShortcutHint(SHORTCUTS.VIEW_TIMELINE)}</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("reports")}>
                <BarChart3 className="mr-2 size-4" />
                <span>报告</span>
                <CommandShortcut>{formatShortcutHint(SHORTCUTS.VIEW_REPORTS)}</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("settings")}>
                <Settings className="mr-2 size-4" />
                <span>设置</span>
                <CommandShortcut>{formatShortcutHint(SHORTCUTS.VIEW_SETTINGS)}</CommandShortcut>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="操作">
              <CommandItem onSelect={() => handleSelect("new-task")}>
                <Plus className="mr-2 size-4" />
                <span>新建任务</span>
                <CommandShortcut>{formatShortcutHint(SHORTCUTS.NEW_TASK)}</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("trigger-collect")}>
                <Download className="mr-2 size-4" />
                <span>触发采集</span>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("mark-processed")}>
                <CheckCircle className="mr-2 size-4" />
                <span>标记事件已处理</span>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="最近事件">
              {MOCK_EVENTS.map((event) => (
                <CommandItem
                  key={event.id}
                  onSelect={() => handleSelect(`event-${event.id}`)}
                >
                  <FileText className="mr-2 size-4" />
                  <div className="flex-1">
                    <div>{event.title}</div>
                    <div className="text-xs text-muted-foreground">{event.time}</div>
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>

            <CommandGroup heading="任务">
              {MOCK_TASKS.map((task) => (
                <CommandItem
                  key={task.id}
                  onSelect={() => handleSelect(`task-${task.id}`)}
                >
                  <CheckSquare className="mr-2 size-4" />
                  <div className="flex-1">
                    <div>{task.title}</div>
                    <div className="text-xs text-muted-foreground">
                      {task.status} · {task.priority}优先级
                    </div>
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </div>
    </div>
  );
}

// ─── Variant B: Categorized with Icons ──────────────────────────

function VariantB() {
  const [currentView, setCurrentView] = useState("dashboard");

  useKeyboardShortcuts([
    { ...SHORTCUTS.VIEW_EVENTS, handler: () => setCurrentView("events") },
    { ...SHORTCUTS.VIEW_TASKS, handler: () => setCurrentView("tasks") },
    { ...SHORTCUTS.VIEW_TIMELINE, handler: () => setCurrentView("timeline") },
    { ...SHORTCUTS.VIEW_REPORTS, handler: () => setCurrentView("reports") },
    { ...SHORTCUTS.VIEW_SETTINGS, handler: () => setCurrentView("settings") },
  ]);

  const handleSelect = useCallback((action: string) => {
    console.log("Selected:", action);
  }, []);

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold mb-2">Variant B: Categorized with Icons</h1>
        <p className="text-muted-foreground">
          分类图标布局，功能按类别分组，视觉更丰富。
        </p>
      </div>

      <div className="flex gap-4 mb-8">
        <div className="flex-1 p-4 border rounded-lg">
          <h3 className="font-medium mb-2">当前视图: {currentView}</h3>
          <p className="text-sm text-muted-foreground">
            使用 ⌘1-5 切换视图
          </p>
        </div>
      </div>

      <div className="max-w-2xl mx-auto">
        <Command className="rounded-xl border shadow-lg">
          <CommandInput placeholder="搜索命令、事件、任务..." />
          <CommandList>
            <CommandEmpty>未找到结果</CommandEmpty>

            <CommandGroup heading="快速导航">
              <CommandItem onSelect={() => handleSelect("dashboard")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <LayoutDashboard className="size-4" />
                </div>
                <div className="flex-1">
                  <div>工作台</div>
                  <div className="text-xs text-muted-foreground">首页概览</div>
                </div>
                <CommandShortcut>⌘1</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("events")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <CalendarDays className="size-4" />
                </div>
                <div className="flex-1">
                  <div>事件</div>
                  <div className="text-xs text-muted-foreground">查看所有事件</div>
                </div>
                <CommandShortcut>⌘2</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("tasks")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <CheckSquare className="size-4" />
                </div>
                <div className="flex-1">
                  <div>任务</div>
                  <div className="text-xs text-muted-foreground">管理任务</div>
                </div>
                <CommandShortcut>⌘3</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("timeline")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <Clock className="size-4" />
                </div>
                <div className="flex-1">
                  <div>时间线</div>
                  <div className="text-xs text-muted-foreground">时间轴视图</div>
                </div>
                <CommandShortcut>⌘4</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("reports")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <BarChart3 className="size-4" />
                </div>
                <div className="flex-1">
                  <div>报告</div>
                  <div className="text-xs text-muted-foreground">数据报告</div>
                </div>
                <CommandShortcut>⌘5</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("settings")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <Settings className="size-4" />
                </div>
                <div className="flex-1">
                  <div>设置</div>
                  <div className="text-xs text-muted-foreground">应用设置</div>
                </div>
                <CommandShortcut>⌘,</CommandShortcut>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="常用操作">
              <CommandItem onSelect={() => handleSelect("new-task")}>
                <div className="flex size-8 items-center justify-center rounded-md bg-primary text-primary-foreground">
                  <Plus className="size-4" />
                </div>
                <div className="flex-1">
                  <div>新建任务</div>
                  <div className="text-xs text-muted-foreground">创建新的工作任务</div>
                </div>
                <CommandShortcut>⌘N</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("trigger-collect")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <Download className="size-4" />
                </div>
                <div className="flex-1">
                  <div>触发采集</div>
                  <div className="text-xs text-muted-foreground">从外部系统获取数据</div>
                </div>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("mark-processed")}>
                <div className="flex size-8 items-center justify-center rounded-md border">
                  <CheckCircle className="size-4" />
                </div>
                <div className="flex-1">
                  <div>标记事件已处理</div>
                  <div className="text-xs text-muted-foreground">批量处理事件</div>
                </div>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="AI 推荐">
              <CommandItem onSelect={() => handleSelect("ai-suggest")}>
                <div className="flex size-8 items-center justify-center rounded-md bg-yellow-500 text-white">
                  <Zap className="size-4" />
                </div>
                <div className="flex-1">
                  <div>生成今日报告</div>
                  <div className="text-xs text-muted-foreground">基于今日工作生成报告</div>
                </div>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="搜索结果">
              {MOCK_EVENTS.map((event) => (
                <CommandItem
                  key={event.id}
                  onSelect={() => handleSelect(`event-${event.id}`)}
                >
                  <div className="flex size-8 items-center justify-center rounded-md border">
                    <FileText className="size-4" />
                  </div>
                  <div className="flex-1">
                    <div>{event.title}</div>
                    <div className="text-xs text-muted-foreground">{event.time}</div>
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </div>
    </div>
  );
}

// ─── Variant C: Minimal with Sections ───────────────────────────

function VariantC() {
  const [currentView, setCurrentView] = useState("dashboard");

  useKeyboardShortcuts([
    { ...SHORTCUTS.VIEW_EVENTS, handler: () => setCurrentView("events") },
    { ...SHORTCUTS.VIEW_TASKS, handler: () => setCurrentView("tasks") },
    { ...SHORTCUTS.VIEW_TIMELINE, handler: () => setCurrentView("timeline") },
    { ...SHORTCUTS.VIEW_REPORTS, handler: () => setCurrentView("reports") },
    { ...SHORTCUTS.VIEW_SETTINGS, handler: () => setCurrentView("settings") },
  ]);

  const handleSelect = useCallback((action: string) => {
    console.log("Selected:", action);
  }, []);

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold mb-2">Variant C: Minimal with Sections</h1>
        <p className="text-muted-foreground">
          极简分段布局，突出搜索和常用操作。
        </p>
      </div>

      <div className="flex gap-4 mb-8">
        <div className="flex-1 p-4 border rounded-lg">
          <h3 className="font-medium mb-2">当前视图: {currentView}</h3>
          <p className="text-sm text-muted-foreground">
            使用 ⌘1-5 切换视图
          </p>
        </div>
      </div>

      <div className="max-w-2xl mx-auto">
        <Command className="rounded-xl border shadow-lg">
          <CommandInput placeholder="搜索..." />
          <CommandList>
            <CommandEmpty>未找到结果</CommandEmpty>

            <CommandGroup heading="最近使用">
              <CommandItem onSelect={() => handleSelect("dashboard")}>
                <LayoutDashboard className="mr-2 size-4" />
                <span>工作台</span>
                <CommandShortcut>⌘1</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("tasks")}>
                <CheckSquare className="mr-2 size-4" />
                <span>任务</span>
                <CommandShortcut>⌘3</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("new-task")}>
                <Plus className="mr-2 size-4" />
                <span>新建任务</span>
                <CommandShortcut>⌘N</CommandShortcut>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="导航">
              <CommandItem onSelect={() => handleSelect("events")}>
                <CalendarDays className="mr-2 size-4" />
                <span>事件</span>
                <CommandShortcut>⌘2</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("timeline")}>
                <Clock className="mr-2 size-4" />
                <span>时间线</span>
                <CommandShortcut>⌘4</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("reports")}>
                <BarChart3 className="mr-2 size-4" />
                <span>报告</span>
                <CommandShortcut>⌘5</CommandShortcut>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("settings")}>
                <Settings className="mr-2 size-4" />
                <span>设置</span>
                <CommandShortcut>⌘,</CommandShortcut>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="操作">
              <CommandItem onSelect={() => handleSelect("trigger-collect")}>
                <Download className="mr-2 size-4" />
                <span>触发采集</span>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("mark-processed")}>
                <CheckCircle className="mr-2 size-4" />
                <span>标记事件已处理</span>
              </CommandItem>
              <CommandItem onSelect={() => handleSelect("ai-suggest")}>
                <Zap className="mr-2 size-4" />
                <span>生成今日报告</span>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="事件">
              {MOCK_EVENTS.map((event) => (
                <CommandItem
                  key={event.id}
                  onSelect={() => handleSelect(`event-${event.id}`)}
                >
                  <FileText className="mr-2 size-4" />
                  <div className="flex-1">
                    <div>{event.title}</div>
                    <div className="text-xs text-muted-foreground">{event.time}</div>
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>

            <CommandGroup heading="任务">
              {MOCK_TASKS.map((task) => (
                <CommandItem
                  key={task.id}
                  onSelect={() => handleSelect(`task-${task.id}`)}
                >
                  <CheckSquare className="mr-2 size-4" />
                  <div className="flex-1">
                    <div>{task.title}</div>
                    <div className="text-xs text-muted-foreground">
                      {task.status} · {task.priority}优先级
                    </div>
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </div>
    </div>
  );
}

// ─── Prototype Switcher ─────────────────────────────────────────

function PrototypeSwitcher({
  variants,
  current,
  onSwitch,
}: {
  variants: { key: string; name: string }[];
  current: string;
  onSwitch: (key: string) => void;
}) {
  const currentIndex = variants.findIndex((v) => v.key === current);

  const goPrev = () => {
    const prevIndex = (currentIndex - 1 + variants.length) % variants.length;
    onSwitch(variants[prevIndex].key);
  };

  const goNext = () => {
    const nextIndex = (currentIndex + 1) % variants.length;
    onSwitch(variants[nextIndex].key);
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        return;
      }

      if (e.key === "ArrowLeft") {
        goPrev();
      } else if (e.key === "ArrowRight") {
        goNext();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [currentIndex]);

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50">
      <div className="flex items-center gap-4 rounded-full bg-popover px-6 py-3 shadow-lg border">
        <button
          onClick={goPrev}
          className="p-2 hover:bg-muted rounded-full transition-colors"
        >
          ←
        </button>
        <div className="text-sm font-medium">
          {current} — {variants[currentIndex]?.name}
        </div>
        <button
          onClick={goNext}
          className="p-2 hover:bg-muted rounded-full transition-colors"
        >
          →
        </button>
      </div>
    </div>
  );
}

// ─── Main Prototype Component ───────────────────────────────────

export default function CommandPalettePrototype() {
  const [variant, setVariant] = useState(() => {
    const params = new URLSearchParams(window.location.search);
    return params.get("variant") ?? "A";
  });

  const variants = [
    { key: "A", name: "Compact List" },
    { key: "B", name: "Categorized with Icons" },
    { key: "C", name: "Minimal with Sections" },
  ];

  const handleSwitch = useCallback((key: string) => {
    setVariant(key);
    const url = new URL(window.location.href);
    url.searchParams.set("variant", key);
    window.history.replaceState({}, "", url.toString());
  }, []);

  useEffect(() => {
    const handlePopState = () => {
      const params = new URLSearchParams(window.location.search);
      const newVariant = params.get("variant") ?? "A";
      setVariant(newVariant);
    };

    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, []);

  return (
    <>
      {variant === "A" && <VariantA />}
      {variant === "B" && <VariantB />}
      {variant === "C" && <VariantC />}
      {process.env.NODE_ENV !== "production" && (
        <PrototypeSwitcher
          variants={variants}
          current={variant}
          onSwitch={handleSwitch}
        />
      )}
    </>
  );
}
