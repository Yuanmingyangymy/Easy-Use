import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { I18nProvider } from "../i18n";
import { MobileUpload } from "./MobileUpload";

describe("MobileUpload", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders text photo video and file choices", () => {
    render(
      <I18nProvider>
        <MobileUpload />
      </I18nProvider>
    );

    expect(screen.getByText("Send text")).toBeInTheDocument();
    expect(screen.getByText("Send photo")).toBeInTheDocument();
    expect(screen.getByText("Send video")).toBeInTheDocument();
    expect(screen.getByText("Send file")).toBeInTheDocument();
  });

  it("uses focused file input accept attributes", () => {
    render(
      <I18nProvider>
        <MobileUpload />
      </I18nProvider>
    );

    expect(screen.getByLabelText("Send photo")).toHaveAttribute("accept", "image/*");
    expect(screen.getByLabelText("Send photo")).toHaveAttribute("multiple");
    expect(screen.getByLabelText("Send video")).toHaveAttribute("accept", "video/*");
    expect(screen.getByLabelText("Send video")).toHaveAttribute("multiple");
    expect(screen.getByLabelText("Send file")).toHaveAttribute("multiple");
  });
});
