---
review_type: pr-security-review
pr: 129
story: S-3.02
branch: feature/S-3.02
worktree: .worktrees/S-3.02
head_commit: c01d57f4
diff_base: 2a7b83f5
rebase_context: post-rebase onto develop @ 2a7b83f5 (post-S-3.06-merge)
reviewer: security-reviewer (claude-sonnet-4-6)
review_date: 2026-05-06
total_findings: 5
critical: 0
high: 1
medium: 2
low: 2
files_reviewed: 12
verdict: NEEDS-CHANGES
---

# Security Review — PR #129 (S-3.02 Query Materialization)
## Post-Rebase Fresh-Context Review

**Worktree:** `.worktrees/S-3.02` / branch `feature/S-3.02` / PR #129 / post-rebase HEAD `c01d57f4`

---

## Executive Summary

PR #129 delivers the structural scaffolding and security-critical helper code for the
S-3.02 ephemeral materialization pipeline. The majority of execution logic
(`run_materialization_pipeline`, `resolve_source_refs`, `QueryEngine::execute`,
`RocksDbTableProvider::scan`, `register_internal_tables`) remains as `todo!()` stubs;
this review focuses on the non-stub implemented code, the inherited S-3.06 integration,
and design-time security posture.

**Prior fixes verified as landed and correct:** F-PR129-CR-001 through CR-009, SEC-001
through SEC-003. The scopeguard removal (CR-002), memory error accuracy (CR-004),
DataFusion error redaction (SEC-003), and `pub(crate)` counter enforcement (CR-003) are
all confirmed present in correct form.

**One HIGH finding** (SEC-P129-004) is a pre-existing residual information disclosure in
`map_datafusion_memory_error` that was NOT addressed by F-PR129-SEC-003. That fix only
covered `collect_record_batch_stream`; the non-`ResourcesExhausted` branch of
`map_datafusion_memory_error` still forwards raw DataFusion error text.

**Two MEDIUM findings** are design-time observations with risk window limited to the stub
phase. **Two LOW findings** are quality and defense-in-depth notes.

---

## Files Reviewed

1. `crates/prism-query/src/engine.rs`
2. `crates/prism-query/src/materialization.rs`
3. `crates/prism-query/src/memory.rs`
4. `crates/prism-query/src/session.rs`
5. `crates/prism-query/src/scoping.rs`
6. `crates/prism-query/src/virtual_fields.rs`
7. `crates/prism-query/src/pushdown.rs`
8. `crates/prism-query/src/internal_tables.rs`
9. `crates/prism-query/src/write_ast.rs`
10. `crates/prism-query/src/write_verb_registry.rs`
11. `crates/prism-query/src/lib.rs`
12. `crates/prism-query/src/visit.rs`
13. `crates/prism-query/src/security.rs` (inherited, read for context)
14. `crates/prism-query/src/filter_parser.rs` (inherited, denylist/write paths)
15. `crates/prism-query/src/sql_parser.rs` (inherited, DML guard)
16. `tests/external/perimeter-violation/src/main.rs`

---

## Checklist Disposition

### 1. Cross-tenant data leakage (BC-2.11.011) — PASS

`scoping.rs::resolve_clients` enforces intersection semantics correctly. When
`clients: None`, the tool-parameter scope expands to all registered clients.
`intersect_query_client_predicates` filters the tool scope down — it cannot expand it.
An out-of-scope client predicate is silently excluded, never added. No leakage path in
the implemented code. The fan-out stubs (`resolve_source_refs`, `run_materialization_pipeline`)
are `todo!()` — risk deferred to implementation phase.

### 2. Virtual field spoofing prevention (BC-2.11.012) — PASS

`virtual_fields.rs::inject_virtual_fields` calls `remove_spoofed_virtual_columns` first,
which rebuilds the schema excluding any column named `_sensor`, `_client`, or
`_source_table`. The canonical engine-injected values are appended after removal, making
overwrite impossible. The reserved names are compile-time constants (`VIRTUAL_FIELD_*`).
Comparison uses `f.name().as_str()` against `&[&str]`, which is exact string match.
F-PR129-SEC-002 (prior fix) confirmed working.

### 3. Memory budget DoS (BC-2.11.006 E-QUERY-004) — MOSTLY PASS, one residual (SEC-P129-004 HIGH)

