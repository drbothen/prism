---
document_type: behavioral-contract
level: L3
version: "1.12"
status: active
producer: product-owner
timestamp: 2026-05-15T00:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: "2026-05-15"
modified: "2026-05-19"
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
input-hash: null
traces_to: ["CAP-029"]
extracted_from: ".factory/specs/prd.md"
supersedes: BC-2.16.004
---

# BC-2.16.011: CustomAdapter Rust Trait Retirement — Removal of Trait, Registry, and All Call Sites

## Description

The `CustomAdapter` Rust trait in `crates/prism-spec-engine/src/custom_adapter.rs` and its
companion `CustomAdapterRegistry` are removed wholesale. `BC-2.16.004` (deprecated in
PREREQ-F) described the old escape hatch; this contract defines the REMOVAL boundary: which
files are deleted or modified, what the post-removal public API surface must look like, and
what behavioral guarantees survive. The `.prx` WASM plugin model is the sole escape hatch
for non-declarative sensor behavior from this story forward. The three call sites that
re-export or exercise `CustomAdapter` in `lib.rs`, `examples/demo_spec_loading.rs`, and
`tests/bc_2_16_004_test.rs` are all removed in this story's atomic commit.

## Preconditions

- BC-2.16.004 is `lifecycle_status: deprecated`. This BC operationalizes its retirement to `removed`.
- `S-PLUGIN-PREREQ-A` has merged: `SensorId(Arc<str>)` is the canonical identity type. Any
  `CustomAdapterRegistry` internal key type that referenced `SensorType` (if any survived
  PREREQ-A) must have already been migrated or is confirmed to use `&str` / `String`.
- `S-WAVE5-PREP-01` has already removed dead `custom_adapter_registry` references from
  `crates/prism-bin/src/boot.rs`. `boot.rs` receives exactly ONE 1-line insertion in this
  story (`prism_query::invalidation::mark_query_phase_started();` per F-LP56-HIGH-001
  adjudication / ADR-026 D7 v1.23); this serves the WriteToolInvalidationMap query-phase flag
  (BC-2.16.012 scope), NOT CustomAdapter removal. No other `boot.rs` changes are made.
