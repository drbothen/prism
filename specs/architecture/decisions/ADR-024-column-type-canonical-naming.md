---
document_type: adr
adr_id: "ADR-024"
title: "ColumnType Canonical Naming — Domain-Level Variant Names for Sensor Schema API; prism-spec-engine Shadow Enum Retirement"
status: ACCEPTED
date: "2026-05-12"
version: "1.0"
producer: architect
subsystems_affected: [SS-16, SS-11, SS-01]
supersedes: null
superseded_by: null
anchor_stories: []
locked_decisions: ["domain_level_naming_is_canonical"]
runtime_deliverables:
  - "Remove prism_spec_engine::types::ColumnType — retire the shadow enum"
  - "Update prism-spec-engine parse/conversion sites to use prism_core::column::ColumnType"
  - "Update prism-spec-engine test callsites to import from prism_core::column"
wiring_deferred_to: "S-PLUGIN-PREREQ-C sub-fix 2 implementer dispatch"
---

# ADR-024: ColumnType Canonical Naming — Domain-Level Variant Names for Sensor Schema API; prism-spec-engine Shadow Enum Retirement

## Context

As of 2026-05-12, the prism workspace contains three distinct `ColumnType` enums serving
different layers. Two of them collide semantically for sensor schema column types:

**`prism_core::column::ColumnType`** — the canonical sensor schema API (S-1.11):

```
String | Integer | Float | Boolean | Datetime | Json
```

Exported from `prism-core` public API (`column::ColumnType — spec-engine column type enum
(S-1.11)`). Enforced by `external_default_construction.rs` tests. Uses domain-level names.
TOML serde rename: `"integer"`, `"float"`, `"datetime"`.

**`prism_spec_engine::types::ColumnType`** — a local stub introduced during S-1.12:

```
String | Int64 | Float64 | Boolean | Timestamp | Json
```

The comment in `types.rs` explains it as "local stubs per the story dependency model."
Uses wire-encoding names: `Int64`, `Float64`, `Timestamp`. This is the source of the naming
collision that blocked TD-S-PLUGIN-PREREQ-C-001-A sub-fix 2: 4 of 6 variant names differ
from the canonical enum.

**`prism_core::types::ColumnType`** — internal table schema (S-2.03):

```
Text | Int64 | UInt64 | Float64 | Bool | Timestamp | Json | Bytes
```

Used exclusively by `InternalTableDescriptor` for internal RocksDB table schemas (schedules,
cases, diff_results, etc.). This enum is correct, distinct, and not affected by this ADR.

The implementer hit a genuine semantic question when attempting to close sub-fix 2: are sensor
schema column types named at the domain level (`Integer`, `Float`, `Datetime`) or at the
wire-encoding level (`Int64`, `Float64`, `Timestamp`)? This ADR answers that question.

## Decision

**`prism_core::column::ColumnType` with variants `String / Integer / Float / Boolean / Datetime / Json` is the canonical enum for sensor TOML schema column types.**

The shadow enum `prism_spec_engine::types::ColumnType` is retired. All sites in
prism-spec-engine that use the local enum — including `ColumnDef.column_type`, the
`parse_column_type` function in `add_sensor_spec.rs`, and all test imports — must be
migrated to import and use `prism_core::column::ColumnType`.

`prism_core::types::ColumnType` (the `Text/Int64/UInt64/…` enum for `InternalTableDescriptor`)
is unchanged and distinct.

Migration variant mapping for `parse_column_type`:

| TOML token(s) | After migration |
|---------------|-----------------|
| `"int64" \| "int" \| "integer" \| "bigint"` | `ColumnType::Integer` |
| `"float64" \| "float" \| "double" \| "real"` | `ColumnType::Float` |
| `"timestamp" \| "datetime"` | `ColumnType::Datetime` |
| `"boolean" \| "bool"` | `ColumnType::Boolean` |
| `"json" \| "object"` | `ColumnType::Json` |
| default | `ColumnType::String` |

## Rationale

