---
document_type: behavioral-contract
level: L3
version: "1.1"
status: active
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 1a
inputs: [".factory/specs/domain-spec/capabilities.md", ".factory/specs/domain-spec/invariants.md", ".factory/specs/architecture/decisions/ADR-041-prismql-llm-auto-onboarding-4-layer-teaching-surface-for-automatic-agent-query-authoring.md"]
input-hash: "TBD"
traces_to: ["CAP-034"]
extracted_from: null
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: 2026-06-19
modified: 2026-06-22
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.014: `prismql://reference` Static PQL Grammar Reference Resource (L3)

## Description

The `prismql://reference` MCP resource is a non-parameterized (static URI) resource that provides the complete PQL grammar and language reference on demand. It is server-authored at build time and never sourced from sensor data or user input. It is always registered but not always-present in the model's context — the model fetches it when it needs deeper grammar guidance than the L1 tool-description primer provides. This resource is the L3 layer of the ADR-041 4-layer LLM onboarding mechanism.

## Preconditions

1. The MCP server has registered `prismql://reference` as a static (non-template) URI in `resources/list`.
2. Resource content is compiled into the `prism-mcp` binary at build time (not loaded from the filesystem at runtime). It is a static `&str` or `Cow<'static, str>` embedded via `include_str!`.
3. Resource content is authored by the Prism engineering team (server-authored). It is NEVER generated from sensor data, user input, or external API responses.

## Postconditions

### Resource registration

- `prismql://reference` appears in `resources/list` as a static (non-template) URI.
- `mimeType: "text/markdown"` (preferred; `text/plain` is acceptable if the MCP client does not support markdown rendering).
- MCP `annotations.priority` is set to indicate model-relevant content (per MCP 2025-06-18 `annotations` field) — this signals to MCP clients that can honor priority hints that this resource is relevant for LLM reasoning.
- Resource description: "Full PrismQL grammar reference, operator semantics, error code quick-reference, and query examples. Fetch when the L1 primer is insufficient for the query you want to write."

### Resource content — required sections

The resource MUST contain the following sections (order may vary; headers are required):

1. **`## What is PrismQL`** — one paragraph: "PrismQL (PQL) is a custom federated query DSL for the Prism MSSP platform. It queries live sensor APIs via an ephemeral in-memory data lake. PQL is NOT SQL — it has its own syntax that must be learned from this reference."

2. **`## Clause Grammar (BNF)`** — BNF-style grammar covering:
   - `SELECT` clause: `SELECT * | SELECT col1, col2, COUNT(*), ...`
   - `FROM` clause: `FROM <sensor_table>` (table names come from `prism_describe`)
   - `WHERE` clause: field predicates, boolean operators (`AND`, `OR`, `NOT`), comparison operators (`=`, `!=`, `>`, `>=`, `<`, `<=`), `IN (...)`, `LIKE`, `BETWEEN`
   - `GROUP BY` clause
   - `ORDER BY ... ASC|DESC`
   - `LIMIT N`
   - Filter mode shorthand: `field = value AND field >= value` (no SELECT/FROM required)
   - Pipe mode: `FROM <table> | where ... | sort ... | head N | enrich ... | limit N`

3. **`## Operators and Types`** — table mapping: operator → valid ColumnTypes → invalid ColumnTypes, with one example per operator. Covers: `=`, `!=`, `<`, `>`, `<=`, `>=`, `IN`, `LIKE`, `BETWEEN`, `AND`, `OR`, `NOT`.

4. **`## Datetime Arithmetic`** — `NOW()`, `INTERVAL 'Nh'` / `INTERVAL 'Nd'` / `INTERVAL 'Nm'`, OCSF timestamp field paths (e.g., `time`, `start_time`, `end_time`).

5. **`## Error Code Quick-Reference`** — table of caller-recoverable E-QUERY-NNN codes:

   | Code | Trigger | Recovery |
   |------|---------|---------|
   | E-QUERY-001 | Parse error (syntax) | Check grammar in this reference; near_text field shows offending token |
   | E-QUERY-002 | Type error (wrong operator for column type) | Use valid_operators_for_type list in the error |
   | E-QUERY-003 | Security limit exceeded (query too long, too deep) | Shorten or simplify the query |
   | E-QUERY-037 | Table unavailable (sensor not configured) | Use available_tables list in the error; call prism_describe to see configured tables |
   | E-QUERY-038 | Column not found | Use available_columns list in the error; call prism_describe to see real column names |

6. **`## Query Examples (5–10)`** — multi-clause examples covering:
   - Aggregate with GROUP BY and ORDER BY (at least one example)
   - Multi-condition WHERE clause (at least one example)
   - Pipe mode (at least one example)
   - Time-bounded query using `NOW()` (at least one example)
   - Cross-field correlation (at least one example)
   
   All examples use placeholder table names (e.g., `<sensor_table>`) or generic names (e.g., `sensor_detections`). No examples may hard-code a real sensor vendor table name that would cause hallucination on clients lacking that sensor.

7. **`## Self-Correction Workflow`** — prose: "On E-QUERY error: (1) read the error's actionable fields (near_text, available_columns, available_tables, valid_operators_for_type, did_you_mean); (2) consult this reference for correct grammar; (3) retry up to 3 times before reporting failure to the user."

