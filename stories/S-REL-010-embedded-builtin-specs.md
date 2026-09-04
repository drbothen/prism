---
document_type: story
story_id: S-REL-010
title: "prism-bin: binary self-sufficiency — embed built-in sensor+infusion specs as BASE layer, make spec_dir optional"
wave: post-v1.0
epic_id: E-REL
priority: P1
status: draft
# BC status: pending PO authorship — behavioral_contracts must be non-empty before status: ready (S-7.01).
# The fail-fast spec_dir boot path is governed by boot-sequence BCs (SS-22 Process Lifecycle).
# PO must author or amend BCs covering the optional spec_dir boot path before dispatch.
version: "0.1"
level: "L4"
producer: story-writer
timestamp: "2026-09-03T00:00:00Z"
modified: "2026-09-03"
phase: 3
cycle: v1.0.0-brownfield
tdd_mode: strict
# tdd_mode: strict — boot path rewiring and precedence logic require full TDD red-green discipline.
# SAC-1: Enumerated Red Gate list and density check required before status: ready.
subsystems: [SS-22, SS-16]
# Subsystem anchor justifications:
#   SS-22 (Process Lifecycle) owns boot step 4 modification — the fail-fast spec_dir check and
#     the embedded-BASE-layer wiring live in prism-bin boot.rs (SS-22 scope per ARCH-INDEX).
#   SS-16 (Spec Engine) owns the spec parsing logic — parse_spec_directory and the embedded
#     BUNDLED_SPEC_SCHEMAS static already live in prism-spec-engine / prism-query; the new
#     production wiring of the embedded base layer uses this subsystem's machinery.
crates_touched: [prism-bin, prism-spec-engine, prism-query]
# prism-query: BUNDLED_SPEC_SCHEMAS static (already has the include_str! embeds — see materialization.rs).
#   This story wires those pre-existing embeds into the production boot path. prism-query is
#   touched to promote the test-only static to the production-reachable path (or move the embed
#   to a shared location per the single-source-of-truth constraint).
target_module: prism-bin
capabilities: []
behavioral_contracts: []
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios at remove-uncertainty time.
assumption_validations: []
risk_mitigations:
  - "Single-source-of-truth for embed: prism-query/materialization.rs already declares
    CROWDSTRIKE_SPEC_TOML, ARMIS_SPEC_TOML, CLAROTY_SPEC_TOML, CYBERINT_SPEC_TOML via
    include_str! (confirmed in crates/prism-query/src/materialization.rs). The new boot embed
    MUST reuse these same statics — do NOT introduce a parallel include_str! in prism-bin.
    A duplicated embed creates two compile-time copies that can silently diverge when TOML
    specs are updated. Architectural decision (likely an ADR) needed before implementation."
  - "spec_dir fail-fast removal: the current MED-3 guard in step4_load_sensor_specs logs
    'spec_dir does not exist' and returns Err. Changing to a warning log + fallback-to-embedded
    path changes observable boot behavior. Any BC governing the boot failure map (ADR-022 §A
    exit-code table, SS-22 boot-sequence contract) must be reviewed and potentially amended."
  - "Precedence rule: when spec_dir is present AND contains a sensor TOML with the same
    sensor_id as an embedded built-in, the disk version must take effect and the embedded
    version must be suppressed. This is a merge/override at the sensor_id key level, not at
    the directory level."
  - "Per-org overlay interaction: the customers/ subdirectory mechanism (step4a/4b in boot.rs)
    applies per-org overlays on top of TYPE specs. When spec_dir is absent, per-org overlay
    paths under customers/ cannot be resolved. Document the new behavior: overlays require
    spec_dir to be set; overlay paths are relative to spec_dir."
  - "Infusion specs: threatintel.infusion.toml and nvd.infusion.toml are referenced via
    include_str! in prism-spec-engine/tests/enrichment_pivot_002_tests.rs. These must also
    be embedded as a BASE layer for the infusion engine, matching the sensor-spec embedding
    pattern. PO to clarify scope at remove-uncertainty time."
