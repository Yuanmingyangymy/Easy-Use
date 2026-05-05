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
});
