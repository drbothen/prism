---
document_type: cycle-manifest
cycle_id: wave-3-multi-tenant
cycle_type: feature
version: wave-3
status: in-progress
started: 2026-04-27T18:00:00Z
completed: ~
producer: orchestrator
---

# Cycle Manifest: Wave 3 — Multi-Tenant

## Delivered

| Metric | Value |
|--------|-------|
| Stories delivered | TBD — Phase 3.A spec authoring in progress |
| BCs created | TBD — BCs 3.1.*-3.7.* pending spec authoring |
| VPs created | TBD |
| Holdout scenarios | TBD (HS-006/007 refresh queued) |
| Total cost | TBD |
| Adversarial passes | 0 (spec authoring phase; implementation not started) |
| Final holdout satisfaction | TBD |
| Release version | TBD |

## Epics

| Epic | Scope | Estimate | Status |
|------|-------|----------|--------|
| E-3.1: OrgId/OrgSlug split + translation layer | OrgId (UUID v7) + OrgSlug (kebab) + OrgRegistry; dual-persist in audit | 5-7 days | SPEC_PENDING |
| E-3.2: Multi-tenant DTU state segregation | Per-org DTU isolation; logical + network isolation | 5-7 days | SPEC_PENDING |
| E-3.3: Customer config schema + harness | TOML `[[dtu]] mode = shared\|client`; validation harness | 5-7 days | SPEC_PENDING |
| E-3.4: Test migration to harness | Migrate existing tests; overnight mutation runs | 3-4 days | SPEC_PENDING |
| E-3.5: src/ convention sweep | Standardize workspace source layout | 0.5-1 day | SPEC_PENDING |
| E-3.6: HS-006/HS-007 refresh | Refresh holdout scenarios (stale BC refs) | 1-2 days | SPEC_PENDING |
| E-3.7: Multi-tenant data generator | Archetype catalog + deterministic generator | 5-7 days | SPEC_PENDING |

## Approved Decisions

D-040 (7-epic plan), D-041 (org identity), D-042 (configurable mode), D-043 (hybrid generator), D-044 (network isolation in-wave), D-045 (spec-first phasing BLOCKING), D-046 (housekeeping triage).

## Spec Changes

| Artifact | Change | Before | After |
|----------|--------|--------|-------|
| ADRs 006-012 | New — pending architect | — | TBD |
| BCs 3.1.*-3.7.* | New — pending spec-writer | — | TBD |

## Living Spec Snapshot

Captured at: TBD (post-Phase-3.A convergence)

## Deprecations (if any)

| Artifact | Deprecated By | Replacement | Sunset Date |
|----------|--------------|-------------|-------------|
| TenantId (existing type) | E-3.1 | OrgId (UUID v7) + OrgSlug (kebab-case) | Wave 3 close |

## Tech Debt Created

_None yet — spec authoring phase._

## Governance Policies Adopted

| Policy | Adopted In | Incident Reference | Generalization |
|--------|-----------|-------------------|----------------|
| spec_first_blocking (D-045) | Wave 3 kickoff | D-045 | Phase 3.A must fully converge before any implementation begins |

## Notes

Phase 3.A is BLOCKING per D-045. No implementation of any kind until ADRs 006-012 + BCs 3.1.*-3.7.* + story decomposition + spec convergence (3 clean passes + consistency-validator + spec-reviewer + drift check) and human approval all complete.

ADR-009 (data generator): schemas sourced from 1898-owned repos (poller-bear OpenAPI for Claroty, poller-express for Cyberint, Armis/CrowdStrike SDK types). No external attribution required.
