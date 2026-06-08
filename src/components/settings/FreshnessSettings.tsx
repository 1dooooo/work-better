import { useState } from "react";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

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
    <div className="space-y-4">
      <p className="text-xs text-muted-foreground">
        各类保鲜任务的频率和策略
      </p>

      <div className="space-y-1">
        {rules.map((r) => (
          <div
            key={r.id}
            className="flex items-center justify-between rounded-md border border-border px-3 py-2"
          >
            <div className="flex items-center gap-3">
              <Switch
                checked={r.enabled}
                onCheckedChange={() => toggleRule(r.id)}
              />
              <span className="text-sm">{r.name}</span>
            </div>
            <span className="text-xs text-muted-foreground">
              {FREQUENCY_LABELS[r.frequency] ?? r.frequency}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
