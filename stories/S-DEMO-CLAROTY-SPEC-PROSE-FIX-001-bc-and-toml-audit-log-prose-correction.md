---
document_type: story
story_id: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
title: "Claroty audit_log spec-prose + TOML-comment fidelity fix — BC-2.16.013 §Postconditions + claroty.sensor.toml audit_logs comments (closes F-P2-DEFER-001)"
wave: wave-5-e-demo-fidelity
epic_id: E-DTU-FIDELITY
priority: P2
status: draft
# BC status: pending PO authorship
# S-7.01 gate: behavioral_contracts empty → status must remain draft until PO authors BCs.
# Candidate BCs: BC-2.16.013 (Bundled Sensor Spec Authoring — prose correction is within
# BC-2.16.013 scope). PO owns BC-2.16.013 edits per Agent Routing Table.
# PO authorship required before this story can be dispatched.
version: "1.0"
level: "L1"
producer: story-writer
timestamp: "2026-06-01T00:00:00Z"
tdd_mode: strict
subsystems: [SS-16, SS-17]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine) owns prism-sensors and claroty.sensor.toml; the TOML audit_logs
#   comment corrections are a spec-engine documentation surface under SS-16.
#   SS-17 (DTU Clones) owns BC-2.16.013 (DTU-parity verification BC authored by PO for
#   the DTU fidelity surface); the prose correction to BC-2.16.013 §Postconditions is SS-17
#   scope because it documents DTU route registration status.
crates_touched: [prism-sensors]
# Also amends: .factory/specs/behavioral-contracts/BC-2.16.013*.md (PO-owned edit)
target_module: prism-sensors
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
depends_on:
  - S-DEMO-CLAROTY-AUDIT-DTU-001
  # Dependency anchor: the prose corrections are only accurate AFTER S-DEMO-CLAROTY-AUDIT-DTU-001
  # merges and Gap-CL-006 is actually closed. Correcting the prose before the route exists would
  # be a false closure. This is a hard ordering dependency — the corrections reference the merged
  # story by ID and declare the gap CLOSED.
blocks: []
points: 1
estimated_days: 0.5
risk: LOW
# Risk justification:
#   Pure documentation / comment corrections. No code logic changes. The TOML audit_logs
#   table itself (path_template, response_path, columns) was already corrected by the
#   Gap-CL-002 fix at develop@72baf413. Only the inline comments and the BC prose
#   describing the gap status need updating.
assumption_validations: []
risk_mitigations: []
# Deferred-finding closure:
#   Closes F-P2-DEFER-001 (out-of-perimeter finding: stale "DTU gap: no route yet" comments
#   in claroty.sensor.toml audit_logs table and stale BC-2.16.013 §Postconditions §1 prose,
#   deferred out-of-perimeter during S-DEMO-CLAROTY-AUDIT-DTU-001 cascade because TOML was
#   forbidden-modify for that story). Promoted to goal task per user direction 2026-06-02.
---

# S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 v1.0 — Claroty Audit Log Spec-Prose Fidelity Fix

**Story ID:** S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
**Status:** draft (pending PO BC authorship)
**Version:** v1.0
**Wave:** wave-5-e-demo-fidelity
**Priority:** P2
**Points:** 1

---

## Origin

**F-P2-DEFER-001** — Deferred out-of-perimeter during S-DEMO-CLAROTY-AUDIT-DTU-001 cascade
(2026-05-31). The S-DEMO-CLAROTY-AUDIT-DTU-001 story explicitly forbids modifying
`crates/prism-sensors/specs/claroty.sensor.toml` (TOML already correct for the route; the
story is DTU-only). However, audit_logs table-header and step comments in that TOML still
say "DTU gap: no /api/v1/audit_log/get route ... yet ... 404 until DTU route lands."
After S-DEMO-CLAROTY-AUDIT-DTU-001 merges, those comments become stale misinformation.

Similarly, BC-2.16.013 §Postconditions §1 contains an "audit_logs" prose clause
(approximately line 169 in the current file) that reads:

> "GET /api/v1/audit_logs via offset pagination. No DTU route registered."

This was accurate when first authored (pre-S-DEMO-CLAROTY-AUDIT-DTU-001) but becomes
incorrect once Gap-CL-006 is closed.

