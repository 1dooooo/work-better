import { useState, useEffect, useCallback } from "react";
import {
  getFeishuMode,
  saveFeishuMode,
  getFeishuChatId,
  saveFeishuChatId,
  getCollectorGroups,
  enableCollector,
  disableCollector,
  enableCollectorGroup,
  disableCollectorGroup,
  type CollectorGroup,
  type CollectorStatus,
} from "@/lib/tauri";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import {
  Loader2,
  Check,
  Radio,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { cn } from "@/lib/utils";

export default function CollectorSettings() {
  const [groups, setGroups] = useState<CollectorGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [feishuMode, setFeishuMode] = useState<"api" | "cli">("cli");
  const [chatId, setChatId] = useState("");
  const [saved, setSaved] = useState(false);
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());

  const refresh = useCallback(async () => {
    try {
      const [groupList, mode, cid] = await Promise.all([
        getCollectorGroups(),
        getFeishuMode(),
        getFeishuChatId(),
      ]);
      setGroups(groupList);
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

  const toggleGroup = useCallback(
    (groupId: string) => {
      setExpandedGroups((prev) => {
        const next = new Set(prev);
        if (next.has(groupId)) {
          next.delete(groupId);
        } else {
          next.add(groupId);
        }
        return next;
      });
    },
    [],
  );

  const handleGroupToggle = useCallback(
    async (groupId: string, enabled: boolean) => {
      try {
        if (enabled) {
          await enableCollectorGroup(groupId);
        } else {
          await disableCollectorGroup(groupId);
        }
        setGroups((prev) =>
          prev.map((g) => (g.id === groupId ? { ...g, enabled } : g)),
        );
      } catch (err) {
        console.error("Failed to toggle collector group:", err);
        refresh();
      }
    },
    [refresh],
  );

  const handleCollectorToggle = useCallback(
    async (collectorId: string, enabled: boolean) => {
      try {
        if (enabled) {
          await enableCollector(collectorId);
        } else {
          await disableCollector(collectorId);
        }
        setGroups((prev) =>
          prev.map((g) => ({
            ...g,
            collectors: g.collectors.map((c) =>
              c.id === collectorId ? { ...c, enabled } : c,
            ),
          })),
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

  const getHealthBadge = (collector: CollectorStatus) => {
    if (!collector.enabled) {
      return (
        <Badge variant="outline" className="text-[10px]">
          未启用
        </Badge>
      );
    }
    switch (collector.health_level) {
      case "healthy":
        return (
          <Badge variant="secondary" className="text-[10px]">
            正常
          </Badge>
        );
      case "degraded":
        return (
          <Badge
            variant="outline"
            className="text-[10px] border-yellow-500 text-yellow-600"
          >
            降级
          </Badge>
        );
      case "unhealthy":
        return (
          <Badge variant="destructive" className="text-[10px]">
            异常
          </Badge>
        );
      default:
        return (
          <Badge variant="outline" className="text-[10px]">
            未知
          </Badge>
        );
    }
  };

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
      {/* Collector Groups */}
      <div className="space-y-2">
        <Label>采集器</Label>
        <div className="space-y-3">
          {groups.map((group) => (
            <div
              key={group.id}
              className="rounded-lg border border-border overflow-hidden"
            >
              {/* Group Header */}
              <div className="flex items-center justify-between px-3 py-2.5 bg-muted/30">
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => toggleGroup(group.id)}
                    className="flex items-center gap-1 text-sm font-medium hover:text-foreground/80 transition-colors"
                  >
                    {expandedGroups.has(group.id) ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                    {group.name}
                  </button>
                  <Badge variant="outline" className="text-[10px]">
                    {group.collectors.filter((c) => c.enabled).length}/
                    {group.collectors.length}
                  </Badge>
                </div>
                <Switch
                  checked={group.enabled}
                  onCheckedChange={(checked) =>
                    handleGroupToggle(group.id, checked)
                  }
                />
              </div>

              {/* Collectors List */}
              {expandedGroups.has(group.id) && (
                <div className="divide-y divide-border">
                  {group.collectors.map((collector) => (
                    <div
                      key={collector.id}
                      className={cn(
                        "flex items-center justify-between px-4 py-2",
                        !group.enabled && "opacity-50 pointer-events-none",
                      )}
                    >
                      <div className="flex items-center gap-2">
                        <span className="text-sm">{collector.name}</span>
                        {getHealthBadge(collector)}
                      </div>
                      <Switch
                        checked={collector.enabled}
                        disabled={!group.enabled}
                        onCheckedChange={(checked) =>
                          handleCollectorToggle(collector.id, checked)
                        }
                      />
                    </div>
                  ))}
                </div>
              )}
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
                  : "border-border text-muted-foreground hover:bg-muted",
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
