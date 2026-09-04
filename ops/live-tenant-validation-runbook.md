---
document_type: ops-runbook
title: Live Sensor Tenant Validation — Per-Story Merge Gate
version: "2.0"
producer: devops-engineer
created: 2026-08-30
status: active
portability: project-specific (prism + test-soc live-soc)
changelog:
  - version: "2.0"
    date: 2026-09-04
    author: devops-engineer
    summary: >
      Generalized from Claroty-only to all Prism sensors (claroty, crowdstrike, cyberint, armis).
      Added §4.5 (driver script live-validate-sensor.py) with concrete invocation examples.
      Added §8 (MCP wire-format reference) from RC1 validation lessons. Documented correct
      table-name extraction from TOML spec, client config parsing, parallel query approach,
      and LRG_TBL vs FAIL distinction. Runbook now covers full-sensor audit, not just per-story.
      Validated 2026-09-04 against monroe/claroty RC1 (14 tables, PASS).
  - version: "1.0"
    date: 2026-08-30
    author: state-manager
    summary: Initial creation. Documents per-story live-tenant merge gate, variant procedures (Variant-1 cargo tests, Variant-2 MCP-driven), prerequisites, and live-tenant caveats. Origin D-2310 lesson + D-2367 deadline directive.
---

# Live Sensor Tenant Validation — Per-Story Merge Gate (Runbook)

## 1. Why This Exists

The DTU story-level holdout gate FALSE-GREENS on thin DTU fixtures (lesson D-2310). Therefore every sensor story REQUIRES a live-tenant validation pass BEFORE merge (D-2367 deadline directive). This runbook is the canonical, repeatable procedure so it is not re-discovered each session. It covers all Prism sensor sources: `claroty`, `crowdstrike`, `cyberint`, `armis`.

**Full-sensor audits** (e.g., at release gates or after adding a new client) use the same procedure against all tables, not just the story's new table.

## 2. Canonical External Source

`/Users/jmagady/Dev/test-soc/` — specifically:
- `live-soc/README.md` — Path B (REAL client pilot) overview and known limitations
- `prism-live-mcp-wrapper.sh` — live MCP server launcher
- `live-validate-sensor.py` — reusable Python validation driver (§4.5)
- `live-soc/setup-client.sh` — one-time client onboarding
- `live-soc/clients/_template.env` — credential template

## 3. Prerequisites (Human, One-Time Per Live Client; AD-017 — Secrets Never Enter AI Context)

Onboard the live client in `/Users/jmagady/Dev/test-soc/live-soc`:

```bash
cp clients/_template.env clients/<client-id>.env
chmod 600 clients/<client-id>.env   # fill in tenant base URL + API token
./setup-client.sh <client-id> <sensor-id>  # e.g. claroty, crowdstrike, armis
```

Secrets land in the OS keyring (`prism/<client>/<sensor>/<name>`); the env file is transport-only, the script offers to delete it afterward. Values NEVER appear in AI prompts, transcripts, or logs (AD-017).

**Credential model:** keyring key `prism/<client>/<sensor>/bearer_token`; env-var fallback
`PRISM_CLIENTS_<CLIENT>_SENSORS_<SENSOR>_<NAME>` (hyphens→underscores).

**"NO RESET" rule:** Never run `setup-client.sh` again for an already-onboarded client — it would wipe live credentials. Never delete or overwrite `live-soc/<client-id>/` state.

## 4. Variant-2 (Python MCP Driver, Read-Only) — PRIMARY Validation

### 4.1 Quick Invocation

```bash
# Validate all tables for a single sensor against a live client
python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
  --client monroe --sensor claroty

# Other sensors (same command, change --sensor)
python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
  --client acme --sensor crowdstrike

python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
  --client acme --sensor armis

python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
  --client acme --sensor cyberint

# Full audit of ALL sensors for a client (run once per sensor)
for sensor in claroty crowdstrike armis cyberint; do
  python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
    --client acme --sensor "$sensor"
done
```

Options: `--limit N` (rows per table, default 3), `--timeout N` (seconds per query, default 45), `--spec PATH` (override spec auto-detection), `--wrapper PATH` (override MCP wrapper).

### 4.2 What the Script Checks

Four sequential checks (all required for PASS):