depends_on: []
# No build-order dependencies — this story builds on top of the v1.0.0 codebase.
blocks: []
points: 13
estimated_days: 4
risk: HIGH
# Risk justification: Changes observable boot behavior (spec_dir optional → binary is self-sufficient
# without operator setup). Requires design decision (ADR) for embed architecture. Touches boot
# critical path. Wrong precedence logic silently masks operator customizations.
acceptance_criteria_count: 0
# acceptance_criteria_count: 0 — draft stub; ACs authored when PO writes BCs.
red_gate_tests: 0
# red_gate_tests: 0 — draft stub; RG tests enumerated after ACs exist.
# SAC-1 requirement: before status: ready, an enumerated RG-001..RG-NNN list and BC-5.38.001
# density check paragraph are required.
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-query/src/materialization.rs"
  - "crates/prism-sensors/specs/"
  - "specs/infusions/"
input-hash: "[pending-recompute]"
traces_to: []
cycle_note: "D-2439 human-directed deferral 2026-09-03 — v1.0.0 ships Option 1 (archive-bundled specs); embedding is the turnkey end-state for v1.1+."
---

# S-REL-010 — Binary Self-Sufficiency: Embedded Built-In Specs as BASE Layer

> **DRAFT STUB — NOT FOR IMMEDIATE DELIVERY.** This story captures the human-directed
> post-v1.0.0 deferral (D-2439, 2026-09-03). v1.0.0 ships Option 1 (archive-bundled
> specs placed by the operator). This story delivers the turnkey end-state for a future
> release. PO must author BCs before dispatch. Status MUST remain `draft` until
> `behavioral_contracts:` is populated per S-7.01.
>
> SAC-1: Before status: ready, an enumerated Red Gate list (RG-001..RG-NNN) and
> BC-5.38.001 density check paragraph are required.

## Authority

- **ADR-022 §B** boot sequence (11-step; step 4 = spec loading)
- **SS-22** (Process Lifecycle, `prism-bin/src/boot.rs`) — owner of boot step 4 modification
- **SS-16** (Spec Engine, `prism-spec-engine`) — owner of spec parsing and `parse_spec_directory`
- **`crates/prism-query/src/materialization.rs`** — BUNDLED_SPEC_SCHEMAS static with existing
  `include_str!` embeds for 4 built-in sensor TOMLs (test-only fallback path today)

---

## Narrative

As a SOC analyst or MSSP operator who downloads the `prism` binary, I want `prism start` to
boot with working sensors without having to manually extract and place spec files, so that
a single-binary deployment "just works" out of the box without operator setup overhead.

---

## Background and Current State

v1.0.0 ships with archive-bundled specs: the release archive contains a `specs/` directory
that operators must extract and place at the `spec_dir` path configured in `prism.toml`.
Boot step 4 (`step4_load_sensor_specs`) fail-fasts if `spec_dir` does not exist.

The codebase already has the infrastructure for embedding:

- `crates/prism-query/src/materialization.rs` declares `CROWDSTRIKE_SPEC_TOML`,
  `ARMIS_SPEC_TOML`, `CLAROTY_SPEC_TOML`, `CYBERINT_SPEC_TOML` via `include_str!` and a
  `BUNDLED_SPEC_SCHEMAS` static initialized by `build_bundled_spec_schemas()`. These are
  wired to `pre_register_empty_tables` — a test-only fallback that registers empty table
  schemas when no live `TableRegistry` is available. They are NOT wired into the production
  boot path.

This story wires those embedded specs into the production boot path as a BASE layer, making
`spec_dir` optional (graceful fallback when absent, not a fail-fast error).

---

## Scope

1. **Architectural decision (ADR required before implementation):** Where does the production
   embed live — in `prism-query/materialization.rs` (promoted from test-only to production),
   in `prism-spec-engine` (as a new `builtin_specs` module), or in `prism-bin` (inline)?
   Single-source-of-truth constraint: the boot-time embed and the test-time embed must reference
   the same compile-time constant. No parallel `include_str!` invocations for the same file.

