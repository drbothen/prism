---
document_type: adr
adr_id: "ADR-041"
title: "PrismQL LLM Auto-Onboarding — 4-Layer Teaching Surface for Automatic Agent Query Authoring"
status: proposed
date: "2026-06-19"
modified: "2026-06-23"
version: "1.2"
producer: architect
subsystems_affected: [SS-10, SS-11]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-5.04]
related_adrs: [ADR-005, ADR-022, ADR-039]
related_bcs: [BC-2.10.008, BC-2.10.009, BC-2.10.011]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-041: PrismQL LLM Auto-Onboarding — 4-Layer Teaching Surface for Automatic Agent Query Authoring

## Status

PROPOSED v1.2 (2026-06-23). Amended to allocate E-QUERY-039 (enrichment-UDF-not-found
gate; SR-004 resolution); closes AUDIT-005; covers EnrichStage pipe-mode and
FuncCall::Scalar{ScalarFunc::Unknown} SQL-mode SELECT projection path. See §Changelog.

v1.1 (2026-06-19). Amended to adopt OPD-1 (echo-normalized-PQL-back,
Pattern B) in v1 per human product decision 2026-06-19 (project lead). See §Changelog.

v1.0 authored by architect to define the 4-layer teaching mechanism that enables an
LLM agent (Claude in Claude Code, stdio MCP) to author valid PrismQL queries against a
per-tenant live schema without human hand-holding. Feeds the product-owner story for
S-5.04 and the broader capability-discovery block (D-1162 REQUIRED per DEMO-SCOPE.md).

Research basis: `.factory/research/llm-query-dsl-onboarding-2026-06-19.md`
(research-agent, 3 Perplexity deep-research calls + 2 WebFetch verifications against
MCP 2025-06-18 spec).

---

## Context

### The Teaching Problem

PrismQL (PQL) is a custom DSL that is not part of any LLM's training corpus. Unlike
SQL (pretrained-known by Claude and every other modern model), PQL requires the model
to be explicitly taught: its surface syntax, its clause vocabulary, and — critically —
which tables and columns exist for a specific MSSP client. Without a teaching channel,
the model will hallucinate SQL, guess table names from sensor brand names, and produce
queries that fail at parse time, plan time, or both.

The prior art is clear: every production DB MCP server (SQLite, ClickHouse, Postgres
MCP Pro, Snowflake Cortex) relies on pretrained SQL fluency and adds only schema
introspection (`list_tables` / `describe-table`). None embed a formal grammar or
few-shot examples in tool metadata. Prism cannot copy this lean posture — it must
supply explicit grammar teaching that SQL servers skip. (Research finding Q1, Q2.)

### The Multi-Tenant Grounding Problem

Prism is a multi-tenant process: different MSSP clients have genuinely different sensor
combinations (CrowdStrike + Claroty for client A; Armis + Cyberint for client B). A
teaching surface that names tables from sensors a given client lacks produces hallucinated
column references that fail at plan time. The `TableRegistry` (S-3.13, ADR-039,
`E-QUERY-037`) is the authoritative per-tenant table availability oracle at plan time.
The teaching surface must be grounded in the same live per-client data, not a static doc.

### The Self-Correction Problem

Grammar-constrained / structured decoding (PICARD, GBNF, Outlines, XGrammar/vLLM)
requires token-logit access. Hosted models (Claude in Claude Code) do not expose
logit-level hooks; this technique is infeasible for Prism's runtime model. The
research-confirmed alternative is **post-hoc enforcement**: Chumsky parse → structured
`E-QUERY` reject → model reads the pedagogical error → retries. Execution-error
self-correction is the best-evidenced accuracy lever available to hosted models:
DIN-SQL lifted Spider SOTA +5.4pt; MAC-SQL lifted BIRD +13.24pt; Self-Debugging lifted
hardest queries +9pt. The existing `E-QUERY-NNN` infrastructure is the correct
foundation — it needs pedagogical enrichment, not a new mechanism. (Research §1e, §1f.)

### Existing Surfaces This ADR Composes

- `query` tool: `crates/prism-mcp/src/server.rs:1735-1744` — current description is
  security/pagination focused; no PQL grammar primer, no discovery pointer.
- `explain_query` tool: `crates/prism-mcp/src/server.rs:1870-1882` — plan-time gate
  that returns `available_tables` in `E-QUERY-037` errors (ADR-039 org-scoped filter).
- `list_capabilities` meta-tool (BC-2.10.011): always-registered; returns tri-state
  capability matrix for a client; note: capabilities != schema/tables; this is
  complementary, not the discovery surface.
- `prism://config/clients` and `prism://config/clients/{client_id}/sensors` resources
  (BC-2.10.008, S-5.03 MERGED develop@85ac7b06): client + sensor inventory. These are
  the configuration-inventory surface — they show which sensors are provisioned, not
  which PQL tables are available. They are composable with capability-discovery but are
  not the discovery surface itself.
- MCP Prompts (BC-2.10.009, S-5.03): four static workflow prompts (`triage_alerts`,
  `investigate_host`, `client_overview`, `cross_client_status`). These encode
  analyst-workflow scaffolding; they do NOT encode the PQL-authoring workflow.