`build_session_context` correctly creates a per-query `GreedyMemoryPool` scoped to the
single query execution. `map_datafusion_memory_error` maps `ResourcesExhausted` to the
structured `QueryMemoryBudgetExceeded` error with redacted detail. However: the
`_ => PrismError::QueryExecutionFailed { detail: err.to_string() }` fallback branch
forwards raw DataFusion error text — see SEC-P129-004.

### 4. Query timeout DoS (BC-2.11.006 E-QUERY-005) — DESIGN VERIFIED

`engine.rs::execute` documents the 30-second `tokio::time::timeout` wrap and returns
`PrismError::QueryTimeout`. The config type enforces `timeout_secs: 30` by default.
The actual `tokio::time::timeout` call site is in the `todo!()` stub body; correct by
spec intent.

### 5. Sensor partial-failure handling (BC-2.11.005) — DESIGN VERIFIED

`QueryResult` includes `sensor_errors: Vec<String>` for partial sensor failures. The
`resolve_source_refs` stub comment correctly specifies that missing `(source, client)`
pairs are "silently skipped" per BC-2.11.011. Full implementation deferred.

### 6. Session scope drop on panic (BC-2.11.005 AC-7) — PASS

`session.rs::SessionScope` uses plain `Drop` with `self.inner.take()`. Rust guarantees
`Drop::drop` runs on panic unwind unless the runtime calls `abort()`. The
`Option::take()` in `Drop::drop` makes double-drop impossible. The two `panic!()` calls
in `context()` and `into_arc()` are programmer error guards (use-after-move), not
reachable from external callers. No scopeguard dependency detected (confirmed absent from
`Cargo.toml`). F-PR129-CR-002 correctly resolved.

### 7. Internal table SQL injection — DESIGN SAFE (stub phase)

`register_mem_table` uses `ctx.register_table(table_name, ...)` where `table_name` is
the source ref string from the AST. DataFusion's `register_table` does not execute SQL —
it registers a logical name. No SQL passthrough path exists in the reviewed code. The
write-mode DML guard (`is_internal_prism_table`) correctly uses
`table_name.to_ascii_lowercase().starts_with("prism_")` — case-insensitive, covering all
Unicode-free case variants. The internal table guard is applied to all three DML
operations (INSERT at line 1003, UPDATE at 1097, DELETE at 1169 of sql_parser.rs).

### 8. REQUIRED column bypass (BC-2.11.007) — PASS

`pushdown.rs::column_push_down_option_from_spec` reads `col.options.contains(&ColumnOptions::Required)`
where `ColumnOptions` is a typed enum from `prism_core`. There is no hardcoded string
comparison. An attacker controlling sensor spec content would need to avoid declaring
`ColumnOptions::Required` in the spec to suppress push-down, but the engine falls back to
`ColumnPushDownOption::Default` (post-filter), not to skipping the required parameter
check. The REQUIRED invariant is formally verified by VP-031.

### 9. Workspace ripple risk (arrow 53→58) — PASS

Arrow 58.2.0 is in `Cargo.lock`. No published RUSTSEC advisories for Arrow 58.x were
found via advisory DB review. The `deny.toml` policy sets `ignore = []` (no suppressed
advisories) and `yanked = "deny"`. Arrow 58 is the version DataFusion 53.1 requires
transitively; the `prism-sensors/Cargo.toml` update to arrow 58 aligns the workspace,
preventing dual-version conflicts that could cause IPC/schema mismatch bugs.

### 10. Hardcoded secrets — PASS

No credentials, API keys, tokens, or secrets found in any reviewed S-3.02 file.
`QueryEngine` holds `Arc<dyn CredentialStore>` — the AI-opaque credential boundary is
maintained.

### 11. Dependency CVEs — PASS

DataFusion 53.1.0, Arrow 58.2.0, tokio (workspace pinned), async-trait — no active
RUSTSEC advisories observed. `scopeguard` was removed in F-PR129-CR-002 and is absent
from `Cargo.toml`. `deny.toml` has zero ignored advisories.

### 12. Cross-PR contamination check — PASS

S-3.06's parser code inherited via rebase is verified secure:

- `parse_with_registry` calls `check_denied_keywords` before pipe/filter routing
  (line 156 of filter_parser.rs) — denylist is propagated to both public entry points.
- Perimeter-violation crate (`tests/external/perimeter-violation/src/main.rs`) includes
  all 10 new S-3.06 restricted symbols (lines 122–159): `parse_pipe_with_write`,
  `build_write_stage_parser`, `build_write_arg_parser`, `extract_sensor_prefix`,
  `parse_sql_dml`, `parse_sql_dml_with_limits`, `build_dml_parser`,
  `is_internal_prism_table`, `check_unbounded_write`, `reject_write_verbs_in_filter`.
  All are `pub(crate)`.
