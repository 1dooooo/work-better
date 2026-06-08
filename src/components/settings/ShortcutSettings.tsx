import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { RotateCcw } from "lucide-react";

interface Shortcut {
  id: string;
  label: string;
  key: string;
  modifiers: string[];
}

const DEFAULT_SHORTCUTS: Shortcut[] = [
  { id: "capture", label: "快速捕获", key: "Space", modifiers: ["cmd", "shift"] },
  { id: "search", label: "全局搜索", key: "K", modifiers: ["cmd"] },
  { id: "task", label: "新建任务", key: "N", modifiers: ["cmd", "shift"] },
];

export default function ShortcutSettings() {
  const [shortcuts] = useState<Shortcut[]>(DEFAULT_SHORTCUTS);
  const [editing, setEditing] = useState<string | null>(null);

  const formatKey = (s: Shortcut) =>
    [...s.modifiers.map((m) => (m === "cmd" ? "⌘" : m === "shift" ? "⇧" : m === "alt" ? "⌥" : "⌃")), s.key].join(" + ");

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
              >
                {formatKey(s)}
              </kbd>
              {editing === s.id && (
                <span className="text-xs text-muted-foreground">
                  按下新的快捷键组合...
                </span>
              )}
            </div>
          </div>
        ))}
      </div>

      <Button
        variant="outline"
        size="sm"
        onClick={() => setEditing(null)}
        className="gap-1.5"
      >
        <RotateCcw className="h-3.5 w-3.5" />
        恢复默认
      </Button>
    </div>
  );
}
