import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StorageConfig {
  vault_path: string;
  db_path: string;
}

export default function StorageSettings() {
  const [config, setConfig] = useState<StorageConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<StorageConfig>("get_storage_config")
      .then(setConfig)
      .catch(console.error);
  }, []);

  const handleSave = useCallback(async () => {
    if (!config) return;
    setSaving(true);
    setSaved(false);
    try {
      await invoke("save_storage_config", { config });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save storage config:", err);
    } finally {
      setSaving(false);
    }
  }, [config]);

  if (!config) {
    return <div className="settings__loading">加载中...</div>;
  }

  return (
    <section className="settings__section">
      <h3 className="settings__section-title">存储配置</h3>

      <div className="settings__field">
        <label className="settings__label" htmlFor="vault-path">
          Obsidian Vault 路径
        </label>
        <input
          id="vault-path"
          className="settings__input"
          type="text"
          value={config.vault_path}
          onChange={(e) =>
            setConfig({ ...config, vault_path: e.target.value })
          }
          placeholder="~/Documents/Obsidian"
        />
      </div>

      <div className="settings__field">
        <label className="settings__label" htmlFor="db-path">
          数据库路径
        </label>
        <input
          id="db-path"
          className="settings__input"
          type="text"
          value={config.db_path}
          onChange={(e) =>
            setConfig({ ...config, db_path: e.target.value })
          }
          placeholder="~/.work-better/data.db"
        />
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
