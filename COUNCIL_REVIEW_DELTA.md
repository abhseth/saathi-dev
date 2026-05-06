# SAATHI Critical Review — Delta (After P0 Fixes)

**Date:** 2026-04-30  
**Scope:** Delta review after 11 fixes deployed (5 security + 6 structural)  
**Status:** 11 fixes verified · **4 critical NEW regressions found** · **22 remaining issues**

---

## Executive Summary

The 11 deployed fixes are **functionally correct in isolation**, but the delta review uncovered **4 critical new regressions** introduced by those fixes, plus **several remaining gaps that have worsened** due to interaction effects.

**Most dangerous NEW finding:** Soft-deleted timetable slots continue spawning phantom lecture sessions every day because `list_all_today_sessions` and `list_faculty_today_sessions` CTEs were never updated to filter `deleted_at IS NULL`.

**Second most dangerous:** `refresh_escalations` runs a full-table write storm on **every** `list_tickets` request, turning read operations into O(n) DoS vectors.

**Key insight:** The fixes were tactical and localized. No agent reviewed cross-file interactions, so regressions slipped through at the integration boundaries.

---

## 1. NEW Critical Regressions (Introduced by Our Fixes)

### 🔴 R1. Soft-Deleted Timetable Slots Spawn Phantom Sessions Every Day
**Introduced by:** Fix #7 (soft-delete) + Fix #6 (Migration 40)  
**Severity:** P0  
**Files:** `backend/src/repositories.rs:3860-3893`, `repositories.rs:4315-4336`

The CTEs in `list_all_today_sessions` and `list_faculty_today_sessions` select from `timetable_slots` without filtering `deleted_at IS NULL`:

```sql
SELECT ts.id AS template_id
FROM timetable_slots ts
WHERE ts.day_of_week = ?2
  AND NOT EXISTS (SELECT 1 FROM holidays ...)
  -- MISSING: AND ts.deleted_at IS NULL
```

**Impact:** A soft-deleted slot (e.g., obsolete Grade 11 JEE slot) continues generating `lecture_sessions` rows every day. Faculty see phantom sessions. Attendance reports include non-existent classes.

**Fix:** Add `AND ts.deleted_at IS NULL` to both CTEs.

---

### 🔴 R2. `refresh_escalations` is a Synchronous Full-Table DoS on Every Read
**Severity:** P0  
**File:** `backend/src/routes/tickets.rs:19`

`list_tickets` calls `refresh_escalations` on **every request**. The function selects **all tickets**, loops in Rust, and issues one UPDATE + history INSERT per changed ticket. Under the 10-connection SQLite pool, concurrent list requests queue-lock each other.

**Impact:** At ~1,000 tickets, every list view triggers ~1,000 writes. At ~10,000 tickets, the app becomes unresponsive.

**Fix:** Move escalation refresh to a background Tokio task (every 60s) or use SQLite triggers (`AFTER UPDATE ON tickets`).

---

### 🔴 R3. CSV Formula Injection Bypass via Quoted Fields
**Introduced by:** Fix #5 (CSV sanitization)  
**Severity:** P1  
**File:** `backend/src/routes/export.rs:6-19`

The `field()` function wraps strings containing commas/quotes/newlines in `"..."` **before** checking for formula prefixes. A payload like `=cmd|' /C calc'!A0,extra` becomes `"=cmd|' /C calc'!A0,extra"`. The first char is `"`, so no `'` prefix is added. Excel still executes the formula inside the quotes.

**Fix:** Check for formula triggers **before** CSV escaping.

---

### 🔴 R4. 90 Explicit `AppError::internal(format!())` Calls Bypass Sanitizer
**Introduced by:** Fix #9 (500 sanitization)  
**Severity:** P1  
**Files:** `backend/src/routes/*.rs` (90 occurrences)

The `From<String>` impl sanitizes errors, but route handlers explicitly construct:
- `"DB pool error: {e}"` → exposes pool health/timeouts
- `"VACUUM INTO failed: {e}"` → exposes filesystem paths
- `"Backup init failed: {e}"` → exposes SQLite internals

**Fix:** Audit all `AppError::internal(format!(...))` calls and replace with generic messages.

---

