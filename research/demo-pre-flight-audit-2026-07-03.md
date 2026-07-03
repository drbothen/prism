---
document_type: research
producer: automated-audit (scripts/t13-preflight-audit.py — MCP stdio driver)
timestamp: 2026-07-03
topic: T13 Capstone Live Demo — Pre-Flight Re-Audit (post PR #208 + PR #213 merge)
status: complete
feeds: T13 capstone demo preparation; D-1312 blocker verification
branch: develop
supersedes_audit: .factory/research/demo-pre-flight-audit-2026-06-26.md
develop_head: 122228e8
---

# T13 Capstone Live Demo — Pre-Flight Re-Audit (2026-07-03)

**Audit target:** develop@122228e8  
**PRs since last audit:** PR #208 (S-DEMO-FIDELITY-REMEDIATION-001, fidelity fixes N1/N1-B/N2/AUDIT-001/AUDIT-004), PR #213 (S-PERF-GATE-008)  
**Audit script:** `scripts/t13-preflight-audit.py` (18-item live MCP stdio driver)

---

## 1. Setup

Binaries rebuilt at HEAD:
```bash
cargo build --release -p prism-bin -p prism-dtu-demo-server --features dtu,fixture-gen
```

Completed 2026-07-03 (~5 min). Binary timestamps now post-date the fidelity fix (Jul 3
14:14 UTC merge).

DTU fleet start:
```bash
bash scripts/demo-setup.sh   # idempotent; provisions config + 10 keyring creds
bash scripts/demo-run.sh     # starts DTU fleet; writes per-org overlays; emits env block
```

DTU ports this run: ThreatIntel=56229, NVD=56230 (ephemeral; vary per run).

---

## 2. Audit Results

All 18 items PASS. Script invocation:
```bash
PRISM_THREATINTEL_PORT=56229 PRISM_NVD_PORT=56230 python3 scripts/t13-preflight-audit.py
```

| # | Item | Status | Evidence |
|---|------|--------|----------|
| 1 | INIT: MCP server boots and responds | PASS | server={'name': 'prism', 'version': '0.1.0'} |
| 2 | MAJOR-001: list_capabilities client_registered=true | PASS | client_registered=true for org-c |
| 3 | AUDIT-001: prism_describe sensor-prefixed table names | PASS | 10 tables, all sensor-prefixed (armis_alerts, armis_devices, claroty_alerts, claroty_audit_logs, claroty_devices, crowdstrike_detections, crowdstrike_devices, crowdstrike_incidents, cyberint_alerts, cyberint_incidents) |
| 4 | CS-OAUTH: CrowdStrike query returns data | PASS | 3 rows from FROM crowdstrike_detections \| limit 3 |
| 5 | PIPE-SYNTAX: pipe-mode query parses and runs | PASS | 3 rows from FROM armis_devices \| where device_id IS NOT NULL \| limit 3 |
| 6 | HANG-FIX: query_tutorial returns promptly | PASS | 0.00s, 1 message (pure sync, no I/O) |
| 7 | HANG-FIX: investigate_host returns promptly | PASS | 0.00s, 1 message (pure sync, no I/O) |
| 8 | HANG-FIX: list_infusions returns promptly | PASS | NYA -32003 in 0.00s |
| 9 | HANG-FIX: plugin_status returns promptly | PASS | NYA -32003 in 0.00s |
| 10 | HANG-FIX: infusion_status returns promptly | PASS | NYA -32003 in 0.00s |
| 11 | N1: prismql://reference per-field UDF names | PASS | threat_score + cvss_base_score present; no old infusion_id call forms |
| 12 | N1-B: unknown enrich UDF returns E-QUERY-039 | PASS | E-QUERY-039 — enrichment infusion 'nonexistent_udf' is not registered |
| 13 | N1-B F1: SQL builtin (COUNT) NOT E-QUERY-039 | PASS | COUNT executed OK, 1 rows |
| 14 | N2: dot-notation FROM returns E-QUERY-037 | PASS | E-QUERY-037 for FROM crowdstrike.detections \| limit 3 |
| 15 | AUDIT-004: prompts use FROM-ready underscore names | PASS | triage_alerts prompt uses crowdstrike_detections etc. |
| 16 | E-QUERY-032/-037: cyberint for org-a errors | PASS | E-QUERY-032 (org-a has no cyberint sensor) |
| 17 | SCENARIO: Stage 4 armis_devices visible | PASS | 5 rows, sample device_id=dev-0196f4b2-200-0 (scenario_start_secs ~10 days past) |
| 18 | IOC-FIELDS: cyberint iocs_value at Stage 4 | PASS | 5 rows with iocs_value populated |

**SUMMARY: 18 PASS / 0 FAIL / 0 WARN**

---

## 3. D-1312 Blocker Disposition

All D-1312 blockers verified closed:

| Blocker | Disposition |
|---------|-------------|
| MAJOR-001: list_capabilities client_registered=true | CLOSED — PASS (item 2) |
| CrowdStrike OAuth corruption | CLOSED — PASS (item 4): 3 rows returned |
| Runbook pipe-syntax drift (queries fail to parse) | CLOSED — PASS (items 4, 5): pipe syntax accepted |
| query_tutorial / investigate_host hang | CLOSED — PASS (items 6, 7): 0.00s, pure sync |
| list_infusions / plugin_status / infusion_status hang | CLOSED — PASS (items 8-10): NYA -32003 immediate |
| N1: prismql://reference old infusion_id forms | CLOSED — PASS (item 11) |
| N1-B: unknown enrich UDF no error | CLOSED — PASS (item 12): E-QUERY-039 fires |
| N2: dot-notation FROM silently succeeds | CLOSED — PASS (item 14): E-QUERY-037 fires |
| AUDIT-001: dot-notation in prism_describe | CLOSED — PASS (item 3): 10 sensor-prefixed names |
| AUDIT-004: dot-notation in prompts | CLOSED — PASS (item 15): underscore names in prompts |

---

## 4. Key Wire-Protocol Notes

Discovered during audit script development — relevant for future MCP tooling:

1. **Newline-delimited JSON** (not Content-Length framing): prism's rmcp stdio transport
   sends/receives `{...}\n` — NOT LSP-style `Content-Length: N\r\n\r\n{...}`.

2. **Tool name is `query`** (not `query_execute` or `execute_query`): the canonical
   PrismQL tool name is just `query`. Parameter is `clients: ["org-id"]` (array),
   not `client_id`.

3. **Pipe-mode syntax mandatory for FROM queries**: `FROM table LIMIT N` is rejected
   with a parse error. Must use `FROM table | limit N`. SQL SELECT syntax is separate.

4. **Error responses are plain text**: in-band errors from `query` return
   `content[0]["text"]` = `"ERROR: [type] - E-CODE: message"` (not a JSON envelope).
   Success responses return a JSON envelope string with `results` at the top level
   (not at body root).

5. **Safety envelope structure**: `content[0]["text"]` is a JSON string containing
   `{"_meta":{...},"results":{...},"content":[...],"structuredContent":{...}}`.
   The data lives at `envelope["results"]`, not at the top level.

---

## 5. DEMO-READY Verdict

**DEMO-READY: YES**

All 18 audit items PASS against develop@122228e8.  
All D-1312 blockers closed.  
DTU fleet (3 orgs × sensors + ThreatIntel/NVD enrichment) operates correctly.  
Scenario clock at Stage 4 (elapsed ~10 days, well past 600s threshold).

---

## 6. Repeatability

To re-run this audit after future develop changes:

```bash
# 1. Rebuild if binaries are stale
cargo build --release -p prism-bin -p prism-dtu-demo-server --features dtu,fixture-gen

# 2. Start DTU fleet (note the ThreatIntel/NVD ports in output)
bash scripts/demo-run.sh

# 3. Run audit
PRISM_THREATINTEL_PORT=<from step 2> PRISM_NVD_PORT=<from step 2> \
    python3 scripts/t13-preflight-audit.py
```

The script at `scripts/t13-preflight-audit.py` is the canonical pre-flight audit runner.
It handles: server boot, all 18 checks, per-org sensor scoping, scenario clock, IOC fields,
and the D-1312 blocker list. Expected runtime: ~60-90s (dominated by sensor DTU latency on
CrowdStrike, Armis, Cyberint queries).
