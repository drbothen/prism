---
document_type: prd-supplement-test-vectors
level: L3
version: "2.6"
status: draft
producer: product-owner
timestamp: 2026-04-19T00:00:00Z
phase: 1a
inputs: [prd.md, behavioral-contracts/]
input-hash: "4ed2d4d"
traces_to: prd.md
---

# Canonical Test Vectors: Prism

> PRD supplement — extracted from PRD Section 5b.
> Referenced by: test-writer, implementer, holdout-evaluator, consistency-validator.

## Conventions

- **BC source-of-truth rule:** Every vector in this file is derived from the anchor BC's
  Postconditions section. If a BC's postconditions and a vector here conflict, the BC wins
  and this file must be updated in the same commit.
- **Credential placeholders:** Inputs that embed credential material use `<CREDENTIAL_REF:*>`
  (e.g., `<CREDENTIAL_REF:cs_oauth>`). These are never substituted with real values.
- **Client placeholder:** `<CLIENT_ID>` substitutes for a valid TenantId in test fixtures.
- **Non-deterministic fields:** Timestamps, generated IDs, and elapsed times are marked
  `<GENERATED>` with a generation rule note (e.g., `<GENERATED:ISO8601-UTC>`,
  `<GENERATED:created_at+300s>`).
- **Error codes:** All error codes reference `error-taxonomy.md` as canonical source.
  The CONFIRM error namespace is REMOVED — see error-taxonomy.md line 270. Use `E-FLAG-003`
  (token expired) or `E-FLAG-007` (cap reached) for confirmation token errors.
- **Trace line format:** Each BC section ends with a `**Trace:**` line listing the BC ID,
  VP IDs, and DI/invariant IDs it covers.

---

## Per-Subsystem Test Vectors

> **Scope note (v2.6):** Per-Subsystem Test Vectors covers the 7 highest-risk subsystems
> in the Phase-2 scope (SS-04, SS-05, SS-11, SS-13, SS-14, SS-10, SS-16). Remaining
> subsystems defer to integration-level test vectors defined in individual BC files and
> the story-level acceptance criteria. This is a deliberate scoping decision — not a gap.

### Subsystem: SS-05 Audit (CAP-007, CAP-025)

#### BC-2.05.003: Credential Values Are Never Present in Audit Entries

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| `fire_action` with `parameters.credential_ref = "<CREDENTIAL_REF:cs_oauth>"` | Audit entry has `parameters.credential_ref = "[REDACTED]"`; no substring of the actual credential value appears anywhere in the entry body | happy-path | TV-001; field name preserved, value replaced |
| `fire_action` with nested parameter `parameters.auth.api_key = "<secret>"` | Audit entry has `parameters.auth.api_key = "[REDACTED]"` (recursively applied to all nesting depths) | edge-case | EC-05-004 |
| `fire_action` with parameter `hostname = "my_token_server"` (value contains `_token` substring) | Audit entry preserves `hostname = "my_token_server"` unchanged; only fields whose **names** match secret patterns are redacted | edge-case | EC-05-005; value-substring does not trigger redaction |
| Any audit entry body scan across all fields (result_summary, capability_checks, safety_flags) | Zero occurrences of any credential value substring in any field | invariant | DI-002 enforced; integration only (no Kani/Proptest VP anchored) |

**Trace:** BC-2.05.003 postconditions 1-4, DI-002

---