**Domain naming is semantically correct at this layer.** The sensor schema API
(`[[table.columns]]` TOML entries) describes the *domain model* of a column — what
kind of data it holds conceptually. `Integer` and `Float` are domain concepts; `Int64`
and `Float64` are storage/wire-encoding details. When an MSSP analyst writes a sensor spec,
they declare "this column is a `datetime`", not "this column is a 64-bit signed integer
interpreted as a Unix timestamp in microseconds." Domain naming is appropriate for the
operator-facing layer; wire naming belongs in the Arrow schema builder implementation.

**Separation of concerns.** The mapping from domain type to Arrow wire type is a translation
step implemented once in prism-query's DataFusion schema registration (`column::ColumnType::Datetime`
→ Arrow `TimestampMicrosecond`). Naming the enum variant `Timestamp` conflates the domain
concept with the Arrow encoding detail and makes that translation boundary invisible.

**Existing adoption is unambiguous.** `prism_core::column::ColumnType` is the published S-1.11
API consumed by test infrastructure including `external_default_construction.rs` and
`bc_2_16_003_test.rs` (which uses `ColumnType::Datetime` and `ColumnType::String`). The
prism-spec-engine shadow enum was an internal-only stub that was never promoted. The canonical
type already has higher adoption across the codebase.

**Spec wins over code** (CLAUDE.md precedence rule 7). The S-1.11 story spec established the
domain-level naming. The prism-spec-engine stub was an implementation convenience that diverged
without an architectural decision authorizing the divergence. This ADR retroactively closes
that gap and designates the spec-level name as canonical.

## Consequences

### Positive

- Single source of truth for sensor schema column types: one enum, one set of variant names
- `#[non_exhaustive]` on `prism_core::column::ColumnType` continues to enforce forward-compat
- TOML operator experience improved: `"datetime"` and `"integer"` are unambiguous human-readable
  tokens; `"int64"` and `"float64"` are Arrow-specific jargon
- TD-S-PLUGIN-PREREQ-C-001-A sub-fix 2 unblocked

### Negative / Trade-offs

- All prism-spec-engine sites using `ColumnType::Int64`, `::Float64`, or `::Timestamp` must be
  updated to `::Integer`, `::Float`, `::Datetime` — a breaking change within the crate boundary
- Test helpers and match arms in prism-spec-engine tests will need import updates

### Status as of 2026-05-12

ACCEPTED, migration not yet executed. Assigned to TD-S-PLUGIN-PREREQ-C-001-A sub-fix 2
implementer dispatch. All prism-spec-engine `ColumnType` use-sites must be migrated before
that fix-burst can close.

## Alternatives Considered

- **Option A: Adopt wire-encoding names in prism-core (Integer → Int64, Float → Float64, Datetime → Timestamp).** Rejected. Migrating the canonical public API to storage-implementation terminology would put Arrow-specific jargon in front of sensor spec authors. Migration cost is also higher: prism-core's public API is consumed by all downstream crates; the shadow enum is internal-only.

- **Option B: Keep both enums, add a `From` conversion impl.** Rejected. Two parallel enums for the same concept with different variant names is a maintenance anti-pattern that guarantees future confusion. The conversion impl must be kept in sync across the crate boundary for no semantic benefit. Retire the duplicate.

- **Option C: Introduce a third unified ColumnType with variant aliases.** Rejected. The problem is naming, not coverage. The canonical enum's 6 variants cover all supported column types. A third enum adds complexity without benefit.

## Source / Origin

- `crates/prism-core/src/column.rs` — canonical `ColumnType` enum with domain-level names (S-1.11)
- `crates/prism-spec-engine/src/types.rs` — shadow enum with wire-encoding names (S-1.12 stub)
- `crates/prism-spec-engine/src/add_sensor_spec.rs` — `parse_column_type` function using shadow enum
- `crates/prism-core/src/lib.rs` — public API export declaration (`column::ColumnType — spec-engine column type enum (S-1.11)`)
- TD-S-PLUGIN-PREREQ-C-001-A sub-fix 2 — implementer block that surfaced the naming question

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-12 | architect | Initial decision — domain-level naming canonical; prism-spec-engine shadow enum retired |
