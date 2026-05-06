# GEMINI AUDIT #05: DOMAIN OPERATIONS

**Date:** 2026-05-02
**Status:** MEDIUM RISK (Logic Gaps in Workflow & Substitution)
**Scope:** Substitution Engine, Timetable Compliance, SLA Logic, and Master Data Integrity

---

## 1. Executive Summary
The Domain Operations audit reveals that while the core workflows are implemented, they lack "Operational Resilience." The system operates on an idealized model that does not account for business hours (SLA logic), ad-hoc session edge cases (Restore bug), or the dynamic nature of school track changes (Master Data fragmentation). The substitution engine is technically sound but logically rigid.

---

## 2. Substitution & Timetable Logic

### **2.1 The "Restore" Logic Defect**
*   **Observation:** `restore_session` (reverting a cancellation/substitution) sets `actual_faculty_user_id` to `NULL`.
*   **Risk:** For ad-hoc (makeup) sessions that exist outside the master timetable, this action results in a permanent loss of the assigned faculty, as there is no "timetable slot" to fall back to.

### **2.2 Rigidity in Substitution Matching**
*   **Observation:** Ranking is strictly tied to `subject_id`.
*   **Risk:** NEET Physics and JEE Physics are treated as unrelated subjects. A subject expert on one track is ranked the same as a non-expert for the other, leading to poor substitution suggestions in specialized schools.

### **2.3 Compliance Reporting Inaccuracies**
*   **Observation:** `list_compliance_metrics` ignores school-level subject opt-ins (English/SST).
*   **Risk:** Schools appear "Non-Compliant" for subjects they do not teach, leading to inaccurate administrative dashboards.

---

## 3. Ticketing & SLA Workflows

### **3.1 "Clock-Time" vs "Business-Time" SLA**
*   **Observation:** SLA breach timers use raw SQLite `datetime` additions.
*   **Risk:** Breach alerts trigger over weekends and holidays, creating thousands of false-positive escalations and devaluing the alerting system.

### **3.2 The "Unassigned" Black Hole**
*   **Observation:** Tickets without an assignment rule default to "Unassigned."
*   **Risk:** Critical academic support requests may sit unaddressed until an SLA breach occurs, as there is no human "Queue Owner" fallback.

---

## 4. Master Data Integrity

### **4.1 Orphaned Entities on Track Change**
*   **Observation:** `Student` and `faculty_assignment` records are not synced when a `SchoolClassPlan` track (JEE/NEET) is modified.
*   **Risk:** Severe data fragmentation. Students and faculty become invisible to the timetable and attendance systems until manually re-enrolled.

### **4.2 Batch-Pattern Blindness**
*   **Observation:** Alerts for unfilled periods do not validate the current day against the `batch_pattern` (Weekday/Weekend).
*   **Risk:** Noise. Weekend batches generate "unfilled gap" alerts during the work week.

---

## 5. Recommendations

| Priority | Category | Action Item |
| :--- | :--- | :--- |
| **P0** | **Integrity** | Fix `restore_session` to check if a session is ad-hoc before nullifying the assigned faculty. |
| **P0** | **SLA** | Refactor SLA calculation to respect weekends and the `holidays` table. |
| **P1** | **Integrity** | Implement cascading track updates or a "Migration Wizard" for Students/Faculty during plan changes. |
| **P1** | **Validation** | Enhance `validate_swap` to include active `lecture_sessions` and `leave_requests` in its conflict check. |
| **P2** | **Intelligence** | Add "Subject Affinity" scoring to the substitution engine (e.g., matching NEET/JEE Physics). |
| **P2** | **Ops** | Require a "Default Assignee" for all active ticketing queues. |

---
**Lead Auditor:** Gemini CLI Agent
**Teams Involved:** Domain Operations Team, Core Systems Team
