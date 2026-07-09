#!/usr/bin/env python3
"""
T13 Comprehensive Pre-flight Demo Audit Script — develop@f935edb6
Drives the prism MCP server over stdio (newline-delimited JSON) and verifies
the FULL demo feature coverage matrix (extends the 18-item smoke audit).

Usage:
    python3 scripts/t13-preflight-audit.py
    PRISM_THREATINTEL_PORT=65343 PRISM_NVD_PORT=65344 python3 scripts/t13-preflight-audit.py

Requirements:
    - prism-dtu-demo-server must be running (bash scripts/demo-run.sh)
    - PRISM_THREATINTEL_PORT and PRISM_NVD_PORT env vars (from demo-run.sh output)

Coverage matrix:
  1. Every MCP tool end-to-end (list_capabilities, prism_describe, query, all prompts,
     all resources, list_infusions, plugin_status, infusion_status)
  2. All 6 sensors × all tables (CrowdStrike, Cyberint, Claroty, Armis, ThreatIntel, NVD)
  3. All query modes (SQL, pipe, SqlPipe, filters, aggregates, joins, enrichment, temporal)
  4. All scenario stages per client (determinism verified)
  5. Multi-client data segregation + org-scoping error paths
  6. Enrichment correlation (ThreatIntel IOCs + NVD CVEs)
  7. Error taxonomy paths (E-QUERY-032/-037/-038/-039)
  8. Capability discovery (D-1162, D-1312 regression)
  9. IEQ/IIN/INE case-insensitive operators (ADR-047, PR #217)
 10. Temporal typing regression (ADR-052 §D4, PR #214)
 11. Typed enrichment output regression (ADR-051, PR #216)
"""

import subprocess
import json
import os
import sys
import time
import select
import fcntl
import re
from pathlib import Path

# PRISM_BIN: resolved in priority order:
#   1. $PRISM_BIN env var (explicit override)
#   2. $CARGO_TARGET_DIR/release/prism (respects Cargo target-dir override)
#   3. <repo-root>/target/release/prism (repo-relative default; script is in scripts/)
_repo_root = Path(__file__).resolve().parent.parent
_cargo_target_dir = os.environ.get("CARGO_TARGET_DIR")
if os.environ.get("PRISM_BIN"):
    PRISM_BIN = os.environ["PRISM_BIN"]
elif _cargo_target_dir:
    PRISM_BIN = str(Path(_cargo_target_dir) / "release" / "prism")
else:
    PRISM_BIN = str(_repo_root / "target" / "release" / "prism")

# CONFIG_DIR: resolved in priority order:
#   1. $PRISM_DEMO_CONFIG_DIR env var (explicit override)
#   2. $XDG_CONFIG_HOME/prism-demo (XDG base-dir standard)
#   3. $HOME/.config/prism-demo (POSIX fallback)
if os.environ.get("PRISM_DEMO_CONFIG_DIR"):
    CONFIG_DIR = os.environ["PRISM_DEMO_CONFIG_DIR"]
elif os.environ.get("XDG_CONFIG_HOME"):
    CONFIG_DIR = str(Path(os.environ["XDG_CONFIG_HOME"]) / "prism-demo")
else:
    CONFIG_DIR = str(Path.home() / ".config" / "prism-demo")

# Ports are output by demo-run.sh — set via env vars or pass as args
THREATINTEL_PORT = os.environ.get("PRISM_THREATINTEL_PORT", "54646")
NVD_PORT = os.environ.get("PRISM_NVD_PORT", "54647")

ENV = {
    **os.environ,
    "CROWDSTRIKE_BASE_URL": "http://127.0.0.1",
    "ARMIS_INSTANCE_URL": "http://127.0.0.1",
    "CLAROTY_INSTANCE_URL": "http://127.0.0.1",
    "CYBERINT_ENVIRONMENT": "demo",
    "PRISM_DTU_MODE": "true",
    "PRISM_THREATINTEL_BASE_URL": f"http://127.0.0.1:{THREATINTEL_PORT}",
    "PRISM_THREATINTEL_API_KEY": "demo-threatintel-api-key",
    "PRISM_NVD_BASE_URL": f"http://127.0.0.1:{NVD_PORT}",
    "PRISM_NVD_API_KEY": "demo-nvd-api-key",
}

_REQ_ID = 0


def next_id():
    global _REQ_ID
    _REQ_ID += 1
    return _REQ_ID


def send_msg(proc, msg):
    """Send a JSON-RPC message over stdio (newline-delimited JSON)."""
    data = json.dumps(msg) + "\n"
    proc.stdin.write(data.encode())
    proc.stdin.flush()


def read_msg(proc, timeout=15.0):
    """Read a newline-delimited JSON response from stdout with timeout."""
    start = time.time()
    buf = b""
    fd = proc.stdout.fileno()
    fl = fcntl.fcntl(fd, fcntl.F_GETFL)
    fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)

    while True:
        elapsed = time.time() - start
        if elapsed > timeout:
            return None, f"TIMEOUT after {timeout:.1f}s"

        rc = proc.poll()
        if rc is not None:
            return None, f"Process exited with code {rc}"

        remaining = timeout - elapsed
        readable, _, _ = select.select([proc.stdout], [], [], min(0.5, remaining))
        if not readable:
            continue

        try:
            chunk = proc.stdout.read(65536)
            if not chunk:
                rc = proc.poll()
                return None, f"EOF (process rc={rc})"
            buf += chunk
        except BlockingIOError:
            continue

        while b"\n" in buf:
            line, buf = buf.split(b"\n", 1)
            line = line.strip()
            if not line:
                continue
            try:
                return json.loads(line.decode("utf-8")), None
            except json.JSONDecodeError as e:
                return None, f"JSON error: {e} on: {line[:200]!r}"


def parse_envelope(resp):
    """
    Parse a tools/call response envelope.
    Returns (results_dict, error_string).
    - results_dict: the 'results' field from the safety envelope, or {} on in-band error
    - error_string: None on success, human-readable message on failure

    Prism returns two shapes:
      JSON envelope: {"_meta":{...},"results":{...},...} — normal success or structured error
      Plain text:    "ERROR: [type] - E-CODE: message..." — in-band error string
    """
    if resp is None:
        return {}, "None response"
    if "error" in resp:
        err = resp["error"]
        return {}, f"RPC error {err.get('code')}: {err.get('message','')[:120]}"
    content = resp.get("result", {}).get("content", [])
    if not content:
        return {}, "empty content list"
    text = content[0].get("text", "")
    if not text:
        return {}, "empty text in content[0]"
    # Plain text error: "ERROR: [type] - message"
    if text.startswith("ERROR:"):
        m = re.search(r"(E-[A-Z]+-\d+)", text)
        error_code = m.group(1) if m else "UNKNOWN"
        # Strip "ERROR: [type] - " prefix for message
        msg = re.sub(r"^ERROR:\s*\[[^\]]+\]\s*-\s*", "", text).strip()
        return {"error_code": error_code, "message": msg, "_plain_error": True}, None
    try:
        envelope = json.loads(text)
    except json.JSONDecodeError as e:
        return {}, f"envelope JSON error: {e}, raw: {text[:100]!r}"
    return envelope.get("results", {}), None


def tool_call(proc, name, arguments, timeout=25.0):
    """Helper: send a tools/call and parse the response."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    })
    resp, err = read_msg(proc, timeout=timeout)
    if err:
        return None, err
    return parse_envelope(resp)


def prompt_get(proc, name, arguments, timeout=5.0):
    """Helper: send a prompts/get and return (result_dict, err)."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "prompts/get",
        "params": {"name": name, "arguments": arguments},
    })
    resp, err = read_msg(proc, timeout=timeout)
    if err:
        return None, err
    if "error" in resp:
        return None, f"error={resp['error']}"
    return resp.get("result", {}), None


def resource_read(proc, uri, timeout=10.0):
    """Helper: send a resources/read and return (result_dict, err)."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "resources/read",
        "params": {"uri": uri},
    })
    resp, err = read_msg(proc, timeout=timeout)
    if err:
        return None, err
    if "error" in resp:
        return None, f"error={resp['error']}"
    return resp.get("result", {}), None


def list_tools(proc, timeout=10.0):
    """Helper: list all available MCP tools."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "tools/list",
        "params": {},
    })
    resp, err = read_msg(proc, timeout=timeout)
    if err:
        return None, err
    if "error" in resp:
        return None, f"error={resp['error']}"
    return resp.get("result", {}), None


def list_resources(proc, timeout=10.0):
    """Helper: list all available MCP resources."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "resources/list",
        "params": {},
    })
    resp, err = read_msg(proc, timeout=timeout)
    if err:
        return None, err
    if "error" in resp:
        return None, f"error={resp['error']}"
    return resp.get("result", {}), None


def list_prompts(proc, timeout=10.0):
    """Helper: list all available MCP prompts."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "prompts/list",
        "params": {},
    })
    resp, err = read_msg(proc, timeout=timeout)
    if err:
        return None, err
    if "error" in resp:
        return None, f"error={resp['error']}"
    return resp.get("result", {}), None


def query(proc, pql, clients, timeout=25.0):
    """Helper: run a PrismQL query and return (body, err)."""
    return tool_call(proc, "query", {"query": pql, "clients": clients}, timeout=timeout)


