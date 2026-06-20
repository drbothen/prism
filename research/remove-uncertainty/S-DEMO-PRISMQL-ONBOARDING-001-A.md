# REMOVE-UNCERTAINTY Pass — S-DEMO-PRISMQL-ONBOARDING-001-A

**Story:** S-DEMO-PRISMQL-ONBOARDING-001-A — PrismQL LLM Auto-Onboarding: MCP Teaching Surface (L1+L2+L3)
**Story version at pass start:** 1.0
**Pass date:** 2026-06-20
**Pass agent:** research-agent (per D-1110 REMOVE-UNCERTAINTY protocol)
**Codebase baseline:** develop (story cites develop@9114e028 / @f6739764; current local develop snapshot read directly)
**Crates touched by story:** [prism-mcp]

---

## Executive Summary

| Metric | Count |
|--------|-------|
| Total uncertainties scanned | 11 |
| Resolved by codebase (develop is source of truth) | 7 |
| Resolved by external research (Context7 / registry) | 3 |
| Inconclusive | 0 |
| Internally consistent / no change (informational) | 1 |
| **Spec edits applied (low-risk)** | 2 |
| **Items flagged for specialist routing** | 3 |

**Headline finding (U-01, CRITICAL):** The story repeatedly asserts that `Arc<dyn TableRegistry>`
is injected into `PrismServer` at boot (wired by S-3.13) and that `prism_describe` /
`prismql://schema/{client_id}` read from that trait object. **This is factually wrong against
develop.** `prism_query::table_registry::TableRegistry` is a **concrete `#[non_exhaustive] struct`,
not a trait** — there is no `dyn TableRegistry`. `PrismServer` holds NO TableRegistry field at all;
it holds `query_engine: Option<Arc<QueryEngine>>` and accesses tables via
`query_engine.table_registry() -> Option<Arc<TableRegistry>>`. Furthermore, `TableRegistry` stores
**only table-name strings** (`registered: HashSet<String>`), NOT per-column schema — so the
`prism_describe` column catalog (TableDescriptor.columns / ColumnDescriptor) CANNOT come from
`TableRegistry`. This is a load-bearing data-source error that affects ACs, the Architecture
Mapping, multiple Architecture Compliance Rules, and the points justification. It is routed to
the architect + story-writer (NOT auto-edited) because it changes the DI design and AC wording.

The genuinely external uncertainties — the rmcp 1.7 subscribe/notify API surface — all **validated
as feasible and correctly named** in the story. No invented rmcp APIs.

---

## Uncertainties, Resolutions, and Recommended Changes

### U-01 — `Arc<dyn TableRegistry>` injection model — RESOLVED BY CODEBASE — CRITICAL — FLAG FOR ROUTING

**Claim in story:** Multiple sites assert `Arc<dyn TableRegistry>` is injected into `PrismServer`
at boot by S-3.13, and that `prism_describe` + `prismql://schema/{client_id}` read from "the same
`Arc<dyn TableRegistry>` instance." Sites: frontmatter `risk_mitigations` (×2), Previous Story
Intelligence §S-3.13, Tasks Phase 2 ("`prism_describe(client_id)` handler receiving
`Arc<dyn TableRegistry>`"), Architecture Mapping row, Architecture Compliance Rules (×3 rows:
"Arc<dyn TableRegistry> injected at boot", "MUST read from same Arc<dyn TableRegistry>",
"do NOT construct new instance").

