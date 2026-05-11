# AC-2 Evidence: pub enum SensorType DELETED from prism-core/src/types.rs

## AC Text (verbatim)

> `pub enum SensorType` is deleted from `prism-core/src/types.rs` (and from all
> production source under `crates/*/src/`). Only doc comments (mentioning the
> historical enum name) and the intentional perimeter-violation import remain.

## Evidence Type

`grep` across all production source files under `crates/*/src/`.

## Command and Output

```
$ grep -rn "pub enum SensorType" crates/*/src/
(no output — exit code 1)
```

Zero matches for the enum declaration in any production source file.

```
$ grep -rn "SensorType" crates/*/src/ | grep -v "/tests/"
crates/prism-core/src/sensor_id.rs:3://! Replaces the closed `SensorType` enum (ADR-023 §C1, BC-2.01.013).
crates/prism-core/src/sensor_id.rs:10://! # VP: VP-PLUGIN-001 — SensorId open-newtype replaces SensorType closed enum
crates/prism-core/src/sensor_id.rs:19:/// Replaces the closed `SensorType` enum per ADR-023 §C1 + BC-2.01.013.
crates/prism-sensors/src/adapter.rs:306:    /// closed `SensorType` enum. Implementations typically return
```

Only 4 hits, all doc comments (prefixed with `//!` or `///`). No code references to
`SensorType` remain in production source outside of test fixtures.

The `tests/external/perimeter-violation/src/main.rs:69` reference
(`use prism_core::SensorType;`) is the intentional compile-fail test — that file
is under `tests/external/`, not `crates/*/src/`.

## Verdict: SATISFIED

`pub enum SensorType` does not exist anywhere in `crates/*/src/` production code.
The only references are doc comments describing the historical migration. The
closed enum has been fully deleted per ADR-023 §C1.
