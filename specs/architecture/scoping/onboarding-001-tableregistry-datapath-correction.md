---
document_type: architecture-scoping-correction
version: "1.0"
status: active
producer: architect
timestamp: "2026-06-20T00:00:00Z"
traces_to: [S-DEMO-PRISMQL-ONBOARDING-001-A, S-DEMO-PRISMQL-ONBOARDING-001-B]
inputs:
  - crates/prism-query/src/table_registry.rs
  - crates/prism-query/src/engine.rs
  - crates/prism-mcp/src/server.rs
  - crates/prism-mcp/src/resources.rs
  - crates/prism-spec-engine/src/spec_parser.rs
  - crates/prism-spec-engine/src/overlay.rs
  - .factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md
  - .factory/stories/S-DEMO-PRISMQL-ONBOARDING-001-A-mcp-teaching-surface-l1-l2-l3.md
  - .factory/stories/S-DEMO-PRISMQL-ONBOARDING-001-B-query-engine-l4-errors-normalized-pql.md
  - .factory/specs/architecture/decisions/ADR-041-prismql-llm-auto-onboarding-4-layer-teaching-surface-for-automatic-agent-query-authoring.md
routing: story-writer applies edits to 001-A + 001-B; no BC amendment required
---

# Onboarding-001 TableRegistry Data-Path Correction

## Purpose

Resolve the shared data-path defect flagged by the remove-uncertainty passes on
S-DEMO-PRISMQL-ONBOARDING-001-A (R1 CRITICAL, R2 HIGH) and
S-DEMO-PRISMQL-ONBOARDING-001-B (FLAG-001) before TDD begins.

This document is the canonical architect adjudication. Story-writer applies the
edit list below to both stories. No production code is written here.

---

## 1. Corrected Data-Source Model

### 1.1 TableRegistry — what it actually stores

`TableRegistry` (`crates/prism-query/src/table_registry.rs`) is a concrete
`#[non_exhaustive] struct`, not a trait. It stores:

- `registered: Arc<RwLock<HashSet<String>>>` — table name strings only
  (e.g. `"crowdstrike_alerts"`).
- `sensor_by_table: Arc<RwLock<HashMap<String, String>>>` — reverse map of
  table name → sensor_id.

It has NO per-column schema. There is no `columns: Vec<ColumnSpec>` field, no
`column_type` data, no `available_columns` method.

### 1.2 Where column-level schema lives

Column-level schema is in the spec layer:

- `prism_spec_engine::spec_parser::TableSpec { columns: Vec<ColumnSpec>, .. }`
- `prism_spec_engine::spec_parser::ColumnSpec { name: String, column_type: ColumnType, .. }`
- These are reachable via `SensorSpec.tables` (in `ConfigSnapshot.sensor_specs`)
  and via `ResolvedSensorSpec.spec.tables` (in `resolved_spec_map`).

### 1.3 How `check_availability_gate` already receives spec data

`check_table_availability` in `engine.rs` (the E-QUERY-037 gate) passes
`resolved_spec_map: Option<&HashMap<ResolvedSpecKey, ResolvedSensorSpec>>` to
`TableRegistry::check_availability_gate`. This parameter is already threaded through
from `QueryEngine.resolved_spec_map` at call time.

`ResolvedSpecKey = (OrgSlug, SensorId)`. A lookup of
`resolved_spec_map.get(&(org_slug, sensor_id))` yields a `ResolvedSensorSpec` whose
`.spec.tables` is `Vec<TableSpec>`, and each `TableSpec.columns` is
`Vec<ColumnSpec>`.

### 1.4 Canonical answer — E-QUERY-038 `available_columns` data path

**Canonical:** the E-QUERY-038 column gate reads column names from
`resolved_spec_map`, not from `TableRegistry`.

Specifically:
1. The gate receives `resolved_spec_map` (already available as a parameter in
   `check_availability_gate` and in `check_table_availability`).
