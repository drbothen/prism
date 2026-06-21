---
document_type: story
story_id: S-DEMO-PRISMQL-ONBOARDING-001-A
title: "PrismQL LLM Auto-Onboarding — MCP Teaching Surface (L1 Primer + L2 Discovery + L3 Reference)"
wave: null
# Wave assignment deferred — schedules after S-5.04 merges (prism-mcp crate conflict avoidance per D-1244).
target_module: prism-mcp
subsystems: [SS-10]
# Subsystem anchor justifications:
#   SS-10 (MCP Interface) owns this sub-story's scope exclusively. All L1 (query tool description
#   upgrade + query_tutorial prompt), L2 (prism_describe tool + prismql://schema/{client_id}
#   resource template), and L3 (prismql://reference static resource) surfaces live in prism-mcp
#   per ADR-041 v1.1 Architectural Surface table and ARCH-INDEX Subsystem Registry.
priority: P0
# P0: DEMO-BLOCKING per D-1243 (S-DEMO-PRISMQL-ONBOARDING-001 set). D-1162 capability-discovery
# block REQUIRED per DEMO-SCOPE.md. Without prism_describe, Claude cannot author PQL queries
# against live per-client schemas in the multi-client SOC demo capstone.
depends_on:
  - S-5.03
  # S-5.03 (MERGED — provides the ServerHandler override pattern for resources/prompts; prism_describe
  # and prismql:// resources register using the established patterns).
  - S-3.13
  # S-3.13 (MERGED — wires `TableRegistry` into `QueryEngine`; S-3.13's per-org org-scope filter
  # helpers are the model for the per-org column-scope filter that `prism_describe` applies to
  # `resolved_spec_map`).
# Dependency anchors:
#   S-DEMO-PRISMQL-ONBOARDING-001-A depends on S-5.03 because the ServerHandler resource/prompt
#     patterns (list_resource_templates, list_resources, read_resource, PromptRouter) are
#     established by S-5.03 and this sub-story uses them directly.
#   S-DEMO-PRISMQL-ONBOARDING-001-A depends on S-3.13 because TableRegistry is wired into
#     QueryEngine by S-3.13, and S-3.13's org-scope filter helpers are the model for the per-org
#     column-scope filter that `prism_describe` applies to `resolved_spec_map`.
#   NOTE: no hard dependency on S-5.04 (Sensor Health — also prism-mcp). For smooth merge
#     sequencing (D-1244 crate-conflict avoidance), this sub-story SHOULD pipeline after S-5.04.
#     The orchestrator should merge S-5.04 first where possible to reduce rebase friction on
#     prism-mcp. This is a scheduling preference, not a functional hard dependency.
blocks: []
# S-DEMO-PRISMQL-ONBOARDING-001-B depends on this sub-story for the normalized_pql MCP
# response envelope (prism-mcp side of the OPD-1 field). That ordering is recorded in B's
# depends_on, not here.
estimated_days: 3
points: 7
# Points justification (ADR-041 L1+L2+L3 scope):
#   L1 — query tool description primer upgrade (≤500 tokens; server.rs edit): 0.5 pts
#   L1 — query_tutorial MCP Prompt (5 structural elements; prompts.rs): 1 pt
#   L2 — prism_describe tool (new file, resolved_spec_map column reads, audit event, response types,
#         3 #[non_exhaustive] types, wired via query_engine.resolved_spec_map()): 2.5 pts
#   L2 — prismql://schema/{client_id} resource template (subscribe/notify — NET-NEW
#         ServerHandler overrides + subscriber registry): 2 pts
#   L3 — prismql://reference static resource (include_str! embed + pql_reference.md content
#         with 7 required sections ≤3000 tokens): 1 pt
#   Total: 7 pts
level: "L4"
status: merged
# BC status: behavioral_contracts is non-empty (4 BCs). POL-14: all 4 BCs promoted draft→active
# on merge of PR #197 (develop@ffe9315a; D-1277 post-merge burst 2026-06-21).
# merged_sha: ffe9315a (PR #197 squash-merge to develop; 2026-06-21)
version: "1.10"
producer: story-writer
timestamp: "2026-06-21"
input-hash: "TBD"
traces_to: [D-1241, D-1243, D-1244]
cycle: "v1.0.0-greenfield"
epic_id: "E-5"
# Epic E-5 (MCP Interface). Sub-story of S-DEMO-PRISMQL-ONBOARDING-001 per D-1244 decomposition.
phase: 2
acceptance_criteria_count: 11
red_gate_tests: 15
tdd_mode: strict
behavioral_contracts:
  [BC-2.10.009, BC-2.10.012, BC-2.10.013, BC-2.10.014]
# BC array propagation (bc_array_changes_propagate_to_body_and_acs):
# BC-2.10.009 — query_tutorial MCP Prompt + L1 tool description upgrade (cited in AC-009, AC-010)
# BC-2.10.012 — prism_describe schema discovery tool (cited in AC-001, AC-002, AC-003, AC-004)
# BC-2.10.013 — prismql://schema/{client_id} resource template (cited in AC-005, AC-006, AC-011)
# BC-2.10.014 — prismql://reference static resource (cited in AC-007, AC-008)
# All 4 BCs cited in at least one AC body trace (bidirectional trace satisfied).
verification_properties: []
# VP assignments TBD — architect assigns after story decomposition.
assumption_validations: []
risk_mitigations:
  - "Subscribe/notify for prismql://schema/{client_id} is NET-NEW machinery (not an existing
     precedent as of develop@9114e028). Implementer MUST treat AC-006's subscribe/notify path
     as new construction: declare enable_resources_subscribe(), implement subscribe/unsubscribe
     ServerHandler overrides, implement per-client subscriber registry, call
     notify_resource_updated on TableRegistry change. Do NOT assume S-5.03 patterns cover this."
  - "Three new #[non_exhaustive] public types: PrismDescribeResponse, TableDescriptor,
     ColumnDescriptor. ci.yml EXPECTED must be incremented by 3 (baseline: 79 on develop@9114e028;
     target after this story: 82). scripts/check-non-exhaustive.sh EXPECTED=82. CLAUDE.md count
     updated at merge per D-1178 mechanism."
  - "prism_describe and prismql://schema/{client_id} MUST read column schema from the same data
     source: `query_engine.resolved_spec_map()` in multi-tenant mode, `config_manager` in
     single-tenant/test fallback. DI-008 client isolation enforced by OrgSlug filter applied to
     `resolved_spec_map` keys. Under no circumstances may a call for client 'acme' return 'globex'
     table or column names."
  - "prismql://reference content MUST be embedded via include_str! (build-time static). NOT loaded
     from filesystem at runtime. Content must be ≤3,000 tokens (~12KB)."
crates_touched: [prism-mcp, prism-query]
# prism-mcp: tool/resource/prompt registration (L1/L2/L3 surfaces); response envelope update
#             for normalized_pql is owned by 001-B.
# prism-query: ADR-042 added ArcSwap field + rebuild_resolved_spec_map() to engine.rs and
#              arc-swap dep to Cargo.toml (D-1267 folded ADR-042 scope into this sub-story).
anchor_bcs: [BC-2.10.009, BC-2.10.012, BC-2.10.013, BC-2.10.014]
anchor_subsystem: ["SS-10"]
parent_story: S-DEMO-PRISMQL-ONBOARDING-001
# This is a decomposed sub-story of S-DEMO-PRISMQL-ONBOARDING-001 (per D-1244).
# The parent story is marked superseded-by-sub-stories; see parent story for full context.
---

