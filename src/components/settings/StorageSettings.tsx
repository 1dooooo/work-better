import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Loader2, Check } from "lucide-react";

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
        <Label htmlFor="vault-path">Obsidian Vault 路径</Label>
        <Input
          id="vault-path"
          type="text"
          value={config.vault_path}
          onChange={(e) =>
            setConfig({ ...config, vault_path: e.target.value })
          }
          placeholder="~/Documents/Obsidian"
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="db-path">数据库路径</Label>
        <Input
          id="db-path"
          type="text"
          value={config.db_path}
          onChange={(e) =>
            setConfig({ ...config, db_path: e.target.value })
          }
          placeholder="~/.work-better/data.db"
        />
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
