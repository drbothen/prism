---
document_type: adr
adr_id: "ADR-027"
title: "CustomAdapter Rust Trait Same-Burst Removal — Perimeter Enforcement in Wave 1/A"
status: Proposed
date: "2026-05-17"
version: "1.9"
producer: architect
subsystems_affected: [SS-07, SS-16, SS-17]
supersedes: null
superseded_by: null
amends: ADR-023
anchor_stories: [S-PLUGIN-PREREQ-E, PLUGIN-MIGRATION-001-A]
runtime_deliverables:
  - "Remove pub mod custom_adapter; from crates/prism-spec-engine/src/lib.rs"
  - "Remove pub use custom_adapter::{CustomAdapter, CustomAdapterRegistry}; from lib.rs"
  - "Delete crates/prism-spec-engine/src/custom_adapter.rs"
  - "Delete crates/prism-spec-engine/examples/demo_spec_loading.rs (exercises CustomAdapter)"
  - "Delete crates/prism-spec-engine/tests/bc_2_16_004_test.rs (superseded BC test)"
  - "Add compile-fail enforcement: no new CustomAdapter import is admissible post-PREREQ-E"
wiring_deferred_to: null
---

# ADR-027: CustomAdapter Rust Trait Same-Burst Removal — Perimeter Enforcement in Wave 1/A

## Status

Proposed 2026-05-15, v1.0. Governs the PLUGIN-PREREQ-E dead-code cleanup for the
`CustomAdapter` Rust trait and the Wave 1/A complete removal. Implementation is tracked by
S-PLUGIN-PREREQ-E (dead-code erasure at three call sites) and PLUGIN-MIGRATION-001-A (compile-fail
perimeter enforcement that closes the door on any new callers).

---

## Context

ADR-023 Rule 5 mandated retirement of the `CustomAdapter` Rust trait in
`crates/prism-spec-engine/src/custom_adapter.rs`. The audit (PLUGIN-AUDIT-001, 2026-05-10)
confirmed no in-tree production callers exist — only three non-production sites use it:

1. `crates/prism-spec-engine/src/lib.rs` — re-exports `CustomAdapter` and `CustomAdapterRegistry`
   via `pub use custom_adapter::*` (the public API surface).
2. `crates/prism-spec-engine/examples/demo_spec_loading.rs` — imports and exercises the registry
   against a mock adapter (example code, not production).
3. `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` — BC test exercising `CustomAdapterRegistry`
   (superseded when BC-2.16.004 is retired and replaced by WASM plugin tests).

ADR-023 Rule 5 determined that since `prism-spec-engine` has never been published to crates.io
with `CustomAdapter` exposed, no deprecation grace period is required. Same-burst removal is safe.

This ADR details the atomic-deletion scope in PREREQ-E, the Wave 1/A unblock criteria, and the
perimeter enforcement strategy that prevents `CustomAdapter` from being re-introduced after
deletion.

---

## Decision

### D1 — PREREQ-E scope: erase the three call sites, delete custom_adapter.rs

S-PLUGIN-PREREQ-E performs dead-code cleanup as a single atomic commit:

1. Remove `pub mod custom_adapter;` from `lib.rs`.
2. Remove `pub use custom_adapter::{CustomAdapter, CustomAdapterRegistry};` from `lib.rs`.
3. Delete `crates/prism-spec-engine/src/custom_adapter.rs` (the trait, registry, and `CustomAuth`
   placeholder are all gone).
4. Delete `crates/prism-spec-engine/examples/demo_spec_loading.rs` (the only caller of the
   registry in non-test code; the example becomes dead after step 3).
5. Delete `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` (the only test exercising
   `CustomAdapterRegistry`; the BC it tests — BC-2.16.004 — is retired per ADR-023).

The commit must pass `cargo build --workspace` and `cargo test --workspace` with zero references
to `CustomAdapter` or `CustomAdapterRegistry` in non-generated code.

**No `#[deprecated]` annotation phase.** ADR-023 Rule 5 confirmed: no external consumers exist,
so no deprecation grace period is required. The trait and registry are deleted, not deprecated.
If a future audit discovers a published version of `prism-spec-engine` with `CustomAdapter`
exposed, this decision is amended to introduce a one-cycle deprecation window (this ADR would
be updated to v1.1 with a `#[deprecated]` annotation phase added before deletion).

### D2 — Atomic-deletion semantics and perimeter-window scope (PREREQ-E through Wave 1/A)

Between PREREQ-E merge and Wave 1/A gate, `CustomAdapter` no longer exists in the codebase.
There is no migration window where new callers could be introduced, because the type is
deleted atomically in PREREQ-E. The perimeter enforcement (D3) prevents re-introduction.