# S-DEMO-PRISMQL-ONBOARDING-001-A — PrismQL LLM Auto-Onboarding: MCP Teaching Surface (L1 + L2 + L3)

**Decomposition context (D-1244):** This sub-story covers the **prism-mcp side** of the
4-layer teaching mechanism (ADR-041 v1.1): L1 (primer + prompt), L2 (discovery tool +
schema resource), and L3 (reference resource). It is separated from
S-DEMO-PRISMQL-ONBOARDING-001-B (prism-query/prism-core L4 side) to eliminate the
prism-mcp ↔ prism-query crate conflict that the monolithic 13-pt parent story would have
caused (D-1244 §Pairwise crate overlap). This sub-story pipelines behind S-5.04 (also
prism-mcp) for lowest merge friction; S-DEMO-PRISMQL-ONBOARDING-001-B pipelines behind
PIVOT-003 (prism-query).

---

## Narrative

As a Claude Code AI agent orchestrating multi-client MSSP security investigations, I want an
always-present PrismQL primer in the `query` tool description, a per-client schema discovery
tool (`prism_describe`), a schema resource template, and a full grammar reference resource, so
that I can discover which tables and columns are available for any client and author correct
PrismQL queries without human hand-holding.

---

## Behavioral Contracts

| BC ID | Title | Key Clauses |
|-------|-------|-------------|
| BC-2.10.009 v1.5 | MCP Prompts for Common Workflows (Including PQL Query Tutorial) | query_tutorial prompt: 5 structural elements; DI-006 security reminder; L1 primer in query tool description |
| BC-2.10.012 v1.2 | `prism_describe` Schema Discovery Tool (L2) | Always-registered; readOnlyHint: true; per-client table/column catalog; audit event on every call; DI-008 client isolation; non-existent/empty client success posture |
| BC-2.10.013 v1.2 | `prismql://schema/{client_id}` Resource Template (L2) | RFC 6570 URI template; mimeType: "application/json"; server-side subscribe/listChanged; single-source-of-truth with prism_describe; per-client subscription scoping; multi-tenant hot-reload notify (EC-10-034 dual-mode: notify all per-org subscribers on resolved_spec_map rebuild; per-client scoping EC-10-029) |
| BC-2.10.014 v1.0 | `prismql://reference` Static PQL Grammar Reference Resource (L3) | 7 required section headers; ≤3,000 tokens; build-time static via include_str!; no vendor table names in examples; text/markdown MIME |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| BC-2.10.009 v1.5 | ~800 |
| BC-2.10.012 v1.2 | ~1,200 |
| BC-2.10.013 v1.2 | ~1,000 |
| BC-2.10.014 v1.0 | ~800 |
| ADR-041 v1.1 (teaching surface architecture) | ~5,000 |
| ADR-042 (reload-aware resolved_spec_map) | ~600 |
| `crates/prism-mcp/src/server.rs` (query tool description, resource/prompt registration) | ~3,500 |
| `crates/prism-mcp/src/tools/prism_describe.rs` (new) | ~2,500 |
| `crates/prism-mcp/src/resources/schema.rs` (new) | ~2,000 |
| `crates/prism-mcp/src/prompts.rs` (query_tutorial addition) | ~500 |
| `crates/prism-mcp/src/pql_reference.md` (new — 7 sections, ≤3000 tokens) | ~3,000 |
| `crates/prism-query/src/engine.rs` (ADR-042: ArcSwap field + rebuild_resolved_spec_map) | ~800 |
| Test files (14 stubs × ~50 lines each) | ~2,100 |
| Tool outputs (nextest, clippy) | ~1,000 |
| **Total estimate** | **~28,800** |

At ~200k context window: ~14.4% — within the 20-30% ceiling.

---

## Tasks

### Pre-flight: read substrate before writing anything

- [ ] Read `crates/prism-mcp/src/server.rs` — locate the `query` `#[tool]` description block
  (the `#[tool(description = "...")]` annotation on the `query` handler); confirm current
  description text; identify injection point for L1 primer
- [ ] Read `crates/prism-mcp/src/prompts.rs` — confirm 4 existing prompts; identify injection
  point for `query_tutorial` as 5th prompt
- [ ] Read `crates/prism-mcp/src/resources.rs` (or `resources/mod.rs`) — confirm ServerHandler
  override pattern from S-5.03; identify `list_resource_templates` and `list_resources` overrides
- [ ] Confirm `query_engine.resolved_spec_map()` return type (`Option<Arc<HashMap<ResolvedSpecKey,
  ResolvedSensorSpec>>>`) in `engine.rs` — this is the primary column-schema source for
  `prism_describe`. Confirm `config_manager` field on `PrismServer` (`server.rs`) — this is the
  single-tenant fallback. Confirm `ResolvedSensorSpec.spec.tables: Vec<TableSpec>` and
  `TableSpec.columns: Vec<ColumnSpec>` in `prism-spec-engine/src/spec_parser.rs`.
- [ ] Read rmcp 1.7 docs (Context7) to confirm `subscribe`/`unsubscribe` ServerHandler
  override signatures and `notify_resource_updated` call pattern before implementing

### Phase 1 — L1: query tool description + query_tutorial prompt

- [ ] Write failing test 10 (FAIL first): `test_BC_2_10_009_l1_primer_query_tool_description`
- [ ] Upgrade `query` tool description in `server.rs` (the `#[tool]` description annotation on the `query` handler):
  - Add PQL primer: DSL declaration, clause vocabulary, pipe-mode hint, 3 schema-agnostic
    skeletons (`<table>` NOT vendor names), discovery pointer to `prism_describe` +
    `prismql://reference`
  - Primer ≤500 tokens added; existing security/pagination text preserved
- [ ] Write failing test 9 (FAIL first): `test_BC_2_10_009_query_tutorial_prompt`
- [ ] Add `query_tutorial` as 5th prompt in `prompts.rs`:
  - Arguments: `client_id` (required), `goal` (optional)
  - 5 required structural elements (Step 1: prism_describe; Step 2: write PQL + prismql://reference;
    Step 3: E-QUERY retry ≤3 times reading near_text/available_columns/did_you_mean/
    valid_operators_for_type/how_to_fix; Step 4: DI-006 security reminder; Step 5: goal
    contextualization when goal arg present)
  - Static server-authored content; no sensor data interpolation
- [ ] Verify tests 9, 10 pass

### Phase 2 — L2: prism_describe tool

- [ ] Write failing tests 1, 3, 4 (FAIL first):
  `test_BC_2_10_012_prism_describe_happy_path_catalog`
  `test_BC_2_10_012_prism_describe_empty_and_unknown_client`
  `test_BC_2_10_012_prism_describe_invalid_client_id`
