---
document_type: story
story_id: S-OCSF-FIDELITY-ARMIS-001
title: "Armis OCSF Schema Validation and Correction (ADR-058 §K Methodology)"
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
  - crates/prism-sensors/specs/armis.sensor.toml
  - crates/prism-ocsf/src/class_selector.rs
  - crates/prism-ocsf/ocsf-schema/1.7.0/schema.json
input-hash: "f5f8bdb"
traces_to: .factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md
crates_touched:
  - prism-sensors
  - prism-ocsf
depends_on: []
# depends_on note: S-ADR058-OCSF-ROUTING-001 is the methodology precedent reference
# (ADR-058 §K) AND shares the `("armis", "audit_log")` class_selector.rs arm in scope
# (see SHARED ARM COORDINATION below). Execution dependency: if S-ADR058-OCSF-ROUTING-001
# has NOT yet landed, the `("armis", "audit_log")` arm fix must be included in this story.
# If S-ADR058-OCSF-ROUTING-001 HAS landed, this story validates the fix and applies any
# remaining armis.sensor.toml corrections. At materialization, the story-writer must
# reconcile against whatever S-ADR058-OCSF-ROUTING-001 has landed.
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

> **Execute:** `/vsdd-factory:deliver-story S-OCSF-FIDELITY-ARMIS-001`

# S-OCSF-FIDELITY-ARMIS-001: Armis OCSF Schema Validation and Correction

## Authority

**ADR-058 v2.4 §K** is the authority for the dual-validation methodology applied here.
§K1 Methodology defines the procedure: enumerate all `ocsf_class` and `ocsf_field`
declarations, validate each against the committed OCSF v1.7.0 schema at
`crates/prism-ocsf/ocsf-schema/1.7.0/schema.json`, cross-validate against the official
OCSF v1.7.0 schema, and reconcile all discrepancies.

`.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`
(see §K1 Methodology, §K2 OCSF Class Verdicts, §K3 ocsf_field Path Verdicts, §K4 Finding Summary,
§K5 Divergence 3, §I5 KF-01 code obligation)

**Claroty precedent (S-ADR058-OCSF-ROUTING-001)** is the methodology exemplar and shares
a class_selector.rs arm with this story (see SHARED ARM COORDINATION below).

---

## SHARED ARM COORDINATION (CRITICAL — READ BEFORE MATERIALIZATION)

**The `("armis", "audit_log")` arm in `class_selector.rs` `select()` is already
implicated by S-ADR058-OCSF-ROUTING-001 via ADR-058 §I5 + §K5 Divergence 3.**

ADR-058 §I5 states (KF-01 code obligation, v2.4):

> Also update the `("armis", "audit_log")` arm in `select()` (same semantic defect —
> TD-VSDD-097 dimension 1 sibling sweep). `account_change` (3001) lacks `comment`;
> `entity_management` (3004) has it; Claroty and Armis audit logs record entity changes,
> not IAM account changes.

ADR-058 §K5 Divergence 3 sibling note confirms:

> `class_selector.rs` `select()` also maps `("armis", "audit_log") =>
> Ok(CLASS_UID_ACCOUNT_CHANGE)`. Armis audit logs carry the same entity-management
> semantics. The same code fix should update this arm.

**Materialization reconciliation rule:**

If S-ADR058-OCSF-ROUTING-001 has NOT yet landed:
- This story MUST include the `("armis", "audit_log")` arm fix in its scope (change
  `CLASS_UID_ACCOUNT_CHANGE` → `CLASS_UID_ENTITY_MANAGEMENT = 3004`).