- `parse_sql_dml_with_limits` is `pub(crate)` (line 864 of sql_parser.rs) — correctly
  listed in perimeter-violation crate.
- Visitor traverses both `DmlNode` (walk_sql_statement line 151) and `WriteNode`
  (walk_pipe_query line 208). Both `walk_dml_node` and `walk_write_node` are implemented.
- `is_internal_prism_table` uses `to_ascii_lowercase().starts_with("prism_")` — case
  insensitive. Applied to INSERT (line 1003), UPDATE (line 1097), DELETE (line 1169).
- No security regression introduced by S-3.06 merge into this PR.

### 13. Error message safety post-SEC-003 — PARTIAL PASS (see SEC-P129-004)

`collect_record_batch_stream` (materialization.rs lines 335–349): correctly redacts
DataFusion error — raw error logged server-side, `<redacted; see server logs>` returned
to client. SEC-003 fix verified present.

`map_datafusion_memory_error` (memory.rs lines 88–102): `ResourcesExhausted` branch
is correctly handled with structured `QueryMemoryBudgetExceeded`. The fallback `_ =>`
branch uses `detail: err.to_string()`, which forwards raw DataFusion error text. This
is the residual finding (SEC-P129-004, HIGH).

---

## Findings

### SEC-P129-001: Audit Table `prism_audit` Registered Before Capability Check

- **Severity:** MEDIUM
- **CWE:** CWE-862 (Missing Authorization)
- **OWASP:** A01:2021 — Broken Access Control
- **Attack Vector:** A query that accesses `prism_audit` during the `scan()` call.
  If the capability gate fires only at scan time (deferred from registration), an
  adversary can confirm the table exists via schema introspection
  (`DESCRIBE TABLE prism_audit` or `SELECT *` failing with capability error rather than
  table-not-found error) — leaking table existence even without `audit.read`.
- **Impact:** Information disclosure about internal table presence; potential
  side-channel enumeration of audit log existence. Not a data-access bypass, but
  violates least-privilege for table enumeration.
- **Evidence:** `internal_tables.rs` lines 161–165:
  ```
  // `prism_audit` registration is deferred — the capability check occurs at
  // `scan()` time, not registration time. This allows the table to appear in
  // schema introspection regardless of capability.
  ```
  The doc comment explicitly acknowledges this behavior as intentional.
- **Proposed Mitigation:** At registration time, the table name should be visible in
  schema introspection only when the calling session has `audit.read`. Two options:
  (1) Register `prism_audit` conditionally — check `audit.read` capability before
  registering; or (2) Accept the current design with a formal security note documenting
  that table-existence leakage via introspection is an accepted risk (audit log existence
  is not sensitive in MSSP context where all analysts know the system). If option 2,
  add an explicit comment to that effect for the next reviewer.
- **Status:** This is a stub (`todo!()`) — the decision must be made before
  `register_internal_tables` is implemented.

---

### SEC-P129-002: `translate_push_down_filter` Emits Debug Representation of AST Expr

- **Severity:** MEDIUM
- **CWE:** CWE-209 (Information Exposure Through an Error Message)
- **OWASP:** A04:2021 — Insecure Design
- **Attack Vector:** The stub `translate_push_down_filter` (pushdown.rs line 195) uses
  `format!("{}={:?}", predicate.column_name, predicate.expr)` to produce filter strings.
  `{:?}` on an `Expr` emits the full Rust debug representation, including internal enum
  variant names and nesting structure. If this stub output is ever forwarded to a sensor
  adapter or logged without sanitization, it could expose AST internals to sensor API
  servers or external log consumers.
- **Impact:** Internal Rust type information (`Expr::Compare { lhs: Field(FieldPath { ... }) }`)
  could appear in sensor API request bodies or operator logs, leaking implementation
  details and aiding attackers in crafting malformed queries.
- **Evidence:** `pushdown.rs` lines 189–196:
  ```rust
  pub(crate) fn translate_push_down_filter(
      predicate: &Predicate,
      _columns: &[ColumnSpec],
  ) -> Option<String> {
      Some(format!("{}={:?}", predicate.column_name, predicate.expr))
  }
  ```
  This is marked as a stub; the doc comment says "The stub implementation emits a generic
  `column=value` string; full sensor-native translations will be added per sensor story."
