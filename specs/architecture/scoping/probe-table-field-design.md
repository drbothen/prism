---
document_type: architecture-scoping
scope: probe_table-field-design
decision_ref: D-1260
traces_to: [BC-2.08.001]
produced_by: architect
timestamp: 2026-06-20
---

# Design: `probe_table` Sensor TOML Field

Covers all seven design dimensions requested by D-1260. Intended audience: product-owner
(BC authoring), story-writer (story decomposition), and implementer (S-5.04 TDD).

---

## 1. Schema

### Location

`probe_table` is a **top-level field on `SensorSpec`** (i.e., declared at the root
of a `*.sensor.toml` file, not inside a `[[tables]]` block).

Rationale: the probe target is a per-sensor decision — it is not a property of any
individual table. Placing it at the root of the spec alongside `sensor_id`, `name`,
and `auth_type` is semantically correct, visible at a glance, and avoids the awkward
pattern of embedding a cross-cutting concern inside one table's block.

### TOML declaration

```toml
probe_table = "alerts"   # Optional. Must match a table_name in [[tables]] if present.
```

### Type

`Option<String>` — the string value is the **unqualified table name** as declared in
a `[[tables]]` block's `table_name` field. The fully-qualified form
(`{sensor_id}.{table_name}`) is NOT used in the TOML; the spec engine qualifies it at
consumption time.

### Validation rules

1. **If `probe_table` is absent or `None`**: back-compat default behavior applies
   (see Section 3 — Fallback). No validation error.

2. **If `probe_table` is `Some(name)` AND the spec has at least one `[[tables]]` block**:
   `name` MUST case-sensitively match the `table_name` of exactly one declared table.
   Violation → new error code `E-SPEC-026` (see below).

3. **If `probe_table` is `Some(name)` AND `spec.tables` is empty**: this is also an
   `E-SPEC-026` condition — a probe_table reference with no tables to match against
   cannot be valid. Reject at parse time.

4. **Validation timing**: same parse-time pass as all other TOML validation
   (Rules 1–7 in `SpecLoader::parse`). Add as "Rule 8" in `BC-2.16.009`.

### New error code

`E-SPEC-026` (broken, validation):
```
"sensor '{sensor_id}' declares probe_table = '{name}' but no [[tables]] block
 has table_name = '{name}'. Declared tables: [{table_list}]. Remove probe_table
 or add a matching [[tables]] block."
```

`{table_list}` is the sorted, comma-separated list of declared table names (same
pattern as the timestamp_fallback_chain resolution error). This closes the
three-way alignment: BC body ↔ taxonomy row ↔ code emission.

---

## 2. Parsing — prism-spec-engine Changes

### Struct change: `spec_parser::SensorSpec`

Add one field to `SensorSpec`:

```rust
/// Health-probe table name (BC-2.08.001 probe_table).
///
/// When `Some(name)`, `name` MUST match the `table_name` of a declared [[tables]]
/// block in this spec. Validated at parse time (E-SPEC-026).
///
/// When `None`, the connectivity probe falls back to the first declared table, or
/// to a no-op if no tables are declared (backward-compatible behavior).
///
/// `#[serde(default)]` ensures existing TOML files without this field parse without error.
#[serde(default)]
pub probe_table: Option<String>,
```

`#[serde(default)]` is mandatory — all existing sensor TOML files lack this field
and must continue to parse successfully.

### `Default` impl update

`SensorSpec::default()` already provides `..` expansion for all `Option<T>` fields.
Add `probe_table: None` to the explicit `Default` impl body.

### `SensorSpec::new()` constructor

The existing `SensorSpec::new()` constructor takes positional args. Since `probe_table`
is a new optional infrastructure field (like `auth_plugin`, `file_hash`, `source_path`,
`mode`), do NOT add it to `new()`'s positional signature. It defaults to `None` via
`..Default::default()` inside the `new()` body. External callers that need to set it
use struct-literal + `..Default::default()`.

### Validation in `SpecLoader::parse()`

Add "Rule 8" after the existing Rule 7 (HTTP method whitelist), before `Ok(spec)`:

