# SAATHI AI Development Plan

Date: 2026-05-03
Source audit: `CHIEF_AUDIT_REPORT_2026-05-03.md`
Purpose: phase-wise implementation plan for an AI development agent. After each step or phase, the chief auditor should review the diff, run verification, and approve or redirect before the next phase begins.

## Operating Rules for the Development Agent

1. Do not start feature expansion until release blockers are closed.
2. Keep changes small and reviewable. Prefer one phase, or one step inside a phase, per development pass.
3. Do not delete user data or database content.
4. Do not deploy.
5. Do not run production deployment scripts.
6. Do not remove historical audit markdown unless explicitly instructed later.
7. Preserve existing project conventions in `AGENTS.md`.
8. Every step must end with verification output and a short implementation note.
9. If a step exposes unrelated breakage, stop and report it instead of silently widening scope.
10. The development agent should not proceed to the next phase until the auditor approves the current phase.

## Global Definition of Done

The project is not considered release-ready until all of the following pass:

- `cd backend && cargo test`
- `cd frontend && npm run test`
- `cd frontend && npm run typecheck`
- `cd frontend && npm run build`
- `bash scripts/ci-gate.sh`
- Role smoke checks for `admin`, `agent`, `viewer`, `aom`, `faculty`, and `head`
- No P0/P1 audit findings remain open

## Phase 0: Safety Baseline and CI Gate

Goal: make the build pipeline capable of catching the current integration failures.

This phase does not aim to fix all functional bugs. It first makes the failures visible and repeatable.

### Step 0.1: Add frontend type-check command

Implementation:

- Add `"typecheck": "tsc --noEmit"` to `frontend/package.json`.
- Do not change TypeScript config unless required to run the existing compiler.
- Update `scripts/ci-gate.sh` to run `npm run typecheck` before `npm run build`.

Acceptance:

- `npm run typecheck` runs and fails with current known errors.
- `scripts/ci-gate.sh` fails because type-check fails.
- No runtime behavior changed.

Auditor checkpoint:

- Confirm CI now exposes TypeScript failures.
- Confirm the agent did not weaken compiler strictness to hide errors.

### Step 0.2: Capture current TypeScript failure inventory

Implementation:

- Run `cd frontend && npm run typecheck`.
- Save a concise failure inventory in a new file: `TYPECHECK_REMEDIATION_PLAN_2026-05-03.md`.
- Group errors by cause, not by raw compiler output:
  - App/component prop contract drift.
  - stale model fields in `components.tsx`.
  - admin panel router prop mismatch.
  - missing shared types.
  - student timeline shape mismatch.

Acceptance:

- Inventory exists.
- It identifies the smallest safe fix order.
- No code behavior changed except scripts from Step 0.1.

Auditor checkpoint:

- Confirm inventory matches actual compiler errors.
- Approve the order of fixes before implementation begins.

## Phase 1: Type Safety Restoration

Goal: make `npm run typecheck` pass without papering over real integration defects.

Important constraint: do not use `any`, broad casts, or `// @ts-ignore` as the primary fix. If a temporary cast is unavoidable, document why and file a follow-up item.

### Step 1.1: Fix App-to-component prop contracts

Implementation:

- Fix `BottomNav` usage in `App.tsx`.
- Fix `ProgramFilters` usage in `App.tsx`.
- Fix `StudentTimelinePanel` prop shape mismatch.
- Fix `UserManagementPanel` prop mismatch or route to the correct component.

Decision required:

- Choose whether the active mobile model is the old `showMobileDetail` model or the new `mobileView` model.
- For this step, prefer the smallest safe restoration unless Phase 2 shell migration is started immediately.

Acceptance:

- Type errors in `App.tsx` are resolved.
- Mobile ticket list-to-detail transition works in code path.
- Bottom nav handlers are defined and cannot call `undefined`.

Auditor checkpoint:

- Review `App.tsx` diff carefully.
- Manually inspect mobile navigation logic.
- Confirm no accidental partial shell migration occurred.

