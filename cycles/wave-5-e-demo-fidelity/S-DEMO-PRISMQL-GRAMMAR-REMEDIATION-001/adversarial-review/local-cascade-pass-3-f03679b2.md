---
document_type: adversarial-review-pass
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
cascade: LOCAL
pass: 3
frozen_head: f03679b2
diff_range: "903c8fcb..f03679b2"
reviewer_perimeter: "per-story (story diff 903c8fcb..f03679b2 + 8 BCs + ADR-043/044/045/046 + error-taxonomy)"
verdict_strict: "CLEAN(strict)=NO"
verdict_pr_merge: "CLEAN(PR-merge)=NO"
pass_outcome: "NOT CLEAN — 1 HIGH (SAP-1 catalog gap, CLOSED by PO BC-2.16.002 v1.89→v1.90) + 1 MED (POL-24 taxonomy drift E-QUERY-036, CLOSED by PO error-taxonomy v1.97→v1.98) + 1 LOW (OBS-1 code-side label fix, being CLOSED by implementer)"
post_pass_head: f03679b2
streak_after: "0/3 RESET — code HEAD will move with incoming implementer fix-burst"
timestamp: 2026-06-25T11:00:00Z
---

# LOCAL Adversary Pass 3 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Story:** S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
**Cascade:** LOCAL
**Pass:** 3 (of rolling cascade; per BC-5.39.001 3-CLEAN protocol)
**Frozen HEAD reviewed:** `f03679b2`
**Diff range:** `903c8fcb..f03679b2`
**Story version read:** v1.4 (BC version-pin corrections applied by story-writer at D-1339)
**Full workspace gate:** `just check` EXIT=0; 4915 tests GREEN

---

## Verdict

```
CLEAN(strict): NO
CLEAN(PR-merge): NO
```

**3-CLEAN streak:** RESET to 0/3. Code HEAD will advance with incoming implementer fix-burst
(OBS-1 code-side label `"Available sensors:" → "Available tables:"`). Re-gate on new
implementer HEAD required per frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Findings

### F-HIGH-1 — HIGH | SAP-1: `filter.sql_lowering` + `filter.sql_planning_error` missing BC-2.16.002 catalog rows

**Severity:** HIGH
**Rule:** SAP-1 (CLAUDE.md §Standing Adversary Probes — SAP-1 tracing emission catalog completeness)
**Status:** CLOSED by product-owner (BC-2.16.002 v1.89→v1.90; catalog v1.55→v1.56; count 86→88)

**Finding:** The Area D `Ast::Filter` arm introduced in the HIGH-1 fix-burst (commit range
`e518d96c..f03679b2`) added two new `tracing::*!(event_type=…)` emission sites in
`execute_against_session` (`crates/prism-query/src/materialization.rs`):

1. `filter.sql_lowering` DEBUG — emitted after the filter predicate is lowered to
   `SELECT * FROM <table> WHERE <predicate>` SQL, before `session_ctx.sql(&filter_sql)` is called.
2. `filter.sql_planning_error` ERROR — emitted in the `Err` arm of `session_ctx.sql(&filter_sql).await`
   when DataFusion SQL planning fails.

Neither emission appeared in the BC-2.16.002 Canonical Structured Event Catalog (then at v1.55,
count 86). Per SAP-1 and PG-LP11-001, absence of a catalog row for any `event_type =` site in
in-scope crates is a P1 finding.

**Closure:** Product-owner amended BC-2.16.002 v1.89→v1.90:
- Two new catalog rows added: `filter.sql_lowering` (DEBUG, `execute_against_session`
  `Ast::Filter` arm, fields: `filter_sql: %display`, audit role: NON-audit operational debug;
  recurrence: once per successful filter lowering; SECURITY: structural SQL only — no sensor
  response data or credentials) and `filter.sql_planning_error` (ERROR, `execute_against_session`
  `Ast::Filter` arm, fields: `error: %display` + `filter_sql: %display`, audit role:
  error/operational diagnostic; recurrence: once per DataFusion planning failure; CWE-209:
  client receives redacted message, server retains full error).