- [ ] Create `crates/prism-mcp/src/tools/prism_describe.rs`:
  - `prism_describe(client_id: String)` handler reading column schema from
    `self.query_engine.resolved_spec_map()` (multi-tenant: filter by OrgSlug, walk
    `ResolvedSensorSpec.spec.tables`, collect `TableSpec.columns`) or
    `self.config_manager` (single-tenant fallback: `sensor_specs.get(sensor_id).tables`,
    same pattern as `render_schema_resource` in `resources.rs`)
  - Response types: `PrismDescribeResponse { client_id, tables: Vec<TableDescriptor>, pql_hints: Vec<String> }`
    `TableDescriptor { name, sensor_type, description, columns: Vec<ColumnDescriptor>, example_query }`
    `ColumnDescriptor { name, type: prism_core::column::ColumnType, description: Option<String>, nullable: bool }`
    (canonical sensor-schema enum = `prism_core::ColumnType`, variants String/Integer/Float/Boolean/Datetime/Json;
    NOT `prism_core::types::ColumnType`/`InternalColumnType` — CLAUDE.md §Conventions ColumnType canonical naming;
    this is the type carried by `prism_spec_engine::spec_parser::ColumnSpec.column_type`. REMOVE-UNCERTAINTY E2, 2026-06-20)
  - All 3 response types carry `#[non_exhaustive]`
  - Auto-generated example queries per table: count-recent fallback always; severity-filter if
    `severity` column present; aggregate if aggregatable column present
  - Non-existent/empty org: success with `tables: []` + informative hint (not error)
  - Format validation (`client_id`): `OrgSlug::new()` / `[a-zA-Z0-9_-]{1,64}`; `E-MCP-001` on failure
    (canonical; `TenantId` is a deprecated alias removed in Wave 4 — see `pub type TenantId = OrgSlug;`
    in prism-core `tenant.rs` and the re-export in `lib.rs`; all sibling tool/resource/prompt validators
    use `OrgSlug::new()` — confirmed in `prompts.rs` and `resources.rs` `client_id` validation paths.
    REMOVE-UNCERTAINTY E1, 2026-06-20)
  - Audit event per call: `tool_name: "prism_describe"`, `client_id`, `operation: "schema_enumeration"`,
    `outcome: "success"|"error"`; if audit emission fails → call proceeds + `_meta.audit_warning: true`
  - MCP tool annotations: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`,
    `openWorldHint: false`
  - `SafetyEnvelopeBuilder` with `trust_level: "internal"`
- [ ] Register `prism_describe` in always-registered tool tier in `server.rs`
- [ ] Write failing test 5 (FAIL first): `test_BC_2_10_012_prism_describe_client_isolation_via_resolved_spec_map`
- [ ] Write failing test 2 (FAIL first): `test_BC_2_10_012_prism_describe_audit_event_emitted`
- [ ] Verify tests 1–5 pass

### Phase 3 — L2: prismql://schema/{client_id} resource template

- [ ] Write failing tests 6, 7 (FAIL first):
  `test_BC_2_10_013_schema_resource_parity_via_dispatch`
  `test_BC_2_10_013_schema_resource_subscribe_notify`
- [ ] Create `crates/prism-mcp/src/resources/schema.rs` (or extend existing resources.rs):
  - Register `prismql://schema/{client_id}` in `list_resource_templates`:
    - URI template (RFC 6570), `mimeType: "application/json"`, description with "subscribe" hint
    - Content: identical JSON to `prism_describe` response (same `TableRegistry` projection)
    - ≤5s TTL cache; invalidate on `TableRegistry::changed()` signal
  - NET-NEW subscribe/notify machinery (NOT an existing precedent in prism-mcp):
    - `enable_resources_subscribe()` on `ServerCapabilitiesBuilder` in `get_info()` (rmcp 1.7)
    - `ServerHandler::subscribe(SubscribeRequestParams, ctx)` override
    - `ServerHandler::unsubscribe(UnsubscribeRequestParams, ctx)` override
    - Per-client subscriber registry: `HashMap<OrgSlug, Vec<SubscriberHandle>>` (ArcSwap or Mutex)
    - On `TableRegistry` change for client "X": `Peer<RoleServer>::notify_resource_updated(
      ResourceUpdatedNotificationParam { uri: "prismql://schema/X", .. })` to all X-subscribers
  - DI-008: `{client_id}` scoping — "acme" read MUST NOT return "globex" tables
  - No separate audit event for resource reads (BC-2.10.013 §Audit rationale)
- [ ] Verify tests 6, 7 pass

### Phase 4 — L3: prismql://reference static resource

- [ ] Write failing tests 8a, 8b (FAIL first):
  `test_BC_2_10_014_reference_resource_sections`
  `test_BC_2_10_014_reference_resource_static_invariant`
- [ ] Create `crates/prism-mcp/src/pql_reference.md` with ALL 7 required sections:
  1. `## What is PrismQL`
  2. `## Clause Grammar (BNF)` — SELECT, FROM, WHERE, GROUP BY, ORDER BY, LIMIT, filter-mode, pipe-mode
  3. `## Operators and Types` — per-ColumnType operator table
  4. `## Datetime Arithmetic` — NOW(), INTERVAL syntax, OCSF timestamp fields
  5. `## Error Code Quick-Reference` — E-QUERY-001, -002, -003, -037, -038 with trigger + recovery
  6. `## Query Examples (5–10)` — ALL examples use `<sensor_table>` placeholder; NO hardcoded vendor names
  7. `## Self-Correction Workflow` — on E-QUERY error: read fields → consult reference → retry ≤3 times
  Content ≤3,000 tokens (~12KB); measure before commit
- [ ] Register `prismql://reference` in `list_resources` (static non-template):
  - `mimeType: "text/markdown"`, `annotations.priority: 0.8`, `annotations.audience: ["assistant"]`
  - Content embedded via `include_str!("pql_reference.md")` at build time; NOT runtime fs read
  - Description: "Full PrismQL grammar reference..."
  - NO subscribe/listChanged (static content)
- [ ] Verify tests 8a, 8b pass

### Phase 5 — CI gate and final checks

- [ ] Count new `#[non_exhaustive]` types: PrismDescribeResponse, TableDescriptor, ColumnDescriptor
  (3 types). Increment `ci.yml EXPECTED` from 79 to 82. Update `scripts/check-non-exhaustive.sh`.
  NOTE: CLAUDE.md count update lands at merge per D-1178 mechanism (not in this PR).
- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — ensure all new `event_type=` emissions
  have BC-2.16.002 catalog rows. Expected new event_type values:
  `"schema_enumeration.started"`, `"schema_enumeration.success"`, `"schema_enumeration.rejected"`
- [ ] Run `just check` — all 10 Red Gate tests pass; zero clippy warnings; fmt clean

---

## Acceptance Criteria

### AC-001 — prism_describe tool registration and annotations
(traces to BC-2.10.012 postcondition — Tool registration and annotations)

Given the MCP server is running and `tools/list` is requested,
when the response is inspected,
then `prism_describe` appears in the tool list with `readOnlyHint: true`, `idempotentHint: true`,
`openWorldHint: false`, and its description contains "Call this tool before writing a PrismQL
query to discover which tables and columns are available."

Red Gate: `test_BC_2_10_012_prism_describe_tool_annotations` (annotations: readOnlyHint/idempotentHint/openWorldHint); `test_BC_2_10_012_prism_describe_happy_path_catalog` (response shape/tables/catalog — combined with AC-002)

### AC-002 — prism_describe happy-path response shape and audit event
(traces to BC-2.10.012 postconditions — Response shape, Auto-generated example queries, pql_hints content, Audit event emission)

Given client "acme" has CrowdStrike configured with 3 tables,
when `prism_describe("acme")` is called,
then the response contains `client_id: "acme"`, a `tables` array with 3 entries (each with
non-empty `name`, `sensor_type: "crowdstrike"`, `columns` array with ≥1 entry, and `example_query`
using the real table name), and `pql_hints` is a non-empty array; an `AuditEntry` is emitted with
`tool_name: "prism_describe"`, `client_id: "acme"`, `operation: "schema_enumeration"`,
`outcome: "success"`.

Red Gate: `test_BC_2_10_012_prism_describe_audit_event_emitted`

### AC-003 — prism_describe empty, unknown, and invalid client_id handling
(traces to BC-2.10.012 postconditions — Non-existent/empty client_id handling)

