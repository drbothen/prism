---
document_type: verification-property
level: L4
version: "0.4"
status: draft
producer: architect
timestamp: 2026-05-15T00:00:00Z
phase: prereq-e
inputs:
  - .factory/specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md
input-hash: "[pending-recompute]"
traces_to: .factory/specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md
source_bc: BC-2.16.011
source_adr: ADR-027
source_invariant: null
module: prism-spec-engine
priority: P0
proof_method: integration_test
verification_method: integration_test
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: draft
introduced: "2026-05-15"
modified: "2026-05-16"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-155: CustomAdapter Absent from prism-spec-engine Public API (Compile-Fail Perimeter)

## Property Statement

After S-PLUGIN-PREREQ-E merges, the symbol `CustomAdapter` MUST NOT be accessible via
`prism_spec_engine::CustomAdapter` or any re-export path from the `prism-spec-engine` crate.
Any attempt to import `CustomAdapter` from `prism_spec_engine` MUST produce a compile error
(`error[E0432]: unresolved import`).

This property is enforced by a compile-fail test file in the `tests/external/no-hardcoded-sensors/`
perimeter crate (ADR-023 §VP-PLUGIN-001 enforcement mechanism 1). The compile-fail test joins
the FORBIDDEN-SYMBOLS-001 catalog and is counted by the CI file-count assertion.

Similarly, `CustomAdapterRegistry` MUST NOT be importable from `prism_spec_engine`. One
compile-fail file per symbol is required (two files total: `import_custom_adapter.rs` and
`import_custom_adapter_registry.rs`).

## Source Contract

- **BC:** BC-2.16.011 (CustomAdapter Rust Trait Retirement) — INV-ADAPTER-RETIRE-002 (`prism-spec-engine` crate public API does NOT expose any type, trait, or function from the retired `custom_adapter` module). BC-2.16.011 §VP Anchors explicitly lists VP-155 as the verification mechanism for INV-ADAPTER-RETIRE-002.
- **Supporting ADR:** ADR-027 §Decision (CustomAdapter deletion mandate) — D3 specifies the compile-fail perimeter enforcement mechanism that VP-155 implements. ADR-023 §VP-PLUGIN-001 establishes the FORBIDDEN-SYMBOLS-001 catalog and perimeter pattern.
- **Module:** prism-spec-engine (the crate whose public API is under test)
- **Category:** API Surface Enforcement / Perimeter

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| integration_test | compile-fail test (trybuild or compiletest) | Yes — binary compile outcome | Absence of CustomAdapter and CustomAdapterRegistry from prism-spec-engine public API |

**Feasibility:** Compile-fail tests are the canonical mechanism for enforcing "type X is not
importable." The existing perimeter at `tests/external/perimeter-violation/` demonstrates this
pattern works in the Prism workspace. Extending it with two new files is mechanical.

**Relationship to VP-PLUGIN-001 (VP-146):** VP-146 asserts that the 9 FORBIDDEN-SYMBOLS-001
sensor-named types (CrowdStrikeAdapter, etc.) are absent from production code. VP-155 asserts
that `CustomAdapter` and `CustomAdapterRegistry` are absent from the `prism-spec-engine` public
API specifically. VP-155 is a new FORBIDDEN-SYMBOLS-001 catalog entry (catalog grows from 9 to
11 entries — `CustomAdapter` and `CustomAdapterRegistry`). The CI file-count assertion must be
updated to reflect 11 entries when VP-155's files are added.

## Proof Harness Skeleton

