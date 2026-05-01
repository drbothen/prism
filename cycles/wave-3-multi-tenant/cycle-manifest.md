---
document_type: cycle-manifest
cycle_id: wave-3-multi-tenant
cycle_type: feature
version: wave-3
status: closed
started: 2026-04-27T18:00:00Z
completed: 2026-04-30T00:00:00Z
producer: state-manager
closed_by: W3-FIX-G
---

# Cycle Manifest: Wave 3 — Multi-Tenant

## Delivered

| Metric | Value |
|--------|-------|
| Stories delivered | 40 (37 Wave 3 MT stories + 3 devx fix stories: W3-FIX-WIN-001, W3-FIX-LEFTHOOK-001, W3-FIX-CI-001) |
| BCs created | 22 (BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001) |
| VPs created | 74 (VP-063–VP-136) |
| Holdout scenarios | HS-006 + HS-007 refreshed (S-3.6.01 PR #83, S-3.6.02 PR #84) |
| Total cost | TBD (session cost tracking pending) |
| Adversarial passes | 47 Phase 3.A spec passes + integration gate passes (in progress) |
| Final holdout satisfaction | 0.71 mean (gate-step-f CONDITIONAL_PASS, 16/30 must-pass — gate integration in progress) |
| Release version | TBD (Wave 4 target) |
| First story merged | S-3.0.01 (PR #73, 6696e374, 2026-04-28) |
| Last story merged | W3-FIX-CI-001 (PR #112, a3bd5a0f, 2026-04-30) |
| Total PRs | 40 (PRs #73–#112) |
| Workspace tests at close | 2363 (nextest-verified, 2363/2363 passing) |
| Wave closed | 2026-04-30 |

## Epics

| Epic | Scope | Estimate | Status |
|------|-------|----------|--------|
| E-3.1: OrgId/OrgSlug split + translation layer | OrgId (UUID v7) + OrgSlug (kebab) + OrgRegistry; dual-persist in audit | 5-7 days | COMPLETE (7 stories: S-3.1.01–07; PRs #81, #93–#98) |
| E-3.2: Multi-tenant DTU state segregation | Per-org DTU isolation; logical + network isolation | 5-7 days | COMPLETE (8 stories: S-3.2.01–08; PRs #85–#92, #102) |
| E-3.3: Customer config schema + harness | TOML `[[dtu]] mode = shared\|client`; validation harness | 5-7 days | COMPLETE (6 stories: S-3.3.01–06; PRs #92, #97, #100–#101, #103–#104) |
| E-3.4: Test migration to harness | Migrate existing tests; overnight mutation runs | 3-4 days | COMPLETE (5 stories: S-3.4.01–05; PRs #107–#111) |
| E-3.5: src/ convention sweep + devx fixes | Workspace crate layout + CI wall-clock + lefthook + Windows port fix | 2-3 days | COMPLETE (4 stories: S-3.5.01, W3-FIX-WIN-001, W3-FIX-LEFTHOOK-001, W3-FIX-CI-001; PRs #82, #105, #106, #112) |
| E-3.6: HS-006/HS-007 holdout refresh | Refresh holdout scenarios (stale BC refs) | 1-2 days | COMPLETE (2 stories: S-3.6.01–02; PRs #83–#84) |
| E-3.7: Multi-tenant data generator | Archetype catalog + deterministic generator | 5-7 days | COMPLETE (6 stories: S-3.7.00–05; PRs #75–#80) |
| E-3.0: Pre-wave quick fix-PRs | lefthook fmt hook fix + DTU_DEFAULT_MODE registry | 0.5 days | COMPLETE (2 stories: S-3.0.01–02; PRs #73–#74) |

## Approved Decisions

D-040 (7-epic plan), D-041 (org identity), D-042 (configurable mode), D-043 (hybrid generator), D-044 (network isolation in-wave), D-045 (spec-first phasing BLOCKING), D-046 (housekeeping triage), D-047–D-180 (Wave 3 implementation decisions — see decisions-archive-d047-d114.md and STATE.md D-115–D-180).

## Spec Changes

| Artifact | Change | Before | After |
|----------|--------|--------|-------|
| ADRs 006-012 | New — Wave 3 multi-tenant architecture | — | All 7 ADRs ACCEPTED v0.9–v0.14 |
| BCs 3.1.*-3.7.* | New — 22 Wave 3 behavioral contracts | — | All 22 BCs at v0.3+ PROPOSED |
| PRD | OrgId/OrgSlug + multi-tenant additions | v1.5 | v1.7 |

## Living Spec Snapshot

Captured at: 2026-04-28 (Phase 3.A converged — 3-clean-pass window P45+P46+P47; human-approved 2026-04-28)

develop HEAD at close: `a3bd5a0f`

## Deprecations

| Artifact | Deprecated By | Replacement | Sunset Date |
|----------|--------------|-------------|-------------|
| TenantId (existing type) | E-3.1 (S-3.1.02, PR #93) | OrgId (UUID v7) + OrgSlug (kebab-case) | Wave 4 (alias retained Wave 3 per D-157) |

## Tech Debt Created

| ID | Priority | Description |
|----|----------|-------------|
| TD-W3-S-3.0.02-DOC-001 | P3 | Marker comment wording in story v0.6 |
| TD-W3-S-3.7.01-001 | P3 | Bare constants in pagination.rs |
| TD-S3705-001 | P3 | prism-core dep optionality |
| TD-S3501-W3-001 | P3 | Pre-existing clippy errors in sensor DTU crates |
| TD-W3-CI-MSVC-001 | P3 | Windows MSVC CI flake |
| TD-VSDD-025..029 | P3 | Process-gap TDs deferred to vsdd-factory plugin |

## Governance Policies Adopted

| Policy | Adopted In | Incident Reference | Generalization |
|--------|-----------|-------------------|----------------|
| spec_first_blocking (D-045) | Wave 3 kickoff | D-045 | Phase 3.A must fully converge before any implementation begins |
| atomic-rename pattern (D-156) | Batch 4 | D-156 | TDD + `-D warnings` requires stub+impl merge atomically for mechanical renames |
| sibling-merge rebase (D-148) | Batch 1 | D-148 | Parallel stories modifying shared files: merge origin/develop, resolve additively, push as new commit |

## Notes

Wave 3 (Multi-Tenant) CLOSED 2026-04-30. All 37 Wave 3 MT stories merged (PRs #73–#112 excl. #105/#106/#112 devx fixes). 3 devx fix stories (W3-FIX-WIN-001, W3-FIX-LEFTHOOK-001, W3-FIX-CI-001) merged 2026-04-30 as E-3.5 devx sprint. 5 multi-tenant capabilities implemented: CAP-036 (DTU Test Harness), CAP-037 (Workspace Crate Layout Convention), CAP-038 (Customer Config Schema), CAP-039 (DTU Mode Tagging), CAP-040 (Shared-Mode OrgId Tagging). Integration gate (Steps C–G) in progress as of 2026-05-01 cycle-manifest closure. W3-FIX-G state-hygiene burst executed 2026-05-01 to close WGCV-W3-001..004.
