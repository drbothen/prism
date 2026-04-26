# Red Gate Log — S-2.08 Event Table Abstraction and Local Buffering

**Story:** S-2.08 v1.6 — prism-sensors: Event Table Abstraction and Local Buffering
**Dispatch:** test-writer
**Date:** 2026-04-26
**Branch:** feature/S-2.08-event-tables

---

## Operation A — Stub Realignment (v1.5 → v1.6)

### Commit: `8885685d`
**Title:** `refactor(S-2.08): realign stubs to v1.6 (TableType→prism-core, SensorQueryDescriptor→prism-query/types.rs)`

### Defect 1 Fix (SensorQueryDescriptor)
- Created `crates/prism-query/src/types.rs` with canonical `SensorQueryDescriptor` struct
- Updated `prism-query/src/lib.rs` to `pub mod types;`
- Updated `prism-query/src/materialization.rs` to import `crate::types::SensorQueryDescriptor`
- Removed the v1.5 local `SensorQueryDescriptor` definition from `materialization.rs`

### Defect 2 Fix (TableType canonical home)
- Created `crates/prism-core/src/table_type.rs` as single canonical `TableType` enum
- Updated `prism-core/src/lib.rs`: `pub mod table_type; pub use table_type::TableType;`
- Renamed `crates/prism-sensors/src/table_type.rs` → `crates/prism-sensors/src/table_dispatch.rs`
  - `table_dispatch.rs` re-exports `pub use prism_core::TableType;` (no local definition)
  - Retains `TableTypeRouteDecision` + `route_table_query` stub
- Updated `prism-sensors/src/lib.rs`: `pub mod table_dispatch;` + re-export
- Removed local `TableType` enum from `prism-spec-engine/src/spec_parser.rs`
- Added `use prism_core::{..., TableType};` import in `spec_parser.rs`
- Updated `prism-spec-engine/src/lib.rs`: removed `TableType` from spec_parser re-export, added `pub use prism_core::TableType;`

**`cargo check --workspace` after Operation A: PASS**

---

## Operation B — Red Gate Test Suite

### Commit: (Operation B)

### Test Files Written

| File | Tests | RED | GREEN-BY-DESIGN |
|------|-------|-----|-----------------|
| `crates/prism-core/src/tests/table_type_tests.rs` | 16 | 0 | 16 |
| `crates/prism-query/src/tests/materialization_tests.rs` | 12 | 12 | 0 |
| `crates/prism-sensors/src/tests/event_buffer_tests.rs` | 17 | 16 | 1 |
| `crates/prism-sensors/src/tests/poller_tests.rs` | 22 | 11 | 11 |
| `crates/prism-sensors/src/tests/table_dispatch_tests.rs` | 8 | 4 | 4 |
| `crates/prism-spec-engine/tests/bc_2_16_table_type_test.rs` | 20 | 11 | 9 |
| **TOTAL** | **95** | **54** | **41** |

Note: `poller_tests.rs` includes 2 `start_pollers` tests that are RED, plus the async `run()` test.
Final run counts: 50 RED / 38 GREEN (some tests within prism-sensors lib are shared with baseline).

### Red Gate Results

```
prism-sensors --lib:  68 PASS (includes 13 NEW GREEN-BY-DESIGN) | 27 FAIL RED (all todo!() panics)
prism-query --lib:     0 PASS | 12 FAIL RED (all todo!() panics)
prism-spec-engine bc_2_16_table_type:  9 PASS (GREEN-BY-DESIGN) | 11 FAIL RED (todo!() panics)
prism-core --lib:     16 PASS (all GREEN-BY-DESIGN) | 0 FAIL
```

**Total workspace:** 1318 PASS / 50 FAIL (--no-fail-fast)
**Baseline was:** 1276 PASS / 0 FAIL
**New tests added:** 42 GREEN + 50 RED = 92 new tests

**S-2.08-specific RED ratio: 50/92 = 54.3%** — exceeds the 50% minimum target.

### GREEN-BY-DESIGN Exceptions (documented)

The following test categories are GREEN-BY-DESIGN (fully implemented in stubs) and
do not violate the Red Gate:

1. **`prism-core/table_type_tests.rs` (16 tests):** `TableType` enum variants, `as_str()`,
   `Display`, `Default`, serde round-trip, `Copy`, `Hash`. These are pure enum→string
   mappings with no business logic — stub author correctly marked GREEN-BY-DESIGN.

2. **`prism-sensors/table_dispatch_tests.rs` variants (4 tests):** `TableTypeRouteDecision`
   enum variant existence checks. The variants are defined (no todo!()), only `route_table_query`
   is todo!().

3. **`prism-sensors/poller_tests.rs` constructor/accessor (9 tests):** `EventPoller::new`,
   `EventPoller::id()`, `EventPoller::debug`, `PollerId` display/equality/hash,
   `PollerStatus` variants, `PollerDiagnostics` field accessibility. All fully implemented.