- `E-QUERY-037` plan-time table-availability gate (S-3.13 MERGED): rejects queries
  referencing tables unavailable for the requesting org; already returns
  `available_sensors` / `available_tables` in the error payload. This is the L4
  self-correction backstop.
- D-1162 capability-discovery block: REQUIRED per DEMO-SCOPE.md §Capability-Discovery
  Block. S-5.02 (MERGED), S-5.03 (MERGED), S-5.04 (not-started), S-3.13 (MERGED).
  This ADR defines the architecture that S-5.04 (Sensor Health Subsystem) and the
  PQL-teaching story must implement.

### The D-1162 Reconciliation

D-1162 scoped the "capability-discovery block" as S-5.02 + S-5.03 + S-5.04 + S-3.13.
This ADR clarifies what "capability discovery" means in the context of LLM onboarding:
it is the **schema-teaching surface** — the mechanism by which the model learns which
tables/columns/types exist for a specific client. This is distinct from the
`list_capabilities` tool (BC-2.10.011), which answers "what write operations can this
client perform." Both are needed; neither substitutes for the other. The capability-
discovery surface defined in this ADR is the **prism_describe** tool and the
`prismql://schema/{client_id}` resource template.

---

## Decision

We adopt a **4-layer teaching mechanism** for automatic LLM onboarding to PrismQL:

### L1 — Always-Present PQL Primer (tool description + MCP Prompt)

**Location:** The `prism-query` tool's `description` field (always-present per-turn)
AND a new `query_tutorial` MCP Prompt.

**Dual-home rationale:** The tool `description` is present on every turn — the model
reads it before generating any tool call. An MCP Prompt (`query_tutorial`) lets the
host (or the model) invoke a richer procedure skeleton that encodes the discovery-first
workflow. Both serve the same primer text; the Prompt extends it with the multi-step
invocation workflow. This does NOT duplicate content: the tool description carries the
inline grammar skeleton; the Prompt carries the workflow ordering ("step 1: call
`prism_describe`; step 2: write PQL; step 3: on E-QUERY error, read the error's
`available_tables` and retry").

**Token budget:** Target 300–500 tokens for the tool description addition. This is
above the SQL-server norm (which adds zero grammar) but justified because PQL is
custom and unrecognized. The research confirms a concise primer is net-positive for
custom DSLs; full grammar inlining would exceed token budget and degrade reasoning.

**Primer content:**
1. One-sentence "what PrismQL is" (federated OCSF query over sensor APIs).
2. Clause vocabulary: `SELECT ... FROM <sensor_table> [WHERE ...] [GROUP BY ...] [ORDER BY ...] [LIMIT N]`.
3. Pipe-mode hint: `| filter`, `| enrich`, `| limit` suffixes (reference only).
4. 3 schema-agnostic intent↔query skeletons — NOT using real sensor table names:
   - Count recent events: `SELECT COUNT(*) FROM <table> WHERE timestamp > NOW() - INTERVAL '1h'`
   - Filter by field: `SELECT * FROM <table> WHERE severity IN ('high', 'critical') LIMIT 50`
   - Aggregate with group: `SELECT source_ip, COUNT(*) FROM <table> GROUP BY source_ip ORDER BY COUNT(*) DESC LIMIT 10`
5. Explicit instructions: "call `prism_describe` with your `client_id` first to get available tables and columns; full grammar is at resource `prismql://reference`."

The skeletons are schema-agnostic to prevent hallucination on clients that don't have
any specific sensor. They show PQL shape without anchoring to a real table.

### L2 — Capability/Schema Discovery: `prism_describe` Tool + `prismql://schema/{client_id}` Resource Template

**Architecture decision: BOTH tool AND resource, for different access patterns.**

A new `prism_describe` MCP tool is the **primary discovery path**:

```
prism_describe(client_id: String) -> {
  client_id: String,
  tables: [{
    name: String,
    sensor_type: String,
    description: String,
    columns: [{ name: String, type: ColumnType, description: String, nullable: bool }],
    example_query: String,   // grounded to this client's real table name
  }],
  pql_hints: [String],       // 1-3 client-specific tips (e.g., "this client has no Claroty tables")
}
```

**Why a tool (model-decides-when) rather than host-injection only:** For a security
product, every schema-enumeration call is an auditable action. A model-initiated tool
call generates an audit event (prism-audit pipeline) with `(client_id, timestamp,
caller: model)`. A host-side injection that silently prepends schema to every prompt
produces no audit trace. Prism's threat model (multi-tenant, MSSP, every data access
auditable) requires the tool path. (Security rationale mirrors ADR-005: the audit
trail is the security requirement, not a nice-to-have.)

The tool is **always-registered** (not gated by feature flags), annotated
`readOnlyHint: true`, `idempotentHint: true`, `openWorldHint: false`.

A `prismql://schema/{client_id}` **resource template** (RFC 6570 URI) is ALSO
registered, containing identical content. The resource serves two additional purposes:
(a) hosts that prefetch schema and inject it as context can use the resource path; (b)
the resource participates in MCP's `listChanged`/`subscribe` notification mechanism,
enabling clients that support it to receive proactive schema-freshness signals when a
sensor comes online or offline (a `TableRegistry` change event).