2. Given the validated table name (e.g., `crowdstrike_alerts`) and the requesting
   org(s) (`org_scope`), the gate filters `resolved_spec_map` to entries whose
   `org_slug` is in `org_scope` and whose `spec.tables` contains the target table.
3. From the matching `TableSpec`, it reads `.columns.iter().map(|c| c.name.clone())`
   to build `available_columns`.
4. The `did_you_mean` Levenshtein computation runs over those column names using
   `strsim::levenshtein` (already a direct dep of prism-query, `0.11.1`).

When `resolved_spec_map` is `None` (single-tenant / legacy mode):
- Fall back to `config_manager.load().load().sensor_specs.get(sensor_id)` to find
  the `SensorSpec.tables` entry, then read its columns. This is the same pattern
  used by `render_schema_resource` in `resources.rs` (lines 772–797), which accesses
  `config_manager.load().load().sensor_specs.get(sensor_id)` and then
  `spec.tables.iter().find(|t| t.table_name == table_name)`.
- If neither `resolved_spec_map` nor `config_manager` provides columns (test mode),
  `available_columns` is `[]`.

**Summary:** `available_columns` comes from `(resolved_spec_map OR
config_manager/ConfigSnapshot) → SensorSpec.tables → TableSpec.columns →
ColumnSpec.name`. Not from `TableRegistry`.

### 1.5 Canonical answer — `prism_describe` column read path (001-A)

`prism_describe` must return per-client table/column catalogs. The correct read path,
consistent with existing handlers (`render_client_sensors_resource`,
`render_schema_resource`), is:

**Multi-tenant path (resolved_spec_map is `Some`):**
- Read from `self.query_engine.resolved_spec_map()` (already accessible via
  `PrismServer.query_engine`).
- Filter by `org_slug == client_id`, iterate over each `ResolvedSensorSpec.spec.tables`,
  read `TableSpec.columns` → `ColumnSpec.name` + `ColumnSpec.column_type`.
- This is exactly what `render_client_sensors_resource` does for `table_names`
  (lines 674–693 in `resources.rs`), extended to also read `.columns`.

**Single-tenant / ConfigManager fallback (resolved_spec_map is `None`):**
- Read from `self.config_manager.as_ref()?.load().load().sensor_specs` keyed by
  `sensor_id`.
- This is what `render_schema_resource` does (lines 772–797 in `resources.rs`).

Neither path requires any new `Arc<dyn TableRegistry>` injection into
`prism_describe`. The data is already reachable via `PrismServer.query_engine`
(for `resolved_spec_map`) and `PrismServer.config_manager`. No new fields on
`PrismServer`.

**The read path is NOT NET-NEW.** `render_schema_resource` already reads
`config_manager.load().load().sensor_specs.get(sensor_id).tables`, and
`render_client_sensors_resource` already reads `resolved_spec_map` filtered by
`OrgSlug`. `prism_describe` composes these two existing paths. Only the aggregation
logic (iterating all tables for a client, combining into `TableDescriptor` with
`ColumnDescriptor` list) is new.

---

## 2. Corrected DI/Wiring Statement

### What TableRegistry is and how it is accessed

`TableRegistry` is a concrete `#[non_exhaustive] struct` in `prism-query`, NOT a
trait. It stores table-name strings and a sensor_id reverse map only.

Access in production: `query_engine.table_registry() -> Option<Arc<TableRegistry>>`.

`PrismServer` has no direct `table_registry` field. It accesses the registry only
through `self.query_engine.as_ref()?.table_registry()`, as seen in `server.rs`
(e.g. line 3012–3014: `qe.table_registry().map(|r| r.registered_sensor_ids())`).

Column schema is NOT in `TableRegistry`. It is in:
- `query_engine.resolved_spec_map()` — `Arc<HashMap<(OrgSlug, SensorId),
  ResolvedSensorSpec>>` — preferred, multi-tenant