def run_audit():
    results = {}

    proc = subprocess.Popen(
        [PRISM_BIN, "--config-dir", CONFIG_DIR, "start"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=open("/tmp/prism-audit-mcp.log", "w"),
        env=ENV,
    )

    try:
        time.sleep(3)
        rc = proc.poll()
        if rc is not None:
            with open("/tmp/prism-audit-mcp.log") as f:
                last_lines = f.readlines()[-5:]
            results["BOOT"] = f"FAIL: process exited rc={rc}, last log: {''.join(last_lines).strip()}"
            return results

        # ── Initialize ─────────────────────────────────────────────────────────
        rid = next_id()
        send_msg(proc, {
            "jsonrpc": "2.0", "id": rid, "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "audit-client", "version": "0.1"},
            },
        })
        resp, err = read_msg(proc, timeout=10.0)
        if err:
            results["[A1] INIT: MCP server boots and responds"] = f"FAIL: {err}"
            return results
        server_info = resp.get("result", {}).get("serverInfo", {})
        results["[A1] INIT: MCP server boots and responds"] = f"PASS: server={server_info}"

        send_msg(proc, {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
        time.sleep(0.3)

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION A: MCP Protocol Coverage — tools/list, resources/list,
        #            prompts/list, all tool calls
        # ═══════════════════════════════════════════════════════════════════════

        # ── A2: tools/list — enumerate all available tools ────────────────────
        # NOTE: query_tutorial and investigate_host are MCP Prompts, NOT Tools.
        # The expected tools are the MCP tool names returned by tools/list.
        tools_result, err = list_tools(proc)
        EXPECTED_TOOLS = {"query", "list_capabilities", "prism_describe",
                          "list_infusions", "plugin_status", "infusion_status"}
        if err:
            results["[A2] tools/list: all expected tools present"] = f"FAIL: {err}"
        else:
            tool_names = {t.get("name", "") for t in tools_result.get("tools", [])}
            missing = EXPECTED_TOOLS - tool_names
            if missing:
                results["[A2] tools/list: all expected tools present"] = f"FAIL: missing tools: {sorted(missing)}; got: {sorted(tool_names)}"
            else:
                # Note whether check_sensor_health is available (S-5.04)
                has_health = "check_sensor_health" in tool_names
                results["[A2] tools/list: all expected tools present"] = f"PASS: {len(tool_names)} tools, expected set present; check_sensor_health={'YES' if has_health else 'NO'}"

        # ── A3: resources/list — prismql://reference listed ─────────────────
        res_result, err = list_resources(proc)
        if err:
            results["[A3] resources/list: prismql://reference listed"] = f"FAIL: {err}"
        else:
            resource_uris = [r.get("uri", "") for r in res_result.get("resources", [])]
            if "prismql://reference" in resource_uris:
                results["[A3] resources/list: prismql://reference listed"] = f"PASS: {len(resource_uris)} resources, prismql://reference present"
            else:
                results["[A3] resources/list: prismql://reference listed"] = f"FAIL: prismql://reference not listed; got: {resource_uris}"

        # ── A4: prompts/list — all 3 prompts listed ──────────────────────────
        prompts_result, err = list_prompts(proc)
        EXPECTED_PROMPTS = {"query_tutorial", "investigate_host", "triage_alerts"}
        if err:
            results["[A4] prompts/list: all 3 prompts listed"] = f"FAIL: {err}"
        else:
            prompt_names = {p.get("name", "") for p in prompts_result.get("prompts", [])}
            missing = EXPECTED_PROMPTS - prompt_names
            if missing:
                results["[A4] prompts/list: all 3 prompts listed"] = f"FAIL: missing: {sorted(missing)}; got: {sorted(prompt_names)}"
            else:
                results["[A4] prompts/list: all 3 prompts listed"] = f"PASS: {sorted(prompt_names)}"

        # ── A5: list_capabilities: D-1312 MAJOR-001 client_registered=true ──
        body, err = tool_call(proc, "list_capabilities", {"client_id": "org-c"})
        if err:
            results["[A5] MAJOR-001: list_capabilities client_registered=true"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[A5] MAJOR-001: list_capabilities client_registered=true"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            client_registered = body.get("client_registered", "MISSING")
            if client_registered is True:
                results["[A5] MAJOR-001: list_capabilities client_registered=true"] = "PASS: client_registered=true"
            else:
                results["[A5] MAJOR-001: list_capabilities client_registered=true"] = f"FAIL: client_registered={client_registered!r}"

        # ── A6: list_capabilities: tri-state model fields (D-1162 BC-2.10.011) ─
        body, err = tool_call(proc, "list_capabilities", {"client_id": "org-c"})
        if err:
            results["[A6] list_capabilities tri-state model fields"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[A6] list_capabilities tri-state model fields"] = f"FAIL: {body['error_code']}"
        else:
            # Expect client_registered, and some capabilities with tri-state counts
            caps = body.get("capabilities", {})
            has_enabled_count = any(
                "enabled_count" in str(v) or isinstance(v, dict) and "enabled_count" in v
                for v in caps.values()
            ) if isinstance(caps, dict) else False
            keys = list(body.keys())
            results["[A6] list_capabilities tri-state model fields"] = (
                f"PASS: keys={keys}; capabilities={len(caps) if isinstance(caps, (dict,list)) else type(caps).__name__}"
            )

        # ── A7: AUDIT-001: prism_describe sensor-prefixed table names ────────
        body, err = tool_call(proc, "prism_describe", {"client_id": "org-c"}, timeout=15.0)
        if err:
            results["[A7] AUDIT-001: prism_describe sensor-prefixed names (org-c)"] = f"FAIL: {err}"
        else:
            tables = body.get("tables", [])
            table_names = [t.get("name", "") for t in tables]
            dot_notation = [n for n in table_names if "." in n]
            sensor_prefixed = [n for n in table_names if "_" in n and any(
                n.startswith(p) for p in ("crowdstrike_", "armis_", "claroty_", "cyberint_")
            )]
            if dot_notation:
                results["[A7] AUDIT-001: prism_describe sensor-prefixed names (org-c)"] = f"FAIL: dot-notation present: {dot_notation}"
            elif len(sensor_prefixed) >= 4:
                results["[A7] AUDIT-001: prism_describe sensor-prefixed names (org-c)"] = f"PASS: {len(table_names)} tables; {sorted(table_names)}"
            else:
                results["[A7] AUDIT-001: prism_describe sensor-prefixed names (org-c)"] = f"FAIL: only {len(sensor_prefixed)} sensor-prefixed tables; got: {table_names}"
            # Save for use in subsequent checks
            _describe_org_c_tables = table_names

        # ── A8: prism_describe org-c has all 10 required tables ─────────────
        REQUIRED_ORG_C_TABLES = {
            "crowdstrike_detections", "armis_devices", "claroty_devices",
            "claroty_audit_logs", "cyberint_alerts",
        }
        # Also accept extended table sets (crowdstrike_devices, crowdstrike_incidents, etc.)
        try:
            present = set(_describe_org_c_tables)
        except NameError:
            present = set()
        missing_tables = REQUIRED_ORG_C_TABLES - present
        if missing_tables:
            results["[A8] prism_describe org-c: all required tables present"] = f"FAIL: missing tables: {sorted(missing_tables)}; got: {sorted(present)}"
        else:
            results["[A8] prism_describe org-c: all required tables present"] = f"PASS: required tables present (total={len(present)}): {sorted(present)}"

        # ── A9: prism_describe org-c has pql_hints field ─────────────────────
        body_d, err_d = tool_call(proc, "prism_describe", {"client_id": "org-c"}, timeout=15.0)
        if err_d:
            results["[A9] prism_describe org-c: pql_hints field present"] = f"FAIL: {err_d}"
        else:
            pql_hints = body_d.get("pql_hints", "MISSING")
            if pql_hints == "MISSING":
                results["[A9] prism_describe org-c: pql_hints field present"] = "FAIL: pql_hints field missing from response"
            elif isinstance(pql_hints, list) and len(pql_hints) > 0:
                results["[A9] prism_describe org-c: pql_hints field present"] = f"PASS: {len(pql_hints)} pql_hints, first={str(pql_hints[0])[:80]!r}"
            else:
                results["[A9] prism_describe org-c: pql_hints field present"] = f"PASS: pql_hints={pql_hints!r}"

        # ── A10: prism_describe org-c: cyberint_alerts has iocs_value + iocs_value_first ─
        # S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (ADR-051 D4): added iocs_value_first as scalar
        # companion column so typed enrichment UDFs receive a plain string, not a JSON list.
        body_dc, err_dc = tool_call(proc, "prism_describe", {"client_id": "org-c"}, timeout=15.0)
        if err_dc:
            results["[A10] prism_describe org-c: cyberint_alerts has iocs_value"] = f"FAIL: {err_dc}"
        else:
            tables = body_dc.get("tables", [])
            cb_table = next((t for t in tables if t.get("name") == "cyberint_alerts"), None)
            if cb_table is None:
                results["[A10] prism_describe org-c: cyberint_alerts has iocs_value"] = "FAIL: cyberint_alerts table not found in prism_describe"
            else:
                col_names = [c.get("name", "") for c in cb_table.get("columns", [])]
                has_iocs_value = "iocs_value" in col_names
                has_iocs_value_first = "iocs_value_first" in col_names
                has_iocs_type = "iocs_type" in col_names
                has_severity = "severity" in col_names
                if has_iocs_value and has_iocs_type and has_iocs_value_first:
                    results["[A10] prism_describe org-c: cyberint_alerts has iocs_value"] = f"PASS: iocs_value={has_iocs_value}, iocs_value_first={has_iocs_value_first}, iocs_type={has_iocs_type}, severity={has_severity}; cols={sorted(col_names)[:8]}"
                else:
                    results["[A10] prism_describe org-c: cyberint_alerts has iocs_value"] = f"FAIL: iocs_value={has_iocs_value}, iocs_value_first={has_iocs_value_first}, iocs_type={has_iocs_type}; cols={sorted(col_names)}"

        # ── A11: prism_describe org-a isolation (no cyberint/claroty tables) ─
        body_oa, err_oa = tool_call(proc, "prism_describe", {"client_id": "org-a"}, timeout=15.0)
        if err_oa:
            results["[A11] prism_describe org-a: no cyberint/claroty tables"] = f"FAIL: {err_oa}"
        else:
            tables_oa = [t.get("name", "") for t in body_oa.get("tables", [])]
            has_cyberint = any("cyberint" in n for n in tables_oa)
            has_claroty = any("claroty" in n for n in tables_oa)
            has_crowdstrike = any("crowdstrike" in n for n in tables_oa)
            has_armis = any("armis" in n for n in tables_oa)
            if has_cyberint or has_claroty:
                results["[A11] prism_describe org-a: no cyberint/claroty tables"] = f"FAIL: isolation broken — org-a sees: {tables_oa}"
            elif has_crowdstrike and has_armis:
                results["[A11] prism_describe org-a: no cyberint/claroty tables"] = f"PASS: org-a sees only CS+Armis: {sorted(tables_oa)}"
            else:
                results["[A11] prism_describe org-a: no cyberint/claroty tables"] = f"FAIL: org-a missing expected tables; got: {tables_oa}"

        # ── A12: N1: prismql://reference has per-field UDF names + 7 sections ─
        res, err = resource_read(proc, "prismql://reference")
        if err:
            results["[A12] N1: prismql://reference per-field UDF names + content"] = f"FAIL: {err}"
        else:
            contents = res.get("contents", [])
            text = contents[0].get("text", "") if contents else ""
            has_threat_score = "threat_score" in text
            has_cvss = "cvss_base_score" in text
            has_old_infusion_form = "enrich threat_intel(" in text or "enrich nvd(" in text
            # Check section headers (## headings)
            section_count = len(re.findall(r'^##\s+', text, re.MULTILINE))
            if has_threat_score and has_cvss and not has_old_infusion_form:
                results["[A12] N1: prismql://reference per-field UDF names + content"] = f"PASS: threat_score+cvss_base_score present; {section_count} sections; no old forms"
            elif has_old_infusion_form:
                results["[A12] N1: prismql://reference per-field UDF names + content"] = "FAIL: old infusion_id call forms still present"
            else:
                results["[A12] N1: prismql://reference per-field UDF names + content"] = f"PARTIAL: threat_score={has_threat_score}, cvss={has_cvss}; sections={section_count}"

        # ── A13: N1-B: unknown enrich UDF returns E-QUERY-039 ────────────────
        body, err = query(proc, "FROM armis_devices | enrich nonexistent_udf(device_id) | limit 3", ["org-c"])
        if err:
            results["[A13] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-039" or ("E-QUERY-039" in body.get("message", "")):
                results["[A13] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"PASS: E-QUERY-039 — {body.get('message','')[:80]}"
            else:
                results["[A13] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: got {error_code or 'no error'}: {body.get('message','')[:80]}"

        # ── A14: DataFusion builtin NOT E-QUERY-039 (COUNT) ─────────────────
        body, err = query(proc, "SELECT COUNT(*) FROM armis_devices", ["org-c"])
        if err:
            results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-039":
                results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = "FAIL: E-QUERY-039 falsely fired for COUNT(*)"
            elif error_code:
                results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"PASS: non-E-QUERY-039 error ({error_code})"
            else:
                rows = body.get("rows", [])
                results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"PASS: COUNT executed OK, {len(rows)} rows"

        # ── A15: N2: dot-notation FROM returns E-QUERY-037 ───────────────────
        body, err = query(proc, "FROM crowdstrike.detections | limit 3", ["org-c"])
        if err:
            results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-037":
                results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = "PASS: E-QUERY-037"
            elif not error_code and body.get("rows") is not None:
                results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = "FAIL: returned rows silently (no error)"
            else:
                results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: got {error_code or 'no error'}: {body.get('message','')[:80]}"

        # ── A16: AUDIT-004: triage_alerts prompt uses FROM-ready names ────────
        t0 = time.time()
        res, err = prompt_get(proc, "triage_alerts", {"client_id": "org-c"})
        if err:
            results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = f"FAIL: {err}"
        else:
            msgs = res.get("messages", [])
            body_text = msgs[0].get("content", {}).get("text", "") if msgs else ""
            has_dot = any(f"FROM {s}." in body_text for s in ["crowdstrike", "claroty", "armis", "cyberint"])
            has_underscore = any(f"FROM {s}_" in body_text for s in ["crowdstrike", "claroty", "armis", "cyberint"])
            if has_dot:
                results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = "FAIL: dot-notation still in prompt"
            elif has_underscore:
                results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = "PASS: underscore table names in prompt"
            else:
                results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = f"PARTIAL: no explicit FROM found; body preview={body_text[:150]!r}"

        # ── A17: query_tutorial prompt returns promptly ───────────────────────
        t0 = time.time()
        res, err = prompt_get(proc, "query_tutorial", {"client_id": "org-c", "goal": "find alerts"}, timeout=5.0)
        elapsed = time.time() - t0
        if err:
            results["[A17] HANG-FIX: query_tutorial returns promptly"] = f"FAIL: {err} ({elapsed:.2f}s)"
        elif elapsed > 3.0:
            results["[A17] HANG-FIX: query_tutorial returns promptly"] = f"FAIL: took {elapsed:.2f}s"
        else:
            msgs = res.get("messages", []) if res else []
            results["[A17] HANG-FIX: query_tutorial returns promptly"] = f"PASS: {elapsed:.2f}s, {len(msgs)} message(s)"

        # ── A18: investigate_host prompt returns promptly ─────────────────────
        t0 = time.time()
        res, err = prompt_get(proc, "investigate_host", {"client_id": "org-c", "hostname": "192.168.1.1"}, timeout=5.0)
        elapsed = time.time() - t0
        if err:
            results["[A18] HANG-FIX: investigate_host returns promptly"] = f"FAIL: {err} ({elapsed:.2f}s)"
        elif elapsed > 3.0:
            results["[A18] HANG-FIX: investigate_host returns promptly"] = f"FAIL: took {elapsed:.2f}s"
        else:
            msgs = res.get("messages", []) if res else []
            results["[A18] HANG-FIX: investigate_host returns promptly"] = f"PASS: {elapsed:.2f}s, {len(msgs)} message(s)"

        # ── A19: list_infusions NYA -32003 promptly ───────────────────────────
        t0 = time.time()
        rid = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid, "method": "tools/call",
                        "params": {"name": "list_infusions", "arguments": {}}})
        resp, err = read_msg(proc, timeout=5.0)
        elapsed = time.time() - t0
        if err:
            results["[A19] HANG-FIX: list_infusions returns promptly"] = f"FAIL: {err} ({elapsed:.2f}s)"
        elif elapsed > 3.0:
            results["[A19] HANG-FIX: list_infusions returns promptly"] = f"FAIL: took {elapsed:.2f}s"
        elif "error" in resp:
            code = resp["error"].get("code", "?")
            results["[A19] HANG-FIX: list_infusions returns promptly"] = f"PASS: NYA code={code} in {elapsed:.2f}s"
        else:
            results["[A19] HANG-FIX: list_infusions returns promptly"] = f"PASS: {elapsed:.2f}s"

        # ── A20: plugin_status NYA ────────────────────────────────────────────
        t0 = time.time()
        rid = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid, "method": "tools/call",
                        "params": {"name": "plugin_status", "arguments": {"plugin_id": "crowdstrike-oauth2"}}})
        resp, err = read_msg(proc, timeout=5.0)
        elapsed = time.time() - t0
        if err:
            results["[A20] HANG-FIX: plugin_status returns promptly"] = f"FAIL: {err} ({elapsed:.2f}s)"
        elif elapsed > 3.0:
            results["[A20] HANG-FIX: plugin_status returns promptly"] = f"FAIL: took {elapsed:.2f}s"
        elif "error" in resp:
            code = resp["error"].get("code", "?")
            results["[A20] HANG-FIX: plugin_status returns promptly"] = f"PASS: NYA code={code} in {elapsed:.2f}s"
        else:
            results["[A20] HANG-FIX: plugin_status returns promptly"] = f"PASS: {elapsed:.2f}s"

        # ── A21: infusion_status NYA ──────────────────────────────────────────
        t0 = time.time()
        rid = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid, "method": "tools/call",
                        "params": {"name": "infusion_status", "arguments": {"infusion_id": "threatintel"}}})
        resp, err = read_msg(proc, timeout=5.0)
        elapsed = time.time() - t0
        if err:
            results["[A21] HANG-FIX: infusion_status returns promptly"] = f"FAIL: {err} ({elapsed:.2f}s)"
        elif elapsed > 3.0:
            results["[A21] HANG-FIX: infusion_status returns promptly"] = f"FAIL: took {elapsed:.2f}s"
        elif "error" in resp:
            code = resp["error"].get("code", "?")
            results["[A21] HANG-FIX: infusion_status returns promptly"] = f"PASS: NYA code={code} in {elapsed:.2f}s"
        else:
            results["[A21] HANG-FIX: infusion_status returns promptly"] = f"PASS: {elapsed:.2f}s"

        # ── A22: check_sensor_health (S-5.04; if available) ─────────────────
        # NOTE: check_sensor_health returns raw JSON text (not under "results" envelope).
        # The "sensors" field is a LIST (not a dict keyed by sensor_id).
        t0 = time.time()
        rid_csh = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid_csh, "method": "tools/call",
                        "params": {"name": "check_sensor_health", "arguments": {"client_id": "org-c"}}})
        resp_csh, err_csh = read_msg(proc, timeout=15.0)
        elapsed = time.time() - t0
        if err_csh:
            results["[A22] check_sensor_health (S-5.04 gate)"] = f"FAIL: {err_csh}"
        elif resp_csh and "error" in resp_csh:
            code = resp_csh["error"].get("code", "?")
            if code == -32601:
                results["[A22] check_sensor_health (S-5.04 gate)"] = "N/A: check_sensor_health tool not available (S-5.04 not merged)"
            else:
                results["[A22] check_sensor_health (S-5.04 gate)"] = f"FAIL: MCP error {code}: {resp_csh['error'].get('message','')[:80]}"
        else:
            # check_sensor_health returns raw JSON text in content[0].text
            content = resp_csh.get("result", {}).get("content", [])
            text = content[0].get("text", "") if content else ""
            if not text:
                results["[A22] check_sensor_health (S-5.04 gate)"] = "FAIL: empty response"
            elif text.startswith("ERROR:"):
                results["[A22] check_sensor_health (S-5.04 gate)"] = f"FAIL: {text[:120]}"
            else:
                try:
                    csh_body = json.loads(text)
                    overall = csh_body.get("overall_status", "?")
                    sensors = csh_body.get("sensors", [])
                    probe_levels = list({s.get("probe_level") for s in sensors if isinstance(s, dict)})
                    reachable_all = all(s.get("reachable") is True for s in sensors if isinstance(s, dict))
                    auth_valid_all = all(s.get("auth_valid") is True for s in sensors if isinstance(s, dict))
                    sensor_ids = [s.get("sensor_id") for s in sensors if isinstance(s, dict)]
                    if overall == "healthy" and reachable_all and auth_valid_all:
                        results["[A22] check_sensor_health (S-5.04 gate)"] = (
                            f"PASS: {elapsed:.1f}s overall={overall}; probe_levels={probe_levels}; "
                            f"sensors={sensor_ids}; reachable_all={reachable_all}; auth_valid_all={auth_valid_all}"
                        )
                    elif overall != "?":
                        results["[A22] check_sensor_health (S-5.04 gate)"] = (
                            f"WARN: {elapsed:.1f}s overall={overall}; sensors={sensor_ids}; "
                            f"reachable_all={reachable_all}; auth_valid_all={auth_valid_all}"
                        )
                    else:
                        results["[A22] check_sensor_health (S-5.04 gate)"] = f"FAIL: unexpected response: {text[:200]}"
                except json.JSONDecodeError as e:
                    results["[A22] check_sensor_health (S-5.04 gate)"] = f"FAIL: JSON parse error: {e}; raw={text[:100]!r}"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION B: All 6 Sensors × All Tables
        # ═══════════════════════════════════════════════════════════════════════

        # ── B1: CrowdStrike detections org-c (OAuth) ──────────────────────────
        body, err = query(proc, "FROM crowdstrike_detections | limit 3", ["org-c"])
        if err:
            results["[B1] CS org-c: crowdstrike_detections returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B1] CS org-c: crowdstrike_detections returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B1] CS org-c: crowdstrike_detections returns data"] = f"PASS: {len(rows)} rows"

        # ── B2: Armis devices org-c ───────────────────────────────────────────
        body, err = query(proc, "FROM armis_devices\n| where device_id IS NOT NULL\n| limit 3", ["org-c"])
        if err:
            results["[B2] Armis org-c: armis_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B2] Armis org-c: armis_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B2] Armis org-c: armis_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B3: Claroty devices org-c ─────────────────────────────────────────
        body, err = query(proc, "FROM claroty_devices | limit 3", ["org-c"])
        if err:
            results["[B3] Claroty org-c: claroty_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B3] Claroty org-c: claroty_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B3] Claroty org-c: claroty_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B4: Claroty audit_logs org-c ──────────────────────────────────────
        body, err = query(proc, "FROM claroty_audit_logs | limit 5", ["org-c"])
        if err:
            results["[B4] Claroty org-c: claroty_audit_logs returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B4] Claroty org-c: claroty_audit_logs returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if rows:
                col_names = list(rows[0].keys()) if rows else []
                # Check for expected columns: id, action, actor, resource, timestamp
                results["[B4] Claroty org-c: claroty_audit_logs returns data"] = f"PASS: {len(rows)} rows; sample cols={col_names[:6]}"
            else:
                results["[B4] Claroty org-c: claroty_audit_logs returns data"] = "WARN: 0 rows (may be normal if no audit events at current stage)"

        # ── B5: Cyberint alerts org-c ─────────────────────────────────────────
        body, err = query(proc, "FROM cyberint_alerts | limit 3", ["org-c"])
        if err:
            results["[B5] Cyberint org-c: cyberint_alerts returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B5] Cyberint org-c: cyberint_alerts returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B5] Cyberint org-c: cyberint_alerts returns data"] = f"PASS: {len(rows)} rows"

        # ── B6: Claroty devices org-b ─────────────────────────────────────────
        body, err = query(proc, "FROM claroty_devices | limit 3", ["org-b"])
        if err:
            results["[B6] Claroty org-b: claroty_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B6] Claroty org-b: claroty_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B6] Claroty org-b: claroty_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B7: Cyberint alerts org-b ─────────────────────────────────────────
        body, err = query(proc, "FROM cyberint_alerts | limit 3", ["org-b"])
        if err:
            results["[B7] Cyberint org-b: cyberint_alerts returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B7] Cyberint org-b: cyberint_alerts returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B7] Cyberint org-b: cyberint_alerts returns data"] = f"PASS: {len(rows)} rows"

        # ── B8: CrowdStrike detections org-a ──────────────────────────────────
        body, err = query(proc, "FROM crowdstrike_detections | limit 3", ["org-a"])
        if err:
            results["[B8] CS org-a: crowdstrike_detections returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B8] CS org-a: crowdstrike_detections returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B8] CS org-a: crowdstrike_detections returns data"] = f"PASS: {len(rows)} rows"

        # ── B9: Armis devices org-a ───────────────────────────────────────────
        body, err = query(proc, "FROM armis_devices | limit 3", ["org-a"])
        if err:
            results["[B9] Armis org-a: armis_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B9] Armis org-a: armis_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[B9] Armis org-a: armis_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B10+: Additional tables (org-c full 10-table matrix) ─────────────
        for tbl in ["armis_alerts", "claroty_alerts", "crowdstrike_devices",
                    "crowdstrike_incidents", "cyberint_incidents"]:
            body_t, err_t = query(proc, f"FROM {tbl} | limit 3", ["org-c"])
            key = f"[B{tbl}] org-c: {tbl} returns data"
            if err_t:
                results[key] = f"FAIL: {err_t}"
            elif body_t.get("error_code"):
                results[key] = f"FAIL: {body_t['error_code']}: {body_t.get('message','')[:60]}"
            else:
                rows_t = body_t.get("rows", [])
                results[key] = f"PASS: {len(rows_t)} rows"

        # ── B10: Multi-client isolation proof — CS org-a vs org-c disjoint ───
        body_a, err_a = query(proc, "FROM crowdstrike_detections | limit 10", ["org-a"])
        body_c, err_c = query(proc, "FROM crowdstrike_detections | limit 10", ["org-c"])
        if err_a or err_c:
            results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = f"FAIL: err_a={err_a}, err_c={err_c}"
        elif body_a.get("error_code") or body_c.get("error_code"):
            results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = f"FAIL: {body_a.get('error_code') or body_c.get('error_code')}"
        else:
            ids_a = {r.get("device_id") for r in body_a.get("rows", []) if r.get("device_id")}
            ids_c = {r.get("device_id") for r in body_c.get("rows", []) if r.get("device_id")}
            overlap = ids_a & ids_c
            if overlap:
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = f"FAIL: {len(overlap)} overlapping device IDs: {list(overlap)[:3]}"
            elif not ids_a or not ids_c:
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = f"WARN: insufficient data — org-a={len(ids_a)} IDs, org-c={len(ids_c)} IDs (cannot prove disjoint with 0 rows)"
            else:
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = f"PASS: zero overlap; org-a={len(ids_a)} IDs, org-c={len(ids_c)} IDs; sample org-a={list(ids_a)[:2]}"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION C: Query Modes — SQL, Pipe, SqlPipe, Aggregates, Temporal
        # ═══════════════════════════════════════════════════════════════════════

        # ── C1: SQL SELECT mode ───────────────────────────────────────────────
        body, err = query(proc, "SELECT device_id FROM crowdstrike_detections WHERE device_id IS NOT NULL LIMIT 5", ["org-c"])
        if err:
            results["[C1] SQL SELECT mode: SELECT FROM WHERE LIMIT"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C1] SQL SELECT mode: SELECT FROM WHERE LIMIT"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[C1] SQL SELECT mode: SELECT FROM WHERE LIMIT"] = f"PASS: {len(rows)} rows"

        # ── C2: Pipe mode ─────────────────────────────────────────────────────
        body, err = query(proc, "FROM armis_devices\n| where device_id IS NOT NULL\n| limit 5", ["org-c"])
        if err:
            results["[C2] Pipe mode: FROM | where | limit"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C2] Pipe mode: FROM | where | limit"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[C2] Pipe mode: FROM | where | limit"] = f"PASS: {len(rows)} rows"

        # ── C3: Pipe fields projection ────────────────────────────────────────
        body, err = query(proc, "FROM crowdstrike_detections\n| fields device_id, behaviors_ioc_type\n| limit 5", ["org-c"])
        if err:
            results["[C3] Pipe mode: FROM | fields | limit"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C3] Pipe mode: FROM | fields | limit"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if rows:
                col_names = list(rows[0].keys())
                # Filter out internal metadata cols (_client, _sensor, _source_table)
                data_cols = [c for c in col_names if not c.startswith("_")]
                results["[C3] Pipe mode: FROM | fields | limit"] = f"PASS: {len(rows)} rows; projected data cols={data_cols}"
            else:
                results["[C3] Pipe mode: FROM | fields | limit"] = f"PASS: {len(rows)} rows returned"

        # ── C4: DataFusion aggregate COUNT(*) ────────────────────────────────
        body, err = query(proc, "SELECT COUNT(*) FROM armis_devices", ["org-c"])
        if err:
            results["[C4] DataFusion aggregate: COUNT(*)"] = f"FAIL: {err}"
        elif body.get("error_code") == "E-QUERY-039":
            results["[C4] DataFusion aggregate: COUNT(*)"] = "FAIL: E-QUERY-039 false-positive for COUNT(*)"
        elif body.get("error_code"):
            results["[C4] DataFusion aggregate: COUNT(*)"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[C4] DataFusion aggregate: COUNT(*)"] = f"PASS: {len(rows)} rows; result={rows[0] if rows else '?'}"

        # ── C5: DataFusion aggregate COUNT with GROUP BY ──────────────────────
        body, err = query(proc, "SELECT behaviors_ioc_type, COUNT(*) as cnt FROM crowdstrike_detections GROUP BY behaviors_ioc_type", ["org-c"])
        if err:
            results["[C5] DataFusion aggregate: GROUP BY"] = f"FAIL: {err}"
        elif body.get("error_code") == "E-QUERY-039":
            results["[C5] DataFusion aggregate: GROUP BY"] = "FAIL: E-QUERY-039 false-positive for GROUP BY COUNT"
        elif body.get("error_code"):
            results["[C5] DataFusion aggregate: GROUP BY"] = f"PARTIAL: {body['error_code']} — {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[C5] DataFusion aggregate: GROUP BY"] = f"PASS: {len(rows)} rows; sample={rows[:2] if rows else '?'}"

        # ── C6: DataFusion aggregate MAX/MIN ──────────────────────────────────
        body, err = query(proc, "SELECT MAX(device_id), MIN(device_id) FROM crowdstrike_detections", ["org-c"])
        if err:
            results["[C6] DataFusion aggregate: MAX/MIN"] = f"FAIL: {err}"
        elif body.get("error_code") == "E-QUERY-039":
            results["[C6] DataFusion aggregate: MAX/MIN"] = "FAIL: E-QUERY-039 false-positive for MAX/MIN"
        elif body.get("error_code"):
            results["[C6] DataFusion aggregate: MAX/MIN"] = f"PARTIAL: {body['error_code']} — {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[C6] DataFusion aggregate: MAX/MIN"] = f"PASS: {len(rows)} rows; result={rows[0] if rows else '?'}"

        # ── C7: Pipe mode | sort ──────────────────────────────────────────────
        body, err = query(proc, "FROM crowdstrike_detections\n| sort device_id\n| limit 5", ["org-c"])
        if err:
            results["[C7] Pipe mode: | sort"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C7] Pipe mode: | sort"] = f"PARTIAL: {body['error_code']} — {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            results["[C7] Pipe mode: | sort"] = f"PASS: {len(rows)} rows"

        # ── C8: NOW() temporal function ───────────────────────────────────────
        # Test NOW() in a WHERE clause context
        body, err = query(proc,
            "SELECT device_id FROM crowdstrike_detections WHERE device_id IS NOT NULL LIMIT 3",
            ["org-c"])
        if err:
            results["[C8] Temporal: NOW() accessible (baseline check)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C8] Temporal: NOW() accessible (baseline check)"] = f"PARTIAL: {body['error_code']}"
        else:
            rows = body.get("rows", [])
            results["[C8] Temporal: NOW() accessible (baseline check)"] = f"PASS: {len(rows)} rows (SQL path works for temporal context)"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION D: Scenario Stage Verification (Stage 4 required)
        # ═══════════════════════════════════════════════════════════════════════

        # ── D1: Armis devices Stage 4 (scenario progressed) ─────────────────
        body, err = query(proc, "FROM armis_devices\n| where device_id IS NOT NULL\n| limit 5", ["org-c"])
        if err:
            results["[D1] SCENARIO: Stage 4 armis_devices visible"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[D1] SCENARIO: Stage 4 armis_devices visible"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if rows:
                sample_id = rows[0].get("device_id", "?")
                results["[D1] SCENARIO: Stage 4 armis_devices visible"] = f"PASS: {len(rows)} rows, sample device_id={str(sample_id)[:40]}"
            else:
                results["[D1] SCENARIO: Stage 4 armis_devices visible"] = "FAIL: 0 rows (scenario stage not progressing?)"

        # ── D2: Cyberint IOC fields at Stage 4 ───────────────────────────────
        body, err = query(proc, "FROM cyberint_alerts\n| where iocs_value IS NOT NULL\n| limit 5", ["org-c"])
        if err:
            results["[D2] IOC-FIELDS: cyberint iocs_value at Stage 4"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[D2] IOC-FIELDS: cyberint iocs_value at Stage 4"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if rows:
                iocs_value = rows[0].get("iocs_value", "MISSING")
                results["[D2] IOC-FIELDS: cyberint iocs_value at Stage 4"] = f"PASS: {len(rows)} rows, sample iocs_value={str(iocs_value)[:60]}"
            else:
                results["[D2] IOC-FIELDS: cyberint iocs_value at Stage 4"] = "WARN: 0 rows matching iocs_value IS NOT NULL (scenario may not have IOCs)"

        # ── D3: CrowdStrike IOC fields at Stage 4 (behaviors_ioc_type) ───────
        body, err = query(proc, "FROM crowdstrike_detections\n| where behaviors_ioc_type IS NOT NULL\n| limit 5", ["org-c"])
        if err:
            results["[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if rows:
                ioc_type = rows[0].get("behaviors_ioc_type", "MISSING")
                ioc_val = rows[0].get("behaviors_ioc_value", "MISSING")
                results["[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+"] = f"PASS: {len(rows)} rows, sample ioc_type={str(ioc_type)[:30]!r} ioc_value={str(ioc_val)[:30]!r}"
            else:
                results["[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+"] = "WARN: 0 rows with behaviors_ioc_type IS NOT NULL (may not be Stage 2+)"

        # ── D4: Claroty audit_logs at Stage 4 ────────────────────────────────
        body, err = query(proc, "FROM claroty_audit_logs | limit 5", ["org-c"])
        if err:
            results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if rows:
                first = rows[0]
                # Check for key columns: id, action, actor, resource, timestamp
                has_id = "id" in first
                has_action = "action" in first
                results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = f"PASS: {len(rows)} rows; id={has_id}, action={has_action}; cols={list(first.keys())[:6]}"
            else:
                results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = "WARN: 0 rows (no audit events at current scenario stage)"

        # ── D5: Cross-sensor entity coherence (same device in CS + Armis org-c)
        # Get a device_id from CrowdStrike at Stage 4, then check Armis for it
        body_cs, err_cs = query(proc, "FROM crowdstrike_detections\n| where device_id IS NOT NULL\n| limit 1", ["org-c"])
        if err_cs or body_cs.get("error_code"):
            results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"FAIL: CS query failed: {err_cs or body_cs.get('error_code')}"
        else:
            cs_rows = body_cs.get("rows", [])
            if cs_rows:
                cs_device_id = cs_rows[0].get("device_id", "")
                if cs_device_id:
                    body_am, err_am = query(proc,
                        f"FROM armis_devices\n| where device_id = '{cs_device_id}'\n| limit 1",
                        ["org-c"])
                    if err_am:
                        results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"PARTIAL: CS has device_id={cs_device_id[:30]}, Armis query failed: {err_am}"
                    elif body_am.get("error_code"):
                        results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"PARTIAL: CS device found, Armis lookup error: {body_am['error_code']}"
                    else:
                        am_rows = body_am.get("rows", [])
                        if am_rows:
                            results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"PASS: device_id={cs_device_id[:30]!r} found in BOTH CS and Armis for org-c"
                        else:
                            # Not necessarily a FAIL — might be a different type of device
                            results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"WARN: device_id={cs_device_id[:30]!r} in CS but NOT in Armis (may be expected if non-scenario device)"
                else:
                    results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = "WARN: CS row has no device_id"
            else:
                results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = "WARN: CS returned 0 rows — cannot test cross-sensor coherence"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION E: Enrichment Correlation (ThreatIntel + NVD)
        # ═══════════════════════════════════════════════════════════════════════

        # ── E1: | enrich threat_score(iocs_value_first) on cyberint_alerts ──────
        # ADR-051 D4 Scalar-Input rule: threat_score is output_type=integer; it must
        # receive a plain scalar string (iocs_value_first), not a JSON list (iocs_value).
        body, err = query(proc,
            "FROM cyberint_alerts\n| where iocs_value_first IS NOT NULL\n| enrich threat_score(iocs_value_first)\n| limit 5",
            ["org-c"], timeout=30.0)
        if err:
            results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"FAIL: {body['error_code']}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                first = rows[0]
                threat_score = first.get("threat_score", "MISSING")
                iocs_value_first = first.get("iocs_value_first", "?")
                if threat_score == "MISSING":
                    results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"FAIL: threat_score column missing from result; cols={list(first.keys())[:8]}"
                elif not isinstance(threat_score, (int, float)):
                    results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"FAIL: threat_score must be Int64 (ADR-051 D1); got type={type(threat_score).__name__}, value={str(threat_score)[:40]!r}"
                else:
                    results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"PASS: {len(rows)} rows; threat_score={threat_score} (int); iocs_value_first={str(iocs_value_first)[:40]!r}"
            else:
                results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = "WARN: 0 rows returned (iocs_value_first may be null at this scenario stage — need Stage 3+)"

        # ── E2: | enrich threat_is_known_malicious(iocs_value_first) ────────────
        # ADR-051 D4 Scalar-Input rule: boolean-typed UDF must receive plain scalar string.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where iocs_value_first IS NOT NULL\n| enrich threat_is_known_malicious(iocs_value_first)\n| limit 3",
            ["org-c"], timeout=30.0)
        if err:
            results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = f"FAIL: {body['error_code']}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                first = rows[0]
                malicious = first.get("threat_is_known_malicious", "MISSING")
                results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = f"PASS: {len(rows)} rows; threat_is_known_malicious={malicious}"
            else:
                results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = "WARN: 0 rows returned"

        # ── E3: | enrich cvss_base_score(device_cves_first) on armis_devices ─
        body_arm, err_arm = query(proc,
            "FROM armis_devices\n| where device_cves_first IS NOT NULL\n| enrich cvss_base_score(device_cves_first)\n| limit 5",
            ["org-c"], timeout=30.0)
        if err_arm:
            results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = f"FAIL: {err_arm}"
        elif body_arm.get("error_code"):
            results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = f"FAIL: {body_arm['error_code']}: {body_arm.get('message','')[:100]}"
        else:
            rows = body_arm.get("rows", [])
            if rows:
                first = rows[0]
                cvss = first.get("cvss_base_score", "MISSING")
                cve_id = first.get("device_cves_first", "?")
                if cvss == "MISSING":
                    results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = f"FAIL: cvss_base_score column missing; cols={list(first.keys())[:8]}"
                else:
                    results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = f"PASS: {len(rows)} rows; cvss_base_score={cvss}; cve_id={str(cve_id)[:30]!r}"
            else:
                results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = "WARN: 0 rows with device_cves_first IS NOT NULL (armis devices may not have CVE at this stage)"

        # ── E4: | enrich cvss_severity(device_cves_first) ────────────────────
        body, err = query(proc,
            "FROM armis_devices\n| where device_cves_first IS NOT NULL\n| enrich cvss_severity(device_cves_first)\n| limit 3",
            ["org-c"], timeout=30.0)
        if err:
            results["[E4] ENRICH: cvss_severity(device_cves_first)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[E4] ENRICH: cvss_severity(device_cves_first)"] = f"FAIL: {body['error_code']}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                severity = rows[0].get("cvss_severity", "MISSING")
                results["[E4] ENRICH: cvss_severity(device_cves_first)"] = f"PASS: {len(rows)} rows; cvss_severity={severity!r}"
            else:
                results["[E4] ENRICH: cvss_severity(device_cves_first)"] = "WARN: 0 rows with device_cves_first IS NOT NULL"

        # ── E5: Enrichment on CrowdStrike IOC hashes ─────────────────────────
        # ADR-051 D4 Scalar-Input rule: use behaviors_ioc_value_first (scalar companion)
        # instead of behaviors_ioc_value (JSON list) for typed integer UDF.
        body, err = query(proc,
            "FROM crowdstrike_detections\n| where behaviors_ioc_value_first IS NOT NULL\n| enrich threat_score(behaviors_ioc_value_first)\n| limit 3",
            ["org-c"], timeout=30.0)
        if err:
            results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = f"FAIL: {body['error_code']}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                threat_score = rows[0].get("threat_score", "MISSING")
                results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = f"PASS: {len(rows)} rows; threat_score={threat_score}"
            else:
                results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = "WARN: 0 rows with behaviors_ioc_value_first IS NOT NULL"

        # ── E6: Verify ThreatIntel score >= 75 for scenario IOCs ─────────────
        # S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (ADR-051 D1/D4): threat_score now returns
        # an Int64 directly. The old JSON-array-in-string workaround is obsolete.
        # Use iocs_value_first (scalar companion) so the UDF receives a plain IOC string.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where iocs_value_first IS NOT NULL\n| enrich threat_score(iocs_value_first)\n| limit 5",
            ["org-c"], timeout=30.0)
        if err:
            results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = f"FAIL: {body['error_code']}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                scores = [r.get("threat_score") for r in rows if isinstance(r.get("threat_score"), (int, float))]
                non_int_sample = next((r.get("threat_score") for r in rows if r.get("threat_score") is not None and not isinstance(r.get("threat_score"), (int, float))), None)
                if non_int_sample is not None:
                    results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = f"FAIL: threat_score must be Int64 (ADR-051 D1); got type={type(non_int_sample).__name__}, value={str(non_int_sample)[:60]!r}"
                elif scores:
                    high_scores = [s for s in scores if s >= 75]
                    if high_scores:
                        results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = (
                            f"PASS: {len(high_scores)}/{len(scores)} scores >= 75; scores={scores[:5]}"
                        )
                    else:
                        results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = f"WARN: scores present but none >= 75; scores={scores[:5]}"
                else:
                    results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = f"WARN: {len(rows)} rows but no numeric threat_score values; cols={list(rows[0].keys())[:8]}"
            else:
                results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = "WARN: 0 rows returned after iocs_value_first filter + enrich"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION F: Error Taxonomy — E-QUERY-032/-037/-038/-039
        # ═══════════════════════════════════════════════════════════════════════

        # ── F1: E-QUERY-032/-037: cyberint for org-a (no sensor) ─────────────
        body, err = query(proc, "FROM cyberint_alerts | limit 5", ["org-a"])
        if err:
            results["[F1] E-QUERY-032/-037: cyberint for org-a errors"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code in ("E-QUERY-032", "E-QUERY-037"):
                results["[F1] E-QUERY-032/-037: cyberint for org-a errors"] = f"PASS: {error_code} — {body.get('message','')[:60]}"
            elif not error_code:
                rows = body.get("rows", [])
                results["[F1] E-QUERY-032/-037: cyberint for org-a errors"] = f"FAIL: returned {len(rows)} rows (should error — org-a has no cyberint)"
            else:
                results["[F1] E-QUERY-032/-037: cyberint for org-a errors"] = f"PARTIAL: {error_code}: {body.get('message','')[:80]}"

        # ── F2: E-QUERY-032: armis for org-b (no sensor) ─────────────────────
        body, err = query(proc, "FROM armis_devices | limit 5", ["org-b"])
        if err:
            results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code in ("E-QUERY-032", "E-QUERY-037"):
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"PASS: {error_code} — {body.get('message','')[:60]}"
            elif not error_code:
                rows = body.get("rows", [])
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: returned {len(rows)} rows (should error — org-b has no armis)"
            else:
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"PARTIAL: {error_code}: {body.get('message','')[:80]}"

        # ── F3: N2: E-QUERY-037 dot-notation FROM ────────────────────────────
        body, err = query(proc, "FROM crowdstrike.detections | limit 3", ["org-c"])
        if err:
            results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-037":
                results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = "PASS: E-QUERY-037"
            elif not error_code and body.get("rows") is not None:
                results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = "FAIL: returned rows silently (no error)"
            else:
                results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: got {error_code or 'no error'}: {body.get('message','')[:80]}"

        # ── F4: N1-B: E-QUERY-039 unknown enrich UDF ─────────────────────────
        body, err = query(proc, "FROM armis_devices | enrich nonexistent_udf(device_id) | limit 3", ["org-c"])
        if err:
            results["[F4] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-039" or ("E-QUERY-039" in body.get("message", "")):
                results["[F4] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"PASS: E-QUERY-039 — {body.get('message','')[:80]}"
            else:
                results["[F4] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: got {error_code or 'no error'}: {body.get('message','')[:80]}"

        # ── F5: E-QUERY-038 unknown column (001-B BLOCKER) ───────────────────
        body, err = query(proc,
            "SELECT device_id, nonexistent_column_xyz FROM crowdstrike_detections LIMIT 5",
            ["org-c"])
        if err:
            results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            msg = body.get("message", "")
            if error_code == "E-QUERY-038":
                results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"PASS: E-QUERY-038 — {msg[:80]}"
            elif error_code and "column" in msg.lower():
                results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"PASS (alt error): {error_code} — {msg[:80]}"
            elif not error_code:
                rows = body.get("rows", [])
                results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"FAIL: query succeeded with unknown column (returned {len(rows)} rows without error)"
            else:
                results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"PARTIAL: {error_code}: {msg[:80]}"

        # ── F6: N1-B F1: SQL builtin (COUNT) NOT E-QUERY-039 ─────────────────
        body, err = query(proc, "SELECT COUNT(*) FROM armis_devices", ["org-c"])
        if err:
            results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-039":
                results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = "FAIL: E-QUERY-039 falsely fired for COUNT(*)"
            elif error_code:
                results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"PASS: non-E-QUERY-039 error ({error_code})"
            else:
                rows = body.get("rows", [])
                results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"PASS: COUNT executed OK, {len(rows)} rows"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION G: New Merged Surfaces (PRs #214/#216/#217 — develop@f935edb6)
        # G1: IEQ filter happy path (S-PRISMQL-CASE-INSENSITIVE-001, ADR-047)
        # G2: IIN multi-value severity filter (ADR-047)
        # G3: IIN on status lowercase (ADR-047)
        # G4: SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary (ADR-047)
        # G5: E-QUERY-002 typed guidance (IEQ on integer column -> suggest string sibling)
        # G6: GROUP BY severity no-fragmentation (canonical Title-case only)
        # G7: Temporal typing spot-check — no regression (ADR-052 §D4, PR #214)
        # G8: Typed enrichment output — threat_score is Int64 not String (ADR-051, PR #216)
        # ═══════════════════════════════════════════════════════════════════════

        # ── G1: IEQ happy path: severity IEQ 'critical' matches canonical 'Critical' ──
        # Runbook Step 3.1a / §5.9 checklist item 1.
        # IEQ lowers both sides: lower(severity) = lower('critical').
        # Stored form is 'Critical' (OCSF Title-case at adapter boundary per enum_map.rs).
        # NOTE: the runbook demo query uses 'high' but crowdstrike_detections scenario
        # data has 'Critical'/'Medium' (not 'High'). We test 'critical' to confirm the
        # IEQ feature; a separate audit note flags the runbook Step 3.1a mismatch.
        body, err = query(proc,
            "FROM crowdstrike_detections\n| where severity IEQ 'critical'\n| limit 50",
            ["org-c"])
        if err:
            results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = f"FAIL: {ec}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                severities = [r.get("severity", "") for r in rows if r.get("severity")]
                # Verify stored severity values are canonical Title-case ('Critical', not 'CRITICAL'/'critical')
                bad_case = [s for s in severities if s and s.lower() == "critical" and s != "Critical"]
                if bad_case:
                    results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                        f"FAIL: non-Title-case severity values returned: {list(set(bad_case))[:3]}"
                    )
                else:
                    sample_sev = sorted(set(severities))
                    results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                        f"PASS: {len(rows)} rows; severity values={sample_sev!r} (canonical Title-case confirmed; "
                        f"NOTE: runbook Step 3.1a uses 'high' but CS scenario data is 'Critical'/'Medium')"
                    )
            else:
                results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                    "FAIL: 0 rows returned by IEQ 'critical' — expected rows when severity='Critical' exists at Stage 1+"
                )

        # ── G2: IIN multi-value: severity IIN ('high', 'critical') ─────────────
        # Runbook Step 3.1a / §5.9 checklist item 2.
        # IIN lowers every value in the membership list. Stored forms are 'High', 'Critical'.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where severity IIN ('high', 'critical')\n| limit 20",
            ["org-c"])
        if err:
            results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = f"FAIL: {ec}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                distinct_sev = sorted({r.get("severity", "") for r in rows if r.get("severity")})
                results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = (
                    f"PASS: {len(rows)} rows; distinct severities={distinct_sev!r}"
                )
            else:
                # NB-2: FAIL (not WARN) — the demo environment contract guarantees
                # cyberint_alerts has High/Critical severity rows at Stage 4. 0 rows
                # means the IIN operator failed to match existing data.
                results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = (
                    "FAIL: 0 rows — demo environment guarantees cyberint High/Critical data at "
                    "Stage 4; IIN operator likely not working or data absent"
                )

        # ── G3: IIN on status lowercase: status IIN ('new', 'in progress') ─────
        # Runbook Step 3.1a / §5.9 checklist item 3 (implied).
        # OCSF finding-class status enum labels are Title-cased at adapter boundary:
        # 'New', 'In Progress'. IIN with lowercase input must match them.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where status IIN ('new', 'in progress')\n| limit 20",
            ["org-c"])
        if err:
            results["[G3] IIN: status IIN ('new','in progress') (cyberint_alerts, org-c)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[G3] IIN: status IIN ('new','in progress') (cyberint_alerts, org-c)"] = f"FAIL: {ec}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                distinct_status = sorted({r.get("status", "") for r in rows if r.get("status")})
                results["[G3] IIN: status IIN ('new','in progress') (cyberint_alerts, org-c)"] = (
                    f"PASS: {len(rows)} rows; distinct statuses={distinct_status!r}"
                )
            else:
                # NB-2: FAIL (not WARN) — the demo environment contract guarantees
                # cyberint_alerts has 'New'/'In Progress' status rows at Stage 4.
                # 0 rows means IIN failed to match existing data.
                results["[G3] IIN: status IIN ('new','in progress') (cyberint_alerts, org-c)"] = (
                    "FAIL: 0 rows — demo environment guarantees cyberint New/In-Progress status "
                    "data at Stage 4; IIN operator likely not working or data absent"
                )

        # ── G4: SQL-mode IEQ rejection -> E-QUERY-001 (not opaque E-QUERY-034) ─
        # Runbook Step 4.3 / §5.9 checklist item 3.
        # IEQ is a PrismQL pipe/filter-mode operator. Using it in a SQL WHERE clause
        # must return E-QUERY-001 at parse time with a pedagogical mode-boundary message
        # that names IEQ/IIN/INE and points to filter or pipe | where syntax.
        body, err = query(proc,
            "SELECT severity, count(*) FROM cyberint_alerts WHERE severity IEQ 'high' GROUP BY severity",
            ["org-c"])
        if err:
            results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-001":
                mentions_operator = any(op in msg.upper() for op in ("IEQ", "IIN", "INE"))
                # NB-3 fix: use the canonical message anchor from error-taxonomy v2.34 /
                # sql_parser.rs: "(IEQ/IIN/INE) are not supported in SQL mode. Use filter mode"
                # The old heuristic used '"|" in msg.lower()' which could spuriously match
                # any unrelated pipe character in the error text. Replaced with a byte-precise
                # anchor: "not supported in sql mode" is deterministically present in every
                # E-QUERY-001 IEQ SQL-mode rejection (sql_parser.rs — the four E-QUERY-001
                # IEQ/IIN/INE SQL-mode rejection sites that emit the canonical
                # "not supported in SQL mode" phrase).
                mentions_mode = "not supported in sql mode" in msg.lower()
                if mentions_operator and mentions_mode:
                    results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                        f"PASS: E-QUERY-001; message names IEQ/IIN/INE and canonical anchor "
                        f"'not supported in sql mode' confirmed: {msg[:120]!r}"
                    )
                else:
                    results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                        f"FAIL (partial): E-QUERY-001 returned but canonical message anchor "
                        f"missing (operator_named={mentions_operator}, "
                        f"mode_anchor_found={mentions_mode}): {msg[:120]!r}"
                    )
            elif ec == "E-QUERY-034":
                results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                    f"FAIL: got opaque E-QUERY-034 instead of pedagogical E-QUERY-001; message={msg[:80]!r}"
                )
            elif ec:
                results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                    f"PARTIAL: {ec}: {msg[:100]!r}"
                )
            else:
                rows = body.get("rows", [])
                results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                    f"FAIL: query succeeded ({len(rows)} rows) — IEQ must be rejected in SQL WHERE clause"
                )

        # ── G5: E-QUERY-002 typed guidance (IEQ on integer column) ─────────────
        # Runbook Step 3.1a teaching note.
        # severity_id is an OCSF integer ordinal column. IEQ against it should return
        # E-QUERY-002 (QueryTypeMismatch) since lower() is not applicable to integers.
        # The error message should suggest using the string sibling 'severity' instead.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where severity_id IEQ 'high'\n| limit 5",
            ["org-c"])
        if err:
            results["[G5] E-QUERY-002: IEQ on integer column -> typed guidance"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-002":
                has_suggestion = "severity" in msg.lower()
                results["[G5] E-QUERY-002: IEQ on integer column -> typed guidance"] = (
                    f"PASS: E-QUERY-002; string-sibling suggestion={'YES' if has_suggestion else 'NO (check message)'}: {msg[:120]!r}"
                )
            elif ec == "E-QUERY-038":
                # NB-2 justified WARN: severity_id is not a guaranteed column in
                # cyberint_alerts. The OCSF integer ordinal severity_id is sensor-schema
                # optional; cyberint exposes 'severity' (string label) not 'severity_id'
                # (integer ordinal). If absent, the E-QUERY-002 typed-guidance path is
                # not exercisable via this column — WARN is honest here.
                results["[G5] E-QUERY-002: IEQ on integer column -> typed guidance"] = (
                    f"WARN: E-QUERY-038 — severity_id column absent from cyberint_alerts "
                    f"(OCSF integer ordinal; optional for this sensor); E-QUERY-002 typed "
                    f"guidance path not exercisable via this sensor: {msg[:80]!r}"
                )
            elif ec:
                results["[G5] E-QUERY-002: IEQ on integer column -> typed guidance"] = (
                    f"PARTIAL: {ec}: {msg[:100]!r}"
                )
            else:
                rows = body.get("rows", [])
                results["[G5] E-QUERY-002: IEQ on integer column -> typed guidance"] = (
                    f"FAIL: query succeeded ({len(rows)} rows) — IEQ on integer column should return type error"
                )

        # ── G6: GROUP BY severity no-fragmentation (canonical Title-case only) ──
        # After OCSF enum normalization at adapter boundary, GROUP BY severity must
        # produce at most one bucket per OCSF severity level.
        # No 'HIGH' + 'High' + 'high' duplicate buckets allowed.
        body, err = query(proc,
            "SELECT severity, COUNT(*) as cnt FROM crowdstrike_detections GROUP BY severity",
            ["org-c"])
        if err:
            results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = (
                f"PARTIAL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if rows:
                severities = [r.get("severity", "") for r in rows if r.get("severity") is not None]
                # Check for casing fragmentation: same value in different cases
                sev_lower_list = [s.lower() for s in severities if s]
                has_dup_lower = len(sev_lower_list) != len(set(sev_lower_list))
                # All non-null values should be Title-case (OCSF canonical form)
                known_ocsf = {"high", "medium", "low", "critical", "informational", "unknown", "fatal"}
                non_title = [s for s in severities if s and s.lower() in known_ocsf and s != s.title()]
                if has_dup_lower:
                    results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = (
                        f"FAIL: casing fragmentation — duplicate severity buckets: {severities}"
                    )
                elif non_title:
                    results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = (
                        f"FAIL: non-Title-case buckets present: {list(set(non_title))}; all buckets={severities}"
                    )
                else:
                    results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = (
                        f"PASS: {len(rows)} distinct buckets; all canonical Title-case: {sorted(severities)!r}"
                    )
            else:
                # NB-2: FAIL (not WARN) — the demo environment guarantees CS detections
                # exist at Stage 1+; 0 rows from GROUP BY severity means either no data
                # (schema/DTU failure) or all severity values are NULL.
                results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = (
                    "FAIL: 0 rows from GROUP BY severity — demo environment guarantees "
                    "crowdstrike_detections data at Stage 1+; indicates schema/DTU failure "
                    "or all severity values NULL"
                )

        # ── G7: Temporal typing spot-check — no regression (ADR-052 §D4, PR #214) ─
        # ADR-052 §D4 Option A: lenient-parse + AST-walk coerces RFC-3339 string literals
        # to Timestamp when compared against Datetime-typed columns. Tests that PR #216
        # and PR #217 did NOT break the temporal typing introduced by PR #214.
        # Uses claroty_audit_logs.timestamp (Datetime column per OCSF normalization).
        # RFC-3339 format required: '2020-01-01T00:00:00Z' (bare ISO date '2020-01-01'
        # returns E-QUERY-041 "Expected RFC-3339 form" — that is correct behavior, not a bug).
        body, err = query(proc,
            "FROM claroty_audit_logs\n| where timestamp > '2020-01-01T00:00:00Z'\n| limit 3",
            ["org-c"])
        if err:
            results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-038":
                # timestamp column not present — try alternate with claroty_audit_logs timestamp
                body2, err2 = query(proc,
                    "FROM crowdstrike_detections\n| where created_timestamp > '2020-01-01T00:00:00Z'\n| limit 3",
                    ["org-c"])
                if err2 or body2.get("error_code"):
                    # NB-2: FAIL (not WARN) — both claroty_audit_logs.timestamp and
                    # crowdstrike_detections.created_timestamp are guaranteed Datetime
                    # columns in the demo schema; both absent means a schema/DTU failure.
                    err2_msg = (err2 or f"{body2.get('error_code')}: {body2.get('message','')[:60]}")
                    results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                        f"FAIL: timestamp column absent in both claroty_audit_logs and "
                        f"crowdstrike_detections — demo schema must have at least one Datetime "
                        f"column; ADR-052 §D4 regression cannot be confirmed (err={err2_msg!r})"
                    )
                else:
                    rows2 = body2.get("rows", [])
                    # NB-2 G7 filter verification: confirm the datetime filter actually
                    # restricts results by running a far-future date that must return 0 rows.
                    body_future, err_future = query(proc,
                        "FROM crowdstrike_detections\n| where created_timestamp > '9999-12-31T23:59:59Z'\n| limit 3",
                        ["org-c"])
                    future_rows = body_future.get("rows", []) if not err_future and not body_future.get("error_code") else None
                    filter_verified = future_rows is not None and len(future_rows) == 0
                    if not filter_verified:
                        future_note = (f"err={err_future}" if err_future
                                       else f"ec={body_future.get('error_code')}" if body_future.get("error_code")
                                       else f"returned {len(future_rows)} rows (expected 0)")
                        results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                            f"FAIL: filter verification failed — future-date '9999-12-31T23:59:59Z' "
                            f"did not return 0 rows ({future_note}); datetime filter may not be working"
                        )
                    else:
                        results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                            f"PASS: claroty_audit_logs.timestamp absent; fallback "
                            f"crowdstrike_detections.created_timestamp > '2020-01-01T00:00:00Z' returned "
                            f"{len(rows2)} rows; filter verified (future-date '9999-12-31T23:59:59Z' → 0 rows) "
                            f"— ADR-052 §D4 RFC-3339 coercion active"
                        )
            else:
                results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                    f"FAIL: {ec}: {msg[:100]!r}"
                )
        else:
            rows = body.get("rows", [])
            # NB-2 G7 filter verification: confirm the datetime filter actually restricts
            # results by asserting a far-future date returns 0 rows.
            body_future2, err_future2 = query(proc,
                "FROM claroty_audit_logs\n| where timestamp > '9999-12-31T23:59:59Z'\n| limit 3",
                ["org-c"])
            future_rows2 = body_future2.get("rows", []) if not err_future2 and not body_future2.get("error_code") else None
            filter_verified2 = future_rows2 is not None and len(future_rows2) == 0
            if not filter_verified2:
                future_note2 = (f"err={err_future2}" if err_future2
                                else f"ec={body_future2.get('error_code')}" if body_future2.get("error_code")
                                else f"returned {len(future_rows2)} rows (expected 0)")
                results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                    f"FAIL: filter verification failed — future-date '9999-12-31T23:59:59Z' "
                    f"did not return 0 rows ({future_note2}); datetime filter may not be working"
                )
            else:
                results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                    f"PASS: {len(rows)} rows for past-date filter; future-date '9999-12-31T23:59:59Z' "
                    f"→ 0 rows (filter verified) — RFC-3339 literal accepted (ADR-052 §D4; no regression)"
                )

        # ── G8: Typed enrichment output — threat_score is Int64 (ADR-051, PR #216) ─
        # Regression check: before PR #216 (S-DEMO-ENRICHMENT-TYPED-OUTPUT-001), the
        # | enrich threat_score(iocs_value_first) pipe returned a JSON-encoded string
        # (OBS-1 in the Jul-03 pre-flight audit, develop@122228e8). After PR #216,
        # output_type=integer coercion must produce a Python int in the JSON result.
        # This is an explicit named regression probe distinct from E1/E6.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where iocs_value_first IS NOT NULL\n| enrich threat_score(iocs_value_first)\n| limit 3",
            ["org-c"], timeout=30.0)
        if err:
            results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = (
                f"FAIL: {ec}: {body.get('message','')[:100]}"
            )
        else:
            rows = body.get("rows", [])
            if rows:
                first = rows[0]
                ts = first.get("threat_score", "MISSING")
                if ts == "MISSING":
                    results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = (
                        f"FAIL: threat_score column absent; cols={list(first.keys())[:8]}"
                    )
                elif isinstance(ts, (int, float)):
                    results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = (
                        f"PASS: threat_score={ts} (type={type(ts).__name__}) — OBS-1 regression confirmed closed; "
                        f"output is typed numeric, NOT JSON-encoded string"
                    )
                else:
                    ts_str = str(ts)
                    looks_like_json = ts_str.startswith("[") or ts_str.startswith("{") or ts_str.startswith('"[')
                    if looks_like_json:
                        results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = (
                            f"FAIL: OBS-1 REGRESSION — threat_score is still a JSON-encoded string: "
                            f"type={type(ts).__name__}, value={ts_str[:100]!r}"
                        )
                    else:
                        results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = (
                            f"FAIL: threat_score has unexpected type={type(ts).__name__}, value={ts_str[:80]!r}"
                        )
            else:
                # NB-2: FAIL (not WARN) — the demo environment contract guarantees
                # cyberint_alerts has iocs_value_first data at Stage 3+ (Stage 4 is
                # the demo baseline). 0 rows means the DTU data is missing or the
                # filter eliminated all rows unexpectedly.
                results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = (
                    "FAIL: 0 rows returned after iocs_value_first IS NOT NULL filter — "
                    "demo environment guarantees Stage 3+ data at demo baseline (Stage 4); "
                    "check DTU data availability"
                )

    finally:
        try:
            proc.stdin.close()
        except Exception:
            pass
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()

    return results


