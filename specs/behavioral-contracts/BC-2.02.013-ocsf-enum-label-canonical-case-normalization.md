---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-06T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: draft
introduced: 2026-07-06
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/specs/architecture/decisions/ADR-047-prismql-case-sensitivity-policy-ieq-iin-and-adapter-boundary-normalization.md"
input-hash: "TBD"
traces_to: ["CAP-003"]
extracted_from: null
---

# BC-2.02.013: Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization

## Description

At the adapter/normalizer boundary in `prism-ocsf`, before any `DynamicMessage` is created (BC-2.02.002), OCSF enum-label string fields are normalized to their canonical OCSF Title-case casing as defined by `enum_map.rs`. The canonical form is established by the OCSF spec captions embedded in `enum_map.rs` (e.g., `severity_id=4 → "High"`, `severity_id=5 → "Critical"`, `severity_id=1 → "Informational"`). Without this normalization, sensors that store enum labels as all-caps (`'HIGH'`) or as-received vendor strings (`'Unresolved'`) fragment `GROUP BY severity` aggregations into multiple buckets for the same semantic value.

**Scope (OD-1 resolution per D-1398, 2026-06-27):** ALL OCSF enum-label string fields are normalized at the adapter boundary. The minimum guaranteed set for the T13 demo is `severity` and `status`. All other OCSF enum-label fields (activity, disposition, category, and their sensor-specific mappings) are also normalized under this BC.

This normalization is complementary to, not a replacement for, the `IEQ`/`IIN`/`INE` operators (BC-2.11.024). Normalization makes stored data uniformly canonical; `IEQ`/`IIN` provide an ergonomic safety net for fields not yet normalized or free-form non-enum fields such as hostnames and usernames.

## Preconditions

- A sensor adapter has returned a raw sensor record and the `prism-ocsf` normalizer is constructing the normalized event representation prior to `DynamicMessage` creation (BC-2.02.002).
- The `enum_map.rs` OCSF enum-value-to-caption map (BC-2.02.010) is loaded in the binary and available to the normalizer at runtime — it is embedded at build time via `include_str!` (or equivalent). No network I/O is required.
- The normalizer has identified the OCSF field type for the field being normalized (e.g., `severity` is an OCSF enum-label field; `device.hostname` is a string field — NOT subject to this normalization).

## Postconditions

- Before the `DynamicMessage` is populated (BC-2.02.002), every OCSF enum-label string field in the normalized record is rewritten to its canonical OCSF Title-case casing from `enum_map.rs`.
- **Severity (guaranteed):** `severity` field values are normalized to OCSF Title-case per OCSF v1.x: `'HIGH'` → `'High'`, `'high'` → `'High'`, `'CRITICAL'` → `'Critical'`, `'critical'` → `'Critical'`, etc.
- **Status (guaranteed):** `status` field values are normalized to OCSF Title-case per OCSF v1.x status captions.
- **All other OCSF enum-label fields:** normalized to their respective OCSF Title-case captions — activity, disposition, category, and sensor-specific OCSF enum mappings.
- The normalization function is **idempotent**: if the field already contains the canonical-case value (e.g., `'High'`), the value is unchanged. Re-normalizing already-canonical data has no effect.
- Values NOT found in `enum_map.rs` (e.g., vendor-specific extension values, wholly vendor-proprietary strings) are **left as-received**. A warning is logged with the unrecognized value, field name, and sensor type. These values are queryable but require `IEQ` for case-insensitive matching (BC-2.11.024).
- The `DynamicMessage` emitted from BC-2.02.002 contains only canonical-cased OCSF enum-label string values. Downstream PrismQL queries using case-sensitive `=` predicates against OCSF enum-label fields operate against this canonical data, producing consistent results without requiring `IEQ` — this is the intended steady-state: normalized data + case-sensitive compare = consistent behavior.
- **Cost:** normalization is paid once at ingest per record. There is no per-query normalization cost.

## Invariants

