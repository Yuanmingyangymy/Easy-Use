import { describe, expect, it } from "vitest";
import { render, screen } from "../test/test-utils";
import { SessionTimer } from "./SessionTimer";

describe("SessionTimer", () => {
  it("renders an active countdown", () => {
    const expiresAt = Math.floor(Date.now() / 1000) + 90;
    render(<SessionTimer expiresAt={expiresAt} />);
    expect(screen.getByText(/This session expires in/)).toBeInTheDocument();
  });
});