**`listChanged`/`subscribe` client support:** Whether Claude Code's MCP client honors
resource `subscribe`/`listChanged` as of the current build is **not confirmed** from
public docs (research finding Q6: inconclusive). The server MUST implement the
subscribe/notify side (MCP 2025-06-18 spec). Whether the specific client acts on it
is an implementation-time verification task. The server-side implementation is required
regardless; it will be exercised when clients that support it are encountered.

**Content — grounded examples generation:** The `prism_describe` response generates
one `example_query` per table from the live `TableRegistry` using the actual table name
for that client. This is a **hybrid approach**: the server generates examples
programmatically from the registry (not curated by humans), using a small set of
canonical query templates (count, recent-N, field-filter) instantiated with real
column names. This gives live grounding (preventing hallucination of columns the client
lacks) without requiring per-client curation. The research justification: live-schema
grounding is strongly motivated (all text-to-SQL sources insist on it) though the
marginal gain vs. static examples is not separately measured. Production-grade default
is live grounding because the multi-tenant risk (hallucinating tables a client lacks)
outweighs the complexity cost.

**Single source of truth:** `prism_describe` and `prismql://schema/{client_id}` are
both computed from the same `TableRegistry` projection used by the `E-QUERY-037`
plan-time gate. There is exactly one authoritative per-client schema source.

### L3 — Full Grammar Reference Resource (`prismql://reference`)

A non-parameterized MCP resource registered as `prismql://reference` provides the
complete PQL grammar and reference on demand.

**Content:**
- BNF-style PQL grammar (all clauses, operators, functions, pipe verbs).
- Operator semantics (OCSF field paths, datetime arithmetic, pattern matching).
- Error code quick-reference: which `E-QUERY-NNN` codes are caller-recoverable and how.
- 5–10 fuller query examples (multi-clause, pipe-mode, GROUP BY with HAVING).

**Access pattern:** The model fetches this resource when: (a) the L1 primer's grammar
skeleton is insufficient for the query it wants to write, (b) it receives an E-QUERY
parse error and wants to cross-check its syntax, or (c) the host injects it proactively
(e.g., at session start for a new client). The resource is NOT always-present (that
would add ~1,500 tokens per turn). It is registered with MCP `annotations.priority`
set to indicate model-relevant content, per MCP 2025-06-18 spec.

**Content authorship:** Server-authored only (never sourced from sensor data or user
input), consistent with the injection-safety boundary described in L4.

### L4 — Pedagogical `E-QUERY-NNN` Self-Correction Loop

Every parse-time and plan-time PQL error that a model can recover from MUST be
**actionable**: it tells the model exactly what was wrong AND what to try instead.

**Error-shape contract:** All pedagogical `E-QUERY-NNN` errors MUST include:
- `error_code`: the `E-QUERY-NNN` string (already present).
- `message`: human/model-readable diagnosis (already present for most codes).
- `available_[noun]`: a list of valid alternatives for the rejected value.
- `did_you_mean`: optional closest-match string (edit-distance, best-effort).

**Codes getting the full pedagogical treatment (new or upgraded):**

| E-QUERY code | Current state | Pedagogical upgrade |
|---|---|---|
| E-QUERY-001 (parse error) | message only | Add `near_text` (the offending token) + pointer to `prismql://reference` |
| E-QUERY-037 (table unavailable) | `available_sensors`+`available_tables` present (ADR-039) | Already pedagogical; add `did_you_mean` (closest table name by edit distance) |
| E-QUERY-002 (type error) | field + actual_type + operator | Add `valid_operators_for_type: [...]` list |
| E-QUERY-003 (security limits) | limit_detail string | Add `how_to_fix` field (e.g., "narrow WHERE clause to reduce depth") |
| NEW: column-not-found | not yet allocated | New code `E-QUERY-038`: "column '{col}' not found in table '{table}'; available: [...]". Triggered at plan time when a column reference is not in the `TableRegistry` schema for that table/client. |
| NEW: enrichment-UDF-not-found | not yet allocated | New code `E-QUERY-039`: "enrichment infusion '{infusion}' is not registered; available: {available_infusions}, did_you_mean: {did_you_mean}". Triggered at plan time when an `EnrichStage.infusion` token (pipe mode) OR a `FuncCall::Scalar { func: ScalarFunc::Unknown(name) }` in a SQL SELECT projection does NOT match any key in `InfusionRegistry.udf_to_infusion`. Closes AUDIT-005 (E-INT-001 on unregistered enrichment function call). Note: `available_infusions` reflects the process-global `InfusionRegistry`; per-org scoping is a future follow-up. |

**E-QUERY-038 allocation:** This is a new code required by this ADR. The error-taxonomy
owner (product-owner) must register `E-QUERY-038` in `prd-supplements/error-taxonomy.md`
as part of the story delivery. The payload shape is:
```
E-QUERY-038: column '{column}' not found in table '{table}' for client '{client_id}';
  available_columns: [...],
  did_you_mean: Option<String>
```

**Retry loop ownership:** The retry loop (read error → adjust query → retry) is the
**agent's responsibility**, not the server's. Prism's job is error quality and
actionability. Prism does NOT implement a server-side retry loop. This is the correct
separation: the server provides pedagogical signal; the LLM agent (Claude) decides
how many retries to attempt. The research confirms this pattern (DIN-SQL, MAC-SQL,
Self-Debugging all run the retry loop client-side with the model as the agent).

