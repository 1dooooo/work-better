import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Loader2, Check } from "lucide-react";

interface ModelConfig {
  api_endpoint: string;
  api_key: string;
  token_budget: number;
}

export default function ModelSettings() {
  const [config, setConfig] = useState<ModelConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<ModelConfig>("get_model_config").then(setConfig).catch(console.error);
  }, []);

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

  if (!config) {
    return (
      <div className="flex items-center justify-center py-8 text-muted-foreground">
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
        加载中...
      </div>
    );
  }

  return (
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
      </div>

      <div className="space-y-2">
        <Label htmlFor="api-key">API Key</Label>
        <Input
          id="api-key"
          type="password"
          value={config.api_key}
          onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
          placeholder="sk-..."
        />
      </div>

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
