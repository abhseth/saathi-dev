# GEMINI AUDIT #04: RELIABILITY & QUALITY

**Date:** 2026-05-02
**Status:** MEDIUM RISK (Testing Dead Zones & Deployment Safety)
**Scope:** Test Coverage, Error Handling, and Build/Deploy Integrity

---

## 1. Executive Summary
The SAATHI project demonstrates high quality in its core data access layer (Rust repositories) and database safety operations. However, significant "Reliability Dead Zones" exist in critical data-entry paths (CSV parsing) and the frontend error-handling strategy. Deployment automation is also identified as a high-risk area due to a lack of environment isolation.

---

## 2. Testing & Coverage Audit

### **2.1 Critical Dead Zone: CSV Imports**
*   **Observation:** The custom CSV parsing logic in `backend/src/routes/imports.rs` handles complex multi-row school and student imports but has **zero automated tests**.
*   **Risk:** Regression in parsing logic could lead to silent data corruption during bulk master data updates.

### **2.2 Frontend Test Gaps**
*   **Observation:** Frontend unit tests are limited to `formatters.ts`. Core interactive logic (Timetables, Ticket List filters) is untested.
*   **Risk:** High dependency on manual QA for UI changes.

### **2.3 Integration Test Isolation**
*   **Observation:** The `verify-scoping.sh` script is manual and not integrated into the `ci-gate.sh` automated pipeline.
*   **Risk:** RBAC (Role-Based Access Control) regressions could be introduced and missed by the standard CI process.

---

## 3. Error Handling & Recovery

### **3.1 "Cascade Crash" Risk**
*   **Observation:** `ErrorBoundary` usage in `App.tsx` is too coarse, wrapping entire functional areas.
*   **Risk:** A failure in one panel (e.g., a broken API call in "Timetable") crashes the entire "Faculty" section, forcing a full page reload and potential data loss.

### **3.2 Missing Client-Side Telemetry**
*   **Observation:** Production UI errors are logged only to the browser console.
*   **Risk:** Developers have zero visibility into user-side crashes unless manually reported.

---

## 4. Build & Deployment Audit

### **4.1 High-Risk Deployment Script**
*   **Observation:** `deploy-frontend.sh` is hardcoded to the production Vercel project with no safety checks.
*   **Risk:** Accidental "Fat-Finger" deployment of development code to the live site.

### **4.2 Data Integrity (Restore)**
*   **Result:** **EXCELLENT.** The `db_restore` implementation in `admin.rs` uses the SQLite Backup API and runs schema migrations on the uploaded file *before* swapping it into production.

---

## 5. Recommendations

| Priority | Category | Action Item |
| :--- | :--- | :--- |
| **P0** | **Testing** | Add unit tests for `parse_csv_rows` covering edge cases (commas in fields, missing headers). |
| **P0** | **Deployment** | Implement environment validation in `deploy-frontend.sh` to block accidental production pushes. |
| **P1** | **Recovery** | Modularize `ErrorBoundary` wrappers to isolate panel-level failures. |
| **P1** | **CI/CD** | Integrate `verify-scoping.sh` into `ci-gate.sh` and automate it via `docker-compose` or similar. |
| **P2** | **Observability** | Introduce a client-side error logger that pings the backend `audit_log` on UI crashes. |

---
**Lead Auditor:** Gemini CLI Agent
**Teams Involved:** Reliability & Quality Team, Core Systems Team