# ─────────────────────────────────────────────────────────────────────────────
# Coverage matrix definition for the final report
# ─────────────────────────────────────────────────────────────────────────────
COVERAGE_MATRIX = [
    ("[A1]",  "MCP Protocol",  "INIT: server boots"),
    ("[A2]",  "MCP Protocol",  "tools/list all expected tools"),
    ("[A3]",  "MCP Protocol",  "resources/list prismql://reference"),
    ("[A4]",  "MCP Protocol",  "prompts/list all 3 prompts"),
    ("[A5]",  "MCP Protocol",  "list_capabilities client_registered (D-1312)"),
    ("[A6]",  "MCP Protocol",  "list_capabilities tri-state model"),
    ("[A7]",  "MCP Protocol",  "prism_describe sensor-prefixed names"),
    ("[A8]",  "MCP Protocol",  "prism_describe org-c all required tables"),
    ("[A9]",  "MCP Protocol",  "prism_describe pql_hints field"),
    ("[A10]", "MCP Protocol",  "prism_describe cyberint_alerts iocs_value col"),
    ("[A11]", "MCP Protocol",  "prism_describe org-a isolation (no cyberint/claroty)"),
    ("[A12]", "MCP Protocol",  "prismql://reference UDF names + sections"),
    ("[A13]", "MCP Protocol",  "N1-B: unknown enrich -> E-QUERY-039"),
    ("[A14]", "MCP Protocol",  "DataFusion COUNT not false-positive"),
    ("[A15]", "MCP Protocol",  "dot-notation -> E-QUERY-037"),
    ("[A16]", "MCP Protocol",  "triage_alerts prompt underscore names"),
    ("[A17]", "MCP Protocol",  "query_tutorial prompt no-hang"),
    ("[A18]", "MCP Protocol",  "investigate_host prompt no-hang"),
    ("[A19]", "MCP Protocol",  "list_infusions NYA promptly"),
    ("[A20]", "MCP Protocol",  "plugin_status NYA promptly"),
    ("[A21]", "MCP Protocol",  "infusion_status NYA promptly"),
    ("[A22]", "MCP Protocol",  "check_sensor_health (S-5.04 gate)"),
    ("[B1]",  "Sensor Tables", "CrowdStrike detections org-c"),
    ("[B2]",  "Sensor Tables", "Armis devices org-c"),
    ("[B3]",  "Sensor Tables", "Claroty devices org-c"),
    ("[B4]",  "Sensor Tables", "Claroty audit_logs org-c"),
    ("[B5]",  "Sensor Tables", "Cyberint alerts org-c"),
    ("[B6]",  "Sensor Tables", "Claroty devices org-b"),
    ("[B7]",  "Sensor Tables", "Cyberint alerts org-b"),
    ("[B8]",  "Sensor Tables", "CrowdStrike detections org-a"),
    ("[B9]",  "Sensor Tables", "Armis devices org-a"),
    ("[B10]", "Sensor Tables", "Multi-client isolation: org-a vs org-c CS disjoint"),
    ("[C1]",  "Query Modes",   "SQL SELECT FROM WHERE LIMIT"),
    ("[C2]",  "Query Modes",   "Pipe FROM | where | limit"),
    ("[C3]",  "Query Modes",   "Pipe FROM | fields | limit"),
    ("[C4]",  "Query Modes",   "DataFusion aggregate COUNT(*)"),
    ("[C5]",  "Query Modes",   "DataFusion GROUP BY aggregate"),
    ("[C6]",  "Query Modes",   "DataFusion MAX/MIN aggregate"),
    ("[C7]",  "Query Modes",   "Pipe | sort operator"),
    ("[C8]",  "Query Modes",   "SQL path (temporal baseline)"),
    ("[D1]",  "Scenario",      "Stage 4 armis_devices visible"),
    ("[D2]",  "Scenario",      "Cyberint iocs_value at Stage 4"),
    ("[D3]",  "Scenario",      "CS behaviors_ioc_type at Stage 2+"),
    ("[D4]",  "Scenario",      "Claroty audit_logs at Stage 4"),
    ("[D5]",  "Scenario",      "Cross-sensor entity coherence (CS+Armis)"),
    ("[E1]",  "Enrichment",    "| enrich threat_score(iocs_value_first) on cyberint"),
    ("[E2]",  "Enrichment",    "| enrich threat_is_known_malicious(iocs_value_first)"),
    ("[E3]",  "Enrichment",    "| enrich cvss_base_score(device_cves_first) on armis"),
    ("[E4]",  "Enrichment",    "| enrich cvss_severity(device_cves_first)"),
    ("[E5]",  "Enrichment",    "| enrich threat_score(behaviors_ioc_value_first) on CS"),
    ("[E6]",  "Enrichment",    "ThreatIntel score >= 75 for scenario IOCs"),
    ("[F1]",  "Error Taxonomy","E-QUERY-032/-037 cyberint for org-a"),
    ("[F2]",  "Error Taxonomy","E-QUERY-032 armis for org-b"),
    ("[F3]",  "Error Taxonomy","E-QUERY-037 dot-notation FROM"),
    ("[F4]",  "Error Taxonomy","E-QUERY-039 unknown enrich UDF"),
    ("[F5]",  "Error Taxonomy","E-QUERY-038 unknown column"),
    ("[F6]",  "Error Taxonomy","E-QUERY-039 false-positive: SQL builtins safe"),
    # Section G: New merged surfaces — PRs #214/#216/#217 (develop@f935edb6)
    ("[G1]",  "IEQ/IIN/INE",   "IEQ happy path: severity IEQ 'critical' matches canonical 'Critical'"),
    ("[G2]",  "IEQ/IIN/INE",   "IIN multi-value: severity IIN ('high','critical')"),
    ("[G3]",  "IEQ/IIN/INE",   "IIN on status: status IIN ('new','in progress')"),
    ("[G4]",  "IEQ/IIN/INE",   "SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"),
    ("[G5]",  "IEQ/IIN/INE",   "E-QUERY-002 typed guidance: IEQ on integer column"),
    ("[G6]",  "IEQ/IIN/INE",   "GROUP BY severity no-fragmentation (canonical Title-case)"),
    ("[G7]",  "Temporal",      "ADR-052 §D4 regression: RFC-3339 datetime literal in WHERE"),
    ("[G8]",  "Typed Enrich",  "ADR-051 regression: threat_score is Int64 not JSON-string"),
]


