import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import CaptureWindow from "./CaptureWindow";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

describe("CaptureWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-08)", () => {
    const { container } = render(<CaptureWindow />);
    expect(container).toBeTruthy();
  });

  it("displays the capture title", () => {
    render(<CaptureWindow />);
    expect(screen.getByText("快速捕获")).toBeInTheDocument();
  });

  it("shows the textarea input", () => {
    render(<CaptureWindow />);
    expect(
      screen.getByPlaceholderText(/记录一条想法/),
    ).toBeInTheDocument();
  });

  it("shows the submit button", () => {
    render(<CaptureWindow />);
    expect(screen.getByText("提交")).toBeInTheDocument();
  });

  it("shows the close button", () => {
    render(<CaptureWindow />);
    expect(screen.getByLabelText("关闭")).toBeInTheDocument();
  });

  it("shows keyboard hints", () => {
    render(<CaptureWindow />);
    expect(screen.getByText(/Enter 提交/)).toBeInTheDocument();
  });

  it("disables submit when input is empty", () => {
    render(<CaptureWindow />);
    const submitBtn = screen.getByText("提交");
    expect(submitBtn).toBeDisabled();
  });
});