- The `enum_map.rs` in `prism-ocsf` is the **sole canonical casing authority** for OCSF enum-label string fields at the adapter boundary. No sensor-specific adapter or TOML spec override may produce a different casing for an OCSF enum-label field without explicit justification against the OCSF schema and a corresponding `enum_map.rs` amendment.
- Normalization is applied **before** `DynamicMessage` creation — it is part of the adapter-boundary pipeline stage, not a post-creation mutation of the `DynamicMessage`.
- The normalization function is a pure in-memory lookup-and-rewrite: it reads `enum_map.rs` (embedded in the binary), performs a map lookup, and rewrites the field value. No network I/O, no RocksDB access, no external state.
- DI-005: The `DynamicMessage` produced after normalization conforms to the compiled OCSF protobuf descriptor. Canonical-cased string values are valid OCSF string field values — normalization does not produce out-of-spec data.
- This BC governs OCSF **enum-label** string fields only. Free-form string fields (hostnames, usernames, file paths, process names, registry keys) are NOT subject to this normalization.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning (non-fatal) | An OCSF enum-label field value has no matching caption in `enum_map.rs` (unrecognized vendor extension value) | Value left as-received; warning logged with field name, value, and sensor type; `IEQ` is the ergonomic fallback for non-canonical values |
| Warning (non-fatal) | `enum_map.rs` contains no mapping for the OCSF field type being normalized (missing field-type coverage) | Value left as-received; warning logged; not a fatal error — field coverage is extended by amending `enum_map.rs` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-020 | CrowdStrike adapter emits `severity='High'` (already canonical Title-case) | Idempotent — value unchanged; no warning |
| EC-02-021 | Armis adapter emits `severity='UNHANDLED'` (vendor-specific value, not an OCSF severity caption) | Value left as-received (`'UNHANDLED'`); warning logged; `severity = 'UNHANDLED'` (case-sensitive) or `severity IEQ 'unhandled'` both work for querying |
| EC-02-022 | Claroty adapter emits `status='Unresolved'` (vendor-specific value not in OCSF `status` captions) | Value left as-received; warning logged; Claroty `'Unresolved'` is vendor-specific — `status IEQ 'unresolved'` works for case-insensitive querying |
| EC-02-023 | Sensor emits `severity='high'` (all-lowercase) | Normalized to `'High'` (OCSF canonical Title-case) |
| EC-02-024 | Sensor emits `severity='CRITICAL'` (all-caps) | Normalized to `'Critical'` (OCSF canonical Title-case) |
| EC-02-025 | Field value is `null` | Value left as `null` — null is not an enum label and cannot be normalized; nulls pass through unchanged |
| EC-02-026 | `GROUP BY severity` PrismQL query across CrowdStrike (emits `'High'`) + Armis (originally emits `'HIGH'`) after normalization | Both sensors now contribute to the same `'High'` bucket — no fragmentation into `'High'` + `'HIGH'` variants; cross-sensor aggregation is correct |
| EC-02-027 | `severity = 'High'` PrismQL query after normalization | Returns rows from all sensors regardless of original sensor casing — normalization ensures all store `'High'` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| CrowdStrike record with `severity='High'` (already canonical) | `severity='High'` in DynamicMessage (idempotent) | happy-path (idempotent) |
| Armis record with `severity='HIGH'` (all-caps) | `severity='High'` in DynamicMessage | normalization |
| Any sensor record with `severity='low'` (all-lowercase) | `severity='Low'` in DynamicMessage | normalization |
| Any sensor record with `severity='CRITICAL'` | `severity='Critical'` in DynamicMessage | normalization |
| Record with unrecognized vendor value `severity='UNHANDLED'` | `severity='UNHANDLED'` unchanged; warning logged | unrecognized-value fallback |
| PrismQL `severity = 'High'` after all-sensor normalization | Returns rows from ALL sensors regardless of original sensor casing | integration |
| PrismQL `GROUP BY severity` across CrowdStrike + Armis after normalization | `'High'` appears as one bucket — not split into `'High'` + `'HIGH'` | aggregation consistency |
| Record where field value is `null` | `null` passes through unchanged; no normalization attempted | null passthrough |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) | proptest |
| VP-022 | OCSF normalizer: never panics on arbitrary input (fuzz) | fuzz |
| (VP-TBD) | After normalization, `GROUP BY severity` across all four sensor adapters produces at most 7 distinct buckets (OCSF severity cardinality: Unknown/Informational/Low/Medium/High/Critical/Other) | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| Capability Anchor Justification | CAP-003 ("OCSF Normalization") per capabilities.md §CAP-003 — this BC specifies canonical-case normalization of OCSF enum-label string fields at the adapter boundary. CAP-003 is the authoritative capability for OCSF normalization: "Normalize all sensor records to OCSF v1.x via the DynamicMessage protobuf pattern. Enables cross-sensor correlation by mapping vendor-specific fields to a common schema." Canonical casing is a required aspect of normalization — two sensors producing `'High'` and `'HIGH'` for the same OCSF severity concept violates the cross-sensor consistency goal of CAP-003. The consistency postcondition of CAP-003 ("cross-sensor correlation via unified OCSF table") cannot be achieved without canonical casing at the adapter boundary. |
| L2 Invariants | DI-005 |
| ADR | ADR-047 §D.3 — Adapter-Boundary Canonical-Case Normalization: ACCEPTED per human sign-off D-1398, 2026-06-27 (OD-1 resolved: ALL OCSF enum-label fields; demo-minimum set severity + status guaranteed) |
| Architecture Module | SS-02 (OCSF Normalization) — `prism-ocsf` normalization pipeline, `enum_map.rs` |
| Priority | P1 |

