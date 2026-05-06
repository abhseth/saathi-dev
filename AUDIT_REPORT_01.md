# Audit Report 01

Date: 2026-05-02
Repository: `/home/abhi/saathi-dev`

## Scope

This audit consolidates parallel review tracks across:

- UI/UX and accessibility
- Product workflow and user friendliness
- Backend workflow integrity
- Auth, scoping, permissions, and data integrity
- Release readiness, QA, and operational safety
- Architecture and maintainability

The report combines direct code inspection, current build/test verification, and external expert sub-audit input.

## Current Verification Status

- `cargo test`: passed
- `npm run build`: passed
- `npm run test`: failing

Frontend test failures are current and real:

- [frontend/src/components.test.tsx](/home/abhi/saathi-dev/frontend/src/components.test.tsx:17)
- [frontend/src/components.test.tsx](/home/abhi/saathi-dev/frontend/src/components.test.tsx:64)

These failures reflect UI-contract drift around:

- `SlaBreachAlert` no longer exposing `role="alert"`
- active `ErrorBoundary` behavior diverging from the tested contract

## Executive Summary

The program has improved materially, especially in shell/navigation structure and feature breadth, but it is not release-clean today. The highest-risk issues are:

1. authorization and scoping gaps in ticketing and automation
2. workflow correctness issues in leave/substitution
3. mobile navigation/state truthfulness problems
4. shell dead routes and misleading CTAs
5. accessibility inconsistencies in high-traffic overlays
6. release-gate failure and documentation drift

## Critical Findings

### 1. Ticket mutation permissions are too open

Ticket and comment write paths are exposed to any authenticated role via the protected router, with school-scope checks but no strong role restrictions.

Refs:

- [backend/src/routes/mod.rs](/home/abhi/saathi-dev/backend/src/routes/mod.rs:27)
- [backend/src/routes/tickets.rs](/home/abhi/saathi-dev/backend/src/routes/tickets.rs:22)
- [backend/src/routes/tickets.rs](/home/abhi/saathi-dev/backend/src/routes/tickets.rs:38)
- [backend/src/routes/tickets.rs](/home/abhi/saathi-dev/backend/src/routes/tickets.rs:94)

Impact:

- documented read-only `viewer` model is broken
- non-ticket roles can mutate ticket records

### 2. Several `aom` automation write endpoints still lack scope enforcement

High-risk automation mutations still accept `admin_or_aom` without explicit per-school scope enforcement.

Refs:

- [backend/src/routes/automation.rs](/home/abhi/saathi-dev/backend/src/routes/automation.rs:220)
- [backend/src/routes/automation.rs](/home/abhi/saathi-dev/backend/src/routes/automation.rs:270)
- [backend/src/routes/automation.rs](/home/abhi/saathi-dev/backend/src/routes/automation.rs:282)
- [backend/src/routes/automation.rs](/home/abhi/saathi-dev/backend/src/routes/automation.rs:294)
- [backend/src/routes/automation.rs](/home/abhi/saathi-dev/backend/src/routes/automation.rs:349)

Impact:

- scoped AOM users can mutate data outside assigned schools

### 3. Leave approval appears to affect sessions outside the leave request’s school

Approved leave updates `lecture_sessions` by faculty and date range, but the session update is not constrained to the leave request school.

Refs:

- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:7427)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:7448)

Impact:

- multi-school faculty leave can spill into the wrong school’s sessions

### 4. Mobile navigation still loses context and misstates location

The mobile shell is improved, but state transitions still collapse users into generic `home/work` returns instead of preserving origin context.

Refs:

- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:1532)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:1541)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:1563)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:1575)

Impact:

- mobile users cannot reliably infer where they are or where “back” will go

### 5. Admin `users` is a dead route in the new shell

The admin landing and registry expose `users`, but the `sectionTool` render chain does not actually render user management from that path.

Refs:

- [frontend/src/components/landing/AdminLanding.tsx](/home/abhi/saathi-dev/frontend/src/components/landing/AdminLanding.tsx:16)
- [frontend/src/navigation.ts](/home/abhi/saathi-dev/frontend/src/navigation.ts:354)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2184)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2515)

Impact:

- users hit a loading-style dead end instead of the intended tool

### 6. Release gate is not green

The current release gate requires backend tests, frontend tests, and frontend build. Frontend tests are currently failing.

Refs:

- [scripts/ci-gate.sh](/home/abhi/saathi-dev/scripts/ci-gate.sh:10)
- [frontend/src/components.test.tsx](/home/abhi/saathi-dev/frontend/src/components.test.tsx:17)
- [frontend/src/components.test.tsx](/home/abhi/saathi-dev/frontend/src/components.test.tsx:64)
- [frontend/src/components.tsx](/home/abhi/saathi-dev/frontend/src/components.tsx:158)

Impact:

- current UI behavior and expected UI contract are out of sync

## High-Impact Findings

### 7. JWT role/scope changes do not take effect until token expiry

