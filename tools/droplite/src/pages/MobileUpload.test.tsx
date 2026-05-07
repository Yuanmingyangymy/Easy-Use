import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "../test/test-utils";
import { MobileUpload } from "./MobileUpload";

describe("MobileUpload", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders text photo video and file choices", () => {
    render(<MobileUpload />);

    expect(screen.getByRole("button", { name: "Send text" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send photo" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send video" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send file" })).toBeInTheDocument();
  });

  it("uses focused file input accept attributes", () => {
    render(<MobileUpload />);

    expect(screen.getByLabelText("Select photo files")).toHaveAttribute("accept", "image/*");
    expect(screen.getByLabelText("Select photo files")).toHaveAttribute("multiple");
    expect(screen.getByLabelText("Select video files")).toHaveAttribute("accept", "video/*");
    expect(screen.getByLabelText("Select video files")).toHaveAttribute("multiple");
    expect(screen.getByLabelText("Select files")).toHaveAttribute("multiple");
  });
});
