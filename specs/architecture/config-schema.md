---
document_type: architecture-section
level: L3
section: "config-schema"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T23:45:00
phase: 1b
inputs: [system-overview.md, security-architecture.md, sensor-adapters.md, operational-pipeline.md]
traces_to: ARCH-INDEX.md
---

# Configuration Schema

## File Layout

```
~/.prism/config/
  prism.toml                  # Main configuration (this document)
  aliases.toml                # Query aliases (global + per-client)
  sensors/                    # *.sensor.toml spec files
  infusions/                  # *.infusion.toml spec files
  actions/                    # *.action.toml spec files
  rules/                      # *.detect detection rule files
  plugins/                    # *.prx WASM plugin files
  ioc/                        # *.ioc IOC pattern files
  data/                       # Lookup data (*.mmdb, *.csv, *.json)
  templates/                  # Report/email templates
```

## `prism.toml` — Full Schema

```toml
# ============================================================================
# Prism Configuration
# ============================================================================

# --------------------------------------------------------------------------
# Server Settings
# --------------------------------------------------------------------------
[server]
log_level = "info"                       # trace, debug, info, warn, error
log_format = "json"                      # json or pretty (json for production, pretty for dev)

# --------------------------------------------------------------------------
# Config Sources — Multi-Repo Git Subscriptions
# --------------------------------------------------------------------------
# Prism merges config from multiple git repos by priority (higher overrides lower).
# Local files in this directory have highest priority (implicit priority = 999).

[[config_sources]]
name = "core"
repo = "git@github.com:1898co/prism-config-core.git"
branch = "main"
priority = 0                             # Base layer — company-wide defaults
sync_interval = "5m"
ssh_key = { source = "file", path = "~/.ssh/id_ed25519" }

[[config_sources]]
name = "clients"
repo = "git@github.com:1898co/prism-config-clients.git"
branch = "main"
priority = 20                            # Overrides core
sync_interval = "5m"

# [[config_sources]]
# name = "community"
# repo = "git@github.com:prism-community/detection-packs.git"
# branch = "stable"
# priority = 5
# read_only = true                       # Cannot override core

# --------------------------------------------------------------------------
# Client Definitions
# --------------------------------------------------------------------------
# Each client represents an MSSP customer with their sensor mappings.

[clients.acme]
name = "Acme Corporation"
contact_email = "security@acmecorp.com"  # Used by email actions

[clients.acme.sensors.crowdstrike]
enabled = true
base_url = "https://api.crowdstrike.com" # Override per-client if needed
# Credentials are NOT in this file — use prism credential set CLI
# or reference via [clients.acme.sensors.crowdstrike.credentials] below

[clients.acme.sensors.crowdstrike.credentials]
# Reference-based — Prism resolves at query time. Values NEVER here.
client_id = { source = "keyring" }       # Default: look up in OS keyring
client_secret = { source = "keyring" }

[clients.acme.sensors.claroty]
enabled = true
base_url = "https://acme.claroty.cloud"

[clients.acme.sensors.claroty.credentials]
api_key = { source = "env", key = "PRISM_ACME_CLAROTY_API_KEY" }

# Per-client capability overrides (feature flags)
[clients.acme.capabilities]
"sensor.crowdstrike.containment" = "Allow"
"sensor.crowdstrike.write" = "Deny"      # No CrowdStrike write operations for Acme
"alert.write" = "Allow"
"case.write" = "Allow"

# --------------------------------------------------------------------------
# Second client example
# --------------------------------------------------------------------------
[clients.globex]
name = "Globex Inc"
contact_email = "sec-ops@globex.com"

[clients.globex.sensors.crowdstrike]
enabled = true

[clients.globex.sensors.crowdstrike.credentials]
client_id = { source = "vault", path = "secret/data/prism/globex/crowdstrike", key = "client_id" }
client_secret = { source = "vault", path = "secret/data/prism/globex/crowdstrike", key = "client_secret" }

[clients.globex.sensors.armis]
enabled = true
base_url = "https://globex.armis.com"

[clients.globex.sensors.armis.credentials]
api_secret = { source = "env", key = "PRISM_GLOBEX_ARMIS_API_SECRET" }

[clients.globex.capabilities]
"sensor.crowdstrike.containment" = "Allow"
"case.write" = "Allow"

# --------------------------------------------------------------------------
# Default Settings (apply to all clients unless overridden)
# --------------------------------------------------------------------------
[defaults]

# Default capabilities (deny-by-default per DI-003)
[defaults.capabilities]
# Everything is implicitly Deny unless explicitly listed here
"alert.write" = "Allow"                  # All clients can acknowledge alerts
"case.write" = "Allow"                   # All clients can create/update cases

# Resource limits
[defaults.limits]
max_schedules = 500                      # DI-028
max_rules = 1000                         # DI-028
max_concurrent_schedules = 16            # DI-032
max_concurrent_api_calls_per_query = 10  # Per-query fan-out semaphore
max_concurrent_http_connections = 200    # Global HTTP connection semaphore
max_materialized_records = 10000         # DI-019
max_internal_table_scan = 50000          # BC-2.15.011
max_alerts_per_rule_per_hour = 100       # Detection rate limit
max_alerts_global_per_hour = 1000        # Detection global rate limit
max_group_keys_per_rule = 10000          # Detection state amplification cap

# Query defaults
[defaults.query]
timeout_seconds = 30                     # DI-019
max_query_length = 65536                 # 64 KB, CWE-400
max_nesting_depth = 64                   # CWE-674
max_pipe_stages = 32                     # CWE-400

# Cache defaults
[defaults.cache]
response_cache_entries = 50              # Per client per sensor (DI-018)
response_cache_ttl_seconds = 300         # 5 minutes default
discovery_cache_ttl_seconds = 3600       # 1 hour for pack discovery queries

# Schedule defaults
[defaults.schedule]
splay_percent = 10                       # DI-022 — splay window as % of interval
min_interval_seconds = 60                # Minimum schedule interval

# Alert defaults
[defaults.alerts]
max_snapshot_event_data_bytes = 4096     # EventSnapshot event_data excerpt size
# 0 to disable event_data excerpts, max 16384

# --------------------------------------------------------------------------
# Watchdog Settings
# --------------------------------------------------------------------------
[watchdog]
level = "normal"                         # normal, restrictive, permissive
check_interval_seconds = 3               # RSS check interval
process_rss_limit_mb = 512               # DI-027 — self-SIGTERM threshold

# Per-level overrides (only if level != normal)
# memory_limit_mb = 200                  # Per-query memory budget
# query_timeout_seconds = 30
# max_materialized_records = 10000

# --------------------------------------------------------------------------
# Plugin Settings
# --------------------------------------------------------------------------
[plugins]
max_memory_mb = 64                       # WASM plugin memory limit
max_fuel = 10000000000                   # WASM instruction fuel limit (10B)

# --------------------------------------------------------------------------
# Config Reload Settings
# --------------------------------------------------------------------------
[config]
file_watch_enabled = true                # AD-018 — automatic filesystem watching
file_watch_debounce_ms = 500             # Debounce window (min 100, max 5000)

# --------------------------------------------------------------------------
# Vault Integration (optional, feature-gated)
# --------------------------------------------------------------------------
# [vault]
# backend = "hashicorp"                  # hashicorp, aws-sm, azure-kv, gcp-sm
# address = "https://vault.1898co.com"
# auth_method = "approle"                # approle, token, kubernetes
# role_id = { source = "env", key = "VAULT_ROLE_ID" }
# secret_id = { source = "env", key = "VAULT_SECRET_ID" }
# namespace = "prism"
# ttl_cache_seconds = 300                # Cache vault-fetched credentials

# --------------------------------------------------------------------------
# Audit Settings
# --------------------------------------------------------------------------
[audit]
buffer_max_entries = 100000              # CAP-025
sync_writes = true                       # DI-026 — WAL sync for audit buffer

# External audit forwarding (optional)
# [audit.forward]
# type = "vector"                        # vector, syslog, webhook
# endpoint = "https://vector.1898co.com:8686"
# format = "json"
# retry_base_seconds = 2
# retry_max_seconds = 60
```