JWTs are trusted as self-contained claims for the full token life and are not revalidated against DB state after login.

Refs:

- [backend/src/auth.rs](/home/abhi/saathi-dev/backend/src/auth.rs:13)
- [backend/src/auth.rs](/home/abhi/saathi-dev/backend/src/auth.rs:42)

### 8. Leave notifications leak too broadly

Leave request notifications are sent using a helper that appears to return all school members, not only approvers.

Refs:

- [backend/src/routes/substitutions.rs](/home/abhi/saathi-dev/backend/src/routes/substitutions.rs:53)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:7539)

### 9. Substitution assignment commits invalid operational states

The system computes substitute-fit signals, but assignment and acceptance do not enforce eligibility, conflict, or workload checks when writing final state.

Refs:

- [backend/src/routes/faculty.rs](/home/abhi/saathi-dev/backend/src/routes/faculty.rs:334)
- [backend/src/routes/faculty.rs](/home/abhi/saathi-dev/backend/src/routes/faculty.rs:667)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:4550)
- [backend/src/substitution_engine.rs](/home/abhi/saathi-dev/backend/src/substitution_engine.rs:6)

### 10. Ticket action wording and effect are misaligned

The ticket workbench contains a trust-breaking primary CTA: `Assign to me` does not reliably mean “assign to the current user”.

Refs:

- [frontend/src/components/tickets/TicketDetail.tsx](/home/abhi/saathi-dev/frontend/src/components/tickets/TicketDetail.tsx:176)
- [frontend/src/components/tickets/TicketList.tsx](/home/abhi/saathi-dev/frontend/src/components/tickets/TicketList.tsx:110)

### 11. Summary cards and notifications still produce weak or dead-end journeys

Examples:

- viewer school-health rows open generic `schools` rather than a scoped destination
- notifications are largely passive and not reliably actionable

Refs:

- [frontend/src/components/mobile/SpocExecutiveSummary.tsx](/home/abhi/saathi-dev/frontend/src/components/mobile/SpocExecutiveSummary.tsx:69)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2653)
- [frontend/src/components/mobile/NotificationCenter.tsx](/home/abhi/saathi-dev/frontend/src/components/mobile/NotificationCenter.tsx:12)

### 12. Protected data loading is too eager

The shell still loads too much too early and mounts notifications polling broadly rather than in a tightly role-aware manner.

Refs:

- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:366)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:1264)
- [frontend/src/hooks/useNotifications.ts](/home/abhi/saathi-dev/frontend/src/hooks/useNotifications.ts:57)

## UX and Accessibility Findings

### 13. Modal accessibility is inconsistent

Create-ticket is still not implemented as a robust dialog, and the student timeline panel lacks a visible close affordance.

Refs:

- [frontend/src/components/tickets/CreateTicketModal.tsx](/home/abhi/saathi-dev/frontend/src/components/tickets/CreateTicketModal.tsx:80)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2806)
- [frontend/src/components.tsx](/home/abhi/saathi-dev/frontend/src/components.tsx:1559)

### 14. Keyboard accessibility is still weak across the new shell

Important controls are visually reset with `all: unset` but do not share strong focus-visible treatment.

Refs:

- [frontend/src/components/LeftRail.tsx](/home/abhi/saathi-dev/frontend/src/components/LeftRail.tsx:94)
- [frontend/src/components/landing/SectionLanding.tsx](/home/abhi/saathi-dev/frontend/src/components/landing/SectionLanding.tsx:43)
- [frontend/src/styles.css](/home/abhi/saathi-dev/frontend/src/styles.css:196)
- [frontend/src/styles.css](/home/abhi/saathi-dev/frontend/src/styles.css:252)

### 15. Several clickable elements still lack proper semantics

Notable examples include clickable list items and table rows in operational dashboards.

Refs:

- [frontend/src/components/approver/ApproverDashboard.tsx](/home/abhi/saathi-dev/frontend/src/components/approver/ApproverDashboard.tsx:86)
- [frontend/src/components/approver/ApproverDashboard.tsx](/home/abhi/saathi-dev/frontend/src/components/approver/ApproverDashboard.tsx:114)
- [frontend/src/components/dashboards/DeviationScoreboardPanel.tsx](/home/abhi/saathi-dev/frontend/src/components/dashboards/DeviationScoreboardPanel.tsx:47)

### 16. Several top-level interactions are misleading or incomplete

Examples:

- `/` search shortcut is broken because the targeted search input does not match the selector
- template selectors snap back to placeholder after selection
- web attachment/snapshot actions expose controls that only raise “unavailable” notices

Refs:

- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2131)
- [frontend/src/components.tsx](/home/abhi/saathi-dev/frontend/src/components.tsx:214)
- [frontend/src/components/tickets/CreateTicketModal.tsx](/home/abhi/saathi-dev/frontend/src/components/tickets/CreateTicketModal.tsx:92)
- [frontend/src/components/tickets/TicketDetail.tsx](/home/abhi/saathi-dev/frontend/src/components/tickets/TicketDetail.tsx:635)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:1587)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:1648)