- **Proposed Mitigation:** Replace the `{:?}` debug format with a safe display formatter
  that only emits the literal value portion of the expression (not the enum variant
  structure). Before this function leaves stub phase, add a compiler assertion that the
  output never contains Rust enum variant names. In the interim, ensure call sites never
  log or forward the return value to external systems.
- **Status:** Stub phase — risk is dormant. Must be addressed before sensor stories
  implement `fan_out()`.

---

### SEC-P129-003: `build_session_context` Forwards Runtime Env Error with Internal Detail

- **Severity:** LOW
- **CWE:** CWE-209 (Information Exposure Through an Error Message)
- **OWASP:** A04:2021 — Insecure Design
- **Attack Vector:** If `RuntimeEnvBuilder::build()` fails (system resource exhaustion,
  invalid pool configuration), the raw DataFusion error message is forwarded in the
  `PrismError::QueryExecutionFailed.detail` field: `format!("failed to build DataFusion
  runtime env: {e}")` (memory.rs line 59).
- **Impact:** DataFusion runtime env build errors can include platform-specific resource
  information. However, this failure is catastrophic (server can't create sessions) and
  unlikely under normal operation. Risk is LOW.
- **Evidence:** `memory.rs` lines 55–64:
  ```rust
  let runtime_env = RuntimeEnvBuilder::new()
      .with_memory_pool(pool)
      .build()
      .map_err(|e| PrismError::QueryExecutionFailed {
          detail: format!("failed to build DataFusion runtime env: {e}"),
      })?;
  ```
- **Proposed Mitigation:** Redact to `"failed to initialize query runtime: <redacted; see
  server logs>"` and log the raw error server-side. Pattern is already established by
  `collect_record_batch_stream`.

---

### SEC-P129-004: `map_datafusion_memory_error` Leaks Raw DataFusion Error Text

- **Severity:** HIGH
- **CWE:** CWE-209 (Information Exposure Through an Error Message)
- **OWASP:** A04:2021 — Insecure Design
- **Attack Vector:** Any DataFusion error that is NOT `ResourcesExhausted` (e.g.,
  schema mismatch, plan compilation failure, type coercion error) passes through
  `map_datafusion_memory_error`'s fallback branch as a raw `err.to_string()`.
  DataFusion error messages routinely include table names, column names, schema field
  names, and type information. When this `PrismError::QueryExecutionFailed` is
  serialized into the MCP `query` tool response, the raw DataFusion internals become
  visible to the MCP client (the LLM agent or analyst).
- **Impact:** Schema enumeration without authorization. An attacker-controlled query can
  trigger type-coercion errors (e.g., by comparing a string column to an integer literal)
  and receive DataFusion's error message containing column names and types that they should
  not know about. This is an indirect schema disclosure channel that complements direct
  SQL introspection. The prior fix (F-PR129-SEC-003 / commit `94b566cb`) addressed
  `collect_record_batch_stream` but left this sibling code path untouched.
- **Evidence:** `memory.rs` lines 95–101:
  ```rust
  _ => PrismError::QueryExecutionFailed {
      detail: err.to_string(),  // Raw DataFusion error forwarded to MCP client
  },
  ```
  Contrast with the correct pattern in `materialization.rs` lines 340–348:
  ```rust
  .map_err(|e| {
      tracing::error!(error = %e, "stream collection error (detail redacted from client response)");
      PrismError::QueryExecutionFailed {
          detail: "stream collection error: <redacted; see server logs>".to_string(),
      }
  })
  ```
- **Proposed Mitigation:** Apply the same log-and-redact pattern to `map_datafusion_memory_error`:
  ```rust
  _ => {
      tracing::error!(
          error = %err,
          "DataFusion error (detail redacted from client response)"
      );
      PrismError::QueryExecutionFailed {
          detail: "query execution error: <redacted; see server logs>".to_string(),
      }
  }
  ```
  This is a one-line fix to an already-implemented non-stub function and should be
  addressed before merging, as `map_datafusion_memory_error` is on the hot path for all
  query execution errors that callers map through this function.

---

### SEC-P129-005: `SessionScope::context()` Panic Reveals Internal Error Code in MCP Response

- **Severity:** LOW
- **CWE:** CWE-390 (Detection of Error Condition Without Action)
- **OWASP:** N/A (internal programmer error guard)
- **Attack Vector:** `session.rs` line 73: if `SessionScope::context()` is called after
  `into_arc()` has been called, it panics with message
  `"E-INT-001: SessionScope::context called after into_arc — context already moved out"`.
  If the Tokio runtime does not catch this panic at a task boundary (e.g., if the panic
  propagates up through async code without a `catch_unwind`), it could manifest as an
  unhandled panic message in MCP error responses or logs.
- **Impact:** LOW. This is a programmer error guard for `execute_scheduled` misuse, not
  a reachable attack path from external input. The error code `E-INT-001` is internal and
  non-sensitive. The panic is only reachable if the implementer of `execute_scheduled`
  violates the `into_arc` invariant, which is protected by the doc comment.
- **Evidence:** `session.rs` lines 68–77:
  ```rust
  pub fn context(&self) -> &SessionContext {
      match self.inner.as_ref() {
          Some(ctx) => ctx,
          None => panic!("E-INT-001: SessionScope::context called after into_arc ..."),
      }
  }
  ```
- **Proposed Mitigation:** Acceptable as-is for programmer-error guards. However, at
  `execute_scheduled` implementation time, wrap the call site with a `debug_assert!` or
  structural review to ensure `context()` is never called on a scope that had `into_arc`
  applied. No immediate fix required.

---

## Risk Register Dispositions

Security-category R-NNN entries from the L2 Domain Spec Risk Register:

| R-NNN | Risk | Disposition | Notes |
|-------|------|-------------|-------|
| R-005 | Prompt injection via sensor data | **partially-mitigated** | `VirtualField::SafetyFlags` exists in the AST (pushdown.rs `virtual_field_name` match). The `_safety_flags` virtual field is defined in spec. S-3.02's stub pipeline does not yet implement the regex-based suspicious pattern flagging described in the risk mitigation. The structural separation (data in structured fields) is maintained via Arrow RecordBatch/OCSF model. |
| R-006 | Credential exposure | **mitigated** | `QueryEngine` holds `Arc<dyn CredentialStore>` — credentials are AI-opaque. No credential values appear in any S-3.02 source file. Error messages are redacted (partially — see SEC-P129-004). The `CredentialStore` trait pattern prevents credential transit through query engine code. |
| R-012 | Confirmation token replay/forgery | **partially-mitigated** | S-3.02 adds write-mode AST types (`write_ast.rs`) and write-verb registry. The token confirmation system is a future story concern (S-3.07 dispatch). The `DmlNode` carries a `filter` field with the actual parsed predicate, enabling S-3.07 to enforce bounded-write semantics. Token generation/validation is not in scope for S-3.02. |

---

## Cross-PR Contamination Assessment

S-3.06 parser code inherited via rebase: **CLEAN — no regression**.

Specific verifications:
1. `parse_with_registry` applies `check_denied_keywords` before pipe/filter dispatch.
2. Perimeter-violation crate lists all 28 expected symbols (18 original + 10 S-3.06 additions).
3. `parse_sql_dml_with_limits` is `pub(crate)` at sql_parser.rs:864.
4. `walk_dml_node` and `walk_write_node` both implemented in `visit.rs`.
5. `is_internal_prism_table` uses case-insensitive `to_ascii_lowercase()` comparison.
6. Guard applied to all three DML operations (INSERT/UPDATE/DELETE).

---

## Verdict

**NEEDS-CHANGES**

One HIGH finding (SEC-P129-004) blocks merge per policy. The fix is a one-line change to
`memory.rs::map_datafusion_memory_error` fallback branch: log raw error server-side and
return redacted string to client. This is a non-stub already-implemented function on the
query hot path.

**Required before merge:**
- SEC-P129-004 (HIGH): Redact raw DataFusion errors in `map_datafusion_memory_error` fallback

**Recommended before merge (MEDIUM):**
- SEC-P129-001 (MEDIUM): Document the `prism_audit` registration-time enumeration design
  decision explicitly, or defer to registration-conditional capability check
- SEC-P129-002 (MEDIUM): Gate `translate_push_down_filter` debug format behind a stub
  assertion that prevents forwarding to external systems

**Acceptable for current stub phase (LOW):**
- SEC-P129-003 (LOW): Redact `build_session_context` failure detail (fix before GA)
- SEC-P129-005 (LOW): Programmer error guard; acceptable as designed

---

## Finding Count Summary

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 0 | — |
| HIGH | 1 | SEC-P129-004 |
| MEDIUM | 2 | SEC-P129-001, SEC-P129-002 |
| LOW | 2 | SEC-P129-003, SEC-P129-005 |
| **Total** | **5** | |
