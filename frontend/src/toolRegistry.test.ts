import { describe, it, expect } from "vitest";
import { APP_TOOLS, isToolVisible, canAccessTool } from "./toolRegistry";

describe("isToolVisible", () => {
  it("returns false when role is not in the tool's roles", () => {
    const tool = APP_TOOLS.find((t) => t.id === "master-data")!;
    expect(isToolVisible(tool, "viewer")).toBe(false);
    expect(isToolVisible(tool, "agent")).toBe(false);
  });

  it("returns true when role is in the tool's roles", () => {
    const tool = APP_TOOLS.find((t) => t.id === "master-data")!;
    expect(isToolVisible(tool, "admin")).toBe(true);
    expect(isToolVisible(tool, "aom")).toBe(true);
  });

  it("returns false for admin-only tools when accessed by non-admin", () => {
    const auditLog = APP_TOOLS.find((t) => t.id === "audit-log")!;
    const routing = APP_TOOLS.find((t) => t.id === "routing")!;
    const sla = APP_TOOLS.find((t) => t.id === "sla")!;
    const users = APP_TOOLS.find((t) => t.id === "users")!;
    const programDashboard = APP_TOOLS.find((t) => t.id === "program-dashboard")!;

    expect(isToolVisible(auditLog, "viewer")).toBe(false);
    expect(isToolVisible(routing, "viewer")).toBe(false);
    expect(isToolVisible(sla, "viewer")).toBe(false);
    expect(isToolVisible(users, "viewer")).toBe(false);
    expect(isToolVisible(programDashboard, "viewer")).toBe(false);
    expect(isToolVisible(programDashboard, "agent")).toBe(false);
    expect(isToolVisible(programDashboard, "aom")).toBe(false);
    expect(isToolVisible(programDashboard, "admin")).toBe(true);
  });
});

describe("canAccessTool", () => {
  it("returns false for unknown tool ids", () => {
    expect(canAccessTool("nonexistent", "admin")).toBe(false);
  });

  it("viewer can access reports and directory but not program-dashboard", () => {
    expect(canAccessTool("program-dashboard", "viewer")).toBe(false);
    expect(canAccessTool("reports", "viewer")).toBe(true);
    expect(canAccessTool("directory", "viewer")).toBe(true);
  });

  it("viewer cannot access master-data, audit-log, users, backup, routing, sla", () => {
    expect(canAccessTool("master-data", "viewer")).toBe(false);
    expect(canAccessTool("audit-log", "viewer")).toBe(false);
    expect(canAccessTool("users", "viewer")).toBe(false);
    expect(canAccessTool("routing", "viewer")).toBe(false);
    expect(canAccessTool("sla", "viewer")).toBe(false);
  });

  it("agent can access communications but not master-data or program-dashboard", () => {
    expect(canAccessTool("communications", "agent")).toBe(true);
    expect(canAccessTool("master-data", "agent")).toBe(false);
    expect(canAccessTool("program-dashboard", "agent")).toBe(false);
  });

  it("aom can access master-data, faculty-assignments, timetable but not users, backup, routing, sla, program-dashboard", () => {
    expect(canAccessTool("master-data", "aom")).toBe(true);
    expect(canAccessTool("faculty-assignments", "aom")).toBe(true);
    expect(canAccessTool("timetable", "aom")).toBe(true);
    expect(canAccessTool("users", "aom")).toBe(false);
    expect(canAccessTool("routing", "aom")).toBe(false);
    expect(canAccessTool("sla", "aom")).toBe(false);
    expect(canAccessTool("program-dashboard", "aom")).toBe(false);
  });
});

describe("APP_TOOLS", () => {
  it("every tool has a non-empty roles array", () => {
    for (const tool of APP_TOOLS) {
      expect(tool.roles, `tool ${tool.id} should have non-empty roles`).toBeDefined();
      expect(tool.roles.length, `tool ${tool.id} should have at least one role`).toBeGreaterThan(0);
    }
  });
});
