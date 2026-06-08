import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Cpu,
  Radio,
  Database,
  Keyboard,
  Sparkles,
  FileBarChart,
} from "lucide-react";
import ModelSettings from "../settings/ModelSettings";
import CollectorSettings from "../settings/CollectorSettings";
import StorageSettings from "../settings/StorageSettings";
import ShortcutSettings from "../settings/ShortcutSettings";
import FreshnessSettings from "../settings/FreshnessSettings";
import ReportSettings from "../settings/ReportSettings";

const SETTINGS_TABS = [
  { id: "model", label: "模型", icon: Cpu, description: "配置 AI 模型参数" },
  { id: "collector", label: "采集器", icon: Radio, description: "管理数据采集源" },
  { id: "storage", label: "存储", icon: Database, description: "配置存储路径" },
  { id: "shortcuts", label: "快捷键", icon: Keyboard, description: "全局快捷键" },
  { id: "freshness", label: "维护", icon: Sparkles, description: "数据质量规则" },
  { id: "reports", label: "报告", icon: FileBarChart, description: "报告调度" },
] as const;

export default function SettingsView() {
  const [activeTab, setActiveTab] = useState("model");

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">设置</h1>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="mb-4 h-9">
            {SETTINGS_TABS.map((tab) => {
              const Icon = tab.icon;
              return (
                <TabsTrigger
                  key={tab.id}
                  value={tab.id}
                  className="gap-1.5 text-xs"
                >
                  <Icon className="h-3.5 w-3.5" />
                  {tab.label}
                </TabsTrigger>
              );
            })}
          </TabsList>

          {SETTINGS_TABS.map((tab) => (
            <TabsContent key={tab.id} value={tab.id}>
              <Card className="border-border">
                <CardHeader className="pb-3">
                  <div className="flex items-center gap-2">
                    <CardTitle className="text-sm">{tab.label}</CardTitle>
                    <Badge variant="secondary" className="text-[10px]">
                      {tab.description}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent>
                  {tab.id === "model" && <ModelSettings />}
                  {tab.id === "collector" && <CollectorSettings />}
                  {tab.id === "storage" && <StorageSettings />}
                  {tab.id === "shortcuts" && <ShortcutSettings />}
                  {tab.id === "freshness" && <FreshnessSettings />}
                  {tab.id === "reports" && <ReportSettings />}
                </CardContent>
              </Card>
            </TabsContent>
          ))}
        </Tabs>
      </div>
    </div>
  );
}
