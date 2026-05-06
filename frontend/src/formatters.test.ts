import { describe, it, expect } from "vitest";
import { formatTimestamp, formatField, formatBytes, getSlaState } from "./formatters";

describe("formatTimestamp", () => {
  it("replaces T with space", () => {
    expect(formatTimestamp("2024-01-15T09:30:00")).toBe("2024-01-15 09:30:00");
  });

  it("returns empty string for falsy input", () => {
    expect(formatTimestamp("")).toBe("");
    expect(formatTimestamp(null as unknown as string)).toBe("");
  });
});

describe("formatField", () => {
  it("capitalizes snake_case", () => {
    expect(formatField("issue_category")).toBe("Issue Category");
  });
});

describe("formatBytes", () => {
  it("formats bytes", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(2097152)).toBe("2.0 MB");
  });
});

describe("getSlaState", () => {
  it("returns Met for closed tickets", () => {
    expect(getSlaState({ status: "Closed", sla_due_at: "2024-01-01 00:00:00" })).toBe("Met");
  });

  it("returns Breached for past due dates", () => {
    const past = new Date(Date.now() - 86400000).toISOString().replace("T", " ");
    expect(getSlaState({ status: "Open", sla_due_at: past })).toBe("Breached");
  });
});
