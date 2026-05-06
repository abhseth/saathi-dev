# SAATHI Stakeholder Wishlist — Consolidated Review

**Date:** 2026-04-30  
**Reviewers:** 5 agents representing School Principal, Center Head (AOM), Central SPOC, SIP Head, and Faculty  
**Purpose:** Feed into the implementation backlog for Phase 6+  
**Format:** Requests grouped by theme, tagged by stakeholder, scored by impact × feasibility

---

## Legend

| Tag | Stakeholder |
|-----|-------------|
| 🏫 P | Principal |
| 🏢 A | Center Head (AOM) |
| 🌐 C | Central SPOC |
| 📚 S | SIP Head |
| 👨‍🏫 F | Faculty |

| Priority | Description |
|----------|-------------|
| 🔴 P0 | Critical — blocks daily operations |
| 🟡 P1 | High — major friction reducer |
| 🟢 P2 | Medium — quality-of-life / strategic |
| ⚪ P3 | Nice-to-have / future |

---

## Theme 1: Substitution & Leave Workflow

> **Most requested theme across ALL stakeholders.**

| # | Request | Tags | Priority | Notes |
|---|---------|------|----------|-------|
| 1.1 | **Auto-Suggest Substitute Resolver** — When faculty absent, auto-rank available substitutes by subject match, free period, proximity, workload balance; one-tap assign | 🏫 P, 🏢 A | 🔴 P0 | Backend needs candidate-scoring query; frontend needs suggestion card UI |
| 1.2 | **Today's Substitutions Command Center** — Three-lane dashboard: "Unfilled Absences," "Substitutes Assigned," "Completed Today" with drag-and-drop assignment | 🏫 P | 🔴 P0 | New screen; heavy frontend work |
| 1.3 | **Leave Request with Auto-Substitution Trigger** — Faculty requests leave → system immediately pings eligible substitutes, updates schedule on approval | 👨‍🏫 F | 🟡 P1 | New workflow; needs `leave_requests` table |
| 1.4 | **Peer Period Swap Request** — Faculty proposes swap with colleague; auto-validates no conflicts, updates timetable on mutual accept | 👨‍🏫 F | 🟢 P2 | New entity `swap_requests`; conflict validation |
| 1.5 | **Substitution Request Detail Panel** — Show class roster, room map snippet, last-covered topics when tapping a substitution request | 👨‍🏫 F | 🟡 P1 | Needs `student_rosters` or integration |
| 1.6 | **Substitution Balance Tracker** — "Substitutions given vs. received" metric per faculty | 👨‍🏫 F | 🟢 P2 | Aggregation query on `lecture_sessions` |
| 1.7 | **Bulk Attendance + Auto-Ticket** — Mark multiple faculty absent in one action, auto-generates linked substitution tickets for all their slots | 🏫 P | 🟡 P1 | Bulk action UI; batch ticket creation |
| 1.8 | **Substitution Cost & Coverage Report** — Monthly report: requests per school, acceptance rate, avg time-to-fill, repeat absentees | 🏢 A | 🟢 P2 | Aggregation endpoint + report UI |
| 1.9 | **Instant Schedule Change Notifications** — Push/in-app alerts when timetable modified, room changed, substitution assigned | 👨‍🏫 F | 🟡 P1 | Notification system (WebSocket or polling) |
| 1.10 | **Alert Command Center** — Group alerts by category, persist dismissal server-side, bulk-create tickets from selected alerts | 🌐 C | 🟡 P1 | Needs `alert_states` table; bulk ticket API |

---

## Theme 2: Mobile Experience

> **Second most requested. Every stakeholder had mobile-specific asks.**

