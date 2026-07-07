---
document_type: feature-design-map
title: "PrismQL Case-Insensitive Operators — Feature Design Map for S-PRISMQL-CASE-INSENSITIVE-001"
story: S-PRISMQL-CASE-INSENSITIVE-001
adr: ADR-047
status: draft
date: "2026-06-27"
producer: architect
subsystems_affected: [SS-11, SS-02]
demo_critical: true
traces_to: ADR-047
---

# Feature Design Map: PrismQL Case-Insensitive Operators

Story `S-PRISMQL-CASE-INSENSITIVE-001` (DEMO-CRITICAL, T13 demo).

This document maps the **complete change surface** for implementing ADR-047. It is the
input the story-writer uses to produce acceptance criteria and the implementer uses to
scope the TDD red-gate tests. It does NOT specify code; it specifies what changes and
where.

---

## Change Surface Overview

| Area | Scope | Crate | Files |
|------|-------|-------|-------|
| A. Grammar | Add IEQ/IIN/INE keyword alternatives | prism-query | `filter_parser.rs` |
| B. AST | Add `case_insensitive: bool` to Predicate::Compare and Predicate::In | prism-query | `ast.rs` |
| C. Emitter | Branch on `case_insensitive` to emit `lower()=lower()` | prism-query | `pipe_sql_emitter.rs` |
| D. Round-trip normalizer | Extend PQL normalizer and op-table to handle IEQ/IIN | prism-query | `ast.rs` (normalizer section) |
| E. Adapter normalization | Canonicalize OCSF enum-label fields at DTU/adapter output | prism-ocsf | `enum_map.rs`, sensor normalizer |
| F. Discoverability | Add IEQ/IIN examples to grammar resource and `prism describe` | prism-query, prism-mcp | auto-generated grammar resource |
| G. Tests (TDD red gate) | Failing integration tests written before implementation | prism-query | `tests/` or `src/*/tests` |

---

## A. Grammar Changes (`filter_parser.rs`)

### Location
`crates/prism-query/src/filter_parser.rs`

### Existing pattern (the model to follow)
Lines 985-993 define the `ICONTAINS`/`ISTARTSWITH`/`IENDSWITH` keyword block:

```
// Existing:
kw("ICONTAINS") => StringOp::Contains { case_insensitive: true }
kw("ISTARTSWITH") => StringOp::StartsWith { case_insensitive: true }
kw("IENDSWITH") => StringOp::EndsWith { case_insensitive: true }
```

### New keywords to add
In the comparison operator block (alongside `=`, `!=`, `IN`), add:

| Keyword | Maps to |
|---------|---------|
| `IEQ` (case-insensitive in grammar) | `CompareOp::Eq` with `case_insensitive: true` |
| `INE` (case-insensitive in grammar) | `CompareOp::Ne` with `case_insensitive: true` |
| `IIN` (case-insensitive in grammar) | `Predicate::In` with `case_insensitive: true` |

**Grammar constraint:** `kw(...)` is already case-insensitive for keywords, so `ieq`,
`IEQ`, `Ieq` all parse. No additional handling needed.

**Collision check:** `IIN` must be parsed BEFORE `IN` in the alternative chain (longest
match first, per Chumsky combinator ordering). Confirm the parser combinator for `IN` is
not a prefix-match that would consume `IIN` prematurely. The implementer must verify the
exact ordering in the `just` / `text::keyword` chain.

---

## B. AST Changes (`ast.rs`)

### Location
`crates/prism-query/src/ast.rs`

### Change 1: `Predicate::Compare`

Current definition region (~line 1284-1306): `Predicate::Compare { field, op, value }`.

Add `case_insensitive: bool` field:

```rust
// Before (structural description):
Predicate::Compare { field: String, op: CompareOp, value: Value }

// After:
Predicate::Compare { field: String, op: CompareOp, value: Value, case_insensitive: bool }
```

**Non-exhaustive impact:** `Predicate` is `#[non_exhaustive]`. Adding a field to an
existing variant is a non-breaking additive change — existing match arms do not need
updating. The `EXPECTED=87` non-exhaustive gate counts annotated TYPES, not field
additions; adding a field to an existing variant does NOT increment the count. The
implementer must verify this interpretation at implementation time by checking whether
the CI gate uses `#[non_exhaustive]` attribute presence or something else.

**Default value for existing construction sites:** All existing call sites that construct
`Predicate::Compare` must add `case_insensitive: false` (TD-VSDD-060 sibling-site sweep).
The implementer must grep all `Predicate::Compare {` construction sites in `prism-query`
and add the field before the PR merges.

