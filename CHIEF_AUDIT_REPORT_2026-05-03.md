# SAATHI Chief Audit Report

Date: 2026-05-03
Scope: deployment readiness, feature richness, UI/style critique, scalability, redundancies, inefficiencies, and half-implemented features.
Method: direct source review, project gate execution, TypeScript type-check, and specialist sub-agent review for frontend UX, backend/security, and product completeness.

## Executive Verdict

SAATHI is not deployment-ready for a production release in its current working tree.

The backend and frontend smoke gates pass, and the codebase contains a much richer product surface than earlier revisions. However, the current active application shell is inconsistent with the newer navigation and role-specific modules. The frontend does not type-check. Several role-critical workflows are either unreachable, visible to the wrong roles, or wired to old component contracts. Backend authorization still has high-risk gaps around ticket mutations, global operations, and stale JWT scope. These are release blockers.

Recommended release posture: do not deploy until the P0/P1 items below are resolved and enforced by CI.

## Verification Performed

- `cd backend && cargo test`: passed, 34 tests.
- `cd frontend && npm run test`: passed, 12 tests.
- `cd frontend && npm run build`: passed.
- `bash scripts/ci-gate.sh`: passed.
- `cd frontend && npx tsc --noEmit`: failed with many TypeScript errors.

Important interpretation: the current CI gate is insufficient. Vite build transpiles successfully but does not type-check, so it misses real broken prop contracts and stale type/model mismatches.

## P0 Release Blockers

### 1. Frontend does not type-check

`npx tsc --noEmit` fails with errors across `App.tsx`, `components.tsx`, `AdminPanelRouter.tsx`, hooks, and shared types.

Examples:

- `App.tsx`: `ProgramFilters` receives `string[]` where `School[]` is expected.
- `App.tsx`: `BottomNav` receives old props such as `activeFilter`, `showingAdmin`, `onFilterChange`, and `onMasterDataClick`, while the component now expects `currentSection`, `mobileView`, `onHomeClick`, and `onWorkClick`.
- `AdminPanelRouter.tsx`: many panels receive old prop shapes that no longer match their component definitions.
- `components.tsx`: many fields referenced in UI no longer exist on types, such as `School.director_name`, `LectureSession.subject_name`, `TimetableSlot.period_number`, and others.
- `hooks/useFacultyState.ts`: references missing `CreateMakeupSessionInput`.
- `types.ts`: references missing `SubjectGap`.

Impact: the passing production build is misleading. Runtime behavior can break even though CI reports success.

Action:

- Add `"typecheck": "tsc --noEmit"` to `frontend/package.json`.
- Make `scripts/ci-gate.sh` fail on type errors.
- Fix type drift before any production deployment.

### 2. Active UI shell is internally inconsistent

The active app still renders the legacy `Sidebar` plus `adminView` model in `frontend/src/App.tsx`, while newer shell assets exist but are not the live source of truth.

Conflicting systems:

- Legacy: `Sidebar`, `adminView`, `toolRegistry.tsx`, `AdminPanelRouter.tsx`.
- Newer: `navigation.ts`, `LeftRail.tsx`, `components/landing/*`, role/task-oriented landing pages.

Impact: major features exist but are orphaned, and future fixes may target dead code. The product appears updated but still behaves like a partially reverted shell refactor.

Action:

- Choose one canonical navigation model.
- Wire all live tools through one registry/render path.
- Remove or quarantine abandoned shell code after migration.
- Add role-by-role smoke tests that assert each seeded role lands on the intended first screen.

### 3. Faculty and head user experiences are not properly routed

The codebase contains a dedicated faculty app and approver dashboard, but the active `App.tsx` does not clearly route `faculty` users into `FacultyApp` or `head` users into `ApproverDashboard`.

Impact: two important scoped roles may log in to an admin/ticket-oriented shell rather than their operational workspace.

Action:

- Add explicit top-level role routing for `faculty`, `head`, and `aom`.
- Validate seeded users: `admin`, `agent`, `viewer`, `aom1`, `faculty1`, `head1`.
- Make the role landing page part of automated UI smoke coverage.

## P1 Security and Authorization Findings

### 4. Ticket mutations are open to too many authenticated roles

Backend ticket create/update/comment handlers enforce school scope but do not enforce a ticket-writer role. Documented read-only `viewer` behavior is not protected server-side. Scoped roles such as `faculty` or `head` may also mutate tickets if they can hit the endpoints.

References:

- `backend/src/routes/tickets.rs`
- `backend/src/routes/mod.rs`

Action:

- Add explicit `require_ticket_reader` and `require_ticket_writer` helpers.
- Allow reads for intended roles only.
- Block `viewer` from all mutations on the backend.
- Mirror backend permissions in UI by hiding or disabling mutating controls.

