import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useKeyboardShortcuts, SHORTCUTS, formatShortcutHint } from "./useKeyboardShortcuts";

describe("useKeyboardShortcuts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should call handler when matching shortcut is pressed", () => {
    const handler = vi.fn();
    renderHook(() => useKeyboardShortcuts([
      { ...SHORTCUTS.COMMAND_PALETTE, handler },
    ]));

    // jsdom doesn't set metaKey, use ctrlKey (matchesShortcut uses ctrlKey on non-Mac)
    const event = new KeyboardEvent("keydown", {
      key: "k",
      ctrlKey: true,
    });
    window.dispatchEvent(event);

    expect(handler).toHaveBeenCalledOnce();
  });

  it("should not call handler when key does not match", () => {
    const handler = vi.fn();
    renderHook(() => useKeyboardShortcuts([
      { ...SHORTCUTS.COMMAND_PALETTE, handler },
    ]));

    const event = new KeyboardEvent("keydown", {
      key: "j",
      metaKey: true,
    });
    window.dispatchEvent(event);

    expect(handler).not.toHaveBeenCalled();
  });

  it("should skip shortcuts in input elements", () => {
    const handler = vi.fn();
    renderHook(() => useKeyboardShortcuts([
      { ...SHORTCUTS.VIEW_EVENTS, handler },
    ]));

    const input = document.createElement("input");
    document.body.appendChild(input);
    input.focus();

    const event = new KeyboardEvent("keydown", {
      key: "2",
      metaKey: true,
      bubbles: true,
    });
    input.dispatchEvent(event);

    expect(handler).not.toHaveBeenCalled();
    document.body.removeChild(input);
  });

  it("should call handler in input when allowInInput is true", () => {
    const handler = vi.fn();
    renderHook(() => useKeyboardShortcuts([
      { ...SHORTCUTS.ESCAPE, handler, allowInInput: true },
    ]));

    const input = document.createElement("input");
    document.body.appendChild(input);
    input.focus();

    const event = new KeyboardEvent("keydown", {
      key: "Escape",
      bubbles: true,
    });
    input.dispatchEvent(event);

    expect(handler).toHaveBeenCalled();
    document.body.removeChild(input);
  });
});

describe("SHORTCUTS", () => {
  it("should have correct key mappings", () => {
    expect(SHORTCUTS.COMMAND_PALETTE.key).toBe("k");
    expect(SHORTCUTS.VIEW_DASHBOARD.key).toBe("1");
    expect(SHORTCUTS.VIEW_EVENTS.key).toBe("2");
    expect(SHORTCUTS.VIEW_TASKS.key).toBe("3");
    expect(SHORTCUTS.VIEW_SETTINGS.key).toBe(",");
  });
});

describe("formatShortcutHint", () => {
  it("should format meta key shortcut on Mac", () => {
    vi.spyOn(navigator, "platform", "get").mockReturnValue("MacIntel");
    const hint = formatShortcutHint({ key: "k", metaKey: true });
    expect(hint).toBe("⌘K");
  });

  it("should format ctrl key shortcut on non-Mac", () => {
    vi.spyOn(navigator, "platform", "get").mockReturnValue("Win32");
    const hint = formatShortcutHint({ key: "k", metaKey: true });
    expect(hint).toBe("Ctrl+K");
  });

  it("should format shift modifier", () => {
    vi.spyOn(navigator, "platform", "get").mockReturnValue("MacIntel");
    const hint = formatShortcutHint({ key: "n", metaKey: true, shiftKey: true });
    expect(hint).toBe("⌘⇧N");
  });

  it("should map special key names", () => {
    vi.spyOn(navigator, "platform", "get").mockReturnValue("MacIntel");
    expect(formatShortcutHint({ key: "Escape" })).toBe("Esc");
    expect(formatShortcutHint({ key: "Enter" })).toBe("↵");
  });
});
