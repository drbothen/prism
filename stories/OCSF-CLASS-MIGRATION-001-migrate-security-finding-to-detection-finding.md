---
document_type: story
story_id: OCSF-CLASS-MIGRATION-001
title: "prism-ocsf + sensor TOMLs: Migrate ocsf_class security_finding → detection_finding (OCSF v1.1 deprecation)"
wave: 5
epic_id: E-DEMO
priority: P2
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-06-01T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns the production sensor TOML specs that declare ocsf_class;
#     the 4 TOML edits live in prism-spec-engine's sensor config directory.
#   SS-16 (Spec Engine) owns crates/prism-ocsf/src/class_selector.rs where
#     select_by_class_name maps string names to OCSF class_uid integers.
target_module: prism-ocsf
crates_touched: [prism-ocsf, prism-spec-engine]
behavioral_contracts:
  - BC-2.02.012   # OCSF Class Selection — pending amendment to acknowledge transitional alias
  - BC-2.01.013   # DataSource Trait — v1.9 transitional note: "use security_finding until migration"
# BC status: pending PO confirmation at dispatch. BC-2.02.012 requires amendment (see §Gating).
# The two BCs above are listed provisionally from architect adjudication D-925; PO must confirm
# at dispatch that BC-2.02.012 has been updated before this story can transition to ready.
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

1. **D-925 intentional transitional decision.** BC-2.01.013 v1.9 explicitly records:
   > "real sensors use this name — use until migration to detection_finding"
   The current `ocsf_class = "security_finding"` in 4 production sensor TOMLs is intentional,
   not an oversight.

2. **Current behavior is valid, not broken.** `select_by_class_name("security_finding")` maps
   to OCSF class_uid 2001. OCSF class 2001 was deprecated in OCSF v1.1.0 in favor of 2004
   (Detection Finding). The data produced is structurally valid OCSF; it uses the deprecated
   class UID, but downstream consumers that handle 2001 continue to work. No runtime crash,
   no data loss.

3. **BC-2.02.012 ↔ BC-2.01.013 v1.9 contradiction.** BC-2.02.012 contains a "no deprecated
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

## Scope

### 1. Update 4 production sensor TOML specs