### Step 1.2: Fix shared frontend model drift

Implementation:

- Resolve stale field references in `components.tsx`.
- Either update types to match actual backend models or update UI references to current type fields.
- Do not invent fields only to satisfy the UI.
- For removed fields such as `director_name`, `period_number`, `faculty_name`, and `subject_name`, confirm whether replacement fields exist.

Acceptance:

- Type errors caused by stale field names are resolved.
- UI still displays meaningful labels where data exists.
- Missing data is handled with clear fallback text.

Auditor checkpoint:

- Check whether fixes reflect real backend response shapes.
- Ensure no fake placeholder fields were added to types.

### Step 1.3: Fix `AdminPanelRouter` contract drift

Implementation:

- Align `AdminPanelRouter.tsx` with the current props expected by each panel.
- If a panel is stale and not production-reachable, either:
  - wire it correctly, or
  - temporarily remove it from active routing with an explicit TODO and hidden navigation entry.
- Do not leave broken routes reachable.

Acceptance:

- Admin route panel type errors are resolved.
- Every active `adminView` target either renders with correct props or is removed from active navigation.

Auditor checkpoint:

- Compare `toolRegistry.tsx` targets against `AdminPanelRouter.tsx`.
- Confirm no navigation item points to an unrenderable panel.

### Step 1.4: Fix missing shared types

Implementation:

- Resolve missing `CreateMakeupSessionInput`.
- Resolve missing `SubjectGap`.
- Confirm whether these types should be real backend/API types or should be removed from frontend code.

Acceptance:

- No missing-type errors remain.
- Types mirror backend models where appropriate.

Auditor checkpoint:

- Verify backend/frontend model alignment.

### Step 1.5: Make type-check pass

Implementation:

- Run all frontend checks.
- Fix remaining TypeScript errors using narrow, correct changes.

Acceptance:

- `cd frontend && npm run typecheck` passes.
- `cd frontend && npm run test` passes.
- `cd frontend && npm run build` passes.

Auditor checkpoint:

- Run or review all three frontend checks.
- Reject if the agent weakened type safety or hid errors.

## Phase 2: Canonical Shell and Role Routing

Goal: choose one application shell and make all roles land in the correct workspace.

This is the highest UX/product leverage phase. Do not mix old and new shell models indefinitely.

### Step 2.1: Choose canonical shell model

Implementation:

- Decide between:
  - Option A: restore and stabilize current legacy `Sidebar/adminView` shell.
  - Option B: complete the newer section-based shell using `navigation.ts`, `LeftRail`, and landing pages.
- Recommended: Option B, but only if implemented in controlled steps.
- Write the decision in `SHELL_DECISION_2026-05-03.md`.

Acceptance:

- One shell model is declared canonical.
- Non-canonical shell files are listed for later deletion/quarantine.
- No large UI rewrite happens in this decision step.

Auditor checkpoint:

- Approve shell direction before code changes.

### Step 2.2: Implement role landing matrix

Implementation:

- Define explicit landing behavior:
  - `admin`: Work dashboard or section home.
  - `agent`: ticket queue/work workspace.
  - `viewer`: read-only ticket/report overview.
  - `aom`: approvals, substitutions, school health, alerts.
  - `faculty`: `FacultyApp`.
  - `head`: `ApproverDashboard`.
- Route each role deterministically after login/session restore.

Acceptance:

- Each seeded role reaches a coherent first screen.
- Faculty users do not land in an admin/ticket shell unless explicitly intended.
- Head users can reach approvals without hunting through admin tools.

Auditor checkpoint:

- Manual role walkthrough or test evidence for all six roles.

### Step 2.3: Unify navigation registry

Implementation:

- Choose one source of truth for tools and role visibility.
- Prefer `navigation.ts` if completing the section shell.
- Ensure every tool has explicit role visibility.
- Remove default "visible to everyone" behavior for sensitive tools.

Acceptance:

