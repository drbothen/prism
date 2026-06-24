---
document_type: adversarial-review
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
review_type: FRESH-CONTEXT-AUDIT
review_scope: Group-2 (prism-mcp)
worktree_head: 3b73cc08
diff_range: b63aef87..3b73cc08
date: 2026-06-24
reviewers:
  - agent: vsdd-factory:adversary
    agent_id: aef91718b194f786c
  - agent: vsdd-factory:code-reviewer
    agent_id: af8ab95cd823d5fdb
convergence_status: NOT-PRODUCTION-GRADE
three_clean_streak: 0/3
clean_strict: false
clean_pr_merge: false
fix_burst_routed: true
decision: D-1323
---

# Group-2 Fresh-Context Audit — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Review type:** FRESH-CONTEXT AUDIT (NOT a 3-CLEAN cascade pass — does NOT advance streak)
**Worktree HEAD:** `3b73cc08`
**Diff range:** `b63aef87..3b73cc08`
**Date:** 2026-06-24
**Reviewers:** vsdd-factory:adversary (aef91718b194f786c) + vsdd-factory:code-reviewer (af8ab95cd823d5fdb)
**Verdict:** NOT production-grade. 3 CRIT + 3 HIGH + 2 MED + 3 LOW. Fix-burst routed.

---

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL  | 3     | Routed to implementer (CRIT-001, CRIT-002) + test-writer/implementer (CRIT-003) |
| HIGH      | 3     | Routed to implementer (HIGH-001, HIGH-002, HIGH-003) |
| MEDIUM    | 2     | Routed to implementer (MED-001, MED-002) |
| LOW       | 3     | Routed to implementer (fold into CRIT-002 and sibling bursts) |
| VERIFIED CLEAN | — | OrgRegistry Arc-DI wiring; guard-reorder on 3 named tools; SAP-1 catalog; SAP-2 N/A |

Both reviewers converged independently on HIGH-002 (partial guard-reorder sweep 3/41) and
HIGH-003 (wildcard E-QUERY-001 fallback).

---

## Critical Findings

### CRIT-001 — Dead reference-content path (TD-VSDD-059 paper-fix)

**BC:** BC-2.11.022
**Severity:** CRITICAL
**Route:** implementer

`build_reference_content` is dead code. The production dispatch path is:

```
dispatch_read_resource → prismql://reference arm → schema::render_pql_reference_resource
  → PQL_REFERENCE_CONTENT = include_str!("pql_reference.md")
```

The `include_str!` macro was NOT removed during Group-2 implementation. Demo-blocking issues
GRAMMAR-008/009/017 are therefore NOT closed on the live path — the new `build_reference_content`
function exists but is never called. This is a TD-VSDD-059 paper-fix: the structural change
(new function) is present but the wiring that would make it the live path is absent.

**Required fix:** Wire `build_reference_content` as the production implementation under
`dispatch_read_resource`'s `prismql://reference` arm. Remove or tombstone the `include_str!`
fallback so the compile-time reference parity gate (BC-2.11.022) exercises the real runtime path.

---

### CRIT-002 — `normalized_pql` field unwired on production path (TD-VSDD-059 paper-fix)

**BC:** BC-2.11.023 / AC-010
**Severity:** CRITICAL
**Route:** implementer + test-writer

The production error path `prism_error_to_structured_call_result → QueryParseFailed arm`
hardcodes `normalized_pql: None`. The function `map_prism_error_to_structured` (which computes
the rewrite via `mode_bridge_normalized_pql`) has **no production caller** — it is test-only.
The BC postcondition requiring `normalized_pql` to carry the rewritten query on parse failure is
therefore violated at runtime.