```rust
// Rule 8 (E-SPEC-026): probe_table must reference a declared table name.
if let Some(probe_name) = &spec.probe_table {
    let table_names: Vec<&str> = spec.tables.iter().map(|t| t.table_name.as_str()).collect();
    if !table_names.contains(&probe_name.as_str()) {
        let mut sorted = table_names.clone();
        sorted.sort_unstable();
        return Err(PrismError::Spec(SpecError {
            code: SpecErrorCode::ESpec026,
            message: format!(
                "sensor '{}' declares probe_table = '{}' but no [[tables]] block \
                 has table_name = '{}'. Declared tables: [{}]. Remove probe_table \
                 or add a matching [[tables]] block.",
                spec.sensor_id,
                probe_name,
                probe_name,
                sorted.join(", ")
            ),
            toml_path: Some("sensor.probe_table".to_string()),
            file_path: None,
            line_number: None,
        }));
    }
}
```

### `SpecErrorCode` enum

Add `ESpec026` variant to `prism_core::SpecErrorCode`. This is an additive change
to a `#[non_exhaustive]` enum so it is NOT a semver-breaking change.

### `#[non_exhaustive]` gate impact

`probe_table: Option<String>` is a new field on `SensorSpec`, which is already
`#[non_exhaustive]`. Adding a field to a `#[non_exhaustive]` struct does NOT trigger
a new compile-fail gate entry — the existing E0639 tests for `SensorSpec` already pass
because external code uses `..Default::default()`. **EXPECTED count stays at 79.** No
CI change required.

The `SpecErrorCode` enum is `#[non_exhaustive]` — adding `ESpec026` follows the same
pattern as all prior `ESpec0NN` additions and does not require a new gate entry either.
EXPECTED remains 79.

---

## 3. Consumption — S-5.04 `connectivity.rs` Change

### Current behavior (S-5.04 as implemented)

```rust
let probe_source_table = format!("{}_devices", adapter.sensor_type());
```

This hardcodes `{sensor_type}_devices`. For Cyberint, `sensor_type()` returns
`"cyberint"`, so `probe_source_table = "cyberint_devices"`. The `[[tables]]` blocks in
`cyberint.sensor.toml` are `alerts` and `incidents` — neither is `devices`. The
`SpecDrivenSensorAdapter::fetch()` finds no matching table, returns `Ok([])` without
making any HTTP request, and connectivity.rs classifies the result as `Up`. The probe
is semantically hollow: it claims the sensor is reachable but never actually contacted
it.

### Target behavior

`probe_connectivity` receives a `SensorSpec` reference (or `probe_table: Option<&str>`)
from the caller, reads `spec.probe_table`, and uses it:

```rust
// Resolve probe_source_table from the spec's probe_table field.
// Falls back to first declared table (if any) for sensors that don't set probe_table.
let probe_source_table: String = spec
    .probe_table
    .as_deref()
    .or_else(|| {
        // Back-compat fallback: first declared table.
        spec.tables.first().map(|t| t.table_name.as_str())
    })
    .map(|name| format!("{}.{}", spec.sensor_id, name))  // fully-qualified
    .unwrap_or_else(|| format!("{}_devices", adapter.sensor_type())); // legacy fallback
```

The `SensorSpec` referenced here is the parsed spec from `ConfigSnapshot`, not the
`prism_sensors::adapter::SensorSpec` request struct (different types — be careful
with naming at the implementation site).

### Fallback chain priority

1. `SensorSpec.probe_table` is `Some(name)` → use `{sensor_id}.{name}` (validated to exist)
2. `SensorSpec.probe_table` is `None` AND `spec.tables` is non-empty → use the first
   declared table (`spec.tables[0].table_name`)
3. `SensorSpec.probe_table` is `None` AND `spec.tables` is empty → use the current legacy
   string `format!("{}_devices", adapter.sensor_type())` — this produces the same no-op
   behavior as today (adapter::fetch finds nothing, returns Ok([]))

Rationale for fallback option 2: it is strictly better than the current legacy behavior
because at least one real table exists; using it means the probe makes an actual HTTP call.
The production-grade default demands a live probe, not a no-op.

### `probe_connectivity` signature

The function needs access to the loaded `SensorSpec` to read `probe_table`. The caller
(`SensorHealthChecker::check_one` in `health/mod.rs`) has access to the
`ConfigSnapshot` (via `ArcSwap::load()`). Pass the relevant `SensorSpec` reference —
or just `probe_table: Option<&str>` + `first_table: Option<&str>` — as a parameter.
The exact plumbing is left to the implementer; the contract is:

- `probe_connectivity` MUST use `probe_table` (if set in spec) as the probe table name.
- If unset, it MUST use `spec.tables[0]` (if tables exist).
- Legacy fallback ONLY when no tables are declared.

