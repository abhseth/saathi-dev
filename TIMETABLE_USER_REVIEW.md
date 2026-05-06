# Timetable Views — Stakeholder Review Report

**Date:** 2026-05-01
**Reviewers:** 5 stakeholder agents (Principal, Center Head, Central SPOC, SIP Head, Faculty)
**Purpose:** Collect timetable-view requirements from each role to guide the dev team

---

## 1. Executive Summary Matrix

| Stakeholder | Primary View | Granularity | Mobile Critical? | Print Need |
|-------------|-------------|-------------|------------------|------------|
| **Principal** | Day-at-a-glance (default), Grade-wise, Faculty-wise | School | **Yes** — corridor lookups, parent queries | PDF (parents/inspectors), A4 print (noticeboard) |
| **Center Head** | Region-wide compliance dashboard, Cross-school faculty | 6 schools + region | **Yes** — travel, school visits | PDF (regional office), Excel (school prints) |
| **Central SPOC** | National health dashboard, Ticket-linked timetable context | National → Region → School | No (desktop-first) | Excel/CSV (escalation), PDF (meetings) |
| **SIP Head** | Program-wide compliance analytics, Deviation scores | Program → Grade → School | No (desktop-first) | PDF (school visits), Excel (council meetings) |
| **Faculty** | Today (mobile), Week view (planning), Substitution inbox | Personal | **Yes** — between classes, walking | Simple PDF (backup print) |

---

## 2. Current Implementation vs. Needs Gap Analysis

### What exists today (5 views)

| View | Data Source | R/W | Scope | Audience |
|------|-------------|-----|-------|----------|
| **Timetable** | `timetable_slots` (recurring template) | Write | School + Grade + Track + Batch | Admin/AOM — edits master pattern |
| **Weekly Timetable** | `timetable_weekly_slots` (date-specific) | Write | School + Grade + Track + Batch + Week | Admin/AOM — plans actual weeks |
| **School Master** | `timetable_weekly_slots` | Read | School + Week (all batches) | Ops — school-wide printout |
| **Grade Timetable** | `timetable_weekly_slots` | Read | School + Grade + Week | Grade coordinator |
| **Faculty Timetable** | `timetable_weekly_slots` | Read | School + Faculty + Week | Faculty scheduling |

### What's missing (stakeholder-identified gaps)

| # | Gap | Who Needs It | Current Workaround |
|---|-----|-------------|-------------------|
| 1 | **Day-at-a-glance view** — entire school today, period by period | Principal | None — not built |
| 2 | **Cross-school faculty view** — one teacher's schedule across all schools | Center Head, SPOC | None — not built |
| 3 | **National/Regional health dashboard** — Green/Amber/Red per school | SPOC | None — not built |
| 4 | **Ticket ↔ Timetable integration** — timetable context inside tickets, raise ticket from timetable | SPOC | Manual — separate systems |
| 5 | **Compliance analytics** — deviation scores, planned vs. actual, subject coverage gaps | SIP Head, Center Head | Excel export + manual analysis |
| 6 | **Substitution management UI** — accept/decline, history, notifications | Faculty | WhatsApp groups |
| 7 | **Mobile-optimized personal timetable** — swipe days, next-class banner, color coding | Faculty | None — Faculty App has "Today" only |
| 8 | **Room/lab conflict detection** | Principal, Center Head | Discovered physically |
| 9 | **Alert system** — unfilled periods, double bookings, faculty overload | All | Phone calls / WhatsApp |
| 10 | **Batch vs. class plan mismatch detection** | SPOC, Center Head | Manual audit |

---

## 3. Detailed Stakeholder Findings

---

### 3.1 School Principal

**Context:** Grades 6–12, ~800 students, 40 faculty. Manages daily operations, parent communication, crisis response.

**Must-Have Views:**
1. **Day-at-a-glance** (default screen) — entire school schedule for *today*, period by period, scrollable. Used during morning assembly and unexpected events.
2. **Grade-wise view** — pull up any grade instantly for parent complaints (e.g., "my child has no teacher in Period 3").
3. **Faculty-wise view** — check teacher availability for meetings, substitutions, or verification.

**Information per cell:** Subject + Faculty name + Room/Lab number + Batch/Section. Substitution flagged visually (red dot).

**Filters:** Grade, Batch, Faculty name (alphabetical search), Day, Subject, Room/Lab.

**Primary use cases:**
- Parent calls at 9:15 AM: "No teacher in my child's class" → verify scheduled vs. actual
- Teacher calls in sick → find who is free in that period for substitution
- Exam season prep → verify no subject is neglected
- Parent claims extracurricular clash → show free periods

