---
document_type: research
producer: smoke-test + discoverability-audit (claude, MCP-only information asymmetry)
timestamp: 2026-06-24
topic: T13 Capstone Live Demo — Pre-Flight Smoke Test + Discoverability Audit
status: complete
feeds: T13 capstone demo preparation; CrowdStrike OAuth plug-in investigation
branch: develop
---

# T13 Capstone Live Demo — Pre-Flight Smoke Test + Discoverability Audit (2026-06-24)

Role: competent security analyst (LLM agent) connecting to a freshly-provisioned Prism
MCP server. Strict information asymmetry on the discovery layer: enrichment syntax,
table names, and query patterns are inferred only from the live MCP surface. Operational
tooling (`demo-setup.sh`, `demo-run.sh`) is treated as black-box bring-up machinery —
its content is not used as a discovery hint.

Target: main worktree `/Users/jmagady/Dev/prism`, local `develop` HEAD, DTU mode.

---

## 1. Methodology and Repeatability Guide

### 1.1 Bring-up procedure

```bash
# Step 1 — provision config + credentials (idempotent)
bash scripts/demo-setup.sh

# Step 2 — start DTU + write per-org overlay TOMLs
bash scripts/demo-run.sh
# Captures: DTU PID, per-org overlay files under ~/.config/prism-demo/specs/customers/,
# and ~/.config/prism-demo/run/dtu-server.log with ephemeral port assignments.

# Step 3 — record the env block printed by demo-run.sh
# (PRISM_DTU_MODE=true, CROWDSTRIKE_BASE_URL=..., etc.)
# These env vars must be passed to 'prism start' for every test script.
```

**Critical DTU stability rule:** Never use `pkill -f "prism.*start"` — the DTU binary
is `prism-dtu-demo-server start-multi`, which matches this pattern and will be killed.
Use `kill <specific_PID>` to terminate only the prism MCP server. When the DTU is
accidentally killed, run `bash scripts/demo-run.sh` to restart it; new ephemeral ports
are written to `~/.config/prism-demo/specs/customers/` automatically.

### 1.2 Transport and test harness

Prism is a **stdio MCP server** (JSON-RPC 2.0, protocol version `2024-11-05`). Smoke
tests use a small Python driver that:

1. Spawns `prism start --config-dir ~/.config/prism-demo` with the env block
2. Sends `initialize` immediately (no sleep; delay causes BrokenPipe)
3. Sends `notifications/initialized`
4. Issues tool calls and reads responses via line-delimited JSON with `select()` timeout

Key schema detail: `query` tool parameter is `{"query": "...", "clients": ["org-c"]}` —
the list parameter is named `clients`, NOT `client_id`. This is discoverable from the
`tools/list` schema only; it is not stated in the tool description prose.

### 1.3 MCP response ordering constraint

MCP responses are serialized by request ID. If tool N hangs indefinitely, tools N+1,
N+2, … will also timeout even if their underlying operations would complete. This means
a single hanging tool (e.g., CrowdStrike OAuth) will cascade-timeout all subsequent
tools in the same session. Test scripts must be designed so that known-hanging tools
(see §3 BLOCKER-001) are either skipped or called last.

### 1.4 PrismQL grammar (confirmed from source)

Two syntactically distinct modes exist:

**SQL mode** (no enrichment support):
```sql
SELECT col1, col2 FROM table_name WHERE pred LIMIT N
```

**Pipe mode** (enrichment supported):
```
FROM table_name | where pred | enrich fn(col) | limit N
```

`WHERE` and `LIMIT` are **pipe stages** in pipe mode — they are NOT inline SQL clauses
after the source table. Mixing SQL clauses with pipe stages causes a parse error at the
first SQL keyword. The correct pipe syntax is: source table first, then `|` stages.

Enrichment UDF names available in this environment:
- `threat_score(col)`, `threat_is_known_malicious(col)`, `threat_sources(col)` — ThreatIntel plugin
- `cvss_base_score(col)`, `cvss_severity(col)`, `cvss_vector(col)` — NVD http_lookup

