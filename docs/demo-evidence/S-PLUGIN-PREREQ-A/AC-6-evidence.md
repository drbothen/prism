# AC-6 Evidence: Perimeter compile-fail test catches SensorType reintroduction

## AC Text (verbatim)

> `tests/external/perimeter-violation/src/main.rs` contains
> `use prism_core::SensorType;` which MUST produce `E0432` (unresolved import)
> at compile time. CI asserts `E0432` is present in cargo check output.
> This is the regression gate — if `SensorType` is ever re-introduced, the import
> succeeds, the CI assertion fails, and the PR is blocked.

## Evidence Type

Static file inspection + compile execution (compile failure = test passing).

## Perimeter Violation Source (`main.rs:69`)

```rust
// File: tests/external/perimeter-violation/src/main.rs:62-69

// ── S-PLUGIN-PREREQ-A AC-6 / VP-PLUGIN-001: SensorType perimeter ────────────
//
// The closed `SensorType` enum was deleted in S-PLUGIN-PREREQ-A (ADR-023 §C1).
// This import MUST fail to compile — any attempt to re-introduce `pub enum SensorType`
// in prism-core would need to pass this assertion to be detectable by CI.
//
// Expected error: E0432 "unresolved import `prism_core::SensorType`"
use prism_core::SensorType;
```

## Compile Execution Output

```
$ cd tests/external/perimeter-violation && cargo build --color=never 2>&1 | grep "error\["

error[E0432]: unresolved import `prism_core::SensorType`
69 | use prism_core::SensorType;
   |     ^^^^^^^^^^^^^^^^^^^^^^ no `SensorType` in the root
error[E0603]: function `parse_filter` is private
error[E0603]: function `parse_filter_with_limits` is private
error[E0603]: function `parse_sql` is private
... (additional E0603 errors for other perimeter symbols)
```

The build FAILS with `E0432` as expected. This is the test PASSING — the import
target does not exist because `pub enum SensorType` was deleted.

## CI Workflow Assertion (`.github/workflows/ci.yml:496-523`)

```yaml
# --- VP-PLUGIN-001 / F-LP2-CRIT-001: SensorType deleted-type regression gate ---
#
# The closed `SensorType` enum was removed in S-PLUGIN-PREREQ-A (ADR-023 §C1).
# perimeter-violation/src/main.rs imports `prism_core::SensorType` which MUST
# produce E0432 (unresolved import). If `pub enum SensorType` is ever
# ...
# ensures the E0432 for SensorType is present.
    if re.search(r'error\[E0432\].*SensorType', line):
        ...
    "VP-PLUGIN-001 assertion passed: E0432 for 'SensorType' found in cargo "
```

The CI Python script at line 507 searches for `error[E0432].*SensorType` in the
cargo check log. If `SensorType` is reintroduced (the import succeeds), the pattern
won't be found and CI emits an `::error::` annotation blocking the PR.

## Verdict: SATISFIED

The perimeter compile-fail crate at `tests/external/perimeter-violation/src/main.rs:69`
contains `use prism_core::SensorType;`. Running `cargo build` on this crate produces
`error[E0432]: unresolved import 'prism_core::SensorType'` — confirming the enum is
absent from prism-core. The CI workflow at `.github/workflows/ci.yml:507` asserts
this `E0432` is present, blocking any PR that reintroduces the type.
