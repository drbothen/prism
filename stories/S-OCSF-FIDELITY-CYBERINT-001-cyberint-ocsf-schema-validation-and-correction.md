---
document_type: story
story_id: S-OCSF-FIDELITY-CYBERINT-001
title: "Cyberint OCSF Schema Validation and Correction (ADR-058 §K Methodology)"
level: ops
version: "0.1"
status: draft
producer: story-writer
timestamp: "2026-08-16T00:00:00Z"
phase: 3
wave: tbd
epic_id: EPIC-OCSF-FIDELITY
cycle: v3-brownfield
priority: P2
points: tbd
tdd_mode: strict
target_module: prism-sensors
subsystems:
  - SS-12
inputs:
  - crates/prism-sensors/specs/cyberint.sensor.toml
  - crates/prism-ocsf/src/class_selector.rs
  - crates/prism-ocsf/ocsf-schema/1.7.0/schema.json
input-hash: "34cbf84"
traces_to: .factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md
crates_touched:
  - prism-sensors
  - prism-ocsf
depends_on: []
# depends_on note: S-ADR058-OCSF-ROUTING-001 is the methodology precedent reference
# (ADR-058 §K) but is NOT a hard execution dependency — this story targets
# cyberint.sensor.toml, a different sensor TOML. No shared file; no ordering constraint.
blocks: []
behavioral_contracts: []
# BC status: pending PO authorship — full BC layer authored at story-materialization time.
# status MUST remain draft until behavioral_contracts is non-empty (Spec-First Gate S-7.01).
verification_properties: []
assumption_validations: []
risk_mitigations: []
estimated_days: tbd
modified: "2026-08-16"
---

> **STUB — draft.** Full acceptance criteria, Red Gate test list, and BC layer are
> authored at story-materialization time per the human directive (2026-08-16):
> "fix claroty now, create stories for the other sensors."

> **Execute:** `/vsdd-factory:deliver-story S-OCSF-FIDELITY-CYBERINT-001`

# S-OCSF-FIDELITY-CYBERINT-001: Cyberint OCSF Schema Validation and Correction

## Authority

**ADR-058 v2.4 §K** is the authority for the dual-validation methodology applied here.
§K1 Methodology defines the procedure: enumerate all `ocsf_class` and `ocsf_field`
declarations, validate each against the committed OCSF v1.7.0 schema at
`crates/prism-ocsf/ocsf-schema/1.7.0/schema.json`, cross-validate against the official
OCSF v1.7.0 schema, and reconcile all discrepancies.

`.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`
(see §K1 Methodology, §K2 OCSF Class Verdicts, §K3 ocsf_field Path Verdicts, §K4 Finding Summary)

**Claroty precedent (S-ADR058-OCSF-ROUTING-001)** is the methodology exemplar: 12 of 35
defects found across defect classes (a) `finding.*` → `finding_info.*` under detection_finding
2004; (b) invalid `ocsf_class` names; (c) reserved base-event metadata overwrite
(`class_name`, `type_name`, `category_name`, `count`) → envelope corruption;
(d) matching `class_selector.rs` routing arm defect; (e) SUBOPTIMAL field choices.

---

## Narrative

- **As a** MSSP analyst consuming OCSF-normalized Cyberint data via Prism
- **I want** every `ocsf_class` and `ocsf_field` declaration in `cyberint.sensor.toml`
  to be valid against OCSF v1.7.0 and the corresponding `class_selector.rs` arms to route
  to the correct class UIDs
- **So that** OCSF consumers receive semantically correct event envelopes rather than
  silently corrupted or misclassified records

---

## Scope

Apply the ADR-058 §K dual-validation methodology to `crates/prism-sensors/specs/cyberint.sensor.toml`
and the corresponding `class_selector.rs` routing arms.

**Validation surface (current TOML):**

| Table | `ocsf_class` (current) | Pre-validation status |
|-------|------------------------|-----------------------|
| `alerts` | `detection_finding` | ocsf_class likely VALID; ocsf_field paths TBD |
| `incidents` | `incident_finding` | validity TBD at materialization |

All 14 `ocsf_field` declarations require per-column path-validation at materialization
per the §K3 pattern from ADR-058. Known defect classes to probe: `finding.*` vs
`finding_info.*` prefix under `detection_finding` 2004 (same root as Claroty KF-03/KF-04/KF-07);
reserved computed field overwrites (same root as Claroty KF-08..KF-11).

