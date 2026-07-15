---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [6]
feature_head_at_review: 97cb070e
date: 2026-07-15
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 0
streak_after: 1/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 6 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 6 (frozen 97cb070e; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 0/3→1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

Streak: **1/3** (BC-5.39.001 strict criterion: pass-6 has ZERO findings of any severity; CLEAN(strict)=YES; streak advances 0/3→1/3 on frozen HEAD 97cb070e; DRIFT-ORCH-PRLEVEL-PUSH-001 clean — HEAD verified 97cb070e local=origin=PR before and after)

Cascade tally (as of this pass): **6 passes / 3 fix-bursts**

CLEAN(strict): YES — ZERO in-perimeter findings
CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings

---

## Findings

No in-perimeter findings.

---

## Out-of-Scope Deferred Item (carried from LOCAL cascade and prior PR-LEVEL passes; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1+2+3+4+5; not re-raised as a PR-LEVEL finding.

---

## Positive Verifications

- **Boundary/composition gate-interaction probe — NO BYPASS:** Gate ordering verified per ADR-048 §D.7.4 (WHERE aggregate check before HAVING interception before infusion-registry guard; E-QUERY-001 fires before E-QUERY-039 for HAVING aggregate names; enrich-UDF check before temporal check in position order). No bypass path identified where a malformed aggregate name in HAVING position reaches the registry guard without intercepting at the `DATAFUSION_BUILTIN_AGGREGATE_NAMES` check.

- **Detail-builder spec-code-test triangle probe — NO BYPASS:** `having_aggregate_interception_detail(name: &str) -> String` (private helper in `check_enrich_udf_availability`) verified byte-exact vs BC-2.11.019 v1.23 §OBS-004 canonical template for both branches: (1) `percentile` branch → byte-verbatim `(field, p)` canonical template locked by `test_having_aggregate_detail_percentile_canonical`; (2) else branch → signature-neutral `(...)` template locked by `test_having_aggregate_detail_generic_uses_ellipsis`. Three full-string unit tests lock both branches. Single production call site in HAVING-interception loop — no uncovered call path.

- **Offset truthfulness across all modes probe — NO BYPASS:** Error offset (Span / byte-position markers) verified correct per-mode including SqlPipe shift helpers. No mode-specific offset skew identified. The offset computation paths for SQL, SqlPipe, and Pipe modes each carry independent evidence from the LOCAL cascade GREEN locks (EC-11-086 mirrors verified GREEN at 97cb070e per pass-5 confirmation; two SqlPipe-head-HAVING tests from fix-burst-37 verified bit-identical).

- **Repeated-construct idempotence probe — NO BYPASS:** `having_aggregate_interception_detail` called multiple times with the same `name` argument produces identical output on each call (pure function — no mutable state; `String::from` and string literal concatenation; no side effects). Idempotence holds trivially; the `eq_ignore_ascii_case` predicate is deterministic.

- **SAP-1 PASS:** Zero net-new `event_type =` emissions added in fix-burst-38 (confirmed from pass-5 verification; HEAD 97cb070e unchanged). Settled methodology carries from pass-5: 55 raw occurrences / 12 distinct values verified against BC-2.16.002 v1.61 catalog 92 rows; ZERO net-new emissions.

- **POL-22 Phase A+C PASS:**
  - Phase A (adversary independently re-derived all load-bearing evidence; no reliance on implementer disclosure)
  - Phase C (all positive verifications cross-checked against code at 97cb070e, not only pass reports)
  - `having_aggregate_interception_detail` present at 97cb070e — grep confirmed; two-branch structure (`eq_ignore_ascii_case("percentile")` guard) verified
  - `debug_assert_eq!` ABSENT at 97cb070e in HAVING-interception loop — `rg 'debug_assert' crates/prism-query/src/engine.rs` returns ZERO results in HAVING-interception loop context
  - EC-11-086 five tests (2 SqlPipe-head-HAVING mirrors + 3 casing-lock verbatim tests from fix-burst-37) verified GREEN at 97cb070e — behavioral output of HAVING interception unchanged
  - EC-11-087 passthrough test group (4 tests) carry NEGATIVE-CLAIM-ONLY LOCK doc-comments per test-writer fix-burst-38; distinct_count passthrough to registry confirmed (not in DATAFUSION_BUILTIN_AGGREGATE_NAMES; OBS-004 deliberate asymmetry intact)
  - Pin sweep completeness (pass-5 verification): 18 v1.22 cite sites updated to v1.23; `grep -r "BC-2.11.019 v1.22" .factory/` returns ZERO live non-exempt results at 97cb070e

- **TD-VSDD-059 PASS:** All fix-burst-38 closures have load-bearing tests (per pass-5 verification; HEAD unchanged at 97cb070e): three unit tests lock `having_aggregate_interception_detail` behavior structurally; reverting the two-branch fix causes `test_having_aggregate_detail_generic_uses_ellipsis` to fail. Doc-comment-only closures (F-PQLFN-PR5-OBS-001) are substantiated by the NEGATIVE-CLAIM-ONLY doc-comment pattern; no paper-fix class detected.

- **TD-VSDD-060 PASS:** `having_aggregate_interception_detail` is private (`fn`, not `pub fn`). Single production call site in HAVING-interception loop — no external callsites to sweep.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose.

- **POL-34 PASS:** `fail_loud_invariant_enforcement_standard` registered in policies.yaml v1.34 (F-PQLFN-PR5-OBS-003 codification). The exemplar compliant implementation (two-branch detail-builder) satisfies POL-34: typed-branch handling active in release builds; no `debug_assert!` as sole enforcement of a production invariant whose violation produces incorrect user-facing output in release.

- **POL-23 sweep verified clean (at 97cb070e):** `grep -r "BC-2.11.019 v1.22" .factory/` — ZERO live non-exempt pins after pass-5 18-site sweep. HEAD unchanged; no new BC-2.11.019 pin sites introduced in fix-burst-38 test-writer commit 97cb070e (doc-comments only; no spec-pin text).

- **Non-exhaustive gate:** No new `pub` types introduced in fix-burst-38. Non-exhaustive EXPECTED=91 unchanged at 97cb070e.

- **No secrets, no AI attribution, no `--no-verify` markers in diff.**

- **No new `unwrap()` / `expect()` in production code paths.**

- **No `reqwest` changes** — ADR-050 rustls-tls constraint untouched.

- **Spec versions verified at 97cb070e:**
  - BC-2.11.019 v1.23 (two-branch detail-builder; debug_assert REMOVED; DML scope cross-note; §OBS-004 updated)
  - BC-2.11.004 v1.48 (EC-11-085/086/087; RESERVED_KEYWORDS 21 keywords; 3 live BC sites)
  - ADR-048 v1.16 (§D.2 rewritten; §D.7.3 HAVING-exemption caveat; 13 live pins)
  - error-taxonomy E-QUERY-039 template current (v2.52)
  - policies.yaml v1.34 (POL-34 registered)

- **Novelty: LOW** — pass-6 verifies a clean HEAD with no new spec or code changes since fix-burst-38. All structural concerns from passes 1–5 are closed. Remaining cascade is convergence-only (streak 1/3; two more CLEAN(strict) passes needed on unchanged frozen 97cb070e).

---

## CI Note

CI on 97cb070e: Test (x86_64-unknown-linux-gnu) FAILED on first run (run 29404746333). Root cause: runner disk exhaustion (`mold: Disk full` + `collect2: fatal error: ld terminated with signal 7 [Bus error]`). This is the **third consecutive occurrence** of this class of infrastructure flake in the DEFECT-PQL-FNCALL-LHS-001 cascade:

| Run | Target | HEAD | Error |
|-----|--------|------|-------|
| 29394488318 | x86_64-unknown-linux-musl | 76c0fa60 (2026-07-15) | `mold: failed to write to an output file. Disk full?` |
| 29399778005 | x86_64-unknown-linux-gnu | 72d8ed8d (2026-07-15) | `couldn't create a temp dir: No space left on device` |
| 29404746333 | x86_64-unknown-linux-gnu | 97cb070e (2026-07-15) | `mold: Disk full` + `collect2: fatal error: ld terminated with signal 7 [Bus error]` |

The identical job re-run succeeded **43/43**. No code changes were required or made for the re-run.

**D-1780 watch-note threshold reached (3rd occurrence):** S-MAINT-CI-DISK-EXHAUSTION-001 drafted (story-writer; D-1781; P2; 5 pts; 5 ACs; 2 Red Gate tests; tdd_mode strict; track: Platform Engineering; `maintenance/ci-disk-hardening` branch isolated from defect PRs; 5 ACs: disk-free preflight + `jlumbroso/free-disk-space` ≥25 GB gate + `CARGO_PROFILE_DEV_DEBUG=1` + failure annotation + 3-green-run evidence). Story registered in STORY-INDEX v2.687→v2.688; total_stories 242→243.

This infra flake does not affect the frozen HEAD 97cb070e or the pass-6 review findings; the review was conducted on the frozen HEAD per DRIFT-ORCH-PRLEVEL-PUSH-001 discipline.

---

## Convergence Status

- CLEAN(strict): YES — ZERO in-perimeter findings
- CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings
- Streak: **1/3** (BC-5.39.001 strict criterion MET; streak advances 0/3→1/3; frozen HEAD 97cb070e UNCHANGED since fix-burst-38; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; NO pushes between pass-5 close and pass-6)
- Frozen HEAD: **97cb070e** (PR #223 HEAD; CI 43/43 PASS after re-run of run 29404746333)
- DRIFT-ORCH-PRLEVEL-PUSH-001: NO commits pushed since fix-burst-38; all subsequent passes must use 97cb070e as the frozen HEAD; passes taken before fix-burst-38 on earlier frozen HEADs do NOT count toward the 97cb070e streak

---

## Next Step

PR-LEVEL pass-7 on SAME frozen 97cb070e (streak 1/3 → target 2/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; NO pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 97cb070e → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge).
