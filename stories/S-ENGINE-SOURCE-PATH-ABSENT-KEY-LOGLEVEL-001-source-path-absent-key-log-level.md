---
document_type: story
story_id: S-ENGINE-SOURCE-PATH-ABSENT-KEY-LOGLEVEL-001
title: "prism-spec-engine: distinguish KeyNotFound from ExtractionFailed in source_path extraction — debug log level for absent optional keys"
level: "L4"
wave: tbd
epic_id: E-SPEC-ENGINE-HARDENING
priority: P1
status: draft
# BC status: BC-2.16.002 active (promotes on PR merge per POL-14). Spec-crystallization
# (ACs, RGTs, holdout scenarios) deferred to pickup time.
# S-7.01: behavioral_contracts non-empty (BC-2.16.002); status=draft — not yet ready for dispatch.
version: "1.0"
producer: story-writer
timestamp: "2026-08-25T00:00:00Z"
modified: "2026-08-25"
phase: 3
cycle: v1.0.0-brownfield
inputs: []
input-hash: "[pending-recompute]"
# inputs list and input-hash finalized at spec-crystallization when BCs and ACs are authored.
traces_to: "BC-2.16.002"
points: 3
# 3 pts — targeted return-type change + log-level dispatch in two known functions; no new
# algorithm design needed.
estimated_days: 1
tdd_mode: strict
subsystems: [SS-16]
# Subsystem anchor justification (ARCH-INDEX Subsystem Registry):
#   SS-16 (Spec Engine) owns this story's scope because the change is entirely in
#   prism-spec-engine (pipeline.rs `extract_at_path`/`extract_with_tokens` return type;
#   column_mapping.rs `map_record` log-level dispatch). SS-16 is the canonical owner of
#   prism-spec-engine per ARCH-INDEX Subsystem Registry.
target_module: prism-spec-engine
crates_touched: [prism-spec-engine]
capabilities: []
behavioral_contracts:
  - BC-2.16.002
  # BC-2.16.002 — Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable
  # Interpolation. Canonical Structured Event Catalog governs all event_type emissions.
  # This story adds a catalog row for `column_source_path_key_absent` (debug level,
  # normal optional-field absence, no audit role). PO authors the catalog row at
  # spec-crystallization time, before TDD begins (SAP-1 requirement).
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios during remove-uncertainty
# pass (same touchpoint as spec-crystallization); stored under the holdout directory;
# test-writer and implementer MUST NOT read (contamination control).
# The story-level holdout gate (human-approved 2026-07-13) is BLOCKING before demo/push.
depends_on: []
# Dependency justification: engine-only change in prism-spec-engine; no delivery dependency
# on S-CLAROTY-VULNS-001 (concurrent); but avoid parallel merge on column_mapping.rs
# if both stories touch it — coordinate via branch strategy. No hard Cargo dep change.
blocks: []
acceptance_criteria_count: 3
risk: LOW
# Risk justification: targeted return-type change in two functions; function signatures
# are internal to prism-spec-engine (no pub API surface change). Regression tests across
# all sensor source_path columns before merge.
assumption_validations: []
risk_mitigations: []
---

# S-ENGINE-SOURCE-PATH-ABSENT-KEY-LOGLEVEL-001: Distinguish KeyNotFound from ExtractionFailed — Debug Log for Absent Optional source_path Keys

**STATUS: DRAFT STUB — Deferred from S-CLAROTY-VULNS-001 LOCAL cascade round-4 finding F-R4A-OBS-001 (architect-ruled out of story engine scope 2026-08-25). Spec-crystallization (ACs, RGTs, holdout scenarios) deferred to pickup time.**

---

## Origin

Deferred from S-CLAROTY-VULNS-001 LOCAL cascade round-4 finding F-R4A-OBS-001 (2026-08-25).
Architect ruled the engine change out of S-CLAROTY-VULNS-001's scope to preserve that story's
TOML-only scope discipline. Anchor for the ~1000-WARN/fetch log-severity concern: absent optional
keys (`source_path = "$.id"` in claroty_vulnerabilities) currently emit at WARN level, polluting
the structured log baseline during normal operation.

