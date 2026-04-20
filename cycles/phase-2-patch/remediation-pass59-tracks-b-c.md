---
document_type: remediation-manifest
pass: 59
track: B-C
producer: product-owner (Track B), architect (Track C)
date: 2026-04-20
findings_addressed: [MED-001-p59, LOW-002-p59]
status: complete
---

# Remediation Pass-59 Tracks B and C

Supplementary manifest documenting pass-59 remediation work performed by the
product-owner (Track B) and architect (Track C) agents. Written by state-manager
to close LOW-001 from pass-60 (manifest gap finding).

Track A manifest is at: `.factory/cycles/phase-2-patch/remediation-pass59-track-a.md`

---

## Track B — Product-Owner: epics.md E-6 Wave Assignment Fix

**Finding addressed:** MED-001 (pass-59) — E-6 wave column in epics.md listed `6`
but the Option 2 DTU-first strategy (decided 2026-04-20) distributes DTU clones across
Waves 0–3 to precede their product consumers. Wave `6` alone was incorrect.

**File modified:** `/Users/jmagady/Dev/prism/.factory/specs/epics.md`

### Change

| Field | Before | After |
|-------|--------|-------|
| E-6 Wave column | `6` | `0–3, 6` |
| Version | `1.0` | `1.1` |

**Footnote added:** E-6 receives a two-track delivery under Option 2 DTU-first:
DTU clone stories (S-6.06–S-6.19) are distributed across Waves 0–3 as prerequisites
for their product consumers; the product-facing CLI/integration work (S-6.04, S-6.05)
ships in Wave 6. See `cycles/phase-2-patch/remediation-step5-option2-dtu.md` for the
full DTU-first wave schedule.

### Changelog Row Added

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-04-20 | product-owner | E-6 wave column corrected to `0–3, 6`; footnote added for Option 2 DTU-first two-track delivery |

---

## Track C — Architect: verification-coverage-matrix.md Integration Tests Column

**Finding addressed:** LOW-002 (pass-59) — `verification-coverage-matrix.md` had columns
for Unit Tests, Property Tests, and Holdout Scenarios but was missing an Integration Tests
column. VP-INDEX v1.5 includes 39 VPs of which 6 are integration-test-type VPs; the missing
column meant per-module VP counts could not sum correctly.

**File modified:** `/Users/jmagady/Dev/prism/.factory/specs/architecture/verification-coverage-matrix.md`

### Change

**Integration Tests column added** to the per-module coverage matrix. Column placement:
after Property Tests, before Holdout Scenarios.

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | (original) | architect | Initial matrix |
| 1.1 | 2026-04-20 | architect | Integration Tests column added; per-module counts populated; VP-INDEX parity confirmed (20 + 11 + 6 + 2 = 39 VPs) |

**VP-INDEX parity verification:**

| VP Type | Count | Source |
|---------|-------|--------|
| Unit test VPs | 20 | VP-INDEX P0 tier |
| Property test VPs | 11 | VP-INDEX P0 tier |
| Integration test VPs | 6 | VP-INDEX P1 tier |
| Holdout scenario VPs | 2 | VP-INDEX P1 tier |
| **Total** | **39** | **VP-INDEX v1.5** |

Per-module integration test counts distributed across subsystems with integration
boundaries (query engine, sensor adapters, MCP transport, credential store).

### Version

| Field | Before | After |
|-------|--------|-------|
| Version | `1.0` | `1.1` |

---

## Summary

| Track | Agent | File | Finding Closed | Version |
|-------|-------|------|---------------|---------|
| B | product-owner | specs/epics.md | MED-001 (pass-59) — E-6 wave incorrect | 1.0→1.1 |
| C | architect | specs/architecture/verification-coverage-matrix.md | LOW-002 (pass-59) — Integration Tests column missing | 1.0→1.1 |

Both tracks complete. No further remediation required for these findings.
Pass-60 LOW-001 (manifest gap) closed by this document.
