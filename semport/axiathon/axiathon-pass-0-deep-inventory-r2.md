# Pass 0 Deep: Inventory -- Round 2

**Project:** Axiathon
**Pass:** 0 (Inventory)
**Round:** 2
**Date:** 2026-04-13

---

## Purpose

Hallucination audit of R1 claims, fill remaining gaps (detection rule files, API route inventory, docs/.archive/ scope), and verify workspace configuration details.

---

## 1. Hallucination Audit

### 1.1 R1 Claims Verified Against Source

| R1 Claim | Verification | Status |
|----------|-------------|--------|
| "Two separate Cargo workspaces" | Root Cargo.toml members=["crates/*"], spike/Cargo.toml members=[19 crates] | CORRECT |
| "Production MSRV 1.85" | Root Cargo.toml rust-version = "1.85" | CORRECT |
| "Spike MSRV 1.88" | spike/Cargo.toml rust-version = "1.88" | CORRECT |
| "prost 0.14 in production" | Root Cargo.toml prost = "0.14" | CORRECT |
| "prost 0.13 in spike" | spike/Cargo.toml prost = "0.13" | CORRECT |
| "prost-reflect 0.15 production / 0.14 spike" | Root: 0.15, spike: 0.14 | CORRECT |
| "8 production crates" | ls crates/ shows 8 directories | CORRECT |
| "19 spike crates" | spike/Cargo.toml members lists 19 | CORRECT |
| "forbid(unsafe_code) in all 8 production crates" | grep confirmed 8 files | CORRECT |
| "No forbid(unsafe_code) in spike" | grep found 0 in spike | CORRECT |
| "Iceberg git fork" | spike/Cargo.toml: git = "https://github.com/drbothen/iceberg-rust" | CORRECT |
| "extism for WASM" | spike/Cargo.toml: extism = "1" | CORRECT |
| "axiathon-types planned but not implemented" | depgraph-rules.toml mentions it, unmatched_config_entries = "warn" | CORRECT |
| "axiathon-ai planned but not implemented" | depgraph-rules.toml mentions it | CORRECT |

**No hallucinations found in R1.**

### 1.2 Broad Sweep Claims Re-verified

| Broad Sweep Claim | R2 Verification | Status |
|-------------------|-----------------|--------|
| "axiathon-core ~600 LOC" | Unable to verify (sandbox prevents wc -l), but 5 source files with substantial content | PLAUSIBLE |
| "axiathon-query ~1200 LOC" | 7 source files including parser.rs (500+ lines visible) | PLAUSIBLE |
| "spike/axiathon-core ~1200 LOC" | 8 source files + build.rs + bin | PLAUSIBLE |
| "Chumsky 0.10 for production" | Root Cargo.toml: chumsky = "0.10" | CORRECT |
| "Pest for spike detection DSL" | spike/Cargo.toml: pest = "2", pest_derive = "2" | CORRECT |
| "arrow 57, datafusion 51" | Both workspaces: arrow = "57", datafusion = "51" | CORRECT |

---

## 2. Detection Rule File Inventory (NEW)

6 `.axd` files in `spike/rules/`:

| File | Rule ID | Type | Description |
|------|---------|------|-------------|
| `root-login.axd` | root_login | SingleEvent | Root login detected |
| `suspicious-ip.axd` | suspicious_source_ip | SingleEvent | Login from suspicious IP range (203.0.113.0/24) |
| `privilege-escalation.axd` | privilege_escalation | SingleEvent | Sudo/su privilege escalation |
| `brute-force.axd` | brute_force | Correlation | 5+ failed logins in 5m by src IP |
| `brute-then-success.axd` | brute_then_success | Sequence | Success after 3+ failures in 10m by src IP |
| `ot-unauthorized-plc-access.axd` | ot_unauthorized_plc_access | SingleEvent | Claroty xDome unauthorized PLC access |

These 6 files match the 6 BUILTIN_RULE_SOURCES constants in state.rs (identical rule definitions). The .axd files are loaded by the benchmark suite via `load_rules_from_dir()`.

### Detection Rule Conventions (NEW)

- File naming: kebab-case matching rule description
- Rule ID: snake_case matching rule block name
- One rule per file
- Structure: `rule <id> { meta { ... } match <clause> alert { ... } }`
- MITRE ATT&CK ID in meta block (T1078, T1110, T1133, T1548, T0821)
- Template variables in alert: `{field}`, `{count}`, `{window}`, `{step.field}`, `{step.count}`

---

## 3. API Route Inventory (NEW)

35 routes in spike API:

### Tenant-Scoped Routes (32 routes, require X-Tenant-ID header)

