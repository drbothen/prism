---
pass: 14
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
frozen_head: 71b196ad
base_head: 903c8fcb
date: 2026-06-25
adversary_pass_type: LOCAL
clean_strict: true
clean_pr_merge: true
findings_count: 0
streak: "3/3 — CONVERGED"
---

# LOCAL Adversarial Pass 14 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Frozen HEAD:** `71b196ad` (UNCHANGED from passes 12 and 13)
**Base (develop):** `903c8fcb`
**CLEAN(strict):** YES — zero findings of ANY severity
**CLEAN(PR-merge):** YES
**Streak:** 3/3 — **BC-5.39.001 LOCAL 3-CLEAN CONVERGED**

## Pass Summary

Pass 14 deliberately targeted the highest-risk surfaces not yet stress-tested in passes 12–13, to ensure the third pass is a genuine signal and not a retread.

### Highest-Risk Surface 1: FORBID-BOTH Data-Independence (EmptyVecAdapter)

Adversary probed whether the FORBID-BOTH check operates on the *plan* independently of sensor data — i.e., whether an adversary could route `SELECT * FROM t LIMIT 5 | tail 3` past the check by supplying an empty result from a sensor that returns zero rows. The fix introduced in `71b196ad` calls `plan_sqlpipe_query` at plan time (not execution time), so the check fires before any sensor fetch occurs. `EmptyVecAdapter` (the test double for zero-row sensors) cannot suppress it. Data-independence confirmed. No finding.

### Highest-Risk Surface 2: Tail/OFFSET Composition Against Real PipeStage Variants

Adversary re-examined `PipeStage` variant exhaustiveness. The `plan_sqlpipe_query` FORBID-BOTH check uses a match arm covering `PipeStage::Limit` and `PipeStage::Tail`. Adversary inspected the full `PipeStage` enum definition for any variant that could produce a row ceiling and is not covered. Confirmed: `Join`, `Where`, `Sort`, `Stats`, `Fields`, `Dedup`, `Enrich` do not produce a `LIMIT N` cap. The `parse_pipe_stage` entry point is the only production parse entry point for pipe stages. No uncovered variant. TD-VSDD-060 sibling sweep was thorough. No finding.

### Highest-Risk Surface 3: Third Parse Entry Point `parse_and_plan`

Adversary specifically probed `parse_and_plan` (the unified parse+plan function exposed by the `QueryEngine`), which routes through `normalize_pql` → AST dispatch. Verified: `plan_sqlpipe_query` is called from `execute_against_session`'s `Ast::SqlPipe` arm, which is the execution path reachable from `parse_and_plan`. The FORBID-BOTH check is not bypassable by entering through `parse_and_plan` vs the test-only `plan_sqlpipe_query` direct call. No third entry point exposed. No finding.

### Highest-Risk Surface 4: OrgSlug Embedded-Validity

`OrgSlug::new_unchecked` was removed from production paths (AC-019 scope). Adversary verified `new_unchecked_audit.rs` allowlist does not include any new call sites added in `71b196ad`. The only `new_unchecked` call sites in the diff are in `#[cfg(test)]` blocks under the allowlisted test paths. AD-017 / CLAUDE.md credential safety rule upheld. No finding.

### Highest-Risk Surface 5: D2 Mode-Bridge False Positive / False Negative

The `rewrite_d2_sql_keyword_in_pipe_position` helper (AC-027) triggers on SQL keywords appearing in pipe position. Adversary probed:
- `SELECT * FROM t | WHERE host = 'x'` (uppercase WHERE in pipe position) → rewrite fires, produces D2 guided message. Correct.
- `SELECT * FROM t | where host = 'x'` (lowercase where — valid PrismQL filter stage keyword) → rewrite does NOT fire (case-sensitive detection per the original adversary finding that led to WHERE/LIMIT uppercase exclusion). Correct — no false positive on lowercase pipe keywords.
- `SELECT * FROM t | limit 5` (LIMIT lowercase, valid pipe keyword) → rewrite does NOT fire. Correct.
Confirmed: no false positives on valid lowercase pipe keywords; no false negatives on SQL-cased keywords in pipe position. No finding.

### Highest-Risk Surface 6: SQL Injection Escaping in `sqlpipe_to_executable_sql`

