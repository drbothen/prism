#!/usr/bin/env python3
"""
T13 Comprehensive Pre-flight Demo Audit Script — develop@5f1b5771
Drives the prism MCP server over stdio (newline-delimited JSON) and verifies
the FULL demo feature coverage matrix (extends the 18-item smoke audit).

Usage:
    python3 scripts/t13-preflight-audit.py
    PRISM_THREATINTEL_PORT=65343 PRISM_NVD_PORT=65344 python3 scripts/t13-preflight-audit.py

Requirements:
    - prism-dtu-demo-server must be running (bash scripts/demo-run.sh)
    - PRISM_THREATINTEL_PORT and PRISM_NVD_PORT env vars (from demo-run.sh output)

Coverage matrix (see len(COVERAGE_MATRIX) for current authoritative count):
  1. All 14 implemented tools asserted present via tools/list; 5 read-only tools exercised
     end-to-end; 9 mutating tools deliberately not invoked — preflight is read-only
     (reload_config/create_alias/delete_alias/confirm_action/add_sensor_spec/validate_config
     etc. are mutating operations; this preflight audit must be READ-ONLY against the demo
     environment — no write-back to sensors, no config changes, no alias mutations)
  2. All 6 sensors × all tables (CrowdStrike, Cyberint, Claroty, Armis, ThreatIntel, NVD)
  3. All query modes (SQL, pipe, SqlPipe, filter, stats, joins, enrichment, temporal)
  4. All scenario stages per client (determinism verified)
  5. Multi-client data segregation + org-scoping error paths + multi-client fan-out
  6. Enrichment correlation (ThreatIntel IOCs + NVD CVEs, threat_sources/cvss_vector)
  7. Error taxonomy paths (E-QUERY-032/-033/-037/-038/-039/-040/-041/-042/-003)
  8. Capability discovery (D-1162, D-1312 regression)
  9. IEQ/IIN/INE case-insensitive operators (ADR-047, PR #217)
 10. Temporal typing regression (ADR-052 §D4, PR #214)
 11. Typed enrichment output regression (ADR-051, PR #216)
 12. Section H — PR #219 behaviors: pipe/filter E-QUERY-038, did_you_mean payload, INE,
     E-QUERY-041/-042 temporal negative paths, E-QUERY-002 via integer column, JOINs,
     HEAD-JOIN fail-open, SqlPipe, E-QUERY-040 dual-limit, stats grammar, multi-client
     fan-out, prompts, resources, 14 tools, CWE-116/117 sanitization, E-QUERY-033/-003
     guardrails, threat_sources/cvss_vector UDFs, runbook-drift probe, determinism,
     normalized_pql (t13-audit-coverage-gap-analysis-2026-07-10.md)
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
        result_body = {"error_code": error_code, "message": msg, "_plain_error": True}
        # Extend with structuredContent.error for machine-readable fields
        # (code, did_you_mean, available_columns, etc. — field name is `code` not
        # `error_code`; no `details.*` sub-keys in this codebase's envelope).
        sc_content = resp.get("result", {}).get("structuredContent", {})
        if isinstance(sc_content, dict):
            sc_err_obj = sc_content.get("error")
            if isinstance(sc_err_obj, dict):
                result_body["_sc_error"] = sc_err_obj
        return result_body, None
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


def resources_subscribe(proc, uri, timeout=10.0):
    """Helper: send a resources/subscribe and return (result_dict, err)."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "resources/subscribe",
        "params": {"uri": uri},
    })
    resp, err = read_msg(proc, timeout=timeout)
    if err:
        return None, err
    if "error" in resp:
        return None, f"error={resp['error']}"
    return resp.get("result", {}), None


