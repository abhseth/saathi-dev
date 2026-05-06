# GEMINI AUDIT #01: ARCHITECTURE SCALABILITY

**Date:** 2026-05-02
**Status:** HIGH RISK (Maintenance Bottleneck)
**Scope:** Frontend (React/TS) and Backend (Rust/SQLite) Structural Integrity

---

## 1. Executive Summary
The SAATHI codebase is currently functional but exhibits "Monolithic Debt." Both the frontend and backend are structured around single, massive files that aggregate unrelated domains. While this served initial rapid development, it now poses a significant risk to feature velocity, performance, and developer onboarding as the project enters Phase 6.

---

## 2. Frontend Analysis: The "God Component" Pattern

### **2.1 Monolithic State Management (`App.tsx`)**
*   **Observation:** `App.tsx` (~2900 lines) manages **60+ independent `useState` hooks**.
*   **Risk:** Extreme "Prop Drilling." State and setters are passed through multiple layers of components, making data flow opaque and refactoring dangerous.
*   **Performance:** Any update to global state (e.g., a simple notification toggle) triggers a potential re-render of the entire application tree unless `React.memo` is meticulously applied (which it is not).

### **2.2 The "Shotgun" Loading Problem**
*   **Observation:** Data fetching for almost all entities is centralized in a massive `useEffect` hook in `App.tsx`.
*   **Risk:** No caching or request deduplication. Navigating between sections often triggers redundant network requests or relies on stale local state.

### **2.3 Rendering Bottleneck**
*   **Observation:** A **250+ line nested ternary block** determines panel rendering based on `sectionTool`.
*   **Risk:** High cognitive load. Adding a new feature requires modifying this fragile central logic. It also prevents the use of `React.lazy` for code-splitting, resulting in a large initial PWA bundle.

---

## 3. Backend Analysis: The "Repository Megalith"

### **3.1 Domain Aggregation (`repositories.rs`)**
*   **Observation:** `backend/src/repositories.rs` (~8000 lines) contains logic for Ticketing, Schools, Faculty, Attendance, and Audit Logs in a single module.
*   **Risk:** Circular dependency risks and long compile times. It is difficult to isolate changes to specific business domains.

### **3.2 "God Functions"**
*   **Observation:** Functions like `update_ticket` and `get_student_timeline` exceed 150 lines.
*   **Risk:** These functions handle validation, DB reads, multiple conditional updates, and history logging in one block, making them hard to unit test and prone to side-effect bugs.

### **3.3 Concurrency & Locking**
*   **Observation:** The system relies on a single `Mutex<Connection>` for writes.
*   **Risk:** As Phase 6 introduces high-frequency automated alerts and attendance marking, this write-lock will become a bottleneck, potentially causing API latency or "Database is locked" errors during peak school hours.

---

## 4. Recommendations

| Priority | Category | Action Item |
| :--- | :--- | :--- |
| **P0** | **Backend** | Split `repositories.rs` into domain modules (e.g., `repo/tickets.rs`, `repo/faculty.rs`). |
| **P0** | **Frontend** | Modularize `components.tsx` by moving `XxxPanel` components into feature folders. |
| **P1** | **Frontend** | Migrate server state from `useState` to **React Query** (TanStack Query) for caching. |
| **P1** | **Backend** | Extract business logic from repository functions into a separate **Service Layer**. |
| **P2** | **Frontend** | Replace the nested ternary in `App.tsx` with a registry-based routing/dispatching pattern. |

---

## 5. Auditor Notes
The "Resist refactoring" mandate in `AGENTS.md` should be officially relaxed for structural organization (moving code) while maintaining the current logic. The project is at the "tipping point" where the cost of modularization is lower than the cost of continued monolithic development.

---
**Lead Auditor:** Gemini CLI Agent
**Teams Involved:** Core Systems Team, Interface & Experience Team
