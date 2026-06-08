import {
  Zap,
  CalendarDays,
  CheckSquare,
  Clock,
  BarChart3,
  Settings,
  Sun,
  Moon,
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
import { useState, useEffect } from "react";

export type ViewId = "events" | "tasks" | "timeline" | "reports" | "settings";

interface NavItem {
  id: ViewId;
  label: string;
  icon: LucideIcon;
}

const NAV_ITEMS: NavItem[] = [
  { id: "events", label: "事件", icon: CalendarDays },
  { id: "tasks", label: "任务", icon: CheckSquare },
  { id: "timeline", label: "时间线", icon: Clock },
  { id: "reports", label: "报告", icon: BarChart3 },
  { id: "settings", label: "设置", icon: Settings },
];

interface SidebarProps {
  activeView: ViewId;
  onViewChange: (view: ViewId) => void;
  unprocessedCount: number;
}

function useTheme() {
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    const stored = document.documentElement.dataset.theme;
    if (stored === "dark" || stored === "light") return stored;
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  });

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const toggle = () => setTheme((t) => (t === "dark" ? "light" : "dark"));

  return { theme, toggle };
}

export default function Sidebar({
  activeView,
  onViewChange,
  unprocessedCount,
}: SidebarProps) {
  const { theme, toggle } = useTheme();

  return (
    <aside className="flex h-full w-[200px] flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
      {/* Brand */}
      <div className="flex items-center gap-2 px-4 py-4">
        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
          <Zap className="h-4 w-4" />
        </div>
        <span className="text-sm font-semibold tracking-tight">
          Work Better
        </span>
      </div>

      <Separator className="bg-sidebar-border" />

      {/* Navigation */}
      <ScrollArea className="flex-1 px-2 py-2">
        <nav className="flex flex-col gap-0.5">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            const isActive = activeView === item.id;
            return (
              <Tooltip key={item.id}>
                <TooltipTrigger
                  onClick={() => onViewChange(item.id)}
                  className={cn(
                    "flex h-8 w-full items-center gap-2.5 rounded-md px-2.5 text-sm transition-colors",
                    "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring",
                    isActive &&
                      "bg-sidebar-accent font-medium text-sidebar-accent-foreground"
                  )}
                >
                  <Icon className="h-4 w-4 shrink-0" />
                  <span className="truncate">{item.label}</span>
                  {item.id === "events" && unprocessedCount > 0 && (
                    <Badge
                      variant="secondary"
                      className="ml-auto h-5 min-w-5 justify-center rounded-full px-1 text-[10px]"
                    >
                      {unprocessedCount}
                    </Badge>
                  )}
                </TooltipTrigger>
                <TooltipContent side="right" sideOffset={8}>
                  {item.label}
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
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7 text-muted-foreground hover:text-foreground"
          onClick={toggle}
          title={theme === "dark" ? "切换到亮色" : "切换到暗色"}
        >
          {theme === "dark" ? (
            <Sun className="h-3.5 w-3.5" />
          ) : (
            <Moon className="h-3.5 w-3.5" />
          )}
        </Button>
      </div>
    </aside>
  );
}
