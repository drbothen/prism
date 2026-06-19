# AC-007 — VP-050 proptest: no API key patterns, no full URL paths in sensor resource

**AC:** AC-7 (VP-050; BC-2.10.008 postconditions)
**Modality:** Test-execution transcript — proptest (Rust)
**Tests:**
- `prop_vp050_uuid_credential_redacted` — UUID-format API keys replaced with `[REDACTED]`
- `prop_vp050_bearer_credential_redacted` — Bearer token prefix stripped
- `prop_vp050_url_stripped_to_host_port` — full URL paths stripped to scheme+host+port
- `test_vp050_strip_url_to_host_port_strips_userinfo` — RFC 3986 userinfo (user:pass@) stripped
**File:** `crates/prism-mcp/src/proofs/sensor_resource_redaction.rs`

---

## Scenario

Proptest generates `SensorConfigEntry` payloads with:
- Fabricated API keys in UUID format (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)
- Bearer token prefix credentials (`Bearer <16+ char token>`)
- Full URL paths with paths, query strings, and embedded credentials

Asserts that `render_sensor_inventory_resource` output:
- Contains no API key pattern matches
- Contains only `scheme+host+port` in `api_base_url`, never full paths or query strings

## Command

```
cargo nextest run -p prism-mcp -E 'test(vp_050) or test(prop_vp050)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.63s
────────────
 Nextest run ID bc4ba92d-8b63-4502-8238-be75c6ffc09f with nextest profile: default
    Starting 4 tests across 8 binaries (243 tests skipped)
        PASS [   0.034s] (1/4) prism-mcp proofs::sensor_resource_redaction::vp_050_tests::test_vp050_strip_url_to_host_port_strips_userinfo
        PASS [   0.066s] (2/4) prism-mcp proofs::sensor_resource_redaction::vp_050_tests::prop_vp050_bearer_credential_redacted
        PASS [   0.071s] (3/4) prism-mcp proofs::sensor_resource_redaction::vp_050_tests::prop_vp050_uuid_credential_redacted
        PASS [   0.084s] (4/4) prism-mcp proofs::sensor_resource_redaction::vp_050_tests::prop_vp050_url_stripped_to_host_port
────────────
     Summary [   0.084s] 4 tests run: 4 passed, 243 skipped
```

## Assertions verified

- UUID-format credentials replaced with `[REDACTED]` (not passed through to API response)
- Bearer token credentials replaced with `[REDACTED]`
- Full URL `https://api.crowdstrike.com/path?key=secret` → `https://api.crowdstrike.com`
- Userinfo `https://user:secret@host:443/path` → `https://host:443` (RFC 3986 §3.2.1)
- `api_base_url` never contains `/`, `?`, or `=` after the host component

## Observed result

PASS — VP-050 proptest confirms `render_sensor_inventory_resource` produces no API key patterns and no full URL paths.