Promoted to goal task per user direction 2026-06-02.

**Note on ownership:** BC-2.16.013 edits are product-owner (PO) owned per Agent Routing Table
(architecture → architect, behavioral contracts → product-owner). The implementer for this
story must route the BC edit to the PO for authorship, or the PO must be identified as the
implementer. The TOML comment corrections are implementer scope (spec-prose within the spec
file, not behavioral contract authorship).

---

## Narrative

As a developer reading the Claroty sensor spec or reviewing BC-2.16.013,
I want the audit_logs comments in `claroty.sensor.toml` and the audit_logs prose in
BC-2.16.013 §Postconditions §1 to reflect the actual state of the system after
S-DEMO-CLAROTY-AUDIT-DTU-001 merges,
so that spec-readers and AI agents do not believe the DTU gap still exists when it has
been closed, and so that the next adversarial pass does not re-flag stale prose as a
finding.

---

## Scope

### Change 1 — `claroty.sensor.toml` audit_logs table comments

File: `crates/prism-sensors/specs/claroty.sensor.toml`

Find and replace the stale "DTU gap" comments in the `audit_logs` table block
(table-header and step comments). The exact text varies; the canonical replacement target is
any comment line containing any of:

- `"DTU gap"`
- `"no /api/v1/audit_log/get route"`
- `"404 until DTU route lands"`

Replace with:

```
# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.
# POST /api/v1/audit_log/get route registered in prism-dtu-claroty.
```

Preserve all functional TOML content (path_template, method, response_path, columns).
Only comment lines are changed.

### Change 2 — BC-2.16.013 §Postconditions §1 audit_logs prose

File: `.factory/specs/behavioral-contracts/BC-2.16.013-*.md`
(PO-OWNED EDIT — requires product-owner authorship per Agent Routing Table)

Locate the audit_logs clause in §Postconditions §1 (approximately line 169 in current
file). The clause currently reads (paraphrased):

> "GET /api/v1/audit_logs via offset pagination. No DTU route registered."

Correct to:

> "POST /api/v1/audit_log/get; DTU route registered by S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED)."

The PO must also bump the BC version and update the BC-INDEX row for BC-2.16.013.

---

## Behavioral Contracts

Pending PO authorship. Candidate:

| BC (candidate) | Title | Why relevant |
|----------------|-------|-------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification | The prose correction is within BC-2.16.013's scope — it documents DTU route registration status, which is the subject of that BC. |

PO must author canonical BCs and set `behavioral_contracts:` before `status: ready`.

---

## Acceptance Criteria (stub — expand after BC authorship)

### AC-001: Stale "DTU gap" comments removed from claroty.sensor.toml
`crates/prism-sensors/specs/claroty.sensor.toml` contains no comment lines with
`"DTU gap"`, `"no /api/v1/audit_log/get route"`, or `"404 until DTU route lands"` in
the audit_logs table block.
(traces to BC-TBD — pending PO authorship)

### AC-002: Gap-CL-006 closure comment present in claroty.sensor.toml
The audit_logs table block in `claroty.sensor.toml` contains a comment line with
`"Gap-CL-006 CLOSED"` and a reference to `S-DEMO-CLAROTY-AUDIT-DTU-001`.
(traces to BC-TBD)

### AC-003: BC-2.16.013 §Postconditions §1 audit_logs prose updated (PO-owned)
BC-2.16.013 §Postconditions §1 audit_logs clause no longer reads "No DTU route registered."
The updated clause references `POST /api/v1/audit_log/get` and cites Gap-CL-006 CLOSED.
BC version is bumped and BC-INDEX row updated.
(traces to BC-TBD — PO owns this edit)