4. **`prism-spec-engine/bc_2_16_table_type_test.rs` TOML parse (9 tests):** `SpecLoader::parse`
   TOML deserialization for `table_type`, `poll_interval_secs`, `retention_secs` fields
   is driven by `serde` and is already working (struct + `#[serde(default)]` derive).
   The import checks are compile-time structural tests.

5. **`event_buffer_tests.rs` slash rejection (1 test):** This test expects `write_events`
   to return `Err` when sensor_id contains '/'. With todo!() stub it will panic RED
   rather than return Err — this test is effectively RED even though it expects Err.

### Failure Mode Verification

All 50 failing tests fail via `todo!()` panic with `"not yet implemented: ..."` messages
that include the AC/BC reference in the todo!() text. None fail due to compile errors or
logic assertion failures. This is correct Red Gate failure mode.

Example failures:
- `crates/prism-sensors/src/event_buffer.rs:77:9: not yet implemented: AC-2 / AC-4: implement RocksDB CF batch write...`
- `crates/prism-sensors/src/poller.rs:160:9: not yet implemented: Task-8: return PollerDiagnostics snapshot...`
- `crates/prism-query/src/materialization.rs:41:5: not yet implemented: AC-9 / AC-10: implement _source_type injection...`
- `crates/prism-spec-engine/src/spec_parser.rs:202:9: not yet implemented: AC-7 / EC-002: validate TableSpec...`
- `crates/prism-sensors/src/table_dispatch.rs:50:5: not yet implemented: AC-2 / AC-3 / AC-8: implement table-type routing...`

### Production todo!() Count

12 production `todo!()` stubs — unchanged from stub baseline at commit `3c6f77be`:

| File | Todo | AC Reference |
|------|------|-------------|
| `prism-sensors/src/event_buffer.rs` | `write_events` | AC-2/AC-4 |
| `prism-sensors/src/event_buffer.rs` | `scan_events` | AC-2 |
| `prism-sensors/src/event_buffer.rs` | `evict_expired` | AC-4 |
| `prism-sensors/src/event_buffer.rs` | `has_data` | AC-5 |
| `prism-sensors/src/event_buffer.rs` | `buffer_size_bytes` | Task-8 |
| `prism-sensors/src/poller.rs` | `run` | AC-1/AC-4/AC-6 |
| `prism-sensors/src/poller.rs` | `diagnostics` | Task-8 |
| `prism-sensors/src/poller.rs` | `start_pollers` | AC-1 |
| `prism-sensors/src/table_dispatch.rs` | `route_table_query` | AC-2/AC-3/AC-8 |
| `prism-sensors/src/fanout.rs` | `dispatch_by_table_type` | AC-2/AC-3/AC-5/AC-8 |
| `prism-query/src/materialization.rs` | `inject_source_type` | AC-9/AC-10 |
| `prism-spec-engine/src/spec_parser.rs` | `validate_table_spec` | AC-7/EC-002 |

### Architecture Compliance Verification

- **No DataFusion in prism-query:** Confirmed. `prism-query/Cargo.toml` deps: prism-core, prism-storage, prism-spec-engine, serde, serde_json only.
- **TableType single canonical home:** Confirmed. Only defined in `prism-core/src/table_type.rs`. prism-sensors and prism-spec-engine both import via `prism_core::TableType`. The `test_BC_2_08_table_type_from_spec_engine_matches_prism_core` test validates this at compile time.
- **SensorQueryDescriptor distinct from InternalTableDescriptor:** Confirmed. SensorQueryDescriptor lives in `prism-query/src/types.rs` (sensor routing context). InternalTableDescriptor remains in `prism-core` (internal RocksDB tables).

### Compile / Clippy / Fmt Status

| Check | Status |
|-------|--------|
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets` | PASS (no errors) |
| `cargo fmt --all -- --check` | PASS |
| `cargo test --workspace --no-fail-fast` | 1318 PASS, 50 FAIL (all new RED tests) |

### Spec Gaps / Concerns

1. **EC-005 (hot-reload poller lifecycle):** No tests written for hot-reload poller
   cancellation and re-spawn. EC-005 requires CancellationToken cancellation when spec
   is hot-reloaded. This is out of scope for this dispatch (hot-reload is S-1.12/S-3.13)
   but the cancellation mechanism is already stubbed in `EventPoller`.

2. **AC-6 (HTTP 429 WARN + continue):** No test written for the "poller logs WARN and
   continues on API error" behavior. This requires a mock `SensorAdapter` that returns
   errors, which depends on S-2.06 adapter interfaces. Flagged for implementer: must
   verify AC-6 behavior during implementation, then add a specific AC-6 test.

3. **`event_buffer_tests.rs` slash rejection test** is structurally RED (todo!() panics)
   but the test expectation (`match result { Err(_) => {} Ok(_) => panic!() }`) means
   when implemented, the implementer must choose: return Err, or let the slash produce
   malformed keys silently. The test documents the expectation. Implementer should decide.
