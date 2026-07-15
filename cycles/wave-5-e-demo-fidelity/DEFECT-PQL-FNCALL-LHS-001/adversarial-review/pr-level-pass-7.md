---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [7]
feature_head_at_review: 97cb070e
date: 2026-07-15
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 4
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 4
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 7 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 7 (frozen 97cb070e; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 1/3 RESET → 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

CLEAN(strict): NO — 4 OBS reported by adversary; BC-5.39.001 strict criterion requires ZERO findings of ANY severity (including OBS); streak RESETS 1/3 → 0/3.

CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings; PR is merge-eligible pending human merge-gate decisions.

Cascade tally (as of this pass): **7 passes / 4 fix-bursts** (fix-burst-39 is the disclosure-only closure of F-PQLFN-PR7-OBS-001)

---

## Observations (as reported by adversary)

**Orchestrator adjudication:** Of the 4 OBS reported, 3 (F-PQLFN-PR7-OBS-002, OBS-003, OBS-004) are VERIFICATION NOTES asserting correct behavior — they contain no defect statement and no remediation surface. Per the meta-discipline codified in §Next Step below, these belong in §Positive Verifications, not §Observations. They are listed here for audit-trail completeness (adversary filed them as OBS), with their adjudication rationale, and are also carried in §Positive Verifications. F-PQLFN-PR7-OBS-001 is the sole substantive finding.

---

### F-PQLFN-PR7-OBS-001 — LOW-006 `contains` keyword collision (substantive; disclosed in PR body)

**Severity:** OBS (contextually LOW-tier behavioral note)

**Defect statement:** The LOW-006 21-keyword gate (BC-2.11.004 v1.48, EC-11-004-006) lists `CONTAINS` / `STARTSWITH` / `ENDSWITH` / `ICONTAINS` / `ISTARTSWITH` / `IENDSWITH` as blocked predicate-LHS forms. Of these, `contains` is a same-spelled DataFusion 53.1 scalar function — so `WHERE contains(hostname, 'malware') = TRUE` is rejected at parse time even though `contains(...)` succeeds in SELECT projection position. The underscore-spelled DataFusion variants (`starts_with`, `ends_with`) are NOT in the keyword list and remain usable as predicate-LHS fn-calls.

**Is this a regression?** NO. Pre-branch, all fn-call LHS predicates failed with a generic parse error (undifferentiated). Post-branch, fn-call LHS predicates pass the gate for non-keyword names; keyword-named predicates receive a clear E-QUERY-001 rejection message. The `contains(...)` case transitions from one failure mode (generic parse error) to a different failure mode (keyword rejection message) — not from success to failure.

**Is this fail-safe?** YES. The keyword-rejection message is clear and actionable. No silent data corruption; no security implication.

**Rationale for not fixing on this branch:** BC-2.11.004 EC-11-004-006 explicitly mandates: "Surface in PR body for human feature-decision." The appropriate response is disclosure at the merge gate, not a code change. Removing `contains` from the keyword list would require adjudicating whether `contains` as an operator keyword (the LHS operator form `contains(field, value)`) should remain reserved or be re-classified as a plain fn-call. This is a product-scope decision, not a bug fix.

**Disposition:** CLOSED — fix-burst-39 (disclosure-only): pr-manager appended `### Merge-Gate Disclosure — LOW-006 DataFusion-name collision cost (F-PQLFN-PR7-OBS-001)` to PR #223 body (§MERGE-GATE DISCLOSURES section) + corresponding pre-merge checklist row `MERGE-GATE FEATURE-DECISION: LOW-006 keyword list adjudication`. Same content appended to pr-223-created.md (source-of-record). HEAD 97cb070e verified UNCHANGED after edit (body edits touch no commits; DRIFT-ORCH-PRLEVEL-PUSH-001 not triggered; streak counting continues on same frozen HEAD).

---

### F-PQLFN-PR7-OBS-002 — Walker recursion completeness probe (VERIFICATION NOTE; adjudicated as positive verification)

**Adversary filing (raw):** Walker coverage across all Predicate/Expr recursive positions — InSubquery visit.

**Orchestrator adjudication:** This is a VERIFICATION NOTE asserting correct behavior, not a defect statement. The adversary verified walker recursion is complete across all Predicate and Expr recursive positions. The sole gap (InSubquery skip) was adjudicated in §OBS-001 of the local cascade (D-PQLFN-P47-OBS-001); that adjudication is carried unchanged and out-of-perimeter per BC-5.39.002 PC2. No defect exists; no remediation surface. Carried in §Positive Verifications below.

---

### F-PQLFN-PR7-OBS-003 — Generic branch unit-test coverage probe (VERIFICATION NOTE; adjudicated as positive verification)

**Adversary filing (raw):** Generic branch of `having_aggregate_interception_detail` is load-bearing unit-tested.

**Orchestrator adjudication:** This is a VERIFICATION NOTE asserting correct behavior. The adversary confirmed the generic (else) branch is structurally locked by `test_having_aggregate_detail_generic_uses_ellipsis` (added in fix-burst-38; would fail on revert). No defect; no remediation. Carried in §Positive Verifications below.

---

### F-PQLFN-PR7-OBS-004 — Security probe: sanitize-at-construction + bounded disclosure (VERIFICATION NOTE; adjudicated as positive verification)

**Adversary filing (raw):** CWE-117/200 sanitize-at-construction + bounded disclosure fields VERIFIED.

**Orchestrator adjudication:** This is a VERIFICATION NOTE asserting correct behavior. The adversary confirmed CWE-117 sanitize-at-construction (infusion + column paths) and CWE-200 server-controlled disclosure fields only — both pass. No defect; no remediation. Carried in §Positive Verifications below.

---

## Out-of-Scope Deferred Item (carried from LOCAL cascade and prior PR-LEVEL passes; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1+2+3+4+5+6; not re-raised as a PR-LEVEL finding.

---

## Fix-burst-39 Trail (disclosure-only; HEAD unchanged)

- **Author:** pr-manager
- **Nature:** Disclosure-only; no code change, no spec change, no new tests, no branch commit.
- **PR #223 body edit:** Appended `### Merge-Gate Disclosure — LOW-006 DataFusion-name collision cost (F-PQLFN-PR7-OBS-001)` to the `## MERGE-GATE DISCLOSURES` section of PR #223 body. Content: collision description, not-a-regression rationale, fail-safe characterization, HUMAN FEATURE-DECISION request (keep 21-keyword list as-is recommended; or open follow-up story).
- **PR #223 body checklist:** Added row `- [ ] MERGE-GATE FEATURE-DECISION: LOW-006 keyword list adjudication — keep 21-keyword list as-is or open follow-up story (F-PQLFN-PR7-OBS-001)` to the pre-merge checklist.
- **pr-223-created.md:** Same disclosure content appended to pr-223-created.md (source-of-record for PR lifecycle events).
- **HEAD verification:** `97cb070e` verified UNCHANGED after all edits. Body edits touch GitHub PR description, not git commits. DRIFT-ORCH-PRLEVEL-PUSH-001 NOT triggered; streak counting continues on frozen HEAD 97cb070e.

---

## Positive Verifications

- **F-PQLFN-PR7-OBS-002 (reclassified from §Observations): Walker recursion completeness VERIFIED.** All Predicate/Expr recursive positions are visited by the predicate-walker (engine.rs; 7-position walker load-bearing). The sole gap is InSubquery — adjudicated §OBS-001 of the local cascade (D-PQLFN-P47-OBS-001; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch). No new walker gaps introduced in fix-burst-38 or fix-burst-39.

- **F-PQLFN-PR7-OBS-003 (reclassified from §Observations): Generic branch IS load-bearing unit-tested.** `having_aggregate_interception_detail` else branch (signature-neutral `(...)` template) locked by `test_having_aggregate_detail_generic_uses_ellipsis` (fix-burst-38; structurally load-bearing — revert causes test failure). Two-branch structure verified complete: percentile path and generic path each have a dedicated unit test. No coverage gap.

- **F-PQLFN-PR7-OBS-004 (reclassified from §Observations): Security — sanitize-at-construction + bounded disclosure VERIFIED.**
  - CWE-117 (Log Injection): `sanitize_for_log` applied at construction in infusion path and column-not-found path; bidi override chars are sanitized before reaching log output.
  - CWE-200 (Sensitive Information Exposure): `ColumnNotFoundDetails` Display fields are server-controlled only (require admin TOML access to configure); no user-supplied secret-class data in disclosure fields.
  - Both paths verified as per security review APPROVE on 973aedcf (0 CRIT/HIGH/MED; SEC-001 + SEC-002 carried as LOW non-blocking; no change in fix-burst-38 or fix-burst-39).

- **Regression probe — NO regressions.** No previously-valid query class flips to rejected at frozen HEAD 97cb070e. All newly-gated classes (fn-call LHS predicates with keyword names) previously failed with generic parse errors; post-branch they receive structured E-QUERY-001 rejection messages. The E-QUERY-001 message is a strict improvement over the prior generic error. No query that previously succeeded now fails.

- **Security probes — ALL PASS at 97cb070e:**
  - CWE-117 (Log Injection): `sanitize_for_log` at construction in infusion path (`check_enrich_udf_availability`) and column-not-found path (F-PQLFN-PR7-OBS-004 above). PASS.
  - CWE-200 (Sensitive Information Exposure): Server-controlled disclosure fields only in `ColumnNotFoundDetails`; no path for user input to populate disclosure-sensitive fields. PASS.
  - CWE-407 (Algorithmic Complexity — ReDoS/BDoS): `cap_name_for_levenshtein` 128B pre-Levenshtein cap enforced before any fuzzy-match computation. PASS.
  - DoS (parse-time): `check_query_size` (64KB gate) + `check_paren_depth` (depth-64 gate) bound parse-time cost. PASS.
  - CWE-89 (SQL Injection): `pipe_sql_emitter` `is_safe` charset gate prevents injection of unescaped SQL through the PIPE→SQL translation path. PASS.

- **Least-attention perimeter file probe — PASS.** Non-obvious perimeter files (error taxonomy, policies.yaml, non-exhaustive gate, BC-2.11.004 pin sites) verified: policies.yaml v1.34 correct (POL-34 registered); non-exhaustive EXPECTED=91 unchanged; BC-2.11.004 v1.48 at all 3 live BC cite sites; error-taxonomy v2.52 E-QUERY-001 template unchanged; ADR-048 v1.16 live.

- **LOW-006 error propagation verified across all four parse modes.** E-QUERY-001 rejection (via the 21-keyword gate) fires correctly and propagates cleanly through SQL, SqlPipe-head, Pipe, and Pipe-head execution paths. No mode-specific propagation gap.

- **SAP-1 PASS:** Zero net-new `event_type =` emissions in fix-burst-39 (disclosure-only; no code changes). Settled methodology from passes 5/6 carries: 55 raw occurrences / 12 distinct values at 97cb070e verified against BC-2.16.002 v1.61 catalog 92 rows; ZERO net-new emissions.

- **POL-22 Phase A+C PASS:**
  - Phase A: all positive verifications derived independently by adversary (no reliance on implementer disclosure).
  - Phase C: all claims cross-checked against code at 97cb070e; no reliance on pass reports alone.
  - `having_aggregate_interception_detail` present at 97cb070e (grep confirmed; two-branch structure verified).
  - `debug_assert_eq!` ABSENT in HAVING-interception loop at 97cb070e (rg returns ZERO results in loop context).
  - Disclosure edits (fix-burst-39) are GitHub PR body edits only; no code change at 97cb070e.

- **TD-VSDD-059 PASS:** No new code closures in fix-burst-39 (disclosure-only). Prior closures from fix-burst-38 carry load-bearing tests (verified at 97cb070e per pass-6 confirmation; HEAD unchanged).

- **TD-VSDD-060 PASS:** No signature, constant, or canonical identifier changes in fix-burst-39 (disclosure-only).

- **TD-VSDD-091 PASS:** No new volatile line-pins introduced. Disclosure prose (pr-223-created.md, PR body) uses behavioral anchors, not `file.rs:NNN` line numbers.

- **POL-34 PASS:** `fail_loud_invariant_enforcement_standard` registered policies.yaml v1.34; two-branch detail-builder exemplar satisfies policy. No changes in fix-burst-39.

- **No secrets, no AI attribution, no `--no-verify` markers in diff (fix-burst-39 is a PR body edit; no git diff).**

- **No new `unwrap()` / `expect()` in production code paths** (fix-burst-39 is disclosure-only).

- **No `reqwest` changes** — ADR-050 rustls-tls constraint untouched.

- **Spec versions verified at 97cb070e:**
  - BC-2.11.019 v1.23 (two-branch detail-builder; debug_assert REMOVED; DML scope cross-note; §OBS-004 updated)
  - BC-2.11.004 v1.48 (EC-11-085/086/087; RESERVED_KEYWORDS 21 keywords; 3 live BC sites)
  - ADR-048 v1.16 (§D.2 rewritten; §D.7.3 HAVING-exemption caveat; 13 live pins)
  - error-taxonomy E-QUERY-001/E-QUERY-039 templates current (v2.52)
  - policies.yaml v1.34 (POL-34 registered)

- **Novelty: LOW** — convergence signal. Three of four adversary observations were verification probes asserting correct behavior. One substantive finding (F-PQLFN-PR7-OBS-001) is a documented scope limit with clear disclosure, not a code defect. Remaining cascade is convergence-only (streak 0/3 after reset; three CLEAN(strict) passes needed on unchanged frozen 97cb070e).

---

## Convergence Status

- CLEAN(strict): NO — 4 OBS reported; BC-5.39.001 strict criterion requires ZERO findings of ANY severity; streak RESETS 1/3 → 0/3
- CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings; PR #223 is merge-eligible pending human merge-gate decisions (DRIFT-PQLFN-OD7 ratification + LOW-006 adjudication + BC-2.11.019 sequencing confirmation)
- Streak: **0/3** (reset from 1/3; BC-5.39.001 strict criterion applied; DRIFT-ORCH-PRLEVEL-PUSH-001 clean — HEAD verified 97cb070e local=origin=PR before and after; no pushes since fix-burst-38)
- Frozen HEAD: **97cb070e** (PR #223 HEAD; UNCHANGED; fix-burst-39 is PR-body-only; no branch commits)
- DRIFT-ORCH-PRLEVEL-PUSH-001: NO commits pushed since fix-burst-38; streak counting continues on frozen 97cb070e; passes taken on prior frozen HEADs (973aedcf, 76c0fa60, 72d8ed8d) do NOT count toward the 97cb070e streak

---

## Next Step

PR-LEVEL pass-8 on SAME frozen HEAD 97cb070e (streak 0/3 → target 1/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; NO pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 97cb070e → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + LOW-006 keyword-list adjudication + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge).

**Meta-instruction for future passes (pass-8 and beyond):** Verification notes asserting correct behavior belong in §Positive Verifications, NOT §Observations. A finding in §Observations requires BOTH a defect statement (what is wrong) AND a failure scenario (what breaks and how). Probes that conclude "VERIFIED — no defect" are positive verifications; filing them as OBS inflates the finding count, resets the strict streak, and obscures the true convergence signal. Future adversary passes on this cascade should apply this discipline.
