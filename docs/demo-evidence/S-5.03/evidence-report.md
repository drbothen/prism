# Evidence Report — S-5.03

**Story:** prism-mcp: Resources and Prompts
**Story version:** v1.22
**Story ID:** S-5.03
**BCs:** BC-2.10.008 v1.12 · BC-2.10.009 · BC-2.08.005 v1.7 · BC-2.08.006 v1.6 · BC-2.16.007 v1.7
**VP:** VP-050
**Product type:** MCP server (Rust library, no UI) — TEST-EXECUTION evidence per POL-10
**Code under test:** feature/S-5.03 (HEAD 54a473a4); based on develop@60249ccc (S-3.13 merge)
**Evidence recorded:** 2026-06-18
**Test command:** `cargo nextest run -p prism-mcp`
**Result:** 247 tests run — 247 passed, 0 skipped, 0 failed

---

## Coverage Map: Acceptance Criteria → Tests → Status

| AC | Description (abbreviated) | Primary Test(s) | Secondary Tests | Evidence Artifact | Status |
|----|---------------------------|-----------------|-----------------|-------------------|--------|
| AC-1 | `prism://config/clients` returns JSON array; `display_name` from `OrgEntry.name`; "acme"→`"Acme Corp"`, "globex"→`null`; `capabilities_summary` absent | `test_BC_2_10_008_config_clients_returns_all_clients` | `test_BC_2_10_008_invariant_zero_clients_returns_empty_array` | `AC-001-config-clients-returns-all-clients.md` | PASS |
| AC-2 | `prism://config/clients/acme/sensors` returns ONLY acme sensors (DI-008 isolation); `api_base_url` stripped to scheme+host+port; canonical field set; path traversal rejected (EC-001) | `test_BC_2_10_008_client_sensors_acme_does_not_include_globex_sensors` · `test_BC_2_10_008_client_sensors_invalid_id_returns_error` | `test_BC_2_10_008_per_org_scoping_acme_has_crowdstrike_and_claroty_not_armis` · `test_BC_2_10_008_per_org_scoping_globex_has_armis_not_acme_sensors` · `test_BC_2_10_008_ec_10_017_org_with_no_overlay_returns_empty_sensors` | `AC-002-client-sensors-per-client-filter-and-url-strip.md` | PASS |
| AC-3 | `prompts/list` includes 4 mandated prompts: `triage_alerts`, `investigate_host`, `client_overview`, `cross_client_status`; all prompt messages contain DI-006 security reminder ("untrusted") | `test_BC_2_10_009_prompts_list_includes_four_mandated_prompts` · `test_BC_2_10_009_triage_alerts_includes_security_reminder` | `test_BC_2_10_009_investigate_host_includes_security_reminder` · `test_BC_2_10_009_client_overview_includes_security_reminder` · `test_BC_2_10_009_cross_client_status_includes_security_reminder` · `test_BC_2_10_009_invalid_prompt_name_returns_error` | `AC-003-prompts-list-four-mandated-di006-reminder.md` | PASS |
| AC-4 | `check_sensor_health(client_id: "acme")` spec-only: `probe_level: "spec-only"`, `reachable: null`, `auth_valid: null`, `resource_pressure` null-encoded; prose contains `"spec-only: no live probe performed"`; `trust_level: "internal"`; `structuredContent` present | `test_BC_2_08_005_check_sensor_health_returns_structured_result` · `test_BC_2_08_005_check_sensor_health_returns_spec_only_probe_level` (server::tests) | `test_BC_2_08_005_check_sensor_health_trust_level_is_internal` · `test_BC_2_08_005_check_sensor_health_structured_content_shape` · `test_BC_2_08_005_check_sensor_health_requires_client_id` | `AC-004-check-sensor-health-spec-only-probe-level.md` | PASS |
| AC-5 | After `check_sensor_health` run, `prism://sensors/health` returns cached data in keyed-object schema `{ clients: { [client_id]: { sensors: { [sensor_id]: SensorHealthResult } } } }` | `test_BC_2_08_006_sensors_health_resource_returns_cached_data` | `test_BC_2_08_006_sensors_health_resource_keyed_object_shape` | `AC-005-sensors-health-returns-cached-keyed-object.md` | PASS |
| AC-6 | `prism://sensors/health` before any `check_sensor_health`: returns sentinel `{ "status": "unknown", "message": "Run check_sensor_health..." }` — NOT an error | `test_BC_2_08_006_sensors_health_resource_returns_unknown_before_check` | `test_BC_2_08_006_sensors_health_zero_clients_returns_unknown_sentinel` | `AC-006-sensors-health-unknown-sentinel-before-check.md` | PASS |
| AC-7 | VP-050 proptest: `render_sensor_inventory_resource` output contains no API key patterns, no full URL paths — only scheme+host+port | `prop_vp050_uuid_credential_redacted` · `prop_vp050_bearer_credential_redacted` · `prop_vp050_url_stripped_to_host_port` | `test_vp050_strip_url_to_host_port_strips_userinfo` | `AC-007-vp050-proptest-no-api-keys-no-full-urls.md` | PASS |
| AC-8 | `prism://config/clients` lists ONLY sensors in `TableRegistry.registered_sensor_ids()`; sensors absent from TableRegistry excluded; `enabled_sensors` carries sensor IDs not table names | `test_BC_2_10_008_config_clients_resource_reflects_registered_tables` | `test_BC_2_10_008_config_clients_returns_all_clients` (AC-1, uses TableRegistry intersection) | `AC-008-config-clients-reflects-table-registry.md` | PASS |
| AC-9 | Hot-reload dispatches BOTH `notifications/resources/list_changed` AND `notifications/tools/list_changed` when table-name set changes; dispatches NEITHER when table set unchanged (table-set-delta gate BC-2.16.007 v1.7) | `test_BC_2_16_007_hot_reload_sends_mcp_list_changed_notification` | `test_BC_2_16_007_reload_config_wires_dispatch_hot_reload_notifications` (server::tests — wiring) | `AC-009-hot-reload-notifications-table-set-delta-gate.md` | PASS |

