import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Loader2,
  Check,
  XCircle,
  CheckCircle2,
  RefreshCw,
  Zap,
} from "lucide-react";
import { toast } from "sonner";

interface ModelConfig {
  api_endpoint: string;
  api_key: string;
  token_budget: number;
  small_model: string | null;
  large_model: string | null;
}

interface ModelInfo {
  id: string;
  name: string;
}

interface TestResult {
  success: boolean;
  message: string;
  latency_ms: number;
}

export default function ModelSettings() {
  const [config, setConfig] = useState<ModelConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [testing, setTesting] = useState<"small" | "large" | null>(null);
  const [testResult, setTestResult] = useState<TestResult | null>(null);

  useEffect(() => {
    invoke<ModelConfig>("get_model_config").then((cfg) => {
      setConfig(cfg);
      // 首次加载时，如果已有 API Key，自动拉取模型列表
      if (cfg.api_key && cfg.api_endpoint) {
        invoke<ModelInfo[]>("list_models", {
          apiEndpoint: cfg.api_endpoint,
          apiKey: cfg.api_key,
        }).then(setModels).catch(console.error);
      }
    }).catch(console.error);
  }, []);

  // 获取模型列表
  const fetchModels = useCallback(async () => {
    if (!config?.api_key || !config?.api_endpoint) return;
    setLoadingModels(true);
    try {
      const list = await invoke<ModelInfo[]>("list_models", {
        apiEndpoint: config.api_endpoint,
        apiKey: config.api_key,
      });
      setModels(list);
      if (list.length === 0) {
        toast.info("API 返回了空的模型列表，你可以手动输入模型名称");
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("Failed to fetch models:", err);
      toast.error("获取模型列表失败", { description: message });
    } finally {
      setLoadingModels(false);
    }
  }, [config?.api_endpoint, config?.api_key]);

  const handleSave = useCallback(async () => {
    if (!config) return;
    setSaving(true);
    setSaved(false);
    try {
      await invoke("save_model_config", { config });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save model config:", err);
    } finally {
      setSaving(false);
    }
  }, [config]);

  // 测试模型
  const handleTest = useCallback(
    async (which: "small" | "large") => {
      if (!config) return;
      const model = which === "small" ? config.small_model : config.large_model;
      if (!model) {
        toast.error("请先选择模型");
        return;
      }
      setTesting(which);
      setTestResult(null);
      try {
        const result = await invoke<TestResult>("test_model", {
          apiEndpoint: config.api_endpoint,
          apiKey: config.api_key,
          model,
        });
        setTestResult(result);
        if (result.success) {
          toast.success(`${model} 测试通过`, {
            description: `延迟 ${result.latency_ms}ms`,
          });
        } else {
          toast.error(`${model} 测试失败`, {
            description: result.message,
          });
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        toast.error("测试出错", { description: message });
      } finally {
        setTesting(null);
      }
    },
    [config],
  );

  if (!config) {
    return (
      <div className="flex items-center justify-center py-8 text-muted-foreground">
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
        加载中...
      </div>
    );
  }

  const hasApiKey = !!config.api_key;
  const hasModels = models.length > 0;

  return (
    <div className="space-y-6">
      {/* API 配置 */}
      <div className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="api-endpoint">API Endpoint</Label>
          <Input
            id="api-endpoint"
            type="url"
            value={config.api_endpoint}
            onChange={(e) =>
              setConfig({ ...config, api_endpoint: e.target.value })
            }
            placeholder="https://api.openai.com/v1"
          />
          <p className="text-xs text-muted-foreground">
            支持 OpenAI 兼容端点和 Anthropic 端点
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="api-key">API Key</Label>
          <Input
            id="api-key"
            type="password"
            value={config.api_key}
            onChange={(e) =>
              setConfig({ ...config, api_key: e.target.value })
            }
            placeholder="sk-..."
          />
        </div>
      </div>

      {/* 模型选择 */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium">模型选择</h3>
          {hasApiKey && (
            <Button
              variant="ghost"
              size="sm"
              onClick={fetchModels}
              disabled={loadingModels}
              className="h-7 gap-1.5 text-xs"
            >
              {loadingModels ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <RefreshCw className="h-3 w-3" />
              )}
              刷新列表
            </Button>
          )}
        </div>

        {!hasApiKey && (
          <p className="text-sm text-muted-foreground">
            请先填写 API Key，模型列表将自动加载
          </p>
        )}

        {/* 小模型 */}
        <div className="space-y-2">
          <Label>小模型</Label>
          <div className="flex gap-2">
            {hasModels ? (
              <Select
                value={config.small_model ?? undefined}
                onValueChange={(v) =>
                  setConfig({ ...config, small_model: v })
                }
              >
                <SelectTrigger className="flex-1">
                  <SelectValue placeholder="选择小模型" />
                </SelectTrigger>
                <SelectContent>
                  {models.map((m) => (
                    <SelectItem key={m.id} value={m.id}>
                      {m.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <Input
                value={config.small_model ?? ""}
                onChange={(e) =>
                  setConfig({ ...config, small_model: e.target.value })
                }
                placeholder="gpt-4o-mini"
                className="flex-1"
                disabled={!hasApiKey}
              />
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleTest("small")}
              disabled={!hasApiKey || !config.small_model || testing !== null}
              className="shrink-0 gap-1.5"
            >
              {testing === "small" ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Zap className="h-3.5 w-3.5" />
              )}
              测试
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            快速、低成本的模型（用于高置信度分类和提取）
          </p>
        </div>

        {/* 大模型 */}
        <div className="space-y-2">
          <Label>大模型</Label>
          <div className="flex gap-2">
            {hasModels ? (
              <Select
                value={config.large_model ?? undefined}
                onValueChange={(v) =>
                  setConfig({ ...config, large_model: v })
                }
              >
                <SelectTrigger className="flex-1">
                  <SelectValue placeholder="选择大模型" />
                </SelectTrigger>
                <SelectContent>
                  {models.map((m) => (
                    <SelectItem key={m.id} value={m.id}>
                      {m.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <Input
                value={config.large_model ?? ""}
                onChange={(e) =>
                  setConfig({ ...config, large_model: e.target.value })
                }
                placeholder="gpt-4o"
                className="flex-1"
                disabled={!hasApiKey}
              />
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleTest("large")}
              disabled={!hasApiKey || !config.large_model || testing !== null}
              className="shrink-0 gap-1.5"
            >
              {testing === "large" ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Zap className="h-3.5 w-3.5" />
              )}
              测试
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            强大、高成本的模型（用于低置信度或复杂任务）
          </p>
        </div>

        {/* 测试结果 */}
        {testResult && (
          <div
            className={`rounded-lg border p-3 text-sm ${
              testResult.success
                ? "border-status-success-border bg-status-success-bg"
                : "border-status-error-border bg-status-error-bg"
            }`}
          >
            <div className="flex items-start gap-2">
              {testResult.success ? (
                <CheckCircle2 className="h-4 w-4 text-status-success-text shrink-0 mt-0.5" />
              ) : (
                <XCircle className="h-4 w-4 text-status-error-text shrink-0 mt-0.5" />
              )}
              <div>
                <p
                  className={
                    testResult.success
                      ? "text-status-success-text"
                      : "text-status-error-text"
                  }
                >
                  {testResult.message}
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  延迟：{testResult.latency_ms}ms
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Token 预算 */}
      <div className="space-y-2">
        <Label htmlFor="token-budget">Token 预算</Label>
        <Input
          id="token-budget"
          type="number"
          min={256}
          max={128000}
          step={256}
          value={config.token_budget}
          onChange={(e) =>
            setConfig({ ...config, token_budget: Number(e.target.value) })
          }
          className="max-w-[200px]"
        />
        <p className="text-xs text-muted-foreground">
          单次处理最大 token 数（256 - 128000）
        </p>
      </div>

      {/* 保存 */}
      <div className="flex items-center gap-2 pt-2">
        <Button size="sm" onClick={handleSave} disabled={saving}>
          {saving ? "保存中..." : "保存"}
        </Button>
        {saved && (
          <span className="flex items-center gap-1 text-xs text-success">
            <Check className="h-3.5 w-3.5" />
            已保存
          </span>
        )}
      </div>
    </div>
  );
}
