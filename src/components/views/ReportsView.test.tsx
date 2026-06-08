import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ReportsView from "./ReportsView";

describe("ReportsView", () => {
  it("renders without crashing (D1-06)", () => {
    const { container } = render(<ReportsView />);
    expect(container).toBeTruthy();
  });

  it("displays the view title", () => {
    render(<ReportsView />);
    expect(screen.getByText("报告")).toBeInTheDocument();
  });

  it("shows report type cards", () => {
    render(<ReportsView />);
    expect(screen.getByText("日报")).toBeInTheDocument();
    expect(screen.getByText("周报")).toBeInTheDocument();
    expect(screen.getByText("月报")).toBeInTheDocument();
  });

  it("shows description when a report card is clicked", async () => {
    render(<ReportsView />);
    await userEvent.click(screen.getByText("日报"));
    expect(
      screen.getByText(/自动生成每日工作总结/),
    ).toBeInTheDocument();
  });

  it("shows the upcoming feature placeholder", () => {
    render(<ReportsView />);
    const elements = screen.getAllByText(/即将推出/);
    expect(elements.length).toBeGreaterThan(0);
  });
});
