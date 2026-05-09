import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "../test/test-utils";
import { MobileUpload } from "./MobileUpload";
import type { OutboxItem } from "../lib/types";

function outboxItem(overrides: Partial<OutboxItem>): OutboxItem {
  return {
    id: "outbox-1",
    kind: "text",
    displayName: "Text",
    createdAt: "2026-05-09T00:00:00Z",
    status: "pending",
    content: "hello from desktop",
    ...overrides
  };
}

describe("MobileUpload", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
    window.history.pushState({}, "", "/?token=test-token&lang=en");
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined)
      }
    });
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ ok: true, items: [] })
      })
    );
  });

  it("renders send to desktop choices and receive from desktop area", async () => {
    render(<MobileUpload poll={false} />);

    expect(screen.getByRole("button", { name: "Send text" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send photo" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send video" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send file" })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Receive from desktop" })).toBeInTheDocument();
    expect(await screen.findByText("No items from desktop yet")).toBeInTheDocument();
  });

  it("uses focused file input accept attributes", () => {
    render(<MobileUpload poll={false} />);

    expect(screen.getByLabelText("Select photo files")).toHaveAttribute("accept", "image/*");
    expect(screen.getByLabelText("Select photo files")).toHaveAttribute("multiple");
    expect(screen.getByLabelText("Select video files")).toHaveAttribute("accept", "video/*");
    expect(screen.getByLabelText("Select video files")).toHaveAttribute("multiple");
    expect(screen.getByLabelText("Select files")).toHaveAttribute("multiple");
  });

  it("renders desktop text items and acknowledges after copy", async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ ok: true, items: [outboxItem({ id: "text-1" })] })
    } as Response);
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ ok: true })
    } as Response);

    render(<MobileUpload poll={false} />);

    expect(await screen.findByText("hello from desktop")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Copy text" }));

    await waitFor(() => expect(navigator.clipboard.writeText).toHaveBeenCalledWith("hello from desktop"));
    expect(fetchMock).toHaveBeenCalledWith("/api/outbox/text-1/ack?token=test-token", { method: "POST" });
    expect(await screen.findByRole("button", { name: "Copied" })).toBeInTheDocument();
  });

  it("renders desktop file items with a token-protected download URL and acknowledges after click", async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () =>
        Promise.resolve({
          ok: true,
          items: [
            outboxItem({
              id: "file-1",
              kind: "file",
              displayName: "report.pdf",
              mimeType: "application/pdf",
              sizeBytes: 2048,
              content: undefined
            })
          ]
        })
    } as Response);
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ ok: true })
    } as Response);

    render(<MobileUpload poll={false} />);

    const link = await screen.findByRole("link", { name: "Download" });
    expect(screen.getByText("report.pdf")).toBeInTheDocument();
    expect(link).toHaveAttribute("href", "/api/outbox/file-1/download?token=test-token");

    fireEvent.click(link);

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith("/api/outbox/file-1/ack?token=test-token", { method: "POST" })
    );
  });

  it("shows an expired session when outbox polling is rejected", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      status: 401,
      json: () => Promise.resolve({ error: "expired" })
    } as Response);

    render(<MobileUpload poll={false} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("Session expired");
  });

  it("shows a friendly error when outbox polling fails", async () => {
    vi.mocked(fetch).mockRejectedValueOnce(new Error("network"));

    render(<MobileUpload poll={false} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("Failed to load desktop items");
  });

  it("has English and Simplified Chinese receive labels", () => {
    render(<MobileUpload poll={false} />, { language: "zh-CN" });

    expect(screen.getByRole("region", { name: "从电脑接收" })).toBeInTheDocument();
    expect(screen.getByText("暂无来自电脑的内容")).toBeInTheDocument();
  });
});
