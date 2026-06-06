import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CollectorStatus {
  id: string;
  name: string;
  enabled: boolean;
  healthy: boolean;
}

export default function CollectorSettings() {
  const [collectors, setCollectors] = useState<CollectorStatus[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<CollectorStatus[]>("get_collector_statuses");
      setCollectors(list);
    } catch (err) {
      console.error("Failed to load collector statuses:", err);
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
          await invoke("enable_collector", { id });
        } else {
          await invoke("disable_collector", { id });
        }
        // Optimistic update
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

  if (loading) {
    return <div className="settings__loading">加载中...</div>;
  }

  return (
    <section className="settings__section">
      <h3 className="settings__section-title">采集器配置</h3>

      <ul className="settings__collector-list">
        {collectors.map((collector) => (
          <li key={collector.id} className="settings__collector-item">
            <div className="settings__collector-info">
              <span className="settings__collector-name">
                {collector.name}
              </span>
              <span
                className={`settings__collector-health ${
                  collector.healthy
                    ? "settings__collector-health--ok"
                    : "settings__collector-health--error"
                }`}
              >
                {collector.healthy ? "正常" : "异常"}
              </span>
            </div>
            <label className="settings__toggle">
              <input
                type="checkbox"
                checked={collector.enabled}
                onChange={(e) =>
                  handleToggle(collector.id, e.target.checked)
                }
              />
              <span className="settings__toggle-slider" />
            </label>
          </li>
        ))}
      </ul>
    </section>
  );
}
