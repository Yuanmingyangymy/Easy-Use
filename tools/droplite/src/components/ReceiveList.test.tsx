import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ReceiveList } from "./ReceiveList";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => path
}));

describe("ReceiveList", () => {
  it("renders an empty state", () => {
    render(<ReceiveList items={[]} onOpenFolder={() => undefined} />);
    expect(screen.getByText("No account. No cloud. No history.")).toBeInTheDocument();
  });

  it("renders received text", () => {
    render(
      <ReceiveList
        onOpenFolder={() => undefined}
        items={[
          {
            id: "1",
            kind: "text",
            name: "Text",
            text: "hello from phone",
            received_at: 1
          }
        ]}
      />
    );

    expect(screen.getByText("hello from phone")).toBeInTheDocument();
    expect(screen.getByText("Copy text")).toBeInTheDocument();
  });

  it("renders an image item with a thumbnail", () => {
    render(
      <ReceiveList
        onOpenFolder={() => undefined}
        items={[
          {
            id: "image-1",
            kind: "image",
            name: "photo.jpg",
            path: "C:/DropLite/photo.jpg",
            size: 2048,
            mime: "image/jpeg",
            received_at: 1
          }
        ]}
      />
    );

    expect(screen.getByText("photo.jpg")).toBeInTheDocument();
    expect(screen.getByAltText("photo.jpg thumbnail")).toHaveAttribute("src", "C:/DropLite/photo.jpg");
    expect(screen.getByText(/image\/jpeg/)).toBeInTheDocument();
    expect(screen.getByText(/2.0 KB/)).toBeInTheDocument();
  });

  it("renders a video item as a received file card", () => {
    render(
      <ReceiveList
        onOpenFolder={() => undefined}
        items={[
          {
            id: "video-1",
            kind: "video",
            name: "clip.mp4",
            path: "C:/DropLite/clip.mp4",
            size: 5 * 1024 * 1024,
            mime: "video/mp4",
            received_at: 1
          }
        ]}
      />
    );

    expect(screen.getByText("clip.mp4")).toBeInTheDocument();
    expect(screen.getByLabelText("video preview")).toBeInTheDocument();
    expect(screen.getByText(/video\/mp4/)).toBeInTheDocument();
    expect(screen.getByText(/5.0 MB/)).toBeInTheDocument();
  });

  it("renders a normal file item with name size and time", () => {
    render(
      <ReceiveList
        onOpenFolder={() => undefined}
        items={[
          {
            id: "file-1",
            kind: "file",
            name: "report.pdf",
            path: "C:/DropLite/report.pdf",
            size: 1234,
            mime: "application/pdf",
            received_at: 1
          }
        ]}
      />
    );

    expect(screen.getByText("report.pdf")).toBeInTheDocument();
    expect(screen.getByLabelText("file preview")).toBeInTheDocument();
    expect(screen.getByText(/application\/pdf/)).toBeInTheDocument();
    expect(screen.getByText(/1.2 KB/)).toBeInTheDocument();
  });
});
