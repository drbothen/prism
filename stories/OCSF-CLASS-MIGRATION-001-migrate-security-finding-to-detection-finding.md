---
document_type: story
story_id: OCSF-CLASS-MIGRATION-001
title: "prism-ocsf + sensor TOMLs: Migrate ocsf_class security_finding → detection_finding (OCSF v1.1 deprecation)"
wave: 5
epic_id: E-DEMO
priority: P2
status: ready
version: "1.6"
level: "L4"
producer: story-writer
timestamp: "2026-06-01T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns the production sensor TOML specs that declare ocsf_class;
#     the 4 TOML edits live in crates/prism-sensors/specs/ (NOT prism-spec-engine).
#   SS-16 (Spec Engine) owns crates/prism-ocsf/src/class_selector.rs where
#     select_by_class_name maps string names to OCSF class_uid integers.
#   prism-bin is touched because the conformance test fixture
#     crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs asserts class_uid 2001→2004.
#     prism-bin is not a subsystem anchor for this story (SS-01 and SS-16 cover the
#     production scope); the prism-bin fixture touch is a test-only side effect.
target_module: prism-ocsf
crates_touched: [prism-ocsf, prism-bin]
behavioral_contracts:
  - BC-2.02.012   # OCSF Class Selection — v1.5 (Wave-5-Phase-B-gate-F-002 2026-06-03):
                  # select_by_class_name mapping table added; "detection_finding"→2004 PRIMARY;
                  # "security_finding"→2004 transitional alias with ocsf.deprecated_class_alias
                  # WARN emission (Option A); INV-NO-2001-SELECT-PATH + INV-PRODUCTION-TOML-NO-SECURITY-FINDING.
                  # OQ-1 CLOSED (Option A selected). OQ-2 CLOSED (BC amended). BC is now v1.5
                  # (v1.4→v1.5: status-sync draft→active only; no semantic change).
  - BC-2.01.013   # DataSource Trait — v1.14 (S-DEMO-QUERY-PUSHDOWN-001-v2.1-armis-aql-full-wiring 2026-06-05):
                  # Armis push-down scope extended to AQL-clause augmentation (v1.14); OCSF Conformance
                  # Clause unchanged — select_by_class_name table still correct:
                  # "security_finding"→2004 transitional alias per BC-2.02.012 v1.5 Option A;
                  # TV-BC-2.01.013-005 asserts 2004. This story's relevant clause (OCSF Conformance)
                  # was last semantically amended at v1.10 (Wave-5 Phase-A PO burst 2026-06-03);
                  # v1.14 does NOT change OCSF Conformance Clause semantics.
# BC status: BOTH BCs amended by PO. BC-2.02.012 is v1.5 (active). BC-2.01.013 is v1.14 (active).
# S-7.01 gate: behavioral_contracts is non-empty and both BCs are active → story MAY
# transition to ready once OQ-3 (TOML file paths) is resolved at dispatch.
# AC↔BC bidirectional trace verification required before status=ready transition.
verification_properties: []
depends_on:
  - S-DEMO-001   # Must merge first — establishes the conformance test fixture this story may update
blocks: []
points: 3
estimated_days: 1
risk: LOW
traces:
  - "OBS-2 (S-DEMO-001 PR #166 PR-LEVEL adversarial pass 1, 2026-06-01)"
  - "architect adjudication: non-blocking deferral, D-925 intentional transitional (2026-06-01)"
---

# OCSF-CLASS-MIGRATION-001 — Migrate ocsf_class security_finding → detection_finding

## Deferral Rationale

**Filing:** OBS-2 from S-DEMO-001 PR #166 PR-LEVEL adversarial pass 1 (2026-06-01).
**Adjudication:** Architect — non-blocking deferral. Non-blocking because:

1. **D-925 intentional transitional decision.** BC-2.01.013 v1.9 (now v1.14; OCSF Conformance Clause unchanged from v1.10) explicitly records:
   > "real sensors use this name — use until migration to detection_finding"
   The current `ocsf_class = "security_finding"` in 4 production sensor TOMLs is intentional,
   not an oversight.