## Related BCs

- BC-2.02.002 (composes with): normalization is applied before DynamicMessage creation in the adapter-boundary pipeline; BC-2.02.002 §Postconditions amended (v1.5) to make this explicit
- BC-2.02.010 (composes with): `enum_map.rs` enum-value-map is the canonical casing authority; BC-2.02.010 §Postconditions and §Invariants amended (v1.5) to state that the map's authority extends to adapter-boundary normalization, not only MCP display enrichment
- BC-2.11.024 (composes with): `IEQ`/`IIN`/`INE` operators are the ergonomic safety net for values left as-received (vendor-specific, non-canonical) and for free-form non-enum fields; the two mechanisms together eliminate the T13 demo failure mode

## Architecture Anchors

- `architecture/decisions/ADR-047` §D.3 — Adapter-Boundary Canonical-Case Normalization
- `architecture/decisions/ADR-047` §Context §Three Facts — "Prism already encodes canonical OCSF casing" (`enum_map.rs`, Title-case captions)
- `architecture/decisions/ADR-047` §Context §Current inconsistency by sensor — CrowdStrike Title-case; Armis UPPER-case; Claroty as-received
- `architecture/decisions/ADR-047` §Consequences — "Column semantics in sensor TOML specs change: `severity` column value is now contractually Title-case per OCSF"
- `architecture/decisions/ADR-047` §Alternatives Alt-6 (query-time normalization only rejected as insufficient — does not fix GROUP BY fragmentation)

## Story Anchor

S-PRISMQL-CASE-INSENSITIVE-001 — implements adapter-boundary enum-label normalization in the `prism-ocsf` normalization pipeline using the existing `enum_map.rs` caption map.

## VP Anchors

VP-016 (existing), VP-022 (existing). VP for cross-sensor aggregation cardinality to be assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-PRISMQL-CASE-INSENSITIVE-001-bc-burst | 2026-07-06 | product-owner | Initial draft. Adapter-boundary OCSF enum-label canonical-case normalization using `enum_map.rs`. Scope: ALL OCSF enum-label fields (OD-1 resolved per D-1398 2026-06-27; demo-minimum: severity + status). Resolves ADR-047 §D.3. |