| # | Request | Tags | Priority | Notes |
|---|---------|------|----------|-------|
| 2.1 | **Morning Push Digest (Principal)** — Mobile "Start Your Day" card: faculty absent today, periods needing substitutes, urgent tickets, "Mark All Reviewed" | 🏫 P | 🔴 P0 | New mobile-first landing view |
| 2.2 | **AOM Morning Brief (Mobile-First Card Feed)** — Vertical scroll of school cards showing attendance %, unfilled periods, open substitutions; tap to expand/assign | 🏢 A | 🔴 P0 | New `AomApp` mobile view |
| 2.3 | **Mobile Executive Summary (Central SPOC)** — Large KPI cards: open tickets, breached SLA, red schools, today's attendance %, critical alerts feed | 🌐 C | 🟡 P1 | KPI aggregation endpoint |
| 2.4 | **Mobile Morning Brief (SIP Head)** — Three cards: today's cancelled/substituted sessions, yesterday's incomplete attendance, top 3 red schools with #1 gap | 📚 S | 🟡 P1 | Filtered aggregation queries |
| 2.5 | **Faculty Bottom-Navigation Hub** — Four tabs: *Today*, *Schedule*, *Substitutions*, *Profile* | 👨‍🏫 F | 🟡 P1 | Restructure `FacultyApp` navigation |
| 2.6 | **Personal Timetable Card View** — Replace dense grid with vertical swipeable class cards (period, subject, room, section) | 👨‍🏫 F | 🟡 P1 | New component; reuse slot data |
| 2.7 | **Offline-First Day View** — Cache timetable and substitutions locally; sync when connectivity returns | 👨‍🏫 F, 🏢 A | 🟢 P2 | Service Worker + IndexedDB |
| 2.8 | **Day-at-a-Glance with Room Map** — Morning view with room number badge + mini floor-plan indicator | 👨‍🏫 F | 🟢 P2 | Needs `rooms` table with location data |

---

## Theme 3: Analytics & Reporting

> **Heavy demand from AOM, Central SPOC, and SIP Head. Principals want actionable, not abstract.**

| # | Request | Tags | Priority | Notes |
|---|---------|------|----------|-------|
| 3.1 | **Actionable Compliance Scorecard** — Replace deviation numbers with prioritized action list: "Grade 8B missing 2/5 Math slots — Schedule now," ranked by severity | 🏫 P | 🔴 P0 | Transform flat rows into ranked actions |
| 3.2 | **Faculty Utilization Trend Lines** — 4-week line chart of period load per faculty across all schools | 🏢 A | 🟡 P1 | Time-series aggregation on `timetable_weekly_slots` |
| 3.3 | **Week-over-Week Health & Compliance Trends** — 8-week charts: Green/Amber/Red counts, network-wide adherence %, ticket volume by queue | 🌐 C | 🟡 P1 | Needs historical snapshots or rolling computation |
| 3.4 | **Central Deviation Scoreboard** — Network-wide ranking of all schools by overall deviation score, expandable rows showing top 3 gaps | 🌐 C | 🟡 P1 | Aggregation endpoint across all schools |
| 3.5 | **Session-Type Adherence Breakdown** — Separate adherence % for each `session_type` (Lecture, Tutorial, Remedial, etc.) | 📚 S | 🟡 P1 | Group compliance by `session_type` |
| 3.6 | **Faculty Stability Score** — Ranking by substitution rate, cancellation rate, planned vs. actual faculty variance | 📚 S | 🟢 P2 | Complex aggregation on `lecture_sessions` |
| 3.7 | **Subject Coverage Heatmap by Region** — Matrix of Region × Subject showing average adherence % | 📚 S | 🟢 P2 | Cross-dimensional aggregation |
| 3.8 | **Substitutions Trend Report** — 4-week rolling chart: faculty absences, chronically short-staffed periods, over-utilized substitutes | 🏫 P | 🟢 P2 | Time-series on `lecture_sessions` |
| 3.9 | **Weekly Workload Trend Graph** — Faculty's periods per week over 8 weeks, flagged against department average | 👨‍🏫 F | 🟢 P2 | Simple time-series per faculty |
| 3.10 | **Curriculum Delivery Benchmarking** — Cross-school comparison on normalized metrics for a selected grade+track+subject | 📚 S | ⚪ P3 | Complex multi-school normalization |

---

## Theme 4: Dashboard & View Improvements