If S-ADR058-OCSF-ROUTING-001 HAS landed:
- This story validates the `("armis", "audit_log")` arm was correctly updated (the
  claroty story's §I5 obligation covers it), and does NOT re-fix it.
- The story-writer must grep `class_selector.rs` at materialization and confirm the
  arm was updated before removing it from scope.

**Do not leave this coordination unresolved at materialization.** A double-fix
introduces duplicate constant definitions; a missed fix leaves data loss on the Armis
`note → comment` TOML mapping (same as Claroty KF-01 — `account_change` 3001 lacks
`comment`; `entity_management` 3004 has it).

---

## Narrative

- **As a** MSSP analyst consuming OCSF-normalized Armis data via Prism
- **I want** every `ocsf_class` and `ocsf_field` declaration in `armis.sensor.toml`
  to be valid against OCSF v1.7.0 and the corresponding `class_selector.rs` arms to route
  to the correct class UIDs
- **So that** OCSF consumers receive semantically correct event envelopes rather than
  silently corrupted or misclassified records

---

## Scope

Apply the ADR-058 §K dual-validation methodology to `crates/prism-sensors/specs/armis.sensor.toml`
and the corresponding `class_selector.rs` routing arms — subject to SHARED ARM
COORDINATION above.

**Known pre-validation finding (from Claroty precedent):**

- `ocsf_class = "device"` on the `devices` table is structurally WRONG — `device` is an
  OCSF v1.7.0 object, not a class (same defect as Claroty KF-02). Correct class:
  `inventory_info` (class_uid 5001). This fix requires a `class_selector.rs` arm update
  for the `("armis", "devices")` routing arm.

**Validation surface (current TOML):**

| Table | `ocsf_class` (current) | Pre-validation status |
|-------|------------------------|-----------------------|
| `devices` | `device` | ocsf_class WRONG — object not class (same as Claroty KF-02) |
| `alerts` | `detection_finding` | ocsf_class likely VALID; ocsf_field paths TBD |

All 23 `ocsf_field` declarations require per-column path-validation at materialization
per the §K3 pattern from ADR-058.

Note: Armis has no `audit_log` table in the current TOML (`armis.sensor.toml` declares
only `devices` and `alerts`). The `("armis", "audit_log")` arm in `class_selector.rs`
pre-dates the TOML spec and is residual code — its correction is coordinated via
S-ADR058-OCSF-ROUTING-001 per §I5 (see SHARED ARM COORDINATION above).

---

## Acceptance Criteria

> N/A at stub stage — authored at story-materialization time.
> Each AC will trace to a BC-S.SS.NNN clause per Spec-First Gate S-7.01.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Armis sensor spec | `crates/prism-sensors/specs/armis.sensor.toml` | Effectful (sensor config) |
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
| `armis.sensor.toml` | effectful-config | Parsed at sensor-boot; drives live API calls |
| `class_selector.rs` | pure-core | Deterministic lookup; no I/O |

---

## Token Budget Estimate (MANDATORY)

> Estimate completed at materialization when full AC/RG scope is known.

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~4 000 |
| `armis.sensor.toml` | ~4 000 |
| `class_selector.rs` | ~2 000 |
| OCSF schema (relevant sections) | ~5 000 |
| Test files (TBD at materialization) | TBD |
| **Total** | **TBD** |
| Agent context window | 200 K (Sonnet) |
| **Budget usage** | **< 10% pre-test** |

---

## Tasks (MANDATORY)

> Tasks expanded at materialization. Pre-materialization phase:

- [ ] T-PREP-01: Reconcile S-ADR058-OCSF-ROUTING-001 landing status re: `("armis", "audit_log")` arm (see SHARED ARM COORDINATION)
- [ ] T-PREP-02: Run §K1 methodology against `armis.sensor.toml`; produce §K2/§K3/§K4 finding tables
- [ ] T-PREP-03: PO authors BCs for all correction obligations
- [ ] T-PREP-04: Story-writer expands stub to full ACs + Red Gate list (SAC-1 format)
- [ ] T-PREP-05: Transition status draft → ready (requires non-empty `behavioral_contracts`)
- [ ] T-IMPL-01: Write failing Red Gate tests (test-writer)
- [ ] T-IMPL-02: Correct `armis.sensor.toml` TOML entries per §K findings
- [ ] T-IMPL-03: Correct `class_selector.rs` Armis routing arms per §K findings (scope: `("armis", "devices")` arm plus `("armis", "audit_log")` arm IFF S-ADR058-OCSF-ROUTING-001 has not landed)
- [ ] T-IMPL-04: Run `just check`; verify all Red Gate tests pass

---

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-ADR058-OCSF-ROUTING-001 | ADR-058 §K dual-validation methodology for Claroty; found 12/35 defects; §I5 mandates `("armis", "audit_log")` arm be updated in the same commit as the Claroty KF-01 fix | §K1-§K4 finding table format; `inventory_info` for device tables; `finding_info.*` not `finding.*` under detection_finding | `device` is an OCSF object not a class; the `("armis", "audit_log")` arm in `class_selector.rs` maps to `CLASS_UID_ACCOUNT_CHANGE` (3001) which lacks `comment` — entity_management (3004) is correct; TD-VSDD-097 dimension 1 sibling sweep governs this shared arm |
| S-WAVE-A-ARMIS-SPEC-001 | Armis device columns + `device_cves_first` source_path fix; SAP-2 DTU parity methodology | Per-column DTU parity verification before adding any TOML column | SAP-2 Rule 6: the wire-emission site (route handler JSON) is authoritative over the struct definition |

---

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Every `ocsf_class` must be a valid class key in OCSF v1.7.0 (not an object or absent) | ADR-058 v2.4 §K1 | Cross-check against `crates/prism-ocsf/ocsf-schema/1.7.0/schema.json` class enumeration |
| Every `ocsf_field` path must resolve segment-by-segment in the schema | ADR-058 v2.4 §K1 | Per-column validation per §K3 pattern |
| `class_selector.rs` `select()` Armis arms must map to the same class_uid as the TOML `ocsf_class` | ADR-058 v2.4 §K2/§I5 | Grep `select_by_class_name` and `select()` arms; verify mapping |
| Reserved OCSF computed fields must not be overwritten with vendor values | ADR-058 v2.4 §K3 KF-08..KF-11 precedent | Remove `ocsf_field` on columns targeting reserved computed attrs |
| Shared `("armis", "audit_log")` arm reconciliation required | ADR-058 v2.4 §I5 + SHARED ARM COORDINATION above | Check git log for S-ADR058-OCSF-ROUTING-001 landing before scoping this fix |

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
| `crates/prism-sensors/specs/armis.sensor.toml` | modify | Correct `ocsf_class` and `ocsf_field` defects per §K findings |
| `crates/prism-ocsf/src/class_selector.rs` | modify | Correct Armis routing arms to match corrected `ocsf_class` values; reconcile `("armis", "audit_log")` arm per SHARED ARM COORDINATION |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-16 | story-writer | Initial draft stub — scope capture per human directive; full BC/AC/RG deferred to materialization; SHARED ARM COORDINATION section added for ("armis","audit_log") class_selector.rs arm dependency with S-ADR058-OCSF-ROUTING-001 §I5 |