## Product and Feature Opportunities

### 17. Landing pages need to become real work surfaces

They are structurally better now, but several still feel like launchers more than operational homes.

Priority opportunities:

- role-specific top tasks
- scoped recent/problem items
- direct actions from summaries
- decision-oriented reports entry points

Refs:

- [frontend/src/components/landing/SectionLanding.tsx](/home/abhi/saathi-dev/frontend/src/components/landing/SectionLanding.tsx:13)
- [frontend/src/components/landing/ReportsLanding.tsx](/home/abhi/saathi-dev/frontend/src/components/landing/ReportsLanding.tsx:9)
- [frontend/src/navigation.ts](/home/abhi/saathi-dev/frontend/src/navigation.ts:1)

### 18. Role homes should become action-complete

The product has the beginnings of role-based operations surfaces, but too many homes still behave like summary launchers rather than decisive workspaces.

Targets:

- `aom`
- `head`
- `viewer`
- `agent`

### 19. Notifications should become action surfaces

Every notification should answer:

- what happened
- what entity it affects
- what the user can do now

### 20. Reports should be organized by decisions, not tool names

The reporting surface is broad, but the UX still asks users to browse analytics by internal panel name rather than by question or exception.

## Architecture and Program Development Findings

### 21. Shell and repository centralization are now active delivery risks

The frontend shell still centralizes too much orchestration in `App.tsx`, while the backend keeps accumulating domain logic in `repositories.rs`.

Refs:

- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:229)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:101)

### 22. Registry and contract drift risk remains high

Routes, frontend API dispatch, navigation registry, and render switch are still maintained in separate places.

Refs:

- [backend/src/routes/mod.rs](/home/abhi/saathi-dev/backend/src/routes/mod.rs:18)
- [frontend/src/api.ts](/home/abhi/saathi-dev/frontend/src/api.ts:61)
- [frontend/src/navigation.ts](/home/abhi/saathi-dev/frontend/src/navigation.ts:1)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2184)

### 23. Documentation drift is severe enough to mislead operations and QA

`AGENTS.md` and `HANDOFF.md` no longer accurately describe the route surface, security defaults, and migration state.

Refs:

- [AGENTS.md](/home/abhi/saathi-dev/AGENTS.md:62)
- [AGENTS.md](/home/abhi/saathi-dev/AGENTS.md:213)
- [HANDOFF.md](/home/abhi/saathi-dev/HANDOFF.md:41)
- [backend/src/routes/mod.rs](/home/abhi/saathi-dev/backend/src/routes/mod.rs:18)
- [backend/src/main.rs](/home/abhi/saathi-dev/backend/src/main.rs:52)
- [backend/src/db.rs](/home/abhi/saathi-dev/backend/src/db.rs:1350)

## Recommended Improvement Headers

### Security and Authorization

- tighten route-role permissions
- enforce scope on every scoped mutation
- reduce token staleness risk
- remove client-controlled authorship where identity must be authoritative

### Workflow Correctness

- fix leave/substitution semantics
- make ticket and notification actions mean exactly what they say
- remove shell dead routes and dead CTAs

### Mobile and UI Truthfulness

- make mobile navigation state explicit
- make back behavior origin-preserving
- ensure summary cards and notifications deep-link to exact scoped destinations

### Accessibility and User Friendliness

- standardize actual dialog behavior
- restore keyboard reachability and focus visibility
- replace hidden/expert interaction patterns where data quality matters

### Release Readiness and QA

- require a green CI gate
- add route smoke coverage
- add one end-to-end happy path per major domain
- align tests with the real UI contract or restore the expected contract deliberately

### Architecture and Maintainability

- reduce global shell orchestration
- reduce backend god-module growth
- converge route/tool/API registries toward a stronger contract

## Recommended Remediation Sequence

1. Fix authorization and scope leaks.
2. Restore a green release gate.
3. Fix mobile navigation truth and shell dead routes.
4. Fix leave/substitution workflow correctness.
5. Fix misleading CTAs and high-traffic accessibility issues.
6. Then do architecture and loading-strategy cleanup.

## Notes on Earlier Findings That Are No Longer Active

Some issues reported in earlier review passes appear fixed in the current code and should not stay on the active defect list:

- admin `csv-export` dead-end
- global ticket filters showing on all sections
- unscoped global `.primary-action` override
- ticket-detail mobile class mismatch

## Conclusion

The program is directionally stronger than before, especially in feature coverage and shell structure, but it still has real authorization gaps, workflow correctness defects, mobile navigation truth issues, and release-health problems. The next audit pass should check whether:

- route-role enforcement is tightened
- scoped mutations are consistently enforced
- leave/substitution semantics are corrected
- mobile shell state is explicit and origin-preserving
- frontend tests are green again
