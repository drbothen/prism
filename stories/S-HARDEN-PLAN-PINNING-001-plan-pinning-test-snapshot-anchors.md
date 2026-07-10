---
document_type: story
story_id: "S-HARDEN-PLAN-PINNING-001"
title: "Plan-pinning test snapshot anchors — convert high002 substring guards to structural SQL-shape assertions"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
spec_version: "v0.1"
level: ops
producer: story-writer
timestamp: "2026-07-10"
modified: "2026-07-10"
input-hash: ""
inputs:
  - crates/prism-query/src/tests/high002_plan_pinning_tests.rs
  - .factory/research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md
  - .factory/specs/behavioral-contracts/BC-2.11.021-temporal-grammar-now-interval-planning-time-constant-injection.md
  - .factory/specs/behavioral-contracts/BC-2.11.003-prismql-sql-mode.md
traces_to: "F-CSD-P4-005"
origin_finding: "F-CSD-P4-005 [process-gap]"
origin_cascade: "DEFECT-CSDEVICES-EMPTY-PIPELINE-001 LOCAL pass-4, 2026-07-10"
origin_artifact: ".factory/research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-11]
crates_touched:
  - prism-query
target_module: "crates/prism-query/src/tests/high002_plan_pinning_tests.rs"
behavioral_contracts:
  - BC-2.11.021
  - BC-2.11.003
# BC status: BC-2.11.021 and BC-2.11.003 are both active. Story can advance
# to ready when test-writer and PO confirm the anchored-assertion scope.
# AC↔BC bidirectional traces required before status=ready (S-7.01).
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.0
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 4
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-HARDEN-PLAN-PINNING-001: Plan-pinning test snapshot anchors

## §Origin — [process-gap] F-CSD-P4-005

**Cascade:** DEFECT-CSDEVICES-EMPTY-PIPELINE-001 LOCAL pass-4, 2026-07-10
**Adjudication artifact:** `.factory/research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md`
**Co-evidence:** DEFECT-EQUERY042-GROUPBY-DEADARM-001 PR-LEVEL pass-2 (same class)

Tests in `high002_plan_pinning_tests.rs` that exercise the `normalize_expr` /
`normalize_predicate` surface currently assert only substring absence/presence:

```rust
// Current pattern — insufficient
assert!(!normalized.to_uppercase().contains("NOW()"), ...);
assert!(!normalized.to_uppercase().contains("INTERVAL"), ...);
assert!(normalized.contains('\''), ...);
```

This guards against the injected NOW() surviving, but does NOT anchor the resulting
SQL shape. Consequence demonstrated twice this cycle:

1. **F-CSD-P4-001 (HIGH):** An unauthorized `Expr::InSubquery → COUNT(*)` rewrite was
   introduced and `test_med1_expr_insubquery_select_projection_temporal_folded` passed
   cleanly — no NOW() or INTERVAL in the rewritten form, a quoted string was present —
   because the rewritten SQL structure diverged from the correct `arrow_cast(...)` form
   with no assertion catching the structural change.

2. **EQUERY042 PR-LEVEL pass-2:** The same class was flagged independently — substring
   guards prove only that unwanted tokens are absent, not that the wanted tokens are
   present in the right structural positions.

**Proposed remediation (F-CSD-P4-005):** establish a pinned-plan convention that
snapshots/anchors the expected normalized SQL string alongside the existing substring
guards, so any structural rewrite changes the pinned string and the delta is auditable
in PR review. Also document the SAP-3 candidate probe for future codification
via session-review.

## Narrative

As a Prism developer reviewing a PR that modifies `normalize_expr` or
`normalize_predicate`, I want the `high002_plan_pinning_tests.rs` tests to include
anchored SQL-shape assertions that fail whenever the emitted SQL structure changes,
so that structural rewrites (e.g., an unauthorized IN-subquery → aggregate rewrite)
are caught by the test suite before they reach PR review.

## Behavioral Contracts