Given client "acme" has zero sensor overlays (well-formed, no tables),
when `prism_describe("acme")` is called,
then response is `{client_id: "acme", tables: [], pql_hints: ["No sensor tables are available for client 'acme'."]}` with NO error;
when `prism_describe("nonexistent")` is called (valid format, not in OrgRegistry),
then response is `{client_id: "nonexistent", tables: [], pql_hints: ["Client 'nonexistent' is not registered..."]}` with NO error;
when `prism_describe("acme/../etc")` is called,
then `E-MCP-001` is returned with `original_params_valid: false`.

Red Gate: `test_BC_2_10_012_prism_describe_empty_and_unknown_client` and `test_BC_2_10_012_prism_describe_invalid_client_id`

### AC-004 — prism_describe client isolation (DI-008)
(traces to BC-2.10.012 invariant DI-008; BC-2.10.012 Canonical Test Vectors — client-isolation)

Given a multi-tenant deployment with "acme" (crowdstrike tables) and "globex" (claroty tables),
when `prism_describe("acme")` is called,
then the response contains ONLY crowdstrike table names — no claroty table names appear in ANY
field of the response (tables, pql_hints, example_query strings, column names).

Red Gate: `test_BC_2_10_012_prism_describe_client_isolation_via_resolved_spec_map`

### AC-005 — prismql://schema/{client_id} resource template registration and parity
(traces to BC-2.10.013 postconditions — Resource template registration, Resource content, Single source of truth invariant)

Given `resources/list` is queried,
when the response is inspected,
then `prismql://schema/{client_id}` appears as a URI template with `mimeType: "application/json"`
and description containing "Per-client PQL table/column/type schema catalog";
when `resources/read("prismql://schema/acme")` is called for client "acme" with CrowdStrike configured,
then the response JSON is structurally identical to `prism_describe("acme")` — same client_id,
same tables array, same pql_hints.

Red Gate: `test_BC_2_10_013_schema_resource_parity_via_dispatch`

### AC-006 — prismql://schema/{client_id} subscribe/notify per-client scoping
(traces to BC-2.10.013 v1.2 postconditions — Server-side subscribe/listChanged support; EC-10-029 per-client scoping, EC-10-030, EC-10-034 dual-mode multi-tenant hot-reload notify)

Given a client subscribes via `resources/subscribe("prismql://schema/acme")` and then a
hot-reload adds a new CrowdStrike sensor spec for "acme",
when the `TableRegistry` change event fires,
then the server sends `notifications/resources/updated` with `uri: "prismql://schema/acme"` within
1 second of the change;
when a `TableRegistry` change occurs only for "globex",
then NO notification is sent to the "acme" subscriber (per-client subscription scoping).

Red Gate: `test_BC_2_10_013_schema_resource_subscribe_notify`

### AC-007 — prismql://reference static resource registration and required sections
(traces to BC-2.10.014 postconditions — Resource registration, Resource content required sections)

Given `resources/list` is queried,
when the response is inspected,
then `prismql://reference` appears as a static URI with `mimeType: "text/markdown"` (or `text/plain`)
and `annotations.priority` set;
when `resources/read("prismql://reference")` is called,
then the content contains ALL 7 required section headers: `## What is PrismQL`,
`## Clause Grammar (BNF)`, `## Operators and Types`, `## Datetime Arithmetic`,
`## Error Code Quick-Reference`, `## Query Examples`, `## Self-Correction Workflow`;
the error quick-reference table contains rows for E-QUERY-001, E-QUERY-002, E-QUERY-003,
E-QUERY-037, and E-QUERY-038.

Red Gate: `test_BC_2_10_014_reference_resource_sections`

### AC-008 — prismql://reference content authorship invariant
(traces to BC-2.10.014 postconditions — Content authorship invariant; EC-10-035, EC-10-036)

Given `resources/read("prismql://reference")` is called,
when the content is inspected,
then: (a) no hardcoded vendor table names appear in the `## Query Examples` section (no strings
matching `crowdstrike_`, `claroty_`, `armis_`, `cyberint_` in the examples — only `<sensor_table>`
or generic placeholders); (b) content length does not exceed 3,000 tokens (~12KB);
(c) content is identical on two successive reads within the same server process (static invariant).

Red Gate: `test_BC_2_10_014_reference_resource_static_invariant`

### AC-009 — query_tutorial MCP Prompt structural elements
(traces to BC-2.10.009 v1.5 postconditions — query_tutorial prompt spec, all structural elements)

Given `prompts/list` is queried,
when the response is inspected,
then at least 5 prompts are listed including `query_tutorial`;
when `query_tutorial` is invoked with `client_id: "acme"` and no `goal` argument,
then the prompt message contains all 4 required structural elements: Step 1 (prism_describe call
instruction), Step 2 (PQL writing with prismql://reference reference), Step 3 (E-QUERY error
self-correction with named fields: near_text, available_columns, did_you_mean,
valid_operators_for_type, how_to_fix), Step 4 (DI-006 security reminder); Step 5 is absent;
when `query_tutorial` is invoked with `client_id: "acme"` and `goal: "find critical detections"`,
then the prompt message additionally contains Step 5: "Your query goal: find critical detections."

Red Gate: `test_BC_2_10_009_query_tutorial_prompt`

### AC-010 — query tool description L1 primer
(traces to BC-2.10.009 v1.5 §L1 primer spec — query tool description upgrade)

Given `tools/list` response for the `query` tool is inspected,
when the description is read,
then it contains: "PrismQL (PQL) is a custom DSL", the clause vocabulary pattern `SELECT ... FROM`,
the pipe-mode hint `|`, all three schema-agnostic skeleton queries using `<table>` placeholder,
and the discovery pointer phrase "Call `prism_describe`";
the description MUST NOT contain any hardcoded vendor table name (no substring matches for
`crowdstrike_`, `claroty_`, `armis_`, `cyberint_` within the skeleton section).

Red Gate: `test_BC_2_10_009_l1_primer_query_tool_description`

### AC-011 — Reload-aware multi-tenant schema-change notification (ADR-042 / BC-2.10.013 EC-10-034)
(traces to BC-2.10.013 v1.2 postconditions — multi-tenant hot-reload notify EC-10-034 dual-mode; ADR-042 rebuild_resolved_spec_map)

Given the QueryEngine holds a resolved_spec_map via ArcSwap (ADR-042),
when `rebuild_resolved_spec_map()` is called after a config hot-reload adds or removes a sensor
spec for client "acme",
then the ArcSwap atom is updated atomically with the new resolved spec map and the prism-mcp
subscribe/notify path receives the TableRegistry change signal and delivers
`notifications/resources/updated` with `uri: "prismql://schema/acme"` to all subscribed clients
within 1 second;
when a hot-reload occurs for "globex" only,
then subscribers to `prismql://schema/acme` receive no notification (per-client scoping EC-10-029);
when single-tenant mode is active (resolved_spec_map ArcSwap is None),
then the config_manager fallback path is used and subscribe/notify is not invoked (single-tenant
mode has no hot-reload subscriber path — EC-10-034 dual-mode gate).

Red Gate:
- `test_BC_ADR_042_single_tenant_rebuild_is_noop_returns_ok_zero` (prism-query, engine.rs mod adr_042_tests)
- `test_BC_ADR_042_inflight_snapshot_isolation_during_rebuild` (prism-query, engine.rs mod adr_042_tests)
- `test_BC_ADR_042_multitenant_notify_org_not_equal_sensor_triggers_acme_not_globex` (prism-mcp, server.rs mod adr_042_tests)
- `test_BC_ADR_042_prism_describe_reflects_post_reload_schema` (prism-mcp, server.rs mod adr_042_tests)

---

## Red Gate Test Names