---

## Full test run

All 247 `prism-mcp` tests pass on HEAD 54a473a4:

```
cargo nextest run -p prism-mcp
────────────
 Nextest run ID 0664605f-1152-499f-975d-3d46f8ca1dfa with nextest profile: default
    Starting 247 tests across 8 binaries
    ... [all PASS] ...
────────────
     Summary [   0.951s] 247 tests run: 247 passed, 0 skipped
```

---

## Key Assertion Details (load-bearing tests per BC)

### AC-1 — `display_name` from `OrgEntry.name` (BC-2.10.008 v1.11)

The `render_client_list_resource` per-org path reads `org_display_names` from the config snapshot,
sourced from `[[orgs]].name` in `prism.toml`. The fallback path (no org_registry) produces
`display_name: null` for all entries (no TOML org context in test mode). Both paths serialize
correctly to JSON `null` when the name is absent.

### AC-2 — DI-008 per-client filter (BC-2.10.008 v1.8 postcondition 2)

The load-bearing assertion `entries.len() == 1` for `client_id="crowdstrike"` FAILS if:
- The filter is broken and returns all 3 sensors (len=3)
- The `sensor_id == client_id` stopgap is in place and returns nothing for multi-tenant orgs (len=0)

The per-org production path uses `resolved_spec_map.filter(|(org,_), _| org == client_id)`.
The single-tenant fallback path uses `sensor_specs.values().filter(|s| s.sensor_id == client_id)`.

### AC-4 — RECONCILIATION-3: `ResourcePressure` null-encoded (BC-2.08.005 v1.7)

`ResourcePressure::new(None, None)` in S-5.03 scope — both counts serialize as JSON `null`.
`ResourcePressure::new(0, 0)` is FORBIDDEN: an AI consumer cannot distinguish hardcoded zero
from a genuine empty queue. S-5.04 obligation: wire live counts via `QueryEngine::cursor_count()`
and `QueryEngine::token_count()`.

### AC-5 — Keyed-object `sensors` schema (BC-2.08.006 v1.5 postcondition 2)