- Catalog count 86→88; catalog label `(v1.55)` → `(v1.56)`.
- Scope statement extended to include `prism-query` filter-mode SQL lowering and planning-error
  emissions from `execute_against_session`.
- Both rows modeled on sibling `pipe.sql_lowering` / `pipe.sql_planning_error` (ENRICH-4-B,
  catalog rows 178/179 per pass-1 verification).
- Frontmatter v1.89→v1.90; changelog row added.

**Residual code verification:** `rg 'filter\.sql_lowering\|filter\.sql_planning_error' crates/`
confirms both emission sites live in `execute_against_session` `Ast::Filter` arm
(`crates/prism-query/src/materialization.rs`). No other in-scope crates contain these event_type values.

---

### OBS-1 — MED | POL-24: E-QUERY-036 Display appended undocumented suffix `Available sensors:` label

**Severity:** MED (POL-24 taxonomy drift)
**Rule:** POL-24 — Display impl MUST match the error-taxonomy.md Message Format byte-for-byte
**Status:** CLOSED by product-owner (error-taxonomy v1.97→v1.98, Message Format amended to canonical
form). Code-side label fix `"Available sensors:" → "Available tables:"` being CLOSED by implementer
in parallel fix-burst (new code HEAD pending — state-manager will refresh on next burst).

**Finding:** `PrismError::UnknownSourceTable` Display impl (introduced by S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
AC-021 / QRY cascade P6-02 E-QUERY-036 variant work) appended an available-list + did_you_mean suffix
consistent with E-QUERY-037/038/039 parity, but:

1. The suffix was not documented in the error-taxonomy.md E-QUERY-036 Message Format field (prior
   format ended at `"…register the sensor in prism.toml."`).
2. The label used in the Display impl was `"Available sensors:"` but the field is named
   `available_tables` and sibling E-QUERY-037 uses `"Available tables:"` for the table-list label.

Both gaps constitute a POL-24 violation: the taxonomy row is the canonical form; code must match
byte-for-byte; neither undocumented suffixes nor wrong labels are acceptable.

**Closure (taxonomy side):** Product-owner amended error-taxonomy v1.97→v1.98:
- E-QUERY-036 Message Format updated to include `Available tables: [{available_tables}].{did_you_mean}` suffix.
- Full canonical format: `"E-QUERY-036: unknown source table '{source_name}': table is not a registered sensor
  or internal table. Check spelling or register the sensor in prism.toml. Available tables: [{available_tables}].{did_you_mean}"`
  where `{did_you_mean}` is `""` when no candidate or `" Did you mean: '{x}'?"` (leading space, one candidate).
- Label choice rationale documented: "Available tables:" (not "Available sensors:") — field is named
  `available_tables`, sibling E-QUERY-037 uses same label, covers both sensor tables and internal tables.
- Emitter updated: `PrismError::UnknownSourceTable` gains `available_tables: Vec<String>` and
  `did_you_mean: Option<String>` fields (AC-021 implementer obligation).
- POL-24 compliance note added: implementer Display impl MUST match canonical format byte-for-byte.
- Changelog row added (v1.97→v1.98).

**Closure (code side):** Being CLOSED by implementer — label `"Available sensors:"` → `"Available tables:"` in
`UnknownSourceTable` Display impl, plus any test assertions that matched the old label. New code HEAD pending.
State-manager will record the new HEAD on the next burst.

---

### OBS-2 — LOW: `available_tables` field in `UnknownSourceTable` Display rendered under wrong label

**Severity:** LOW
**Status:** Being CLOSED by implementer (coupled to OBS-1; same code fix changes the label).

**Finding:** `PrismError::UnknownSourceTable` Display rendered `available_tables` content under the label
`"Available sensors:"` instead of the canonical `"Available tables:"` (the label used by E-QUERY-037 for
the analogous field, and now codified in the amended E-QUERY-036 Message Format). This is the code-side
manifestation of the OBS-1 taxonomy-vs-implementation gap. No independent fix needed — same implementer
fix-burst as OBS-1 (label change `"Available sensors:" → "Available tables:"` in the Display impl).

---

## Positives Verified Clean (do NOT reflag in subsequent passes)

The following surfaces were reviewed and found clean. Each is explicitly declared clean to prevent
re-flagging on subsequent passes:

- **Temporal NOW()/INTERVAL production wiring:** `execute_against_session` `Ast::SqlPipe` arm correctly
  uses `pipe_sql_emitter::pipe_to_executable_sql` for SQL lowering; NOW()/INTERVAL constant-fold is wired
  to `eval_temporal_now_minus_interval` in `materialization.rs`; PqlNormalizer temporal arm emits bare ISO
  string (spec-compliant: production OCSF Datetime columns are Arrow Utf8, not epoch-millis i64 — architect
  adjudication at D-1335 resolves the type question). Tests include discriminating + negative-control assertions
  (confirmed pass-1 D-1334, pass-2 D-1336, pass-3 this pass).

- **FORBID-BOTH 0-row hoist Step 1b:** `Ast::SqlPipe` FORBID-BOTH enforcement hoists the 0-row short-circuit
  correctly; E-QUERY-040 fires before DataFusion planning when both LIMIT and PIPE-LIMIT are present;
  `execute_against_session` step order verified (BC-2.11.020 invariant).

- **Filter-mode load-bearing tests:** `filter_mode.rs` integration tests confirmed load-bearing with
  `SeverityStubAdapter` (2H+3L rows; exact row-count asserts); HIGH-1 fix (f03679b2) wires `Ast::Filter`
  execution into `execute_against_session` with correct DataFusion MemTable registration; assertions fail
  without the filter execution path (negative-control property confirmed D-1338).

- **E-QUERY-040 verbatim:** `E-QUERY-040` Display impl `#[error]` template matches error-taxonomy.md
  Message Format byte-for-byte (OBS-2 fix-burst 8f6bb337 verified; test asserts full substring including
  "PIPE…FORBID-BOTH" pedagogical content).

- **Relocated `mode_bridge_normalized_pql` / `find_first_unquoted_pipe`:** Functions confirmed in
  `prism-query/src/error_recovery.rs` (not `prism-mcp/src/error_mapping.rs`); prism-mcp calls via
  public path; BC-2.11.023 Architecture Anchor aligns; story File Structure mandate satisfied (D-1338 OBS-1).

- **MCP `-32602` mapping + negative `-32000` controls:** `map_prism_error` has explicit `-32602 INVALID_PARAMS`
  arm for `PrismError::UnknownSourceTable` (P6-02 implementer work-order fulfilled); `#[non_exhaustive]`
  catch-all `-32000` does NOT fire for `UnknownSourceTable`; unit test confirms `-32602` / E-QUERY-036 substring.

- **BC version pins synced (story v1.4):** Story Behavioral Contracts table verified at v1.4 — BC-2.11.023
  at v1.2 and BC-2.10.015 at v1.1 (corrected by D-1339 story-writer fix). No stale pins remain.

- **`#[non_exhaustive]` on new types:** All new public types added in diff `903c8fcb..f03679b2` carry
  `#[non_exhaustive]` where required; non-exhaustive gate 87/87; ci.yml EXPECTED=87 consistent.

- **SAP-2 N/A:** No `.prism/specs/sensors/*.toml` files modified in this diff; SAP-2 probe not applicable.

- **SID-1 PASS:** No `#[ignore]`'d integration tests drive new behaviors without unit-test coverage;
  all new AC behaviors have unit tests in `filter_mode.rs` and adjacent modules.

---

## Summary

Pass 3 on `f03679b2` reading story v1.4 identified:
- 1 HIGH (SAP-1 catalog gap) — CLOSED by PO (BC-2.16.002 v1.90)
- 1 MED (POL-24 E-QUERY-036 taxonomy drift) — CLOSED by PO (error-taxonomy v1.98)
- 1 LOW (OBS-1 code-side label) — being CLOSED by implementer (new code HEAD incoming)

3-CLEAN streak RESETS to 0/3 (code HEAD will change with implementer fix). Re-gate required on
new implementer HEAD per frozen-HEAD streak rule.

NEXT: Implementer closes OBS-1/OBS-2 code-side → `just check` GREEN on new HEAD → re-freeze →
LOCAL adversary Pass 1 on new HEAD (streak 0/3, fresh start).