2. **Boot step 4 rewiring:** Replace the fail-fast `spec_dir` check with:
   - If `spec_dir` is configured and exists: load from disk (same path as today). Disk specs
     override embedded specs by `sensor_id` (operator customization / sensor version pinning).
   - If `spec_dir` is absent or not configured: load from the embedded BASE layer only.
   Log the source of each spec at boot (embedded BASE vs disk override).

3. **Embedded BASE layer:** The 4 sensor specs (claroty, crowdstrike, cyberint, armis) AND the
   2 infusion specs (threatintel, nvd) embedded at compile time. Scope boundary: PO adjudicates
   infusion spec embedding at remove-uncertainty time.

4. **Precedence rule (sensor_id-keyed):** When both an embedded spec and a disk spec exist for
   the same `sensor_id`, the disk spec wins. Log this as `spec_override` at INFO level.

5. **Per-org overlay compatibility:** When `spec_dir` is absent, the `customers/` overlay
   mechanism cannot resolve relative paths. Document this constraint in prism.toml.example
   and emit a clear warning (not an error) if overlay paths are configured without `spec_dir`.

---

## Acceptance Criteria (sketch — authoritative ACs authored when BCs exist)

*These are implementation-scope sketches, NOT final ACs. PO authors canonical ACs from BCs.*

- AC-001 (placeholder): `prism start` succeeds with an empty or absent `spec_dir` and all 4
  built-in sensor table names are queryable via the MCP `query` tool.

- AC-002 (placeholder): When `spec_dir` contains a sensor TOML with the same `sensor_id` as
  a built-in, the disk version is loaded and the embedded version is suppressed. The boot log
  emits a `spec_override` event for the overridden sensor.

- AC-003 (placeholder): When `spec_dir` contains a new sensor TOML (different `sensor_id`
  from all built-ins), that sensor is registered alongside the built-ins.

- AC-004 (placeholder): The production boot embed and the test-path embed (`BUNDLED_SPEC_SCHEMAS`
  in materialization.rs) use the same compile-time source constants — verified by a compile-fail
  or doc test that detects divergence.

- AC-005 (placeholder): `prism start` without `spec_dir` emits an INFO log indicating which
  specs were loaded from the embedded BASE layer.

---

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| (pending PO authorship) | Boot sequence — step 4 spec loading (SS-22 scope) | N/A | Governs optional spec_dir boot path |
| (pending PO authorship) | Embedded spec precedence (SS-16 scope) | N/A | Governs disk-overrides-embedded rule |

*No BCs are in `behavioral_contracts:` frontmatter (stub; PO authorship required per S-7.01).*

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Embedded spec constants | `prism-query/src/materialization.rs` or `prism-spec-engine/src/builtin_specs.rs` (ADR decides) | Pure (compile-time constants) |
| Boot step 4 rewiring | `prism-bin/src/boot.rs §step4_load_sensor_specs` | Effectful (I/O — file system read + registry write) |
| Spec precedence merge | `prism-spec-engine` config_manager | Pure (merge by sensor_id key) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| Embedded spec constants (`include_str!` statics) | Pure | Compile-time constants — no I/O at runtime |
| Spec precedence merge (sensor_id key override) | Pure | Deterministic map merge — no side effects |
| `step4_load_sensor_specs` (boot step 4) | Effectful | File system read + registry write; same classification as today |
| `prism.toml.example` update | N/A | Documentation — no Rust purity boundary applies |
| ADR artifact | N/A | Architecture documentation — no Rust purity boundary applies |

---

## Token Budget Estimate (draft — refined at scheduling)

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,000 |
| `boot.rs` (step 4 region ~120 lines) | ~1,800 |
| `materialization.rs` (BUNDLED_SPEC_SCHEMAS region ~80 lines) | ~1,200 |
| Governing ADR (new — ~400 lines at authoring) | ~4,800 |
| BCs (pending PO authorship, ~2) | ~4,000 |
| Sensor TOML files (4 × ~200 lines each, already embedded) | ~0 (compile-time) |
| Total | ~15,000 |

