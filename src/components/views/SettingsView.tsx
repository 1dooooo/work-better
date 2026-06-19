import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Cpu,
  Radio,
  Database,
  Keyboard,
  Sparkles,
  FileBarChart,
  Bug,
  type LucideIcon,
} from "lucide-react";
import ModelSettings from "../settings/ModelSettings";
import CollectorSettings from "../settings/CollectorSettings";
import StorageSettings from "../settings/StorageSettings";
import ShortcutSettings from "../settings/ShortcutSettings";
import FreshnessSettings from "../settings/FreshnessSettings";
import ReportSettings from "../settings/ReportSettings";
import DeveloperSettings from "../settings/DeveloperSettings";

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
  { id: "developer", label: "开发者", icon: Bug, component: DeveloperSettings },
];

export default function SettingsView() {
  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="border-b border-border px-6 py-4">
        <h1 className="text-lg font-semibold">设置</h1>
      </header>

      {/* Tabs */}
      <Tabs defaultValue="model" className="flex-1 flex flex-col">
        <div className="border-b border-border px-6">
          <TabsList variant="line">
            {SETTINGS_TABS.map((tab) => {
              const Icon = tab.icon;
              return (
                <TabsTrigger key={tab.id} value={tab.id}>
                  <Icon className="h-4 w-4" />
                  {tab.label}
                </TabsTrigger>
              );
            })}
          </TabsList>
        </div>

        {/* Content */}
        <ScrollArea className="flex-1">
          <div className="p-6">
            <div className="max-w-2xl">
              {SETTINGS_TABS.map((tab) => (
                <TabsContent key={tab.id} value={tab.id}>
                  <tab.component />
                </TabsContent>
              ))}
            </div>
          </div>
        </ScrollArea>
      </Tabs>
    </div>
  );
}