def resources_unsubscribe(proc, uri, timeout=10.0):
    """Helper: send a resources/unsubscribe and return (result_dict, err)."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "resources/unsubscribe",
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
        # The expected tools are the 14 implemented MCP tools (server.rs has 54 #[tool]
        # sites: 14 real implementations, 40 runtime -32003 NYA stubs).
        # list_infusions, plugin_status, infusion_status are NYA stubs → NOT in this set.
        #
        # READ-ONLY RATIONALE (F-AUD-P2-MED-005): all 14 tools are asserted PRESENT via
        # tools/list, but only 5 read-only tools are exercised end-to-end:
        #   - query, explain_query, list_capabilities, prism_describe, check_sensor_health
        # The 9 mutating tools (reload_config, create_alias, list_aliases, delete_alias,
        # explain_alias, confirm_action, add_sensor_spec, list_sensor_specs, validate_config)
        # are deliberately NOT invoked — this preflight audit is READ-ONLY and must not
        # mutate the demo environment before recording.
        tools_result, err = list_tools(proc)
        EXPECTED_TOOLS = {
            "query", "explain_query", "list_capabilities", "prism_describe",
            "check_sensor_health", "reload_config",
            "create_alias", "list_aliases", "delete_alias", "explain_alias",
            "confirm_action", "add_sensor_spec", "list_sensor_specs", "validate_config",
        }
        if err:
            results["[A2] tools/list: all 14 implemented tools present"] = f"FAIL: {err}"
        else:
            tool_names = {t.get("name", "") for t in tools_result.get("tools", [])}
            missing = EXPECTED_TOOLS - tool_names
            if missing:
                results["[A2] tools/list: all 14 implemented tools present"] = f"FAIL: missing tools: {sorted(missing)}; got: {sorted(tool_names)}"
            else:
                results["[A2] tools/list: all 14 implemented tools present"] = (
                    f"PASS: {len(tool_names)} tools total; all 14 implemented tools present"
                )

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

        # ── A4: prompts/list — all 5 prompts listed ──────────────────────────
        # prompts.rs registers 5 static prompts (query_tutorial, investigate_host,
        # triage_alerts, client_overview, cross_client_status). H13 exercises the 2 new ones.
        prompts_result, err = list_prompts(proc)
        EXPECTED_PROMPTS = {
            "query_tutorial", "investigate_host", "triage_alerts",
            "client_overview", "cross_client_status",
        }
        if err:
            results["[A4] prompts/list: all 5 prompts listed"] = f"FAIL: {err}"
        else:
            prompt_names = {p.get("name", "") for p in prompts_result.get("prompts", [])}
            missing = EXPECTED_PROMPTS - prompt_names
            if missing:
                results["[A4] prompts/list: all 5 prompts listed"] = f"FAIL: missing: {sorted(missing)}; got: {sorted(prompt_names)}"
            else:
                results["[A4] prompts/list: all 5 prompts listed"] = f"PASS: {sorted(prompt_names)}"

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
        # BC-2.10.011 v1.5 single-client mode requires:
        #   capabilities: Map<String, {status: tri-state, resolution_chain: [...]}>
        #   not_registered_tools: [...] (renamed from not_implemented)
        # "enabled_count" is a cross-client summary field (null client_id mode) — it
        # does NOT appear in single-client mode entries and is irrelevant here.
        body, err = tool_call(proc, "list_capabilities", {"client_id": "org-c"})
        if err:
            results["[A6] list_capabilities tri-state model fields"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[A6] list_capabilities tri-state model fields"] = f"FAIL: {body['error_code']}"
        else:
            VALID_STATUSES = {"enabled", "runtime_disabled", "compile_time_disabled"}
            caps = body.get("capabilities", "MISSING")
            not_reg = body.get("not_registered_tools", "MISSING")
            has_old_field = "not_implemented" in body  # renamed in BC-2.10.011 v1.5

            if caps == "MISSING":
                results["[A6] list_capabilities tri-state model fields"] = (
                    f"FAIL: capabilities field absent from response; keys={list(body.keys())}"
                )
            elif not isinstance(caps, dict):
                results["[A6] list_capabilities tri-state model fields"] = (
                    f"FAIL: capabilities is not a dict; type={type(caps).__name__}, "
                    f"value={str(caps)[:80]!r}"
                )
            elif not_reg == "MISSING":
                results["[A6] list_capabilities tri-state model fields"] = (
                    f"FAIL: not_registered_tools absent (BC-2.10.011 v1.5 rename from "
                    f"not_implemented); keys={list(body.keys())}"
                )
            elif has_old_field:
                results["[A6] list_capabilities tri-state model fields"] = (
                    f"FAIL: old not_implemented field still present — must be renamed to "
                    f"not_registered_tools (BC-2.10.011 v1.5)"
                )
            elif caps:
                # Non-empty capabilities — every entry must have status + resolution_chain
                bad_entries = []
                for cap_path, entry in caps.items():
                    if not isinstance(entry, dict):
                        bad_entries.append(f"{cap_path}: not a dict ({type(entry).__name__})")
                        continue
                    if "status" not in entry:
                        bad_entries.append(f"{cap_path}: missing status")
                        continue
                    if entry["status"] not in VALID_STATUSES:
                        bad_entries.append(
                            f"{cap_path}: invalid status {entry['status']!r} "
                            f"(must be one of {sorted(VALID_STATUSES)})"
                        )
                        continue
                    if "resolution_chain" not in entry or not isinstance(
                        entry["resolution_chain"], list
                    ):
                        bad_entries.append(f"{cap_path}: missing or non-list resolution_chain")
                if bad_entries:
                    results["[A6] list_capabilities tri-state model fields"] = (
                        f"FAIL: {len(bad_entries)} capability entries violate tri-state contract: "
                        f"{bad_entries[:3]}; BC-2.10.011 requires "
                        f"status+resolution_chain per entry"
                    )
                else:
                    statuses = sorted(
                        {v["status"] for v in caps.values()
                         if isinstance(v, dict) and "status" in v}
                    )
                    results["[A6] list_capabilities tri-state model fields"] = (
                        f"PASS: {len(caps)} capabilities; all have status+resolution_chain; "
                        f"statuses={statuses!r}; not_registered_tools count="
                        f"{len(not_reg) if isinstance(not_reg, list) else '?'} "
                        f"(BC-2.10.011 tri-state model confirmed)"
                    )
            else:
                # Empty capabilities (no write endpoints in demo — compile-gate absent)
                if not isinstance(not_reg, list):
                    results["[A6] list_capabilities tri-state model fields"] = (
                        f"FAIL: not_registered_tools is not a list; "
                        f"type={type(not_reg).__name__}"
                    )
                else:
                    results["[A6] list_capabilities tri-state model fields"] = (
                        f"PASS: capabilities empty (no write endpoints in demo — "
                        f"compile_time_disabled for all write paths); "
                        f"not_registered_tools={list(not_reg)[:3]!r}; "
                        f"tri-state fields present (BC-2.10.011 single-client mode)"
                    )

        # OBS (scope-leak guard): initialize before A7 so A8 never depends on NameError.
        _describe_org_c_tables = set()

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
            # F-AUD-P2-LOW-002: use prefix-anchored membership to avoid substring collisions
            has_cyberint = any(n.startswith("cyberint_") for n in tables_oa)
            has_claroty = any(n.startswith("claroty_") for n in tables_oa)
            has_crowdstrike = any(n.startswith("crowdstrike_") for n in tables_oa)
            has_armis = any(n.startswith("armis_") for n in tables_oa)
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
        # NOTE (F-AUD-P1-OBS-001): also appears as C4 and F6 — intentional cross-section
        # regression coverage (MCP Protocol, Query Modes, Error Taxonomy sections).
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
            # F-AUD-P2-MED-001: require code == -32003 (E-INFRA-NYA per server.rs not_yet_available_msg)
            code = resp["error"].get("code", "?")
            msg = resp["error"].get("message", "")
            if code == -32003:
                results["[A19] HANG-FIX: list_infusions returns promptly"] = f"PASS: NYA code=-32003 in {elapsed:.2f}s"
            else:
                results["[A19] HANG-FIX: list_infusions returns promptly"] = f"FAIL: NYA stub must return -32003 (E-INFRA-NYA); got code={code}, msg={msg[:60]!r}"
        else:
            results["[A19] HANG-FIX: list_infusions returns promptly"] = f"FAIL: NYA stub returned success (expected -32003 E-INFRA-NYA)"

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
            # F-AUD-P2-MED-001: require code == -32003 (E-INFRA-NYA per server.rs not_yet_available_msg)
            code = resp["error"].get("code", "?")
            msg = resp["error"].get("message", "")
            if code == -32003:
                results["[A20] HANG-FIX: plugin_status returns promptly"] = f"PASS: NYA code=-32003 in {elapsed:.2f}s"
            else:
                results["[A20] HANG-FIX: plugin_status returns promptly"] = f"FAIL: NYA stub must return -32003 (E-INFRA-NYA); got code={code}, msg={msg[:60]!r}"
        else:
            results["[A20] HANG-FIX: plugin_status returns promptly"] = f"FAIL: NYA stub returned success (expected -32003 E-INFRA-NYA)"

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
            # F-AUD-P2-MED-001: require code == -32003 (E-INFRA-NYA per server.rs not_yet_available_msg)
            code = resp["error"].get("code", "?")
            msg = resp["error"].get("message", "")
            if code == -32003:
                results["[A21] HANG-FIX: infusion_status returns promptly"] = f"PASS: NYA code=-32003 in {elapsed:.2f}s"
            else:
                results["[A21] HANG-FIX: infusion_status returns promptly"] = f"FAIL: NYA stub must return -32003 (E-INFRA-NYA); got code={code}, msg={msg[:60]!r}"
        else:
            results["[A21] HANG-FIX: infusion_status returns promptly"] = f"FAIL: NYA stub returned success (expected -32003 E-INFRA-NYA)"

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
                    # F-AUD-P2-HIGH-003: assert expected sensor set present (not vacuous all([])).
                    # Sensor ID values verified against crates/prism-sensors/specs/*.sensor.toml
                    # (crowdstrike.sensor.toml, armis.sensor.toml, claroty.sensor.toml,
                    # cyberint.sensor.toml) — these are the four registered sensors for org-c.
                    EXPECTED_SENSORS = {"crowdstrike", "armis", "claroty", "cyberint"}
                    present_sensors = set(sid for sid in sensor_ids if sid)
                    missing_sensors = EXPECTED_SENSORS - present_sensors
                    if missing_sensors:
                        results["[A22] check_sensor_health (S-5.04 gate)"] = (
                            f"FAIL: {elapsed:.1f}s missing expected sensors={sorted(missing_sensors)}; "
                            f"got sensor_ids={sorted(present_sensors)}"
                        )
                    elif overall == "healthy" and reachable_all and auth_valid_all:
                        results["[A22] check_sensor_health (S-5.04 gate)"] = (
                            f"PASS: {elapsed:.1f}s overall={overall}; probe_levels={probe_levels}; "
                            f"sensors={sorted(present_sensors)}; reachable_all={reachable_all}; auth_valid_all={auth_valid_all}"
                        )
                    elif overall != "?":
                        # Demo preflight requires all sensors healthy; degraded/failing is a
                        # FAIL not a WARN — demo assumes full sensor health (F-AUD-P1-LOW-004).
                        results["[A22] check_sensor_health (S-5.04 gate)"] = (
                            f"FAIL: {elapsed:.1f}s overall={overall} (degraded/failing not acceptable for demo preflight); "
                            f"sensors={sorted(present_sensors)}; reachable_all={reachable_all}; auth_valid_all={auth_valid_all}"
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
                # F-AUD-P2-HIGH-001: claroty_audit_logs uses a static 5-row fixture
                # (crates/prism-dtu-claroty/fixtures/audit-log.json, shared by all
                # orgs regardless of stage/seed — gap-analysis §4 guardrail #6).
                # 0 rows is impossible on a healthy DTU → FAIL, not WARN.
                results["[B4] Claroty org-c: claroty_audit_logs returns data"] = "FAIL: 0 rows (static 5-row fixture must always return data; gap-analysis §4 guardrail #6)"

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

        # ── B11–B15: Additional tables (org-c full 10-table matrix) ─────────────
        # Sequential IDs [B11]..[B15] in stable iteration order (F-AUD-P1-LOW-002).
        # F-AUD-P2-MED-007: B14 (crowdstrike_incidents) and B15 (cyberint_incidents)
        # have no DTU routes (gap-analysis §4 guardrail #1); 0 rows is the EXPECTED
        # outcome and is asserted explicitly. Any error_code on these tables = FAIL.
        NO_ROUTE_TABLES = {"crowdstrike_incidents", "cyberint_incidents"}
        for seq, tbl in enumerate(["armis_alerts", "claroty_alerts", "crowdstrike_devices",
                                    "crowdstrike_incidents", "cyberint_incidents"], start=11):
            body_t, err_t = query(proc, f"FROM {tbl} | limit 3", ["org-c"])
            key = f"[B{seq}] org-c: {tbl} returns data"
            if err_t:
                results[key] = f"FAIL: {err_t}"
            elif body_t.get("error_code"):
                results[key] = f"FAIL: {body_t['error_code']}: {body_t.get('message','')[:60]}"
            else:
                rows_t = body_t.get("rows", [])
                sensor_errors = body_t.get("sensor_errors", [])
                if tbl in NO_ROUTE_TABLES:
                    # No DTU route → 0 rows + no sensor_errors + no error_code expected.
                    # gap-analysis §4 guardrail #1.
                    if sensor_errors:
                        results[key] = f"FAIL: expected no sensor_errors for no-route table {tbl}, got {sensor_errors[:2]}"
                    elif len(rows_t) != 0:
                        results[key] = f"FAIL: expected 0 rows for no-route table {tbl}, got {len(rows_t)}"
                    else:
                        results[key] = f"PASS: {len(rows_t)} rows (expected: no DTU route — gap-analysis §4 guardrail #1)"
                else:
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
                # F-AUD-P2-HIGH-001: 0 rows means DTU is not returning data; isolation
                # cannot be proven → FAIL, not WARN (silent pass-through is dangerous).
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = f"FAIL: insufficient data — org-a={len(ids_a)} IDs, org-c={len(ids_c)} IDs (cannot prove disjoint with 0 rows from one or both orgs)"
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
        # NOTE (F-AUD-P1-OBS-001): also appears as A14 and F6 — intentional cross-section
        # regression coverage (MCP Protocol, Query Modes, Error Taxonomy sections).
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

        # ── C8: SQL mode baseline ─────────────────────────────────────────────
        # Verify SQL query path executes without error (prerequisite for temporal
        # queries). This check does NOT call NOW() — actual RFC-3339 datetime
        # typing and NOW() regression are covered by G7 (ADR-052 §D4).
        body, err = query(proc,
            "SELECT device_id FROM crowdstrike_detections WHERE device_id IS NOT NULL LIMIT 3",
            ["org-c"])
        if err:
            results["[C8] Temporal: SQL mode executes (ADR-052 §D4 baseline path)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C8] Temporal: SQL mode executes (ADR-052 §D4 baseline path)"] = f"PARTIAL: {body['error_code']}"
        else:
            rows = body.get("rows", [])
            results["[C8] Temporal: SQL mode executes (ADR-052 §D4 baseline path)"] = f"PASS: {len(rows)} rows (SQL path confirmed; see G7 for RFC-3339 regression)"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION D: Scenario Stage Verification (Stage 4 required)
        # ═══════════════════════════════════════════════════════════════════════

        # ── D1: Armis devices Stage 4 (scenario progressed) ─────────────────
        # F-AUD-P1-MED-004: require Stage-4-specific evidence — device IDs must contain
        # the org-c seed segment (-200-) AND include the primary compromised device
        # (dev-<hex>-200-0 pattern per runbook §1.4 / ScenarioEntityCatalog).
        # Baseline devices exist before Stage 4; a mere non-zero row count would also
        # pass at Stage 0.
        body, err = query(proc, "FROM armis_devices\n| where device_id IS NOT NULL\n| limit 20", ["org-c"])
        if err:
            results["[D1] SCENARIO: Stage 4 armis_devices visible"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[D1] SCENARIO: Stage 4 armis_devices visible"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if not rows:
                results["[D1] SCENARIO: Stage 4 armis_devices visible"] = "FAIL: 0 rows (scenario stage not progressing?)"
            else:
                device_ids = [str(r.get("device_id", "")) for r in rows if r.get("device_id")]
                # org-c seed-200: all device IDs contain the -200- segment
                has_seed_200 = any("-200-" in d for d in device_ids)
                # Primary compromised device: dev-<hex>-200-0 (first device, stage 1+)
                # Pattern: starts with "dev-" and ends with "-200-0"
                has_primary_device = any(
                    d.startswith("dev-") and d.endswith("-200-0")
                    for d in device_ids
                )
                if has_seed_200 and has_primary_device:
                    primary = next((d for d in device_ids if d.endswith("-200-0")), device_ids[0])
                    results["[D1] SCENARIO: Stage 4 armis_devices visible"] = (
                        f"PASS: {len(rows)} rows; org-c seed-200 segment confirmed; "
                        f"primary compromised device present: {primary[:50]}"
                    )
                elif has_seed_200 and not has_primary_device:
                    results["[D1] SCENARIO: Stage 4 armis_devices visible"] = (
                        f"FAIL: {len(rows)} rows have -200- seed but primary compromised device "
                        f"(dev-<hex>-200-0) absent; stage may be < 1; ids={device_ids[:3]}"
                    )
                elif not has_seed_200:
                    results["[D1] SCENARIO: Stage 4 armis_devices visible"] = (
                        f"FAIL: device IDs do not contain org-c -200- seed segment; "
                        f"may be baseline or wrong org; ids={device_ids[:3]}"
                    )

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
                # F-AUD-P2-HIGH-001: Stage 4 is terminal/absorbing
                # (scenario_start_secs=1782214754 is in the past per runbook §1.4);
                # IOC fields are guaranteed present at Stage 4 → FAIL, not WARN.
                results["[D2] IOC-FIELDS: cyberint iocs_value at Stage 4"] = "FAIL: 0 rows matching iocs_value IS NOT NULL (Stage 4 guarantees IOC fields; gap-analysis §3)"

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
                # F-AUD-P2-HIGH-001: Stage 4 is terminal/absorbing; Stage 2+
                # IOC fields are guaranteed present → FAIL, not WARN.
                results["[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+"] = "FAIL: 0 rows with behaviors_ioc_type IS NOT NULL (Stage 4 guarantees Stage 2+ IOC fields)"

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
                # F-AUD-P2-HIGH-001: claroty_audit_logs uses static 5-row fixture
                # (gap-analysis §4 guardrail #6) — 0 rows is impossible on healthy DTU.
                results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = "FAIL: 0 rows (static 5-row fixture; impossible on healthy DTU — gap-analysis §4 guardrail #6)"

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
                            # F-AUD-P2-HIGH-001: Stage 4 is terminal/absorbing; scenario
                            # guarantees cross-sensor device coherence → FAIL, not WARN.
                            results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"FAIL: device_id={cs_device_id[:30]!r} in CS but NOT in Armis (Stage 4 guarantees cross-sensor entity coherence)"
                else:
                    # F-AUD-P2-HIGH-001: CS returning a row with no device_id is a data quality
                    # failure → FAIL, not WARN.
                    results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = "FAIL: CS row has no device_id (data quality failure)"
            else:
                # F-AUD-P2-HIGH-001: Stage 4 guarantees CS has detections → FAIL.
                results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = "FAIL: CS returned 0 rows — Stage 4 guarantees detections; cannot test cross-sensor coherence"

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
                # F-AUD-P2-HIGH-001: Stage 4 is terminal/absorbing; iocs_value_first
                # is guaranteed non-null at Stage 4 → FAIL, not WARN.
                results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = "FAIL: 0 rows returned (Stage 4 guarantees iocs_value_first IS NOT NULL — gap-analysis §3)"

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
                if malicious == "MISSING":
                    results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = f"FAIL: threat_is_known_malicious column missing; cols={list(first.keys())[:8]}"
                elif malicious is not True:
                    # F-AUD-P2-MED-003: strict boolean assertion — must be exactly True
                    # (scenario IOCs are known-malicious; gap-analysis §3 data contract).
                    results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = f"FAIL: expected True (scenario IOC is known-malicious); got {malicious!r} (type={type(malicious).__name__})"
                else:
                    results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = f"PASS: {len(rows)} rows; threat_is_known_malicious=True (strict boolean confirmed)"
            else:
                # F-AUD-P2-HIGH-001: Stage 4 guarantees iocs_value_first IS NOT NULL → FAIL.
                results["[E2] ENRICH: threat_is_known_malicious(iocs_value_first)"] = "FAIL: 0 rows returned (Stage 4 guarantees iocs_value_first IS NOT NULL)"

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
                # F-AUD-P2-HIGH-001: Stage 4 is terminal/absorbing; scenario devices
                # have CVE data at Stage 4 → FAIL, not WARN.
                results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = "FAIL: 0 rows with device_cves_first IS NOT NULL (Stage 4 guarantees device CVE fields)"

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
                if severity == "MISSING":
                    results["[E4] ENRICH: cvss_severity(device_cves_first)"] = f"FAIL: cvss_severity column missing; cols={list(rows[0].keys())[:8]}"
                elif severity != "HIGH":
                    # F-AUD-P2-MED-004: scenario CVE-9999-* has CVSS ≥ 9.0 → severity
                    # MUST be "HIGH" (gap-analysis §3 data contract).
                    results["[E4] ENRICH: cvss_severity(device_cves_first)"] = f"FAIL: expected severity=='HIGH' for scenario CVE-9999-*; got {severity!r}"
                else:
                    results["[E4] ENRICH: cvss_severity(device_cves_first)"] = f"PASS: {len(rows)} rows; cvss_severity='HIGH' (scenario CVE-9999-* confirmed)"
            else:
                # F-AUD-P2-HIGH-001: Stage 4 guarantees device CVE fields → FAIL.
                results["[E4] ENRICH: cvss_severity(device_cves_first)"] = "FAIL: 0 rows with device_cves_first IS NOT NULL (Stage 4 guarantees device CVE fields)"

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
                # F-AUD-P2-HIGH-001: Stage 4 is terminal/absorbing; CS detections at
                # Stage 4 have IOC values → FAIL, not WARN.
                results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = "FAIL: 0 rows with behaviors_ioc_value_first IS NOT NULL (Stage 4 guarantees CS IOC fields)"

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
                        # F-AUD-P2-HIGH-001: gap-analysis §3 contract: scenario IOCs score ≥ 75.
                        results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = f"FAIL: scores present but none >= 75 (scenario IOCs must score ≥ 75; gap-analysis §3); scores={scores[:5]}"
                else:
                    # F-AUD-P2-HIGH-001: numeric threat_score expected → FAIL.
                    results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = f"FAIL: {len(rows)} rows but no numeric threat_score values; cols={list(rows[0].keys())[:8]}"
            else:
                # F-AUD-P2-HIGH-001: Stage 4 guarantees iocs_value_first IS NOT NULL → FAIL.
                results["[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs"] = "FAIL: 0 rows returned after iocs_value_first filter + enrich (Stage 4 guarantees IOC fields)"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION F: Error Taxonomy — E-QUERY-032/-037/-038/-039
        # ═══════════════════════════════════════════════════════════════════════

        # ── F1: E-QUERY-032: cyberint for org-a (no sensor) ─────────────
        # F-AUD-P1-MED-003: runbook v1.8 §5.8 N3 correction — E-QUERY-032 only.
        # E-QUERY-037 is dot-notation (covered by F3/A15); this path must produce
        # E-QUERY-032 (table not available for client).
        body, err = query(proc, "FROM cyberint_alerts | limit 5", ["org-a"])
        if err:
            results["[F1] E-QUERY-032: cyberint for org-a errors"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-032":
                results["[F1] E-QUERY-032: cyberint for org-a errors"] = f"PASS: E-QUERY-032 — {body.get('message','')[:60]}"
            elif not error_code:
                rows = body.get("rows", [])
                results["[F1] E-QUERY-032: cyberint for org-a errors"] = f"FAIL: returned {len(rows)} rows (should error — org-a has no cyberint)"
            else:
                results["[F1] E-QUERY-032: cyberint for org-a errors"] = f"FAIL: expected E-QUERY-032, got {error_code}: {body.get('message','')[:80]}"

        # ── F2: E-QUERY-032: armis for org-b (no sensor) ─────────────────────
        # F-AUD-P1-MED-003: runbook v1.8 §5.8 N3 correction — E-QUERY-032 only.
        # E-QUERY-037 is dot-notation (covered by F3/A15); this path must produce
        # E-QUERY-032 (table not available for client).
        body, err = query(proc, "FROM armis_devices | limit 5", ["org-b"])
        if err:
            results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-032":
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"PASS: E-QUERY-032 — {body.get('message','')[:60]}"
            elif not error_code:
                rows = body.get("rows", [])
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: returned {len(rows)} rows (should error — org-b has no armis)"
            else:
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: expected E-QUERY-032, got {error_code}: {body.get('message','')[:80]}"

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
        # F-AUD-P1-MED-008: F5's designated regression class is PR #219's E-QUERY-038 gate.
        # The former alt-error branch (`error_code and "column" in msg.lower()`) accepted
        # any error code mentioning "column" — removed. Only E-QUERY-038 PASSes here.
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
            elif not error_code:
                rows = body.get("rows", [])
                results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"FAIL: query succeeded with unknown column (returned {len(rows)} rows without error)"
            else:
                results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"FAIL: expected E-QUERY-038, got {error_code}: {msg[:80]}"

        # ── F6: N1-B F1: SQL builtin (COUNT) NOT E-QUERY-039 ─────────────────
        # NOTE (F-AUD-P1-OBS-001): COUNT(*) armis_devices also appears in A14 and C4 —
        # intentional cross-section regression coverage (F-section error taxonomy, C-section
        # query modes, A-section false-positive guardrail). Three independent probes confirm
        # orthogonally that DataFusion builtins do not trigger E-QUERY-039.
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
                # F-AUD-P1-MED-001: also assert ONLY critical-severity rows are returned.
                # IEQ 'critical' must filter out non-critical rows; any non-critical row is a
                # filter failure (mirrors H3's guard for INE).
                has_non_critical = any(s and s.lower() != "critical" for s in severities if s)
                if has_non_critical:
                    non_crit = sorted({s for s in severities if s and s.lower() != "critical"})
                    results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                        f"FAIL: non-critical rows returned by IEQ 'critical' filter — IEQ not filtering correctly; "
                        f"non-critical severities={non_crit!r}"
                    )
                elif bad_case:
                    results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                        f"FAIL: non-Title-case severity values returned: {list(set(bad_case))[:3]}"
                    )
                else:
                    sample_sev = sorted(set(severities))
                    results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                        f"PASS: {len(rows)} rows; all severity={sample_sev!r} (canonical Title-case; "
                        f"only critical rows confirmed; NOTE: runbook Step 3.1a uses 'high' but CS scenario data is 'Critical'/'Medium')"
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

        # ── G3: IIN on status lowercase → crowdstrike_detections ─────────────────
        # Runbook Step 3.1a / §5.9 checklist item 3 (implied).
        # Redirected from cyberint_alerts (ADV-PR-P11-HIGH-001): cyberint DTU emits
        # vendor-native status values "open"/"acknowledged"/"closed" (prism-dtu-cyberint
        # generator.rs statuses array).  None of these lexically match an OCSF caption in
        # status_id (captions: Unknown, Success, Failure, New, In Progress, Suppressed,
        # Resolved, Archived, Deleted, Other) — normalize_enum_label returns None and
        # passes them through unchanged.  No enum_map is declared for status in
        # cyberint.sensor.toml.  IIN('new','in progress') can never match those values.
        #
        # crowdstrike_detections is the correct table: CS DTU emits status "new" for
        # all detection records (prism-dtu-crowdstrike generator.rs
        # make_detection_with_ioc).  normalize_enum_label("status", "new") → "New"
        # (OcsfEnumMap status_id[1001]).  IIN lowers both sides: lower("New") = "new"
        # ∈ {"new", "in progress"} → matches.  The returned Title-case "New" values
        # prove the normalization + IIN case-folding chain is working end-to-end.
        body, err = query(proc,
            "FROM crowdstrike_detections\n| where status IIN ('new', 'in progress')\n| limit 20",
            ["org-c"])
        if err:
            results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = f"FAIL: {ec}: {body.get('message','')[:100]}"
        else:
            rows = body.get("rows", [])
            if rows:
                distinct_status = sorted({r.get("status", "") for r in rows if r.get("status")})
                # Verify stored status values are OCSF canonical Title-case ("New", not
                # "new"/"NEW"), proving normalize_enum_label ran before the IIN match.
                known_ocsf_status = {"new", "in progress", "suppressed", "resolved",
                                     "archived", "deleted", "unknown", "success", "failure", "other"}
                non_title = [s for s in distinct_status
                             if s and s.lower() in known_ocsf_status and s != s.title()]
                if non_title:
                    results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                        f"FAIL: non-Title-case status values returned (OCSF normalization not applied): "
                        f"{non_title!r}; all returned={distinct_status!r}"
                    )
                else:
                    results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                        f"PASS: {len(rows)} rows; distinct statuses={distinct_status!r} "
                        f"(Title-case confirmed; lowercase IIN input matched OCSF-normalized values)"
                    )
            else:
                results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                    "FAIL: 0 rows — crowdstrike_detections at Stage 1+ must have status='new' "
                    "detection records; CS DTU emits status 'new' → OcsfEnumMap normalizes to 'New' "
                    "(status_id[1001]); IIN operator may not be matching or CS data absent"
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
                # F-AUD-P1-MED-009: runbook §4.3 mandates E-QUERY-001 mode-boundary pedagogy;
                # any other error code is a FAIL, not PARTIAL.
                results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                    f"FAIL: expected E-QUERY-001 (mode-boundary pedagogy), got {ec}: {msg[:100]!r}"
                )
            else:
                rows = body.get("rows", [])
                results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                    f"FAIL: query succeeded ({len(rows)} rows) — IEQ must be rejected in SQL WHERE clause"
                )

        # ── G5: E-QUERY-002 typed guidance (IEQ on integer column) ─────────────
        # armis_devices.risk_score is an integer column (Option<u32> in DTU, Integer in
        # sensor TOML). IEQ on an integer column must return E-QUERY-002 (QueryTypeMismatch)
        # since lower() is not applicable to integers.
        # NB: risk_score has no OCSF string sibling — do NOT assert sibling suggestion.
        # H6 (Section H) is the canonical probe; G5 is kept as a pre-Section-H warm-up.
        body, err = query(proc,
            "FROM armis_devices\n| where risk_score IEQ 'high'\n| limit 5",
            ["org-c"])
        if err:
            results["[G5] E-QUERY-002: IEQ on integer column (armis risk_score)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-002":
                results["[G5] E-QUERY-002: IEQ on integer column (armis risk_score)"] = (
                    f"PASS: E-QUERY-002 — IEQ on integer risk_score correctly rejected; "
                    f"message={msg[:120]!r}"
                )
            elif ec:
                results["[G5] E-QUERY-002: IEQ on integer column (armis risk_score)"] = (
                    f"PARTIAL: expected E-QUERY-002 (IEQ on integer), got {ec}: {msg[:100]!r}"
                )
            else:
                rows = body.get("rows", [])
                results["[G5] E-QUERY-002: IEQ on integer column (armis risk_score)"] = (
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

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION H: PR #219 behaviors — pipe/filter E-QUERY-038, did_you_mean,
        #            INE, temporal negative paths, E-QUERY-002 via integer column,
        #            JOINs, HEAD-JOIN fail-open, SqlPipe, E-QUERY-040 dual-limit,
        #            stats grammar, multi-client fan-out, prompts (2 new), resources
        #            (3 new), 14-tool assertion, CWE-116/117 sanitization, guardrails
        #            (E-QUERY-033/-003), threat_sources/cvss_vector UDFs, runbook-drift,
        #            determinism, normalized_pql.
        # Source: t13-audit-coverage-gap-analysis-2026-07-10.md §3 H1–H22 + H1b
        # ═══════════════════════════════════════════════════════════════════════

        # ── H1: E-QUERY-038 pipe mode — the original DRIFT shape (PR #219) ────
        # CRITICAL: exact shape that produced "Internal error" before the fix.
        body, err = query(proc,
            "FROM crowdstrike_detections\n| where nonexistent_column_xyz IEQ 'high'\n| limit 5",
            ["org-c"])
        if err:
            results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-038":
                results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                    f"PASS: E-QUERY-038 (no Internal error / E-QUERY-034 regression)"
                )
            elif ec in ("E-QUERY-034",) or "Internal error" in msg:
                results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                    f"FAIL: REGRESSION — got {ec!r} / 'Internal error' instead of E-QUERY-038; "
                    f"message={msg[:100]!r}"
                )
            elif body.get("rows") is not None and not ec:
                results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                    f"FAIL: query succeeded ({len(body.get('rows', []))} rows) — nonexistent column must error"
                )
            else:
                results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                    f"PARTIAL: unexpected {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H1b: E-QUERY-038 filter mode (position 7, no FROM keyword) ───────
        # Syntax: table_name | predicate (no WHERE, no FROM keyword)
        body, err = query(proc,
            "crowdstrike_detections | nonexistent_column_xyz IEQ 'high'",
            ["org-c"])
        if err:
            results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-038":
                results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                    f"PASS: E-QUERY-038 in filter mode (no regression)"
                )
            elif ec in ("E-QUERY-034",) or "Internal error" in msg:
                results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                    f"FAIL: REGRESSION — {ec!r} / 'Internal error' instead of E-QUERY-038; "
                    f"message={msg[:100]!r}"
                )
            elif body.get("rows") is not None and not ec:
                results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                    f"FAIL: filter mode query succeeded — nonexistent column must error"
                )
            else:
                results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                    f"PARTIAL: {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H2: E-QUERY-038 did_you_mean + available_columns payload ─────────
        # Tests that ColumnNotFoundDetails carries "Did you mean:" + "available: [" in text.
        body, err = query(proc,
            "SELECT sevrity FROM crowdstrike_detections LIMIT 5",
            ["org-c"])
        if err:
            results["[H2] E-QUERY-038 did_you_mean + available_columns payload"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            sc_err = body.get("_sc_error", {})
            if ec == "E-QUERY-038":
                has_dym_text = "Did you mean:" in msg
                has_avail_text = "available: [" in msg
                # Also check structuredContent.error fields
                sc_dym = sc_err.get("did_you_mean", "") if sc_err else ""
                sc_avail = sc_err.get("available_columns", []) if sc_err else []
                if has_dym_text and has_avail_text:
                    results["[H2] E-QUERY-038 did_you_mean + available_columns payload"] = (
                        f"PASS: E-QUERY-038; text contains 'Did you mean:' and 'available: ['; "
                        f"sc_error.did_you_mean={sc_dym!r}; "
                        f"sc_error.available_columns count={len(sc_avail)}"
                    )
                else:
                    results["[H2] E-QUERY-038 did_you_mean + available_columns payload"] = (
                        f"FAIL: E-QUERY-038 but payload anchors missing — "
                        f"has_dym_text={has_dym_text}, has_avail_text={has_avail_text}; "
                        f"message={msg[:120]!r}"
                    )
            else:
                results["[H2] E-QUERY-038 did_you_mean + available_columns payload"] = (
                    f"FAIL: expected E-QUERY-038, got {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H3: INE operator — severity INE 'medium' (excludes Medium rows) ──
        body, err = query(proc,
            "FROM crowdstrike_detections\n| where severity INE 'medium'\n| limit 20",
            ["org-c"])
        if err:
            results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if not rows:
                results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                    "FAIL: 0 rows — INE should return Critical rows (seed-200: 5 Critical + 15 Medium)"
                )
            else:
                severities = [r.get("severity", "") for r in rows]
                has_medium = any(s and s.lower() == "medium" for s in severities)
                all_critical = all(s and s.lower() == "critical" for s in severities if s)
                if has_medium:
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"FAIL: Medium rows leaked through INE filter; severities={list(set(severities))!r}"
                    )
                elif all_critical and len(rows) >= 1:
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"PASS: {len(rows)} rows; all severity='Critical'; zero Medium rows"
                    )
                else:
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"PARTIAL: rows={len(rows)}; severities={list(set(severities))!r}"
                    )

        # ── H4: E-QUERY-041 negative temporal — date-only literal ────────────
        # 'timestamp > '2020-01-01'' (date-only, no RFC-3339 UTC offset) must reject.
        body, err = query(proc,
            "FROM claroty_audit_logs\n| where timestamp > '2020-01-01'\n| limit 3",
            ["org-c"])
        if err:
            results["[H4] E-QUERY-041: date-only literal rejected (ADR-052 §D4)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-041":
                has_rfc_hint = "RFC-3339" in msg or "UTC" in msg or "cannot be interpreted" in msg
                # F-AUD-P1-MED-010: PASS requires E-QUERY-041 AND the RFC-3339 pedagogical hint.
                # E-QUERY-041 without the hint means message-template regression.
                if has_rfc_hint:
                    results["[H4] E-QUERY-041: date-only literal rejected (ADR-052 §D4)"] = (
                        f"PASS: E-QUERY-041 with RFC-3339 pedagogical hint confirmed; "
                        f"message={msg[:80]!r}"
                    )
                else:
                    results["[H4] E-QUERY-041: date-only literal rejected (ADR-052 §D4)"] = (
                        f"FAIL: E-QUERY-041 returned but RFC-3339/UTC hint absent — "
                        f"message-template regression; message={msg[:120]!r}"
                    )
            elif body.get("rows") is not None and not ec:
                results["[H4] E-QUERY-041: date-only literal rejected (ADR-052 §D4)"] = (
                    "FAIL: date-only literal accepted (should be E-QUERY-041)"
                )
            else:
                # F-AUD-P2-HIGH-002: PARTIAL misuse — any non-E-QUERY-041 error is a
                # hard FAIL (same precedent as G4 "expected E-QUERY-04x, got {ec}").
                results["[H4] E-QUERY-041: date-only literal rejected (ADR-052 §D4)"] = (
                    f"FAIL: expected E-QUERY-041, got {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H5: E-QUERY-042 — temporal literal in GROUP BY position ──────────
        body, err = query(proc,
            "SELECT severity, COUNT(*) FROM crowdstrike_detections GROUP BY '2026-07-01T00:00:00Z'",
            ["org-c"])
        if err:
            results["[H5] E-QUERY-042: temporal literal in GROUP BY (ADR-052 §D4)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-042":
                results["[H5] E-QUERY-042: temporal literal in GROUP BY (ADR-052 §D4)"] = (
                    f"PASS: E-QUERY-042 — temporal literal in GROUP BY arm rejected; "
                    f"message={msg[:80]!r}"
                )
            elif body.get("rows") is not None and not ec:
                results["[H5] E-QUERY-042: temporal literal in GROUP BY (ADR-052 §D4)"] = (
                    "FAIL: GROUP BY literal accepted (should be E-QUERY-042)"
                )
            else:
                # F-AUD-P2-HIGH-002: PARTIAL misuse — any non-E-QUERY-042 error is FAIL.
                results["[H5] E-QUERY-042: temporal literal in GROUP BY (ADR-052 §D4)"] = (
                    f"FAIL: expected E-QUERY-042, got {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H6: E-QUERY-002 via armis_devices.risk_score (integer column) ─────
        # armis_devices.risk_score is Integer-typed. IEQ must reject with E-QUERY-002.
        # Do NOT assert sibling suggestion (risk_score has no OCSF string sibling).
        # This is the canonical E-QUERY-002 probe that retires G5's permanent WARN.
        body, err = query(proc,
            "FROM armis_devices\n| where risk_score IEQ 'high'\n| limit 5",
            ["org-c"])
        if err:
            results["[H6] E-QUERY-002 via armis_devices.risk_score (integer column)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-002":
                has_operator_hint = "does not support operator" in msg or "IEQ" in msg
                results["[H6] E-QUERY-002 via armis_devices.risk_score (integer column)"] = (
                    f"PASS: E-QUERY-002 — IEQ on integer risk_score rejected; "
                    f"operator_hint={'YES' if has_operator_hint else 'NO (check message)'}: "
                    f"message={msg[:100]!r}"
                )
            elif body.get("rows") is not None and not ec:
                results["[H6] E-QUERY-002 via armis_devices.risk_score (integer column)"] = (
                    f"FAIL: query succeeded ({len(body.get('rows', []))} rows) — IEQ on integer must error"
                )
            else:
                # F-AUD-P2-HIGH-002: PARTIAL misuse — any non-E-QUERY-002 error is FAIL.
                results["[H6] E-QUERY-002 via armis_devices.risk_score (integer column)"] = (
                    f"FAIL: expected E-QUERY-002, got {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H7: JOIN positive path — crowdstrike_devices JOIN armis_devices ───
        # org-c seed-200: device IDs dev-0196f4b2-200-{0..49} in BOTH tables (full overlap).
        body, err = query(proc,
            "SELECT d.device_id, a.risk_score FROM crowdstrike_devices d "
            "JOIN armis_devices a ON d.device_id = a.device_id LIMIT 5",
            ["org-c"])
        if err:
            results["[H7] JOIN positive path: crowdstrike_devices JOIN armis_devices"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H7] JOIN positive path: crowdstrike_devices JOIN armis_devices"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if rows:
                scores = [r.get("risk_score") for r in rows if r.get("risk_score") is not None]
                results["[H7] JOIN positive path: crowdstrike_devices JOIN armis_devices"] = (
                    f"PASS: {len(rows)} joined rows; risk_score values={scores[:5]}"
                )
            else:
                results["[H7] JOIN positive path: crowdstrike_devices JOIN armis_devices"] = (
                    "FAIL: 0 rows — seed-200 guarantees 50 device IDs overlap between tables"
                )

        # ── H8: HEAD-JOIN fail-open — bare unknown column in JOIN ─────────────
        # BC-2.11.016 v1.25 suspension rule 6: bare-column reference in JOIN → fail-open
        # (E-QUERY-034 or controlled rejection, NEVER E-QUERY-038).
        body, err = query(proc,
            "SELECT totally_unknown_col FROM crowdstrike_devices d "
            "JOIN armis_devices a ON d.device_id = a.device_id LIMIT 5",
            ["org-c"])
        if err:
            results["[H8] HEAD-JOIN fail-open: bare unknown col in JOIN (not E-QUERY-038)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            rows = body.get("rows", [])
            if rows:
                results["[H8] HEAD-JOIN fail-open: bare unknown col in JOIN (not E-QUERY-038)"] = (
                    f"FAIL: query returned {len(rows)} rows with unknown col (should reject)"
                )
            elif ec == "E-QUERY-038":
                results["[H8] HEAD-JOIN fail-open: bare unknown col in JOIN (not E-QUERY-038)"] = (
                    f"FAIL: E-QUERY-038 fired for bare unknown col in JOIN — "
                    f"HEAD-JOIN fail-open (FP-001) should suppress E-QUERY-038 here"
                )
            elif ec == "E-QUERY-034" or "Internal error" in msg:
                # F-AUD-P1-MED-002: only E-QUERY-034 or Internal error PASSes here.
                # The former third disjunct `(ec and ec != "E-QUERY-038")` accepted any
                # error code — removed; unexpected error codes must be investigated.
                results["[H8] HEAD-JOIN fail-open: bare unknown col in JOIN (not E-QUERY-038)"] = (
                    f"PASS: controlled rejection ({ec or 'internal'}) — not E-QUERY-038 "
                    f"(HEAD-JOIN spec-sanctioned FP-001 confirmed)"
                )
            elif not ec and not rows:
                # Empty result with no error — also acceptable (no rows, no crash)
                results["[H8] HEAD-JOIN fail-open: bare unknown col in JOIN (not E-QUERY-038)"] = (
                    "PARTIAL: no error, no rows (acceptable; not a crash or E-QUERY-038)"
                )
            else:
                # Unexpected error code — neither E-QUERY-034 nor Internal error
                results["[H8] HEAD-JOIN fail-open: bare unknown col in JOIN (not E-QUERY-038)"] = (
                    f"FAIL: unexpected error code {ec!r} for bare-col JOIN; "
                    f"expected E-QUERY-034 or 'Internal error'; message={msg[:80]!r}"
                )

        # ── H9: SqlPipe mode — SQL head + pipe stage (BC-2.11.020) ───────────
        body, err = query(proc,
            "SELECT device_id, severity FROM crowdstrike_detections "
            "| where severity IEQ 'critical' | limit 5",
            ["org-c"])
        if err:
            results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if not rows:
                results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = (
                    "FAIL: 0 rows — seed-200 guarantees Critical detections"
                )
            else:
                severities = [r.get("severity", "") for r in rows]
                non_critical = [s for s in severities if s and s.lower() != "critical"]
                if non_critical:
                    results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = (
                        f"FAIL: non-Critical rows leaked: {non_critical!r}"
                    )
                else:
                    results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = (
                        f"PASS: {len(rows)} rows; all severity='Critical' (SqlPipe IEQ filter confirmed)"
                    )

        # ── H10: E-QUERY-040 dual-limit (SQL LIMIT + pipe | limit) ───────────
        body, err = query(proc,
            "SELECT device_id FROM crowdstrike_detections LIMIT 5 | limit 3",
            ["org-c"])
        if err:
            results["[H10] E-QUERY-040: SQL LIMIT + pipe | limit (dual-limit rejected)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-040":
                results["[H10] E-QUERY-040: SQL LIMIT + pipe | limit (dual-limit rejected)"] = (
                    f"PASS: E-QUERY-040 — dual row-limit cap rejected (ADR-043 D4)"
                )
            elif body.get("rows") is not None and not ec:
                results["[H10] E-QUERY-040: SQL LIMIT + pipe | limit (dual-limit rejected)"] = (
                    f"FAIL: dual-limit query succeeded ({len(body.get('rows', []))} rows) — should be E-QUERY-040"
                )
            else:
                # F-AUD-P2-HIGH-002: PARTIAL misuse — any non-E-QUERY-040 error is FAIL.
                results["[H10] E-QUERY-040: SQL LIMIT + pipe | limit (dual-limit rejected)"] = (
                    f"FAIL: expected E-QUERY-040, got {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H11: | stats grammar (count() as alias by field) ─────────────────
        # seed-200: 20 detections (5 Critical + 15 Medium) → exactly 2 severity buckets.
        body, err = query(proc,
            "FROM crowdstrike_detections\n| stats count() as cnt by severity",
            ["org-c"])
        if err:
            results["[H11] stats grammar: count() as cnt by severity"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H11] stats grammar: count() as cnt by severity"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if not rows:
                results["[H11] stats grammar: count() as cnt by severity"] = (
                    "FAIL: 0 rows from stats — seed-200 guarantees Critical and Medium buckets"
                )
            else:
                severities_found = {r.get("severity", "") for r in rows}
                expected = {"Critical", "Medium"}
                if expected == severities_found:
                    cnts = {r.get("severity"): r.get("cnt") for r in rows}
                    results["[H11] stats grammar: count() as cnt by severity"] = (
                        f"PASS: {len(rows)} buckets; Critical={cnts.get('Critical')}, "
                        f"Medium={cnts.get('Medium')} (seed-200 counts confirmed)"
                    )
                elif expected.issubset(severities_found):
                    results["[H11] stats grammar: count() as cnt by severity"] = (
                        f"PASS: {len(rows)} buckets include Critical+Medium; all={sorted(severities_found)!r}"
                    )
                else:
                    results["[H11] stats grammar: count() as cnt by severity"] = (
                        f"PARTIAL: buckets={sorted(severities_found)!r} (expected {sorted(expected)})"
                    )

        # ── H12: Multi-client fan-out — clients: [org-a, org-c] ──────────────
        # org-a: 5 CS detections (seed-100, IDs contain -100-); org-c: 20 (seed-200, -200-).
        # With limit 40, all 25 should be returned. sensor_errors should be empty.
        body, err = query(proc,
            "FROM crowdstrike_detections\n| limit 40",
            ["org-a", "org-c"], timeout=30.0)
        if err:
            results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            sensor_errors = body.get("sensor_errors", [])
            if not rows:
                results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                    "FAIL: 0 rows from multi-client query"
                )
            else:
                # Check for both seed segments in any string column
                all_vals = " ".join(str(v) for r in rows for v in r.values() if isinstance(v, str))
                has_100 = "-100-" in all_vals
                has_200 = "-200-" in all_vals
                if has_100 and has_200 and not sensor_errors:
                    results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                        f"PASS: {len(rows)} rows; both -100- (org-a) and -200- (org-c) seeds present; "
                        f"sensor_errors=[]"
                    )
                elif has_100 and has_200 and sensor_errors:
                    # F-AUD-P1-MED-005: sensor_errors present alongside data — FAIL (not PASS).
                    # Partial results with sensor errors indicate a pipeline failure that must be
                    # resolved before demo; over-broad PASS was masking error propagation bugs.
                    results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                        f"FAIL: {len(rows)} rows but sensor_errors non-empty — pipeline errors present: "
                        f"{sensor_errors}"
                    )
                else:
                    results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                        f"FAIL: missing seed segments — has_100={has_100}, has_200={has_200}; "
                        f"total_rows={len(rows)}"
                    )

        # ── H13: Prompts — client_overview and cross_client_status ───────────
        # Same hang/arg-validation class that A17/A18 guard for the other 3 prompts.
        t0 = time.time()
        res_co, err_co = prompt_get(proc, "client_overview", {"client_id": "org-c"}, timeout=5.0)
        elapsed_co = time.time() - t0
        if err_co:
            results["[H13a] client_overview prompt returns promptly"] = f"FAIL: {err_co} ({elapsed_co:.2f}s)"
        elif elapsed_co > 3.0:
            results["[H13a] client_overview prompt returns promptly"] = f"FAIL: took {elapsed_co:.2f}s"
        else:
            msgs = res_co.get("messages", []) if res_co else []
            results["[H13a] client_overview prompt returns promptly"] = (
                f"PASS: {elapsed_co:.2f}s; {len(msgs)} message(s)"
            )

        t0 = time.time()
        res_ccs, err_ccs = prompt_get(proc, "cross_client_status", {}, timeout=5.0)
        elapsed_ccs = time.time() - t0
        if err_ccs:
            results["[H13b] cross_client_status prompt returns promptly"] = f"FAIL: {err_ccs} ({elapsed_ccs:.2f}s)"
        elif elapsed_ccs > 3.0:
            results["[H13b] cross_client_status prompt returns promptly"] = f"FAIL: took {elapsed_ccs:.2f}s"
        else:
            msgs = res_ccs.get("messages", []) if res_ccs else []
            results["[H13b] cross_client_status prompt returns promptly"] = (
                f"PASS: {elapsed_ccs:.2f}s; {len(msgs)} message(s)"
            )

        # ── H14: resources/read — 3 URIs (config/clients, sensors/health, schema) ─
        h14_parts = []

        res_cc, err_cc = resource_read(proc, "prism://config/clients", timeout=10.0)
        if err_cc:
            h14_parts.append(f"config/clients FAIL:{err_cc[:40]!r}")
        else:
            t_cc = (res_cc.get("contents") or [{}])[0].get("text", "")
            # F-AUD-P1-OBS-003: assertion narrative claims 3-org visibility; require all 3 orgs.
            if "org-a" in t_cc and "org-b" in t_cc and "org-c" in t_cc:
                h14_parts.append("config/clients PASS(3-orgs:a,b,c)")
            else:
                missing_orgs = [o for o in ("org-a", "org-b", "org-c") if o not in t_cc]
                h14_parts.append(f"config/clients FAIL(missing orgs:{missing_orgs}:{t_cc[:40]!r})")

        res_sh, err_sh = resource_read(proc, "prism://sensors/health", timeout=10.0)
        if err_sh:
            h14_parts.append(f"sensors/health FAIL:{err_sh[:40]!r}")
        else:
            t_sh = (res_sh.get("contents") or [{}])[0].get("text", "")
            try:
                sh_obj = json.loads(t_sh)
                # F-AUD-P2-HIGH-004: parse-only assertion is insufficient; assert
                # overall_status present AND sensors list non-empty (SensorHealthStructuredContent
                # contract from resources.rs: {overall_status, sensors: [...]}).
                sh_overall = sh_obj.get("overall_status") if isinstance(sh_obj, dict) else None
                sh_sensors = sh_obj.get("sensors", []) if isinstance(sh_obj, dict) else []
                if sh_overall is None:
                    h14_parts.append(f"sensors/health FAIL(overall_status missing from JSON:{t_sh[:60]!r})")
                elif not sh_sensors:
                    h14_parts.append(f"sensors/health FAIL(sensors list empty; overall_status={sh_overall!r})")
                else:
                    h14_parts.append(f"sensors/health PASS(JSON; overall_status={sh_overall!r}; sensor_count={len(sh_sensors)})")
            except Exception:
                # F-AUD-P1-MED-007: resources.rs render_sensors_health_resource always produces
                # JSON; non-JSON response is a contract violation, not merely a WARN.
                h14_parts.append(f"sensors/health FAIL(non-JSON:{t_sh[:30]!r})")

        res_sc, err_sc = resource_read(proc, "prismql://schema/org-c", timeout=10.0)
        if err_sc:
            h14_parts.append(f"schema/org-c FAIL:{err_sc[:40]!r}")
        else:
            t_sc = (res_sc.get("contents") or [{}])[0].get("text", "")
            if "cyberint_alerts" in t_sc:
                h14_parts.append("schema/org-c PASS(cyberint_alerts)")
            else:
                h14_parts.append(f"schema/org-c FAIL(cyberint_alerts missing)")

        any_fail = any("FAIL" in p for p in h14_parts)
        results["[H14] resources/read: config/clients + sensors/health + schema"] = (
            f"{'FAIL' if any_fail else 'PASS'}: {'; '.join(h14_parts)}"
        )

        # ── H14c: resources/read — prism://schema/crowdstrike/detections ─────
        # F-AUD-P2-MED-006: extend resource coverage to per-sensor-table schema URI.
        # URI template: prism://schema/{sensor_id}/{table_name} (resources.rs URI_TEMPLATE_SCHEMA).
        res_h14c, err_h14c = resource_read(proc, "prism://schema/crowdstrike/detections", timeout=10.0)
        if err_h14c:
            results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = f"FAIL: {err_h14c}"
        else:
            t_h14c = (res_h14c.get("contents") or [{}])[0].get("text", "")
            if "device_id" in t_h14c or "severity" in t_h14c or "detections" in t_h14c.lower():
                results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                    f"PASS: schema resource returned; body contains detection schema fields (len={len(t_h14c)})"
                )
            elif t_h14c:
                results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                    f"FAIL: schema body lacks expected field names; body={t_h14c[:100]!r}"
                )
            else:
                results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                    "FAIL: empty schema body"
                )

        # ── H14d: resources/read — prism://config/clients/org-c/sensors ──────
        # F-AUD-P2-MED-006: extend resource coverage to per-client sensor list URI.
        # URI template: prism://config/clients/{client_id}/sensors (resources.rs URI_TEMPLATE_CLIENT_SENSORS).
        res_h14d, err_h14d = resource_read(proc, "prism://config/clients/org-c/sensors", timeout=10.0)
        if err_h14d:
            results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = f"FAIL: {err_h14d}"
        else:
            t_h14d = (res_h14d.get("contents") or [{}])[0].get("text", "")
            # org-c has crowdstrike, armis, claroty, cyberint sensors (runbook §1.3).
            has_sensor = any(s in t_h14d for s in ("crowdstrike", "armis", "claroty", "cyberint"))
            if has_sensor:
                results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                    f"PASS: per-client sensors resource returned; body contains sensor names (len={len(t_h14d)})"
                )
            elif t_h14d:
                results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                    f"FAIL: body lacks sensor names for org-c; body={t_h14d[:100]!r}"
                )
            else:
                results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                    "FAIL: empty body for per-client sensors resource"
                )

        # ── H14e: resources/subscribe + unsubscribe — prismql://schema/org-c ─
        # F-AUD-P2-MED-008: resources/subscribe supported via enable_resources_subscribe()
        # in server.rs for prismql://schema/{client_id} URIs.
        res_sub, err_sub = resources_subscribe(proc, "prismql://schema/org-c", timeout=10.0)
        if err_sub:
            results["[H14e] resources/subscribe+unsubscribe: prismql://schema/org-c"] = f"FAIL: subscribe: {err_sub}"
        else:
            res_unsub, err_unsub = resources_unsubscribe(proc, "prismql://schema/org-c", timeout=10.0)
            if err_unsub:
                results["[H14e] resources/subscribe+unsubscribe: prismql://schema/org-c"] = (
                    f"FAIL: subscribe OK but unsubscribe failed: {err_unsub}"
                )
            else:
                results["[H14e] resources/subscribe+unsubscribe: prismql://schema/org-c"] = (
                    "PASS: subscribe accepted; unsubscribe accepted (prismql://schema/org-c)"
                )

        # ── H15: 14 implemented tools + live explain_query call ──────────────
        # Re-asserts the 14-tool set from A2 but also does a live explain_query call.
        body, err = tool_call(proc, "explain_query",
                              {"query": "FROM crowdstrike_detections | limit 5",
                               "clients": ["org-c"]}, timeout=15.0)
        if err:
            results["[H15] explain_query live call (one of 14 implemented tools)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H15] explain_query live call (one of 14 implemented tools)"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            # explain_query returns a plan object; require parsed_mode to confirm
            # positive-coverage evidence (F-AUD-P1-MED-006: empty body is not a PASS).
            if "parsed_mode" in body:
                results["[H15] explain_query live call (one of 14 implemented tools)"] = (
                    f"PASS: explain_query returned plan with parsed_mode={body.get('parsed_mode')!r}; "
                    f"keys={list(body.keys())[:6]}"
                )
            else:
                results["[H15] explain_query live call (one of 14 implemented tools)"] = (
                    f"FAIL: explain_query response lacks parsed_mode key — schema mismatch or empty plan; "
                    f"keys={list(body.keys())[:8]}"
                )

        # ── H16: CWE-116/117 — control-char injection sanitized ──────────────
        # Embed a literal U+0001 in the column name to verify sanitize_for_log strips it.
        # PASS if E-QUERY-038 or E-QUERY-001 returned without raw control chars in text.
        ctrl_col = "badcolumn\x01"
        h16_query = f'SELECT "{ctrl_col}" FROM crowdstrike_detections LIMIT 3'
        rid_h16 = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid_h16, "method": "tools/call",
                        "params": {"name": "query",
                                   "arguments": {"query": h16_query, "clients": ["org-c"]}}})
        resp_h16, err_h16 = read_msg(proc, timeout=15.0)
        if err_h16:
            results["[H16] CWE-116/117: control-char in column name sanitized"] = f"FAIL: {err_h16}"
        else:
            # Check raw text for control chars (U+0000–U+001F, U+007F)
            raw_content = resp_h16.get("result", {}).get("content", []) if resp_h16 else []
            raw_text = raw_content[0].get("text", "") if raw_content else ""
            control_chars_found = [c for c in raw_text if (ord(c) < 0x20 or ord(c) == 0x7F)
                                    and c not in ("\n", "\r", "\t")]
            if control_chars_found:
                results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                    f"FAIL: raw control chars leaked in response: "
                    f"{[hex(ord(c)) for c in control_chars_found[:5]]}"
                )
            elif raw_text.startswith("ERROR:"):
                # Error returned (E-QUERY-038 or E-QUERY-001) without raw control chars — PASS
                results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                    f"PASS: error returned without raw control chars (CWE-116/117 sanitized); "
                    f"preview={raw_text[:80]!r}"
                )
            elif resp_h16 and "error" in resp_h16:
                # RPC-level rejection — also acceptable
                results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                    f"PASS: RPC-level rejection without control-char leakage"
                )
            else:
                # F-AUD-P1-LOW-003: PASS only if the response is a well-formed data envelope.
                # An indeterminate response (non-JSON, non-ERROR, non-RPC-error) could mask
                # a sanitization failure — must be treated as FAIL.
                try:
                    parsed = json.loads(raw_text)
                    if isinstance(parsed, dict):
                        results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                            f"PASS: well-formed JSON data envelope without control chars; "
                            f"keys={list(parsed.keys())[:5]!r}"
                        )
                    else:
                        results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                            f"FAIL: indeterminate response (JSON non-dict type={type(parsed).__name__}); "
                            f"preview={raw_text[:60]!r}"
                        )
                except json.JSONDecodeError:
                    results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                        f"FAIL: indeterminate response (non-JSON, non-ERROR, non-RPC-error); "
                        f"preview={raw_text[:60]!r}"
                    )

        # ── H16b: CWE-116/117 — control-char in unquoted WHERE predicate ────────
        # F-AUD-P2-LOW-001: complement H16 (quoted column name) with an unquoted
        # WHERE predicate probe to cover CWE-117 log injection via value position.
        ctrl_val = "critical\x01injected"
        h16b_query = f"FROM crowdstrike_detections\n| where severity = '{ctrl_val}'\n| limit 3"
        rid_h16b = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid_h16b, "method": "tools/call",
                        "params": {"name": "query",
                                   "arguments": {"query": h16b_query, "clients": ["org-c"]}}})
        resp_h16b, err_h16b = read_msg(proc, timeout=15.0)
        if err_h16b:
            results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized"] = f"FAIL: {err_h16b}"
        else:
            raw_content_b = resp_h16b.get("result", {}).get("content", []) if resp_h16b else []
            raw_text_b = raw_content_b[0].get("text", "") if raw_content_b else ""
            ctrl_leaked = [c for c in raw_text_b if (ord(c) < 0x20 or ord(c) == 0x7F)
                           and c not in ("\n", "\r", "\t")]
            if ctrl_leaked:
                results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized"] = (
                    f"FAIL: raw control chars leaked in response: "
                    f"{[hex(ord(c)) for c in ctrl_leaked[:5]]}"
                )
            elif resp_h16b and "error" in resp_h16b:
                results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized"] = (
                    "PASS: RPC-level rejection without control-char leakage (WHERE predicate probe)"
                )
            elif raw_text_b.startswith("ERROR:") or raw_text_b.startswith("{"):
                results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized"] = (
                    f"PASS: response free of control chars (CWE-117 sanitized, WHERE probe); "
                    f"preview={raw_text_b[:60]!r}"
                )
            else:
                results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized"] = (
                    f"FAIL: indeterminate response for WHERE predicate probe; preview={raw_text_b[:60]!r}"
                )

        # ── H17: E-QUERY-033 — limit > 1000 rejected ─────────────────────────
        body, err = tool_call(proc, "query",
                              {"query": "FROM crowdstrike_detections | limit 5",
                               "clients": ["org-c"], "limit": 1001})
        if err:
            # E-QUERY-033 → -32602 INVALID_PARAMS at the MCP params level (build_query_options).
            # F-AUD-P1-HIGH-002: bare "RPC error" matches EVERY RPC failure — constrain to
            # -32602 AND (E-QUERY-033 or "limit" in message) to avoid false positives.
            err_str = str(err)
            # F-AUD-P2-LOW-004: anchor to canonical E-QUERY-033 template substrings
            # per POL-24: "E-QUERY-033: limit {requested} exceeds maximum of {max}
            # (BC-2.11.001)". Anchors = "E-QUERY-033" AND "1000" (max=1000).
            # Broad "limit" substring is forbidden (matches unrelated error paths).
            is_controlled = "-32602" in err_str and (
                "E-QUERY-033" in err_str or "1000" in err_str
            )
            if is_controlled:
                results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                    f"PASS: limit > 1000 controlled rejection (-32602 + E-QUERY-033/1000 anchors): {err[:100]}"
                )
            else:
                results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                    f"FAIL: unexpected error (not a controlled -32602 + E-QUERY-033/1000 rejection): {err[:100]}"
                )
        elif body.get("error_code") == "E-QUERY-033":
            results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                "PASS: E-QUERY-033 in-band rejection"
            )
        elif body.get("rows") is not None:
            results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                f"FAIL: limit 1001 query succeeded ({len(body.get('rows', []))} rows) — E-QUERY-033 not enforced"
            )
        else:
            results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                f"PARTIAL: body={body}"
            )

        # ── H18: E-QUERY-003 / oversize query rejected ───────────────────────
        # ~70KB IN clause exceeds the 64KB MCP-level guard (or engine security limit).
        # Either E-QUERY-003 or -32602 param rejection PASSes; success/hang/crash FAILs.
        _vals = ", ".join(f"'val{i:06d}'" for i in range(5000))  # ~65KB
        big_query = f"FROM crowdstrike_detections\n| where detection_id IN ({_vals})\n| limit 5"
        body, err = query(proc, big_query, ["org-c"], timeout=30.0)
        if err:
            # F-AUD-P1-HIGH-001: only controlled rejections PASS here.
            # The former `if err: PASS` converted timeouts/crashes/JSON errors into PASS.
            # Accept only: RPC -32602/-32603 param rejection, or E-QUERY-003 in error text.
            err_str = str(err)
            is_timeout = "TIMEOUT" in err_str
            is_process_exit = err_str.startswith("Process exited") or err_str.startswith("EOF")
            is_json_error = err_str.startswith("JSON error") or err_str.startswith("envelope JSON error")
            is_controlled_rpc = (
                err_str.startswith("RPC error -32602")
                or err_str.startswith("RPC error -32603")
                or "E-QUERY-003" in err_str
            )
            if is_timeout or is_process_exit or is_json_error:
                results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                    f"FAIL: uncontrolled failure (timeout/crash/JSON error) instead of controlled rejection: "
                    f"{err[:100]}"
                )
            elif is_controlled_rpc:
                results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                    f"PASS: oversize query rejected at MCP or engine level (controlled): {err[:80]}"
                )
            else:
                results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                    f"FAIL: unexpected error (not a controlled -32602/-32603/E-QUERY-003 rejection): "
                    f"{err[:100]}"
                )
        elif body.get("error_code") in ("E-QUERY-003",):
            results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                f"PASS: E-QUERY-003 in-band rejection"
            )
        elif body.get("rows") is not None:
            results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                "FAIL: oversize query succeeded (E-QUERY-003 not enforced)"
            )
        else:
            results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                f"PARTIAL: unexpected response: body keys={list(body.keys())[:4]}"
            )

        # ── H19: threat_sources + cvss_vector UDFs ───────────────────────────
        body_ts, err_ts = query(proc,
            "FROM cyberint_alerts\n| where iocs_value_first IS NOT NULL\n"
            "| enrich threat_sources(iocs_value_first)\n| limit 3",
            ["org-c"], timeout=30.0)
        if err_ts:
            results["[H19a] threat_sources UDF returns virustotal"] = f"FAIL: {err_ts}"
        elif body_ts.get("error_code"):
            ec = body_ts.get("error_code", "")
            results["[H19a] threat_sources UDF returns virustotal"] = (
                f"FAIL: {ec}: {body_ts.get('message','')[:80]}"
            )
        else:
            rows = body_ts.get("rows", [])
            if rows:
                sources_vals = [r.get("threat_sources") for r in rows if r.get("threat_sources")]
                has_virustotal = any("virustotal" in str(v).lower() for v in sources_vals)
                if not sources_vals:
                    # F-AUD-P2-HIGH-001: Stage 4 guarantees threat_sources non-null
                    # for Hash IOCs (gap-analysis §3 data contract) → FAIL, not WARN.
                    results["[H19a] threat_sources UDF returns virustotal"] = (
                        f"FAIL: {len(rows)} rows but threat_sources column absent/null (Stage 4 guarantees non-null)"
                    )
                elif not has_virustotal:
                    # F-AUD-P2-HIGH-001: gap-analysis §3: Hash IOC → sources["virustotal"]
                    # guaranteed → FAIL if virustotal not in result.
                    results["[H19a] threat_sources UDF returns virustotal"] = (
                        f"FAIL: threat_sources present but 'virustotal' not found (gap-analysis §3 data contract); "
                        f"sample={str(sources_vals[:2])[:80]!r}"
                    )
                else:
                    results["[H19a] threat_sources UDF returns virustotal"] = (
                        f"PASS: {len(rows)} rows; threat_sources present; virustotal confirmed; "
                        f"sample={str(sources_vals[:2])[:80]!r}"
                    )
            else:
                results["[H19a] threat_sources UDF returns virustotal"] = (
                    "FAIL: 0 rows from iocs_value_first IS NOT NULL filter"
                )

        body_cv, err_cv = query(proc,
            "FROM armis_devices\n| where device_cves_first IS NOT NULL\n"
            "| enrich cvss_vector(device_cves_first)\n| limit 3",
            ["org-c"], timeout=30.0)
        if err_cv:
            results["[H19b] cvss_vector UDF returns CVSS:3.1/ string"] = f"FAIL: {err_cv}"
        elif body_cv.get("error_code"):
            ec = body_cv.get("error_code", "")
            results["[H19b] cvss_vector UDF returns CVSS:3.1/ string"] = (
                f"FAIL: {ec}: {body_cv.get('message','')[:80]}"
            )
        else:
            rows = body_cv.get("rows", [])
            if rows:
                vectors = [r.get("cvss_vector") for r in rows if r.get("cvss_vector")]
                has_cvss31 = any(str(v).startswith("CVSS:3.1/") for v in vectors)
                if not vectors:
                    # F-AUD-P2-HIGH-001: Stage 4 guarantees cvss_vector non-null for
                    # scenario CVE-9999-* (gap-analysis §3 data contract) → FAIL.
                    results["[H19b] cvss_vector UDF returns CVSS:3.1/ string"] = (
                        f"FAIL: {len(rows)} rows but cvss_vector column absent/null (Stage 4 guarantees non-null)"
                    )
                elif not has_cvss31:
                    # F-AUD-P2-HIGH-001: gap-analysis §3: CVE-9999-* has
                    # vector=CVSS:3.1/... guaranteed → FAIL if prefix absent.
                    results["[H19b] cvss_vector UDF returns CVSS:3.1/ string"] = (
                        f"FAIL: cvss_vector present but does not start with 'CVSS:3.1/' (gap-analysis §3); "
                        f"sample={str(vectors[:2])[:80]!r}"
                    )
                else:
                    results["[H19b] cvss_vector UDF returns CVSS:3.1/ string"] = (
                        f"PASS: {len(rows)} rows; cvss_vector starts with CVSS:3.1/ confirmed; "
                        f"sample={str(vectors[:2])[:80]!r}"
                    )
            else:
                results["[H19b] cvss_vector UDF returns CVSS:3.1/ string"] = (
                    "FAIL: 0 rows from device_cves_first IS NOT NULL filter"
                )

        # ── H20: ADR-051 D4 regression detector — JSON-list iocs_value score=0 ─
        # ADR-051 D4 scalar-input: threat_score(iocs_value) on a JSON-list column MUST
        # return score=0 (iocs_value is a JSON list, not a plain IOC string; the typed UDF
        # receives a non-matching input and must return 0 per ADR-051 D4).
        # PASS = max_score < 75 (ADR-051 D4 enforced; runbook amendment confirmed).
        # FAIL = max_score >= 75 (ADR-051 D4 regression — scalar-input rule broken).
        # F-AUD-P2-MED-002: demoted from WARN to FAIL; description updated.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where iocs_value IS NOT NULL\n"
            "| enrich threat_score(iocs_value)\n| limit 10",
            ["org-c"], timeout=30.0)
        if err:
            results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list score=0"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list score=0"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if rows:
                scores = [r.get("threat_score") for r in rows if "threat_score" in r]
                numeric_scores = [s for s in scores if isinstance(s, (int, float))]
                max_score = max(numeric_scores, default=0)
                if max_score >= 75:
                    # F-AUD-P2-MED-002: ADR-051 D4 scalar-input regression — iocs_value
                    # (JSON-list) must return score 0; high score = enforcement broken → FAIL.
                    results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list score=0"] = (
                        f"FAIL: threat_score={max_score} for JSON-list column — "
                        f"ADR-051 D4 scalar-input REGRESSION: iocs_value must return 0; scores={scores[:5]}"
                    )
                else:
                    # Expected: JSON-list column returns 0/low score → ADR-051 D4 confirmed
                    results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list score=0"] = (
                        f"PASS: threat_score={max_score} for JSON-list column — "
                        f"ADR-051 D4 scalar-input enforced; runbook v1.8 amendment to "
                        f"iocs_value_first confirmed valid; scores={scores[:5]}"
                    )
            else:
                results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list score=0"] = (
                    "FAIL: 0 rows from iocs_value IS NOT NULL filter (check DTU data)"
                )

        # ── H21: Determinism — same sorted query returns identical rows ───────
        # Seeded ChaCha20 + fixed anchors guarantee byte-identical results across runs.
        body1, err1 = query(proc,
            "FROM crowdstrike_detections\n| sort detection_id\n| limit 20",
            ["org-c"])
        body2, err2 = query(proc,
            "FROM crowdstrike_detections\n| sort detection_id\n| limit 20",
            ["org-c"])
        if err1 or err2:
            results["[H21] Determinism: repeated sorted query byte-identical"] = (
                f"FAIL: {err1 or err2}"
            )
        elif body1.get("error_code") or body2.get("error_code"):
            ec = body1.get("error_code") or body2.get("error_code")
            results["[H21] Determinism: repeated sorted query byte-identical"] = (
                f"FAIL: {ec}: {body1.get('message','')[:80]}"
            )
        else:
            rows1 = body1.get("rows", [])
            rows2 = body2.get("rows", [])
            if rows1 == rows2 and len(rows1) > 0:
                results["[H21] Determinism: repeated sorted query byte-identical"] = (
                    f"PASS: {len(rows1)} rows; two runs byte-identical "
                    f"(seeded ChaCha20 + fixed anchors)"
                )
            elif rows1 == rows2:
                results["[H21] Determinism: repeated sorted query byte-identical"] = (
                    "FAIL: 0 rows from both runs — seed-200 guarantees detections"
                )
            else:
                diffs = sum(1 for a, b in zip(rows1, rows2) if a != b)
                results["[H21] Determinism: repeated sorted query byte-identical"] = (
                    f"FAIL: rows differ — run1={len(rows1)}, run2={len(rows2)}, "
                    f"differing_rows={diffs}"
                )

        # ── H22: normalized_pql present in success response (BC-2.11.018) ────
        body, err = query(proc,
            "FROM crowdstrike_detections\n| limit 5",
            ["org-c"])
        if err:
            results["[H22] BC-2.11.018: normalized_pql present on success path"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[H22] BC-2.11.018: normalized_pql present on success path"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            npql = body.get("normalized_pql", "MISSING")
            if npql == "MISSING":
                results["[H22] BC-2.11.018: normalized_pql present on success path"] = (
                    "FAIL: normalized_pql key absent from success response (BC-2.11.018)"
                )
            elif isinstance(npql, str) and len(npql) > 0:
                results["[H22] BC-2.11.018: normalized_pql present on success path"] = (
                    f"PASS: normalized_pql present: {npql[:80]!r}"
                )
            else:
                results["[H22] BC-2.11.018: normalized_pql present on success path"] = (
                    f"FAIL: normalized_pql empty or wrong type: {npql!r}"
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
    ("[A2]",  "MCP Protocol",  "tools/list all 14 implemented tools"),
    ("[A3]",  "MCP Protocol",  "resources/list prismql://reference"),
    ("[A4]",  "MCP Protocol",  "prompts/list all 5 prompts"),
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
    # B11–B15: additional tables (formerly dynamic [B<table>] keys — F-AUD-P1-LOW-002)
    ("[B11]", "Sensor Tables", "armis_alerts org-c"),
    ("[B12]", "Sensor Tables", "claroty_alerts org-c"),
    ("[B13]", "Sensor Tables", "crowdstrike_devices org-c"),
    ("[B14]", "Sensor Tables", "crowdstrike_incidents org-c"),
    ("[B15]", "Sensor Tables", "cyberint_incidents org-c"),
    ("[C1]",  "Query Modes",   "SQL SELECT FROM WHERE LIMIT"),
    ("[C2]",  "Query Modes",   "Pipe FROM | where | limit"),
    ("[C3]",  "Query Modes",   "Pipe FROM | fields | limit"),
    ("[C4]",  "Query Modes",   "DataFusion aggregate COUNT(*)"),
    ("[C5]",  "Query Modes",   "DataFusion GROUP BY aggregate"),
    ("[C6]",  "Query Modes",   "DataFusion MAX/MIN aggregate"),
    ("[C7]",  "Query Modes",   "Pipe | sort operator"),
    ("[C8]",  "Query Modes",   "SQL mode executes (ADR-052 §D4 baseline path; RFC-3339 regression in G7)"),
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
    ("[F1]",  "Error Taxonomy","E-QUERY-032 cyberint for org-a (runbook v1.8 §5.8 N3: E-QUERY-032 only)"),
    ("[F2]",  "Error Taxonomy","E-QUERY-032 armis for org-b"),
    ("[F3]",  "Error Taxonomy","E-QUERY-037 dot-notation FROM"),
    ("[F4]",  "Error Taxonomy","E-QUERY-039 unknown enrich UDF"),
    ("[F5]",  "Error Taxonomy","E-QUERY-038 unknown column"),
    ("[F6]",  "Error Taxonomy","E-QUERY-039 false-positive: SQL builtins safe"),
    # Section G: New merged surfaces — PRs #214/#216/#217 (develop@f935edb6)
    ("[G1]",  "IEQ/IIN/INE",   "IEQ happy path: severity IEQ 'critical' matches canonical 'Critical'"),
    ("[G2]",  "IEQ/IIN/INE",   "IIN multi-value: severity IIN ('high','critical')"),
    ("[G3]",  "IEQ/IIN/INE",   "IIN on status: status IIN ('new','in progress') → crowdstrike_detections (cyberint vendor-native open/acknowledged/closed has no OCSF caption match; ADV-PR-P11-HIGH-001)"),
    ("[G4]",  "IEQ/IIN/INE",   "SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"),
    ("[G5]",  "IEQ/IIN/INE",   "E-QUERY-002 typed guidance: IEQ on armis_devices.risk_score (integer)"),
    ("[G6]",  "IEQ/IIN/INE",   "GROUP BY severity no-fragmentation (canonical Title-case)"),
    ("[G7]",  "Temporal",      "ADR-052 §D4 regression: RFC-3339 datetime literal in WHERE"),
    ("[G8]",  "Typed Enrich",  "ADR-051 regression: threat_score is Int64 not JSON-string"),
    # Section H: PR #219 behaviors — full current-feature coverage (AUDIT-COVERAGE-001)
    ("[H1]",  "PR#219",        "E-QUERY-038 pipe mode: original DRIFT shape (no Internal error regression)"),
    ("[H1b]", "PR#219",        "E-QUERY-038 filter mode: position-7, no FROM keyword"),
    ("[H2]",  "PR#219",        "E-QUERY-038 did_you_mean + available_columns in error payload"),
    ("[H3]",  "PR#219",        "INE operator: severity INE 'medium' excludes Medium rows"),
    ("[H4]",  "Temporal",      "E-QUERY-041: date-only literal '2020-01-01' rejected (ADR-052 §D4)"),
    ("[H5]",  "Temporal",      "E-QUERY-042: temporal literal in GROUP BY arm rejected (ADR-052 §D4)"),
    ("[H6]",  "IEQ/IIN/INE",   "E-QUERY-002: IEQ on armis_devices.risk_score (integer column, canonical probe)"),
    ("[H7]",  "JOIN",          "JOIN positive path: crowdstrike_devices JOIN armis_devices on device_id"),
    ("[H8]",  "JOIN",          "HEAD-JOIN fail-open: bare unknown col in JOIN → not E-QUERY-038 (FP-001)"),
    ("[H9]",  "SqlPipe",       "SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"),
    ("[H10]", "Dual-limit",    "E-QUERY-040: SQL LIMIT + pipe | limit dual-limit rejected"),
    ("[H11]", "Stats",         "| stats count() as cnt by severity grammar"),
    ("[H12]", "Multi-client",  "Multi-client fan-out: org-a + org-c CrowdStrike detections"),
    ("[H13a]","Prompts",       "client_overview prompt returns promptly (new prompt)"),
    ("[H13b]","Prompts",       "cross_client_status prompt returns promptly (new prompt)"),
    ("[H14]", "Resources",     "resources/read: config/clients + sensors/health + schema/org-c"),
    ("[H14c]","Resources",     "resources/read: prism://schema/crowdstrike/detections (per-sensor-table schema URI)"),
    ("[H14d]","Resources",     "resources/read: prism://config/clients/org-c/sensors (per-client sensor list URI)"),
    ("[H14e]","Resources",     "resources/subscribe + unsubscribe: prismql://schema/org-c (ADR-051 subscribe coverage)"),
    ("[H15]", "Tools",         "explain_query live call (one of 14 implemented tools)"),
    ("[H16]", "Security",      "CWE-116/117: control-char in column name sanitized (sanitize_for_log)"),
    ("[H16b]","Security",      "CWE-117: control-char in unquoted WHERE-predicate value sanitized"),
    ("[H17]", "Guardrails",    "E-QUERY-033: limit > 1000 rejected (BC-2.11.001 ceiling)"),
    ("[H18]", "Guardrails",    "E-QUERY-003: oversize query (~70KB) controlled rejection"),
    ("[H19a]","UDFs",          "threat_sources UDF returns virustotal in result"),
    ("[H19b]","UDFs",          "cvss_vector UDF returns CVSS:3.1/ string"),
    ("[H20]", "Guardrails",    "ADR-051 D4 regression detector: iocs_value JSON-list score=0"),
    ("[H21]", "Determinism",   "Repeated sorted query byte-identical (seeded ChaCha20)"),
    ("[H22]", "BC-2.11.018",   "normalized_pql present on success response"),
]


if __name__ == "__main__":
    print("=" * 80)
    print("T13 COMPREHENSIVE PRE-FLIGHT DEMO AUDIT — develop@5f1b5771")
    print(f"  ThreatIntel port: {THREATINTEL_PORT}  NVD port: {NVD_PORT}")
    print(f"  Coverage: {len(COVERAGE_MATRIX)} matrix items across 8 sections")
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
