---
document_type: behavioral-contract
level: L3
version: "1.10"
status: active
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 1a
inputs: [".factory/specs/domain-spec/capabilities.md", ".factory/specs/domain-spec/invariants.md", ".factory/specs/architecture/decisions/ADR-041-prismql-llm-auto-onboarding-4-layer-teaching-surface-for-automatic-agent-query-authoring.md"]
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: 2026-06-19
modified: 2026-07-08
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.016: E-QUERY-038 Column-Not-Found Plan-Time Gate (L4)

## Description

When a PrismQL query references a column name that does not exist in the `TableRegistry` schema for the specified table and client, the plan-time column-availability gate rejects the query with `E-QUERY-038` before any sensor API call is made. The error payload includes `available_columns` (always present, org-scoped) and `did_you_mean` (present when a Levenshtein distance ≤ 3 match exists against the available column names). This enables an LLM agent to self-correct the column reference in a single retry without human intervention. The gate fires at the same plan-time validation point as the E-QUERY-037 table-availability gate and shares the same `TableRegistry` read.

## Preconditions

1. A PQL query has been parsed successfully by Chumsky (no E-QUERY-001 parse error).
2. The query references a column name in a position where column resolution is possible. The gate covers fourteen clause positions across all three AST modes — **this is an exhaustive closed list**:

   **SQL-mode positions (positions 1–6, `Ast::Sql` and `Ast::SqlPipe` head):**
   - Position 1: `SELECT <column>` projection
   - Position 2: `WHERE <column> = ...` predicate
   - Position 3: `GROUP BY <column>`
   - Position 4: `ORDER BY <column>`
   - Position 5: `JOIN ON <column>`
   - Position 6: `HAVING <column>` / `HAVING <agg>(<column>)` — base-column refs including refs nested inside aggregate function calls such as `count(col)`, validated on parity with WHERE and GROUP BY

   **Filter-mode position (position 7, `Ast::Filter`):**
   - Position 7: Filter-mode root predicate — all columns in the predicate tree, walking `And`/`Or`/`Not` recursively

   **Pipe-mode stage positions (positions 8–12, `Ast::Pipe` and `Ast::SqlPipe` stages):**
   - Position 8: Pipe-mode `| where` stage predicates — every `PipeStage::Where` in the stages vec, walking the embedded `FilterExpr` tree recursively
   - Position 9: SqlPipe `| where` stage predicates — `Ast::SqlPipe` head SQL clauses covered by positions 1–6; `| where` stages walked identically to position 8
   - Position 10: Pipe `| sort` field key references — every field reference in `PipeStage::Sort` sort-key list
   - Position 11: Pipe `| stats ... by` grouping field references — every field reference in `PipeStage::Stats { by: Vec<Expr> }` grouping list
   - Position 12: Pipe `| fields` column references — every field reference in `PipeStage::Fields` inclusion/exclusion list
   - Position 13: Pipe `| enrich f(input_col)` input column — the column argument to the enrichment infusion in each `PipeStage::Enrich` stage; checked against the current binding set BEFORE the stage updates it
   - Position 14: Pipe `| dedup` field path references — every field reference in `PipeStage::Dedup` dedup-key list

   **DERIVED-COLUMN BINDING RULE (pipe-mode positions 8–14):**

   The gate maintains a running binding context `{ available: Set<String>, suspended: bool }` while walking pipe stages in order. Checking is performed against the binding context at the point each stage is reached — the context evolves as stages are processed:

   - **Initial state:** `available = schema_columns(table, OrgId)` (full set registered in `TableRegistry` for this table/client; same source as positions 1–7); `suspended = false`.
   - **Enrich stage (`PipeStage::Enrich`):** Position 13 (input column) is checked against `available` BEFORE this stage updates the binding context. After the position-13 check: output columns are resolved via `InfusionRegistry::udf_descriptors()` → `enrich_descriptor(infusion_id)` → `EnrichStageDescriptor.output_columns`. When output columns are successfully resolved, those names are **unioned** into `available` (the infusion output columns become bindable for downstream stages). When output column resolution fails — because no `InfusionRegistry` is wired into the engine (registry-absent engine context: single-tenant, test, or no-infusions deployment) or due to a defensive lookup failure on an absent descriptor — `suspended := true` and all E-QUERY-038 checking for subsequent stages is suspended (fail-open). **Note (E-QUERY-039 pre-gate guarantee):** in a fully wired engine with an `InfusionRegistry` present, suspension is unreachable-by-design: `check_enrich_udf_availability` (E-QUERY-039) pre-gates unknown UDF names BEFORE the binding-context walk runs, so every enrich stage that reaches the binding-context walk in a registry-wired engine is guaranteed to resolve to a known descriptor. The suspend path is therefore live only in registry-absent engine contexts. Rationale: downstream references to infusion output columns (e.g., `cvss_base_score` from the canonical NVD pivot per BC-2.06.019 v1.13 §PC-4) are valid at execution time; a false positive on a correct query is forbidden by invariant FP-001.
   - **Stats stage (`PipeStage::Stats`):** The by-field refs (position 11) and aggregation expressions are checked against `available` BEFORE this stage updates the binding context. After those checks: `available` is **replaced** with `{explicit aggregate aliases} ∪ {bare column names from by_fields}`. Rationale: after a `| stats` stage, only aggregate outputs and GROUP BY keys remain in the result; continuing to check against the original schema would produce false negatives. Anonymous aggregations (no explicit `as alias`) do not contribute to the replacement set — their DataFusion-generated names are not predictable at plan time; fail-open for those references in subsequent stages. If any by-field expression is non-static (not a bare `Field` name): `suspended := true`.
   - **All other stages** (Where, Sort, Fields, Dedup): `available` and `suspended` are unchanged.
   - **Suspension propagation:** once `suspended = true`, ALL subsequent stages in the pipe sequence skip E-QUERY-038 entirely. Not reversible within a single query.

