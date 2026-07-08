---
document_type: code-review
level: ops
version: "1.0"
status: draft
producer: code-reviewer
timestamp: 2026-07-08T00:00:00
phase: 5
inputs:
  - "feature/S-PRISMQL-CASE-INSENSITIVE-001 @ a71b8912"
  - "develop @ ea714d14"
input-hash: "diff ea714d14...a71b8912"
traces_to: "BC-2.11.024, BC-2.02.013"
pass: 1
previous_review: null
---

# Code Review: S-PRISMQL-CASE-INSENSITIVE-001 PR #217 (Pass 1)

Diff range: `git diff ea714d14...a71b8912`
Review scope: IEQ/IIN/INE case-insensitive PrismQL operators, OCSF enum-label
normalization at adapter boundary, E-QUERY-002 `suggested_column` enrichment,
and all related test infrastructure.

**Known-good context:** LOCAL adversarial cascade converged 3-CLEAN over 35 passes;
74 Red Gate tests GREEN; `just check` 5310/5310 GREEN. This review does NOT re-litigate
findings already resolved by the adversarial cascade.

**Persisted claims under adjudication (Standing Rule 3 §1):**
Three prior-session findings (CR-001..CR-003) are adjudicated below with CONFIRMED /
REFUTED verdicts backed by independent evidence.

---

## Part B — Findings

### CR-001: `segments.last()` pre-flight CI type check (persisted claim adjudication)

- **Severity:** LOW
- **Category:** spec-fidelity
- **Verdict:** REFUTED
- **Location:** `crates/prism-query/src/materialization.rs` — `collect_ci_fields_inner`
- **BC Reference:** BC-2.11.024
- **Description:** The persisted claim asserted that using `segments.last()` on a dotted
  `FieldPath` (e.g., `a.b.c`) in `collect_ci_fields_inner` would cause the CI type
  pre-flight check to miss the column and silently fall through to E-QUERY-034 instead
  of E-QUERY-002. The proposed fix was special-casing multi-segment paths.
- **Evidence:** The claim is REFUTED. Arrow schemas store columns under flat (non-dotted)
  names. The PrismQL parser produces `FieldPath { segments: ["severity"] }` for a
  column reference — a single segment. Multi-segment field paths are not a valid
  PrismQL construct targeting Arrow column names; dotted access in PQL resolves to
  column names at the sensor spec level, which are always flat. The existing
  non-CI type checker `resolve_col_type` in `materialization.rs` uses the same
  `fp.segments.last()` pattern to look up column names against the Arrow schema,
  and that function is correct by 35 adversarial passes and 74 RED GATE tests.
  The `field_with_name(col_name)` call in `check_ci_column_types` uses the same
  flat-name approach. There is no scenario in which a multi-segment FieldPath from
  the parser reaches `check_ci_column_types` with a column name that differs from
  `fp.segments.last()`.
- **Proposed Fix:** No fix required. The implementation is correct as written.

---

### CR-002: Unconditional `to_owned()` allocation in `OcsfNormalizer::normalize_with_mappers` (persisted claim adjudication)

- **Severity:** LOW
- **Category:** performance
- **Verdict:** CONFIRMED (with file-path correction)
- **Location:** `crates/prism-ocsf/src/normalizer.rs:162`
- **BC Reference:** BC-2.02.013
- **Description:** The persisted claim cited the wrong file path
  (`crates/prism-sensors/src/normalizer.rs:158` — this file does not exist in the
  workspace). The actual location is `crates/prism-ocsf/src/normalizer.rs:162`.
  The behavior is confirmed: when `normalize_enum_label` returns `Some(canonical)`,
  `msg.set_field_by_name(field, ProtoValue::String(canonical.to_owned()))` is called
  unconditionally — including when `current == canonical` (i.e., the value is already
  canonical). The code comment "Idempotent: if already canonical, this is a no-op
  rewrite (BC-2.02.013 RG-020)" correctly describes behavioral idempotency but
  conceals the allocation: `canonical.to_owned()` always allocates a new `String`
  and `set_field_by_name` always rewrites the field even when no change is needed.
- **Evidence:**
  ```rust
  // normalizer.rs:160-162
  if let Some(canonical) = map.normalize_enum_label(field, &current) {
      // Idempotent: if already canonical, this is a no-op rewrite (BC-2.02.013 RG-020).
      msg.set_field_by_name(field, ProtoValue::String(canonical.to_owned()));
  ```
  A sensor that already emits `"High"` for `severity` will trigger an unnecessary
  `to_owned()` + `set_field_by_name` call on every record, even though the value
  is already canonical. On high-volume sensor data this is an O(fields_per_record)
  unnecessary allocation.
- **Proposed Fix:** Add an equality guard before the rewrite:
  ```rust
  if let Some(canonical) = map.normalize_enum_label(field, &current) {
      if canonical != current.as_str() {
          msg.set_field_by_name(field, ProtoValue::String(canonical.to_owned()));
      }
  }
  ```
  Note: this is the SECONDARY (DynamicMessage/protobuf) emission path per BC-2.16.002
  catalog row 91 architectural adjudication 2026-07-07. The PRIMARY production path
  is `build_column_array` in `spec_driven_adapter.rs`, which already has the correct
  `normalize_enum_label` return-None-when-canonical semantics because it only writes
  when `normalize_enum_label` returns `Some`. No analogous fix is needed at the
  PRIMARY site.

