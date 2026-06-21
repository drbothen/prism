---
document_type: behavioral-contract
level: L3
version: "1.3"
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
modified: 2026-06-21
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.013: `prismql://schema/{client_id}` Resource Template (L2)

## Description

The `prismql://schema/{client_id}` MCP resource template exposes the per-client table/column/type schema catalog as an MCP resource, providing an alternative access pattern to the `prism_describe` tool (BC-2.10.012). Both surfaces are computed from the same spec-layer column data — `query_engine.resolved_spec_map()` (multi-tenant) or `config_manager` (single-tenant fallback) — which is the single source of truth. The resource template supports server-side `subscribe`/`listChanged` notification (MCP 2025-06-18 spec), enabling MCP clients that support subscription to receive proactive schema-freshness signals when the `TableRegistry` changes (the `TableRegistry` change event is the hot-reload signal, even though column data itself comes from `resolved_spec_map`). The resource is always registered and uses `application/json` MIME type.

## Preconditions

1. The MCP server has registered the `prismql://schema/{client_id}` URI template in `resources/list` using RFC 6570 template syntax.
2. The `TableRegistry` has been initialized (may be empty).
3. The resource handler accesses column schema via the same two `PrismServer` fields used by `prism_describe` (BC-2.10.012) — no additional injection required:
   - `self.query_engine.resolved_spec_map()` → `Arc<HashMap<(OrgSlug, SensorId), ResolvedSensorSpec>>` (multi-tenant preferred path)
   - `self.config_manager` → `Arc<ArcSwap<ConfigManager>>` (single-tenant/test fallback)
   Note: `TableRegistry` is a concrete `#[non_exhaustive] struct` storing table-name strings only; it holds NO per-column schema. Column data comes from the spec layer above.
4. Server-side subscribe/notify infrastructure is wired (consistent with `notifications/resources/updated` already used by BC-2.10.008 for config resources).

## Postconditions

### Resource template registration

- `prismql://schema/{client_id}` appears in `resources/list` as a URI template (not a static URI).
- The template is declared in `resources/list` with `uriTemplate: "prismql://schema/{client_id}"` per MCP resource template spec (MCP 2025-06-18).
- The resource is annotated with `mimeType: "application/json"`.
- The resource description states: "Per-client PQL table/column/type schema catalog. Subscribe to receive schema-change notifications."

### Resource content — identical to `prism_describe` response shape

The resource read result for `prismql://schema/{client_id}` returns exactly the same JSON content as a successful `prism_describe(client_id)` call (BC-2.10.012):

```json
{
  "client_id": "<client_id from URI>",
  "tables": [ { "name": "...", "sensor_type": "...", "description": "...", "columns": [...], "example_query": "..." } ],
  "pql_hints": ["..."]
}
```

The content is computed from the same spec-layer data source as `prism_describe`: `query_engine.resolved_spec_map()` (multi-tenant) or `config_manager` (single-tenant fallback). There is no separate schema snapshot for the resource — both paths share the same underlying `ArcSwap`-backed data.

### Non-existent / empty client_id behavior

Mirrors BC-2.10.012: unknown-but-well-formed `client_id` → success with `tables: []` and a hint. Invalid format → MCP resource error.

### Server-side `subscribe` / `listChanged` support

The server MUST implement the subscribe/notify side of the MCP 2025-06-18 resource subscription spec:

1. A client that calls `resources/subscribe` with URI `prismql://schema/acme` is registered as a subscriber for schema changes affecting "acme".
2. When the resolved schema changes for "acme" — either via `TableRegistry` change (a sensor is added or removed) or via a hot-reload of the underlying sensor spec that causes `resolved_spec_map` to be rebuilt and swapped per ADR-042 (e.g., a new table is added to the CrowdStrike TYPE spec) — the server MUST send `notifications/resources/updated` with `uri: "prismql://schema/acme"` to all subscribers for that client. This applies in both single-tenant mode and multi-tenant overlay mode (org-slug != sensor-id), because the notify-diff reads from the rebuilt `resolved_spec_map` in multi-tenant mode (ADR-042 §D5).
3. The server-side subscriber registry persists for the lifetime of the MCP session (stdio connection).
4. `resources/unsubscribe` removes the subscriber.
5. Whether Claude Code's current MCP client acts on `subscribe`/`listChanged` is an implementation-time verification task (ADR-041 §Architectural Surface — "not confirmed from public docs" at design time). The server implementation is required regardless; it will be exercised when clients that support it are encountered.

### Caching policy