```rust
// File 1:
// tests/external/no-hardcoded-sensors/import_custom_adapter.rs
//
// VP-155: CustomAdapter must not be importable from prism-spec-engine public API.
// DO NOT REMOVE — compile-fail gate for ADR-027.
//
// Asserting that `use prism_spec_engine::CustomAdapter` produces E0432.
// This file must remain in this perimeter crate permanently.
//
use prism_spec_engine::CustomAdapter; //~ ERROR unresolved import

// File 2:
// tests/external/no-hardcoded-sensors/import_custom_adapter_registry.rs
//
// VP-155: CustomAdapterRegistry must not be importable from prism-spec-engine public API.
// DO NOT REMOVE — compile-fail gate for ADR-027.
//
use prism_spec_engine::CustomAdapterRegistry; //~ ERROR unresolved import
```

**CI count-assertion update (required in PLUGIN-MIGRATION-001-A scope):**

The CI step that asserts `file_count(tests/external/no-hardcoded-sensors/) == CATALOG_SIZE`
must be updated from 9 to 11. The two new files are:
- `import_custom_adapter.rs`
- `import_custom_adapter_registry.rs`

If the CI assertion uses a hardcoded constant, update that constant. If it reads from a manifest
file (e.g., `.factory/forbidden-symbols-catalog.toml`), add the two new entries there.

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Binary | Compile: yes or no |
| Proof complexity | Very low | Two compile-fail files, zero runtime logic |
| Tool support | Full | compiletest or trybuild; same tooling as existing perimeter-violation tests |
| Harness dependencies | Low | Requires PREREQ-E to have deleted custom_adapter.rs first; otherwise the test would fail with "expected to fail but compiled successfully" |
| Estimated proof time | <5 seconds (compile only) | Fast — no test execution overhead |

**Sequencing note:** VP-155's compile-fail files must NOT be added to the perimeter crate
until PREREQ-E merges and deletes `custom_adapter.rs`. Adding them before the deletion would
cause CI to fail with "expected compilation failure but compilation succeeded" on the current
codebase. The correct sequencing is: PREREQ-E merges (deletes the type) → PLUGIN-MIGRATION-001-A
adds the compile-fail files → CI count assertion is updated in the same commit.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-15 | architect (PREREQ-E ADR burst) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | plugin-prereq-e-adr-burst | 2026-05-15 | architect | Initial stub. Traces to ADR-027 D3. Two compile-fail files required (CustomAdapter + CustomAdapterRegistry). Catalog grows from 9 to 11. Authoring in PLUGIN-MIGRATION-001-A scope; MUST sequence after PREREQ-E merge. Priority P0. |
| 0.2 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `plugin-prereq-e` was informal slug; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 0.3 | fix-burst-5 renumber-repair-redo | 2026-05-15 | state-manager | F-LP5-HIGH-003 renumber-repair-redo. FB4 assigned both the changelog-repair row and the modified-field-sync row to v0.2, producing two rows at the same version and violating monotonic strict order. Repair row renumbered 0.2→0.3. Absorbs FB4 modified-field-sync content: `modified:` field confirmed synced to ISO date "2026-05-15" per F-LP4-LOW-002 / POL-27 (most recent change: state-manager POL-20 catch in fix-burst-1). Content summary retained: prior changelog had duplicate 0.1 entries (architect adr-burst + state-manager catch both labeled 0.1); state-manager catch correctly renumbered to 0.2. Each distinct content change now holds a unique version. Frontmatter version updated to 0.3. Monotonic sequence verified: 0.1 → 0.2 → 0.3. |
| 0.4 | prereq-e-fix-burst-6 | 2026-05-16 | architect | F-LP6-HIGH-001 — `source_bc` set to BC-2.16.011 (was null; BC-2.16.011 §VP Anchors explicitly lists VP-155 as enforcing INV-ADAPTER-RETIRE-002; same defect class as F-LP1-CRIT-001 VP-154 source_bc fix in FB1; sibling-sweep miss). §Source Contract rewritten: leads with BC-2.16.011 INV-ADAPTER-RETIRE-002 ownership; ADR-027 §Decision demoted to supporting reference. Bidirectional traceability symmetry restored (BC-2.16.011 claims VP-155 ↔ VP-155 now claims BC-2.16.011). |