#### BC-2.05.011: Audit Forwarding — At-Least-Once Delivery with Exponential Backoff

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| 2 destinations (`vector-prod`, `splunk-prod`); `audit_buffer` entries 1..10; all deliveries succeed | Watermarks advance to 10 on both destinations; no backoff applied; no entries lost | happy-path | TV-009 scenario 1; INV-AUDIT-FWD-001 |
| `vector-prod` returns HTTP 503 for entries 3..5; `splunk-prod` succeeds throughout | `vector-prod` retries at 2s/4s/8s; `splunk-prod` watermark = 10 throughout; `vector-prod` watermark stays at 2 during outage, advances to 10 after recovery | edge-case | TV-009 scenario 2; INV-AUDIT-FWD-002; transient failure |
| `vector-prod` returns HTTP 400 on entry 5 (permanent failure) | Entry 5 skipped (watermark advances past 5); `E-AUDIT-005` WARN emitted with entry reference; entries 6..10 forwarded normally | error | TV-009 scenario 3; permanent failure path |
| `audit_buffer` exceeds `buffer_cap_mb` (100K cap per BC-2.15.004); lagging destination prevents GC | Only entries where `min(watermark_across_destinations) >= entry_seq` are FIFO-evicted; undelivered entries for any destination are NEVER evicted unless buffer-full last resort; CRITICAL log emitted on any eviction | edge-case | TV-009 scenario 4; INV-AUDIT-FWD-003; INV-AUDIT-FWD-004 |
| Server restarts mid-forward (killed after destination ACK, before watermark write) | On restart, forwarding task reads last durable watermark from RocksDB; ACKed entry may be re-forwarded (at-least-once; harmless duplicate) | edge-case | EC-05-021; INV-AUDIT-FWD-002 |

**Trace:** BC-2.05.011 postconditions 1-4, VP-039, INV-AUDIT-FWD-001/002/003/004, DI-026

---

### Subsystem: SS-04 Feature Flags and Write Gate (CAP-005, CAP-006)

#### BC-2.04.005: Hidden Tools Pattern — Stateless Tool List Based on Configured Capabilities

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| Configuration: no clients have any write capabilities enabled | `tools/list` response contains only read-only tools; `fire_action`, `create_case`, `update_case`, `delete_rule`, `configure_credential_source` are completely ABSENT (not returned as disabled — absent) | happy-path | TV-007; EC-04-011 |
| Configuration: Client A has `case.write = true`, Client B has `case.write = false`; `tools/list` requested | `create_case` and `update_case` appear in `tools/list` (enabled for at least one client) | happy-path | EC-04-010; per-invocation enforcement, not per-list filtering |
| Re-read `tools/list` with identical configuration (second call, no state change) | Identical tool list returned (stateless function of config; no runtime enabled/disabled state tracked) | invariant | DI-003; VP-003 |
| Agent invokes `create_case` with `client_id: "b"` where Client B lacks `case.write` | Structured error `E-FLAG-001` with denied capability path; tool is NOT "unknown tool" (it appeared in list) | error | EC-04-010; per-invocation gating |
| Agent invokes write tool with `client_id: null` | Structured error `E-FLAG-006`; cross-client writes not supported | error | EC-04-033 |

**Trace:** BC-2.04.005 postconditions, VP-003, DI-003

---

#### BC-2.04.009: Confirmation Token Generation for Irreversible Write Operations (100-Token Active Cap)

> CRITICAL: Values in this section are derived from BC-2.04.009 body (canonical source).
> Token TTL = **300 seconds (5 minutes)**. Token ID = cryptographic random string.
> Cap error = **E-FLAG-007**. The CONFIRM error namespace is REMOVED (see error-taxonomy.md line 270).

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| Irreversible write tool invoked (e.g., `contain_host`); `analyst_id = "analyst-42"`; params valid; fewer than 100 active tokens | Response: `{token_id: "<GENERATED:crypto-random>", client_id: "<CLIENT_ID>", tool_name: "contain_host", action_summary: "...", expires_at: "<GENERATED:created_at+300s>"}` written to in-memory token store; write operation NOT executed | happy-path | TV-002; 300s TTL per DI-007 |
| Same analyst; 100 active (non-expired) tokens already in store; 101st generation request | Expired tokens swept first; if store still at 100 after sweep, rejection with `E-FLAG-007` (`"Token store capacity reached (100 active tokens)"`); existing 100 tokens unchanged | error | TV-002 cap test; EC-04-019 |
| Token generated at T=0; confirmed at T=299s | Token accepted; write operation executed; token consumed from store | happy-path | Expiry boundary; 300s TTL validated |
| Token generated at T=0; confirmation attempt at T=301s | Rejection with `E-FLAG-003` (token expired); no write executed | error | VP-007 boundary; BC-2.04.011 expiry enforcement |
| Same action params submitted twice by same analyst | Two independent tokens created; both valid until consumed or expired (subject to cap) | edge-case | EC-04-018; token per-request, not per-action |
| Server restart while 5 tokens are active | All 5 in-memory tokens lost; analyst must re-request; acceptable — tokens are short-lived (5 min) | edge-case | EC-04-034; in-memory only, not persisted |

