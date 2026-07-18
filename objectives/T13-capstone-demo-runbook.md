---
document_type: demo-runbook
objective: T13-capstone
level: ops
version: "1.12"
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
project: prism
status: draft
gates_on:
  - S-DEMO-PRISMQL-ONBOARDING-001-B MERGED
  - S-5.04 MERGED
related:
  - .factory/objectives/DEMO-SCOPE.md
  - .factory/objectives/multi-client-soc-demo-tasks.md
  - .factory/SESSION-HANDOFF.md §RESUME SNAPSHOT D-1292
---

# T13 Capstone Demo Runbook — Multi-Client SOC-Analyst Live Demo

> **Hard gates:** This runbook is authored against the merged feature set described in
> DEMO-SCOPE.md. Two stories remain in-flight as of 2026-06-22:
> S-DEMO-PRISMQL-ONBOARDING-001-B (PR #198, MERGED develop@5504c152) and S-5.04 (TDD-ready v2.1,
> code HEAD 38b2726b, LOCAL streak 0/3). Steps that depend on 001-B or S-5.04 are marked "VERIFY IN DRY-RUN" where
> the integration path has not been exercised end-to-end, and "PENDING S-5.04 MERGE"
> where the feature is not yet in code.
>
> **Binding invariant — DTU-EVERYTHING (D-1163):** Every data source in this demo
> is a prism DTU behavioral clone. No real third-party API connections. Read-only.
>
> **Out of scope — TDE/write-back:** Any action that writes back to a sensor
> (containment commands, rule deployment, alert closure) is DEFERRED to the
> prism-operations crate. Do NOT demonstrate or imply write-back capability.

---

## 1. Demo Frame and Setup

### 1.1 What Prism Is (30-second framing for the recording)

Prism runs as a **per-analyst MCP server** inside Claude Code. The analyst speaks
PrismQL — a SQL-flavored query language — against named sensor tables. Prism is a
**single process with a multi-threaded tokio runtime**: when a query executes, it fans
out to all relevant vendor APIs **in parallel** (bounded by MAX_FANOUT_CONCURRENCY=10
concurrent fan-out tasks, nested under HTTP_SEMAPHORE_PERMITS=200 global HTTP permits).
Concurrent queries from the same analyst session are **not serialized by any lock** —
`PrismServer::query` takes `&self` and config is read lock-free via ArcSwap. Prism
normalizes every response to OCSF + protobuf format and returns a unified result.
The analyst never writes HTTP clients. The analyst never manages per-vendor auth by hand.
Prism does it.

> **Concurrency note for demo presenters:** The only sequential aspect of query
> execution is the stdio transport's message framing — one MCP client request is sent
> at a time by Claude Code's stdio client. This is a transport/client characteristic,
> not engine serialization, and it does not make sensor fan-out sequential. Within a
> single query, sensors are fetched in parallel.

In this demo: three client organizations, each with a different sensor combination,
all under management from one analyst workstation. Every "vendor API" is a prism DTU
behavioral clone running on localhost — the demo is fully air-gapped and reproducible.

### 1.2 Three Client Orgs and Their Sensor Combos

Defined in `scripts/demo.toml`. These are the canonical demo organizations:

| Org slug | org_id (UUID) | Sensors configured | Seed |
|----------|---------------|-------------------|------|
| `org-a` | `0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000` | CrowdStrike, Armis | 100 |
| `org-b` | `0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001` | Claroty, Cyberint | 150 |
| `org-c` | `0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002` | CrowdStrike, Armis, Claroty, Cyberint | 200 |

Distinct seeds satisfy INV-DISTINCT-DATA-001: `devices(org-a) ∩ devices(org-c) = ∅`
even though both have CrowdStrike. Per-client isolation holds across all queries.

### 1.3 Six-Sensor DTU Fleet

| DTU clone | Role | Orgs that have it |
|-----------|------|-------------------|
| `prism-dtu-crowdstrike` | Endpoint detection, device inventory | org-a, org-c |
| `prism-dtu-armis` | IoT/OT device inventory, alerts | org-a, org-c |
| `prism-dtu-claroty` | OT device inventory, audit log | org-b, org-c |
| `prism-dtu-cyberint` | External threat intelligence alerts (with IOC fields) | org-b, org-c |
| `prism-dtu-threatintel` | IOC lookup enrichment source | all orgs (via enrich) |
| `prism-dtu-nvd` | CVE lookup enrichment source | all orgs (via enrich) |

All six clones are started by `start-multi` in a single process via
`MultiInstanceHarness`. Each org gets its own per-sensor socket.

### 1.4 The Unfolding Attack Scenario

Each org's data is seeded with a `CompromisedEndpoint` scenario that progresses
deterministically over wall-clock time from scenario start. Stage thresholds:

| Stage | Name | Activates after | What becomes visible |
|-------|------|-----------------|---------------------|
| 0 | Baseline | 0s | Normal device inventory; no attacker activity |
| 1 | Recon | 60s | Compromised device `dev-<hex>-<seed>-0` appears in sensor data |
| 2 | Lateral Movement | 180s | Lateral spread devices appear; `ioc_hashes` surface on CrowdStrike detections |
| 3 | Exfil | 360s | IOC IPs and domains appear; Cyberint alerts fire with `iocs_value`/`iocs_type` fields |
| 4 | Containment | 600s | All IOC fields visible; containment-related isolation flags set |

The same device ID appears coherently across **CrowdStrike, Armis, and Claroty** for
the same org at the same stage (cross-DTU entity coherence, BC-2.06.019). The IOC
values seeded into CrowdStrike detections and Cyberint alerts are pre-loaded into the
ThreatIntel DTU as Malicious (threat score >= 75) and the CVE IDs are pre-loaded into
the NVD DTU at HIGH CVSS 8.1 (BC-2.06.020). All synthetic CVEs use
`CVE-9999-NNNN` format — never a real advisory number.

**For the demo recording:** start scenario clock at least 6 minutes before recording
so Stage 4 (Containment) is visible. If you want to show the "growing picture"
narrative in real time, start at Stage 1 (60s after start) and let the recording run
through Stage 3.

### 1.5 How to Launch the Fleet and Prism

**Step 1 — One-time setup (first run only, idempotent):**

```bash
bash scripts/demo-setup.sh
```

This builds both binaries in release mode, creates the `~/.config/prism-demo/`
directory structure, writes a 3-org `prism.toml`, and bootstraps 10 keyring
credentials (3 for org-a, 2 for org-b, 5 for org-c). Safe to re-run.

**Step 2 — Start the DTU fleet + generate overlay TOMLs:**

```bash
bash scripts/demo-run.sh
```

This calls `prism-dtu-demo-server start-multi --config scripts/demo.toml` in the
background (one process, all orgs × sensors), polls for the `urls-multi.json` nested
sidecar, then writes per-org overlay TOMLs at
`~/.config/prism-demo/specs/customers/<org_slug>/<sensor_id>.sensor.toml`. Each
overlay contains `extends`, `instance_id`, and `base_url` pointing to the ephemeral
localhost port for that org's clone. Prints the command to start prism-bin.

**Step 3 — Start prism (as instructed by demo-run.sh output):**

```bash
# The exact command is printed by demo-run.sh. It will look like:
target/release/prism start --config ~/.config/prism-demo/prism.toml
```

**Step 4 — Wire prism into Claude Code as an MCP server:**

In Claude Code's MCP configuration (`~/.claude/claude_desktop_config.json` or
project `.mcp.json`), add:

```json
{
  "mcpServers": {
    "prism": {
      "command": "/path/to/target/release/prism",
      "args": ["start", "--config", "/Users/<you>/.config/prism-demo/prism.toml"]
    }
  }
}
```

Prism communicates over stdio. Claude Code connects at startup and the prism MCP
tools appear in Claude's tool list.

**VERIFY IN DRY-RUN:** Confirm `start-multi` subcommand is present in the merged
binary (`prism-dtu-demo-server start-multi --help`). The `start-multi` subcommand was
delivered in S-DEMO-LAUNCHER-CONSOLIDATION-001 (T11, merged PR #190
`develop@c3ecf6c8`).

**VERIFY IN DRY-RUN:** Confirm `demo-run.sh` correctly waits for `urls-multi.json`
(not the flat `urls.json` sidecar), generates overlay TOMLs in the right paths, and
that prism-bin reads them at boot step 4c.

---

### 1.6 Pre-Flight Audit (Go/No-Go Gate)

Run the pre-flight audit after `demo-run.sh` reports the fleet up — before executing
any analyst-walkthrough queries in §3. It is the go/no-go gate for the recording
session.

**How to invoke:**

`demo-run.sh` prints `PRISM_THREATINTEL_BASE_URL` and `PRISM_NVD_BASE_URL` as part of
the multi-line prism start command block it outputs (the `==>  To start prism` block).
Copy those two values verbatim — DTU ports are ephemeral and change on every
`demo-run.sh` invocation. Never reuse port values from a previous run.