**Authoritative source (codebase):**
- `crates/prism-query/src/table_registry.rs:66-72` — `#[non_exhaustive] pub struct TableRegistry { registered: Arc<RwLock<HashSet<String>>>, sensor_by_table: Arc<RwLock<HashMap<String,String>>> }`. It is a **concrete struct**, explicitly `#[non_exhaustive]` (CR-002, S-3.13 fix-burst). There is **no `pub trait TableRegistry`** anywhere in `crates/prism-query/src`.
- `crates/prism-mcp/src/server.rs:96-131` — `pub struct PrismServer` fields are `query_engine: Option<Arc<QueryEngine>>` and `config_manager: Option<Arc<arc_swap::ArcSwap<ConfigManager>>>` (plus `prompt_router`, write/context). **No `table_registry` field, no `Arc<dyn TableRegistry>`.**
- `crates/prism-query/src/engine.rs:504` — access is `pub fn table_registry(&self) -> Option<Arc<TableRegistry>>` (concrete `Arc<TableRegistry>`, returned `Option`).
- `crates/prism-mcp/src/server.rs:1958` and existing tests construct/consume `qe.table_registry()` (concrete), never a `dyn`.

**Why this matters:** Architecture Compliance Rule "Arc<dyn TableRegistry> injected at boot; do
NOT construct new instance in handler" and the adversary grep probe "grep for `TableRegistry::new()`
in prism_describe.rs" are both predicated on a DI shape that does not exist. The correct,
codebase-true wiring is: `prism_describe` receives `&self` on `PrismServer` and reads
`self.query_engine.as_ref().and_then(|qe| qe.table_registry())` — exactly the pattern the
existing `resources.rs` resource handlers already use (see U-02).

**Recommended spec change (DO NOT auto-apply — architect + story-writer):** Replace every
`Arc<dyn TableRegistry>` with the concrete access path. Reword to: "`prism_describe` reads the
live `Arc<TableRegistry>` via `self.query_engine.table_registry()` (concrete struct, S-3.13);
it MUST NOT call `TableRegistry::new()`. Per-column schema is read from the spec layer
(`ConfigManager` / `resolved_spec_map`), not from `TableRegistry`." Update: risk_mitigations
×2, Previous Story Intelligence §S-3.13, Tasks Phase 2, Architecture Mapping, 3 Architecture
Compliance Rules. This touches AC-002/AC-004 wording indirectly (data source) → product-owner
should confirm AC text. **Routed, not edited.**

---

### U-02 — Column schema data source for `prism_describe` — RESOLVED BY CODEBASE — HIGH — FLAG FOR ROUTING

**Claim in story:** `prism_describe` returns `TableDescriptor { columns: Vec<ColumnDescriptor> }`
with `ColumnDescriptor { name, type: ColumnType, description, nullable }`, and Tasks Phase 2 +
AC-002 imply the table/column catalog comes from `TableRegistry`. The schema resource template
(AC-005) is said to be "the same `TableRegistry` projection."

**Authoritative source (codebase):**
- `TableRegistry` stores ONLY table-name strings (`HashSet<String>`) and a table→sensor_id map
  (`table_registry.rs:67-72`). It exposes `registered_tables() -> Vec<String>` and
  `registered_sensor_ids() -> Vec<String>` (lines 237, 397). **No column data exists in `TableRegistry`.**
- Per-table column schema lives in `prism-spec-engine`:
  `spec_parser.rs:291` `pub struct TableSpec { table_name: String, columns: Vec<ColumnSpec>, .. }`
  and `spec_parser.rs:199` `pub struct ColumnSpec { name, column_type: ColumnType, .. }`.
- The existing `prism://schema/{sensor_id}/{table_name}` resource (`resources.rs:752-802`,
  `render_schema_resource`) already sources the table schema from the `ConfigManager` snapshot
  (`snapshot.sensor_specs.get(sensor_id)` → `spec.tables` → serialize `table_spec`). This is the
  established, codebase-true pattern for column-level schema.
- Per-org column isolation (DI-008) is achievable today via `resolved_spec_map`
  (`prism_spec_engine::ResolvedSpecKey = (OrgSlug, SensorId)`), exactly as
  `render_client_sensors_resource` (`resources.rs:633-734`) already filters by `OrgSlug`.

**Why this matters:** AC-002 ("columns array with ≥1 entry"), AC-004 (column-name isolation),
and AC-005 (parity) cannot be satisfied from `TableRegistry` alone. The implementer needs the
`ConfigManager`/`resolved_spec_map` path for columns. `PrismServer` already holds `config_manager`
and the `QueryEngine` exposes `org_registry()` / `resolved_spec_map()` — so the data is reachable
without new boot wiring, but the story's stated data source is wrong.