3. The table referenced in the query has been validated to exist in the `TableRegistry` (E-QUERY-037 passed — no point checking column availability for a non-existent table).
4. The `TableRegistry` has been initialized with the per-client schema for the requesting org.

## Postconditions

### Gate firing conditions

The E-QUERY-038 gate fires when a query passes the E-QUERY-037 table-availability check (the table exists) but references a column name that is NOT in the `TableRegistry` schema for that table under the requesting client's `OrgId`.

The gate fires at plan time (after parse, before fan-out), consistent with E-QUERY-037. No sensor API call is made for a rejected query.

### E-QUERY-038 error payload shape

```
E-QUERY-038: column '{column}' not found in table '{table}' for client '{client_id}';
  available_columns: [...],
  did_you_mean: Option<String>
```

**Payload fields:**

- `column`: the exact column name as written by the model in the query (e.g., `"sevrity"`)
- `table`: the table name in which the column was not found (e.g., `"crowdstrike_alerts"`)
- `client_id`: the requesting client's org slug (identifies which per-client schema was checked)
- `available_columns`: an array of all column names registered in the `TableRegistry` for `(table, OrgId)` at error-construction time. This list is ALWAYS present (never null, never omitted). If the table has zero columns, the array is empty `[]`. **Org-scoped:** same `filter_to_org_visible` principle as E-QUERY-037 — available_columns reflects ONLY the columns for this client's registered schema for this table; not columns from a different client's sensor overlay for the same table.
- `did_you_mean`: optional field. Present when the Levenshtein distance between the requested column name and the closest available column name is ≤ 3. Implementation: `strsim::levenshtein` (same crate used by E-QUERY-037 per D-1163). If present, contains the single closest-match column name as a string. If absent (no match within threshold), the field is omitted (not null, not empty string — absent).

### Structured error response (MCP surface)

E-QUERY-038 surfaces as MCP `-32602 INVALID_PARAMS` (caller-resolvable — the model supplied an invalid column name; it can retry with a correct name). This is consistent with E-QUERY-037's mapping.

The error is delivered as a BC-2.10.007 structured error response with:
- `code: "E-QUERY-038"`
- `category: "validation"`
- `severity: "broken"` (query cannot proceed without correction)
- `retryable: false` (without a configuration change or corrected query — retry with the same column name will fail identically)
- `suggestion: "Call prism_describe('<client_id>') to see available columns, or use the available_columns field in this error to correct the column name."`

### Injection-safety of `available_columns`

The `available_columns` list is sourced ENTIRELY from the `TableRegistry`, which is populated from operator-controlled TOML spec files (validated at load time per ADR-005 pattern). Specifically:

- Column names in the `TableRegistry` are operator-defined schema field names (e.g., `"severity"`, `"host_name"`, `"timestamp"`).
- They DO NOT contain: credential values, API key substrings, full URL paths, authentication tokens, internal connection strings, or any runtime sensor API response data.
- The injection-safety requirement is: **no string in `available_columns` may contain a secret-shaped value** (API key pattern, bearer token, password pattern). This is verifiable because the column names come from TOML spec files, not from live API responses.
- `did_you_mean` is derived from `available_columns` via Levenshtein computation — same injection-safety guarantee applies.

### Implementation location

The column-not-found gate is colocated with the E-QUERY-037 table-availability gate in `prism-query/src/engine.rs` (or equivalent plan validation step). Both gates share the single `TableRegistry` read at plan time. The column gate fires AFTER the table gate (table must exist before checking its columns).

**Gate positions — `check_query_column_availability`:**

