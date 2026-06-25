---
pass: PR-LEVEL-1
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pr: 203
head: e0374818
clean_strict: YES
clean_pr_merge: YES
findings: 0
streak_after: 1/3
date: 2026-06-25
---

# PR-LEVEL Pass 1 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (PR #203)

**HEAD reviewed:** e0374818  
**CLEAN (strict):** YES — zero findings of any severity  
**CLEAN (PR-merge):** YES — zero CRIT/HIGH/MED findings  
**Streak after:** 1/3 (RESET by Pass 2 — see prlevel-pass-2-e0374818.md)

## Scope Verified

All 27 ACs reviewed against diff 903c8fcb..e0374818:

- **AC-001–003 (BC-2.11.020 SqlPipe + FORBID-BOTH):** `Ast::SqlPipe` parse and `plan_sqlpipe_query` E-QUERY-040 dual-limit check verified. Pure modes unaffected.
- **AC-004–005 (BC-2.11.021 Temporal):** `NOW()` / `INTERVAL` parse and plan-time `inject_now` substitution verified. `Expr::Now` / `Expr::Interval` arms present in all 4 AST branches.
- **AC-006–008, AC-023, AC-026 (BC-2.11.022 Reference Content):** `build_reference_content()` generates 13 required sections from live `TableRegistry`. CI 3-tier parity gate (`test_bc_2_11_022_ci_3tier_gate`) drives `plan_sqlpipe_query` and asserts `Err(RedundantRowLimit)` — load-bearing, not tautological.
- **AC-009–012, AC-027 (BC-2.11.023 Mode-Bridge + D7):** D1 diagnostic (SQL-pipe stage unknown) and D2 (SQL clause in pipe position) both implemented in `error_recovery.rs`. `build_predicate_parser()` shared across SQL/Pipe/Filter. `normalized_pql` on `StructuredErrorFields` confirmed.
- **AC-013–014 (BC-2.10.015 OrgRegistry DI):** `Arc<OrgRegistry>` wired through `FeatureFlagEvaluator` constructor. `client_registered` gate uses `slug_exists`. No `new_unchecked` in production path.
- **AC-015–016 (BC-2.10.016 Prompts fast-return):** Prompts handler returns immediately — no sensor fan-out triggered. `test_bc_2_10_016_prompts_fast_return_within_5s` is a timing test, non-ignore.
- **AC-017–018 (BC-2.10.017 NYA fast-fail):** NOT_YET_AVAILABLE guard fires before audit I/O. `test_bc_2_10_017_not_yet_available_guard_precedes_audit` and `test_bc_2_10_017_sibling_handlers_guard_precedes_audit` assert ordering.
- **AC-019:** BLOCKER-001 deferral preserved (D-1326; S-RESILIENCE-FEDERATED-001).
- **AC-020:** Runbook v1.4 §5.5 correct pipe syntax confirmed.
- **AC-021:** `UnknownSourceTableDetails` `did_you_mean` + `available_tables` confirmed.
- **AC-022, AC-025:** Enrich guided errors in all pipeline positions (SQL, Pipe, SqlPipe) confirmed via `test_bc_2_11_obs1_sqlpipe_enrich_missing_column_arg_guided_error`.
- **AC-024:** GRAMMAR-013 table present in PR description; section labels verified at PR creation.

## Standing Probes

- **SAP-1 (tracing emission catalog):** `rg 'event_type\s*=' crates/ --type rust` shows all emission sites cataloged in BC-2.16.002 §Postconditions. No new sites in diff.
- **SAP-2 (DTU↔TOML parity):** No TOML spec changes in diff. N/A.
- **SID-1 (no-ignored-test rationalization):** No `#[ignore]` gates introduced. All new tests are non-ignore.
- **Security (AD-017):** `OrgSlug::new_unchecked` absent from all new production paths. Credential values do not appear in diff.
- **FORBID-BOTH data-independence:** `plan_sqlpipe_query` FORBID-BOTH check fires at plan time (before data retrieval) via `EmptyVecAdapter`. Confirmed.
- **D1/D2 false-positive/negative:** Lowercase pipe keywords not false-positive for D2. `WHERE`/`LIMIT` uppercase correctly excluded.
- **non-exhaustive gate:** EXPECTED=87, `ExampleKind`/`SqlPipeQuery`/`UnknownSourceTableDetails` gated. Compile-fail test present.

## Verdict

CLEAN (strict): YES  
CLEAN (PR-merge): YES  

Streak 1/3 on FROZEN HEAD e0374818.  
Pass 2 dispatched on SAME HEAD e0374818 per BC-5.39.001.  
Pass 2 found 2 LOW findings (OBS-1/OBS-2) — streak RESET 0/3; see prlevel-pass-2-e0374818.md.