**Alerts needed:**
- 🚨 "Teacher absent — substitution not assigned" (critical, before period starts)
- 🚨 "Double booking" (teacher or room)
- 🚨 "Grade X has no [Subject] teacher on [Day]" (core subjects)
- ℹ️ "Timetable changed today" (overnight edits)

**Pain points with current paper/Excel:**
- Cannot search (3 Excel sheets to find when Mrs. Sharma is free)
- Changes invisible (paper timetable wrong until reprinted)
- No conflict detection (discovered when two teachers show up at lab door)
- Sharing painful (can't send parent a clean view of just their child's schedule)
- Paper outdated by October

> *"I need a timetable that acts like a live control panel, not a static document."*

---

### 3.2 Center Head (6 schools)

**Context:** Oversees 6 schools, ~4,000 students, 200 faculty. Reports to regional office. Coordinates cross-school faculty sharing.

**Must-Have Views:**
1. **Region-wide weekly snapshot** (landing page) — control tower view of all 6 schools side-by-side
2. **Single-school drill-down** — full weekly timetable, period by period
3. **Cross-school faculty view** — click a teacher, see their schedule across ALL schools they serve

**Information needed:**
- Scheduled vs. filled periods per school
- Faculty active vs. idle per day
- Batch counts by grade (spot double/single batches)
- Faculty utilization (periods per week, cross-school splits)
- Subject-wise coverage (min periods for Math, Science, English)

**Filters:** School (multi-select), Grade, Subject, Faculty, Lecture model type, **"Shared faculty"**, **"Gaps only"**.

**Primary use cases:**
- School A's Science teacher on leave → find School B's Science teacher with a free period
- Identify underutilized teachers (12 periods when norm is 24)
- Verify SAATHI lecture model compliance (not defaulting to lecture-only)

**Compliance Dashboard (for regional officer visits):**
- 4 KPIs at top: % periods filled, % matching lecture model, % subjects meeting min periods, % shared faculty with no conflicts
- Bar chart comparing 6 schools, red-yellow-green
- Click red school → see exact periods/teachers/subjects causing the problem

**Alerts:**
- 📱 "School X has Y unfilled periods today" (8 AM daily)
- 📱 "Faculty [Name] double-booked at School A and School B on [Day] Period Z"
- 📊 Weekly Friday: "School X below 80% lecture model compliance"
- 📊 "Faculty [Name] utilization below 60% this week"

**Print/Export:**
- One-page PDF summary (Monday → regional office): attendance vs. scheduled, lecture model breakdown, gaps
- Printable weekly timetable per school (clean grid, staffroom display)

---

### 3.3 Central SPOC

**Context:** National single point of contact. Bridges ground-level staff and central management. Handles escalations, coordinates with tech team.

**Must-Have Views:**
1. **National overview** — Region → Schools → % Complete Timetable (spot trouble regions in 10 seconds)
2. **Regional drill-down** — traffic-light status (Green/Amber/Red) per school + AOM name
3. **School-specific detail** — grade-by-grade, track-by-track, faculty names, session statuses

**Information needed:**
- Ticket-linked schedule issues (ticket says "Grade 11 JEE Physics not happening" → verify slot exists, faculty assigned, session history)
- Faculty overload across ALL schools (quantify 28 periods across 3 schools)
- School-wise completeness (fully populated vs. partial vs. not started)
- Batch vs. class plan mismatch (class plan says 2 sections, timetable has 1)
- Attendance linkage (sessions generated but attendance not marked by 11 AM)

**Ticket ↔ Timetable Integration (critical gap):**
- Inside ticket view: **"Timetable Context" panel** showing:
  - School's master timetable snapshot for relevant grade/track
  - Last 7 days + next 7 days of lecture sessions
  - Gaps (missing faculty, cancelled without substitute)
  - Direct link to full School Timetable Detail
- From timetable view: **one-click raise ticket** pre-filled with school, grade, subject, date range
- Auto-suggest ticket linking when title/description mentions grade + subject

**Filters:** Region, School, Program Model, Grade, Track, Batch Pattern, Faculty, Issue Type, **Timetable Health Status**.

**Timetable Health Dashboard (morning newspaper):**

| School | Region | AOM | Program Model | Class Plans? | Master Timetable? | Sessions Generated? | Gaps Found? | Last Updated | Status |
|---|---|---|---|---|---|---|---|---|---|