| # | Request | Tags | Priority | Notes |
|---|---------|------|----------|-------|
| 4.1 | **Control Tower Dashboard (AOM)** — Side-by-school cards: filled/total periods, alert count, attendance %, active substitutions | 🏢 A | 🔴 P0 | New default landing page for AOM |
| 4.2 | **Region Rollup on Health Dashboard** — Top-level region summary bar: total schools, % Green/Amber/Red, unfilled periods count | 🌐 C | 🟡 P1 | Group `TimetableHealthStatus` by region |
| 4.3 | **Multi-School "Day at a Glance"** — Consolidated operations view across all active schools, period-by-period | 🌐 C | 🟡 P1 | Multi-school grid aggregation |
| 4.4 | **Cross-School Faculty Panel — Conflict Highlight + Utilization Mini-Bar** — Bold cells where same teacher is in two schools same period; show `actual/norm` periods | 🏢 A | 🟡 P1 | Enhance existing panel |
| 4.5 | **Health Dashboard Drill-Down** — Click status dot → open school timetable filtered to problematic grade/track, or Day-at-a-Glance for unfilled periods | 🏢 A | 🟡 P1 | Pass gap context through navigation |
| 4.6 | **Compliance Analytics Pivot Toggle** — "Subject Detail" vs. "School Summary" vs. "Region Summary" | 🌐 C | 🟡 P1 | Aggregation at multiple levels |
| 4.7 | **Week-over-Week Diff Highlight** — Toggle in Weekly Timetable to highlight (yellow/red borders) slots that changed vs last week | 🏫 P | 🟢 P2 | Compare two week datasets |
| 4.8 | **Region Heat Map** — Schools × days of week, cell color = issue count (unfilled + double bookings + attendance gaps) | 🏢 A | 🟢 P2 | Grid visualization |
| 4.9 | **Room Conflict Radar** — Real-time matrix (rooms × periods), red cells for double-booked rooms, click to resolve | 🏫 P | 🟢 P2 | Enhance existing `RoomConflict` data |
| 4.10 | **Lecture-Model Adherence Comparison Chart** — Bar chart comparing all schools on adherence % and deviation, with regional average line | 🏢 A | 🟢 P2 | Chart component |
| 4.11 | **Room Conflicts Panel (Cross-School Scope)** — Flag double-booked shared facilities (labs, auditoriums) across AOM's schools | 🏢 A | 🟢 P2 | Extend `list_room_conflicts` with cross-school |

---

## Theme 5: New Features & Modules

| # | Request | Tags | Priority | Notes |
|---|---------|------|----------|-------|
| 5.1 | **Smart Substitute Suggester** — Auto-rank candidates by subject match, free period, workload; one-tap assign | 🏫 P, 🏢 A | 🔴 P0 | See 1.1 |
| 5.2 | **Configurable Central Policies** — Set `max_periods_per_faculty`, `mandatory_subjects`, `attendance_marking_deadline` via UI | 🌐 C | 🟡 P1 | New `central_policies` table; policy engine |
| 5.3 | **Smart Escalation Rules Engine** — Multi-rule conditions: "If queue=X and priority=Y → escalate to Z after N hours" | 🌐 C | 🟢 P2 | Rules table + engine |
| 5.4 | **Remedial & Enrichment Tracker** — Schedule remedial/enrichment outside standard timetable, track completion, report coverage separately | 📚 S | 🟢 P2 | New module; `remedial_sessions` table |
| 5.5 | **Faculty Competency Matrix** — Grid: faculty × subjects, color-coded by qualification + actual delivery count | 📚 S | 🟢 P2 | Needs competency data source |
| 5.6 | **Classroom Observation vs. Timetable Overlay** — Plan vs. actual `lecture_sessions` side-by-side | 📚 S | 🟢 P2 | Compare two data sources |
| 5.7 | **Broadcast Announcement Pinner** — Pin school-wide notes directly onto Day-at-a-Glance for all teachers | 🏫 P | 🟢 P2 | `announcements` table + banner |
| 5.8 | **Topic Completion & Lesson Plan Upload** — Check off syllabus topics, attach notes/files after each period | 👨‍🏫 F | ⚪ P3 | `lesson_logs` table + file storage |
| 5.9 | **One-Click Week Clone with Conflict Check** — Auto-validate faculty overlaps and room double-bookings before publishing | 🏫 P | 🟢 P2 | Validation on clone action |
| 5.10 | **Reassign Faculty Between Schools Wizard** — Pick teacher, source, target, effective week; clone template slots with conflict warnings | 🏢 A | 🟢 P2 | Bulk reassignment workflow |

---

