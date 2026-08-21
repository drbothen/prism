---
document_type: story
story_id: S-JSON-EXTRACT-UDF-001
title: "DataFusion json_extract_string ScalarUDF for PrismQL Tier-2 JSON Path Filtering"
level: "L4"
version: "0.1"
status: draft
producer: story-writer
timestamp: "2026-08-21T00:00:00Z"
phase: 3
wave: tbd
epic_id: EPIC-OCSF-ROUTING
cycle: v3-brownfield
priority: P1
points: tbd
tdd_mode: strict
target_module: prism-query
subsystems:
  - SS-01
inputs:
  - crates/prism-query/src/memory.rs
  # session_context.rs does not exist yet — it will be created by this story; added at story-materialization time
input-hash: "50796c8"
traces_to: .factory/stories/S-ADR058-OCSF-ROUTING-001-sensor-spec-ocsf-field-name-routing.md
crates_touched:
  - prism-query
depends_on:
  - S-ADR058-OCSF-ROUTING-001
blocks: []
behavioral_contracts: []
# BC status: pending PO authorship — full BC layer authored at story-materialization time.
# status MUST remain draft until behavioral_contracts is non-empty (Spec-First Gate S-7.01).
verification_properties: []
assumption_validations: []
risk_mitigations: []
holdout_scenarios: []
estimated_days: tbd
modified: "2026-08-21"
---

> **STUB — draft.** Full acceptance criteria, Red Gate test list, and BC layer are
> authored at story-materialization time.

> **v1-chain obligation:** This story MUST merge before the Claroty live-validation
> gate. Merging ROUTING-001 without this story leaves Tier-2 `raw_extensions`
> filtering unimplemented — analysts can query the JSON blob but cannot filter
> individual fields inside it (OQ-002 human decision 2026-08-21).

> **Execute:** `/vsdd-factory:deliver-story S-JSON-EXTRACT-UDF-001`

# S-JSON-EXTRACT-UDF-001: DataFusion json_extract_string ScalarUDF

## Authority

**OQ-002 (human decision 2026-08-21):** `raw_extensions` Tier-2 filtering requires
a `json_extract_string(json_col, '$.path') -> VARCHAR` DataFusion ScalarUDF registered
in the PrismQL session context. This is the v1-chain story that operationalizes Tier-2
filtering — without it, `WHERE raw_extensions LIKE '%value%'` is the only filtering
path, which is unreliable and does not support indexed or type-safe access.

**S-ADR058-OCSF-ROUTING-001 (depends_on):** The OCSF field-name routing story
establishes `raw_extensions` as the Arrow column carrying Tier-2 (non-mapped) JSON
blobs. The current story registers the UDF that enables querying into that blob via
JSON path syntax within PrismQL.

---

## Narrative

- **As a** PrismQL user querying Claroty (and other sensor) data
- **I want** a `json_extract_string(raw_extensions, '$.some.path')` function available
  in PrismQL SELECT/WHERE/HAVING clauses
- **So that** I can filter and project individual JSON fields from `raw_extensions`
  Tier-2 data without requiring full schema promotion to named Arrow columns

---

## Scope

Register a DataFusion `ScalarUDF` named `json_extract_string` in
`build_session_context` (or `prism-query::memory`, depending on session-context
factoring at dispatch time). The UDF wraps `serde_json` path extraction:

- **Signature:** `json_extract_string(json_col: String, path: String) -> String | NULL`
- **Semantics:**
  - `json_col` is a DataFusion `String`/`Utf8` column (e.g., `raw_extensions`)
  - `path` is a `$.dot.notation` path string (e.g., `'$.audit_status'`)
  - Returns the string value at that path if found and it is a JSON string primitive
  - Returns `NULL` if the path is absent or the value is not a JSON string
  - Returns `NULL` and emits `tracing::warn!(event_type = "json.extract.type_mismatch",
    ...)` if `json_col` is not valid JSON
- **serde_json path resolution:** Use `serde_json::Value::pointer` for
  `$.`-rooted path extraction (converting `$.a.b` to `/a/b` per JSON Pointer RFC 6901)
  or implement a lightweight dot-notation traversal as a project-specific convention.
  Document the chosen convention in the implementation for SAP-3 reachability.

**Out of scope for v1:**
- Numeric / boolean / array extraction (v2 — extend via `json_extract_int`, `json_extract_bool`)
- Wildcard path segments (`$.items[*].name`)
- Registration of the UDF for non-prism-query consumers

---

## Acceptance Criteria

> N/A at stub stage — authored at story-materialization time.
> Each AC will trace to a BC-S.SS.NNN clause per Spec-First Gate S-7.01.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `json_extract_string` UDF | `crates/prism-query/src/session_context.rs` or `crates/prism-query/src/memory.rs` | Pure (deterministic string → string function; no I/O) |
| `serde_json` path traversal | stdlib at call site within UDF | Pure |

---

## Edge Cases