**Trace:** BC-2.04.009 postconditions, VP-010, DI-007, DI-015

---

### Subsystem: SS-11 Query Engine (CAP-015, CAP-016)

#### BC-2.11.001: `query` MCP Tool Accepts Scoping + PrismQL Query String

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| `{clients: ["acme-corp"], sensors: ["crowdstrike", "cyberint"], query: "SELECT hostname, process FROM processes WHERE user='root' LIMIT 100"}` | Planner expands scope into per-sensor federated subqueries for (acme-corp, crowdstrike) and (acme-corp, cyberint); returns OCSF-normalized rows bounded by LIMIT 100; `query_context` includes `clients_queried`, `sensors_queried`, `is_truncated`, `total_available` | happy-path | TV-003; DI-008 provenance |
| `{query: "..."}` with no clients configured in Prism at all | Rejection with `E-CFG-001` (no matching clients/sensors found) | error | Required scope absent |
| `{clients: ["acme"], sensors: ["crowdstrike"], query: "SELECT * FROM alerts"}` where query text also contains `client_id = "globex"` | Intersection of tool-level scope and query predicate is empty; empty result set returned with metadata explaining intersection was empty; not an error | edge-case | EC-11-001; scope intersection narrowing |
| Query that would materialize more than 10,000 records | Rejection with `E-QUERY-006` (materialization cap); structured error with per-sensor counts and narrowing suggestions | error | DEC-023; DI-019 enforced |
| Query execution exceeds 30 seconds | Rejection with `E-QUERY-004` (timeout); no partial results | error | DEC-026; DI-019 enforced |
| `{limit: 25}` but 500 records match | Returns 25 records with `is_truncated: true`, `total_available: 500` | happy-path | EC-11-032; ephemeral model — no cursor |

**Trace:** BC-2.11.001 postconditions, VP-014, DI-004, DI-008, DI-019

---

#### BC-2.11.012: Virtual Fields in Queries — `_sensor`, `_client`, `_source_table`

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| `SELECT _sensor, _client, _source_table, COUNT(*) FROM processes GROUP BY _sensor, _client, _source_table` | Each result row carries originating `_sensor` (e.g., `"crowdstrike"`), `_client` (TenantId), `_source_table` (e.g., `"processes"`) values injected at scan time; no data column collision | happy-path | TV-004; virtual fields as Arrow columns |
| `_sensor = "crowdstrike"` predicate in query AND `sensors: ["cyberint"]` in tool params | Intersection: empty result set (crowdstrike excluded by tool scope); `_sensor` predicate participates in scope intersection | edge-case | EC-11-001 analogue; scope intersection |
| `_sensor > "armis"` (numeric comparison on virtual field) | Type error `E-QUERY-002`: "Field '_sensor' is a string virtual field. Use = or != for comparison." | error | Virtual field type enforcement |
| `SELECT _sensor, _client, _source_table FROM events` (project only virtuals) | Valid projection; returns only virtual field columns for each event | edge-case | EC-11-030 |
| `SELECT * FROM events` with internal table `prism.alerts` in scope | `_sensor = "prism"`, `_source_table = "alerts"` injected for internal table rows; no API fan-out for internal tables | happy-path | BC-2.11.001 internal table path |

**Canonical virtual field set (exhaustive):** `_sensor`, `_client`, `_source_table`. No other underscore-prefixed names are reserved virtual fields.

**Trace:** BC-2.11.012 postconditions, VP-015, DI-020

---

### Subsystem: SS-13 Detection Engine (CAP-020)

#### BC-2.13.014: IOC File Loading and Pattern Store — At-Startup Load with Hot Reload and Bounded Memory