---

### CR-003: `normalize_predicate` infallible — release build falls through to "IEQ" placeholder (persisted claim adjudication)

- **Severity:** LOW
- **Category:** code-quality
- **Verdict:** CONFIRMED
- **Location:** `crates/prism-query/src/ast.rs` — `normalize_predicate` / `PqlNormalizer`
- **BC Reference:** BC-2.11.024
- **Description:** For `Predicate::Compare { case_insensitive: true, op: <non-Eq/Ne>, .. }`,
  the `normalize_predicate` function uses `debug_assert!(false, ...)` in the fallback
  arm. In release builds, the `debug_assert!` is a no-op and execution falls through to
  emit the string `"IEQ"` — a syntactically valid PQL keyword but an incorrect
  representation for the actual operator (e.g., `>` with `case_insensitive=true` would
  produce `"fieldname IEQ 'val'"` rather than an error). This is a hand-built predicate
  invariant violation that the parser cannot produce, but the diagnostic fidelity in
  release builds is low.
- **Evidence:**
  ```rust
  // ast.rs:2077-2090
  _ => {
      // AST invariant: the parser only produces case_insensitive=true for
      // Eq/Ne. A hand-built predicate with another op + ci=true violates
      // BC-2.11.024. Panic in debug builds to catch it early; fall back
      // gracefully in release so a hand-built predicate doesn't cause a DoS.
      debug_assert!(
          false,
          "case_insensitive=true is only valid for Eq/Ne; got {op:?} — \
           manually-constructed predicate violates BC-2.11.024 invariant"
      );
      // Fallback: emit "IEQ" (canonical placeholder) so the normalized
      // string is at least syntactically valid PQL.
      "IEQ"
  }
  ```
  The "gracefully degrade in release" rationale is documented in the comment. The concern
  is that if a future code path hand-builds such a predicate and calls `normalize_predicate`
  in a context where the output matters (e.g., EXPLAIN display), the "IEQ" placeholder
  would produce misleading diagnostic output without any log or error.
