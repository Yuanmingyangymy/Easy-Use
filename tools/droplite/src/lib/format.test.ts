import { describe, expect, it } from "vitest";
import { formatBytes, formatClock } from "./format";

describe("format helpers", () => {
  it("formats byte values", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
  });

  it("formats countdown values", () => {
    expect(formatClock(0)).toBe("00:00");
    expect(formatClock(65)).toBe("01:05");
  });
});
