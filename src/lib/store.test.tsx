import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState, useCallback } from "react";

// ─── Test Helpers ─────────────────────────────────────────────────────

// A simple component that demonstrates view switching pattern (D3-01)
function ViewSwitcher() {
  const [activeView, setActiveView] = useState<"events" | "tasks" | "settings">("events");

  return (
    <div>
      <div data-testid="current-view">{activeView}</div>
      <button onClick={() => setActiveView("events")}>Events</button>
      <button onClick={() => setActiveView("tasks")}>Tasks</button>
      <button onClick={() => setActiveView("settings")}>Settings</button>
    </div>
  );
}

// A simple component that demonstrates form state pattern (D3-02)
function FormStateDemo() {
  const [formData, setFormData] = useState({ name: "", email: "" });
  const [submitted, setSubmitted] = useState(false);

  const handleSubmit = useCallback(() => {
    if (formData.name && formData.email) {
      setSubmitted(true);
    }
  }, [formData]);

  const resetForm = useCallback(() => {
    setFormData({ name: "", email: "" });
    setSubmitted(false);
  }, []);

  return (
    <div>
      <input
        placeholder="Name"
        value={formData.name}
        onChange={(e) => setFormData((prev) => ({ ...prev, name: e.target.value }))}
      />
      <input
        placeholder="Email"
        value={formData.email}
        onChange={(e) => setFormData((prev) => ({ ...prev, email: e.target.value }))}
      />
      <button onClick={handleSubmit}>Submit</button>
      <button onClick={resetForm}>Reset</button>
      {submitted && <div data-testid="submitted">Submitted</div>}
      <div data-testid="form-data">{JSON.stringify(formData)}</div>
    </div>
  );
}

// A simple component that demonstrates window toggle pattern (D3-03)
function WindowToggleDemo() {
  const [isOpen, setIsOpen] = useState(false);
  const [count, setCount] = useState(0);

  const toggle = useCallback(() => {
    setIsOpen((prev) => !prev);
  }, []);

  const increment = useCallback(() => {
    setCount((prev) => prev + 1);
  }, []);

  return (
    <div>
      <button onClick={toggle}>{isOpen ? "Close" : "Open"}</button>
      <button onClick={increment}>Increment</button>
      {isOpen && <div data-testid="window-content">Window is open</div>}
      <div data-testid="count">{count}</div>
    </div>
  );
}

// ─── D3-01: View Switching ────────────────────────────────────────────

describe("State Management: View Switching (D3-01)", () => {
  it("starts with default view", () => {
    render(<ViewSwitcher />);
    expect(screen.getByTestId("current-view")).toHaveTextContent("events");
  });

  it("switches to tasks view", async () => {
    render(<ViewSwitcher />);
    await userEvent.click(screen.getByText("Tasks"));
    expect(screen.getByTestId("current-view")).toHaveTextContent("tasks");
  });

  it("switches to settings view", async () => {
    render(<ViewSwitcher />);
    await userEvent.click(screen.getByText("Settings"));
    expect(screen.getByTestId("current-view")).toHaveTextContent("settings");
  });

  it("switches back to events view", async () => {
    render(<ViewSwitcher />);
    await userEvent.click(screen.getByText("Tasks"));
    expect(screen.getByTestId("current-view")).toHaveTextContent("tasks");
    await userEvent.click(screen.getByText("Events"));
    expect(screen.getByTestId("current-view")).toHaveTextContent("events");
  });

  it("handles multiple rapid switches", async () => {
    render(<ViewSwitcher />);
    await userEvent.click(screen.getByText("Tasks"));
    await userEvent.click(screen.getByText("Settings"));
    await userEvent.click(screen.getByText("Events"));
    expect(screen.getByTestId("current-view")).toHaveTextContent("events");
  });
});

// ─── D3-02: Form State ────────────────────────────────────────────────

