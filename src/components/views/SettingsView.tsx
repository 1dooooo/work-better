import ModelSettings from "../settings/ModelSettings";
import CollectorSettings from "../settings/CollectorSettings";
import StorageSettings from "../settings/StorageSettings";

export default function SettingsView() {
  return (
    <div className="view settings-view">
      <div className="view__header">
        <h2 className="view__title">设置</h2>
      </div>
      <p className="view__description">
        配置模型参数、采集器和存储路径。
      </p>

      <ModelSettings />
      <CollectorSettings />
      <StorageSettings />
    </div>
  );
}
