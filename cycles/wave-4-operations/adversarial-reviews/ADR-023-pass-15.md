---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T18:30:00Z
phase: 5
pass: 15
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-15
date: 2026-05-10
reviewer: adversary
target_artifact_sha_at_review: "0c22c555"
target_artifact_version: "v1.11"
findings_total: 4
findings_by_tier:
  CRIT: 0
  HIGH: 1
  MED: 2
  LOW: 1
  OBS: 0
process_gap_findings: 0
pass_number: 15
previous_review: "ADR-023-pass-14.md"
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 1
new_findings_this_pass: 3
streak_status: "0/3 unchanged — 6th S-7.01 sibling-site recurrence"
trajectory: "26→16→12→14→3→3→1→0→0→4→2→0→1→1→4"
verifications_performed: 22
related_tasks: []
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-14.md"
  - "crates/prism-bin/src/boot.rs"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 15)

## Finding ID Convention

Finding IDs use the pass-15-scoped format:

- `F-PASS15-{CRIT,HIGH,MED,LOW,OBS}-NNN` — finding in pass-15

This pass surfaces **ONE HIGH finding, TWO MEDIUM findings, and ONE LOW finding**:
F-PASS15-HIGH-001 (stale "step 7/step-8" semantics at 4 body sites — 6th S-7.01
recurrence), F-PASS15-MED-001 (PREREQ-E directed to delete non-existent boot.rs code —
logical impossibility), F-PASS15-MED-002 (Context + Rule 5 ambiguous about plugin-load step
insertion), F-PASS15-LOW-001 (Rule 5 "PREREQ-E wires step 8 cleanup" — incoherent verb +
stale step number).

---

## Part A — Fix Verification (Pass 14 Residuals)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-PASS14-HIGH-001 | HIGH | PARTIALLY_RESOLVED | C5 paragraph rewritten correctly — PREREQ-D vs PREREQ-E boot wiring attribution now clean at C5 (L618-628). However, Migration Plan PREREQ-D (L921-922) + PREREQ-E (L925-926) + Rule 5 (L290+L292) retain stale "step 7/8" semantics at sibling sites not named in pass-14 report. The fix-burst-11 named-site approach closed C5 but left four sibling sites. |
| F-PASS14-OBS-001 | OBS | RESOLVED | Cosmetic deferred; not re-raised. |
| F-PASS14-OBS-002 | OBS | RESOLVED | boot.rs step numbering process-gap closed — C5 now explains canonical vs plugin-load step placement. Partial: same clarity not propagated to Context, Rule 5, Migration Plan (subsumed by F-PASS15-HIGH-001). |

---

## Part B — New Findings (or all findings for pass 1)

### HIGH

#### F-PASS15-HIGH-001: Stale "step 7/step-8" Semantics Surviving at 4 Body Sites
- **Severity:** HIGH
- **Category:** ambiguous-language / S-7.01 sibling-site propagation (6th recurrence)
- **Location:** Context L124-125; Rule 5 L290; Rule 5 L292; Migration Plan PREREQ-D L921-922
- **Description:** Fix-burst-11 correctly rewrote the C5 paragraph to clarify that PREREQ-D inserts a new plugin-load step between canonical step 7 (storage) and canonical step 8 (query-engine), and that PREREQ-E performs cleanup only. However, the same defect class survived at four additional body sites that were not in the named-site sweep. These four sites are the primary reader entry points — Context and Rule 5 are read before C5.
- **Evidence:**
  - Context L124-125: "implements steps 7-11 as `todo!()` stubs awaiting PREREQ-D wiring." The phrase "steps 7-11" is ambiguous between ADR-022 canonical numbering (storage=7, query-engine=8) and post-PREREQ-D numbering where a new plugin-load step is inserted between 7 and 8. An implementer cannot determine whether the step numbers shift after insertion.
  - Rule 5 L290: "The boot sequence in `crates/prism-bin/src/boot.rs` implements steps 7-11 as `todo!()` stubs (post-S-WAVE5-PREP-01, commit `53b87961`) awaiting live wiring in PREREQ-D." Same ambiguity as Context.
  - Rule 5 L292: "PREREQ-E wires step 8 cleanup if needed." (a) "wires" is the boot-sequence integration verb that belongs to PREREQ-D, not PREREQ-E; (b) "step 8 cleanup" is incoherent given post-fix-burst-11 C5 language stating PREREQ-E performs dead-code cleanup with no live wiring.
  - Migration Plan PREREQ-D L921-922: "Step 7 wiring is in PREREQ-D scope (F-MED-NEW-005)." Refers to "step 7" in isolation, conflicting with C5's clarification that PREREQ-D inserts a NEW step at the boundary between canonical 7 and 8 — not merely "wiring step 7."
- **Proposed Fix:** Body-wide grep sweep of "step 7", "step-7", "step 8", "step-8" across the entire document body (excluding changelog/historical rows). At Context and Rule 5: replace "steps 7-11" with "canonical steps 7-11 (storage init, query-engine init, MCP server, hot-reload watcher, signal handlers)" and note PREREQ-D inserts a new plugin-load step between canonical 7 and 8. At Rule 5 L292: replace "PREREQ-E wires step 8 cleanup if needed" with "PREREQ-E performs dead-code cleanup only at the three call sites identified in C5 (lib.rs re-export, examples/, tests/); no boot.rs wiring required." At Migration Plan PREREQ-D: replace "Step 7 wiring is in PREREQ-D scope" with "Plugin-load step insertion (between canonical step 7 storage and canonical step 8 query-engine) is in PREREQ-D scope."