## `aliases.toml` — Query Aliases

```toml
# ============================================================================
# PrismQL Aliases
# ============================================================================
# Global aliases available to all clients.
# Per-client aliases override global aliases of the same name.

[global]
critical = 'severity_id >= 5'
high_and_above = 'severity_id >= 4'
recent = 'time > 24h'
last_hour = 'time > 1h'
last_week = 'time > 7d'

# Parameterized alias with defaults
[global.for_ip]
query = 'device_ip = "${ip}"'
params = { ip = "10.0.0.1" }
description = "Filter events by IP address"

[global.top_talkers]
query = '''
SELECT device_ip, COUNT(*) AS event_count, _sensor
FROM EVENTS
WHERE time > ${period}
GROUP BY device_ip, _sensor
ORDER BY event_count DESC
LIMIT ${limit}
'''
params = { period = "24h", limit = "20" }

# Per-client alias (overrides global if same name)
[clients.acme]
ot_devices = '_sensor = "claroty" AND device_type != "IT"'
acme_critical = 'severity_id >= 4 AND _client = "acme"'

[clients.globex]
globex_servers = 'device_hostname CONTAINS "srv" AND _client = "globex"'
```

## Environment Variable Overrides

Any TOML config value can be overridden via environment variable. The pattern is `PRISM_{SECTION}_{KEY}` (uppercased, dots to underscores):

