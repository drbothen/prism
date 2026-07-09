---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [13]
feature_head_at_review: 09ea9979
date: 2026-07-09
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  low: 0
  obs: 0
code_behavior_defects: 0
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 13 — FIX-IEQ-ERRPATH-001

---

## Pass 13 (frozen 09ea9979; fresh-context adversary; diversified angles; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK ADVANCES: 0/3 → 1/3** (BC-5.39.001). No fix-burst required.

**Code HEAD at review:** 09ea9979 (frozen; D-1623 fix-burst: EC-11-070/071 star-with-join suspension — BC-2.11.016 v1.18 STAR-WITH-JOIN SUSPENSION RULE + doc-count fix; 5371/5371 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**LOCAL 3-CLEAN(strict) streak after pass-13:** 1/3 (BC-5.39.001). NEXT: LOCAL adversary pass 14 on UNCHANGED HEAD 09ea9979 (fresh context, strict; no commits between passes per DRIFT-ORCH-PRLEVEL-PUSH-001). If CLEAN, pass 15 completes 3-CLEAN → push branch → open fix-PR via pr-manager.

---

## Finding Survey — ZERO findings

Fresh-context adversary performed a full-scope review of frozen HEAD 09ea9979 with diversified angles targeting the 14 BC-2.11.016 v1.18 positions, FP-001 probes, standing probes (SAP-1, POL-24), and production discipline checks. No defects were found.

Three candidate observations were evaluated and discarded as NON-defects; they are documented below.

---

## Considered-and-Discarded Nits (NON-defects)

### NON-DEFECT 1 — Weak fail-open-consistent assertions in EC-11-070/071 tests

**Description:** The two tests added at @09ea9979 for EC-11-070 (shape A: `SELECT j.* FROM t JOIN u j ON ... | where u_col`) and EC-11-071 (shape B: `SELECT * FROM t JOIN ... | where u_col`) assert that the result is NOT an E-QUERY-038 ColumnNotFound error. The adversary considered whether these assertions could be strengthened to assert the specific success shape (e.g., a successful plan output rather than merely absence of E-QUERY-038).

**Disposition:** NON-defect. The BC-2.11.016 v1.18 STAR-WITH-JOIN SUSPENSION RULE mandates fail-open behavior for this shape: the suspended:=true path allows downstream pipe stages to run without column gating. Asserting "not E-QUERY-038" is precisely the contract claim — it tests the fail-open obligation, not a specific success shape. A stricter assertion would over-constrain the implementation and could create false regression signals when the full multi-source union is implemented as a permissible future strengthening (per BC-2.11.016 §Future-strengthening note). The assertions are load-bearing for the FP-001 obligation and are correctly scoped.

### NON-DEFECT 2 — Conservative TableStar-of-FROM treatment without JOIN

**Description:** The adversary probed whether shape `SELECT * FROM t` (bare TableStar, no JOIN) correctly reaches the EC-11-041 is_registered() gate when a downstream pipe stage references a column absent from t's registered schema. The branches (a) and (c) only set suspended:=true when the JOIN list is non-empty. A bare-star head with no JOIN and a missing column reference should still fire E-QUERY-038 via the EC-11-041 gate.

**Disposition:** NON-defect. This behavior is exactly as specified by BC-2.11.016 v1.18: the STAR-WITH-JOIN SUSPENSION RULE activates only when `JOIN list is non-empty`. Without a JOIN, the EC-11-041 is_registered() disambiguation path applies: if the table is registered and the column is absent, E-QUERY-038 fires; if the table is unregistered, fail-open applies. This conservative treatment is ratified by BC-2.11.016 §Invariants. The adversary verified the gate activates correctly at both EC-11-041 code sites.

### NON-DEFECT 3 — Cosmetic COVERAGE_MATRIX description strings used only for count

**Description:** The adversary observed that the COVERAGE_MATRIX comments in the test module describe each EC entry using brief shorthand strings (e.g., "star+join+join-source-only-col", "bare-star+join+join-source-only-col") that do not fully describe the shape. These description strings are used only for count/inventory purposes and do not drive test logic.

**Disposition:** NON-defect. The description strings are documentation-only. The actual test assertions are the contract-enforcing artifacts. The shorthand descriptions are consistent with the established convention in EC-11-039..071; no other EC entry uses verbose shape descriptions. Changing these would be cosmetic churn with no behavioral improvement.

---

## Full Verification Trace — BC-2.11.016 v1.18 Clause Coverage

### 14-Position Catalog (EC-11-039 through EC-11-071 subset relevant to this pass)

The adversary traced each of the 14 active gate positions in the SqlPipe head-binding path against the implementation at @09ea9979:

| Position | EC Entry | Shape | Branch | Result |
|----------|----------|-------|--------|--------|
| 1 | EC-11-039 | bare Field in head → direct name seeding | N/A | PASS |
| 2 | EC-11-040 | alias AS binding seeded via alias name | N/A | PASS |
| 3 | EC-11-041 | unregistered table → is_registered() fail-open gate | N/A | PASS (both sites verified) |
| 4 | EC-11-042..058 | agg-arg suspension; anonymous non-Field | (a)/(c) | PASS |
| 5 | EC-11-059 | head alias seeded DERIVED; downstream resolves | (b) | PASS |
| 6 | EC-11-060 | stats REPLACE semantics (prior names cleared, new binding) | N/A | PASS |
| 7 | EC-11-061 | anonymous agg in head → suspended; no DERIVED seeding | (a) | PASS |
| 8 | EC-11-062 | MIXED-STAR: star present + Field present → branch (c) additive | (c) | PASS |
| 9 | EC-11-063 | MIXED-STAR: star + agg alias → suspended; DERIVED seeded for alias | (c) | PASS |
| 10 | EC-11-064 | MIXED-STAR: star + Field in GROUP BY → branch (c) refs seeded | (c) | PASS |
| 11 | EC-11-069 | join-qualified last-segment seeding via LAST-SEGMENT OUTPUT-NAME RULE | (b) | PASS |
| 12 | EC-11-070 | star/TableStar + JOIN non-empty → branch (a) returns Some((vec![],{},true)) | (a) | PASS |
| 13 | EC-11-071 | MIXED-STAR + JOIN non-empty → branch (c) sets suspended:=true additively | (c) | PASS |
| 14 | Enrich/Fields UNION/TRANSITION | enrich UNION semantics; FIELDS TRANSITION semantics | N/A | PASS |

All 14 positions PASS at frozen @09ea9979.

### Branch Logic Verification

**Branch (a) — Star/TableStar only:**
- Without JOIN: returns the full schema seed via is_registered() / fail-open (EC-11-041). PASS.
- With JOIN (EC-11-070): returns `Some((vec![], HashMap::new(), true))` — empty seed, no bindings, suspended=true. PASS.

**Branch (b) — Non-star Field items:**
- EC-11-069 LAST-SEGMENT OUTPUT-NAME RULE: last segment of qualified path (e.g., `j.col` → `col`) seeded with DERIVED provenance. Branch (b) unchanged at @09ea9979 (no modification from D-1621). PASS.

**Branch (c) — MIXED-STAR (star + other items):**
- Without JOIN: additive seeding of non-star items (Field names, aliases). PASS.
- With JOIN (EC-11-071): additive suspended:=true applied alongside existing DERIVED-seeding logic. PASS.

### FP-001 Extended Probe Results

| Probe | Description | Result |
|-------|-------------|--------|
| FP-001-A | Star+JOIN alias qualifier → fail-open on join-source-only column | PASS (suspended:=true; fail-open; EC-11-070) |
| FP-001-B | Bare SELECT * with JOIN → fail-open on join-source-only column | PASS (suspended:=true; fail-open; EC-11-070) |
| FP-001-C | Star WITHOUT JOIN → E-QUERY-038 on truly absent column (registered table) | PASS (suspension NOT triggered; EC-11-041 gate active) |
| FP-001-D | EC-11-069 LAST-SEGMENT still seeds DERIVED when star NOT present | PASS (branch (b) unchanged; DERIVED seeded) |
| FP-001-E | MIXED-STAR with JOIN → fail-open via branch (c) additive suspended:=true | PASS (EC-11-071) |
| FP-001-F | Shadow alias (SELECT count(*) AS severity ... \| where severity > 5) | PASS (SIBLING-GATE CONSISTENCY per-name RAW/DERIVED provenance; DERIVED alias resolves; no false E-QUERY-002) |

### DERIVED Provenance State Machine

- **stats REPLACE semantics (EC-11-060):** Prior binding names cleared; new binding context derived from stats output columns. PASS.
- **enrich UNION semantics:** Enrich stage unions the new enrichment output columns with the prior binding context. PASS.
- **FIELDS TRANSITION semantics:** FIELDS pipe stage replaces binding context with the declared fields. PASS.
- **SIBLING-GATE CONSISTENCY:** Per-column provenance (RAW vs DERIVED) correctly skips the sibling-gate for DERIVED-provenance columns. PASS.

### FROM-ALIAS Threading

The `from_alias` field (if present in the head SELECT's FROM clause) is threaded through both callers of `compute_sqlpipe_head_binding`. Adversary verified both call sites at @09ea9979 pass `from_alias` correctly and consume the result consistently. PASS.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace. Three `column_not_found.rejected` emission sites established at D-1618 pass-7 are present and cataloged. Two `reload.*` sites from the existing module are cataloged. No new `event_type` assignments introduced at @09ea9979 (D-1623 fix-burst touched only branch logic in `compute_sqlpipe_head_binding` and one doc-comment). Five total catalog rows, zero gaps.

**POL-24 (byte-verbatim EC-body):** PASS — EC-11-070 and EC-11-071 added to BC-2.11.016 v1.18 at D-1623 carry full field schema, audit role, and recurrence policy in byte-parity with the canonical EC-11-039..069 body format. Pass-12 verified byte-parity; pass-13 confirms no subsequent drift (no code changes between e5170899 and 09ea9979 touch BC-2.11.016 directly; @09ea9979 is the fix-burst commit).

**TD-VSDD-060 (sibling-site sweep):** PASS — No function signatures, constants, or canonical identifiers changed at @09ea9979. The `compute_sqlpipe_head_binding` signature is unchanged. No sibling-site sweep required.

**TD-VSDD-091 (no volatile line-pin citations):** PASS — No new `file.rs:NNN` line-number citations introduced in any diff at @09ea9979. All behavioral anchors use EC-NN-NNN identifiers and function names.

---

## Production Discipline Checks

**`unwrap()` / `expect()` in non-test code:** PASS — @09ea9979 introduces no new `unwrap()` or `expect()` calls in production code. The `NonZeroUsize::new(N).unwrap()` const-eval exception is unchanged.

**`println!` in production code:** PASS — no `println!` calls in production code at @09ea9979.

**`#[non_exhaustive]` preserved:** PASS — no new public types introduced. Non-exhaustive gate EXPECTED=89 unchanged. All 89 registered types intact.

**Story pins current after fix-burst:** PASS — 4-story pin round complete at @09ea9979 (D-1623): S-PRISMQL-CASE-INSENSITIVE-001 v1.47, S-DEMO-FIDELITY-REMEDIATION-001 v2.36, S-DEMO-PRISMQL-ONBOARDING-001-B v2.13, S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.22. BC-2.11.016 v1.18 propagated to all 4 carriers. Pins unchanged at pass-13 (no new code; no spec evolution needed).

**Sibling BCs propagated:** PASS — BC-2.11.017 v1.6, BC-2.11.020 v1.11, BC-2.11.004 v1.23, error-taxonomy v2.31. All confirmed in sync with BC-2.11.016 v1.18. BC-INDEX v7.70. STORY-INDEX v2.642.

---

## Convergence Assessment

**Trajectory:** 6 → 3 → 3 → 2 → 1 → 1 → 0 → 2(low) → 0

**Pattern:** Pass 13 finds ZERO findings on a frozen HEAD that had 2 LOW findings at pass 12. This is a return to CLEAN after the low-severity regression. The trajectory pattern is consistent with genuine convergence: the behavioral surface is stable, the star+JOIN suspension gap (the novel angle found at pass 12) has been structurally resolved, and the adversary could not construct any FP-001 trigger at @09ea9979.

The three considered-and-discarded nits confirm that the adversary applied diversified angles (assertion coverage strength, edge-case shape permutations, documentation accuracy) and found no exploitable defects. Novelty LOW: no novel angles remain from the 13-pass cumulative probe set.

**Novelty assessment:** LOW — The behavioral surface has been covered by EC-11-039..071 (33 entries). The star+JOIN gap (the highest-novelty angle) was resolved at D-1623. Pass 13 confirms the fix holds under fresh-context adversarial review with diversified angles. Remaining uncertainty is minimal.

**Streak status:** 1/3 (advancing). NEXT: LOCAL adversary pass 14 on FROZEN HEAD 09ea9979 (fresh context, strict; no commits between passes per DRIFT-ORCH-PRLEVEL-PUSH-001). If CLEAN(strict), pass 15 completes BC-5.39.001 3-CLEAN → push branch → open fix-PR via pr-manager.
