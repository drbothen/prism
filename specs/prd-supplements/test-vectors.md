---
document_type: prd-supplement
supplement_type: test-vectors
version: "1.0"
parent_prd: /Users/jmagady/Dev/prism/.factory/specs/prd.md
created: 2026-04-19
created_by: product-owner
supersedes: null
status: draft
input-hash: null
---

# PRD Supplement: Canonical Test Vectors

## Purpose

This supplement catalogs canonical input/output pairs for behavioral contracts whose
correctness is verifiable via deterministic example inputs. Each vector is a binding
reference for test-writer agents producing TDD fixtures in Phase 3.

**Authoritative references:**
- BC files (behavioral-contracts/BC-*.md) — source of truth for postconditions
- error-taxonomy.md — source of truth for error codes and messages
- interface-definitions.md — source of truth for schema shapes

When a BC's postconditions or error taxonomy changes, its canonical test vectors must
be updated in the same commit (Policy 7: source-of-truth integrity).

## Conventions

- Each vector block MUST include: BC reference, input, expected output, error case references.
- Inputs that embed secrets, tenant data, or PII MUST use placeholder markers
  (`<CREDENTIAL_REF>`, `<CLIENT_ID>`) and state the substitution rule.
- Non-deterministic fields (timestamps, UUIDs) MUST be marked `<GENERATED>` with a
  generation rule note.
- Every vector block ends with a `**Trace:** <BC>, <VP>` line linking to the property(ies)
  that use it.

---

## TV-001 — BC-2.05.003: Audit Entry Credential Redaction

**Anchor BC:** BC-2.05.003 (Credential Values Are Never Present in Audit Entries)

**Input (sensor write operation with credential reference):**
```json
{
  "tool": "execute_action",
  "sensor": "crowdstrike-prod",
  "client_id": "<CLIENT_ID>",
  "parameters": {
    "host_id": "abc-123",
    "credential_ref": "<CREDENTIAL_REF:crowdstrike_api_key>",
    "action": "contain"
  },
  "timestamp": "<GENERATED:ISO8601-UTC>"
}
```

**Expected audit entry (JSON, written to audit_buffer CF):**
```json
{
  "user_identity": "<CLIENT_ID>",
  "tool_name": "execute_action",
  "parameters": {
    "host_id": "abc-123",
    "credential_ref": "[REDACTED]",
    "action": "contain"
  },
  "timestamp": "<GENERATED:ISO8601-UTC>",
  "client_id": "<CLIENT_ID>",
  "sensor": "crowdstrike-prod",
  "capability_checks": [
    {"capability": "WRITE", "status": "granted"}
  ],
  "result_summary": {"outcome": "success", "resource_id": "abc-123"}
}
```

**Invariant:** The string `"[REDACTED]"` replaces any field value that was referenced by
`<CREDENTIAL_REF:*>` in the input. No substring of the actual credential value may
appear anywhere in the entry body.

**Trace:** BC-2.05.003, VP-034

---

## TV-002 — BC-2.04.009: Confirmation Token Generation

**Anchor BC:** BC-2.04.009 (Confirmation Token Generation for Irreversible Write Operations)

**Input (write-tool precheck requesting confirmation):**
```json
{
  "tool": "execute_action",
  "action": "isolate_host",
  "irreversible": true,
  "analyst_id": "analyst-42"
}
```

**Expected response (confirmation token):**
```json
{
  "confirmation_token": "<GENERATED:UUID-v4>",
  "expires_at": "<GENERATED:ISO8601-UTC+15m>",
  "action_summary": "Isolate host on crowdstrike-prod; irreversible=true"
}
```

**Cap test (active tokens = 100):**
- Input: 101st generation request with same analyst_id.
- Expected: rejection with `E-CONFIRM-001` (cap exceeded) + existing tokens untouched.

**Trace:** BC-2.04.009, VP-010

---

## TV-003 — BC-2.11.001: Query Tool Scoping

**Anchor BC:** BC-2.11.001 (`query` MCP Tool Accepts Scoping + PrismQL Query String)

**Input:**
```json
{
  "tool": "query",
  "scope": {
    "clients": ["acme-corp"],
    "sensors": ["crowdstrike-prod", "cyberint-prod"]
  },
  "query": "SELECT hostname, process FROM processes WHERE user='root' LIMIT 100"
}
```

**Expected behavior:** Query planner expands scope into per-sensor federated subqueries;
returns OCSF-normalized rows bounded by the LIMIT.

**Error case:** Missing `scope.clients` → reject with `E-QUERY-001` (required scope absent).

**Trace:** BC-2.11.001, VP-014

---

## TV-004 — BC-2.11.012: Virtual Fields in Queries

**Anchor BC:** BC-2.11.012 (Virtual Fields in Queries — `_sensor`, `_client`, `_source_table`)

