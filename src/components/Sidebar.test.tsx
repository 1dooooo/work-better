import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Sidebar from "./Sidebar";

describe("Sidebar", () => {
  const defaultProps = {
    activeView: "events" as const,
    onViewChange: vi.fn(),
    unprocessedCount: 0,
  };

  it("renders without crashing (D1-02)", () => {
    const { container } = render(<Sidebar {...defaultProps} />);
    expect(container).toBeTruthy();
  });

  it("displays all navigation items", () => {
    render(<Sidebar {...defaultProps} />);
    expect(screen.getByText("事件")).toBeInTheDocument();
    expect(screen.getByText("任务")).toBeInTheDocument();
    expect(screen.getByText("时间线")).toBeInTheDocument();
    expect(screen.getByText("报告")).toBeInTheDocument();
    expect(screen.getByText("设置")).toBeInTheDocument();
  });

  it("highlights the active view", () => {
    render(<Sidebar {...defaultProps} activeView="tasks" />);
    const taskButton = screen.getByText("任务").closest("button");
    expect(taskButton?.className).toContain("sidebar__item--active");
  });

  it("calls onViewChange when a nav item is clicked", async () => {
    const onViewChange = vi.fn();
    render(<Sidebar {...defaultProps} onViewChange={onViewChange} />);
    await userEvent.click(screen.getByText("设置"));
    expect(onViewChange).toHaveBeenCalledWith("settings");
  });

  it("shows badge when unprocessedCount > 0", () => {
    render(<Sidebar {...defaultProps} unprocessedCount={5} />);
    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("hides badge when unprocessedCount is 0", () => {
    render(<Sidebar {...defaultProps} unprocessedCount={0} />);
    expect(screen.queryByText("0")).not.toBeInTheDocument();
  });

  it("displays version info", () => {
    render(<Sidebar {...defaultProps} />);
    expect(screen.getByText("v0.1.0")).toBeInTheDocument();
  });
});