- **Proposed Fix:** The full fix would change `normalize_predicate`'s return type to
  `Result<String, PrismError>` so the invalid case can be surfaced at the call site.
  This is a non-trivial refactor because `normalize_predicate` is called from string
  interpolation contexts (e.g., `format!("WHERE {}", Self::normalize_predicate(pred))`).
  A narrower fix: add a `tracing::warn!` emission before the placeholder fallback so
  the invalid predicate is at least observable in production logs (not registered in
  BC-2.16.002 catalog since it's a programming-error diagnostic, not a domain event).
  The current comment "gracefully fallback in release so a hand-built predicate doesn't
  cause a DoS" is correct; the only gap is observability in production.

  **NOTE: IIN pinned error message is UNTOUCHABLE.** Any work touching the SQL-mode
  rejection error message path in `parse_sql_with_limits` MUST preserve the byte-exact
  string pinned at `test_case_insensitive_operators.rs` (~line 1555):
  ```
  "E-QUERY-001: parse error near 'IIN': case-insensitive operators (IEQ/IIN/INE) are
  not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe |
  where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."
  ```
  This is a byte-exact `assert_eq!` pinned by RG tests and must not be modified.

---

### CR-004: SEC-001 anchor verification — CWE-117 control-char gap confirmed at both emission sites

- **Severity:** LOW
- **Category:** code-quality
- **Verdict:** SEC-001 CONFIRMED (security finding, fix already planned)
- **Location:**
  - PRIMARY: `crates/prism-bin/src/spec_driven_adapter.rs:1109-1110`
  - SECONDARY: `crates/prism-ocsf/src/normalizer.rs:170`
- **BC Reference:** BC-2.02.013, BC-2.16.002 catalog row 91
- **Description:** Both `ocsf.enum_label_unrecognized` warn emission sites apply a
  50-codepoint truncation cap (SEC-002 / CWE-532 defense) but do NOT strip Unicode
  control characters or ANSI escape sequences before emitting the `value` and
  `sensor_type` fields into the structured log. A sensor that returns a field value
  containing `\n`, `\r`, or `\x1b[...` escape sequences could inject multi-line log
  entries or terminal control sequences (CWE-117: Improper Output Neutralization for
  Logs). This is a distinct gap from the over-length protection that SEC-002 provides.
  The fix is understood and already planned; this review verifies the anchor locations.
- **Evidence:**
  ```rust
  // spec_driven_adapter.rs:1107-1110
  event_type = "ocsf.enum_label_unrecognized",
  field_name = %col.name,
  value = %s.chars().take(50).collect::<String>(),       // truncation only, no ctrl-char filter
  sensor_type = %sensor_id.chars().take(50).collect::<String>(),  // same gap
  ```
  ```rust
  // normalizer.rs:168-172
  event_type = "ocsf.enum_label_unrecognized",
  field_name = %field,
  value = %current.chars().take(50).collect::<String>(),           // truncation only
  sensor_type = %sensor.chars().take(50).collect::<String>(),      // same gap
  ```
- **Proposed Fix:** Apply a control-character filter before truncation at both sites.
  A minimal approach: replace control characters (Unicode category Cc) with their
  escaped form or a placeholder. Example helper:
  ```rust
  fn sanitize_log_value(s: &str) -> String {
      s.chars()
          .take(50)
          .map(|c| if c.is_control() { '?' } else { c })
          .collect()
  }
  ```
  Replace `.chars().take(50).collect::<String>()` with `sanitize_log_value(&s)` at
  both PRIMARY and SECONDARY sites. The BC-2.16.002 catalog row 91 field descriptions
  should be amended to note control-char sanitization alongside the existing 50-codepoint
  cap note (per SAP-1 discipline, same commit as the code change).

---

### CR-005: `build_example_note` function name partially misleading

- **Severity:** LOW
- **Category:** maintainability
- **Location:** `crates/prism-mcp/src/tools/prism_describe.rs` — `build_example_note` (line ~620)
- **BC Reference:** BC-2.10.012
- **Description:** `build_example_note` is the authoritative implementation returning
  `(String, Option<String>)` — i.e., `(example_query, optional_note)`. The name suggests
  it builds a note, but it also builds the primary example query. The thin wrapper
  `build_example_query` delegates to `build_example_note(..).0` and exists for callers
  that only need the query string. The function comment (line ~615) reads "This is the
  authoritative implementation; `build_example_query` is a thin wrapper" which partially
  mitigates the confusion. However, a future reader adding a third call site must read
  the wrapper to understand the API shape — the name alone is insufficient.
- **Evidence:**
  ```rust
  // prism_describe.rs:602-604
  pub fn build_example_query(table_name: &str, columns: &[ColumnDescriptor]) -> String {
      build_example_note(table_name, columns).0
  }

  // prism_describe.rs:615-620
  /// This is the authoritative implementation; `build_example_query` is a thin wrapper
  pub fn build_example_note(
  ```
  The pair `build_example_query` + `build_example_note` follows the pattern where the
  "note" function returns a richer result. This is a naming inversion — the function
  that returns MORE is named with the narrower term.
- **Proposed Fix:** Rename to better reflect the return shape. Two options:
  - Option A (minimal): rename `build_example_note` to `build_example_with_note` and
    keep `build_example_query` as the simpler name — callers getting `(query, note)`
    call `build_example_with_note`; callers getting only `query` call `build_example_query`.
  - Option B (idiomatic): eliminate `build_example_query` wrapper and have all callers
    use `.0` destructuring inline, with a single `build_example_note` that documents
    its return tuple clearly.
  Given the 35-pass adversarial clean and 5310/5310 GREEN build, a rename is low-risk
  but requires a sibling-sweep (TD-VSDD-060) of all `build_example_query` and
  `build_example_note` callsites.

---

## Summary

| ID | Severity | Category | Verdict | Location |
|----|----------|----------|---------|----------|
| CR-001 | LOW | spec-fidelity | **REFUTED** — `segments.last()` is correct | `materialization.rs` / `collect_ci_fields_inner` |
| CR-002 | LOW | performance | **CONFIRMED** (wrong file in claim; actual: `prism-ocsf/src/normalizer.rs:162`) | Unconditional `to_owned()` when `current == canonical` |
| CR-003 | LOW | code-quality | **CONFIRMED** — release build falls to "IEQ" placeholder silently | `ast.rs` / `normalize_predicate` |
| CR-004 | LOW | code-quality | **SEC-001 CONFIRMED** — CWE-117 gap at both emission sites; fix already planned | `spec_driven_adapter.rs:1109-1110`, `normalizer.rs:170` |
| CR-005 | LOW | maintainability | NEW | `prism_describe.rs` / `build_example_note` name |

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 5 |

## Convergence Verdict

All five findings are LOW severity. Zero CRITICAL/HIGH/MEDIUM findings. The implementation
is correct: `segments.last()` (CR-001 REFUTED), SAP-1 `ocsf.enum_label_unrecognized`
catalog compliance verified (both emission sites, BC-2.16.002 row 91), `#[non_exhaustive]`
discipline intact (no new pub types in gated crates without annotation; `PrismError` already
`#[non_exhaustive]` and `QueryTypeMismatch` callers use `..`), no `unwrap()`/`expect()` on
critical paths, no `reqwest` dep changes, and the pinned IIN error message is untouched.

The three confirmed LOW findings (CR-002 redundant allocation, CR-003 silent placeholder
in release, CR-004 planned security fix) are known, scoped, and do not block merge under
CLEAN(PR-merge) criteria (zero CRIT/HIGH/MED). CR-005 is a naming suggestion.

`CONVERGENCE_REACHED` — no CRITICAL or HIGH findings; all LOW findings are scoped and
do not affect correctness or BC contracts.
