---
document_type: ops-runbook
title: Live Claroty xDome Tenant Validation — Per-Story Merge Gate
version: "1.0"
producer: state-manager
created: 2026-08-30
status: active
portability: project-specific (prism + test-soc live-soc)
changelog:
  - version: "1.0"
    date: 2026-08-30
    author: state-manager
    summary: Initial creation. Documents per-story live-tenant merge gate, variant procedures (Variant-1 cargo tests, Variant-2 MCP-driven), prerequisites, and live-tenant caveats. Origin D-2310 lesson + D-2367 deadline directive.
---

# Live Claroty xDome Tenant Validation — Per-Story Merge Gate (Runbook)

## 1. Why This Exists

The DTU story-level holdout gate FALSE-GREENS on thin DTU fixtures (lesson D-2310). Therefore every Claroty xDome sensor story (G1 S-CLAROTY-VULNS-001 + G2–G6) REQUIRES a live-tenant validation pass BEFORE merge (D-2367 deadline directive; xdome-endpoint-expansion-plan.md per-story pipeline: "LOCAL adversary 3-CLEAN → live validation on monroe (Variant-1 structural + optional Variant-2 agent)"). This runbook is the canonical procedure so it is not re-discovered each session.

## 2. Canonical External Source

`/Users/jmagady/Dev/test-soc/` (the Prism SOC live/demo environment repo) — specifically `/Users/jmagady/Dev/test-soc/live-soc/README.md` (Path B — REAL client pilot). `test-soc` also contains `prism-live-mcp-wrapper.sh` (live MCP server), `setup-client.sh`, and `clients/_template.env`.

## 3. Prerequisites (Human, One-Time Per Live Client; AD-017 — Secrets Never Enter AI Context)

Onboard the live client in `/Users/jmagady/Dev/test-soc/live-soc`:

1. `cp clients/_template.env clients/<client-id>.env`
2. `chmod 600 clients/<client-id>.env` — fill in tenant base URL + API token
3. `./setup-client.sh <client-id> claroty` — `claroty` = Claroty xDome spec

Secrets land in the OS keyring (`prism/<client>/<sensor>/<name>`); the env file is transport-only and the script offers to delete it. Values never appear in AI prompts, transcripts, or logs.

**Credential model:** keyring `prism/<client>/claroty/bearer_token`; env fallback `PRISM_CLIENTS_<CLIENT>_SENSORS_CLAROTY_<NAME>` (hyphens→underscores). "NO RESET CONCEPT — never wipe live state" for Path B.

## 4. Variant-2 (Agent / MCP-Driven, Read-Only) — PRIMARY Validation

Connect the `prism-live` MCP server to the Claude Code session (open Claude Code in `/Users/jmagady/Dev/test-soc` so `.mcp.json` loads it, OR register `prism-live` → `/Users/jmagady/Dev/test-soc/prism-live-mcp-wrapper.sh` in settings.json; reconnect MCP). Then run the read-only gate against the live client:

1. Read resource `prism://config/clients` — confirm the live client is registered with claroty enabled (there is NO `default` client; never guess a client_id).
2. `check_sensor_health` for `<client-id>` — claroty adapter must be healthy (`E-SENSOR-030` = upstream credential/connectivity failure, not a prism bug).
3. `prism_describe <client-id>` — confirm the story's new table + columns are present in the live schema.
4. Smoke `query` scoped `clients: [<client-id>]` for the story's table, e.g. `SELECT * FROM claroty_vulnerabilities LIMIT 5` (or pipe `FROM claroty_vulnerabilities | limit 5`). Confirm `_source_type: "live"`, correct wire-level shape (`class_uid`, expected columns, `raw_extensions`), and that the new columns resolve on REAL tenant data.

## 5. Variant-1 (Structural / Cargo `#[ignore]` Integration Tests) — Secondary

When a raw URL + token is available to the test process (AD-017 credential model):

Set `CLAROTY_INSTANCE_URL` to the tenant base URL (auth via keyring/env per AD-017); run the story's ignored live tests:

```bash
cargo nextest run -p prism-sensors -E 'test(BC_2_16_015)' --run-ignored ignored-only
```

For example, RG-004 wire-shape and RG-005 `raw_extensions` tests. These tests self-skip (`#[ignore]`) unless `CLAROTY_INSTANCE_URL` is set.

## 6. Merge Gate

A Claroty xDome story does NOT merge until its live-tenant validation passes (Variant-2 at minimum). Record the evidence — client_id, table, row count, wire-shape confirmation, `_source_type: live` — in the story's demo/live-evidence directory and reference it in the merge decision.

## 7. Known Live-Tenant Caveats

From `live-soc/README.md` §Known limitations:

1. Spec column grammar lacks array types → xDome `ip_list`/`mac_list` (+ some os/model fields) not queryable.
2. xDome alerts have no severity field (candidate `alert_class` vocabulary still needs live validation; reference fixtures are contradictory).

Raise new fidelity gaps with the Prism team; first real-API contact IS the fidelity test.
