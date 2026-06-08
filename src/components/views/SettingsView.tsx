import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Cpu,
  Radio,
  Database,
  Keyboard,
  Sparkles,
  FileBarChart,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import ModelSettings from "../settings/ModelSettings";
import CollectorSettings from "../settings/CollectorSettings";
import StorageSettings from "../settings/StorageSettings";
import ShortcutSettings from "../settings/ShortcutSettings";
import FreshnessSettings from "../settings/FreshnessSettings";
import ReportSettings from "../settings/ReportSettings";

interface SettingsTab {
  id: string;
  label: string;
  icon: LucideIcon;
  component: React.ComponentType;
}

const SETTINGS_TABS: SettingsTab[] = [
  { id: "model", label: "模型", icon: Cpu, component: ModelSettings },
  { id: "collector", label: "采集器", icon: Radio, component: CollectorSettings },
  { id: "storage", label: "存储", icon: Database, component: StorageSettings },
  { id: "shortcuts", label: "快捷键", icon: Keyboard, component: ShortcutSettings },
  { id: "freshness", label: "维护", icon: Sparkles, component: FreshnessSettings },
  { id: "reports", label: "报告", icon: FileBarChart, component: ReportSettings },
];

export default function SettingsView() {
  const [activeTab, setActiveTab] = useState("model");
  const active = SETTINGS_TABS.find((t) => t.id === activeTab)!;

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="border-b border-border px-6 py-4">
        <h1 className="text-lg font-semibold">设置</h1>
      </header>

      {/* Tab bar */}
      <div className="border-b border-border px-6">
        <div className="flex gap-1">
          {SETTINGS_TABS.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  "flex items-center gap-1.5 px-3 py-2.5 text-sm transition-colors",
                  "border-b-2 -mb-px",
                  isActive
                    ? "border-primary text-primary font-medium"
                    : "border-transparent text-muted-foreground hover:text-foreground"
                )}
              >
                <Icon className="h-4 w-4" />
                {tab.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="max-w-2xl">
            <active.component />
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