**Input (query using virtual fields):**
```sql
SELECT _sensor, _client, _source_table, COUNT(*)
FROM processes
WHERE _sensor = 'crowdstrike-prod'
GROUP BY _sensor, _client, _source_table
```

**Expected behavior:** Planner injects virtuals at scan time; each row carries the
originating sensor name, client id, and source-table identifier. Virtuals never
collide with user data columns (always prefixed with underscore).

**Canonical virtual field set (exhaustive):** `_sensor`, `_client`, `_source_table`.

**Trace:** BC-2.11.012, VP-015

---

## TV-005 — BC-2.13.014: IOC File Loading

**Anchor BC:** BC-2.13.014 (IOC File Loading and Pattern Store — At-Startup Load with Hot Reload and Bounded Memory)

**NOTE:** Values in this vector (100,000 patterns / 10 MB) are taken directly from
BC-2.13.014 postconditions and invariants (INV-IOC-003), which are the source of truth.
Story bodies and ACs referencing different values (e.g., 1,000,000 / 50 MB) have drifted
from the BC. Resolve by reading BC-2.13.014 body — story-writer must update S-4.03 Task 8a
and AC-9 to match.

**Input (startup IOC load from `{config_dir}/ioc/blacklist-ips.ioc`):**
```
10.0.0.1
10.0.0.2
# comment
10.0.0.3
```

**Expected pattern store (Rust representation):**
```
PatternStore {
  "blacklist-ips" => RegexSet {
    patterns: ["10.0.0.1", "10.0.0.2", "10.0.0.3"],  // compiled as regex::Regex each
    count: 3
  }
}
```

**Limits (per BC-2.13.014 INV-IOC-003):**
- Max patterns per file: **100,000** → rejection with `E-IOC-003` (pattern count exceeded).
- Max file size: **10 MB** → rejection with `E-IOC-002` (size limit exceeded).
- Max IOC files: **50** → rejection with `E-IOC-004` (file count cap).
- Invalid regex pattern in file: rejection with `E-IOC-001`; prior `RegexSet` retained.
- Malformed line with valid other lines: skip with WARN log; do not crash (INV-IOC-004).

**UDF registration:** `ioc_match(field_expr: Utf8, list_name: Utf8) -> Boolean` available
at query time. If `list_name` not found: returns `false` for all rows + `E-UDF-001` WARN.

**Boundary edge case (EC-13-038):** File with exactly 100,000 patterns loads successfully.
**Boundary edge case (EC-13-039):** File with 100,001 patterns is rejected with `E-IOC-003`.

**Trace:** BC-2.13.014, VP-018

---

## TV-006 — BC-2.14.002: Case State Transitions

**Anchor BC:** BC-2.14.002 (Case State Transitions — 5-State Machine with 12 Valid Transitions)

**States:** `New`, `In_Progress`, `Contained`, `Resolved`, `False_Positive`.

**Transition matrix (valid transitions only):**

| From | To | Allowed |
|------|----|---------|
| New | In_Progress | yes |
| New | False_Positive | yes |
| In_Progress | Contained | yes |
| In_Progress | Resolved | yes |
| In_Progress | False_Positive | yes |
| Contained | Resolved | yes |
| Contained | In_Progress | yes (re-open) |
| Resolved | In_Progress | yes (re-open) |
| False_Positive | (terminal) | no |
| any | same state | no |

**Valid transitions:** 12. Invalid transitions reject with `E-CASE-003`.

**Trace:** BC-2.14.002, VP-005, VP-006

---

## TV-007 — BC-2.04.005: Hidden Tools Pattern

**Anchor BC:** BC-2.04.005 (Hidden Tools Pattern — Stateless Tool List Based on Configured Capabilities)

**Input (client connects with feature-flags `write_actions: false`):**
```yaml
enabled_capabilities:
  - READ
  - QUERY
  # WRITE not in list
```

**Expected tools/list response (subset):**
```json
{
  "tools": [
    {"name": "query", "description": "..."},
    {"name": "list_cases", "description": "..."}
  ]
}
```

`execute_action`, `create_case`, `update_case`, `delete_rule` MUST be absent from the
response — not returned-with-error, but ABSENT from the list.

**Semantic invariant:** The list is a stateless function of configured capabilities; no
runtime enabled/disabled state is tracked. Re-reading the capability config produces
the same tool list.

**Trace:** BC-2.04.005, VP-003

---

## TV-008 — BC-2.10.006: Stdio Transport Semantics

**Anchor BC:** BC-2.10.006 (Stdio Transport)

**Scope:** This BC governs MCP JSON-RPC over stdin/stdout. It does NOT govern log
forwarding (S-5.09) or trust-level metadata (BC-2.09.005). Any story referencing
BC-2.10.006 must constrain claims to stdio framing semantics.