The resource handler MAY cache the per-client schema response (derived from `resolved_spec_map` or `config_manager`) with a short TTL (e.g., 5 seconds) to serve repeated reads efficiently. The cache MUST be invalidated on `TableRegistry::changed()` signal (the same signal used to trigger `notifications/resources/updated`). Cache invalidation and subscribe notification are triggered by the same event — they share a single `TableRegistry` change listener. Note: the `TableRegistry` change event is the hot-reload signal (authority for which table names are registered); the cached value being invalidated is the derived spec-layer schema, not a `TableRegistry` struct.

### Audit — NO separate audit event

Reading `prismql://schema/{client_id}` as a resource does NOT emit a separate `AuditEntry`. The audit trail for schema enumeration is tied to `prism_describe` tool calls (BC-2.10.012 §Postconditions — Audit event emission). Resource reads are considered equivalent to passive context injection (e.g., host prefetching schema to inject as prompt context) and do not generate per-read audit events. This is a deliberate design decision: resource reads may occur frequently in automated host pipelines; forcing an audit event for each read would flood the audit log with non-analyst-initiated events.

Rationale: ADR-041 §Architectural Surface §Rationale ("Why a tool AND a resource"): the audit requirement motivates the TOOL path. The resource path serves hosts and subscriptions. The two paths are complementary.

### Single source of truth invariant

At any moment, `resources/read("prismql://schema/acme")` and `prism_describe("acme")` MUST return semantically identical content for the same client. If there is any difference (e.g., due to cache), it is a transient inconsistency bounded by the cache TTL. Persistent inconsistency (e.g., the resource caches stale data while the tool returns fresh data) is a violation of this postcondition.

## Invariants

- DI-002: Credential isolation — same as BC-2.10.012; no credential values in resource content.
- DI-008: Client data separation — the `{client_id}` URI segment is the scoping boundary. A resource read for `prismql://schema/acme` MUST NOT return tables belonging to "globex".
- DI-006: Resource content is server-authored (operator TOML specs → `resolved_spec_map` / `config_manager`); not sensor data; no prompt injection scan required.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| MCP resource error | `client_id` in URI fails format validation | MCP resource error: "Invalid client_id in resource URI" (consistent with other resource errors in BC-2.10.008) |

**Non-error cases:**
- Unknown-but-well-formed `client_id` → `{tables: [], pql_hints: [...]}` (not an error)
- Empty schema → `{tables: [], pql_hints: [...]}` (not an error)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-029 | `resources/subscribe` for `prismql://schema/acme` followed by a hot-reload that adds a CrowdStrike table for "acme" — applies in BOTH single-tenant mode and multi-tenant overlay mode (including the case where org-slug "acme" != sensor-id "crowdstrike") | Server sends `notifications/resources/updated` with `uri: "prismql://schema/acme"` within 1 second of the resolved schema change event (rebuilt `resolved_spec_map` swap per ADR-042 in multi-tenant mode, or `TableRegistry` change in single-tenant mode) |
| EC-10-030 | `resources/subscribe` for `prismql://schema/acme`; TableRegistry changes for "globex" (different client) | No notification sent for "acme" — per-client subscription scoping |
| EC-10-031 | `resources/read("prismql://schema/acme")` and `prism_describe("acme")` called within 5 seconds of a hot-reload | Both return the same schema (cache TTL window); if cache is stale, next read after TTL returns fresh schema |
| EC-10-032 | MCP client does not support `resources/subscribe` | Server registers the template in `resources/list` unconditionally; no subscribe calls arrive; no error |
| EC-10-033 | URI with invalid `client_id` format (`"prismql://schema/acme/../etc"`) | MCP resource error: "Invalid client_id in resource URI" |
| EC-10-034 | Org "acme" maps to sensor "crowdstrike" (org-slug != sensor-id); a hot-reload that modifies the underlying crowdstrike TYPE spec to add a table; a second org "globex" whose resolved schema did NOT change also has subscribers | Server sends `notifications/resources/updated` with `uri: "prismql://schema/acme"` within 1 second (ADR-042 D5 per-org notify-diff reads from rebuilt `resolved_spec_map`, keyed by org-slug). "globex" subscribers receive NO notification — per-client scoping enforced by DI-008. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `resources/read("prismql://schema/acme")` with CrowdStrike configured for "acme" | Same JSON structure as `prism_describe("acme")` — `client_id: "acme"`, tables array, pql_hints | happy-path |
| `resources/read("prismql://schema/acme")` with zero tables configured | `{client_id: "acme", tables: [], pql_hints: [...]}` | empty-schema |
| `resources/list` | `prismql://schema/{client_id}` appears as URI template with `mimeType: "application/json"` | registration |
| Content comparison: `resources/read("prismql://schema/acme")` vs `prism_describe("acme")` (same instant) | Structurally identical JSON | single-source-of-truth |
| `resources/subscribe("prismql://schema/acme")` then `prism_describe` for acme shows new table after hot-reload | `notifications/resources/updated` sent with `uri: "prismql://schema/acme"` | subscribe-notify |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (VP-TBD) | `prismql://schema/{client_id}` resource read and `prism_describe(client_id)` return identical table names and column names for the same client | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| Capability Anchor Justification | CAP-034 ("MCP Server & Transport") per capabilities.md §CAP-034 — this BC defines a new MCP resource template (`prismql://schema/{client_id}`) registered in the MCP server's `resources/list`. CAP-034 explicitly covers "MCP resources expose dynamic Prism state as subscribable `resources/list` entries; `notifications/resources/updated` is sent when resource content changes." The schema discovery resource template is exactly such a dynamic, subscribable resource. |
| L2 Invariants | DI-002, DI-006, DI-008 |
| ADR | ADR-041 v1.1 §L2 — `prismql://schema/{client_id}` resource template; §Rationale — "Why a tool AND a resource" |
| Architecture Module | SS-10 (MCP Interface) |
| Priority | P1 |