**Retry cap guidance (not server-enforced):** The `query_tutorial` MCP Prompt
(L1) includes a recommendation: "attempt up to 3 retries on E-QUERY errors before
reporting the failure to the user." This is a model-facing guideline, not a server
enforcement. The server does not count retries or enforce a cap — it responds to each
call independently.

### Cross-Cutting: Injection-Safety Boundary

The 4-layer teaching channel is **server-authored and trusted**:
- L1 primer, L3 reference, and all error messages originate from Prism server code —
  never from sensor data, user free-text, or external untrusted content.
- L2 discovery output (table names, column names, example queries) originates from the
  live `TableRegistry`, which is populated from operator-controlled TOML specs
  (validated at load time per ADR-005 pattern). Sensor API response data NEVER flows
  into the teaching channel.
- PQL strings generated by the model are ALWAYS parse-gated by Chumsky
  (`crates/prism-query`) before plan execution. There is no path from model-generated
  text to backend sensor API calls that bypasses the parser. This is the post-hoc
  parse-gate equivalent of grammar-constrained decoding (confirmed infeasible for
  hosted models per research §1f).
- OCSF-structured data return: sensor API response bodies are normalized at the adapter
  boundary (prism-sensors OCSF normalization). Raw API bodies never reach the model.
  This is itself an injection defense: structured, server-shaped results limit
  adversarial content surface.

### Echo-Normalized-PQL-Back (Pattern B — Cortex-Analyst): ADOPTED IN V1

**Human product decision, 2026-06-19, project lead. This overrides the architect's
prior default of "defer to v2."**

The Snowflake Cortex Analyst pattern — returning the server-validated and
canonicalized/normalized PQL string in the successful `query` tool response — is
**adopted in v1**. The model accrues grounded exemplars over a session: each successful
query response shows the model exactly what normalized PQL the server accepted, which it
can use as a template for subsequent queries in the same session (L4 composition benefit).

#### What Is Echoed

The `normalized_pql` field contains the **validated + canonicalized PQL query string as
emitted by the Chumsky parser/normalizer** — the query in the form the planner accepted
after parsing, normalization, and whitespace/alias canonicalization. This is the string
the server would replay verbatim to reproduce the identical plan.

**Injection-safety reasoning:** The `normalized_pql` value is server-authored output of
the Prism Chumsky parser and normalizer pipeline. It is a normalized re-emission of the
model's own syntactically-valid PQL input, produced entirely by trusted server code. It
never reflects raw model input verbatim (it reflects the normalized form the server
computed from that input). The injection-safety boundary established in "Cross-Cutting:
Injection-Safety Boundary" above holds: the normalized string is server-produced, never
sourced from sensor data, user free-text, or external untrusted content.

#### What Is Excluded

The following are **not** included in `normalized_pql` or any field of the echo surface:

- **DataFusion physical plan internals** — execution plan node trees, physical operator
  choices, sort/hash strategy decisions.
- **Cost estimates** — optimizer cost annotations, row count estimates, selectivity
  guesses.
- **Alias-expansion internals** — how PQL column aliases were resolved internally to
  DataFusion projection expressions.
- **Join-order decisions** — which join side was chosen as probe vs. build.
- **Partition/pushdown details** — filter pushdown decisions, partition pruning choices.

These details leak optimizer internals to the model surface, add noise without teaching
value (the model cannot act on join-order to improve its PQL), and would create a
surface-coupling to DataFusion internals that complicates future query engine changes.

#### Response Envelope Placement

`normalized_pql` is an **optional, additive field** on the successful `query` tool
JSON response. It is present only when the query succeeds (parse + plan + execute
all pass). Error responses do not carry it.

```
// Successful query response — existing fields unchanged; new field additive:
{
  "results": [...],          // existing
  "row_count": N,            // existing
  "execution_time_ms": N,    // existing
  // ... other existing fields ...
  "normalized_pql": "SELECT host_name, COUNT(*) FROM crowdstrike_detections WHERE ..."
                             // NEW: optional, present on success only
}
```

The field name `normalized_pql` is the adopted name. The product-owner may choose an
alternate name (`planned_query`, `echoed_query`) if BC authoring motivates it — this
ADR defers the exact wire name to the BC author.

#### Token Cost

Adding `normalized_pql` costs approximately +50–200 tokens per successful query
response (proportional to query complexity). This overhead is **accepted per the human
product decision**. At current model pricing and expected query rates, the cost is
negligible. It is a bounded, per-response overhead rather than always-present context
(it does not add to non-query tool calls).

#### Composition with L4

A model that receives `normalized_pql` in a successful query response has a grounded
exemplar it can use as a template for subsequent queries in the same session. This
composes with L4 (pedagogical error loop): if the model bases its next query on the
echoed normalized form, the probability of parse-time rejection falls (it already knows
this shape is valid for this server). This is the in-session self-teaching benefit the
Cortex Analyst pattern delivers. The benefit is strongest in sessions with multiple
queries against the same schema (multi-step analyst workflows — the core Prism demo
scenario).

**OPD-1 STATUS: RESOLVED — Adopted v1 (human decision, 2026-06-19).**

