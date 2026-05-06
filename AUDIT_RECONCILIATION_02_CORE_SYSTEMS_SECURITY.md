# Audit Reconciliation 02: Core Systems Security

Date: 2026-05-02
Repository: `/home/abhi/saathi-dev`
Compared documents:

- [gemini_audit_02_core_systems_security.md](/home/abhi/saathi-dev/gemini_audit_02_core_systems_security.md)
- [AUDIT_REPORT_01.md](/home/abhi/saathi-dev/AUDIT_REPORT_01.md)

## Summary

Gemini’s core-systems/security audit is broadly compatible with the committee findings. The strongest overlap is on:

- `aom` scoping leaks in automation
- substitution workflow integrity problems
- weak transactional/concurrency hardening
- backend robustness issues in utility paths

The main adjustment is that Gemini’s overall security conclusion is too mild. The application-level authorization and data-integrity defects are more serious than “foundation secure, app logic underdeveloped.”

## Common Points

### 1. Automation scoping leaks are real

Gemini flags school-scope leaks in `automation.rs` for scoped `aom` users.

Refs:

- [gemini_audit_02_core_systems_security.md](/home/abhi/saathi-dev/gemini_audit_02_core_systems_security.md:16)
- [AUDIT_REPORT_01.md](/home/abhi/saathi-dev/AUDIT_REPORT_01.md:64)

### 2. Substitution workflows still have integrity problems

Gemini emphasizes race conditions and check-then-act behavior. The committee audit emphasized missing eligibility/conflict enforcement. These are complementary views of the same risk surface.

Refs:

- [gemini_audit_02_core_systems_security.md](/home/abhi/saathi-dev/gemini_audit_02_core_systems_security.md:37)
- [AUDIT_REPORT_01.md](/home/abhi/saathi-dev/AUDIT_REPORT_01.md:158)

### 3. Concurrency and transactional safety are underdeveloped

Gemini focuses on missing `busy_timeout` and missing transactions. The committee audit also identified backend blocking/concurrency weaknesses, though with more emphasis on synchronous SQLite hot paths and workflow correctness.

Refs:

- [gemini_audit_02_core_systems_security.md](/home/abhi/saathi-dev/gemini_audit_02_core_systems_security.md:29)
- [AUDIT_REPORT_01.md](/home/abhi/saathi-dev/AUDIT_REPORT_01.md:293)

### 4. Backend robustness has real gaps outside auth

Gemini’s CSV parsing and SQL-hygiene observations are valid and confirmed in current code.

Refs:

- [backend/src/bulk_ops.rs](/home/abhi/saathi-dev/backend/src/bulk_ops.rs:33)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:2489)

## Differences

### 1. Gemini understates the severity of the auth model problems

Gemini frames the security foundation as largely secure and focuses on scoping/concurrency. The committee audit found broader application-level authorization breaks:

- ticket/comment mutation open to too many roles
- JWT role/scope changes do not take effect until token expiry
- leave notifications leak too broadly

Refs:

- [AUDIT_REPORT_01.md](/home/abhi/saathi-dev/AUDIT_REPORT_01.md:48)
- [AUDIT_REPORT_01.md](/home/abhi/saathi-dev/AUDIT_REPORT_01.md:140)
- [AUDIT_REPORT_01.md](/home/abhi/saathi-dev/AUDIT_REPORT_01.md:149)

### 2. Gemini adds valid issues the committee had not emphasized enough

These should be treated as real additions:

- missing `busy_timeout`
- lack of encompassing transactions around some multi-step writes
- substitution acceptance race due to check-then-act flow

Confirmed refs:

- [backend/src/routes/faculty.rs](/home/abhi/saathi-dev/backend/src/routes/faculty.rs:691)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:4566)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:7427)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:7645)

### 3. Gemini’s swap-request scope finding is narrower than the committee’s broader scope concern

Gemini points out that `list_swap_requests` does not apply `scope_filter`. That is correct, but the route still faculty-filters non-admin/non-AOM users, so the exposure is narrower than a fully global leak for all roles.

Refs:

- [gemini_audit_02_core_systems_security.md](/home/abhi/saathi-dev/gemini_audit_02_core_systems_security.md:19)
- [backend/src/routes/substitutions.rs](/home/abhi/saathi-dev/backend/src/routes/substitutions.rs:209)

### 4. The committee audit includes meaningful issues Gemini does not mention

Important omissions from Gemini’s report:

- client-controlled ticket/comment authorship
- quick-attendance bypass of stricter integrity checks
- alert inbox leakage of per-user dismissed/snoozed state

Refs:

- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:428)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:739)
- [backend/src/repositories.rs](/home/abhi/saathi-dev/backend/src/repositories.rs:8007)
- [backend/src/routes/automation.rs](/home/abhi/saathi-dev/backend/src/routes/automation.rs:106)

## Merged Position

The combined view suggests these priorities for core systems/security:

1. Fix route-role and school-scope enforcement first.
2. Add atomicity to substitution/leave multi-step writes.
3. Harden substitution acceptance against races with atomic conditional updates.
4. Add SQLite concurrency hardening, including `busy_timeout`.
5. Remove client-controlled authorship and other integrity leaks.
6. Replace fragile CSV parsing and clean up latent SQL-hygiene issues.