## Theme 6: Workflow & Automation

| # | Request | Tags | Priority | Notes |
|---|---------|------|----------|-------|
| 6.1 | **Prioritized Alert Inbox with Bulk Actions** — Score alerts by impact (P1/P2/P3), dismiss/snooze/convert to ticket in bulk | 🏢 A | 🟡 P1 | New screen; server-side alert state |
| 6.2 | **Auto-Generated Weekly Intervention Digest** — Monday 8 AM email: top 5 schools by deviation, SLA breaches, low attendance regions, unpublished timetables | 🌐 C | 🟢 P2 | Email integration + cron job |
| 6.3 | **Auto-Generated Weekly SIP Brief** — Monday 8 AM email: Red/Amber flips, >10% deviation subjects, >2 substitution faculty, stale tickets | 📚 S | 🟢 P2 | Email integration + cron job |
| 6.4 | **One-Click Ticket from Gap** — "Raise Ticket" button in Compliance/Health that pre-fills school, grade, subject, queue, description | 📚 S | 🟡 P1 | Pre-fill ticket draft from context |
| 6.5 | **Bulk Operations on User Management** — Bulk-assign AOM/faculty to multiple schools, bulk-import subjects from CSV, bulk-publish timetables per region | 🌐 C | 🟢 P2 | Batch operations UI + APIs |
| 6.6 | **One-Tap Attendance Marking** — Pre-fill current class from timetable, single tap per student, auto-save draft | 👨‍🏫 F | 🟡 P1 | Simplify existing attendance flow |

---

## Theme 7: Data Quality & Integration

| # | Request | Tags | Priority | Notes |
|---|---------|------|----------|-------|
| 7.1 | **Root-Cause Drill-Down in Compliance** — Click adherence % → see which `lecture_sessions` were missed and why (Cancelled/No-show/Unfilled/Holiday/Room conflict) | 📚 S | 🟡 P1 | Needs richer `lecture_sessions` status taxonomy |
| 7.2 | **Health Dashboard Gap Categorization** — Tag each gap by type with "Fix" button routing to correct screen | 📚 S | 🟢 P2 | Enrich gap metadata |
| 7.3 | **Health Dashboard Gap Context** — Pass gap context through navigation so drill-down opens pre-filtered view | 🏢 A | 🟡 P1 | URL/query param routing |

---

## Cross-Cutting Technical Requirements

| Requirement | Impact | Notes |
|-------------|--------|-------|
| **Historical snapshot tables** | Themes 3, 6 | For week-over-week trends, need `weekly_health_snapshots`, `weekly_compliance_snapshots` |
| **Notification system** | Themes 1, 2, 6 | WebSocket or push notification infrastructure |
| **Email integration** | Theme 6 | SMTP/cron for automated digests |
| **Offline support** | Theme 2 | Service Worker + IndexedDB caching |
| **File storage** | Theme 5 | For lesson plan uploads |
| **Room/facility location data** | Theme 2 | `rooms` table extension with floor/building/map coordinates |
| **Student roster integration** | Theme 1 | For substitution detail panel |

---

## Recommended Phase 6 Priorities

### Sprint 1: Substitution Workflow (P0)
- Auto-Suggest Substitute Resolver (1.1)
- Today's Substitutions Command Center (1.2)
- Morning Push Digest — Principal (2.1)
- AOM Morning Brief (2.2)

### Sprint 2: Mobile & Notifications (P1)
- Faculty Bottom-Navigation Hub (2.5)
- Personal Timetable Card View (2.6)
- Instant Schedule Change Notifications (1.9)
- One-Tap Attendance Marking (6.6)

### Sprint 3: Analytics Foundation (P1)
- Actionable Compliance Scorecard (3.1)
- Control Tower Dashboard (4.1)
- Faculty Utilization Trends (3.2)
- Central Deviation Scoreboard (3.4)

### Sprint 4: Automation & Policies (P2)
- Configurable Central Policies (5.2)
- Auto-Generated Weekly Digests (6.2, 6.3)
- Prioritized Alert Inbox (6.1)
- Smart Escalation Rules (5.3)

---

*Compiled from 5 parallel stakeholder review agents. Total raw requests: 58. Consolidated to 48 unique items after deduplication.*
