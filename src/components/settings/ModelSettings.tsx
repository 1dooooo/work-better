import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

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
    return <div className="settings__loading">加载中...</div>;
  }

  return (
    <section className="settings__section">
      <h3 className="settings__section-title">模型配置</h3>

      <div className="settings__field">
        <label className="settings__label" htmlFor="api-endpoint">
          API Endpoint
        </label>
        <input
          id="api-endpoint"
          className="settings__input"
          type="url"
          value={config.api_endpoint}
          onChange={(e) =>
            setConfig({ ...config, api_endpoint: e.target.value })
          }
          placeholder="https://api.openai.com/v1"
        />
      </div>

      <div className="settings__field">
        <label className="settings__label" htmlFor="api-key">
          API Key
        </label>
        <input
          id="api-key"
          className="settings__input"
          type="password"
          value={config.api_key}
          onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
          placeholder="sk-..."
        />
      </div>

      <div className="settings__field">
        <label className="settings__label" htmlFor="token-budget">
          Token 预算
        </label>
        <input
          id="token-budget"
          className="settings__input settings__input--short"
          type="number"
          min={256}
          max={128000}
          step={256}
          value={config.token_budget}
          onChange={(e) =>
            setConfig({ ...config, token_budget: Number(e.target.value) })
          }
        />
        <span className="settings__hint">
          单次处理最大 token 数（256 - 128000）
        </span>
      </div>

      <div className="settings__actions">
        <button
          className="view__btn"
          onClick={handleSave}
          disabled={saving}
        >
          {saving ? "保存中..." : "保存"}
        </button>
        {saved && <span className="settings__saved">已保存</span>}
      </div>
    </section>
  );
}