**AC-010 adjudication (source-of-truth precedence per CLAUDE.md):** BC-2.11.020 (SqlPipe
composition, later and more-specific) supersedes the old "mode-mixing error" example in the
test. The test query `SELECT * FROM t WHERE severity = 'HIGH' | limit 10` now parses
successfully (Group-1 SqlPipe composition), so the test premise is stale. Resolution: the
test-writer must replace it with a genuinely-invalid mode-mix that still triggers a D1 error
post-composition AND drive the REAL production query-tool error envelope (not the unwired
helper). The BC-2.11.023 postcondition itself remains valid and is currently VIOLATED.

**Required fix:**
1. test-writer: replace the stale AC-010 test with a production-path test using a genuine
   D1 error trigger
2. implementer: wire `map_prism_error_to_structured` (or its equivalent logic) into the
   production `QueryParseFailed` arm so `normalized_pql` is populated when the mode-bridge
   rewrite is available

---

### CRIT-003 — Tautological CI 3-tier gate; ExampleKind variants do not match BC mandate

**BC:** BC-2.11.022 / ADR-045 D3 / AC-007
**Severity:** CRITICAL
**Route:** test-writer + implementer

The CI 3-tier gate has a tautological negative gate: the "No NegativeE040 entries in current
stub" message means the gate that should reject malformed queries NEVER fires because no
`NegativeE040`-tagged examples exist. Additionally:
- The registry-parity gate is absent
- Error examples are comment-prefixed and skipped from round-trip testing
- `ExampleKind` variants are `Basic / Advanced / Error` but BC-2.11.022 mandates
  `Positive / NegativeE040 / NegativeOther`

The net effect: the CI parity gate that BC-2.11.022 requires to enforce GRAMMAR-008/009/017
closure does not actually exercise the negative-path contracts.

**Required fix:** Rename or extend `ExampleKind` to match BC-mandated variants
(`Positive / NegativeE040 / NegativeOther`). Add at least one `NegativeE040` example to the
reference content so the gate is non-vacuous. Wire comment-prefixed error examples into the
round-trip. Add registry-parity assertion.

---

## High Findings

### HIGH-001 — BLOCKER-003 prompt-hang fix unverified (forbidden defer-pattern)

**BC:** BC-2.10.016 / ADR-046 D6
**Severity:** HIGH
**Route:** implementer

AC-015/016 tests wrap `std::future::ready()` which cannot time out. No `cargo-expand`
investigation was performed on the `prompts.rs` dispatch path. The dispatch path in
`prompts.rs` itself is unchanged from before the fix. Claiming BLOCKER-003 closed via tests
that are structurally incapable of detecting a hang is a forbidden defer-pattern
("deferred to manual validation" is not a production-grade closure).

**Required fix:** Replace `std::future::ready()` wrappers with tests that exercise the real
async dispatch path. Either (a) use a mock executor that can detect non-completion within a
timeout, or (b) refactor the production path so a unit test can assert on the sync vs async
branching decision. `cargo-expand` investigation of `prompts.rs` required before closure.

---

### HIGH-002 — Guard-reorder applied to only 3 of ~41 NOT_YET_AVAILABLE handlers

**BC:** BC-2.10.017 (INV-NOT-YET-AVAILABLE-GUARD-ORDER)
**Severity:** HIGH
**Route:** implementer

The guard-reorder (emit `not_yet_available` BEFORE audit) was applied to only 3 handlers:
`list_infusions`, `infusion_status`, `plugin_status`. Approximately 38 sibling handlers still
emit `emit_tool_audit` before returning `-32003`:

- `create_schedule`, `list_schedules`, `delete_schedule`
- `get_diff_results`
- `list_plugins`, `reload_infusion`, `reload_plugin`
- case/rule/pack/action/credential/alert handlers (≥12 more)

The invariant text in BC-2.10.017 is universal — it applies to all NOT_YET_AVAILABLE tool
dispatch paths. Both reviewers converged on this finding independently. Under the
production-grade default (CLAUDE.md §Canonical Principle): production-grade reading = fix all
~41 handlers in scope.

