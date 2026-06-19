---
document_type: behavioral-contract
level: L3
version: "1.12"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [domain-spec/capabilities.md, domain-spec/invariants.md]
input-hash: "4a1657f"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
extracted_from: null
subsystem: "SS-10"
capability: "CAP-008, CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: ["cycle-1-burst-45", "cycle-1-burst-49", "pass-69-housekeeping", "pass-73-fix", "pass-79-fix", "F-S503-002-adjudication-2026-06-17", "S-5.03-org-no-overlay-semantics-2026-06-17", "RECONCILIATION-1-clients-shape-2026-06-17", "B1-display-name-addition-2026-06-18", "RECONCILIATION-1B-sensor-inventory-shape-2026-06-18"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.008: MCP Resources for Client List and Sensor Inventory

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.12 | RECONCILIATION-1B-sensor-inventory-shape-2026-06-18 | 2026-06-18 | product-owner | **RECONCILIATION-1B — postcondition 2 field names reconciled to shipped `SensorConfigEntry` shape; EC-10-015 retired.** Postcondition 2 rewritten: correct fields are `sensor_type`, `status`, `credential_ref`, `sources`, `api_base_url` (matching the story spec Task 1 / AC-2 shape). The stale field names `sensor_id`, `enabled` (boolean), `configured_sources` have been removed — they did not match the story-authoritative `SensorConfigEntry` shape. **`status` semantics ruling:** `status` is always `"active"` in the config-driven overlay model. Sensors only appear in `resolved_spec_map` when provisioned by an overlay; presence in `resolved_spec_map` is definitionally "active". The overlay model has no "disabled but present" state — a sensor absent from the overlay does not appear in this resource at all. Therefore `status: "active"` as a constant is definitionally correct and is NOT a hollow hardcode; it reflects a true invariant of the data model. EC-10-015 ("all disabled → enabled: false") is retired with this ruling: the scenario is structurally impossible under the config-driven model. EC-10-015 is marked retired in the edge cases table with an un-retire condition for a future story that introduces per-sensor lifecycle flags. No code change required for `status` (the constant value is correct). Stale field name `sensor_id` in the old postcondition text was a pre-RECONCILIATION-1 artifact; `api_base_url` host-only requirement retained unchanged. |
| 1.11 | B1-display-name-addition-2026-06-18 | 2026-06-18 | product-owner | **B1 — `display_name` added to `ClientInventoryEntry` per human-approved architect ruling.** Postcondition 1 updated: `ClientInventoryEntry` shape is now `{ client_id, display_name (String\|null), sensor_count, enabled_sensors }`. `display_name` = human-readable org name sourced from `[[orgs]].name` in `prism.toml` (the new `OrgEntry.name: Option<String>` field, `#[serde(default)]`); serializes to JSON `null` when absent. Read directly from config snapshot in `render_client_list_resource`. `capabilities_summary` explicitly noted as OUT OF SCOPE (canonical surface = BC-2.10.011 `list_capabilities`). Canonical test vector updated: "acme" with `name = "Acme Corp"` → `display_name: "Acme Corp"`; "globex" with no `name` → `display_name: null`. Bumped v1.10→v1.11. |
| 1.10 | RECONCILIATION-1-clients-shape-2026-06-17 | 2026-06-17 | product-owner | **RECONCILIATION-1 — `prism://config/clients` response shape caught up to shipped `ClientInventoryEntry`.** Postcondition 1 rewritten: the correct fields are `client_id`, `sensor_count`, `enabled_sensors` (matching `ClientInventoryEntry` as shipped per Story S-5.03 Task 1 deliberate decision). The old postcondition 1 fields (`display_name`, `sensors` as a bare list, `capabilities_summary`) were stale spec text that was superseded by the story scope decision. **Verdict: pure spec-catch-up (BC amended to match shipped shape).** `display_name` and `capabilities_summary` were never anchored to a product requirement or domain invariant in CAP-008/CAP-009 — they were placeholder fields in the original draft. The story spec (source-of-truth for implementation scope per CLAUDE.md §1) defines `ClientInventoryEntry` as `{ client_id, sensor_count, enabled_sensors }`. No code change required. Canonical test vectors updated to reference `ClientInventoryEntry` by name. **Note for human:** if `display_name` (human-readable org name) and `capabilities_summary` (count of write-capable sensors) are genuinely desired in the MCP surface for AI consumers, that requires a product decision and a new story — they are NOT present in the current implementation. Flag to orchestrator if needed. **Bumped v1.9→v1.10.** |
| 1.9 | S-5.03-org-no-overlay-semantics-2026-06-17 | 2026-06-17 | product-owner | **S-5.03 Item 1 — org-with-no-overlay semantics documented (Option B: zero sensors).** Postcondition 1 extended with an unambiguous zero-sensor clause: an org that is registered in `OrgRegistry` but has zero entries in `resolved_spec_map` for its `OrgSlug` MUST expose zero sensors via `prism://config/clients/{client_id}/sensors`. Rationale: (a) `resolved_spec_map: HashMap<(OrgSlug,SensorId),ResolvedSensorSpec>` is the definitive per-org provisioned set — only orgs with explicit overlay entries or registered TYPE-spec-backed adapter entries appear in it; an org with zero entries is not provisioned for any sensor. (b) The multi-client SOC demo requires DIFFERENTIATED sensor combinations per org: if Option A (all TYPE specs visible by default) were adopted, every org would see every sensor type, destroying the differentiation that drives the demo's value. (c) Option A would require inventing a "global TYPE spec list" data source that is separate from `resolved_spec_map`, introducing a new code path not anchored to any existing data structure. (d) MSSP correctness: an org is provisioned for the sensors they explicitly configure (via `customers/<slug>/*.overlay.toml`); unexplained visibility of unprovisioned sensor types would be an over-disclosure. BC-2.06.012 EC-012-003 confirms that a SaaS sensor with no per-org overlay produces NO `ResolvedSensorSpec` entry (fanout engine fallback to TYPE spec is an EXECUTOR optimization, not an indicator of provisioned inventory). **Bumped v1.8→v1.9.** |
| 1.8 | F-S503-002-adjudication-2026-06-17 | 2026-06-17 | product-owner | **F-S503-002 adjudication — per-client scoping and host-only URL field clarified.** (1) Postcondition 2 rewritten to be unambiguous: `prism://config/clients/{client_id}/sensors` MUST filter by the `client_id` URI segment — returning all sensors regardless of `client_id` is a DI-008 violation; this is IN SCOPE for S-5.03. `api_base_url` field added explicitly to the postcondition with the requirement that it be present and contain only scheme+host+port (no path, no query string, no credentials). (2) DI-008 Invariant strengthened: the `client_id` path segment is the authorization boundary; ignoring it is a data separation defect, not a multi-tenant deferral. DI-002 Invariant expanded to call out `api_base_url` host-only requirement. (3) VP-050 proptest already verifies the URL redaction; no VP change required. Story-writer propagation required for S-5.03: update AC-2 to explicitly assert per-client filtering and `api_base_url` host-only field presence. **Bumped v1.7→v1.8.** |
| 1.7 | pass-79-fix | 2026-04-20 | state-manager | MED-001 fix: removed stale `pass-72-fix` entry from modified array (no corresponding changelog row existed; pass-72 did not touch this file). |
| 1.6 | pass-73-fix | 2026-04-20 | state-manager | Renumbered changelog to close v1.4 gap: old v1.5→v1.4; old v1.6→v1.5; this row closes the sequence at v1.6. Original v1.3→v1.5 spanned two distinct burst events that were conflated at authoring time. |
| 1.5 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. (originally recorded as v1.6; renumbered by pass-73-fix) |
| 1.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-050); normalized changelog schema to canonical 5-col form. (originally recorded as v1.5; renumbered by pass-73-fix) |
| 1.3 | cycle-1-burst-45 | 2026-04-20 | product-owner | pre-build-sweep: Template-compliance sweep — appended Changelog row (version bump 1.2→1.3). |
| 1.2 | cycle-1-burst-49 | 2026-04-19 | architect | Burst 49 / P3P48-A-HIGH-003: Canonicalized all resource URIs: `prism://clients` → `prism://config/clients`; `prism://clients/{client_id}/sensors` → `prism://config/clients/{client_id}/sensors`. Updated Description, Postconditions, Invariants, Error Cases, Edge Cases, Canonical Test Vectors, Verification Properties, and Architecture Anchors. |
| 1.1 | cycle-1-burst-45 | 2026-04-19 | product-owner | Burst 45 / P3P44-A-HIGH-003: Health resource reference updated from `prism://health/{client_id}` to `prism://sensors/health` (global matrix) to match api-surface.md. Added missing template sections: Description, Canonical Test Vectors, Verification Properties. |
| 1.0 | — | 2026-04-14 | product-owner | Initial draft |