## 2. Remaining Critical Issues (Pre-Existing, Still Unfixed)

### 🔴 C1. Ticket Status: Any → Any Transitions (No State Machine)
**Severity:** P1  
**File:** `backend/src/repositories.rs:368-424`

`validate_status` only checks membership, not transitions. `Open → Closed` without resolution is allowed.

### 🔴 C2. Faculty Calendar Conflicts Unenforced
**Severity:** P1  
**Files:** `repositories.rs:3519-3602`, `repositories.rs:3781-3800`, `repositories.rs:4209-4224`

Weekly overrides, makeup sessions, and substitutions bypass double-booking checks.

### 🔴 C3. `get_student_timeline` Loads Unscoped Global Data + Misses Old Records
**Severity:** P1  
**File:** `repositories.rs:249-306`

Calls `list_tickets(conn, None)` (now capped at 1000) → only sees 1000 most recent global tickets. Filters in-memory by student name. Old tickets vanish. History and attachments are **unbounded**.

### 🔴 C4. Rate Limiter Allows Username Enumeration + Unlimited Usernames
**Severity:** P1  
**File:** `backend/src/routes/auth.rs:14-45`

No IP-based cap. Attacker gets 5 attempts × unlimited usernames. Error messages distinguish "lockout" vs "invalid password", enabling username enumeration.

### 🟡 C5. CSV Imports Row-by-Row Without Transactions
**Severity:** P1  
**File:** `backend/src/routes/imports.rs`

All CSV imports loop with per-row INSERT/UPDATE, no `BEGIN TRANSACTION`.

### 🟡 C6. Cancelled Sessions Spawn Phantom Attendance (GET Route)
**Severity:** P2  
**File:** `backend/src/routes/faculty.rs:220-232`

`session_attendance` GET calls `ensure_session_students` which inserts `Absent` rows without checking if session is cancelled.

### 🟡 C7. Viewer/Agent Global Read Scope
**Severity:** P2  
**File:** `backend/src/auth.rs:93-101`

`scope_filter` returns `None` for viewer/agent, granting cross-school visibility.

---

## 3. Fix Verification Matrix (11 Fixes)

| # | Fix | Status | Notes |
|---|-----|--------|-------|
| 1 | JWT fallback removed | ✅ Solid | No fallback path |
| 2 | Test seeding gated | ✅ Solid | `TEST_SEED=1` required |
| 3 | Scope bypass closed | ✅ Solid | Resolves school before enforcing scope |
| 4 | Rate limiting | ⚠️ Weak | Per-username only; unlimited usernames; enumeration leak |
| 5 | CSV formula injection | ❌ Bypassed | Quoted fields bypass `'` prefix |
| 6 | Migration 40 FKs | ⚠️ Data loss | Orphan DELETE destroyed sessions before new schema preserved them |
| 7 | Soft-delete | ⚠️ Regression | CTEs in today-sessions don't filter `deleted_at IS NULL` |
| 8 | LIMIT 1000 | ⚠️ Partial | 12 functions capped; 7 list functions still unbounded |
| 9 | 500 sanitization | ❌ Bypassed | 90 explicit `AppError::internal(format!())` calls leak info |
| 10 | Admin read endpoints | ✅ Solid | All 4 now require `admin` or `aom` |
| 11 | CORS hardened | ✅ Solid | Env-configurable origin |

**Score:** 5 fully solid, 4 with bypasses/regressions, 2 partially fixed.

---

## 4. Agent-by-Agent Findings

### Database Agent
- **Migration 40:** FKs restored correctly, but orphan cleanup `DELETE`d data the new schema would have preserved as `NULL`
- **Migration 41:** Soft-delete column added correctly
- **Indexes:** Well-chosen, cover hot query paths
- **New issue:** CTEs in today-sessions queries forgot `deleted_at IS NULL` filter
- **Still missing:** `updated_at` on mutable tables, `tickets.school_id` FK, denormalized `school_name`
- **Big bet:** Session instances table (materialized view) replacing runtime CTE merges

### Workflow Agent
- **Soft-delete regression confirmed:** Phantom sessions from deleted slots
- **LIMIT 1000 creates silent data loss:** Large schools with >1000 students see truncated lists
- **Scope bypass remains:** Viewer/agent global read
- **State machine still missing:** Any → Any transitions
- **Big bet:** Unified Scheduling & Conflict Engine (graph-based availability validator)