- **Green:** Plans configured, slots exist for all grades/tracks/batches, sessions generated, no unassigned faculty
- **Amber:** Partial (some grades missing, one vacant slot)
- **Red:** No plans, no timetable, or multiple unassigned grades

KPI bar: Total 50 | Green 32 | Amber 12 | Red 6

**Alerts:**
- "School X timetable not published for next week" (Thursday EOD)
- "Faculty absenteeism spike in Region Y" (>X% cancelled over 3 days)
- "Grade X Subject Y unassigned in School Z"
- "Attendance not marked by 11:00 AM"
- SLA risk on schedule-related tickets

**Print/Export:**
- Excel/CSV of "Schools with Incomplete Timetables" (school, region, AOM contact, what's missing, since when)
- PDF of single school's week (A4 landscape, management review calls)

---

### 3.4 SIP Head

**Context:** Academic program quality, curriculum alignment, learning outcomes. Works with subject matter experts. Ensures lecture model implementation.

**Must-Have Views:**
1. **Program-wide analytics** (monthly academic council review)
2. **Grade-wise analysis** (systemic issues: "Grade 9 consistently under-allocated across schools")
3. **School-specific compliance** (school visits, principal feedback)

Drill-down: dashboard summary → single school's weekly timetable in **two clicks**.

**Information needed:**
- Subject-wise period distribution per grade per school
- Faculty-subject-batch mapping
- Lecture model adherence (large group lecture vs. small group tutorial)
- Actual batch sizes vs. planned
- Weekly hours tally for core subjects vs. mandated minimums
- Co-curricular/elective slots (are they appearing or swallowed by core?)

**Analytics / Comparisons:**
- Average Science hours per week across all schools
- Variance in Math periods between schools
- Schools deviating from recommended lecture model by >1 period/week
- Planned vs. actual: subject hours, batch sizes, faculty load
- **Deviation score per school**

**Filters:** Program track, Grade, Subject (core/elective/co-curricular), Lecture model type, School, **Compliance status**.

**Program Health Indicators (red/amber/green):**
1. Subject coverage gaps (subject missing entirely from a grade)
2. Faculty overload (>24 periods/week)
3. Underutilized batches (<80% of planned sessions)
4. Lecture model mismatch (large lectures for tutorial-designated grades)
5. Elective concentration (too many students in one elective, others vacant)

**Alerts:**
- Grade at any school below minimum required periods for core subject
- Batch has no faculty assigned
- Faculty in overlapping periods
- School's lecture model distribution shifts >20% from approved plan
- Weekly Monday digest: all compliance violations from previous week

**Print/Export:**
- PDF: school-specific compliance reports (for principals before visits)
- Excel: consolidated (for academic council meetings)
- One-page summary (for school inspections)

---

### 3.5 Faculty Member

**Context:** Teaches Math Grades 8–10. Uses Faculty App for attendance/session management. Needs instant info while walking between classes.

**Must-Have Views:**
1. **Today** (mobile lifeline) — checked between periods, during breakfast, before leaving home
2. **Week view** — Sunday evening planning, substitution duty prep
3. **Substitution inbox** — accept/decline requests, coverage history

**Information per class:**
- Period, Grade/Section, Room, Planned topic/notes
- For substitutions: original teacher name, subject, room, batch
- Class type: regular lecture / test / lab (changes what to carry)
- Prep periods clearly marked "PREP" (gray)

**Ideal Mobile View:**
- Swipe between days like calendar app
- Top banner: *"Next: Grade 9-A, Period 4, Room 105, 20 mins left"*
- Each period = card with subject, class, room, small dot if substitution
- Color-code by grade: Grade 8 blue, Grade 9 green, Grade 10 orange
- Tap period → show planned topic or notes

**Substitution Management:**
- In-app card/banner (not buried in menu): *"Grade 8 Science, Period 3, Room 204 — accept or decline?"*
- One-tap accept/decline + short reason if declining
- "My Substitutions" view (monthly summary for admin)
- See which colleagues covered my classes (to thank them)

**Filters:** By day (default), By grade/section, By subject (for substitution management).

**Alerts:**
- ⏰ "Next class in 15 minutes — Grade 10-B, Room 302"
- 🏫 "Period 3 shifted to Room 201 today"
- ❓ "Substitution request: Grade 8 Science, Period 5 — accept?"
- ❌ "Your Grade 9 class cancelled tomorrow due to assembly"
- 🌙 Gentle reminder evening before accepted substitution duty

**Print:** Simple clean PDF for the week (no giant logos) — backup for internet failure or dead battery.

**Pain points with current system:**
- WhatsApp groups: 40 messages by 8 AM, miss own substitution duty
- Paper notices: hidden behind others or rained on
- Excel on phone: unreadable
- Worst: finding out about substitution by walking into a room with 40 waiting students

> *"If it's not in the app, it doesn't exist."*

---

## 4. Consolidated Requirements for Dev Team

### A. New Views to Build (in priority order)

| Priority | View | Description | Primary Users |
|----------|------|-------------|---------------|
| P0 | **Day-at-a-Glance** | Entire school today, period by period, all batches | Principal |
| P0 | **Mobile Personal Timetable** | Swipe days, next-class banner, color-coded grades, substitution dots | Faculty |
| P0 | **Substitution Inbox** | Accept/decline cards, history, coverage summary | Faculty |
| P1 | **Cross-School Faculty View** | One teacher's schedule across all assigned schools | Center Head, SPOC |
| P1 | **Timetable Health Dashboard** | Green/Amber/Red per school, national → region → school drill-down | SPOC |
| P1 | **Compliance Analytics** | Deviation scores, planned vs. actual, subject coverage gaps | SIP Head, Center Head |
| P2 | **Ticket-Timetable Context Panel** | Timetable snapshot inside ticket view + one-click ticket raise from timetable | SPOC |

### B. New Features Across Existing Views

| Feature | Description | Affected Views |
|---------|-------------|----------------|
| Room conflict detection | Highlight when same room double-booked | Timetable, Weekly Timetable |
| Substitution flagging | Visual indicator (dot/color) when slot is a substitution | All read-only views |
| Search by faculty | Alphabetically find teacher and see their week | School Master, Grade, Faculty |
| Filter by subject | Show all Physics periods across school on a day | School Master |
| Print-friendly CSS | Large fonts, clear grids, A4 optimized | All read-only views |
| Batch vs. plan mismatch warning | Warn when class plan expects 2 batches but timetable has 1 | Weekly Timetable |

### C. Alert / Notification Requirements

| Alert | Trigger | Audience | Channel |
|-------|---------|----------|---------|
| Unfilled periods today | 8 AM, gaps in today's schedule | Center Head, Principal | App + SMS |
| Teacher absent, no substitute | Attendance marked absent, no sub assigned | Principal | App (red) |
| Double booking | Same teacher/room in same period | Principal, Center Head | App |
| Faculty overload | >24 periods/week or >1 school conflict | Center Head, SPOC | App |
| Core subject below minimum | Grade missing required periods | SIP Head, Center Head | Weekly digest |
| Timetable not published | Next week sessions not generated by Thu EOD | SPOC | App |
| Attendance not marked | Sessions exist, attendance pending at 11 AM | SPOC | App |
| Next class reminder | 15 min before scheduled period | Faculty | Push notification |
| Substitution request | New coverage request | Faculty | Push + in-app banner |
| Room change | Period shifted to different room | Faculty | Push notification |
| Weekly compliance digest | Monday morning summary of all violations | SIP Head, Center Head | Email |

### D. Data / API Gaps

| Gap | What We Need | Current State |
|-----|-------------|---------------|
| Room/Lab field | Store room assignment per slot | `timetable_slots` and `timetable_weekly_slots` have no `room_id` column |
| Lecture model type per slot | Distinguish lecture / tutorial / activity / assessment / remedial | Not stored per slot — only inferred from class plan |
| Session generation tracking | Know whether lecture sessions were generated for a week | No explicit flag — must query `lecture_sessions` |
| Cross-school faculty schedule | Query faculty's slots across ALL schools | API scoped to single school |
| Deviation score | Algorithm: compare actual subject hours vs. mandated minimums | No API — manual calculation |
| Batch coverage % | % of planned sessions that actually ran | No aggregate query |
| Substitution linkage | Link a slot to a substitution record | `lecture_sessions` has `actual_faculty_user_id` but no explicit substitution flag |

---

## 5. Recommended Next Steps

1. **Short-term (sprint):** Build Day-at-a-Glance view for Principals and mobile-optimized Today view for Faculty — these have the highest daily impact.
2. **Medium-term:** Add room field to slots, build conflict detection, and create the Substitution Inbox.
3. **Long-term:** Build the Health Dashboard and Compliance Analytics with drill-down capability. This requires new aggregation APIs and potentially a materialized view or caching layer.
4. **Cross-cutting:** Design the alert/notification system early — most stakeholders want alerts, and they share many triggers.

---

*End of report. Compiled from interviews with 5 simulated stakeholder agents representing real SAATHI user roles.*