### Change 2: `Predicate::In`

Current definition region (~line 1550-1554): `Predicate::In { field, values, negated }`.

Add `case_insensitive: bool` field:

```rust
// After:
Predicate::In { field: String, values: Vec<Value>, negated: bool, case_insensitive: bool }
```

Same sibling-site sweep obligation as Change 1: grep all `Predicate::In {` construction
sites and add `case_insensitive: false`.

**Note on `INE`:** This is implemented via `Predicate::Compare { op: CompareOp::Ne,
case_insensitive: true }` — no new variant or field needed. `INE` maps through the same
AST path as `IEQ`, only with `Ne` as the operator.

---

## C. Predicate Emitter Changes (`pipe_sql_emitter.rs`)

### Location
`crates/prism-query/src/pipe_sql_emitter.rs`

### Function to change
`predicate_to_datafusion_sql(pred: &Predicate)` at approximately line 506.

### Existing CI lowering (the model to follow)
Lines 541-564 handle `StringOp::Contains { case_insensitive: true }`:

```
// Existing pattern (structural):
if case_insensitive {
    format!("lower({}) LIKE lower('%{}%')", field, escape_like(pat))
} else {
    format!("{} LIKE '%{}%'", field, escape_like(pat))
}
```

### New branches to add

**For `Predicate::Compare { case_insensitive: true, op: CompareOp::Eq, .. }`:**

```
lower({field}) = lower('{value}')
```

**For `Predicate::Compare { case_insensitive: true, op: CompareOp::Ne, .. }`:**

```
lower({field}) != lower('{value}')
```

**For `Predicate::In { case_insensitive: true, .. }`:**

```
lower({field}) IN (lower('{v1}'), lower('{v2}'), ...)
```

**Existing `case_insensitive: false` paths are UNCHANGED.** The branches are additive.

### Sibling-site sweep obligation (TD-VSDD-060)

The op table at approximately lines 743-749 maps `CompareOp` variants to SQL operator
strings. After adding `case_insensitive` branching, verify this table is not bypassed for
the `IEQ`/`INE` paths. If the emitter uses the op table to produce `=`/`!=` strings and
then applies CI wrapping, ensure the CI wrap correctly encloses the full expression.

---

## D. PQL Round-Trip Normalizer (`ast.rs` normalizer section)

### Location
`crates/prism-query/src/ast.rs` — the PQL round-trip normalizer at approximately lines
1977-2016 (the AST-to-PQL-string emitter used by `normalized_pql` BC-2.11.018).

### What must change

The normalizer produces a canonical PQL string from an AST. Currently it renders
`Predicate::Compare` as `field = 'value'` or `field != 'value'`. After this change it
must render:

| AST state | Normalized PQL output |
|-----------|----------------------|
| `Compare { op: Eq, case_insensitive: false }` | `field = 'value'` (unchanged) |
| `Compare { op: Eq, case_insensitive: true }` | `field IEQ 'value'` |
| `Compare { op: Ne, case_insensitive: false }` | `field != 'value'` (unchanged) |
| `Compare { op: Ne, case_insensitive: true }` | `field INE 'value'` |
| `In { case_insensitive: false }` | `field IN ('v1', 'v2')` (unchanged) |
| `In { case_insensitive: true }` | `field IIN ('v1', 'v2')` |

**Round-trip invariant** (BC-2.11.018 extension): `parse(normalize(ast)) == ast`. The
test vector for this invariant: `severity IEQ 'high'` — parsed to AST — normalized back
to PQL string — reparsed — must produce the same AST. The test-writer must include this
vector in the red-gate suite.

---

## E. Adapter-Boundary Normalization (Parallel Track, Separate Sub-Story)

**This change surface is CONDITIONED on OD-1 human decision (scope resolution).**

### Location
`crates/prism-ocsf/src/` — the adapter normalizer pipeline that produces `OcsfEvent`
from raw sensor records.

### What must change

The normalizer currently sets OCSF string-label fields by copying values from the sensor
response. The DTU test data reveals inconsistent casing by sensor:

| Sensor | Current `severity` emission | Target (canonical) |
|--------|-----------------------------|--------------------|
| CrowdStrike | `"High"` (Title-case) | `"High"` (no change) |
| Armis | `"UNHANDLED"` (UPPER) | `"Unhandled"` or mapped via `enum_map.rs` |
| Claroty | `"Unresolved"` (as-received) | per OCSF enum_map canonical caption |