### D3 — Compile-fail perimeter enforcement (Wave 1/A scope)

PLUGIN-MIGRATION-001-A (the story that closes the forbidden-symbols perimeter gate per
ADR-023 §Verification Properties (VP-PLUGIN-001 bullet)) will add TWO compile-fail files
to the FORBIDDEN-SYMBOLS-001 compile-fail test crate at `tests/external/no-hardcoded-sensors/`:

**File 1:** `tests/external/no-hardcoded-sensors/import_custom_adapter.rs`
```rust
// tests/external/no-hardcoded-sensors/import_custom_adapter.rs
// DO NOT REMOVE — compile-fail gate for ADR-027 (VP-155 P0)
// Asserts that CustomAdapter is not re-introduced to the prism-spec-engine public API.
use prism_spec_engine::CustomAdapter; //~ ERROR unresolved import
```

**File 2:** `tests/external/no-hardcoded-sensors/import_custom_adapter_registry.rs`
```rust
// tests/external/no-hardcoded-sensors/import_custom_adapter_registry.rs
// DO NOT REMOVE — compile-fail gate for ADR-027 (VP-155 P0)
// Asserts that CustomAdapterRegistry is not re-introduced to the prism-spec-engine public API.
use prism_spec_engine::CustomAdapterRegistry; //~ ERROR unresolved import
```

Both files become permanent parts of the FORBIDDEN-SYMBOLS-001 compile-fail catalog. They
ensure that any future attempt to re-export `CustomAdapter` or `CustomAdapterRegistry` from
`prism-spec-engine` fails CI. Both types were exported together via `pub use custom_adapter::*`
(D1 §2); both must be independently gated to prevent partial re-introduction.

The count enforcement in ADR-023 (CI asserts that file count in
`tests/external/no-hardcoded-sensors/` equals FORBIDDEN-SYMBOLS-001 catalog size) must be
updated when these two files are added. The catalog grows by **two entries**: `CustomAdapter`
and `CustomAdapterRegistry`, advancing the total catalog size from 9 to 11
(matching VP-155 §Proof Method (Relationship to VP-PLUGIN-001 paragraph) and HS-PREREQ-E-002-05 §Steps `CATALOG_SIZE=11` assertion).

### D4 — Wave 1/A unblock criteria

For PLUGIN-MIGRATION-001-A to proceed, the following must be true at PREREQ-E merge:

1. `custom_adapter.rs` does not exist at `crates/prism-spec-engine/src/custom_adapter.rs`.
2. `lib.rs` contains no `pub use custom_adapter::*` or any re-export of `CustomAdapter`
   or `CustomAdapterRegistry`.
3. `cargo build --workspace` passes with zero `E0432` (unresolved import) errors referencing
   `custom_adapter` in production source.
4. `bc_2_16_004_test.rs` is deleted; `demo_spec_loading.rs` is deleted.
5. BC-2.16.004 lifecycle_status is `deprecated` per ADR-023 retirement schedule.

These five conditions are the acceptance gate that the PLUGIN-MIGRATION-001-A story-writer
reads before starting Wave 1 work.

### D5 — Spec_parser.rs scope: verification clean-pass AND hardcoded-sensor-string dispatch audit

ADR-023 §Architectural Constraints (C5 bullet) originally listed `spec_parser.rs` as a site to check for `CustomAdapter` or
`CustomAdapterRegistry` references. GREP VERIFICATION (2026-05-15): `spec_parser.rs` contains
zero `CustomAdapter` or `CustomAdapterRegistry` references — the `CustomAdapter` clean-pass
for this file is satisfied mechanically.

**Expanded scope (F-LP4-LOW-001, prereq-e-fix-burst-4):** BC-2.16.012 INV-SPEC-PARSER-OPEN-001
establishes the broader invariant that `spec_parser.rs` MUST NOT contain hardcoded sensor-string
dispatch arms (e.g., `match sensor_type { "crowdstrike" => ..., "cyberint" => ... }`). Story
Task 6 and AC-7 both mandate an audit for this pattern, not just a `CustomAdapter` grep.

PREREQ-E's scope at `spec_parser.rs` is therefore two-part:

1. **Mechanical clean-pass (CustomAdapter):** Verify zero `CustomAdapter` or
   `CustomAdapterRegistry` references exist. Document the grep result in the PR. (Satisfied
   by GREP VERIFICATION above — this part requires no code changes.)