### Content authorship invariant

EVERY byte in `prismql://reference` MUST originate from server-authored Prism engineering team code. The following are PROHIBITED in this resource:
- Sensor API response data (any field or value from a sensor API)
- User free-text input (analyst chat messages, query history, user-entered config)
- Dynamically interpolated external content

This is the injection-safety boundary: the reference resource is trusted content; its safety does not depend on runtime scanning.

### Not always-present (on-demand only)

The `prismql://reference` resource is NOT injected into every model prompt. It is fetched on demand. The L1 primer (BC-2.10.009 amendment + `query` tool description) is always-present; this resource provides depth for complex queries. This design avoids burning ~1,500 tokens per turn for simple queries.

### No `subscribe`/`listChanged`

The static reference resource does NOT support `subscribe`/`listChanged`. Its content changes only on Prism binary updates (build-time embedded), not on runtime events. Subscribing to it would generate spurious update notifications on server restart and add complexity for zero benefit.

## Invariants

- DI-006: Resource content is 100% server-authored. No sensor data, no user input, no external API content flows into this resource.
- The injection-safety boundary for `prismql://reference` is simpler than for `prismql://schema/{client_id}` (which relies on operator TOML → `TableRegistry` chain): the reference is pure build-time static content.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| MCP resource not found | Resource URI does not match `prismql://reference` exactly (e.g., `prismql://references`) | Standard MCP resource-not-found error |

There are no parameter validation errors for a static URI resource (no URI template variables).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-034 | Model fetches `prismql://reference` during a Prism hot-reload (config update) | Resource read returns the build-time static content regardless of config state — static content does not change on reload |
| EC-10-035 | Model fetches `prismql://reference` when no clients are configured | Same content returned — the reference is schema-agnostic by design |
| EC-10-036 | `prismql://reference` content size | Content MUST be ≤ 3,000 tokens (approximately 12KB plain text). This bound prevents the resource from becoming an always-present token drain if a host injects it by default. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `resources/read("prismql://reference")` | Response with `mimeType: "text/markdown"` (or `text/plain`); content contains all 7 required sections (## What is PrismQL, ## Clause Grammar, ## Operators and Types, ## Datetime Arithmetic, ## Error Code Quick-Reference, ## Query Examples, ## Self-Correction Workflow) | happy-path |
| `resources/list` | `prismql://reference` appears as a static (non-template) URI with correct mimeType and annotations.priority set | registration |
| Content inspection: `prismql://reference` after hot-reload | Identical to pre-reload content — static build-time content does not change | static-invariant |
| Content inspection: contains no real sensor table names | No hardcoded vendor table names (e.g., not `crowdstrike_alerts`, `armis_devices`) in examples — only `<sensor_table>` or generic placeholder names | injection-safety |
| Content inspection: error quick-reference table | Contains rows for E-QUERY-001, E-QUERY-002, E-QUERY-003, E-QUERY-037, E-QUERY-038 | error-reference |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (VP-TBD) | `prismql://reference` content never changes between successive reads within the same server process lifetime | integration test (read twice, compare) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| Capability Anchor Justification | CAP-034 ("MCP Server & Transport") per capabilities.md §CAP-034 — this BC defines a new MCP resource (`prismql://reference`) registered in the MCP server's `resources/list`. CAP-034 explicitly states "MCP resources expose dynamic Prism state as subscribable `resources/list` entries." Although `prismql://reference` is static (not dynamic), it is still an MCP resource registered by the MCP server layer (SS-10), which is the surface defined by CAP-034. The static/dynamic distinction is an implementation detail; the MCP surface ownership is CAP-034. |
| L2 Invariants | DI-006 |
| ADR | ADR-041 v1.1 §L3 — Full Grammar Reference Resource (`prismql://reference`) |
| Architecture Module | SS-10 (MCP Interface) |
| Priority | P1 |

## Related BCs

- BC-2.10.012 — composes with: `prism_describe` discovers per-client schema; `prismql://reference` provides the grammar to write queries against that schema
- BC-2.10.009 (amended) — composes with: `query_tutorial` MCP Prompt references `prismql://reference` in its "step 3: consult reference on grammar error" workflow
- BC-2.11.016 — composes with: E-QUERY-001 pedagogical upgrade includes a pointer to `prismql://reference` in the error response

## Architecture Anchors

- `architecture/decisions/ADR-041` §L3 — "Full Grammar Reference Resource": on-demand depth; build-time static; ~1,500 tokens; not always-present

## Story Anchor

S-5.04 (or dedicated ADR-041 teaching story — to be assigned by story-writer)

## VP Anchors

VP assignments TBD — assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | F-001B-FRESH2-MED-001-pol20-normalization | 2026-06-22 | product-owner | POL-20 normalization: `introduced: ADR-041-teaching-burst-2026-06-19` → `introduced: 2026-06-19` (opaque burst-ID format prohibited by POL-20 anchored-regex; ISO date extracted). Also set `modified: 2026-06-22` (first amendment; POL-27). Sibling sweep of BC-2.11.016/017/018 in same burst. No body semantics changed. |
| 1.0 | ADR-041-teaching-burst-2026-06-19 | 2026-06-19 | product-owner | Initial draft — ADR-041 L3 `prismql://reference` static grammar resource contract |