| # | Test Name | AC | Crate | Behavior Asserted |
|---|-----------|----|----|-------------------|
| 1 | `test_BC_2_10_012_prism_describe_tool_annotations` | AC-001 | prism-mcp | readOnlyHint=true, idempotentHint=true, openWorldHint=false on the production tool catalog entry; description non-empty and mentions schema |
| 2 | `test_BC_2_10_012_prism_describe_happy_path_catalog` | AC-001 + AC-002 | prism-mcp | per-client table/column catalog: response shape, client_id, tables array with name/sensor_type/columns/example_query using real table name, pql_hints non-empty; is_error=false |
| 3 | `test_BC_2_10_012_prism_describe_audit_event_emitted` | AC-002 | prism-mcp | AuditEntry with schema_enumeration operation emitted on every call |
| 4 | `test_BC_2_10_012_prism_describe_empty_and_unknown_client` | AC-003 | prism-mcp | Zero-table and unknown client return success + empty tables + hint (not error) |
| 5 | `test_BC_2_10_012_prism_describe_invalid_client_id` | AC-003 | prism-mcp | Path-traversal client_id returns E-MCP-001 |
| 6 | `test_BC_2_10_012_prism_describe_client_isolation_via_resolved_spec_map` | AC-004 | prism-mcp | Multi-tenant: acme response never contains globex table/column names |
| 7 | `test_BC_2_10_013_schema_resource_parity_via_dispatch` | AC-005 | prism-mcp | resources/read("prismql://schema/acme") content is structurally identical to prism_describe("acme") |
| 8 | `test_BC_2_10_013_schema_resource_subscribe_notify` | AC-006 | prism-mcp | Subscribe + hot-reload → notifications/resources/updated for subscribed client; no notification for different client |
| 9a | `test_BC_2_10_014_reference_resource_sections` | AC-007 | prism-mcp | resources/read("prismql://reference") contains all 7 required section headers + 5 error codes in quick-reference |
| 9b | `test_BC_2_10_014_reference_resource_static_invariant` | AC-008 | prism-mcp | No vendor table names in examples section; content unchanged between reads; ≤3000 tokens |
| 10 | `test_BC_2_10_009_query_tutorial_prompt` | AC-009 | prism-mcp | query_tutorial: all 4 required elements without goal; Step 5 absent then present with goal |
| 11 | `test_BC_2_10_009_l1_primer_query_tool_description` | AC-010 | prism-mcp | query tool description contains DSL declaration, clause vocab, 3 skeletons with <table>, discovery pointer; no vendor names |
| 12 | `test_BC_ADR_042_single_tenant_rebuild_is_noop_returns_ok_zero` | AC-011 | prism-query (engine.rs mod adr_042_tests) | Single-tenant rebuild is a no-op returning Ok(0); ArcSwap atom unchanged |
| 13 | `test_BC_ADR_042_inflight_snapshot_isolation_during_rebuild` | AC-011 | prism-query (engine.rs mod adr_042_tests) | Concurrent readers hold a stable Arc snapshot during rebuild; no torn read |
| 14 | `test_BC_ADR_042_multitenant_notify_org_not_equal_sensor_triggers_acme_not_globex` | AC-011 | prism-mcp (server.rs mod adr_042_tests) | Hot-reload for "globex" does NOT notify "acme" subscriber (per-client scoping); hot-reload for "acme" DOES notify "acme" subscriber |
| 15 | `test_BC_ADR_042_prism_describe_reflects_post_reload_schema` | AC-011 | prism-mcp (server.rs mod adr_042_tests) | prism_describe response reflects newly-added column after rebuild_resolved_spec_map() completes |

---

## Architecture Mapping

| Component | Module | Crate | Pure/Effectful |
|-----------|--------|-------|----------------|
| L1 `query` tool description upgrade | SS-10 (MCP Interface) | prism-mcp (`server.rs`) | Effectful (MCP tool registration) |
| L1 `query_tutorial` MCP Prompt | SS-10 | prism-mcp (`prompts.rs`) | Effectful (MCP prompt registration) |
| L2 `prism_describe` tool handler | SS-10 | prism-mcp (`tools/prism_describe.rs`) | Effectful (tool call, audit event, reads column schema via `query_engine.resolved_spec_map()` or `config_manager`) |
| L2 `prismql://schema/{client_id}` resource template | SS-10 | prism-mcp (`resources/schema.rs`) | Effectful (MCP resource, subscribe/notify) |
| L3 `prismql://reference` static resource | SS-10 | prism-mcp (`resources/schema.rs` or `reference.rs`) | Pure (build-time static content; registration effectful) |
| ADR-042 `rebuild_resolved_spec_map()` + ArcSwap field | SS-11 (Query Engine) | prism-query (`engine.rs`, `Cargo.toml`) | Pure (ArcSwap atomic update); effectful at call boundary |

Architecture references: ADR-041 v1.1 (teaching surface), ADR-042 (reload-aware resolved_spec_map).

Subsystem anchor justification: SS-10 (prism-mcp, MCP Interface) owns the L1/L2/L3 teaching
surfaces. SS-11 (Query Engine) owns the prism-query changes mandated by ADR-042 (resolved_spec_map
ArcSwap + rebuild method), which were folded into this sub-story per D-1267. No SS-11 query-engine
L4 work (E-QUERY-038 gate, error enrichments, normalized_pql Chumsky source) — that remains
in S-DEMO-PRISMQL-ONBOARDING-001-B.

---

## Previous Story Intelligence

**S-5.03 (MERGED develop@85ac7b06 — MCP Resources and Prompts):** The `ServerHandler` override
pattern for resources is in place. `PromptRouter<PrismServer>` + `#[prompt_handler]` macro is
wired. The four existing prompts (`triage_alerts`, `investigate_host`, `client_overview`,
`cross_client_status`) register using this pattern.
**CORRECTION:** The per-resource `subscribe` + `notifications/resources/updated` path (AC-006)
is NET-NEW — NOT an existing precedent. S-5.03 shipped only the list-changed path
(`notify_resource_list_changed()`). There is no `fn subscribe` / `fn unsubscribe` `ServerHandler`
override, no `notify_resource_updated` call site, and no `enable_resources_subscribe()` in
prism-mcp as of develop@9114e028. Implementer MUST build this from scratch.

**S-3.13 (MERGED — Dynamic Table Availability / TableRegistry):** `TableRegistry` is a concrete
`#[non_exhaustive] struct` wired into `QueryEngine` (NOT directly into `PrismServer`). Accessed
from MCP handlers via `self.query_engine.as_ref()?.table_registry()`. It stores table-name strings
only — NO column schema. `prism_describe` column data comes from
`self.query_engine.resolved_spec_map()` (multi-tenant) or
`self.config_manager.load().load().sensor_specs` (single-tenant fallback), following the same
pattern used by `render_schema_resource` and `render_client_sensors_resource` in `resources.rs`.

**S-5.02 (MERGED — Tool Routing):** `ClientIdGuard` middleware validates `client_id` parameters.
`prism_describe`'s `client_id` validation uses `TenantId::new()` consistently with other tools.

