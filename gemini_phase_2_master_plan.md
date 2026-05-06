# GEMINI: PHASE 2 MASTER PLAN (ARCHITECTURE DE-RISKING)

**Status:** AUTHORITATIVE GUIDANCE
**Objective:** Decouple the "God Files" (`repositories.rs` and `App.tsx`) to eliminate technical debt and enable the high-load feature rollout in Phase 3.

---

## 1. Backend: The Repository Split (`repositories.rs`)
The 8,000+ line repository will be decomposed into domain-specific modules.

### **1.1 Module Map**
*   **`repo/common.rs`**: Shared row mappers (`_from_row`) and low-level validation helpers.
*   **`repo/tickets.rs`**: Ticketing, SLA policies, and assignment rules.
*   **`repo/schools.rs`**: School profiles, regions, class plans, and VP centers.
*   **`repo/faculty.rs`**: User management, subjects, timetable slots, and holidays.
*   **`repo/ops.rs`**: High-frequency overrides (Substitutions, Attendance, Leave, Swaps).
*   **`repo/analytics.rs`**: Reporting, compliance metrics, and trends.

### **1.2 Implementation Strategy**
*   **Backward Compatibility:** `repo/mod.rs` must re-export all public functions. Routes should continue to import from `crate::repositories` (aliased to the `repo` module in `main.rs`).
*   **Leaf-First Extraction:** Move `common.rs` first to resolve shared dependencies, then proceed with the least-coupled domains (Tickets/Schools).

---

## 2. Frontend: The State Ownership Split (`App.tsx`)
`App.tsx` will be refactored from a "God Component" into a "Shell & Router."

### **2.1 Hook Extraction**
Extract the 60+ `useState` hooks and data loaders into domain hooks in `/src/hooks/`:
*   **`useTicketState.ts`**: Handles all ticket CRUD, filters, and comments.
*   **`useMasterDataState.ts`**: Handles schools, regions, and the new Master Data forms.
*   **`useFacultyState.ts`**: Handles timetables, faculty assignments, and attendance records.

### **2.2 Shell Responsibility**
`App.tsx` should only manage global state: `currentUser`, `sectionTool`, `error`, and `notice`. All domain logic must be consumed via the new hooks.

---

## 3. System: Contract Convergence
Eliminate the redundancy of updating multiple files for a single feature.

### **3.1 Unified Tool Registry**
*   Replace the 250-line nested ternary in `App.tsx` with a **Registry-based Dispatcher**.
*   Map `toolId` to a `Component` definition in a centralized `TOOL_REGISTRY` configuration.

### **3.2 API Mirroring**
*   Refactor `frontend/src/api.ts` to use a type-safe map that mirrors the backend's `routes/mod.rs`.
*   Establish a "Parity Check" (test or script) to ensure route and API names never drift.

---

## 4. Performance: Hot Write Isolation
Address the primary database bottleneck identified in the audit.

### **4.1 Escalation Decoupling**
*   **Task:** Isolate `refresh_escalations` from the standard `update_ticket` repository flow.
*   **Mechanism:** Move escalation scans to a background worker or an explicit admin trigger. This ensures that routine ticket edits are not blocked by a full-table SLA scan.

---

## 5. Definition of Done (Milestone C)

1.  **Backend:** `repositories.rs` is deleted; logic is contained within `/repo/` modules.
2.  **Frontend:** `App.tsx` is reduced to under 1,000 lines of shell and routing logic.
3.  **Stability:** `cargo test` and `npm run build` pass with zero new warnings.
4.  **Verification:** The `verify-scoping.sh` script passes against the new modularized backend.

---
**Lead Auditor/Planner:** Gemini CLI Agent
**Primary Stakeholder:** Engineering Team