---

## Authority

**BC-2.16.002 §Postconditions (Canonical Structured Event Catalog)** governs all `event_type`
emission sites in prism-spec-engine. This story adds a new catalog row:

- `column_source_path_key_absent`: debug level, normal optional-field absence, no audit role,
  recurrence = per absent source_path field per row.

The product-owner authors this catalog row in BC-2.16.002 §Postconditions at spec-crystallization
time (before TDD begins), per PG-LP11-001 and SAP-1.

---

## Narrative

As a SOC operator monitoring prism-spec-engine logs,
I want absent optional source_path keys to log at DEBUG (not WARN) level,
so that normal operation of sensors with optional fields (e.g., `id` in claroty_vulnerabilities,
any future sensor with source_path-only columns) does not produce ~1000 WARN entries per fetch
and pollute the structured log baseline.

## Background

Currently, `extract_at_path` / `extract_with_tokens` in `pipeline.rs` return `Result<Value,
String>`, and both "key not found" and "extraction failed" are treated as one failure mode.
Both are logged at `warn!(event_type="column_source_path_extraction_failed", ...)`. For optional
source_path columns (such as `id` in claroty_vulnerabilities where the field is outside the
fields_enum), absent keys are expected and normal; logging them at WARN inflates the warning
baseline and degrades alert signal quality.

**Change:** Introduce `enum ExtractError { KeyNotFound(String), ExtractionFailed(String) }` in
`pipeline.rs`. Update `extract_at_path` / `extract_with_tokens` return type from
`Result<Value, String>` to `Result<Value, ExtractError>`. The `PathToken::Key` absent-pointer
arm returns `ExtractError::KeyNotFound`; all other failures return `ExtractError::ExtractionFailed`.
In `column_mapping.rs::map_record`: `KeyNotFound` → `tracing::debug!(event_type=
"column_source_path_key_absent", ...)`, `ExtractionFailed` → keep existing
`warn!(event_type="column_source_path_extraction_failed", ...)` behavior.

**Scope:** prism-spec-engine only (`pipeline.rs`, `column_mapping.rs`). Sensor-agnostic
(POL-36-compatible — all sensors with source_path optional columns benefit). No changes to
sensor TOMLs, DTU clones, or any other crate.

---

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | (current at pickup) | Canonical Structured Event Catalog — new `column_source_path_key_absent` catalog row required before TDD begins (PG-LP11-001) |

## Acceptance Criteria

ACs will be fully authored at spec-crystallization. Preliminary scope:

### AC-001: ExtractError enum with KeyNotFound and ExtractionFailed variants introduced in pipeline.rs (traces to BC-2.16.002 postconditions — Canonical Structured Event Catalog event_type emission completeness)

`pipeline.rs` introduces `enum ExtractError { KeyNotFound(String), ExtractionFailed(String) }`.
`extract_at_path` / `extract_with_tokens` return `Result<Value, ExtractError>`. The
`PathToken::Key` absent-pointer arm returns `ExtractError::KeyNotFound`; all other failure modes
return `ExtractError::ExtractionFailed`. No behavioral change for existing `ExtractionFailed`
callers.

### AC-002: KeyNotFound logs at debug; ExtractionFailed logs at warn (traces to BC-2.16.002 postconditions — Canonical Structured Event Catalog event_type field schema)

In `column_mapping.rs::map_record`, `ExtractError::KeyNotFound` produces
`tracing::debug!(event_type="column_source_path_key_absent", column_name=%col, source_path=%path,
...)`. `ExtractError::ExtractionFailed` retains existing
`tracing::warn!(event_type="column_source_path_extraction_failed", ...)` behavior. No WARN is
emitted for a missing optional key.