**S-DEMO-PRISMQL-ONBOARDING-001-B (sibling sub-story — query/core L4):** The `normalized_pql`
MCP response envelope update (task 13 in the parent story's Tasks §Sub-burst B) is owned by the
001-B sub-story. This sub-story does NOT add `normalized_pql` to the response envelope. If
001-B merges first, 001-A must rebase and pick up any shared `server.rs` changes.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `prism_describe` is always-registered — do NOT gate behind feature flag | BC-2.10.012 precondition 1 | Adversary: grep for feature-gate wrapping `prism_describe` registration |
| `prism_describe` annotations: all four MUST be present | BC-2.10.012 postcondition | AC-001 unit test |
| `prism_describe` reads column schema from `query_engine.resolved_spec_map()` or `config_manager`; do NOT attempt `TableRegistry::new()` or `Arc<dyn TableRegistry>` in prism_describe.rs | ADR-022 wiring | Adversary: grep for `TableRegistry::new()` in prism_describe.rs; grep for `Arc<dyn TableRegistry>` in prism_describe.rs and FAIL if found — correct wiring is through `query_engine.resolved_spec_map()` |
| `prism_describe` and `prismql://schema/{client_id}` MUST read from the same data source (`query_engine.resolved_spec_map()` or `config_manager` fallback) so that `resources/read("prismql://schema/acme")` produces identical JSON to `prism_describe("acme")` (AC-005 parity test) | BC-2.10.012 + BC-2.10.013 invariant | Adversary: verify single code path for column enumeration |
| `prismql://reference` content embedded via `include_str!` — NOT `fs::read_to_string` | BC-2.10.014 postcondition | Adversary: grep for `read_to_string` in reference handler |
| Subscribe/notify capability: `enable_resources_subscribe()` MUST be declared in `get_info()` | rmcp 1.7 + BC-2.10.013 | AC-006 subscribe test |
| Per-client subscription scoping: "acme" change MUST NOT notify "globex" subscribers | DI-008 + BC-2.10.013 EC-10-030 | AC-006 isolation test |
| All 3 new public response types carry `#[non_exhaustive]` | CLAUDE.md §Conventions | ci.yml EXPECTED=82 |
| `near_text` truncated to ≤50 chars (DI-006) | BC-2.11.017 (sibling BC owned by 001-B) | Note: this rule applies in 001-B; referenced here for cross-sub-story awareness |
| Forbidden: `prism-mcp` MUST NOT depend on `prism-sensors` directly | BC-2.10.012 §Forbidden dependencies | Adversary: Cargo.toml check for prism-sensors in prism-mcp deps |
| All new `event_type=` tracing emissions require BC-2.16.002 catalog rows (SAP-1) | CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| `rebuild_resolved_spec_map()` MUST use ArcSwap atomic store — MUST NOT use Mutex or replace the entire QueryEngine instance | ADR-042 | Adversary: grep for `Mutex` in engine.rs resolved_spec_map update path; grep for `ArcSwap` to verify field present |
| Concurrent readers (prism_describe, schema resource) MUST hold a cloned `Arc` snapshot from ArcSwap load() for the duration of their read — MUST NOT re-load mid-read | ADR-042 §Invariant | AC-011 concurrency test |

---

## Library & Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| rmcp | 1.7 (workspace) | `ServerHandler` overrides; `subscribe`/`unsubscribe`; `Peer<RoleServer>::notify_resource_updated`; `enable_resources_subscribe()`; `#[tool(... annotations(...))]`; `PromptRouter` + `#[prompt_handler]` |
| serde / serde_json | 1.x (workspace) | Serialize PrismDescribeResponse, TableDescriptor, ColumnDescriptor |
| prism-core | workspace | OrgSlug (replaces deprecated TenantId), ColumnType (`prism_core::column::ColumnType`; variants String/Integer/Float/Boolean/Datetime/Json) |
| prism-spec-engine | workspace (existing dep via prism-mcp → prism-query chain) | `ResolvedSensorSpec`, `ResolvedSpecKey`, `TableSpec`, `ColumnSpec` — column schema source for `prism_describe` and `prismql://schema/{client_id}` |
| tracing | workspace | Structured event emission; new event_type values must be in BC-2.16.002 |

**Version pinning note:** rmcp = "1.7" confirmed on develop@9114e028 (`root Cargo.toml`,
resolves 1.7.0 in `Cargo.lock`). Do NOT invent version numbers — use workspace pins.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-mcp/src/server.rs` | Modify (adds `#[cfg(test)] mod adr_042_tests`) | (1) Upgrade `query` tool description (L1 primer); (2) register `prism_describe` tool in always-registered tier; (3) register `prismql://schema/{client_id}` resource template; (4) register `prismql://reference` static resource; (5) `enable_resources_subscribe()` in `get_info()`; (6) wire subscribe/notify via `TableRegistry` change event (table-name additions/removals trigger `notify_resource_updated`; column data for content is still read from `resolved_spec_map`); (7) AC-011 Red Gate tests live here in `mod adr_042_tests` (`test_BC_ADR_042_multitenant_notify_org_not_equal_sensor_triggers_acme_not_globex`, `test_BC_ADR_042_prism_describe_reflects_post_reload_schema`) |
| `crates/prism-mcp/src/tools/prism_describe.rs` | Create | `prism_describe` tool handler; PrismDescribeResponse + TableDescriptor + ColumnDescriptor types; TableRegistry reads; audit event; example query generation |
| `crates/prism-mcp/src/resources/schema.rs` | Create (or extend existing) | `prismql://schema/{client_id}` resource template handler + per-client subscriber registry + subscribe/unsubscribe overrides; `prismql://reference` static resource handler |
| `crates/prism-mcp/src/pql_reference.md` | Create | Build-time static PQL grammar reference (7 sections, ≤3000 tokens); embedded via `include_str!` |
| `crates/prism-mcp/src/prompts.rs` | Modify | Add `query_tutorial` as 5th prompt (client_id required, goal optional; 5 structural elements) |
| `ci.yml` | Modify | Increment `EXPECTED` from 79 to 82 (+3 new `#[non_exhaustive]` types) |
| `scripts/check-non-exhaustive.sh` | Modify | `EXPECTED=82` |
| `crates/prism-mcp/tests/mcp_prism_describe.rs` | Create | Integration tests for AC-001 through AC-006 (prism_describe + resource surface) |
| `crates/prism-mcp/tests/mcp_reference_prompts.rs` | Create | Integration tests for AC-007 through AC-010 (reference resource + prompts + L1 primer) |
| `crates/prism-query/src/engine.rs` | Modify (adds `#[cfg(test)] mod adr_042_tests`) | ADR-042: add ArcSwap field for resolved_spec_map; add `rebuild_resolved_spec_map()` method that atomically updates the ArcSwap atom; update `resolved_spec_map()` accessor to load from ArcSwap; AC-011 Red Gate tests live here in `mod adr_042_tests` (`test_BC_ADR_042_single_tenant_rebuild_is_noop_returns_ok_zero`, `test_BC_ADR_042_inflight_snapshot_isolation_during_rebuild`) |
| `crates/prism-query/Cargo.toml` | Modify | ADR-042: add `arc-swap` workspace dependency |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.10.012 EC-10-028 | `prism_describe("acme")` — audit emission fails | Call proceeds; `_meta.audit_warning: true` in response; DI-004 fail-open for reads |
| EC-002 | BC-2.10.012 EC-10-025 | `prism_describe("acme")` — one table has zero columns | Table returned with `columns: []`; `example_query` uses count-recent fallback |
| EC-003 | BC-2.10.012 EC-10-026 | `prism_describe("acme")` — `TableRegistry` undergoing hot-reload at call time | Returns snapshot visible at `Arc<TableRegistry>` read time via `query_engine.table_registry()`; ArcSwap ensures no partial-reload risk for the table-name set; column data read from `resolved_spec_map` which uses the same ArcSwap snapshot semantics |
| EC-004 | BC-2.10.013 EC-10-032 | `resources/read("prismql://schema/acme")` — MCP client does not support `resources/subscribe` | Server registers template unconditionally; no subscribe calls arrive; no error |
| EC-005 | BC-2.10.014 | `resources/read("prismql://reference")` during config hot-reload | Returns build-time static content unchanged (prismql://reference is build-time static via include_str! — unaffected by hot-reload; BC-2.10.014 postcondition) |
| EC-006 | BC-2.10.014 EC-10-036 | `pql_reference.md` token count exceeds 3,000 at authoring time | Trim content before commit; do not exceed ceiling |
| EC-007 | BC-2.10.013 EC-10-033 | `resources/read("prismql://schema/acme/../etc")` (invalid URI client_id) | MCP resource error: "Invalid client_id in resource URI" |

---

## Non-Exhaustive Types and CI Gate

New public types requiring `#[non_exhaustive]`:
- `PrismDescribeResponse` (new in `prism-mcp`)
- `TableDescriptor` (new in `prism-mcp`)
- `ColumnDescriptor` (new in `prism-mcp`)

`ci.yml EXPECTED` baseline: 79 (CLAUDE.md on develop@9114e028). Increment to 82 (+3).
`scripts/check-non-exhaustive.sh EXPECTED=82`. CLAUDE.md count update at merge per D-1178.

---

## Structured Event Catalog Obligation (BC-2.16.002 / PG-LP11-001)

New `event_type` values added by this story MUST have BC-2.16.002 catalog rows before PR merges:
- `event_type = "schema_enumeration.started"` (prism_describe tool call start)
- `event_type = "schema_enumeration.success"` (prism_describe success)
- `event_type = "schema_enumeration.rejected"` (E-MCP-001 format failure)

Implementer: `rg 'event_type\s*=' crates/ --type rust` before declaring done (SAP-1).

---

## Coherence Note for Orchestrator

This sub-story's `query_tutorial` Step 3 names pedagogical error fields (`near_text`,
`available_columns`, `did_you_mean`, `valid_operators_for_type`, `how_to_fix`). Those fields
are defined and tested by S-DEMO-PRISMQL-ONBOARDING-001-B. If 001-A ships before 001-B,
the `query_tutorial` prompt text will reference fields that do not yet exist in error responses
— this is acceptable (the prompt is a GUIDE for the model; the fields will exist after 001-B
merges). The ordering is flexible; the orchestrator should coordinate merge order to avoid
user-facing confusion during the window between the two merges.

BC-2.11.001 cross-reference: the orchestrator MUST route a micro-edit dispatch to the
product-owner to add E-QUERY-038 / pedagogical fields / normalized_pql cross-references to
BC-2.11.001 before either sub-story's PR is merged. This is product-owner domain; story-writer
cannot edit BC bodies.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.10 | F-PR197-RG-P3-MED-001-FIX-2026-06-21 | 2026-06-21 | story-writer | Red Gate table corrected for F-PR197-RG-P3-MED-001. (MED) Row 1: `test_BC_2_10_012_prism_describe_happy_path_catalog` behavior description corrected from "tool annotation + ..." to "per-client table/column catalog: response shape, client_id, tables array, pql_hints" — this test does NOT assert tool annotations (readOnlyHint/idempotentHint/openWorldHint). (MED) New row 1 added: `test_BC_2_10_012_prism_describe_tool_annotations` mapped to AC-001 — this is the test that actually asserts readOnlyHint=true, idempotentHint=true, openWorldHint=false on the production tool catalog. Former rows 1-14 renumbered 2-15; row labels 8a/8b preserved as 9a/9b. AC-001 inline Red Gate citation updated to cite both tests. `red_gate_tests` frontmatter: 14→15. Full re-grep: all 15 row test names verified to resolve to real function definitions in the feature worktree. |
| 1.9 | ROUND8-FIX-BURST-HEAD-REFRESH-2026-06-21 | 2026-06-21 | state-manager | Round-8 LOCAL cascade (3 passes on frozen `5a385d4f`) NOT clean; fix-burst commits d282fe7f/2d2a65e6/fae58bdb/15e43516 closed all findings (F-R8PA-HIGH-001/F-R8PC-HIGH-001 hand-rolled envelope → SafetyEnvelopeBuilder, F-R8PC-HIGH-002 paper-pass test → safety_flags assertion, F-R8PB-MED-001 overlay-only notify gate decoupled, F-R8PB-MED-002 unsubscribe correct-by-construction adjudicated by architect, F-R8PC-MED-001 POL-27 BC dates normalized, F-R8PB-LOW-001 EC-10-033 invalid-char message, OBS-R8PC-1 adjudication doc superseded note). Feature HEAD advanced `5a385d4f → 15e43516`. `just check` GREEN; non-exhaustive gate EXPECTED=82. Frozen HEAD for round-9 re-gate: `15e43516`; streak resets 0/3 (DRIFT-ORCH-PRLEVEL-PUSH-001). STATE.md D-1274. Changelog-only (HEAD refresh + round-8 summary); no story body/AC content altered. |
| 1.8 | COMPLETE-RED-GATE-CITATION-SWEEP-2026-06-21 | 2026-06-21 | story-writer | COMPLETE Red Gate citation sweep (all rows grep-verified) + BC-2.10.012 v1.2 label; closes round-6 POL-21 rows 5/6 + version-label drift. (HIGH — POL-21) Red Gate Tests table row 5: `test_BC_2_10_012_prism_describe_client_isolation` → `test_BC_2_10_012_prism_describe_client_isolation_via_resolved_spec_map` (phantom name from v1.7 partial fix). (HIGH — POL-21) Red Gate Tests table row 6: `test_BC_2_10_013_schema_resource_template_parity` → `test_BC_2_10_013_schema_resource_parity_via_dispatch` (phantom name surviving v1.7 partial sweep). (HIGH — POL-21) AC-004 inline Red Gate citation corrected to match row 5. (HIGH — POL-21) AC-005 inline Red Gate citation corrected to match row 6. (HIGH — POL-21) Tasks Phase 2 failing-test citation corrected to match row 5. (HIGH — POL-21) Tasks Phase 3 failing-test citation corrected to match row 6. (MED) BC-2.10.012 version label updated v1.1→v1.2 in §Behavioral Contracts table and §Token Budget Estimate (canonical is v1.2 per D-1263). All 14 Red Gate rows now exact-match grep-verified real tests at feature HEAD 8b14f3ab. red_gate_tests count remains 14. |
| 1.7 | PHANTOM-ANCHOR-CORRECTION-2026-06-21 | 2026-06-21 | story-writer | POL-21 phantom-anchor defect corrected. (HIGH) Red Gate Tests table rows 11-14: replaced 4 invented ADR-042 test names (`test_BC_ADR_042_rebuild_resolved_spec_map_updates_arcswap`, `test_BC_ADR_042_arcswap_atomic_on_concurrent_read`, `test_BC_ADR_042_mcp_notify_on_spec_map_rebuild`, `test_BC_ADR_042_notify_scoped_to_subscribing_client`) with 4 ACTUAL names verified by grep at feature HEAD: `test_BC_ADR_042_single_tenant_rebuild_is_noop_returns_ok_zero` + `test_BC_ADR_042_inflight_snapshot_isolation_during_rebuild` (engine.rs mod adr_042_tests) and `test_BC_ADR_042_multitenant_notify_org_not_equal_sensor_triggers_acme_not_globex` + `test_BC_ADR_042_prism_describe_reflects_post_reload_schema` (server.rs mod adr_042_tests). (HIGH) AC-011 inline Red Gate listing updated to match. (HIGH) §File Structure Requirements: removed phantom `crates/prism-query/tests/adr_042_tests.rs` (Create) and `crates/prism-mcp/tests/adr_042_mcp_tests.rs` (Create) rows — these files do not exist; ADR-042 tests live in existing `crates/prism-query/src/engine.rs` and `crates/prism-mcp/src/server.rs` as `#[cfg(test)] mod adr_042_tests`; both rows updated to "Modify (adds mod adr_042_tests)". red_gate_tests count remains 14 (4 real ADR-042 tests replace 4 invented ones — same count). |
| 1.6 | ROUND5-CASCADE-FIXES-2026-06-20 | 2026-06-20 | story-writer | Five cross-document drift fixes from round-5 cascade. (HIGH) crates_touched updated: added prism-query (ADR-042 touched crates/prism-query/src/engine.rs + Cargo.toml per D-1267); removed "does NOT touch prism-query" comment. (HIGH) §File Structure Requirements: added prism-query engine.rs (ArcSwap field + rebuild_resolved_spec_map), Cargo.toml (arc-swap dep), and two ADR-042 test files. (HIGH) BC version labels: BC-2.10.009 v1.4→v1.5 and BC-2.10.013 v1.1→v1.2 in §Behavioral Contracts table, §Token Budget Estimate, and AC trace labels; BC-2.10.013 key-clause cell expanded with multi-tenant notify scope (EC-10-034 dual-mode + EC-10-029). (HIGH) AC-011 added: reload-aware multi-tenant schema-change notification per ADR-042/BC-2.10.013 v1.2 EC-10-034; 4 ADR-042 Red Gate tests added (test_BC_ADR_042_* Test1–Test4 in prism-query and prism-mcp); ADR-042 added to Architecture Mapping, Architecture Compliance Rules, and §File Structure. (MED) EC-005 re-anchored from BC-2.10.013 EC-10-034 to BC-2.10.014 (static-content-during-reload is a BC-2.10.014 concern, not BC-2.10.013). acceptance_criteria_count: 10→11; red_gate_tests: 10→14. |
| 1.5 | STORY-HYGIENE-2026-06-20 | 2026-06-20 | story-writer | POL-32 changelog re-ordered to strict monotonic-descending (the v1.4 reorder was applied in the wrong direction); F-P1-MED-001/F-P3-PASS3-MED-001. Corrected v1.4 row description which incorrectly claimed ascending order was the target. No AC changes. No BC array changes. |
| 1.4 | STORY-HYGIENE-2026-06-20 | 2026-06-20 | story-writer | Cascade hygiene fixes (F-P3-MED-001 + F-P3-LOW-001). (MED — POL-32) Changelog reordered to monotonic-descending. (LOW — TD-VSDD-091) Three volatile line-number pins converted to symbol/behavioral anchors: (1) Tasks pre-flight "server.rs lines 1735–1744" → `#[tool]` description annotation on the `query` handler; (2) Tasks Phase 1 "server.rs (~lines 1735–1744)" → same symbol anchor; (3) frontmatter REMOVE-UNCERTAINTY comment `tenant.rs:219` + `lib.rs:9` + `prompts.rs:49` + `resources.rs:651` → symbol-based anchors (`pub type TenantId = OrgSlug;` in tenant.rs, re-export in lib.rs, `client_id` validation paths in prompts.rs and resources.rs). No AC changes. No BC array changes. |
| 1.3 | LOCAL-CASCADE-FINDINGS-2026-06-20 | 2026-06-20 | story-writer | Two LOCAL cascade findings resolved: (HIGH) story body BC version labels for BC-2.10.012 and BC-2.10.013 updated from v1.0 → v1.1 in both the §Behavioral Contracts table and §Token Budget Estimate table, reflecting the D-1263 BC v1.1 bump; (OBS) duplicate `document_type: story` frontmatter key removed (second occurrence at former line 57, keeping the canonical first occurrence). No AC changes. No BC array changes. |
| 1.2 | TABLEREGISTRY-DATAPATH-CORRECTION-2026-06-20 | 2026-06-20 | story-writer | Architect adjudication applied (onboarding-001-tableregistry-datapath-correction.md, D-1259). Wiring-not-redesign corrections for R1 (CRITICAL) and R2 (HIGH) from remove-uncertainty pass. Edits: (1) `depends_on` S-3.13 comment — removed `Arc<dyn TableRegistry>` language; (2) `risk_mitigations` bullet 3 — replaced TableRegistry injection with `resolved_spec_map`/`config_manager` data-source statement; (3) dependency anchor comment for S-3.13 corrected; (4) points justification comment corrected; (5) Tasks Phase 2 pre-flight — replaced "Confirm Arc<dyn TableRegistry> injection" with `resolved_spec_map`/`config_manager` confirmation task; (6) Tasks Phase 2 handler description — replaced "receiving Arc<dyn TableRegistry>" with column-schema read-path description via `resolved_spec_map`/`config_manager`; (7) Previous Story Intelligence S-3.13 paragraph — corrected: concrete struct in QueryEngine, no column schema in TableRegistry, column data from `resolved_spec_map`/`config_manager`; (8) Architecture Mapping row updated; (9) Architecture Compliance Rules — fixed two TableRegistry-injection rules and flipped adversary grep probe from "verify injection EXISTS" to "FAIL if found"; (10) Library & Framework Requirements — replaced `TableRegistry trait` row with correct `OrgSlug`/`ColumnType` row and added `prism-spec-engine` row. No AC-semantic changes. No BC array changes. Both BCs remain: BC-2.10.009, BC-2.10.012, BC-2.10.013, BC-2.10.014. |
| 1.1 | REMOVE-UNCERTAINTY-2026-06-20 | 2026-06-20 | research-agent | D-1110 REMOVE-UNCERTAINTY pass. Applied 2 low-risk codebase-validated corrections in Tasks Phase 2: (E1) `TenantId::new()` → `OrgSlug::new()` (TenantId is a deprecated alias removed in Wave 4; all sibling validators use OrgSlug); (E2) `ColumnDescriptor.type: ColumnType` → `prism_core::column::ColumnType` (disambiguated from the internal `types::ColumnType`/`InternalColumnType` per CLAUDE.md §Conventions). Report: `.factory/research/remove-uncertainty/S-DEMO-PRISMQL-ONBOARDING-001-A.md`. THREE items FLAGGED for specialist routing (NOT auto-edited): R1 (CRITICAL — `Arc<dyn TableRegistry>` injection model is fictional; TableRegistry is a concrete `#[non_exhaustive]` struct accessed via `query_engine.table_registry()`, PrismServer has no TableRegistry field → architect + story-writer + product-owner); R2 (HIGH — column schema data source is the spec layer `ConfigManager`/`resolved_spec_map`, not TableRegistry which holds only table-name strings; read path is NOT NET-NEW → architect + story-writer); R3 (INFO — pre-existing BC-2.11.001 micro-edit + 001-A/001-B merge sequencing → product-owner + orchestrator). rmcp 1.7 subscribe/notify API surface VALIDATED feasible (Context7): subscribe/unsubscribe ServerHandler overrides, notify_resource_updated, ResourceUpdatedNotificationParam, enable_resources_subscribe all confirmed real. |
| 1.0 | D-1244-decomposition-2026-06-19 | 2026-06-19 | story-writer | Initial sub-story decomposition — split from S-DEMO-PRISMQL-ONBOARDING-001 (13 pts) per D-1244 §Parallel Execution Plan. Covers L1+L2+L3 MCP surfaces (prism-mcp only). 4 BCs: BC-2.10.009, BC-2.10.012, BC-2.10.013, BC-2.10.014. 10 ACs + 10 Red Gate tests. 7 pts. Pipelines behind S-5.04 for crate-conflict avoidance. |