| File | Field | Old value | New value |
|------|-------|-----------|-----------|
| `.prism/specs/sensors/crowdstrike.sensor.toml` (detections table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |
| `.prism/specs/sensors/armis.sensor.toml` (alerts table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |
| `.prism/specs/sensors/claroty.sensor.toml` (alerts table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |
| `.prism/specs/sensors/cyberint.sensor.toml` (alerts table) | `ocsf_class` | `"security_finding"` | `"detection_finding"` |

Verify exact file paths and table names at dispatch — the TOMLs are the authoritative
source. If additional tables in any TOML also declare `ocsf_class = "security_finding"`,
those must be updated in the same PR.

### 2. Update `crates/prism-ocsf/src/class_selector.rs`

Two possible sub-decisions (architect or PO to choose at dispatch):

**Option A — Add `detection_finding` as primary, keep `security_finding` as transitional alias:**

```rust
"detection_finding" => Some(2004),
"security_finding"  => Some(2004),  // transitional alias; log deprecation warning
```

**Option B — Add `detection_finding` as primary, remove `security_finding` mapping:**

```rust
"detection_finding" => Some(2004),
// "security_finding" removed — no longer produced by Prism
```

Option A is safer for external TOML specs that Prism does not control; Option B is
cleaner if all in-tree TOMLs are migrated in this PR. Record the decision as an ADR-XXX
amendment or inline in the commit message.

Regardless of option chosen, verify that `"detection_finding"` → 2004 is correctly wired.

### 3. Amend BC-2.02.012

Scope the "no deprecated class_uid 2001" invariant to the `select()` path only and add a
postcondition: "`select_by_class_name('detection_finding')` returns 2004". Document that the
transitional 2001 path (via `select_by_class_name("security_finding")`) was valid per D-925
until this story migrated all callers. Version-bump BC-2.02.012 to v1.1.

### 4. Update S-DEMO-001 conformance test fixture (if applicable)

If S-DEMO-001's conformance test fixture (likely in `tests/` or `prism-spec-engine/tests/`)
asserts `ocsf_class_uid = 2001` for any of the 4 sensors, update the assertion to 2004.
Search for `2001` in test fixtures after the TOML change to find any stale assertions.

---

## Acceptance Criteria (stub — expand at dispatch)

**AC-001:** All 4 production sensor TOMLs declare `ocsf_class = "detection_finding"`. No
production TOML in the repo declares `ocsf_class = "security_finding"` after this PR merges.
(traces to BC-2.02.012 postcondition — canonical class name is detection_finding)

**AC-002:** `prism_ocsf::class_selector::select_by_class_name("detection_finding")` returns
`Some(2004)`. The mapping is covered by a unit test.
(traces to BC-2.02.012 postcondition — select_by_class_name maps detection_finding to 2004)

**AC-003:** BC-2.02.012 is amended to version v1.1 with the `select_by_class_name` postcondition
added and the deprecated-2001 invariant scoped to the `select()` path.
(traces to BC-2.02.012 — BC reflects migrated behavior)

**AC-004:** No test in the workspace asserts `ocsf_class_uid = 2001` for any of the 4 sensors
after this PR merges.
(traces to BC-2.02.012 postcondition — all in-tree tests use current class UID)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `select_by_class_name` | `crates/prism-ocsf/src/class_selector.rs` (SS-16) | Pure |
| Production sensor TOMLs | `.prism/specs/sensors/*.sensor.toml` (SS-01) | Config (data) |
| BC-2.02.012 | `.factory/specs/behavioral-contracts/BC-2.02.012.md` | Spec |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This stub story | ~800 |
| `class_selector.rs` (read + edit) | ~500 |
| 4 sensor TOML files (read + edit) | ~2,000 |
| BC-2.02.012 (read + amend) | ~800 |
| Unit test additions | ~500 |
| S-DEMO-001 fixture check (search + conditional edit) | ~300 |
| Total | ~4,900 |

Well within 30% context window budget. This is a P2 maintenance story.

---

## Tasks (stub — expand at dispatch)

1. Read `crates/prism-ocsf/src/class_selector.rs` — confirm current `select_by_class_name` arms.
2. Choose Option A or Option B (architect/PO decision at dispatch) and update the mapping.
3. Read each of the 4 sensor TOMLs, update `ocsf_class` field.
4. Search `rg '2001' crates/ tests/` — update stale assertions.
5. Read BC-2.02.012 — amend to v1.1 per §Scope task 3.
6. Write unit test: `assert_eq!(select_by_class_name("detection_finding"), Some(2004))`.
7. Run `just iter prism-ocsf` + `just check` (pre-push gate).

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
| OCSF class_uid 2001 MUST NOT appear in any production TOML after this story | BC-2.02.012 v1.1 (post-amend) | `rg '"security_finding"' .prism/specs/` must return 0 results |
| `select_by_class_name` changes MUST have a unit test | BC-2.02.012 — AC-002 | CI: `just iter prism-ocsf` |
| BC-2.02.012 version MUST be bumped when invariant scope changes | VSDD spec versioning policy | Adversary pass verification |

**Forbidden Dependencies:** No new crate dependencies. This story is pure config + mapping changes.

---

## Library & Framework Requirements

No new dependencies. All changes are within existing `prism-ocsf` and sensor TOML config files.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `.prism/specs/sensors/crowdstrike.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `.prism/specs/sensors/armis.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `.prism/specs/sensors/claroty.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `.prism/specs/sensors/cyberint.sensor.toml` | Modify | `ocsf_class` → `"detection_finding"` |
| `crates/prism-ocsf/src/class_selector.rs` | Modify | Add/confirm `"detection_finding"` → 2004 mapping |
| `.factory/specs/behavioral-contracts/BC-2.02.012.md` | Modify | Amend to v1.1 per §Scope task 3 |
| Test fixtures (TBD search) | Modify if found | Remove stale `2001` assertions |

---

## Gating / Open Questions at Dispatch

| OQ | Question | Owner | Resolution |
|----|----------|-------|-----------|
| OQ-1 | Option A (keep security_finding alias) or Option B (remove)? | Architect + PO | Must decide before story transitions to `ready` |
| OQ-2 | Has BC-2.02.012 been amended per §Scope task 3? | Product Owner | Required before `status: ready` (Spec-First Gate S-7.01) |
| OQ-3 | Are there additional tables in any sensor TOML that also declare `ocsf_class = "security_finding"`? | Implementer (grep at dispatch) | `rg 'ocsf_class.*security_finding' .prism/` at start of story |

**Spec-First Gate S-7.01 note:** `behavioral_contracts` contains BC-2.02.012 and BC-2.01.013
provisionally. These BCs must be confirmed non-empty and valid at the `draft → ready` transition.
Until OQ-2 is resolved (BC-2.02.012 amended), this story MUST remain `status: draft`.

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
| 1.0 | 2026-06-01 | story-writer | Initial stub created from OBS-2 (S-DEMO-001 PR #166) architect adjudication D-925. Non-blocking deferral per Canonical Principle Rule 3. |