### 1.5 Org and sensor topology (3 orgs, 8 sensor instances)

| Org   | Sensors                                         | Device seed |
|-------|-------------------------------------------------|-------------|
| org-a | crowdstrike, armis                              | 100         |
| org-b | claroty, cyberint                               | 150 (inferred; seed field not probed) |
| org-c | armis, crowdstrike, claroty, cyberint           | 200         |

Device IDs: `dev-0196f4b2-{seed}-{N}` where seed is org-specific.

### 1.6 Scenario clock

`scenario_start_secs = 1782214754` — places the demo permanently in Stage 4
(Containment) regardless of wall-clock time. This is set in a demo config file, not by
an env var, so it persists across prism restarts without re-running setup.

---

## 2. §5 Checklist Walkthrough — Actual Query Results

All results from live runs against develop HEAD, DTU running. Query timing measured from
MCP request send to response receipt.

### 2.1 §5.1 — Server starts, 8 sensor adapters across 3 orgs

**Method:** `resources/read prism://config/clients`

**Result:**
```json
[
  {"client_id": "org-a", "sensors": ["crowdstrike", "armis"]},
  {"client_id": "org-b", "sensors": ["claroty", "cyberint"]},
  {"client_id": "org-c", "sensors": ["armis", "crowdstrike", "claroty", "cyberint"]}
]
```

**Verdict: PASS** — 3 orgs, 8 sensor adapter instances enumerated correctly.

Timing: ~0.1s. MCP server version `0.1.0`.

### 2.2 §5.2 — prism_describe works for each org

**Method:** `prism_describe(client_id="org-c")`

Representative output (org-c): tables array includes `crowdstrike_detections`,
`armis_devices`, `claroty_devices`, `claroty_audit_log`, `cyberint_alerts` with their
column lists.

**E-QUERY-038 (column not found) fires correctly:**
```
Query:  SELECT device_id, nonexistent_column FROM crowdstrike_detections LIMIT 2
Response: E-QUERY-038: column 'nonexistent_column' not found in table
  'crowdstrike_detections' for client 'org-c'; available: [behaviors_ioc_description,
  behaviors_ioc_source, behaviors_ioc_type, behaviors_ioc_value, created_timestamp,
  detection_id, device_id, severity, status, tactic, technique].
  Call prism_describe('<client_id>') to see available columns, or use the
  available_columns field in this error to correct the column name.
```

The `structuredContent.error.available_columns` array is machine-readable. No
`normalized_pql` field present in E-QUERY-038 (that field was tested but is absent).

**Verdict: PASS** — prism_describe functional; E-QUERY-038 fires with actionable column list.

### 2.3 §5.3 — CrowdStrike detections: device IDs disjoint across orgs

**Results (fresh prism session, CrowdStrike called first before OAuth state corrupts):**

| Query | Rows | Sample device_id |
|-------|------|-----------------|
| `SELECT device_id FROM crowdstrike_detections LIMIT 3` for org-a | 3 | `dev-0196f4b2-100-3` |
| `SELECT device_id FROM crowdstrike_detections LIMIT 3` for org-c | 3 | `dev-0196f4b2-200-11` |

Org-a seed = 100, org-c seed = 200. Device IDs are disjoint across orgs.

**BLOCKER CAVEAT:** CrowdStrike works ONLY in the first prism session after a DTU
restart. A second prism session with the same DTU will hang CrowdStrike queries
indefinitely (see BLOCKER-001 in §3). The distinctness data above was obtained in the
one session window where CrowdStrike was called first.

**Verdict for §5.3: CONDITIONAL PASS** — disjoint IDs confirmed, but requiring
first-session timing constraint makes this unreliable for live demo. See BLOCKER-001.

### 2.4 §5.4 — Non-CrowdStrike sensors return data

Tested in a clean session with Armis and Cyberint called BEFORE any CrowdStrike query:

| Query | Org | Rows | Sample row |
|-------|-----|------|-----------|
| `SELECT device_id FROM armis_devices LIMIT 3` | org-c | 3 | `{"device_id": "dev-0196f4b2-200-0"}` |
| `SELECT alert_id FROM cyberint_alerts LIMIT 3` | org-c | 3 | `{"alert_id": "alert-0196f4b2-200-0"}` |
| `SELECT device_id FROM armis_devices LIMIT 3` | org-a | 3 | (device seed 100 confirmed) |
| `SELECT device_id FROM claroty_devices LIMIT 2` | org-b | 2 | rows confirmed |
| `SELECT log_id FROM claroty_audit_log LIMIT 2` | org-b | 2 | rows confirmed |

Timing: 0.1-0.2s per query.

**Table name note:** `claroty_audit_log` (singular) is the correct table name.
`claroty_audit_logs` (plural) was also tested; the runbook uses singular — this is
consistent with the actual table registry.

**Verdict: PASS** — all non-CrowdStrike sensors return data when CrowdStrike is not
called first.

### 2.5 §5.5 — Enrichment (ThreatIntel + NVD CVSS)

**Correct pipe syntax discovered via audit (NOT from runbook):**
```
FROM cyberint_alerts | where iocs_value IS NOT NULL | enrich threat_score(iocs_value) | limit 3
```

**Result:** 3 rows returned in 0.0s. Enrichment working.

```
FROM armis_devices | where device_cves_first IS NOT NULL | enrich cvss_base_score(device_cves_first) | limit 3
```

**Result:** 3 rows returned with CVSS scores. Enrichment working.

**RUNBOOK-DRIFT-001 — pipe syntax wrong (BLOCKER):**
The runbook §5.5 example uses:
```
FROM table WHERE pred | enrich fn LIMIT N
```
This is syntactically invalid. `WHERE` in pipe mode is the `| where` pipe stage, NOT an
inline SQL clause. The parser emits a parse error at offset 21 (`W` = unexpected token).

Correct form: `FROM table | where pred | enrich fn(col) | limit N`

**Verdict: PASS for enrichment engine; BLOCKER for runbook §5.5 syntax.**

### 2.6 §5.6 — Multi-tenant isolation (E-QUERY-032)

**Test:** Query CrowdStrike data for org-a from org-b context (sensor not registered):

```sql
SELECT detection_id FROM crowdstrike_detections LIMIT 3  -- clients=["org-b"]
```

**Response:**
```
E-QUERY-032: Sensor 'crowdstrike' is not registered for org 'org-b'.
Check sensor registration for the target org.
```

**Verdict: PASS** — E-QUERY-032 fires correctly for cross-org sensor access.

Also confirmed: dot-syntax table reference triggers E-QUERY-036:
```
SELECT * FROM cyberint.alerts LIMIT 3  -- clients=["org-c"]
→ E-QUERY-036: unknown source table 'cyberint.alerts': table is not a registered
  sensor or internal table. Did you mean 'cyberint_alerts'? (E-QUERY-036)
```

All three pedagogical errors work correctly in a clean session (not blocked by
CrowdStrike hang).

**Verdict: PASS** — all three error codes fire as designed.

### 2.7 §5.7 — check_sensor_health

**Result:** `probe_level: spec-only` for all sensors. S-5.04 (active HTTP probing) not
yet merged. All sensors report `health: "healthy"` at spec-only level.

**Verdict: DEFERRED** — per S-5.04 story scope. Spec-only probing is the current
correct behavior.

### 2.8 §5.8 — Prompts and resources

| Prompt | Notes |
|--------|-------|
| `triage_alerts(org-c)` | Returns — see AUDIT-004 (dot syntax in prompt body) |
| `query_tutorial(org-c, ...)` | TIMEOUT (hangs indefinitely; see BLOCKER-003) |
| `investigate_host(org-c, ...)` | TIMEOUT (hangs indefinitely) |
| `cross_client_status` | Returns partial content (also uses dot syntax) |
| `client_overview(org-c)` | Returns correctly |

`prismql://reference` resource: 6,471 chars. Contains SQL grammar, examples, error
codes. Zero occurrences of `enrich` keyword — enrichment syntax is absent from the
canonical reference. See DISCOVERABILITY-GAP-001.