> Expanded at materialization.

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 (TBD) | `json_col` is valid JSON but `path` absent | Return NULL; no warning |
| EC-002 (TBD) | `json_col` is not valid JSON | Return NULL; emit `json.extract.type_mismatch` WARN |
| EC-003 (TBD) | `path` resolves to a non-string value (number, bool, object, array) | Return NULL; no warning for v1 |
| EC-004 (TBD) | `json_col` is SQL NULL | Return NULL (DataFusion null propagation) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `json_extract_string` UDF body | pure-core | Deterministic transformation of string input; no external I/O; side-effect-free except for `tracing::warn!` on invalid JSON (observability only) |

---

## Token Budget Estimate (MANDATORY)

> Full estimate completed at materialization when AC/RG scope is known.

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2 000 |
| `session_context.rs` (relevant section) | ~3 000 |
| `serde_json` path traversal reference | ~1 000 |
| Test files (TBD at materialization) | TBD |
| **Total** | **TBD** |
| Agent context window | 200 K (Sonnet) |
| **Budget usage** | **< 5% pre-test** |

---

## Tasks (MANDATORY)

> Tasks expanded at materialization. Pre-materialization phase:

- [ ] T-PREP-01: PO authors behavioral contracts for `json_extract_string` semantics
  (NULL-propagation, type-mismatch WARN, path-absent NULL, valid-JSON-no-path NULL)
- [ ] T-PREP-02: Story-writer expands stub to full ACs + Red Gate list (SAC-1 format),
  including density check ≥ 0.5 per BC-5.38.001
- [ ] T-PREP-03: Transition status draft → ready (requires non-empty `behavioral_contracts`)
- [ ] T-IMPL-01: Test-writer writes failing Red Gate tests (one per AC/BC clause)
- [ ] T-IMPL-02: Register `json_extract_string` ScalarUDF in `build_session_context`;
  implement via `serde_json` path extraction with NULL semantics per ACs
- [ ] T-IMPL-03: Add `tracing::warn!(event_type = "json.extract.type_mismatch", ...)` on
  invalid-JSON input (SAP-1 obligation: add catalog row to BC-2.16.002 §Canonical
  Structured Event Catalog in same atomic commit)
- [ ] T-IMPL-04: Run `just iter prism-query` — all Red Gate tests must pass
- [ ] T-IMPL-05: Run `just check` — full workspace gate must stay GREEN

---

## Previous Story Intelligence (MANDATORY)

> N/A — first story in EPIC-OCSF-ROUTING chain that directly targets prism-query
> session context UDF registration. No prior story covers this surface.

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-ADR058-OCSF-ROUTING-001 | `raw_extensions` is the Tier-2 Arrow column carrying non-OCSF-mapped fields (JSON blob per BC-2.16.003 §Interpretation A Tier-2) | Tier-2 accumulation loop in `pipeline_result_to_record_batch`; JSON blob is an Arrow `String` column | Tier-2 filtering requires a ScalarUDF — DataFusion does not support `$.path` syntax natively; this story closes that gap |

---

## Architecture Compliance Rules (MANDATORY)

> Expanded at materialization. Provisional rules from scope analysis:

| Rule | Source | Enforcement |
|------|--------|-------------|
| `json_extract_string` MUST be registered in the DataFusion `SessionContext` via `build_session_context` (or equivalent) so it is available to all PrismQL queries | OQ-002 human decision 2026-08-21 | Compile-pass; confirmed callable from PrismQL `SELECT json_extract_string(raw_extensions, '$.path')` in integration test |
| `tracing::warn!` on invalid JSON MUST have a catalog row in BC-2.16.002 §Canonical Structured Event Catalog in the same atomic commit as the emission site | SAP-1 / PG-LP11-001 | Adversary SAP-1 probe on every pass |
| `prism-query` MUST NOT import `prism-bin` | `dependency-graph.md §Dependency Rules Rule 2` Level 6 / Level 7 ordering | `cargo tree -p prism-query` must not show `prism-bin` after this story |
| UDF NULL-propagation semantics MUST match SQL NULL conventions (absent → NULL, not-string → NULL, invalid-JSON → NULL) | OQ-002 scope | Wire-shape Red Gate assertions |

---

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `serde_json` | per `Cargo.toml` workspace pin | JSON path traversal within UDF |
| `datafusion` | 53.1 (per dependency-graph.md workspace pin) | ScalarUDF registration API |
| Rust stable | per `rust-toolchain.toml` | Build toolchain |

---

## File Structure Requirements (MANDATORY)

> Expanded at materialization.

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-query/src/session_context.rs` (or `memory.rs`) | Modify | Register `json_extract_string` ScalarUDF in `build_session_context` |
| `crates/prism-query/src/udfs/json_extract.rs` (new or inline) | Create/Modify | UDF implementation wrapping `serde_json` path extraction |
| `crates/prism-query/src/session_context.rs` `#[cfg(test)] mod tests` (or `crates/prism-query/tests/`) | Modify | Red Gate tests for all AC/BC clauses |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-21 | story-writer | Initial draft stub — scope capture per OQ-002 human decision 2026-08-21; full BC/AC/RG deferred to materialization. v1-chain obligation documented. |
