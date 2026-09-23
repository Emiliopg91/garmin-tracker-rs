import { describe, expect, it } from "vitest";
import { TimeUtils } from "@/utils/TimeUtils";

const toEpoch = (date: Date) => date.getTime() / 1000;

describe("TimeUtils.formatDuration", () => {
  it("formats zero seconds", () => {
    expect(TimeUtils.formatDuration(0)).toBe("0:00");
  });

  it("strips leading zeros from minutes", () => {
    expect(TimeUtils.formatDuration(5)).toBe("0:05");
    expect(TimeUtils.formatDuration(59)).toBe("0:59");
    expect(TimeUtils.formatDuration(65)).toBe("1:05");
  });

  it("keeps two-digit minutes", () => {
    expect(TimeUtils.formatDuration(600)).toBe("10:00");
    expect(TimeUtils.formatDuration(3599)).toBe("59:59");
  });

  it("includes hours when needed", () => {
    expect(TimeUtils.formatDuration(3600)).toBe("1:00:00");
    expect(TimeUtils.formatDuration(3661)).toBe("1:01:01");
    expect(TimeUtils.formatDuration(36000)).toBe("10:00:00");
  });

  it("truncates fractional seconds", () => {
    expect(TimeUtils.formatDuration(59.9)).toBe("0:59");
  });
});

describe("TimeUtils.formatTimeDate / formatDate", () => {
  const date = new Date(2024, 0, 5, 7, 3);

  it("formats time and date with padding", () => {
    expect(TimeUtils.formatTimeDate(toEpoch(date))).toBe("07:03 05/01/2024");
  });

  it("formats date only", () => {
    expect(TimeUtils.formatDate(toEpoch(date))).toBe("05/01/2024");
  });
});

describe("TimeUtils.parseLocalDateTime", () => {
  it("parses a valid local date time", () => {
    expect(TimeUtils.parseLocalDateTime("21:45 31/12/2023")).toEqual(
      new Date(2023, 11, 31, 21, 45, 0),
    );
  });

  it("round-trips with formatTimeDate", () => {
    const text = "09:30 15/06/2025";
    const parsed = TimeUtils.parseLocalDateTime(text);
    expect(TimeUtils.formatTimeDate(toEpoch(parsed))).toBe(text);
  });

  it.each([
    "",
    "9:30 15/06/2025",
    "09:30 15-06-2025",
    "09:30 15/06/2025 ",
    "15/06/2025 09:30",
  ])("rejects malformed input %j", (input) => {
    expect(() => TimeUtils.parseLocalDateTime(input)).toThrow(
      "Wrong date format",
    );
  });

  it.each([
    "09:30 31/02/2025",
    "09:30 01/13/2025",
    "25:00 01/01/2025",
    "10:60 01/01/2025",
  ])("rejects out of range values %j", (input) => {
    expect(() => TimeUtils.parseLocalDateTime(input)).toThrow(
      "Wrong date format",
    );
  });
});
