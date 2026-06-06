import ModelSettings from "../settings/ModelSettings";
import CollectorSettings from "../settings/CollectorSettings";
import StorageSettings from "../settings/StorageSettings";
import ShortcutSettings from "../settings/ShortcutSettings";
import FreshnessSettings from "../settings/FreshnessSettings";
import ReportSettings from "../settings/ReportSettings";

export default function SettingsView() {
  return (
    <div className="view settings-view">
      <div className="view__header">
        <h2 className="view__title">设置</h2>
      </div>
      <p className="view__description">
        配置模型参数、采集器、存储路径和系统行为。
      </p>

      <ModelSettings />
      <CollectorSettings />
      <StorageSettings />
      <ShortcutSettings />
      <FreshnessSettings />
      <ReportSettings />
    </div>
  );
}