### 5. Ticket requester and comment author are partly client-controlled

`create_ticket` persists `input.requester`; `add_comment` persists `input.author`. The authenticated actor is used in some audit/history paths, but visible business records can still be spoofed by the client.

Action:

- Derive internal requester/comment author from JWT identity.
- If external requester is a business field, store it separately from `created_by_user_id` / `author_user_id`.
- Update frontend labels to distinguish "reported by" from "created by".

### 6. JWT role and school scope remain valid until token expiry

JWT claims embed role and `school_ids`. Middleware verifies signature/expiry but does not re-check `is_active`, current role, or current school assignments.

Impact: disabling a user or changing their school scope does not immediately revoke existing access.

Action:

- Add token/session versioning, or DB revalidation for sensitive routes.
- Consider shorter access tokens plus refresh/session invalidation.
- Include `is_active`, role, and school-scope revocation tests.

### 7. Scoped users can still perform some global or region operations

Examples found:

- Region-based bulk timetable publish enforces scope only for explicit `school_ids`; if a region expands internally, the final school set may bypass route-level scope checks.
- AOM users can create global announcements when `school_id` is absent.
- AOM users may create global/region holidays because school scope is only enforced for school-scoped holidays.

Action:

- Resolve concrete target school sets before executing bulk operations.
- Enforce scope on each final target school.
- Restrict global/region announcements and holidays to `admin`, unless explicit region-scoped authorization is implemented.

### 8. Manual transaction handling can leave open transactions

Several repository functions manually call `BEGIN` and then use `?` inside the transaction body. Early errors can skip rollback.

Action:

- Prefer `rusqlite::Transaction` RAII.
- If pooled connection mutability is awkward, wrap manual transactions in helper closures that always rollback on error.
- Add failure-path tests for leave approval, swap acceptance, and substitute assignment.

## P1 Functional and UX Findings

### 9. Mobile bottom navigation is broken by prop-contract drift

`BottomNav` expects the new section/mobile-view props, but `App.tsx` still passes the older ticket-filter props. Vite build does not catch this. TypeScript does.

Impact: mobile Home/Work actions can call undefined handlers or render incorrect active state.

Action:

- Adopt one mobile state model: either the new `mobileView` model or the old `showMobileDetail` model.
- Remove the other state path.
- Add a mobile navigation test covering Home, Work, Create, More, and ticket detail back navigation.

### 10. Mobile ticket detail navigation is likely broken

`useTicketState.selectTicket()` can call optional `onSetMobileView("detail")`, but `App.tsx` does not pass that callback. The app still renders detail using older `showMobileDetail` state.

Action:

- Wire `onSetMobileView`, or restore `setShowMobileDetail(true)` inside the active ticket selection path.
- Add a test: mobile list click opens detail, back returns to list.

### 11. Program filters are mismatched with current state

`ProgramFilters` expects `school_ids` and `School[]`, while the active ticket filter state uses `school_name`, `grade_level`, `program_track`, `issue_category`, and `queue`, and passes string school names.

Impact: filters can render blank options and update fields that the ticket filter logic does not use.

Action:

- Standardize filter state to either ID-based or name-based filtering.
- Prefer ID-based filtering for schools, with display names resolved from master data.
- Add tests for each filter field affecting ticket count.

### 12. App eagerly loads too much protected data

On mount and after login, `App.tsx` loads tickets, policies, templates, schools, audit log, all comments, regions, class plans, dashboard, students, subjects, and faculty assignments.

Impact:

- More rate-limit pressure.
- Slower login.
- Unnecessary data exposure surface.
- Harder role-based reasoning.

Action:

- Load only `get_current_user` first.
- Role-gate data loaders after auth.
- Screen-gate heavy resources such as audit log, all comments, reports, analytics, and faculty scheduling.

### 13. Viewer read-only behavior is incomplete

The UI hides ticket creation for viewers, but ticket detail still exposes status/assignment controls. Backend mutation protection is also missing.

Action:

- Enforce viewer read-only at backend route level.
- Pass role into ticket detail/action components.
- Disable or hide status, assignment, comment, and destructive controls as appropriate.

## P2 Feature Completeness Findings

### 14. Many rich features exist but are not reachable in the active shell

Implemented or partially implemented surfaces include:

- `LeaveSwapPanel`
- `SubstitutionCommandCenter`
- `ApproverDashboard`
- `NotificationCenter`
- section landing pages
- multiple analytics dashboards
- notification hooks
- digest endpoints
- substitution reports

The active shell does not expose many of these coherently.

Action:

- Create a feature reachability matrix: route, API command, component, navigation entry, role visibility, tests.
- Promote real features into the canonical navigation model.
- Hide preview/dead-end features until wired.

### 15. Several visible workflows are knowingly unavailable

Examples:

- Ticket attachments expose Browse, Attach File, and Open controls, but handlers only raise "not available" notices.
- Daily Sync export/import paths expose controls but are unavailable in the web version.
- Student timeline action shows a notice even though timeline panel/API concepts exist.
- Digest endpoints generate data but do not send email.
- Alert-to-ticket creates placeholder tickets from alert hash only.
- Substitution detail has placeholder `last_covered_topics`.
- Substitution reports hardcode `avg_minutes_to_fill = 0`.

Action:

- Remove unavailable CTAs from production UI, or mark them disabled with explicit "coming soon/internal preview" state.
- Do not present placeholder operational metrics as real analytics.
- For each unfinished feature, choose one state: implement, hide, or delete.

### 16. Faculty attendance UX is risky for real classroom use

The faculty app cycles attendance through six states on tap: Present, Late, Excused, Leave, Out of Class, Absent.

Impact: efficient for expert users, but easy to mis-mark on mobile and not discoverable.

Action:

- Use explicit segmented controls for common states.
- Put rare states behind a menu or long-press.
- Add undo and save confirmation for bulk attendance.

### 17. Leave/swap workflow is not user-friendly enough

The swap request form asks users to enter raw `slot_a_id` and `slot_b_id`.

Action:

- Replace ID entry with timetable period selectors.
- Show school, grade, subject, current faculty, conflicts, and preview before submit.
- Validate conflicts before submission and again at commit time.

## P2 Scalability and Performance Findings

### 18. SQLite operational posture has improved but remains bounded

Improvements present:

- WAL mode exists.
- `busy_timeout` is configured.
- SLA escalation refresh moved to a background scanner.
- Ticket and all-comment list endpoints now support pagination.
- Health and readiness routes are wired at `/health` and `/ready`.
- CORS is now origin-configurable instead of allowing all origins by default.

Remaining risks:

- Most request handlers still perform synchronous `rusqlite` work inside async handlers.
- Heavy analytics and report queries run synchronously in request handlers.
- Broad startup loading in the frontend can spike API calls.
- Frontend derived state still filters/reduces whole arrays in memory.

Action:

- Keep simple CRUD inline only if measured latency is acceptable.
- Move heavy analytics/import/export/reporting work behind `spawn_blocking`, queue workers, or snapshots.
- Add query-plan review and indexes for dashboard endpoints.
- Add server-side pagination and UI pagination/windowing for ticket lists, comments, and reports.

### 19. Analytics are broad but need product and query discipline

The codebase exposes many analytics commands, but the CI contract check reports 29 dispatch commands unused by frontend source. These include reports such as `control_tower`, `compliance_scorecard`, `faculty_utilization`, `health_trends`, `region_heatmap`, and others.

Action:

- Decide which analytics are production features.
- Wire production analytics into one reports IA.
- Remove or mark orphan commands as internal.
- Add time-window caps and indexes for broad analytics queries.

## UI and Style Critique

### 20. Visual system is split between old and new vocabularies

The CSS contains newer tokens and left-rail/landing styles, while the active app still uses older `.sidebar`, `.ticket-modal`, and admin modal surfaces. This creates a mixed product feel.

Action:

- Define one active component vocabulary for shell, cards, tables, filters, modals, sheets, and action buttons.
- Prune abandoned styles after shell decision.
- Add a visual QA checklist per role and viewport.

### 21. Modal and overlay accessibility is inconsistent

Examples:

- `CreateTicketModal` uses modal-looking UI but lacks a robust dialog primitive.
- `NotificationCenter` is a floating panel without clear dialog/popover semantics.
- Multiple ticket/admin modal surfaces have inconsistent focus, escape, and labelling behavior.

Action:

- Build or standardize one `Dialog`/`Sheet` primitive.
- Required behavior: `role="dialog"`, `aria-modal`, labelled title, focus trap, Escape close, contained scroll, focus restore.
- Migrate Create Ticket, Notification Center, Alert Inbox, and admin modals first.

### 22. Mobile IA is still tool-first rather than task-first

`MobileMoreMenu` lists raw admin tools. For mobile users, especially AOM/head/faculty, the interface should lead with daily tasks.

Action:

- Agent: My queue, follow-ups, SLA risk, create ticket.
- AOM/head: approvals, substitutions, school health, urgent alerts.
- Faculty: today, attendance, requests, substitutions.
- Viewer: read-only overview and reports.

## Redundancies and Maintenance Risks

