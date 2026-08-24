---
title: "Prism LIVE Claroty xDome — Operations & Validation Runbook"
document_type: runbook
version: "1.0"
created: 2026-08-24
---

# Prism LIVE Claroty xDome — Operations & Validation Runbook

## Deployment

- **Location:** `/Users/jmagady/Dev/test-soc/`
- **Config dir:** `.prism-live`
- **Binary:** `bin/prism`
- **Client:** `monroe` (org_id `019f80f5-d021-7dfd-bc60-561409449c1e`)
- **Sensor:** `claroty@monroe` → `https://api.claroty.com` (DIRECT HTTPS, PR #237; the live-soc/relay `xdome-relay.py` is LEGACY/unused)

## Keep the Binary Current

```bash
cargo build --release -p prism-bin   # from develop HEAD
cp target/release/prism test-soc/bin/prism  # back up first
```

D-1882: macOS Keychain ACL is keyed to the writing binary. If credential reads fail post-swap (E-SENSOR-030 / AllTargetsFailed), the OPERATOR re-stores the Claroty token via the deployed binary (AD-017: values never enter AI context).

Current deployed binary built from develop `3f1e66179`.

## Keep the Spec Current

```bash
cp crates/prism-sensors/specs/claroty.sensor.toml test-soc/.prism-live/specs/claroty.sensor.toml
# OR:
live-soc/setup-client.sh monroe claroty
```

NOTE: overwriting live config requires explicit operator authorization (classifier gate). No hot-reload (S-1.12-FOLLOWUP) — relaunch prism to pick up spec changes.

## Launch

```bash
test-soc/prism-live-mcp-wrapper.sh
# Sets env, exec `bin/prism --config-dir .prism-live start`, MCP stdio
# OR via test-soc/.mcp.json server `prism-live`
```

## MCP Read-Only Surface

| Tool / Resource | Notes |
|----------------|-------|
| ReadMcpResource `prism://config/clients` | Client discovery — never guess client_id |
| `prism_describe {client}` | Schema discovery |
| `query {clients:["monroe"], <PrismQL>, limit≤1000}` | SQL or pipe mode; enums OCSF Title-case; use IEQ/IIN; `\| enrich`; `force_refresh` to bypass cache |
| `check_sensor_health` | Health check |
| `list_capabilities` | Capability listing |
| `explain_query` | Query explanation |
| Resource `prismql://reference` | PrismQL syntax reference |

Action / case / rule / containment tools NOT registered (-32003) — the loop is READ-ONLY.

`_source_type:"live"` ⇒ trust `untrusted_external`.

## Variant 1 — Direct MCP Structural Validation

Drive prism-live MCP over stdio. Per table assert wire shapes:
- Every Tier-1 col resolves + Arrow type
- Every Tier-2 key in `raw_extensions`
- Synthesized `class_uid` / `_sensor` fields
- Endpoints (trailing-slash, pagination, audit_log push-down `filter_by`)
- Coercion (polymorphic ID→String, datetime, JSON-list-string)
- E-QUERY-038 absent
- E-SPEC-018 absent
- TLS direct

## Variant 2 — Agent-in-the-Loop (tmux-cli)

```bash
tmux-cli launch "zsh"
tmux-cli send "cd /Users/jmagady/Dev/test-soc && claude" --pane=<id>
```

The nested claude shows an MCP-server approval + trust gate. **The AI harness CLASSIFIER BLOCKS an AI from auto-answering these — the OPERATOR must approve them** (enable `prism-live`).

Make it visible:
```bash
osascript -e 'tell application "Terminal" to do script "tmux attach -t remote-cli-session"'
# OR:
tmux attach -t remote-cli-session
```

Once approved, the session is auto-accept. Drive analyst prompts via `tmux-cli send`.

IMPORTANT: the SecOps-Factory "Morgan" persona has NO "onboarding gate" skill — drive prism-live DIRECTLY per test-soc/CLAUDE.md (discovery → `check_sensor_health` → `prism_describe` → `query`).

## Discipline

- AD-017: credentials never in AI context
- Customer data = structural observables only, never persisted
- Live loop is read-only
