import { useState } from "react";

interface ReportConfig {
  id: string;
  name: string;
  enabled: boolean;
  schedule: string;
  autoConfirm: boolean;
}

const DEFAULT_CONFIGS: ReportConfig[] = [
  { id: "daily", name: "日报", enabled: true, schedule: "0 18 * * 1-5", autoConfirm: false },
  { id: "weekly", name: "周报", enabled: true, schedule: "0 17 * * 5", autoConfirm: false },
  { id: "monthly", name: "月报", enabled: true, schedule: "0 17 L * *", autoConfirm: false },
  { id: "quarterly", name: "季报", enabled: false, schedule: "0 17 1 1,4,7,10 *", autoConfirm: false },
];

const SCHEDULE_LABELS: Record<string, string> = {
  "0 18 * * 1-5": "工作日 18:00",
  "0 17 * * 5": "每周五 17:00",
  "0 17 L * *": "每月最后一天 17:00",
  "0 17 1 1,4,7,10 *": "每季度首月1日 17:00",
};

export default function ReportSettings() {
  const [configs, setConfigs] = useState<ReportConfig[]>(DEFAULT_CONFIGS);

  const toggleReport = (id: string) => {
    setConfigs((prev) =>
      prev.map((c) => (c.id === id ? { ...c, enabled: !c.enabled } : c))
    );
  };

  const toggleAutoConfirm = (id: string) => {
    setConfigs((prev) =>
      prev.map((c) => (c.id === id ? { ...c, autoConfirm: !c.autoConfirm } : c))
    );
  };

  return (
    <section className="settings__section">
      <h3 className="settings__section-title">报告配置</h3>
      <p className="settings__hint">报告定时任务和确认策略</p>

      <div className="settings__list">
        {configs.map((c) => (
          <div key={c.id} className="settings__list-item">
            <label className="settings__toggle">
              <input
                type="checkbox"
                checked={c.enabled}
                onChange={() => toggleReport(c.id)}
              />
              <span className="settings__toggle-slider" />
            </label>
            <span>{c.name}</span>
            <span className="settings__hint">
              {SCHEDULE_LABELS[c.schedule] ?? c.schedule}
            </span>
            <label className="settings__auto-label">
              <input
                type="checkbox"
                checked={c.autoConfirm}
                onChange={() => toggleAutoConfirm(c.id)}
                disabled={!c.enabled}
              />
              <span>自动确认</span>
            </label>
          </div>
        ))}
      </div>
    </section>
  );
}
