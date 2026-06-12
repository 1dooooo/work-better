import { useState, useEffect, useCallback } from "react";
import {
  getDeveloperMode,
  saveDeveloperMode,
  triggerBatchProcess,
  getUnprocessedCount,
  getModelConfig,
  type BatchProcessResult,
} from "@/lib/tauri";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Loader2,
  ScrollText,
  Bug,
  Sparkles,
  CheckCircle2,
  XCircle,
  SkipForward,
  Inbox,
  AlertTriangle,
} from "lucide-react";
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

      {/* 开发者模式开关 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Bug className="h-5 w-5" />
              <CardTitle className="text-base">开发者模式</CardTitle>
            </div>
            <div className="flex items-center gap-2">
              {saving && (
                <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
              )}
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
                  <span>
                    <strong>处理日志</strong>：查看大模型的输入、输出、Token
                    消耗和成本
                  </span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-primary">•</span>
                  <span>
                    <strong>执行日志</strong>：查看采集器和定时任务的执行状态和耗时
                  </span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-primary">•</span>
                  <span>
                    <strong>统计摘要</strong>：总处理数、总 Token、总成本、成功率
                  </span>
                </li>
              </ul>
            </div>

            <div className="flex items-center gap-2">
              <Badge variant="secondary">当前状态</Badge>
              <span className="text-sm">
                {developerMode ? (
                  <span className="text-green-600 dark:text-green-400">
                    已开启
                  </span>
                ) : (
                  <span className="text-muted-foreground">已关闭</span>
                )}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 主动整理 */}
      <OrganizeCard />
    </div>
  );
}

// ─── 主动整理卡片 ─────────────────────────────────────────────────

function OrganizeCard() {
  const [unprocessedCount, setUnprocessedCount] = useState<number | null>(null);
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<BatchProcessResult | null>(null);
  const [apiKeyConfigured, setApiKeyConfigured] = useState<boolean | null>(null);

  const refreshCount = useCallback(() => {
    getUnprocessedCount()
      .then(setUnprocessedCount)
      .catch(() => setUnprocessedCount(null));
  }, []);

  useEffect(() => {
    refreshCount();
    // 检查 API Key 是否已配置
    getModelConfig()
      .then((config) => setApiKeyConfigured(!!config.api_key))
      .catch(() => setApiKeyConfigured(false));
  }, [refreshCount]);

  const handleOrganize = async () => {
    setProcessing(true);
    setResult(null);
    try {
      const batchResult = await triggerBatchProcess();
      setResult(batchResult);
      refreshCount();

      if (batchResult.total === 0) {
        toast.info("没有待处理的事件", {
          description: "所有事件都已处理完毕",
        });
      } else if (batchResult.failed === 0) {
        toast.success("整理完成", {
          description: `成功处理 ${batchResult.success} 条事件`,
        });
      } else {
        toast.warning("整理完成（部分失败）", {
          description: `成功 ${batchResult.success}，失败 ${batchResult.failed}，跳过 ${batchResult.skipped}`,
        });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error("整理失败", { description: message });
    } finally {
      setProcessing(false);
    }
  };

  const needsApiKey = apiKeyConfigured === false;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Sparkles className="h-5 w-5" />
            <CardTitle className="text-base">主动整理</CardTitle>
          </div>
          <Button
            size="sm"
            onClick={handleOrganize}
            disabled={processing || needsApiKey}
            className="gap-1.5"
          >
            {processing ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Sparkles className="h-3.5 w-3.5" />
            )}
            {processing ? "整理中..." : "开始整理"}
          </Button>
        </div>
        <CardDescription>
          手动触发批量事件处理。遍历所有未处理事件，调用大模型进行分类和提取。
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {/* API Key 未配置警告 */}
          {needsApiKey && (
            <div className="rounded-lg border border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-950/30 p-4">
              <div className="flex items-start gap-3">
                <AlertTriangle className="h-5 w-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
                <div className="space-y-2">
                  <p className="text-sm font-medium text-amber-800 dark:text-amber-300">
                    未配置 API Key
                  </p>
                  <p className="text-sm text-amber-700 dark:text-amber-400">
                    主动整理需要调用大模型进行事件分类和信息提取。
                    请先在「设置 → 模型」中配置 API Key。
                  </p>
                  <p className="text-xs text-amber-600 dark:text-amber-500">
                    未配置时应用将以只读模式运行：可以查看已采集的事件，但无法进行智能分析。
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* 只读模式提示 */}
          {apiKeyConfigured === true && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <CheckCircle2 className="h-3.5 w-3.5 text-green-600 dark:text-green-400" />
              <span>API Key 已配置，大模型调用已就绪</span>
            </div>
          )}

          {/* 待处理数量 */}
          <div className="flex items-center gap-2">
            <Badge variant="secondary">待处理</Badge>
            <span className="text-sm">
              {unprocessedCount === null ? (
                <span className="text-muted-foreground">加载中...</span>
              ) : unprocessedCount === 0 ? (
                <span className="text-muted-foreground flex items-center gap-1">
                  <Inbox className="h-3.5 w-3.5" />
                  无待处理事件
                </span>
              ) : (
                <span className="text-amber-600 dark:text-amber-400 font-medium">
                  {unprocessedCount} 条事件
                </span>
              )}
            </span>
          </div>

          {/* 处理结果 */}
          {result && (
            <div className="rounded-lg border p-4 space-y-3">
              <h4 className="text-sm font-medium">处理结果</h4>
              <div className="grid grid-cols-4 gap-3">
                <ResultStat
                  label="总计"
                  value={result.total}
                  className="text-foreground"
                />
                <ResultStat
                  label="成功"
                  value={result.success}
                  icon={<CheckCircle2 className="h-3.5 w-3.5" />}
                  className="text-green-600 dark:text-green-400"
                />
                <ResultStat
                  label="失败"
                  value={result.failed}
                  icon={<XCircle className="h-3.5 w-3.5" />}
                  className="text-destructive"
                />
                <ResultStat
                  label="跳过"
                  value={result.skipped}
                  icon={<SkipForward className="h-3.5 w-3.5" />}
                  className="text-muted-foreground"
                />
              </div>

              {/* 失败详情 */}
              {result.failed > 0 && (
                <div className="space-y-1">
                  <p className="text-xs text-muted-foreground">失败详情：</p>
                  {result.details
                    .filter((d) => d.status === "failed")
                    .slice(0, 5)
                    .map((d) => (
                      <p
                        key={d.event_id}
                        className="text-xs text-destructive truncate"
                      >
                        {d.event_id}: {d.error}
                      </p>
                    ))}
                  {result.details.filter((d) => d.status === "failed").length >
                    5 && (
                    <p className="text-xs text-muted-foreground">
                      ...还有{" "}
                      {result.details.filter((d) => d.status === "failed")
                        .length - 5}{" "}
                      条
                    </p>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function ResultStat({
  label,
  value,
  icon,
  className = "",
}: {
  label: string;
  value: number;
  icon?: React.ReactNode;
  className?: string;
}) {
  return (
    <div className="flex flex-col items-center gap-1">
      <span className={`text-lg font-semibold ${className}`}>{value}</span>
      <span className="text-[11px] text-muted-foreground flex items-center gap-1">
        {icon}
        {label}
      </span>
    </div>
  );
}