- `S-PLUGIN-PREREQ-F` has confirmed (per ADR-023 Rule 5 publication-history determination —
  "since `prism-spec-engine` has never been published to crates.io with CustomAdapter
  exposed, no deprecation grace period is required" — and PLUGIN-AUDIT-001 HIGH-3
  dead-code confirmation — "CustomAdapterRegistry and CustomAdapter Rust trait are RETIRED;
  locals are created and immediately dropped at the end of the boot function scope") that no
  external deprecation period is required before removal.
- The three confirmed live call sites (verified by grep in ADR-023 §Architectural Constraints (C5 bullet)) are:
  1. `crates/prism-spec-engine/src/lib.rs` — `pub use custom_adapter::*` re-export
  2. `crates/prism-spec-engine/examples/demo_spec_loading.rs` — demo file that exercises the registry
  3. `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` — BC test for the deprecated behavior

## Postconditions

- `crates/prism-spec-engine/src/custom_adapter.rs` is DELETED. The file contains the
  `CustomAdapter` trait, `CustomAdapterRegistry` struct, the `CustomAuth` placeholder
  (the `SensorAuth` duplicate for sealed-trait workaround), and all associated impls.
- `crates/prism-spec-engine/src/lib.rs` no longer contains `pub use custom_adapter::*`
  or any `mod custom_adapter;` declaration. The public API surface of `prism-spec-engine`
  does not expose `CustomAdapter`, `CustomAdapterRegistry`, or `CustomAuth`.
- `crates/prism-spec-engine/examples/demo_spec_loading.rs` is DELETED or has its
  `CustomAdapter`-using section removed. If the example contains other spec-loading
  behavior unrelated to `CustomAdapter`, the relevant sections may be preserved; the
  `CustomAdapter`-specific portions are removed.
- `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` is DELETED. This file exercised
  `CustomAdapterRegistry` behavior that no longer exists. A replacement test demonstrating
  the WASM plugin path is not required in this story (it belongs to PLUGIN-MIGRATION-001-C).
- After removal, `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` returns
  ZERO matches in production source (`src/` paths) and zero test matches (test files are also
  removed in this story). The E-SPEC-008 error code entry in `error-taxonomy.md` citing
  `CustomAdapter` is updated to reflect retirement.
- BC-2.16.004 `lifecycle_status` is updated from `deprecated` to `removed` (the file
  `BC-2.16.004-rust-escape-hatch.md` frontmatter is amended with all four field mutations):
  - `deprecated_by` field: `ADR-023` → `ADR-027` (ADR-027 §Decision is the operational deletion mandate; ADR-023 Rule 5 is the deprecation philosophy that ADR-027 operationalizes)
  - `removal_reason` field: `"PREREQ-E retirement per ADR-027 §Decision + ADR-023 Rule 5"`
  - `removed` field: `"<PREREQ-E merge date>"` (implementer substitutes actual PREREQ-E merge date at PR-create time)
  - `lifecycle_status` field: `deprecated` → `removed`
- The `E-SPEC-008` error taxonomy entry is updated: its description is changed from
  "A CustomAdapter (BC-2.16.004) panicked during execution" to a retired/removed note, or
  the entry is preserved with a `retired: true` annotation and a note that the code is no
  longer active as of S-PLUGIN-PREREQ-E.

## Invariants

- **INV-ADAPTER-RETIRE-001:** After this story merges, `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` returns ZERO matches in all non-comment, non-doc-string Rust source lines.
- **INV-ADAPTER-RETIRE-002:** The `prism-spec-engine` crate public API (as reflected in `src/lib.rs` re-exports) does NOT expose any type, trait, or function from the retired `custom_adapter` module.
- **INV-ADAPTER-RETIRE-003:** `boot.rs` receives exactly ONE 1-line insertion in this story (`prism_query::invalidation::mark_query_phase_started();` per F-LP56-HIGH-001 adjudication; ADR-026 D7 v1.23) which serves the BC-2.16.012 WriteToolInvalidationMap query-phase flag — NOT CustomAdapter removal. No CustomAdapter-related `boot.rs` changes are made. The boot sequence was cleaned of `CustomAdapterRegistry` references by `S-WAVE5-PREP-01`; PREREQ-E confirms that state for CustomAdapter cleanup, while the BC-2.16.012-scoped query-phase flag insertion is the sole new `boot.rs` change.
- **INV-ADAPTER-RETIRE-004:** The `.prx` WASM plugin model is the sole surviving escape hatch for non-declarative sensor behavior. No parallel Rust-trait escape hatch is introduced. ADR-023 Rule 5 is now fully implemented.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-008` (retired) | This error code previously indicated `CustomAdapter` panic. After PREREQ-E, the `CustomAdapter` code path does not exist. E-SPEC-008 is preserved in the error taxonomy with `retired: true` and no live code path triggers it. Plugin execution panics are surfaced via `E-PLUGIN-001` (BC-2.17.001). AC-11 enforces this invariant at two layers: (1) **Code-side** — Rust test `test_BC_2_16_011_e_spec_008_retired_annotation` (prism-spec-engine) greps `crates/*/src/` and asserts zero `ESpec008` / `E-SPEC-008` construction sites exist; POL-1 (append-only numbering) exempts the variant declaration in `prism-core/src/error.rs` itself. (2) **Spec-side** — `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh` asserts that `error-taxonomy.md` E-SPEC-008 row contains both `"RETIRED in S-PLUGIN-PREREQ-E"` and `"ADR-027"` markers; this hook runs in the `.factory/` pre-commit chain and as a wave-gate hygiene check. Relocation rationale: architect adjudication `FB-PR-1-error-taxonomy-test-relocation.md` (Option 1) — the spec-governance annotation invariant belongs in the `.factory/` hook chain, not a compiled test binary; the code-side construction-site gate belongs in the Rust test. |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-011-001 | `cargo build --workspace` immediately after deleting `custom_adapter.rs` | Fails until the three call sites in `lib.rs`, `examples/demo_spec_loading.rs`, and `tests/bc_2_16_004_test.rs` are also removed in the same atomic commit |
| EC-016-011-002 | `demo_spec_loading.rs` contains non-`CustomAdapter` spec loading code | That code is preserved; only `CustomAdapter` / `CustomAdapterRegistry` usage sections are removed. If the file becomes empty or a trivial stub, it is deleted |
| EC-016-011-003 | A future story (PLUGIN-MIGRATION-001-C) needs to test the WASM plugin escape hatch | That story writes NEW test files targeting `PluginRuntime`. The deleted `bc_2_16_004_test.rs` is NOT resurrected; a fresh test file under a new name is authored |
| EC-016-011-004 | `E-SPEC-008` appears in error-handling match arms in other crates | None currently exist (PLUGIN-AUDIT-001 confirms no in-tree callers). If any are found during implementation, they are removed in the same atomic commit |
| EC-016-011-005 | `BC-2.16.004-rust-escape-hatch.md` frontmatter update conflicts with deprecation metadata | `deprecated_by` bumps `ADR-023` → `ADR-027` (ADR-027 §Decision is the operational deletion mandate; ADR-023 Rule 5 was the deprecation philosophy that ADR-027 operationalizes); add `removed: "<PREREQ-E merge date>"`, `removal_reason: "PREREQ-E retirement per ADR-027 §Decision + ADR-023 Rule 5"`, change `lifecycle_status: deprecated → removed` |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.011-001 | `grep -rn "CustomAdapter\|CustomAdapterRegistry\|CustomAuth" crates/` after merge | Zero matches in any `src/` or `tests/` path |
| TV-BC-2.16.011-002 | `cargo build --workspace --all-features` after all three call sites removed | Clean build, zero warnings with `-D warnings` |
| TV-BC-2.16.011-003 | `prism-spec-engine` `rustdoc` output | No `CustomAdapter`, `CustomAdapterRegistry`, or `CustomAuth` in public API documentation |
| TV-BC-2.16.011-004 | `cargo test -p prism-spec-engine` | Passes; `bc_2_16_004_test.rs` no longer exists so its tests are absent (not failing) |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-154 | CustomAdapter Behavioral Equivalence — PluginRuntime Dispatch Produces Equivalent Output (integration test). Verifies that the WASM plugin dispatch path through `PluginRuntime` produces equivalent output to the deleted `CustomAdapter::override_fetch` path. Priority P1; authored in PREREQ-E ADR burst; full harness authored in PLUGIN-MIGRATION-001-A scope. |
| VP-155 | CustomAdapter Absent from prism-spec-engine Public API (compile-fail perimeter). Two compile-fail files assert that `prism_spec_engine::CustomAdapter` and `prism_spec_engine::CustomAdapterRegistry` produce `error[E0432]`. Files added to perimeter crate in PLUGIN-MIGRATION-001-A scope after PREREQ-E merges. Priority P0. |

### VP-154 Fixture Acceptance Criterion

The integration test fixture for VP-154 MUST produce records conforming to the following canonical OCSF Detection Finding (class 2004) schema. This criterion resolves architect's open question (Q3 — VP-154 fixture WASM acceptance criterion).

**Canonical OCSF record schema for the test fixture:**

```json
{
  "type_uid":        2004001,
  "class_uid":       2004,
  "category_uid":    2,
  "severity_id":     3,
  "severity":        "Medium",
  "time":            "<RFC 3339 timestamp, e.g., 2026-05-15T00:00:00Z>",
  "message":         "Mock sensor fetch result from WASM plugin fixture",
  "finding_info": {
    "uid": "test-001",
    "title": "mock_event"
  },
  "raw_data":        "{\"source\": \"minimal_sensor_fetch.prx\", \"id\": \"test-001\"}"
}
```

**Required fields:** `type_uid`, `class_uid`, `category_uid`, `severity_id`, `severity`, `time`, `message`, `finding_info.uid`, `raw_data`. The `raw_data` field carries the fixture-specific payload as a JSON-encoded string per the OCSF raw_data convention.

**Count threshold:** The integration test asserts `records.len() >= 1` (at least one record per plugin-hook invocation when the `.prx` fixture is loaded and a basic SELECT * / `PipelineExecutor::execute` is issued for the mock sensor's table). A single well-formed record is sufficient to prove the dispatch path is non-empty and non-panicking.

**Behavioral equivalence definition:** **Semantic equivalence** is the correct definition for this test, NOT byte-identical equality. Rationale: The `time` field in real OCSF records contains a timestamp that varies per invocation. The fixture can emit a fixed timestamp (e.g., `"2026-01-01T00:00:00Z"`) to enable byte-identical comparison in CI, but the acceptance criterion does NOT require byte-identical records — it requires that:
1. `records[0]["finding_info"]["uid"]` == `"test-001"` (fixture-controlled stable ID)
2. `records[0]["class_uid"]` == `2004` (Detection Finding class)
3. `records[0]["severity_id"]` is a valid OCSF integer (1–5 or 99)
4. `records.len() >= 1`

Byte-identical comparison would create timestamp-driven flakiness in CI. Semantic equality on stable fields produces a non-flaky, deterministic test. The fixture SHOULD emit a hardcoded timestamp to allow for an optional byte-identical CI mode if test-writer prefers it.

## Related BCs

- BC-2.16.004 (Rust Escape Hatch for Custom Adapters — DEPRECATED, now removed): this BC supersedes it. The deprecated BC file's `lifecycle_status` is updated to `removed`.
- BC-2.01.016 (SensorAuth Open Trait): sibling contract authored in this story; the `CustomAuth` placeholder deleted here existed only to work around sealed `SensorAuth`.
- BC-2.16.012 (PluginRegistry Call-Site Migration in spec_parser.rs): sibling contract; establishes the PluginRegistry as the replacement dispatch mechanism.

## Architecture Anchors

- `crates/prism-spec-engine/src/custom_adapter.rs` — primary deletion target
- `crates/prism-spec-engine/src/lib.rs` — re-export removal site
- `crates/prism-spec-engine/examples/demo_spec_loading.rs` — example cleanup site
- `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` — test deletion site
- ADR-023 §Architectural Constraints (C5 bullet, Rule 5) — authoritative CustomAdapter retirement specification
- ADR-027 — CustomAdapter Rust Trait Same-Burst Removal — Perimeter Enforcement in Wave 1/A; §D3 defines compile-fail perimeter (VP-155) and §Verification Property Anchors defines PluginRuntime behavioral equivalence requirement (VP-154)

## Story Anchor

S-PLUGIN-PREREQ-E

## VP Anchors

- VP-154 (CustomAdapter Behavioral Equivalence — integration test verifying WASM dispatch path produces non-empty, semantically-equivalent OCSF Detection Finding records; P1; authored PLUGIN-MIGRATION-001-A scope)
- VP-155 (CustomAdapter Absent from prism-spec-engine Public API — compile-fail perimeter; two files asserting E0432 for CustomAdapter + CustomAdapterRegistry; P0; added in PLUGIN-MIGRATION-001-A scope after PREREQ-E merge)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029. The `CustomAdapter` trait was the Rust-code escape hatch within CAP-029 for sensors that config could not express. Retiring it completes CAP-029's transition to purely config-driven + WASM-plugin-driven adapters, as mandated by ADR-023 Rule 5. |
| L2 Invariants | DI-012 (amended in PREREQ-F: runtime enforcement only; PREREQ-E closes the compile-time escape hatch path) |
| Related BCs | BC-2.16.004 (superseded — now removed), BC-2.01.016 (SensorAuth open trait), BC-2.16.012 (PluginRegistry migration) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.12 | D-726-post-merge | 2026-05-19 | state-manager | POL-14 auto-promotion at merge: PR #151 (S-PLUGIN-PREREQ-E) squash-merged to develop@80ebe794 at 2026-05-19T18:06:44Z; PR-LEVEL adversary cascade BC-5.39.001 3-CLEAN CONVERGED; status draft→active, lifecycle_status draft→active. |
| 1.11 | FB-PR-1 | 2026-05-19 | product-owner | FB-PR-1 AC-11 relocation: spec-governance annotation invariant moved from Rust test sub-assertion A to `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh` per architect adjudication `FB-PR-1-error-taxonomy-test-relocation.md` (Option 1). §Error Cases E-SPEC-008 row updated to describe two-layer enforcement model: (1) code-side Rust test `test_BC_2_16_011_e_spec_008_retired_annotation` asserts zero `ESpec008`/`E-SPEC-008` construction sites in `crates/*/src/` (POL-1 exemption for variant declaration); (2) `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh` asserts `"RETIRED in S-PLUGIN-PREREQ-E"` + `"ADR-027"` present in E-SPEC-008 row. |
| 1.10 | FB73 | 2026-05-17 | product-owner | F-LP85-HIGH-001 closure (PO scope): ADR-026 D7 pin v1.22→v1.23 propagation at BC-2.16.011 Preconditions line 54 and INV-ADAPTER-RETIRE-003 line 101 (2 sites). Sibling files story v1.46 + BC-2.16.012 v1.26 + BC-2.16.002 v1.31 (POL-30 Fork B preserved) + VP-156 v0.18 + HS-003 v1.15 + error-taxonomy v1.38 swept in same burst. |
| 1.9 | FB69 | 2026-05-17 | product-owner | F-LP81-HIGH-001 closure: INV-ADAPTER-RETIRE-003 + Preconditions amended to reflect F-LP56-HIGH-001 adjudication (FB44 D-666) — boot.rs receives 1-line insertion for BC-2.16.012 WriteToolInvalidationMap query-phase flag (sibling scope), NOT CustomAdapter removal. 37-pass-surviving BC↔story semantic contradiction closed per CLAUDE.md Source-of-Truth Precedence Rule 1. POL-23 within-FB sibling-sweep + POL-22 named-entity-semantics restored. |
| 1.8 | FB51 | 2026-05-17 | product-owner | POL-23 sibling-sweep (F-LP63-MED-001 family): §Preconditions PLUGIN-AUDIT-001 HIGH-3 mis-anchored citation corrected to Option (a) split provenance — publication-history routed to ADR-023 Rule 5 (correct source); dead-code claim routed to PLUGIN-AUDIT-001 HIGH-3 (correct source). Restores bidirectional traceability. Parallel fix to BC-2.01.016 v1.8 in same burst. |
| 1.7 | FB47 | 2026-05-16 | product-owner | §Architecture Anchors line 178: ADR-027 framing label updated from "deprecation/removal" to "Same-Burst Removal — Perimeter Enforcement in Wave 1/A" per ADR-027 v1.8 title (FB46 F-LP58-HIGH-001 closure downstream propagation). |
| 1.6 | prereq-e-fix-burst-19 | 2026-05-16 | state-manager | F-LP21-HIGH-001 closure — §Changelog renumber-repair-redo (D-611-equivalent pattern applied to sibling BC that was missed in FB14): state-manager catch row v1.2 → v1.3, cascade shift v1.3 → v1.4 → v1.5 → v1.6 (via new repair row insertion at top). POL-26 monotonic strict-ordering violation pre-existing FB1 (invisible to passes 1-20) now resolved. |
| 1.5 | prereq-e-fix-burst-7 | 2026-05-16 | product-owner | F-LP7-HIGH-002 + F-LP7-MED-004 — sibling-sweep close: (1) §Postconditions removal_reason advanced "ADR-023 Rule 5" → "ADR-027 §Decision + ADR-023 Rule 5" + explicit enumeration of all four BC-2.16.004 frontmatter mutations (deprecated_by/removed/removal_reason/lifecycle_status); (2) §Architecture Anchors VP-154 anchor corrected ADR-027 §D5 → §Verification Property Anchors (FB4 D5 scope expansion sibling-sweep miss). TD-VSDD-059 paper-fix detection. |
| 1.4 | prereq-e-fix-burst-6 | 2026-05-16 | architect | F-LP6-MED-004 — EC-016-011-005 `deprecated_by` adjudicated: bumps `ADR-023` → `ADR-027`. ADR-027 §Decision is the operational deletion mandate (timeline + perimeter + removal mechanism); ADR-023 Rule 5 introduced the deprecation philosophy which ADR-027 operationalizes. EC-016-011-005 Resolution cell updated: `deprecated_by: ADR-027`, `removed: "<PREREQ-E merge date>"` (placeholder for actual merge date when PREREQ-E ships), `removal_reason: "PREREQ-E retirement per ADR-027 §Decision + ADR-023 Rule 5"`, `lifecycle_status: deprecated → removed`. |
| 1.3 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `S-PLUGIN-PREREQ-E` was story-ID format; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 1.2 | S-PLUGIN-PREREQ-E-fix-burst-1 | 2026-05-15 | product-owner | F-LP1-HIGH-003 closure: Three §C5 phantom-heading citations corrected per POL-21 — `ADR-023 §C5` → `ADR-023 §Architectural Constraints (C5 bullet)` and `ADR-023 §C5 Rule 5` → `ADR-023 §Architectural Constraints (C5 bullet, Rule 5)`. ADR-023 has no `## C5` heading; C5 is a bold-labeled bullet inside `## Architectural Constraints`. |
| 1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Q3 resolution: Added §VP-154 Fixture Acceptance Criterion with canonical OCSF Detection Finding 2004 schema (required fields: type_uid/class_uid/category_uid/severity_id/severity/time/message/finding_info.uid/raw_data), count threshold (>= 1 record), and behavioral equivalence definition (semantic equality on stable fields — not byte-identical, to avoid timestamp-driven flakiness). VP-154 and VP-155 added to §Verification Properties and §VP Anchors. ADR-027 architecture anchor added. |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Operationalizes ADR-023 §Architectural Constraints (C5 bullet, Rule 5) CustomAdapter Rust trait retirement. Supersedes BC-2.16.004 (deprecated in PREREQ-F, now removed). |