- `PrismServer.config_manager` — `Arc<ArcSwap<ConfigManager>>` — fallback,
  single-tenant

### Corrected text for story narratives, Tasks, Architecture Mapping, and Compliance Rules

Replace any occurrence of:
- "`Arc<dyn TableRegistry>` injected into `PrismServer`" with "column schema
  read from `query_engine.resolved_spec_map()` (multi-tenant) or `config_manager`
  (single-tenant fallback)"
- "TableRegistry stores column schema" or "TableRegistry schema" (when referring
  to columns) with "spec-layer column schema in `resolved_spec_map →
  ResolvedSensorSpec.spec.tables → TableSpec.columns → ColumnSpec`"
- "sourced ENTIRELY from `TableRegistry`" (for `available_columns`) with "sourced
  from operator TOML specs via `resolved_spec_map → TableSpec.columns` (the same
  spec-layer data that populates TableRegistry's table-name strings)"
- The fictional single-method "`filter_to_org_visible()`" with the actual pair
  `filter_to_org_visible_sensors()` / `filter_to_org_visible_tables()` (crate-private
  helpers in `table_registry.rs`)

Note: `TableRegistry` is still relevant for 001-A's subscribe/notify (AC-006). The
hot-reload signal that triggers `notify_resource_updated` is a `TableRegistry`
change event (sensors added/removed). This is correct — `TableRegistry` is the
authority on which table NAMES exist; the notify trigger is fine as-is.

---

## 3. Per-Story Edit List

### 3.1 S-DEMO-PRISMQL-ONBOARDING-001-A edits

**Section: `depends_on` frontmatter (line 23–24)**

Reword the S-3.13 dependency note:

> Current: "S-3.13 (MERGED — provides Arc<dyn TableRegistry>; prism_describe +
> prismql://schema/{client_id} both read from the same TableRegistry instance
> injected at boot)."

> Corrected: "S-3.13 (MERGED — wires `TableRegistry` into `QueryEngine`; S-3.13's
> per-org org-scope filter helpers are the model for the per-org column-scope filter
> that `prism_describe` applies to `resolved_spec_map`)."

**Section: `risk_mitigations` bullet 3 (line 89–91)**

> Current: "prism_describe and prismql://schema/{client_id} MUST read from the same
> Arc<dyn TableRegistry> instance."

> Corrected: "prism_describe and prismql://schema/{client_id} MUST read column schema
> from the same data source: `query_engine.resolved_spec_map()` in multi-tenant mode,
> `config_manager` in single-tenant/test fallback. DI-008 client isolation enforced
> by OrgSlug filter applied to `resolved_spec_map` keys. Under no circumstances may a
> call for client 'acme' return 'globex' table or column names."

**Section: Tasks Phase 2 — L2: prism_describe tool (lines 193–219)**

Replace the pre-flight task:
> "Confirm `Arc<dyn TableRegistry>` injection point in `PrismServer` struct
> (wired by S-3.13)"

With:
> "Confirm `query_engine.resolved_spec_map()` return type (`Option<Arc<HashMap<
> ResolvedSpecKey, ResolvedSensorSpec>>>`) in `engine.rs` — this is the primary
> column-schema source for `prism_describe`. Confirm `config_manager` field on
> `PrismServer` (`server.rs`) — this is the single-tenant fallback. Confirm
> `ResolvedSensorSpec.spec.tables: Vec<TableSpec>` and `TableSpec.columns:
> Vec<ColumnSpec>` in `prism-spec-engine/src/spec_parser.rs`."

Replace the `prism_describe` handler description bullet:
> "prism_describe(client_id: String) handler receiving `Arc<dyn TableRegistry>`"

With:
> "prism_describe(client_id: String) handler reading column schema from
> `self.query_engine.resolved_spec_map()` (multi-tenant: filter by OrgSlug, walk
> `ResolvedSensorSpec.spec.tables`, collect `TableSpec.columns`) or
> `self.config_manager` (single-tenant fallback: `sensor_specs.get(sensor_id).tables`,
> same pattern as `render_schema_resource` in `resources.rs`)."

**Section: Previous Story Intelligence (lines 462–464)**

> Current: "S-3.13 (MERGED — Dynamic Table Availability / TableRegistry): `Arc<dyn
> TableRegistry>` is wired into `PrismServer` at boot. `TableRegistry::
> registered_tables()` returns `Vec<String>`. `prism_describe` uses the same `Arc<>`
> injection pattern."

> Corrected: "S-3.13 (MERGED — Dynamic Table Availability / TableRegistry):
> `TableRegistry` is a concrete `#[non_exhaustive] struct` wired into `QueryEngine`
> (NOT directly into `PrismServer`). Accessed from MCP handlers via
> `self.query_engine.as_ref()?.table_registry()`. It stores table-name strings only —
> NO column schema. `prism_describe` column data comes from
> `self.query_engine.resolved_spec_map()` (multi-tenant) or
> `self.config_manager.load().load().sensor_specs` (single-tenant fallback), following
> the same pattern used by `render_schema_resource` and
> `render_client_sensors_resource` in `resources.rs`."