> Canonical limits per BC-2.13.014 INV-IOC-003: **100,000 patterns/file**, **10 MB/file**, **50 files max**.
> Error codes: `E-IOC-001` (invalid regex), `E-IOC-002` (size exceeded), `E-IOC-003` (pattern count exceeded), `E-IOC-004` (file count exceeded), `E-UDF-001` (unknown list in ioc_match).

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| `{config_dir}/ioc/blacklist-ips.ioc` with 3 valid IP patterns and 1 comment line | `PatternStore["blacklist-ips"] = RegexSet` with 3 compiled patterns; comment lines discarded; INFO log emitted | happy-path | TV-005; EC-13-038 boundary setup |
| IOC file with exactly 100,000 valid patterns | File loads successfully; `RegexSet` built with 100,000 patterns | edge-case | EC-13-038; boundary at cap — must succeed |
| IOC file with 100,001 patterns | File rejected with `E-IOC-003`; prior `RegexSet` for that file retained in pattern store; WARN log emitted | error | EC-13-039; INV-IOC-003 enforced |
| IOC file size 10.01 MB | File rejected before compilation with `E-IOC-002`; prior state retained | error | Size cap; INV-IOC-003 |
| 51st `.ioc` file added to `ioc/` directory (50 already loaded) | 51st file rejected with `E-IOC-004`; all 50 existing files continue to function | error | EC-13-045; file count cap |
| IOC file containing one invalid regex pattern (e.g., backtracking `(a+)+b`) | Whole file rejected with `E-IOC-001`; WARN log lists first 3 failing patterns; prior `RegexSet` for that file retained | error | EC-13-043; INV-IOC-004 (no crash) |
| IOC file with 1 malformed line among 99 valid lines (e.g., encoding error on line 7) | File rejected with `E-IOC-001` if any pattern fails (whole-file rejection policy); prior state retained | error | INV-IOC-003 (no partial load) |
| IOC file with 0 patterns (empty after stripping comments) | File accepted; `RegexSet` with 0 patterns; `ioc_match` returns `false` for all rows; INFO log emitted | edge-case | EC-13-044 |
| `ioc_match('blacklist-ips', ip_value)` after successful load | UDF evaluates per-row; returns `true` for matching IPs, `false` otherwise; registered at query startup | happy-path | UDF registration and execution |
| `ioc_match('missing-list', value)` where `'missing-list'` not in pattern store | Returns `false` for all rows; WARN log `"ioc_match: list 'missing-list' not found; returning false for all rows"`; query continues (not fatal) | error | EC-13-041 analogue; E-UDF-001 |
| Hot reload while `ioc_match` query is in-flight against 500K rows | In-flight query completes against pre-reload `RegexSet` snapshot (arc-swap guard); hot reload proceeds concurrently; results deterministic for snapshot | edge-case | EC-13-040; INV-IOC-001/002 |

**Trace:** BC-2.13.014 postconditions, VP-018, DI-019, DI-024, INV-IOC-001/002/003/004

---

### Subsystem: SS-14 Case Management (CAP-022)

#### BC-2.14.002: Case State Transitions — 5-State Machine with 12 Valid Transitions

> CRITICAL: Canonical states per BC-2.14.002 and DI-025 are: **New**, **Acknowledged**,
> **Investigating**, **Resolved**, **Closed**. `In_Progress`, `Contained`, and `False_Positive`
> are NOT states. `FalsePositive` is a DispositionCode (a separate field), not a state.
> Error codes: `E-CASE-004` (invalid transition), `E-CASE-005` (self-transition),
> `E-CASE-006` (Resolved without disposition).

**Valid transitions (12, exhaustive per BC-2.14.002 postconditions):**

| From | To | Type |
|------|----|------|
| New | Acknowledged | Forward (linear) |
| Acknowledged | Investigating | Forward (linear) |
| Investigating | Resolved | Forward (linear) |
| Resolved | Closed | Forward (linear) |
| New | Investigating | Forward (skip) |
| New | Resolved | Forward (skip) |
| New | Closed | Forward (skip) |
| Acknowledged | Resolved | Forward (skip) |
| Acknowledged | Closed | Forward (skip) |
| Investigating | Closed | Forward (skip) |
| Resolved | Investigating | Reopen |
| Closed | Investigating | Reopen |