### 23. Duplicate or competing frontend systems

Observed redundancies:

- `navigation.ts` and `toolRegistry.tsx`.
- `LeftRail`/landing pages and legacy `Sidebar`.
- Dedicated `components/faculty/FacultyApp.tsx` and older faculty implementation in `components.tsx`.
- Duplicate or competing announcement/banner concepts.
- Many admin panels with stale prop contracts.

Action:

- Mark canonical implementations.
- Delete or archive unused variants only after tests prove reachability through the chosen shell.
- Add a lint/test that prevents untyped prop drift from returning.

### 24. Repository/documentation hygiene is poor

The root contains many audit, phase, review, and committee markdown files. This creates confusion about which plan is current.

Examples:

- `AUDIT_REPORT_01.md`
- `COMMITTEE_REVIEW_REPORT.md`
- `COUNCIL_REVIEW*.md`
- `PHASE_*_ANALYSIS.md`
- `gemini_audit_*.md`
- `URGENT_FIXES.md`
- `TEAM_FIXES.md`

There are also stray files: `test_blank`, `test_multiline`, `test_prefix`.

Action:

- Do not delete blindly. Archive historical reports into `docs/audits/archive/`.
- Keep only current reports at root: `AGENTS.md`, `HANDOFF.md`, current roadmap, and current audit.
- Remove stray test files if they are not intentionally used.

### 25. Working tree is not release-clean

The current tree has many modified and untracked files, including local SQLite WAL/SHM files.

Action:

- Clean or explicitly stage intended changes.
- Ensure DB runtime artifacts are ignored and not shipped.
- Require clean `git status` before release tagging.

## Deployment Readiness Assessment

Ready:

- Backend tests pass.
- Frontend unit tests pass.
- Frontend Vite build passes.
- CI gate script exists and passes.
- Backend has `/health` and `/ready`.
- CORS can be configured by `CORS_ORIGIN`.
- WAL and `busy_timeout` are configured.

Not ready:

- Frontend TypeScript fails.
- CI gate does not run TypeScript type-check.
- Active UI has broken prop contracts.
- Critical role routing is incomplete.
- Backend role authorization has high-risk gaps.
- Deployment script still contains live production warnings and hardcoded production alias behavior.
- Working tree is dirty and includes runtime artifacts.

Deployment script note: `deploy-frontend.sh` now requires `DEPLOY_ENV`, which is better, but staging still runs `vercel deploy --prod --yes`. Production aliasing remains hardcoded to `saathi-pink.vercel.app`. This script should not be used for routine dev/staging deployment without cleanup.

## Recommended Roadmap

### Phase 0: Stop-the-line release gate

1. Add frontend type-check to package scripts and CI.
2. Fix all `tsc --noEmit` errors.
3. Require clean CI gate: backend tests, contract check, frontend tests, frontend type-check, frontend build.
4. Clean working tree and remove runtime artifacts from release candidates.

### Phase 1: Stabilize active shell and roles

1. Choose canonical shell: legacy `Sidebar/adminView` or new section shell.
2. Wire faculty and head users to correct role workspaces.
3. Fix mobile nav state and ticket-detail transitions.
4. Replace duplicate navigation registries with one source of truth.
5. Add role-reachability tests for all seeded roles.

### Phase 2: Close authorization gaps

1. Add explicit backend permission helpers for ticket read/write/admin operations.
2. Make viewer read-only server-side.
3. Remove client-controlled author/requester for authenticated records.
4. Fix scoped global/region operation gaps.
5. Add token/scope revocation strategy.

### Phase 3: Hide or complete unfinished features

1. Inventory every visible CTA and classify it as production, preview, hidden, or delete.
2. Remove unavailable attachment/sync/timeline controls from production UI unless implemented.
3. Complete substitution metrics before showing them as operational truth.
4. Wire notifications, announcements, and digests only after the product flow is complete.

### Phase 4: Improve operational UX

1. Redesign faculty attendance status selection.
2. Replace raw swap slot IDs with timetable selectors.
3. Build task-first mobile homes by role.
4. Standardize modal/sheet accessibility.

### Phase 5: Scalability hardening

1. Move heavy analytics/reporting/import/export work out of async request workers.
2. Add query-plan/index review for dashboards.
3. Add server pagination/windowing where arrays can grow.
4. Add performance budgets for login and first screen.

## Final Recommendation

The codebase has grown into a feature-rich operations platform, but the current integration state is unstable. Treat this as a consolidation phase, not a feature expansion phase.

The next engineering sprint should focus on type safety, shell consolidation, role routing, and backend authorization. New features should pause until the active product surface is coherent, type-checked, and protected by role-aware tests.