---

## Rationale

### Why 4 layers, not fewer?

**L1 alone (primer only):** Fails on column hallucination. Without live discovery, the
model invents column names for sensors it hasn't introspected. The research (Q4) shows
every credible text-to-SQL system grounds generation in the actual current schema.
Primer-only is the "prototype path that doesn't scale."

**L2 alone (discovery only, no primer):** The model needs to know PQL *exists* and has
a syntax before it knows to call `prism_describe`. Without an always-present signal
that PQL is a custom DSL and that a discovery tool is available, the model defaults to
writing SQL (pretrained fluency). The primer primes the pump.

**L3 alone (reference resource only):** A 1,500-token resource that isn't fetched until
the model already has a query in mind is too late for first-contact orientation. The
primer and discovery must come first; the reference fills in depth when needed.

**L4 alone (errors only):** Self-correction without a primer means the model corrects
from zero baseline — multiple unnecessary round-trips. The prior art (DIN-SQL) gets its
gains from self-correction *combined with* decomposed prompting, not self-correction
alone.

The 4 layers are additive: L1 orients, L2 grounds, L3 deepens, L4 corrects. Each layer
has a different access pattern (always-present, model-called, on-demand, error-triggered)
and a different cost profile. This matches the "concise always-present + on-demand depth"
recommendation from the research synthesis (§3).

### Why a tool AND a resource for L2?

The audit requirement (security product, MSSP, every schema enumeration auditable) makes
the tool mandatory. The resource is additive: it enables host-driven injection and MCP
subscribe/listChanged — features the tool path does not support. Two surface points for
the same underlying data source, both computed from the same `TableRegistry`, is
minimal duplication relative to the capability gap it fills.

### Why hybrid (auto-generated from registry) rather than curated examples?

Curated examples per sensor type are static. When a sensor's schema changes (new
columns in a TOML spec), curated examples go stale. Auto-generated examples are always
consistent with the live registry. The cost is that auto-generated examples may be less
idiomatic than human-curated ones. Production-grade default is live grounding: in a
multi-tenant security product, a stale example that names a column the client's TOML
spec removed is a worse failure mode than a slightly mechanical example.

### Why is the retry loop the agent's responsibility?

The server does not know how many retries a given LLM session has budget for. The model
knows its context window, its session constraints, and the user's tolerance for latency.
Putting retry logic in the server would be an anti-pattern (the server would need to
store per-session state for a stateless MCP protocol). This also matches the research
consensus: every effective self-correction system (MAC-SQL, DIN-SQL, Self-Debugging)
runs the retry loop at the agent level.

### Why is grammar-constrained decoding explicitly ruled out?

It requires token-logit access to the model's decoding loop. Hosted APIs (Anthropic
Claude API, Claude Code) expose no such interface. Research (§1f) confirms this is
"only possible for models run locally; not external APIs." Stating this explicitly
prevents future implementers from re-investigating it and wasting effort. The feasible
equivalent is the server-side parse gate, which Prism already has.

---

## Consequences

### Positive

- An LLM agent calling `prism_describe` before writing a PQL query has the ground truth
  of which tables/columns exist for the requesting client — hallucinated identifiers are
  eliminated at the grounding step, not just corrected after a failed query.
- The `E-QUERY-038` column-not-found error with `available_columns` + `did_you_mean`
  gives the model everything it needs to self-correct a column reference in one retry,
  matching the ~5–13pt accuracy gains reported by the self-correction literature.
- The L1 primer in the `query` tool description guarantees that even a model that never
  calls `prism_describe` has a minimal PQL orientation on every turn.
- The `prismql://reference` resource allows hosts to inject full grammar for complex
  query sessions without burning context on simple ones.
- The injection-safety boundary is explicit and traceable: every teaching byte originates
  from server-authored code or operator TOML, never from sensor API bodies.
- The `listChanged`/`subscribe` server implementation positions Prism for clients that
  support it, providing proactive schema-freshness signals without polling.

### Negative / Trade-offs

- `prism_describe` adds one tool to the tool registry (currently 28 read + 24 gated).
  The research notes initial tool count should stay ≤~20 for high call accuracy; Prism
  is already above this. Mitigation: `prism_describe` is highly relevant only when the
  model wants to write a query — its description should make the "when to call" intent
  explicit to minimize spurious calls.
- Auto-generated example queries from the registry are more mechanical than
  human-curated examples. The quality gap is unknown until eval. Curation can be added
  as a v2 enhancement if evals show it matters.
- E-QUERY-038 (column-not-found) requires new error-taxonomy registration and a new
  `PrismError` variant in `crates/prism-query`. This is bounded scope but does touch
  the error taxonomy, which has already undergone significant reconciliation
  (ADR-035, ADR-038).
- The primer in the `query` tool description (~300–500 tokens) is always-present token
  cost. At 20 query tool calls per session, this is 6,000–10,000 tokens of permanent
  context overhead. At current model pricing this is negligible, but it is not zero.

### Status as of v1.2 (2026-06-23)