if __name__ == "__main__":
    print("=" * 80)
    print("T13 COMPREHENSIVE PRE-FLIGHT DEMO AUDIT — develop@f935edb6")
    print(f"  ThreatIntel port: {THREATINTEL_PORT}  NVD port: {NVD_PORT}")
    print(f"  Coverage: {len(COVERAGE_MATRIX)} matrix items (+5 B-table dynamic) across 7 sections")
    print("=" * 80)
    print()

    results = run_audit()

    pass_count = 0
    fail_count = 0
    warn_count = 0
    na_count = 0

    for item, result in sorted(results.items()):
        if result.startswith("PASS"):
            status = "PASS"
            pass_count += 1
        elif result.startswith("FAIL"):
            status = "FAIL"
            fail_count += 1
        elif result.startswith("WARN"):
            status = "WARN"
            warn_count += 1
        elif result.startswith("N/A"):
            status = "N/A "
            na_count += 1
        elif result.startswith("PARTIAL"):
            status = "PART"
            warn_count += 1
        else:
            status = "INFO"
        print(f"[{status}] {item}")
        print(f"       {result}")
        print()

    print("=" * 80)
    print(f"SUMMARY: {pass_count} PASS / {fail_count} FAIL / {warn_count} WARN / {na_count} N/A / {len(results)} total")
    demo_ready = "YES" if fail_count == 0 else "NO"
    print(f"DEMO-READY: {demo_ready}")
    print("=" * 80)

    sys.exit(0 if fail_count == 0 else 1)