The `PipeQueryBuilder::new_with_cte` / `sqlpipe_to_executable_sql` path emits DataFusion-compatible SQL. Adversary examined whether sensor table names or column names injected into the CTE SQL string are escaped correctly. Confirmed: sensor table identifiers are validated at the `OrgSlug` + source-registry boundary before reaching the emitter. Column references are bound through the schema registry (not user-supplied strings to the emitter). No injection surface. No finding.

### Final AC-001..AC-027 Completeness Sweep

Full 27-AC sweep on pass 14. Adversary looked for any AC where the implementing test is present but does not actually exercise the production path (TD-VSDD-059 paper-fix pattern). None found. Every test drives production code, not a stub or isolated function that could diverge from the wired execution path.

### SAP-1

Full workspace grep: no unregistered `event_type` emission sites. BC-2.16.002 catalog complete and consistent. PASS.

### SAP-2

No TOML sensor spec changes in diff. N/A.

### SID-1

No `#[ignore]`'d tests added. All tests in-process, non-ignored. PASS.

### TD-VSDD-059

No paper-fixes detected. All closures structurally load-bearing. PASS.

### just check EXIT=0

`just check` full workspace on `71b196ad`: 4929 tests GREEN, EXIT=0. fmt-canonical clean. non-exhaustive gate 87. No clippy warnings.

## Conclusion

**CLEAN(strict) = YES. CLEAN(PR-merge) = YES. Zero findings.**

The three highest-risk surfaces on pass 14 — FORBID-BOTH data-independence, Tail/OFFSET coverage, third parse entry point, OrgSlug embedded-validity, D2 false-pos/neg, SQL injection escaping — all confirmed closed. Genuine clean, not a retread of passes 12–13.

**Streak: 3/3.** BC-5.39.001 LOCAL 3-CLEAN CONVERGENCE ACHIEVED on frozen HEAD `71b196ad`.

---

## Cascade Summary (Passes 1–14)

| Pass | Head | Finding Count | Notes |
|------|------|--------------|-------|
| 1 (local-cascade-pass-1.md) | e518d96c | 3 CRIT + 2 HIGH + several OBS | Systemic unwired pattern; FORBID-BOTH/NOW unexecuted |
| 2 (local-cascade-pass-2-f03679b2.md) | f03679b2 | 2 HIGH + 2 MED | Mode-bridge + D2 missing |
| 3 (local-cascade-pass-3-f03679b2.md) | f03679b2 | 1 MED | AC-009 D1 message substrings |
| 4 (pass-4-81372a22.md) | 81372a22 | 2 HIGH + 2 MED | D1 verbatim + D2 unimplemented |
| 5 (pass-5-9eb55cfe-CLEAN.md) | 9eb55cfe | 0 | CLEAN streak 1/3 |
| 6 (pass-6-9eb55cfe.md) | 9eb55cfe | 1 MED | AC-023 IS-NOT-NULL note missing |
| 7 (pass-7-64d91111.md) | 64d91111 | 3 LOW/OBS | Gate test + path + runbook |
| 8 (pass-8-a0ebd740.md) | a0ebd740 | 1 LOW | SqlPipe enrich error parity |
| 9 (pass-9-f58bb9a0.md) | f58bb9a0 | 1 MED | parse_with_registry parity |
| 10 (pass-10-3f685515-CLEAN.md) | 3f685515 | 0 | CLEAN streak 1/3 (RESET by P11) |
| 11 (pass-11-3f685515.md) | 3f685515 | 1 HIGH + 1 LOW | FORBID-BOTH Tail bypass + Filter temporal guard |
| **12 (pass-12-71b196ad-CLEAN.md)** | **71b196ad** | **0** | **CLEAN strict 1/3** |
| **13 (pass-13-71b196ad-CLEAN.md)** | **71b196ad** | **0** | **CLEAN strict 2/3** |
| **14 (pass-14-71b196ad-CLEAN.md)** | **71b196ad** | **0** | **CLEAN strict 3/3 — CONVERGED** |

Total fix-bursts: ~10. All findings closed in-scope. No deferrals to tech-debt-register. Production-grade standard maintained throughout.

**NEXT:** per-AC demo evidence (demo-recorder, POL-10 story-scoped under `docs/demo-evidence/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001/`) → push feature branch → PR (pr-manager 9-step, targets develop) → PR-LEVEL 3-CLEAN cascade on frozen PR HEAD → CI → squash-merge (--admin authorized per D-1337 for harness-blocked GH approvals) → re-run pre-flight demo audit → T13 capstone → T14 recording.