**Normalization approach:**
- `enum_map.rs` already defines the canonical caption per `_id` integer value.
- At normalization time, after setting `severity_id`, look up `enum_map[severity_id]`
  and write that caption to the `severity` string field.
- For sensors that do not provide an `_id` (string-only severity fields), apply a
  case-folding lookup table built from `enum_map.rs` captions (e.g.,
  `"HIGH" → "High"`, `"high" → "High"`, `"CRITICAL" → "Critical"`).

**Fields in scope (OD-1 resolution determines final set):**
Minimum for T13 demo: `severity`, `status`.
Recommended full scope: all OCSF enum-label string fields that have a corresponding
`_id` integer sibling: `severity`, `status`, `activity`, `disposition`, `category`,
`type_name`.

**DTU test vector impact:**
Any existing DTU integration test asserting `severity == "UNHANDLED"` or equivalent
UPPER-case value will break. This is intentional — those tests encode the pre-normalization
(wrong) behavior. The implementer must update test vectors to assert the canonical form.

**Story scoping note:**
The implementer may deliver E (adapter normalization) as a sub-story of
`S-PRISMQL-CASE-INSENSITIVE-001` or as a parallel story (`S-PRISMQL-ADAPTER-NORM-001`).
The story-writer should propose the split. The demo-critical minimum (severity + status
normalization for CrowdStrike/Claroty/Armis) must land before T13.

---

## F. Discoverability / Documentation Changes

### Auto-generated PrismQL grammar resource (ADR-045 parity gate)

The auto-generated `PrismQL Reference` MCP resource (BC-2.11.022, ADR-045) must include
`IEQ`, `IIN`, and `INE` in the operator table. The grammar-registry parity gate must pass.

### `prism describe` pedagogical examples (ADR-041 L2 teaching surface)

The `prism describe <table>` output includes example queries for each table. Add:

```
-- Case-insensitive equality (for OCSF enum-label fields)
SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'

-- Case-insensitive membership
SELECT * FROM crowdstrike_detections WHERE severity IIN ('high', 'critical')
```

Include the OCSF casing note: *"OCSF severity is stored as Title-case (`'High'`). Use
`IEQ`/`IIN` to match regardless of the case you type, or `= 'High'` for the exact
canonical form."*

### OD-4 scope (pedagogical hint — pending human decision)

If OD-4 is resolved as "implement now": when a `=`/`IN` filter returns zero rows but a
case-insensitive match would return rows, emit a structured hint in the response. The
detection mechanism: after zero-row execution, re-execute the same query with
`IEQ`/`IIN` substituted for `=`/`IN` predicates (or perform a server-side case-fold
check). If the re-execution returns rows > 0, include a `hint` field in the response:
`{ "hint": "Query returned 0 rows with case-sensitive '='. Did you mean 'IEQ'? A
case-insensitive search would match N rows." }`. Scope this separately — it requires
a second DataFusion execution per query and needs its own BC.

---

## G. TDD Red-Gate Test Requirements

These tests must be FAILING before implementation begins (BC-5.38.001 Red Gate).

| Test ID | Behavior | Assertion |
|---------|----------|-----------|
| T-CASE-001 | `severity IEQ 'HIGH'` parses to `Predicate::Compare { case_insensitive: true }` | AST equality |
| T-CASE-002 | `severity IEQ 'HIGH'` emits `lower(severity) = lower('HIGH')` to DataFusion | Emitter output string |
| T-CASE-003 | `severity IEQ 'high'` returns rows where stored value is `'High'` (DataFusion execution) | Row count > 0 |
| T-CASE-004 | `severity = 'high'` returns 0 rows where stored value is `'High'` (case-sensitive preserved) | Row count = 0 |
| T-CASE-005 | `status IIN ('open', 'new')` parses to `Predicate::In { case_insensitive: true }` | AST equality |
| T-CASE-006 | `status IIN ('OPEN', 'NEW')` returns rows where stored value is `'open'`/`'new'` | Row count > 0 |
| T-CASE-007 | `normalized_pql` for `severity IEQ 'high'` round-trip: parse → normalize → string contains `IEQ` | String contains `IEQ` |
| T-CASE-008 | `normalized_pql` round-trip reparsed produces same AST as original | AST equality |
| T-CASE-009 | `severity IEQ 'high' AND severity IEQ 'high'` does not panic (fuzz-seed regression) | No panic |
| T-CASE-010 | Existing `severity = 'High'` still returns rows where stored value is `'High'` (no semantic regression) | Row count > 0 |

---

## Dependency Order (Implementation Sequencing)

The changes have a strict dependency ordering for TDD:

```
B (AST) → A (Grammar parses into new AST) → C (Emitter reads new AST) → D (Normalizer renders new AST)
                                                                          ↑
E (Adapter normalization) is INDEPENDENT — can be delivered in parallel
F (Discoverability) depends on A+B+C+D being complete
```

Recommended TDD order:
1. Write red-gate tests T-CASE-001 through T-CASE-010 (all failing)
2. Implement B (AST field additions + sibling-site sweep)
3. Implement A (grammar keywords — now tests T-CASE-001/005 pass)
4. Implement C (emitter lowering — now T-CASE-002/006 pass)
5. Run DataFusion integration tests (T-CASE-003/004/006 pass)
6. Implement D (round-trip normalizer — T-CASE-007/008 pass)
7. Verify T-CASE-009/010 (regression guard)
8. Implement F (discoverability — grammar resource parity gate)
9. E (adapter normalization) in parallel or as separate story

---

## Non-Exhaustive Gate Impact Assessment

| Change | New `#[non_exhaustive]` type? | EXPECTED=87 increment? |
|--------|-------------------------------|------------------------|
| Add `case_insensitive: bool` to `Predicate::Compare` | No (existing variant) | No |
| Add `case_insensitive: bool` to `Predicate::In` | No (existing variant) | No |
| New `IEQ`/`IIN`/`INE` grammar keywords | No (grammar, not types) | No |

**Conclusion:** The `EXPECTED=87` non-exhaustive gate count is UNCHANGED by this story.
The implementer should confirm this at compilation time — if any new public type is
introduced, the count must be updated in `ci.yml` and this design map amended.

---

## Backward Compatibility Confirmation

All changes are strictly additive:
- `=`, `!=`, `IN` retain case-sensitive exact-match semantics unchanged.
- No existing query changes behavior.
- `#[non_exhaustive]` field additions do not require downstream match-arm updates.
- `normalized_pql` output for existing queries is unchanged (field defaults to
  `case_insensitive: false`, round-trip emitter uses existing rendering path).

---

## Human Decisions Required Before Story Can Be Implemented

The story-writer and implementer are blocked on OD-1 and OD-4 for full scope definition.
OD-2 (case-sensitive default) and OD-3 (IEQ/IIN/INE spelling) are pre-conditions for
any grammar work. All four ODs are recorded in ADR-047 §Open Decisions.

The grammar work (A+B+C+D) for IEQ/IIN is unblocked by OD-2 + OD-3 resolution alone.
The adapter normalization work (E) requires OD-1 resolution (scope of fields).
The pedagogical hint (F partial) requires OD-4 resolution.

Minimum unblocked scope to start after OD-2 + OD-3 human sign-off: A, B, C, D, and the
demo-critical subset of E (severity + status for the three in-demo sensors).

---

## Architecture Decision Note: Adapter-Boundary Normalization Insertion Point

**Date:** 2026-07-07
**Raised by:** LOCAL adversary pass-5 F-CRIT-002
**Adjudicated by:** architect

### Finding

BC-2.02.013 v1.2 §Postconditions (now superseded by v1.3) originally pinned the normalization insertion point at
`OcsfNormalizer::normalize_with_mappers` in `crates/prism-ocsf/src/normalizer.rs`
(F-CRIT-001 closure). However, this function has **zero production callers** on the
spec-driven adapter path.

The actual production path is:

```
SpecDrivenSensorAdapter::fetch()
  → PipelineExecutor::execute()        [prism-spec-engine]
  → PipelineResult (raw serde_json)
  → pipeline_result_to_record_batch()  [prism-bin]
  → build_column_array()               [prism-bin]
  → Arrow StringArray                  (what DataFusion sees)
```

`normalize_with_mappers` creates a `DynamicMessage` via the protobuf path, which is
never invoked in this flow. The existing normalization logic in `normalize_with_mappers`
is correct but unreachable for production queries.

### Candidate Insertion Points

| Option | Description | Verdict |
|--------|-------------|---------|
| (a) Route through `normalize_with_mappers` (DynamicMessage roundtrip) | Heavy: adds full protobuf round-trip per record to the Arrow materialization path | REJECTED — wrong architectural path, heavyweight |
| (b) `build_column_array` in `prism-bin/src/spec_driven_adapter.rs` — String arm | Light: calls `OcsfEnumMap::normalize_enum_label` only for String-typed OCSF enum-label columns | **RATIFIED** (see below) |
| (c) New `normalize_enum_label_fields` pass in `prism-spec-engine/src/pipeline.rs` sibling to `normalize_timestamp_fields` | Architecturally cleaner choke point; however requires adding `prism-ocsf` (with protobuf binary) as a new production dependency on `prism-spec-engine`, and the `prism-spec-engine` MUST-NOT-depend-on-arrow invariant implies binary-weight constraints | REJECTED — adds heavyweight protobuf dep to a library crate that currently has none |