**Input (JSON-RPC request on stdin, one line):**
```
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
```

**Expected response (JSON-RPC response on stdout, one line):**
```
{"jsonrpc":"2.0","id":1,"result":{"tools":[...]}}
```

**Invariants:**
- stdout contains ONLY framed JSON-RPC responses; no log messages ever printed to stdout.
- stderr receives all log output.
- One process = one session = one analyst; RocksDB LOCK enforces single-process.

**Trace:** BC-2.10.006, DI-017

---

## TV-009 — BC-2.05.011: Audit Forwarder Watermark Monotonicity

**Anchor BC:** BC-2.05.011 (Audit Forwarding At-Least-Once with Backoff)

**Setup:**
- 2 destinations configured: `vector-prod`, `splunk-prod`.
- audit_buffer contains entries 1..10.

**Scenario 1 — Normal forwarding:**
- All deliveries succeed for both destinations.
- Watermarks advance monotonically: `forward_watermark:vector-prod = 10`, `forward_watermark:splunk-prod = 10`.
- No backoff applied.

**Scenario 2 — Transient outage on vector-prod:**
- vector-prod returns 503 for entries 3..5; splunk-prod succeeds throughout.
- Retry backoff: 2s, 4s, 8s. After 3rd attempt (backoff 8s), vector-prod recovers.
- Watermarks: `splunk-prod = 10` throughout; `vector-prod` stays at 2 during outage, advances to 10 after recovery.

**Scenario 3 — Permanent failure (E-AUDIT-005):**
- vector-prod returns 400 on entry 5 (malformed payload per destination).
- Entry 5 is skipped (watermark advances past), ERROR logged with entry reference.
- Entries 6..10 continue delivering normally.

**Scenario 4 — Buffer cap eviction:**
- audit_buffer size exceeds `buffer_cap_mb`.
- ONLY entries with `min(watermark_across_destinations) >= entry_seq` are evicted (FIFO).
- Entries not yet delivered to at least one destination are NEVER evicted.

**Trace:** BC-2.05.011, VP-039, INV-AUDIT-FWD-001/002/003

---

## TV-010 — BC-2.16.001: Sensor Spec File Loading

**Anchor BC:** BC-2.16.001 (Sensor Spec File Loading)

**Input (`{config_dir}/sensors/crowdstrike-prod.toml`):**
```toml
[sensor]
name = "crowdstrike-prod"
type = "crowdstrike"
version = "7.x"

[api]
base_url = "https://api.crowdstrike.com"
auth = { type = "oauth2_client_credentials", credential_ref = "<CREDENTIAL_REF:cs_oauth>" }

[capabilities]
read = true
write = true
query = ["processes", "host_info", "alerts"]
```

**Expected load-time validation:**
- Missing required fields → rejection with `E-SPEC-001` + file path in message.
- Unknown `type` value → rejection with `E-SPEC-002`.
- Malformed TOML → rejection with `E-SPEC-003`.

**Hot-reload scenario (BC-2.16.007):**
- File modified at runtime.
- Validation runs on new version; if invalid, PREVIOUS spec stays active (reload atomicity).
- If valid, new spec activates atomically; no in-flight queries see half-reloaded state.

**Trace:** BC-2.16.001, BC-2.16.007, DI-030, DI-031, VP-023

---

## Traceability Matrix (supplement-wide)

| Vector | Anchor BC | Anchor DIs | VPs that may consume |
|--------|-----------|-----------|---------------------|
| TV-001 | BC-2.05.003 | DI-002 | VP-034 |
| TV-002 | BC-2.04.009 | DI-015 | VP-010 |
| TV-003 | BC-2.11.001 | DI-019 | VP-014 |
| TV-004 | BC-2.11.012 | DI-020 | VP-015 |
| TV-005 | BC-2.13.014 | DI-019, DI-024 | VP-018 |
| TV-006 | BC-2.14.002 | DI-025 | VP-005, VP-006 |
| TV-007 | BC-2.04.005 | DI-003 | VP-003 |
| TV-008 | BC-2.10.006 | DI-017 | — (integration test; no Kani/Proptest VP) |
| TV-009 | BC-2.05.011 | INV-AUDIT-FWD-001 | VP-039 |
| TV-010 | BC-2.16.001 | DI-030, DI-031 | VP-023 |

## Change log

- v1.0 (2026-04-19): Initial catalog seeded with 10 vectors across 8 subsystems (audit, authn/feature-flags, query, detection, case, stdio, spec-engine); Burst 27 closure of P3P26-A-H-006. TV-005 limits corrected to match BC-2.13.014 source of truth (100K patterns / 10 MB, not 1M / 50 MB).
