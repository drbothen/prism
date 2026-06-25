---
pass: 13
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
frozen_head: 71b196ad
base_head: 903c8fcb
date: 2026-06-25
adversary_pass_type: LOCAL
clean_strict: true
clean_pr_merge: true
findings_count: 0
streak: "2/3"
---

# LOCAL Adversarial Pass 13 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Frozen HEAD:** `71b196ad` (UNCHANGED from pass 12)
**Base (develop):** `903c8fcb`
**CLEAN(strict):** YES — zero findings of ANY severity
**CLEAN(PR-merge):** YES
**Streak:** 2/3 (second consecutive clean pass on UNCHANGED HEAD `71b196ad`)

## Pass Summary

Pass 13 deliberately targeted novel attack vectors NOT probed in pass 12, to avoid re-treading the same surface. Goal: find any remaining gap before declaring streak complete.

### Novel Attack Vector 1: OFFSET Handling

PrismQL grammar does not support `OFFSET` in the pipe DSL; SqlPipe queries route through `plan_sqlpipe_query`. Adversary probed whether an `OFFSET`-bearing SQL head (valid DataFusion SQL) could be composed with a `| tail N` pipe stage, thereby constructing a double-row-cap path that bypassed the Limit+Tail FORBID-BOTH check. Verified: `OFFSET` is a SQL-head construct evaluated in the DataFusion layer, not a `PipeStage` variant. The FORBID-BOTH check operates on `PipeStage` only. No bypass path exists — correct.

### Novel Attack Vector 2: Hidden Stats/Dedup Caps

`PipeStage::Stats` and `PipeStage::Dedup` do not cap rows in the sense of `LIMIT N` — they aggregate or deduplicate but do not impose a hard row count ceiling per the grammar spec. Adversary verified neither variant is counted in the FORBID-BOTH check and confirmed this is correct per ADR-043 D4: the invariant covers row-LIMIT semantics, not aggregation. No finding.

### Novel Attack Vector 3: LIMIT 0

`SELECT * FROM t | limit 0` — Adversary probed whether `PipeStage::Limit(0)` is correctly handled. The FORBID-BOTH check fires on detection of any `Limit` or `Tail` stage pair, not on the value. A `| limit 0` alone (without a second capping stage) does not trigger FORBID-BOTH. DataFusion handles `LIMIT 0` as an empty result set, which is correct behavior. No finding.

### Novel Attack Vector 4: Multiple Limit Stages in Sequence

`SELECT * FROM t | limit 10 | limit 5` — two consecutive Limit stages with no Tail. Adversary verified `plan_sqlpipe_query` FORBID-BOTH check fires on the first Limit seen and then finds the second Limit — `Err(RedundantRowLimit)`. The existing `test_bc_2_11_022_limit_after_limit_rejected` test covers this. No gap.

### Novel Attack Vector 5: Nested Temporal

`SELECT * FROM t WHERE ts > NOW() - INTERVAL '7d' | tail 5` — compound: temporal filter in SQL head + `| tail` pipe stage. Adversary probed both the temporal injection path and the FORBID-BOTH path. The temporal filter is in the SQL head (WHERE clause), not a pipe stage, so FORBID-BOTH does not fire. The `inject_now` substitution applies to the SQL head before DataFusion execution. The `| tail 5` is a pipe stage — it is the first (and only) row-capping stage, so FORBID-BOTH does not fire. Result is valid. No finding.

`SELECT * FROM t WHERE ts > NOW() - INTERVAL '7d' | limit 5 | tail 3` — temporal + LIMIT + Tail. FORBID-BOTH fires on Limit+Tail pair regardless of the temporal head. Correct.

### Novel Attack Vector 6: Prompt-Dispatch Path

Adversary reviewed the MCP prompt-dispatch path (AC-015/AC-016) for any gap in the `investigate_host` and missing-arg routing introduced in the fix burst. The two `mcp_infrastructure.rs` tests drive these via real rmcp duplex transport. No paper-fix pattern detected. No finding.

### AC-001..AC-027 Spot-Check (Different Angles)

On pass 13, adversary re-verified the ACs that had the highest historical finding density:

- **AC-002** (FORBID-BOTH gate): verified via novel vectors 2, 3, 4 above. Solid.
- **AC-004/AC-005** (NOW()/INTERVAL, all 4 AST arms): verified inject_now is called for SQL mode (normalize_pql path) and SqlPipe head. Both arms confirmed producing `Literal::Timestamp` not raw `NOW()`. No regression.
- **AC-007** (E-QUERY-040 verbatim string): string comparison confirms no drift. No finding.
- **AC-022/AC-025** (guided error messages for pipe stages): rewrite helpers present and DRY'd. No finding.

### SAP-1

No new emission sites. BC-2.16.002 catalog complete. PASS.

### SAP-2

No TOML sensor spec changes. N/A.

### SID-1

No `#[ignore]`'d tests. PASS.

### TD-VSDD-059 Paper-Fix Check

All closures have load-bearing assertions. No doc-only fixes. PASS.

## Conclusion

**CLEAN(strict) = YES. CLEAN(PR-merge) = YES. Zero findings.**

Novel attack vectors — OFFSET composition, hidden stats/dedup caps, LIMIT 0 edge, multi-limit, nested temporal, prompt-dispatch — all handled correctly. No gap found on any surface probed.

**Streak: 2/3.** Third consecutive clean pass required on UNCHANGED HEAD `71b196ad`.