```bash
# Set these from the values printed by demo-run.sh in the prism start command:
export PRISM_THREATINTEL_BASE_URL=http://127.0.0.1:<PORT-from-demo-run.sh>
export PRISM_NVD_BASE_URL=http://127.0.0.1:<PORT-from-demo-run.sh>

# PRISM_BIN resolves to target/release/prism (repo-relative) by default.
# Override only if the release binary is at a non-default path:
# export PRISM_BIN=/path/to/target/release/prism

python3 scripts/t13-preflight-audit.py
```

**Env-var precedence (`scripts/t13-preflight-audit.py` docstring):**

- `PRISM_THREATINTEL_BASE_URL` / `PRISM_NVD_BASE_URL` — full URLs; passed through
  verbatim. This is the correct path for demo runs (set from `demo-run.sh` output).
- `PRISM_THREATINTEL_PORT` / `PRISM_NVD_PORT` — bare port numbers; wrapped as
  `http://127.0.0.1:PORT`. Accepted as a lower-precedence alternative.
- Built-in defaults (54646 / 54647) — fixed only for static lab setups that do not
  run `demo-run.sh`. Almost always wrong for demo environments with ephemeral ports.

**Exit-code capture discipline:** When piping output through `tee`, `$?` captures
`tee`'s exit code (always 0), not the script's exit code. To log and preserve the
real exit code, use either form:

```bash
# Redirect to file — $? is the script's exit code:
python3 scripts/t13-preflight-audit.py > /tmp/prism-audit.log 2>&1
echo "EXIT=$?"

# Pipe to tee — use PIPESTATUS[0], not $?:
python3 scripts/t13-preflight-audit.py | tee /tmp/prism-audit.log
echo "EXIT=${PIPESTATUS[0]}"
```

**Interpreting the results:**

The script exercises a 106-check coverage matrix across 8 sections (A–H): tool
catalog, sensor adapter tables, query modes, scenario stage determinism, multi-client
isolation, enrichment correlation, error taxonomy paths, and regression probes. Each
check prints `PASS`, `FAIL`, or `WARN` with its check ID (e.g., `[A1]`, `[B3]`,
`[H24]`).

The final output line is the verdict:

| Verdict | Exit code | Action |
|---------|-----------|--------|
| `DEMO-READY: YES` | 0 | Proceed to the analyst walkthrough (§2 / §3). |
| `DEMO-READY: NO` | 1 | **STOP. Do not record.** Triage the `FAIL` rows — each carries a check ID that maps to a specific behavior in `scripts/t13-preflight-audit.py`. |

A healthy fleet passes 106/106 checks. There are no expected failures —
`DEMO-READY: NO` always indicates a real problem in the fleet that must be resolved
before recording.

---

## 2. Narrative Arc

The demo tells one coherent investigation story across three clients. Wall-clock time
advances the scenario. The analyst is discovering a coordinated threat campaign — the
same attack pattern appearing across different client environments.

### Act 1 — Orientation (5 min)

The analyst starts by orienting Claude: "what can you see?" Claude uses `prism_describe`
to enumerate all three clients' sensor tables and column schemas. This establishes:
(a) what orgs exist, (b) which sensors each has, (c) what fields are queryable.
Claude writes its first PrismQL queries against the discovered schema — no human
hand-holding required.

### Act 2 — First Anomaly in org-c (10 min)

The analyst queries org-c (the most instrumented client — all four sensors). At Stage 1
(Recon), a previously-unseen device appears in Armis and CrowdStrike. The analyst
correlates: same device ID across both sensors. Cross-sensor correlation of a single
compromised endpoint tells a richer story than either sensor alone.

### Act 3 — Cross-Client Pivot (8 min)

The analyst checks whether org-a shows the same device signature. It does not — org-a
and org-c have disjoint entity sets (different seeds). This is the multi-client
isolation proof: per-client data is genuinely distinct, not just routed differently.
Org-b (Claroty + Cyberint only) shows a different IoT profile — different threat
surface, different query pattern.

### Act 4 — IOC Enrichment (10 min)

At Stage 3 (Exfil), Cyberint alerts for org-b and org-c surface with IOC fields
(`iocs_value`, `iocs_type`). CrowdStrike detection behaviors for org-c
surface with `behaviors_ioc_type`/`behaviors_ioc_value` on the `behaviors` array. The analyst enriches
in-prism via `| enrich threat_score(iocs_value_first)` (registered UDF name from `threatintel.infusion.toml`)
and `| enrich cvss_base_score(device_cves_first)` (registered UDF name from `nvd.infusion.toml`).
The ThreatIntel DTU resolves every scenario IOC as Malicious (score >= 75). The NVD
DTU resolves every scenario CVE at HIGH CVSS 8.1. The enrichment flows through the
real prism code path — DataFusion UDF → InfusionRegistry → WASM plugin → DTU HTTP.

### Act 5 — Sensor Health Check (3 min)

PENDING S-5.04 MERGE. The analyst runs `check_sensor_health` to verify all DTU clones
are live and reachable before asserting conclusions. This exercises the live-probe path
(not spec-only). Confirms `probe_level: "live"`, `reachable: true`, `auth_valid: true`
for all configured sensors.

### Act 6 — Scope and Containment (5 min)

At Stage 4 (Containment), the full blast radius is visible: primary device, lateral
spread devices, all IOC types. The analyst queries Claroty's audit log for org-c to
see OT-side activity. The investigation closes with a complete picture across four
sensor types for one client. NOTE: write-back / containment commands are NOT
demonstrated. The demo ends at data-read.

---

## 3. Scripted Step-by-Step Walkthrough

### Setup Step — Before Recording

Verify the scenario clock is at the desired stage. The scenario is deterministic:
`elapsed = now - scenario_start_secs`. To be in Stage 3 (Exfil, elapsed >= 360s),
start the fleet at least 6 minutes before recording. Stage 4 (Containment,
elapsed >= 600s) requires 10 minutes.

VERIFY IN DRY-RUN: Confirm `scenario.enabled = true` and `scenario_start_secs` are
set in the DTU config that `start-multi` reads from `scripts/demo.toml`. The
`ScenarioConfig` fields are consumed by Story B's `build_clone_pairs` entry point.
If `scenario.enabled` is false, all clones serve static fixture data (no stage
progression).

---

### Query Block 1 — Capability Discovery and Schema Orientation

**Step 1.1 — List all MCP capabilities for org-c**

Analyst prompt to Claude:

> "Use list_capabilities for org-c to show me what prism can do for this client."

MCP call: `list_capabilities(client_id: "org-c")`

Expected return (BC-2.10.011 tri-state model, S-5.02 merged):
- `client_registered: true`
- Per-capability tri-state entries: `enabled_count`, `runtime_disabled_count`,
  `compile_time_disabled_count`
- Tools available: `query_execute`, `list_capabilities`, `prism_describe`,
  `check_sensor_health`, `reload_config`, `list_sensor_specs`, `add_sensor_spec`

**What it demonstrates:** Capability discovery. Claude learns what tools are available
for this client before issuing any query. The tri-state model shows not just what's
enabled but why something might be disabled (feature flag vs. compile gate).

**Talking point:** "Prism exposes a self-describing capability surface. Claude doesn't
need to know the tool list in advance — it discovers it. This is how Claude handles
new sensor types without a prompt rewrite."

---

**Step 1.2 — Discover schema for org-c**

Analyst prompt to Claude:

> "Use prism_describe to show me all the tables and columns available for org-c."

MCP call: `prism_describe(client_id: "org-c")`