---

## 4. Per-Sensor `probe_table` Values

Based on actual `[[tables]]` blocks in `crates/prism-sensors/specs/*.sensor.toml`
(the canonical spec path, not `sensors/` root which has partial specs):

| Sensor      | Declared tables               | Recommended `probe_table` | Rationale |
|-------------|-------------------------------|---------------------------|-----------|
| crowdstrike | `detections`, `devices`, `incidents` | `detections` | First declared table; most consistently populated; detections is the core detection-finding surface. |
| cyberint    | `alerts`, `incidents`          | `alerts`                  | Cyberint has NO `devices` table; `alerts` is the primary query surface and the first declared table. |
| claroty     | `alerts`, `audit_logs`, `devices` | `devices`              | Claroty's primary asset intelligence surface; `devices` is the canonical IoT/OT table. |
| armis       | `devices`, `alerts`            | `devices`                 | Armis is an asset intelligence sensor; `devices` is its primary table. |

Note: the `sensors/` root directory contains partial specs (some without `[[tables]]`
sections, or with only a subset of columns). The canonical full specs are in
`crates/prism-sensors/specs/`. The `probe_table` field in the canonical specs governs.

For sensors with no declared tables (e.g., credential-only fixture specs), omit
`probe_table` — the legacy fallback applies and the probe is a no-op (acceptable for
credential-only specs).

---

## 5. BC Implications

### No new BC required; amend BC-2.08.001

The existing `BC-2.08.001: On-Demand Connectivity Check Per Sensor Per Client` governs
connectivity probe behavior. Product-owner should amend it (version bump) to add:

**In Preconditions:**
- "The sensor spec declares a `probe_table` field or has at least one declared table"

**In Postconditions (add):**
- "The probe routes the `LIMIT 0` request to the table named by `probe_table` in the
  sensor spec. If `probe_table` is absent, routes to the first declared table. If no
  tables are declared, the probe is a structural no-op returning `status: Up`."
- "Probes against sensors with no declared read tables (empty `spec.tables` and no
  `probe_table`) are accepted but guaranteed not to make HTTP contact; `Up` reflects
  only that the adapter was reachable by the runtime, not that the sensor API was
  contacted."

**In Error Cases (add row):**
| `E-SPEC-026` | `probe_table` names a table not in `[[tables]]` | Spec load rejected; sensor not registered; no probe attempted |

**New Invariant (add):**
- "The probe table MUST be a table declared in the sensor spec when `probe_table` is
  explicitly set (enforced at parse time via E-SPEC-026; invariant cannot be violated
  at runtime)."

**Also amend `BC-2.16.009`** (sensor spec validation rules) to add Rule 8 for
`E-SPEC-026` parallel to the existing rules 1–7.

---

## 6. Story Decomposition

### Recommendation: standalone story `S-5.04-PROBE-TABLE`

**Rationale for separation from S-5.04:** `S-5.04` is already in an open worktree on
branch `feature/S-5.04` with implemented code that depends on the current
`probe_source_table = format!("{}_devices", ...)` pattern. Introducing `probe_table`
as a new field modifies:
- `prism-spec-engine` (new field on `SensorSpec`, new validation rule, new error code)
- `crates/prism-sensors/specs/*.sensor.toml` (4 files) and potentially `sensors/*.sensor.toml`
- `prism-mcp` health module (consumption of the new field)

These span two crates beyond `prism-mcp`, and the `prism-spec-engine` change is a
prerequisite for the consumption change. Embedding this in S-5.04 mid-delivery creates
scope expansion in an in-flight story with an active cascade.

**Recommended approach:** Create `S-5.04-PROBE-TABLE` (story ID suggestion) as a
sequential dependency after S-5.04 closes. S-5.04 can merge at its current state (the
no-op probe is a known, bounded deficiency, not a correctness regression — it reported
`Up` before and continues to do so; it does not falsely report `Down`). The probe_table
feature makes it correct; that is appropriately a follow-on scope unit.

### Story spec outline: `S-5.04-PROBE-TABLE`

```yaml
story_id: S-5.04-PROBE-TABLE
title: "probe_table field: spec schema + health probe routing"
wave: 5           # follows S-5.04
points: 1         # ~1pt: additive field + validation + 4 TOML files + consumption site
depends_on: [S-5.04, S-1.11]
crates_touched: [prism-spec-engine, prism-sensors, prism-mcp]
behavioral_contracts: [BC-2.08.001, BC-2.16.009]
```