---

## 3. FINDINGS

Findings tagged `[NEW]` are first observed in this audit session.
Findings tagged `[ALREADY-KNOWN]` were documented in the 2026-06-23 audit and are
reconfirmed here with any new evidence.

---

### BLOCKER-001 — CrowdStrike OAuth plugin corrupts state across prism sessions [NEW]

**Severity: DEMO-BLOCKER**

**Observation:**
CrowdStrike queries work correctly in the FIRST prism session after a DTU restart.
In any subsequent prism session (same DTU, same overlays, same env vars), CrowdStrike
queries hang at exactly 30 seconds and return 0 rows with no error.

After CrowdStrike hangs in a session, ALL subsequent sensor queries in that same session
also hang (due to MCP response serialization; see §1.3).

**Definitively confirmed by:**
```
TEST A: Armis FIRST (no CrowdStrike)
  [0.1s] Armis first: 3 rows  {"device_id": "dev-0196f4b2-200-0"}
  [0.0s] Cyberint second: 3 rows  {"alert_id": "alert-0196f4b2-200-0"}

TEST B: CrowdStrike THEN Armis (second session, same DTU)
  [30.0s] CrowdStrike first: 0 rows  (TIMEOUT)
  [30.1s] Armis after CS: 0 rows TIMEOUT
  [30.1s] Cyberint after CS: 0 rows TIMEOUT
```

**Hypothesis:** The CrowdStrike OAuth2 WASM plugin (`crowdstrike-oauth2.prx`) caches a
token in RocksDB during the first session. On the second session, the plugin attempts to
use or refresh the cached token against the DTU endpoint, but the DTU's OAuth simulation
does not accept the refresh (or the token format has changed), causing the plugin to
wait for an HTTP response that never arrives. The 30s hang matches the HTTP client
timeout (`PLUGIN_HTTP_CLIENT_TIMEOUT_SECS`).