describe("State Management: Form State (D3-02)", () => {
  it("starts with empty form", () => {
    render(<FormStateDemo />);
    const formData = JSON.parse(screen.getByTestId("form-data").textContent || "{}");
    expect(formData).toEqual({ name: "", email: "" });
  });

  it("updates name field", async () => {
    render(<FormStateDemo />);
    const nameInput = screen.getByPlaceholderText("Name");
    await userEvent.type(nameInput, "John");
    const formData = JSON.parse(screen.getByTestId("form-data").textContent || "{}");
    expect(formData.name).toBe("John");
  });

  it("updates email field", async () => {
    render(<FormStateDemo />);
    const emailInput = screen.getByPlaceholderText("Email");
    await userEvent.type(emailInput, "john@example.com");
    const formData = JSON.parse(screen.getByTestId("form-data").textContent || "{}");
    expect(formData.email).toBe("john@example.com");
  });

  it("submits when both fields are filled", async () => {
    render(<FormStateDemo />);
    await userEvent.type(screen.getByPlaceholderText("Name"), "John");
    await userEvent.type(screen.getByPlaceholderText("Email"), "john@example.com");
    await userEvent.click(screen.getByText("Submit"));
    expect(screen.getByTestId("submitted")).toBeInTheDocument();
  });

  it("does not submit when fields are empty", async () => {
    render(<FormStateDemo />);
    await userEvent.click(screen.getByText("Submit"));
    expect(screen.queryByTestId("submitted")).not.toBeInTheDocument();
  });

  it("resets form state", async () => {
    render(<FormStateDemo />);
    await userEvent.type(screen.getByPlaceholderText("Name"), "John");
    await userEvent.type(screen.getByPlaceholderText("Email"), "john@example.com");
    await userEvent.click(screen.getByText("Submit"));
    expect(screen.getByTestId("submitted")).toBeInTheDocument();

    await userEvent.click(screen.getByText("Reset"));
    expect(screen.queryByTestId("submitted")).not.toBeInTheDocument();
    const formData = JSON.parse(screen.getByTestId("form-data").textContent || "{}");
    expect(formData).toEqual({ name: "", email: "" });
  });

  it("maintains independent field state", async () => {
    render(<FormStateDemo />);
    await userEvent.type(screen.getByPlaceholderText("Name"), "John");
    const formData = JSON.parse(screen.getByTestId("form-data").textContent || "{}");
    expect(formData.name).toBe("John");
    expect(formData.email).toBe("");
  });
});

// ─── D3-03: Window Toggle ─────────────────────────────────────────────

describe("State Management: Window Toggle (D3-03)", () => {
  it("starts with window closed", () => {
    render(<WindowToggleDemo />);
    expect(screen.queryByTestId("window-content")).not.toBeInTheDocument();
  });

  it("opens window on toggle", async () => {
    render(<WindowToggleDemo />);
    await userEvent.click(screen.getByText("Open"));
    expect(screen.getByTestId("window-content")).toBeInTheDocument();
    expect(screen.getByText("Close")).toBeInTheDocument();
  });

  it("closes window on second toggle", async () => {
    render(<WindowToggleDemo />);
    await userEvent.click(screen.getByText("Open"));
    expect(screen.getByTestId("window-content")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Close"));
    expect(screen.queryByTestId("window-content")).not.toBeInTheDocument();
  });

  it("preserves count state when toggling window", async () => {
    render(<WindowToggleDemo />);
    await userEvent.click(screen.getByText("Increment"));
    await userEvent.click(screen.getByText("Increment"));
    expect(screen.getByTestId("count")).toHaveTextContent("2");

    await userEvent.click(screen.getByText("Open"));
    expect(screen.getByTestId("count")).toHaveTextContent("2");

    await userEvent.click(screen.getByText("Close"));
    expect(screen.getByTestId("count")).toHaveTextContent("2");
  });

  it("handles rapid toggling", async () => {
    render(<WindowToggleDemo />);
    await userEvent.click(screen.getByText("Open"));
    await userEvent.click(screen.getByText("Close"));
    await userEvent.click(screen.getByText("Open"));
    expect(screen.getByTestId("window-content")).toBeInTheDocument();
  });

  it("maintains counter independence from window state", async () => {
    render(<WindowToggleDemo />);
    await userEvent.click(screen.getByText("Increment"));
    await userEvent.click(screen.getByText("Open"));
    await userEvent.click(screen.getByText("Increment"));
    await userEvent.click(screen.getByText("Close"));
    await userEvent.click(screen.getByText("Increment"));
    expect(screen.getByTestId("count")).toHaveTextContent("3");
  });
});