- No tool without role metadata is exposed by default.
- `viewer`, `faculty`, and `head` do not see admin-only tools.
- Navigation registry maps cleanly to renderable components.

Auditor checkpoint:

- Review role-to-tool matrix.
- Confirm no admin surfaces are accidentally visible to scoped roles.

### Step 2.4: Stabilize mobile navigation

Implementation:

- Use one mobile state model.
- Required bottom nav:
  - Home
  - Work
  - Create, hidden or disabled for read-only roles
  - More
- Detail drill-in must have predictable back behavior.

Acceptance:

- Mobile Home and Work buttons call valid handlers.
- Selecting a ticket opens detail.
- Back from detail returns to Work.
- Viewer cannot create tickets.

Auditor checkpoint:

- Review mobile state transitions.
- Confirm the original reported "screen breaks after a minute" class of state mismatch is not reintroduced.

### Step 2.5: Remove or quarantine abandoned shell code

Implementation:

- After canonical shell is working, remove or quarantine unused shell artifacts.
- Do not delete functional panels that are merely not yet wired unless approved.

Acceptance:

- No duplicate active shell systems remain.
- Dead navigation code is either removed or clearly marked inactive.

Auditor checkpoint:

- Run `rg` for old shell symbols.
- Confirm only intended compatibility paths remain.

## Phase 3: Backend Authorization and Data Integrity

Goal: close high-risk security and workflow integrity gaps.

This phase should be implemented before adding new features.

### Step 3.1: Ticket permission model

Implementation:

- Add explicit backend helpers:
  - `require_ticket_reader`
  - `require_ticket_writer`
  - `require_ticket_admin` if needed
- Apply them to ticket create, update, delete, comments, and history operations.
- Preserve school scope checks.

Suggested policy:

- `admin`: full access.
- `agent`: ticket read/write.
- `viewer`: ticket read-only.
- `aom`: scoped read/write only if product requires it.
- `faculty`: no general ticket mutation unless specifically allowed.
- `head`: no general ticket mutation unless specifically allowed.

Acceptance:

- Viewer cannot mutate tickets by API.
- Unauthorized scoped roles cannot mutate tickets directly.
- Existing authorized ticket workflows still pass.

Auditor checkpoint:

- Review route-level permissions.
- Request role-specific API smoke evidence.

### Step 3.2: Server-derived authorship

Implementation:

- Stop trusting client-provided visible author for authenticated comments.
- Stop trusting client-provided requester as internal creator identity.
- Add or clarify separate fields if external requester/reported-by is required.

Acceptance:

- Authenticated comment author is derived from JWT.
- Audit/history actor and visible actor are consistent or intentionally separate.
- Tests cover spoof attempts.

Auditor checkpoint:

- Inspect payload handling in routes and repositories.

### Step 3.3: Scoped global operation hardening

Implementation:

- Fix region-based bulk timetable publish by resolving target schools before mutation and checking every school.
- Restrict global announcements to admin unless region/school scope is explicitly implemented.
- Restrict global/region holidays to admin unless region authorization is implemented.
- Fix alert inbox arbitrary `user_id` access by using `claims.sub` for normal users.

Acceptance:

- AOM cannot affect schools outside scope through region expansion.
- AOM cannot create global operational state unless explicitly allowed.
- Users cannot load another user's alert triage state.

Auditor checkpoint:

- Review route-level checks and edge-case tests.

### Step 3.4: Transaction safety

Implementation:

- Replace manual `BEGIN`/`COMMIT` paths with RAII transaction handling where possible.
- At minimum, guarantee rollback on every error path.
- Cover:
  - leave approval
  - swap acceptance
  - substitute assignment

Acceptance:

- Failure-path tests prove no open transaction is left.
- No partial writes occur on validation failure.

Auditor checkpoint:

- Review error paths, not only happy paths.

### Step 3.5: Token revocation strategy

Implementation:

- Pick a pragmatic revocation approach:
  - session/token version checked in middleware, or
  - DB revalidation for sensitive routes, or
  - shorter access tokens plus refresh invalidation.
- Implement minimum viable protection for disabled users and changed school scopes.

Acceptance:

- Disabled user cannot continue using old token beyond defined tolerance.
- Changed school assignments affect access without waiting eight hours.

Auditor checkpoint:

- Review security tradeoff and test coverage.

## Phase 4: Feature Reachability and Product Completeness

Goal: make the existing feature surface honest, reachable, and role-appropriate.

### Step 4.1: Build feature reachability matrix

Implementation:

- Create `FEATURE_REACHABILITY_MATRIX_2026-05-03.md`.
- For each feature record:
  - backend route
  - frontend API command
  - component
  - navigation entry
  - allowed roles
  - status: production, preview, hidden, delete candidate

Acceptance:

- Matrix includes ticketing, master data, faculty, substitutions, leave/swap, reports, notifications, digests, imports/exports, backup/sync, and student timeline.

Auditor checkpoint:

- Confirm no major feature is missing from the matrix.

### Step 4.2: Hide or disable known dead-end CTAs

Implementation:

- Address:
  - ticket attachments
  - daily sync import/export
  - student timeline
  - digest email sending
  - alert-to-ticket placeholder flow
  - hardcoded substitution metrics
- For each, choose: implement now, hide, or explicit disabled preview state.

Acceptance:

- Production UI does not expose buttons that only say "not available".
- Placeholder metrics are not shown as operational truth.

Auditor checkpoint:

- Manually inspect affected screens.

### Step 4.3: Wire high-value existing workflows

Implementation priority:

1. Faculty app for `faculty`.
2. Approver dashboard for `head` and relevant `aom`.
3. Leave/swap panel.
4. Substitution command center.
5. Notification center.
6. Reports dashboards.

Acceptance:

- Each wired workflow is reachable from the canonical shell.
- Each workflow has role-appropriate access.
- Each workflow has at least a smoke test or documented manual check.

Auditor checkpoint:

- Review one workflow at a time.
- Do not approve broad wiring without role tests.

### Step 4.4: Improve faculty and swap UX

Implementation:

- Replace six-state attendance tap cycle with explicit controls for non-default states.
- Replace raw swap slot ID inputs with timetable selectors.
- Add conflict preview before swap submission.

Acceptance:

- Faculty can perform attendance without knowing hidden tap sequence.
- Faculty can request swaps without database IDs.

Auditor checkpoint:

- Review user flow, not just code.

## Phase 5: UI System, Accessibility, and Style Consolidation

Goal: make the redesigned UI coherent and accessible.

### Step 5.1: Define active UI vocabulary

Implementation:

- Document canonical classes/components for:
  - shell
  - cards
  - tables
  - filters
  - modals
  - sheets
  - buttons
  - empty/loading/error states

Acceptance:

- `UI_SYSTEM_NOTES_2026-05-03.md` exists.
- It identifies old classes/styles to remove later.

Auditor checkpoint:

- Confirm the style direction matches current product goals.

### Step 5.2: Standardize modal/sheet primitive

Implementation:

- Create one shared dialog/sheet primitive.
- Required behavior:
  - `role="dialog"`
  - `aria-modal="true"`
  - labelled title
  - Escape-to-close
  - focus trap
  - focus restore
  - contained scroll

Migrate first:

- Create Ticket
- Notification Center
- Alert Inbox
- high-traffic admin modal

Acceptance:

- Migrated modals have consistent behavior.
- No background scroll/focus leak.

Auditor checkpoint:

- Inspect DOM semantics and keyboard behavior.

### Step 5.3: Mobile task-first IA

Implementation:

- Replace raw mobile admin tool list with role task groups.
- Define mobile home cards by role.

Acceptance:

- Mobile users see daily tasks before admin tool names.
- More menu is secondary, not the primary workflow.

Auditor checkpoint:

- Review screenshots or manual walkthrough for each role.