**Test vectors:**

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| Transition: `New → Acknowledged` | Success; `case.state = Acknowledged`; `StatusChange` timeline entry written | happy-path | TV-006; forward linear |
| Transition: `New → Closed` (skip all intermediate states) | Success; useful for false positive dismissal | happy-path | EC-14-005 |
| Transition: `Resolved → Investigating` (reopen) | Success; `closed_at` cleared to null; `resolved_at` preserved (first resolution time retained for MTTR) | happy-path | Reopen path |
| Transition: `Closed → Investigating` (reopen) | Success; `closed_at` cleared; `resolved_at` preserved | happy-path | EC-14-006 |
| Transition: `Investigating → Resolved` WITHOUT disposition set | Rejection with `E-CASE-006`: "Disposition is required before resolving a case." | error | Invariant: disposition required for Resolved |
| Transition: `Investigating → Resolved` WITH `disposition: FalsePositive` set | Success; `case.state = Resolved`; `case.disposition = FalsePositive`; `resolved_at` set to current UTC | happy-path | Disposition is a separate field, not a state |
| Transition: `Closed → New` (backward to initial state) | Rejection with `E-CASE-004`: "Cannot transition from Closed to New. Valid targets: [Investigating]" | error | Backward transitions to New/Acknowledged disallowed |
| Transition: `Acknowledged → New` (backward) | Rejection with `E-CASE-004`; only `Investigating` is a valid reopen target | error | All backward-to-initial-state transitions rejected |
| Transition: `Investigating → Investigating` (self-transition) | Rejection with `E-CASE-005`: "Case is already in Investigating status" | error | Self-transitions rejected |
| `case.state = Closed`; any outgoing transition attempted | Rejection with `E-CASE-004`; Closed is terminal EXCEPT for reopen to Investigating | error | Closed → Investigating is valid; Closed → anything-else is terminal |
| 3 rapid valid transitions within 1 second on the same case | All accepted; each generates a separate `StatusChange` timeline entry with sub-second precision | edge-case | EC-14-007 |
| Concurrent transition requests for the same case (2 simultaneous callers) | RocksDB write serializes; second request sees updated state; second transition may fail if now invalid | edge-case | EC-14-008 |

**Trace:** BC-2.14.002 postconditions, VP-005, VP-006, DI-004, DI-025

---

### Subsystem: SS-10 MCP Interface (CAP-034)

#### BC-2.10.006: Stdio Transport

> Scope: BC-2.10.006 governs MCP JSON-RPC over stdin/stdout framing ONLY.
> Does NOT govern log forwarding (S-5.09) or trust-level metadata (BC-2.09.005).
> RocksDB LOCK enforcement (single-process) is governed by BC-2.15.001.

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| `{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}` sent on stdin (one line) | `{"jsonrpc":"2.0","id":1,"result":{"tools":[...]}}` on stdout (one line); no log content on stdout | happy-path | TV-008; stdout purity |
| Log statement emitted during request handling (e.g., INFO trace from handler) | Log line written to stderr ONLY; stdout receives zero bytes of non-JSON-RPC content during entire session | invariant | DI-017 via stdout purity invariant |
| stdin pipe broken (MCP client process dies) | Prism detects broken pipe; initiates graceful shutdown (BC-2.10.010); no crash | error | FM-011; BC-2.10.010 invoked |
| Very large MCP response (>1 MB of sensor data from query) | Response written as a single JSON-RPC message; no MCP-level chunking; pagination at tool level keeps individual responses bounded | edge-case | EC-10-011 |
| Prism binary launched without stdin connected (e.g., accidental direct launch) | Immediate stdin read error; Prism exits with error message to stderr | error | EC-10-010 |

**Trace:** BC-2.10.006 postconditions, DI-017 (single-process via BC-2.15.001 RocksDB LOCK)

---

### Subsystem: SS-16 Spec Engine (CAP-029, CAP-030)