PROPOSED. No implementation exists. The `prism_describe` tool, `prismql://schema/` and
`prismql://reference` resources, `query_tutorial` MCP Prompt, E-QUERY-038, and
E-QUERY-039 error codes are all new. The L1 primer is an upgrade to an existing tool
description. The L4 pedagogical upgrades to E-QUERY-037 (already pedagogical per
ADR-039), E-QUERY-001, E-QUERY-002, E-QUERY-003 are incremental additions to existing
error codes. The `normalized_pql` echo field on successful `query` responses is adopted
(v1.1 amendment, OPD-1 resolved). E-QUERY-039 (v1.2 amendment, SR-004 resolution)
allocates the infusion-not-registered gate covering EnrichStage + ScalarFunc::Unknown
paths; closes AUDIT-005. Delivery scope is defined in the "Scope Statement for
Product-Owner Story" section below.

---

## Alternatives Considered

- **Option A — Schema-in-tool-description only (no discovery tool):** Inline the full
  PQL grammar and all table/column information in the `query` tool description.
  Rejected because: (a) the schema is per-tenant dynamic — a static inline doc would
  be immediately stale for multi-client deployments; (b) the token cost of inlining
  schema for every turn grows with the tenant's sensor set; (c) the research explicitly
  identifies this as "fine for prototypes but not scaling" (Select Star).

- **Option B — Host-injection only (resource, no tool):** Expose schema as a resource
  only; the host prefetches and injects schema. No model-callable `prism_describe` tool.
  Rejected because: (a) schema enumeration produces no audit event in the host-injection
  path — violates Prism's audit-trail requirement for an MSSP security product; (b) MCP
  resource subscription client support is currently unconfirmed; relying solely on it
  is fragile at this time.

- **Option C — No teaching surface (rely on error correction only):** Use only L4
  (pedagogical errors). The model writes a PQL guess; the server corrects it.
  Rejected because: (a) zero-baseline self-correction requires multiple round-trips for
  even basic orientation; (b) the model has no prior information that PQL is a custom
  DSL — it will write SQL; (c) the research shows self-correction gains +5–13pt
  *combined with* decomposed prompting, not in isolation.

- **Option D — Echo-normalized-PQL-back (Pattern B, scoped adoption):** Return the
  server-validated and canonicalized/normalized PQL string in every successful query
  response (Cortex Analyst pattern). **ADOPTED IN V1** (human product decision,
  2026-06-19) with the following scoping: the `normalized_pql` field carries the
  Chumsky-normalized PQL string only — NOT DataFusion physical plan internals, cost
  estimates, alias-expansion details, join-order, or partition/pushdown choices. The
  prior concerns (token inflation, plan-internal leakage) are addressed by the scoping:
  (a) +50–200 tokens per successful response is accepted as negligible; (b) optimizer
  internals are explicitly excluded, so there is no plan-internal leakage; (c) the
  in-session accrual benefit is real for multi-query analyst sessions (the core demo
  scenario) even without strong session-continuity guarantees — a model that sees its
  normalized PQL echoed has an in-context exemplar it can template from. OPD-1 resolved.

- **Option E — Grammar-constrained decoding via local model:** Pre-validate model
  PQL output using a locally-run grammar-constrained model before sending to Prism.
  Rejected as out of scope: Prism is a stdio MCP server consumed by Claude in Claude
  Code. There is no local model in the runtime architecture. Grammar-constrained
  decoding on hosted APIs is technically infeasible (no logit access).

---

## Architectural Surface

### Crate / Module Ownership

| Layer | Component | Owning Crate | Owning Subsystem |
|---|---|---|---|
| L1 primer | `query` tool description | `prism-mcp` (`server.rs:1735`) | SS-10 |
| L1 workflow prompt | `query_tutorial` MCP Prompt | `prism-mcp` | SS-10 |
| L2 discovery tool | `prism_describe` tool | `prism-mcp` + `prism-query` (schema projection) | SS-10 + SS-11 |
| L2 schema resource | `prismql://schema/{client_id}` resource template | `prism-mcp` | SS-10 |
| L3 reference resource | `prismql://reference` static resource | `prism-mcp` | SS-10 |
| L4 pedagogical errors | `E-QUERY-001/002/003/037/038/039` enrichment | `prism-query` (error.rs) | SS-11 |
| L4 column-not-found gate | New plan-time column gate | `prism-query` (engine.rs) | SS-11 |
| Echo — `normalized_pql` field | Chumsky-normalized PQL string in successful query response (optional, additive) | `prism-mcp` (response envelope) + `prism-query` (normalized query string source) | SS-10 + SS-11 |

### Interface Relationships

**`prism_describe` → `TableRegistry`:** The tool reads the live `TableRegistry`
(the same registry used by `E-QUERY-037`, ADR-039) to enumerate tables and columns
for a given `OrgId`. The `TableRegistry` is an in-process `Arc<dyn TableRegistry>`
wired at boot (ADR-022); `prism_describe` receives it via the same `Arc<>` injection
pattern as all other tools. No new state; new projection on existing state.

**`prismql://schema/{client_id}` → `TableRegistry`:** Same data, different MCP
primitive. The resource handler computes from `TableRegistry` and caches the result
(short TTL or invalidated on `TableRegistry::changed()` signal) for efficient repeated
reads.

