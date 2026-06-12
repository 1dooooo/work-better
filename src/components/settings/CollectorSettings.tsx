import { useState, useEffect, useCallback } from "react";
import {
  getFeishuMode,
  saveFeishuMode,
  getFeishuChatId,
  saveFeishuChatId,
  getCollectorStatuses,
  enableCollector,
  disableCollector,
  type CollectorStatus,
} from "@/lib/tauri";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Loader2, Check, Radio } from "lucide-react";
import { cn } from "@/lib/utils";

export default function CollectorSettings() {
  const [collectors, setCollectors] = useState<CollectorStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [feishuMode, setFeishuMode] = useState<"api" | "cli">("cli");
  const [chatId, setChatId] = useState("");
  const [saved, setSaved] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const [list, mode, cid] = await Promise.all([
        getCollectorStatuses(),
        getFeishuMode(),
        getFeishuChatId(),
      ]);
      setCollectors(list);
      setFeishuMode(mode as "api" | "cli");
      setChatId(cid);
    } catch (err) {
      console.error("Failed to load collector settings:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleToggle = useCallback(
    async (id: string, enabled: boolean) => {
      try {
        if (enabled) {
          await enableCollector(id);
        } else {
          await disableCollector(id);
        }
        setCollectors((prev) =>
          prev.map((c) => (c.id === id ? { ...c, enabled } : c)),
        );
      } catch (err) {
        console.error("Failed to toggle collector:", err);
        refresh();
      }
    },
    [refresh],
  );

  const handleModeChange = useCallback(async (mode: "api" | "cli") => {
    setFeishuMode(mode);
    try {
      await saveFeishuMode(mode);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save feishu mode:", err);
    }
  }, []);

  const handleChatIdSave = useCallback(async () => {
    const cleaned = chatId.trim().replace(/，/g, ",");
    setChatId(cleaned);
    try {
      await saveFeishuChatId(cleaned);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save chat id:", err);
    }
  }, [chatId]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8 text-muted-foreground">
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
        加载中...
      </div>
    );
  }

  return (
    <div className="space-y-5">
      {/* Collector List */}
      <div className="space-y-2">
        <Label>采集器</Label>
        <div className="space-y-1">
          {collectors.map((collector) => (
            <div
              key={collector.id}
              className="flex items-center justify-between rounded-md border border-border px-3 py-2"
            >
              <div className="flex items-center gap-2">
                <span className="text-sm">{collector.name}</span>
                <Badge
                  variant={
                    !collector.enabled
                      ? "outline"
                      : collector.healthy
                        ? "secondary"
                        : "destructive"
                  }
                  className="text-[10px]"
                >
                  {!collector.enabled
                    ? "未启用"
                    : collector.healthy
                      ? "正常"
                      : "异常"}
                </Badge>
              </div>
              <Switch
                checked={collector.enabled}
                onCheckedChange={(checked) =>
                  handleToggle(collector.id, checked)
                }
              />
            </div>
          ))}
        </div>
      </div>

      {/* Feishu Mode */}
      <div className="space-y-2">
        <Label>飞书接入方式</Label>
        <div className="flex gap-3">
          {(["api", "cli"] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => handleModeChange(mode)}
              className={cn(
                "flex items-center gap-2 rounded-md border px-3 py-2 text-sm transition-colors",
                feishuMode === mode
                  ? "border-primary bg-primary/5 text-primary"
                  : "border-border text-muted-foreground hover:bg-muted"
              )}
            >
              <Radio className="h-3.5 w-3.5" />
              {mode === "api" ? "API 直连" : "lark-cli"}
            </button>
          ))}
        </div>
        <p className="text-xs text-muted-foreground">
          {feishuMode === "cli"
            ? "通过 lark-cli 命令行工具采集，适合快速验证"
            : "通过飞书开放平台 API 采集，需配置应用凭证"}
        </p>
      </div>

      {/* Chat ID */}
      <div className="space-y-2">
        <Label htmlFor="chat-id">飞书会话 ID</Label>
        <Input
          id="chat-id"
          value={chatId}
          onChange={(e) => setChatId(e.target.value)}
          onBlur={handleChatIdSave}
          placeholder="输入飞书会话 ID"
        />
        <p className="text-xs text-muted-foreground">
          采集时使用的飞书会话标识
        </p>
      </div>

      {saved && (
        <span className="flex items-center gap-1 text-xs text-success">
          <Check className="h-3.5 w-3.5" />
          已保存
        </span>
      )}
    </div>
  );
}