| BC | Title | Relevance |
|----|-------|-----------|
| BC-2.11.021 | Temporal Grammar — NOW()/INTERVAL Planning-Time Constant Injection | Postcondition: DataFusion receives a concrete `arrow_cast(...)` comparison — the exact SQL shape must be asserted, not just the absence of NOW() |
| BC-2.11.003 | PrismQL SQL Mode Parsing | Invariant: Expr::InSubquery in projection/GROUP BY/ORDER BY → E-QUERY-043 plan-time rejection; structural rewrites (e.g., COUNT(*) substitution) must not silently pass the test suite |

## Acceptance Criteria

### AC-001 — Epoch-fixed helper replaces `Utc::now()` in anchored tests
(traces to BC-2.11.021 invariant: "NOW() is always evaluated at plan time")

The tests that assert on the full normalized SQL string use a deterministic, epoch-fixed
`TimestampLiteral` constant (e.g., ISO string `"2020-01-01T00:00:00Z"`) instead of
`Utc::now()`, so the expected SQL string is hardcodable and stable across CI runs.
Existing live-clock tests that only assert on substring absence/presence
(`test_high002_sql_mode_normalized_form_has_no_runtime_now`,
`test_high002_sqlpipe_head_normalized_has_no_runtime_now`) may retain `Utc::now()` —
the anchored assertions target the normalize-surface tests.

### AC-002 — Normalize-surface tests anchor the full SQL string
(traces to BC-2.11.021 postcondition: "DataFusion sees a concrete `WHERE timestamp > arrow_cast('...', 'Timestamp(Microsecond, Some(\"UTC\"))')` comparison")

For each `normalize_expr` / `normalize_predicate` test that previously only checked
substring absence/presence, add an `assert_eq!(actual_sql, expected_sql, ...)` against
a deterministic expected string capturing the full normalized SQL shape, including the
`arrow_cast` form and the structural position of the injected timestamp. At minimum:
`test_high003_pqlnormalizer_round_trips_standard_sql_demo_queries` and
`test_med1_expr_insubquery_select_projection_temporal_folded` (or its replacement)
gain anchored assertions. Other normalize-surface tests in the same module follow
the same convention.

### AC-003 — InSubquery projection test asserts correct plan-time rejection, not structural rewrite
(traces to BC-2.11.003 invariant: "Expr::InSubquery in SELECT projection/GROUP BY/ORDER BY IS NOT SUPPORTED → E-QUERY-043 plan-time rejection")

`test_med1_expr_insubquery_select_projection_temporal_folded` (or a successor test)
asserts that after `inject_now` fold, the normalized AST is NOT passed to DataFusion
with a structural rewrite — the correct expectation is either:
(a) `normalize` returns `None` (detect guard fires — correct for a query containing
an unsupported Expr::InSubquery that reaches the normalizer), OR
(b) if normalization is tested on a pre-validated path (after the E-QUERY-043 gate),
the assertion documents the exact normalized form that is expected.
Either way, the test must fail if a COUNT(*) rewrite or any other structural substitution
of the IN-subquery is silently present in the output.

### AC-004 — SAP-3 candidate documented for session-review
(traces to BC-2.11.003 invariant: "Expr::InSubquery... rejection is plan-time only, after temporal gate")

A SAP-3 candidate entry is added to the current wave's `lessons.md`
(`.factory/cycles/wave-5-e-demo-fidelity/lessons.md` or the equivalent open cycle
lessons file) describing the class:

> **SAP-3 candidate — normalize_expr/normalize_predicate structural-rewrite detection:**
> For every adversarial pass on stories or PRs touching
> `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` or any module that
> calls `PqlNormalizer::normalize`, `normalize_expr`, or `normalize_predicate`:
> verify that affected tests assert the FULL normalized SQL shape (exact string or
> `assert_eq!`), not only substring absence/presence. A test that asserts only
> `!contains("NOW()")` and `contains('\'')` is INSUFFICIENT as a normalization
> guard — it cannot detect structural rewrites of the intervening expression.
> Finding class: PROCESS-GAP (P3); add as a finding if violation present.