2. **Hardcoded-sensor-string audit (INV-SPEC-PARSER-OPEN-001):** Audit `spec_parser.rs`
   for hardcoded sensor-string dispatch arms. If any are found, migrate them to
   `PluginRegistry` lookup per BC-2.16.012 INV-SPEC-PARSER-OPEN-001 before the story closes.
   If none are found, document the clean result in the PR.

**Alignment rationale:** The original D5 framing ("verify clean only") was an early-pass
narrow scope that predated BC-2.16.012 INV-SPEC-PARSER-OPEN-001 and Story Task 6/AC-7.
The story's broader scope IS correct per the production-grade default (Canonical Principle
Rule 1) and BC-2.16.012 INV-SPEC-PARSER-OPEN-001 alignment. This D5 expansion matches the
story spec without contradicting D1–D4; `spec_parser.rs` cleanup and `custom_adapter.rs`
deletion are orthogonal operations in the same atomic commit.

---

## Rationale

**Same-burst deletion is safe for an unpublished type.** ADR-023 Rule 5 articulated the
reasoning: no external consumers means no breaking change. A `#[deprecated]` annotation phase
serves consumers who need time to migrate. With zero consumers, the annotation adds no value
and extends the migration window unnecessarily, during which the dead code creates confusion
for any implementer reading the codebase.

**Deleting the test alongside the trait is correct.** `bc_2_16_004_test.rs` tests
`CustomAdapterRegistry` behavior that will not exist after PREREQ-E. Keeping a test that
exercises a deleted type is impossible. The behavioral guarantees the test enforced (Rust escape
hatch for non-declarative sensors) are superseded by the WASM plugin model; replacement
coverage comes from VP-PLUGIN-002 (PipelineExecutor integration test against wiremock DTU
clone) and VP-154 (behavioral equivalence between WASM plugin dispatch and the old
CustomAdapter override path).

**Compile-fail perimeter is the correct post-deletion enforcement.** Once the type is gone,
the perimeter test makes re-introduction impossible without also breaking CI. This is stronger
than `#[deprecated]` (which only warns) and is consistent with the existing perimeter pattern
at `tests/external/perimeter-violation/` enforcing the sensor-named type bans.

---

## Consequences

### Positive

- Zero dead code: `custom_adapter.rs`, `demo_spec_loading.rs`, and `bc_2_16_004_test.rs` are
  deleted. No future implementer is confused by the presence of a trait the architecture prohibits.
- Compile-fail perimeter prevents silent re-introduction.
- BC-2.16.004 retirement completes the behavioral contract cleanup.
- Wave 1/A unblocked: PLUGIN-MIGRATION-001-A has clear acceptance criteria (D4).

### Negative / Trade-offs

- `examples/demo_spec_loading.rs` is deleted. If there are any informal integration patterns
  documented there (e.g., how to load a spec directory), that information must be preserved
  elsewhere before deletion (in architecture docs or a new, WASM-focused example). Story
  S-PLUGIN-PREREQ-E must include a sub-task: check `demo_spec_loading.rs` for documentation
  value and extract any non-CustomAdapter patterns into `docs/` before deletion.
- Any external consumer who DID import `CustomAdapter` from a published crate (not found by
  audit, but possible in theory) will encounter a compile error after upgrading. This is
  accepted risk per Rule 5 and the pre-condition check in that rule.
- **ADR-027 scope includes `prism-query` (SS-07) call-site migration to `PluginRegistry::dispatch`
  per BC-2.16.012 INV-INVALIDATION-EXT-001.** The story's crate scope includes `prism-query` (see
  story frontmatter `crates_touched`). The `crates/prism-query/src/invalidation.rs`
  `WriteToolInvalidationMap` container is migrated from `LazyLock<Vec<...>>` to
  `RwLock<Vec<...>>` with a `register_write_tool` API (TD-S-PLUGIN-PREREQ-A-003), enabling
  plugin-registered write tools to participate in cache invalidation at runtime. SS-07 is
  explicitly included in `subsystems_affected` because this ADR's PREREQ-E delivery lands
  concrete code changes in `prism-query`. The D4 Wave 1/A unblock criteria are scoped to
  `prism-spec-engine` clean-pass confirmation; the `prism-query` changes are governed by
  ADR-026 D7 and BC-2.16.012 INV-INVALIDATION-EXT-001.

---

## Verification Property Anchors

- **VP-154** — CustomAdapter behavioral equivalence: integration test verifying that the
  PluginRuntime WASM dispatch path (PREREQ-D/B) produces equivalent behavior to the deleted
  `CustomAdapter::override_fetch` path for the canonical mock-sensor test case.
  Module: prism-spec-engine. Method: integration_test. Priority: P1. Anchor story:
  PLUGIN-MIGRATION-001-A.