**Recommended spec change (DO NOT auto-apply — architect + story-writer):** Clarify the data
source split: table **names** from `query_engine.table_registry()`; table **columns + types**
from the spec layer (`ConfigManager` snapshot `sensor_specs` for single-tenant, `resolved_spec_map`
keyed by `(OrgSlug, SensorId)` for multi-tenant). Cite `render_schema_resource` /
`render_client_sensors_resource` as the existing precedent (contradicts the story's "NET-NEW"
framing for the read path; only subscribe/notify is genuinely NET-NEW). **Routed, not edited.**

---

### U-03 — `client_id` validation: `TenantId::new()` vs `OrgSlug::new()` — RESOLVED BY CODEBASE — LOW — EDIT APPLIED

**Claim in story:** Tasks Phase 2 + Architecture Compliance imply `prism_describe` validates
`client_id` via `TenantId::new()` (Previous Story Intelligence §S-5.02 also says "uses
`TenantId::new()` consistently with other tools"). AC-003 references `[a-zA-Z0-9_-]{1,64}`.

**Authoritative source (codebase):**
- `crates/prism-core/src/tenant.rs:219` — `pub type TenantId = OrgSlug;` — `TenantId` is a
  **deprecated alias** for `OrgSlug`.
- `crates/prism-core/src/lib.rs:9` — `tenant::TenantId — deprecated alias for OrgSlug; removed in Wave 4`.
- Actual production validation in prism-mcp uses `OrgSlug::new()` everywhere:
  `prompts.rs:49` (`validate_client_id`), `resources.rs:651` (`render_client_sensors_resource`),
  and `server.rs` `validate_client_ids` (E-MCP-001 emitter). The `OrgSlug` regex is
  `^[a-zA-Z0-9_-]{1,64}$` (`tenant.rs:25`) — matches the story's stated pattern exactly.

**Resolution:** Functionally equivalent (alias), but `TenantId` is deprecated and slated for
removal in Wave 4; new code should use the canonical `OrgSlug::new()` to match all sibling tool /
resource / prompt validators and avoid introducing a fresh use of a to-be-removed alias.

**Spec edit APPLIED:** In Tasks Phase 2, changed
`Format validation (client_id): TenantId::new() / [a-zA-Z0-9_-]{1,64}`
→ `Format validation (client_id): OrgSlug::new() / [a-zA-Z0-9_-]{1,64} (canonical; TenantId is a deprecated alias removed in Wave 4 — see prism-core tenant.rs:219, lib.rs:9)`.
Low-risk: pure mechanical correction of a confirmed deprecated symbol to its canonical
replacement; does not change the AC behavior (same regex, same E-MCP-001 outcome). Inline source
citation added.

---

### U-04 — rmcp version pin (`rmcp = "1.7"`) — RESOLVED BY CODEBASE — LOW — NO CHANGE

**Claim in story:** Library & Framework Requirements table: `rmcp 1.7 (workspace)`; version-pinning
note "rmcp = "1.7" confirmed on develop@9114e028 (root Cargo.toml, resolves 1.7.0 in Cargo.lock)."

**Authoritative source (codebase):** `Cargo.toml:74` (workspace.dependencies):
`rmcp = { version = "1.7", features = ["server", "macros", "transport-io"] }`. `crates/prism-mcp/Cargo.toml:29`:
`rmcp = { workspace = true }`. Comment confirms "rmcp 1.7: Official Rust MCP SDK." Pin is accurate.

**Resolution:** Confirmed accurate. No change. (Did NOT independently re-resolve Cargo.lock to a
patch version — the story claims 1.7.0; the workspace pin `"1.7"` is a caret range that admits
1.7.x. The minor pin `1.7` is the authoritative requirement and is correct.)

---

### U-05 — rmcp `ServerHandler::subscribe` / `unsubscribe` override signatures — RESOLVED BY RESEARCH (Context7) — VALIDATED

**Claim in story:** AC-006 / Tasks Phase 3 require implementing
`ServerHandler::subscribe(SubscribeRequestParams, ctx)` and
`ServerHandler::unsubscribe(UnsubscribeRequestParams, ctx)` overrides (described as NET-NEW).

**Authoritative source (research):** Context7 `/websites/rs_rmcp` — `ServerHandler` trait
definition (docs.rs/rmcp/latest, trait.ServerHandler.html) lists provided (default) methods:
```rust
fn subscribe(&self, request: SubscribeRequestParams, context: RequestContext<RoleServer>)
    -> impl Future<Output = Result<(), McpError>> + ...
fn unsubscribe(&self, request: UnsubscribeRequestParams, context: RequestContext<RoleServer>)
    -> impl Future<Output = Result<(), McpError>> + ...
```
`SubscribeRequestParams` / `UnsubscribeRequestParams` exist with `{ meta: Option<Meta>, uri: String }`
and `::new(uri)` constructors (rmcp `model.rs`). Both are `#[non_exhaustive]`.

**Resolution:** Story is CORRECT — these are real rmcp default methods that can be overridden.
Type names in the story (`SubscribeRequestParams`, `UnsubscribeRequestParams`) match rmcp exactly.
The "NET-NEW" framing is accurate for the subscribe/notify path (prism-mcp currently overrides
only `list_resources` / `list_resource_templates` / `read_resource`, confirmed at
`server.rs:5354-5384`; no `subscribe`/`unsubscribe` override exists). No change.

---

### U-06 — rmcp `Peer<RoleServer>::notify_resource_updated` + `ResourceUpdatedNotificationParam` — RESOLVED BY RESEARCH (Context7) — VALIDATED

**Claim in story:** Tasks Phase 3 / risk_mitigations require
`Peer<RoleServer>::notify_resource_updated(ResourceUpdatedNotificationParam { uri, .. })`.

**Authoritative source (research):** Context7 `/websites/rs_rmcp` — docs.rs/rmcp/latest
`type.ClientSink.html`: `pub async fn notify_resource_updated(&self, params: ResourceUpdatedNotificationParam) -> Result<(), ServiceError>`
"specific to `Peer<RoleServer>`." `ResourceUpdatedNotificationParam` is the param type.

**Cross-check (codebase):** prism-mcp already uses sibling notification methods on
`Peer<RoleServer>`: `resources.rs:1072` `peer.notify_resource_list_changed()` and `:1076`
`peer.notify_tool_list_changed()` — confirming the `Peer<RoleServer>` notify-method family is
live and wired. `notify_resource_updated` is the per-resource analog.

**Resolution:** Story is CORRECT — `notify_resource_updated(ResourceUpdatedNotificationParam)`
is a real rmcp 1.7 method on `Peer<RoleServer>`. No invented API. No change.

---

### U-07 — `ServerCapabilitiesBuilder::enable_resources_subscribe()` — RESOLVED BY RESEARCH (Context7) + CODEBASE — VALIDATED

**Claim in story:** Tasks Phase 3 / Architecture Compliance: declare `enable_resources_subscribe()`
on `ServerCapabilitiesBuilder` in `get_info()`.

**Authoritative source (research):** Context7 `/websites/rs_rmcp` — docs.rs/rmcp/latest
`model/capabilities.rs`:
```rust
pub fn enable_resources_subscribe(mut self) -> Self {
    if let Some(c) = self.resources.as_mut() { c.subscribe = Some(true); }
    self
}
```
**Cross-check (codebase):** `server.rs:5340-5344` already calls
`ServerCapabilities::builder().enable_tools().enable_prompts().enable_resources().build()`.
Adding `.enable_resources_subscribe()` to this chain is the exact pattern. Note: in rmcp,
`enable_resources_subscribe()` only sets the flag if `enable_resources()` was already called
(it mutates the existing resources capability) — so the call ORDER matters: it must come AFTER
`.enable_resources()`. Implementer note worth surfacing but not a spec defect.

**Resolution:** Story is CORRECT — real builder method. No change. (Minor implementer guidance:
chain `.enable_resources_subscribe()` after `.enable_resources()`.)

---

### U-08 — `#[resource_handler]` macro non-existence — RESOLVED BY CODEBASE — LOW — NO CHANGE (already correct)

**Context:** The story uses `list_resources` / `list_resource_templates` / `read_resource`
`ServerHandler` overrides (not a `#[resource_handler]` macro). This is correct.

**Authoritative source (codebase):** `resources.rs:11` header comment: "There is NO
`#[resource_handler]` macro in rmcp 1.7 — confirmed against rmcp source." `server.rs:5349-5352`
restates this and overrides the three methods directly. The story's approach matches. No change.

---

### U-09 — `PromptRouter` + `#[prompt_handler]` pattern for `query_tutorial` — RESOLVED BY CODEBASE — LOW — NO CHANGE

**Claim in story:** Add `query_tutorial` as 5th prompt in `prompts.rs` using the established
`PromptRouter<PrismServer>` + `#[prompt_handler]` pattern.

**Authoritative source (codebase):** `prompts.rs:33-37` imports
`rmcp::handler::server::router::prompt::{PromptRoute, PromptRouter}`; `build_prompt_router()`
(`prompts.rs:134-235`) registers 4 prompts via `PromptRouter::new().with_route(PromptRoute::new_dyn(...))`.
`server.rs:5325` `#[prompt_handler(router = self.prompt_router)]`. The pattern is exactly as the
story describes; adding a 5th `PromptRoute` is mechanical. The 4 existing prompt names
(`triage_alerts`, `investigate_host`, `client_overview`, `cross_client_status`) match the story's
Previous Story Intelligence. No change.

**Note for implementer (not a defect):** `PromptArgument::new(...).with_description(...).with_required(bool)`
is the existing arg-builder pattern (`prompts.rs:141-143`); `query_tutorial`'s `client_id`
(required) + `goal` (optional) follow it directly.

---

### U-10 — `ColumnType` canonical identity for `ColumnDescriptor.type` — RESOLVED BY CODEBASE — LOW — EDIT APPLIED

**Claim in story:** `ColumnDescriptor { name, type: ColumnType, .. }` (unqualified `ColumnType`);
Library & Framework Requirements lists `prism-core ... ColumnType`.

**Authoritative source (codebase):** Two `ColumnType` enums exist (CLAUDE.md §Conventions):
- `prism_core::column::ColumnType` (variants String/Integer/Float/Boolean/Datetime/Json) — the
  **canonical sensor schema API** (`column.rs:19`, re-exported `prism_core::ColumnType` at `lib.rs:101`).
  This is the type carried by `prism_spec_engine::spec_parser::ColumnSpec.column_type` (`spec_parser.rs:203`).
- `prism_core::types::ColumnType` (Text/Int64/UInt64/...) — internal table schemas only,
  re-exported as `InternalColumnType` (`lib.rs:150`). MUST NOT be used here.

Since `prism_describe` columns derive from `ColumnSpec.column_type` (per U-02), the correct type
is `prism_core::column::ColumnType` (= `prism_core::ColumnType`).

**Spec edit APPLIED:** In Tasks Phase 2 response-types block, changed
`ColumnDescriptor { name, type: ColumnType, description: Option<String>, nullable: bool }`
→ `ColumnDescriptor { name, type: prism_core::column::ColumnType (canonical sensor-schema enum, = prism_core::ColumnType; NOT prism_core::types::ColumnType/InternalColumnType — CLAUDE.md §Conventions), description: Option<String>, nullable: bool }`.
Low-risk: disambiguates between two same-named enums to the one CLAUDE.md mandates and that the
source data (`ColumnSpec.column_type`) actually carries. No behavior change.

---

### U-11 — E-QUERY codes (001/002/003/037/038) and E-MCP-001 referenced by L3 reference / query_tutorial — RESOLVED BY CODEBASE — LOW — NO CHANGE (informational)

**Claim in story:** AC-007 requires the L3 reference error quick-reference to contain rows for
E-QUERY-001, -002, -003, -037, -038; `prism_describe` emits E-MCP-001 on invalid client_id;
query_tutorial Step 3 names pedagogical fields (near_text, available_columns, did_you_mean,
valid_operators_for_type, how_to_fix).

**Authoritative source (codebase):**
- E-QUERY-001/002/003/037 all defined in `.factory/specs/prd-supplements/error-taxonomy.md`
  (lines 228-230, 255). E-QUERY-038 defined at line 257 but explicitly marked **NEW variant
  (ADR-041), prism-core error.rs** — i.e., it does NOT yet exist in code on develop.
- E-MCP-001 + `original_params_valid: false` are live in prism-mcp (`server.rs:1388-1413`,
  `error_mapping.rs:86`). The story's E-MCP-001 / `original_params_valid: false` usage is correct.
- E-QUERY-037's taxonomy row already cites ADR-041 L4 + `prism_describe('<client_id>')` suggestion;
  E-QUERY-038 cites BC-2.11.016. Both pedagogical-field families are owned by sub-story 001-B.

**Resolution:** Consistent. The story's own Coherence Note already flags that E-QUERY-038 +
pedagogical fields are owned by 001-B and may not exist when 001-A ships (acceptable — the
reference/prompt are pedagogical GUIDES). This is internally consistent and needs no change.
**Informational only.** (One adjacent item surfaced below as a routing flag for cross-story
sequencing, not a defect in this story.)

---

## Spec Edits Applied (low-risk, codebase-validated)

| # | Location | Change | Source | Risk |
|---|----------|--------|--------|------|
| E1 | Tasks Phase 2 — client_id validation bullet | `TenantId::new()` → `OrgSlug::new()` (+ deprecation note) | tenant.rs:219, lib.rs:9; prompts.rs:49 / resources.rs:651 use OrgSlug | LOW — alias→canonical, same regex/behavior |
| E2 | Tasks Phase 2 — ColumnDescriptor type | `type: ColumnType` → `type: prism_core::column::ColumnType` (+ disambiguation) | column.rs:19, lib.rs:101/150; CLAUDE.md §Conventions; spec_parser.rs:203 | LOW — disambiguation of two same-named enums |

Both edits add inline source citations and do NOT touch AC bodies, BC references, points, or scope.

---

## Items Flagged for Specialist Routing (NOT auto-edited)

| # | Uncertainty | Severity | Route to | Reason not auto-edited |
|---|-------------|----------|----------|------------------------|
| R1 | U-01 — `Arc<dyn TableRegistry>` injection model is fictional; `TableRegistry` is a concrete `#[non_exhaustive]` struct accessed via `query_engine.table_registry()`; `PrismServer` has no TableRegistry field | CRITICAL | architect (DI design) → story-writer (reword risk_mitigations, Tasks, Architecture Mapping, 3 Compliance Rules, Previous Story Intelligence) → product-owner (confirm AC-002/004 data-source wording) | Changes DI design + AC wording + adversary probe text; cross-component reasoning; out of research-agent edit scope |
| R2 | U-02 — column schema data source is the spec layer (`ConfigManager`/`resolved_spec_map`), NOT `TableRegistry` (which holds only table-name strings); read path is NOT "NET-NEW" (precedent: `render_schema_resource`/`render_client_sensors_resource`) | HIGH | architect → story-writer | Affects AC-002/004/005 data-source semantics and the "single TableRegistry projection" parity claim in AC-005; needs product-owner confirmation of AC text |
| R3 | U-11 adjacent — story's Coherence Note already mandates a product-owner micro-edit to BC-2.11.001 (E-QUERY-038 / pedagogical fields / normalized_pql cross-refs) before either sub-story merges; E-QUERY-038 is NEW-not-yet-in-code on develop | INFO/PROCESS | product-owner (BC body edit) + orchestrator (merge sequencing 001-A vs 001-B) | BC-body edit is product-owner domain (story-writer/research cannot edit BC bodies); this is a pre-existing note, surfaced to ensure it is not lost |

**Routing recommendation for orchestrator:** R1 + R2 are the same root cause (the DI/data-source
model) and should be dispatched together as one architect-then-story-writer micro-cycle BEFORE
TDD delivery. The fix is "wiring/reword, not redesign" — the data is already reachable through
`PrismServer`'s existing `query_engine` + `config_manager` fields, so no new boot wiring is needed;
the story narrative simply describes the wrong access shape. R3 is a pre-existing note already in
the story; surfaced here so it is tracked.

---

## What Did NOT Change (validated-correct, high confidence)

- rmcp version pin `1.7` (workspace) — correct (U-04).
- rmcp `ServerHandler::subscribe`/`unsubscribe` override approach + param type names — correct (U-05).
- rmcp `Peer<RoleServer>::notify_resource_updated(ResourceUpdatedNotificationParam)` — real API (U-06).
- rmcp `enable_resources_subscribe()` builder method — real API (U-07); chain after `enable_resources()`.
- No `#[resource_handler]` macro; direct ServerHandler overrides — correct (U-08).
- `PromptRouter` + `#[prompt_handler]` 5th-prompt pattern — correct (U-09).
- E-MCP-001 / `original_params_valid` usage — matches live prism-mcp code (U-11).
- `include_str!` build-time static for `pql_reference.md` — standard Rust, no uncertainty.
- The subscribe/notify machinery genuinely being NET-NEW (no existing `subscribe` override) — correct.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 0 | Not needed — every genuinely external uncertainty (rmcp 1.7 API surface) was answered authoritatively by Context7 against docs.rs/rmcp/latest, and every other uncertainty resolved against the develop codebase (the designated source of truth). Per agent mandate, Context7 is the preferred first call for library-API-specific questions; this pass was overwhelmingly codebase + library-doc bound, not multi-source-synthesis bound. |
| Perplexity perplexity_reason | 0 | n/a |
| Perplexity perplexity_search | 0 | n/a |
| Perplexity perplexity_ask | 0 | n/a |
| Context7 resolve-library-id | 1 | Resolved `rmcp` → `/websites/rs_rmcp` (official Rust MCP SDK docs, 18.5k snippets, High reputation) |
| Context7 query-docs | 3 | (1) subscribe/unsubscribe params + enable_resources_subscribe; (2) notify_resource_updated + ResourceUpdatedNotificationParam; (3) full ServerHandler trait method signatures |
| Tavily (any) | 0 | n/a — Context7 was authoritative for the rmcp API; no cross-validation gap |
| WebFetch / WebSearch | 0 | n/a |
| Codebase reads (Read/Grep) | 16 | Story spec; prism-mcp Cargo.toml + root Cargo.toml; resources.rs; prompts.rs; server.rs (capabilities, query tool, ServerHandler impl, PrismServer struct); table_registry.rs; engine.rs; tenant.rs; column.rs; types.rs; spec_parser.rs; error-taxonomy.md |
| Training data | 1 area | General Rust/`include_str!` semantics (uncontroversial) — flagged explicitly; not load-bearing |

**Total MCP tool calls:** 4 (1 Context7 resolve + 3 Context7 query-docs)
**Training data reliance:** low — every load-bearing claim is sourced either to a cited file:symbol
in the develop codebase or to a cited Context7/docs.rs rmcp doc page. The single training-data area
(`include_str!` build-time embedding) is uncontroversial standard-library behavior and not a point
of uncertainty in the story.

**MCP-availability note:** Context7 was available and used. `perplexity_research` was deliberately
not invoked because the pass had zero genuinely-external multi-source-synthesis questions: the rmcp
API questions are library-doc lookups (Context7's exact specialty), and everything else is
"what does prism-mcp actually do on develop" (codebase reads). This deviation from the
"perplexity_research-first" default is justified per the agent mandate's Context7-before-Perplexity
rule for narrow library-API questions.
