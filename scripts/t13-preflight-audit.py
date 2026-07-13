#!/usr/bin/env python3
"""
T13 Comprehensive Pre-flight Demo Audit Script — develop baseline at AUDIT-COVERAGE-001 branch point + fix/T13-audit-coverage Section-H extension
Drives the prism MCP server over stdio (newline-delimited JSON) and verifies
the FULL demo feature coverage matrix (extends the 18-item smoke audit).

Usage:
    python3 scripts/t13-preflight-audit.py
    PRISM_THREATINTEL_PORT=65343 PRISM_NVD_PORT=65344 python3 scripts/t13-preflight-audit.py

Requirements:
    - prism-dtu-demo-server must be running (bash scripts/demo-run.sh)
    - PRISM_THREATINTEL_PORT and PRISM_NVD_PORT env vars (from demo-run.sh output)

Coverage matrix (see len(COVERAGE_MATRIX) for current authoritative count):
  1. Full 54-tool catalog asserted present via tools/list (14 live LIVE_TOOLS + 40 NYA stubs;
     OBS-001 union); 5 read-only live tools exercised end-to-end; 9 non-exercised live tools:
     4 structurally read-only but out of preflight scope (list_aliases, list_sensor_specs,
     explain_alias, validate_config) + 5 mutating (reload_config, create_alias, delete_alias,
     confirm_action, add_sensor_spec) —
     preflight is READ-ONLY; no write-back to sensors, no config changes, no alias mutations
  2. 4 sensor adapters × their tables (CrowdStrike, Cyberint, Claroty, Armis — Section B) + 2 global enrichment DTUs (ThreatIntel, NVD — Section E, exercised as typed-UDF callees, not as sensors-with-tables)
  3. All query modes (SQL, pipe, SqlPipe, filter, stats, joins, enrichment, temporal)
  4. All scenario stages per client (in-session determinism verified (H21))
  5. Multi-client data segregation + org-scoping error paths + multi-client fan-out
  6. Enrichment correlation (ThreatIntel IOCs + NVD CVEs, threat_sources/cvss_vector)
  7. Error taxonomy paths (E-QUERY-032/-033/-037/-038/-039/-040/-041/-042/-043/-003)
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
 13. B-hardening pass-21 additions: H5b (E-QUERY-042 OrderBy arm — Timestamp literal in
     ORDER BY position, DEFECT-EQUERY042-GROUPBY-DEADARM-001 extension, ADR-052 §D4 arm 7),
     H5c (E-QUERY-042 NonColumnLhsComparison arm — function-call LHS, ADR-052 §D4 arm 4),
     H24 (E-QUERY-043 IN-subquery in projection position — check_expr_insubquery_projection,
     F-CSD-P4-001, DEFECT-CSDEVICES-EMPTY-PIPELINE-001)
"""

import subprocess
import json
import os
import sys
import time
import select
import fcntl
import re
import traceback
import unicodedata
from pathlib import Path
import itertools

# PRISM_BIN: resolved in priority order:
#   1. $PRISM_BIN env var (explicit override)
#   2. $CARGO_TARGET_DIR/release/prism (respects Cargo target-dir override)
#   3. <repo-root>/target/release/prism (repo-relative default; script is in scripts/)
_repo_root = Path(__file__).resolve().parent.parent
_cargo_target_dir = os.environ.get("CARGO_TARGET_DIR")

# F-AUD-P11-HIGH-001: portable .factory/ resolver.
# _repo_root is the git worktree root (e.g. .worktrees/AUDIT-COVERAGE-001/ when run from a
# worktree).  The .factory/ directory is a separate git worktree mounted at the MAIN repo
# root (layout: <main>/.worktrees/<id>/; .factory/ is at <main>/.factory/).
# F-AUD-P18-LOW-001: worktree-aware resolver discipline.
# Resolution order:
#   1. $PRISM_FACTORY_ROOT env var (explicit override — set to main repo root path when
#      running in a detached worktree that has a stale .factory/ snapshot)
#   2. Walk up _repo_root's ancestors; when a candidate root's .git is a FILE (gitdir
#      pointer = worktree marker), skip its .factory/ — worktree .factory/ snapshots are
#      stale; the canonical factory-artifacts mount lives at the main repo root (whose
#      .git is a directory). Continue walking until a .git-directory root is found.
#   3. None if not found — caller emits a FAIL listing all tried paths.
def _find_factory_file(*rel_parts: str) -> tuple:
    """Locate a file inside .factory/ portably across main and worktree layouts.

    Returns (Path, tried_list) where Path is the resolved file (or None if not found)
    and tried_list is the list of Paths attempted (for FAIL diagnostics).

    Worktree-aware: when a candidate ancestor's .git entry is a FILE (not a directory),
    that ancestor is a git worktree whose .factory/ may be a stale snapshot. Such
    ancestors are skipped; only ancestors with a .git DIRECTORY (main repo root) are
    accepted. Set $PRISM_FACTORY_ROOT to the main repo root path to override all
    walking logic (useful when running from an unusual directory layout).
    """
    _rel = Path(".factory").joinpath(*rel_parts)
    _tried: list = []
    # $PRISM_FACTORY_ROOT: explicit override — bypasses all walking logic.
    _factory_root_override = os.environ.get("PRISM_FACTORY_ROOT")
    if _factory_root_override:
        _cand = Path(_factory_root_override) / _rel
        _tried.append(_cand)
        if _cand.exists():
            return _cand, _tried
        return None, _tried
    # Walk from _repo_root upward (includes _repo_root itself as first candidate).
    # Worktree roots have .git as a FILE (gitdir pointer), not a directory.
    # Their .factory/ may be a stale snapshot — skip and continue walking to the
    # main repo root (whose .git is a directory). Canonical factory-artifacts mount
    # is always at the main repo root.
    for _ancestor in [_repo_root, *_repo_root.parents]:
        _cand = _ancestor / _rel
        _tried.append(_cand)
        _git = _ancestor / ".git"
        if _git.is_file():
            # Worktree: .git is a gitdir-pointer file — skip this ancestor's .factory/
            # to avoid reading a stale snapshot (F-AUD-P18-LOW-001).
            continue
        if _cand.exists():
            return _cand, _tried
        # Stop at filesystem root
        if _ancestor == _ancestor.parent:
            break
    return None, _tried
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

# OBS-004: CROWDSTRIKE_BASE_URL / ARMIS_INSTANCE_URL / CLAROTY_INSTANCE_URL have no port.
# This is intentional — DTU ports are ephemeral (scripts/demo.toml [harness] has no port
# field; the OS assigns a random available port at server start). These env vars satisfy
# boot step 4a env_resolver.rs (E-SPEC-024 guard: vars must be non-empty); per-org overlay
# TOMLs generated by demo-run.sh after the DTU server starts then replace base_url with the
# actual DTU clone URL (including ephemeral port) at step 4c.
# CROWDSTRIKE_BASE_URL must be exactly "http://127.0.0.1" (no port): the crowdstrike-oauth2
# plugin manifest allowed_urls = ["api.crowdstrike.com", "127.0.0.1"] — SEC-003 validates
# the TYPE spec base_url host at step 7.5b; adding a port here would cause SEC-003 to reject
# the host "127.0.0.1:NNNNN" (host includes port in URL parse) on first boot.
# Source: scripts/demo-run.sh lines 354-373; scripts/demo.toml [harness] (no port).
# PRISM_THREATINTEL_PORT / PRISM_NVD_PORT are env-provided at runtime (printed by demo-run.sh
# after the DTU server starts); they use the same ephemeral pattern but are exposed as
# explicit env vars because their base_url is not replaced via per-org overlays.
ENV = {
    **os.environ,
    "CROWDSTRIKE_BASE_URL": "http://127.0.0.1",      # no port: ephemeral DTU; overlay provides port at step 4c
    "ARMIS_INSTANCE_URL": "http://127.0.0.1",         # no port: ephemeral DTU; overlay provides port at step 4c
    "CLAROTY_INSTANCE_URL": "http://127.0.0.1",       # no port: ephemeral DTU; overlay provides port at step 4c
    "CYBERINT_ENVIRONMENT": "demo",
    "PRISM_DTU_MODE": "true",
    "PRISM_THREATINTEL_BASE_URL": f"http://127.0.0.1:{THREATINTEL_PORT}",
    "PRISM_THREATINTEL_API_KEY": "demo-threatintel-api-key",
    "PRISM_NVD_BASE_URL": f"http://127.0.0.1:{NVD_PORT}",
    "PRISM_NVD_API_KEY": "demo-nvd-api-key",
}

_REQ_ID = 0


class _PrismCrashError(Exception):
    """Raised by send_msg when the prism process crashes (BrokenPipeError/OSError).

    MED-005: propagates through tool_call/query/helpers to run_audit's except clause,
    which records partial results and allows the SUMMARY + matrix-mismatch gate to
    surface all uncollected checks as "in COVERAGE_MATRIX but no result written".
    Never exits 0 — the partial results dict contains at least one FAIL entry.
    """


def next_id():
    global _REQ_ID
    _REQ_ID += 1
    return _REQ_ID


def send_msg(proc, msg):
    """Send a JSON-RPC message over stdio (newline-delimited JSON).

    MED-005: raises _PrismCrashError (instead of propagating raw BrokenPipeError/OSError)
    so callers receive a structured crash signal rather than a raw traceback.  run_audit
    catches _PrismCrashError, records partial results, and still prints SUMMARY + DEMO-READY=no.
    """
    data = json.dumps(msg) + "\n"
    try:
        proc.stdin.write(data.encode())
        proc.stdin.flush()
    except (BrokenPipeError, OSError) as e:
        # prism crashed mid-audit — stdin write/flush failed.
        rc = proc.poll()
        raise _PrismCrashError(
            f"prism process crashed (exit={rc}, {type(e).__name__}: {e})"
        ) from e


# F-AUD-P19-LOW-006(b): module-level residual buffer persists partial reads across
# read_msg calls so that multi-message chunks aren't silently discarded.
# Keyed by stdout fd to remain correct if callers ever use multiple procs in the
# same process lifetime (no caller currently does, but defensive is cheap).
_READ_BUF: dict = {}  # fd -> bytearray


def read_msg(proc, timeout=15.0, expected_id=None):
    """Read a newline-delimited JSON response from stdout with timeout.

    F-AUD-P19-LOW-006:
    (a) expected_id matching: skip JSON-RPC notifications (no 'id' field) and
        mismatched-id messages (up to timeout), returning only the matching response.
    (b) Residual buffer (_READ_BUF) persists across calls so that multi-message
        chunks (e.g. a notification followed by the response in the same read()
        syscall) are not discarded.
    """
    fd = proc.stdout.fileno()
    fl = fcntl.fcntl(fd, fcntl.F_GETFL)
    # Set O_NONBLOCK once per fd — all reads go through the select+non-blocking
    # pattern, so stdout remains non-blocking for the process lifetime by design.
    # Guard avoids redundant fcntl syscalls on every read_msg invocation.
    if not (fl & os.O_NONBLOCK):
        fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
    if fd not in _READ_BUF:
        _READ_BUF[fd] = bytearray()
    buf = _READ_BUF[fd]

    start = time.time()
    while True:
        elapsed = time.time() - start
        if elapsed > timeout:
            return None, f"TIMEOUT after {timeout:.1f}s"

        # Drain any complete lines from the residual buffer first — may contain a
        # valid response written before the last read() call returned.
        while b"\n" in buf:
            idx = buf.index(b"\n")
            raw_line = bytes(buf[:idx]).strip()
            del buf[:idx + 1]
            if not raw_line:
                continue
            try:
                parsed = json.loads(raw_line.decode("utf-8"))
            except json.JSONDecodeError as e:
                return None, f"JSON error: {e} on: {raw_line[:200]!r}"
            if not isinstance(parsed, dict):
                return None, (
                    f"non-dict JSON-RPC response: {type(parsed).__name__}: "
                    f"{raw_line[:80]!r}"
                )
            # F-AUD-P19-LOW-006(a): skip notifications (no 'id' field) and
            # mismatched-id messages when expected_id is given.
            if expected_id is not None:
                msg_id = parsed.get("id")
                if msg_id is None:
                    # JSON-RPC notification — no id; skip and keep draining.
                    continue
                if msg_id != expected_id:
                    # Mismatched id — skip and keep draining.
                    continue
            return parsed, None

        # No complete line in buffer yet; check process state before blocking.
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
            buf.extend(chunk)
        except BlockingIOError:
            continue


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
    # OBS-004 (F-AUD-P28): guard against non-dict result — e.g. a bare string or list in
    # the JSON-RPC result field would cause AttributeError on .get("content") below.
    # F-AUD-P30-OBS-002: bind _res once so isinstance check and type() use the same
    # object; handles absent-key (None → NoneType diagnostic) cleanly without double-lookup.
    _res = resp.get("result")
    if not isinstance(_res, dict):
        return {}, f"non-dict result: {type(_res).__name__}"
    content = _res.get("content", [])
    if not content:
        return {}, "empty content list"
    if not isinstance(content[0], dict):  # OBS-006: guard non-dict content[0] (mirrors envelope dict-guard)
        return {}, "non-dict content[0]"
    text = content[0].get("text", "")
    if not isinstance(text, str):  # F-AUD-P32-LOW-001: guard non-string text (mirrors content[0] dict-guard)
        return {}, f"non-string text: {type(text).__name__}"
    if not text:
        return {}, "empty text in content[0]"
    # F-AUD-P16-MED-002: extract structuredContent.error unconditionally — consumed on
    # BOTH the plain-text error path and the JSON-envelope path below.
    # (code, did_you_mean, available_columns, etc. — field name is `code` not
    # `error_code`; no `details.*` sub-keys in this codebase's envelope).
    sc_content = resp.get("result", {}).get("structuredContent", {})
    sc_err_obj = sc_content.get("error") if isinstance(sc_content, dict) else None
    # Plain text error: "ERROR: [type] - message"
    if text.startswith("ERROR:"):
        m = re.search(r"(E-[A-Z]+-\d+)", text)
        error_code = m.group(1) if m else "UNKNOWN"
        # Strip "ERROR: [type] - " prefix for message
        msg = re.sub(r"^ERROR:\s*\[[^\]]+\]\s*-\s*", "", text).strip()
        result_body = {"error_code": error_code, "message": msg}
        if isinstance(sc_err_obj, dict):
            result_body["_sc_error"] = sc_err_obj
        return result_body, None
    try:
        envelope = json.loads(text)
    except json.JSONDecodeError as e:
        return {}, f"envelope JSON error: {e}, raw: {text[:100]!r}"
    # F-AUD-P16-MED-001 sweep: guard against non-dict JSON envelope (list/int/str
    # would raise AttributeError on .get() below).
    if not isinstance(envelope, dict):
        return {}, f"envelope non-dict JSON: {type(envelope).__name__}: {text[:80]!r}"
    results_body = envelope.get("results", {})
    # F-AUD-P35-OBS-001: guard against non-dict envelope 'results' (null/list/scalar from
    # malformed server response would TypeError on subscript/mutate below; mirrors sibling
    # dict-guards for _res, content[0], and envelope earlier in this function).
    if not isinstance(results_body, dict):
        return {}, f"non-dict envelope 'results': {type(results_body).__name__}"
    # F-AUD-P16-MED-002: attach _sc_error on the JSON-envelope path as well.
    if isinstance(sc_err_obj, dict):
        results_body["_sc_error"] = sc_err_obj
    return results_body, None