**Section: Architecture Compliance Rules (lines 478–488)**

Replace the rule:
> "| `Arc<dyn TableRegistry>` injected at boot; do NOT construct new instance in
> handler | ADR-022 wiring | Adversary: grep for `TableRegistry::new()` in
> prism_describe.rs |"

With:
> "| `prism_describe` reads column schema from `query_engine.resolved_spec_map()` or
> `config_manager`; do NOT attempt `TableRegistry::new()` or `Arc<dyn TableRegistry>`
> in prism_describe.rs | ADR-022 wiring | Adversary: grep for `TableRegistry::new()`
> in prism_describe.rs; grep for `Arc<dyn TableRegistry>` injection into
> prism_describe.rs |"

Replace the rule:
> "| `prism_describe` and `prismql://schema/{client_id}` MUST read from same
> `Arc<dyn TableRegistry>` | BC-2.10.012 + BC-2.10.013 invariant | Adversary: verify
> single injection point |"

With:
> "| `prism_describe` and `prismql://schema/{client_id}` MUST read from the same
> data source (`query_engine.resolved_spec_map()` or `config_manager` fallback) so
> that `resources/read("prismql://schema/acme")` produces identical JSON to
> `prism_describe("acme")` (AC-005 parity test) | BC-2.10.012 + BC-2.10.013
> invariant | Adversary: verify single code path for column enumeration |"

**Section: Adversary grep probe — Architecture Compliance Rules**

The removal-uncertainty report R1 flagged the adversary grep probe
"grep for fictional `Arc<dyn TableRegistry>`". Change the compliance rule adversary
probe from:
> "Adversary: grep for `Arc<dyn TableRegistry>` injection into prism_describe.rs"

To guard that it does NOT exist (the current wording accidentally instructs the
adversary to verify a fictional injection exists). The corrected adversary probe is:
> "Adversary: grep for `Arc<dyn TableRegistry>` in prism_describe.rs and FAIL if
> found — correct wiring is through `query_engine.resolved_spec_map()`."

**Section: Library & Framework Requirements (lines 495–503)**

The row:
> "| prism-core | workspace | TableRegistry trait, OrgId/OrgSlug/TenantId, ColumnType |"

Change to:
> "| prism-core | workspace | OrgSlug (replaces deprecated TenantId), ColumnType
> (prism_core::column::ColumnType; variants String/Integer/Float/Boolean/Datetime/Json) |"
> "| prism-spec-engine | workspace (existing dep via prism-mcp → prism-query chain) |
> ResolvedSensorSpec, ResolvedSpecKey, TableSpec, ColumnSpec — column schema source
> for prism_describe and prismql://schema/{client_id} |"

