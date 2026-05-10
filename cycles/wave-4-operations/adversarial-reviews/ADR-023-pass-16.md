---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T23:45:00Z
phase: 5
inputs:
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
input-hash: "3d5270e"
traces_to: .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
pass: 16
previous_review: ADR-023-pass-15.md
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 16)

## Finding ID Convention

Finding IDs use the format: `ADV-P16-<SEV>-<SEQ>`

Target document: `ADR-023-plugin-only-sensor-architecture.md` v1.12 (target_sha `7287a2b3`).
Verdict: NOT_CLEAN — 3 findings (0 CRIT + 0 HIGH + 1 MED + 2 LOW). Streak: 0/3 unchanged.
7th S-7.01 recurrence at semantic-sibling level.

**Key insight from this pass:** The comprehensive lexical sweep from fix-burst-12 looked for
"step 7" / "step-7" / "step 8" / "step-8" token patterns. It did not catch "instantiation"
at L924 because that token was not in the sweep target list. Lexical sweep has systematic
blind spots for synonym-class defects. The ASSERTION-CHECK METHODOLOGY is required: every
factual claim about boot.rs current state must be verified against actual boot.rs source,
not just swept for known defect tokens.

Trajectory: `26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3`

---

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-PASS15-HIGH-001 | HIGH | RESOLVED | step 7/8 disambiguation at Context + Rule 5 + C4 + Migration Plan PREREQ-D — all 4 sites swept and corrected in v1.12 |
| F-PASS15-MED-001 | MED | RESOLVED | PREREQ-E impossible boot.rs directive replaced with three actual call sites (lib.rs re-export, examples/, tests/) — no boot.rs changes required confirmed |
| F-PASS15-MED-002 | MED | RESOLVED | Context + Rule 5 boot.rs framing for plugin-load step insertion clarified in v1.12 |
| F-PASS15-LOW-001 | LOW | RESOLVED | Rule 5 "wires step 8 cleanup" incoherent verb replaced with coherent framing in v1.12 |

All 4 pass-15 findings RESOLVED. Zero residuals.

---

## Part B — New Findings

### CRITICAL

None.

### HIGH

None.

### MEDIUM

#### ADV-P16-MED-001: L924 "(live plugin load replaces dead instantiation)" — semantic sibling of S-7.01 defect class

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** ADR-023-plugin-only-sensor-architecture.md Migration Plan, PLUGIN-PREREQ-D bullet, L924
- **Description:** The parenthetical "(live plugin load replaces dead instantiation)" makes a
  factual claim about boot.rs current state — specifically that a "dead instantiation" of some
  kind exists in boot.rs that PREREQ-D will replace with a live plugin load. This is false.
  `crates/prism-bin/src/boot.rs` (S-WAVE5-PREP-01 commit `53b87961`) contains steps 7–11 as
  `todo!()` stubs. There is no `PluginRuntime` instantiation, dead or live, in boot.rs. The
  parenthetical is a semantic sibling of the S-7.01 defect class (factual claim about boot.rs
  current state that is incorrect) — but uses the token "instantiation" rather than "step 7/8",
  which is why the comprehensive lexical sweep in fix-burst-12 did not catch it. This is the
  7th recurrence of the S-7.01 pattern.
- **Evidence:** `grep -n 'instantiation\|PluginRuntime' crates/prism-bin/src/boot.rs` returns
  zero results. boot.rs doc-comment (L4-6): "Steps 1–6 are fully implemented per the story's
  AC numbering. Steps 7–11 are annotated `todo!()` stubs for sibling stories." No instantiation
  of any type exists.
- **Proposed Fix:** Delete the entire parenthetical `(live plugin load replaces dead
  instantiation)`. The surrounding L923 and L926-928 text already correctly characterizes
  PREREQ-D scope without it.

### LOW

#### ADV-P16-LOW-001: L923 tense inconsistency — "wire it into" implies step exists; L926-928 says step is being inserted

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** ADR-023-plugin-only-sensor-architecture.md Migration Plan, PLUGIN-PREREQ-D bullet, L923
- **Description:** The PREREQ-D bullet opens with "Deliver `PluginRuntime` infrastructure AND
  wire it into boot.rs plugin-load step". The verb "wire it into" implies the plugin-load step
  already exists in boot.rs and PREREQ-D will wire something into an existing step. But the same
  bullet at L926-928 correctly states: "Plugin-load step insertion (between canonical step 7
  storage and canonical step 8 query-engine) is in PREREQ-D scope." PREREQ-D creates the step;
  the leading verb should reflect creation, not integration into a pre-existing step. The same
  bullet contradicts itself within 5 lines.
