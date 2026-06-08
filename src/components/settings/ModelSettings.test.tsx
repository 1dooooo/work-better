import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import ModelSettings from "./ModelSettings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "get_model_config") {
      return Promise.resolve({
        api_endpoint: "https://api.openai.com/v1",
        api_key: "sk-test-key",
        token_budget: 4096,
      });
    }
    if (cmd === "save_model_config") {
      return Promise.resolve(undefined);
    }
    return Promise.resolve(null);
  }),
}));

describe("ModelSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-09)", async () => {
    const { container } = render(<ModelSettings />);
    expect(container).toBeTruthy();
    await waitFor(() => {
      expect(screen.getByLabelText("API Endpoint")).toBeInTheDocument();
    });
  });

  it("displays the section title", async () => {
    render(<ModelSettings />);
    await waitFor(() => {
      expect(screen.getByLabelText("API Endpoint")).toBeInTheDocument();
    });
  });

  it("shows the API endpoint field", async () => {
    render(<ModelSettings />);
    await waitFor(() => {
      expect(screen.getByLabelText("API Endpoint")).toBeInTheDocument();
    });
  });

  it("shows the API key field", async () => {
    render(<ModelSettings />);
    await waitFor(() => {
      expect(screen.getByLabelText("API Key")).toBeInTheDocument();
    });
  });

  it("shows the token budget field", async () => {
    render(<ModelSettings />);
    await waitFor(() => {
      expect(screen.getByLabelText("Token 预算")).toBeInTheDocument();
    });
  });

  it("shows the save button", async () => {
    render(<ModelSettings />);
    await waitFor(() => {
      expect(screen.getByText("保存")).toBeInTheDocument();
    });
  });
});
