# AC-002 — `prism://config/clients/{client_id}/sensors` — per-client filter + URL strip

**AC:** AC-2 (BC-2.10.008 v1.12 postcondition 2; DI-008; VP-050)
**Modality:** Test-execution transcript — MCP stdio server (Rust)
**Tests:**
- `test_BC_2_10_008_client_sensors_acme_does_not_include_globex_sensors` — DI-008 isolation + URL strip
- `test_BC_2_10_008_client_sensors_invalid_id_returns_error` — EC-001 path-traversal rejection
**File:** `crates/prism-mcp/tests/resources.rs`

---

## Scenario A — DI-008 per-client filter

Config has three sensors: crowdstrike, claroty, armis. Request `client_id="crowdstrike"`.
- ONLY crowdstrike must appear (not claroty or armis).
- `api_base_url` field for crowdstrike must contain `"https://api.crowdstrike.com"` — full URL
  `"https://api.crowdstrike.com/path/that/must/be/stripped?key=secret"` stripped to host+port only.
- Stale field names `sensor_id`, `enabled` (boolean), `configured_sources` must not appear.
- Each entry uses canonical fields: `sensor_type`, `status`, `credential_ref`, `sources`, `api_base_url`.

## Scenario B — EC-001 path traversal

`client_id = "../../etc/passwd"` must be rejected by `OrgSlug::new()` before any CF scan.
Error message must NOT echo the raw path-traversal string (DI-006 prompt-injection defense).

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_10_008_client_sensors)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.49s
────────────
 Nextest run ID 6a8e5ef6-ab91-49c4-8422-d664766d6ebd with nextest profile: default
    Starting 2 tests across 8 binaries (242 tests skipped)
        PASS [   0.034s] (1/2) prism-mcp::resources test_BC_2_10_008_client_sensors_acme_does_not_include_globex_sensors
        PASS [   0.034s] (2/2) prism-mcp::resources test_BC_2_10_008_client_sensors_invalid_id_returns_error
────────────
     Summary [   0.034s] 2 tests run: 2 passed, 242 skipped
```

## Assertions verified

Scenario A:
- `entries.len() == 1` (only crowdstrike returned — FAILS if filter is broken: returns 3)
- `sensor_types.contains("crowdstrike")` — correct sensor present
- `!sensor_types.contains("claroty")` — DI-008: cross-org leak prevented
- `!sensor_types.contains("armis")` — DI-008: cross-org leak prevented
- `api_base_url` does not contain `/path/`, `/v1/`, `/api/`, `?`, `secret`, `key=`
- `api_base_url` is `"https://api.crowdstrike.com"` (scheme+host only)

Scenario B:
- `result.is_err()` — path traversal rejected
- `err_msg` does NOT contain `"../../etc/passwd"` (no prompt-injection echo)
- `err_msg` contains `"invalid"` or `"not found"`

## Observed result

PASS — per-client isolation enforced; URL stripped to host+port; path traversal rejected cleanly.
