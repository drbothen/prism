# Demo Evidence — S-1.01: Foundational Types

**Story:** [S-1.01 — prism-core: Foundational Types](../../../../.factory/stories/S-1.01-foundational-types.md)
**Policy:** POL-010 (per-AC demo evidence)
**Crate:** `prism-core` (library crate, no CLI binary)
**Test state:** 43/43 pass at commit `dbc5ee1`

## What This Is

`prism-core` is a pure library — there is no binary to run. Demo evidence takes the form
of VHS recordings showing `cargo test` runs scoped to individual integration test binaries.
Each binary name matches its AC: `cargo test --test ac_1_tenant_id_rejects_empty -p prism-core`.

## Files

| File | AC | Format |
|------|----|--------|
| `AC-1-tenant-id-rejects-empty.{gif,webm,tape}` | AC-1 | VHS recording |
| `AC-2-tenant-id-valid-input.{gif,webm,tape}` | AC-2 | VHS recording |
| `AC-3-tenant-id-rejects-path-traversal.{gif,webm,tape}` | AC-3 | VHS recording |
| `AC-4-storage-domain-all-16.{gif,webm,tape}` | AC-4 | VHS recording |
| `AC-5-prism-error-display.{gif,webm,tape}` | AC-5 | VHS recording |
| `AC-6-kani-proof-vp001.md` | AC-6 | Placeholder — Phase 5 |
| `AC-7-tenant-id-serde-round-trip.{gif,webm,tape}` | AC-7 | VHS recording |
| `AC-8-AC-9-tenant-id-boundary.{gif,webm,tape}` | AC-8, AC-9 | VHS recording |
| `evidence-report.md` | All | Full coverage map |

## Reproduction

Any tape can be replayed from the worktree root:

```bash
cd /path/to/.worktrees/S-1.01-foundational-types
vhs docs/demo-evidence/S-1.01/AC-1-tenant-id-rejects-empty.tape
```

Or run the test directly without VHS:

```bash
cargo test --test ac_1_tenant_id_rejects_empty -p prism-core
cargo test --test ac_2_tenant_id_valid_input -p prism-core
cargo test --test ac_3_tenant_id_rejects_path_traversal -p prism-core
cargo test --test ac_4_storage_domain_all_16 -p prism-core
cargo test --test ac_5_prism_error_display -p prism-core
cargo test --test ac_7_tenant_id_serde_round_trip -p prism-core
cargo test --test ac_8_ac_9_tenant_id_boundary -p prism-core
```

## AC-6 (Kani / VP-001)

Not run here. Formal verification runs in Phase 5. Command:

```bash
cargo kani --proof verify_tenant_id_validation -p prism-core --features kani
```

See `AC-6-kani-proof-vp001.md` for full details.
