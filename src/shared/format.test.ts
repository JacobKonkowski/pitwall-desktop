import { describe, expect, it } from "vitest";
import { formatDelta, formatLapTime } from "./format";

describe("formatLapTime", () => {
  it("formats under a minute", () => {
    expect(formatLapTime(23_456)).toBe("23.456");
  });
  it("formats minutes", () => {
    expect(formatLapTime(83_456)).toBe("1:23.456");
  });
  it("handles empty", () => {
    expect(formatLapTime(null)).toBe("—");
  });
});

describe("formatDelta", () => {
  it("signs positive", () => {
    expect(formatDelta(184)).toBe("+0.184");
  });
});
