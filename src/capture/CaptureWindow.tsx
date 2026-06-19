import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

export default function CaptureWindow() {
  const [text, setText] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [status, setStatus] = useState<"idle" | "success" | "error">("idle");
  const [imageData, setImageData] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  // 图片粘贴支持
  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    const items = Array.from(e.clipboardData.items);
    const imageItem = items.find((item) => item.type.startsWith("image/"));
    if (imageItem) {
      e.preventDefault();
      const blob = imageItem.getAsFile();
      if (blob) {
        const reader = new FileReader();
        reader.onload = () => {
          setImageData(reader.result as string);
        };
        reader.readAsDataURL(blob);
      }
    }
  }, []);

  const handleSubmit = useCallback(async () => {
    const trimmed = text.trim();
    if (!trimmed || submitting) return;

    setSubmitting(true);
    setStatus("idle");

    try {
      await invoke("trigger_manual_capture", { text: trimmed });
      setText("");
      setStatus("success");
      // 成功后自动隐藏窗口
      setTimeout(() => {
        invoke("hide_capture_window");
        setStatus("idle");
      }, 800);
    } catch (err) {
      console.error("Capture failed:", err);
      setStatus("error");
    } finally {
      setSubmitting(false);
    }
  }, [text, submitting]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        handleSubmit();
      }
      if (e.key === "Escape") {
        invoke("hide_capture_window");
      }
    },
    [handleSubmit],
  );

  const handleClose = useCallback(() => {
    invoke("hide_capture_window");
  }, []);

  return (
    <div className="flex h-full flex-col bg-background text-foreground select-none overflow-hidden">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-border">
        <span className="text-sm font-medium">快速捕获</span>
        <button
          className="flex h-6 w-6 items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
          onClick={handleClose}
          aria-label="关闭"
        >
          ✕
        </button>
      </header>

      {/* Input */}
      <textarea
        ref={textareaRef}
        className="flex-1 px-4 py-3 bg-transparent text-sm resize-none outline-none placeholder:text-muted-foreground"
        placeholder="记录一条想法、笔记或任务...支持粘贴图片"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        onPaste={handlePaste}
        disabled={submitting}
        rows={5}
      />

      {/* Image Preview */}
      {imageData && (
        <div className="relative mx-4 mb-3">
          <img
            src={imageData}
            alt="粘贴的图片"
            className="max-h-32 rounded-md object-cover"
          />
          <button
            className="absolute top-1 right-1 flex h-5 w-5 items-center justify-center rounded-full bg-background/80 text-muted-foreground hover:text-foreground transition-colors"
            onClick={() => setImageData(null)}
          >
            ✕
          </button>
        </div>
      )}

      {/* Footer */}
      <footer className="flex items-center justify-between px-4 py-3 border-t border-border">
        <span className="text-xs text-muted-foreground">
          ⌘+Enter 提交 · Esc 关闭
        </span>
        <button
          className={cn(
            "px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
            "bg-primary text-primary-foreground hover:bg-primary/90",
            "disabled:opacity-50 disabled:cursor-not-allowed"
          )}
          onClick={handleSubmit}
          disabled={!text.trim() || submitting}
        >
          {submitting ? "提交中..." : "提交"}
        </button>
      </footer>

      {/* Toast */}
      {status === "success" && (
        <div className="absolute bottom-16 left-1/2 -translate-x-1/2 px-3 py-1.5 rounded-md bg-success text-success-foreground text-sm font-medium">
          已捕获
        </div>
      )}
      {status === "error" && (
        <div className="absolute bottom-16 left-1/2 -translate-x-1/2 px-3 py-1.5 rounded-md bg-destructive text-destructive-foreground text-sm font-medium">
          捕获失败，请重试
        </div>
      )}
    </div>
  );
}