## Related BCs

- BC-2.10.012 — composes with: `prism_describe` is the tool-path for the same data; this BC is the resource-path complement
- BC-2.10.008 — composes with: `prism://config/clients/{client_id}/sensors` shows provisioned sensors; this resource shows queryable tables — different abstraction levels; both use subscribe/listChanged pattern
- BC-2.10.005 — depends on: `notifications/tools/list_changed` pattern established there is the precedent for server-push notification on registry changes

## Architecture Anchors

- `architecture/decisions/ADR-041` §L2 — "Why a tool AND a resource": audit motivates the tool path; host injection and subscribe/listChanged motivate the resource path
- `architecture/decisions/ADR-022` — Arc-DI wiring: the resource handler reads column schema via `PrismServer.query_engine.resolved_spec_map()` and `PrismServer.config_manager` — existing wired fields, no new `Arc<dyn TableRegistry>` injection (correction per architect D-1259)
- `architecture/decisions/ADR-039` — `TableRegistry` is the authority for which table NAMES are registered per org (and the change-notification signal source); column schema for those tables comes from `resolved_spec_map → ResolvedSensorSpec.spec.tables → TableSpec.columns → ColumnSpec` (per onboarding-001-tableregistry-datapath-correction.md D-1259)
- `architecture/decisions/ADR-042` — Reload-aware `resolved_spec_map`: changes field type to `Option<Arc<ArcSwap<HashMap>>>`, adds `rebuild_resolved_spec_map` called after each hot-reload, and updates the per-org notify-diff to read from the rebuilt map (D5). This makes EC-10-029 and EC-10-034 satisfiable in multi-tenant overlay mode (org-slug != sensor-id).

## Story Anchor

S-5.04 (or dedicated ADR-041 teaching story — to be assigned by story-writer)

## VP Anchors

VP assignments TBD — assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | OBS-PR197-RG3-P3-hygiene-2026-06-21 | 2026-06-21 | product-owner | Spec hygiene (OBS-PR197-RG3-P3-001): `introduced:` normalized from opaque burst-ID to bare ISO date `2026-06-19` (POL-20). No body semantics changed. |
| 1.2 | ADR-042-bc-update-2026-06-20 | 2026-06-20 | product-owner | D-1267 / ADR-042: multi-tenant schema-change notify now IN SCOPE for 001-A. Removed single-tenant-only limitation from EC-10-029 (now applies to both modes). Added EC-10-034 (org "acme" maps to sensor "crowdstrike"; hot-reload of TYPE spec notifies "acme" only, not "globex"). Updated §Subscribe/listChanged item 2 to reference resolved_spec_map rebuild trigger (ADR-042 §D5) in addition to TableRegistry change. Added ADR-042 to §Architecture Anchors. No existing postcondition weakened. |
| 1.1 | 001-A-local-cascade-bc-correction-2026-06-20 | 2026-06-20 | product-owner | D-1259 propagation (onboarding-001-tableregistry-datapath-correction.md). Replaced fictional `Arc<dyn TableRegistry>` injection model with correct data-source model in §Description, Preconditions 3–4, §Resource content, §Caching policy, and §Architecture Anchors. Behavioral postconditions, DI-008 isolation requirement, subscribe/notify behavior, edge cases, and test vectors unchanged (finding F-PASS2-HIGH-004 / F-P3-HIGH-002). |
| 1.0 | ADR-041-teaching-burst-2026-06-19 | 2026-06-19 | product-owner | Initial draft — ADR-041 L2 `prismql://schema/{client_id}` resource template contract |