---

### 3.2 S-DEMO-PRISMQL-ONBOARDING-001-B edits

**Section: `depends_on` frontmatter (lines 24–27)**

> Current: "S-3.13 (MERGED — provides Arc<dyn TableRegistry> wired into PrismServer;
> E-QUERY-038 reads the registry's per-org column schema for the table being queried)."

> Corrected: "S-3.13 (MERGED — wires `TableRegistry` into `QueryEngine`; establishes
> the `check_availability_gate(query_str, org_scope, resolved_spec_map)` pattern and
> org-scope filter helpers that E-QUERY-038 extends for column-level checking. The
> `resolved_spec_map` parameter already flows through `check_table_availability` and
> into `check_availability_gate` — E-QUERY-038 reads column data from this same
> parameter, not from `TableRegistry` itself)."

**Section: Tasks Phase 3 — E-QUERY-038 plan-time column gate (lines 247–259)**

Replace the column availability lookup instruction:
> "Column availability checked against `TableRegistry` for `(table, OrgId)` pair
> (same lookup pattern as E-QUERY-037 per D-1163)"

With:
> "Column availability checked via `resolved_spec_map` for `(table, OrgId)` pair.
> Specifically: filter `resolved_spec_map` to entries where `org_slug` is in
> `org_scope` (same org-scope rules as E-QUERY-037); among matching entries, find
> the `ResolvedSensorSpec` whose `spec.sensor_id + spec.tables[i].table_name` equals
> the requested table; read `spec.tables[i].columns.iter().map(|c| c.name.clone())`
> as `available_columns`. When `resolved_spec_map` is `None` (single-tenant / test
> mode), read from `ConfigSnapshot.sensor_specs.get(sensor_id)?.tables` via a helper
> that calls `config_manager` or `table_registry`'s registered name set as a fallback.
> `TableRegistry` itself does not hold column schema."

Replace:
> "`available_columns` is ALWAYS present (empty `[]` if table has zero columns);
> org-scoped per DI-008"

With:
> "`available_columns` is ALWAYS present (empty `[]` if table has zero columns or
> if `resolved_spec_map` is None and ConfigSnapshot cannot be reached); org-scoped
> per DI-008 using the same org-scope pattern as `filter_to_org_visible_sensors()`
> and `filter_to_org_visible_tables()` already in `table_registry.rs`."

**Section: risk_mitigations bullet (line 110–113)**

> Current: "available_columns in E-QUERY-038 sourced ENTIRELY from TableRegistry
> (operator TOML → registry). MUST NOT contain API keys, bearer tokens, URL paths,
> or credentials."

> Corrected: "available_columns in E-QUERY-038 sourced from operator TOML specs via
> `resolved_spec_map → ResolvedSensorSpec.spec.tables → TableSpec.columns →
> ColumnSpec.name` (the same TOML specs that populate TableRegistry's table-name
> strings). `ColumnSpec.name` is an operator-defined schema field name from the TOML
> spec (e.g., `\"severity\"`, `\"host_name\"`). MUST NOT contain API keys, bearer
> tokens, URL paths, or credentials — which it cannot, because column names are
> operator-specified strings in TOML, not API response data."

**Section: Previous Story Intelligence — S-3.13 paragraph (lines 466–478)**

The corrected paragraph is already partly right in v1.1 (remove-uncertainty applied
(3) and (4)). Extend the FLAG statement:

Replace:
> "CRITICAL ARCHITECTURE FLAG ... affects how AC-001/AC-002 are satisfied — flagged
> for architect adjudication before TDD."

