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

function mockOutbox(items: OutboxItem[]) {
  vi.mocked(fetch).mockResolvedValueOnce({
    ok: true,
    status: 200,
    json: () => Promise.resolve({ ok: true, items })
  } as Response);
}

describe("MobileUpload", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
    window.history.pushState({}, "", "/?token=test-token&lang=en");
    Object.defineProperty(window.navigator, "userAgent", {
      value: "Chrome",
      configurable: true
    });
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined)
      }
    });
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => undefined);
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ ok: true, items: [] })
      })
    );
  });

  it("renders send and receive tabs without losing upload choices", async () => {
    render(<MobileUpload poll={false} />);

    expect(screen.getByRole("button", { name: "Send to desktop" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("button", { name: "Receive (0)" })).toHaveAttribute("aria-selected", "false");
    expect(screen.getByRole("button", { name: "Send text" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send photo" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send video" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send file" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Receive (0)" }));
    expect(screen.getByRole("region", { name: "Receive from desktop" })).toBeInTheDocument();
    expect(await screen.findByText("No items from desktop yet")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Send to desktop" }));
    expect(screen.getByRole("button", { name: "Send photo" })).toBeInTheDocument();
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
    mockOutbox([outboxItem({ id: "text-1" })]);
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ ok: true })
    } as Response);

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    expect(await screen.findByText("hello from desktop")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Copy text" }));

    await waitFor(() => expect(navigator.clipboard.writeText).toHaveBeenCalledWith("hello from desktop"));
    expect(fetchMock).toHaveBeenCalledWith("/api/outbox/text-1/ack?token=test-token", { method: "POST" });
    expect(await screen.findByRole("button", { name: "Copied" })).toBeInTheDocument();
  });

  it("renders desktop file items with a token-protected download URL and acknowledges after download starts", async () => {
    const fetchMock = vi.mocked(fetch);
    mockOutbox([
      outboxItem({
        id: "file-1",
        kind: "file",
        displayName: "report.pdf",
        mimeType: "application/pdf",
        sizeBytes: 2048,
        content: undefined
      })
    ]);
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ ok: true })
    } as Response);

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    const link = await screen.findByRole("link", { name: "Download" });
    expect(screen.getByText("report.pdf")).toBeInTheDocument();
    expect(link).toHaveAttribute("href", "/api/outbox/file-1/download?token=test-token");

    fireEvent.click(link);

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith("/api/outbox/file-1/ack?token=test-token", { method: "POST" })
    );
  });

  it("shows the WeChat download hint only in WeChat-like browsers", async () => {
    Object.defineProperty(window.navigator, "userAgent", {
      value: "MicroMessenger",
      configurable: true
    });
    mockOutbox([
      outboxItem({
        id: "file-1",
        kind: "file",
        displayName: "report.pdf",
        content: undefined
      })
    ]);

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    expect(
      screen.getByText(
        "WeChat's in-app browser may block downloads. Tap '...' and open this page in your system browser to download files."
      )
    ).toBeInTheDocument();
  });

  it("does not show the WeChat download hint in regular browsers", async () => {
    mockOutbox([
      outboxItem({
        id: "file-1",
        kind: "file",
        displayName: "report.pdf",
        content: undefined
      })
    ]);

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    expect(screen.queryByText(/WeChat's in-app browser may block downloads/)).not.toBeInTheDocument();
  });

  it("does not acknowledge when download is blocked and shows a browser fallback", async () => {
    const fetchMock = vi.mocked(fetch);
    Object.defineProperty(window.navigator, "userAgent", {
      value: "MicroMessenger",
      configurable: true
    });
    mockOutbox([
      outboxItem({
        id: "file-1",
        kind: "file",
        displayName: "report.pdf",
        mimeType: "application/pdf",
        sizeBytes: 2048,
        content: undefined
      })
    ]);

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    fireEvent.click(await screen.findByRole("link", { name: "Download" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Download may be blocked in this browser");
    expect(screen.getByText("Please open this page in your system browser to download files")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Open in browser to download" })).toHaveAttribute(
      "href",
      "/api/outbox/file-1/download?token=test-token"
    );
    expect(screen.getAllByRole("button", { name: "Copy download link" }).length).toBeGreaterThan(0);
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("copying a download link copies the token URL without acknowledging", async () => {
    const fetchMock = vi.mocked(fetch);
    mockOutbox([
      outboxItem({
        id: "file-1",
        kind: "file",
        displayName: "report.pdf",
        content: undefined
      })
    ]);

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    fireEvent.click(await screen.findByRole("button", { name: "Copy download link" }));

    await waitFor(() =>
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
        "http://localhost:3000/api/outbox/file-1/download?token=test-token"
      )
    );
    expect(await screen.findByRole("alert")).toHaveTextContent("Download link copied");
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("keeps download-triggered UX even if acknowledgement fails", async () => {
    const fetchMock = vi.mocked(fetch);
    mockOutbox([
      outboxItem({
        id: "file-1",
        kind: "file",
        displayName: "report.pdf",
        content: undefined
      })
    ]);
    fetchMock.mockRejectedValueOnce(new Error("ack failed"));

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    fireEvent.click(await screen.findByRole("link", { name: "Download" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Download started");
  });

  it("deduplicates repeated outbox ids from polling", async () => {
    mockOutbox([
      outboxItem({ id: "file-1", kind: "file", displayName: "report.pdf", content: undefined }),
      outboxItem({ id: "file-1", kind: "file", displayName: "report.pdf", content: undefined })
    ]);

    render(<MobileUpload poll={false} />);

    fireEvent.click(await screen.findByRole("button", { name: "Receive (1)" }));
    expect(await screen.findByText("report.pdf")).toBeInTheDocument();
    expect(screen.getAllByText("report.pdf")).toHaveLength(1);
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

    expect(screen.getByRole("button", { name: "\u53d1\u9001\u5230\u7535\u8111" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "\u63a5\u6536 (0)" }));
    expect(screen.getByRole("region", { name: "\u4ece\u7535\u8111\u63a5\u6536" })).toBeInTheDocument();
    expect(screen.getByText("\u6682\u65e0\u6765\u81ea\u7535\u8111\u7684\u5185\u5bb9")).toBeInTheDocument();
  });
});
