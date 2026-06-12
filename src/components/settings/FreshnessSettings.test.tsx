import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import FreshnessSettings from "./FreshnessSettings";

describe("FreshnessSettings", () => {
  it("renders without crashing (D1-13)", () => {
    const { container } = render(<FreshnessSettings />);
    expect(container).toBeTruthy();
  });

  it("displays the section title", () => {
    render(<FreshnessSettings />);
    expect(screen.getByText("各类保鲜任务的频率和策略")).toBeInTheDocument();
  });

  it("shows all freshness rules", () => {
    render(<FreshnessSettings />);
    expect(screen.getByText("任务状态同步")).toBeInTheDocument();
    expect(screen.getByText("链接完整性检查")).toBeInTheDocument();
    expect(screen.getByText("文档质量检查")).toBeInTheDocument();
    expect(screen.getByText("过期数据清理")).toBeInTheDocument();
  });

  it("shows frequency labels", () => {
    render(<FreshnessSettings />);
    expect(screen.getByText("每 15 分钟")).toBeInTheDocument();
    expect(screen.getByText("每 6 小时")).toBeInTheDocument();
  });
});
