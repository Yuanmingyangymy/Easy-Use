import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactElement } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { ReceiveList } from "./ReceiveList";
import { I18nProvider } from "../i18n";

function renderWithI18n(ui: ReactElement) {
  return render(<I18nProvider>{ui}</I18nProvider>);
}

describe("ReceiveList", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders an empty state", () => {
    renderWithI18n(<ReceiveList items={[]} onOpenFolder={() => undefined} />);
    expect(screen.getByText("No account. No cloud. No history.")).toBeInTheDocument();
  });

  it("renders received text", () => {
    renderWithI18n(
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
    renderWithI18n(
      <ReceiveList
        onOpenFolder={() => undefined}
        items={[
          {
            id: "image-1",
            kind: "image",
            name: "photo.jpg",
            path: "C:/DropLite/photo.jpg",
            preview_url: "http://127.0.0.1:1234/api/received/image-1/preview?token=abc",
            size: 2048,
            mime: "image/jpeg",
            received_at: 1
          }
        ]}
      />
    );

    expect(screen.getByText("photo.jpg")).toBeInTheDocument();
    expect(screen.getByAltText("photo.jpg thumbnail")).toHaveAttribute(
      "src",
      "http://127.0.0.1:1234/api/received/image-1/preview?token=abc"
    );
    expect(screen.getByText(/image\/jpeg/)).toBeInTheDocument();
    expect(screen.getByText(/2.0 KB/)).toBeInTheDocument();
  });

  it("shows a custom placeholder when an image thumbnail fails", () => {
    renderWithI18n(
      <ReceiveList
        onOpenFolder={() => undefined}
        items={[
          {
            id: "image-1",
            kind: "image",
            name: "broken.png",
            preview_url: "http://127.0.0.1:1234/api/received/image-1/preview?token=abc",
            size: 2048,
            mime: "image/png",
            received_at: 1
          }
        ]}
      />
    );

    fireEvent.error(screen.getByAltText("broken.png thumbnail"));
    expect(screen.getByLabelText("image preview")).toBeInTheDocument();
  });

  it("renders a video item as a received file card", () => {
    renderWithI18n(
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
    renderWithI18n(
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