| Position | AST mode | Clause | Extraction mechanism |
|----------|----------|--------|---------------------|
| 1 | SQL (`Ast::Sql`, `Ast::SqlPipe` head) | SELECT projection | `extract_field_paths_from_expr` per `SelectItem::Expr` |
| 2 | SQL | WHERE predicate | `extract_predicate_columns` over `sql_query.where_` |
| 3 | SQL | GROUP BY | `extract_field_paths_from_expr` per group-by `Expr` |
| 4 | SQL | ORDER BY | `extract_field_paths_from_expr` per `OrderBy::expr` |
| 5 | SQL | JOIN ON | `extract_field_paths_from_expr` over `join.on` |
| 6 | SQL | HAVING predicate | `extract_predicate_columns` over `sql_query.having` (same `Option<Predicate>` type as WHERE; same extraction path) |
| 7 | Filter (`Ast::Filter`) | Root predicate | `extract_predicate_columns` recursively over the root `FilterExpr` tree (`And`/`Or`/`Not` recursive walk) |
| 8 | Pipe (`Ast::Pipe`) | `| where` stage predicates | `extract_predicate_columns` over each `PipeStage::Where(FilterExpr)` in `stages` vec |
| 9 | SqlPipe (`Ast::SqlPipe`) | `| where` stage predicates | Head SQL covered by positions 1–6; `| where` stages in `sq.stages` walked identically to position 8 |
| 10 | Pipe (`Ast::Pipe`, `Ast::SqlPipe`) | `| sort` field keys | `extract_field_paths_from_expr` per each field reference in `PipeStage::Sort` sort-key list |
| 11 | Pipe (`Ast::Pipe`, `Ast::SqlPipe`) | `| stats ... by` grouping refs | `extract_field_paths_from_expr` per each field reference in `PipeStage::Stats { by: Vec<Expr> }` grouping list |
| 12 | Pipe (`Ast::Pipe`, `Ast::SqlPipe`) | `| fields` column refs | `extract_field_paths_from_expr` per each field reference in `PipeStage::Fields` inclusion/exclusion list |
| 13 | Pipe (`Ast::Pipe`, `Ast::SqlPipe`) | `\| enrich` input column | `extract_field_paths_from_expr` for the input argument in `PipeStage::Enrich { input_col }` — checked against the current binding set BEFORE the Enrich stage updates it |
| 14 | Pipe (`Ast::Pipe`, `Ast::SqlPipe`) | `\| dedup` field keys | `extract_field_paths_from_expr` per each field reference in `PipeStage::Dedup { fields: Vec<FieldPath> }` dedup-key list |

HAVING was added at v1.5 to close a pedagogical coverage asymmetry: the sibling gates E-QUERY-037 and E-QUERY-039 both walk HAVING; omitting it from E-QUERY-038 caused a `HAVING count(typo_col)` typo to bypass the clean "column not found" diagnostic and surface a less-actionable DataFusion error instead.

### PrismError variant

`PrismError::ColumnNotFound(Box<ColumnNotFoundDetails>)` is the variant in `prism-core/src/error.rs`, where `ColumnNotFoundDetails` is a `#[non_exhaustive]` struct carrying `{ column: String, table: String, client_id: String, available_columns: Vec<String>, did_you_mean: Option<String> }`. The boxed form was chosen at implementation time (story v1.4 CORRECTION-2, gate=83) because the `Vec<String>` field for `available_columns` pushes the inline variant over the `clippy::result_large_err` threshold — the same justification as `TableNotAvailableDetails` for E-QUERY-037. The `#[non_exhaustive]` attribute on `ColumnNotFoundDetails` is required per CLAUDE.md `#[non_exhaustive]` discipline (the struct is a public type in `prism-core`; external match arms must include a wildcard). The Display output and MCP surface are unchanged from the original design.

`#[non_exhaustive]` is NOT applicable to `PrismError` enum variants themselves (it is the enum-level attribute). The `PrismError` enum already carries `#[non_exhaustive]` on the type (CLAUDE.md conventions). The new variant is added to the existing non-exhaustive enum.

`map_prism_error` in `prism-mcp/src/error_mapping.rs` MUST add an explicit `-32602 INVALID_PARAMS` arm for `PrismError::ColumnNotFound` — it MUST NOT fall through to the `#[non_exhaustive]` catch-all `-32000`.

### Interaction with DataFusion column resolution

DataFusion itself would produce a column resolution error if the query reached execution. The E-QUERY-038 gate is a plan-time pre-check that fires BEFORE DataFusion receives the query. This means the model gets a structured, pedagogical error with `available_columns` rather than a DataFusion internal error string (which would be redacted into E-QUERY-034).

## Invariants

- DI-019: This gate fires at plan time, before fan-out, consistent with the security limits principle that problems are caught early.
- DI-004: The query rejection event is included in the `AuditEntry` for the `query` tool call (outcome: `"rejected"`, reason: `"column_not_found"`).
- DI-002: `available_columns` contains no credential values (operator TOML schema → `TableRegistry`; no sensor API data).
- DI-008: `available_columns` is org-scoped — columns for this client's schema only; not columns from another client's sensor overlay.
- **AST-completeness (OBS-002):** For every `Ast::*` variant carrying a `Predicate` sub-tree or schema-bound column references, the column-availability gate MUST walk all embedded column positions. New `Ast` variants introduced in future development require a corresponding gate arm in `check_query_column_availability`. The `_ => Ok(())` fallback is deliberate fail-open ONLY for variants that carry no schema-bound column references (e.g., a hypothetical non-schema-referencing utility node). Any `Ast::*` variant that introduces new schema-bound column positions MUST extend the gate enumeration — omission is a HIGH-severity finding in adversarial review.
- **False-positive prohibition (FP-001):** The plan-time gate MUST NEVER reject a query that would succeed at execution time. When binding provenance is uncertain — the infusion output schema is not statically resolvable, an anonymous aggregate alias is not predictable at plan time, or any non-Field by-expression produces an unpredictable column name — the gate MUST fail open (suspend E-QUERY-038 for affected downstream stages). A false negative (a column typo in infusion output is not caught) is an acceptable cost; a false positive on a correct query (e.g., the canonical NVD pivot where `cvss_base_score` is a real infusion output) is a blocking defect.