2. **Current behavior is valid, not broken.** `select_by_class_name("security_finding")` maps
   to OCSF class_uid 2001. OCSF class 2001 was deprecated in OCSF v1.1.0 in favor of 2004
   (Detection Finding). The data produced is structurally valid OCSF; it uses the deprecated
   class UID, but downstream consumers that handle 2001 continue to work. No runtime crash,
   no data loss.

3. **BC-2.02.012 ↔ BC-2.01.013 v1.9 (now v1.14) contradiction.** BC-2.02.012 contains a "no deprecated
   2001 class_uid" invariant that is scoped to the `select()` path. The `select_by_class_name`
   path returned 2001 transitionally per the D-925 decision. This story resolves the
   contradiction by migrating away from 2001 entirely and amending BC-2.02.012 to close the
   gap. Until this story dispatches, the spec-level contradiction is documented (not silently
   tolerated — it is legitimately deferred with this anchor story).

**This story must be dispatched before any OCSF v1.1-strict consumer integration goes live.**

---

## Narrative

As the Prism sensor data pipeline, I want all production sensor TOMLs to declare
`ocsf_class = "detection_finding"` (OCSF v1.1 class_uid 2004) and the
`select_by_class_name` mapping in `prism-ocsf` to canonically resolve that name, so that
Prism emits current (non-deprecated) OCSF class UIDs and downstream consumers that
validate against OCSF v1.1+ schemas do not reject Prism output.

---

## Behavioral Contracts

| BC ID | Version | Title | Role in This Story |
|-------|---------|-------|-------------------|
| BC-2.02.012 | v1.5 | OCSF Event Class Selection Per Sensor Record Type | Primary anchor. v1.4 added `select_by_class_name()` mapping table with `"detection_finding"` → 2004 (PRIMARY) and `"security_finding"` → 2004 (transitional alias, Option A). v1.5 is a status-sync only (draft→active; no semantic change). ACs implement INV-NO-2001-SELECT-PATH, INV-PRODUCTION-TOML-NO-SECURITY-FINDING, and the deprecation WARN emission `ocsf.deprecated_class_alias`. |
| BC-2.01.013 | v1.14 | DataSource Trait Eliminates Per-Sensor Code Duplication | SpecDrivenSensorAdapter OCSF Conformance Clause: `select_by_class_name` must map `"security_finding"` to 2004 (NOT 2001) as transitional alias. TV-BC-2.01.013-005 corrected to assert `class_uid == 2004` for `"security_finding"` input. AC-002 and the conformance boundary test must satisfy this TV. (OCSF Conformance Clause semantically unchanged from v1.10; v1.14 added Armis AQL-clause augmentation to push-down scope — no semantic change to OCSF Conformance Clause relevant to this story.) |

---

## Scope

### 1. Update 4 production sensor TOML specs