| Method | Path | Handler | Domain |
|--------|------|---------|--------|
| POST | /api/v1/ingest | ingest_events | Ingestion |
| POST | /api/v1/ingest/raw | ingest_raw | Ingestion |
| GET | /api/v1/alerts | list_alerts | Alerts |
| GET | /api/v1/alerts/stream | alert_stream | Alerts (SSE) |
| POST | /api/v1/query | execute_query | Query |
| GET | /api/v1/rules | list_rules | Detection |
| POST | /api/v1/rules | create_rule | Detection |
| POST | /api/v1/rules/validate | validate_rule | Detection |
| GET | /api/v1/rules/{id} | get_rule | Detection |
| PUT | /api/v1/rules/{id} | update_rule | Detection |
| POST | /api/v1/cases | create_case | Cases |
| GET | /api/v1/cases | list_cases | Cases |
| GET | /api/v1/cases/metrics | case_metrics | Cases |
| GET | /api/v1/cases/{id} | get_case | Cases |
| PATCH | /api/v1/cases/{id}/status | update_case_status | Cases |
| POST | /api/v1/cases/{id}/alerts | link_alerts | Cases |
| POST | /api/v1/cases/{id}/annotations | add_annotation | Cases |
| PATCH | /api/v1/cases/{id}/disposition | set_disposition | Cases |
| POST | /api/v1/vault/credentials | store_credential | Vault |
| GET | /api/v1/vault/credentials | list_credentials | Vault |
| DELETE | /api/v1/vault/credentials/{name} | delete_credential | Vault |
| GET | /api/v1/plugins | list_plugins | Plugins |
| GET | /api/v1/plugins/metrics | plugin_metrics | Plugins |
| GET | /api/v1/plugins/{id} | get_plugin | Plugins |
| PATCH | /api/v1/plugins/{id}/status | update_plugin_status | Plugins |
| PUT | /api/v1/plugins/{id}/config | update_plugin_config | Plugins |
| POST | /api/v1/plugins/{id}/health | refresh_plugin_health | Plugins |
| POST | /api/v1/plugins/{id}/install | install_plugin_for_tenant | Plugins |
| GET | /api/v1/tenant/plugins | list_tenant_plugins | Plugins |
| GET | /api/v1/tenant/plugins/{id} | get_tenant_plugin | Plugins |
| PUT | /api/v1/tenant/plugins/{id}/config | update_tenant_plugin_config | Plugins |
| PATCH | /api/v1/tenant/plugins/{id}/enabled | toggle_tenant_plugin_enabled | Plugins |

### Public Routes (3 routes, no auth)

| Method | Path | Handler | Domain |
|--------|------|---------|--------|
| GET | /health | health | Health |
| GET | /api/v1/admin/mssp-dashboard | mssp_dashboard | MSSP Admin |
| POST | /api/v1/admin/promote | promote_fields_handler | Schema Admin |

### Route Domain Distribution

| Domain | Routes | % |
|--------|--------|---|
| Plugins | 11 | 31% |
| Cases | 8 | 23% |
| Detection Rules | 5 | 14% |
| Ingestion | 2 | 6% |
| Alerts | 2 | 6% |
| Vault | 3 | 9% |
| Query | 1 | 3% |
| Admin | 3 | 9% |

---

## 4. Additional Configuration Details (NEW)

### 4.1 VS Code Settings

rust-analyzer configured with:
- clippy as check command
- All targets and all features enabled
- Nightly rustfmt
- Inlay hints (type, chaining, parameter, closure return)
- Semantic highlighting
- Format on save

### 4.2 rust-toolchain.toml

Simply `channel = "stable"`. No component pinning.

### 4.3 Brewfile

3 tools: lefthook, taplo, typos-cli (subset of full tool requirements from CONTRIBUTING.md)

### 4.4 Git LFS

CONTRIBUTING.md mentions Git LFS setup with `git lfs install`. Binary assets add ~95MB. The `.gitattributes` file likely tracks large files (not examined in detail).

### 4.5 Environment Variables (spike)

| Variable | Default | Purpose |
|----------|---------|---------|
| AXIATHON_WAREHOUSE | "data/warehouse" | Storage directory path |
| PORT | 3000 | HTTP server port |
| RUST_LOG (via EnvFilter) | "axiathon_api=debug,tower_http=debug" | Log level |

---

## Delta Summary
- New items added: 6 detection rule files inventoried with conventions, 35 API routes documented with domain distribution, VS Code configuration, environment variables, Git LFS usage, rust-toolchain.toml (stable channel)
- Existing items refined: All R1 claims verified (0 hallucinations found), broad sweep LOC estimates marked as plausible but unverifiable
- Remaining gaps: Exact LOC per file still unavailable, docs/.archive/ content summary (100+ files), _bmad-output/ artifact structure

## Novelty Assessment
Novelty: NITPICK

The API route inventory (35 routes) and detection rule file catalog (6 .axd files) add detail to already-known bounded contexts. The plugin domain having 31% of all routes is mildly interesting but predictable given the 8-trait SDK. The hallucination audit confirmed all R1 claims. None of these findings change how you'd spec the system -- the route inventory is implementation detail, and the .axd files duplicate the BUILTIN_RULE_SOURCES already documented in R1.

Would removing this round's findings change how you'd spec the system? No. The API surface and rule file conventions are implementation artifacts within already-identified bounded contexts.

## Convergence Declaration
Pass 0 has converged -- findings are detail additions within already-known structure, not new gaps. The remaining items (exact LOC, docs/.archive/ content) are low-priority and would not change the spec.

## State Checkpoint
```yaml
pass: 0
round: 2
status: complete
files_scanned: 15
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
convergence: Pass 0 inventory has converged
```