1. **Client registration** — reads `prism://config/clients`, confirms `<client-id>` is present and `<sensor-id>` is in `enabled_sensors`.
2. **Sensor health** — calls `check_sensor_health{client_id}`, expects `overall_status: healthy`. `E-SENSOR-030` = upstream credential/connectivity failure (not a prism bug).
3. **Schema completeness** — calls `prism_describe{client_id}`, confirms every table from the sensor's TOML spec is present in the live schema with the correct column count.
4. **Smoke queries** — runs `FROM <table> | limit N` for every table in parallel. Confirms `_source_type: "live"`, `class_uid` present, `raw_extensions` present.

### 4.3 Status Codes

| Code | Meaning | Blocks PASS? |
|------|---------|-------------|
| `PASS` | Query succeeded (0 rows is valid — empty table, not an error) | No |
| `LRG_TBL` | Prism-side timeout: table is in schema but has high-volume data | No |
| `KNW_LIM` | Known spec limitation (e.g., array columns not queryable) | No |
| `TIMEOUT` | Client-side script timeout (increase `--timeout`) | Investigate |
| `NO_TBL` | E-QUERY-037: table not found in schema (spec/deployment mismatch) | Yes |
| `E_030` | E-SENSOR-030: upstream connectivity or auth failure | Yes |
| `FAIL` | Unexpected error | Yes |

### 4.4 Evidence Output

The script writes structural evidence (D-2410 compliant — no live row values, no credentials) to:
```
/Users/jmagady/Dev/test-soc/live-soc/<client-id>/state/live-validation-<sensor-id>.json
```

### 4.5 How Table Names Are Derived (Critical)

**ALWAYS derive expected table names from the TOML spec, not from story names.**

The qualified PrismQL name for each table is: `{sensor_id}_{table_name}` where
`table_name` comes from each `[[tables]]` block's `table_name = "..."` field in the spec.

```bash
# Extract table names manually for any sensor spec:
grep '^table_name\s*=' /path/to/<sensor>.sensor.toml | sed 's/table_name\s*=\s*"//; s/"$//' \
  | while read t; do echo "<sensor_id>_${t}"; done
```

Story names (e.g., `S-CLAROTY-ALERTS-001`) do NOT map 1:1 to table names. Story names reflect
the implementation unit; table names come from the API spec. Using story names as table names
produces false E-QUERY-037 failures.

## 5. Variant-1 (Structural / Cargo `#[ignore]` Integration Tests) — Secondary

When a raw URL + token is available to the test process (AD-017 credential model):

Set `<SENSOR>_INSTANCE_URL` (or `<SENSOR>_BASE_URL`) to the tenant base URL; run the sensor story's ignored live tests:

```bash
# Claroty xDome example
CLAROTY_INSTANCE_URL=https://api.claroty.com \
  cargo nextest run -p prism-sensors -E 'test(BC_2_16_015)' --run-ignored ignored-only

# General pattern for any sensor's live tests
<SENSOR>_INSTANCE_URL=<URL> \
  cargo nextest run -p prism-sensors -E 'test(<test_filter>)' --run-ignored ignored-only
```

Tests self-skip unless the env var is set. This variant is secondary to Variant-2; use it when
Variant-2 infrastructure is unavailable or for targeted per-AC wire-shape assertions.

## 6. Per-Story Merge Gate

A sensor story does NOT merge until its live-tenant validation passes (Variant-2 at minimum).
Record the evidence path in the PR description and reference it in the merge decision:

```
Live validation: PASS (2026-09-04)
Evidence: /Users/jmagady/Dev/test-soc/live-soc/monroe/state/live-validation-claroty.json
```

## 7. Full-Sensor Audit (Release Gate / New Client Onboarding)

When deploying a new binary version or onboarding a new live client, run the full-sensor audit
against all configured sensors. This is the RC/release gate process:

```bash
# Step 1: Build and deploy new binary
cargo build --release -p prism-bin
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
cp /Users/jmagady/Dev/test-soc/bin/prism \
   /Users/jmagady/Dev/test-soc/bin/prism.bak.pre-<label>-${TIMESTAMP}
cp /Users/jmagady/Dev/prism/target/release/prism \
   /Users/jmagady/Dev/test-soc/bin/prism
/Users/jmagady/Dev/test-soc/bin/prism --version   # confirm version string

# Step 2: Refresh sensor specs (for each sensor that changed)
cp /Users/jmagady/Dev/prism/crates/prism-sensors/specs/<sensor>.sensor.toml \
   /Users/jmagady/Dev/test-soc/.prism-live/specs/
grep -c '^\[\[tables\]\]' /Users/jmagady/Dev/test-soc/.prism-live/specs/<sensor>.sensor.toml

# Step 3: Run validation for each sensor × client pair
python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py --client monroe --sensor claroty
# repeat for other sensors / clients
```