**Acceptance Criteria outline:**

- AC-1: `SensorSpec` in `spec_parser.rs` gains `probe_table: Option<String>` with
  `#[serde(default)]`; `Default` impl sets it to `None`.
- AC-2: `SpecLoader::parse()` validates `probe_table` against declared table names when
  present; violation emits `E-SPEC-026` with the canonical message template; spec is
  rejected (DI-030 fail-fast).
- AC-3: `error-taxonomy.md` gains an `E-SPEC-026` row with the canonical message
  template. `SpecErrorCode::ESpec026` variant added to `prism-core`.
- AC-4: All four canonical sensor specs in `crates/prism-sensors/specs/` gain a
  `probe_table` declaration: crowdstrike→`detections`, cyberint→`alerts`,
  claroty→`devices`, armis→`devices`. `sensors/` root partial specs do NOT add
  `probe_table` (they are incomplete and not the canonical source).
- AC-5: `probe_connectivity()` in `prism-mcp/src/health/connectivity.rs` reads the
  `probe_table` field from the loaded `SensorSpec` and uses it as `probe_source_table`
  (with fallback chain per Section 3).
- AC-6: Red Gate tests verify: (a) E-SPEC-026 fires for a mismatched `probe_table`;
  (b) a correctly set `probe_table` reaches the adapter's fetch path (unit test with
  mock adapter); (c) absence of `probe_table` falls back to first declared table (unit
  test).
- AC-7: BC-2.08.001 and BC-2.16.009 amended by product-owner per Section 5 above;
  version bumped.

### Points estimate: 1pt

Scope: one new `Option<String>` field, one validation rule, one error code, four TOML
lines across four files, and one consumption-site change. No new public types, no
semver-breaking changes, no new `#[non_exhaustive]` gate entries. Total estimated
effort ~3–5 hours implementation + tests.

---

## 7. Crate-Conflict and Parallelism Assessment

### Crates touched

| Crate | Change | Why |
|-------|--------|-----|
| `prism-spec-engine` | `SensorSpec.probe_table`, `SpecLoader::parse()` Rule 8, `E-SPEC-026` | Schema + validation owner |
| `prism-core` | `SpecErrorCode::ESpec026` variant | Error code enum lives here |
| `prism-sensors` | `crates/prism-sensors/specs/*.sensor.toml` (4 files) | Canonical sensor specs |
| `prism-mcp` | `health/connectivity.rs` probe_source_table resolution | Consumption site |

### Conflict with in-flight work

**001-A [prism-mcp]**: Active worktree on `feature/001-A`. The `prism-mcp` changes in
`S-5.04-PROBE-TABLE` touch `health/connectivity.rs` only. Review 001-A's diff scope to
confirm no overlap. If 001-A does not touch `health/connectivity.rs`, the two stories
can be sequenced (001-A merge → S-5.04 merge → S-5.04-PROBE-TABLE). If 001-A does
touch `health/`, sequence S-5.04-PROBE-TABLE strictly after 001-A merges.

**001-B [prism-core, prism-query, prism-mcp]**: The `prism-core` change adds one enum
variant. Assuming 001-B is not currently adding `SpecErrorCode` variants, there is no
merge conflict. If 001-B is also touching `SpecErrorCode`, coordinate the variant
addition to avoid a diff conflict on the same enum definition.

**S-5.04 [prism-mcp]**: S-5.04-PROBE-TABLE strictly sequenced after S-5.04 merges.
S-5.04 owns `health/connectivity.rs`; S-5.04-PROBE-TABLE modifies the same file.
Sequential dependency is hard — do not attempt to parallelize.

**S-1.11**: S-5.04-PROBE-TABLE depends on S-1.11 (already a dependency of S-5.04) for
the same reason: without declared read tables in the canonical sensor specs, the
`probe_table` validation and fallback behavior are partially vacuous. S-1.11 MUST merge
before S-5.04-PROBE-TABLE.

### Recommended sequencing

```
S-1.11 → 001-A → 001-B → S-5.04 → S-5.04-PROBE-TABLE
```

S-5.04-PROBE-TABLE is NOT on the critical demo path. It is a correctness hardening of
the health probe. It should be scheduled in Wave 5 after S-5.04 closes, ideally in the
same wave batch to avoid the probe remaining hollow across a wave boundary.