**Required fix:** Apply the guard-reorder sweep to all NOT_YET_AVAILABLE handlers in the
MCP tool dispatch. TD-VSDD-060 sibling-site sweep obligation applies.

---

### HIGH-003 — Wildcard arm assigns E-QUERY-001 to all non-parse errors

**BC:** error-taxonomy
**Severity:** HIGH
**Route:** implementer (fold into CRIT-002 fix; delegate to shared helper)

`map_prism_error_to_structured` wildcard arm `_ => ("E-QUERY-001", ...)` assigns code
E-QUERY-001 to all non-parse error variants. The correct per-variant mappings are:

| PrismError variant | Correct code |
|--------------------|-------------|
| QueryPlanFailed    | E-QUERY-002  |
| QuerySecurityLimitExceeded | E-QUERY-003 |
| ColumnNotFound     | E-QUERY-038  |
| TableNotAvailable  | E-QUERY-037  |

Both reviewers converged on this finding. The fix should be folded into the CRIT-002 wiring
burst: replace the wildcard arm with exhaustive per-variant matching and a delegate to a
shared helper so the production path and any test path use the same dispatch table.

---

## Medium Findings

### MED-001 — Test helper wires empty OrgRegistry (TD-VSDD-060 sibling-sweep miss)

**BC:** BC-2.10.015
**Severity:** MEDIUM
**Route:** implementer

`server_with_write_executor_acme_crowdstrike` in `tool_dispatch_tests.rs` wires
`OrgRegistry::new()` (empty) → silent `client_registered: false` for "acme". This is a
TD-VSDD-060 sibling-sweep miss: the Group-2 OrgRegistry Arc-DI wiring was applied to the
production boot path but the test helper was not updated to match.

**Required fix:** Wire a seeded `OrgRegistry` containing "acme" into
`server_with_write_executor_acme_crowdstrike` so `client_registered` returns `true` for
the expected client.

---

### MED-002 — `find_first_unquoted_pipe` does not handle SQL escaped quotes

**BC:** BC-2.11.023
**Severity:** MEDIUM
**Route:** implementer

`find_first_unquoted_pipe` does not handle SQL escaped quotes (`''` within string literals).
For a query containing `WHERE note = 'it''s a test' | limit 10`, the function computes the
wrong pipe offset, producing an incorrect rewrite. The BC-2.11.023 postcondition for
`normalized_pql` correctness is violated for any input with escaped single quotes in string
literals.

**Required fix:** Extend the tokenizer in `find_first_unquoted_pipe` to recognize `''` as an
escaped quote within a string literal (advance by 2, remain inside string state).

---

## Low Findings

### LOW-001 — `near_text` computed at offset 0 unconditionally

**BC:** BC-2.11.023
**Severity:** LOW
**Route:** implementer (fold into CRIT-002 fix)

`map_prism_error_to_structured` computes `near_text` at offset 0 unconditionally, ignoring
`QueryParseFailed.offset`. The diagnostic context window therefore always shows the beginning
of the input rather than the actual error location.

**Required fix:** Extract `near_text` from the error's `.offset` field when available; use
a ±20-character window centered on the parse error position.

---

### LOW-002 — `build_reference_content` three-pass design; non-exhaustive `ExampleKind`

**Severity:** LOW
**Route:** implementer

`build_reference_content` makes three passes over `REFERENCE_EXAMPLES`. The `ExampleKind`
enum is non-exhaustive by implication (no `#[non_exhaustive]`), which means future variants
will be silently omitted from the reference output if the match arms are not updated.

**Required fix:** Add `#[non_exhaustive]` if the enum is meant to be extended from outside
the crate, or make the match exhaustive so the compiler enforces completeness. Combine the
three-pass into one where feasible.

---

### LOW-003 — Doc-comment citation drift (POL-22)

**BC:** BC-2.11.019 vs story anchor BC-2.11.023
**Severity:** LOW
**Route:** implementer (fold)

