# GEMINI AUDIT: CONSOLIDATED EXECUTIVE SUMMARY & ROADMAP

**Date:** 2026-05-02
**Overall Project Status:** YELLOW (Architectural Debt & Operational Gaps)
**Lead Auditor:** Gemini CLI Agent

---

## 1. Executive Assessment
SAATHI is a technically impressive project with a robust core. However, it is currently "stretching" its initial monolithic design beyond its limits. As the project enters **Phase 6 (Automation & High Load)**, the current architectural patterns (massive files, missing transactions, lack of offline robustness) pose a significant risk to stability and feature velocity.

---

## 2. Top-Priority "Critical" Risks (P0)

### **A. Security: Auth Scoping Leaks**
*   **Finding:** Several administrative and automated endpoints (Automation, Substitutions) allow AOMs to modify data for schools they do not own.
*   **Impact:** Potential cross-tenant data breach.
*   **Action:** Enforce `enforce_school_scope` across all route modules.

### **B. Stability: SQLite Concurrency**
*   **Finding:** Missing `busy_timeout` and lack of SQL Transactions for multi-step operations (e.g., Leave/Swap logic).
*   **Impact:** "Database is locked" errors and "half-updated" inconsistent records during peak usage.
*   **Action:** Configure `busy_timeout = 5000` and wrap all multi-table mutations in Transactions.

### **C. Reliability: Testing Dead Zones**
*   **Finding:** Custom CSV parser handles critical master data with zero tests. `restore_session` logic is broken for ad-hoc sessions.
*   **Impact:** Silent data corruption and permanent loss of faculty assignments during "Restore" operations.
*   **Action:** Unit test `parse_csv_rows` and patch the `restore_session` logic.

### **D. Deployment Safety**
*   **Finding:** `deploy-frontend.sh` lacks environment guards and points directly to production.
*   **Impact:** High risk of accidental production overwrite from a development branch.
*   **Action:** Implement mandatory `--env=prod` flag for deployment scripts.

---

## 3. Strategic Roadmap

### **Phase 1: The "Safe Foundation" (Immediate)**
*   **Backend:** Add `busy_timeout`, implement SQL Transactions, and close Auth scoping leaks.
*   **Scripts:** Add environment guards to `deploy-frontend.sh` and automate `verify-scoping.sh` in the CI gate.
*   **Domain:** Fix the `restore_session` bug for ad-hoc sessions.

### **Phase 2: Architectural De-risking (Short Term)**
*   **Backend:** Modularize `repositories.rs` by domain. Extract business logic into a Service Layer.
*   **Frontend:** Modularize `components.tsx`. Replace the massive nested ternary in `App.tsx` with a Registry-based Dispatcher.
*   **Quality:** Implement unit tests for CSV parsing and core SLA calculation.

### **Phase 3: Operational Nuance (Medium Term)**
*   **SLA:** Refactor SLA logic to respect weekends and the `holidays` table.
*   **Master Data:** Implement "Track Migration" logic to sync students/faculty when school plans change.
*   **Substitution:** Add "Related Subject" scoring to allow cross-track matching (JEE vs NEET).

### **Phase 4: Interface & PWA Polish (Ongoing)**
*   **Performance:** Implement React Query for server-state (eliminating Prop Drilling).
*   **PWA:** Fix `sw.js` to enable offline loading.
*   **UX:** Replace "Loading..." text with `SkeletonBlock` and add loading states to all "Save" buttons.

---

## 4. Final Recommendation
The "Resist Refactoring" mandate was useful for early development but is now the project's primary bottleneck. We recommend a **"Structural Sprint"** where the team focuses exclusively on moving code into modules and adding transactions before implementing any new Phase 6 features.

---
**Lead Auditor:** Gemini CLI Agent
**Teams:** Core Systems, UX/UI, Reliability, Domain Ops