### MEDIUM

#### F-PASS15-MED-001: PREREQ-E Directed to Delete Non-Existent boot.rs Code
- **Severity:** MEDIUM
- **Category:** spec-fidelity / logical impossibility
- **Location:** Migration Plan PREREQ-E L925-926
- **Description:** The Migration Plan's PREREQ-E scope bullet directs: "remove dead step-8 `custom_adapter_registry` code from boot.rs." This code does not exist. C5 L622-624 (added in fix-burst-11) correctly states: "No dead code removal is required from the current boot.rs since S-WAVE5-PREP-01 already removed pre-existing dead `custom_adapter_registry` references." The Migration Plan contradicts C5 with an impossible implementation directive.
- **Evidence:**
  - Migration Plan PREREQ-E L925-926: "remove dead step-8 `custom_adapter_registry` code from boot.rs (F-MED-NEW-005: PREREQ-E owns step-8 cleanup only, not step-7 wiring)"
  - grep verification: `grep -r 'custom_adapter_registry\|custom_adapter' crates/prism-bin/src/boot.rs` → zero matches
  - C5 L622-624: "No dead code removal is required from the current boot.rs since S-WAVE5-PREP-01 already removed pre-existing dead `custom_adapter_registry` references."
  - Internal contradiction: the same PREREQ-E bullet that says "remove dead step-8 code from boot.rs" cites F-MED-NEW-005 ("PREREQ-E owns step-8 cleanup only, not step-7 wiring") — but step-8 cleanup of boot.rs is itself impossible, making the F-MED-NEW-005 citation here self-defeating.
- **Proposed Fix:** Replace PREREQ-E boot.rs directive with the three actual call sites from C5: (1) `pub use custom_adapter::{...}` re-export in `crates/prism-spec-engine/src/lib.rs`; (2) `examples/demo_spec_loading.rs` CustomAdapter usage; (3) `tests/bc_2_16_004_test.rs` CustomAdapter usage. Add explicit note: "No boot.rs changes required — S-WAVE5-PREP-01 commit `53b87961` already removed pre-existing dead `custom_adapter_registry` references."

#### F-PASS15-MED-002: Context + Rule 5 Ambiguous About Plugin-Load Step Insertion
- **Severity:** MEDIUM
- **Category:** ambiguous-language / missing-edge-cases (step renumbering not specified)
- **Location:** Context L124-125; Rule 5 L290
- **Description:** Both Context and Rule 5 say boot.rs "implements steps 7-11 as `todo!()` stubs" without stating that PREREQ-D will insert a NEW step between canonical 7 and 8, renumbering steps 8-11. A reader who encounters Context or Rule 5 first (before reaching C5) does not know whether boot.rs post-PREREQ-D will still have a "step 8" in the ADR-022 canonical sense or whether the step sequence shifts. C5 makes this clear at the constraint level, but Context and Rule 5 are the natural reader entry points.
- **Evidence:** Context L124-125 and Rule 5 L290 both use bare "steps 7-11" without disambiguation. C5 uses "positioned between the canonical storage and query-engine steps per ADR-022 numbering; PREREQ-D specifies exact placement" — clearer but only at C5.
- **Proposed Fix:** Subsumed by F-PASS15-HIGH-001 fix — the body-wide sweep at Context and Rule 5 sites should include the renumbering note.

### LOW

#### F-PASS15-LOW-001: Rule 5 L292 "PREREQ-E Wires Step 8 Cleanup" — Incoherent Verb
- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** Rule 5 L292
- **Description:** "PREREQ-E wires step 8 cleanup if needed." "Wires" is the boot-sequence integration verb (PREREQ-D wires the plugin-load step into boot.rs). PREREQ-E performs deletion/cleanup — it does not wire anything. The phrase "wires step 8 cleanup" is syntactically malformed. Additionally "step 8" is ambiguous for the same reasons as F-PASS15-HIGH-001 and the phrase "if needed" is vague given that C5 is definitive about what cleanup is required.
- **Evidence:** Rule 5 L292: "PREREQ-E wires step 8 cleanup if needed." C5 post-fix-burst-11 uses "PREREQ-E performs dead-code cleanup only (no live wiring)" — which directly contradicts the "wires" verb at Rule 5 L292.
- **Proposed Fix:** Subsumed by F-PASS15-HIGH-001 fix — the Rule 5 L292 site replacement handles this.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate (streak 0/3 unchanged)
**Readiness:** Requires fix-burst-12 before pass-16

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 15 |
| **New findings** | 3 (F-PASS15-MED-001, F-PASS15-MED-002, F-PASS15-LOW-001) |
| **Duplicate/variant findings** | 1 (F-PASS15-HIGH-001 is a sibling-site recurrence of F-PASS14-HIGH-001 defect class) |
| **Novelty score** | 0.75 (3 new / 4 total) |
| **Median severity** | 2.5 (between MED and HIGH) |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4 |
| **Verdict** | FINDINGS_REMAIN |