## Phase 6: Scalability and Operational Hardening

Goal: make the app safer under growth and operational load.

### Step 6.1: Reduce eager frontend loading

Implementation:

- Load only current user before auth settles.
- Role-gate post-login loaders.
- Screen-gate heavy resources:
  - audit log
  - all comments
  - reports
  - analytics
  - timetable health
  - faculty assignments

Acceptance:

- Initial login network call count is materially lower.
- Unauthorized roles do not request irrelevant datasets.

Auditor checkpoint:

- Review loader sequence.
- If possible, compare network logs before/after.

### Step 6.2: Server pagination and UI pagination/windowing

Implementation:

- Ensure ticket list uses backend pagination properly.
- Avoid globally loading all comments unless needed.
- Add pagination/windowing for large report/ticket/comment views.

Acceptance:

- Ticket and comment views do not assume bounded arrays.
- Search/filter behavior is defined for paginated data.

Auditor checkpoint:

- Review data flow carefully; pagination can create UX regressions.

### Step 6.3: Heavy backend work isolation

Implementation:

- Identify slow analytics/report/import/export endpoints.
- Move heavy synchronous work to `spawn_blocking`, queue jobs, or snapshots.
- Add time-window caps for analytics.

Acceptance:

- Heavy reports do not block Tokio worker threads unnecessarily.
- Analytics endpoints have bounded input windows.

Auditor checkpoint:

- Review implementation approach endpoint by endpoint.

### Step 6.4: Query/index review

Implementation:

- Review query plans for dashboards and analytics:
  - health trends
  - substitution trends
  - faculty utilization
  - control tower
  - compliance scorecard
  - ticket list filters
- Add indexes where justified.

Acceptance:

- Query plan notes are documented.
- Index migrations are data-preserving and idempotent.

Auditor checkpoint:

- Verify migrations follow project rules.

## Phase 7: Deployment Readiness

Goal: prepare a clean, auditable release candidate.

### Step 7.1: Deployment script safety

Implementation:

- Fix `deploy-frontend.sh` staging behavior.
- Staging must not use production alias.
- Production deploy must remain manually confirmed.
- Remove hardcoded production alias unless this repo is the production deployment source.

Acceptance:

- Staging and production paths are clearly separated.
- Script cannot accidentally alias staging to production.

Auditor checkpoint:

- Review script without running deployment.

### Step 7.2: Repository hygiene

Implementation:

- Move historical audit markdown into an archive folder if approved.
- Remove stray test files if confirmed unused:
  - `test_blank`
  - `test_multiline`
  - `test_prefix`
- Ensure SQLite runtime artifacts are ignored.

Acceptance:

- Root directory contains only active operational docs.
- `git status` is understandable.

Auditor checkpoint:

- Confirm no useful audit history was deleted without approval.

### Step 7.3: Final release candidate gate

Implementation:

- Run complete verification:
  - backend tests
  - frontend tests
  - typecheck
  - build
  - CI gate
  - role smoke checks
  - health/readiness check

Acceptance:

- All checks pass.
- No P0/P1 findings remain open.
- Known P2/P3 issues are documented as accepted risks or scheduled work.

Auditor checkpoint:

- Chief auditor performs final deployment readiness review.

## Review Protocol After Each Development Submission

When the development agent reports completion, request the following:

1. Files changed.
2. Exact commands run and results.
3. Screens or flows manually checked.
4. Any deviations from the plan.
5. Any new risks discovered.

The chief auditor should then:

1. Inspect the diff.
2. Run targeted tests.
3. Check that scope did not expand silently.
4. Mark the step as accepted, rejected, or accepted with follow-up.
5. Provide the next step instruction.

## First Instruction to Give the Development Agent

Start with Phase 0 only.

Implement Step 0.1 and Step 0.2. Do not fix the TypeScript errors yet. The goal is to make type-checking a formal gate and document the failure inventory. After that, stop and report back with changed files and command output.