### AC-004: No functional TOML content changed
`claroty.sensor.toml` audit_logs `[[tables.steps]]` path_template, method, response_path,
and column declarations are identical before and after this story (comments only).
(traces to BC-TBD)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty.sensor.toml` audit_logs comments | `crates/prism-sensors/specs/claroty.sensor.toml` | Pure (static spec comments) |
| BC-2.16.013 §Postconditions §1 prose | `.factory/specs/behavioral-contracts/BC-2.16.013-*.md` | Pure (spec document, PO-owned) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-16 Spec Engine
- `architecture/module-decomposition.md` §SS-17 DTU Clones

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | The TOML audit_logs block has multiple stale comment lines | All stale lines replaced; none remain |
| EC-002 | BC-2.16.013 file not found at expected path | Implementer reads BC-INDEX to locate current canonical BC-2.16.013 file path before editing |
| EC-003 | BC-2.16.013 audit_logs prose is in a different subsection than §Postconditions §1 | Implementer reads the full BC file and locates the audit_logs mention regardless of exact line number (TD-VSDD-091: do not rely on line numbers) |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~1,500 |
| `claroty.sensor.toml` (audit_logs section scan) | ~1,000 |
| `BC-2.16.013` full file | ~6,000 |
| **Total estimate** | **~8,500 tokens** |

Minimal context requirement. Well within 20-30% of agent context window.

---

## Tasks (stub — expand at dispatch)

- [ ] **Task 1:** Read `crates/prism-sensors/specs/claroty.sensor.toml` — locate audit_logs
  table block; identify all stale "DTU gap" / "no route" / "404" comment lines.
- [ ] **Task 2:** Replace stale comments with Gap-CL-006 CLOSED notation per Change 1 scope
  above. Verify no functional TOML content changed (AC-004).
- [ ] **Task 3 (PO-gated):** Route BC-2.16.013 §Postconditions §1 prose correction to
  product-owner. The PO must edit the BC file (Change 2 above), bump the BC version, and
  update BC-INDEX. This task is blocked on PO availability — if PO is in the same dispatch,
  include the BC edit; otherwise note the outstanding PO task in the PR description.
- [ ] **Task 4:** Verify `cargo nextest run -p prism-sensors` still passes after TOML comment
  changes (TOML parser must still accept the file).
- [ ] **Task 5:** SAP-1 sweep — `rg 'event_type\s*=' crates/ --type rust` — no new emissions
  expected (comment-only change).
- [ ] **Task 6:** `just check` — final pre-push gate.

---

## Previous Story Intelligence

N/A — first story in E-DTU-FIDELITY for spec-prose corrections. This story has no code
logic; it is pure documentation cleanup deferred from S-DEMO-CLAROTY-AUDIT-DTU-001.

Read S-DEMO-CLAROTY-AUDIT-DTU-001 §File Structure Requirements "Files MUST NOT be modified"
section to understand why the TOML was excluded from that story's scope.

---

## Architecture Compliance Rules

- PO owns BC edits (Agent Routing Table — behavioral contracts → product-owner)
- Implementer may edit TOML comment lines but must NOT change functional TOML content
- TD-VSDD-091: cite behavioral anchors / story IDs in comments, not file:line numbers
- No `event_type` emissions expected (comment-only change; SAP-1 still required as final check)

---

## Library & Framework Requirements

None — no code changes. TOML comment edit only (plus BC prose edit by PO).

---

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | audit_logs table comment lines only |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.16.013-*.md` | §Postconditions §1 audit_logs prose (PO-owned edit) |
| MODIFY | `.factory/specs/behavioral-contracts/BC-INDEX.md` | BC-2.16.013 version bump row (PO-owned) |

---

## Forbidden Dependencies

No new crate dependencies. This story introduces no Rust code.

---

## References

- F-P2-DEFER-001 — deferred finding: stale "DTU gap" comments in claroty.sensor.toml
- S-DEMO-CLAROTY-AUDIT-DTU-001 §File Structure Requirements — "Files MUST NOT be modified" list
- BC-2.16.013 §Postconditions §1 — audit_logs clause (current location: ~line 169)
- Gap-CL-006 — DTU audit_log route gap (CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001)
- ADR-031 §D1 — DTU clone isolation (no doc changes needed in ADR; story reference only)

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-06-01 | story-writer | Initial stub. Captures scope (TOML audit_logs comment corrections + BC-2.16.013 §Postconditions §1 prose update), gating (depends_on S-DEMO-CLAROTY-AUDIT-DTU-001), PO ownership boundary for BC edits, and finding closure (F-P2-DEFER-001). Status draft pending PO BC authorship per S-7.01. |