- **Evidence:** L923: "wire it into boot.rs plugin-load step" (implies step exists). L926-928:
  "Plugin-load step insertion... is in PREREQ-D scope" (correctly states step does not yet
  exist). boot.rs verified: no plugin-load step between steps 7 and 8 exists today.
- **Proposed Fix:** Change "AND wire it into boot.rs plugin-load step" to "AND insert a new
  plugin-load step into boot.rs" — aligning the leading verb with L926-928.

#### ADV-P16-LOW-002: L931-934 and C5 L630-632 path qualification asymmetry

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** Migration Plan PREREQ-E bullet L931-934 and C5 paragraph L630-632
- **Description:** The PREREQ-E bullet describes three cleanup operations. Item (1) uses a
  fully-qualified path (`crates/prism-spec-engine/src/lib.rs`) but items (2) and (3) use bare
  unqualified paths (`examples/demo_spec_loading.rs`, `tests/bc_2_16_004_test.rs`). The
  unqualified paths are ambiguous — which crate's `examples/` or `tests/` directory is meant?
  The same asymmetry exists at the sibling site C5 L630-632 where `lib.rs` is unqualified and
  the examples and tests paths are also unqualified. Both sites should fully qualify all three
  paths to eliminate ambiguity.
- **Evidence:** L931-934 item (1): `` `crates/prism-spec-engine/src/lib.rs` `` (fully qualified).
  L933: `` `examples/demo_spec_loading.rs` `` (bare). L934: `` `tests/bc_2_16_004_test.rs` ``
  (bare). C5 L630: `` `lib.rs` `` (bare). L631: `` `examples/demo_spec_loading.rs` `` (bare).
  L632: `` `tests/bc_2_16_004_test.rs` `` (bare).
- **Proposed Fix:** Fully qualify all three paths at both locations: `crates/prism-spec-engine/src/lib.rs`,
  `crates/prism-spec-engine/examples/demo_spec_loading.rs`,
  `crates/prism-spec-engine/tests/bc_2_16_004_test.rs`.

---

## Assertion-Check Methodology — Boot.rs Claim Verification

Per the insight from this pass: every body claim about boot.rs current state was
cross-checked against actual `crates/prism-bin/src/boot.rs` source.

| Claim Location | Claim Summary | Verification | Result |
|----------------|---------------|--------------|--------|
| L124-128 (Context) | "boot.rs implements canonical steps 7-11 as todo!() stubs" | Confirmed: boot.rs L4-6 doc-comment + step7..step11 all `todo!()` | PASS |
| L293-296 (Rule 5) | Same as L124-128 | Same verification | PASS |
| L565-568 (C4) | "plugin-load step in boot.rs WILL load .prx WASM plugins" (future tense) | No plugin-load step in current boot.rs — future tense is correct | PASS |
| L623-629 (C5) | "No dead code removal required — S-WAVE5-PREP-01 removed custom_adapter_registry" | `grep 'custom_adapter_registry' boot.rs` → zero results | PASS |
| L923-924 (PREREQ-D) | "(live plugin load replaces dead instantiation)" | `grep 'instantiation\|PluginRuntime' boot.rs` → zero results | FAIL — ADV-P16-MED-001 |
| L934-937 (PREREQ-E) | "No boot.rs changes required — already removed custom_adapter_registry" | Same as C5 — zero results confirmed | PASS |

5 of 6 boot.rs claims PASS. 1 FAIL (ADV-P16-MED-001). The failure was invisible to lexical
sweep because "instantiation" was not in the fix-burst-12 sweep target list.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 1 |
| LOW | 2 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision — fix-burst-13 dispatched

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 16 |
| **New findings** | 3 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (3 / (3 + 0)) |
| **Median severity** | LOW (1 MED + 2 LOW) |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3 |
| **Verdict** | FINDINGS_REMAIN |

Note: All 3 findings are new (zero residuals from pass-15). The MED finding is a 7th-recurrence
of the S-7.01 semantic pattern — same defect class (false factual claim about boot.rs current
state) but different token ("instantiation" vs "step N"), which is why it evaded the fix-burst-12
comprehensive lexical sweep. Streak reset remains at 0/3.