### Ratified Decision: Option (b)

**Insertion point:** `build_column_array` in `crates/prism-bin/src/spec_driven_adapter.rs`,
specifically in the `ColumnType::String` branch (add an explicit `ColumnType::String =>` arm
ahead of the existing `_ =>` catch-all).

**Dependency:** `prism-bin` already depends on `prism-ocsf` (see `prism-bin/Cargo.toml` line 81).
No new cross-crate dependency is required.

**Secondary insertion point (unchanged):** `OcsfNormalizer::normalize_with_mappers` retains its
normalization for the DynamicMessage/protobuf-export path (future path). When that path gains
production callers, normalization will already be correct there. No changes to
`normalize_with_mappers` are required; it is NOT dead code — it is future-path code.

**Idempotency:** `OcsfEnumMap::normalize_enum_label` is idempotent. If data ever flows through
both paths, double-normalization is a harmless no-op.

**All other production materialization paths verified:**

| Path | Does it bypass `build_column_array`? |
|------|--------------------------------------|
| WASM plugin (crowdstrike-oauth2) provides auth only; still calls `SpecDrivenSensorAdapter::fetch()` → `pipeline_result_to_record_batch()` | No — same path |
| MCP tool `prism query` → fan_out → `SpecDrivenSensorAdapter::fetch()` | No — same path |
| DTU demo servers serve raw JSON over HTTP; the spec-driven adapter fetches them via the same pipeline | No — same path |
| `OcsfNormalizer::normalize_with_mappers` | Zero production callers today; not a query-path materialization route |

**Conclusion:** `build_column_array` is the single choke point through which ALL production
sensor data flowing into DataFusion passes. Normalization there satisfies the BC intent
"the data DataFusion materializes carries only canonical-cased enum labels."

### OcsfEnumMap Access Pattern

`OcsfEnumMap` must not be re-instantiated per call (it builds a `HashMap` on construction).
Use a `OnceLock<OcsfEnumMap>` static in `spec_driven_adapter.rs`:

```rust
use std::sync::OnceLock;
use prism_ocsf::OcsfEnumMap;

static SPEC_ADAPTER_ENUM_MAP: OnceLock<OcsfEnumMap> = OnceLock::new();
fn spec_adapter_enum_map() -> &'static OcsfEnumMap {
    SPEC_ADAPTER_ENUM_MAP.get_or_init(OcsfEnumMap::new)
}
```

This is the same pattern as `OCSF_ENUM_MAP` in `prism-ocsf/src/normalizer.rs`. Two
`OcsfEnumMap` instances (one per process image) are safe — `OcsfEnumMap::new()` is
pure in-memory with no external state.

### In-Scope OCSF Enum-Label Field Names

The column selection rule uses the same set as `OCSF_ENUM_LABEL_FIELDS` in
`prism-ocsf/src/normalizer.rs`:

```rust
const OCSF_ENUM_LABEL_FIELDS: &[&str] = &["severity", "status", "activity_name", "disposition"];
```

Only `ColumnType::String` columns whose `col.name` is in this set are normalized.
`ColumnType::Json` and other variants are not affected.

### Unrecognized-Value Warn

Per BC-2.02.013 §Postconditions §Error Cases, unrecognized values are left as-received and
`tracing::warn!(event_type = "ocsf.enum_label_unrecognized", ...)` is emitted. The warn is
registered in BC-2.16.002 §Postconditions catalog row 91. The implementer MUST emit it from
`build_column_array` on the `normalize_enum_label` → `None` branch, using the same field
schema as the existing emission in `normalize_with_mappers`:

```rust
tracing::warn!(
    event_type = "ocsf.enum_label_unrecognized",
    field_name = %col.name,
    // SEC-002: cap at 50 codepoints (untrusted sensor data; consistent with Datetime warn)
    value = %s.chars().take(50).collect::<String>(),
    sensor_type = %sensor_id,
    "unrecognized OCSF enum label value; leaving as-received"
);
```

The `ocsf.enum_label_unrecognized` event_type is ALREADY registered in BC-2.16.002 catalog
row 91 (by the `normalize_with_mappers` emission). The implementer does not need to add a
new catalog row — the same row covers both emission sites because the `event_type`, field
schema, and semantics are identical.
