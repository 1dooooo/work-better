import { useState, useEffect, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { RotateCcw, Loader2, Check } from "lucide-react";
import {
  getShortcutConfig,
  saveShortcutConfig,
  type ShortcutConfig,
} from "@/lib/tauri";

const DEFAULT_SHORTCUTS: ShortcutConfig[] = [
  { id: "capture", label: "快速捕获", key: "Space", modifiers: ["cmd", "shift"] },
  { id: "screenshot", label: "截图速记", key: "S", modifiers: ["cmd", "shift"] },
  { id: "search", label: "全局搜索", key: "K", modifiers: ["cmd"] },
  { id: "task", label: "新建任务", key: "N", modifiers: ["cmd", "shift"] },
];

export default function ShortcutSettings() {
  const [shortcuts, setShortcuts] = useState<ShortcutConfig[]>(DEFAULT_SHORTCUTS);
  const [editing, setEditing] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  // 加载已保存的快捷键配置
  useEffect(() => {
    getShortcutConfig()
      .then((config) => {
        if (config.length > 0) {
          setShortcuts(config);
        }
      })
      .catch(() => {
        // 使用默认配置
      });
  }, []);

  const formatKey = (s: ShortcutConfig) =>
    [
      ...s.modifiers.map((m) =>
        m === "cmd" ? "⌘" : m === "shift" ? "⇧" : m === "alt" ? "⌥" : "⌃"
      ),
      s.key,
    ].join(" + ");

  const handleKeyDown = useCallback(
    (e: KeyboardEvent, shortcutId: string) => {
      e.preventDefault();
      e.stopPropagation();

      // 忽略单独的修饰键
      if (["Meta", "Shift", "Alt", "Control"].includes(e.key)) return;

      const modifiers: string[] = [];
      if (e.metaKey) modifiers.push("cmd");
      if (e.shiftKey) modifiers.push("shift");
      if (e.altKey) modifiers.push("alt");
      if (e.ctrlKey) modifiers.push("ctrl");

      const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;

      setShortcuts((prev) =>
        prev.map((s) =>
          s.id === shortcutId ? { ...s, key, modifiers } : s
        )
      );
      setEditing(null);
    },
    []
  );

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveShortcutConfig(shortcuts);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("保存快捷键配置失败:", err);
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setShortcuts(DEFAULT_SHORTCUTS);
  };

  return (
    <div className="space-y-4">
      <p className="text-xs text-muted-foreground">自定义全局快捷键</p>

      <div className="space-y-1">
        {shortcuts.map((s) => (
          <div
            key={s.id}
            className="flex items-center justify-between rounded-md border border-border px-3 py-2"
          >
            <span className="text-sm">{s.label}</span>
            <div className="flex items-center gap-2">
              <kbd
                className="cursor-pointer rounded border border-border bg-muted px-2 py-0.5 font-mono text-xs hover:bg-accent"
                onClick={() => setEditing(editing === s.id ? null : s.id)}
                onKeyDown={(e) => {
                  if (editing === s.id) {
                    handleKeyDown(e.nativeEvent, s.id);
                  }
                }}
                tabIndex={editing === s.id ? 0 : -1}
                role="button"
                aria-label={`修改 ${s.label} 快捷键`}
              >
                {formatKey(s)}
              </kbd>
              {editing === s.id && (
                <span className="text-xs text-muted-foreground animate-pulse">
                  按下新的快捷键组合...
                </span>
              )}
            </div>
          </div>
        ))}
      </div>

      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleReset}
          className="gap-1.5"
        >
          <RotateCcw className="h-3.5 w-3.5" />
          恢复默认
        </Button>
        <Button
          size="sm"
          onClick={handleSave}
          disabled={saving}
          className="gap-1.5"
        >
          {saving ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : saved ? (
            <Check className="h-3.5 w-3.5" />
          ) : null}
          {saving ? "保存中..." : saved ? "已保存" : "保存配置"}
        </Button>
      </div>
    </div>
  );
}
