---
document_type: research
producer: smoke-test + discoverability-audit (claude, MCP-only information asymmetry)
timestamp: 2026-06-26
topic: T13 Capstone Live Demo — Pre-Flight Re-Audit (post PR #203 + #204)
status: complete
feeds: T13 capstone demo preparation; runbook drift remediation
branch: develop
supersedes_audit: .factory/research/demo-pre-flight-audit-2026-06-24.md
develop_head: f05a9f0e
---

# T13 Capstone Live Demo — Pre-Flight Re-Audit (2026-06-26)

Role: competent security analyst (LLM agent) connecting to a freshly-provisioned Prism
MCP server. Strict information asymmetry on the discovery layer: enrichment syntax,
table names, and query patterns are inferred only from the live MCP surface
(`tools/list` schemas, prompts, `prismql://reference`). Operational tooling
(`demo-setup.sh`, `demo-run.sh`) and the T13 runbook are treated as black-box
operational references — their content is not used as a discovery hint.

Target: main worktree `/Users/jmagady/Dev/prism`, develop HEAD `f05a9f0e`, DTU mode.

**Why this re-audit:** The prior audit (2026-06-24) predates the merge of PR #203
(`7e60df03` — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001, grammar remediation + MCP fixes)
and PR #204 (`f05a9f0e` — test-gate hardening + `build_http_client` timeout fix), and
PR #202 (`903c8fc` — S-5.04 sensor health subsystem live probes). This re-audit
re-verifies every prior finding against the current binaries built from `f05a9f0e`.

---

## 1. Methodology and Repeatability Guide

### 1.1 Bring-up procedure

```bash
# Rebuild binaries at current HEAD (prior binaries predate PR #203/#204)
cargo build --release -p prism-bin
cargo build --release -p prism-dtu-demo-server --features dtu,fixture-gen

bash scripts/demo-setup.sh          # provision config + 10 keyring creds (idempotent)
bash scripts/demo-run.sh            # start DTU start-multi; write overlays; print env block
```

Env block recorded this session (ephemeral ports vary per run):

```
CROWDSTRIKE_BASE_URL=http://127.0.0.1
ARMIS_INSTANCE_URL=http://127.0.0.1
CLAROTY_INSTANCE_URL=http://127.0.0.1
CYBERINT_ENVIRONMENT=demo
PRISM_DTU_MODE=true
PRISM_THREATINTEL_BASE_URL=http://127.0.0.1:60245
PRISM_THREATINTEL_API_KEY=demo-threatintel-api-key
PRISM_NVD_BASE_URL=http://127.0.0.1:60246
PRISM_NVD_API_KEY=demo-nvd-api-key
```

DTU PID this session: 2323. Started fresh against an empty `~/.config/prism-demo/state/`
(clean RocksDB) so the BLOCKER-001 cross-session test is valid.

**Critical DTU stability rule (unchanged):** Never `pkill -f "prism.*start"` — that
matches `prism-dtu-demo-server start-multi`. Use `kill <specific_PID>` for the prism
MCP server only; restart DTU via `bash scripts/demo-run.sh`.

### 1.2 Transport and test harness

Prism is a stdio MCP server (JSON-RPC 2.0, protocol `2024-11-05`). A small Python
driver (`/tmp/mcp_driver.py`) spawns `prism --config-dir ~/.config/prism-demo start`,
sends `initialize` immediately (no sleep), then `notifications/initialized`, then issues
tool/prompt/resource calls reading line-delimited JSON with a `select()` timeout. The
`query` tool parameter is `clients` (array), e.g. `{"query":"...","clients":["org-c"]}`.

Response shapes observed this session:
- `query` results: `result.structuredContent.results.rows` (+ `returned_results`,
  `normalized_pql`, `sensor_errors`, `total_available`, `is_truncated`).
- `check_sensor_health`: data is directly in `result.structuredContent` (NOT under
  `.results`).
- Error envelopes: `result.isError == true` with the message in `result.content[0].text`.

### 1.3 MCP response ordering constraint

Response serialization by request ID still holds in principle, but it is **no longer an
operational hazard**: every tool and prompt that previously hung (BLOCKER-003/004) now
returns in < 0.2s, so there is no longer a cascade-timeout risk from a stuck request.

### 1.4 Scenario clock

`scenario_start_secs = 1782214754` for org-c (set in `scripts/demo.toml`). At audit
wall-clock (`~1782526472`), elapsed ≈ 311,718s ≫ 600s, so org-c is permanently at
Stage 4 (Containment): all IOC/CVE fields populated. Confirmed live (`iocs_value`,
`device_cves_first`, `behaviors_ioc_*` all non-null).

### 1.5 Org and sensor topology (3 orgs, 8 sensor instances)

From `prism://config/clients` (live):

| Org   | Sensors                                     | Seed |
|-------|---------------------------------------------|------|
| org-a | crowdstrike, armis                          | 100  |
| org-b | cyberint, claroty                           | 150  |
| org-c | crowdstrike, cyberint, claroty, armis       | 200  |

Device IDs `dev-0196f4b2-{seed}-{N}`; org-a/org-c disjoint (100 vs 200).

---

## 2. §5 Checklist Walkthrough — Actual Query Results

All results from live runs against develop HEAD `f05a9f0e`, DTU running.

### 2.1 §5.1 — Server starts, 8 sensor adapters across 3 orgs

`prism://config/clients` → 3 orgs, sensor_count 2/2/4 = 8 instances. **PASS.** ~0.0s.

### 2.2 §5.2 — prism_describe per org

`prism_describe(org-c)` returns 10 tables; `prism_describe(org-a)` returns 5 tables
(no claroty/cyberint) — per-client isolation holds. E-QUERY-038 fires correctly with a
machine-readable `available` column list. **PASS** for isolation + column gate.
**Caveat: AUDIT-001 STILL-OPEN** — see §3.

### 2.3 §5.3 — CrowdStrike detections disjoint across orgs

| Query | Org | Rows | Sample device_id |
|-------|-----|------|------------------|
| `SELECT device_id FROM crowdstrike_detections LIMIT 3` | org-a | 3 | `dev-0196f4b2-100-3` |
| `SELECT device_id FROM crowdstrike_detections LIMIT 3` | org-c | 3 | `dev-0196f4b2-200-11` |

Seeds 100 vs 200, disjoint. **PASS — and now UNCONDITIONAL** (no first-session timing
constraint; see BLOCKER-001 RESOLVED in §3).

### 2.4 §5.4 — Non-CrowdStrike sensors return data

Armis (org-c, org-a), Cyberint (org-c), Claroty devices (org-b) all return rows in
0.00–0.01s, in any call order, including after a CrowdStrike call. **PASS.**

### 2.5 §5.5 — Enrichment (ThreatIntel + NVD CVSS)

Correct pipe syntax (runbook v1.4) confirmed working:
```
FROM cyberint_alerts | where iocs_value IS NOT NULL | enrich threat_score(iocs_value) | limit 10
→ 10 rows; threat_score field carries full ThreatIntel record (threat_score:95, known_malicious:true)

FROM armis_devices | where device_cves_first IS NOT NULL | enrich cvss_base_score(device_cves_first) | enrich cvss_severity(device_cves_first) | limit 3  [org-c]
→ 3 rows; cvss_base_score:"8.1", cvss_severity:"HIGH", device_cves_first:"CVE-9999-72859"

SELECT * FROM cyberint_alerts | enrich threat_score(iocs_value) | limit 5  [SqlPipe mode]
→ 5 rows enriched (new SqlPipe composition from PR #203 works)

FROM crowdstrike_detections | where behaviors_ioc_type IS NOT NULL | enrich threat_score(behaviors_ioc_value) | limit 5
→ enriched (sensor-agnostic enrich on CrowdStrike behaviors)
```

Old broken runbook form (pre-v1.4) still correctly rejected:
```
FROM cyberint_alerts WHERE iocs_value IS NOT NULL | enrich threat_score(iocs_value) LIMIT 3
→ E-QUERY-001 parse error at offset 21 (found 'W')
```

E-QUERY-040 FORBID-BOTH (new in PR #203) fires correctly:
```
SELECT * FROM cyberint_alerts LIMIT 10 | limit 5
→ E-QUERY-040 redundant row limit, with remediation guidance
```

**Verdict: PASS for enrichment engine; runbook §5.5 syntax now correct (v1.4).** New
finding N1 (reference documents WRONG function names) — see §3.

### 2.6 §5.6 — Multi-tenant isolation and pedagogical errors

| Test | Result | Code |
|------|--------|------|
| crowdstrike for org-b (not registered) | error | **E-QUERY-032** |
| claroty for org-a (not registered) | error | **E-QUERY-032** |
| cyberint for org-a (not registered) | error | **E-QUERY-032** |
| `FROM totally_unknown_table` | error | **E-QUERY-037** (lists available sensors + tables) |
| `SELECT device_id, nonexistent_column ...` | error | **E-QUERY-038** (available column list) |
| `FROM cyberint.alerts` (dot syntax) | **0 rows, isError=false** | sensor_errors: E-SENSOR-030 — see N2 |

Isolation IS enforced. E-QUERY-038 and E-QUERY-037 (unknown table) are pedagogically
strong. **CHANGED:** sensor-not-registered now returns **E-QUERY-032**, not E-QUERY-037
as the runbook predicts (§3 N3); dot-syntax now produces a silent-empty partial failure
rather than the prior E-QUERY-036 did-you-mean (§3 N2).

### 2.7 §5.7 — check_sensor_health (S-5.04 merged, PR #202)

`check_sensor_health(org-c)` → all 4 sensors `probe_level: "live"`, `reachable: true`,
`auth_valid: true`, `overall_status: "healthy"`, with real `latency_ms` (armis 5,
claroty 5, crowdstrike 107, cyberint 5) and `last_successful_query_at` timestamps.
Summary: "4 of 4 sensor(s) healthy for client 'org-c' (live probe)".

**Verdict: PASS — was DEFERRED (spec-only) in prior audit; S-5.04 now delivers live
probing.** (Lazy resource `prism://sensors/health` returns a placeholder until
`check_sensor_health` is called once — minor, non-blocking.)

### 2.8 §5.8 — Prompts and resources

| Prompt | Prior | Now |
|--------|-------|-----|
| `client_overview(org-c)` | returns | returns 0.0s; embeds dot-syntax (AUDIT-004) |
| `triage_alerts(org-c)` | returns (dot syntax) | returns 0.0s; dot-syntax (AUDIT-004) |
| `cross_client_status` | partial | returns 0.0s; dot-syntax (AUDIT-004) |
| `query_tutorial(org-c)` | **HANGS** | **returns 0.0s — RESOLVED**; well-formed (points to prism_describe + reference, uses `<sensor_table>` placeholder, no dot syntax) |
| `investigate_host(org-c)` | **HANGS** | **returns 0.0s — RESOLVED**; embeds dot-syntax (AUDIT-004) |

`prismql://reference`: now **7,085 chars, runtime-assembled** (PR #203 replaced the
deleted static `pql_reference.md`). Contains the four-mode grammar (Filter/SQL/Pipe/
SqlPipe), `| where`/`| limit`/`| enrich` pipe stages, datetime arithmetic, the full
E-QUERY error quick-reference, and an enrichment section. **DISCOVERABILITY-GAP-001 is
substantially RESOLVED** — enrichment grammar is now discoverable. Two residual defects
in the reference content (N1 wrong function names, N4 dot-syntax examples) — see §3.

---

## 3. FINDINGS

Findings tagged `[RESOLVED]`, `[STILL-OPEN]`, `[CHANGED]`, or `[NEW]` relative to the
2026-06-24 audit.

### BLOCKER-001 — CrowdStrike OAuth cross-session state corruption [RESOLVED]

**Prior:** CrowdStrike worked only in the FIRST prism session after a DTU restart;
session 2+ hung 30s then 0 rows, cascading to all subsequent queries.

**Now: RESOLVED.** Verified across 3 separate prism sessions against the SAME unchanged
DTU (persisted RocksDB from session 1's CrowdStrike token cache):
- Session 1: CS org-a 0.20s (3 rows), CS org-c 0.11s (3 rows).
- Session 2 (CS first, persisted state): CS org-c **0.11s, 3 rows, no hang**; armis
  after CS 0.01s.
- Session 3 (armis first, then CS): armis 0.01s, CS 0.11s (2 rows), cyberint 0.01s.

The 30s OAuth hang and 0-row silent failure are gone. CrowdStrike is stable in any
session, any call order, with no state-clearing workaround required. PR #204's
`build_http_client` timeout fix and/or PR #203 plausibly remediated the underlying
plugin HTTP path. The §5.3 demo step no longer needs the "CrowdStrike first / clear
RocksDB" workaround.

### BLOCKER-002 — Runbook §5.5 pipe syntax invalid [RESOLVED]

**Prior:** Runbook enrichment examples used `FROM table WHERE pred | enrich fn LIMIT N`
(invalid — parse error at offset 21).

**Now: RESOLVED (runbook fixed).** The T13 runbook is at **v1.4** with changelog entry
"AC-020/BLOCKER-002 (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001): corrected pipe-mode query
syntax in Steps 3.2, 3.4, 3.5, 6.2." Verified live: the corrected `| where ... | enrich
... | limit` form works (§2.5), and the old broken form still produces the offset-21
parse error (so the documented fix is the real fix). New SqlPipe mode `SELECT ... FROM t
| enrich ... | limit N` also works. **Residual runbook drift unrelated to syntax: N4,
N5 below.**

### BLOCKER-003 — query_tutorial + investigate_host prompts hang [RESOLVED]

**Prior:** Both prompts hung indefinitely (30s+ timeout), cascading to all subsequent
calls.

**Now: RESOLVED.** Both return in 0.0s. `query_tutorial` is now well-designed (directs
the analyst to `prism_describe` and `prismql://reference`, uses `<sensor_table>`
placeholders, documents the E-QUERY self-correction fields). `investigate_host` returns
its step-by-step body (but embeds dot-syntax table names — see AUDIT-004). PR #203's
`server.rs` rewrite (831 lines) and `error_recovery.rs` mode-bridge eliminated the hang.

### BLOCKER-004 — list_infusions / plugin_status / infusion_status hang [RESOLVED]

**Prior:** All three hung indefinitely (regression from the 2026-06-23 fast-fail).

**Now: RESOLVED.** All fast-fail at 0.00s:
- `list_infusions` → `-32003` "Feature not yet available: infusion management
  (prism-operations not merged)" (returns to the clean -32003 fast-fail, no hang).
- `list_plugins` → `-32003` "plugin management (prism-operations not merged)" (NOTE:
  prior audit said list_plugins worked; it now also fast-fails -32003 — see N6, not
  blocking).
- `infusion_status` → `-32602` "missing field `infusion_id`" (correct: the tool
  requires an `infusion_id` param; calling with none is a schema error, not a hang).
- `plugin_status` → `-32602` "missing field `plugin_id`" (same — correct schema gate).

No hangs remain. The §6 demo-execution constraint to "skip list_infusions/plugin_status"
is obsolete.

### MAJOR-001 — list_capabilities client_registered:false [CHANGED → mostly RESOLVED]

**Prior:** `list_capabilities(org-c)` returned `client_registered: False` for all orgs.

**Now: CHANGED.** `client_registered: true` (the core false-negative is fixed).
However the `capabilities` map is **empty (`{}`)** and a large `not_registered_tools`
array is returned (40 tool names, mostly prism-operations/TDE tools that are legitimately
not wired). The misleading "this client has nothing" signal is gone, but the tri-state
capability map the runbook §1.1 Block 1 expects (`enabled_count` /
`runtime_disabled_count` / `compile_time_disabled_count` per capability) is not
populated for org-c. **Downgraded from MAJOR to LOW/CHANGED:** discoverability is no
longer actively misleading, but `list_capabilities` is not yet a rich capability surface.
Not demo-blocking (the demo can rely on `prism_describe` + `prismql://reference`).

### AUDIT-001 — prism_describe table name vs FROM-ready name mismatch [STILL-OPEN]

**Unchanged.** `prism_describe(org-c)` returns table `name` values as SHORT unqualified
strings: `alerts` (×3 — one each for crowdstrike/cyberint/claroty/armis with collisions),
`devices` (×3), `detections`, `audit_logs`, `incidents` (×2). `FROM` requires the
sensor-prefixed names (`cyberint_alerts`, `armis_devices`, `claroty_audit_logs`, etc.).
The `pql_hints` even instruct `SELECT * FROM <table>` which would fail with these short
names. For org-c this is now arguably WORSE than the prior audit noted: three tables all
named `alerts` and three named `devices` with no disambiguator. A naive analyst copying a
describe `name` into a FROM clause gets E-QUERY-037 (unknown table) or E-SENSOR-030. The
recovery path (E-QUERY-037 lists the real FROM-ready names) still works, so it is
self-correctable, but the primary describe output remains misleading.

### AUDIT-004 — prompts embed dot-syntax table names [STILL-OPEN]

**Unchanged.** PR #203 did NOT regenerate prompt bodies. Four of five prompts still
embed `sensor.table` dot-syntax that the query engine does not accept as a FROM target:
- `triage_alerts`: `FROM crowdstrike.alerts`, `FROM claroty.alerts`, `FROM armis.alerts`
- `client_overview`: `FROM crowdstrike.alerts`, `FROM claroty.alerts`
- `cross_client_status`: `FROM crowdstrike.alerts`
- `investigate_host`: `FROM crowdstrike.devices`, `FROM claroty.assets`, `FROM armis.devices`

A Claude agent literally following these prompts produces 0-row silent failures
(dot-syntax → E-SENSOR-030, see N2) rather than usable data. `query_tutorial` is the one
prompt that is clean (placeholder-based, points at prism_describe). The server's own
guidance still contradicts the query engine. **Demo impact:** any block that invokes
these four prompts and follows the embedded queries verbatim will return no data.

### DISCOVERABILITY-GAP-001 — enrichment absent from prismql://reference [RESOLVED (with residual N1)]

**Prior:** `prismql://reference` (6,471 chars) had zero `enrich` content.

**Now: RESOLVED.** The runtime-assembled reference (7,085 chars) documents Pipe and
SqlPipe modes, the `| enrich <fn>(<col>)` stage, an "Available enrichment functions"
section, and an `enrich threat_score(src_ip)` example. Enrichment grammar IS now
discoverable from the canonical reference. **Residual defect N1** (the reference lists
the WRONG function names) is filed separately below.

### AUDIT-005-DEFERRED — check_sensor_health spec-only [RESOLVED]

**Prior:** probe_level: spec-only; S-5.04 not merged. **Now: RESOLVED** — S-5.04 merged
(PR #202). `probe_level: "live"` with real latency and reachability per sensor (§2.7).

---

### N1 — prismql://reference documents WRONG enrichment function names [NEW]

**Severity: MAJOR (discoverability trap).**

The reference's "Available enrichment functions" section lists:
```
- enrich nvd(col)
- enrich threat_intel(col)
```
But the actual registered UDF names are the per-`[[infusion.fields]]` names
(`threat_score`, `threat_is_known_malicious`, `threat_sources`, `cvss_base_score`,
`cvss_severity`, `cvss_vector`). The single-function forms `threat_intel(...)` / `nvd(...)`
are explicitly NOT registered (runbook §6 caveat). Worse, the reference is internally
contradictory: its own example two sections later uses the CORRECT name
(`| enrich threat_score(src_ip)`).

Calling the documented `threat_intel(iocs_value)` live returns an opaque
`[upstream_error] - Internal error; see audit log` — NOT a clean E-QUERY-039
"enrichment infusion not registered". So an analyst who follows the reference's function
list verbatim gets an internal error with no self-correction hint. **Fix:** correct the
reference's enrichment-functions list to the registered per-field UDF names, and ensure
an unregistered enrichment function returns E-QUERY-039 (not an internal error).

### N2 — Dot-syntax table names now silent-empty instead of E-QUERY-036 [NEW / CHANGED]

**Severity: MEDIUM (pedagogical regression).**

`FROM cyberint.alerts` (dot syntax) now parses `cyberint.alerts` as a bare table name,
routes to the cyberint sensor, and returns `isError:false, returned=0` with
`sensor_errors: ["cyberint.alerts: all targets failed (E-SENSOR-030)"]`. The prior audit
saw a clean plan-time **E-QUERY-036** with a "Did you mean 'cyberint_alerts'?"
suggestion. The pedagogical did-you-mean path for dot-syntax has regressed to a
silent-empty partial failure. This compounds AUDIT-004: the four dot-syntax prompts now
fail silently rather than emitting a self-correcting error. (Genuinely unknown tables
like `totally_unknown_table` DO still get a clean E-QUERY-037.) **Fix:** restore the
E-QUERY-036 dot-syntax detection at plan time, OR ensure E-SENSOR-030 carries the
did-you-mean hint.

### N3 — Sensor-not-registered returns E-QUERY-032, runbook expects E-QUERY-037 [NEW / runbook drift]

**Severity: LOW (runbook expectation mismatch; behavior is correct).**

Querying a sensor that exists at TYPE level but is not registered for the org
(`claroty`/`cyberint` for org-a; `crowdstrike` for org-b) returns **E-QUERY-032**
("Sensor X is not registered for org Y"). The runbook Query Block 4 / §5.8 expects
**E-QUERY-037** ("Table not available") for `FROM claroty_devices ... org-a` and
`FROM cyberint_alerts ... org-a`. The isolation is correctly enforced; only the error
CODE differs from the runbook's prediction. **Fix:** update runbook Query Block 4 / §5.8
to expect E-QUERY-032 for not-registered-sensor cases (reserve E-QUERY-037 for genuinely
unknown tables).

### N4 — Runbook Step 3.5 queries armis for org-b (org-b has no armis) [NEW / runbook drift]

**Severity: MEDIUM (a demo step as written returns an error).**

Runbook Step 3.5 (and §5.5 dry-run) uses
`FROM armis_devices | enrich cvss_base_score(device_cves_first) ... client_id="org-b"`.
org-b has only claroty + cyberint — no armis. Run verbatim it returns
**E-QUERY-032** "Sensor 'armis' is not registered for org 'org-b'". The same query
against **org-c** (which has armis) works perfectly (cvss_base_score 8.1, HIGH). **Fix:**
change Step 3.5 to `client_id="org-c"`, or pick an org-b table that exists.

### N5 — Runbook §6.3 uses claroty_audit_log (singular); real table is claroty_audit_logs [NEW / runbook drift]

**Severity: MEDIUM (a demo step as written returns E-QUERY-037).**

Runbook Step 6.3 uses `FROM claroty_audit_log LIMIT 20` (singular). The real registered
table is **`claroty_audit_logs`** (plural), columns `[action, actor, id, resource,
timestamp]` (NOT `log_id`). `claroty_audit_log` (singular) → E-QUERY-037. NOTE: this
CONTRADICTS the prior 2026-06-24 audit §2.4, which asserted singular was correct — the
prior audit was wrong on this point; plural is correct against current HEAD. **Fix:**
update runbook §6.3 to `claroty_audit_logs` and use a real column (e.g. `id`), not
`log_id`.

### N6 — list_plugins now fast-fails -32003 (prior audit said it worked) [NEW / OBS]

**Severity: OBSERVATION (non-blocking).**

The prior audit listed `list_plugins` as a working alternative to `plugin_status`. On
current HEAD `list_plugins` returns `-32003` "Feature not yet available: plugin
management (prism-operations not merged)" — same fast-fail as `list_infusions`. No demo
flow depends on it; recorded for accuracy only.

---

## 4. §5 Checklist Summary Table

| Item | Query / Tool | Result | Status |
|------|-------------|--------|--------|
| §5.1 Server starts, 8 adapters, 3 orgs | `prism://config/clients` | 3 orgs, 2/2/4 = 8 | PASS |
| §5.2 prism_describe per org | `prism_describe` | tables+columns; isolation holds; E-QUERY-038 fires | PASS (AUDIT-001 caveat) |
| §5.3 CrowdStrike disjoint | `SELECT device_id FROM crowdstrike_detections` | seeds 100/200 disjoint, unconditional | PASS |
| §5.4 Non-CS sensors data | armis/cyberint/claroty | rows in any order | PASS |
| §5.5 Enrichment ThreatIntel | `| enrich threat_score(iocs_value)` | threat_score 95, malicious | PASS |
| §5.5 Enrichment NVD CVSS | `| enrich cvss_base_score(device_cves_first)` | 8.1 HIGH | PASS |
| §5.5 SqlPipe enrich | `SELECT * FROM t | enrich ... | limit N` | enriched rows | PASS (new PR #203) |
| §5.5 Old broken pipe form | `FROM t WHERE p | enrich fn LIMIT N` | parse error offset 21 (correctly rejected) | PASS |
| §5.5 FORBID-BOTH | `... LIMIT 10 | limit 5` | E-QUERY-040 with remediation | PASS (new PR #203) |
| §5.6 Cross-org isolation | crowdstrike for org-b | E-QUERY-032 | PASS (N3 code note) |
| §5.6 Unknown table | `FROM totally_unknown_table` | E-QUERY-037 + available list | PASS |
| §5.6 Column not found | `SELECT nonexistent_column` | E-QUERY-038 + available cols | PASS |
| §5.6 Dot syntax | `FROM cyberint.alerts` | 0 rows, E-SENSOR-030 (was E-QUERY-036) | CHANGED (N2) |
| §5.7 check_sensor_health | `check_sensor_health(org-c)` | probe_level: live, 4/4 healthy | PASS (was DEFERRED) |
| §5.8 triage_alerts / client_overview / cross_client_status / investigate_host | `prompts/get` | return 0.0s; embed dot-syntax | PASS/DRIFT (AUDIT-004) |
| §5.8 query_tutorial | `prompts/get` | returns 0.0s; clean | PASS (was BLOCKER) |
| §5.8 prismql://reference | resource read | 7085 chars, enrichment documented | PASS (N1 caveat) |
| CrowdStrike session 2+ | CS query in session 2/3 | 0.11s, 3 rows, no hang | PASS (was BLOCKER) |
| list_infusions/plugin_status/infusion_status | tool calls | fast-fail -32003/-32602, no hang | PASS (was BLOCKER) |
| list_capabilities | `list_capabilities(org-c)` | client_registered:true; capabilities {} | CHANGED (was MAJOR) |

---

## 5. Runbook ↔ Code Drift Summary (current)

| Location | Runbook Text | Actual Behavior | Severity |
|----------|--------------|-----------------|----------|
| §5.5 enrichment syntax | `\| where ... \| enrich ... \| limit` (v1.4) | works | RESOLVED |
| `prismql://reference` enrichment fn list | (server-owned) lists `nvd(col)`/`threat_intel(col)` | wrong names → internal error | MAJOR (N1) |
| `triage_alerts`/`client_overview`/`cross_client_status`/`investigate_host` prompts | `FROM crowdstrike.alerts` etc. | dot-syntax → 0 rows E-SENSOR-030 | MEDIUM (AUDIT-004 + N2) |
| Query Block 4 / §5.8 | expects E-QUERY-037 for not-registered sensor | returns E-QUERY-032 | LOW (N3) |
| Step 3.5 | `armis_devices ... client_id="org-b"` | org-b has no armis → E-QUERY-032 | MEDIUM (N4) |
| Step 6.3 | `FROM claroty_audit_log` (singular), `log_id` | real table `claroty_audit_logs`, col `id` | MEDIUM (N5) |
| prism_describe table `name` | (server-owned) short names `alerts`/`devices` | not FROM-ready | MEDIUM (AUDIT-001) |

---

## 6. Demo Execution Order Recommendation (refreshed)

The fragile constraints from the prior audit are GONE: CrowdStrike is stable across
sessions, no tool/prompt hangs, no cascade-timeout risk. The execution order is now
driven only by the residual content-drift items:

1. Start DTU (`bash scripts/demo-run.sh`) and prism with the env block. No RocksDB
   state-clearing required; CrowdStrike works in any session.
2. `prism://config/clients` → `prism_describe(org-c)` → `prism_describe(org-a)`. When
   writing FROM clauses, use the sensor-prefixed names (`crowdstrike_detections`,
   `cyberint_alerts`, `armis_devices`, `claroty_devices`, `claroty_audit_logs`) — NOT the
   short `name` strings from describe output (AUDIT-001).
3. Sensor queries in any order (CrowdStrike no longer needs to go first/last).
4. Enrichment: use the REGISTERED UDF names — `threat_score(...)`, `cvss_base_score(...)`,
   `cvss_severity(...)` — NOT `threat_intel(...)`/`nvd(...)` (N1). Pipe form
   `FROM t | where p | enrich fn(col) | limit N`, or SqlPipe `SELECT * FROM t | enrich
   fn(col) | limit N`.
5. Pedagogical errors: E-QUERY-038 (bad column), E-QUERY-037 (unknown table),
   E-QUERY-032 (cross-org sensor), E-QUERY-040 (FORBID-BOTH) all fire cleanly. Avoid
   demonstrating dot-syntax as the "wrong table" example — it now silent-empties (N2);
   use a genuinely unknown table for the E-QUERY-037 teaching moment.
6. `check_sensor_health(org-c)` → live probe, 4/4 healthy.
7. If demoing prompts, prefer `query_tutorial` (clean). Do NOT have Claude execute the
   verbatim queries inside `triage_alerts`/`investigate_host`/`client_overview`/
   `cross_client_status` — they contain dot-syntax that returns no data (AUDIT-004).

---

## 7. Delta vs 2026-06-24 Audit

| Finding | 2026-06-24 | 2026-06-26 | Change |
|---------|-----------|-----------|--------|
| BLOCKER-001 CrowdStrike OAuth cross-session | DEMO-BLOCKER | **RESOLVED** | ✅ fixed |
| BLOCKER-002 Runbook §5.5 pipe syntax | DEMO-BLOCKER | **RESOLVED** | ✅ runbook v1.4 |
| BLOCKER-003 query_tutorial/investigate_host hang | DEMO-BLOCKER | **RESOLVED** | ✅ fixed (PR #203) |
| BLOCKER-004 list_infusions/plugin_status hang | BLOCKER | **RESOLVED** | ✅ fast-fail, no hang |
| MAJOR-001 list_capabilities client_registered:false | MAJOR | **CHANGED** (now true; capabilities map empty) | ⬇ downgraded to LOW |
| DISCOVERABILITY-GAP-001 enrichment not in reference | MAJOR | **RESOLVED** (residual N1) | ✅ enrichment documented |
| AUDIT-005-DEFERRED health spec-only | DEFERRED | **RESOLVED** (S-5.04 live) | ✅ fixed (PR #202) |
| AUDIT-001 describe name vs FROM-name | STILL-OPEN | **STILL-OPEN** | ↔ unchanged |
| AUDIT-004 prompts dot-syntax | STILL-OPEN | **STILL-OPEN** | ↔ unchanged |
| N1 reference wrong enrich fn names | — | **NEW (MAJOR)** | 🆕 |
| N2 dot-syntax silent-empty (was E-QUERY-036) | — | **NEW/CHANGED (MEDIUM)** | 🆕 regression |
| N3 E-QUERY-032 vs runbook's E-QUERY-037 | — | **NEW (LOW)** | 🆕 runbook drift |
| N4 runbook §3.5 wrong org (armis for org-b) | — | **NEW (MEDIUM)** | 🆕 runbook drift |
| N5 runbook §6.3 claroty_audit_log singular | — | **NEW (MEDIUM)** | 🆕 runbook drift |
| N6 list_plugins now fast-fails | — | **NEW (OBS)** | 🆕 non-blocking |

**Net: all 4 prior DEMO-BLOCKERS resolved; both prior MAJOR discoverability gaps
resolved or downgraded; the prior `check_sensor_health` deferral resolved.** The
remaining issues are 2 carry-over discoverability gaps (AUDIT-001, AUDIT-004) and 6 newly
surfaced items, none of which is a hard DEMO-BLOCKER (no hangs, no data loss, all
core query/enrichment/health/isolation paths work). The new items are content/drift
defects that should be fixed for a clean recording but do not prevent the demo from
running.

---

## 8. Bottom Line — Pre-Flight Verdict

**DEMO-READY: YES (conditional) — 0 DEMO-BLOCKERS remain.**

All four prior DEMO-BLOCKERS are resolved against develop `f05a9f0e`. The core demo path
is fully functional: 8 sensors / 3 orgs, per-client distinct + disjoint data,
cross-sensor correlation, ThreatIntel + NVD enrichment (pipe AND SqlPipe modes),
multi-tenant isolation, live sensor-health probing, the new FORBID-BOTH/temporal grammar,
and an enrichment-documenting runtime reference. CrowdStrike is stable across sessions
with no workaround.

**The "conditional" is about recording polish, not capability.** Before the T14
recording, the presenter must avoid 3 content-drift traps and should fix 5 doc items:

Hard avoid-list for the recording (no code change needed, just don't do these on camera):
1. Do NOT enrich with `threat_intel(...)` / `nvd(...)` — use `threat_score(...)` /
   `cvss_base_score(...)` (N1).
2. Do NOT have Claude execute the verbatim dot-syntax queries embedded in the
   `triage_alerts`/`investigate_host`/`client_overview`/`cross_client_status` prompts —
   they silent-empty (AUDIT-004 + N2).
3. Do NOT copy the short table `name` from `prism_describe` straight into FROM — use the
   sensor-prefixed names (AUDIT-001).

Recommended pre-recording fixes (route via orchestrator → specialists):
- **N1** (server, prism-mcp `resources.rs`): correct the reference's enrichment-function
  list to the registered per-field UDF names; make an unregistered enrich function return
  E-QUERY-039 instead of an internal error. → implementer.
- **N2** (server, prism-query): restore E-QUERY-036 dot-syntax did-you-mean at plan time.
  → implementer.
- **AUDIT-004** (server, prism-mcp prompt bodies): regenerate prompt bodies to use
  sensor-prefixed table names. → implementer.
- **AUDIT-001** (server, prism-mcp prism_describe): emit FROM-ready table names (or add a
  `query_name`/`from_name` field). → implementer / product-owner for the field contract.
- **N3/N4/N5** (runbook v1.5): fix expected error code (E-QUERY-032), Step 3.5 org
  (org-c), Step 6.3 table name (`claroty_audit_logs`/`id`). → product-owner.

None of these block a successful recording if the presenter follows the §6 execution
order and avoid-list. The demo can be recorded today; fixing N1/N2/AUDIT-004/AUDIT-001
would make the analyst-discoverability story airtight.

**Query performance:** 0.00–0.20s for all queries including CrowdStrike and enrichment.
No latency issues. No hangs anywhere in this audit.