Expected return (BC-2.10.012, S-DEMO-PRISMQL-ONBOARDING-001-A merged PR #197):
```json
{
  "client_id": "org-c",
  "tables": [
    {
      "name": "crowdstrike_detections",
      "columns": [
        {"name": "device_id", "type": "String"},
        {"name": "behavior_id", "type": "String"},
        {"name": "behaviors_ioc_type", "type": "String"},
        {"name": "behaviors_ioc_value", "type": "String"},
        ...
      ]
    },
    {
      "name": "armis_devices",
      "columns": [...]
    },
    {
      "name": "claroty_devices",
      "columns": [...]
    },
    {
      "name": "cyberint_alerts",
      "columns": [
        {"name": "iocs_value", "type": "String"},
        {"name": "iocs_type", "type": "String"},
        {"name": "severity", "type": "String"},
        ...
      ]
    }
  ],
  "pql_hints": ["Use | enrich threat_score(column) to enrich IOC values with threat score", ...]
}
```

**What it demonstrates:** Per-client schema discovery (DI-008 isolation — org-c sees
all 4 sensors; org-a would see only CrowdStrike + Armis tables). Claude now knows
exactly what column names exist before writing any query. The `pql_hints` field
guides Claude toward the enrichment path.

**Talking point:** "prism_describe is the bridge between sensor TOML specs and Claude's
query authorship. Claude enumerates tables, sees real column names, and writes correct
PrismQL on the first try. No hallucinated field names."

**VERIFY IN DRY-RUN:** Confirm `iocs_value`, `iocs_type`, and `severity` appear in the
`cyberint_alerts` column list. These are the ENRICH-1 clean column names (S-DEMO-ENRICH-1) for the IOC fields originally introduced in PIVOT-003 (PR #196 `develop@f6739764`) using bracket-in-name convention. ENRICH-1 renames them to clean SQL identifiers with source_path. Note: `severity` is the top-level alert severity field; there is no separate `ioc_severity` column.

---

**Step 1.3 — Discover schema for org-a and compare**

Analyst prompt to Claude:

> "Now show me the schema for org-a."

MCP call: `prism_describe(client_id: "org-a")`

Expected return: Only `crowdstrike_detections` and `armis_devices` tables. Cyberint
and Claroty tables are absent because org-a does not have those sensors configured.

**What it demonstrates:** Per-client schema isolation. The same analyst, same prism
server, different client context returns a scoped view. DI-008 (client isolation) in
action.

**Talking point:** "Org-a has CrowdStrike and Armis. Org-b has Claroty and Cyberint.
Org-c has all four. One prism server, one analyst, genuinely different sensor
topologies per client. The schema reflects reality — not a generic template."

---

### Query Block 2 — Cross-Sensor Correlation (Stage 1+)

Requires the scenario to be at Stage 1 (elapsed >= 60s).

**Step 2.1 — Find the compromised device in CrowdStrike for org-c**

```sql
FROM crowdstrike_detections
WHERE behaviors_ioc_type IS NOT NULL OR device_id LIKE 'dev-%'
LIMIT 20
client_id = "org-c"
```

Expected at Stage 1+: Returns rows including `device_id` starting with
`dev-<hex>-200-0` (the primary compromised device for org-c, seed=200). The `<hex>`
prefix is derived from org-c's UUID bytes (`0196f4b2...` → first 4 bytes in hex).

**VERIFY IN DRY-RUN:** Confirm the device ID format returned by the CrowdStrike clone
for org-c matches the pattern `dev-<first-4-bytes-of-org-id-hex>-200-0`.

**What it demonstrates:** Prism federates the query to org-c's CrowdStrike DTU clone,
normalizes the response to OCSF, and returns structured results.

**Talking point:** "Notice the device ID. CrowdStrike returns its own proprietary
detection format. Prism normalizes it to OCSF. The analyst queries a uniform schema
regardless of which vendor is underneath."

---

**Step 2.2 — Correlate the same device in Armis for org-c**

```sql
FROM armis_devices
WHERE device_id = '<device_id_from_step_2_1>'
client_id = "org-c"
```

(Claude fills in the device ID from Step 2.1.)

Expected at Stage 1+: Returns the same device with Armis-side fields (device type,
OS, IP address, risk score).

**What it demonstrates:** Cross-sensor correlation. The same device ID appears in
both CrowdStrike and Armis for org-c. This is BC-2.06.019 cross-DTU entity coherence:
`primary_device_id_cs` and `primary_device_id_armis` in `ScenarioEntityCatalog` use
the same derivation, so both sensors surface the same logical device.

**Talking point:** "One compromised endpoint — two sensors, two perspectives. CrowdStrike
sees it as a detection. Armis sees it as an asset. Prism correlates them without any
custom glue code. The analyst asks one question, prism fans out to both sensors in
parallel — single process, async execution, no lock between them."

---

**Step 2.3 — Check org-a for the same device pattern**

```sql
FROM crowdstrike_detections
WHERE device_id LIKE 'dev-%'
LIMIT 10
client_id = "org-a"
```

Expected: Returns a different set of device IDs (prefix derived from org-a's UUID +
seed=100). The device IDs from org-c do NOT appear.

**What it demonstrates:** Per-client data isolation (BC-2.06.018, INV-DISTINCT-DATA-001).
Org-a's CrowdStrike clone has a completely disjoint entity set from org-c's. The same
attack pattern does NOT bleed across clients.

**Talking point:** "Org-a and org-c both have CrowdStrike, but they see completely
different data. Multi-tenancy is not just routing — it's genuine data isolation.
Prism enforces this at the seed level, not just the routing level."

---

**Step 2.4 — Check org-b (Claroty + Cyberint only)**

```sql
FROM claroty_devices
LIMIT 10
client_id = "org-b"
```

Expected: Returns org-b's OT/ICS device inventory. Different device types from
CrowdStrike/Armis (industrial controllers, PLCs, sensors).

**Talking point:** "Org-b is an OT/ICS environment. No CrowdStrike, no Armis —
Claroty and Cyberint are their sensors. Prism serves the right sensor combination
per client. The analyst's workflow doesn't change; only the data does."

---

### Query Block 3 — IOC Enrichment (Stage 3+)

Requires the scenario to be at Stage 3 (elapsed >= 360s). IOC values are visible in
Cyberint alerts and CrowdStrike detection behaviors.

**Step 3.1 — Find Cyberint alerts with IOC fields for org-c**

```sql
FROM cyberint_alerts
WHERE iocs_value IS NOT NULL
LIMIT 10
client_id = "org-c"
```

Expected at Stage 3+: Returns alert rows with `iocs_value` (IP address, domain, or
hash string; wildcard source_path `$.iocs[*].value` → JSON-list string, e.g., `["hash1","hash2"]`) and `iocs_type` (e.g., `"[\"ip\"]"`, `"[\"domain\"]"`, `"[\"hash_sha256\"]"`) populated.
The top-level `severity` column carries the alert severity. These are real-schema fields
added by PIVOT-003 and surfaced via ENRICH-1 clean column names with source_path extraction (note: the wire-level serde alias `ioc_type`/`ioc_value` is resolved inside the DTU Rust struct; the queryable PrismQL column names are `iocs_value` and `iocs_type` per the TOML spec).

**What it demonstrates:** IOC fields are now first-class columns in the TOML sensor
spec, not synthetic filters. The TOML column declarations match the DTU Alert struct
fields exactly (SAP-2 compliance, BC-2.06.019 Per-Sensor IOC-Surface Matrix).

**Talking point:** "Cyberint returns real IOC data — IP addresses, domains, hashes —
in its native alert format. Prism exposes them as typed columns. The analyst can filter,
join, or enrich on them directly in PrismQL."

**VERIFY IN DRY-RUN:** Confirm `iocs_value` column returns non-null values at Stage 3.
If null at Stage 3, the scenario clock may not have advanced far enough, or the
ENRICH-1 `source_path = "$.iocs[*].value"` extraction may not be working (check for `column_source_path_extraction_failed` warn events in the prism log).

---

**Step 3.1a — Filter alerts by severity using case-insensitive IEQ / IIN (ADR-047)**

```sql
FROM crowdstrike_detections
| where severity IEQ 'critical'
| limit 50
client_id = "org-c"
```

Expected: Returns CrowdStrike detection rows where OCSF `severity` is `'Critical'`
(the canonical Title-case form stored after adapter-boundary normalization; the
CompromisedEndpoint scenario seeds Critical and Medium severity for CrowdStrike
detections). The query matches whether the analyst types `'critical'`, `'CRITICAL'`,
or `'Critical'` — `lower(severity) = lower('critical')` is the DataFusion lowering.

To match multiple severity levels in a single predicate:

```sql
FROM cyberint_alerts
| where severity IIN ('high', 'critical')
| limit 20
client_id = "org-c"
```

Expected at Stage 3+: Returns Cyberint alerts where `severity` is `'High'` or
`'Critical'` (stored canonical Title-case forms). `IIN` lowers every value in the
membership list: `lower(severity) IN (lower('high'), lower('critical'))`.

To filter on alert status with the same casing-proof approach:

```sql
FROM cyberint_alerts
| where status IIN ('open', 'closed')
| limit 20
client_id = "org-c"
```

Expected: Returns Cyberint alerts where `status` is `'open'` or `'closed'`.
Cyberint status values (`'open'`, `'acknowledged'`, `'closed'`) are vendor-native
lifecycle identifiers — they do not case-insensitively match any OCSF status caption
(generic: Unknown, Success, Failure, Other; finding-class: New, In Progress, Suppressed,
Resolved, Archived, Deleted), so they pass through the adapter boundary as-is per
BC-2.02.013 RG-021 (same class as Claroty `'Unresolved'` and Armis `'UNHANDLED'`;
DRIFT-AUDIT-RUNBOOK-LITERALS-001 D-1609 adjudication 2026-07-16; documented in
BC-2.02.013 EC-02-029). IIN remains casing-proof against the actual stored
values: `lower('open')` matches `lower('open')`. If a future Cyberint adapter
version aligns to OCSF and normalizes these values, the analyst query would need
updating — but for the current data the vendor-native values are the correct
literals to use.

**Teaching note:** OCSF enum-label fields (`severity`, `status`, `activity_name`,
`disposition`) are normalized to canonical OCSF Title-case at ingestion by
`build_column_array` in the spec-driven adapter (`enum_map.rs` is the casing
authority) — **when the vendor value case-insensitively matches an OCSF caption**.
Example: `severity='HIGH'` → stored as `'High'` because `'High'` is an OCSF
`severity_id` caption. `IEQ`/`IIN`/`INE` provide a safety net for values that were
not normalized (vendor-native pass-throughs) and for free-form non-enum fields.

For Cyberint `status`, the vendor values are `'open'`, `'acknowledged'`, and
`'closed'` — none of these match any OCSF status caption, so they pass through
as-is. `status IIN ('open', 'closed')` works correctly because IIN lowercases both
sides: the stored `'open'` matches `lower('open')` = `'open'`. The casing-proof
guarantee holds whether the field is OCSF-normalized (e.g., `severity`) or vendor
pass-through (e.g., Cyberint `status`).

The default `=` remains case-sensitive for precision filtering (e.g., process names,
file paths, registry keys where exact case is security-meaningful).

**Note on typed column guidance:** `severity_id IEQ 'high'` where `severity_id` is
an integer column returns `E-QUERY-002 (QueryTypeMismatch)` — `lower()` is not
applicable to integers. The error message suggests using the string sibling `severity`
instead. This is how prism steers Claude away from the `_id` integer companion and
toward the string column.

**Talking point:** "The analyst types lowercase severity values because that's the
natural language. Prism's `IEQ`/`IIN` operators absorb the casing difference. Under
the hood, OCSF enum labels are normalized to Title-case at ingestion — so `GROUP BY
severity` across CrowdStrike, Armis, and Cyberint produces one `High` bucket, not
separate `HIGH` and `High` buckets. The analyst's query doesn't need to know which
vendor emitted what casing."

**VERIFY IN DRY-RUN:** Confirm `severity` column appears in `crowdstrike_detections`
and `cyberint_alerts` via `prism_describe(client_id: "org-c")`. Confirm `status`
appears in `cyberint_alerts` before using `status IIN` in the recording. If `severity`
is absent from a table's schema, omit that table from this beat.

---

**Step 3.2 — Enrich the IOC values against ThreatIntel**

```sql
FROM cyberint_alerts
| where iocs_value_first IS NOT NULL
| enrich threat_score(iocs_value_first)
| limit 10
client_id = "org-c"
```

Expected: Each row gains a `threat_score` column (value >= 75, Malicious). The ThreatIntel
DTU clone resolves every scenario IOC as Malicious. To additionally get the known-malicious
boolean, add `| enrich threat_is_known_malicious(iocs_value_first)`. To get source names, add
`| enrich threat_sources(iocs_value_first)`. Each field is a separate registered UDF
(per `threatintel.infusion.toml` `[[infusion.fields]]` entries: `threat_score`,
`threat_is_known_malicious`, `threat_sources`).

**What it demonstrates:** The full in-prism enrichment path (BC-2.19.001/003):
`| enrich` PrismQL pipe → DataFusion ScalarUDF (registered by S-DEMO-ENRICHMENT-PIVOT-001)
→ InfusionRegistry → PluginInfusionSource → WASM plugin (`prism-threatintel-infusion.prx`)
→ DTU HTTP endpoint (`prism-dtu-threatintel`). This is the real prism code path. DTU
clones are the only substituted element.

**Talking point:** "Enrichment is not a post-processing step in a SIEM. It's a
first-class PrismQL operator. The analyst writes | enrich just like a SQL JOIN, and
prism fans out to the enrichment source. ThreatIntel scores, CVE details, geolocation —
all available inline in the query."

**VERIFY IN DRY-RUN:** Confirm the DataFusion UDF `threat_score` is registered and
returns data from the DTU HTTP endpoint. The WASM plugin path (PIVOT-002) or the
fallback direct `reqwest` path (D-1164 contingency) must be active. If `| enrich`
returns E-QUERY-039 "unknown enrichment function" or similar, the InfusionRegistry is
not loaded — check that `threatintel.infusion.toml` is in the specs directory and
that S-1.14-REDO + PIVOT-001/002 are merged. The registered UDF names are
`threat_score`, `threat_is_known_malicious`, and `threat_sources` (one per
`[[infusion.fields]]` entry) — NOT a single `threat_intel` function.

---

**Step 3.3 — Find CrowdStrike detection behaviors with IOC hashes (Stage 2+)**

```sql
FROM crowdstrike_detections
| where behaviors_ioc_type IS NOT NULL
| fields device_id, behaviors_ioc_type, behaviors_ioc_value, behaviors_ioc_description
client_id = "org-c"
```

Expected at Stage 2+: Returns rows with `behaviors_ioc_type` values like
`"hash_sha256"` or `"hash_md5"`, and corresponding `behaviors_ioc_value` containing
the hash string. Note: CrowdStrike behaviors carry hashes and domains — NOT IP
addresses (BC-2.06.019 Per-Sensor IOC-Surface Matrix; `ipv4`/`ipv6` IOC types do not
appear on CrowdStrike detection behaviors).

**Talking point:** "CrowdStrike's detection behaviors carry IOC context from the
agent's own analysis. Prism surfaces that as typed columns. The hash here is the same
IOC that Cyberint flagged — the same attacker infrastructure, observed from two
different sensor perspectives."

---

**Step 3.4 — Enrich the CrowdStrike IOC hash against ThreatIntel**

```sql
FROM crowdstrike_detections
| where behaviors_ioc_type = 'hash_sha256'
| enrich threat_score(behaviors_ioc_value_first)
| limit 5
client_id = "org-c"
```

Expected: Hash values resolve in ThreatIntel with `threat_score >= 75`.

**What it demonstrates:** The enrichment works on any column that contains an IOC
value — whether it originates from Cyberint alerts or CrowdStrike detection behaviors.
The same `| enrich threat_score(...)` UDF is sensor-agnostic.

---

**Step 3.5 — Enrich CVEs from Armis devices against NVD**

```sql
FROM armis_devices
| where device_cves_first IS NOT NULL
| enrich cvss_base_score(device_cves_first)
| enrich cvss_severity(device_cves_first)
| limit 5
client_id = "org-c"
```

Expected: Each row gains `cvss_base_score` (8.1) and `cvss_severity` (`"HIGH"`).
The NVD DTU clone returns HIGH CVSS for all scenario CVEs (`CVE-9999-NNNN` format —
collision-safe, never matches a real advisory). The registered NVD UDFs are
`cvss_base_score`, `cvss_severity`, and `cvss_vector` (one per `[[infusion.fields]]`
entry in `nvd.infusion.toml`). The input field is `device_cves_first` (scalar String
projected per Ruling 1b / BC-2.06.019 §PC-4 — NOT `cve_id`, NOT `device_cves`).

**What it demonstrates:** The NVD enrichment path (HttpLookup path, ADR-040 →
prism-dtu-nvd DTU). Same `| enrich` syntax, different enrichment source.

**Talking point:** "Org-c has Armis for IoT/OT device inventory. The device record
includes a CVE reference. The analyst enriches it against NVD in the same query.
CVSS 8.1 HIGH. No context switching, no copy-paste into a browser — the vulnerability
context is right there."

**N4 correction note:** `client_id = "org-c"` is required — org-b does NOT have Armis
(org-b is Claroty + Cyberint only; see §1.2 org topology). org-a and org-c both have
Armis; org-c is used here because it is the four-sensor reference client for this demo
arc.

---

### Query Block 4 — Dynamic Table Availability (S-3.13)

**Step 4.1 — Query a table that org-a does not have**

```sql
FROM claroty_devices
LIMIT 5
client_id = "org-a"
```

Expected: Returns `E-QUERY-037` error (dynamic table availability check — BC-2.16.007,
S-3.13 merged PR #192 `develop@60249ccc`). Error message includes the available
tables for org-a and a suggestion to use `prism_describe` to check available tables.

**What it demonstrates:** Prism fails at plan time when a query references a sensor
the client does not have configured, rather than returning empty results silently.
The error message is pedagogical — it tells Claude which tables ARE available.

**Talking point:** "If Claude asks for a table that doesn't exist for this client,
prism fails early with a helpful message. Claude learns from the error and retries
with a valid table. This is how prism teaches Claude to write correct PrismQL without
human intervention."

---

**Step 4.2 — Query a column that does not exist (E-QUERY-038)**

```sql
FROM crowdstrike_detections
SELECT device_id, nonexistent_column
LIMIT 5
client_id = "org-c"
```

Expected: Returns `E-QUERY-038` error (plan-time column gate — BC-2.11.016, delivered
by S-DEMO-PRISMQL-ONBOARDING-001-B). Error message names the invalid column and lists
valid alternatives. Response includes `normalized_pql` field showing the normalized
form of the original query.

**What it demonstrates:** Column-level validation at plan time. Claude self-corrects
by reading the error's pedagogical context and the normalized PQL hint.

**VERIFY IN DRY-RUN:** This step requires S-DEMO-PRISMQL-ONBOARDING-001-B to be
merged. If 001-B is not yet merged at recording time, replace with an E-QUERY-037
table-availability demonstration only.

---

**Step 4.3 — SQL mode rejects IEQ: pedagogical E-QUERY-001 (mode-boundary error-UX beat)**

```sql
SELECT severity, count(*) FROM cyberint_alerts WHERE severity IEQ 'high' GROUP BY severity
```

Expected: Returns `E-QUERY-001` — structured parse-time rejection with message:
`"E-QUERY-001: parse error near 'IEQ': case-insensitive operators (IEQ/IIN/INE) are
not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe
| where stage (e.g., FROM cyberint_alerts | where severity IEQ 'high' | limit 20)
instead."` MCP mapping: `-32602 INVALID_PARAMS` (caller-resolvable).

`IEQ`/`IIN`/`INE` are PrismQL extensions; raw SQL mode (`SELECT … FROM …`) has no
knowledge of them and rejects them at parse time — before DataFusion planning — with
a structured error that names the correct syntax. Claude reads the message, switches
to filter or pipe mode, and retries correctly without human intervention.

**What it demonstrates:** PrismQL's mode-boundary enforcement. The teaching surface
is complete: `prism_describe` teaches *what* columns exist; pedagogical errors like
E-QUERY-037, E-QUERY-038, and E-QUERY-001 teach *how* to query them. The analyst
never needs to hand-hold Claude on syntax after the first attempt.

**Talking point:** "IEQ and IIN are pipe-mode and filter-mode operators — not standard
SQL. When Claude tries them in a SQL `WHERE` clause, prism catches it at parse time
and tells Claude exactly what syntax to use instead. The error is the tutorial."

---

### Query Block 5 — Sensor Health Check

**PENDING S-5.04 MERGE — Include this block only after S-5.04 merges.**

**Step 5.1 — Check sensor health for org-c**

Analyst prompt to Claude:

> "Check the health of all sensors for org-c before we draw any conclusions."

MCP call: `check_sensor_health(client_id: "org-c")`

Expected return (BC-2.08.005, live-probe path):
```json
{
  "client_id": "org-c",
  "sensors": {
    "crowdstrike": {
      "sensor_id": "crowdstrike",
      "probe_level": "live",
      "reachable": true,
      "auth_valid": true,
      "overall_status": "healthy"
    },
    "armis": {
      "sensor_id": "armis",
      "probe_level": "live",
      "reachable": true,
      "auth_valid": true,
      "overall_status": "healthy"
    },
    "claroty": { ... "overall_status": "healthy" },
    "cyberint": { ... "overall_status": "healthy" }
  },
  "overall_status": "healthy"
}
```

All four sensors live and reachable because the DTU clones are running. `probe_level`
is `"live"` (not `"spec-only"`), confirming an actual API probe was made — not a
cached status.

**What it demonstrates:** The sensor health subsystem. An MSSP analyst verifies data
quality before drawing conclusions from a query. If a sensor is down, the analyst
knows the query result may be incomplete.

**Talking point:** "Before acting on what we've found, the analyst verifies the
sensors are actually talking. Prism makes a live probe — a real HTTP call to each DTU
— and reports reachability, auth status, and rate-limit state. Incomplete data is
caught before it leads to a false conclusion."

**Note:** The `probe_table` field (folded into S-5.04 per D-1262) means the health
probe uses the TOML-declared `probe_table` as the fetch target, not a hardcoded table
name. Cyberint's health probe correctly uses `alerts` (not `devices`) because the
Cyberint spec declares `probe_table = "alerts"`.

---

### Query Block 6 — Full Blast Radius at Stage 4 (Containment)

Requires elapsed >= 600s.

**Step 6.1 — Full device inventory at containment stage for org-c**

```sql
FROM armis_devices
WHERE device_id LIKE 'dev-%'
client_id = "org-c"
```

Expected at Stage 4: Returns primary device + all lateral spread devices (Stage 2+
introduced lateral devices; they remain visible at Stage 4). All device fields visible.

---

**Step 6.2 — Full IOC surface at containment stage for org-c**

```sql
FROM cyberint_alerts
| where iocs_value_first IS NOT NULL
| enrich threat_score(iocs_value_first)
client_id = "org-c"
```

Expected at Stage 4: IP addresses, domains, and hash IOCs all present in the results.
All resolve as Malicious in ThreatIntel. This is the full blast radius: all IOC types
are visible simultaneously because Stage 4 `StageMask` has all 6 fields set to `true`
(BC-2.06.019 AC-006).

---

**Step 6.3 — Claroty audit log for org-c (OT perspective)**

```sql
FROM claroty_audit_logs
LIMIT 20
client_id = "org-c"
```

Expected: Returns OT audit events including columns `action`, `actor`, `id`,
`resource`, `timestamp`. Claroty's audit log (delivered by S-DEMO-CLAROTY-AUDIT-DTU-001)
surfaces operator-side activity. At Stage 4, there may be containment-related events
visible in the audit log.

The correct FROM-ready table name is `claroty_audit_logs` (plural — derived from
`table_name = "audit_logs"` in the Claroty sensor TOML spec, prefixed with `claroty_`
per AUDIT-001 sensor-prefixed naming). The primary key column is `id` (NOT `log_id` —
the Claroty `AuditLogEntry` struct uses `id: String` per the DTU types.rs). N5
correction: earlier draft used `claroty_audit_log` (singular) and `log_id`; both were
wrong.

**Talking point:** "Org-c has an OT environment alongside its IT environment. Claroty
sees the industrial controller side. The same prism query surface covers both IT and
OT sensors. The analyst doesn't need separate tools."

---

### Query Block 7 — PrismQL Grammar Reference

**Step 7.1 — Fetch the PrismQL grammar reference resource**

Analyst prompt to Claude:

> "Read the PrismQL grammar reference."

MCP resource access: `prismql://reference`

Expected: Returns the static PrismQL grammar reference (embedded via `include_str!`
in prism-mcp at build time). Contains 7 required sections including `| enrich`
operator syntax, OCSF normalization notes, and error code reference. Delivered by
S-DEMO-PRISMQL-ONBOARDING-001-A (BC-2.10.014).

**What it demonstrates:** L3 grammar reference. Claude can read the authoritative
PrismQL spec directly from the server, not from its training data (which may be stale).

---

## 4. Expected Outputs and Talking Points Per Stage

| Stage | Key observable | Talking point |
|-------|---------------|---------------|
| Schema discovery | `prism_describe` returns per-client table+column list | "Claude learns the schema from prism, not from a hardcoded prompt" |
| Multi-client isolation | Org-a and org-c CrowdStrike return disjoint device IDs | "Multi-tenancy is data isolation, not just routing" |
| Sensor-combo scoping | `claroty_devices` returns E-QUERY-037 for org-a | "Prism fails early with a helpful error; Claude self-corrects" |
| Cross-sensor correlation | Same device ID in CrowdStrike and Armis for org-c | "One endpoint, two sensor perspectives, one PrismQL query" |
| IOC enrichment | `| enrich threat_score(iocs_value_first)` returns `threat_score >= 75` Malicious (registered UDF from `threatintel.infusion.toml`) | "Enrichment is in-query, not post-processing" |
| CVE enrichment | `| enrich cvss_base_score(device_cves_first)` returns CVSS 8.1 HIGH (registered UDF from `nvd.infusion.toml`) | "CVE context in the same query that found the alert" |
| Case-insensitive filtering | `severity IEQ 'critical'` matches rows stored as `'Critical'` (CompromisedEndpoint scenario); `severity IIN ('high', 'critical')` matches `'High'` and `'Critical'`; `status IIN ('open', 'closed')` matches Cyberint vendor-native pass-through values; `GROUP BY severity` produces at most 7 buckets — no casing fragmentation across sensors | "OCSF enum labels normalize to Title-case at ingestion; vendor-native values that don't match OCSF captions pass through as-is; IEQ/IIN absorb analyst casing in both cases" |
| Sensor health | `check_sensor_health` returns `probe_level: "live"` | "Verify data quality before drawing conclusions" |
| Full blast radius | Stage 4 shows all IOC types + lateral devices | "Ten minutes into the incident, the full picture is clear" |

---

## 5. Dry-Run Checklist

Run through this checklist top-to-bottom before the recording session. Every item
must pass before recording begins.

### 5.1 Prerequisites

- [ ] Rust workspace builds cleanly: `just check` exits 0 on develop HEAD post 001-B + S-5.04 merge
- [ ] `prism-dtu-demo-server start-multi --help` exits 0 and lists the subcommand
- [ ] `prism --version` runs without error
- [ ] Claude Code is configured with the prism MCP server (stdio transport)
- [ ] 001-B merged to develop (E-QUERY-038 + normalized_pql)
- [ ] S-5.04 merged to develop (check_sensor_health live probe)

### 5.2 Fleet Startup

- [ ] `bash scripts/demo-setup.sh` completes without error (run from repo root)
- [ ] `bash scripts/demo-run.sh` starts `prism-dtu-demo-server` in background and
      writes `urls-multi.json` sidecar within 30s
- [ ] Overlay TOMLs created for all 8 sensor×org combinations:
      `~/.config/prism-demo/specs/customers/{org-a,org-b,org-c}/<sensor>.sensor.toml`
- [ ] `prism start` boots without error; prints boot log showing 8 sensor adapters
      registered across 3 orgs (2 for org-a, 2 for org-b, 4 for org-c)

### 5.3 Per-Client Data Distinctness

- [ ] `FROM crowdstrike_detections | limit 5 client_id="org-a"` returns non-empty data
      with device IDs containing org-a's UUID hex prefix
- [ ] `FROM crowdstrike_detections | limit 5 client_id="org-c"` returns non-empty data
      with DIFFERENT device IDs than org-a
- [ ] The device IDs from org-a and org-c have ZERO overlap

### 5.4 Scenario Clock Progression

- [ ] At elapsed < 60s: `FROM armis_devices | limit 20 client_id="org-c"` returns baseline
      devices only (no `dev-<hex>-200-0` primary compromised device)
- [ ] At elapsed >= 60s: `dev-<hex>-200-0` device appears in Armis results for org-c
- [ ] At elapsed >= 180s: lateral devices appear in Armis/CrowdStrike for org-c
- [ ] At elapsed >= 360s: `iocs_value` column is non-null (returns JSON-list string) in Cyberint alerts for org-c
- [ ] At elapsed >= 600s: all IOC types (IP, domain, hash, CVE) visible simultaneously

**VERIFY IN DRY-RUN:** If stage progression is not working, confirm
`scenario.enabled = true` and `scenario_start_secs` in the DTU server config. Check
`current_stage_index` is being called with the live `Utc::now().timestamp()` in the
route handlers.

### 5.5 Enrichment Path

- [ ] `| enrich threat_score(iocs_value_first)` on a Cyberint alert at Stage 3+ returns
      `threat_score >= 75` for at least one IOC value (`iocs_value_first` is the scalar String companion extracted from the `iocs_value` JSON-list; ADR-051 D4 requires scalar input for typed UDFs). Registered UDF name: `threat_score` (from `threatintel.infusion.toml` `[[infusion.fields]]`).
- [ ] `| enrich cvss_base_score(device_cves_first)` on an Armis device record returns `cvss_base_score == 8.1` for at least one CVE. The NVD DTU injects scenario CVEs (`CVE-9999-NNNN` format) with exactly `base_score = 8.1 / base_severity = "HIGH"` per `NvdClone::new_with_scenario` (BC-2.06.020 PC-4) — the value is deterministic, not a range. Registered UDF name: `cvss_base_score` (from `nvd.infusion.toml` `[[infusion.fields]]`). Input field is `device_cves_first` (scalar String, Ruling 1b).
- [ ] Enrichment response includes the source column value (not just the enrichment fields)

**VERIFY IN DRY-RUN:** If `| enrich` returns a syntax error or unknown function,
check that `threatintel.infusion.toml` and `nvd.infusion.toml` are in the specs
directory and that the InfusionLoader loaded them at startup. The WASM plugin or
fallback HTTP path must be active. The registered UDF names are the per-field names
from `[[infusion.fields]]` entries: `threat_score`, `threat_is_known_malicious`,
`threat_sources` (ThreatIntel) and `cvss_base_score`, `cvss_severity`, `cvss_vector`
(NVD). The single-function forms `threat_intel(...)` and `nvd(...)` are NOT registered.

### 5.6 MCP Teaching Surface

- [ ] `prism_describe(client_id: "org-c")` returns a JSON response with tables and
      columns including `cyberint_alerts` table with `iocs_value`/`iocs_type`/`severity`
      columns (ENRICH-1 clean column names; note: alert-level `severity`, no separate `ioc_severity`)
- [ ] `prism_describe(client_id: "org-a")` does NOT include `claroty_devices` or
      `cyberint_alerts` tables (per-client isolation)
- [ ] `FROM claroty_devices LIMIT 5 client_id="org-a"` returns E-QUERY-037 with
      suggestion to check available tables
- [ ] `prismql://reference` resource returns non-empty grammar text with at least
      7 section headers
- [ ] `list_capabilities(client_id: "org-c")` returns `client_registered: true` and
      a non-empty capabilities map

**VERIFY IN DRY-RUN:** E-QUERY-038 column gate (001-B) — confirm
`SELECT nonexistent_column FROM crowdstrike_detections client_id="org-c"` returns an
error naming the invalid column and listing alternatives.

### 5.7 Sensor Health (PENDING S-5.04 MERGE)

- [ ] `check_sensor_health(client_id: "org-c")` returns `probe_level: "live"` (not
      `"spec-only"`) for all four sensors
- [ ] `reachable: true` and `auth_valid: true` for all four sensors (DTU clones are up)
- [ ] `overall_status: "healthy"` in the response

### 5.9 Case-Insensitive Operators (IEQ / IIN / INE)

- [ ] `FROM crowdstrike_detections | where severity IEQ 'critical' | limit 50 client_id="org-c"`
      returns rows (OCSF canonical stored form is `'Critical'`; CompromisedEndpoint scenario seeds
      Critical/Medium for CrowdStrike; IEQ matches regardless of typed casing;
      `lower(severity) = lower('critical')` is the DataFusion lowering)
- [ ] `FROM cyberint_alerts | where severity IIN ('high', 'critical') | limit 20 client_id="org-c"`
      returns rows at Stage 3+ (matches canonical `'High'` and `'Critical'`)
- [ ] `SELECT severity, count(*) FROM cyberint_alerts WHERE severity IEQ 'high' GROUP BY severity`
      (no `client_id`) returns E-QUERY-001 with a mode-boundary message pointing to filter or
      pipe `| where` syntax — NOT an opaque E-QUERY-034

**VERIFY IN DRY-RUN:** If E-QUERY-001 is not returned (e.g., you get E-QUERY-034 or
DataFusion plan failure instead), the CI-operator SQL-mode gate did not land correctly
on this binary — confirm the build is from the develop HEAD post PR #217 (`f935edb6`).

---

### 5.8 Multi-Tenant Isolation Proof

- [ ] `FROM cyberint_alerts LIMIT 5 client_id="org-a"` returns E-QUERY-032 (org-a
      has no Cyberint sensor registered for this org — sensor exists globally but is NOT
      configured for org-a; per BC-2.10.004 client scoping. E-QUERY-032 = "sensor X is not
      registered for org Y" authorization gate, NOT E-QUERY-037 "table not in TableRegistry"
      plan-time check. N3 correction: earlier draft incorrectly cited E-QUERY-037.)
- [ ] `FROM armis_devices LIMIT 5 client_id="org-b"` returns E-QUERY-032 (org-b has
      no Armis sensor registered — same authorization gate as above)

---

## 6. Honest Gaps and Out-of-Scope Items

### Things NOT shown in this demo

| Feature | Why not shown | Deferred to |
|---------|--------------|-------------|
| Write-back / containment commands | Requires `prism-operations` crate; dead write path (E-SENSOR-070 / TODO W3-FIX-S307-001) | Post-demo TDE workflow |
| PagerDuty / Jira sensor integration | Out of current enrichment scope per D-1072 | S-4.08 + TDE track |
| Alert closure / rule deployment | write-back TDE DEFERRED | S-1.15 TDE scope |
| Cross-client alias scoping (BC-2.11.009 deviation) | P2 follow-up; NOT demo-blocking; tracked in S-ALIAS-CLIENT-SCOPE-001 | After demo |
| `explain_query` output parity | P2 follow-up; tracked in S-EXPLAIN-PARITY-001 | After demo |

### Capability caveats

- **Enrichment at Stage < 3:** The `| enrich threat_score(iocs_value_first)` query will
  return results but `iocs_value_first` will be null for most rows at Stage 0-2 (the underlying
  `iocs_value` JSON-list column is empty or null at Stage 0-2, so its scalar companion
  `iocs_value_first` is also null). IOC values only populate at Stage 3+. If demonstrating enrichment, confirm elapsed >= 360s.
  Registered UDF names: `threat_score`, `threat_is_known_malicious`, `threat_sources` (ThreatIntel);
  `cvss_base_score`, `cvss_severity`, `cvss_vector` (NVD). The single-function forms
  `threat_intel(...)` and `nvd(...)` are NOT registered UDF names.

- **Device ID format in queries:** Device IDs are derived from org_id UUID bytes, NOT
  from the human-readable slug. `WHERE device_id = 'dev-org-c-200-0'` will return zero
  results. Use the hex-prefix form. VERIFY IN DRY-RUN: retrieve one device ID first,
  then use it in subsequent queries.

- **Cyberint auth:** org-b and org-c require a valid `api_key` credential in the keyring.
  The `demo-setup.sh` script bootstraps dummy keys (`demo-cyberint-api-key-org-b`,
  `demo-cyberint-api-key-org-c`) that match the `initial_access_token` values in
  `scripts/demo.toml`. If Cyberint returns 401, the keyring credential is missing or
  mismatched.

- **scenario_start_secs:** If `demo.toml` does not set `scenario_start_secs`, the DTU
  server calls `Utc::now()` once at startup. This means the scenario starts at launch
  time. To control the stage during recording, either (a) pre-start the fleet 10 minutes
  early for Stage 4, or (b) set an explicit `scenario_start_secs` in the TOML to a
  past epoch for reproducibility. VERIFY IN DRY-RUN: confirm the mechanism for
  controlling scenario start time in `scripts/demo.toml`.

- **WASM contingency (D-1164):** If the WASM plugin path (PIVOT-002) has issues at
  recording time, the fallback is `PluginInfusionSource::enrich_single` calling the
  DTU HTTP endpoint directly via `reqwest`. This contingency was accepted by the user
  per D-1164. The analyst-visible behavior is identical — only the internal execution
  path differs.

- **Cyberint IOC column naming — ENRICH-1 clean column names are required (v1.2
  correction, supersedes v1.1):** The queryable PrismQL column names for Cyberint IOC fields are
  `iocs_value` and `iocs_type` — the ENRICH-1 clean SQL identifiers declared as TOML
  `[[tables.columns]]` entries with `source_path = "$.iocs[*].value"` and
  `source_path = "$.iocs[*].type"` respectively (S-DEMO-ENRICH-1). The bracket-in-name
  forms `iocs[].value` / `iocs[].type` (PIVOT-003 convention) are NOT the TOML column names and
  are NOT queryable as PrismQL column names. The short forms `ioc_value` and
  `ioc_type` are serde aliases resolved at the Rust struct level (see Ioc struct
  `#[serde(rename = "type")]` annotation); they are NOT TOML column
  declarations. There is no `ioc_severity` column — the alert-level severity is `severity`.
  Any PrismQL query using `ioc_value`/`ioc_type`/`ioc_severity`/`iocs[].value`/`iocs[].type`
  as column names will fail at plan time with E-QUERY-038. The wildcard source_path
  `$.iocs[*].value` means `iocs_value` returns a JSON-list string (e.g., `["hash1","hash2"]`),
  not a single scalar. Callers should use `iocs_value IS NOT NULL` (or IS NULL) for filtering
  rather than equality comparisons against a scalar value.

---

## 7. Recording Sequence Recommendation (T14)

Suggested presentation order for T14 demo-recorder:

1. Open Claude Code with prism MCP server connected. Show the tool list.
2. Block 1: `list_capabilities(org-c)` → `prism_describe(org-c)` → `prism_describe(org-a)` [schema diff]
3. Block 2: CrowdStrike query org-c → Armis correlation → org-a isolation proof
4. Block 3: Cyberint IOC query at Stage 3 (`| where iocs_value IS NOT NULL`) → Step 3.1a IEQ/IIN filter beat (`severity IEQ 'critical'`, `severity IIN ('high', 'critical')`, `status IIN ('open', 'closed')`) → `| enrich threat_score(iocs_value_first)` → `| enrich cvss_base_score(device_cves_first)`
5. Block 4: E-QUERY-037 table-not-available for org-a → E-QUERY-038 column gate (001-B) → Step 4.3 SQL-mode IEQ rejection E-QUERY-001 (mode-boundary pedagogical beat) → Claude self-corrects across all three error types
6. Block 5 (PENDING S-5.04): `check_sensor_health(org-c)` → live probe confirmation
7. Block 6: Stage 4 full blast radius query → Claroty audit log

Total estimated recording time: 35-40 minutes of analyst interaction.

The recording should show Claude working with prism's teaching surfaces —
`prism_describe` output driving query authorship, pedagogical errors driving
self-correction, `| enrich` enrichment flowing inline. The human presenter adds
context between queries but does not hand-hold Claude on syntax.

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.12 | 2026-07-18 | **DRIFT-AUDIT-COVERAGE-001-RUNBOOK-ENV-BRIDGE-001 closure.** Added §1.6 Pre-Flight Audit (Go/No-Go Gate): operator-facing invocation procedure for `scripts/t13-preflight-audit.py`. Documents `PRISM_THREATINTEL_BASE_URL` / `PRISM_NVD_BASE_URL` copy-from-demo-run.sh workflow, env-var precedence (BASE_URL > PORT > built-in default), optional `PRISM_BIN` override, exit-code capture discipline (`PIPESTATUS[0]` vs `$?`), 106-check coverage matrix summary, and DEMO-READY verdict + exit code semantics. Inserted between §1.5 fleet-startup and §2 narrative arc as the go/no-go gate step. PR #226 develop@97d7335d 2026-07-18. |
| 1.11 | 2026-07-16 | **DRIFT-AUDIT-RUNBOOK-LITERALS-001 D-1609 closure.** Task 1: Full-document sweep for 0-row literals — WARN-1 (`severity IEQ 'high'` on CrowdStrike → fixed to `'critical'` in v1.7) and WARN-2 (`status IIN ('new','in progress')` → fixed to `('open','closed')` in v1.7) confirmed in place in v1.10; no additional 0-row literals found. Seed-data evidence: `crates/prism-dtu-crowdstrike/src/generator.rs` first-5 severity_id=4→"Critical", rest severity_id=2→"Medium" only; `crates/prism-dtu-cyberint/src/generator.rs` statuses=["open","acknowledged","closed"] (lowercase vendor-native). Task 2: D-1609 formal adjudication (A) CORRECT PASSTHROUGH — Cyberint `status` values have no OCSF caption match, pass through per RG-021; documented in BC-2.02.013 EC-02-029 (v1.9 → v1.10). Step 3.1a adjudication reference updated: "2026-07-08" → "D-1609 2026-07-16"; full OCSF caption set enumerated (generic+finding-class); EC-02-029 cross-reference added. |
| 1.10 | 2026-07-12 | **F-AUD-P21-HIGH-003 — §5.5 CVSS threshold tightened from range to exact value.** Adjudication: Option (a). The NVD DTU (`NvdClone::new_with_scenario`, BC-2.06.020 PC-4) hardcodes `base_score = 8.1 / base_severity = "HIGH"` for all scenario CVEs (`CVE-9999-NNNN` format); this is a deterministic contract, not an approximation. §4 Expected Outputs "CVSS 8.1 HIGH" talking point is authoritative. Changes: (1) §5.5 checklist item: `cvss_base_score >= 7.0` → `cvss_base_score == 8.1` with rationale citing `NvdClone::new_with_scenario` PC-4. (2) §3 Step 3.5 expected output: `(~8.1)` → `(8.1)` (tilde removed — value is deterministic). Implementer E3 assertion: `assert cvss_base_score == 8.1`. No other loci require change (lines 105-106, 212-213, 672, and §4 row 904 already cite 8.1 exactly). |
| 1.9 | 2026-07-11 | **F-AUD-P10-HIGH-001 — completed ADR-051 D4 scalar-input amendment — 6 prose loci `iocs_value` → `iocs_value_first` (query blocks were already amended in v1.8).** Locus 1: Act 4 narrative (line ~210). Locus 2: §4 Talking Points IOC enrichment row (line ~903). Locus 3: §5.4 Scenario Clock elapsed >= 360s item — LEFT unchanged (describes raw `iocs_value` JSON-list column being non-null, not an enrich call). Locus 4: §5.5 Enrichment Path checklist (line ~959) — enrich call updated; erroneous parenthetical "enrich operates on the list" replaced with accurate ADR-051 D4 scalar-companion note. Locus 5: §6 Capability Caveats "Enrichment at Stage < 3" bullet (line ~1041) — enrich call updated; null-state description updated to reference `iocs_value_first` scalar companion while retaining explanation of the underlying `iocs_value` JSON-list behavior. Locus 6: §7 Recording Sequence Block 3 (line ~1097). Residual grep: zero remaining `threat_score(iocs_value)` non-`_first` sites. |
| 1.8 | 2026-07-09 | **AUDIT-COVERAGE-001 — ADR-051 D4 scalar-input + parse-error literal corrections.** (1) Steps 3.2 and 6.2: `iocs_value` → `iocs_value_first` in query blocks and follow-on prose UDF examples (`threat_is_known_malicious`, `threat_sources`). Step 3.2 filter: `\| where iocs_value IS NOT NULL` → `\| where iocs_value_first IS NOT NULL`. Step 6.2 filter: `\| where iocs_type IS NOT NULL` → `\| where iocs_value_first IS NOT NULL`. (2) Step 3.4: `\| enrich threat_score(behaviors_ioc_value)` → `\| enrich threat_score(behaviors_ioc_value_first)`. All three changes enforce ADR-051 D4 scalar-input rule: enrichment UDFs require `*_first` scalar companions, not JSON-list columns. (3) §5.3 per-client data distinctness checklist: `FROM crowdstrike_detections LIMIT 5` → `FROM crowdstrike_detections \| limit 5` (two lines). (4) §5.4 scenario clock progression: `FROM armis_devices LIMIT 20` → `FROM armis_devices \| limit 20`. Bare SQL `LIMIT N` without `\|` prefix is a PrismQL parse error; valid pipe form is `\| limit N`. Source: t13-audit-coverage-gap-analysis-2026-07-10.md §5 findings a + b. Frontmatter version 1.7 → 1.8. |
| 1.7 | 2026-07-08 | **DRIFT-AUDIT-RUNBOOK-LITERALS-001 — runbook literal corrections from pre-flight audit 2026-07-08.** (1) Step 3.1a first IEQ beat: `severity IEQ 'high'` → `severity IEQ 'critical'` (CompromisedEndpoint scenario seeds Critical/Medium for CrowdStrike; 'High' absent from scenario data; audit WARN-1). Expected output updated: `'High'` → `'Critical'`; DataFusion lowering note updated accordingly. (2) Step 3.1a status IIN beat: `status IIN ('new', 'in progress')` → `status IIN ('open', 'closed')` (Cyberint status values are vendor-native `{'open', 'acknowledged', 'closed'}` — not OCSF captions — so adapter pass-through is correct per BC-2.02.013 RG-021; audit WARN-2 adjudicated PASS-THROUGH-CORRECT 2026-07-08; IIN is case-insensitive and works correctly against the lowercase stored values; robustness note added to teaching note). Teaching note expanded to distinguish OCSF-normalized fields (severity: vendor value matches OCSF caption → normalized to Title-case) from vendor pass-through fields (Cyberint status: no OCSF caption match → stored as-received; IIN still absorbs analyst casing). (3) §4 Expected Outputs: case-insensitive filtering row updated to cite `severity IEQ 'critical'` and `status IIN ('open', 'closed')` with updated talking point. (4) §5.9 Dry-Run Checklist item 1: `severity IEQ 'high'` → `severity IEQ 'critical'`; expected result updated. (5) §7 Recording Sequence Block 3: `severity IEQ 'high'` → `severity IEQ 'critical'`; `status IIN ('new', 'in progress')` → `status IIN ('open', 'closed')`. Frontmatter version 1.6 → 1.7. |
| 1.6 | 2026-07-08 | **IEQ/IIN/INE case-insensitive operator surface (S-PRISMQL-CASE-INSENSITIVE-001, PR #217, develop@f935edb6; ADR-047 ACCEPTED).** Step 3.1a added: three example queries demonstrating `severity IEQ 'high'` against `crowdstrike_detections`, `severity IIN ('high', 'critical')` against `cyberint_alerts`, and `status IIN ('new', 'in progress')` against `cyberint_alerts`; teaching note on OCSF Title-case normalization at the adapter boundary and E-QUERY-002 typed-column guidance; VERIFY note for column confirmation via `prism_describe`. Step 4.3 added: SQL-mode IEQ rejection (E-QUERY-001 mode-boundary) as a pedagogical error-UX beat in Query Block 4. §4 Expected Outputs: IEQ/IIN case-insensitive filtering row added. §5.9 Dry-Run Checklist section: three IEQ/IIN/SQL-mode checks. §7 Recording Sequence: Block 3 reference includes Step 3.1a IEQ/IIN beat; Block 4 reference includes Step 4.3 SQL-mode E-QUERY-001 beat. Frontmatter version 1.5→1.6. |
| 1.5 | 2026-06-26 | **Demo-fidelity remediation (S-DEMO-FIDELITY-REMEDIATION-001):** Three factual errors corrected against live audit evidence (demo-pre-flight-audit-2026-06-26.md). N3: §5.8 checklist corrected E-QUERY-037 → E-QUERY-032 for sensor-not-registered-for-org checks (`FROM cyberint_alerts client_id="org-a"`, `FROM armis_devices client_id="org-b"`). These orgs are missing the sensor registration entirely; the correct error is E-QUERY-032 (authorization gate "sensor X not registered for org Y"), NOT E-QUERY-037 (plan-time table-not-in-TableRegistry). The distinction matters: E-QUERY-037 fires when the org has the sensor registered but the queried table is wrong; E-QUERY-032 fires when the org has NO such sensor at all. N4: Step 3.5 corrected `client_id="org-b"` → `client_id="org-c"` for the Armis CVE enrichment query. Org-b does NOT have Armis (org-b = Claroty + Cyberint only, per §1.2 org topology); the query would return E-QUERY-032 for org-b. Org-c (all four sensors) is the correct target. Updated talking point accordingly. N5: §6.3 corrected `FROM claroty_audit_log` → `FROM claroty_audit_logs` (plural, per Claroty sensor TOML `table_name = "audit_logs"` → FROM-ready name `claroty_audit_logs`); corrected example column comment `log_id` → `id` (real column per Claroty DTU `AuditLogEntry.id: String`). |
| 1.4 | 2026-06-24 | AC-020/BLOCKER-002 (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001): corrected pipe-mode query syntax in Steps 3.2, 3.4, 3.5, and 6.2. These queries mixed SQL-style `WHERE`/`LIMIT` (without `\|` prefix) with `\| enrich` pipe stages — invalid PrismQL. Corrected to valid pipe mode: `WHERE` → `\| where`, `LIMIT` → `\| limit` throughout all four steps. Updated §7 Block 3 reference (`WHERE` → `\| where`). Concurrent-execution clarification (Task 3): §1.1 framing updated to make explicit that prism is a SINGLE process with a multi-threaded tokio runtime; a query fans out to multiple sensors IN PARALLEL (bounded by MAX_FANOUT_CONCURRENCY=10, nested under HTTP_SEMAPHORE_PERMITS=200); concurrent queries are NOT serialized by any lock (`PrismServer::query` takes `&self`; ArcSwap lock-free config reads). The only sequential aspect is stdin message framing in the stdio transport — a transport/client characteristic, not engine serialization. Talking points updated to reflect "async fan-out" framing. |
| 1.3 | 2026-06-23 | GAP-1: corrected enrichment UDF function names throughout. Actual registered UDF names are per-field from `[[infusion.fields]]`: ThreatIntel → `threat_score`, `threat_is_known_malicious`, `threat_sources`; NVD → `cvss_base_score`, `cvss_severity`, `cvss_vector`. Replaced all `\| enrich threat_intel(...)` with `\| enrich threat_score(...)` and all `\| enrich nvd(...)` with `\| enrich cvss_base_score(device_cves_first)`. Updated Step 3.2 expected output (no `threat_intel.threat_score` namespace prefix; column is `threat_score`), Step 3.5 to use Armis devices with `device_cves_first` (Ruling 1b) and `cvss_base_score`/`cvss_severity` column names, §4 Expected Outputs table, §5.5 dry-run checklist, §6 enrichment caveat, §7 recording sequence. GAP-3: corrected `prism_describe` JSON example key `"table_name"` → `"name"` in all four table entries (matches `TableDescriptor.name` field in `prism-mcp/src/tools/prism_describe.rs`). |
| 1.2 | 2026-06-23 | ENRICH-1 clean-column-name amendment (S-DEMO-ENRICH-1). PIVOT-003 bracket-in-name column references (`iocs[].value`, `iocs[].type`) superseded by ENRICH-1 clean SQL identifiers: `iocs_value` (source_path `$.iocs[*].value`), `iocs_type` (source_path `$.iocs[*].type`). All queries, expected outputs, VERIFY IN DRY-RUN notes, checklist items, Expected Outputs table, and §6 caveat updated. prism_describe JSON example updated (`iocs[].value` → `iocs_value`, `iocs[].type` → `iocs_type`). §6 caveat rewritten: bracket-in-name forms NOT queryable as PrismQL column names; wildcard source_path means `iocs_value` returns JSON-list string (e.g., `["hash1","hash2"]`). Added diagnostic note: check `column_source_path_extraction_failed` warn events if `iocs_value` is unexpectedly null. `behaviors_ioc_type`/`behaviors_ioc_value` (CrowdStrike) were already correct clean names (set in PIVOT-003). |
| 1.1 | 2026-06-23 | Corrected Cyberint IOC column names throughout: `ioc_value` → `iocs_value`, `ioc_type` → `iocs_type`, `ioc_severity` → `severity` (alert-level field; no separate IOC severity column exists). These short forms are serde aliases, not queryable PrismQL column names. Added §6 capability caveat documenting the `iocs[].` nested path requirement. BLOCKER 4 fix from dry-run. |
| 1.0 | 2026-06-22 | Initial draft. |
