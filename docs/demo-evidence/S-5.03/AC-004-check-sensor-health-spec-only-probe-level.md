# AC-004 — `check_sensor_health` returns spec-only structured result

**AC:** AC-4 (BC-2.08.005 v1.7 postconditions 1, 4, 6, 7, 8; RECONCILIATION-3)
**Modality:** Test-execution transcript — MCP stdio server (Rust)
**Tests:**
- `test_BC_2_08_005_check_sensor_health_returns_structured_result` — structured content + trust_level + prose
- `test_BC_2_08_005_check_sensor_health_trust_level_is_internal` — `trust_level: "internal"` (unit)
- `test_BC_2_08_005_check_sensor_health_structured_content_shape` — all required fields present
- `test_BC_2_08_005_check_sensor_health_requires_client_id` — precondition: non-empty `client_id`
- `test_BC_2_08_005_check_sensor_health_returns_spec_only_probe_level` (server::tests) — `probe_level: "spec-only"`, `reachable: null`, `auth_valid: null`
**File:** `crates/prism-mcp/tests/resources.rs`, `crates/prism-mcp/src/server.rs`

---

## Scenario

`check_sensor_health(client_id: "acme")` called in S-5.03 scope (spec-only — no live probe).

Expected response:
- `probe_level: "spec-only"` (NOT "live")
- `reachable: null` (NOT hardcoded `true` — false-positive signal forbidden by BC-2.08.005 v1.7)
- `auth_valid: null` (NOT hardcoded `true`)
- `last_successful_query_at: null` (no query has run)
- `resource_pressure.active_cursor_count: null` (RECONCILIATION-3: not wired in S-5.03 scope)
- `resource_pressure.active_token_count: null` (NOT hardcoded `0` — `0` is misleading)
- `trust_level: "internal"` (health data is Prism-generated)
- Prose `content[].text` contains `"spec-only: no live probe performed"`
- `structuredContent` present with `sensors`, `resource_pressure`, `summary` fields

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_08_005_check_sensor_health)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.56s
────────────
 Nextest run ID 4f3a558e-921b-42d3-91c0-ca7dfca13903 with nextest profile: default
    Starting 5 tests across 8 binaries (239 tests skipped)
        PASS [   0.033s] (1/5) prism-mcp::resources test_BC_2_08_005_check_sensor_health_requires_client_id
        PASS [   0.033s] (2/5) prism-mcp::resources test_BC_2_08_005_check_sensor_health_trust_level_is_internal
        PASS [   0.034s] (3/5) prism-mcp::resources test_BC_2_08_005_check_sensor_health_structured_content_shape
        PASS [   0.052s] (4/5) prism-mcp::resources test_BC_2_08_005_check_sensor_health_returns_structured_result
        PASS [   0.056s] (5/5) prism-mcp server::tests::test_BC_2_08_005_check_sensor_health_returns_spec_only_probe_level
────────────
     Summary [   0.056s] 5 tests run: 5 passed, 239 skipped
```

## Assertions verified

- `structured_content` present in response (not `not_yet_available_msg`)
- `sc["trust_level"] == "internal"` (postcondition 7)
- `sc.get("sensors").is_some()` (postcondition 5)
- Prose contains `"spec-only: no live probe performed"` (postcondition 6)
- `probe_level == "spec-only"` (two-phase probe model)
- `reachable == null` (NOT `true` — hardcoded positive forbidden)
- `auth_valid == null` (NOT `true`)
- `resource_pressure.active_cursor_count == null` (RECONCILIATION-3)
- `resource_pressure.active_token_count == null` (NOT `0`)
- Empty `client_id` returns `INVALID_PARAMS` error (precondition enforced)

## Observed result

PASS — `check_sensor_health` returns correct spec-only shape; no hardcoded positives; resource_pressure null-encoded.