The entry cites F-CSD-P4-001 (COUNT(*) rewrite) and F-CSD-P4-005 (this finding)
as the recurrence evidence. This story does NOT add SAP-3 to `CLAUDE.md` (that is
a human-only edit per Pipeline Authority); it records the candidate for the human
to promote.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `PqlNormalizer` | `crates/prism-query/src/ast.rs` | Pure (normalize_expr / normalize_predicate produce strings from AST) |
| Plan-pinning tests | `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | Pure unit tests (no I/O) |
| Temporal injection | `crates/prism-query/src/ast.rs` (`inject_now`) | Pure |

Architecture section references:
- `architecture/module-decomposition.md` §SS-11 PrismQL Query Engine
- `architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md` §D4

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Anchored test with `Utc::now()` inside the pinned boundary | Must use epoch-fixed constant; `Utc::now()` in the expected string is a test defect |
| EC-002 | `normalize` returns `None` for a detect-path query | Test must explicitly document which path (detect vs emit) is being exercised |
| EC-003 | Future SQL emitter change alters the `arrow_cast` form | Anchored test fails and requires deliberate human decision to update the expectation string |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~130 | ~1,800 |
| BC-2.11.021 (plan-pinning BC) | ~170 | ~2,400 |
| BC-2.11.003 (SQL mode BC) | ~170 | ~2,400 |
| high002_plan_pinning_tests.rs (target file, 1922 lines) | 1922 | ~26,000 |
| ast.rs (PqlNormalizer source) | ~800 | ~11,000 |
| **Total estimate** | | **~43,600 tokens** |

Fits comfortably within a 100k-token agent context window (~44%). No split required.

## Tasks

- [ ] Read `high002_plan_pinning_tests.rs` in full to catalogue all tests that assert
      only substring absence/presence on the normalized SQL output.
- [ ] Add a `test_epoch_fixed_timestamp()` helper or constant (`ISO_EPOCH = "2020-01-01T00:00:00Z"`)
      to the test module for use in anchored assertions.
- [ ] For each identified test, add an `assert_eq!(actual_sql, expected_sql, "...context...")` that
      captures the full normalized SQL form using the epoch-fixed constant.
- [ ] Update or replace `test_med1_expr_insubquery_select_projection_temporal_folded` to satisfy AC-003.
- [ ] Run `cargo nextest run -p prism-query --no-fail-fast` and confirm all new anchored
      assertions are GREEN.
- [ ] Add SAP-3 candidate entry to the appropriate wave lessons.md file (AC-004).

## Previous Story Intelligence

N/A — first story in the plan-pinning test hardening sub-track. Prior context:
- `S-PRISMQL-NATIVE-TEMPORAL-TYPING-001` (merged) established `high002_plan_pinning_tests.rs`
  and the `inject_now` / `PqlNormalizer::normalize` surface. That story added the
  current substring guards as the initial plan-pinning test layer.
- `DEFECT-EQUERY042-GROUPBY-DEADARM-001` (PR #220, merged develop@b9cf3f9b) added 15 tests
  to the same module; no anchored SQL-shape assertions were added.

## Architecture Compliance Rules

- **TD-VSDD-091:** Cite function names and behavioral anchors (`normalize_expr`,
  `PqlNormalizer::normalize`), NOT file/line numbers. The test file name
  (`high002_plan_pinning_tests.rs`) is a stable module-level anchor; line numbers
  are forbidden.
- **No `unwrap()` / `expect()` in production code** — test code may use `expect()` with
  explanatory messages (pattern already established in the test module).
- **Arrow cast form (ADR-052 D3):** the canonical emitter form for `Literal::Timestamp`
  is `arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')`. Anchored assertions
  must reflect this form, not `TIMESTAMP '...'` (which DataFusion 53.1.0 lowers to
  `Timestamp(Nanosecond, None)`).

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `chrono` | workspace-pinned | `DateTime<Utc>`, `TimeZone::with_ymd_and_hms` for epoch-fixed constant |
| `nextest` | workspace-pinned | Test runner; `just iter prism-query` for fast inner loop |

Do NOT introduce `insta` (snapshot-testing crate) — inline `assert_eq!` is sufficient
and avoids a new test-framework dependency. If snapshot testing is desired in future,
an ADR is required first.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | Modify | Add epoch-fixed constant + anchored `assert_eq!` assertions to identified tests |
| `.factory/cycles/wave-5-e-demo-fidelity/lessons.md` (or current open cycle) | Modify | Add SAP-3 candidate entry (AC-004) |

No new files. No new crates. No Cargo.toml changes.
