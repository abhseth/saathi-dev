# Architecture Scalability Recommendations 01

Date: 2026-05-02
Repository: `/home/abhi/saathi-dev`

## Basis

This recommendation set is based on:

- 2 independent architecture/scalability audits from internal review agents
- direct code inspection of the current codebase
- reconciliation with `gemini_audit_01_architecture_scalability.md`

## Executive Position

The codebase is still viable at its current scale, and SQLite is still acceptable for the current product stage. The main risk is not immediate collapse. The main risk is that continued feature growth without structural correction will steadily reduce:

- feature velocity
- regression safety
- runtime headroom
- onboarding clarity

The strongest current architecture/scalability problems are:

1. frontend shell centralization in `App.tsx`
2. eager global data loading
3. synchronous SQLite work inside async handlers
4. multiplicative analytics query patterns
5. weak backend domain boundaries
6. duplicated route/API/navigation/render registries

## Areas of Agreement

### 1. `App.tsx` is the primary frontend bottleneck

The shell still owns too much:

- auth/session
- ticket state
- master data
- analytics
- substitutions
- mobile shell state
- keyboard shortcuts
- polling
- modal orchestration
- large render dispatch logic

Refs:

- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:229)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:366)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2185)

### 2. Loading strategy is too eager

The shell fetches too much data before the user has navigated anywhere and before role/screen intent is fully known.

Refs:

- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:366)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:375)
- [frontend/src/hooks/useNotifications.ts](/home/abhi/saathi-dev/frontend/src/hooks/useNotifications.ts:57)

### 3. Backend layering is too weak

`repositories.rs` is a real god-module and too much workflow pressure accumulates in handlers and repository functions.

Refs:

- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:101)
- [backend/src/routes/mod.rs](/home/abhi/saathi-dev/backend/src/routes/mod.rs:21)

### 4. The clearest runtime scaling limit is blocking DB work inside async routes

The backend runs synchronous SQLite work inline inside async handlers with a small connection pool.

Refs:

- [backend/src/main.rs](/home/abhi/saathi-dev/backend/src/main.rs:44)
- [backend/src/main.rs](/home/abhi/saathi-dev/backend/src/main.rs:47)
- [backend/src/routes/analytics.rs](/home/abhi/saathi-dev/backend/src/routes/analytics.rs:70)
- [backend/src/routes/tickets.rs](/home/abhi/saathi-dev/backend/src/routes/tickets.rs:18)

### 5. Analytics query shape is a real scaling risk

Several analytics endpoints use loop-per-faculty or loop-per-week patterns that will degrade nonlinearly as data grows.

Refs:

- [backend/src/analytics.rs](/home/abhi/saathi-dev/backend/src/analytics.rs:185)
- [backend/src/analytics.rs](/home/abhi/saathi-dev/backend/src/analytics.rs:211)
- [backend/src/analytics.rs](/home/abhi/saathi-dev/backend/src/analytics.rs:231)
- [backend/src/analytics.rs](/home/abhi/saathi-dev/backend/src/analytics.rs:519)
- [backend/src/analytics.rs](/home/abhi/saathi-dev/backend/src/analytics.rs:566)
- [backend/src/analytics.rs](/home/abhi/saathi-dev/backend/src/analytics.rs:605)

### 6. Contract duplication is too high

The route table, frontend API dispatch, navigation registry, and render switch all need synchronized edits.

Refs:

- [backend/src/routes/mod.rs](/home/abhi/saathi-dev/backend/src/routes/mod.rs:21)
- [frontend/src/api.ts](/home/abhi/saathi-dev/frontend/src/api.ts:70)
- [frontend/src/navigation.ts](/home/abhi/saathi-dev/frontend/src/navigation.ts:1)
- [frontend/src/App.tsx](/home/abhi/saathi-dev/frontend/src/App.tsx:2185)

## Important Adjustments to External Recommendations

### 1. The backend is not a single mutexed connection design

Some external wording overstates the backend concurrency mechanism. The current backend uses an `r2d2` SQLite pool, not a single global mutexed connection.

Refs:

- [backend/src/main.rs](/home/abhi/saathi-dev/backend/src/main.rs:44)
- [backend/src/main.rs](/home/abhi/saathi-dev/backend/src/main.rs:47)