**`query_tutorial` Prompt → L1 text + `prism_describe` workflow:** The prompt messages
reference the L1 primer content and encode the multi-step invocation workflow
(discover → write → correct). It is a static MCP Prompt (build-time defined, consistent
with BC-2.10.009 §Postconditions which mandates prompts are static). The workflow text
is prose, not PQL — it is not subject to the injection-scan pipeline.

**`E-QUERY-038` → `TableRegistry` column catalog:** The column-not-found plan-time gate
reads column availability from `TableRegistry` at plan time (same point where
`E-QUERY-037` reads table availability). This means both errors share the same
plan-time gate execution point in `prism-query/src/engine.rs` (or equivalent plan
validation step). They should be colocated to share the single `TableRegistry` read.

### Relationship to D-1162 Capability-Discovery

D-1162 (DEMO-SCOPE.md §Capability-Discovery Block) defines the delivery scope that
culminates in the multi-client SOC demo capstone. This ADR's `prism_describe` tool IS
the capability-discovery surface for D-1162. Specifically:

- The demo requires Claude to author PQL queries against multiple clients' sensor sets.
- Without `prism_describe`, Claude either hallucinates tables or requires human
  hand-holding to know what to query.
- `prism_describe` provides the grounded, per-client "what can I query" answer that
  D-1162 calls "capability discovery."
- The existing `list_capabilities` tool (BC-2.10.011) answers "what can I DO" (write
  operations); `prism_describe` answers "what can I QUERY." Both are required for the
  demo capstone; neither substitutes for the other.

---

## Open Product Decisions

**OPD-1 — RESOLVED (v1.1, 2026-06-19).** Adopted in v1 per human product decision,
project lead, 2026-06-19. Echo the validated/normalized PQL string (`normalized_pql`)
in successful `query` tool responses. Echo surface is scoped to the Chumsky-normalized
PQL string only; optimizer internals excluded. See "Echo-Normalized-PQL-Back" section
in the Decision block above.

*No open product decisions remain for this ADR.*

---

## Scope Statement for Product-Owner Story

The following is the architect's recommended scope for the product-owner story (S-5.04
and/or a dedicated ADR-041-teaching story). The product-owner authors the BCs and ACs
independently; this is an informational hand-off only.

**What must be built:**

1. **L1 Primer:** Upgrade the `query` tool description in `crates/prism-mcp/src/server.rs`
   to include the PQL primer (clause vocabulary, 3 schema-agnostic skeletons, discovery
   pointer). Token budget ≤500 tokens added to description. No new crate dependencies.