- **VP-155** — No CustomAdapter in public API: compile-fail perimeter test asserts
  `use prism_spec_engine::CustomAdapter` fails to compile. Module: prism-spec-engine.
  Method: integration_test. Priority: P0. Anchor story: PLUGIN-MIGRATION-001-A.

---

## Alternatives Considered

**Option A: Deprecate with `#[deprecated]` for one wave, delete in Wave 2.** Rejected.
No external consumers; deprecation grace period buys nothing. Dead code persists for an
additional wave, creating implementer confusion during the migration.

**Option B: Keep CustomAdapter but mark it feature-gated behind a `legacy-adapter` cargo
feature.** Rejected. Feature flags for deprecated types delay deletion indefinitely (features
rarely get cleaned up). The WASM plugin model is the replacement; there is no scenario where
keeping a Rust-trait escape hatch alongside the WASM escape hatch is better than migrating to
WASM.

**Option C: Rename CustomAdapter to InternalAdapterOverride and keep it internal (not pub).**
Rejected. The type serves no internal purpose after the plugin migration. The internal adapters
(CrowdStrikeAuth, etc.) implement `SensorAuth` directly; they do not use `CustomAdapterRegistry`.
Renaming and narrowing scope is scope-creep on dead code — delete it.

---

## Source / Origin

**Convention:** This section lists upstream artifacts that mandated or directly informed this
decision (policy rules, audits, code sites, behavioral contracts). Sibling ADRs addressing the
same epic are coordination artifacts, not upstream sources — they are tracked in §Related ADRs
instead. This convention is intentional and consistent across ADR-026 and ADR-027.

- ADR-023 Rule 5 — CustomAdapter Rust Trait Retirement (mandate for this decision)
- ADR-023 §Architectural Constraints (C5 bullet) — three call sites that must be retired (lib.rs re-export, example, BC test)
- PLUGIN-AUDIT-001 (2026-05-10) — confirmed no in-tree production callers
- `crates/prism-spec-engine/src/lib.rs` — `pub mod custom_adapter;` and `pub use` re-export
- `crates/prism-spec-engine/examples/demo_spec_loading.rs` — only non-test caller
- `crates/prism-spec-engine/tests/bc_2_16_004_test.rs` — only test exercising the registry
- BC-2.16.004 — rust-escape-hatch behavioral contract (retired by ADR-023)
- **BC-2.16.011** — CustomAdapter Rust Trait Retirement (NEW; authored in S-PLUGIN-PREREQ-E by product-owner; primary behavioral contract for the retirement operationalized by this ADR)

---

## Related ADRs