With (now resolved by this document):
> "ARCHITECTURE FLAG RESOLVED (architect, 2026-06-20,
> onboarding-001-tableregistry-datapath-correction.md): E-QUERY-038 `available_columns`
> reads from `resolved_spec_map → ResolvedSensorSpec.spec.tables → TableSpec.columns`
> (NOT from `TableRegistry`). The gate implementation extends
> `check_availability_gate` in `table_registry.rs` or adds a colocated helper in
> `engine.rs`: after table presence is confirmed by E-QUERY-037, look up the
> matching `TableSpec` in `resolved_spec_map` using the validated `(org_slug,
> sensor_id)` key, then extract column names. When `resolved_spec_map` is `None`,
> return `available_columns: []` (fail-open for single-tenant mode — the gate fires
> only when resolved_spec_map is wired). AC-001 and AC-002 are satisfied entirely by
> this resolved_spec_map read path."

**Section: Architecture Compliance Rules (lines 523–533)**

Replace the rule:
> "| `available_columns` sourced ENTIRELY from TableRegistry; MUST NOT contain
> credentials | BC-2.11.016 invariant + DI-008 | AC-002 multi-tenant test |"

With:
> "| `available_columns` sourced from `resolved_spec_map → TableSpec.columns →
> ColumnSpec.name`; MUST NOT contain credentials (operator TOML column names are
> safe schema identifiers) | BC-2.11.016 invariant + DI-008 | AC-002 multi-tenant
> test |"

---

## 4. BC Implications

### Does this require a BC amendment?

**No BC amendment is required.** Here is the analysis per relevant BC:

**BC-2.11.016 (E-QUERY-038):** The BC says `available_columns` is "sourced ENTIRELY
from the `TableRegistry`" and describes the column catalog as coming from
"`TableRegistry` schema for `(table, OrgId)`". This phrasing is imprecise but
semantically correct at the BEHAVIORAL level: `available_columns` is sourced from
operator-controlled TOML specs (which is what the BC's injection-safety guarantees
require) and is org-scoped. The implementation-layer clarification — that the actual
code path is `resolved_spec_map → TableSpec.columns`, not a hypothetical
`TableRegistry.columns` field — does not change any postcondition, invariant, error
code, payload shape, or test vector in BC-2.11.016. The BC is behaviorally correct;
only the implementation description is loose. No BC amendment needed. Story-writer
SHOULD add a note to BC-2.11.016's Implementation Location section clarifying that
"TableRegistry schema" means "spec-layer column data reachable via
`resolved_spec_map`" — but this is an annotation, not a postcondition change. Route
that annotation to product-owner if desired; it is not blocking for TDD.

**BC-2.10.012 (prism_describe):** The BC does not specify the internal data path for
column data; it specifies the behavioral output (per-client table/column catalog, DI-008
isolation). The implementation path (`resolved_spec_map` or `config_manager`) is
invisible to the BC consumer. No amendment needed.

**BC-2.10.013 (prismql://schema/{client_id}):** Same as BC-2.10.012. No amendment.

**Product-owner flag:** No immediate routing to product-owner is required. If the
product-owner wishes to clarify BC-2.11.016's "sourced from TableRegistry" phrasing
to match the implementation, a non-blocking annotation addition to the Implementation
Location section is the appropriate vehicle. Blocking on that amendment before TDD is
not warranted — the postconditions and test vectors are correct.

---

## 5. Confirmation: Wiring-Not-Redesign and Independent Implementability

### This is wiring-not-redesign

The data is already reachable from existing `PrismServer` fields:

| Needed data | Already accessible via |
|-------------|----------------------|
| Column names for `prism_describe` (multi-tenant) | `self.query_engine.resolved_spec_map()` → `ResolvedSensorSpec.spec.tables[i].columns` |
| Column names for `prism_describe` (single-tenant fallback) | `self.config_manager.load().load().sensor_specs.get(sensor_id)?.tables[i].columns` |
| Column names for E-QUERY-038 gate | `resolved_spec_map` param already passed to `check_availability_gate`; `spec.tables[i].columns` reachable once table is identified |
| Org-scope filter | `filter_to_org_visible_sensors()` / `filter_to_org_visible_tables()` already in `table_registry.rs` (pub(crate)); same pattern applies for column filtering |

No new `Arc<dyn TraitName>` injection is required. No new fields on `PrismServer`. No
new cross-crate dependency. The `resolved_spec_map` parameter is already threaded
through to `check_availability_gate` — the E-QUERY-038 column gate is an extension
of that function's body, or a colocated helper function called from it.

ADR-022 "wiring not redesign" clause: adding a column-availability lookup to
`check_availability_gate` using the `resolved_spec_map` parameter it already receives
is wiring, not redesign. The existing `check_availability_gate` function signature
does not change.

### Stories remain independently implementable with one sequencing note

**001-A (prism-mcp only):** Reads column schema from `PrismServer.query_engine.
resolved_spec_map()` and `PrismServer.config_manager` — both already in `server.rs`.
No dependency on 001-B. Can be implemented first or in parallel.

**001-B (prism-core + prism-query + prism-mcp wire):** Adds the E-QUERY-038 gate
inside `check_availability_gate` (or a colocated helper in engine.rs) using the
`resolved_spec_map` parameter already present. Adds `PrismError::ColumnNotFound`
to `prism-core`. No dependency on 001-A (the MCP wire for `normalized_pql` in
`server.rs` is thin and additive).

**Cross-crate sequencing note (unchanged from original spec):** Both stories touch
`prism-mcp/src/server.rs`. 001-A touches `tools/prism_describe.rs` and
`resources/schema.rs` (new files). 001-B touches `error_mapping.rs` and the
`normalized_pql` field on the query response type. These are non-overlapping within
`server.rs`. Merge order is flexible; rebase friction is minimal.

**001-A's `prism-spec-engine` dependency:** `prism-mcp` does not currently have a
direct `prism-spec-engine` dev-dependency (the types flow in via `prism-query` at
runtime). The implementer for 001-A must check `Cargo.toml` for `prism-mcp` and
confirm whether `ResolvedSensorSpec` / `TableSpec` / `ColumnSpec` types are
accessible in test code. If not, add `prism-spec-engine` as a dev-dependency in
`crates/prism-mcp/Cargo.toml`. This is a one-line addition and is within 001-A's
scope (CLAUDE.md: in-scope wiring changes are not deferrals).

---

## Summary for Story-Writer

The defect is: both stories use "TableRegistry" as a proxy for "column schema data"
when in fact `TableRegistry` holds only table-name strings. The correct data path
for column schema is `resolved_spec_map → ResolvedSensorSpec.spec.tables →
TableSpec.columns → ColumnSpec.name`, which is already threaded into
`check_availability_gate` as a parameter.

**Specific changes story-writer must make:**

For **001-A**:
1. `depends_on` S-3.13 note — remove "Arc<dyn TableRegistry>" language
2. `risk_mitigations` bullet 3 — replace TableRegistry injection wording
3. Tasks Phase 2 pre-flight — replace "Confirm Arc<dyn TableRegistry> injection"
4. Tasks Phase 2 handler description — replace "receiving Arc<dyn TableRegistry>"
   with column-schema read-path description
5. Previous Story Intelligence S-3.13 paragraph — replace with corrected wiring facts
6. Architecture Compliance Rules — fix two TableRegistry-injection rules and the
   fictional adversary grep probe direction

For **001-B**:
1. `depends_on` S-3.13 note — clarify resolved_spec_map is the column source
2. Tasks Phase 3 column gate description — replace TableRegistry lookup with
   resolved_spec_map lookup
3. `risk_mitigations` — correct "ENTIRELY from TableRegistry" phrasing
4. Previous Story Intelligence CRITICAL FLAG paragraph — replace with "RESOLVED"
   and canonical implementation path
5. Architecture Compliance Rules — fix available_columns sourcing rule

**No BC amendment required. No scope change. Both stories remain independently
implementable. This is wiring-not-redesign.**