| TOML Path | Environment Variable |
|-----------|---------------------|
| `server.log_level` | `PRISM_SERVER_LOG_LEVEL` |
| `watchdog.level` | `PRISM_WATCHDOG_LEVEL` |
| `watchdog.process_rss_limit_mb` | `PRISM_WATCHDOG_PROCESS_RSS_LIMIT_MB` |
| `defaults.query.timeout_seconds` | `PRISM_DEFAULTS_QUERY_TIMEOUT_SECONDS` |
| `defaults.limits.max_schedules` | `PRISM_DEFAULTS_LIMITS_MAX_SCHEDULES` |
| `config.file_watch_enabled` | `PRISM_CONFIG_FILE_WATCH_ENABLED` |
| `plugins.max_memory_mb` | `PRISM_PLUGINS_MAX_MEMORY_MB` |

Environment variables take precedence over TOML values. Credential values use the separate `_FILE` pattern (see security-architecture.md AD-017).

## Config Validation

All config is validated at startup and on hot reload (DI-031):

- **Tier 1 (prism.toml):** All-or-nothing. Any invalid field rejects the entire reload.
- **Tier 2 (aliases.toml):** All-or-nothing. Cycle detection (DI-020), depth check (max 3).
- **Tier 3 (sensor/infusion/action specs, IOC files, plugins):** Per-file independent. Invalid files rejected individually; valid files still load.

The `prism serve --dry-run` command validates all config without starting the server.

## Config Diff Tool

```bash
# Show what changed since last reload
$ prism config diff

# Show what would change on reload
$ prism config diff --pending

# Show effective config (all sources merged)
$ prism config show --effective

# Show where a specific value comes from (which repo/file/env var)
$ prism config show --trace defaults.limits.max_schedules
  defaults.limits.max_schedules = 500
    Source: prism-config-core (priority 0) / prism.toml line 42
    No override from prism-config-clients (priority 20)
    No environment variable override (PRISM_DEFAULTS_LIMITS_MAX_SCHEDULES not set)
```
