import { useState, useEffect } from "react";
import { getDeveloperMode, saveDeveloperMode } from "@/lib/tauri";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Loader2, ScrollText, Bug } from "lucide-react";
import { toast } from "sonner";

export default function DeveloperSettings() {
  const [developerMode, setDeveloperMode] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    getDeveloperMode()
      .then(setDeveloperMode)
      .catch((err) => {
        console.error("Failed to get developer mode:", err);
      })
      .finally(() => setLoading(false));
  }, []);

  const handleToggle = async (enabled: boolean) => {
    setSaving(true);
    try {
      await saveDeveloperMode(enabled);
      setDeveloperMode(enabled);
      toast.success(enabled ? "已开启开发者模式" : "已关闭开发者模式", {
        description: enabled
          ? "侧边栏将显示「审计」入口，重启应用后生效"
          : "「审计」入口将隐藏，重启应用后生效",
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("Failed to save developer mode:", err);
      toast.error("保存失败", { description: message });
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-40 items-center justify-center text-muted-foreground">
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
        加载中...
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">开发者选项</h2>
        <p className="text-sm text-muted-foreground">
          启用高级调试功能，查看系统内部运行状态
        </p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Bug className="h-5 w-5" />
              <CardTitle className="text-base">开发者模式</CardTitle>
            </div>
            <div className="flex items-center gap-2">
              {saving && <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />}
              <Switch
                checked={developerMode}
                onCheckedChange={handleToggle}
                disabled={saving}
              />
            </div>
          </div>
          <CardDescription>
            开启后可在侧边栏看到「审计」入口，查看采集器运行日志和大模型执行记录
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="rounded-lg border p-4">
              <h4 className="text-sm font-medium mb-2 flex items-center gap-2">
                <ScrollText className="h-4 w-4" />
                审计日志功能
              </h4>
              <ul className="text-sm text-muted-foreground space-y-1.5">
                <li className="flex items-start gap-2">
                  <span className="text-primary">•</span>
                  <span><strong>处理日志</strong>：查看大模型的输入、输出、Token 消耗和成本</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-primary">•</span>
                  <span><strong>执行日志</strong>：查看采集器和定时任务的执行状态和耗时</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-primary">•</span>
                  <span><strong>统计摘要</strong>：总处理数、总 Token、总成本、成功率</span>
                </li>
              </ul>
            </div>

            <div className="flex items-center gap-2">
              <Badge variant="secondary">当前状态</Badge>
              <span className="text-sm">
                {developerMode ? (
                  <span className="text-green-600 dark:text-green-400">已开启</span>
                ) : (
                  <span className="text-muted-foreground">已关闭</span>
                )}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