#### BC-2.16.001: Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| Valid `crowdstrike-prod.sensor.toml` with `sensor_id`, `name`, `auth_type`, `base_url`, `tables`, `rate_limit_hints`, `version` | Spec parsed into `SensorSpec`; each `TableSpec` registered as DataFusion table with name `{sensor_id}.{table_name}`; columns translated to Arrow schema; OCSF field mappings registered; spec included in `ConfigSnapshot` | happy-path | TV-010 |
| Spec file with TOML parse error (e.g., unclosed bracket) | Rejection with `E-SPEC-001` including file path, line number, and parse error message; other valid spec files load normally (DI-030) | error | Tier 3 independent validation |
| Spec file with duplicate `sensor_id` already claimed by a previously loaded spec | Second file rejected with `E-SPEC-009`; first spec wins; no partial registration | error | Duplicate sensor_id handling |
| Spec file with duplicate `table_name` within the same sensor | Entire spec file rejected with `E-SPEC-004` | error | Intra-spec duplicate table |
| Empty `sensor_specs_dir` (zero `.sensor.toml` files) | No config-driven sensors registered; startup succeeds; zero sensor tables available | edge-case | Empty directory is valid |
| Non-`.toml` file in `sensor_specs_dir` | File ignored with debug-level log; other valid specs load normally | edge-case | Extension filter |

**Trace:** BC-2.16.001 postconditions, DI-008, DI-030, VP-023

---

#### BC-2.16.007: Sensor Spec Hot Reload — Add/Remove/Update Sensor Tables Without Restart

| Input | Expected Output | Category | Notes |
|-------|-----------------|----------|-------|
| Valid spec file modified at runtime (column added); `reload_config` invoked | New spec activates atomically via arc-swap; schema change triggers `notifications/tools/list_changed` MCP notification; reload result includes `"modified": ["sensor_id.table_name"]` with `"schema_changed": true` | happy-path | TV-010 co-anchor; DI-031 |
| Invalid spec file modified at runtime (TOML syntax error); `reload_config` invoked | Previous spec stays active; validation error returned in reload result alongside successful updates for other specs; no partial registration | edge-case | DI-030 + DI-031 atomic rollback |
| Spec file deleted from `sensor_specs_dir`; `reload_config` invoked | Tables unregistered from DataFusion catalog; queries targeting removed tables return `E-QUERY-011` | edge-case | Removed spec path |
| New spec file added to `sensor_specs_dir`; `reload_config` invoked | New `SensorSpec` loaded; tables registered; immediately queryable; reload result includes `"added"` list | happy-path | New spec path |
| `reload_config` invoked while a query is in-flight using old spec | In-flight query uses `ConfigSnapshot` captured at query start (arc-swap guard); completes against old schema; next query uses new schema | edge-case | DEC-037; in-flight query safety |
| Non-schema field updated in spec (e.g., `rate_limit_hints` changed); `reload_config` invoked | Spec re-registered; no `list_changed` notification sent (schema unchanged); reload result includes `"schema_changed": false` | edge-case | Non-schema reload; no agent notification |

**Trace:** BC-2.16.007 postconditions, DI-030, DI-031, VP-023

---

## Cross-Subsystem Integration Vectors

| Scenario | Input | Step 1 Output | Step 2 Input | Final Output |
|----------|-------|---------------|-------------|-------------|
| Irreversible write with audit + credential redaction | `crowdstrike_contain_host` (irreversible=true) with `credential_ref` in params | Confirmation token issued (BC-2.04.009); write NOT executed; token stored in-memory | `confirm_action` with valid token within 300s | Write executed; audit entry written with `credential_ref = "[REDACTED]"` (BC-2.05.003); `capability_checks` and `result_summary` present |
| Query + IOC UDF + multi-sensor scope | `query` with `ioc_match('blacklist-ips', src_ip)` across 2 sensors | Federated subqueries fan out to both sensors (BC-2.11.001) | IOC UDF evaluated per-row against `PatternStore["blacklist-ips"]` | Rows filtered to IOC matches; OCSF-normalized output with `_sensor`, `_client`, `_source_table` virtuals |
| Spec hot-reload during in-flight query | `reload_config` invoked while query is materializing from `sentinelone.alerts` | Active query completes against pre-reload `ConfigSnapshot` (BC-2.16.007 arc-swap) | Next query issued after reload | New query uses updated spec; no half-reload state visible |
| Detection → case auto-creation | CRITICAL-severity detection rule fires on alert | Alert persisted (BC-2.13.005); auto-case-creation triggered (BC-2.14.013) | Case created in initial state | New case in `New` state linked to alert via `source_alert_ids`; case state machine ready for transitions (BC-2.14.002) |
| Token cap + audit | 101st confirm token request from same analyst; 100 active tokens already in store | `E-FLAG-007` rejection; no token created; no audit entry for failed generation | — | Existing 100 tokens unaffected; error surfaced to agent |
| Audit forwarding failure during active query session | `vector-prod` destination returns 503 | Audit entries written to `audit_buffer` normally (decoupled); forwarding retries in background with backoff (BC-2.05.011) | Query tools continue to function | Audit entries preserved; forwarding retries until recovery; no query disruption |

