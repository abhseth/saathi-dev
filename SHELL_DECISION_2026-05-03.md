# SAATHI Shell Decision Document

Date: 2026-05-03
Scope: canonical navigation model and role landing behavior

## Decision: Current Shell Remains Canonical for Admin/Agent/Viewer/AOM

For the stabilization phase, the **current Sidebar + adminView + AdminPanelRouter shell** is declared canonical for `admin`, `agent`, `viewer`, and `aom` roles.

### Rationale

- The Sidebar/adminView shell is already type-safe and functional after Phase 1.
- The newer section-based shell (LeftRail, navigation.ts, landing pages) is partially implemented but not yet wired as the live source of truth.
- Migrating to the new shell now would introduce UI regression risk before backend authorization and feature reachability are complete.
- The plan is: stabilize roles in the current shell first, then migrate the shell in a later phase.

## Active Role Landing Pages (Implemented)

The following dedicated components are now wired as active role landings:

- `frontend/src/components/faculty/FacultyApp.tsx` — active landing for `faculty` role.
- `frontend/src/components/approver/ApproverDashboard.tsx` — active landing for `head` role.

## Deferred Shell Assets

The following files are **deferred** to a future shell migration phase. They remain in the tree but are not the active navigation source of truth:

- `frontend/src/navigation.ts`
- `frontend/src/components/LeftRail.tsx`
- `frontend/src/components/landing/*`

## Role Landing Matrix

| Role | Canonical Landing | Rationale |
|------|-------------------|-----------|
| `admin` | Ticket/work shell (current) | Admin needs full tool access; current shell provides this. |
| `agent` | Ticket/work shell (current) | Agent's primary work is ticket queue; current shell is correct. |
| `viewer` | Ticket/work shell (current) | Read-only overview; backend enforcement deferred to Phase 3. |
| `aom` | Ticket/work shell (current) | AOM needs approvals/substitutions; dedicated surface deferred. |
| `faculty` | `FacultyApp` | Dedicated faculty app with attendance, schedule, substitutions. |
| `head` | `ApproverDashboard` | Dedicated approver dashboard; real data for leave, substitutions, sessions, and timetable health loaded on mount. Alerts and drill-down callbacks are intentionally incomplete Phase 2 gaps. |

## Known Gaps

### FacultyApp
- All required props are available from `useFacultyState`.
- `onLoadWeeklySlots` uses a local calendar date helper (`localToday()`) to avoid UTC/local-date drift around midnight.
- `onDeclineSubstitution` arity: FacultyApp expects `(sessionId, reason)` but hook provides `(requestId)`. TypeScript allows this assignment (extra params are ignored).

### ApproverDashboard
- `alerts`: no hook provides `Alert[]`. Passed as `[]` (empty state) until alert endpoints are consumed.
- `onOpenLeaveSwap`, `onOpenSubstitutions`, `onOpenAlerts`, `onOpenDayAtGlance`, `onOpenTimetableHealth`: all navigation callbacks are no-ops until sub-panels are wired.
- These are explicitly incomplete Phase 2 gaps, not finished functionality.

## Next Phase

After backend authorization (Phase 3) and feature reachability (Phase 4) are complete, revisit shell migration:
1. Evaluate LeftRail/section shell readiness.
2. Port role landing logic to the new shell.
3. Delete or archive deferred shell assets.
