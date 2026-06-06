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
} from "../../lib/tauri";

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
    try {
      await saveFeishuChatId(chatId);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save chat id:", err);
    }
  }, [chatId]);

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

      <div className="settings__field">
        <span className="settings__label">飞书接入方式</span>
        <div className="settings__radio-group">
          <label className="settings__radio">
            <input
              type="radio"
              name="feishu-mode"
              checked={feishuMode === "api"}
              onChange={() => handleModeChange("api")}
            />
            <span>API 直连</span>
          </label>
          <label className="settings__radio">
            <input
              type="radio"
              name="feishu-mode"
              checked={feishuMode === "cli"}
              onChange={() => handleModeChange("cli")}
            />
            <span>lark-cli</span>
          </label>
        </div>
        <span className="settings__hint">
          {feishuMode === "cli"
            ? "通过 lark-cli 命令行工具采集，适合快速验证"
            : "通过飞书开放平台 API 采集，需配置应用凭证"}
        </span>
      </div>

      <div className="settings__field">
        <span className="settings__label">飞书会话 ID</span>
        <input
          className="settings__input"
          value={chatId}
          onChange={(e) => setChatId(e.target.value)}
          onBlur={handleChatIdSave}
          placeholder="输入飞书会话 ID"
        />
        <span className="settings__hint">
          采集时使用的飞书会话标识
        </span>
      </div>

      {saved && <span className="settings__saved">已保存</span>}
    </section>
  );
}
