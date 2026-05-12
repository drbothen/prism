---
cycle: wave-4-operations
last_updated: 2026-05-11
maintainer: orchestrator + state-manager
lessons_codified: 1
---

# Wave-4 Operations — Cycle Lessons

This file durably codifies lessons-learned that emerged during the wave-4 operations cycle (PREREQ-A and PREREQ-B per-story-delivery). Lessons here SHOULD be referenced from fix-burst dispatches and adversary reviews so they don't get lost to STATE.md compaction (per TD-VSDD-058 precedent which documents D-214..D-320 lost to fix-burst-17 compaction).

## Codified Lessons

### Lesson 1: structured-event-catalog ↔ tracing-emission discipline (PG-LP11-001)

**Codified:** 2026-05-11 (fix-burst-12 closure of F-LP12-LOW-002)
**Recurrence count at codification:** 2 (F-LP9-MED-001 closed auth events; F-LP11-MED-001 surfaced same pattern for non-auth events)
**Source decision row:** STATE.md D-419
**Subsystem scope:** SS-16 (prism-spec-engine) — pipeline.rs, auth_provider.rs, validation.rs, interpolation.rs

**Operative rule:** Any fix-burst that introduces a new `tracing::*!(event_type = "...")` site in the prism-spec-engine source files MUST amend the BC-2.16.002 Structured Event Catalog in the SAME atomic commit (TD-VSDD-053). The implementer's burst-closure checklist now includes:

1. After making the code change, run `git diff` and grep for new `event_type = "..."` literals
2. If any new event_types are introduced, identify the field-schema (which structured fields beyond event_type the macro emits)
3. Update BC-2.16.002's Structured Event Catalog (currently v1.8) to add a new row with: event_type | level | function | fields | trigger condition
4. Bump BC version in the same commit
5. Update BC-INDEX with the new BC version

**Why this matters:** The Structured Event Catalog is the contract surface SIEM/SOC operators use to build alert pipelines. A new event_type emitted without catalog update means the contract surface lags impl. The adversary surfaced this pattern twice (P9 + P11). Without this codification, the third occurrence would not be caught until pass-N+M.

**Verification at adversary pass:** the adversary review (LOCAL passes) MUST grep `event_type = "` in pipeline.rs / auth_provider.rs / validation.rs / interpolation.rs and cross-reference against BC-2.16.002 catalog rows. Discrepancy = finding.

**Enforcement layers (status as of 2026-05-11 post-fix-burst-13):**

The SOP relies on FOUR enforcement layers. Current wiring status:

1. **Implementer agent: burst-closure self-check** — STATUS: **PAPER** (not wired in engine prompt as of 2026-05-11). The implementer.md prompt does NOT reference lessons.md or the Structured Event Catalog discipline. A future engine-side TD (filed under vsdd-factory plugin work, task #54) should extend implementer.md to cite this lesson. Until that lands, Layer 1 is aspirational, not enforced.

2. **State-manager agent: pre-commit grep verification** — STATUS: **PAPER** (not wired in state-manager.md prompt as of 2026-05-11). The state-manager.md prompt does NOT include a grep step cross-referencing new event_type sites against BC catalog rows. Until tooling lands (Layer 4), Layer 2 is aspirational.

3. **Adversary agent: pass-N closure verification** — STATUS: **ACTIVE**. Each LOCAL adversary pass since pass-9 has applied this verification. F-LP9-MED-001, F-LP11-MED-001, F-LP12-MED-001, and F-LP13-MED-001 all surfaced via this layer. Layer 3 is the sole load-bearing enforcement until other layers wire.

4. **Lefthook automation: pre-commit grep hook** — STATUS: **DEFERRED** (filed as TD-VSDD-093 P3 for tooling-sprint). When TD-093 lands, a `.factory/hooks/` lefthook pre-commit hook will automatically grep new `event_type = "..."` literals and block commits that don't update BC-2.16.002 catalog. Until then, no automated check exists.

**Net enforcement reality:** 1 of 4 layers actively enforces (adversary). Recurrence count of catalog-drift findings has reached 4 (F-LP9/11/12/13) BECAUSE Layer 3 is the only layer catching it post-impl. The other layers need wiring/tooling to provide pre-impl prevention.

**Linked artifacts:**
- BC-2.16.002 Structured Event Catalog (v1.8 latest)
- F-LP9-MED-001 (auth audit-signal drift, 1st occurrence)
- F-LP11-MED-001 (non-auth events drift, 2nd occurrence)
- F-LP12-LOW-002 (codification-durability gap that surfaced this file's creation)
- STATE.md D-419 (original codification, now superseded by this file)