### UI/UX Agent
- **Validation messages now invisible:** Generic 500s mask helpful errors ("School already exists", "Invalid grade level")
- **403 errors lack contextual handling:** Non-admin hitting admin view gets generic banner, not friendly empty-state
- **Rate-limit feedback static:** No countdown, no retry-after, no auto-clear
- **No structural progress:** `components.tsx` still 8,851 lines, `App.tsx` still 61 `useState`
- **Big bet:** TanStack Query + paginated virtualized lists

### Architecture Agent
- **15× `MAX_ROWS` duplication:** Same constant copy-pasted across 12 functions
- **SQL injection risk:** `list_tickets` scope filter uses string interpolation for IN clause
- **`Paginated<T>` is dead code:** Added but never used
- **No new structural progress:** Monoliths remain intact
- **Big bet:** `ScopedQuery` builder to eliminate duplication, SQL injection, and dead pagination code

### Security Agent
- **4 NEW regressions found** (see Section 1)
- **`refresh_escalations` is P0 DoS:** Full-table write storm on every read
- **Rate limiter:** Username enumeration via differential error messages
- **Big bet:** Move escalation refresh to background + enforce repository-level scope guards

---

## 5. Immediate Action Plan (This Week)

### Day 1: Fix Regressions

| # | Fix | File | Effort |
|---|-----|------|--------|
| R1 | Add `AND ts.deleted_at IS NULL` to today-sessions CTEs | `repositories.rs` | 30 min |
| R2 | Move `refresh_escalations` out of `list_tickets` request path | `routes/tickets.rs` | 2 hours |
| R3 | Fix CSV formula check order (before escaping) | `routes/export.rs` | 30 min |
| R4 | Audit + sanitize 90 explicit `AppError::internal(format!())` | `routes/*.rs` | 2 hours |

### Day 2-3: Close Critical Gaps

| # | Fix | File | Effort |
|---|-----|------|--------|
| C1 | Add ticket state transition matrix | `repositories.rs` | 2 hours |
| C3 | Fix `get_student_timeline` to query scoped + unbounded history/attachments | `repositories.rs` | 2 hours |
| C4 | Add IP-based rate limit + uniform error message | `routes/auth.rs` | 2 hours |
| C6 | Move session-status guard into repository layer | `repositories.rs` | 1 hour |

### Day 4-5: Structural Quick Wins

| # | Fix | File | Effort |
|---|-----|------|--------|
| — | Add `LIMIT 1000` to 7 remaining unbounded list queries | `repositories.rs` | 1 hour |
| — | Extract `MAX_ROWS` constant to module level | `repositories.rs` | 30 min |
| — | Parameterize `list_tickets` IN-clause | `repositories.rs` | 1 hour |

---

## 6. Big Bet Synthesis (Council Consensus)

All 5 agents independently converged on similar themes:

| Theme | Agents | Recommended Action |
|-------|--------|-------------------|
| **Query builder / scoped access** | Database + Architecture + Security | Build `ScopedQuery` helper; enforce scoping at repository level |
| **Background jobs for side effects** | Workflow + Security | Move `refresh_escalations` + notifications to Tokio background tasks |
| **Real pagination** | UI/UX + Architecture | Replace 1000-row caps with `Paginated<T>` + frontend controls |
| **Unified scheduling engine** | Workflow + Database | Materialize session instances; centralize conflict checks |
| **React Query for state management** | UI/UX + Architecture | Migrate `App.tsx` 61-useState monolith to `@tanstack/react-query` |

**Council consensus:** The highest-leverage single investment is a **repository-level query builder with mandatory scoping** (`ScopedQuery`). It fixes:
- SQL injection in `list_tickets`
- 15× `MAX_ROWS` duplication
- Unscoped queries in `get_student_timeline`
- Makes `Paginated<T>` live code
- Provides a path to real cursor pagination

Second priority: **Move `refresh_escalations` to a background task** — this is a P0 stability issue that affects every user on every page load.

---

*Delta report generated by Council of 5 after P0 fix deployment. 4 new regressions found. 22 total issues remaining.*