---

## Acceptance Criteria

> N/A at stub stage — authored at story-materialization time.
> Each AC will trace to a BC-S.SS.NNN clause per Spec-First Gate S-7.01.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Cyberint sensor spec | `crates/prism-sensors/specs/cyberint.sensor.toml` | Effectful (sensor config) |
| OCSF class selector | `crates/prism-ocsf/src/class_selector.rs` | Pure-core (routing table) |
| OCSF schema | `crates/prism-ocsf/ocsf-schema/1.7.0/schema.json` | Pure-core (reference data) |

---

## Edge Cases

> N/A at stub stage — authored at materialization once §K findings are produced.

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | TBD — populated from §K findings at materialization | TBD |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `cyberint.sensor.toml` | effectful-config | Parsed at sensor-boot; drives live API calls |
| `class_selector.rs` | pure-core | Deterministic lookup; no I/O |

---

## Token Budget Estimate (MANDATORY)

> Estimate completed at materialization when full AC/RG scope is known.

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3 000 |
| `cyberint.sensor.toml` | ~3 000 |
| `class_selector.rs` | ~2 000 |
| OCSF schema (relevant sections) | ~5 000 |
| Test files (TBD at materialization) | TBD |
| **Total** | **TBD** |
| Agent context window | 200 K (Sonnet) |
| **Budget usage** | **< 10% pre-test** |

---

## Tasks (MANDATORY)

> Tasks expanded at materialization. Pre-materialization phase:

- [ ] T-PREP-01: Run §K1 methodology against `cyberint.sensor.toml`; produce §K2/§K3/§K4 finding tables
- [ ] T-PREP-02: PO authors BCs for all correction obligations
- [ ] T-PREP-03: Story-writer expands stubs to full ACs + Red Gate list (SAC-1 format)
- [ ] T-PREP-04: Transition status draft → ready (requires non-empty `behavioral_contracts`)
- [ ] T-IMPL-01: Write failing Red Gate tests (test-writer)
- [ ] T-IMPL-02: Correct `cyberint.sensor.toml` TOML entries per §K findings
- [ ] T-IMPL-03: Correct `class_selector.rs` Cyberint routing arms per §K findings
- [ ] T-IMPL-04: Run `just check`; verify all Red Gate tests pass

---

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-ADR058-OCSF-ROUTING-001 | ADR-058 §K dual-validation methodology for Claroty; found 12/35 defects | §K1-§K4 finding table format; `finding_info.*` not `finding.*` under detection_finding; reserved fields must not be overwritten | `detection_finding` 2004 uses `finding_info` (required attr), NOT bare `finding`; reserved computed fields (`class_name`, `type_name`, `count`, `category_name`) corrupt OCSF envelope when overwritten by vendor values |

---

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Every `ocsf_class` must be a valid class key in OCSF v1.7.0 (not an object or absent) | ADR-058 v2.4 §K1 | Cross-check against `crates/prism-ocsf/ocsf-schema/1.7.0/schema.json` class enumeration |
| Every `ocsf_field` path must resolve segment-by-segment in the schema | ADR-058 v2.4 §K1 | Per-column validation per §K3 pattern |
| `class_selector.rs` `select()` Cyberint arms must map to the same class_uid as the TOML `ocsf_class` | ADR-058 v2.4 §K2/§I5 | Grep `select_by_class_name` and `select()` arms; verify mapping |
| Reserved OCSF computed fields must not be overwritten with vendor values | ADR-058 v2.4 §K3 KF-08..KF-11 precedent | Remove `ocsf_field` on columns targeting reserved computed attrs |

---

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| OCSF schema | v1.7.0 (pinned at `crates/prism-ocsf/ocsf-schema/1.7.0/`) | Authoritative schema for validation; `OCSF_PINNED_VERSION = "1.7.0"` in `build.rs` |
| Rust stable | per `rust-toolchain.toml` | Build toolchain |

---

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-sensors/specs/cyberint.sensor.toml` | modify | Correct `ocsf_class` and `ocsf_field` defects per §K findings |
| `crates/prism-ocsf/src/class_selector.rs` | modify | Correct Cyberint routing arms to match corrected `ocsf_class` values |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-16 | story-writer | Initial draft stub — scope capture per human directive; full BC/AC/RG deferred to materialization |