---

## Golden File References

| Vector Set | File | Format | BC Coverage |
|-----------|------|--------|-------------|
| Sample sensor spec (CrowdStrike) | `test-data/sensors/crowdstrike-prod.sensor.toml` | TOML | BC-2.16.001, BC-2.16.007 |
| Sample IOC file (valid, 3 patterns) | `test-data/ioc/blacklist-ips.ioc` | line-delimited text | BC-2.13.014 (happy-path) |
| Sample IOC file (at cap boundary, 100K patterns) | `test-data/ioc/boundary-100k.ioc` | line-delimited text | BC-2.13.014 (EC-13-038) |
| Sample IOC file (over cap, 100001 patterns) | `test-data/ioc/over-cap-100001.ioc` | line-delimited text | BC-2.13.014 (EC-13-039) |
| Sample audit entry (credential redacted) | `test-data/audit/sample-entry-redacted.json` | JSON | BC-2.05.003 |
| Sample MCP JSON-RPC request | `test-data/mcp/tools-list-request.json` | JSON | BC-2.10.006 |
| Sample confirmation token response | `test-data/flags/sample-token-response.json` | JSON | BC-2.04.009 |
| Sample case state transition request | `test-data/cases/state-transition-request.json` | JSON | BC-2.14.002 |

> Status: referenced files are test-data scaffolding for Phase 3 test-writer agents.
> Files to be committed to `test-data/` as stories consuming these vectors are implemented.

---

## Traceability Matrix (supplement-wide)

| Vector Set | Subsystem | Anchor BC(s) | Anchor DIs / Invariants | VPs Consuming |
|-----------|-----------|--------------|------------------------|---------------|
| TV-001 | SS-05 | BC-2.05.003 | DI-002 | integration only |
| TV-002 | SS-04 | BC-2.04.009 | DI-007, DI-015 | VP-010 |
| TV-003 | SS-11 | BC-2.11.001 | DI-004, DI-008, DI-019 | VP-014 |
| TV-004 | SS-11 | BC-2.11.012 | DI-020 | VP-015 |
| TV-005 | SS-13 | BC-2.13.014 | DI-019, DI-024, INV-IOC-001/002/003/004 | VP-018 |
| TV-006 | SS-14 | BC-2.14.002 | DI-004, DI-025 | VP-005, VP-006 |
| TV-007 | SS-04 | BC-2.04.005 | DI-003 | VP-003 |
| TV-008 | SS-10 | BC-2.10.006 | DI-017 (via BC-2.15.001) | integration only |
| TV-009 | SS-05 | BC-2.05.011 | DI-026, INV-AUDIT-FWD-001/002/003/004 | VP-039 |
| TV-010 | SS-16 | BC-2.16.001, BC-2.16.007 | DI-008, DI-030, DI-031 | VP-023 |

---

## Change Log

