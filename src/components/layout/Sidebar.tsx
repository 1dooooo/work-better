import {
  Zap,
  CalendarDays,
  CheckSquare,
  Clock,
  BarChart3,
  Settings,
  Sun,
  Moon,
  ScrollText,
  PanelLeftClose,
  PanelLeftOpen,
  type LucideIcon,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useTheme } from "next-themes";
import { formatShortcutHint, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";

export type ViewId = "dashboard" | "events" | "tasks" | "timeline" | "reports" | "settings" | "audit";

interface NavItem {
  id: ViewId;
  label: string;
  icon: LucideIcon;
  /** 快捷键提示 */
  shortcut?: string;
  /** 是否为底部固定项（设置等） */
  isBottom?: boolean;
  /** 是否为开发者模式专属项 */
  developerOnly?: boolean;
}

const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "工作台", icon: Zap, shortcut: formatShortcutHint(SHORTCUTS.VIEW_DASHBOARD) },
  { id: "events", label: "事件", icon: CalendarDays, shortcut: formatShortcutHint(SHORTCUTS.VIEW_EVENTS) },
  { id: "tasks", label: "任务", icon: CheckSquare, shortcut: formatShortcutHint(SHORTCUTS.VIEW_TASKS) },
  { id: "timeline", label: "时间线", icon: Clock, shortcut: formatShortcutHint(SHORTCUTS.VIEW_TIMELINE) },
  { id: "reports", label: "报告", icon: BarChart3, shortcut: formatShortcutHint(SHORTCUTS.VIEW_REPORTS) },
  // 开发者模式专属项
  { id: "audit", label: "审计", icon: ScrollText, developerOnly: true },
  // 底部固定项
  { id: "settings", label: "设置", icon: Settings, shortcut: formatShortcutHint(SHORTCUTS.VIEW_SETTINGS), isBottom: true },
];

interface SidebarProps {
  activeView: ViewId;
  onViewChange: (view: ViewId) => void;
  unprocessedCount: number;
  developerMode?: boolean;
  collapsed?: boolean;
  onCollapsedChange?: (collapsed: boolean) => void;
}

export default function Sidebar({
  activeView,
  onViewChange,
  unprocessedCount,
  developerMode = false,
  collapsed = false,
  onCollapsedChange,
}: SidebarProps) {
  const { resolvedTheme, setTheme } = useTheme();
  const isDark = resolvedTheme === "dark";
  const toggle = () => setTheme(isDark ? "light" : "dark");

  // M6: 使用更清晰的过滤逻辑
  const navItems = NAV_ITEMS.filter((item) => {
    // developerOnly 项仅在开发者模式下显示
    if (item.developerOnly && !developerMode) return false;
    return true;
  });

  return (
    <aside className={cn(
      "flex h-full flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground transition-all duration-200",
      collapsed ? "w-[52px]" : "w-[200px]"
    )}>
      {/* Brand */}
      <div className="flex items-center gap-2 px-4 py-4">
        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
          <Zap className="h-4 w-4" />
        </div>
        {!collapsed && (
          <span className="text-sm font-semibold tracking-tight">
            Work Better
          </span>
        )}
      </div>

      <Separator className="bg-sidebar-border" />

      {/* Navigation */}
      <ScrollArea className="flex-1 px-2 py-2">
        <nav className="flex flex-col gap-0.5">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeView === item.id;
            return (
              <Tooltip key={item.id}>
                <TooltipTrigger
                  onClick={() => onViewChange(item.id)}
                  className={cn(
                    "flex h-8 w-full items-center rounded-md px-2.5 text-sm transition-colors",
                    collapsed ? "justify-center" : "gap-2.5",
                    "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring",
                    isActive &&
                      "bg-sidebar-accent font-medium text-sidebar-accent-foreground"
                  )}
                >
                  <Icon className="h-4 w-4 shrink-0" />
                  {!collapsed && <span className="truncate">{item.label}</span>}
                  {!collapsed && item.id === "events" && unprocessedCount > 0 && (
                    <Badge
                      variant="secondary"
                      className="ml-auto h-5 min-w-5 justify-center rounded-full px-1 text-[10px]"
                    >
                      {unprocessedCount}
                    </Badge>
                  )}
                  {!collapsed && item.shortcut && (
                    <span className="ml-auto text-[10px] text-muted-foreground">
                      {item.shortcut}
                    </span>
                  )}
                </TooltipTrigger>
                <TooltipContent side="right" sideOffset={8}>
                  {item.label}
                  {item.shortcut && ` (${item.shortcut})`}
                </TooltipContent>
              </Tooltip>
            );
          })}
        </nav>
      </ScrollArea>

      <Separator className="bg-sidebar-border" />

      {/* Footer */}
      <div className="flex items-center justify-between px-3 py-3">
        <span className="text-[11px] text-muted-foreground">v0.1.0</span>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground"
            onClick={toggle}
            title={isDark ? "切换到亮色" : "切换到暗色"}
          >
            {isDark ? (
              <Sun className="h-3.5 w-3.5" />
            ) : (
              <Moon className="h-3.5 w-3.5" />
            )}
          </Button>
          {onCollapsedChange && (
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-muted-foreground hover:text-foreground"
              onClick={() => onCollapsedChange(!collapsed)}
              title={collapsed ? "展开侧边栏" : "折叠侧边栏"}
            >
              {collapsed ? (
                <PanelLeftOpen className="h-3.5 w-3.5" />
              ) : (
                <PanelLeftClose className="h-3.5 w-3.5" />
              )}
            </Button>
          )}
        </div>
      </div>
    </aside>
  );
}