### AC-003: BC-2.16.002 Canonical Structured Event Catalog gains column_source_path_key_absent row (traces to BC-2.16.002 postconditions — Canonical Structured Event Catalog completeness invariant per SAP-1)

The product-owner adds `column_source_path_key_absent` to BC-2.16.002 §Postconditions Canonical
Structured Event Catalog before TDD begins: debug level, field schema (column_name, source_path,
sensor_id), no audit role, recurrence = per absent source_path field per row. This row is present
in BC-2.16.002 before the PR merges (PG-LP11-001 / SAP-1 P1 enforcement).

## Red Gate Tests

SAC-1 note: enumerated RG-001..RG-NNN list and BC-5.38.001 density check authored at
spec-crystallization when ACs are finalized and BCs are confirmed. Current `status: draft` —
enumeration deferred per SAC-1 (applicable before status→ready transition only).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `ExtractError` enum + `extract_at_path` return type | `crates/prism-spec-engine/src/pipeline.rs §extract_at_path` | Pure (return type change; no new I/O) |
| `map_record` log-level dispatch | `crates/prism-spec-engine/src/column_mapping.rs §map_record` | Effectful (emits tracing events) |
| BC-2.16.002 catalog row | `.factory/specs/behavioral-contracts/BC-2.16.002-*.md §Postconditions` | Spec artifact (PO-authored at crystallization) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; pipeline.rs,
  column_mapping.rs)
- BC-2.16.002 §Postconditions (Canonical Structured Event Catalog — source of truth for all
  event_type emissions in prism-spec-engine)

## Purity Classification

- **Pure functions:** `ExtractError` enum definition; `extract_at_path` return type change
  (pure extraction logic; no new I/O).
- **Effectful functions:** `map_record` (emits debug/warn tracing events based on error variant).

---

## Edge Cases

Edge cases confirmed at spec-crystallization. Preliminary candidates:

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | source_path key absent for a REQUIRED column | N/A by construction — REQUIRED columns do not use source_path |
| EC-002 | source_path expression fails for reasons other than absent key (malformed path, type mismatch) | `ExtractError::ExtractionFailed` → existing `warn!(event_type="column_source_path_extraction_failed", ...)` preserved |
| EC-003 | Multiple absent optional source_path columns in same row | One `debug!(event_type="column_source_path_key_absent", ...)` per absent field per row; no aggregation |

---

## Token Budget Estimate (MANDATORY)

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,000 |
| `crates/prism-spec-engine/src/pipeline.rs` (extract_at_path + PathToken sections) | ~4,000 |
| `crates/prism-spec-engine/src/column_mapping.rs` (map_record section) | ~2,000 |
| BC-2.16.002 §Postconditions (Canonical Structured Event Catalog section) | ~3,000 |
| BC files (1 BC) | ~500 |
| Test files (RGTs — enumerated at crystallization, estimated 3-4 unit tests) | ~3,000 |
| **Total estimate** | **~15,500 tokens** |

Well within 20-30% of a 200K window. No split required.

---

## Tasks (MANDATORY)

Tasks detailed at spec-crystallization. High-level sequence:

1. **PO pre-work (before TDD):** Product-owner authors BC-2.16.002 catalog row for
   `column_source_path_key_absent` (debug, normal optional-field absence, no audit role)
   in BC-2.16.002 §Postconditions.
2. **Red Gate — test first:** Write RGTs (see §Red Gate Tests enumerated at crystallization)
   for AC-001 (ExtractError enum; return type from extract_at_path) and AC-002
   (KeyNotFound → debug, ExtractionFailed → warn). MUST fail before implementation.
3. **Implementation:** Introduce `ExtractError` enum in `pipeline.rs`; update
   `extract_at_path` / `extract_with_tokens` signatures; update `map_record` in
   `column_mapping.rs` to dispatch on error variant.
4. **SAP-1 self-check:** Verify `column_source_path_key_absent` has a BC-2.16.002 catalog
   row before PR merges.