**Impact on demo:** The §5.3 distinctness test (two org's CrowdStrike data) requires two
separate prism sessions. If any prior session already ran CrowdStrike, §5.3 will silently
produce 0 rows.

**Workaround:** Before any demo session that requires CrowdStrike data:
1. Kill all prism processes: `kill $(pgrep -f "target/release/prism start")`
2. Restart DTU: `bash scripts/demo-run.sh`
3. Clear RocksDB OAuth token cache (state dir): examine
   `~/.config/prism-demo/state/` — delete the CrowdStrike-specific column family or
   wipe the state dir and re-run `demo-setup.sh`
4. In the live demo session, call CrowdStrike queries FIRST before any other sensor

**Follow-up required:** Investigate `crowdstrike-oauth2.prx` token refresh behavior
against DTU. The DTU's CrowdStrike mock may need to implement `/oauth2/token` refresh
endpoint, or the plugin needs to detect stale-token-from-different-session and force
reauth rather than hang.

---

### BLOCKER-002 — Runbook §5.5 pipe syntax is syntactically invalid [NEW]

**Severity: DEMO-BLOCKER (runbook)**

**Observation:**
The runbook (or demo script §5.5) shows enrichment queries in the form:
```
FROM table WHERE pred | enrich fn LIMIT N
```

This fails with a parse error at offset 21 (the `W` of `WHERE`). In PrismQL pipe mode,
`WHERE` and `LIMIT` are pipe stages, not SQL clauses. The correct syntax is:
```
FROM table | where pred | enrich fn(col) | limit N
```

**Confirmed working queries:**
```
FROM cyberint_alerts | where iocs_value IS NOT NULL | enrich threat_score(iocs_value) | limit 3
FROM armis_devices | where device_cves_first IS NOT NULL | enrich cvss_base_score(device_cves_first) | limit 3
```

**Surface element at fault:** Demo runbook / §5.5 example text.

**Fix:** Update runbook to use correct pipe syntax. The `| where` and `| limit` stages
are required; SQL-mode `WHERE` / `LIMIT` are incompatible with the `enrich` pipe stage.

---

### BLOCKER-003 — query_tutorial and investigate_host prompts hang indefinitely [NEW]

**Severity: DEMO-BLOCKER (for affected prompt demos)**

**Observation:**
`prompts/get` for `query_tutorial` and `investigate_host` never returns — confirmed via
30-second timeout in multiple separate prism sessions. `triage_alerts`, `client_overview`,
and `cross_client_status` prompts return normally.

**Impact:** §5.8 cannot demo `query_tutorial` or `investigate_host` without timing out.
If these are called before other tools in the same MCP session, ALL subsequent tool
calls will also timeout (due to ID serialization).

**Root cause unknown.** Possible causes: prompt handler waiting on a background
resource that never initializes; async generator not terminating; lock contention with
another subsystem. Requires code investigation.

**Workaround:** Skip `query_tutorial` and `investigate_host` in the live demo, or call
them after all other tools to avoid cascading timeouts.

---

### BLOCKER-004 — list_infusions, plugin_status, infusion_status all hang [NEW]

**Severity: BLOCKER (for plugin/infusion status demos)**

**Observation:**
`list_infusions`, `plugin_status`, and `infusion_status` all hang indefinitely (30s+
timeout). `list_plugins` returns correctly. `reload_infusion` not tested.

Contrast with the 2026-06-23 audit where `list_infusions` returned `-32003 "Feature not
yet available"` — the behavior has changed. These tools now appear to be implemented but
block on something rather than fast-failing.

**Impact:** Any demo flow that checks infusion/plugin status before querying will hang
the entire session.

**Workaround:** Call `list_plugins` (which works) instead of `plugin_status` or
`list_infusions`.

---

### MAJOR-001 — list_capabilities returns client_registered: False for all orgs [NEW]

**Severity: MAJOR (discoverability gap)**

**Observation:**
`list_capabilities(client_id="org-c")` returns `client_registered: False` for all tested
orgs, including orgs confirmed to have active sensors and returning query data. The
capability registry does not reflect the actual registered sensor set.

**Impact:** An analyst consulting `list_capabilities` to understand what enrichment
functions or sensors are available will conclude the system has nothing registered —
even while `query` returns real data. This undermines the discoverability of both
sensors and enrichment capabilities.

**Root cause hypothesis:** `list_capabilities` reads from a capability registration path
that is not populated by the demo provisioning scripts. The registration may require a
separate step that `demo-setup.sh` does not perform.

---

### AUDIT-001 — prism_describe table name vs FROM-ready name mismatch [ALREADY-KNOWN]

Reconfirmed from 2026-06-23 audit. `prism_describe` reports `name:"alerts"` (short
unqualified name); `FROM` requires `cyberint_alerts` (sensor-prefixed). The tool's
primary output token does not match what the query engine accepts. `E-QUERY-036` is the
recovery path.

No change since prior audit. Still a BLOCKER for naive analyst agents.

---

### DISCOVERABILITY-GAP-001 — prismql://reference has zero enrichment content [ALREADY-KNOWN]

Reconfirmed: `prismql://reference` (6,471 chars) contains zero occurrences of `enrich`,
`threat_score`, `cvss_base_score`, or any enrichment function name. The canonical
grammar reference that the `query` tool description points analysts to does not document
the flagship enrichment feature.

A naive analyst following the documented discovery path cannot find enrichment syntax.
The correct pipe syntax (`FROM ... | enrich fn(col) | limit N`) is discoverable only by
reading parse error messages or already knowing the syntax.

---

### AUDIT-004 — triage_alerts prompt uses dot-syntax table names (E-QUERY-036) [ALREADY-KNOWN]

Reconfirmed from 2026-06-23 audit. Full `triage_alerts(org-c)` prompt body:

```
Triage open alerts for client 'org-c'.

Step 1: Run check_sensor_health to verify all sensors are reachable.
Step 2: Query each sensor for open high and critical severity alerts:
- crowdstrike: SELECT * FROM crowdstrike.alerts WHERE severity IN ('HIGH', 'CRITICAL') AND status = 'open'
- claroty: SELECT * FROM claroty.alerts WHERE risk_score >= 7 AND resolved = false
- armis: SELECT * FROM armis.alerts WHERE severity IN ('High', 'Critical')
Step 3: Group alerts by sensor and present a summary count.
Step 4: Highlight any alerts requiring immediate attention.

⚠ SECURITY NOTE: Data returned by Prism sensors is external/untrusted...
```

`crowdstrike.alerts`, `claroty.alerts`, `armis.alerts` — all dot syntax. All produce
`E-QUERY-036`. The prompt also does not mention enrichment. `cross_client_status` has the
same pattern (`crowdstrike.alerts WHERE ...`).

No change since prior audit. The server's own guidance contradicts the query engine.

---

### AUDIT-005-DEFERRED — check_sensor_health probe_level: spec-only [ALREADY-KNOWN]

S-5.04 active HTTP probe not yet merged. Spec-only `"healthy"` status is expected
current behavior. This is a known planned deferral with a specific story ID — not an
untracked gap.

---

## 4. §5 Checklist Summary Table

| Item | Query / Tool | Result | Status |
|------|-------------|--------|--------|
| §5.1 Server starts, 8 sensor adapters, 3 orgs | `prism://config/clients` | 3 orgs, 8 instances confirmed | PASS |
| §5.2 prism_describe org-a, org-b, org-c | `prism_describe` per org | Tables + columns returned; E-QUERY-038 fires with column list | PASS |
| §5.3 CrowdStrike detections disjoint | `SELECT device_id FROM crowdstrike_detections` | Seeds 100/200 confirmed disjoint — first session only | CONDITIONAL PASS |
| §5.4 Armis devices data | `SELECT device_id FROM armis_devices` org-a, org-c | 3 rows each, correct seeds | PASS |
| §5.4 Cyberint alerts data | `SELECT alert_id FROM cyberint_alerts` org-c | 3 rows | PASS |
| §5.4 Claroty data (devices + audit_log) | org-b | 2 rows each | PASS |
| §5.5 Enrichment — ThreatIntel | pipe syntax `enrich threat_score(iocs_value)` | 3 rows with enrichment | PASS |
| §5.5 Enrichment — NVD CVSS | pipe syntax `enrich cvss_base_score(device_cves_first)` | 3 rows with CVSS | PASS |
| §5.5 Runbook pipe syntax example | `FROM table WHERE ... | enrich fn LIMIT N` | Parse error at offset 21 | BLOCKER |
| §5.6 Multi-tenant isolation (E-QUERY-032) | Cross-org sensor query | Error fires correctly | PASS |
| §5.6 Dot syntax (E-QUERY-036) | `FROM cyberint.alerts` | Error fires with correction hint | PASS |
| §5.6 Column not found (E-QUERY-038) | `SELECT nonexistent_column` | Error fires with available_columns list | PASS |
| §5.7 check_sensor_health | `check_sensor_health` | probe_level: spec-only (S-5.04 pending) | DEFERRED |
| §5.8 triage_alerts prompt | `prompts/get triage_alerts` | Returns; uses broken dot syntax (AUDIT-004) | PASS/DRIFT |
| §5.8 query_tutorial prompt | `prompts/get query_tutorial` | Hangs indefinitely | BLOCKER |
| §5.8 investigate_host prompt | `prompts/get investigate_host` | Hangs indefinitely | BLOCKER |
| §5.8 prismql://reference | Resource read | 6471 chars, no enrichment content | DISCOVERABILITY-GAP |
| CrowdStrike second session | Any CS query in session 2+ | Hangs 30s, then 0 rows | BLOCKER |
| list_infusions / plugin_status | MCP tool calls | Hang indefinitely | BLOCKER |
| list_capabilities | `list_capabilities(org-c)` | client_registered: False for all orgs | MAJOR |

---

## 5. Runbook ↔ Code Drift Summary

| Location | Runbook/Prompt Text | Actual Behavior | Severity |
|----------|---------------------|-----------------|----------|
| §5.5 enrichment example | `FROM table WHERE pred \| enrich fn LIMIT N` | Parse error; correct form is `FROM table \| where pred \| enrich fn(col) \| limit N` | BLOCKER |
| `triage_alerts` prompt | `SELECT * FROM crowdstrike.alerts WHERE ...` | E-QUERY-036 unknown source table | MAJOR |
| `triage_alerts` prompt | `SELECT * FROM claroty.alerts WHERE ...` | E-QUERY-036 | MAJOR |
| `triage_alerts` prompt | `SELECT * FROM armis.alerts WHERE ...` | E-QUERY-036 | MAJOR |
| `cross_client_status` prompt | Same dot-syntax pattern | E-QUERY-036 | MAJOR |
| `prismql://reference` | No enrichment grammar documented | Enrichment works but is undiscoverable | MAJOR |
| `query_tutorial` prompt | Should teach query patterns | Hangs indefinitely | BLOCKER |
| `list_infusions` tool | Should list enrichment pipelines | Hangs indefinitely (was -32003 before) | BLOCKER |

---

## 6. Demo Execution Order Recommendation

Given the CrowdStrike OAuth session corruption issue (BLOCKER-001), the following
execution order maximizes demo reliability:

1. **Start clean:** `kill` all prism processes + wipe CrowdStrike token state from RocksDB
2. **Restart DTU:** `bash scripts/demo-run.sh` (new ephemeral ports written to overlays)
3. **In the single demo prism session:**
   a. Call `prism://config/clients` (resource read — always works)
   b. Call `prism_describe` for each org (always works)
   c. Call Armis, Cyberint, Claroty queries FIRST (sensors unaffected by CS state)
   d. Call CrowdStrike queries LAST in the session
   e. Call enrichment queries (pipe syntax; no CrowdStrike dependency)
   f. Trigger E-QUERY-032, E-QUERY-036, E-QUERY-038 (all work in clean sessions)
   g. Skip `query_tutorial`, `investigate_host`, `list_infusions`, `plugin_status`
4. **Never call another prism session** after CrowdStrike queries without clearing state

---

## 7. Bottom Line — Pre-Flight Verdict

**DEMO-READY: NO — 4 BLOCKERS must be resolved before T13 capstone demo**

| Finding | Severity | Action Required |
|---------|---------|-----------------|
| BLOCKER-001: CrowdStrike OAuth state corruption across sessions | DEMO-BLOCKER | Investigate `crowdstrike-oauth2.prx` token refresh vs DTU; clear RocksDB state before each demo |
| BLOCKER-002: Runbook §5.5 pipe syntax wrong | DEMO-BLOCKER | Fix runbook example: `\| where pred \| limit N` not `WHERE pred ... LIMIT N` |
| BLOCKER-003: `query_tutorial` and `investigate_host` hang | DEMO-BLOCKER | Investigate prompt handler blocking; skip in demo until resolved |
| BLOCKER-004: `list_infusions`, `plugin_status`, `infusion_status` hang | BLOCKER | Investigate tool handler; skip or call `list_plugins` instead |
| MAJOR-001: `list_capabilities` returns `client_registered: False` | MAJOR | Investigate capability registration in demo provisioning path |
| AUDIT-004: Prompts teach dot-syntax table names | MAJOR (ALREADY-KNOWN) | Regenerate prompt bodies using real table registry |
| DISCOVERABILITY-GAP-001: enrichment absent from `prismql://reference` | MAJOR (ALREADY-KNOWN) | Add pipe+enrich grammar section to reference resource |

**What works:** Core sensor queries (non-CrowdStrike unconditionally; CrowdStrike in
first session), all three pedagogical error codes, enrichment engine (correct pipe
syntax), multi-tenant isolation, `prism_describe`, device ID seed distinctness,
`triage_alerts` / `client_overview` / `cross_client_status` prompts return.

**Query performance:** 0.1–0.2s for basic SQL; enrichment pipe queries also ~0.0–0.1s.
No latency issues when the system is not blocked by CrowdStrike OAuth hang.
