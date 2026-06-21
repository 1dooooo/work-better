import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  formatRelativeTime,
  formatDate,
  getStatusColor,
  getPriorityLabel,
  getPriorityOrder,
  getViewIcon,
  getStatusIcon,
  truncateText,
  truncateWords,
  getContentString,
  getEventType,
} from "./utils";

describe("Time Formatting (D2-01)", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-06T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("formats 'just now' for recent timestamps", () => {
    const recent = new Date("2026-06-06T11:59:30Z").toISOString();
    expect(formatRelativeTime(recent)).toBe("刚刚");
  });

  it("formats minutes ago", () => {
    const fiveMinAgo = new Date("2026-06-06T11:55:00Z").toISOString();
    expect(formatRelativeTime(fiveMinAgo)).toBe("5 分钟前");
  });

  it("formats hours ago", () => {
    const twoHoursAgo = new Date("2026-06-06T10:00:00Z").toISOString();
    expect(formatRelativeTime(twoHoursAgo)).toBe("2 小时前");
  });

  it("formats days ago", () => {
    const threeDaysAgo = new Date("2026-06-03T12:00:00Z").toISOString();
    expect(formatRelativeTime(threeDaysAgo)).toBe("3 天前");
  });

  it("formats future timestamps", () => {
    const inFiveMin = new Date("2026-06-06T12:05:00Z").toISOString();
    expect(formatRelativeTime(inFiveMin)).toBe("5 分钟后");
  });

  it("formats dates older than a week as locale date", () => {
    const oldDate = new Date("2026-05-01T12:00:00Z").toISOString();
    const result = formatRelativeTime(oldDate);
    // Should be a locale date string, not relative
    expect(result).not.toContain("前");
    expect(result).not.toContain("后");
  });

  it("accepts Date objects", () => {
    const date = new Date("2026-06-06T11:55:00Z");
    expect(formatRelativeTime(date)).toBe("5 分钟前");
  });

  it("formats date with formatDate", () => {
    const date = new Date("2026-06-06T12:00:00Z");
    const result = formatDate(date);
    expect(result).toBeTruthy();
    expect(typeof result).toBe("string");
  });

  it("formats string date with formatDate", () => {
    const result = formatDate("2026-06-06T12:00:00Z");
    expect(result).toBeTruthy();
  });
});

describe("Status Color Mapping (D2-02)", () => {
  it("returns correct color for todo status", () => {
    expect(getStatusColor("todo")).toBe("#6b7280");
  });

  it("returns correct color for in_progress status", () => {
    expect(getStatusColor("in_progress")).toBe("#3b82f6");
  });

  it("returns correct color for done status", () => {
    expect(getStatusColor("done")).toBe("#10b981");
  });

  it("returns correct color for error status", () => {
    expect(getStatusColor("error")).toBe("#ef4444");
  });

  it("returns correct color for success status", () => {
    expect(getStatusColor("success")).toBe("#10b981");
  });

  it("returns correct color for idle status", () => {
    expect(getStatusColor("idle")).toBe("#9ca3af");
  });

  it("returns correct color for loading status", () => {
    expect(getStatusColor("loading")).toBe("#f59e0b");
  });

  it("returns fallback color for unknown status", () => {
    expect(getStatusColor("unknown" as any)).toBe("#9ca3af");
  });
});

describe("Priority Label Mapping (D2-03)", () => {
  it("returns '高' for high priority", () => {
    expect(getPriorityLabel("high")).toBe("高");
  });

  it("returns '中' for medium priority", () => {
    expect(getPriorityLabel("medium")).toBe("中");
  });

  it("returns '低' for low priority", () => {
    expect(getPriorityLabel("low")).toBe("低");
  });

  it("returns fallback for unknown priority", () => {
    expect(getPriorityLabel("unknown" as any)).toBe("未知");
  });

  it("returns correct sort order for high", () => {
    expect(getPriorityOrder("high")).toBe(0);
  });

  it("returns correct sort order for medium", () => {
    expect(getPriorityOrder("medium")).toBe(1);
  });

  it("returns correct sort order for low", () => {
    expect(getPriorityOrder("low")).toBe(2);
  });

  it("returns fallback order for unknown priority", () => {
    expect(getPriorityOrder("unknown" as any)).toBe(99);
  });
});