| File | Field | Old value | New value |
|------|-------|-----------|-----------|
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` (detections table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |
| `crates/prism-sensors/specs/armis.sensor.toml` (alerts table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |
| `crates/prism-sensors/specs/claroty.sensor.toml` (alerts table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |
| `crates/prism-sensors/specs/cyberint.sensor.toml` (alerts table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |

Verify exact file paths and table names at dispatch — the TOMLs are the authoritative
source. If additional tables in any TOML also declare `ocsf_class = "security_finding"`,
those must be updated in the same PR.

### 2. Update `crates/prism-ocsf/src/class_selector.rs`

**Option A is selected (D-989 PO decision, Wave-5 Phase-A PO burst 2026-06-03; codified in BC-2.02.012 v1.4, now v1.5).** OQ-1 is CLOSED.

Implement `select_by_class_name` per BC-2.02.012 v1.5 mapping table:

```rust
pub fn select_by_class_name(class_name: &str) -> Option<u32> {
    match class_name {
        "detection_finding"    => Some(2004),  // PRIMARY — OCSF v1.1 canonical
        "security_finding"     => {             // Transitional alias; WARN per BC-2.02.012 v1.5
            tracing::warn!(
                event_type = "ocsf.deprecated_class_alias",
                class_name = "security_finding",
                resolved_class_uid = 2004,
                "sensor TOML uses deprecated ocsf_class value 'security_finding'; \
                 update to 'detection_finding'"
            );
            Some(2004)
        }
        "incident_finding"     => Some(2005),
        "vulnerability_finding"=> Some(2002),
        "device"               => Some(5001),
        "audit_activity"       => Some(3001),
        _                      => None,
    }
}
```

Note: `"security_finding"` maps to 2004 (NOT 2001). External TOML specs not under Prism
control may still use the old string value — Option A keeps them working with a WARN.
The `event_type = "ocsf.deprecated_class_alias"` emission MUST be registered in the
BC-2.16.002 Structured Event Catalog (SAP-1 probe — same PR).

### 3. BC-2.02.012 is already amended (v1.5)

BC-2.02.012 was amended to v1.4 in the Wave-5 Phase-A PO burst (2026-06-03), then advanced to
v1.5 (status-sync draft→active; no semantic change). OQ-2 is CLOSED.
The implementer must READ BC-2.02.012 v1.5 to confirm implementation semantics. Do NOT
amend BC-2.02.012 again in this story's PR — it is PO-owned and already at the required version.

### 4. Update S-DEMO-001 conformance test fixture (if applicable)

If S-DEMO-001's conformance test fixture at
`crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs`
asserts `ocsf_class_uid = 2001` (or `class_uid == 2001`) for any of the 4 sensors,
update the assertion to 2004.
Search for `2001` in test fixtures after the TOML change to find any stale assertions.

---

## Acceptance Criteria

### AC-001: All 4 production sensor TOMLs declare `ocsf_class = "detection_finding"`
After this PR merges, all 4 production sensor TOML specs in `crates/prism-sensors/specs/`
declare `ocsf_class = "detection_finding"` for all alert/detection tables. The grep audit
`rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/` returns zero results.
No production TOML in the repo declares `ocsf_class = "security_finding"` after this PR merges.
(traces to BC-2.02.012 v1.5 postcondition INV-PRODUCTION-TOML-NO-SECURITY-FINDING and TV-BC-2.02.012-009)

Red Gate test: `test_BC_2_02_012_no_production_toml_uses_security_finding`

### AC-002: `select_by_class_name("detection_finding")` returns 2004; no WARN emitted
`prism_ocsf::class_selector::select_by_class_name("detection_finding")` returns `Some(2004)`.
No `ocsf.deprecated_class_alias` WARN is emitted for this input (it is the canonical PRIMARY entry).
This matches TV-BC-2.02.012-007.
(traces to BC-2.02.012 v1.5 postcondition `select_by_class_name("detection_finding")` postcondition
and BC-2.01.013 v1.14 `select_by_class_name` specification — OCSF Conformance Clause semantically unchanged from v1.10)

Red Gate test: `test_BC_2_02_012_select_by_class_name_detection_finding_returns_2004_no_warn`

### AC-003: `select_by_class_name("security_finding")` returns 2004 (NOT 2001) with deprecation WARN
`prism_ocsf::class_selector::select_by_class_name("security_finding")` returns `Some(2004)` — the
transitional alias maps to Detection Finding (2004), NOT Security Finding (2001, deprecated).
The implementation emits exactly:
`tracing::warn!(event_type = "ocsf.deprecated_class_alias", class_name = "security_finding", resolved_class_uid = 2004, "sensor TOML uses deprecated ocsf_class value 'security_finding'; update to 'detection_finding'")`.
This matches TV-BC-2.02.012-008 and BC-2.01.013 v1.14 TV-BC-2.01.013-005 (class_uid == 2004,
NOT 2001; WARN emitted; `event_type = "ocsf.deprecated_class_alias"` is catalogued in BC-2.16.002
Structured Event Catalog per SAP-1).
(traces to BC-2.02.012 v1.5 `select_by_class_name()` path — transitional alias clause;
and BC-2.01.013 v1.14 SpecDrivenSensorAdapter OCSF Conformance Clause item 2 — semantically unchanged from v1.10)

Red Gate test: `test_BC_2_02_012_select_by_class_name_security_finding_returns_2004_with_warn`

### AC-004: `select()` path MUST NOT return class_uid 2001 for any record-type token
The `EventClassSelector::select(sensor_id, record_type)` function — the record-type-token path —
returns class_uid 2001 for NO token. INV-NO-2001-SELECT-PATH is enforced: any token that
previously mapped to 2001 now maps to 2004 (or another current OCSF class as appropriate).
No new record-type token is introduced that maps to 2001.
(traces to BC-2.02.012 v1.5 `select()` path invariant INV-NO-2001-SELECT-PATH)

Red Gate test: `test_BC_2_02_012_select_path_no_token_returns_2001`

### AC-005: No test in the workspace asserts ocsf_class_uid = 2001 for any of the 4 sensors
After this PR merges, no test asserts `class_uid == 2001` for any production sensor record.
`rg '2001' crates/ tests/` grep-and-review pass confirms all stale `2001` assertions are
updated to `2004`.
(traces to BC-2.02.012 v1.5 postcondition — production tests use current class UID)

Red Gate test: `test_BC_2_02_012_no_stale_2001_assertions_in_workspace` (manual grep audit
at dispatch; CI-enforced by the AC-002/AC-003/AC-004 unit tests failing if 2001 were returned)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `select_by_class_name` | `crates/prism-ocsf/src/class_selector.rs` (SS-16) | Pure |
| Production sensor TOMLs | `crates/prism-sensors/specs/*.sensor.toml` (SS-01) | Config (data) |
| Conformance test fixture | `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` | Test (effectful — spawns subprocess) |
| BC-2.02.012 | `.factory/specs/behavioral-contracts/BC-2.02.012.md` | Spec |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| `crates/prism-ocsf/src/class_selector.rs` (read + edit) | ~1,500 |
| 4 sensor TOML files (read + edit, `crates/prism-sensors/specs/`) | ~3,000 |
| BC-2.02.012 v1.5 (read only — no PO amendment needed) | ~2,000 |
| BC-2.01.013 v1.14 (read only) | ~3,000 |
| Unit test additions (5 Red Gate tests) | ~1,500 |
| `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` (read + conditional edit, `rg '2001'` audit) | ~1,000 |
| BC-2.16.002 (read + `ocsf.deprecated_class_alias` catalog row add) | ~2,000 |
| **Total** | **~18,000 tokens (~7% of 256K context)** |

Well within 30% context window budget. This is a P2 maintenance story.

---

## Tasks

1. **Read** `crates/prism-ocsf/src/class_selector.rs` — confirm current `select_by_class_name`
   arms and exact function signature.
2. **Read** BC-2.02.012 v1.5 — confirm the full mapping table, Option A semantics, and the
   `ocsf.deprecated_class_alias` WARN emission spec. Do NOT amend this BC (PO-owned, already at v1.5).
3. **Write Red Gate tests (stub phase — ALL must FAIL before implementation):**
   - `test_BC_2_02_012_no_production_toml_uses_security_finding` (AC-001)
   - `test_BC_2_02_012_select_by_class_name_detection_finding_returns_2004_no_warn` (AC-002)
   - `test_BC_2_02_012_select_by_class_name_security_finding_returns_2004_with_warn` (AC-003)
   - `test_BC_2_02_012_select_path_no_token_returns_2001` (AC-004)
   - Manual grep: `rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/` (AC-001 audit)
4. **Implement `select_by_class_name`** per §Scope task 2 code block (Option A, BC-2.02.012 v1.5).
   Emit `tracing::warn!(event_type = "ocsf.deprecated_class_alias", ...)` for `"security_finding"` input.
5. **Add BC-2.16.002 catalog row** for `ocsf.deprecated_class_alias` event (SAP-1 — same PR).
6. **Grep production TOML directory** — `rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/` — update
   ALL occurrences to `"detection_finding"` in the same PR.
7. **Search** `rg '2001' crates/ tests/` — identify and update stale `2001` assertions (AC-005).
8. **Run** `just iter prism-ocsf` — all 5 Red Gate tests GREEN. `just check` — pre-push gate.

---

## Previous Story Intelligence

**S-DEMO-001 (merged, PR #166):** Established `SpecDrivenSensorAdapter` and boot step 9A.
The sensor TOMLs updated here are the same TOMLs that S-DEMO-001 verified load correctly —
the OCSF class field was present and loaded but pointed to the deprecated 2001 value.
This story changes that field value only; the loading machinery is unchanged.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `"security_finding"` MUST NOT appear in any production TOML after this story | BC-2.02.012 v1.5 INV-PRODUCTION-TOML-NO-SECURITY-FINDING | `rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/` must return 0 results |
| `select_by_class_name("security_finding")` MUST return 2004 (NOT 2001) | BC-2.02.012 v1.5 transitional alias clause | AC-003 Red Gate test |
| `select_by_class_name("security_finding")` MUST emit `event_type = "ocsf.deprecated_class_alias"` WARN | BC-2.02.012 v1.5 deprecation WARN clause | AC-003 Red Gate test captures tracing output |
| `ocsf.deprecated_class_alias` event_type MUST be in BC-2.16.002 Structured Event Catalog | SAP-1 + PG-LP11-001 | Adversary SAP-1 sweep on every pass |
| `select_by_class_name` changes MUST have unit tests | BC-2.02.012 v1.5 — AC-002 + AC-003 | CI: `just iter prism-ocsf` |
| BC-2.02.012 is ALREADY at v1.5 — do NOT amend it again in this story's PR | PO-owns-BCs rule (Agent Routing Table) | Implementer reads BC; does not edit it |
| No new crate dependencies | Story scope | Cargo.toml unchanged |

**Forbidden Dependencies:** No new crate dependencies. This story is pure config + mapping + test changes.

---

## Library & Framework Requirements

No new dependencies. All changes are within existing `prism-ocsf`, sensor TOML config files, and the `prism-bin` conformance test fixture.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `crates/prism-sensors/specs/armis.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `crates/prism-ocsf/src/class_selector.rs` | Modify | Add/confirm `"detection_finding"` → 2004 mapping |
| `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` | Modify if stale | Update any `class_uid == 2001` assertion → 2004 (AC-005 / EC-003) |
| `.factory/specs/behavioral-contracts/BC-2.02.012-ocsf-event-class-selection.md` | Read only | Already at v1.5 — do NOT amend (PO-owned) |

---

## Gating / Open Questions at Dispatch

| OQ | Question | Owner | Resolution |
|----|----------|-------|-----------|
| OQ-1 | Option A (keep security_finding alias) or Option B (remove)? | Architect + PO | **CLOSED** — Option A selected per D-989 PO decision (Wave-5 Phase-A PO burst 2026-06-03; BC-2.02.012 v1.4 Changelog). |
| OQ-2 | Has BC-2.02.012 been amended per §Scope task 3? | Product Owner | **CLOSED** — BC-2.02.012 amended to v1.4 in Wave-5 Phase-A PO burst 2026-06-03. |
| OQ-3 | Are there additional tables in any sensor TOML that also declare `ocsf_class = "security_finding"`? | Implementer (grep at dispatch) | OPEN — `rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/` at start of story; all occurrences in that spec directory must be updated in the same PR. |

**Spec-First Gate S-7.01 note:** `behavioral_contracts` now contains BC-2.02.012 v1.5 and
BC-2.01.013 v1.14 (both active). OQ-1 and OQ-2 are CLOSED. This story MAY transition to
`ready` once OQ-3 is resolved at dispatch and all AC↔BC bidirectional traces are verified.
AC-001 through AC-005 must each cite a specific BC clause before `status: ready`.
(BC-2.01.013 bumped to v1.14 by S-DEMO-QUERY-PUSHDOWN-001-v2-bc-respec; OCSF Conformance Clause relevant to this story is semantically unchanged from v1.10.)

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | External TOML spec (user-supplied) still uses `"security_finding"` | If Option A chosen: still maps to 2004 with a deprecation warning; no parse failure |
| EC-002 | `select_by_class_name("security_finding")` caller exists outside the 4 production TOMLs | Grep audit required at dispatch; update all callers in the same PR |
| EC-003 | S-DEMO-001 conformance test asserts class_uid 2001 | Update fixture to assert 2004; test MUST pass with new mapping |

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.6 | 2026-06-06 | story-writer | OBS-1+OBS-2 LOCAL-pass-2 fix: dropped stale .prism/specs OQ-3 grep clause; corrected crates_touched (prism-spec-engine→prism-bin) + synced §FSR/§Architecture-Mapping/§Token-Budget; corrected §Scope task-4 fixture path to crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs; corrected subsystem anchor comment; updated §Library & Framework Requirements blurb. |
| 1.5 | 2026-06-06 | story-writer | OBS-1 + OBS-2 LOCAL-pass-1 fix: (1) Advanced all BC-2.02.012 cite pins v1.4→v1.5 — confirmed v1.5 is a status-sync only (draft→active; no semantic change per BC-2.02.012 v1.5 Changelog entry "Wave-5-Phase-B-gate-F-002"). Sites updated: frontmatter behavioral_contracts comment (×1); # BC status annotation (×1); §Behavioral Contracts table Version column (×1); AC-001/AC-002/AC-003/AC-004/AC-005 trace lines (×5); §Architecture Compliance Rules table source column (×6); §Scope task 2 code comment + mapping table note (×2); §Scope §3 header + body (×2); §Tasks task 2 (×1). (2) Corrected `.prism/specs/sensors/` → `crates/prism-sensors/specs/` path anchors in §Scope TOML table (×4), §Architecture Mapping table (×1), §File Structure Requirements TOML rows (×4), §Tasks task 6 grep command (×1), Token Budget TOML row (×1). BC-2.02.012 body NOT edited (PO-owned). STORY-INDEX.md NOT edited (state-manager owns). |
| 1.4 | 2026-06-05 | story-writer | Cite-pin sweep (POLICY 29 — BC-2.01.013 v1.13→v1.14): S-DEMO-QUERY-PUSHDOWN-001 v2.1 armis-aql-full-wiring burst bumped BC-2.01.013 to v1.14 (Armis AQL-clause augmentation IN scope per human directive 2026-06-05). OCSF Conformance Clause semantically unchanged from v1.10; v1.14 does not affect this story's scope. Updated 8 sites: frontmatter behavioral_contracts comment (×1); # BC status: annotation (×1); §Deferral Rationale items 1+3 (×2); §Behavioral Contracts table version column (×1); AC-002 trace line (×1); AC-003 trace line (×1); Token Budget table (×1). Story version 1.3→1.4. |
| 1.3 | 2026-06-05 | product-owner | Cite-pin sweep (POLICY 29 sibling-sweep — BC-2.01.013 v1.12→v1.13). Updated BC-2.01.013 version pins at: frontmatter `behavioral_contracts` comment (×1); `# BC status:` frontmatter annotation (×1); §Deferral Rationale items 1+3 (×2); §Behavioral Contracts table version column (×1); AC-002 trace line (×1); AC-003 trace line (×1); Token Budget table (×1); §Gating S-7.01 note (×2). All pins updated from v1.12 to v1.13. No AC semantics changed — OCSF Conformance Clause unchanged in v1.13. Story version 1.2→1.3. |
| 1.2 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; BC-2.02.012 v1.5 active (PO D-989) + BC-2.01.013 v1.11 active — OCSF-CLASS-MIGRATION-001 "both active" annotation now accurate; depends_on S-DEMO-001 (merged PR #166) SATISFIED; S-7.01 gate CLEARED. |
| 1.1 | 2026-06-03 | story-writer | Wave-5 Phase-A BC-array propagation burst (D-989). PO authored BC-2.02.012 v1.4 (Option A selected, OQ-1 CLOSED) and BC-2.01.013 v1.10 (transitional alias → 2004, TV-005 corrected). Propagated into story: (1) `behavioral_contracts` frontmatter updated with v1.4/v1.10 commentary. (2) Added §Behavioral Contracts table with BC roles. (3) ACs rewritten from BC postconditions: AC-001 → INV-PRODUCTION-TOML-NO-SECURITY-FINDING; AC-002 → `"detection_finding"` → 2004 no-WARN; AC-003 → `"security_finding"` → 2004 with `ocsf.deprecated_class_alias` WARN (Option A behavior per BC-2.02.012 v1.4); AC-004 → INV-NO-2001-SELECT-PATH; AC-005 → no stale 2001 assertions. (4) §Scope task 2 updated: Option A DECIDED, decision reference recorded. (5) OQ-1/OQ-2 CLOSED in §Gating. (6) §Architecture Compliance Rules updated for v1.4 semantics. (7) Token budget updated. (8) Tasks expanded with 5 Red Gate test names. Version bump 1.0 → 1.1. |
| 1.0 | 2026-06-01 | story-writer | Initial stub created from OBS-2 (S-DEMO-001 PR #166) architect adjudication D-925. Non-blocking deferral per Canonical Principle Rule 3. |