`error_mapping.rs` doc-comments cite BC-2.11.019 but the authoritative story-level contract
for the three-mode correctness behavior is BC-2.11.023. Under POL-22 (anti-volatile-pin:
cite behavioral anchors, not stale identifiers), the doc-comments should cite BC-2.11.023.

**Required fix:** Update doc-comment citations from BC-2.11.019 to BC-2.11.023 in
`error_mapping.rs`.

---

## Verified Clean Items

The following areas were explicitly checked and found CLEAN (no finding):

- **BC-2.10.015 OrgRegistry Arc-DI wiring:** `boot.rs:2842 Arc::clone` is the live registry
  (real wiring, not placeholder construction). `OrgSlug::new` validated per AD-017. The
  production boot path is correctly wired.

- **Guard-reorder correctness for the 3 named tools:** `list_infusions`, `infusion_status`,
  `plugin_status` — the reorder is correct on these three. No audit-skip regression on the
  live/available path (`query` tool and `list_capabilities` still emit audit normally).

- **SAP-1 (tracing emission catalog):** `rg 'event_type\s*=' crates/ --type rust` — no new
  `event_type` emission sites added in the Group-2 diff. Catalog clean.

- **SAP-2 (DTU↔TOML schema parity):** Not applicable — no sensor TOML or DTU crate touched
  in the Group-2 diff.

---

## Routing Summary

| Finding | Route | Notes |
|---------|-------|-------|
| CRIT-001 | implementer | Wire `build_reference_content` into production dispatch |
| CRIT-002 | test-writer + implementer | AC-010 test replacement + production wiring |
| CRIT-003 | test-writer + implementer | ExampleKind rename + non-vacuous gate |
| HIGH-001 | implementer | Replace untestable `ready()` wrappers |
| HIGH-002 | implementer | Sweep all ~41 NOT_YET_AVAILABLE handlers (TD-VSDD-060) |
| HIGH-003 | implementer | Fold into CRIT-002; exhaustive error-code dispatch |
| MED-001 | implementer | Seed OrgRegistry in test helper (TD-VSDD-060) |
| MED-002 | implementer | Handle `''` escaped quotes in `find_first_unquoted_pipe` |
| LOW-001 | implementer | Fold into CRIT-002; use `.offset` for `near_text` |
| LOW-002 | implementer | Non-exhaustive annotation on `ExampleKind` |
| LOW-003 | implementer | Fold; update BC citation in `error_mapping.rs` |

---

## AC-010 Source-of-Truth Adjudication

Per CLAUDE.md §Source-of-Truth Precedence: BC-2.11.020 (SqlPipe composition; later and
more-specific) supersedes the old "mode-mixing error" example premise. The test query
`SELECT * FROM t WHERE severity = 'HIGH' | limit 10` now parses successfully post-Group-1
because SqlPipe composition absorbs it. The old test was premised on this query erroring.

**Resolution:**
- test-writer must author a NEW test that uses a genuinely-invalid mode-mix (one that still
  triggers a D1 error post-composition, e.g., a query that violates forbid-both at the
  composition boundary, not a valid SQL pipe composition)
- The new test must exercise the REAL production error envelope path (not the unwired
  `map_prism_error_to_structured` helper)
- The BC-2.11.023 postcondition for `normalized_pql` remains valid and binding

---

## Convergence Status

```
CLEAN (strict):    NO  — 8+ findings present
CLEAN (PR-merge):  NO  — CRIT/HIGH findings present
3-CLEAN streak:    0/3 (TDD incomplete; this is a pre-cascade audit, not a cascade pass)
```

Next action: test-writer dispatched for Red-Gate test corrections (CRIT-002 AC-010 production
path + CRIT-003 CI gate) → implementer dispatched to close all findings. 3-CLEAN streak
resets to 0/3 after fix-burst and re-gate on frozen HEAD.
