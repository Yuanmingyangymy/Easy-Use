import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "../test/test-utils";
import { ReceiveFolderControl } from "./ReceiveFolderControl";

const api = vi.hoisted(() => ({
  chooseReceiveDirectory: vi.fn(),
  openReceiveFolder: vi.fn(),
  resetReceiveDirectory: vi.fn(),
  setReceiveDirectory: vi.fn()
}));

vi.mock("../lib/api", () => api);

describe("ReceiveFolderControl", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it("shows the current receive directory and controls", () => {
    render(<ReceiveFolderControl receiveDir="C:/Users/me/Downloads/DropLite" onChange={() => undefined} />);

    expect(screen.getByText("C:/Users/me/Downloads/DropLite")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Change folder" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open folder" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reset" })).toBeInTheDocument();
  });

  it("does not show Windows extended path prefixes", () => {
    render(<ReceiveFolderControl receiveDir={String.raw`\\?\C:\Users\me\Downloads\DropLite`} onChange={() => undefined} />);

    expect(screen.getByText(String.raw`C:\Users\me\Downloads\DropLite`)).toBeInTheDocument();
    expect(screen.queryByText(/\\\\\?\\/)).not.toBeInTheDocument();
  });

  it("updates the directory after choosing a folder", async () => {
    const onChange = vi.fn();
    api.chooseReceiveDirectory.mockResolvedValue("D:/DropLite");
    api.setReceiveDirectory.mockResolvedValue(String.raw`\\?\D:\DropLite`);
    render(<ReceiveFolderControl receiveDir="C:/Old" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Change folder" }));

    await waitFor(() => expect(api.setReceiveDirectory).toHaveBeenCalledWith("D:/DropLite"));
    expect(onChange).toHaveBeenCalledWith(String.raw`D:\DropLite`);
    expect(screen.getByText("Receive folder updated")).toBeInTheDocument();
  });

  it("does nothing when folder selection is cancelled", async () => {
    api.chooseReceiveDirectory.mockResolvedValue(null);
    render(<ReceiveFolderControl receiveDir="C:/Old" onChange={() => undefined} />);

    fireEvent.click(screen.getByRole("button", { name: "Change folder" }));

    await waitFor(() => expect(api.chooseReceiveDirectory).toHaveBeenCalled());
    expect(api.setReceiveDirectory).not.toHaveBeenCalled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows an error when updating the directory fails", async () => {
    api.chooseReceiveDirectory.mockResolvedValue("D:/DropLite");
    api.setReceiveDirectory.mockRejectedValue(new Error("no access"));
    render(<ReceiveFolderControl receiveDir="C:/Old" onChange={() => undefined} />);

    fireEvent.click(screen.getByRole("button", { name: "Change folder" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Failed to update receive folder");
  });

  it("resets the directory to default", async () => {
    const onChange = vi.fn();
    api.resetReceiveDirectory.mockResolvedValue(String.raw`\\?\C:\Users\me\Downloads\DropLite`);
    render(<ReceiveFolderControl receiveDir="D:/DropLite" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Reset" }));

    await waitFor(() => expect(api.resetReceiveDirectory).toHaveBeenCalled());
    expect(onChange).toHaveBeenCalledWith(String.raw`C:\Users\me\Downloads\DropLite`);
    expect(screen.getByText("Receive folder reset to default")).toBeInTheDocument();
  });
});