The real problem is:

- synchronous DB work inside async handlers
- small pool size
- heavier analytics/import/report endpoints with blocking behavior

### 2. The codebase is strained, not collapsing

The right conclusion is not “replace everything now.” The right conclusion is that the structure is now imposing a measurable tax on safe change and performance headroom.

### 3. A broad cosmetic refactor is not the first move

Reducing file size by itself is not the highest-value step. The first wins come from:

- loading strategy
- domain ownership
- blocking-query isolation
- registry drift reduction

## Final Recommendations

### Immediate

#### 1. Fix loading strategy first

- stop eager global loading from `App` mount
- gate protected fetches behind authenticated user state
- load by active role, section, and tool
- move notifications polling behind authenticated, role-aware gating

Why first:

- it reduces wasted work immediately
- it improves startup latency without a full rewrite
- it lowers failure noise before deeper refactors

#### 2. Restore and hold a green quality baseline

- keep `cargo test`, `npm run build`, and `npm run test` green
- add a small route-smoke suite covering:
  - auth
  - tickets
  - one scoped role
  - one analytics endpoint
  - one substitution/leave path

Why:

- architecture changes without a working gate will create blind regressions

#### 3. Add operational basics

- health/readiness endpoint
- request tracing middleware
- request correlation IDs where practical

Why:

- scalability work without observability slows diagnosis and prioritization

#### 4. Isolate the worst blocking DB paths

Start with:

- analytics
- reports
- imports
- any global recomputation paths

Why:

- this is the clearest runtime bottleneck on the backend

### Near-term

#### 5. Split frontend state ownership by domain

Keep the shell in `App.tsx`, but move server-state ownership into feature-local controllers/hooks for:

- tickets
- master data
- timetable/faculty
- analytics

Why:

- this reduces coupling without forcing a full UI rewrite

#### 6. Split `repositories.rs` by domain and add a service layer

First split storage modules into:

- tickets
- schools
- faculty
- substitutions
- analytics-support
- admin

Then move cross-entity workflow logic into service modules.

Why:

- this is the most important backend maintainability move

#### 7. Rewrite the worst analytics endpoints as set-based queries

Priority targets:

- `faculty_utilization_trend`
- `health_trends`
- `substitution_trends`

Why:

- these are the clearest nonlinear scaling risks in the current code

#### 8. Reduce registry drift

Converge:

- `navigation.ts`
- `api.ts`
- `sectionTool` render map
- route definitions

At minimum:

- add parity checks so a new tool/route cannot be wired in one place and forgotten in another

### Later

#### 9. Add pagination and windowing

Targets:

- tickets
- comments
- notifications
- larger analytics tables

Why later:

- the benefit is real, but lower than fixing eager loading and ownership boundaries first

#### 10. Remove global recomputation from common writes

`refresh_escalations` should not remain a full-table side effect on ordinary ticket writes.

Refs:

- [backend/src/routes/tickets.rs](/home/abhi/saathi-dev/backend/src/routes/tickets.rs:34)
- [backend/src/routes/tickets.rs](/home/abhi/saathi-dev/backend/src/routes/tickets.rs:56)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:134)

#### 11. Reassess datastore boundaries after measurement

Do not replace SQLite preemptively. First create clearer boundaries so future extraction is possible if usage volume requires it.

## Recommended Order

1. loading strategy
2. green gate + route-smoke coverage
3. blocking DB isolation
4. frontend domain-state isolation
5. backend repository split + service layer
6. analytics query rewrites
7. registry unification
8. pagination/windowing
9. write-path recomputation cleanup

## What Not To Do First

- do not begin with a wholesale UI-state-library migration
- do not split files just to make them smaller
- do not replace SQLite before fixing the current hot paths and boundaries
- do not attempt a one-shot architecture rewrite

## Bottom Line

The architecture is not beyond recovery. It is at the point where continued feature growth without structural correction will steadily reduce reliability and development speed.

The first actions should be:

1. reduce eager loading
2. reduce central ownership in `App.tsx`
3. split backend domain boundaries
4. fix blocking and multiplicative query hot spots