5. **Regression:** `just iter prism-spec-engine --no-fail-fast` to confirm no regressions
   across all sensor source_path columns.
6. **Final gate:** `just check` passes; story-level holdout gate before push.

---

## Previous Story Intelligence (MANDATORY)

1. **S-CLAROTY-VULNS-001 (in-flight at stub creation time):** Source of this story; deferred
   finding F-R4A-OBS-001. The `id` column in claroty_vulnerabilities uses
   `source_path = "$.id"` and is an optional field absent from the fields_enum. This is the
   primary motivating case. Pick up after S-CLAROTY-VULNS-001 merges to avoid merge conflicts
   on `column_mapping.rs`.

2. **S-ADR058-OCSF-COERCION-001 (merged PR #240):** Closed EC-016-013-007/008/009 (coercion
   path fixes). The `extract_at_path` return type change must not regress the coercion path
   that was fixed there.

---

## Architecture Compliance Rules (MANDATORY)

From `architecture/module-decomposition.md` §SS-16 Spec Engine:
- `pipeline.rs §extract_at_path` owns source_path extraction logic. `column_mapping.rs
  §map_record` owns the column-mapping dispatch (including log emission).
- New `enum ExtractError` MUST be defined in `pipeline.rs` (collocated with
  `extract_at_path`). Do NOT define it in a separate `errors.rs` file — this is an internal
  extract-path error, not a public `SpecEngineError`.
- Do NOT re-export `ExtractError` via `pub` to other crates; it is an implementation detail
  of the prism-spec-engine extraction path.

From BC-2.16.002 §Postconditions (Canonical Structured Event Catalog):
- Every `event_type` emission site must have a corresponding catalog row before the PR merges
  (PG-LP11-001 / SAP-1). Adding `column_source_path_key_absent` without the catalog row is
  a P1 finding.

From CLAUDE.md §Conventions (no `println!` in production code):
- Use `tracing::debug!` / `tracing::warn!` with structured fields. No bare `println!`.

---

## Library & Framework Requirements (MANDATORY)

| Library | Version | Source |
|---------|---------|--------|
| `tracing` | per workspace Cargo.toml | `debug!` / `warn!` structured log macros |
| `serde_json::Value` | per workspace Cargo.toml | Return type of `extract_at_path` (unchanged) |

No new Cargo.toml production dependencies required.

---

## File Structure Requirements (MANDATORY)

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-spec-engine/src/pipeline.rs` | Introduce `ExtractError` enum; update `extract_at_path` / `extract_with_tokens` return type |
| MODIFY | `crates/prism-spec-engine/src/column_mapping.rs` | Update `map_record` to dispatch on `ExtractError` variant for log level |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.16.002-*.md §Postconditions` | PO-authored: add `column_source_path_key_absent` catalog row — authored BEFORE TDD begins |

Files that MUST NOT be modified:
- Any sensor TOML spec (`crates/prism-sensors/specs/*.sensor.toml`) — engine-only change; no
  sensor config changes required
- `crates/prism-dtu-*` — DTU clones are unaffected

## Forbidden Dependencies

`prism-spec-engine` MUST NOT gain any new dependency on sensor-specific crates as a result of
this story. The engine change must be sensor-agnostic per POL-36.

---

## References

- BC-2.16.002 (current version at pickup) — Canonical Structured Event Catalog; new catalog row
  required for `column_source_path_key_absent`
- S-CLAROTY-VULNS-001 — origin story; deferred finding F-R4A-OBS-001 (2026-08-25)
- `architecture/module-decomposition.md` §SS-16 — Spec Engine subsystem ownership

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-08-25 | story-writer | Initial draft stub. Deferred from §S-CLAROTY-VULNS-001 §F-R4A-OBS-001 (architect-ruled out-of-scope 2026-08-25). Anchor for ~1000-WARN/fetch log-severity concern. Spec-crystallization (BCs, RGTs, holdout scenarios) deferred to pickup time. |