describe("Icon Mapping (D2-04)", () => {
  it("returns correct icon for events view", () => {
    expect(getViewIcon("events")).toBe("📋");
  });

  it("returns correct icon for tasks view", () => {
    expect(getViewIcon("tasks")).toBe("✅");
  });

  it("returns correct icon for timeline view", () => {
    expect(getViewIcon("timeline")).toBe("📅");
  });

  it("returns correct icon for reports view", () => {
    expect(getViewIcon("reports")).toBe("📊");
  });

  it("returns correct icon for settings view", () => {
    expect(getViewIcon("settings")).toBe("⚙");
  });

  it("returns fallback icon for unknown view", () => {
    expect(getViewIcon("unknown")).toBe("📄");
  });

  it("returns correct icon for todo status", () => {
    expect(getStatusIcon("todo")).toBe("○");
  });

  it("returns correct icon for in_progress status", () => {
    expect(getStatusIcon("in_progress")).toBe("◐");
  });

  it("returns correct icon for done status", () => {
    expect(getStatusIcon("done")).toBe("●");
  });

  it("returns correct icon for error status", () => {
    expect(getStatusIcon("error")).toBe("✕");
  });

  it("returns correct icon for success status", () => {
    expect(getStatusIcon("success")).toBe("✓");
  });

  it("returns fallback icon for unknown status", () => {
    expect(getStatusIcon("unknown")).toBe("◦");
  });
});

describe("Text Truncation (D2-05)", () => {
  describe("truncateText", () => {
    it("returns original text if shorter than maxLength", () => {
      expect(truncateText("hello", 10)).toBe("hello");
    });

    it("returns original text if equal to maxLength", () => {
      expect(truncateText("hello", 5)).toBe("hello");
    });

    it("truncates and adds ellipsis when text exceeds maxLength", () => {
      expect(truncateText("hello world", 5)).toBe("hell…");
    });

    it("returns empty string when maxLength is 0", () => {
      expect(truncateText("hello", 0)).toBe("");
    });

    it("returns ellipsis when maxLength is 1", () => {
      expect(truncateText("hello", 1)).toBe("…");
    });

    it("handles empty string", () => {
      expect(truncateText("", 5)).toBe("");
    });

    it("throws RangeError for negative maxLength", () => {
      expect(() => truncateText("hello", -1)).toThrow(RangeError);
    });
  });

  describe("truncateWords", () => {
    it("returns original text if word count is within limit", () => {
      expect(truncateWords("hello world", 3)).toBe("hello world");
    });

    it("returns original text if word count equals limit", () => {
      expect(truncateWords("hello world", 2)).toBe("hello world");
    });

    it("truncates to specified number of words", () => {
      expect(truncateWords("one two three four five", 3)).toBe("one two three…");
    });

    it("returns empty string when maxWords is 0", () => {
      expect(truncateWords("hello world", 0)).toBe("");
    });

    it("handles single word", () => {
      expect(truncateWords("hello", 1)).toBe("hello");
    });

    it("handles empty string", () => {
      expect(truncateWords("", 5)).toBe("");
    });

    it("throws RangeError for negative maxWords", () => {
      expect(() => truncateWords("hello", -1)).toThrow(RangeError);
    });
  });
});

describe("Content Helpers (D2-06)", () => {
  describe("getContentString", () => {
    it("returns string content as-is", () => {
      expect(getContentString("hello")).toBe("hello");
    });

    it("returns empty string as-is", () => {
      expect(getContentString("")).toBe("");
    });

    it("stringifies object content", () => {
      const obj = { key: "value" };
      expect(getContentString(obj)).toBe('{"key":"value"}');
    });

    it("stringifies array content", () => {
      expect(getContentString([1, 2, 3])).toBe("[1,2,3]");
    });

    it("stringifies null", () => {
      expect(getContentString(null)).toBe("null");
    });

    it("stringifies number", () => {
      expect(getContentString(42)).toBe("42");
    });
  });
});

describe("Event Type Mapping (D2-07)", () => {
  it("returns message config for 'message' type", () => {
    const result = getEventType("message");
    expect(result.label).toBe("MSG");
    expect(result.className).toContain("event-blue");
  });

  it("returns message config for Chinese '消息' type", () => {
    const result = getEventType("新消息");
    expect(result.label).toBe("MSG");
  });

  it("returns issue config for 'issue' type", () => {
    const result = getEventType("issue");
    expect(result.label).toBe("ISS");
    expect(result.className).toContain("event-amber");
  });

  it("returns PR config for 'pr' type", () => {
    const result = getEventType("pr");
    expect(result.label).toBe("PR");
    expect(result.className).toContain("event-green");
  });

  it("returns PR config for 'pull_request' type", () => {
    const result = getEventType("pull_request");
    expect(result.label).toBe("PR");
  });

  it("returns document config for 'document' type", () => {
    const result = getEventType("document");
    expect(result.label).toBe("DOC");
    expect(result.className).toContain("event-gray");
  });

  it("returns document config for Chinese '文档' type", () => {
    const result = getEventType("共享文档");
    expect(result.label).toBe("DOC");
  });

  it("returns default config for unknown type", () => {
    const result = getEventType("unknown");
    expect(result.label).toBe("EVT");
    expect(result.className).toContain("event-gray");
  });

  it("handles case-insensitive matching", () => {
    expect(getEventType("MESSAGE").label).toBe("MSG");
    expect(getEventType("Issue").label).toBe("ISS");
    expect(getEventType("PR").label).toBe("PR");
  });
});
