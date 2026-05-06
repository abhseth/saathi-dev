# GEMINI AUDIT #02: CORE SYSTEMS & SECURITY

**Date:** 2026-05-02
**Status:** MEDIUM RISK (Silent Concurrency Failures & Scoping Leaks)
**Scope:** Backend Auth Integrity, SQLite Concurrency, and Data Integrity

---

## 1. Executive Summary
The Core Systems audit reveals that while the foundation is secure (passwords, JWT secrets), the application-level logic for scoping and concurrency is underdeveloped. The lack of database transactions and a busy timeout creates a high risk of "Silent Failures" and data corruption under the load of Phase 6. Additionally, several administrative and automated endpoints are "leaky," potentially allowing cross-school data access.

---

## 2. Authentication & Scoping Audit

### **2.1 School Scope Leaks**
*   **Vulnerability:** Endpoints in `backend/src/routes/automation.rs` (e.g., `reassign_faculty`, `clone_week_with_check`) do not verify if the requesting user (AOM role) has permissions for the target school ID.
*   **Risk:** An AOM could perform administrative actions on schools outside their assigned `user_schools`.
*   **Vulnerability:** `substitutions.rs::list_swap_requests` does not apply `scope_filter`.
*   **Risk:** Peer-to-peer swap requests are visible globally to anyone with the AOM role.

### **2.2 Sensitive Data Handling**
*   **Result:** PASS. `AppUser` and `CurrentUser` models correctly use `serde` to exclude `password_hash`. `JWT_SECRET` length is validated.

---

## 3. Database & Concurrency Audit

### **3.1 The "Database is Locked" Risk**
*   **Observation:** The SQLite connection pool in `main.rs` does not configure a `busy_timeout`.
*   **Risk:** Immediate write failures under concurrent load (common in school hours). This will manifest as 500 errors in the frontend during peak attendance marking.

### **3.2 Transactional Integrity**
*   **Observation:** Critical operations (Substitution Peer-to-Peer Swaps, Leave Approval Impact) involve multiple `UPDATE` and `INSERT` statements across repository boundaries without an encompassing `BEGIN TRANSACTION`.
*   **Risk:** Partial failures. If the server crashes or a constraint is hit mid-operation, the database is left in a "half-updated" state that is difficult to recover from.

### **3.3 Race Conditions (Substitution)**
*   **Observation:** Substitution acceptance uses "Check then Act."
*   **Risk:** Two faculty members could accept the same "Open" substitution simultaneously. The second one will overwrite the first without warning.

---

## 4. Input Validation & Robustness

### **4.1 Fragile CSV Parsing**
*   **Observation:** `bulk_ops.rs::bulk_import_subjects` uses raw string splitting by comma.
*   **Risk:** Fails on any subject name containing a comma, potentially misaligning database columns or causing panics.

### **4.2 SQL Injection (Latent)**
*   **Observation:** `count_school_field` in `repositories.rs` uses raw string interpolation for column names.
*   **Risk:** Low (internal use only), but violates the security standard and could be exploited if more dynamic metadata fields are added.

---

## 5. Recommendations

| Priority | Category | Action Item |
| :--- | :--- | :--- |
| **P0** | **Concurrency** | Configure `PRAGMA busy_timeout = 5000;` in `main.rs` pool initialization. |
| **P0** | **Security** | Add `enforce_school_scope` to all mutation endpoints in `automation.rs` and `faculty.rs`. |
| **P1** | **Integrity** | Wrap all multi-step substitution/leave updates in `Sqlite` transactions. |
| **P1** | **Security** | Use atomic updates for substitutions (e.g., `UPDATE ... WHERE actual_faculty_user_id IS NULL`). |
| **P2** | **Robustness** | Standardize CSV parsing using the `csv` crate or `parse_csv_rows` helper. |

---
**Lead Auditor:** Gemini CLI Agent
**Teams Involved:** Core Systems Team (Backend & Security)
