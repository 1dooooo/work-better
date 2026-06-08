import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import StorageSettings from "./StorageSettings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "get_storage_config") {
      return Promise.resolve({
        vault_path: "~/Documents/Obsidian",
        db_path: "~/.work-better/data.db",
      });
    }
    if (cmd === "save_storage_config") {
      return Promise.resolve(undefined);
    }
    return Promise.resolve(null);
  }),
}));

describe("StorageSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing (D1-11)", async () => {
    const { container } = render(<StorageSettings />);
    expect(container).toBeTruthy();
    await waitFor(() => {
      expect(screen.getByLabelText("Obsidian Vault 路径")).toBeInTheDocument();
    });
  });

  it("displays the section title", async () => {
    render(<StorageSettings />);
    await waitFor(() => {
      expect(screen.getByLabelText("Obsidian Vault 路径")).toBeInTheDocument();
    });
  });

  it("shows the vault path field", async () => {
    render(<StorageSettings />);
    await waitFor(() => {
      expect(screen.getByLabelText("Obsidian Vault 路径")).toBeInTheDocument();
    });
  });

  it("shows the db path field", async () => {
    render(<StorageSettings />);
    await waitFor(() => {
      expect(screen.getByLabelText("数据库路径")).toBeInTheDocument();
    });
  });

  it("shows the save button", async () => {
    render(<StorageSettings />);
    await waitFor(() => {
      expect(screen.getByText("保存")).toBeInTheDocument();
    });
  });
});
