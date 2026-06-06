import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./capture-window.css";

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
    <div className="capture">
      <header className="capture__header">
        <span className="capture__title">快速捕获</span>
        <button
          className="capture__close"
          onClick={handleClose}
          aria-label="关闭"
        >
          ✕
        </button>
      </header>

      <textarea
        ref={textareaRef}
        className="capture__input"
        placeholder="记录一条想法、笔记或任务...支持粘贴图片"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        onPaste={handlePaste}
        disabled={submitting}
        rows={5}
      />

      {imageData && (
        <div className="capture__image-preview">
          <img src={imageData} alt="粘贴的图片" />
          <button className="capture__image-remove" onClick={() => setImageData(null)}>✕</button>
        </div>
      )}

      <footer className="capture__footer">
        <span className="capture__hint">⌘+Enter 提交 · Esc 关闭</span>
        <button
          className="capture__submit"
          onClick={handleSubmit}
          disabled={!text.trim() || submitting}
        >
          {submitting ? "提交中..." : "提交"}
        </button>
      </footer>

      {status === "success" && (
        <div className="capture__toast capture__toast--success">
          已捕获
        </div>
      )}
      {status === "error" && (
        <div className="capture__toast capture__toast--error">
          捕获失败，请重试
        </div>
      )}
    </div>
  );
}