## Description

This BC governs the MCP resources that expose client inventory and per-client sensor configuration: `prism://config/clients` (all configured clients) and `prism://config/clients/{client_id}/sensors` (sensor inventory for a specific client). It also references `prism://sensors/health` (governed by BC-2.08.006) as part of the complete resources registry. All three resources are read-only and reflect startup-time configuration; they never expose credential values.

## Preconditions

1. Prism has loaded configuration and initialized all client/sensor mappings
2. MCP resources are registered in `resources/list`

## Postconditions

1. `prism://config/clients` resource returns a JSON array of all configured clients. Each element is a `ClientInventoryEntry` with fields: `client_id` (the OrgSlug string), `display_name` (the human-readable org name sourced from `[[orgs]].name` in `prism.toml`; serializes to JSON `null` when the `name` field is absent from the `OrgEntry`), `sensor_count` (integer count of provisioned sensors for this client), `enabled_sensors` (array of sensor ID strings provisioned for this client). The sensor list for each client is derived from `resolved_spec_map` entries for that client's `OrgSlug`. **An org registered in `OrgRegistry` but with zero entries in `resolved_spec_map` for its `OrgSlug` MUST appear in this list with `sensor_count: 0` and `enabled_sensors: []` — it is present as a client but provisioned for zero sensors.** An org whose `OrgSlug` is NOT registered in `OrgRegistry` at all MUST NOT appear in this list. **`capabilities_summary` is NOT part of this shape** — the canonical capabilities surface is `BC-2.10.011` (`list_capabilities` meta-tool); duplicating it here would create a stale-cache hazard. Do not add `capabilities_summary` to `ClientInventoryEntry`.
2. `prism://config/clients/{client_id}/sensors` resource returns detailed sensor inventory SCOPED TO THE SPECIFIED `client_id` ONLY — the handler MUST filter by the `client_id` URI segment before returning results. Each entry is a `SensorConfigEntry` with fields: `sensor_type` (the sensor type identifier, e.g., `"crowdstrike"`), `status` (always `"active"` — sensors appear in this resource only because they are present in `resolved_spec_map`; any sensor in `resolved_spec_map` is definitionally provisioned/active; the config-driven overlay model does not support a "disabled but present in overlay" state — absence from `resolved_spec_map` means the sensor is not provisioned and does not appear here at all), `credential_ref` (reference key used to look up the sensor's credential — never the credential value), `sources` (list of data source identifiers for this sensor), and `api_base_url` (host+port component only — full URL and path MUST be stripped; no credentials). **The `api_base_url` field MUST be present and MUST contain only the scheme+host+port (e.g., `"https://api.crowdstrike.com"`); full URL paths, query strings, and credentials MUST NOT appear in this field.** Any implementation that returns the full API URL, omits `api_base_url`, or returns sensors from a `client_id` different from the URI segment violates this postcondition. **`sensor_id` and `enabled` (boolean) and `configured_sources` are NOT fields of this shape** — the correct fields are `sensor_type`, `status`, `credential_ref`, `sources`, `api_base_url` as described above.
3. `prism://sensors/health` resource returns cached health status per BC-2.08.006 (global cross-client matrix)
4. Resource content uses `application/json` MIME type
5. Resources are read-only and reflect startup-time configuration (no live updates until `reload_config`)
6. Credential values and full API URLs are never exposed in resource content

## Invariants

- DI-002: Credential isolation — no credential values in resource responses; `api_base_url` contains host+port only; full URLs and credentials MUST NOT appear in any field of the response
- DI-008: Client data separation — `prism://config/clients/{client_id}/sensors` MUST filter strictly to the `client_id` in the URI. An implementation that queries all sensors and returns them regardless of the `client_id` parameter violates DI-008. This is IN SCOPE for S-5.03 — not deferred to a multi-tenant story. The `client_id` path segment is the authorization boundary; ignoring it is a data separation defect.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Resource not found | Invalid `client_id` in `prism://config/clients/{client_id}/sensors` URI | MCP resource error: "Client '{id}' not found" |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-014 | Zero clients configured | `prism://config/clients` returns empty JSON array `[]` |
| EC-10-015 | ~~Client has sensors configured but all disabled~~ | **RECONCILED (v1.12, RECONCILIATION-1B):** This edge case is structurally unsatisfiable in the config-driven overlay model. `SensorConfigEntry.status` is always `"active"` because sensors only appear in `resolved_spec_map` when provisioned — there is no "disabled but present" state in the overlay model. A sensor that should be "disabled" is simply absent from the org's overlay files and therefore absent from `resolved_spec_map` and absent from this resource response. The correct edge case for a client with no active sensors is EC-10-016 (empty array). EC-10-015 is retired. If a future story introduces per-sensor enable/disable lifecycle flags in the overlay schema, that story must un-retire this edge case and add the corresponding `status` values. |
| EC-10-016 | Client has no sensors configured | `prism://config/clients/{id}/sensors` returns empty `sensors` array `[]`, not an error |
| EC-10-017 | Org registered in `OrgRegistry` with zero entries in `resolved_spec_map` | `prism://config/clients/{id}/sensors` returns empty `sensors` array `[]`. The org is provisioned but has no sensor overlays — zero sensors exposed. **This is Option B semantics: overlay = provisioned, not "customize a global default."** BC-2.06.012 EC-012-003 (SaaS sensor with no per-org overlay produces NO `ResolvedSensorSpec` entry) is the authoritative grounding. The query-engine fanout fallback to TYPE spec is an executor optimization that does NOT create provisioned inventory visible in MCP resources. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read `prism://config/clients` with two clients ("acme" with `name = "Acme Corp"`, "globex" with no `name`) | JSON array with two `ClientInventoryEntry` objects: `[{"client_id":"acme","display_name":"Acme Corp","sensor_count":N,"enabled_sensors":[...]},{"client_id":"globex","display_name":null,"sensor_count":M,"enabled_sensors":[...]}]`. `display_name` is `null` when `OrgEntry.name` is absent. | happy-path |
| Read `prism://config/clients/acme/sensors` with CrowdStrike and Claroty configured | JSON with two sensor entries; API URL shows host only (e.g., `api.crowdstrike.com`), no full URL or credentials | happy-path |
| Read `prism://config/clients/nonexistent/sensors` | MCP resource error: "Client 'nonexistent' not found" | error |
| Read `prism://config/clients` with zero clients configured | `[]` (empty array) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-050 | `render_sensor_inventory_resource()` given a ClientSensorConfig containing full API base URLs and credential values produces a response JSON where: (a) no string matching an API key pattern appears; (b) the API base URL field contains only the host+port component | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008, CAP-009 (dual anchor) |
| L2 Invariants | DI-002, DI-008 |
| Architecture Module | SS-10 (filled by architect) |
| Stories | S-5.03 |

## Related BCs

- BC-2.08.006 — composes with: `prism://sensors/health` is part of the resource registry covered by this BC
- BC-2.08.005 — depends on: `check_sensor_health` tool populates the health data exposed via `prism://sensors/health`

## Architecture Anchors

- `architecture/api-surface.md#configuration-state-resources` — `prism://config/clients` and `prism://config/clients/{client_id}/sensors` are Configuration State resources
- `architecture/api-surface.md#event-feed-resources` — `prism://sensors/health` is an Event Feed resource

## Story Anchor

S-5.03 — Resources and Prompts
