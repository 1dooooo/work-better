import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import ReportSettings from "./ReportSettings";

describe("ReportSettings", () => {
  it("renders without crashing (D1-14)", () => {
    const { container } = render(<ReportSettings />);
    expect(container).toBeTruthy();
  });

  it("displays the section title", () => {
    render(<ReportSettings />);
    expect(screen.getByText("报告定时任务和确认策略")).toBeInTheDocument();
  });

  it("shows all report types", () => {
    render(<ReportSettings />);
    expect(screen.getByText("日报")).toBeInTheDocument();
    expect(screen.getByText("周报")).toBeInTheDocument();
    expect(screen.getByText("月报")).toBeInTheDocument();
    expect(screen.getByText("季报")).toBeInTheDocument();
  });

  it("shows schedule labels", () => {
    render(<ReportSettings />);
    expect(screen.getByText("工作日 18:00")).toBeInTheDocument();
    expect(screen.getByText("每周五 17:00")).toBeInTheDocument();
  });

  it("shows auto-confirm labels", () => {
    render(<ReportSettings />);
    const autoConfirmLabels = screen.getAllByText("自动确认");
    expect(autoConfirmLabels.length).toBeGreaterThan(0);
  });
});