2. **`query_tutorial` MCP Prompt:** New static MCP Prompt registered in `prism-mcp`
   alongside the four existing prompts (BC-2.10.009). Content: step-by-step workflow
   ("call `prism_describe`, read schema, write PQL, retry on E-QUERY error, consult
   `prismql://reference` for grammar"). Arguments: `client_id` (required), `goal`
   (optional free-text). Must include DI-006 security reminder (consistent with
   BC-2.10.009 invariant).

3. **`prism_describe` tool:** New always-registered tool in `prism-mcp`. Input:
   `client_id: String`. Output: per-client table/column/type catalog + auto-generated
   example queries. Backed by `TableRegistry` read. Audit event emitted per call.
   `readOnlyHint: true`. Non-exhaustive response type per CLAUDE.md conventions.

4. **`prismql://schema/{client_id}` resource template:** New MCP resource template.
   Same content as `prism_describe` response, serialized as `application/json`.
   Server-side `subscribe`/`listChanged` support (per MCP 2025-06-18 spec). Client
   support verification is an implementation-time task.

5. **`prismql://reference` resource:** New static MCP resource. Content: full PQL BNF,
   operator reference, error-code quick-reference, 5–10 extended examples. Server-
   authored, build-time static. `application/text` or `application/markdown` MIME.

6. **E-QUERY pedagogical upgrades:**
   - E-QUERY-001: add `near_text` field + `prismql://reference` pointer.
   - E-QUERY-037: add `did_you_mean` (closest table name by Levenshtein distance).
   - E-QUERY-002: add `valid_operators_for_type` field.
   - E-QUERY-003: add `how_to_fix` field.
   - **E-QUERY-038 (new):** register in error-taxonomy; implement column-not-found gate
     in `prism-query`; payload: `column, table, client_id, available_columns,
     did_you_mean`.
   - **E-QUERY-039 (new):** register in error-taxonomy; implement an
     infusion-not-registered gate in `prism-query` covering BOTH `EnrichStage`
     (pipe-mode) AND `FuncCall::Scalar { ScalarFunc::Unknown }` (SQL-mode SELECT
     projection); payload: `infusion, available_infusions, did_you_mean`; closes
     AUDIT-005. NOTE: `InfusionRegistry` is process-global (not per-org) in the current
     architecture, so `available_infusions` reflects the global registry — per-org
     scoping is a future follow-up (CWE-200 risk only when per-org infusion specs are
     introduced). Bound to BC-2.11.019 + InfusionRegistry (BC-2.19.001).

7. **Non-exhaustive types:** All new public response types (`PrismDescribeResponse`,
   `TableDescriptor`, `ColumnDescriptor`, etc.) must carry `#[non_exhaustive]`.
   Update `ci.yml EXPECTED` count accordingly.

8. **Echo-normalized-PQL (`normalized_pql` field):** Add an optional `normalized_pql`
   field to the successful `query` tool response envelope in `crates/prism-mcp`.
   The field is populated from the validated + canonicalized PQL string produced by
   the Chumsky parser/normalizer in `crates/prism-query` — this is the query in the
   normalized form the planner accepted, not raw model input verbatim. The field is
   absent on error responses. Excluded from the field: DataFusion physical plan
   internals, cost estimates, alias-expansion internals, join-order, and
   partition/pushdown details. The exact wire name (`normalized_pql`,
   `planned_query`, or similar) is deferred to the BC author; the semantic contract
   (normalized-post-parse PQL string, optimizer-internals excluded) is fixed by this
   ADR. The response type carrying this field must be `#[non_exhaustive]`; increment
   `ci.yml EXPECTED` accordingly.

**Suggested BC anchors (product-owner decides final set):**
- BC-2.10.XXX: `prism_describe` schema discovery tool (postconditions: per-client table
  catalog returned, audit event emitted, non-existent client_id handled, empty schema
  handled).
- BC-2.10.YYY: `prismql://schema/{client_id}` resource template (postconditions: same
  content as `prism_describe`, per-client scoped, subscribe/listChanged server-side
  support).
- BC-2.10.ZZZ: `prismql://reference` static resource (postconditions: full grammar
  content present, MIME type correct, content is server-authored).
- BC-2.11.NNN: E-QUERY-038 column-not-found error (postconditions: `available_columns`
  always present; `did_you_mean` present when Levenshtein distance ≤ 3; no credential
  or full-URL data in column names).
- BC-2.10.009 amendment: add `query_tutorial` to the list of required prompts (or a
  new BC if the amendment footprint is too large).
- BC-2.11.MMM (new — echo-normalized-PQL): `query` tool successful response includes
  `normalized_pql` field containing the Chumsky-normalized PQL string.
  Postconditions: (a) field is present and non-empty on every successful query execution;
  (b) field is absent on all error responses (E-QUERY-NNN); (c) value is the
  server-normalized form, not raw model input — i.e., it round-trips through the Chumsky
  parser; (d) value contains no DataFusion plan internals (no node-type strings, cost
  estimates, or optimizer annotations); (e) value is injection-safe: it is server-emitted
  normalized PQL, sourced from trusted Chumsky output, never from sensor API bodies.

---

## Source / Origin

- Research artifact: `.factory/research/llm-query-dsl-onboarding-2026-06-19.md` —
  decision-oriented synthesis; 3 Perplexity deep-research calls; 2 WebFetch
  verifications against MCP 2025-06-18 spec.
- D-1162 USER SCOPE DECISION (2026-06-14): capability-discovery block REQUIRED per
  `DEMO-SCOPE.md §Capability-Discovery Block`.
- `crates/prism-mcp/src/server.rs:1735-1744` — current `query` tool description
  (L1 primer upgrade target).
- `crates/prism-query` — Chumsky parse + plan gate (L4 E-QUERY infrastructure).
- ADR-039 — org-scoped `TableRegistry` error filtering; L4 `E-QUERY-037` already
  pedagogical (available_sensors / available_tables); L2 shares same `TableRegistry`.
- BC-2.10.008 v1.12, BC-2.10.009 v1.3, BC-2.10.011 v1.6 — S-5.03 MERGED
  develop@85ac7b06 — MCP Resources + Prompts surfaces that L2/L3/L1 compose with.
- MCP specification 2025-06-18 (resource templates, subscribe, listChanged,
  annotations): verified live by research-agent 2026-06-19.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-19 | architect | Initial ADR — 4-layer teaching mechanism, 6 open-question resolutions, scope statement for product-owner story. |
| 1.1 | 2026-06-19 | architect (human-directed amendment) | OPD-1 RESOLVED — Echo-normalized-PQL-back (Pattern B) adopted in v1 per human product decision, project lead, 2026-06-19. "Echo-Planned-PQL-Back: DEFERRED" section replaced with "Echo-Normalized-PQL-Back: ADOPTED IN V1" including injection-safety reasoning, echo surface spec (Chumsky-normalized PQL, optimizer internals excluded), `normalized_pql` response field placement, token-cost acceptance, and L4 composition benefit. Architectural Surface table updated (new echo row). Scope Statement updated (item 8 + BC anchor BC-2.11.MMM). Alternatives Considered Option D updated to reflect adoption with scoping. Open Product Decisions section updated to RESOLVED. |
| 1.2 | 2026-06-23 | architect (SR-004 resolution) | E-QUERY-039 allocated (L4 suite); gate covers `EnrichStage` (pipe-mode) + `FuncCall::Scalar{ScalarFunc::Unknown}` SELECT-projection path; payload: `infusion, available_infusions, did_you_mean`; closes AUDIT-005. L4 error-suite table: new E-QUERY-039 row added. Architectural Surface table: L4-pedagogical-errors row updated from `E-QUERY-001/002/003/037/038` to `E-QUERY-001/002/003/037/038/039`. Scope Statement item 6: E-QUERY-039 bullet added with InfusionRegistry process-global note and CWE-200 follow-up flag. Bound to BC-2.11.019 + InfusionRegistry (BC-2.19.001). |
