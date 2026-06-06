import { useState } from "react";

interface FreshnessRule {
  id: string;
  name: string;
  enabled: boolean;
  frequency: string;
}

const DEFAULT_RULES: FreshnessRule[] = [
  { id: "sync", name: "任务状态同步", enabled: true, frequency: "*/15 * * * *" },
  { id: "integrity", name: "链接完整性检查", enabled: true, frequency: "0 */6 * * *" },
  { id: "quality", name: "文档质量检查", enabled: true, frequency: "0 9 * * 1" },
  { id: "cleanup", name: "过期数据清理", enabled: false, frequency: "0 3 * * 0" },
];

const FREQUENCY_LABELS: Record<string, string> = {
  "*/15 * * * *": "每 15 分钟",
  "0 */6 * * *": "每 6 小时",
  "0 9 * * 1": "每周一 9:00",
  "0 3 * * 0": "每周日 3:00",
};

export default function FreshnessSettings() {
  const [rules, setRules] = useState<FreshnessRule[]>(DEFAULT_RULES);

  const toggleRule = (id: string) => {
    setRules((prev) =>
      prev.map((r) => (r.id === id ? { ...r, enabled: !r.enabled } : r))
    );
  };

  return (
    <section className="settings__section">
      <h3 className="settings__section-title">保鲜规则配置</h3>
      <p className="settings__hint">各类保鲜任务的频率和策略</p>

      <div className="settings__list">
        {rules.map((r) => (
          <div key={r.id} className="settings__list-item">
            <label className="settings__toggle">
              <input
                type="checkbox"
                checked={r.enabled}
                onChange={() => toggleRule(r.id)}
              />
              <span className="settings__toggle-slider" />
            </label>
            <span>{r.name}</span>
            <span className="settings__hint">
              {FREQUENCY_LABELS[r.frequency] ?? r.frequency}
            </span>
          </div>
        ))}
      </div>
    </section>
  );
}