- v2.0 (2026-04-19): Full structural rewrite to conform to `prd-supplement-test-vectors-template.md`. Changes from v1.0:
  - **Structure:** Narrative block format replaced with per-subsystem `### Subsystem: SS-NN` grouping and `| Input | Expected Output | Category | Notes |` tables throughout. Cross-Subsystem Integration Vectors and Golden File References sections added.
  - **Frontmatter:** Aligned to official schema (`document_type: prd-supplement-test-vectors`, `level`, `producer`, `timestamp`, `phase`, `inputs`, `traces_to`). Removed extraneous fields: `supplement_type`, `parent_prd`, `created`, `created_by`, `supersedes`.
  - **TV-002 (BC-2.04.009) CRIT fix (P3P27-A-C-002):** Token TTL corrected 15m → 300s (5 minutes) per BC-2.04.009 `expires_at: created_at + 300s`. Token ID corrected UUID-v4 → `cryptographic random string` per BC-2.04.009 postconditions. Cap error corrected from the removed CONFIRM namespace → `E-FLAG-007` per error-taxonomy.md line 270 (CONFIRM namespace fully removed; FLAG namespace owns all token errors).
  - **TV-006 (BC-2.14.002) CRIT fix (P3P27-A-C-001):** Case states corrected from `New/In_Progress/Contained/Resolved/False_Positive` to canonical `New/Acknowledged/Investigating/Resolved/Closed` per BC-2.14.002 lines 26, 32-48. Error code for invalid transitions corrected `E-CASE-003` → `E-CASE-004`. `FalsePositive` correctly identified as a DispositionCode, not a state. Full 12-transition table derived from BC body.
  - **TV-010 (BC-2.16.001/007) HIGH fix (P3P27-A-H-002):** Traceability row now correctly cites both `BC-2.16.001` and `BC-2.16.007` as anchor BCs, and includes `DI-031` (formerly mis-attributed to BC-2.16.001 alone). DI-031 is enforced by BC-2.16.007 (atomic rollback on invalid reload).
  - **TV-007 (BC-2.04.005):** Corrected tool visibility rule — tools absent for ALL clients are hidden; per-client gating enforced at invocation time (not list time). Added `E-FLAG-001` and `E-FLAG-006` error vectors per BC postconditions.
  - **TV-009 (BC-2.05.011):** Added INV-AUDIT-FWD-004 (no silent loss) coverage. Added restart-mid-forward scenario (EC-05-021).
  - **TV-005 (BC-2.13.014):** Added empty-file (EC-13-044), hot-reload-in-flight (EC-13-040), and backtracking-regex (EC-13-043) edge cases. Preserved canonical limits (100K/10MB/50 files) from v1.0.
- v2.1 (2026-04-19): Pass-28 M-002 fix — removed VP-034 mis-citation from TV-001. VP-034 verifies AES-GCM encrypt-round-trip on `prism-credentials` (SS-06), not audit redaction (SS-05). BC-2.05.003 has no VP anchor; integration tests verify postconditions. Traceability matrix row for TV-001 now reads `integration only` matching the TV-008/BC-2.10.006 precedent.
- v2.2 (2026-04-19): Burst 34 pass-33 M-001 fix — removed 5 stale `execute_action` references (lines 46/47/48/75/266). Replaced with appropriate canonical tool names per BC-2.05.003 (audit redaction), BC-2.04.005 (hidden tools), and architecture/api-surface.md line 144 (crowdstrike_contain_host). `execute_action` was an obsolete name that S-5.06 used in early drafts; canonical name `fire_action` was applied to S-5.06 in Burst 33. Lines 46-48 (TV-001, EC-05-004, EC-05-005): replaced with `fire_action` — the direct canonical rename of `execute_action`, illustrating audit redaction behavior for a write tool with credential-like parameter names. Line 75 (TV-007 hidden-tools absent list): replaced with `fire_action` — a canonical write tool gated by `action.write` per api-surface.md:160. Line 266 (Cross-Subsystem Integration): replaced with `crowdstrike_contain_host` — the canonical irreversible sensor write tool per api-surface.md:144, matching the scenario intent of "contain_host, irreversible=true".
- v1.0 (2026-04-19, superseded): Initial catalog — 10 narrative-block vectors across 8 subsystems. Superseded by v2.0 structural rewrite.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.6 | pass-80-remediation | 2026-04-21 | product-owner | F80-003: corrected subsystem header CAP triples — SS-05 CAP-024→CAP-025, SS-04 removed CAP-014, SS-14 removed CAP-021. Added preamble paragraph for Per-Subsystem Test Vectors scope. |
| 2.5 | pass-72-fix | 2026-04-20 | product-owner | Renamed changelog header Notes → Change to match canonical 5-col supplement schema (HIGH-001). |
| 2.4 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added inputs/input-hash/traces_to frontmatter (already present); added Changelog section. |
