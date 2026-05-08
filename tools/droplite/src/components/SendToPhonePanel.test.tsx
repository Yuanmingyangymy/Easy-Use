import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "../test/test-utils";
import { SendToPhonePanel } from "./SendToPhonePanel";
import type { OutboxItem } from "../lib/types";

const api = vi.hoisted(() => ({
  addOutboxFile: vi.fn(),
  addOutboxText: vi.fn(),
  chooseOutboxFiles: vi.fn(),
  listOutboxItems: vi.fn()
}));

vi.mock("../lib/api", () => api);

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: vi.fn().mockResolvedValue(() => undefined)
  })
}));

function item(overrides: Partial<OutboxItem>): OutboxItem {
  return {
    id: "outbox-1",
    kind: "text",
    displayName: "Text",
    createdAt: "2026-05-09T00:00:00Z",
    status: "pending",
    content: "hello phone",
    ...overrides
  };
}

describe("SendToPhonePanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    api.listOutboxItems.mockResolvedValue([]);
    api.chooseOutboxFiles.mockResolvedValue([]);
  });

  it("renders the desktop send area", async () => {
    render(<SendToPhonePanel />);

    expect(await screen.findByRole("heading", { name: "Send to phone" })).toBeInTheDocument();
    expect(screen.getByLabelText("Type text to send to phone")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose files" })).toBeInTheDocument();
    expect(screen.getByLabelText("Drop files here to send to phone")).toBeInTheDocument();
  });

  it("rejects empty text without calling the command", () => {
    render(<SendToPhonePanel />);

    fireEvent.click(screen.getByRole("button", { name: "Send text" }));

    expect(screen.getByRole("alert")).toHaveTextContent("Empty text cannot be sent");
    expect(api.addOutboxText).not.toHaveBeenCalled();
  });

  it("adds text to the outbox and renders the item", async () => {
    api.addOutboxText.mockResolvedValue(item({ content: "hello phone" }));
    render(<SendToPhonePanel />);

    fireEvent.change(screen.getByLabelText("Type text to send to phone"), {
      target: { value: "hello phone" }
    });
    fireEvent.click(screen.getByRole("button", { name: "Send text" }));

    await waitFor(() => expect(api.addOutboxText).toHaveBeenCalledWith("hello phone"));
    expect(await screen.findByText("hello phone")).toBeInTheDocument();
    expect(screen.getByText("Added to phone outbox")).toBeInTheDocument();
  });

  it("shows command errors", async () => {
    api.addOutboxText.mockRejectedValue(new Error("failed"));
    render(<SendToPhonePanel />);

    fireEvent.change(screen.getByLabelText("Type text to send to phone"), {
      target: { value: "hello phone" }
    });
    fireEvent.click(screen.getByRole("button", { name: "Send text" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Failed to add text");
  });

  it("chooses files and renders file kinds without local paths", async () => {
    api.chooseOutboxFiles.mockResolvedValue(["C:/Users/me/secret/photo.jpg"]);
    api.addOutboxFile.mockResolvedValue(
      item({
        id: "image-1",
        kind: "image",
        displayName: "photo.jpg",
        mimeType: "image/jpeg",
        sizeBytes: 2048,
        content: undefined
      })
    );

    render(<SendToPhonePanel />);

    fireEvent.click(screen.getByRole("button", { name: "Choose files" }));

    await waitFor(() => expect(api.addOutboxFile).toHaveBeenCalledWith("C:/Users/me/secret/photo.jpg"));
    expect(await screen.findByText("photo.jpg")).toBeInTheDocument();
    expect(screen.getByLabelText("image outbox item")).toBeInTheDocument();
    expect(screen.queryByText(/C:\/Users\/me\/secret/)).not.toBeInTheDocument();
  });

  it("renders image video and file outbox item kinds", async () => {
    api.listOutboxItems.mockResolvedValue([
      item({ id: "image-1", kind: "image", displayName: "photo.jpg", content: undefined }),
      item({ id: "video-1", kind: "video", displayName: "clip.mp4", content: undefined }),
      item({ id: "file-1", kind: "file", displayName: "report.pdf", content: undefined })
    ]);

    render(<SendToPhonePanel />);

    expect(await screen.findByLabelText("image outbox item")).toBeInTheDocument();
    expect(screen.getByLabelText("video outbox item")).toBeInTheDocument();
    expect(screen.getByLabelText("file outbox item")).toBeInTheDocument();
  });
});