def tool_call(proc, name, arguments, timeout=25.0):
    """Helper: send a tools/call and parse the response."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    })
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
    if err:
        return None, err
    return parse_envelope(resp)



def sensor_errors_gate(check_id: str, body: dict, results: dict) -> bool:
    """Hard-FAIL gate: returns True (and records FAIL) if the query envelope
    carries partial fan-out failure evidence in sensor_errors. Callers must
    skip row inspection when this returns True."""
    _se = body.get("sensor_errors", [])
    if _se:
        results[check_id] = f"FAIL: partial fan-out failure — sensor_errors={_se[:2]} (F-AUD-P30-MED-001)"
        return True
    return False


def prompt_get(proc, name, arguments, timeout=5.0):
    """Helper: send a prompts/get and return (result_dict, err)."""
    rid = next_id()
    send_msg(proc, {
        "jsonrpc": "2.0", "id": rid, "method": "prompts/get",
        "params": {"name": name, "arguments": arguments},
    })
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
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
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
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
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
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
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
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
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
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
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
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
    resp, err = read_msg(proc, timeout=timeout, expected_id=rid)
    if err:
        return None, err
    if "error" in resp:
        return None, f"error={resp['error']}"
    return resp.get("result", {}), None


def query(proc, pql, clients, timeout=25.0):
    """Helper: run a PrismQL query and return (body, err)."""
    return tool_call(proc, "query", {"query": pql, "clients": clients}, timeout=timeout)


def _audit_sort_key(item_key: str):
    """Numeric-aware sort key for audit result keys.

    Parses the leading bracket prefix to produce a tuple (section, number, suffix)
    so that e.g. [A1] < [A2] < [A10] (not [A1] < [A10] < [A2] as lexicographic sort gives).
    [BOOT] sorts before all section letters.
    Unknown / non-matching prefixes sort after all known sections.
    """
    # Grammar shared with mismatch-scan regex (~line 5666): [A-Z]+[0-9]+[a-z]{0,2}
    # (F-AUD-P40-OBS-003: [a-z]? → [a-z]{0,2} for 0/1/2-char suffix parity).
    m = re.match(r'\[([A-Z]+)(\d+)([a-z]{0,2})\]', item_key)
    if m:
        return (m.group(1), int(m.group(2)), m.group(3))
    # "BOOT" key (stored without brackets — pre-initialize short-circuit): sorts first.
    # F-AUD-P11-LOW-001: stored key is "BOOT" (not "[BOOT]"); startswith('[BOOT]') never matched.
    if item_key == "BOOT":
        return ('', 0, '')
    # Unknown prefix: sort after all known sections
    return ('~', 0, item_key)


# OBS-004: H7 result key defined once to prevent write-site / read-site string drift.
# Used at the H7 write site (results[_H7_RESULT_KEY] = ...) and
# the H8 read site (_h7_key = _H7_RESULT_KEY).
_H7_RESULT_KEY = "[H7] JOIN positive path: crowdstrike_devices JOIN armis_devices"
# F-AUD-P15-LOW-001: A22 result key defined once next to _H7_RESULT_KEY to prevent
# the 11 write-site / 1 read-site string drift (12 literals → 1 constant).
_A22_RESULT_KEY = "[A22] check_sensor_health (S-5.04 gate)"
# F-AUD-P38-LOW-001: H23 result key hoisted to prevent 6-site stale-label drift;
# old literal named only iocs_value but check covers the full 6-UDF matrix
# (3 ThreatIntel + 3 NVD/cvss). [H23] prefix kept verbatim for COVERAGE_MATRIX parity guard.
_H23_RESULT_KEY = "[H23] Runbook enrich-call drift: no pre-ADR-051 non-_first UDF forms (6-UDF matrix)"
# F-AUD-P40-OBS-001: H8 result key hoisted to prevent 9-site stale-label drift.
_H8_RESULT_KEY = "[H8] HEAD-JOIN fail-open: bare unknown col in JOIN (not E-QUERY-038)"
# F-AUD-P30-MED-003: EXPECTED_SENSORS hoisted to module level — single source of truth
# shared by A22 (sensor_id set-equality gate) and H14b (resources/read health sensor set-equality).
# CONSCIOUS-UPDATE REQUIRED: if the org-c demo config is updated to add or remove a sensor type,
# update EXPECTED_SENSORS here; A22 and H14b both reference this constant.
# Source: crates/prism-sensors/specs/*.sensor.toml entries registered for org-c
# (demo config: .prism/config.toml / demo-config.toml sensor_type assignments).
EXPECTED_SENSORS = {"crowdstrike", "armis", "claroty", "cyberint"}


def run_audit():
    results = {}
    # F-AUD-P16-OBS-001: PID-suffix prevents log collision under parallel audit runs.
    # F-AUD-P16-LOW-001: open into a named variable so the handle is closed in finally.
    _mcp_log_path = f"/tmp/prism-audit-mcp-{os.getpid()}.log"
    _mcp_log_fh = open(_mcp_log_path, "w")

    try:
        proc = subprocess.Popen(
            [PRISM_BIN, "--config-dir", CONFIG_DIR, "start"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=_mcp_log_fh,
            env=ENV,
        )
    except FileNotFoundError:
        _mcp_log_fh.close()
        results["BOOT"] = (
            f"FAIL: binary not found at {PRISM_BIN} — run cargo build --release"
        )
        return results

    print(f"  MCP server log: {_mcp_log_path}")

    try:
        # MED-001 (F-AUD-P26): boot ready-probe — two-phase design:
        #   (a) Stability window (2.0s): poll every 0.5s; once the process has been
        #       continuously alive for 2.0s, break and proceed to initialize.
        #       Warm starts complete in <0.5s; cold-start RocksDB open can take several
        #       seconds but the process remains alive throughout — the stability window
        #       fires after 2.0s of continued life regardless of server readiness.
        #   (b) Crash detection: if the process exits at ANY point within the 15s total
        #       budget, fail immediately with exit info.
        # The initialize call below has its own 10s read timeout as the real readiness gate.
        _boot_budget_s = 15.0
        _boot_stability_window_s = 2.0
        _boot_poll_interval_s = 0.5
        _boot_elapsed = 0.0
        _boot_stable_elapsed = 0.0
        rc = proc.poll()
        while _boot_elapsed < _boot_budget_s:
            if rc is not None:
                # Process exited before the stability window completed — immediate FAIL.
                _mcp_log_fh.flush()
                with open(_mcp_log_path) as f:
                    last_lines = f.readlines()[-5:]
                results["BOOT"] = (
                    f"FAIL: process exited rc={rc} after {_boot_elapsed:.1f}s, "
                    f"last log: {''.join(last_lines).strip()}"
                )
                return results
            # OBS-002: check stability BEFORE sleeping so the break only fires after
            # _boot_stability_window_s of actual observed wall time (2.0s = 4 × 0.5s
            # sleeps).  The previous ordering incremented _boot_stable_elapsed before
            # sleeping, causing the break to fire after only 1.5s (3 sleeps × 0.5s).
            if _boot_stable_elapsed >= _boot_stability_window_s:
                # Process alive for stability window — proceed to initialize.
                break
            time.sleep(_boot_poll_interval_s)
            _boot_elapsed += _boot_poll_interval_s
            rc = proc.poll()
            _boot_stable_elapsed += _boot_poll_interval_s  # increment AFTER sleep+poll

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
        resp, err = read_msg(proc, timeout=10.0, expected_id=rid)
        if err:
            results["[A1] INIT: MCP server boots and responds"] = f"FAIL: {err}"
            return results
        # HIGH-001 (F-AUD-P25): guard JSON-RPC error response before reading serverInfo.
        # An initialize that returns {"error": {...}} is a protocol failure — reading
        # serverInfo from that envelope would give vacuous {} and silently PASS.
        if resp and "error" in resp:
            _a1_err = resp["error"]
            _a1_code = _a1_err.get("code", "?")
            _a1_msg = str(_a1_err.get("message", ""))[:120]
            results["[A1] INIT: MCP server boots and responds"] = (
                f"FAIL: initialize returned JSON-RPC error code={_a1_code} message={_a1_msg!r}"
            )
            return results
        server_info = resp.get("result", {}).get("serverInfo", {})
        # HIGH-001: assert serverInfo is a non-empty dict with a name field — MCP spec requires
        # serverInfo.name; an empty or missing serverInfo means the server is not correctly
        # identifying itself.
        if not isinstance(server_info, dict) or not server_info.get("name"):
            results["[A1] INIT: MCP server boots and responds"] = (
                f"FAIL: serverInfo missing or has no 'name' field (MCP spec requires serverInfo.name); "
                f"serverInfo={server_info!r}"
            )
            return results
        results["[A1] INIT: MCP server boots and responds"] = f"PASS: server={server_info}"

        send_msg(proc, {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
        time.sleep(0.3)

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION A: MCP Protocol Coverage — tools/list, resources/list,
        #            prompts/list, all tool calls
        # ═══════════════════════════════════════════════════════════════════════

        # ── A2: tools/list — enumerate all available tools ────────────────────
        # NOTE: query_tutorial and investigate_host are MCP Prompts, NOT Tools.
        # The expected tools are ALL 54 registered MCP tools (server.rs has 56 #[tool*]
        # annotations per `rg '^\s*#\[tool' server.rs | wc -l`: 2 struct/impl macros
        # (#[tool_router], #[tool_handler]) + 54 #[tool(...)] method annotations:
        # 14 real implementations (LIVE_TOOLS), 40 runtime -32003 NYA stubs
        # (NOT_YET_AVAILABLE_TOOLS)).
        #
        # tools/list returns ALL 54 registered tools (server.rs has NO custom list_tools
        # override; #[tool_handler] returns tool_router().list_all() — confirmed by
        # test_MCP_01_capability_classification_partitions_tool_catalog).
        # OBS-001: A2 was previously an exact-set check against only the 14 live tools
        # (EXPECTED_TOOLS_FULL), which produced a false-FAIL because tools/list returns
        # all 54 and the 40 NYA tools appeared as "extra".  A2 now asserts the full
        # 54-tool union (EXPECTED_TOOLS_FULL | EXPECTED_TOOLS_NYA), with missing/extra
        # diagnostics split by class (live vs NYA vs unknown).
        # OBS-001 cross-reference → A23: A23 derives the NYA stub set at runtime as
        # (tool_names − EXPECTED_TOOLS_FULL).  With A2 gating full-54 membership, A23's
        # derivation is bounded to exactly EXPECTED_TOOLS_NYA when A2 passes.
        # CONSCIOUS-UPDATE: any tool registration change must update EXPECTED_TOOLS_FULL
        # or EXPECTED_TOOLS_NYA here AND the A23 cross-reference comment in the same
        # commit (grounded in LIVE_TOOLS / NOT_YET_AVAILABLE_TOOLS in
        # crates/prism-mcp/src/server.rs).
        #
        # READ-ONLY RATIONALE (F-AUD-P2-MED-005): all 14 live tools are asserted PRESENT
        # via tools/list, but only 5 read-only tools are exercised end-to-end:
        #   - query, explain_query, list_capabilities, prism_describe, check_sensor_health
        # The 9 non-exercised live tools are deliberately NOT invoked — this preflight
        # audit is READ-ONLY and must not mutate the demo environment before recording.
        # 4 structurally read-only but out of preflight scope: list_aliases,
        # list_sensor_specs, explain_alias, validate_config.
        # 5 mutating (alter demo state): reload_config, create_alias, delete_alias,
        # confirm_action, add_sensor_spec.
        tools_result, err = list_tools(proc)
        # Grounded in LIVE_TOOLS constant in crates/prism-mcp/src/server.rs (14 tools).
        # F-AUD-P26-OBS-002 (exact-set tightening from previous pass): changes to this
        # set require a conscious-update in the same commit.
        EXPECTED_TOOLS_FULL = {
            "query", "explain_query", "list_capabilities", "prism_describe",
            "check_sensor_health", "reload_config",
            "create_alias", "list_aliases", "delete_alias", "explain_alias",
            "confirm_action", "add_sensor_spec", "list_sensor_specs", "validate_config",
        }
        # Grounded in NOT_YET_AVAILABLE_TOOLS constant in crates/prism-mcp/src/server.rs.
        # Byte-exact copy of NOT_YET_AVAILABLE_TOOLS @ server.rs ~line 1447 (40 names).
        # CONSCIOUS-UPDATE: when a stub is implemented, move its name from this set to
        # EXPECTED_TOOLS_FULL in the same commit.
        EXPECTED_TOOLS_NYA = {
            "get_diagnostics", "create_schedule", "list_schedules", "delete_schedule",
            "get_diff_results", "create_rule", "list_rules", "delete_rule",
            "create_case", "list_cases", "get_case", "update_case", "case_metrics",
            "list_credentials", "credential_status", "configure_credential_source",
            "delete_credential", "watchdog_status", "list_alerts", "get_alert",
            "acknowledge_alert", "crowdstrike_contain_host", "crowdstrike_lift_containment",
            "list_packs", "explain_pack", "create_pack", "delete_pack",
            "list_infusions", "infusion_status", "reload_infusion",
            "list_plugins", "plugin_status", "reload_plugin",
            "list_actions", "action_status", "fire_action", "test_action",
            "create_action", "delete_action", "get_help",
        }
        _EXPECTED_TOOLS_ALL = EXPECTED_TOOLS_FULL | EXPECTED_TOOLS_NYA  # 54 total
        if err:
            results["[A2] tools/list: full 54-tool catalog present (14 live + 40 NYA)"] = f"FAIL: {err}"
        else:
            tool_names = {t.get("name", "") for t in tools_result.get("tools", [])}
            missing_live = EXPECTED_TOOLS_FULL - tool_names
            missing_nya = EXPECTED_TOOLS_NYA - tool_names
            extra_unknown = tool_names - _EXPECTED_TOOLS_ALL
            if missing_live or missing_nya or extra_unknown:
                results["[A2] tools/list: full 54-tool catalog present (14 live + 40 NYA)"] = (
                    f"FAIL: 54-tool catalog mismatch (OBS-001): "
                    + (f"missing_live={sorted(missing_live)}; " if missing_live else "")
                    + (f"missing_nya={sorted(missing_nya)}; " if missing_nya else "")
                    + (f"extra_unknown={sorted(extra_unknown)}; " if extra_unknown else "")
                    + f"got {len(tool_names)} tools"
                )
            else:
                results["[A2] tools/list: full 54-tool catalog present (14 live + 40 NYA)"] = (
                    f"PASS: {len(tool_names)} tools — exact 54-tool catalog match "
                    f"(14 live + 40 NYA; OBS-001 union assertion confirmed)"
                )

        # ── A3: resources/list — exact static resource set ───────────────────
        # LOW-003 (F-AUD-P26): require ALL 3 static resources returned by
        # build_resource_list() in crates/prism-mcp/src/resources.rs:
        #   URI_CONFIG_CLIENTS = "prism://config/clients"
        #   URI_SENSORS_HEALTH = "prism://sensors/health"
        #   schema::URI_PQL_REFERENCE = "prismql://reference"
        # Template-resources (prismql://schema/{client_id}, prism://config/clients/{client_id}/sensors,
        # prism://schema/{sensor_id}/{table_name}) live in resources/list_templates, NOT
        # resources/list — do not require them here.
        # Exact-set discipline (A2 parity): missing OR extra static resources → FAIL.
        # CONSCIOUS-UPDATE REQUIRED: if build_resource_list() is updated to add or remove
        # a static resource, update REQUIRED_RESOURCES in the same change.
        REQUIRED_RESOURCES = {
            "prism://config/clients",
            "prism://sensors/health",
            "prismql://reference",
        }
        res_result, err = list_resources(proc)
        if err:
            results["[A3] resources/list: prismql://reference listed"] = f"FAIL: {err}"
        else:
            resource_uris = {r.get("uri", "") for r in res_result.get("resources", [])}
            missing_res = REQUIRED_RESOURCES - resource_uris
            extra_res = resource_uris - REQUIRED_RESOURCES
            if missing_res or extra_res:
                results["[A3] resources/list: prismql://reference listed"] = (
                    f"FAIL: exact-set mismatch — "
                    f"missing={sorted(missing_res) or 'none'}, "
                    f"extra={sorted(extra_res) or 'none'}; "
                    f"got: {sorted(resource_uris)}"
                )
            else:
                results["[A3] resources/list: prismql://reference listed"] = (
                    f"PASS: {len(resource_uris)} resources — exact set match "
                    f"({sorted(resource_uris)})"
                )

        # ── A4: prompts/list — exact 5-prompt set ────────────────────────────
        # prompts.rs registers 5 static prompts (query_tutorial, investigate_host,
        # triage_alerts, client_overview, cross_client_status). H13 exercises the 2 new ones.
        # MED-002 (F-AUD-P26): mirror A2's exact-set discipline — fail on missing OR extra.
        # CONSCIOUS-UPDATE REQUIRED: if prompts.rs is updated to add or remove a prompt,
        # update EXPECTED_PROMPTS in the same change.
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
            extra = prompt_names - EXPECTED_PROMPTS
            if missing or extra:
                results["[A4] prompts/list: all 5 prompts listed"] = (
                    f"FAIL: exact-set mismatch — "
                    f"missing={sorted(missing) or 'none'}, "
                    f"extra={sorted(extra) or 'none'}; "
                    f"got: {sorted(prompt_names)}"
                )
            else:
                results["[A4] prompts/list: all 5 prompts listed"] = (
                    f"PASS: {len(prompt_names)} prompts — exact set match "
                    f"(5 expected, 5 present); {sorted(prompt_names)}"
                )

        # F-AUD-P35-OBS-002: hoist list_capabilities(org-c) body so A6 can reuse it without
        # a redundant tool call (single fetch, shared by A5+A6; mirrors _describe_org_c_body
        # pattern used by A7/A9/A10). Initialized empty; set inside A5's no-err/no-error_code
        # branch. A6 falls back to re-call if A5 had an error or error_code.
        _listcaps_org_c_body: dict = {}

        # ── A5: list_capabilities: D-1312 MAJOR-001 client_registered=true ──
        body, err = tool_call(proc, "list_capabilities", {"client_id": "org-c"})
        if err:
            results["[A5] MAJOR-001: list_capabilities client_registered=true"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[A5] MAJOR-001: list_capabilities client_registered=true"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            # F-AUD-P35-OBS-002: cache positive-probe body for A6 reuse (no err, no error_code).
            _listcaps_org_c_body = body
            client_registered = body.get("client_registered", "MISSING")
            if client_registered is not True:
                results["[A5] MAJOR-001: list_capabilities client_registered=true"] = f"FAIL: client_registered={client_registered!r}"
            else:
                # MED-008: paired negative probe — unknown-but-well-formed client_id must return
                # client_registered: false (not an error). Grounded in list_capabilities handler
                # (server.rs list_capabilities): "unknown-but-well-formed client_id is NOT an error
                # — returns matrix with client_registered: false" (doc comment on the handler).
                body_neg, err_neg = tool_call(proc, "list_capabilities", {"client_id": "org-nonexistent-f-aud-p21"})
                if err_neg:
                    results["[A5] MAJOR-001: list_capabilities client_registered=true"] = (
                        f"FAIL: negative probe (org-nonexistent-f-aud-p21) returned transport error: {err_neg}"
                    )
                elif body_neg.get("error_code"):
                    results["[A5] MAJOR-001: list_capabilities client_registered=true"] = (
                        f"FAIL: negative probe (org-nonexistent-f-aud-p21) returned error code "
                        f"{body_neg['error_code']} — expected client_registered: false (not an error; "
                        f"server.rs list_capabilities contract)"
                    )
                else:
                    neg_registered = body_neg.get("client_registered", "MISSING")
                    if neg_registered is False:
                        results["[A5] MAJOR-001: list_capabilities client_registered=true"] = (
                            f"PASS: client_registered=true (org-c); "
                            f"negative probe org-nonexistent-f-aud-p21 → client_registered=false "
                            f"(server.rs list_capabilities contract confirmed)"
                        )
                    else:
                        results["[A5] MAJOR-001: list_capabilities client_registered=true"] = (
                            f"FAIL: negative probe (org-nonexistent-f-aud-p21) returned "
                            f"client_registered={neg_registered!r} (expected false for unknown client; "
                            f"server.rs list_capabilities contract)"
                        )

        # ── A6: list_capabilities: tri-state model fields (D-1162 BC-2.10.011) ─
        # BC-2.10.011 §single-client mode requires:
        #   capabilities: Map<String, {status: tri-state, resolution_chain: [...]}>
        #   not_registered_tools: [...] (renamed from not_implemented)
        # "enabled_count" is a cross-client summary field (null client_id mode) — it
        # does NOT appear in single-client mode entries and is irrelevant here.
        # F-AUD-P35-OBS-002: reuse A5 positive-probe body (single fetch, shared by A5+A6;
        # mirrors _describe_org_c_body pattern). Falls back to re-call if A5 had an error
        # or error_code, so A6 still yields a proper FAIL in those paths.
        if _listcaps_org_c_body:
            body, err = _listcaps_org_c_body, None
        else:
            body, err = tool_call(proc, "list_capabilities", {"client_id": "org-c"})
        if err:
            results["[A6] list_capabilities tri-state model fields"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[A6] list_capabilities tri-state model fields"] = f"FAIL: {body['error_code']}"
        else:
            VALID_STATUSES = {"enabled", "runtime_disabled", "compile_time_disabled"}
            caps = body.get("capabilities", "MISSING")
            not_reg = body.get("not_registered_tools", "MISSING")
            # OBS-002: scan the full serialized JSON payload — the old key may appear
            # nested inside capabilities entries or other sub-objects, not just at
            # top level.  BC-2.10.011 v1.5 renamed not_implemented → not_registered_tools;
            # any occurrence anywhere in the response is a regression.
            _body_json_str = json.dumps(body)
            has_old_field = '"not_implemented"' in _body_json_str

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
                    # F-AUD-P14-OBS-001: not_registered_tools must be >= 1 in both branches.
                    # server.rs NOT_YET_AVAILABLE_TOOLS has 40 entries on develop;
                    # an empty list means the field was not populated → render regression.
                    if not isinstance(not_reg, list) or len(not_reg) == 0:
                        results["[A6] list_capabilities tri-state model fields"] = (
                            f"FAIL: not_registered_tools is empty or not a list "
                            f"(type={type(not_reg).__name__}, len={len(not_reg) if isinstance(not_reg, list) else 'N/A'}) "
                            f"— render regression: server.rs NOT_YET_AVAILABLE_TOOLS has 40 entries; "
                            f"empty list indicates field was not populated"
                        )
                    else:
                        results["[A6] list_capabilities tri-state model fields"] = (
                            f"PASS: {len(caps)} capabilities; all have status+resolution_chain; "
                            f"statuses={statuses!r}; not_registered_tools count="
                            f"{len(not_reg)} "
                            f"(BC-2.10.011 tri-state model confirmed)"
                        )
            else:
                # Empty capabilities: correct for the AUDIT-COVERAGE-001 demo build.
                # server.rs list_capabilities per-client path:
                #   registry_paths = endpoint_registry.all_capability_paths() → empty
                #     (no [[write_endpoints]] declarations in any sensor TOML)
                #   client_paths = ff.capability_paths_for_client("org-c") → empty
                #     (no clients.org-c.capabilities section in prism-demo.toml)
                #   all_paths = registry_paths ∪ client_paths → empty set
                # Loop over all_paths is a no-op → capabilities = {} (server.rs all_paths iteration).
                # BC-2.10.011 does not require >= 1 capability entry; empty map is valid
                # when no write capability paths are configured at compile-time or runtime.
                # F-AUD-P14-OBS-001: not_registered_tools must be a non-empty list (>= 1).
                # server.rs NOT_YET_AVAILABLE_TOOLS has 40 entries on develop;
                # an empty list means the field was not populated → render regression.
                if not isinstance(not_reg, list):
                    results["[A6] list_capabilities tri-state model fields"] = (
                        f"FAIL: not_registered_tools is not a list; "
                        f"type={type(not_reg).__name__}"
                    )
                elif len(not_reg) == 0:
                    results["[A6] list_capabilities tri-state model fields"] = (
                        f"FAIL: not_registered_tools is empty (len=0) — render regression: "
                        f"server.rs NOT_YET_AVAILABLE_TOOLS has 40 entries; "
                        f"empty list indicates field was not populated "
                        f"(capabilities also empty; BC-2.10.011 single-client mode)"
                    )
                else:
                    results["[A6] list_capabilities tri-state model fields"] = (
                        f"PASS: capabilities empty (all_paths = registry_paths ∪ "
                        f"client_paths = {{}} — no write_endpoint declarations in sensor "
                        f"TOMLs, no capabilities in prism-demo.toml; "
                        f"server.rs list_capabilities); "
                        f"not_registered_tools={list(not_reg)[:3]!r} (len={len(not_reg)}); "
                        f"tri-state fields present (BC-2.10.011 single-client mode)"
                    )

        # OBS-001: initialize before A7 — A8 uses _describe_org_c_tables; A9/A10 use
        # _describe_org_c_body to avoid three redundant prism_describe(org-c) calls.
        _describe_org_c_tables = set()
        _describe_org_c_body: dict = {}

        # ── A7: AUDIT-001: prism_describe sensor-prefixed table names ────────
        body, err = tool_call(proc, "prism_describe", {"client_id": "org-c"}, timeout=15.0)
        if err:
            results["[A7] AUDIT-001: prism_describe sensor-prefixed names (org-c)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[A7] AUDIT-001: prism_describe sensor-prefixed names (org-c)"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
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
            # OBS-001: save full body so A9/A10 can reuse it without a second tool call
            _describe_org_c_body = body
            _describe_org_c_tables = table_names

        # ── A8: prism_describe org-c has all 5 core Stage-4 tables present (extended set covered by B11–B15) ─────────────
        REQUIRED_ORG_C_TABLES = {
            "crowdstrike_detections", "armis_devices", "claroty_devices",
            "claroty_audit_logs", "cyberint_alerts",
        }
        # Also accept extended table sets (crowdstrike_devices, crowdstrike_incidents, etc.)
        # F-AUD-P29-LOW-004: _describe_org_c_tables is unconditionally initialized at A7
        # above (module-level invariant); the NameError guard was dead code and would mask
        # a future initialization regression. Use plain assignment.
        present = set(_describe_org_c_tables)
        missing_tables = REQUIRED_ORG_C_TABLES - present
        if missing_tables:
            results["[A8] prism_describe org-c: all required tables present"] = f"FAIL: missing tables: {sorted(missing_tables)}; got: {sorted(present)}"
        else:
            results["[A8] prism_describe org-c: all required tables present"] = f"PASS: required tables present (total={len(present)}): {sorted(present)}"

        # ── A9: prism_describe org-c has pql_hints field ─────────────────────
        # OBS-001: reuse A7 describe response cache — eliminates redundant tool call
        if _describe_org_c_body:
            body_d, err_d = _describe_org_c_body, None
        else:
            body_d, err_d = tool_call(proc, "prism_describe", {"client_id": "org-c"}, timeout=15.0)
        if err_d:
            results["[A9] prism_describe org-c: pql_hints field present"] = f"FAIL: {err_d}"
        else:
            pql_hints = body_d.get("pql_hints", "MISSING")
            if pql_hints == "MISSING":
                results["[A9] prism_describe org-c: pql_hints field present"] = "FAIL: pql_hints field missing from response"
            elif isinstance(pql_hints, list) and len(pql_hints) > 0:
                # MED-001: assert all elements are non-empty strings. pql_hints of [""],
                # [None], or [{}] pass the len(>0) guard but are not usable by LLMs.
                # build_pql_hints() in prism_describe.rs always returns Vec<String>
                # with non-empty hint text; any empty/non-string element is a regression.
                _invalid_hints = [h for h in pql_hints if not (isinstance(h, str) and h.strip())]
                if _invalid_hints:
                    results["[A9] prism_describe org-c: pql_hints field present"] = (
                        f"FAIL: pql_hints has {len(_invalid_hints)} empty or non-string "
                        f"element(s) (build_pql_hints always returns non-empty strings); "
                        f"invalid={[repr(h) for h in _invalid_hints[:3]]}"
                    )
                else:
                    results["[A9] prism_describe org-c: pql_hints field present"] = f"PASS: {len(pql_hints)} pql_hints (all non-empty strings), first={str(pql_hints[0])[:80]!r}"
            else:
                # MED-001: pql_hints present but empty list or wrong type → FAIL
                # An empty list or non-list value means pql_hints is not usable by the LLM.
                results["[A9] prism_describe org-c: pql_hints field present"] = (
                    f"FAIL: pql_hints present but empty or wrong type — "
                    f"expected non-empty list, got {pql_hints!r} (type={type(pql_hints).__name__})"
                )

        # ── A10: prism_describe org-c: cyberint_alerts has iocs_value + iocs_value_first ─
        # S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (ADR-051 D4): added iocs_value_first as scalar
        # companion column so typed enrichment UDFs receive a plain string, not a JSON list.
        # OBS-001: reuse A7 describe response cache — eliminates redundant tool call
        if _describe_org_c_body:
            body_dc, err_dc = _describe_org_c_body, None
        else:
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
        elif body_oa.get("error_code"):
            results["[A11] prism_describe org-a: no cyberint/claroty tables"] = f"FAIL: {body_oa['error_code']}: {body_oa.get('message','')[:80]}"
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
                # OBS-001: section_count gate tightened to exact equality (not a floor).
                # build_reference_content() in crates/prism-mcp/src/resources.rs emits
                # exactly 7 ## headings: "What is PrismQL", "Clause Grammar (BNF)",
                # "Operators and Types", "Datetime Arithmetic", "Error Code Quick-Reference",
                # "Query Examples", "Self-Correction Workflow".  Drift in EITHER direction
                # (truncation < 7 or undeclared addition > 7) must reconcile with the spec.
                if section_count != 7:
                    results["[A12] N1: prismql://reference per-field UDF names + content"] = (
                        f"FAIL: reference content has {section_count} ## sections "
                        f"(expected exactly 7; source: build_reference_content() in "
                        f"resources.rs emits exactly 7 ## sections: 'What is PrismQL', "
                        f"'Clause Grammar (BNF)', 'Operators and Types', "
                        f"'Datetime Arithmetic', 'Error Code Quick-Reference', "
                        f"'Query Examples', 'Self-Correction Workflow') — "
                        f"{'truncation' if section_count < 7 else 'section addition'} regression"
                    )
                else:
                    results["[A12] N1: prismql://reference per-field UDF names + content"] = f"PASS: threat_score+cvss_base_score present; {section_count} sections (== 7); no old forms"
            elif has_old_infusion_form:
                results["[A12] N1: prismql://reference per-field UDF names + content"] = "FAIL: old infusion_id call forms still present"
            else:
                # PARTIAL sweep: both threat_score and cvss_base_score are required by N1
                # spec; absence of either is a definitive FAIL (not ambiguous).
                results["[A12] N1: prismql://reference per-field UDF names + content"] = (
                    f"FAIL: required UDF names missing — threat_score={has_threat_score}, "
                    f"cvss_base_score={has_cvss}; sections={section_count}"
                )

        # ── A13: N1-B: unknown enrich UDF returns E-QUERY-039 ────────────────
        body, err = query(proc, "FROM armis_devices | enrich nonexistent_udf(device_id) | limit 3", ["org-c"])
        if err:
            results["[A13] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            msg = body.get("message", "")
            if error_code == "E-QUERY-039" or ("E-QUERY-039" in msg):
                # F-AUD-P7-LOW-004 (POL-24): same anchor as F4 — E-QUERY-039 Display.
                has_anchor = "is not registered; available: [" in msg
                if has_anchor:
                    results["[A13] N1-B: unknown enrich UDF -> E-QUERY-039"] = (
                        f"PASS: E-QUERY-039 + anchor 'is not registered; available: [' confirmed; "
                        f"message={msg[:80]!r}"
                    )
                else:
                    results["[A13] N1-B: unknown enrich UDF -> E-QUERY-039"] = (
                        f"FAIL: E-QUERY-039 but message-template anchor "
                        f"'is not registered; available: [' absent — "
                        f"message-template regression (POL-24); message={msg[:80]!r}"
                    )
            else:
                results["[A13] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: got {error_code or 'no error'}: {msg[:80]}"

        # ── A14: DataFusion builtin NOT E-QUERY-039 (COUNT) ─────────────────
        # NOTE (F-AUD-P1-OBS-001): also appears as C4 and F6 — intentional cross-section
        # regression coverage (MCP Protocol, Query Modes, Error Taxonomy sections).
        # F-AUD-P29-LOW-002: strict-success semantics aligned with C4 — any error_code
        # (not just E-QUERY-039) is FAIL; only no error_code with rows is PASS.
        body, err = query(proc, "SELECT COUNT(*) FROM armis_devices", ["org-c"])
        if err:
            results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-039":
                results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = "FAIL: E-QUERY-039 falsely fired for COUNT(*)"
            elif error_code:
                results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"FAIL: unexpected error {error_code}: {body.get('message', '')[:80]}"
            else:
                rows = body.get("rows", [])
                results["[A14] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"PASS: COUNT executed OK, {len(rows)} rows"

        # ── A15: N2: dot-notation FROM returns E-QUERY-037 ───────────────────
        body, err = query(proc, "FROM crowdstrike.detections | limit 3", ["org-c"])
        if err:
            results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            msg = body.get("message", "")
            if error_code == "E-QUERY-037":
                # F-AUD-P7-LOW-004 (POL-24): same two-anchor check as F3 — E-QUERY-037
                # message_template has BOTH segments:
                #   "... Available sensors: [{available_sensors}]. Available tables: [{available_tables}]..."
                # LOW-002 (F-AUD-P26): require both 'Available sensors:' AND 'Available tables:'.
                has_sensor_anchor = "Available sensors:" in msg
                has_table_anchor = "Available tables:" in msg
                if has_sensor_anchor and has_table_anchor:
                    results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = (
                        f"PASS: E-QUERY-037 + both anchors confirmed "
                        f"('Available sensors:' + 'Available tables:'); "
                        f"message={msg[:80]!r}"
                    )
                else:
                    missing_anchors = []
                    if not has_sensor_anchor:
                        missing_anchors.append("'Available sensors:'")
                    if not has_table_anchor:
                        missing_anchors.append("'Available tables:'")
                    results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = (
                        f"FAIL: E-QUERY-037 but message-template anchor(s) "
                        f"{', '.join(missing_anchors)} absent — regression (POL-24); "
                        f"message={msg[:80]!r}"
                    )
            elif not error_code and body.get("rows") is not None:
                results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = "FAIL: returned rows silently (no error)"
            else:
                results["[A15] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: got {error_code or 'no error'}: {msg[:80]}"

        # ── A16: AUDIT-004: triage_alerts prompt uses FROM-ready names ────────
        res, err = prompt_get(proc, "triage_alerts", {"client_id": "org-c"})
        if err:
            results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = f"FAIL: {err}"
        else:
            msgs = res.get("messages", [])
            body_text = msgs[0].get("content", {}).get("text", "") if msgs else ""
            # F-AUD-P4-LOW-001: replace ambiguous has_dot/has_underscore sweep and dead
            # PARTIAL branch with explicit anchor checks against the shipped prompt body
            # (prompts.rs render_triage_alerts).
            # The shipped prompt references exactly 3 sensor-prefixed FROMs:
            #   "FROM crowdstrike_detections", "FROM claroty_alerts", "FROM armis_alerts"
            # cyberint is NOT in the prompt body — do not require it (anchor to reality).
            # Dot-notation check remains: underscore anchors + no-dot is the full assertion.
            # LOW-004: these anchors appear as positive-example queries in the prompt body,
            # not as structural markers or section headings. The prompt contains literal
            # example queries the LLM can copy directly. Asserting their presence proves
            # FROM-underscore names (not dot-notation) appear in the rendered examples.
            required_anchors = [
                "FROM crowdstrike_detections",
                "FROM claroty_alerts",
                "FROM armis_alerts",
            ]
            missing_anchors = [a for a in required_anchors if a not in body_text]
            has_dot = any(f"FROM {s}." in body_text for s in ["crowdstrike", "claroty", "armis"])
            if has_dot:
                results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = (
                    "FAIL: dot-notation still in prompt (underscore regression)"
                )
            elif missing_anchors:
                results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = (
                    f"FAIL: missing FROM-underscore anchors in triage_alerts prompt "
                    f"(template regression): {missing_anchors!r}; "
                    f"body preview={body_text[:150]!r}"
                )
            else:
                results["[A16] AUDIT-004: triage_alerts prompt underscore names"] = (
                    f"PASS: all {len(required_anchors)} FROM-underscore anchors present "
                    f"(crowdstrike_detections, claroty_alerts, armis_alerts); no dot-notation"
                )

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
            if len(msgs) >= 1:
                results["[A17] HANG-FIX: query_tutorial returns promptly"] = f"PASS: {elapsed:.2f}s, {len(msgs)} message(s)"
            else:
                results["[A17] HANG-FIX: query_tutorial returns promptly"] = f"FAIL: prompt returned no messages ({len(msgs)} messages)"

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
            if len(msgs) >= 1:
                results["[A18] HANG-FIX: investigate_host returns promptly"] = f"PASS: {elapsed:.2f}s, {len(msgs)} message(s)"
            else:
                results["[A18] HANG-FIX: investigate_host returns promptly"] = f"FAIL: prompt returned no messages ({len(msgs)} messages)"

        # ── A19: list_infusions NYA -32003 promptly ───────────────────────────
        t0 = time.time()
        rid = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid, "method": "tools/call",
                        "params": {"name": "list_infusions", "arguments": {}}})
        resp, err = read_msg(proc, timeout=5.0, expected_id=rid)
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
        resp, err = read_msg(proc, timeout=5.0, expected_id=rid)
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
        resp, err = read_msg(proc, timeout=5.0, expected_id=rid)
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

        # ── A23: all NYA stubs return -32003/-32602 (dynamic sweep; direct -32003 handler-gate assurance via A19/A20/A21) ─────────
        # OBS-002 (F-AUD-P28): A23 runs BEFORE A22 (check_sensor_health) by deliberate
        # design.  A23 derives the NYA stub set as (tools/list names − EXPECTED_TOOLS_FULL),
        # which is a read-only enumeration probe.  A22 calls check_sensor_health, which
        # exercises the live sensor health path and may mutate probe-state caches.  Running
        # the read-only NYA sweep (A23) before the cache-mutating health probe (A22) keeps
        # the NYA response character stable and prevents A22's side-effects from masking an
        # NYA regression.  Do NOT reorder A23 and A22.
        # F-AUD-P13-OBS-004: Replace 3/40 sampling gap with full dynamic coverage.
        # Derives stub set at runtime: (tools/list names) − (EXPECTED_TOOLS_FULL 14 implemented).
        # OBS-001 cross-reference ← A2: A2 gates full 54-tool membership (14 live + 40 NYA);
        # this derivation (tool_names − EXPECTED_TOOLS_FULL) is consistent with A2 — when A2
        # passes it yields exactly EXPECTED_TOOLS_NYA (40 names).  If tool registration
        # changes, update EXPECTED_TOOLS_NYA in A2 and re-verify the A23 derivation count.
        # Each stub is called once with minimal empty args {}.
        # Acceptable outcomes:
        #   -32003: explicit E-INFRA-NYA (spec-correct NYA response)
        #   -32602: schema param validation fires before handler body (acceptable NYA-equivalent;
        #           stubs whose Param structs have non-optional required fields fail serde
        #           deserialization before the handler runs — this IS validation preceding the
        #           NYA gate per BC-2.10.017 INV-NOT-YET-AVAILABLE-GUARD-ORDER)
        # Deviant outcomes (→ FAIL): success response, any other error code, timeout.
        # Named representatives A19/A20/A21 remain as-is (response-time bounds unchanged).
        # F-AUD-P15-LOW-004: A19 uses {} args for list_infusions and requires strict -32003
        # (not -32602). This is correct: ListInfusionsParams has pub client_id: Option<String>
        # (all fields optional, #[serde(deny_unknown_fields)]); {} deserializes to
        # ListInfusionsParams { client_id: None } without error → handler body runs →
        # not_yet_available_msg fires → -32003. A23 accepts -32602 for the full stub set
        # because some stubs (e.g. InfusionStatusParams.infusion_id: String — required) return
        # -32602 via serde before the handler runs. For list_infusions specifically, {} is a
        # valid param shape that reaches the NYA gate directly.
        # F-AUD-P30-LOW-001: guard against nameless registered tools before deriving the NYA
        # sweep set.  A nameless tool (`name` missing or empty string) is a registration defect
        # that must never reach the NYA sweep — it would contribute "" to the set-difference
        # and appear as an -32602 "compliant" result, masking the defect.
        _all_tool_entries = (tools_result or {}).get("tools", []) if tools_result else []
        _nameless_tools = [t for t in _all_tool_entries if not t.get("name")]
        if _nameless_tools:
            results["[A23] all NYA stubs return -32003/-32602 (dynamic sweep; direct -32003 handler-gate assurance via A19/A20/A21)"] = (
                f"FAIL: {len(_nameless_tools)} registered tool(s) with missing/empty name — "
                f"registration defect; cannot proceed with NYA sweep (F-AUD-P30-LOW-001)"
            )
        else:
            _nya_stub_names = sorted(
                {t.get("name") for t in _all_tool_entries if t.get("name")} - EXPECTED_TOOLS_FULL
            ) if tools_result else []
            _nya_deviants = []   # list of (name, outcome_str) for non-NYA results
            _nya_pass_count = 0
            _nya_total = len(_nya_stub_names)
            for _nya_name in _nya_stub_names:
                _rid = next_id()
                send_msg(proc, {"jsonrpc": "2.0", "id": _rid, "method": "tools/call",
                                "params": {"name": _nya_name, "arguments": {}}})
                _resp, _err = read_msg(proc, timeout=5.0, expected_id=_rid)
                if _err:
                    _nya_deviants.append((_nya_name, f"timeout/error: {_err}"))
                elif "error" in _resp:
                    _code = _resp["error"].get("code", "?")
                    _emsg = _resp["error"].get("message", "")
                    if _code in (-32003, -32602):
                        # -32003 = explicit NYA gate (E-INFRA-NYA per server.rs not_yet_available_msg).
                        # -32602 = JSON schema param validation fires before the handler body;
                        #   stubs whose Param structs have non-optional required fields fail
                        #   serde deserialization before the handler runs. This confirms
                        #   unreachability-via-empty-args — the server never reached the handler
                        #   body where not_yet_available_msg would fire. This is an acceptable
                        #   NYA-equivalent under BC-2.10.017 INV-NOT-YET-AVAILABLE-GUARD-ORDER.
                        #   NOTE: -32602 does NOT provide direct -32003 coverage of the NYA gate
                        #   itself. Direct -32003 positive coverage is provided by the named
                        #   representatives A19 (list_infusions), A20 (plugin_status), and
                        #   A21 (infusion_status) which are called with valid param shapes.
                        _nya_pass_count += 1
                    else:
                        _nya_deviants.append((_nya_name, f"code={_code}, msg={_emsg[:60]!r}"))
                else:
                    # Success response — stub returned data; NYA contract violated
                    _nya_deviants.append((_nya_name, "SUCCESS (expected -32003 E-INFRA-NYA)"))
            if not _nya_stub_names:
                results["[A23] all NYA stubs return -32003/-32602 (dynamic sweep; direct -32003 handler-gate assurance via A19/A20/A21)"] = (
                    "FAIL: could not derive NYA stub set (tools/list unavailable or empty)"
                )
            elif _nya_deviants:
                results["[A23] all NYA stubs return -32003/-32602 (dynamic sweep; direct -32003 handler-gate assurance via A19/A20/A21)"] = (
                    f"FAIL: {len(_nya_deviants)}/{_nya_total} stubs deviated from NYA contract "
                    f"(-32003 expected): "
                    + "; ".join(f"{n}={o}" for n, o in _nya_deviants[:5])
                )
            else:
                results["[A23] all NYA stubs return -32003/-32602 (dynamic sweep; direct -32003 handler-gate assurance via A19/A20/A21)"] = (
                    f"PASS: {_nya_pass_count}/{_nya_total} stubs NYA-compliant "
                    f"(-32003 explicit or -32602 schema-validation-precedes-NYA-gate)"
                )

        # ── A22: check_sensor_health (S-5.04; if available) ─────────────────
        # NOTE: check_sensor_health returns raw JSON text (not under "results" envelope).
        # The "sensors" field is a LIST (not a dict keyed by sensor_id).
        t0 = time.time()
        rid_csh = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid_csh, "method": "tools/call",
                        "params": {"name": "check_sensor_health", "arguments": {"client_id": "org-c"}}})
        resp_csh, err_csh = read_msg(proc, timeout=15.0, expected_id=rid_csh)
        elapsed = time.time() - t0
        if err_csh:
            results[_A22_RESULT_KEY] = f"FAIL: {err_csh}"
        elif resp_csh and "error" in resp_csh:
            code = resp_csh["error"].get("code", "?")
            if code == -32601:
                # OBS-001 fix: check_sensor_health IS registered in LIVE_TOOLS (server.rs);
                # -32601 means the tool is not found → S-5.04 regression, not N/A.
                results[_A22_RESULT_KEY] = (
                    "FAIL: check_sensor_health tool missing — S-5.04 regression "
                    "(check_sensor_health IS registered in LIVE_TOOLS; "
                    "-32601 = method not found)"
                )
            else:
                results[_A22_RESULT_KEY] = f"FAIL: MCP error {code}: {resp_csh['error'].get('message','')[:80]}"
        else:
            # check_sensor_health returns raw JSON text in content[0].text
            content = resp_csh.get("result", {}).get("content", [])
            text = content[0].get("text", "") if content else ""
            if not text:
                results[_A22_RESULT_KEY] = "FAIL: empty response"
            elif text.startswith("ERROR:"):
                results[_A22_RESULT_KEY] = f"FAIL: {text[:120]}"
            else:
                try:
                    csh_body = json.loads(text)
                    # F-AUD-P16-MED-001: dict guard — non-dict JSON (list/int/str) would
                    # raise AttributeError on .get() below; report FAIL with diagnostic
                    # (mirrors H14b guard pattern).
                    if not isinstance(csh_body, dict):
                        results[_A22_RESULT_KEY] = (
                            f"FAIL: non-dict JSON in check_sensor_health response: "
                            f"{type(csh_body).__name__}: {text[:80]!r}"
                        )
                    else:
                        overall = csh_body.get("overall_status", "?")
                        _overall_key_present = "overall_status" in csh_body
                        sensors = csh_body.get("sensors", [])
                        # OBS-005: refactored from dense walrus set comprehension for readability.
                        # Coerce None probe_level → "<missing>" so sorted() never compares
                        # None with str (TypeError crash in the FAIL branch).
                        _probe_level_set: set = set()
                        for _s in sensors:
                            if isinstance(_s, dict):
                                _pl = _s.get("probe_level")
                                _probe_level_set.add(_pl if _pl is not None else "<missing>")
                        probe_levels = list(_probe_level_set)
                        reachable_all = all(s.get("reachable") is True for s in sensors if isinstance(s, dict))
                        auth_valid_all = all(s.get("auth_valid") is True for s in sensors if isinstance(s, dict))
                        # F-AUD-P8-OBS-002: require all probe levels == "live" (defense-in-depth vs
                        # S-5.03 hardcoded-Some(true) regression per BC-2.08.005; runbook Act 5 requires
                        # probe_level "live").
                        probe_level_live = all(p == "live" for p in probe_levels)
                        sensor_ids = [s.get("sensor_id") for s in sensors if isinstance(s, dict)]
                        # F-AUD-P2-HIGH-003: assert expected sensor set present (not vacuous all([])).
                        # Sensor ID values verified against crates/prism-sensors/specs/*.sensor.toml
                        # (crowdstrike.sensor.toml, armis.sensor.toml, claroty.sensor.toml,
                        # cyberint.sensor.toml) — these are the four registered sensors for org-c.
                        # MED-005 (F-AUD-P25, adjudicated: comment-only, no logic change):
                        # POL-24 conscious coupling accepted — the 4-sensor set is defined by
                        # crates/prism-sensors/specs/*.sensor.toml entries registered for org-c
                        # (demo config: .prism/config.toml / demo-config.toml sensor_type assignments).
                        # MED-003 (F-AUD-P26): mirror A2/A4 exact-set discipline — fail on extra
                        # sensors as well as missing, so unexpected sensor registrations surface.
                        # F-AUD-P30-MED-003: EXPECTED_SENSORS now defined at module level (single
                        # source of truth shared with H14b); local assignment removed.
                        present_sensors = set(sid for sid in sensor_ids if sid)
                        missing_sensors = EXPECTED_SENSORS - present_sensors
                        extra_sensors = present_sensors - EXPECTED_SENSORS
                        if missing_sensors or extra_sensors:
                            results[_A22_RESULT_KEY] = (
                                f"FAIL: {elapsed:.1f}s exact-set mismatch — "
                                f"missing={sorted(missing_sensors) or 'none'}, "
                                f"extra={sorted(extra_sensors) or 'none'}; "
                                f"got sensor_ids={sorted(present_sensors)}"
                            )
                        elif overall == "healthy" and reachable_all and auth_valid_all and probe_level_live:
                            results[_A22_RESULT_KEY] = (
                                f"PASS: {elapsed:.1f}s overall={overall}; probe_levels={probe_levels}; "
                                f"sensors={sorted(present_sensors)}; reachable_all={reachable_all}; auth_valid_all={auth_valid_all}"
                            )
                        elif overall == "healthy" and reachable_all and auth_valid_all and not probe_level_live:
                            # Non-live probe levels mean actual sensor calls were not exercised —
                            # BC-2.08.005 / runbook Act 5 requires probe_level "live" for demo preflight.
                            non_live = sorted(set(p for p in probe_levels if p != "live"))
                            results[_A22_RESULT_KEY] = (
                                f"FAIL: {elapsed:.1f}s overall={overall} but non-live probe levels detected "
                                f"(BC-2.08.005 / runbook Act 5 requires probe_level 'live'); "
                                f"non_live_probe_levels={non_live!r}; sensors={sorted(present_sensors)}"
                            )
                        elif overall == "healthy" and not reachable_all:
                            # F-AUD-P19-LOW-002: differentiate FAIL by failed predicate.
                            # overall=healthy but reachable_all is False — sensor(s) unreachable.
                            unreachable = [s.get("sensor_id") for s in sensors
                                           if isinstance(s, dict) and not s.get("reachable")]
                            results[_A22_RESULT_KEY] = (
                                f"FAIL: {elapsed:.1f}s overall={overall} but reachable_all=False "
                                f"(not all sensors reachable — demo preflight requires full sensor reachability); "
                                f"unreachable_sensors={unreachable!r}; auth_valid_all={auth_valid_all}"
                            )
                        elif overall == "healthy" and not auth_valid_all:
                            # F-AUD-P19-LOW-002: overall=healthy but auth_valid_all is False.
                            auth_failed = [s.get("sensor_id") for s in sensors
                                           if isinstance(s, dict) and not s.get("auth_valid")]
                            results[_A22_RESULT_KEY] = (
                                f"FAIL: {elapsed:.1f}s overall={overall} but auth_valid_all=False "
                                f"(not all sensor credentials valid — demo preflight requires valid auth); "
                                f"auth_failed_sensors={auth_failed!r}; reachable_all={reachable_all}"
                            )
                        elif overall not in ("healthy", "?"):
                            # overall is degraded, failing, or other non-healthy non-unknown value.
                            # Demo preflight requires all sensors healthy (F-AUD-P1-LOW-004).
                            results[_A22_RESULT_KEY] = (
                                f"FAIL: {elapsed:.1f}s overall={overall} (degraded/failing not acceptable for demo preflight); "
                                f"sensors={sorted(present_sensors)}; reachable_all={reachable_all}; auth_valid_all={auth_valid_all}"
                            )
                        else:
                            # OBS-004: distinguish missing overall_status key from literal "?" value.
                            # csh_body.get("overall_status", "?") maps BOTH to "?" — separate them.
                            if not _overall_key_present:
                                results[_A22_RESULT_KEY] = (
                                    f"FAIL: {elapsed:.1f}s overall_status key absent from response "
                                    f"(expected 'healthy'); keys={list(csh_body.keys())[:8]!r}"
                                )
                            else:
                                results[_A22_RESULT_KEY] = (
                                    f"FAIL: {elapsed:.1f}s overall_status={csh_body['overall_status']!r} "
                                    f"(unexpected literal '?' value); expected 'healthy'; "
                                    f"body={text[:200]}"
                                )
                except json.JSONDecodeError as e:
                    results[_A22_RESULT_KEY] = f"FAIL: JSON parse error: {e}; raw={text[:100]!r}"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION B: 4 Sensor Adapters × Their Tables (CrowdStrike, Armis, Claroty, Cyberint)
        # ═══════════════════════════════════════════════════════════════════════

        # ── B1: CrowdStrike detections org-c (OAuth) ──────────────────────────
        body, err = query(proc, "FROM crowdstrike_detections | limit 3", ["org-c"])
        if err:
            results["[B1] CS org-c: crowdstrike_detections returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B1] CS org-c: crowdstrike_detections returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B1] CS org-c: crowdstrike_detections returns data", body, results):
                pass
            elif not rows:
                results["[B1] CS org-c: crowdstrike_detections returns data"] = (
                    "FAIL: 0 rows — Stage 1+ must have crowdstrike_detections data (Stage 4 terminal guarantee)"
                )
            else:
                results["[B1] CS org-c: crowdstrike_detections returns data"] = f"PASS: {len(rows)} rows"

        # ── B2: Armis devices org-c ───────────────────────────────────────────
        body, err = query(proc, "FROM armis_devices\n| where device_id IS NOT NULL\n| limit 3", ["org-c"])
        if err:
            results["[B2] Armis org-c: armis_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B2] Armis org-c: armis_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B2] Armis org-c: armis_devices returns data", body, results):
                pass
            elif not rows:
                results["[B2] Armis org-c: armis_devices returns data"] = (
                    "FAIL: 0 rows — Stage 4 terminal guarantee requires armis_devices data"
                )
            else:
                results["[B2] Armis org-c: armis_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B3: Claroty devices org-c ─────────────────────────────────────────
        body, err = query(proc, "FROM claroty_devices | limit 3", ["org-c"])
        if err:
            results["[B3] Claroty org-c: claroty_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B3] Claroty org-c: claroty_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B3] Claroty org-c: claroty_devices returns data", body, results):
                pass
            elif not rows:
                results["[B3] Claroty org-c: claroty_devices returns data"] = (
                    "FAIL: 0 rows — Stage 4 terminal guarantee requires claroty_devices data"
                )
            else:
                results["[B3] Claroty org-c: claroty_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B4: Claroty audit_logs org-c ──────────────────────────────────────
        # F-AUD-P5-LOW-004: query with limit 10 (> fixture size) and assert exactly 5 rows.
        # The static 5-row fixture (crates/prism-dtu-claroty/fixtures/audit-log.json,
        # gap-analysis §4 guardrail #6) always returns exactly 5 rows regardless of limit.
        # Querying with limit 10 and asserting exactly 5 verifies both:
        #   (a) the fixture is returning data, and
        #   (b) the fixture has not unexpectedly grown (which would signal a config error).
        body, err = query(proc, "FROM claroty_audit_logs | limit 10", ["org-c"])
        if err:
            results["[B4] Claroty org-c: claroty_audit_logs returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B4] Claroty org-c: claroty_audit_logs returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B4] Claroty org-c: claroty_audit_logs returns data", body, results):
                pass
            elif len(rows) == 5:
                col_names = list(rows[0].keys()) if rows else []
                results["[B4] Claroty org-c: claroty_audit_logs returns data"] = (
                    f"PASS: exactly 5 rows (static fixture confirmed; gap-analysis §4 guardrail #6); "
                    f"sample cols={col_names[:6]}"
                )
            elif rows:
                results["[B4] Claroty org-c: claroty_audit_logs returns data"] = (
                    f"FAIL: {len(rows)} rows (expected exactly 5 from static fixture, "
                    f"crates/prism-dtu-claroty/fixtures/audit-log.json; "
                    f"gap-analysis §4 guardrail #6)"
                )
            else:
                # F-AUD-P2-HIGH-001: static 5-row fixture must always return data
                results["[B4] Claroty org-c: claroty_audit_logs returns data"] = "FAIL: 0 rows (static 5-row fixture must always return data; gap-analysis §4 guardrail #6)"

        # ── B5: Cyberint alerts org-c ─────────────────────────────────────────
        body, err = query(proc, "FROM cyberint_alerts | limit 3", ["org-c"])
        if err:
            results["[B5] Cyberint org-c: cyberint_alerts returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B5] Cyberint org-c: cyberint_alerts returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B5] Cyberint org-c: cyberint_alerts returns data", body, results):
                pass
            elif not rows:
                results["[B5] Cyberint org-c: cyberint_alerts returns data"] = (
                    "FAIL: 0 rows — Stage 4 terminal guarantee requires cyberint_alerts data"
                )
            else:
                results["[B5] Cyberint org-c: cyberint_alerts returns data"] = f"PASS: {len(rows)} rows"

        # ── B6: Claroty devices org-b ─────────────────────────────────────────
        body, err = query(proc, "FROM claroty_devices | limit 3", ["org-b"])
        if err:
            results["[B6] Claroty org-b: claroty_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B6] Claroty org-b: claroty_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B6] Claroty org-b: claroty_devices returns data", body, results):
                pass
            elif not rows:
                results["[B6] Claroty org-b: claroty_devices returns data"] = (
                    "FAIL: 0 rows — org-b must have claroty_devices data"
                )
            else:
                results["[B6] Claroty org-b: claroty_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B7: Cyberint alerts org-b ─────────────────────────────────────────
        body, err = query(proc, "FROM cyberint_alerts | limit 3", ["org-b"])
        if err:
            results["[B7] Cyberint org-b: cyberint_alerts returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B7] Cyberint org-b: cyberint_alerts returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B7] Cyberint org-b: cyberint_alerts returns data", body, results):
                pass
            elif not rows:
                results["[B7] Cyberint org-b: cyberint_alerts returns data"] = (
                    "FAIL: 0 rows — org-b must have cyberint_alerts data"
                )
            else:
                results["[B7] Cyberint org-b: cyberint_alerts returns data"] = f"PASS: {len(rows)} rows"

        # ── B8: CrowdStrike detections org-a ──────────────────────────────────
        body, err = query(proc, "FROM crowdstrike_detections | limit 3", ["org-a"])
        if err:
            results["[B8] CS org-a: crowdstrike_detections returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B8] CS org-a: crowdstrike_detections returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B8] CS org-a: crowdstrike_detections returns data", body, results):
                pass
            elif not rows:
                results["[B8] CS org-a: crowdstrike_detections returns data"] = (
                    "FAIL: 0 rows — org-a must have crowdstrike_detections data"
                )
            else:
                results["[B8] CS org-a: crowdstrike_detections returns data"] = f"PASS: {len(rows)} rows"

        # ── B9: Armis devices org-a ───────────────────────────────────────────
        body, err = query(proc, "FROM armis_devices | limit 3", ["org-a"])
        if err:
            results["[B9] Armis org-a: armis_devices returns data"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[B9] Armis org-a: armis_devices returns data"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[B9] Armis org-a: armis_devices returns data", body, results):
                pass
            elif not rows:
                results["[B9] Armis org-a: armis_devices returns data"] = (
                    "FAIL: 0 rows — org-a must have armis_devices data"
                )
            else:
                results["[B9] Armis org-a: armis_devices returns data"] = f"PASS: {len(rows)} rows"

        # ── B11–B15: Additional tables (org-c full 10-table matrix) ─────────────
        # Sequential IDs [B11]..[B15] in stable iteration order (F-AUD-P1-LOW-002).
        # F-AUD-P2-MED-007: B14 (crowdstrike_incidents) and B15 (cyberint_incidents)
        # have no DTU routes (gap-analysis §4 guardrail #1); 0 rows is the EXPECTED
        # outcome and is asserted explicitly. Any error_code on these tables = FAIL.
        # MED-004 (F-AUD-P25): NO_ROUTE_TABLES and _DATA_GUARANTEED co-located so the
        # disjointness assertion fires at start (not buried in the loop body).
        # CONSCIOUS-UPDATE (cite F-AUD-P25-MED-004): any DTU route addition or removal
        # must update BOTH sets in the same change — add a new DTU route → move the
        # table from NO_ROUTE_TABLES into _DATA_GUARANTEED (or vice versa on removal).
        NO_ROUTE_TABLES = {"crowdstrike_incidents", "cyberint_incidents"}
        _DATA_GUARANTEED = {"armis_alerts", "claroty_alerts", "crowdstrike_devices"}
        # MED-004: self-consistency assertion — the two sets must be disjoint.
        # A table cannot simultaneously have no DTU route AND be data-guaranteed.
        _med004_overlap = NO_ROUTE_TABLES & _DATA_GUARANTEED
        if _med004_overlap:
            raise AssertionError(
                f"MED-004 (F-AUD-P25): NO_ROUTE_TABLES and _DATA_GUARANTEED are NOT disjoint — "
                f"overlap={sorted(_med004_overlap)!r}. "
                f"Fix by removing the table from one set before running the audit."
            )
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
                    # F-AUD-P10-MED-004: data-guaranteed routed tables must return >= 1 row.
                    # Stage 4 is terminal/absorbing for org-c: "org-c seed 200, all 4 sensors,
                    # Stage 4 guaranteed since scenario_start_secs = 1782214754 is in the past"
                    # (gap-analysis §3). armis_alerts, claroty_alerts, crowdstrike_devices all
                    # have active DTU routes and data at Stage 4 (verified: gap-analysis §4.1
                    # lists only crowdstrike_incidents + cyberint_incidents as no-DTU-route).
                    if sensor_errors:
                        results[key] = (
                            f"FAIL: partial fan-out failure for table {tbl} — "
                            f"sensor_errors={sensor_errors[:2]} (F-AUD-P30-MED-001)"
                        )
                    elif tbl in _DATA_GUARANTEED and len(rows_t) == 0:
                        results[key] = (
                            f"FAIL: 0 rows for data-guaranteed table {tbl} — "
                            f"silent-empty (Standing Rule 3 §2 class); "
                            f"Stage 4 terminal guarantee requires >= 1 row (gap-analysis §3)"
                        )
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
            sensor_errors_a = body_a.get("sensor_errors", [])
            sensor_errors_c = body_c.get("sensor_errors", [])
            ids_a = {r.get("device_id") for r in body_a.get("rows", []) if r.get("device_id")}
            ids_c = {r.get("device_id") for r in body_c.get("rows", []) if r.get("device_id")}
            overlap = ids_a & ids_c
            if sensor_errors_a or sensor_errors_c:
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = (
                    f"FAIL: partial fan-out failure — org-a errors={sensor_errors_a[:2]}, "
                    f"org-c errors={sensor_errors_c[:2]} (F-AUD-P30-MED-001)"
                )
            elif not ids_a or not ids_c:
                # F-AUD-P2-HIGH-001: 0 rows means DTU is not returning data; isolation
                # cannot be proven → FAIL, not WARN (silent pass-through is dangerous).
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = (
                    f"FAIL: insufficient data — org-a={len(ids_a)} IDs, org-c={len(ids_c)} IDs "
                    f"(cannot prove disjoint with 0 rows from one or both orgs)"
                )
            elif overlap:
                # Isolation broken: IDs appear in BOTH orgs — multi-tenant boundary violated.
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = (
                    f"FAIL: ISOLATION BROKEN — {len(overlap)} overlapping device_id(s); "
                    f"sample={sorted(overlap)[:3]!r}"
                )
            else:
                results["[B10] ISOLATION: org-a vs org-c CS device IDs disjoint"] = (
                    f"PASS: zero overlap; org-a={len(ids_a)} IDs, org-c={len(ids_c)} IDs "
                    f"(positive counts; isolation proven)"
                )

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
            if sensor_errors_gate("[C1] SQL SELECT mode: SELECT FROM WHERE LIMIT", body, results):
                pass
            elif not rows:
                results["[C1] SQL SELECT mode: SELECT FROM WHERE LIMIT"] = (
                    "FAIL: 0 rows — crowdstrike_detections must have data at Stage 1+"
                )
            else:
                results["[C1] SQL SELECT mode: SELECT FROM WHERE LIMIT"] = f"PASS: {len(rows)} rows"

        # ── C2: Pipe mode ─────────────────────────────────────────────────────
        body, err = query(proc, "FROM armis_devices\n| where device_id IS NOT NULL\n| limit 5", ["org-c"])
        if err:
            results["[C2] Pipe mode: FROM | where | limit"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C2] Pipe mode: FROM | where | limit"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[C2] Pipe mode: FROM | where | limit", body, results):
                pass
            elif not rows:
                results["[C2] Pipe mode: FROM | where | limit"] = (
                    "FAIL: 0 rows — armis_devices must have data at Stage 1+"
                )
            else:
                results["[C2] Pipe mode: FROM | where | limit"] = f"PASS: {len(rows)} rows"

        # ── C3: Pipe fields projection ────────────────────────────────────────
        body, err = query(proc, "FROM crowdstrike_detections\n| fields device_id, behaviors_ioc_type\n| limit 5", ["org-c"])
        if err:
            results["[C3] Pipe mode: FROM | fields | limit"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[C3] Pipe mode: FROM | fields | limit"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[C3] Pipe mode: FROM | fields | limit", body, results):
                pass
            elif not rows:
                results["[C3] Pipe mode: FROM | fields | limit"] = (
                    "FAIL: 0 rows — crowdstrike_detections must have data at Stage 1+"
                )
            else:
                col_names = list(rows[0].keys())
                # Filter out internal metadata cols (_client, _sensor, _source_table)
                data_cols = [c for c in col_names if not c.startswith("_")]
                # MED-003: assert projection is non-vacuous — | fields must restrict to
                # EXACTLY the specified columns {device_id, behaviors_ioc_type}.
                # If set(data_cols) != expected, the pipe projection operator is broken.
                expected_cols = {"device_id", "behaviors_ioc_type"}
                if set(data_cols) != expected_cols:
                    results["[C3] Pipe mode: FROM | fields | limit"] = (
                        f"FAIL: projected cols {set(data_cols)!r} != expected "
                        f"{expected_cols!r} — | fields did not restrict to the two "
                        f"specified columns (projection operator regression; MED-003)"
                    )
                else:
                    results["[C3] Pipe mode: FROM | fields | limit"] = (
                        f"PASS: {len(rows)} rows; projected data cols={sorted(data_cols)!r} "
                        f"(exactly {expected_cols!r} confirmed; MED-003)"
                    )

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
            if sensor_errors_gate("[C4] DataFusion aggregate: COUNT(*)", body, results):
                pass
            elif not rows:
                results["[C4] DataFusion aggregate: COUNT(*)"] = (
                    "FAIL: COUNT(*) returned 0 rows — armis_devices must have data"
                )
            else:
                # MED-004: assert COUNT(*) value is non-null numeric >= 1.
                # rows[0] is a single-key dict; extract the first (only) value.
                count_val = list(rows[0].values())[0] if rows[0] else None
                if count_val is None or isinstance(count_val, bool) or not isinstance(count_val, (int, float)) or count_val < 1:
                    results["[C4] DataFusion aggregate: COUNT(*)"] = (
                        f"FAIL: COUNT(*) returned non-numeric or < 1 value: {count_val!r} "
                        f"(type={type(count_val).__name__}); "
                        f"armis_devices must have data (MED-004)"
                    )
                else:
                    results["[C4] DataFusion aggregate: COUNT(*)"] = (
                        f"PASS: {len(rows)} rows; COUNT(*)={count_val} "
                        f"(numeric >= 1; result={rows[0]!r})"
                    )

        # ── C5: DataFusion aggregate COUNT with GROUP BY ──────────────────────
        body, err = query(proc, "SELECT behaviors_ioc_type, COUNT(*) as cnt FROM crowdstrike_detections GROUP BY behaviors_ioc_type", ["org-c"])
        if err:
            results["[C5] DataFusion aggregate: GROUP BY"] = f"FAIL: {err}"
        elif body.get("error_code") == "E-QUERY-039":
            results["[C5] DataFusion aggregate: GROUP BY"] = "FAIL: E-QUERY-039 false-positive for GROUP BY COUNT"
        elif body.get("error_code"):
            # HIGH-001: unexpected error_code on GROUP BY → FAIL (mirrors H4/H5/H6/H10 pattern).
            results["[C5] DataFusion aggregate: GROUP BY"] = f"FAIL: {body['error_code']} — {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[C5] DataFusion aggregate: GROUP BY", body, results):
                pass
            elif not rows:
                results["[C5] DataFusion aggregate: GROUP BY"] = (
                    "FAIL: GROUP BY returned 0 rows — crowdstrike_detections must have data "
                    "(seed-200 CompromisedEndpoint guarantees 20 detections)"
                )
            else:
                # CONSCIOUS-UPDATE: bucket keys verified in make_detection_with_ioc() (generator.rs).
                # generate_with_scenario_iocs() stamps behaviors[0].ioc_type="hash_sha256" on
                # detection 0 only (ioc_hashes[0] from ScenarioEntityCatalog); detections 1-19
                # have MITRE-only behaviors with no ioc_type field.
                # source_path "$.behaviors[*].ioc_type" wildcard serializes as JSON-list string
                # (Design Decision 2): detection 0 → '["hash_sha256"]'; detections 1-19 → NULL.
                # seed-200 CompromisedEndpoint: 20 total detections. C5 asserts: bucket
                # '["hash_sha256"]' present + sum(cnt)==20 (not severity distribution;
                # severity distribution (Critical=5, Medium=15) is covered by H11).
                # DataFusion GROUP BY includes NULL as a group (SQL standard): sum(cnt) == 20.
                _ioc_buckets = {r.get("behaviors_ioc_type"): r.get("cnt") for r in rows}
                _hash_key = '["hash_sha256"]'
                _total_cnt = sum(v for v in _ioc_buckets.values() if isinstance(v, (int, float)))
                if _hash_key not in _ioc_buckets:
                    results["[C5] DataFusion aggregate: GROUP BY"] = (
                        f"FAIL: behaviors_ioc_type bucket '{_hash_key}' absent — "
                        f"generate_with_scenario_iocs() must stamp ioc_type=hash_sha256 on "
                        f"detection 0 (make_detection_with_ioc, generator.rs); "
                        f"got buckets={sorted(str(k) for k in _ioc_buckets)!r}"
                    )
                elif _total_cnt != 20:
                    results["[C5] DataFusion aggregate: GROUP BY"] = (
                        f"FAIL: sum(cnt)={_total_cnt} != 20 — seed-200 CompromisedEndpoint "
                        f"guarantees 20 total detections; buckets={list(_ioc_buckets.items())[:4]!r}"
                    )
                else:
                    results["[C5] DataFusion aggregate: GROUP BY"] = (
                        f"PASS: {len(rows)} bucket(s); '{_hash_key}' confirmed; "
                        f"sum(cnt)=20 (seed-200 contract); "
                        f"sample={list(_ioc_buckets.items())[:2]!r}"
                    )

        # ── C6: DataFusion aggregate MAX/MIN ──────────────────────────────────
        body, err = query(proc, "SELECT MAX(device_id), MIN(device_id) FROM crowdstrike_detections", ["org-c"])
        if err:
            results["[C6] DataFusion aggregate: MAX/MIN"] = f"FAIL: {err}"
        elif body.get("error_code") == "E-QUERY-039":
            results["[C6] DataFusion aggregate: MAX/MIN"] = "FAIL: E-QUERY-039 false-positive for MAX/MIN"
        elif body.get("error_code"):
            # HIGH-001: unexpected error_code on MAX/MIN → FAIL (mirrors H4/H5/H6/H10 pattern).
            results["[C6] DataFusion aggregate: MAX/MIN"] = f"FAIL: {body['error_code']} — {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[C6] DataFusion aggregate: MAX/MIN", body, results):
                pass
            elif not rows:
                results["[C6] DataFusion aggregate: MAX/MIN"] = (
                    "FAIL: MAX/MIN returned 0 rows — crowdstrike_detections must have data"
                )
            else:
                # MED-004: assert MAX/MIN aggregate values are non-null.
                # NULL aggregate means crowdstrike_detections.device_id is all-NULL → data regression.
                first = rows[0]
                null_vals = [k for k, v in first.items() if v is None]
                if null_vals:
                    results["[C6] DataFusion aggregate: MAX/MIN"] = (
                        f"FAIL: MAX/MIN has NULL aggregate values: {null_vals!r} — "
                        f"crowdstrike_detections.device_id must be non-null (MED-004); "
                        f"result={first!r}"
                    )
                else:
                    results["[C6] DataFusion aggregate: MAX/MIN"] = (
                        f"PASS: {len(rows)} rows; MAX/MIN non-null confirmed; result={first!r}"
                    )

        # ── C7: Pipe mode | sort ──────────────────────────────────────────────
        # F-AUD-P28-LOW-001: LIMIT raised from 5 to 12 to create a lex-vs-numeric
        # divergence window.  Seed-200 emits 20 detections with device_id suffixes
        # 0..19 (generator evidence: 20 rows available).  With LIMIT 5, only suffixes
        # 0..4 appear — all single-digit, so lex order matches numeric order and
        # sorted(device_ids)==device_ids passes vacuously even if | sort is broken.
        # With LIMIT 12, suffixes 0..11 appear; lex sort places "...-10" and "...-11"
        # BEFORE "...-2".."...-9", producing a different ordering from numeric sort.
        # `sorted(device_ids) == device_ids` now meaningfully validates ascending
        # lex order (which IS the correct sort semantics for string device_id).
        body, err = query(proc, "FROM crowdstrike_detections\n| sort device_id\n| limit 12", ["org-c"])
        if err:
            results["[C7] Pipe mode: | sort"] = f"FAIL: {err}"
        elif body.get("error_code"):
            # HIGH-001: unexpected error_code on | sort → FAIL (mirrors H4/H5/H6/H10 pattern).
            results["[C7] Pipe mode: | sort"] = f"FAIL: {body['error_code']} — {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[C7] Pipe mode: | sort", body, results):
                pass
            elif not rows:
                results["[C7] Pipe mode: | sort"] = (
                    "FAIL: 0 rows — crowdstrike_detections must have data at Stage 1+"
                )
            else:
                # LOW-005: verify | sort device_id actually sorted the results ascending
                device_ids = [r.get("device_id") for r in rows if r.get("device_id") is not None]
                # F-AUD-P18-MED-001: guard against all-null device_id — vacuous sorted-check
                # would PASS and device_ids[0] would IndexError (mirrors H3's has_nonempty).
                if not device_ids:
                    results["[C7] Pipe mode: | sort"] = (
                        f"FAIL: all device_id null in {len(rows)} rows — "
                        f"data-quality regression (Standing Rule 3 §2)"
                    )
                else:
                    is_sorted_asc = device_ids == sorted(device_ids)
                    if is_sorted_asc:
                        results["[C7] Pipe mode: | sort"] = (
                            f"PASS: {len(rows)} rows; device_id sorted ascending confirmed; "
                            f"first={str(device_ids[0])[:40]!r}"
                        )
                    else:
                        results["[C7] Pipe mode: | sort"] = (
                            f"FAIL: {len(rows)} rows but device_id not in sorted order — "
                            f"| sort operator not working; first 5={device_ids[:5]!r}"
                        )

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
            # HIGH-001: unexpected error_code on SQL baseline → FAIL (mirrors H4/H5/H6/H10 pattern).
            results["[C8] Temporal: SQL mode executes (ADR-052 §D4 baseline path)"] = f"FAIL: {body['error_code']}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[C8] Temporal: SQL mode executes (ADR-052 §D4 baseline path)", body, results):
                pass
            elif not rows:
                results["[C8] Temporal: SQL mode executes (ADR-052 §D4 baseline path)"] = (
                    "FAIL: 0 rows — crowdstrike_detections must have data at Stage 1+"
                )
            else:
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
            if sensor_errors_gate("[D1] SCENARIO: Stage 4 armis_devices visible", body, results):
                pass
            elif not rows:
                results["[D1] SCENARIO: Stage 4 armis_devices visible"] = "FAIL: 0 rows (scenario stage not progressing?)"
            else:
                device_ids = [str(r.get("device_id", "")) for r in rows if r.get("device_id")]
                # org-c seed-200: all device IDs contain the -200- segment
                # LOW-004 sibling sweep (F-AUD-P25-TD-VSDD-060): anchor with regex so
                # "-200-" in d is not confused with a value like "dev-1200-abc".
                # Pattern: seed segment followed by ordinal digit (dev-<hex>-200-<n>).
                has_seed_200 = any(re.search(r'-200-\d', d) for d in device_ids)
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
            if sensor_errors_gate("[D2] IOC-FIELDS: cyberint iocs_value at Stage 4", body, results):
                pass
            elif rows:
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
            if sensor_errors_gate("[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+", body, results):
                pass
            elif rows:
                ioc_type = rows[0].get("behaviors_ioc_type", "MISSING")
                ioc_val = rows[0].get("behaviors_ioc_value", "MISSING")
                results["[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+"] = f"PASS: {len(rows)} rows, sample ioc_type={str(ioc_type)[:30]!r} ioc_value={str(ioc_val)[:30]!r}"
            else:
                # F-AUD-P2-HIGH-001: Stage 4 is terminal/absorbing; Stage 2+
                # IOC fields are guaranteed present → FAIL, not WARN.
                results["[D3] IOC-FIELDS: CS behaviors_ioc_type at Stage 2+"] = "FAIL: 0 rows with behaviors_ioc_type IS NOT NULL (Stage 4 guarantees Stage 2+ IOC fields)"

        # ── D4: Claroty audit_logs at Stage 4 ────────────────────────────────
        # F-AUD-P5-LOW-004: D4 queries the same fixture-backed table (claroty_audit_logs)
        # without stage/time filtering — apply the same exact-5 strictness as B4.
        # query with limit 10 (> fixture size) and assert exactly 5 rows.
        body, err = query(proc, "FROM claroty_audit_logs | limit 10", ["org-c"])
        if err:
            results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = f"FAIL: {err}"
        elif body.get("error_code"):
            results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = f"FAIL: {body['error_code']}: {body.get('message','')[:80]}"
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[D4] Claroty audit_logs at Stage 4 (org-c)", body, results):
                pass
            elif len(rows) == 5:
                first = rows[0]
                # Check for key columns: id, action, actor, resource, timestamp
                has_id = "id" in first
                has_action = "action" in first
                results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = (
                    f"PASS: exactly 5 rows (static fixture confirmed; gap-analysis §4 guardrail #6); "
                    f"id={has_id}, action={has_action}; cols={list(first.keys())[:6]}"
                )
            elif rows:
                results["[D4] Claroty audit_logs at Stage 4 (org-c)"] = (
                    f"FAIL: {len(rows)} rows (expected exactly 5 from static fixture, "
                    f"crates/prism-dtu-claroty/fixtures/audit-log.json; "
                    f"gap-analysis §4 guardrail #6)"
                )
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
            sensor_errors_cs = body_cs.get("sensor_errors", [])
            if sensor_errors_cs:
                results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = (
                    f"FAIL: CS partial fan-out failure — sensor_errors={sensor_errors_cs[:2]} (F-AUD-P30-MED-001)"
                )
            elif cs_rows:
                cs_device_id = cs_rows[0].get("device_id", "")
                if cs_device_id:
                    # PQL-injection defense-in-depth: DTU generator currently emits
                    # dev-<hex>-<seed>-<n> (all alphanumeric + hyphens), safe by construction;
                    # validate at the audit-script trust boundary before interpolating into a
                    # PrismQL query string.
                    if not re.fullmatch(r"[A-Za-z0-9._-]+", cs_device_id):
                        results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = (
                            f"FAIL: unexpected device_id shape — cannot safely interpolate into PQL "
                            f"(got {cs_device_id[:30]!r})"
                        )
                    else:
                        body_am, err_am = query(proc,
                            f"FROM armis_devices\n| where device_id = '{cs_device_id}'\n| limit 1",
                            ["org-c"])
                        if err_am:
                            # PARTIAL sweep: transport failure on Armis lookup → demo not ready;
                            # cross-sensor coherence cannot be verified → FAIL.
                            results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"FAIL: CS has device_id={cs_device_id[:30]}, Armis query transport error: {err_am}"
                        elif body_am.get("error_code"):
                            # PARTIAL sweep: Armis returned error; coherence unverified → FAIL.
                            results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = f"FAIL: CS device found, Armis lookup error: {body_am['error_code']}: {body_am.get('message','')[:60]}"
                        else:
                            am_rows = body_am.get("rows", [])
                            sensor_errors_am = body_am.get("sensor_errors", [])
                            if sensor_errors_am:
                                results["[D5] SCENARIO: cross-sensor entity coherence (CS+Armis)"] = (
                                    f"FAIL: Armis partial fan-out failure — sensor_errors={sensor_errors_am[:2]} (F-AUD-P30-MED-001)"
                                )
                            elif am_rows:
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
            if sensor_errors_gate("[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts", body, results):
                pass
            elif rows:
                first = rows[0]
                threat_score = first.get("threat_score", "MISSING")
                iocs_value_first = first.get("iocs_value_first", "?")
                if threat_score == "MISSING":
                    results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"FAIL: threat_score column missing from result; cols={list(first.keys())[:8]}"
                elif isinstance(threat_score, bool) or not isinstance(threat_score, (int, float)):
                    # LOW-002: bool is subclass of int — must exclude explicitly
                    results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"FAIL: threat_score must be Int64 (ADR-051 D1); got type={type(threat_score).__name__}, value={str(threat_score)[:40]!r}"
                elif threat_score < 75:
                    # MED-006: value gate — scenario IOCs must score >= 75 (gap-analysis §3)
                    results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = (
                        f"FAIL: threat_score={threat_score} < 75 — "
                        f"scenario IOCs must score >= 75 (gap-analysis §3 data contract)"
                    )
                else:
                    results["[E1] ENRICH: threat_score(iocs_value_first) on cyberint_alerts"] = f"PASS: {len(rows)} rows; threat_score={threat_score} (int, >= 75); iocs_value_first={str(iocs_value_first)[:40]!r}"
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
            if sensor_errors_gate("[E2] ENRICH: threat_is_known_malicious(iocs_value_first)", body, results):
                pass
            elif rows:
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
            if sensor_errors_gate("[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices", body_arm, results):
                pass
            elif rows:
                first = rows[0]
                cvss = first.get("cvss_base_score", "MISSING")
                cve_id = first.get("device_cves_first", "?")
                if cvss == "MISSING":
                    results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = f"FAIL: cvss_base_score column missing; cols={list(first.keys())[:8]}"
                elif isinstance(cvss, bool) or not isinstance(cvss, (int, float)):
                    # MED-005: type guard — cvss_base_score must be numeric (LOW-002: bool exclusion)
                    results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = (
                        f"FAIL: cvss_base_score must be numeric Float64; "
                        f"got type={type(cvss).__name__}, value={str(cvss)[:40]!r}"
                    )
                elif abs(cvss - 8.1) > 1e-9:
                    # HIGH-003: exact value gate — runbook v1.10 §5.5 ("exactly base_score = 8.1")
                    # and D-1695 PO adjudication are the exact-value authority; the DTU hardcodes
                    # 8.1 as its default.  BC-2.06.020 §PC-3 specifies the >= 7.0 contract floor
                    # with 8.1 as the DTU default — the BC does NOT require exactly 8.1; this gate
                    # is tighter than the BC floor per the runbook/PO adjudication (source-of-truth
                    # precedence: runbook v1.10 §5.5 + D-1695 supersede the >= 7.0 floor for the
                    # preflight gate).  Float tolerance abs(cvss - 8.1) > 1e-9 avoids IEEE 754
                    # representation hazard on the numeric branch.
                    results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = (
                        f"FAIL: cvss_base_score={cvss} != 8.1 (tolerance 1e-9) — "
                        f"DTU hardcodes 8.1; runbook v1.10 §5.5 + D-1695 PO adjudication"
                    )
                else:
                    results["[E3] ENRICH: cvss_base_score(device_cves_first) on armis_devices"] = (
                        f"PASS: {len(rows)} rows; cvss_base_score={cvss} "
                        f"(== 8.1, BC-2.06.020 PC-4 confirmed); cve_id={str(cve_id)[:30]!r}"
                    )
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
            if sensor_errors_gate("[E4] ENRICH: cvss_severity(device_cves_first)", body, results):
                pass
            elif rows:
                severity = rows[0].get("cvss_severity", "MISSING")
                if severity == "MISSING":
                    results["[E4] ENRICH: cvss_severity(device_cves_first)"] = f"FAIL: cvss_severity column missing; cols={list(rows[0].keys())[:8]}"
                elif severity != "HIGH":
                    # F-AUD-P2-MED-004: scenario CVE-9999-* has CVSS ≥ 9.0 → severity
                    # MUST be "HIGH" (gap-analysis §3 data contract).
                    # LOW-003: "HIGH" is the raw enrichment-UDF output (BC-2.06.020 §PC-3);
                    # enrichment outputs bypass OCSF enum-label normalization — if a future
                    # ADR normalizes UDF outputs, this check must be consciously updated.
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
            if sensor_errors_gate("[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections", body, results):
                pass
            elif rows:
                threat_score = rows[0].get("threat_score", "MISSING")
                if threat_score == "MISSING":
                    # HIGH-001: absence guard — threat_score column must be present
                    results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = (
                        f"FAIL: threat_score column missing from result; cols={list(rows[0].keys())[:8]}"
                    )
                elif isinstance(threat_score, bool) or not isinstance(threat_score, (int, float)):
                    # HIGH-001: type guard — must be numeric Int64 (LOW-002: bool exclusion)
                    results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = (
                        f"FAIL: threat_score must be Int64 (ADR-051 D1); "
                        f"got type={type(threat_score).__name__}, value={str(threat_score)[:40]!r}"
                    )
                elif threat_score < 75:
                    # HIGH-001: value gate — CS scenario IOCs must score >= 75 (gap-analysis §3)
                    results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = (
                        f"FAIL: threat_score={threat_score} < 75 — "
                        f"CS scenario IOCs must score >= 75 (gap-analysis §3 data contract)"
                    )
                else:
                    results["[E5] ENRICH: threat_score(behaviors_ioc_value_first) on CS detections"] = (
                        f"PASS: {len(rows)} rows; threat_score={threat_score} (int, >= 75)"
                    )
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
            if sensor_errors_gate("[E6] ENRICH: ThreatIntel score >= 75 for scenario IOCs", body, results):
                pass
            elif rows:
                # LOW-002: bool is subclass of int — exclude booleans from numeric check
                scores = [r.get("threat_score") for r in rows if isinstance(r.get("threat_score"), (int, float)) and not isinstance(r.get("threat_score"), bool)]
                non_int_sample = next((r.get("threat_score") for r in rows if r.get("threat_score") is not None and (isinstance(r.get("threat_score"), bool) or not isinstance(r.get("threat_score"), (int, float)))), None)
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
            msg = body.get("message", "")
            if error_code == "E-QUERY-032":
                # F-AUD-P7-LOW-004 (POL-24): verify message-template anchor from
                # error-taxonomy.md §E-QUERY-032 / PrismError::SensorNotRegistered Display:
                # "E-QUERY-032: Sensor '{sensor_id}' is not registered for org '{org_slug}'"
                has_anchor = "is not registered for org" in msg
                if has_anchor:
                    results["[F1] E-QUERY-032: cyberint for org-a errors"] = (
                        f"PASS: E-QUERY-032 + anchor 'is not registered for org' confirmed; "
                        f"message={msg[:60]!r}"
                    )
                else:
                    results["[F1] E-QUERY-032: cyberint for org-a errors"] = (
                        f"FAIL: E-QUERY-032 but message-template anchor 'is not registered for org' "
                        f"absent — message-template regression (POL-24); message={msg[:80]!r}"
                    )
            elif not error_code:
                rows = body.get("rows", [])
                results["[F1] E-QUERY-032: cyberint for org-a errors"] = f"FAIL: returned {len(rows)} rows (should error — org-a has no cyberint)"
            else:
                results["[F1] E-QUERY-032: cyberint for org-a errors"] = f"FAIL: expected E-QUERY-032, got {error_code}: {msg[:80]}"

        # ── F2: E-QUERY-032: armis for org-b (no sensor) ─────────────────────
        # F-AUD-P1-MED-003: runbook v1.8 §5.8 N3 correction — E-QUERY-032 only.
        # E-QUERY-037 is dot-notation (covered by F3/A15); this path must produce
        # E-QUERY-032 (table not available for client).
        body, err = query(proc, "FROM armis_devices | limit 5", ["org-b"])
        if err:
            results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            msg = body.get("message", "")
            if error_code == "E-QUERY-032":
                # F-AUD-P7-LOW-004 (POL-24): same anchor as F1 — E-QUERY-032 Display.
                has_anchor = "is not registered for org" in msg
                if has_anchor:
                    results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = (
                        f"PASS: E-QUERY-032 + anchor 'is not registered for org' confirmed; "
                        f"message={msg[:60]!r}"
                    )
                else:
                    results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = (
                        f"FAIL: E-QUERY-032 but message-template anchor 'is not registered for org' "
                        f"absent — message-template regression (POL-24); message={msg[:80]!r}"
                    )
            elif not error_code:
                rows = body.get("rows", [])
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: returned {len(rows)} rows (should error — org-b has no armis)"
            else:
                results["[F2] E-QUERY-032: armis for org-b (no sensor)"] = f"FAIL: expected E-QUERY-032, got {error_code}: {msg[:80]}"

        # ── F3: N2: E-QUERY-037 dot-notation FROM ────────────────────────────
        body, err = query(proc, "FROM crowdstrike.detections | limit 3", ["org-c"])
        if err:
            results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            msg = body.get("message", "")
            if error_code == "E-QUERY-037":
                # F-AUD-P7-LOW-004 (POL-24): verify BOTH message-template segments from
                # error-taxonomy.md §E-QUERY-037 message_template:
                #   "... Available sensors: [{available_sensors}]. Available tables: [{available_tables}]..."
                # LOW-002 (F-AUD-P26): require both 'Available sensors:' AND 'Available tables:'
                # — the single-anchor check was incomplete; only requiring 'Available tables:'
                # would miss a regression where the sensors segment was dropped.
                has_sensor_anchor = "Available sensors:" in msg
                has_table_anchor = "Available tables:" in msg
                if has_sensor_anchor and has_table_anchor:
                    results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = (
                        f"PASS: E-QUERY-037 + both anchors confirmed "
                        f"('Available sensors:' + 'Available tables:'); "
                        f"message={msg[:80]!r}"
                    )
                else:
                    missing_anchors = []
                    if not has_sensor_anchor:
                        missing_anchors.append("'Available sensors:'")
                    if not has_table_anchor:
                        missing_anchors.append("'Available tables:'")
                    results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = (
                        f"FAIL: E-QUERY-037 but message-template anchor(s) "
                        f"{', '.join(missing_anchors)} absent — regression (POL-24); "
                        f"message={msg[:80]!r}"
                    )
            elif not error_code and body.get("rows") is not None:
                results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = "FAIL: returned rows silently (no error)"
            else:
                results["[F3] N2: dot-notation FROM -> E-QUERY-037"] = f"FAIL: got {error_code or 'no error'}: {msg[:80]}"

        # ── F4: N1-B: E-QUERY-039 unknown enrich UDF ─────────────────────────
        body, err = query(proc, "FROM armis_devices | enrich nonexistent_udf(device_id) | limit 3", ["org-c"])
        if err:
            results["[F4] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            msg = body.get("message", "")
            if error_code == "E-QUERY-039" or ("E-QUERY-039" in msg):
                # F-AUD-P7-LOW-004 (POL-24): verify message-template anchor from
                # error-taxonomy.md §E-QUERY-039 / EnrichUdfNotFoundDetails Display:
                # "E-QUERY-039: enrichment infusion '...' is not registered; available: [...]"
                has_anchor = "is not registered; available: [" in msg
                if has_anchor:
                    results["[F4] N1-B: unknown enrich UDF -> E-QUERY-039"] = (
                        f"PASS: E-QUERY-039 + anchor 'is not registered; available: [' confirmed; "
                        f"message={msg[:80]!r}"
                    )
                else:
                    results["[F4] N1-B: unknown enrich UDF -> E-QUERY-039"] = (
                        f"FAIL: E-QUERY-039 but message-template anchor "
                        f"'is not registered; available: [' absent — "
                        f"message-template regression (POL-24); message={msg[:80]!r}"
                    )
            else:
                results["[F4] N1-B: unknown enrich UDF -> E-QUERY-039"] = f"FAIL: got {error_code or 'no error'}: {msg[:80]}"

        # ── F5: E-QUERY-038 unknown column + Did you mean anchor (001-B BLOCKER) ──
        # F-AUD-P1-MED-008: F5's designated regression class is PR #219's E-QUERY-038 gate.
        # The former alt-error branch (`error_code and "column" in msg.lower()`) accepted
        # any error code mentioning "column" — removed. Only E-QUERY-038 PASSes here.
        # F-AUD-P7-LOW-004: use a near-miss column 'detction_id' (Levenshtein=1 from
        # 'detection_id') so the "Did you mean:" anchor from ColumnNotFoundDetails Display
        # is exercised directly in F5; F5 also co-tests sc_error.did_you_mean == "detection_id"
        # (F-AUD-P10-MED-005) — both text anchors AND the structured field are load-bearing here.
        # H2 sibling covers the available_columns wire contract (BC-2.11.016).
        # Template anchors verified against error.rs ColumnNotFoundDetails Display:
        #   invariant: "not found in table" (always present)
        #   near-miss:  "Did you mean:" (present when Levenshtein ≤ 3 match exists)
        # COUPLING NOTE (OBS-006): 'detction_id' (Levenshtein=1 from 'detection_id') creates
        # a tight coupling between this test and the crowdstrike_detections schema column name.
        # If 'detection_id' is renamed in the sensor TOML, BOTH the typo string 'detction_id'
        # above AND the sc_dym_f5 == "detection_id" assertion below must be updated together to
        # maintain coverage. The coupling is intentional (POL-24 philosophy: deliberate schema
        # changes must cause a conscious update to this audit). Canonical symbol source:
        # `check_column_availability` + `ColumnNotFoundDetails.did_you_mean` in
        # crates/prism-query/src/engine.rs (TD-VSDD-091: cite function name, not line numbers).
        # MED-004: a change to the Levenshtein implementation or tie-break heuristic
        # (e.g., candidate ordering when multiple columns are Levenshtein-equidistant)
        # could also change the did_you_mean return value — if that happens, consciously
        # update the sc_dym_f5 == "detection_id" assertion and this comment.
        body, err = query(proc,
            "SELECT device_id, detction_id FROM crowdstrike_detections LIMIT 5",
            ["org-c"])
        if err:
            results["[F5] E-QUERY-038: unknown column returns plan-time error"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            msg = body.get("message", "")
            if error_code == "E-QUERY-038":
                # F-AUD-P7-LOW-004 (POL-24): require both invariant anchor and
                # "Did you mean:" anchor (near-miss column guarantees the suggestion).
                has_invariant = "not found in table" in msg
                has_dym = "Did you mean:" in msg
                # F-AUD-P10-MED-005: also require structured field
                # _sc_error.did_you_mean == "detection_id" (expected correction for typo
                # "detction_id"). Verified against H2's extraction helper (_sc_error dict
                # keyed by "did_you_mean") and BC-2.11.016 wire contract: field name is
                # "did_you_mean", value is the corrected column name string.
                # Text anchors present but structured field wrong/absent → FAIL.
                sc_err_f5 = body.get("_sc_error", {})
                sc_dym_f5 = sc_err_f5.get("did_you_mean", "") if sc_err_f5 else ""
                if has_invariant and has_dym and sc_dym_f5 == "detection_id":
                    results["[F5] E-QUERY-038: unknown column returns plan-time error"] = (
                        f"PASS: E-QUERY-038 + anchors 'not found in table' and "
                        f"'Did you mean:' confirmed (POL-24); "
                        f"sc_error.did_you_mean={sc_dym_f5!r} confirmed (MED-005); "
                        f"message={msg[:80]!r}"
                    )
                elif has_invariant and has_dym:
                    # Text anchors present but structured field absent or wrong.
                    results["[F5] E-QUERY-038: unknown column returns plan-time error"] = (
                        f"FAIL: E-QUERY-038 + text anchors present but structuredContent.error "
                        f"regression — sc_error.did_you_mean={sc_dym_f5!r} "
                        f"(expected 'detection_id'); F-AUD-P10-MED-005"
                    )
                elif not has_invariant:
                    results["[F5] E-QUERY-038: unknown column returns plan-time error"] = (
                        f"FAIL: E-QUERY-038 but invariant anchor 'not found in table' "
                        f"absent — message-template regression (POL-24); message={msg[:80]!r}"
                    )
                else:
                    results["[F5] E-QUERY-038: unknown column returns plan-time error"] = (
                        f"FAIL: E-QUERY-038 + invariant anchor present but 'Did you mean:' "
                        f"absent for near-miss column 'detction_id' — "
                        f"message-template regression (POL-24); message={msg[:80]!r}"
                    )
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
        # F-AUD-P29-LOW-002: strict-success semantics aligned with C4 — any error_code
        # (not just E-QUERY-039) is FAIL; only no error_code with rows is PASS.
        body, err = query(proc, "SELECT COUNT(*) FROM armis_devices", ["org-c"])
        if err:
            results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"FAIL: {err}"
        else:
            error_code = body.get("error_code", "")
            if error_code == "E-QUERY-039":
                results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = "FAIL: E-QUERY-039 falsely fired for COUNT(*)"
            elif error_code:
                results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"FAIL: unexpected error {error_code}: {body.get('message', '')[:80]}"
            else:
                rows = body.get("rows", [])
                results["[F6] N1-B F1: SQL builtin COUNT NOT E-QUERY-039"] = f"PASS: COUNT executed OK, {len(rows)} rows"

        # ═══════════════════════════════════════════════════════════════════════
        # SECTION G: New Merged Surfaces (PRs #214/#216/#217 — develop@f935edb6)
        # G1: IEQ filter happy path (S-PRISMQL-CASE-INSENSITIVE-001, ADR-047)
        # G2: IIN multi-value severity filter (ADR-047)
        # G3: IIN on status lowercase (ADR-047)
        # G4: SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary (ADR-047)
        # G5: RETIRED (pass-9) — see retirement comment below
        # G6: GROUP BY severity no-fragmentation (canonical Title-case only)
        # G7: Temporal typing spot-check — no regression (ADR-052 §D4, PR #214)
        # G8: Typed enrichment output — threat_score is Int64 not String (ADR-051, PR #216)
        # ═══════════════════════════════════════════════════════════════════════

        # ── G1: IEQ happy path: severity IEQ 'critical' matches canonical 'Critical' ──
        # Runbook Step 3.1a / §5.9 checklist item 1.
        # IEQ lowers both sides: lower(severity) = lower('critical').
        # Stored form is 'Critical' (OCSF Title-case at adapter boundary per enum_map.rs).
        # F-AUD-P5-LOW-002: runbook v1.7+ (changelog ~line 1116) amended Step 3.1a to
        # use 'critical'; this test is aligned with the amended runbook. No mismatch.
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
            if sensor_errors_gate("[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)", body, results):
                pass
            elif rows:
                # F-AUD-P19-MED-001: null-leak guard — IEQ filter must not return rows
                # with NULL/empty severity; SQL lower(NULL) predicate is UNKNOWN (never
                # matches), so any NULL row in the result is a filter regression.
                null_leak_count = sum(1 for r in rows if not r.get("severity"))
                severities = [r.get("severity", "") for r in rows if r.get("severity")]
                # F-AUD-P15-LOW-003: guard against vacuous-True all() / any() when every
                # severity value is empty — mirrors H3's has_nonempty pattern.
                # IEQ contract guarantees severity column populated for crowdstrike_detections.
                has_nonempty = any(s for s in severities)
                # Verify stored severity values are canonical Title-case ('Critical', not 'CRITICAL'/'critical')
                bad_case = [s for s in severities if s and s.lower() == "critical" and s != "Critical"]
                # F-AUD-P1-MED-001: also assert ONLY critical-severity rows are returned.
                # IEQ 'critical' must filter out non-critical rows; any non-critical row is a
                # filter failure (mirrors H3's guard for INE).
                has_non_critical = any(s and s.lower() != "critical" for s in severities if s)
                if null_leak_count > 0:
                    results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                        f"FAIL: filter leaked {null_leak_count} NULL/empty-severity rows — "
                        f"WHERE/IEQ/IIN regression (SQL: lower(NULL) predicate is UNKNOWN, must not match); "
                        f"total_rows={len(rows)}"
                    )
                elif not has_nonempty:
                    results["[G1] IEQ: severity IEQ 'critical' (crowdstrike_detections, org-c)"] = (
                        f"FAIL: {len(rows)} rows but all severity values empty — "
                        f"data-quality regression (Standing Rule 3 §2); "
                        f"severities={list(set(severities))!r}"
                    )
                elif has_non_critical:
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
                        f"only critical rows confirmed; aligned with runbook v1.7+ Step 3.1a using 'critical')"
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
            if sensor_errors_gate("[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)", body, results):
                pass
            elif rows:
                # F-AUD-P19-MED-001: null-leak guard — IIN filter must not return rows
                # with NULL/empty severity; SQL lower(NULL) is UNKNOWN (never matches).
                null_leak_count = sum(1 for r in rows if not r.get("severity"))
                distinct_sev = sorted({r.get("severity", "") for r in rows if r.get("severity")})
                # F-AUD-P15-LOW-003 sweep: IIN contract guarantees cyberint_alerts has
                # severity values; empty distinct_sev means the column was absent/null.
                if null_leak_count > 0:
                    results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = (
                        f"FAIL: filter leaked {null_leak_count} NULL/empty-severity rows — "
                        f"WHERE/IEQ/IIN regression (SQL: lower(NULL) predicate is UNKNOWN, must not match); "
                        f"total_rows={len(rows)}"
                    )
                elif not distinct_sev:
                    results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = (
                        f"FAIL: {len(rows)} rows but all severity values empty/absent — "
                        f"data-quality regression (Standing Rule 3 §2)"
                    )
                else:
                    # MED-002: filter-echo membership — IIN('high','critical') must only
                    # return rows where severity.lower() ∈ {'high','critical'}
                    foreign = [s for s in distinct_sev if s.lower() not in {"high", "critical"}]
                    if foreign:
                        results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = (
                            f"FAIL: {len(rows)} rows but foreign severity values returned — "
                            f"IIN filter leaked rows not in {{high,critical}}: {foreign!r}; "
                            f"all distinct={distinct_sev!r}"
                        )
                    else:
                        results["[G2] IIN: severity IIN ('high','critical') (cyberint_alerts, org-c)"] = (
                            f"PASS: {len(rows)} rows; distinct severities={distinct_sev!r} "
                            f"(all in {{high,critical}}; IIN filter-echo confirmed)"
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
        # cyberint.sensor.toml.
        # IIN lowers both sides: IIN('new','in progress') cannot match cyberint's
        # "open"/"acknowledged"/"closed" because none of those are in {'new','in progress'}.
        # However IIN('open','closed') CAN match cyberint: lower("open")="open" ∈
        # {"open","closed"} → matches.  G3b below verifies that positive case.
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
            if sensor_errors_gate("[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)", body, results):
                pass
            elif rows:
                # F-AUD-P19-MED-001: null-leak guard — IIN filter must not return rows with
                # NULL/empty status; SQL lower(NULL) is UNKNOWN (never matches).
                # F-AUD-P19-LOW-001: align outer filter with G3b's `is not None` + has_nonempty
                # discipline: use `is not None` so empty-string status values are kept and
                # handled by has_nonempty (mirrors G3b lines below).
                null_leak_count = sum(1 for r in rows if not r.get("status"))
                distinct_status = sorted({r.get("status", "") for r in rows if r.get("status") is not None})
                # G3b discipline: has_nonempty handles the all-empty-string case
                # (distinct_status == [""] after is-not-None filter).
                has_nonempty_status = any(s for s in distinct_status)
                # Check for casing fragmentation: same value in different cases (mirrors G6).
                status_lower_list = [s.lower() for s in distinct_status if s]
                has_dup_lower = len(status_lower_list) != len(set(status_lower_list))
                # Verify stored status values are OCSF canonical Title-case ("New", not
                # "new"/"NEW"), proving normalize_enum_label ran before the IIN match.
                known_ocsf_status = {"new", "in progress", "suppressed", "resolved",
                                     "archived", "deleted", "unknown", "success", "failure", "other"}
                non_title = [s for s in distinct_status
                             if s and s.lower() in known_ocsf_status and s != s.title()]
                # F-AUD-P15-LOW-003 sweep: IIN contract guarantees crowdstrike_detections has
                # status values; empty distinct_status means column absent/null — data-quality
                # regression (mirrors H3's has_nonempty guard).
                if null_leak_count > 0:
                    results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                        f"FAIL: filter leaked {null_leak_count} NULL/empty-status rows — "
                        f"WHERE/IEQ/IIN regression (SQL: lower(NULL) predicate is UNKNOWN, must not match); "
                        f"total_rows={len(rows)}"
                    )
                elif not has_nonempty_status:
                    results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                        f"FAIL: {len(rows)} rows but all status values empty/absent — "
                        f"data-quality regression (Standing Rule 3 §2)"
                    )
                elif has_dup_lower:
                    results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                        f"FAIL: casing fragmentation — duplicate status buckets: {distinct_status}"
                    )
                elif non_title:
                    results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                        f"FAIL: non-Title-case status values returned (OCSF normalization not applied): "
                        f"{non_title!r}; all returned={distinct_status!r}"
                    )
                else:
                    # MED-003: filter-echo membership — IIN('new','in progress') must only
                    # return rows where status.lower() ∈ {'new','in progress'}
                    foreign = [s for s in distinct_status if s.lower() not in {"new", "in progress"}]
                    if foreign:
                        results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                            f"FAIL: {len(rows)} rows but foreign status values returned — "
                            f"IIN filter leaked rows not in {{new,in progress}}: {foreign!r}; "
                            f"all distinct={distinct_status!r}"
                        )
                    else:
                        results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                            f"PASS: {len(rows)} rows; distinct statuses={distinct_status!r} "
                            f"(Title-case confirmed; all in {{new,in progress}}; IIN filter-echo confirmed)"
                        )
            else:
                results["[G3] IIN: status IIN ('new','in progress') (crowdstrike_detections, org-c)"] = (
                    "FAIL: 0 rows — crowdstrike_detections at Stage 1+ must have status='new' "
                    "detection records; CS DTU emits status 'new' → OcsfEnumMap normalizes to 'New' "
                    "(status_id[1001]); IIN operator may not be matching or CS data absent"
                )

        # ── G3b: Runbook Step 3.1a literal — cyberint status IIN ('open', 'closed') ─
        # Verifies the POSITIVE case that G3 redirected away from: cyberint's vendor-native
        # status values "open"/"closed" are passed through unchanged (no OCSF enum_map).
        # IIN lowers both sides: lower("open")="open" ∈ {"open","closed"} → match.
        # T13-capstone-demo-runbook.md line ~515: literal query for this step.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where status IIN ('open', 'closed')\n| limit 20",
            ["org-c"])
        if err:
            results["[G3b] Runbook Step 3.1a literal: cyberint status IIN ('open','closed')"] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results["[G3b] Runbook Step 3.1a literal: cyberint status IIN ('open','closed')"] = (
                f"FAIL: {ec}: {body.get('message','')[:100]}"
            )
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[G3b] Runbook Step 3.1a literal: cyberint status IIN ('open','closed')", body, results):
                pass
            elif rows:
                distinct_status = sorted({r.get("status", "") for r in rows if r.get("status") is not None})
                # Pass-18 sweep: `is not None` filter admits empty strings; guard against both
                # all-None (distinct_status==[]) and all-empty-string (distinct_status==[""]).
                # Mirrors G3's has_nonempty pattern (Standing Rule 3 §2).
                has_nonempty_status = any(s for s in distinct_status)
                expected = {"open", "closed"}
                foreign = [s for s in distinct_status if s.lower() not in expected]
                if not has_nonempty_status:
                    results["[G3b] Runbook Step 3.1a literal: cyberint status IIN ('open','closed')"] = (
                        f"FAIL: {len(rows)} rows but all status values null/empty — "
                        f"data-quality regression (Standing Rule 3 §2)"
                    )
                elif foreign:
                    results["[G3b] Runbook Step 3.1a literal: cyberint status IIN ('open','closed')"] = (
                        f"FAIL: unexpected status values outside {{'open','closed'}}: {foreign!r}; "
                        f"all returned={distinct_status!r}"
                    )
                else:
                    results["[G3b] Runbook Step 3.1a literal: cyberint status IIN ('open','closed')"] = (
                        f"PASS: {len(rows)} rows; distinct statuses={distinct_status!r} "
                        f"(all in {{open,closed}}; IIN vendor-native pass-through confirmed)"
                    )
            else:
                results["[G3b] Runbook Step 3.1a literal: cyberint status IIN ('open','closed')"] = (
                    "FAIL: 0 rows — cyberint_alerts must have open/closed records at Stage 1+; "
                    "IIN('open','closed') should match vendor-native pass-through values"
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
                # LOW-003: word-boundary regex prevents "INE" substring-matching ordinary
                # words (e.g., "combined", "determine", "inline"); keep the existing
                # byte-precise mode anchor AND.
                mentions_operator = bool(re.search(r'\b(IEQ|IIN|INE)\b', msg, re.IGNORECASE))
                # NB-3 fix: use the canonical message anchor from error-taxonomy v2.34 /
                # sql_parser.rs: "(IEQ/IIN/INE) are not supported in SQL mode. Use filter mode"
                # The old heuristic used '"|" in msg.lower()' which could spuriously match
                # any unrelated pipe character in the error text. Replaced with a byte-precise
                # case-sensitive anchor: "not supported in SQL mode" is the verbatim string
                # in sql_parser.rs (crates/prism-query/src/sql_parser.rs, 4 rejection sites).
                # F-AUD-P3-LOW-001: no .lower() — the SQL-mode literal is always mixed-case.
                mentions_mode = "not supported in SQL mode" in msg
                if mentions_operator and mentions_mode:
                    results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                        f"PASS: E-QUERY-001; message names IEQ/IIN/INE and byte-precise anchor "
                        f"'not supported in SQL mode' confirmed (sql_parser.rs verbatim): "
                        f"{msg[:120]!r}"
                    )
                else:
                    results["[G4] SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"] = (
                        f"FAIL: E-QUERY-001 returned but canonical message anchor missing — "
                        f"message-template regression (POL-24); "
                        f"operator_named={mentions_operator}, "
                        f"mode_anchor_found={mentions_mode}: {msg[:120]!r}"
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

        # G5 RETIRED (pass-9, per gap-analysis mandate): probe was retargeted to duplicate H6
        # verbatim; H6 is the canonical E-QUERY-002 armis_devices.risk_score probe. ID G5 not
        # reused.

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
            # PARTIAL sweep: query error → GROUP BY normalization cannot be verified → FAIL.
            results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[G6] GROUP BY severity no-fragmentation (canonical Title-case)", body, results):
                pass
            elif rows:
                severities = [r.get("severity", "") for r in rows if r.get("severity") is not None]
                # Check for casing fragmentation: same value in different cases
                sev_lower_list = [s.lower() for s in severities if s]
                has_dup_lower = len(sev_lower_list) != len(set(sev_lower_list))
                # All non-null values should be Title-case (OCSF canonical form)
                known_ocsf = {"high", "medium", "low", "critical", "informational", "unknown", "fatal"}
                non_title = [s for s in severities if s and s.lower() in known_ocsf and s != s.title()]
                # F-AUD-P15-LOW-003: guard against vacuous PASS when all severity values are
                # empty — GROUP BY contract guarantees non-empty buckets for crowdstrike_detections.
                has_nonempty = any(s for s in severities)
                if not has_nonempty:
                    results["[G6] GROUP BY severity no-fragmentation (canonical Title-case)"] = (
                        f"FAIL: {len(rows)} GROUP BY rows but all severity values empty — "
                        f"data-quality regression (Standing Rule 3 §2)"
                    )
                elif has_dup_lower:
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
                # MED-003 (F-AUD-P25): claroty_audit_logs.timestamp IS spec-guaranteed.
                # Spec evidence: crates/prism-sensors/specs/claroty.sensor.toml [[tables]]
                # block for table_name = "audit_logs", column name = "timestamp",
                # column_type = "datetime" (line ~168); DTU confirms:
                # crates/prism-dtu-claroty/src/types.rs ClarotyAuditLogEntry.timestamp field.
                # E-QUERY-038 on this column = schema regression — FAIL immediately.
                # (Old fallback to crowdstrike_detections.created_timestamp removed per
                # F-AUD-P25-MED-003; a silently-swapped fallback masked schema regressions.)
                results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                    f"FAIL: E-QUERY-038 on claroty_audit_logs.timestamp — schema regression; "
                    f"column is spec-guaranteed (claroty.sensor.toml audit_logs [[tables]] "
                    f"column_type=datetime; DTU: ClarotyAuditLogEntry.timestamp); "
                    f"message={msg[:80]!r}"
                )
            else:
                results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                    f"FAIL: {ec}: {msg[:100]!r}"
                )
        else:
            rows = body.get("rows", [])
            # HIGH-001 / LOW-003: assert past-date filter returned >= 1 row before running
            # the far-future counter-check. If rows is empty, the temporal filter is not
            # working (or claroty_audit_logs has no data) — report immediately, FAIL.
            if sensor_errors_gate("[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)", body, results):
                pass
            elif not rows:
                results["[G7] Temporal: RFC-3339 datetime literal in WHERE (ADR-052 §D4 regression)"] = (
                    "FAIL: 0 rows for past-date RFC-3339 filter "
                    "'timestamp > 2020-01-01T00:00:00Z' — data absent or datetime filter "
                    "rejected all rows (ADR-052 §D4 regression cannot be confirmed without data)"
                )
            else:
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
            if sensor_errors_gate("[G8] ADR-051 regression: threat_score output is Int64 not JSON-string", body, results):
                pass
            elif rows:
                first = rows[0]
                ts = first.get("threat_score", "MISSING")
                if ts == "MISSING":
                    results["[G8] ADR-051 regression: threat_score output is Int64 not JSON-string"] = (
                        f"FAIL: threat_score column absent; cols={list(first.keys())[:8]}"
                    )
                elif isinstance(ts, (int, float)) and not isinstance(ts, bool):
                    # LOW-002: bool is subclass of int — must confirm not-bool for genuine numeric
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
                # F-AUD-P13-OBS-001 (POL-24): require invariant anchor "not found in table"
                # in the message — mirrors F5 message-template regression gate.
                if "not found in table" in msg:
                    results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                        f"PASS: E-QUERY-038 + 'not found in table' anchor (POL-24; "
                        f"no Internal error / E-QUERY-034 regression)"
                    )
                else:
                    results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                        f"FAIL: E-QUERY-038 but 'not found in table' absent — "
                        f"message-template regression (POL-24); message={msg[:80]!r}"
                    )
            # NOTE: E-QUERY-034 is redacted to "Internal error; see audit log" (E-INT-001) at the
            # MCP boundary by map_prism_error (error_mapping.rs); this disjunct is future-proofing —
            # the "Internal error" disjunct handles the live path today.
            # MED-006: code-first discipline — only treat "Internal error" in msg as regression
            # evidence when ec is absent (the pre-fix -32000 redacted shape); when ec is present
            # and non-E-QUERY-038, the unexpected error falls through to the else branch below.
            elif ec == "E-QUERY-034" or (not ec and "Internal error" in msg):
                results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                    f"FAIL: REGRESSION — got {ec!r} / 'Internal error' instead of E-QUERY-038; "
                    f"message={msg[:100]!r}"
                )
            elif body.get("rows") is not None and not ec:
                results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                    f"FAIL: query succeeded ({len(body.get('rows', []))} rows) — nonexistent column must error"
                )
            else:
                # PARTIAL sweep: unexpected error code is still a failed state → FAIL.
                results["[H1] E-QUERY-038 pipe mode (original DRIFT shape)"] = (
                    f"FAIL: unexpected {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H1b: E-QUERY-038 filter mode (position 7, no FROM keyword) ───────
        # Syntax: table_name | predicate (no WHERE, no FROM keyword)
        #
        # F-AUD-P24-MED-004 (comment-only — no logic change):
        # The probe "crowdstrike_detections | nonexistent_column_xyz IEQ 'high'" reaches
        # FILTER mode because `filter_parser.rs::is_pipe_mode()` requires a
        # PIPE_STAGE_KEYWORDS token (e.g. WHERE, LIMIT, ORDER BY) immediately after `|` to
        # enter pipe mode; absent that keyword, the parser treats `|` as a filter separator
        # and enters filter mode.  BC-2.11.002 states a broader precedence rule ("any `|`
        # outside string literals → pipe mode") that does not match this code behaviour.
        # If mode-detection is ever aligned to BC-2.11.002's stated rule, this check's
        # probe must be consciously re-targeted (the current input may route differently).
        # The spec-vs-code drift is queued for PO adjudication at cascade close — it is NOT
        # resolved here.
        body, err = query(proc,
            "crowdstrike_detections | nonexistent_column_xyz IEQ 'high'",
            ["org-c"])
        if err:
            results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-038":
                # F-AUD-P13-OBS-001 (POL-24): require invariant anchor "not found in table"
                # in the message — mirrors F5 message-template regression gate.
                if "not found in table" in msg:
                    results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                        f"PASS: E-QUERY-038 + 'not found in table' anchor (POL-24; "
                        f"no regression in filter mode)"
                    )
                else:
                    results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                        f"FAIL: E-QUERY-038 but 'not found in table' absent — "
                        f"message-template regression (POL-24); message={msg[:80]!r}"
                    )
            # NOTE: E-QUERY-034 is redacted to "Internal error; see audit log" (E-INT-001) at the
            # MCP boundary by map_prism_error (error_mapping.rs); this disjunct is future-proofing —
            # the "Internal error" disjunct handles the live path today.
            # MED-006: code-first discipline — only treat "Internal error" in msg as regression
            # evidence when ec is absent (the pre-fix -32000 redacted shape); when ec is present
            # and non-E-QUERY-038, the unexpected error falls through to the else branch below.
            elif ec == "E-QUERY-034" or (not ec and "Internal error" in msg):
                results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                    f"FAIL: REGRESSION — {ec!r} / 'Internal error' instead of E-QUERY-038; "
                    f"message={msg[:100]!r}"
                )
            elif body.get("rows") is not None and not ec:
                results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                    f"FAIL: filter mode query succeeded — nonexistent column must error"
                )
            else:
                # PARTIAL sweep: unexpected error code → FAIL.
                results["[H1b] E-QUERY-038 filter mode (position 7, no FROM)"] = (
                    f"FAIL: unexpected {ec or 'no error'}: {msg[:80]!r}"
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
                # Also check structuredContent.error fields (MED-002: sc fields required)
                sc_dym = sc_err.get("did_you_mean", "") if sc_err else ""
                # LOW-001: guard against non-list available_columns (e.g. None or wrong type).
                # LOW-006 (F-AUD-P26): when _raw_avail is present but not a list, include
                # type(__name__) in the FAIL diagnostic so shape regressions are distinguishable
                # from empty-list (None produces type 'NoneType'; a dict produces 'dict', etc.).
                _raw_avail = sc_err.get("available_columns") if sc_err else None
                if _raw_avail is not None and not isinstance(_raw_avail, list):
                    _sc_avail_type_note = f" (available_columns wrong type: {type(_raw_avail).__name__!r})"
                else:
                    _sc_avail_type_note = ""
                sc_avail = _raw_avail if isinstance(_raw_avail, list) else []
                # F-AUD-P3-MED-002: PASS requires BOTH text anchors AND both sc fields:
                # did_you_mean must equal "severity" exactly (typo "sevrity" → Levenshtein-1
                # correction; mirrors F5 exact-equality discipline) AND available_columns must
                # be non-empty.  available_columns is unconditionally populated for E-QUERY-038
                # so the OR-disjunct (sc_dym != "") was vacuous — the OR never gated.
                # F-AUD-P24-MED-001: AND required; sc_dym exact-equality asserted.
                has_sc_fields = (sc_dym == "severity") and (len(sc_avail) > 0)
                if has_dym_text and has_avail_text and has_sc_fields:
                    results["[H2] E-QUERY-038 did_you_mean + available_columns payload"] = (
                        f"PASS: E-QUERY-038; text anchors confirmed + sc_error populated; "
                        f"sc_error.did_you_mean={sc_dym!r}; "
                        f"sc_error.available_columns count={len(sc_avail)}"
                    )
                elif has_dym_text and has_avail_text and not has_sc_fields:
                    results["[H2] E-QUERY-038 did_you_mean + available_columns payload"] = (
                        f"FAIL: text anchors present but structuredContent.error regression — "
                        f"sc_dym={sc_dym!r} sc_avail count={len(sc_avail)}"
                        f"{_sc_avail_type_note}; "
                        f"F-AUD-P3-MED-002: sc fields required for structured UX payload"
                    )
                else:
                    results["[H2] E-QUERY-038 did_you_mean + available_columns payload"] = (
                        f"FAIL: E-QUERY-038 but payload anchors missing — "
                        f"has_dym_text={has_dym_text}, has_avail_text={has_avail_text}, "
                        f"has_sc_fields={has_sc_fields}; "
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
            if sensor_errors_gate("[H3] INE operator: severity INE 'medium' (excludes Medium rows)", body, results):
                pass
            elif not rows:
                results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                    "FAIL: 0 rows — INE should return Critical rows (seed-200: 5 Critical + 15 Medium)"
                )
            else:
                # F-AUD-P19-MED-001: null-leak guard — INE filter must not return rows with
                # NULL/empty severity; SQL lower(NULL) predicate is UNKNOWN (never matches),
                # so any NULL/empty-severity row leaking through is a regression.
                null_leak_count = sum(1 for r in rows if not r.get("severity"))
                severities = [r.get("severity", "") for r in rows]
                # F-AUD-P12-LOW-001: guard against vacuous-True all() when every severity
                # value is empty — all(... if s) over all-empty list is vacuously True and
                # would falsely PASS (Standing Rule 3 §2 data-quality regression).
                has_nonempty = any(s for s in severities)
                has_medium = any(s and s.lower() == "medium" for s in severities)
                all_critical = all(s and s.lower() == "critical" for s in severities if s)
                if null_leak_count > 0:
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"FAIL: filter leaked {null_leak_count} NULL/empty-severity rows — "
                        f"WHERE/IEQ/IIN regression (SQL: lower(NULL) predicate is UNKNOWN, must not match); "
                        f"total_rows={len(rows)}"
                    )
                elif not has_nonempty:
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"FAIL: {len(rows)} rows but all severity values empty — "
                        f"data-quality regression (Standing Rule 3 §2); "
                        f"severities={list(set(severities))!r}"
                    )
                elif has_medium:
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"FAIL: Medium rows leaked through INE filter; severities={list(set(severities))!r}"
                    )
                elif all_critical:
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"PASS: {len(rows)} rows; all severity='Critical'; zero Medium rows"
                    )
                else:
                    # PARTIAL sweep: unexpected severity set (neither all-Critical nor has Medium).
                    # seed-200 guarantees Critical+Medium only; other severities indicate drift.
                    results["[H3] INE operator: severity INE 'medium' (excludes Medium rows)"] = (
                        f"FAIL: unexpected severity set — seed-200 guarantees Critical+Medium only; "
                        f"rows={len(rows)}; severities={list(set(severities))!r}"
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
                # F-AUD-P3-MED-001: tighten anchor — BOTH "cannot be interpreted" AND
                # "Expected RFC-3339 format" must be present.  The bare "RFC-3339" anchor was
                # too loose — any RFC-3339 mention (e.g. help text) could match.  The fuller
                # phrase "Expected RFC-3339 format" pins the pedagogical fixture text from the
                # error-taxonomy.md §E-QUERY-041 message_template byte-form:
                #   "…cannot be interpreted as a UTC timestamp. Expected RFC-3339 format
                #    with UTC offset…" (PrismError::TemporalLiteralUnparseable Display,
                #   error.rs; POL-24 / TD-VSDD-059).
                # F-AUD-P24-LOW-001: second anchor upgraded from bare "RFC-3339" to
                # "Expected RFC-3339 format" for message-template regression precision.
                has_rfc_hint = "cannot be interpreted" in msg and "Expected RFC-3339 format" in msg
                # F-AUD-P1-MED-010: PASS requires E-QUERY-041 AND the RFC-3339 pedagogical hint.
                # E-QUERY-041 without both anchors means message-template regression (POL-24).
                if has_rfc_hint:
                    results["[H4] E-QUERY-041: date-only literal rejected (ADR-052 §D4)"] = (
                        f"PASS: E-QUERY-041; both 'cannot be interpreted' and 'RFC-3339' confirmed; "
                        f"message={msg[:80]!r}"
                    )
                else:
                    results["[H4] E-QUERY-041: date-only literal rejected (ADR-052 §D4)"] = (
                        f"FAIL: E-QUERY-041 returned but required anchors absent — "
                        f"message-template regression (POL-24); "
                        f"'cannot be interpreted' present={'cannot be interpreted' in msg}, "
                        f"'RFC-3339' present={'RFC-3339' in msg}; "
                        f"message={msg[:120]!r}"
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
                # F-AUD-P4-MED-003: require POL-24 canonical E-QUERY-042 GroupBy message
                # template substrings (error-taxonomy.md E-QUERY-042 GroupBy position):
                # "E-QUERY-042: GROUP BY expects a column reference, not a literal constant.
                # '...' is a date-shaped literal — grouping by a constant has no effect and
                # is almost certainly a query mistake."
                anchor1 = "GROUP BY expects a column reference"
                anchor2 = "grouping by a constant has no effect"
                if anchor1 in msg and anchor2 in msg:
                    results["[H5] E-QUERY-042: temporal literal in GROUP BY (ADR-052 §D4)"] = (
                        f"PASS: E-QUERY-042 — temporal literal in GROUP BY arm rejected "
                        f"with canonical template (POL-24 anchors confirmed); "
                        f"message={msg[:80]!r}"
                    )
                else:
                    missing = []
                    if anchor1 not in msg:
                        missing.append(repr(anchor1))
                    if anchor2 not in msg:
                        missing.append(repr(anchor2))
                    results["[H5] E-QUERY-042: temporal literal in GROUP BY (ADR-052 §D4)"] = (
                        f"FAIL: E-QUERY-042 received but message-template regression "
                        f"(POL-24): missing anchor(s) {', '.join(missing)}; "
                        f"message={msg[:120]!r}"
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

        # ── H5b: E-QUERY-042 ORDER BY arm — Literal::Timestamp in ORDER BY (ADR-052 §D4 arm 7) ─
        # RFC-3339 literal '2026-07-01T00:00:00Z' parses as Literal::Timestamp in the SQL AST.
        # The check_temporal_literals AST-walker detects it in ORDER BY position → arm 7 → E-QUERY-042.
        # DEFECT-EQUERY042-GROUPBY-DEADARM-001 extended the walker to cover the OrderBy arm.
        # POL-24 anchors from error.rs TemporalLiteralPosition::OrderBy::format_message():
        #   "ORDER BY expects a column reference, not a literal constant."
        #   "ordering by a constant has no effect"
        body, err = query(proc,
            "SELECT device_id FROM crowdstrike_detections ORDER BY '2026-07-01T00:00:00Z'",
            ["org-c"])
        if err:
            results["[H5b] E-QUERY-042: Timestamp literal in ORDER BY arm (ADR-052 §D4)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-042":
                anchor1 = "ORDER BY expects a column reference"
                anchor2 = "ordering by a constant has no effect"
                if anchor1 in msg and anchor2 in msg:
                    results["[H5b] E-QUERY-042: Timestamp literal in ORDER BY arm (ADR-052 §D4)"] = (
                        f"PASS: E-QUERY-042 — RFC-3339 literal in ORDER BY arm rejected "
                        f"with canonical template (POL-24 anchors confirmed); "
                        f"message={msg[:80]!r}"
                    )
                else:
                    missing = []
                    if anchor1 not in msg:
                        missing.append(repr(anchor1))
                    if anchor2 not in msg:
                        missing.append(repr(anchor2))
                    results["[H5b] E-QUERY-042: Timestamp literal in ORDER BY arm (ADR-052 §D4)"] = (
                        f"FAIL: E-QUERY-042 received but message-template regression "
                        f"(POL-24): missing anchor(s) {', '.join(missing)}; "
                        f"message={msg[:120]!r}"
                    )
            elif body.get("rows") is not None and not ec:
                results["[H5b] E-QUERY-042: Timestamp literal in ORDER BY arm (ADR-052 §D4)"] = (
                    "FAIL: ORDER BY literal accepted (should be E-QUERY-042 OrderBy arm)"
                )
            else:
                results["[H5b] E-QUERY-042: Timestamp literal in ORDER BY arm (ADR-052 §D4)"] = (
                    f"FAIL: expected E-QUERY-042, got {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H5c: E-QUERY-042 NonColumnLhsComparison arm — function-call LHS (ADR-052 §D4 arm 4) ─
        # Query: 'FROM crowdstrike_detections | where lower(device_id) = '2026-06-24''
        # lower(device_id) is a non-Field LHS (function call); '2026-06-24' is a date-only
        # RawTemporalLiteral. Arm 4: RawTemporalLiteral where LHS is a function/compound
        # expression → E-QUERY-042 NonColumnLhsComparison.
        # MED-004 (F-AUD-P26): device_id is a String column in crowdstrike_detections —
        # verified against crates/prism-sensors/specs/crowdstrike.sensor.toml [[tables]]
        # detections block (name="device_id", column_type="string").  The previous probe used
        # lower(hostname), but hostname is NOT a column in the detections table (it lives in
        # the devices table); using a non-existent column would cause E-QUERY-038 to fire
        # before the E-QUERY-042 gate, making this check robust only under the current
        # plan-time gate ordering.  lower(device_id) exercises the correct arm 4 path
        # regardless of gate ordering.
        # POL-24 anchors from error.rs TemporalLiteralPosition::NonColumnLhsComparison::format_message():
        #   "A date-like literal compared against a computed expression cannot be"
        #   "type-checked at plan time"
        body, err = query(proc,
            "FROM crowdstrike_detections\n| where lower(device_id) = '2026-06-24'\n| limit 5",
            ["org-c"])
        if err:
            results["[H5c] E-QUERY-042: date-only literal vs function-call LHS (ADR-052 §D4)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-042":
                anchor1 = "A date-like literal compared against a computed expression cannot be"
                anchor2 = "type-checked at plan time"
                if anchor1 in msg and anchor2 in msg:
                    results["[H5c] E-QUERY-042: date-only literal vs function-call LHS (ADR-052 §D4)"] = (
                        f"PASS: E-QUERY-042 — NonColumnLhsComparison arm rejected "
                        f"with canonical template (POL-24 anchors confirmed); "
                        f"message={msg[:100]!r}"
                    )
                else:
                    missing = []
                    if anchor1 not in msg:
                        missing.append(repr(anchor1))
                    if anchor2 not in msg:
                        missing.append(repr(anchor2))
                    results["[H5c] E-QUERY-042: date-only literal vs function-call LHS (ADR-052 §D4)"] = (
                        f"FAIL: E-QUERY-042 received but message-template regression "
                        f"(POL-24): missing anchor(s) {', '.join(missing)}; "
                        f"message={msg[:120]!r}"
                    )
            elif body.get("rows") is not None and not ec:
                results["[H5c] E-QUERY-042: date-only literal vs function-call LHS (ADR-052 §D4)"] = (
                    "FAIL: date-only vs function-call LHS accepted (should be E-QUERY-042 NonColumnLhsComparison arm)"
                )
            else:
                results["[H5c] E-QUERY-042: date-only literal vs function-call LHS (ADR-052 §D4)"] = (
                    f"FAIL: expected E-QUERY-042, got {ec or 'no error'}: {msg[:80]!r}"
                )

        # ── H6: E-QUERY-002 via armis_devices.risk_score (integer column) ─────
        # armis_devices.risk_score is Integer-typed. IEQ must reject with E-QUERY-002.
        # Do NOT assert sibling suggestion (risk_score has no OCSF string sibling).
        # This is the canonical E-QUERY-002 probe; G5 (pass-9 retired duplicate) is gone — ID not reused.
        body, err = query(proc,
            "FROM armis_devices\n| where risk_score IEQ 'high'\n| limit 5",
            ["org-c"])
        if err:
            results["[H6] E-QUERY-002 via armis_devices.risk_score (integer column)"] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-002":
                # F-AUD-P3-HIGH-003: PASS requires operator hint "does not support operator"
                # from error-taxonomy.md E-QUERY-002 canonical template.
                # Without the hint → message-template regression (POL-24) → FAIL.
                has_operator_hint = "does not support operator" in msg
                if has_operator_hint:
                    results["[H6] E-QUERY-002 via armis_devices.risk_score (integer column)"] = (
                        f"PASS: E-QUERY-002 + operator hint 'does not support operator' confirmed; "
                        f"message={msg[:100]!r}"
                    )
                else:
                    results["[H6] E-QUERY-002 via armis_devices.risk_score (integer column)"] = (
                        f"FAIL: E-QUERY-002 returned but operator hint absent — "
                        f"message-template regression (POL-24): "
                        f"'does not support operator' not in message; message={msg[:120]!r}"
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
        # F-AUD-P29-MED-001: explicit AS aliases force unqualified output keys so
        # r.get("risk_score") works regardless of DataFusion's internal key emission.
        body, err = query(proc,
            "SELECT d.device_id AS device_id, a.risk_score AS risk_score "
            "FROM crowdstrike_devices d "
            "JOIN armis_devices a ON d.device_id = a.device_id LIMIT 5",
            ["org-c"])
        if err:
            results[_H7_RESULT_KEY] = f"FAIL: {err}"
        elif body.get("error_code"):
            ec = body.get("error_code", "")
            results[_H7_RESULT_KEY] = (
                f"FAIL: {ec}: {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate(_H7_RESULT_KEY, body, results):
                pass
            elif not rows:
                results[_H7_RESULT_KEY] = (
                    "FAIL: 0 rows — seed-200 guarantees 50 device IDs overlap between tables"
                )
            elif len(rows) != 5:
                # MED-002: tighten to exactly the LIMIT count (5).  seed-200 has 50 overlapping
                # device IDs between crowdstrike_devices and armis_devices; LIMIT 5 must return
                # exactly 5 rows.  A partial count (1-4) indicates a data-quality gap and would
                # give H8's evidence-chain a weak foundation — only the full-strength 5-row result
                # confirms the JOIN machinery is operating correctly.
                results[_H7_RESULT_KEY] = (
                    f"FAIL: {len(rows)} rows returned, expected exactly 5 (LIMIT 5; "
                    f"seed-200 guarantees >= 50 overlapping device IDs — "
                    f"partial count indicates data-quality gap; MED-002)"
                )
            else:
                # F-AUD-P29-MED-001: primary defense is the AS aliases in the query above;
                # tolerate qualified key "a.risk_score" as defense-in-depth so key-shape
                # mismatches produce a diagnosable row_keys dump rather than a silent empty.
                scores = [
                    r.get("risk_score") if r.get("risk_score") is not None else r.get("a.risk_score")
                    for r in rows
                    if r.get("risk_score") is not None or r.get("a.risk_score") is not None
                ]
                # F-AUD-P18-MED-002: guard against all-null risk_score — vacuous "all numeric"
                # PASS would then feed H8's attribution gate on stale data (mirrors H3's
                # has_nonempty pattern; data-quality regression, Standing Rule 3 §2).
                if not scores:
                    _h7_row_keys = list(rows[0].keys()) if rows else []
                    results[_H7_RESULT_KEY] = (
                        f"FAIL: {len(rows)} joined rows but all risk_score null — "
                        f"expected >= 1 numeric risk_score value from JOIN "
                        f"(data-quality regression, Standing Rule 3 §2); "
                        f"row_keys={_h7_row_keys!r}"
                    )
                else:
                    # MED-004: all risk_score values must be numeric (LOW-002: bool exclusion)
                    non_numeric = [s for s in scores if isinstance(s, bool) or not isinstance(s, (int, float))]
                    if non_numeric:
                        results[_H7_RESULT_KEY] = (
                            f"FAIL: {len(rows)} joined rows but risk_score contains non-numeric values — "
                            f"expected Int64/Float64 (ADR-051 D1); "
                            f"non_numeric sample={non_numeric[:3]!r}; all_scores={scores[:5]!r}"
                        )
                    else:
                        results[_H7_RESULT_KEY] = (
                            f"PASS: {len(rows)} joined rows (== 5, LIMIT confirmed); "
                            f"risk_score values={scores[:5]} (all numeric)"
                        )

        # ── H8: HEAD-JOIN fail-open — bare unknown column in JOIN ─────────────
        # BC-2.11.016 §HEAD-JOIN SUSPENSION RULE: bare-column reference in JOIN → fail-open
        # (E-QUERY-034 or controlled rejection, NEVER E-QUERY-038).
        # TD-VSDD-060 anchor: map_prism_error -32000 catch-all display, error_mapping.rs
        # ("Internal error; see audit log"). Future refactors of that display string must
        # sweep this acceptance check (F-AUD-P3-LOW-002).
        body, err = query(proc,
            "SELECT totally_unknown_col FROM crowdstrike_devices d "
            "JOIN armis_devices a ON d.device_id = a.device_id LIMIT 5",
            ["org-c"])
        if err:
            # F-AUD-P5-LOW-001: RPC-level -32000 Internal error is a spec-acceptable
            # FP-001 fail-open outcome (parse_envelope returns body={} on RPC error path).
            # Mirror H18's filter pattern — accept "-32000" + "Internal error" as PASS.
            # Timeouts, JSON errors, other RPC codes still FAIL.
            # TD-VSDD-060 anchor: map_prism_error -32000 catch-all display, error_mapping.rs
            # ("Internal error; see audit log"); future refactors of that display string
            # must sweep this acceptance check.
            err_str = str(err)
            # HIGH-003 sibling sweep (F-AUD-P25): anchor -32000 check to match H18's discipline.
            # parse_envelope produces "RPC error {code}: {message}" so the string always starts
            # with "RPC error -32000" when the server returns code=-32000. Substring "-32000"
            # could false-match a message body like "E-QUERY-32000" on other code paths.
            if err_str.startswith("RPC error -32000") and "Internal error" in err_str:
                # F-AUD-P10-MED-001: attribute -32000 to HEAD-JOIN fail-open only when
                # JOIN-machinery is verified by H7. Without H7 PASS, the -32000 could be
                # any engine failure — not specifically BC-2.11.016 §HEAD-JOIN SUSPENSION RULE (PER-REFERENCE SCOPING; EC-11-074/075/076). H7 must have
                # PASSed in this run (earlier in this same results dict) to establish that
                # the JOIN path itself is functional before we accept its fail-open variant.
                # F-AUD-P25-MED-002: gate intentionally strict — false-FAIL preferred over
                # false-PASS in fail-open attribution (a single root cause in JOIN machinery
                # can produce both H7 and H8 FAILs simultaneously).
                _h7_key = _H7_RESULT_KEY  # OBS-004: single definition above prevents drift
                _h7_result = results.get(_h7_key, "")
                if _h7_result.startswith("PASS"):
                    # F-AUD-P30-LOW-002: PASS-ATTRIBUTED — the -32000 "Internal error" message is
                    # indistinguishable from a generic QueryExecutionFailed at the MCP boundary.
                    # map_prism_error (crates/prism-mcp/src/error_mapping.rs) redacts ALL
                    # QueryExecutionFailed variants to the same "Internal error; see audit log"
                    # string. We attribute this to HEAD-JOIN fail-open (FP-001) on the basis of:
                    # (a) H7 PASS confirms JOIN machinery is functional, and
                    # (b) the query uses a bare unknown column in a JOIN — the pattern for FP-001.
                    # This is probabilistic attribution, not direct fail-open message confirmation.
                    results[_H8_RESULT_KEY] = (
                        "PASS-ATTRIBUTED: -32000 Internal error + H7 healthy; "
                        "HEAD-JOIN fail-open message indistinguishable from generic QueryExecutionFailed "
                        "at MCP boundary (map_prism_error redacts all QueryExecutionFailed to "
                        "'Internal error; see audit log'); "
                        "spec-sanctioned FP-001 outcome (BC-2.11.016 §HEAD-JOIN SUSPENSION RULE); "
                        "H7 JOIN-machinery evidence present"
                    )
                else:
                    # MED-002 (F-AUD-P25): improved diagnostic — single root cause message.
                    results[_H8_RESULT_KEY] = (
                        f"FAIL: H7 non-PASS → fail-open attribution withheld; "
                        f"investigate H7's failure first (single root cause may produce both FAILs); "
                        f"H7 result={_h7_result[:80]!r}"
                    )
            else:
                results[_H8_RESULT_KEY] = f"FAIL: {err}"
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            rows = body.get("rows", [])
            if rows:
                results[_H8_RESULT_KEY] = (
                    f"FAIL: query returned {len(rows)} rows with unknown col (should reject)"
                )
            elif ec == "E-QUERY-038":
                results[_H8_RESULT_KEY] = (
                    f"FAIL: E-QUERY-038 fired for bare unknown col in JOIN — "
                    f"HEAD-JOIN fail-open (FP-001) should suppress E-QUERY-038 here"
                )
            # NOTE: E-QUERY-034 is redacted to "Internal error; see audit log" (E-INT-001) at the
            # MCP boundary by map_prism_error (error_mapping.rs); this disjunct is future-proofing —
            # the "Internal error" disjunct handles the live path today.
            # MED-006: code-first discipline — only treat "Internal error" in msg as a
            # controlled-rejection signal when ec is absent (the -32000 redacted shape);
            # an unexpected ec that happens to mention "Internal error" must not false-PASS H8.
            elif ec == "E-QUERY-034" or (not ec and "Internal error" in msg):
                # F-AUD-P1-MED-002: only E-QUERY-034 or Internal error (with no ec) PASSes here.
                # The former third disjunct `(ec and ec != "E-QUERY-038")` accepted any
                # error code — removed; unexpected error codes must be investigated.
                # F-AUD-P15-MED-001: require H7 JOIN-machinery evidence before attributing
                # in-band E-QUERY-034 / "Internal error" to HEAD-JOIN fail-open (FP-001).
                # Without H7 PASS, the controlled rejection could be an unrelated engine
                # failure — not specifically BC-2.11.016 §HEAD-JOIN SUSPENSION RULE.
                _h7_key = _H7_RESULT_KEY
                _h7_result = results.get(_h7_key, "")
                if _h7_result.startswith("PASS"):
                    # F-AUD-P30-LOW-002: PASS-ATTRIBUTED — "Internal error; see audit log"
                    # (in-band path) is emitted by map_prism_error (error_mapping.rs) for ALL
                    # QueryExecutionFailed variants; the message is indistinguishable from any
                    # other engine failure at the MCP boundary. Attribution to HEAD-JOIN fail-open
                    # (FP-001) is probabilistic: H7 PASS + bare-unknown-col-in-JOIN query shape.
                    results[_H8_RESULT_KEY] = (
                        f"PASS-ATTRIBUTED: {ec or 'Internal error (no ec)'} + H7 healthy; "
                        "HEAD-JOIN fail-open message indistinguishable from generic QueryExecutionFailed "
                        "at MCP boundary (map_prism_error redacts all QueryExecutionFailed to "
                        "'Internal error; see audit log'); "
                        "spec-sanctioned FP-001 outcome (BC-2.11.016 §HEAD-JOIN SUSPENSION RULE); "
                        "H7 JOIN-machinery evidence present"
                    )
                else:
                    # MED-002 (F-AUD-P25): improved diagnostic — single root cause message.
                    # F-AUD-P25-MED-002: gate intentionally strict (false-FAIL preferred over
                    # false-PASS in fail-open attribution); adjudicated KEEP at P25.
                    results[_H8_RESULT_KEY] = (
                        f"FAIL: H7 non-PASS → fail-open attribution withheld; "
                        f"investigate H7's failure first (single root cause may produce both FAILs); "
                        f"H7 result={_h7_result[:80]!r}"
                    )
            elif not ec and not rows:
                # FAIL-DEFECT per BC-2.11.016 §HEAD-JOIN SUSPENSION RULE: fail-open defers to "execution-time DataFusion error"; 0 rows + no error = swallowed DataFusion schema error, not a sanctioned outcome
                results[_H8_RESULT_KEY] = (
                    "FAIL: 0 rows, no error — swallowed DataFusion schema error per BC-2.11.016 "
                    "§HEAD-JOIN SUSPENSION RULE; fail-open path defers to execution-time error, "
                    "not silent 0-row success"
                )
            else:
                # Unexpected error code — neither E-QUERY-034 nor Internal error
                results[_H8_RESULT_KEY] = (
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
            if sensor_errors_gate("[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)", body, results):
                pass
            elif not rows:
                results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = (
                    "FAIL: 0 rows — seed-200 guarantees Critical detections"
                )
            else:
                # F-AUD-P19-MED-001: null-leak guard — SqlPipe IEQ filter must not return rows
                # with NULL/empty severity; SQL lower(NULL) predicate is UNKNOWN (never matches).
                null_leak_count = sum(1 for r in rows if not r.get("severity"))
                severities = [r.get("severity", "") for r in rows]
                non_critical = [s for s in severities if s and s.lower() != "critical"]
                # F-AUD-P15-LOW-003: guard against vacuous PASS when all severity values are
                # empty — SqlPipe IEQ filter contract guarantees severity='Critical' rows.
                has_nonempty = any(s for s in severities)
                if null_leak_count > 0:
                    results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = (
                        f"FAIL: filter leaked {null_leak_count} NULL/empty-severity rows — "
                        f"WHERE/IEQ/IIN regression (SQL: lower(NULL) predicate is UNKNOWN, must not match); "
                        f"total_rows={len(rows)}"
                    )
                elif not has_nonempty:
                    results["[H9] SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"] = (
                        f"FAIL: {len(rows)} rows but all severity values empty — "
                        f"data-quality regression (Standing Rule 3 §2)"
                    )
                elif non_critical:
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
                # F-AUD-P4-MED-004: require POL-24 canonical E-QUERY-040 message template
                # substrings (error-taxonomy.md E-QUERY-040):
                # "E-QUERY-040: redundant row limit. This query caps rows in two places:
                # a SQL `LIMIT {sql_limit}` in the head and a row-capping `| limit`/
                # `| tail` pipe stage (cap: {pipe_limit})."
                anchor1 = "redundant row limit"
                anchor2 = "caps rows in two places"
                if anchor1 in msg and anchor2 in msg:
                    results["[H10] E-QUERY-040: SQL LIMIT + pipe | limit (dual-limit rejected)"] = (
                        f"PASS: E-QUERY-040 — dual row-limit cap rejected with canonical "
                        f"template (POL-24 anchors confirmed); message={msg[:80]!r}"
                    )
                else:
                    missing = []
                    if anchor1 not in msg:
                        missing.append(repr(anchor1))
                    if anchor2 not in msg:
                        missing.append(repr(anchor2))
                    results["[H10] E-QUERY-040: SQL LIMIT + pipe | limit (dual-limit rejected)"] = (
                        f"FAIL: E-QUERY-040 received but message-template regression "
                        f"(POL-24): missing anchor(s) {', '.join(missing)}; "
                        f"message={msg[:120]!r}"
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
            if sensor_errors_gate("[H11] stats grammar: count() as cnt by severity", body, results):
                pass
            elif not rows:
                results["[H11] stats grammar: count() as cnt by severity"] = (
                    "FAIL: 0 rows from stats — seed-200 guarantees Critical and Medium buckets"
                )
            else:
                severities_found = {r.get("severity", "") for r in rows}
                expected = {"Critical", "Medium"}
                if expected == severities_found:
                    # F-AUD-P7-MED-001 + LOW-003 + LOW-007: require EXACT bucket match AND
                    # exact seed-200 DTU counts (Critical=5, Medium=15). The old
                    # issubset-fallback PASS branch is removed — extra buckets FAIL.
                    cnts = {r.get("severity"): r.get("cnt") for r in rows}
                    crit_cnt = cnts.get("Critical")
                    med_cnt = cnts.get("Medium")
                    if crit_cnt == 5 and med_cnt == 15:
                        results["[H11] stats grammar: count() as cnt by severity"] = (
                            f"PASS: exactly 2 severity buckets; Critical={crit_cnt}, "
                            f"Medium={med_cnt} (seed-200 counts confirmed)"
                        )
                    else:
                        results["[H11] stats grammar: count() as cnt by severity"] = (
                            f"FAIL: seed-200 count contract violated — "
                            f"expected Critical=5 Medium=15, "
                            f"got Critical={crit_cnt} Medium={med_cnt}"
                        )
                elif expected.issubset(severities_found):
                    # Extra buckets beyond {Critical, Medium} — seed-200 DTU contract guarantees
                    # EXACTLY these two buckets. Any additional bucket is a FAIL.
                    extra = sorted(severities_found - expected)
                    results["[H11] stats grammar: count() as cnt by severity"] = (
                        f"FAIL: extra severity buckets beyond seed-200 contract "
                        f"{{'Critical', 'Medium'}} — extra={extra!r}; "
                        f"got={sorted(severities_found)!r}"
                    )
                else:
                    # PARTIAL sweep: Critical and/or Medium buckets absent — seed-200 guarantees both.
                    results["[H11] stats grammar: count() as cnt by severity"] = (
                        f"FAIL: Critical and/or Medium buckets absent from stats result — "
                        f"seed-200 guarantees both; buckets={sorted(severities_found)!r}"
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
            if sensor_errors_gate("[H12] Multi-client fan-out: org-a + org-c CrowdStrike", body, results):
                pass
            elif not rows:
                results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                    "FAIL: 0 rows from multi-client query"
                )
            else:
                # F-AUD-P5-LOW-003: restrict seed-segment detection to ID-bearing columns only.
                # Scanning ALL string columns risks false-positives from columns whose values
                # happen to contain "-100-" or "-200-" substrings unrelated to seed identity.
                # CONSCIOUS-UPDATE (F-AUD-P32-LOW-004): device_id and detection_id are the
                # two ID-format columns in crowdstrike_detections. Their formats are generated
                # by org_slug() + seed + ordinal in generator.rs (make_device / make_detection
                # functions): "dev-{org_slug}-{seed}-{n}" for device_id and
                # "alert-{org_slug}-{seed}-{n}" for detection_id. These columns are declared
                # in crates/prism-sensors/specs/crowdstrike.sensor.toml [[tables]] detections
                # block. If either column is renamed there, update this whitelist accordingly.
                _id_cols = {"device_id", "detection_id"}
                # LOW-001 (F-AUD-P22): per-row detection — a row counts for org-a iff
                # -100- appears in that row's device_id or detection_id; similarly for
                # -200-.  Concatenate-then-substring risks cross-row false-matches
                # (e.g., an id ending "-10" adjacent to one starting "0-" joins to
                # "-100-" across the whitespace separator).
                # LOW-004 (F-AUD-P25): anchor with regex — seed segment must be followed
                # by an ordinal digit per DTU ID pattern dev-<hex>-<seed>-<n>.
                # re.search(r'-100-\d', v) matches "-100-0", "-100-1" etc but NOT "-100x"
                # or a value that merely contains "100" without the surrounding dashes+digit.
                # TD-VSDD-060 sibling sweep: same anchor applied at D1 line ~1780
                # (has_seed_200 in armis_devices probe) in same commit.
                has_100 = any(
                    k in _id_cols and isinstance(v, str) and re.search(r'-100-\d', v)
                    for r in rows for k, v in r.items()
                )
                has_200 = any(
                    k in _id_cols and isinstance(v, str) and re.search(r'-200-\d', v)
                    for r in rows for k, v in r.items()
                )
                if has_100 and has_200:
                    # LOW-001: assert len(rows) >= 25 — org-a has 5 detections (seed-100)
                    # and org-c has 20 (seed-200); combined fan-out with limit 40 must
                    # return all 25. Fewer rows means fan-out dropped data silently.
                    if len(rows) < 25:
                        results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                            f"FAIL: both seed segments present but only {len(rows)} rows "
                            f"(expected >= 25: org-a=5 + org-c=20); "
                            f"fan-out may have dropped rows silently (LOW-001)"
                        )
                    else:
                        results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                            f"PASS: {len(rows)} rows (>= 25 confirmed); "
                            f"both -100- (org-a) and -200- (org-c) seeds present; "
                            f"sensor_errors=[]"
                        )
                else:
                    # sensor_errors diagnostics are already surfaced by sensor_errors_gate
                    # (above); reaching this branch guarantees sensor_errors=[] (F-AUD-P40-LOW-002).
                    results["[H12] Multi-client fan-out: org-a + org-c CrowdStrike"] = (
                        f"FAIL: missing seed segments — has_100={has_100}, has_200={has_200}; "
                        f"total_rows={len(rows)}"
                    )

        # ── H13: Prompts — client_overview and cross_client_status ───────────
        # Same hang/arg-validation class that A16/A17/A18 guard for the other 3 prompts:
        # A17 (query_tutorial) and A18 (investigate_host) provide explicit hang guards;
        # A16 (triage_alerts) provides content validation with implicit 5s hang guard.
        # H13a (client_overview) and H13b (cross_client_status) cover the remaining 2.
        t0 = time.time()
        res_co, err_co = prompt_get(proc, "client_overview", {"client_id": "org-c"}, timeout=5.0)
        elapsed_co = time.time() - t0
        if err_co:
            results["[H13a] client_overview prompt returns promptly"] = f"FAIL: {err_co} ({elapsed_co:.2f}s)"
        elif elapsed_co > 3.0:
            results["[H13a] client_overview prompt returns promptly"] = f"FAIL: took {elapsed_co:.2f}s"
        else:
            msgs = res_co.get("messages", []) if res_co else []
            if len(msgs) >= 1:
                results["[H13a] client_overview prompt returns promptly"] = (
                    f"PASS: {elapsed_co:.2f}s; {len(msgs)} message(s)"
                )
            else:
                results["[H13a] client_overview prompt returns promptly"] = (
                    f"FAIL: prompt returned no messages ({len(msgs)} messages)"
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
            if len(msgs) >= 1:
                results["[H13b] cross_client_status prompt returns promptly"] = (
                    f"PASS: {elapsed_ccs:.2f}s; {len(msgs)} message(s)"
                )
            else:
                results["[H13b] cross_client_status prompt returns promptly"] = (
                    f"FAIL: prompt returned no messages ({len(msgs)} messages)"
                )

        # ── H14a: resources/read — prism://config/clients (3-org visibility) ──
        # F-AUD-P3-MED-004: split composite H14 into three independent result keys.
        # F-AUD-P1-OBS-003: assertion narrative claims 3-org visibility; require all 3.
        res_cc, err_cc = resource_read(proc, "prism://config/clients", timeout=10.0)
        if err_cc:
            results["[H14a] resources/read: prism://config/clients — 3-org visibility"] = (
                f"FAIL: {err_cc}"
            )
        else:
            t_cc = (res_cc.get("contents") or [{}])[0].get("text", "")
            # F-AUD-P10-MED-002: parse the config/clients JSON and enumerate actual
            # client_id values; set-difference against expected set — mirrors H14d's
            # discipline of explicit field enumeration over substring scan.
            # Response shape: JSON array of ClientInventoryEntry objects (resources.rs
            # line 757: ClientInventoryEntry { client_id: String, ... }); field is "client_id".
            try:
                _clients_arr = json.loads(t_cc)
                if not isinstance(_clients_arr, list):
                    results["[H14a] resources/read: prism://config/clients — 3-org visibility"] = (
                        f"FAIL: expected JSON array from config/clients, "
                        f"got {type(_clients_arr).__name__}: {t_cc[:80]!r}"
                    )
                else:
                    _actual_ids = {e.get("client_id", "") for e in _clients_arr
                                   if isinstance(e, dict)}
                    _expected_ids = {"org-a", "org-b", "org-c"}
                    _missing_ids = _expected_ids - _actual_ids
                    _extra_ids = _actual_ids - _expected_ids
                    # LOW-002 (F-AUD-P25): exact-set equality — missing OR extra orgs FAIL.
                    # Mirroring A2's discipline: tolerating extra entries hides unintended
                    # client registrations and weakens multi-tenant isolation evidence.
                    # CONSCIOUS-UPDATE: if the demo config adds a new org, update
                    # _expected_ids in the same change.
                    if _missing_ids or _extra_ids:
                        results["[H14a] resources/read: prism://config/clients — 3-org visibility"] = (
                            f"FAIL: exact client_id set mismatch — "
                            + (f"missing={sorted(_missing_ids)!r}; " if _missing_ids else "")
                            + (f"extra={sorted(_extra_ids)!r}; " if _extra_ids else "")
                            + f"actual={sorted(_actual_ids)!r}"
                        )
                    else:
                        results["[H14a] resources/read: prism://config/clients — 3-org visibility"] = (
                            f"PASS: exactly 3 orgs (org-a, org-b, org-c) confirmed as client_id "
                            f"in config/clients JSON array ({len(_clients_arr)} entries)"
                        )
            except (json.JSONDecodeError, ValueError) as _h14a_err:
                results["[H14a] resources/read: prism://config/clients — 3-org visibility"] = (
                    f"FAIL: non-JSON response from config/clients: {_h14a_err}; "
                    f"body={t_cc[:80]!r}"
                )

        # ── H14b: resources/read — prism://sensors/health (CRIT-001 corrected) ─
        # F-AUD-P3-CRIT-001: the resource shape is NOT {overall_status, sensors: [...]}.
        # render_sensors_health_resource in crates/prism-mcp/src/resources.rs emits:
        #   populated: {"clients": {client_id: {"sensors": {sensor_id: SensorHealthResult}}}, "stale": bool}
        #   empty cache: {"status": "unknown", "message": "Run check_sensor_health ..."}
        # The old assertion (overall_status + sensors list) was written for the TOOL shape
        # (check_sensor_health tool), not the RESOURCE shape — false-positive on every run.
        # PASS requires the populated form (clients dict + stale key); requires A22 first.
        res_sh, err_sh = resource_read(proc, "prism://sensors/health", timeout=10.0)
        if err_sh:
            results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                f"FAIL: {err_sh}"
            )
        else:
            t_sh = (res_sh.get("contents") or [{}])[0].get("text", "")
            try:
                sh_obj = json.loads(t_sh)
                if not isinstance(sh_obj, dict):
                    results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                        f"FAIL: non-dict JSON: {t_sh[:80]!r}"
                    )
                elif sh_obj.get("status") == "unknown":
                    # Empty cache: distinguish between three precondition states.
                    # F-AUD-P7-LOW-006: condition the regression wording on A22's RESULT
                    # (not just whether it was executed). If A22 itself FAILed, the empty
                    # cache is A22's fault — "investigate A22 first" is more actionable.
                    _a22_key = _A22_RESULT_KEY
                    _a22_result = results.get(_a22_key, "")
                    # A22 always writes _A22_RESULT_KEY; enforced by dict-guard added in
                    # pass-16 (F-AUD-P16-MED-001) — the unreachable `not in results` branch
                    # was removed (F-AUD-P16-LOW-004).
                    if _a22_result.startswith("PASS"):
                        # A22 passed but cache still empty → health-cache write regression.
                        results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                            f"FAIL: cache empty despite A22 having run and PASSED — "
                            f"health-cache write regression (check_sensor_health did not populate cache); "
                            f"message={sh_obj.get('message','')!r}"
                        )
                    else:
                        # A22 ran but FAILed → cache is empty because A22 failed; diagnose A22 first.
                        results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                            f"FAIL: cache empty; A22 also failed — investigate A22 first; "
                            f"A22 result={_a22_result[:80]!r}; "
                            f"message={sh_obj.get('message','')!r}"
                        )
                elif "clients" in sh_obj and isinstance(sh_obj["clients"], dict) and "stale" in sh_obj:
                    # Populated form: {"clients": {...}, "stale": bool}
                    # F-AUD-P16-MED-003: require client_count >= 1 AND total_sensors >= 1 —
                    # {"clients": {}, ...} is a health-cache-write regression. A22 runs for
                    # org-c only so client_count == 1 is the floor; total_sensors must be
                    # non-zero or the per-sensor health cache write was silently dropped.
                    client_count = len(sh_obj["clients"])
                    total_sensors = sum(
                        len(c.get("sensors", {})) if isinstance(c, dict) else 0
                        for c in sh_obj["clients"].values()
                    )
                    if client_count < 1 or total_sensors < 1:
                        results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                            f"FAIL: cache populated form but zero clients or sensors — "
                            f"health-cache-write regression (A22 must write at least one "
                            f"client with one sensor); "
                            f"client_count={client_count}; total_sensors={total_sensors}"
                        )
                    else:
                        # MED-001 (F-AUD-P25): add A22 gate to the populated-cache PASS branch.
                        # The empty-cache branch already conditions on A22; the populated branch
                        # must too — otherwise a stale cache from a prior run can silently PASS
                        # even when this run's A22 failed (cannot attribute population to this probe).
                        _a22_key = _A22_RESULT_KEY
                        _a22_result = results.get(_a22_key, "")
                        if not _a22_result.startswith("PASS"):
                            results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                                f"FAIL: cache populated but A22 did not PASS — "
                                f"cannot attribute population to this run's probe; "
                                f"investigate A22 first; A22 result={_a22_result[:80]!r}"
                            )
                        else:
                            # F-AUD-P30-MED-003: verify org-c sensor set matches EXPECTED_SENSORS.
                            # render_sensors_health_resource keys the sensors dict by sensor_id
                            # (crates/prism-mcp/src/resources.rs); set(keys()) gives the IDs.
                            # F-AUD-P32-LOW-002: guard non-dict org-c entry before descending
                            # to .get("sensors", {}) — an unexpected value type (e.g., a list or
                            # string) would raise AttributeError on the chained .get() call.
                            _client_org_c = sh_obj["clients"].get("org-c", {})
                            if not isinstance(_client_org_c, dict):
                                results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                                    f"FAIL: org-c entry in clients dict is not a dict — "
                                    f"unexpected shape: {type(_client_org_c).__name__!r}; "
                                    f"value={str(_client_org_c)[:40]!r}"
                                )
                            else:
                                _org_c_sensors = set(
                                    _client_org_c.get("sensors", {}).keys()
                                )
                                if _org_c_sensors != EXPECTED_SENSORS:
                                    results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                                        f"FAIL: org-c sensor set mismatch — "
                                        f"expected={sorted(EXPECTED_SENSORS)!r}, "
                                        f"actual={sorted(_org_c_sensors)!r} (F-AUD-P30-MED-003)"
                                    )
                                else:
                                    results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                                        f"PASS: populated clients{{}} form; "
                                        f"client_count={client_count}; total_sensors={total_sensors}; "
                                        f"stale={sh_obj['stale']!r}; "
                                        f"org-c sensors confirmed={sorted(_org_c_sensors)!r}"
                                    )
                else:
                    # Unexpected shape — neither known form.
                    results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                        f"FAIL: unexpected shape — neither empty-cache nor populated form; "
                        f"keys={list(sh_obj.keys())[:6]!r}; body={t_sh[:100]!r}"
                    )
            except (json.JSONDecodeError, TypeError, AttributeError) as exc:
                # F-AUD-P1-MED-007: render_sensors_health_resource always produces JSON.
                # Narrow to JSONDecodeError + TypeError + AttributeError only; let programmer
                # errors propagate. AttributeError added (F-AUD-P32-LOW-002): guard against
                # an unexpected non-dict value in the clients map that survives isinstance
                # check at a nested level and raises on a chained attribute access.
                results["[H14b] resources/read: prism://sensors/health — populated clients{} form"] = (
                    f"FAIL: non-JSON response or attribute error — resources.rs contract violation; "
                    f"exc={exc!r}; body={t_sh[:40]!r}"
                )

        # ── H14s: resources/read — prismql://schema/org-c (cyberint_alerts present) ─
        # F-AUD-P3-MED-004: split from composite H14; standalone cyberint schema check.
        res_sc, err_sc = resource_read(proc, "prismql://schema/org-c", timeout=10.0)
        if err_sc:
            results["[H14s] resources/read: prismql://schema/org-c — cyberint_alerts present"] = (
                f"FAIL: {err_sc}"
            )
        else:
            t_sc = (res_sc.get("contents") or [{}])[0].get("text", "")
            # MED-005: parse the prismql://schema/org-c response as JSON (delegates to
            # handle_prism_describe → returns {"tables": [...], ...} per resources/schema.rs).
            # Substring scan over raw text risks false positives from table descriptions.
            # Parse → extract table name set → set-difference against required tables.
            try:
                _sc_body = json.loads(t_sc)
                if not isinstance(_sc_body, dict):
                    results["[H14s] resources/read: prismql://schema/org-c — cyberint_alerts present"] = (
                        f"FAIL: expected JSON object from prismql://schema/org-c, "
                        f"got {type(_sc_body).__name__}: {t_sc[:80]!r}"
                    )
                else:
                    _sc_tables = {
                        t.get("name", "") for t in _sc_body.get("tables", [])
                        if isinstance(t, dict)
                    }
                    if "cyberint_alerts" in _sc_tables:
                        results["[H14s] resources/read: prismql://schema/org-c — cyberint_alerts present"] = (
                            f"PASS: prismql://schema/org-c JSON tables array includes "
                            f"cyberint_alerts (total={len(_sc_tables)} tables)"
                        )
                    else:
                        results["[H14s] resources/read: prismql://schema/org-c — cyberint_alerts present"] = (
                            f"FAIL: cyberint_alerts absent from tables array; "
                            f"tables={sorted(_sc_tables)!r}; body={t_sc[:100]!r}"
                        )
            except (json.JSONDecodeError, ValueError) as _h14s_err:
                results["[H14s] resources/read: prismql://schema/org-c — cyberint_alerts present"] = (
                    f"FAIL: non-JSON response from prismql://schema/org-c: {_h14s_err}; "
                    f"body={t_sc[:80]!r}"
                )

        # ── H14c: resources/read — prism://schema/crowdstrike/detections ─────
        # F-AUD-P2-MED-006: extend resource coverage to per-sensor-table schema URI.
        # URI template: prism://schema/{sensor_id}/{table_name} (resources.rs URI_TEMPLATE_SCHEMA).
        res_h14c, err_h14c = resource_read(proc, "prism://schema/crowdstrike/detections", timeout=10.0)
        if err_h14c:
            results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = f"FAIL: {err_h14c}"
        else:
            t_h14c = (res_h14c.get("contents") or [{}])[0].get("text", "")
            # OBS-005: parse the prism://schema/crowdstrike/detections response as JSON.
            # render_schema_resource serializes SensorTableDescriptor from
            # prism-spec-engine/src/types.rs → shape: {"columns": [{"name": "...", ...}, ...]}
            # Dropped string-in checks: '"columns"' in raw text matches column descriptions,
            # not just the structural key. JSON parse → structural walk is authoritative.
            try:
                _h14c_body = json.loads(t_h14c)
                if not isinstance(_h14c_body, dict) or "columns" not in _h14c_body:
                    _h14c_desc = (repr(list(_h14c_body.keys())[:6])
                                  if isinstance(_h14c_body, dict)
                                  else type(_h14c_body).__name__)
                    results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                        f"FAIL: expected JSON object with 'columns' key from schema resource; "
                        f"got keys={_h14c_desc}; "
                        f"body={t_h14c[:80]!r}"
                    )
                else:
                    _h14c_cols = _h14c_body.get("columns", [])
                    if not isinstance(_h14c_cols, list) or not _h14c_cols:
                        results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                            f"FAIL: 'columns' key present but empty or not a list — "
                            f"crowdstrike/detections must have columns in SensorTableDescriptor; "
                            f"columns={_h14c_cols!r}"
                        )
                    else:
                        _h14c_col_names = {c.get("name", "") for c in _h14c_cols if isinstance(c, dict)}
                        _known_cols = {"detection_id", "device_id", "severity"}
                        _confirmed = _known_cols & _h14c_col_names
                        if not _confirmed:
                            results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                                f"FAIL: columns array present ({len(_h14c_col_names)} cols) but "
                                f"none of {sorted(_known_cols)!r} confirmed — "
                                f"schema column names do not match expected crowdstrike/detections schema; "
                                f"sample_names={sorted(_h14c_col_names)[:5]!r}"
                            )
                        else:
                            results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                                f"PASS: SensorTableDescriptor JSON shape confirmed; "
                                f"columns array has {len(_h14c_col_names)} entries; "
                                f"confirmed known column(s)={sorted(_confirmed)!r}"
                            )
            except (json.JSONDecodeError, ValueError) as _h14c_err:
                if t_h14c:
                    results["[H14c] resources/read: prism://schema/crowdstrike/detections"] = (
                        f"FAIL: non-JSON response from schema resource (OBS-005): {_h14c_err}; "
                        f"body={t_h14c[:80]!r}"
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
            # MED-006: parse prism://config/clients/org-c/sensors as JSON array of
            # SensorConfigEntry objects (resources.rs render_client_sensors_resource →
            # JSON array with field "sensor_type" per SensorConfigEntry in resources.rs).
            # Dropped: `if s in t_h14d` substring scan — too weak (matches sensor names
            # embedded in unrelated fields). Parse → extract sensor_type set → set-difference.
            _required_sensors = {"crowdstrike", "armis", "claroty", "cyberint"}
            try:
                _h14d_arr = json.loads(t_h14d)
                if not isinstance(_h14d_arr, list):
                    results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                        f"FAIL: expected JSON array from per-client sensors resource, "
                        f"got {type(_h14d_arr).__name__}: {t_h14d[:80]!r}"
                    )
                else:
                    _present_sensors = {
                        e.get("sensor_type", "") for e in _h14d_arr
                        if isinstance(e, dict) and e.get("sensor_type")
                    }
                    _missing_sensors = _required_sensors - _present_sensors
                    # LOW-002 (F-AUD-P28): mirror H14a/A22 exact-set discipline — fail on
                    # extra sensor_type values as well as missing ones.  Extra entries expose
                    # unintended sensor registrations that weaken the four-sensor isolation
                    # evidence for the org-c reference client.
                    # CONSCIOUS-UPDATE REQUIRED: if the org-c demo config is updated to add or
                    # remove a sensor type, update _required_sensors in the same change.
                    _extra_sensors = _present_sensors - _required_sensors
                    if not _missing_sensors and not _extra_sensors:
                        results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                            f"PASS: all 4 sensor_type values confirmed via JSON parse: "
                            f"{sorted(_present_sensors)!r}; "
                            f"org-c four-sensor reference client confirmed (runbook §1.3)"
                        )
                    else:
                        results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                            f"FAIL: exact sensor_type set mismatch for org-c — "
                            + (f"missing={sorted(_missing_sensors)!r}; " if _missing_sensors else "")
                            + (f"extra={sorted(_extra_sensors)!r}; " if _extra_sensors else "")
                            + f"present={sorted(_present_sensors)!r} "
                            + f"(MED-006 / F-AUD-P28-LOW-002)"
                        )
            except (json.JSONDecodeError, ValueError) as _h14d_err:
                if t_h14d:
                    results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                        f"FAIL: non-JSON response from per-client sensors resource (MED-006): "
                        f"{_h14d_err}; body={t_h14d[:80]!r}"
                    )
                else:
                    results["[H14d] resources/read: prism://config/clients/org-c/sensors"] = (
                        "FAIL: empty body for per-client sensors resource"
                    )

        # ── H14e: resources/subscribe + unsubscribe — prismql://schema/org-c ─
        # F-AUD-P2-MED-008: resources/subscribe supported via enable_resources_subscribe()
        # in server.rs for prismql://schema/{client_id} URIs.
        # F-AUD-P3-HIGH-002 SMOKE-ONLY: this check verifies the server accepts the
        # subscribe/unsubscribe round-trip without hanging, crashing, or returning a
        # transport error.  No client-observable subscription state exists in the MCP
        # protocol — positive coverage (confirming a notification was delivered) would
        # require a mutating trigger, which is excluded by the read-only preflight
        # constraint.  PASS here means: no hang + no panic + no -32601 / transport error.
        # OBS-002: tighten subscribe AND unsubscribe timeouts from 10.0s to 5.0s (smoke-only
        # checks; any valid server response arrives in < 1s; 10s was unnecessarily loose for
        # a no-hang test; F-AUD-P20-LOW-002 applies to both sides of the round-trip)
        # F-AUD-P24-MED-002 (server-side subscribe URI validation gap confirmed):
        #   server.rs::subscribe (`strip_prefix("prismql://schema/")`) dispatches on prefix
        #   only — URIs without this prefix return Ok(()) silently (no rejection).
        #   URIs with the prefix are accepted for any valid-format OrgSlug
        #   ([a-zA-Z0-9_-]{1,64}) regardless of whether the client actually exists; no
        #   resource-existence check is performed.  A negative probe to
        #   `prismql://schema/does-not-exist-preflight-negative-probe` would return Ok()
        #   (valid slug format → accepted), NOT -32602 — adding it here would false-PASS
        #   rather than demonstrate rejection.  Negative-probe coverage is blocked until
        #   the server validates URIs against known client slugs; tracked in
        #   .factory/STATE.md D-1696 S-7.02 cascade-close queue (item: server-side
        #   resources/subscribe URI validation); follow-up story to be drafted at cascade close.
        #   LOW-005 (F-AUD-P26): updated tracking reference from ephemeral cascade-close
        #   placeholder to durable STATE.md D-1696 anchor.
        res_sub, err_sub = resources_subscribe(proc, "prismql://schema/org-c", timeout=5.0)
        if err_sub:
            results["[H14e] resources/subscribe+unsubscribe: prismql://schema/org-c (smoke-only)"] = (
                f"FAIL: subscribe: {err_sub}"
            )
        else:
            res_unsub, err_unsub = resources_unsubscribe(proc, "prismql://schema/org-c", timeout=5.0)
            if err_unsub:
                results["[H14e] resources/subscribe+unsubscribe: prismql://schema/org-c (smoke-only)"] = (
                    f"FAIL: subscribe OK but unsubscribe failed: {err_unsub}"
                )
            else:
                results["[H14e] resources/subscribe+unsubscribe: prismql://schema/org-c (smoke-only)"] = (
                    "PASS: subscribe+unsubscribe round-trip accepted (smoke: no hang/panic/transport error)"
                )

        # ── H15: live explain_query call (one of 14 implemented tools) ───────
        # LOW-003: this check does NOT re-assert the 14-tool set from A2. It verifies
        # explain_query executes correctly and returns a parseable plan.
        # MED-001: parsed_mode is produced by `query_mode_str()` in
        # crates/prism-query/src/explain.rs — canonical values are "filter" | "sql" | "pipe"
        # | "sql_pipe" | "unknown" (OBS-001 F-AUD-P28: complete enumeration per explain.rs;
        # "unknown" is the catch-all for unrecognised parse trees)
        # (TD-VSDD-091: cite function name, not line number; function name is stable
        # across refactors while line numbers drift). "FROM ... | limit N" is pipe-mode;
        # assert parsed_mode == "pipe" exactly to detect mode-detection regressions.
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
            # MED-007: the query "FROM crowdstrike_detections | limit 5" is unambiguously
            # pipe-syntax — assert parsed_mode == "pipe" exactly, not "any valid domain".
            _pm = body.get("parsed_mode")
            # F-AUD-P32-LOW-003: explain_query is plan-time introspection; it must NOT fan out
            # to sensors. Verified in explain_query() in crates/prism-mcp/src/server.rs:
            # the handler calls qe.explain() (query planner only) and wraps result with
            # DataSource::Multiple(vec![]) — no sensor data accessed, no fan-out.
            # A non-empty sensor_errors field would indicate an unexpected regression.
            # Do NOT use sensor_errors_gate() here — that gate emits PASS on empty sensor_errors,
            # which would be semantically correct for a data-fetching tool but is the WRONG
            # signal for explain_query: the absence of sensor_errors is EXPECTED, not a PASS.
            _h15_se = body.get("sensor_errors")
            if _h15_se:
                results["[H15] explain_query live call (one of 14 implemented tools)"] = (
                    f"FAIL: explain_query is plan-time introspection — unexpected sensor "
                    f"fan-out evidence; sensor_errors={str(_h15_se)[:80]!r} "
                    f"(explain_query handler in server.rs uses DataSource::Multiple(vec![]) — "
                    f"no sensor data should be accessed)"
                )
            elif _pm is None:
                results["[H15] explain_query live call (one of 14 implemented tools)"] = (
                    f"FAIL: explain_query response lacks parsed_mode key — schema mismatch or empty plan; "
                    f"keys={list(body.keys())[:8]}"
                )
            elif _pm == "pipe":
                results["[H15] explain_query live call (one of 14 implemented tools)"] = (
                    f"PASS: explain_query returned plan with parsed_mode='pipe' "
                    f"(pipe query confirmed); "
                    f"keys={list(body.keys())[:6]}"
                )
            else:
                # MED-007: any mode other than "pipe" for a pipe query is a regression
                results["[H15] explain_query live call (one of 14 implemented tools)"] = (
                    f"FAIL: expected parsed_mode='pipe' for pipe-syntax query; "
                    f"got {_pm!r} — unexpected mode (mode-detection or serialization regression); "
                    f"keys={list(body.keys())[:6]}"
                )

        # ── H16: CWE-116/117 — control-char injection sanitized ──────────────
        # LOW-002: U+2028 (LINE SEPARATOR) and U+2029 (PARAGRAPH SEPARATOR) are category
        # "Zl"/"Zp" — NOT "Cc" — but sanitize_for_log strips them per E-QUERY-038 spec.
        # Define once, reused by both H16 and H16b control-char leak checks.
        _LS_PS = (" ", " ")  # U+2028 / U+2029
        # Embed a literal U+0001 in a quoted SQL identifier to verify sanitize_for_log strips it.
        #
        # Deterministic layer analysis (F-AUD-P11-OBS-002):
        #   Query: SELECT "badcolumn\x01" FROM crowdstrike_detections LIMIT 3
        #   Layer 1 — SQL parser: accepts quoted identifiers with arbitrary characters
        #             (double-quoted ident syntax allows non-alphanumeric chars, including \x01).
        #   Layer 2 — E-QUERY-038 column gate: "badcolumn\x01" is not in the crowdstrike_detections
        #             schema → ColumnNotFoundDetails::new fires.
        #   Layer 3 — sanitize_for_log: ColumnNotFoundDetails::new applies sanitize_for_log
        #             (prism_core::error::sanitize_for_log) to the column name, stripping U+0001
        #             → "badcolumn" in the error message.
        #   This path is deterministic (not environment-dependent) — the SQL parser always
        #   accepts the quoted ident, the column always fails E-QUERY-038, sanitize_for_log
        #   always fires at ColumnNotFoundDetails construction.
        #
        # PASS requires: "ERROR:" prefix (MCP layer wraps E-QUERY-038 message) AND
        #   "E-QUERY-038" in text (confirms column gate fired, not a parse/other error) AND
        #   no raw U+0001 in response (confirms sanitize_for_log stripped it).
        #
        # F-AUD-P14-OBS-008 — INTENTIONAL COUPLING: if the SQL parser is deliberately
        #   hardened to reject control-chars at parse time (E-QUERY-001), update this
        #   check's expected layer — do not weaken to accept both. This coupling is
        #   intentional: deliberate parser hardening must cause a conscious update to
        #   this audit (same philosophy as POL-24 anchor checks).
        #   Orchestrator adjudication: strict layer coupling retained (LOCAL pass-14,
        #   reaffirmed pass-16; D-1694 adjudication record in STATE.md decision log).
        #   Rust unit-test status: sanitize_for_log has unit test coverage in
        #   crates/prism-core/src/error.rs
        #   (test_sanitize_for_log_strips_unicode_cc_and_line_separators).
        ctrl_col = "badcolumn\x01"
        h16_query = f'SELECT "{ctrl_col}" FROM crowdstrike_detections LIMIT 3'
        rid_h16 = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid_h16, "method": "tools/call",
                        "params": {"name": "query",
                                   "arguments": {"query": h16_query, "clients": ["org-c"]}}})
        resp_h16, err_h16 = read_msg(proc, timeout=15.0, expected_id=rid_h16)
        if err_h16:
            results["[H16] CWE-116/117: control-char in column name sanitized"] = f"FAIL: {err_h16}"
        else:
            # LOW-002: extended control-char scan — covers full Unicode Cc category
            # (E-QUERY-038 sanitize_for_log strips "Unicode Cc + U+2028/U+2029");
            # the prior ASCII-only check (ord < 0x20 / 0x7F) missed non-ASCII C1
            # control chars (U+0080–U+009F) and the line/paragraph separators.
            raw_content = resp_h16.get("result", {}).get("content", []) if resp_h16 else []
            raw_text = raw_content[0].get("text", "") if raw_content else ""
            # F-AUD-P30-MED-002: extract error code from structuredContent.error.code
            # (authoritative MCP error code), falling back to in-band regex extraction.
            # parse_envelope populates _sc_error from structuredContent.error when present;
            # build_structured_error_response sets structuredContent.error.code = E-code string.
            # Source: crates/prism-mcp/src/error_mapping.rs (build_structured_error_response).
            _body_h16, _ = parse_envelope(resp_h16)
            _h16_ec = ((_body_h16.get("_sc_error") or {}).get("code")
                       or _body_h16.get("error_code", ""))
            # _LS_PS (U+2028/U+2029) defined at H16 section header (_LS_PS constant); used here and H16b.
            control_chars_found = [c for c in raw_text
                                    if (unicodedata.category(c) == "Cc" or c in _LS_PS)
                                    and c not in ("\n", "\r", "\t")]
            if control_chars_found:
                results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                    f"FAIL: raw control chars leaked in response: "
                    f"{[hex(ord(c)) for c in control_chars_found[:5]]}"
                )
            elif _h16_ec == "E-QUERY-038" and "badcolumn" in raw_text and "\x01" not in raw_text:
                # Deterministic PASS path: SQL parser accepted the quoted ident → E-QUERY-038 fired
                # → ColumnNotFoundDetails::new applied sanitize_for_log → control char stripped.
                # Require E-QUERY-038 from structuredContent.error.code (not substring match);
                # "badcolumn" confirms column name reached error path; no raw \x01 confirms strip.
                results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                    f"PASS: E-QUERY-038 (structuredContent.error.code confirmed); "
                    f"control-char stripped by sanitize_for_log; "
                    f"'badcolumn' in response, '\\x01' absent "
                    f"(CWE-116/117 sanitized — SQL parser accepted quoted ident → E-QUERY-038 column gate); "
                    f"preview={raw_text[:80]!r}"
                )
            elif raw_text.startswith("ERROR:"):
                # F-AUD-P13-OBS-006: ERROR: prefix but not E-QUERY-038 — unexpected error layer
                # violates the determinism claim (H16 comment §Layer 2 establishes E-QUERY-038 as
                # the deterministic layer; any other error layer is a spec deviation).
                # control_chars_found is [] here (the if-branch above caught any leakage).
                # F-AUD-P29-LOW-003: the "leaked" arm is unreachable — reaching this elif
                # branch requires control_chars_found to be empty (the preceding if at line
                # ~4370 handles non-empty); replace dead ternary with literal "clean".
                _h16_cc_status = "clean"
                results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                    f"FAIL: unexpected error layer (determinism claim violated); "
                    f"expected E-QUERY-038 but got different error type; "
                    f"control-char status: {_h16_cc_status}; preview={raw_text[:80]!r}"
                )
            elif resp_h16 and "error" in resp_h16:
                # HIGH-002 (F-AUD-P25): scan the RPC error message channel for control-char
                # leakage — the error object's "message" and "data" fields are part of the
                # transport surface and must also be free of Cc + U+2028/29.
                _h16_rpc_err = resp_h16["error"]
                _h16_rpc_scan_text = str(_h16_rpc_err.get("message", ""))
                _h16_rpc_data = _h16_rpc_err.get("data")
                if isinstance(_h16_rpc_data, str):
                    _h16_rpc_scan_text += _h16_rpc_data
                _h16_rpc_cc = [c for c in _h16_rpc_scan_text
                               if (unicodedata.category(c) == "Cc" or c in _LS_PS)
                               and c not in ("\n", "\r", "\t")]
                if _h16_rpc_cc:
                    results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                        f"FAIL: control char leaked in RPC error channel (error.message/data): "
                        f"{[hex(ord(c)) for c in _h16_rpc_cc[:5]]}"
                    )
                else:
                    # RPC-level rejection with clean error channel — acceptable
                    results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                        f"PASS: RPC-level rejection; error channel (error.message/data) free of control chars"
                    )
            else:
                # F-AUD-P14-MED-002: remove "well-formed JSON dict without control chars → PASS"
                # fallback. An unexpected success response on this path means the Layer-2
                # E-QUERY-038 column gate was bypassed — sanitize_for_log positive coverage
                # requires the E-QUERY-038 error path. Accepting a successful data envelope
                # here would mask a layer-bypass regression. Always FAIL.
                results["[H16] CWE-116/117: control-char in column name sanitized"] = (
                    f"FAIL: unexpected success response — sanitize_for_log positive coverage "
                    f"requires the E-QUERY-038 error path; well-formed JSON without error "
                    f"indicates the Layer-2 column gate was bypassed; "
                    f"preview={raw_text[:80]!r}"
                )

        # ── H16b: CWE-116/117 — control-char in WHERE-predicate value (smoke-only) ──
        # F-AUD-P2-LOW-001: complement H16 (quoted column name) with a WHERE predicate
        # probe using a control-char value.
        # F-AUD-P7-LOW-001: SMOKE-ONLY relabel — forced echo through sanitize_for_log cannot
        # be achieved via a read-only query.  The echo path requires the control-char value
        # to be echoed back in an error message (e.g., via E-QUERY-038 ColumnNotFoundDetails
        # whose column field is sanitized at construction).  The pipe-parser ident_char
        # definition (c.is_ascii_alphanumeric() || c == '_') rejects Cc chars before any
        # column lookup occurs, so an unquoted identifier containing \x01 produces a parse
        # error — not a ColumnNotFound echo.  Using the \x01 as a value (quoted) against an
        # unrecognized severity string returns 0 rows rather than an error, yielding an
        # empty envelope where no control char can leak.  Positive coverage of sanitize_for_log
        # on the echo path is provided by H16 (quoted identifier containing U+0001 → E-QUERY-038
        # → ColumnNotFoundDetails applies sanitize_for_log → audit asserts no control char in
        # response); H16b remains as a negative-leak smoke check only.
        # PASS here means: no control char leaked in the response to a WHERE-predicate
        # containing \x01 in the value position (neither echo nor passthrough observed).
        ctrl_val = "critical\x01injected"
        h16b_query = f"FROM crowdstrike_detections\n| where severity = '{ctrl_val}'\n| limit 3"
        rid_h16b = next_id()
        send_msg(proc, {"jsonrpc": "2.0", "id": rid_h16b, "method": "tools/call",
                        "params": {"name": "query",
                                   "arguments": {"query": h16b_query, "clients": ["org-c"]}}})
        resp_h16b, err_h16b = read_msg(proc, timeout=15.0, expected_id=rid_h16b)
        if err_h16b:
            results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized (smoke-only)"] = f"FAIL: {err_h16b}"
        else:
            raw_content_b = resp_h16b.get("result", {}).get("content", []) if resp_h16b else []
            raw_text_b = raw_content_b[0].get("text", "") if raw_content_b else ""
            # LOW-002: extend to full Unicode Cc category + U+2028/U+2029 (mirrors H16 fix;
            # reuses _LS_PS defined in H16 block above).
            ctrl_leaked = [c for c in raw_text_b
                           if (unicodedata.category(c) == "Cc" or c in _LS_PS)
                           and c not in ("\n", "\r", "\t")]
            if ctrl_leaked:
                results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized (smoke-only)"] = (
                    f"FAIL: raw control chars leaked in response: "
                    f"{[hex(ord(c)) for c in ctrl_leaked[:5]]}"
                )
            elif resp_h16b and "error" in resp_h16b:
                # HIGH-002 (F-AUD-P25): scan the RPC error message channel for control-char
                # leakage in H16b as well — same predicate as H16.
                _h16b_rpc_err = resp_h16b["error"]
                _h16b_rpc_scan_text = str(_h16b_rpc_err.get("message", ""))
                _h16b_rpc_data = _h16b_rpc_err.get("data")
                if isinstance(_h16b_rpc_data, str):
                    _h16b_rpc_scan_text += _h16b_rpc_data
                _h16b_rpc_cc = [c for c in _h16b_rpc_scan_text
                                if (unicodedata.category(c) == "Cc" or c in _LS_PS)
                                and c not in ("\n", "\r", "\t")]
                if _h16b_rpc_cc:
                    results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized (smoke-only)"] = (
                        f"FAIL: control char leaked in RPC error channel (error.message/data): "
                        f"{[hex(ord(c)) for c in _h16b_rpc_cc[:5]]}"
                    )
                else:
                    results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized (smoke-only)"] = (
                        "PASS: RPC-level rejection; error channel (error.message/data) free of control chars "
                        "(smoke: no hang/panic/leak)"
                    )
            elif raw_text_b.startswith("ERROR:") or raw_text_b.startswith("{") or not raw_text_b:
                # Empty envelope (0-row match) or ERROR string — both are smoke-pass: no leak.
                results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized (smoke-only)"] = (
                    f"PASS: response free of control chars (smoke: no Cc leakage in envelope); "
                    f"preview={raw_text_b[:60]!r}"
                )
            else:
                results["[H16b] CWE-117: control-char in WHERE-predicate value sanitized (smoke-only)"] = (
                    f"FAIL: indeterminate response for WHERE predicate probe; preview={raw_text_b[:60]!r}"
                )

        # ── H17: E-QUERY-033 — limit > 1000 rejected ─────────────────────────
        body, err = tool_call(proc, "query",
                              {"query": "FROM crowdstrike_detections | limit 5",
                               "clients": ["org-c"], "limit": 1001})
        if err:
            # E-QUERY-033 → -32602 INVALID_PARAMS at the MCP params level (build_query_options).
            # F-AUD-P1-HIGH-002: bare "RPC error" matches EVERY RPC failure — constrain to
            # -32602 AND E-QUERY-033 code to avoid false positives.
            err_str = str(err)
            # F-AUD-P4-MED-001: drop the bare "1000" disjunct — "1000" over-matches
            # unrelated error paths (any message containing "1000" would pass).
            # The code discriminator "E-QUERY-033" is sufficient and unambiguous per
            # error-taxonomy.md E-QUERY-033 row. Anchor: "-32602" AND "E-QUERY-033".
            # HIGH-003 (F-AUD-P25): anchor both predicates to prevent over-matching.
            # parse_envelope produces "RPC error {code}: {message}" so the string always
            # starts with "RPC error -32602:" when code=-32602. Substring "-32602" could
            # false-match a message body containing that literal. "E-QUERY-033:" is
            # anchored with a colon to avoid matching "E-QUERY-033x" or similar.
            # TD-VSDD-060 sibling sweep: -32000 in H8 similarly anchored in same commit.
            is_controlled = (
                re.match(r"^RPC error -32602:", err_str) is not None
                and "E-QUERY-033:" in err_str
            )
            if is_controlled:
                results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                    f"PASS: limit > 1000 controlled rejection (-32602 + E-QUERY-033 anchor): {err[:100]}"
                )
            else:
                results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                    f"FAIL: unexpected error (not a controlled -32602 + E-QUERY-033 rejection): {err[:100]}"
                )
        elif body.get("error_code") == "E-QUERY-033":
            results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                "PASS: E-QUERY-033 in-band rejection"
            )
        elif body.get("error_code"):
            # F-AUD-P19-LOW-004: hoist non-E-QUERY-033 error_code branch above the rows
            # check so the FAIL message names the actual unexpected error code.
            ec = body.get("error_code", "")
            results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                f"FAIL: expected E-QUERY-033, got {ec}: {body.get('message','')[:80]}"
            )
        elif body.get("rows") is not None:
            # Rows present with no error_code means the query genuinely succeeded —
            # E-QUERY-033 ceiling not enforced.
            results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                f"FAIL: limit 1001 query succeeded ({len(body.get('rows', []))} rows) — E-QUERY-033 not enforced"
            )
        else:
            # PARTIAL sweep: unexpected response (not a controlled rejection) → FAIL.
            results["[H17] E-QUERY-033: limit 1001 rejected (BC-2.11.001 ceiling)"] = (
                f"FAIL: unexpected response (not a controlled rejection); "
                f"body keys={list(body.keys())[:4]}"
            )

        # ── H18: E-QUERY-003 / oversize query rejected ───────────────────────
        # ~80KB IN clause exceeds the 64KB MCP-level guard (or engine security limit).
        # Either E-QUERY-003 or -32602 param rejection PASSes; success/hang/crash FAILs.
        # Byte arithmetic: 'val{i:09d}' = 14 chars × 5000 = 70,000 + ', ' × 4999 = 9,998
        # + wrapper (~64 chars) ≈ 80,062 bytes — durably over the 65,536-byte threshold.
        # (Previous '06d' format produced only ~65,062 bytes — UNDER the strict > 65,536
        # threshold — so on a healthy system the query would execute and H18 FAILed falsely.)
        _vals = ", ".join(f"'val{i:09d}'" for i in range(5000))  # ~80KB
        big_query = f"FROM crowdstrike_detections\n| where detection_id IN ({_vals})\n| limit 5"
        # F-AUD-P7-LOW-002 / F-AUD-P11-LOW-003: unconditional guard — survives python -O
        # (assert is elided under optimization; this conditional is not).
        # LOW-003: replaced raise RuntimeError with FAIL result so the audit report captures
        # the construction bug rather than aborting with an unhandled exception; the FAIL
        # still gates exit-1 via fail_count in main().
        if len(big_query) <= 65_536 + 1024:
            results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                f"FAIL: H18 payload construction bug — {len(big_query)} bytes <= 66,560 "
                f"threshold (65536 + 1024 margin); check element format — query not sent"
            )
        else:
            # F-AUD-P24-LOW-003: 30s timeout rationale — oversize rejection is O(1) plan-time
            # (size gate fires before query execution); 30s bounds transport congestion and
            # cold-start overhead; TIMEOUT is intentionally UNCONTROLLED FAIL by design
            # (a server that hangs on an oversize query has not correctly implemented the gate).
            body, err = query(proc, big_query, ["org-c"], timeout=30.0)
            if err:
                # F-AUD-P1-HIGH-001: only controlled rejections PASS here.
                # The former `if err: PASS` converted timeouts/crashes/JSON errors into PASS.
                # Accept only: RPC -32602 INVALID_PARAMS, or E-QUERY-003 in error text.
                # F-AUD-P4-MED-002: -32603 INTERNAL_ERROR is explicitly UNCONTROLLED — it
                # signals a crash/panic on the oversize input, which is exactly the failure
                # mode this check must distinguish from a proper controlled rejection.
                # Per error-taxonomy.md E-QUERY-003: canonical MCP surfacing is -32602
                # INVALID_PARAMS; -32603 is INTERNAL_ERROR (uncontrolled crash path).
                err_str = str(err)
                is_timeout = "TIMEOUT" in err_str
                is_process_exit = err_str.startswith("Process exited") or err_str.startswith("EOF")
                is_json_error = err_str.startswith("JSON error") or err_str.startswith("envelope JSON error")
                is_internal_error = err_str.startswith("RPC error -32603")
                is_controlled_rpc = (
                    err_str.startswith("RPC error -32602")
                    or "E-QUERY-003" in err_str
                )
                if is_timeout or is_process_exit or is_json_error:
                    results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                        f"FAIL: uncontrolled failure (timeout/crash/JSON error) instead of controlled rejection: "
                        f"{err[:100]}"
                    )
                elif is_internal_error:
                    results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                        f"FAIL: internal error on oversize input (uncontrolled — -32603 INTERNAL_ERROR "
                        f"signals a crash/panic, not a structured rejection): {err[:100]}"
                    )
                elif is_controlled_rpc:
                    results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                        f"PASS: oversize query rejected at MCP or engine level (controlled): {err[:80]}"
                    )
                else:
                    results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                        f"FAIL: unexpected error (not a controlled -32602/E-QUERY-003 rejection): "
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
                # PARTIAL sweep: unexpected in-band response (not a controlled rejection) → FAIL.
                results["[H18] E-QUERY-003: oversize query controlled rejection"] = (
                    f"FAIL: unexpected response (expected controlled rejection); "
                    f"body keys={list(body.keys())[:4]}"
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
            if sensor_errors_gate("[H19a] threat_sources UDF returns virustotal", body_ts, results):
                pass
            elif rows:
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
            if sensor_errors_gate("[H19b] cvss_vector UDF returns CVSS:3.1/ string", body_cv, results):
                pass
            elif rows:
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

        # ── H20: ADR-051 D4 regression detector — JSON-list iocs_value → NULL ───
        # ADR-051 D4 scalar-input rule: threat_score(iocs_value) where iocs_value is a
        # JSON-list column (e.g. ["hash1","hash2"]) MUST produce NULL + E-INFUSE-014 per
        # ADR-051 D4 ("a JSON-list string (detected by leading '[') as input to a typed-output
        # UDF produces NULL + E-INFUSE-014 at runtime").  The infusion_udf.rs coerce_to_typed
        # implementation: starts_with('[') && output_type != Utf8 → None (NULL sentinel).
        # Sanctioned outcome: query SUCCEEDS with NULL in the threat_score column — NO error code.
        # PASS = no numeric scores >= 75 (all NULL or 0 → ADR-051 D4 enforced).
        # FAIL = any numeric score >= 75 (ADR-051 D4 scalar-input REGRESSION).
        # error_code = unsanctioned (ADR-051 D4 expects query success with NULLs, not a query error).
        # F-AUD-P2-MED-002: demoted from WARN to FAIL; description updated.
        # F-AUD-P11-MED-001: error_code branch annotated with ADR-051 D4 citation; PASS message
        # updated to reflect NULL output (not score=0) as the ADR-sanctioned outcome.
        body, err = query(proc,
            "FROM cyberint_alerts\n| where iocs_value IS NOT NULL\n"
            "| enrich threat_score(iocs_value)\n| limit 10",
            ["org-c"], timeout=30.0)
        if err:
            results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"] = f"FAIL: {err}"
        elif body.get("error_code"):
            # Unsanctioned: ADR-051 D4 expects the query to succeed with NULLs in threat_score.
            # A query-level error_code here means the engine failed to produce a result row at
            # all, which is NOT the sanctioned NULL-output path. Name the specific code.
            ec = body.get("error_code", "")
            results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"] = (
                f"FAIL: {ec} (unsanctioned — ADR-051 D4 sanctions NULL output on JSON-list input, "
                f"not a query-level error): {body.get('message','')[:80]}"
            )
        else:
            rows = body.get("rows", [])
            if sensor_errors_gate("[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL", body, results):
                pass
            elif rows:
                # F-AUD-P15-LOW-002: absent-column guard — distinguish "column present but
                # all NULL" (ADR-051 D4 sanctioned outcome) from "column entirely absent"
                # (enrich stage failed to produce output — not sanctioned).
                if not any("threat_score" in r for r in rows):
                    results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"] = (
                        "FAIL: threat_score column absent from all rows — "
                        "enrich stage did not produce output (ADR-051 D4 requires NULL output, "
                        "not absent column; check infusion_udf.rs / threatintel DTU)"
                    )
                else:
                    scores = [r.get("threat_score") for r in rows if "threat_score" in r]
                    # LOW-002: bool is subclass of int — exclude booleans from numeric check
                    numeric_scores = [s for s in scores if isinstance(s, (int, float)) and not isinstance(s, bool)]
                    max_score = max(numeric_scores, default=0)
                    if max_score >= 75:
                        # Emphatic FAIL: ADR-051 D4 scalar-input regression — iocs_value
                        # (JSON-list) must produce ALL-NULL (coerce_to_typed returns None for '['-input);
                        # a numeric score >= 75 means the NULL sentinel was bypassed → FAIL.
                        results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"] = (
                            f"FAIL: threat_score={max_score} for JSON-list column — "
                            f"ADR-051 D4 scalar-input REGRESSION: iocs_value must produce NULL; scores={scores[:5]}"
                        )
                    elif numeric_scores:
                        # F-AUD-P14-MED-001: partial regression — non-NULL scores present but < 75.
                        # ADR-051 §D4 mandates ALL-NULL for JSON-list input to typed UDFs;
                        # any numeric (non-None) score means coerce_to_typed returned a value
                        # instead of None for a '['-prefix input — regression regardless of magnitude.
                        results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"] = (
                            f"FAIL: non-NULL scores on JSON-list column — partial ADR-051 D4 regression; "
                            f"coerce_to_typed must return None for JSON-list input (not a numeric value); "
                            f"scores={scores[:5]}"
                        )
                    else:
                        # ADR-051 D4 sanctioned outcome: ALL-NULL output on JSON-list input.
                        # Column IS present (absent-column guard above confirmed threat_score
                        # key exists in rows); len(numeric_scores) == 0 confirms all threat_score
                        # values are Python None (filtered out by isinstance check) → max_score
                        # defaults to 0. F-AUD-P10-MED-003: assert only what H20 verifies — that
                        # ADR-051 D4 scalar-input is enforced. Runbook validity checked by H23.
                        results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"] = (
                            f"PASS: ALL-NULL output for JSON-list column (ADR-051 D4 scalar-input enforced); "
                            f"numeric_scores_found={len(numeric_scores)}, all_scores={scores[:5]}"
                        )
            else:
                results["[H20] ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"] = (
                    "FAIL: 0 rows from iocs_value IS NOT NULL filter (check DTU data)"
                )

        # ── H21: Determinism — same sorted query returns identical rows ───────
        # Seeded ChaCha20 + fixed anchors provide in-session determinism: two calls to the
        # same sorted query within one MCP session return byte-identical rows.
        # Note: this probe exercises in-session determinism only (two consecutive calls within
        # a single prism process lifetime). Cross-restart determinism (seed preservation across
        # process restarts) is NOT exercised by this audit. F-AUD-P11-OBS-001.
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
            # LOW-001 (F-AUD-P25): split determinism assertion from fixture count.
            # H21 owns the determinism claim (rows1 == rows2); H11 owns the seed-200 count
            # contract (5 Critical + 15 Medium = 20 rows).  The prior combined gate
            # (rows1 == rows2 AND len == 20) made H21 a duplicate of H11's count check,
            # which conflated two orthogonal properties and produced confusing FAILs when
            # the row count was non-20 but the query WAS deterministic.
            # F-AUD-P30-OBS-001: use json.dumps(sort_keys=True) for comparison to avoid
            # false-nondeterminism from key-ordering differences in Python dict comparison.
            # Python dict equality (rows1 == rows2) is key-order-invariant for dicts, but
            # json.dumps is more explicit and handles edge cases in nested structures.
            if sensor_errors_gate("[H21] Determinism: repeated sorted query byte-identical", body1, results):
                pass
            elif sensor_errors_gate("[H21] Determinism: repeated sorted query byte-identical", body2, results):
                pass
            elif json.dumps(rows1, sort_keys=True) != json.dumps(rows2, sort_keys=True):
                # LOW-004: use zip_longest to count diff rows even when lengths differ;
                # plain zip truncates and produces differing_rows=0 on length mismatch.
                diffs = sum(
                    1 for a, b in itertools.zip_longest(rows1, rows2, fillvalue=None)
                    if json.dumps(a, sort_keys=True) != json.dumps(b, sort_keys=True)
                )
                results["[H21] Determinism: repeated sorted query byte-identical"] = (
                    f"FAIL: rows differ — run1={len(rows1)}, run2={len(rows2)}, "
                    f"differing_rows={diffs} (non-deterministic result)"
                )
            elif len(rows1) == 0:
                # rows1 == rows2 but no data — determinism is vacuously true; report FAIL
                # for data-absence (cannot confirm sort-order or ID presence without rows).
                results["[H21] Determinism: repeated sorted query byte-identical"] = (
                    f"FAIL: 0 rows in both calls — data absent (cannot confirm in-session "
                    f"determinism without data; investigate DTU / sensor connection)"
                )
            else:
                # rows1 == rows2 and len > 0: determinism confirmed.
                # F-AUD-P5-OBS-003: assert detection_id values are lex-sorted ascending
                # (| sort detection_id defaults to Asc per pipe_parser.rs).
                # Pass-18 sweep: guard against all-null detection_id — vacuous sorted-check.
                # F-AUD-P29-LOW-005: filter to non-empty values (truthiness + .strip()) to
                # guard against all-empty-string detection_id yielding a vacuous sorted-
                # comparison PASS.  Prior `is not None` admitted "", which sorted equal to
                # itself and falsely PASSed the lex-sort check.
                det_ids = [
                    r.get("detection_id")
                    for r in rows1
                    if r.get("detection_id")
                    and isinstance(r.get("detection_id"), str)
                    and r.get("detection_id").strip()
                ]
                if not det_ids:
                    # Distinguish null/absent column from present-but-empty values.
                    _h21_raw = [r.get("detection_id") for r in rows1]
                    if all(v is None for v in _h21_raw):
                        results["[H21] Determinism: repeated sorted query byte-identical"] = (
                            f"FAIL: all detection_id null/absent in {len(rows1)} rows — "
                            f"data-quality regression (Standing Rule 3 §2)"
                        )
                    else:
                        results["[H21] Determinism: repeated sorted query byte-identical"] = (
                            f"FAIL: all detection_id present but empty/whitespace in {len(rows1)} rows — "
                            f"data-quality regression (Standing Rule 3 §2)"
                        )
                elif det_ids == sorted(det_ids):
                    results["[H21] Determinism: repeated sorted query byte-identical"] = (
                        f"PASS: {len(rows1)} rows (diagnostic); two consecutive calls byte-identical "
                        f"(in-session determinism confirmed; seeded ChaCha20 + fixed anchors); "
                        f"detection_id lex-sorted ascending confirmed ({len(det_ids)} IDs)"
                    )
                else:
                    results["[H21] Determinism: repeated sorted query byte-identical"] = (
                        f"FAIL: bytes identical but detection_id not lex-sorted ascending — "
                        f"sort stage no-op regression; "
                        f"expected asc, got: {det_ids[:5]!r}"
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
            if sensor_errors_gate("[H22] BC-2.11.018: normalized_pql present on success path", body, results):
                pass
            elif npql == "MISSING":
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

        # ── H23: Runbook enrich-call drift — static text check ──────────────
        # F-AUD-P10-MED-003 + HIGH-001 runbook-side + F-AUD-P11-MED-002:
        # Verify the runbook has no pre-ADR-051 non-first forms for ANY of the 6 typed UDFs
        # (ThreatIntel × 3 + NVD × 3) and has >= 1 correct scalar-companion form total.
        # This is a pure static text check — no MCP call.
        #
        # F-AUD-P11-HIGH-001: path resolved portably via _find_factory_file() (see module-level
        # definition). Works from main checkout (script in scripts/) and from any worktree
        # (.worktrees/<id>/scripts/) — walks up ancestors to find .factory/objectives/.
        # On CI (fresh clone at repo root): _repo_root/.factory/objectives/ resolves directly.
        # In worktree: _repo_root.parent.parent (= main repo) provides .factory/ mount point.
        #
        # ADR-051 D4 UDF matrix (all 6 typed UDFs and their column pairs):
        # | UDF                         | Bad (JSON-list) cols           | Good (_first) cols             |
        # | threat_score                | iocs_value, behaviors_ioc_value| iocs_value_first,              |
        # |                             |                                | behaviors_ioc_value_first      |
        # | threat_is_known_malicious   | iocs_value, behaviors_ioc_value| iocs_value_first,              |
        # |                             |                                | behaviors_ioc_value_first      |
        # | threat_sources              | iocs_value, behaviors_ioc_value| iocs_value_first,              |
        # | (output_type=json, kept for |                                | behaviors_ioc_value_first      |
        # |  D4 completeness)           |                                |                                |
        # | cvss_base_score             | device_cves                    | device_cves_first              |
        # | cvss_severity               | device_cves                    | device_cves_first              |
        # | cvss_vector                 | device_cves                    | device_cves_first              |
        #
        # Scope: scan only the instructional body (content before the "## Changelog"
        # section).  Changelog rows legitimately quote the retired form when documenting
        # amendments; scanning the full file would produce a false-positive FAIL on a healthy
        # runbook (F-ORCH-P10B-001).  If no changelog heading is found, the entire file is
        # scanned (graceful fallback).
        #
        # Regex notes (F-AUD-P11-LOW-002 / closing-paren exclusion):
        #   Bad patterns use UDF\(col\) — the closing \) already excludes _first forms because
        #   threat_score(iocs_value_first) has "_first" between "iocs_value" and ")", so
        #   iocs_value\) does not match. No lookahead needed.
        # F-AUD-P15-OBS-004: use module-level `import re` (already imported at top); no alias needed.
        _rb_path, _rb_tried = _find_factory_file("objectives", "T13-capstone-demo-runbook.md")
        if _rb_path is None:
            results[_H23_RESULT_KEY] = (
                f"FAIL: cannot locate runbook — tried paths: "
                f"{[str(p) for p in _rb_tried]}"
            )
        else:
            class _RunbookFenceError(Exception):
                """OBS-003: raised when fence-stripping detects an unclosed code fence.
                Caught by the inner except below; result key already set before raise."""
            try:
                with open(_rb_path, encoding="utf-8") as _rb_f:
                    _runbook_text_full = _rb_f.read()
                # Truncate at the changelog section so historical amendment descriptions
                # quoting the retired form are not counted as live instructional drift.
                # F-AUD-P15-OBS-001: guard against multiple "## Changelog" headings making
                # truncation ambiguous.
                # OBS-003: regex '^## Changelog\s*$' (MULTILINE) counts only genuine
                # section-level headings (plain str.count would match substring occurrences
                # in prose/code-fences and trigger the ambiguity guard incorrectly).
                _changelog_heading_re = re.compile(r'^## Changelog\s*$', re.MULTILINE)
                # MED-003: strip fenced code blocks before counting/locating headings — a
                # "## Changelog" line inside a code fence would trigger the multi-heading
                # guard (false-FAIL) or, if earlier in the file, silently truncate live
                # instructional body before the real section heading (false-PASS on bad forms).
                # Fence-stripping preserves character offsets by replacing non-newline chars
                # with spaces (newlines remain), so _changelog_match.start() is valid as an
                # index into _runbook_text_full.  Both the count guard and the truncation
                # position use the same fence-stripped semantics.
                _runbook_text_no_fence = re.sub(
                    r'```.*?```',
                    lambda m: re.sub(r'[^\n]', ' ', m.group(0)),
                    _runbook_text_full,
                    flags=re.DOTALL,
                )
                # OBS-003: after fence-stripping, any remaining ``` markers are orphaned
                # (unpaired fences that the regex couldn't match).  An odd count means the
                # re.sub DOTALL scan is operating on a malformed input where one or more
                # code fences were never closed — the fence-aware truncation and bad-form
                # scan are then unreliable.  Fail loudly so the runbook is fixed.
                # Known limitation (OBS-003 F-AUD-P28): the parity check catches
                # odd-count orphaned fences; a pathological even-count arrangement of
                # unpaired fences (e.g., two separate unclosed fences) would pass this
                # guard undetected.  That scenario is out of scope — standard Markdown
                # parsers reject unclosed fences, so such a runbook would already fail
                # other tooling checks.
                _fence_remaining = _runbook_text_no_fence.count("```")
                if _fence_remaining % 2 != 0:
                    results[_H23_RESULT_KEY] = (
                        f"FAIL: runbook has unclosed code fence — fence-aware scan unreliable; "
                        f"fix the runbook ({_fence_remaining} ``` markers remain after stripping)"
                    )
                    raise _RunbookFenceError()  # abort remaining H23 scan — result already set
                # F-AUD-P24-MED-003: replace count-based ambiguity FAIL with positional guard.
                # Locate the LAST occurrence of '## Changelog' (POL-32: changelog is the
                # trailing section); truncate there.  Earlier same-text headings are scanned
                # body content — they legitimately trigger the bad-form regexes if they carry
                # retired forms (which is the desired outcome).  The now-dead count-based FAIL
                # branch is removed; if multiple headings exist, the earlier ones are treated
                # as scanned content (non-blocking — the positional guard handles it).
                _changelog_matches_all = list(_changelog_heading_re.finditer(_runbook_text_no_fence))
                if not _changelog_matches_all:
                    # No changelog heading found — scan entire file (graceful fallback).
                    _runbook_text = _runbook_text_full
                    # LOW-004 (F-AUD-P26): fence-stripped full file for bad-form scan.
                    _runbook_bad_scan_text = _runbook_text_no_fence
                else:
                    # use LAST match (MULTILINE regex on fence-stripped copy); .start() offset
                    # is identical in _runbook_text_full (fence-stripping preserves offsets).
                    # Diagnostic note (non-blocking): if >1 headings exist, the earlier ones
                    # are treated as scanned body content, not truncation targets.
                    _last_changelog_match = _changelog_matches_all[-1]
                    _runbook_text = _runbook_text_full[:_last_changelog_match.start()]
                    # LOW-004 (F-AUD-P26): fence-stripped + truncated for bad-form scan so
                    # pedagogical negative examples inside code fences cannot false-FAIL.
                    # Fence-stripping preserves character offsets, so
                    # _last_changelog_match.start() is valid for both _runbook_text_full
                    # and _runbook_text_no_fence.
                    _runbook_bad_scan_text = _runbook_text_no_fence[:_last_changelog_match.start()]
                # Pattern checks run unconditionally after _runbook_text / _runbook_bad_scan_text
                # are set (regardless of whether a changelog heading was found).
                # ── Bad-form patterns: non-_first column arg for any of the 6 typed UDFs ──
                # Run against _runbook_bad_scan_text (fence-stripped + truncated) so code-fence
                # negative examples in the instructional body don't false-FAIL (LOW-004).
                # ThreatIntel UDFs × JSON-list columns (closing \) excludes _first forms)
                _bad_threatintel = re.findall(
                    r"(?:threat_score|threat_is_known_malicious|threat_sources)"
                    r"\((?:iocs_value|behaviors_ioc_value)\)",
                    _runbook_bad_scan_text,
                )
                # NVD UDFs × JSON-list column (device_cves; device_cves_first excluded by \))
                _bad_nvd = re.findall(
                    r"(?:cvss_base_score|cvss_severity|cvss_vector)\(device_cves\)",
                    _runbook_bad_scan_text,
                )
                _bad_matches_all = _bad_threatintel + _bad_nvd

                # ── Good-form patterns: _first scalar-companion column for any of the 6 UDFs ──
                # Run against _runbook_text (non-fence-stripped + truncated): _first forms
                # don't appear in pedagogical negative examples, so fence-stripping is not
                # required here.
                _good_threatintel = re.findall(
                    r"(?:threat_score|threat_is_known_malicious|threat_sources)"
                    r"\((?:iocs_value_first|behaviors_ioc_value_first)\)",
                    _runbook_text,
                )
                _good_nvd = re.findall(
                    r"(?:cvss_base_score|cvss_severity|cvss_vector)\(device_cves_first\)",
                    _runbook_text,
                )
                _good_matches_all = _good_threatintel + _good_nvd

                if _bad_matches_all:
                    results[_H23_RESULT_KEY] = (
                        f"FAIL: runbook contains {len(_bad_matches_all)} non-first UDF call(s) — "
                        f"pre-ADR-051 D4 drift (F-AUD-P10-MED-003 + HIGH-001 + MED-002); "
                        f"threatintel_bad={len(_bad_threatintel)}, nvd_bad={len(_bad_nvd)}; "
                        f"matches={_bad_matches_all[:5]!r}; "
                        f"update runbook to _first-column forms before live demo"
                    )
                elif not _good_matches_all:
                    results[_H23_RESULT_KEY] = (
                        f"FAIL: no scalar-companion (_first) UDF calls found in runbook — "
                        f"runbook enrich beats missing (expected >= 1 across all 6 typed UDFs)"
                    )
                else:
                    results[_H23_RESULT_KEY] = (
                        f"PASS: {len(_good_matches_all)} _first-form UDF call(s) found "
                        f"(threatintel={len(_good_threatintel)}, nvd={len(_good_nvd)}); "
                        f"no pre-ADR-051 non-first forms — ADR-051 D4 scalar-input confirmed in runbook; "
                        f"runbook={str(_rb_path)!r}"
                    )
            except _RunbookFenceError:
                pass  # OBS-003: result already set before raise; H23 scan aborted cleanly
            except OSError as _rb_err:
                results[_H23_RESULT_KEY] = (
                    f"FAIL: cannot read runbook at {str(_rb_path)!r}: {_rb_err}"
                )

        # ── H24: E-QUERY-043 — IN subquery in projection position ─────────────
        # F-CSD-P4-001 Option A adjudication (2026-07-10): DataFusion 53.1.0 physical planner
        # cannot execute `Expr::InSubquery` in scalar position (SELECT/GROUP BY/ORDER BY).
        # `check_expr_insubquery_projection` in materialization.rs fires a plan-time gate
        # (E-QUERY-043) before DataFusion planning.  WHERE/HAVING IN-subquery is unaffected.
        # POL-24 anchors from error.rs ExprInSubqueryProjectionNotSupported #[error]:
        #   "IN subquery in projection position is not supported"    ← from #[error] prefix
        # + materialization.rs hint:
        #   "Use a WHERE clause subquery instead"
        body, err = query(proc,
            "SELECT device_id IN (SELECT device_id FROM armis_devices) "
            "FROM crowdstrike_detections LIMIT 1",
            ["org-c"])
        if err:
            results["[H24] E-QUERY-043: IN subquery in projection position rejected"] = (
                f"FAIL: {err}"
            )
        else:
            ec = body.get("error_code", "")
            msg = body.get("message", "")
            if ec == "E-QUERY-043":
                anchor1 = "IN subquery in projection position is not supported"
                anchor2 = "Use a WHERE clause subquery instead"
                if anchor1 in msg and anchor2 in msg:
                    results["[H24] E-QUERY-043: IN subquery in projection position rejected"] = (
                        f"PASS: E-QUERY-043 — IN-subquery in projection rejected "
                        f"with canonical template (POL-24 anchors confirmed); "
                        f"message={msg[:100]!r}"
                    )
                else:
                    missing = []
                    if anchor1 not in msg:
                        missing.append(repr(anchor1))
                    if anchor2 not in msg:
                        missing.append(repr(anchor2))
                    results["[H24] E-QUERY-043: IN subquery in projection position rejected"] = (
                        f"FAIL: E-QUERY-043 received but message-template regression "
                        f"(POL-24): missing anchor(s) {', '.join(missing)}; "
                        f"message={msg[:120]!r}"
                    )
            elif body.get("rows") is not None and not ec:
                results["[H24] E-QUERY-043: IN subquery in projection position rejected"] = (
                    f"FAIL: IN-subquery in projection accepted ({len(body.get('rows', []))} rows) — "
                    f"E-QUERY-043 plan-time gate not firing (F-CSD-P4-001)"
                )
            else:
                results["[H24] E-QUERY-043: IN subquery in projection position rejected"] = (
                    f"FAIL: expected E-QUERY-043, got {ec or 'no error'}: {msg[:80]!r}"
                )

    except _PrismCrashError as _crash:
        # MED-005: prism process crashed mid-audit (BrokenPipeError/OSError from send_msg).
        # Partial results collected so far are retained; uncollected checks are absent from
        # results and will surface via the matrix-mismatch gate as "in COVERAGE_MATRIX but
        # no result written".  Record the crash as a FAIL entry so the SUMMARY shows it and
        # DEMO-READY is forced to NO.  The finally block still runs cleanup.
        results["CRASH"] = f"FAIL: {_crash}"
    except Exception as _exc:
        # F-AUD-P29-LOW-001: catch any unexpected exception so SUMMARY/DEMO-READY still
        # emits. _PrismCrashError is handled above; this handler covers any other runtime
        # failure (AttributeError, KeyError, TypeError, etc.). The traceback is printed for
        # diagnostics. AUDIT_INTERNAL_ERROR has no [NNN] prefix so it is exempt from the
        # matrix-drift check (same as BOOT and CRASH) but its "FAIL:" prefix ensures it is
        # counted in fail_count, forces DEMO-READY: NO, and exits nonzero.
        results["AUDIT_INTERNAL_ERROR"] = (
            f"FAIL: internal audit exception: {type(_exc).__name__}: {_exc}"
        )
        traceback.print_exc()  # preserve full traceback in output for diagnostics
    finally:
        # LOW-006: clean up _READ_BUF entry for this process's stdout fd (defensive against
        # future multi-process refactors; no-op when fd was never registered or already gone).
        try:
            _READ_BUF.pop(proc.stdout.fileno(), None)
        except Exception:
            pass
        try:
            proc.stdin.close()
        except Exception:
            pass
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        _mcp_log_fh.close()

    return results


# ─────────────────────────────────────────────────────────────────────────────
# Coverage matrix definition for the final report
# ─────────────────────────────────────────────────────────────────────────────
COVERAGE_MATRIX = [
    ("[A1]",  "MCP Protocol",  "INIT: server boots"),
    ("[A2]",  "MCP Protocol",  "tools/list 54-tool catalog (14 live + 40 NYA; OBS-001 union)"),
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
    ("[A23]", "MCP Protocol",  "all NYA stubs return -32003/-32602 (dynamic sweep; direct -32003 handler-gate assurance via A19/A20/A21)"),
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
    ("[G3]",  "IEQ/IIN/INE",   "IIN on status: status IIN ('new','in progress') → crowdstrike_detections OCSF-normalization variant (IIN lowers both sides; crowdstrike uses OCSF Title-case 'New'/'In Progress' stored via enum_map.rs OCSF normalization; cyberint 'open'/'closed' are vendor-native pass-through — separate probe G3b; ADV-PR-P11-HIGH-001; LOW-004)"),
    ("[G3b]", "IEQ/IIN/INE",   "Runbook Step 3.1a literal: cyberint status IIN ('open','closed') — rows>0 all status in {open,closed} (vendor-native pass-through + IIN lowercase confirmed)"),
    ("[G4]",  "IEQ/IIN/INE",   "SQL-mode IEQ rejection -> E-QUERY-001 mode-boundary"),
    # G5 RETIRED (pass-9, per gap-analysis mandate): duplicate of H6 verbatim. ID not reused.
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
    ("[H5b]", "Temporal",      "E-QUERY-042: Timestamp literal in ORDER BY arm rejected (ADR-052 §D4 arm 7; DEFECT-EQUERY042-GROUPBY-DEADARM-001 extension)"),
    ("[H5c]", "Temporal",      "E-QUERY-042: date-only literal vs function-call LHS (NonColumnLhsComparison arm 4; lower(hostname) = '2026-06-24'; ADR-052 §D4)"),
    ("[H6]",  "IEQ/IIN/INE",   "E-QUERY-002: IEQ on armis_devices.risk_score (integer column, canonical probe)"),
    ("[H7]",  "JOIN",          "JOIN positive path: crowdstrike_devices JOIN armis_devices on device_id"),
    ("[H8]",  "JOIN",          "HEAD-JOIN fail-open: bare unknown col in JOIN → not E-QUERY-038 (FP-001)"),
    ("[H9]",  "SqlPipe",       "SqlPipe mode: SELECT head + pipe stage (BC-2.11.020)"),
    ("[H10]", "Dual-limit",    "E-QUERY-040: SQL LIMIT + pipe | limit dual-limit rejected"),
    ("[H11]", "Stats",         "| stats count() as cnt by severity grammar"),
    ("[H12]", "Multi-client",  "Multi-client fan-out: org-a + org-c CrowdStrike detections"),
    ("[H13a]","Prompts",       "client_overview prompt returns promptly (new prompt)"),
    ("[H13b]","Prompts",       "cross_client_status prompt returns promptly (new prompt)"),
    ("[H14a]","Resources",     "resources/read: prism://config/clients — 3-org visibility (org-a, org-b, org-c)"),
    ("[H14b]","Resources",     "resources/read: prism://sensors/health — populated clients{} form (CRIT-001 corrected; requires A22 first; shape: {clients:{...},stale:bool})"),
    ("[H14c]","Resources",     "resources/read: prism://schema/crowdstrike/detections (per-sensor-table schema URI)"),
    ("[H14d]","Resources",     "resources/read: prism://config/clients/org-c/sensors (per-client sensor list URI)"),
    ("[H14e]","Resources",     "resources/subscribe + unsubscribe: prismql://schema/org-c (smoke-only: detects hang/panic/transport error; no client-observable subscription state exists — positive-coverage requires mutating notification trigger, excluded by read-only preflight constraint)"),
    ("[H14s]","Resources",     "resources/read: prismql://schema/org-c — cyberint_alerts present (split from H14 composite, F-AUD-P3-MED-004)"),
    ("[H15]", "Tools",         "explain_query live call (one of 14 implemented tools)"),
    ("[H16]", "Security",      "CWE-116/117: control-char in column name sanitized (sanitize_for_log)"),
    ("[H16b]","Security",      "CWE-117: control-char in WHERE-predicate value sanitized (smoke-only: detects Cc leakage in response envelope; forced echo through sanitize_for_log excluded — ident_char lexer rejects Cc chars before column lookup; positive echo coverage by H16)"),
    ("[H17]", "Guardrails",    "E-QUERY-033: limit > 1000 rejected (BC-2.11.001 ceiling)"),
    ("[H18]", "Guardrails",    "E-QUERY-003: oversize query (~80KB) controlled rejection"),
    ("[H19a]","UDFs",          "threat_sources UDF returns virustotal in result"),
    ("[H19b]","UDFs",          "cvss_vector UDF returns CVSS:3.1/ string"),
    ("[H20]", "Guardrails",    "ADR-051 D4 regression detector: iocs_value JSON-list → ALL-NULL"),
    ("[H21]", "Determinism",   "Repeated sorted query byte-identical (seeded ChaCha20)"),
    ("[H22]", "BC-2.11.018",   "normalized_pql present on success response"),
    # H23: F-AUD-P10-MED-003 + HIGH-001 runbook-side (static text check, no MCP call)
    ("[H23]", "Guardrails",    "Runbook enrich-call drift: no pre-ADR-051 non-_first forms across the 6-UDF matrix (threat_score/threat_is_known_malicious/threat_sources + cvss_base_score/cvss_severity/cvss_vector); positive _first-form usage >= 1"),
    # H24: MED-007 — E-QUERY-043 IN-subquery in projection position (F-CSD-P4-001, 2026-07-10)
    ("[H24]", "Guardrails",    "E-QUERY-043: IN subquery in projection position rejected (check_expr_insubquery_projection; F-CSD-P4-001)"),
]

# F-AUD-P14-OBS-006 (LOW-003 F-AUD-P28): exact-count equality gate — must equal
# len(COVERAGE_MATRIX); bump when adding checks, drop when removing.  The name
# EXPECTED_COVERAGE_COUNT reflects strict-equality semantics (not a floor).
# Current authoritative count: 106 (as of B-hardening pass-21 fix-burst: +H5b, +H5c, +H24).
EXPECTED_COVERAGE_COUNT = 106


if __name__ == "__main__":
    # F-AUD-P14-OBS-006 (amended F-AUD-P16-LOW-002): runtime coverage equality gate —
    # fail hard if COVERAGE_MATRIX count diverges from EXPECTED_COVERAGE_COUNT in EITHER
    # direction. Shrinkage catches accidental removal; growth catches unenrolled rows
    # (new check added without bumping EXPECTED_COVERAGE_COUNT). Bump EXPECTED_COVERAGE_COUNT
    # explicitly when adding new checks.
    if len(COVERAGE_MATRIX) != EXPECTED_COVERAGE_COUNT:
        print(
            f"ERROR: Coverage count mismatch: {EXPECTED_COVERAGE_COUNT} expected, "
            f"{len(COVERAGE_MATRIX)} present — restore removed COVERAGE_MATRIX rows or "
            f"bump EXPECTED_COVERAGE_COUNT when adding checks "
            f"(mismatch in either direction fails here)."
        )
        sys.exit(1)
    print("=" * 80)
    print("T13 COMPREHENSIVE PRE-FLIGHT DEMO AUDIT — develop baseline at AUDIT-COVERAGE-001 branch point + fix/T13-audit-coverage Section-H extension")
    print(f"  ThreatIntel port: {THREATINTEL_PORT}  NVD port: {NVD_PORT}")
    print(f"  Coverage floor: {EXPECTED_COVERAGE_COUNT} required, {len(COVERAGE_MATRIX)} present")
    print(f"  Coverage: {len(COVERAGE_MATRIX)} matrix items across 8 sections (A–H)")
    print("=" * 80)
    print()

    results = run_audit()

    # ── F-AUD-P4-OBS-001: COVERAGE_MATRIX↔results-key consistency assertion ──────
    # Extract [ID] prefix from every results key and compare against COVERAGE_MATRIX IDs.
    # F-AUD-P4-OBS-002: the "BOOT" key is exempt — it is the pre-initialize short-circuit
    # (server process failed to start before any checks ran) and intentionally has no
    # COVERAGE_MATRIX row. All other result keys must have a matching [ID] in the matrix.
    # F-AUD-P15-OBS-004: use module-level `import re` (already imported at top); no alias needed.
    _matrix_ids = {row[0] for row in COVERAGE_MATRIX}
    _result_ids = set()
    # F-AUD-P32-OBS-003: check-ID grammar: one or more uppercase letters, one or more digits,
    # zero to two lowercase suffix letters — covers current IDs ([A1], [H14b], [H13a]) and
    # hypothetical future IDs ([H14ab], [BX1]) without risk of matching synthetic keys
    # (BOOT, CRASH, AUDIT_INTERNAL_ERROR) which do not start with '['.
    for _k in results:
        _m = re.match(r'^(\[[A-Z]+[0-9]+[a-z]{0,2}\])', _k)
        if _m:
            _result_ids.add(_m.group(1))
    _matrix_only = _matrix_ids - _result_ids
    _results_only = _result_ids - _matrix_ids
    _has_mismatch = bool(_matrix_only or _results_only)
    if _has_mismatch:
        print("=" * 80)
        print("MISMATCH: COVERAGE_MATRIX↔results-key drift (F-AUD-P4-OBS-001) — FAIL")
        if _matrix_only:
            print(f"  In COVERAGE_MATRIX but no result written: {sorted(_matrix_only)}")
        if _results_only:
            print(f"  In results but not in COVERAGE_MATRIX: {sorted(_results_only)}")
        print("  → Restore parity: add missing result keys or update COVERAGE_MATRIX rows.")
        print("=" * 80)
        print()

    pass_count = 0
    fail_count = 0
    warn_count = 0
    # OBS-005 (defense-in-depth): WARN and PARTIAL counter branches MUST NOT be removed
    # even though no current check emits those prefixes.  They are retained so that any
    # FUTURE check that emits WARN or PARTIAL is still gated by the strict-success predicate
    # (F-AUD-P14-MED-003: _strict_pass requires warn_count == 0 and partial_count == 0).
    # Removing them would silently drop WARN/PARTIAL results into the INFO bucket, causing
    # counter-parity failures and DEMO-READY misreporting.
    # Status as of this pass: no PARTIAL emitters (since pass-8, F-AUD-P8-OBS-003);
    # WARN prefix unused as of pass-14.  Both remain as infrastructure, not dead code.
    partial_count = 0  # F-AUD-P5-OBS-001: separate from warn_count
    na_count = 0

    for item, result in sorted(results.items(), key=lambda kv: _audit_sort_key(kv[0])):
        if result.startswith("PASS"):
            status = "PASS"
            pass_count += 1
        elif result.startswith("FAIL"):
            status = "FAIL"
            fail_count += 1
        elif result.startswith("WARN"):
            # OBS-005: defense-in-depth — gates any future WARN emitter (none current).
            status = "WARN"
            warn_count += 1
        elif result.startswith("N/A"):
            status = "N/A "
            na_count += 1
        elif result.startswith("PARTIAL"):
            # OBS-005: defense-in-depth — gates any future PARTIAL emitter (none since pass-8).
            status = "PART"
            partial_count += 1  # F-AUD-P5-OBS-001: separate from warn_count
        else:
            status = "INFO"
        print(f"[{status}] {item}")
        print(f"       {result}")
        print()

    # F-AUD-P7-LOW-005: hard counter-parity check — every result must fall into exactly
    # one known status bucket. Any result whose prefix is not PASS/FAIL/WARN/N/A/PARTIAL
    # falls through to "INFO" and would be silently excluded from the denominator, making
    # the counters lie. Fold into _has_mismatch so demo_ready=NO + nonzero exit propagate.
    _classified_total = pass_count + fail_count + warn_count + partial_count + na_count
    if _classified_total != len(results):
        _info_count = len(results) - _classified_total
        print("=" * 80)
        print(
            f"COUNTER-PARITY FAIL (F-AUD-P7-LOW-005): {_classified_total} classified + "
            f"{_info_count} INFO-bucket = {len(results)} total; "
            f"INFO-bucket items are NOT counted in PASS/FAIL totals — "
            f"fix result prefixes to start with PASS/FAIL/WARN/N/A/PARTIAL."
        )
        print("=" * 80)
        print()
        _has_mismatch = True

    print("=" * 80)
    print(f"SUMMARY: {pass_count} PASS / {fail_count} FAIL / {warn_count} WARN / {partial_count} PARTIAL / {na_count} N/A / {len(results)} total")
    # F-AUD-P14-MED-003: strict-success contract — DEMO-READY YES requires zero FAIL,
    # zero WARN, zero PARTIAL, and no matrix mismatch. Any WARN or PARTIAL emitter is a
    # non-zero deviation from the strict-success predicate.
    _strict_pass = fail_count == 0 and warn_count == 0 and partial_count == 0 and not _has_mismatch
    demo_ready = "YES" if _strict_pass else "NO"
    # OBS-005: the (warn_count > 0 or partial_count > 0) branch is defense-in-depth;
    # no current check produces WARN or PARTIAL, but the branch MUST remain so that any
    # future check that does will surface a clear diagnostic instead of silently inflating
    # the DEMO-READY total.
    if not _strict_pass and (warn_count > 0 or partial_count > 0):
        print(
            f"STRICT-PREDICATE FAIL: {warn_count} WARN + {partial_count} PARTIAL emitters present "
            f"— strict-success contract requires zero (F-AUD-P14-MED-003)"
        )
    print(f"DEMO-READY: {demo_ready}")
    print("=" * 80)

    # F-AUD-P5-OBS-002: gate exit on _has_mismatch too — matrix mismatch must fail
    # the process, matching DEMO-READY: NO.
    # F-AUD-P14-MED-003: also gate on warn_count and partial_count.
    sys.exit(0 if _strict_pass else 1)
