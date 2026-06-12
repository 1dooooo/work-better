import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import ShortcutSettings from "./ShortcutSettings";

describe("ShortcutSettings", () => {
  it("renders without crashing (D1-12)", () => {
    const { container } = render(<ShortcutSettings />);
    expect(container).toBeTruthy();
  });

  it("displays the section title", () => {
    render(<ShortcutSettings />);
    expect(screen.getByText("自定义全局快捷键")).toBeInTheDocument();
  });

  it("shows default shortcuts", () => {
    render(<ShortcutSettings />);
    expect(screen.getByText("快速捕获")).toBeInTheDocument();
    expect(screen.getByText("全局搜索")).toBeInTheDocument();
    expect(screen.getByText("新建任务")).toBeInTheDocument();
  });

  it("shows the reset button", () => {
    render(<ShortcutSettings />);
    expect(screen.getByText("恢复默认")).toBeInTheDocument();
  });
});