Within the 30% context window budget.

---

## Previous Story Intelligence

N/A — first story in the post-v1.0 embedded-specs epic. Implementer should read
S-REL-004 (demo bundle packaging) to understand the v1.0.0 archive-bundled model
this story supersedes for new installs.

---

## Tasks

*N/A — draft stub. Tasks authored when BCs are written and story is materialized for delivery.*

Likely task structure (for planning purposes only — NOT authoritative):

1. Author ADR for embedded spec architecture (embed location + precedence rule).
2. Author/amend BCs with PO for optional spec_dir boot path.
3. Write Red Gate failing tests (one per AC after BCs exist).
4. Promote or relocate BUNDLED_SPEC_SCHEMAS embed to production-reachable path.
5. Modify `step4_load_sensor_specs` to fall back to embedded BASE when spec_dir is absent.
6. Implement sensor_id-keyed precedence merge (disk wins over embedded).
7. Add INFO logging for spec source (embedded BASE vs disk override).
8. Update `prism.toml.example` to document spec_dir as OPTIONAL.
9. Write integration tests: boot without spec_dir, boot with override, overlay warning.
10. Make all Red Gate tests green.
11. LOCAL adversary review (BC-5.39.001 3-CLEAN).
12. Story-level holdout gate.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Single-source-of-truth for embed | Risk mitigation #1 above | ADR + compile-fail or doc test (AC-004) |
| Disk overrides embedded by sensor_id | Precedence rule above | AC-002 |
| No fail-fast on absent spec_dir (post-S-REL-010) | This story scope | AC-001 |
| Boot log source attribution per spec | Scope item 2 | AC-005 |
| Per-org overlays require spec_dir (warn, not error) | Scope item 5 | AC (to be authored by PO) |

---

## Library & Framework Requirements

No new library dependencies anticipated. All plumbing uses existing crate machinery
(include_str!, OnceLock/Lazy static, parse_spec_directory, ConfigSnapshot).

---

## File Structure Requirements (draft)

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/src/boot.rs §step4_load_sensor_specs` | Modify | Remove fail-fast; wire embedded BASE layer fallback |
| `crates/prism-query/src/materialization.rs` OR new `crates/prism-spec-engine/src/builtin_specs.rs` | Modify/Create | Promote or move BUNDLED_SPEC_SCHEMAS to production-reachable path (ADR decides) |
| `.factory/specs/architecture/decisions/ADR-0NN-embedded-builtin-specs.md` | Create | Architectural decision for embed location and precedence rule |
| `prism.toml.example` | Modify | Document `spec_dir` as OPTIONAL with note about overlay dependency |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | spec_dir configured but directory deleted between restarts | Graceful fallback to embedded BASE; emit WARNING log |
| EC-002 | Disk spec has syntax error (TOML parse failure) | Fail-fast on that sensor's disk spec; embedded version is NOT used as fallback (explicit disk override implies operator intent) |
| EC-003 | Same sensor_id in embedded AND disk spec | Disk wins; log `spec_override` at INFO |
| EC-004 | spec_dir absent + overlay customers/ configured | Emit WARNING that overlay paths cannot be resolved without spec_dir; overlay loading is skipped |
| EC-005 | Hot reload with spec_dir absent | `reload_config` with no spec_dir re-loads embedded specs; no change if no disk specs exist |

---

## Forbidden Dependencies

- No parallel `include_str!` invocations for the same `.toml` file in different modules (single-source-of-truth constraint).
- No `unwrap()` / `expect()` on embedded spec parse — built-in specs must parse successfully (compile-time guarantee via const + test).

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.1 | 2026-09-03 | Initial draft stub — D-2439 human-directed post-v1.0.0 deferral (2026-09-03). Captures scope, design constraints, and acceptance sketch. BCs pending PO authorship. D-2440: renumbered S-REL-008→S-REL-010 to avoid collision with S-REL-008 (locked to registry-publish meaning). |