Do NOT run `setup-client.sh` as part of this process — that would wipe live credentials.
Do NOT overwrite customer overlay TOMLs under `.prism-live/specs/customers/`.

## 8. MCP Wire-Format Reference (Lessons from RC1 Validation, 2026-09-04)

These notes capture concrete protocol details discovered during the RC1 validation run.
They are here so future sessions do not re-discover them.

### Transport
- rmcp 1.7.0 (MCP 2024-11-05 protocol) uses **newline-delimited JSON** over stdio.
- Each message is one JSON object followed by `\n`. No Content-Length framing (that is LSP).
- Initialize → notifications/initialized handshake required before tool/resource calls.

### `prism://config/clients` resource
- Returns a JSON array: `[{"client_id":"monroe","enabled_sensors":["claroty"],"sensor_count":1}]`
- Check `enabled_sensors` (array of sensor_id strings) — NOT `sensors` (different field, not present here).

### `check_sensor_health` response
- Prefer `structuredContent` path: `result.structuredContent.overall_status` = `"healthy"` or `"unhealthy"`.
- Per-sensor detail: `result.structuredContent.sensors[i]` with `reachable`, `auth_valid`, `probe_level`, `latency_ms`, `error`.
- `E-SENSOR-030` in `isError=true` text → upstream credential or connectivity failure. Not a prism bug.
  Check keyring entry `prism/<client>/<sensor>/bearer_token`.

### `prism_describe` response
- Tables are under `result.structuredContent.results.tables[]` (or same path in text JSON).
- Each entry: `{"name": "claroty_alerts", "columns": [...], "description": "...", "sensor_type": "..."}`.
- `_meta.total_results: 0` in the describe envelope is the QUERY result count, NOT the table count. Do not confuse.
- `result.isError = false` and an empty rows array = valid (tenant has no data in that table).

### `query` response
- Prefer `result.structuredContent.results.rows[]` for parsed rows.
- Fallback: parse `result.content[0].text` as JSON, then `.results.rows[]`.
- `result.isError = false` + `rows = []` + `total_results = 0` → table exists, tenant has no matching data. Status: PASS.
- `result.isError = true` with `[transient]` or `timeout` in text → prism-side query timeout. Table IS in schema but has high-volume data. Status: LRG_TBL (not blocking).
- `result.isError = true` with `E-QUERY-037` → table not in schema. Status: NO_TBL (blocking).

### MCP serverInfo version
- `result.serverInfo.version` is hardcoded `"0.1.0"` in the prism MCP source regardless of binary version.
- To confirm the actual binary version: `bin/prism --version` outside the MCP protocol.

### Parallel query approach
- 5 parallel workers (ThreadPoolExecutor) with 45s per-query timeout avoids the 14–28 minute wall time
  of sequential queries against a live API. Adjust `--timeout` if the tenant has very large tables.
- Do not exceed 10 workers — Claroty xDome API may rate-limit concurrent requests.

## 9. Known Live-Tenant Caveats (Per Sensor)

### Claroty xDome (`claroty`)
1. `ip_list`/`mac_list` on `devices` and `server_interfaces` are array-type columns — not queryable (spec grammar lacks array types). Status: KNW_LIM.
2. `claroty_alerts` has no severity field; `alert_class` appears in `raw_extensions` (confirmed RC1 2026-09-04).
3. `claroty_vulnerabilities` returns LRG_TBL on tenants with large CVE datasets (prism-side timeout). Table is present and structurally correct.
4. `claroty_ot_activity_events` and `claroty_organization_acl_policies` may return 0 rows on tenants that do not use those features. Status: PASS (0 rows is valid).

### CrowdStrike (`crowdstrike`)
- Not yet validated against a live tenant. Update this section after first live run.

### Armis (`armis`)
- Not yet validated against a live tenant. Update this section after first live run.

### Cyberint (`cyberint`)
- Not yet validated against a live tenant. Update this section after first live run.

Raise new fidelity gaps with the Prism team; first real-API contact IS the fidelity test.
