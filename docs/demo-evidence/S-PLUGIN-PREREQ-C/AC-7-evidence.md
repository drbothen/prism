# AC-7 Evidence — `SensorIdValidationError` Crate-Root Re-export

**Story:** S-PLUGIN-PREREQ-C v1.3
**Status: SATISFIED**
**Resolves:** TD-S-PLUGIN-PREREQ-A-008 (P3)
**BC anchor:** BC-2.01.013 postcondition — the spec-driven sensor identifier surface exposes a
consistent error type at the crate root; ergonomic parity with `SensorId` which is already
re-exported at `prism_core::SensorId` per PREREQ-A.

---

## AC Summary (quoted from story v1.3)

> `SensorIdValidationError` is re-exported at the `prism_core` crate root, making it
> accessible via `use prism_core::SensorIdValidationError;` instead of the currently
> required `use prism_core::sensor_id::SensorIdValidationError;`.
>
> **Red Gate test:** A doctest in `crates/prism-core/src/lib.rs` on the re-export line
> demonstrates the type is accessible at crate root AND can be matched on its variants.
> `cargo test --doc -p prism-core` must pass.

---

## Red Gate Test

**File:** `crates/prism-core/src/lib.rs`
**Mechanism:** Doctest on the `pub use sensor_id::SensorIdValidationError;` re-export line

The doctest:
1. Imports `prism_core::SensorIdValidationError` from the crate root (not from the submodule path)
2. Constructs a `SensorIdValidationError::TooShort` value directly (verifying the variant is
   accessible at the imported path)
3. Matches on the error value with an exhaustive arm for `TooShort`, asserting true

This is a non-tautological doctest (HIGH-008 closure) — it exercises a specific variant rather
than wrapping in `Option<SensorIdValidationError>`. The match arm provides concrete evidence
that the error type is usable, not merely name-accessible.

---

## Real Test Output

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && cargo test --doc -p prism-core

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
   Doc-tests prism_core

running 3 tests
test crates/prism-core/src/sensor_id.rs - sensor_id::SensorId (line 38) ... ignored
test crates/prism-core/src/sensor_id.rs - sensor_id::SensorId::try_from_str (line 305) ... ignored
test crates/prism-core/src/lib.rs - SensorIdValidationError (line 147) ... ok

test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.64s
```

The `SensorIdValidationError` doctest at line 147 of `lib.rs` passes. The two ignored tests
are pre-existing sensor_id module-level doctests that are marked ignored (not relevant to AC-7).

---

## Production Code Reference

**File:** `crates/prism-core/src/lib.rs`

The re-export appears immediately below the existing `pub use sensor_id::SensorId;` line,
establishing ergonomic parity:

```
pub use sensor_id::SensorId;

// S-PLUGIN-PREREQ-C: AC-7 — SensorIdValidationError at crate root for ergonomic parity with SensorId.
/// Re-export of [`sensor_id::SensorIdValidationError`] for ergonomic external use.
/// ...
pub use sensor_id::SensorIdValidationError;
```

The re-export is a one-line addition. No changes to `sensor_id.rs` were required — the type
was already public in the submodule; only the crate-root re-export was missing.

**AC-7 resolves the ergonomic gap identified in TD-S-PLUGIN-PREREQ-A-008:** external crates
(including the upcoming PLUGIN-MIGRATION-001-B dispatch-site conversions) can now write
`use prism_core::SensorIdValidationError;` in pattern parity with `use prism_core::SensorId;`,
eliminating the two-level path that required knowledge of the internal `sensor_id` submodule.

---

## Cross-References

- HIGH-008 closure: upgraded doctest from `Option<SensorIdValidationError>` tautology to direct
  `SensorIdValidationError::TooShort` construction + match (non-tautological evidence)
- PREREQ-A precedent: `SensorId` re-export at crate root (PR #142) — AC-7 follows the identical
  pattern for the companion error type
- TD-S-PLUGIN-PREREQ-A-008: original finding from PREREQ-A adversary PR-level pass (F-LP4-OBS-003)
  noting the missing crate-root re-export
- BC-2.01.013 v1.6: sensor identifier surface ergonomics postcondition
- PLUGIN-MIGRATION-001-B (Wave 1, blocked on PREREQ-C merge): sensor-name dispatch site
  conversions that consume `SensorIdValidationError` at the crate-root path
