import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import TasksView from "./TasksView";

describe("TasksView", () => {
  it("renders without crashing (D1-04)", () => {
    const { container } = render(<TasksView />);
    expect(container).toBeTruthy();
  });

  it("displays the view title", () => {
    render(<TasksView />);
    expect(screen.getByText("任务管理")).toBeInTheDocument();
  });

  it("shows the task creation form", () => {
    render(<TasksView />);
    expect(screen.getByPlaceholderText("新任务标题...")).toBeInTheDocument();
    expect(screen.getByText("添加")).toBeInTheDocument();
  });

  it("displays initial tasks", () => {
    render(<TasksView />);
    expect(screen.getByText("整理周报")).toBeInTheDocument();
    expect(screen.getByText("Review PR #42")).toBeInTheDocument();
  });

  it("shows task count", () => {
    render(<TasksView />);
    expect(screen.getByText(/个任务/)).toBeInTheDocument();
  });

  it("displays scheduled tasks section", () => {
    render(<TasksView />);
    expect(screen.getByText("定时任务")).toBeInTheDocument();
    expect(screen.getByText("每日站会")).toBeInTheDocument();
  });
});