| ADR | Relationship |
|-----|-------------|
| **ADR-023** | This ADR is the detailed specification of ADR-023 Rule 5 / Constraint C5 for CustomAdapter deletion |
| **ADR-026** | SensorAuth unsealing — the companion decision; both are in PREREQ-E scope |
| **ADR-022** | Boot sequence — boot.rs had no live CustomAdapterRegistry references at S-WAVE5-PREP-01 commit |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-15 | architect | Initial proposal — CustomAdapter deprecation/deletion design for S-PLUGIN-PREREQ-E and PLUGIN-MIGRATION-001-A |
| 1.1 | 2026-05-15 | architect | Q5 resolution: add prism-query WriteToolInvalidationMap scope note to §Consequences. The story crates_touched includes prism-query; this ADR was silent on that. Added negative trade-off row explaining the prism-query touch is parallel scope (TD-S-PLUGIN-PREREQ-A-003 / ADR-026 D7 / BC-2.16.012 INV-INVALIDATION-EXT-001) and does not affect D4 Wave 1/A unblock criteria (prism-spec-engine only). |
| 1.2 | 2026-05-15 | architect | prereq-e-fix-burst-2: F-LP2-HIGH-002: TD-A-003 alias canonicalized to TD-S-PLUGIN-PREREQ-A-003 at §Consequences trade-off row (live narrative) and changelog row for v1.1. F-LP2-HIGH-003: Two ADR-023 §C5 phantom-heading citations replaced with §Architectural Constraints (C5 bullet) per POL-21: D5 narrative (line 124) and §Source/Origin (line 228). TD-VSDD-060 workspace-wide greps confirm no further sibling sites in live spec files beyond this ADR. |
| 1.3 | 2026-05-15 | architect | prereq-e-fix-burst-4: F-LP4-LOW-001: D5 scope expanded from "verify clean only" to two-part: (a) mechanical CustomAdapter clean-pass (satisfied by existing grep verification) AND (b) hardcoded-sensor-string dispatch audit per BC-2.16.012 INV-SPEC-PARSER-OPEN-001 + Story Task 6 + AC-7. Original narrow framing predated BC-2.16.012 and story AC-7; story's broader scope is correct per production-grade default. D5 narrative rewritten to enumerate both parts explicitly. No changes to D1–D4 or §Verification Property Anchors. |
| 1.4 | 2026-05-16 | architect | prereq-e-fix-burst-6: F-LP6-MED-002 — SS-07 (Adapter Pagination & Response Cache; prism-query) added to `subsystems_affected`: ADR-027 scope includes prism-query (SS-07) call-site migration per BC-2.16.012 INV-INVALIDATION-EXT-001 (WriteToolInvalidationMap container migration + register_write_tool API via TD-S-PLUGIN-PREREQ-A-003). §Consequences "prism-query is also touched in parallel" hedging rephrased to a statement of ownership: ADR-027 scope includes SS-07; D4 Wave 1/A unblock criteria scoped to prism-spec-engine clean-pass confirmation. `subsystems_affected` updated [SS-16, SS-17] → [SS-07, SS-16, SS-17]. |
| 1.5 | 2026-05-16 | architect | prereq-e-fix-burst-9: F-LP10-HIGH-001 — POL-21 phantom-anchor closure: §D3 live-narrative `ADR-023 §VP-PLUGIN-001` → `ADR-023 §Verification Properties (VP-PLUGIN-001 bullet)`. Sibling-sweep companion site of VP-155 v0.5. |
| 1.6 | 2026-05-16 | architect | prereq-e-fix-burst-18: F-LP20-HIGH-001 — ADR-027 §D3 amended: enumerate BOTH compile-fail files (`import_custom_adapter.rs` + `import_custom_adapter_registry.rs`) matching VP-155 spec; correct "catalog grows by one entry" → "by two entries: `CustomAdapter` and `CustomAdapterRegistry`" matching VP-155 line 74 + HS-002-05 line 187 `CATALOG_SIZE=11` assertion; catalog total 9→11. Closes cross-document semantic anchor contradiction with VP-155 + BC-2.16.011 §VPs + HS-PREREQ-E-002-05. |
| 1.7 | 2026-05-16 | architect | prereq-e-fix-burst-33 (FB33): F-LP42-MED-001 — §D3 line 91 internal crate-naming contradiction resolved: replaced "perimeter-violation compile-fail test crate" with "FORBIDDEN-SYMBOLS-001 compile-fail test crate at `tests/external/no-hardcoded-sensors/`" — aligns with §D3 file paths (lines 93/101), §D3 narrative (lines 114-115), and ADR-023 canonical naming (FORBIDDEN-SYMBOLS-001 perimeter path). The two distinct compile-fail crates are: `tests/external/perimeter-violation/` (existing; BC-2.11.006 prism-query security perimeter) and `tests/external/no-hardcoded-sensors/` (FORBIDDEN-SYMBOLS-001; CustomAdapter + sensor-named type bans). F-LP42-LOW-001 — line 118 TD-VSDD-091 volatile-line-pin resolved: replaced "VP-155 line 74 and HS-PREREQ-E-002-05 line 187" with semantic-anchor form "VP-155 §Proof Method (Relationship to VP-PLUGIN-001 paragraph) and HS-PREREQ-E-002-05 §Steps" per FB32 HS-002-06 Option A precedent. |
| 1.8 | 2026-05-16 | architect | FB46: F-LP58-HIGH-001 closure: title + H1 + D2 heading rewritten to eliminate "deprecation" framing that contradicted §D1 atomic-deletion stance. §Context lead paragraph revised (1 body-prose change beyond heading rewrites) to replace "deprecation mechanism" with "atomic-deletion scope." F-LP58-MED-001 closure: §Source/Origin BC-2.16.011 bullet added for sibling-symmetry with ADR-026 v1.16 §Source/Origin BC-2.01.016 pattern. Body sweep for residual "deprecation" prose: 4 headings/title fixed; remaining occurrences are contextual (#[deprecated] rejection rationale in §D1, §Rationale, §Alternatives Considered) or BC lifecycle-field values — all left as is. |
| 1.9 | 2026-05-17 | architect | F-LP71-HIGH-001 closure: frontmatter `title:` byte-synced to H1 — drop trailing `"— Sole Escape Hatch is .prx WASM"` that was retained from pre-FB46 form. FB46 v1.8 §Changelog claimed `"title + H1 + D2 heading rewritten"` but frontmatter title was missed; within-file sibling-sweep gap survived 24 passes. POL-7 + TD-VSDD-060 + TD-VSDD-059 paper-fix closure. ARCH-INDEX row v1.8 → v1.9 propagation owned by state-manager. |