## Error Cases

| Error Code | Condition | Behavior |
|------------|-----------|----------|
| `E-QUERY-038` | Column referenced in query not found in `TableRegistry` schema for the specified table + client | MCP `-32602 INVALID_PARAMS`; structured payload with `column`, `table`, `client_id`, `available_columns` (always), `did_you_mean` (when within distance ≤ 3) |

**Gate ordering (distinct-from clarification):**
- E-QUERY-001 fires first (parse error — Chumsky cannot parse the query at all)
- E-QUERY-037 fires next (table not in `TableRegistry`)
- E-QUERY-038 fires after 037 (table exists but column does not)
- E-QUERY-034 is the fallback for DataFusion execution failures that reach the engine

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-039 | Query references `sevrity` (typo) in `crowdstrike_alerts` where `severity` is a registered column | `E-QUERY-038` with `column: "sevrity"`, `available_columns: ["severity", "host_name", ...]`, `did_you_mean: "severity"` (Levenshtein distance 1) |
| EC-11-040 | Query references `completely_nonexistent_field` where no column is within Levenshtein distance 3 | `E-QUERY-038` with `column: "completely_nonexistent_field"`, `available_columns: [...]`, `did_you_mean` ABSENT (not null — the field is omitted from the response) |
| EC-11-041 | Table has zero registered columns (empty schema) | `E-QUERY-038` with `available_columns: []`, `did_you_mean` absent |
| EC-11-042 | Query references a column that is registered in another client's overlay for the same table, but not in the requesting client's schema | `E-QUERY-038` fired; `available_columns` shows only the requesting client's columns — the other client's column name does not appear |
| EC-11-043 | Query references both a non-existent table AND a non-existent column | E-QUERY-037 fires first (table not found); E-QUERY-038 does not fire (gate is ordered: table check before column check) |
| EC-11-044 | Multiple columns are invalid in the same query | Behavior at implementer discretion: the gate MAY report the FIRST invalid column encountered (fail-fast) or ALL invalid columns (collect-all). Either is acceptable. The BC requires at minimum one E-QUERY-038 error is returned (not a silent pass). |
| EC-11-045 | `did_you_mean` is present but the suggested column is itself the only available column (table has one column and the model typed it wrong) | `did_you_mean` present with that single column's name; `available_columns` contains `[that_column_name]` |
| EC-11-046 | Column typo in HAVING predicate: `SELECT severity, count(*) FROM crowdstrike_alerts GROUP BY severity HAVING count(typo_col) > 5` | `E-QUERY-038` with `column: "typo_col"`, `table: "crowdstrike_alerts"`. The base-column ref inside the `count(…)` aggregate function is extracted and validated against the `TableRegistry` schema; the query does NOT reach DataFusion. |
| EC-11-047 | Filter-mode query `severity_id = 'high'` against `crowdstrike_alerts` where `severity_id` is not registered but `severity` is | `E-QUERY-038` with `column: "severity_id"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"` (position 7 — filter-mode root predicate) |
| EC-11-048 | Pipe-mode `FROM crowdstrike_alerts \| where severity_id IEQ 'high'` where `severity_id` is not registered | `E-QUERY-038` with `column: "severity_id"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"` (position 8 — pipe `\| where` stage) |
| EC-11-049 | SqlPipe `SELECT * FROM crowdstrike_alerts \| where severity_id IEQ 'high'` where `severity_id` is not registered | `E-QUERY-038` with `column: "severity_id"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"` (position 9 — sqlpipe `\| where` stage; head SQL columns covered by positions 1–6) |
| EC-11-050 | Pipe `FROM crowdstrike_alerts \| sort sevrity desc` where `sevrity` is not registered | `E-QUERY-038` with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"` (position 10 — pipe `\| sort` key) |
| EC-11-051 | Pipe `FROM crowdstrike_alerts \| stats count by sevrity` where `sevrity` is not registered | `E-QUERY-038` with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"` (position 11 — pipe `\| stats ... by` grouping ref) |
| EC-11-052 | Pipe `FROM crowdstrike_alerts \| fields sevrity, timestamp` where `sevrity` is not registered | `E-QUERY-038` with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"` (position 12 — pipe `\| fields` column ref) |
| EC-11-053 | Pipe `FROM crowdstrike_alerts \| stats count() as cnt by severity \| sort cnt` where `cnt` is a stats output alias | No E-QUERY-038 on `cnt` — after the Stats stage the binding set is replaced with `{cnt, severity}`; `\| sort cnt` finds `cnt` in the binding context (ADV-FIX-P2-CRIT-002 regression closed) |
| EC-11-054 | Canonical NVD pivot `FROM armis_devices \| where has device_cves_first \| enrich cvss_base_score(device_cves_first) \| where cvss_base_score >= 7.0 \| sort cvss_base_score desc` | No E-QUERY-038 on `cvss_base_score`. **Union path (registry-wired engine):** `cvss_base_score` is resolved via `udf_descriptors()→enrich_descriptor()→output_columns` and unioned into `available`; downstream `\| where` and `\| sort` stages find it in the binding set — no E-QUERY-038. **Suspend-and-pass path (registry-absent engine):** `suspended = true` after Enrich — downstream stages skip E-QUERY-038 entirely (fail-open). Either path: no E-QUERY-038 on `cvss_base_score` (ADV-FIX-P2-CRIT-001 regression closed; canonical form per BC-2.06.019 v1.13 §PC-4) |
| EC-11-055 | Post-enrich typo `FROM armis_devices \| enrich cvss_base_score(device_cves_first) \| where cvvs_base_score >= 7.0` where `cvvs_base_score` is a typo | Behavior depends on whether an `InfusionRegistry` is wired into the engine. **(a) Registry-absent** (single-tenant/test/no-infusions engine): `suspended = true` after Enrich → no E-QUERY-038 on `cvvs_base_score` (fail-open; false negative tolerated per FP-001). **(b) Registry-wired** (production engine): `cvss_base_score` is resolved via `udf_descriptors()→enrich_descriptor()→output_columns` and unioned into `available`; `cvvs_base_score` is NOT in that set → E-QUERY-038 fires on `cvvs_base_score`. Either outcome is correct — the gate MUST NOT false-positive on the exact infusion output column name (`cvss_base_score`). |
| EC-11-056 | Enrich input typo `FROM armis_devices \| enrich cvss_base_score(device_cves_firsst)` where `device_cves_firsst` is not registered | `E-QUERY-038` with `column: "device_cves_firsst"`, `table: "armis_devices"`, `did_you_mean: "device_cves_first"` (Levenshtein distance 1; position 13 — enrich input column) |
| EC-11-057 | Dedup typo `FROM armis_devices \| dedup device_diid` where `device_id` is registered and `device_diid` is a typo | `E-QUERY-038` with `column: "device_diid"`, `table: "armis_devices"`, `did_you_mean: "device_id"` (Levenshtein distance 2; position 14 — dedup field key) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `query("SELECT sevrity FROM crowdstrike_alerts", clients=["acme"])` where `severity` is registered but `sevrity` is not | `E-QUERY-038` with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `available_columns` includes `"severity"`, `did_you_mean: "severity"` | happy-path (did_you_mean) |
| `query("SELECT completely_bogus_field FROM crowdstrike_alerts", clients=["acme"])` | `E-QUERY-038` with `column: "completely_bogus_field"`, `available_columns` includes real column names, `did_you_mean` absent | no-suggestion |
| `query("SELECT * FROM crowdstrike_alerts")` when `crowdstrike_alerts` is not registered | `E-QUERY-037` (not E-QUERY-038 — table doesn't exist) | gate-ordering |
| `query("SELECT sevrity FROM crowdstrike_alerts", clients=["acme"])` in multi-tenant deployment; "globex" has claroty_alerts with `severity` column | `available_columns` for `crowdstrike_alerts` contains ONLY acme's crowdstrike_alerts columns — contoso/globex column names absent | org-isolation |
| MCP error code for E-QUERY-038 | Surfaces as `-32602 INVALID_PARAMS` (not `-32000`) | mcp-mapping |
| `query("SELECT severity, count(*) FROM crowdstrike_alerts GROUP BY severity HAVING count(typo_col) > 5", clients=["acme"])` | `E-QUERY-038` with `column: "typo_col"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `available_columns` includes registered columns, `did_you_mean` present if a close match exists | having-position |
| `query("severity_id = 'high'", clients=["acme"])` in filter mode against `crowdstrike_alerts` where `severity_id` not registered but `severity` is | `E-QUERY-038` with `column: "severity_id"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `available_columns` includes `"severity"`, `did_you_mean: "severity"` | filter-mode (position 7) |
| `query("FROM crowdstrike_alerts \| where severity_id IEQ 'high'", clients=["acme"])` where `severity_id` not registered | `E-QUERY-038` with `column: "severity_id"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `did_you_mean: "severity"` | pipe-where (position 8) |
| `query("SELECT * FROM crowdstrike_alerts \| where severity_id IEQ 'high'", clients=["acme"])` where `severity_id` not registered | `E-QUERY-038` with `column: "severity_id"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `did_you_mean: "severity"` | sqlpipe-where (position 9) |
| `query("FROM crowdstrike_alerts \| sort sevrity desc", clients=["acme"])` where `sevrity` not registered | `E-QUERY-038` with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `did_you_mean: "severity"` | pipe-sort (position 10) |
| `query("FROM crowdstrike_alerts \| stats count by sevrity", clients=["acme"])` where `sevrity` not registered | `E-QUERY-038` with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `did_you_mean: "severity"` | pipe-stats-by (position 11) |
| `query("FROM crowdstrike_alerts \| fields sevrity, timestamp", clients=["acme"])` where `sevrity` not registered | `E-QUERY-038` with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`, `did_you_mean: "severity"` | pipe-fields (position 12) |
| `query("FROM crowdstrike_alerts \| stats count() as cnt by severity \| sort cnt", clients=["acme"])` | No E-QUERY-038 — `cnt` is in the binding set after Stats stage (`available = {"cnt", "severity"}`); `\| sort cnt` finds it (EC-11-053) | aggregate-alias (position 11 derived) |
| `query("FROM armis_devices \| where has device_cves_first \| enrich cvss_base_score(device_cves_first) \| where cvss_base_score >= 7.0 \| sort cvss_base_score desc", clients=["acme"])` | No E-QUERY-038 on `cvss_base_score` — registry-wired engine: `cvss_base_score` unioned into binding set via `udf_descriptors()→enrich_descriptor()→output_columns`; registry-absent engine: `suspended = true`; either path suppresses downstream E-QUERY-038 (EC-11-054; canonical NVD pivot per BC-2.06.019 v1.13 §PC-4) | enrich-output (position 13 derived) |
| `query("FROM armis_devices \| enrich cvss_base_score(device_cves_firsst)", clients=["acme"])` where `device_cves_firsst` not registered | `E-QUERY-038` with `column: "device_cves_firsst"`, `table: "armis_devices"`, `client_id: "acme"`, `did_you_mean: "device_cves_first"` | enrich-input (position 13) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (VP-TBD) | E-QUERY-038 `available_columns` for any `(table, OrgId)` contains no strings matching credential patterns (API key regex, bearer token regex) | proptest |
| (VP-TBD) | E-QUERY-038 fires before DataFusion column resolution — no DataFusion internal error strings appear in the E-QUERY-038 response | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC defines a new plan-time validation gate in the PQL query engine that rejects queries referencing non-existent columns. CAP-015 is the authoritative capability for the PQL query engine including its validation gates, error responses (`E-QUERY-NNN` codes), and plan-time checks. The E-QUERY-038 column-not-found gate is a plan-time check in the query engine layer, exactly what CAP-015 governs. |
| L2 Invariants | DI-002, DI-004, DI-008, DI-019 |
| ADR | ADR-041 v1.1 §L4 — "NEW: column-not-found" E-QUERY-038; §Architectural Surface — "L4 column-not-found gate" |
| Architecture Module | SS-11 (Query Execution) |
| Priority | P1 |

## Related BCs

- BC-2.11.001 — depends on: `query` MCP tool is the entry point; E-QUERY-038 is one of its error conditions; BC-2.11.001 Error Cases table should reference this BC (story-writer propagation required)
- BC-2.10.012 — composes with: `prism_describe` returns `available_columns` per table; if the model called `prism_describe` first, it would have avoided this error; if it didn't, E-QUERY-038 is the correction backstop
- BC-2.11.017 — composes with: E-QUERY-038 is the new code in the pedagogical error suite; BC-2.11.017 covers upgrades to existing codes; this BC covers the new code
- BC-2.11.010 — deferred parity: `explain_query` should fire E-QUERY-038 (and E-QUERY-037/002/003 enrichments) at plan time for consistency with the `query` path. This parity is NOT currently in BC-2.11.010 §Error Cases (v1.6) and is NOT in scope for S-DEMO-PRISMQL-ONBOARDING-001-B. Follow-up story S-EXPLAIN-PARITY-001 tracks the BC-2.11.010 amendment + `explain.rs` wiring; see adjudication note `.factory/specs/architecture/scoping/onboarding-001-B-obs-adjudication.md` §OBS-2. DO-NOT-REFLAG this gap for the 001-B cascade.

## Architecture Anchors

- `architecture/decisions/ADR-041` §L4 — "E-QUERY-038 allocation: This is a new code required by this ADR"
- `architecture/decisions/ADR-039` — `TableRegistry` and org-scoped enumeration pattern; E-QUERY-038 follows the same org-scoping model as E-QUERY-037
- `architecture/api-surface.md` — E-QUERY-NNN error code namespace; E-QUERY-038 is the next sequential code after E-QUERY-037

## Story Anchor

S-DEMO-PRISMQL-ONBOARDING-001-B — the BC's behavior (E-QUERY-038 column-not-found plan-time gate, available_columns, did_you_mean) is implemented by this story per its `anchor_bcs: [BC-2.11.016, ...]` frontmatter (POL-4/POL-5).

## VP Anchors

VP assignments TBD — assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.10 | FIX-IEQ-ERRPATH-001-bc-2-11-016-enrich-binding-clarification | 2026-07-08 | product-owner | **Enrich binding rule — explicit resolution surface + registry-absent dead-code note (human-directed 2026-07-08).** §Preconditions DERIVED-COLUMN BINDING RULE enrich branch: (1) named the output schema resolution path explicitly (`InfusionRegistry::udf_descriptors()` → `enrich_descriptor(infusion_id)` → `EnrichStageDescriptor.output_columns`); (2) clarified that suspension fires ONLY when no `InfusionRegistry` is wired into the engine (registry-absent engine context: single-tenant, test, or no-infusions deployment) or on defensive lookup failure — not as the normal wired-registry path; (3) added E-QUERY-039 pre-gate guarantee note: in a wired engine the suspend path is unreachable-by-design (`check_enrich_udf_availability` pre-gates unknown UDF names before the binding-context walk, guaranteeing every enrich stage reaching the walk in a wired engine resolves to a known descriptor). §Edge Cases: EC-11-054 annotated with both execution paths (union path when registry-wired; suspend-and-pass path when registry-absent). EC-11-055 reframed from "if statically resolvable" to explicit registry-wired vs. registry-absent dichotomy. §Canonical Test Vectors: enrich-output vector expected output updated to describe both paths. POL-25 sweep: BC-2.11.004 v1.17, BC-2.11.020 v1.5, error-taxonomy v2.24 E-QUERY-038 row — none assert universal-suspend; no wording update required for those files. FP-001 invariant text unchanged. |
| 1.9 | FIX-IEQ-ERRPATH-001-grammar-keyword-correction-sort-by | 2026-07-08 | product-owner | **Grammar keyword correction: `\| sort by` → `\| sort` (position 10).** Test-writer grammar verification (fix-PR FIX-IEQ-ERRPATH-001 LOCAL pass-3) confirmed the PrismQL pipe parser has NO `by` keyword in the `sort` stage — `kw_ci("sort")` is followed directly by `sort_expr` (field + optional direction), with no intervening `by` token. `\| sort cnt desc` is the real syntax; `\| sort by cnt` would parse `by` as a field name. This is the SECOND grammar-keyword miss in this cascade (first: `\|project`→`\|fields` fixed in v1.7/v1.15/v1.3 sweep). Process-Gap Lesson 15 recurrence recorded. **Fixed sites:** §Preconditions.2 position-10 label (`Pipe \`\| sort by\`` → `Pipe \`\| sort\``); §Implementation-location Gate-positions table row 10 clause label; EC-11-050 description trailing position label; EC-11-053 query string and description (`\| sort by cnt` → `\| sort cnt`, both instances); §Canonical Test Vectors aggregate-alias row query string and expected-output description (same correction). **Note:** `\| stats count() as cnt by severity` retains its `by` keyword — `by` is real grammar inside the `stats` stage (grouping clause); only the `sort` stage is affected by this correction. |
| 1.8 | ADV-FIX-P2-CRIT-001-CRIT-002-HIGH-002-HIGH-003 | 2026-07-08 | product-owner | **ADV-FIX-P2-CRIT-001/002 + HIGH-002/003 closure (FIX-IEQ-ERRPATH-001 LOCAL pass-2).** §Preconditions.2: gate expanded from twelve to **fourteen positions**; added (13) `\| enrich` input column and (14) `\| dedup` field keys. Added **DERIVED-COLUMN BINDING RULE** — running `{ available: Set<String>, suspended: bool }` context maintained while walking pipe stages in order: Enrich stage unions infusion output columns into `available` when statically resolvable, else sets `suspended = true` (fail-open; closes CRIT-001 canonical NVD pivot false-positive regression); Stats stage **replaces** `available` with `{explicit_aliases ∪ by_fields}` (closes CRIT-002 aggregate-alias false-positive regression); all other stages leave binding context unchanged; suspension is not reversible within a single query. **HIGH-002 adjudication: IN-SCOPE (position 13)** — enrich input column IS schema-bound/derived-binding-bound and statically checkable; typo there currently yields downstream opaque failure, same pedagogical class as rest of gate. **HIGH-003:** Dedup adjudication: **GATED (position 14)** — `PipeStage::Dedup` carries `Vec<FieldPath>` (plain column refs, same as sort keys position 10); no semantic reason to fail-open. §Invariants: added **FP-001 false-positive prohibition** — gate MUST fail open when binding provenance is uncertain. §Implementation location Gate-positions table: added rows 13–14. §Edge Cases: added EC-11-053 (aggregate alias downstream — no E-QUERY-038), EC-11-054 (enrich output downstream — no E-QUERY-038; canonical NVD pivot), EC-11-055 (post-enrich typo — fail-open tolerated), EC-11-056 (enrich input typo — E-QUERY-038 position 13), EC-11-057 (dedup typo — E-QUERY-038 position 14). §Canonical Test Vectors: added three new vectors for positions 13/14 and binding-context correctness. |
| 1.7 | FIX-IEQ-ERRPATH-001-grammar-keyword-correction | 2026-07-08 | product-owner | **Grammar keyword correction: `\| project` → `\| fields` (position 12).** Test-writer grammar verification (fix-PR FIX-IEQ-ERRPATH-001) confirmed the PrismQL pipe parser has no `\| project` keyword; the projection stage keyword is `\| fields` (`PipeStage::Fields`). Fixed: §Preconditions.2 position-12 prose (`Pipe \`| project\` (\`fields\` stage)` → `Pipe \`| fields\``); §Implementation location Gate-positions table row 12 clause label; EC-11-052 vector query string (`\| project sevrity, host_name` → `\| fields sevrity, timestamp`, mirroring the verified test form); Canonical Test Vectors last row (same query string fix + category label `pipe-project` → `pipe-fields`). |
| 1.6 | ADV-FIX-P1-HIGH-001-HIGH-002-OBS-002-DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 | 2026-07-08 | product-owner | **ADV-FIX-P1-HIGH-001 + ADV-FIX-P1-HIGH-002 + ADV-FIX-P1-OBS-002 closure (DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 fix-PR).** §Preconditions.2: expanded exhaustive position enumeration from 6 to 12 positions covering all three AST modes. Added: (7) Filter-mode root predicate (`Ast::Filter` — `And`/`Or`/`Not` recursive walk), (8) Pipe-mode `\| where` stage predicates (`Ast::Pipe`, every `PipeStage::Where`), (9) SqlPipe `\| where` stage predicates (`Ast::SqlPipe` head covered by positions 1–6; stages walked like position 8), (10) Pipe `\| sort by` field keys (`PipeStage::Sort`), (11) Pipe `\| stats ... by` grouping refs (`PipeStage::Stats { by }`), (12) Pipe `\| project` column refs (`PipeStage::Fields`). **MED-002 adjudication — positions 10/11/12 IN-SCOPE:** SQL GROUP BY (position 3) and ORDER BY (position 4) are already gated; `\| stats by` and `\| sort by` are their pipe analogues — asymmetry is not justifiable pedagogically; `\| project` is the pipe analogue of SELECT projection (position 1, already gated). Demo narrative "the error is the tutorial" and production-grade default both require complete coverage. Blast radius is moderate (reuses same helpers); no genuine product tradeoff justifies deferral. §Implementation location Gate-positions table: added `AST mode` column and rows 7–12 with extraction mechanisms. §Invariants: added AST-completeness invariant (OBS-002) — every `Ast::*` variant carrying schema-bound column refs requires a gate arm; `_ => Ok(())` is deliberate fail-open only for schema-free variants. New edge cases EC-11-047 through EC-11-052 (filter-mode, pipe-where, sqlpipe-where, pipe-sort, pipe-stats-by, pipe-project). New canonical test vectors for positions 7–12. `modified: 2026-07-08`. |
| 1.5 | F-PWL1-LOW-001-having-gate-mandate | 2026-06-28 | product-owner | MANDATE verdict on F-PWL1-LOW-001 (E-QUERY-038 HAVING coverage asymmetry). Added HAVING (Position 6) to the column gate scope. Precondition 2 expanded from illustrative "e.g." list to exhaustive six-position enumeration. §Implementation location: added gate-positions table documenting all 6 positions and their extraction mechanisms; added rationale note for HAVING addition. New edge case EC-11-046 (HAVING `count(typo_col)` pattern). New canonical test vector for `having-position`. HAVING uses same `Option<Predicate>` type as WHERE and same `extract_predicate_columns` extraction path — zero new machinery. |
| 1.4 | F-001B-SCFRESH-MED-001-story-anchor-fix | 2026-06-22 | product-owner | F-001B-SCFRESH-MED-001 closure (POL-4 story-anchor mis-anchoring): `## Story Anchor` corrected from placeholder `S-5.04 (or dedicated ADR-041 teaching story — to be assigned by story-writer)` to the actual implementing story `S-DEMO-PRISMQL-ONBOARDING-001-B`. Exhaustive BC metadata audit: all other surfaces (frontmatter fields, lifecycle_status, subsystem, capability, H1/BC-INDEX title match, DI citations, changelog schema, Related BCs existence) verified clean. `modified:` quote-normalization (YAML scalar, no quotes needed). |
| 1.3 | F-001B-FRESH2-MED-001-pol20-normalization | 2026-06-22 | product-owner | POL-20 normalization: `introduced: ADR-041-teaching-burst-2026-06-19` → `introduced: 2026-06-19` (opaque burst-ID format prohibited by POL-20 anchored-regex; ISO date extracted). No body semantics changed. |
| 1.2 | OBS-001-B-FRESH-001-emitter-shape | 2026-06-22 | product-owner | OBS-001-B-FRESH-001 closure: §PrismError-variant prose updated to reflect the CORRECTION-2 boxed implementation. Replaced "not boxed — … box if necessary" speculative text with the ratified shape: `PrismError::ColumnNotFound(Box<ColumnNotFoundDetails>)` where `ColumnNotFoundDetails` is `#[non_exhaustive]` with `{ column, table, client_id, available_columns, did_you_mean }`. Boxing was triggered by `clippy::result_large_err` (same as `TableNotAvailableDetails`; story v1.4 CORRECTION-2, gate=83). Field set, semantics, Display, and MCP surface unchanged. |
| 1.1 | onboarding-001-B-obs-adjudication | 2026-06-22 | product-owner | OBS-2 adjudication: tightened §Related-BCs note for BC-2.11.010. Replaced ambiguous "explain_query performs the same plan-time validation; E-QUERY-038 fires on explain_query as well as query" with explicit deferral anchor (S-EXPLAIN-PARITY-001, out-of-scope for 001-B, DO-NOT-REFLAG). See `.factory/specs/architecture/scoping/onboarding-001-B-obs-adjudication.md` §OBS-2. |
| 1.0 | ADR-041-teaching-burst-2026-06-19 | 2026-06-19 | product-owner | Initial draft — ADR-041 L4 E-QUERY-038 column-not-found plan-time gate |