The prior implementation emitted `"sensors": [array]` (array, not object). S-5.03 corrects this
to `BTreeMap<sensor_id, SensorHealthResult>` which serializes as a JSON object keyed by sensor ID.
AI consumers look up sensors directly by ID — array scanning is O(n) and fragile.

### AC-9 — Table-set-delta gate (BC-2.16.007 v1.7)

`dispatch_hot_reload_notifications` uses `BTreeSet` comparison: fires only when `old_set != new_set`.
Column-only changes (same table names, different column definitions) do NOT trigger notifications.
Non-schema attribute changes (rate limits, pagination) do NOT trigger notifications.
Column-delta notification is deferred to S-5.11 per spec.

---

## Adversarial-convergence tests (PR-level cascade findings closed)

These tests were added during the LOCAL adversarial cascade (passes 1–n). All pass on HEAD 54a473a4.

| Test | Finding closed | Key assertion |
|------|----------------|---------------|
| `test_BC_2_10_008_schema_resource_path_traversal_not_echoed_in_error` | F-B/DI-006 schema handler path-traversal echo | `!err_msg.contains("../../etc/passwd")` — raw payload not echoed |
| `test_BC_2_10_008_per_org_scoping_acme_has_crowdstrike_and_claroty_not_armis` | IMP-8 DI-008 per-org scoping (CRIT) | `entries.len() == 2`, no armis for acme |
| `test_BC_2_10_008_per_org_scoping_globex_has_armis_not_acme_sensors` | IMP-8 DI-008 cross-org isolation | `entries.len() == 1`, armis only for globex |
| `test_BC_2_10_008_ec_10_017_org_with_no_overlay_returns_empty_sensors` | IMP-8 EC-10-017 Option B semantics | `content_text.trim() == "[]"` for zero-overlay org |
| `test_BC_2_10_008_per_org_list_resource_enumerates_all_orgs_with_correct_counts` | IMP-8 prism://config/clients per-org list | acme→2 sensors, globex→1, empty-org→0 |
| `test_BC_2_10_008_invariant_zero_clients_returns_empty_array` | EC-10-014 synthetic "(all)" entry removed | `content_text.trim() == "[]"` |
| `test_BC_2_08_006_sensors_health_resource_keyed_object_shape` | Keyed-object `sensors` schema (array → object) | `parsed["clients"]["acme"]["sensors"].is_object()` |
| `test_OBS_1_prompt_render_rejects_injection_shaped_args` | OBS-1 prompt argument DI-006 validation | injection-shaped `client_id` returns `Err(INVALID_PARAMS)` |
| `test_LOW_3_unknown_client_rejected_in_co_wired_mode` | LOW-3 unknown org in wired mode | `org_registry` rejects unknown org with 404 |
| `test_SEC_001_dispatch_read_resource_unknown_uri_does_not_echo_uri` | SEC fix-burst: DI-006 URI non-echo hardening | unknown-URI error does not echo the raw request URI back to caller |
| `test_SEC_002_validate_hostname_rejects_metacharacters_accepts_normal_hosts` | SEC fix-burst: hostname metacharacter rejection | `validate_hostname` rejects `; rm -rf`, accepts `acme-corp.example.com` |
| `test_SEC_003_display_name_sanitized_before_ai_context` | SEC fix-burst: display_name AI-context sanitization | display names with injection patterns sanitized before inclusion in MCP output |

---

## Notes

- AC-10 (`prism://diagnostics/summary unregistered_table_queries`) was formally relocated to
  S-5.08 as S-5.08 AC-8 via PO adjudication v1.15 (2026-06-17). S-5.03 has no obligation for AC-10.
- The VHS / Playwright recording modality is not applicable: `prism-mcp` is a Rust MCP stdio
  server library with no CLI output or UI. Test-execution transcripts are the correct evidence
  modality per POL-10 for server/library products. All 9 ACs are covered by executable tests
  that exercise the actual production code paths.
- S-3.13 is a hard prerequisite for AC-8 and AC-9 (provides `TableRegistry::registered_sensor_ids()`
  and `registered_tables()` APIs). S-3.13 IS merged: develop@60249ccc is the S-3.13 merge commit.
  The S-5.03 feature branch is based on this commit.
