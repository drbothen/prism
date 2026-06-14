---
document_type: pr-adversary-pass-report
pass: 4
story: S-DEMO-MULTI-TENANT-DTU-001
pr: 187
pr_head_at_review: 41d093fe
feature_branch: feature/S-DEMO-MULTI-TENANT-DTU-001
base_branch: develop
develop_head_at_review: f7400f83
clean_strict: "NO"
clean_pr_merge: "NO"
streak_before: "0/3"
streak_after: "0/3 (1 MED F-PR4-MED-001 citation-symbol fix + 1 LOW OBS-PR4-1 volatile-SHA fix; BC v1.9→v1.10, story v1.13→v1.14)"
date: 2026-06-14
producer: adversary (D-1156)
---

# PR-LEVEL Adversary Pass 4 — S-DEMO-MULTI-TENANT-DTU-001 PR #187

## Summary

**CLEAN(strict): NO** (1 MED F-PR4-MED-001 + 1 LOW OBS-PR4-1 found; both CLOSED in-scope)
**CLEAN(PR-merge): NO** (MED finding present at start of pass)

Pass 4 has two parts. Part A verifies F-PR3-HIGH-001 closure from Pass 3 (new
prism-sensors integration test + BC v1.9 + story v1.13). Part B conducts a fresh-context
adversarial review of the complete PR diff at HEAD 41d093fe and finds 1 MED + 1 LOW.

Both findings are documentation/citation-accuracy in nature — no code or structural
change required. All axes clean except the 2 citation findings.

---

## Part A — F-PR3-HIGH-001 Closure Verification

**F-PR3-HIGH-001 (AC-006 distinct-listener tautology / spec-vs-perimeter contradiction):**

Verification of the combined fix at 41d093fe:

1. **New test load-bearing check:** `crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs::test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` — drives the REAL `fan_out_with_overlay_map` from `prism-sensors/src/fanout.rs` (NOT a harness-internal proxy). Two live `ArmisClone` instances (acme→S_A port, contoso→S_B port). Assertions: acme query → S_A delta = 6 requests, S_B delta = 0 (zero-leak); contoso query → S_B delta = 6, S_A delta = 0. This is a load-bearing multi-tenant routing isolation assertion, NOT a TCP tautology (test exercises FanOutTarget dispatch through overlay_map wiring).

2. **No dep cycle:** `prism-sensors/Cargo.toml [dev-dependencies]` += `prism-dtu-harness + prism-dtu-armis + prism-dtu-common`. Dev-dependency direction `prism-sensors→prism-dtu-*` is PERMITTED (INV-PERIMETER-001 forbids the REVERSE direction: prism-dtu-harness importing prism-sensors). No perimeter violation.

3. **BC v1.9 coherence with story v1.13:** BC-2.06.017 Postcondition 4 narrows to DISTINCT-LISTENER isolation scope with cross-reference to the real routing proofs. Story AC-006 and Story-Level-Goal narrowed to match. Architecture Mapping + File Structure rows for the new integration test present in story v1.13. The narrowing is not a product-value reduction — FanOutTarget routing is now proven end-to-end via `test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` (what the spec always intended).

4. **`just check` GREEN at 41d093fe** (reported by implementer; `cargo nextest run -p prism-sensors` includes the new integration test PASSING).

**F-PR3-HIGH-001: CONFIRMED CLOSED.** The fix is genuine, load-bearing, and not a paper-fix.

---

## Part B — Full Fresh-Context Adversarial Review

### Review scope

Full PR diff at HEAD 41d093fe. All commits in the PR at this pass:
- 9b4f4154 — initial TDD implementation (MultiInstanceServers, isolation counter, tests)
- a27b0f54 — Pass-6 Drop-only doc fix (MultiInstanceHarness)
- 96fce1ad — tls-removal no-default-features E0053 fix
- 74d0bd4c — SEC-001/SEC-002 CLOSED (validate_harness_key input validation + 6 tests)
- 89764cda — brotli pin fixes (repo-wide, not story-specific)
- 846c21dc — SEC-006 CLOSED (CWE-209 redaction) + Pass-1/Pass-2 changelog reorder
- 41d093fe — F-PR3-HIGH-001 fix (BC v1.9 + prism-sensors integration test + story v1.13)

### F-PR4-MED-001 [MED] Harness-Test Citations in BC + Story Lack the `test_BC_2_06_017_` Infix — Did Not Grep-Resolve

**Severity:** MED
**Status:** CLOSED (product-owner BC v1.9→v1.10; story-writer story v1.13→v1.14; citation-accuracy only, no semantic change)

**Finding:**

EC-017-007 in `BC-2.06.017` v1.9 §Edge Cases reads:

> Test misconfiguration: org A overlay points to instance B socket. All of org A's requests go to instance B. The leakage test `multi_tenant_routing_zero_cross_tenant_leakage` correctly FAILS — detecting the misconfiguration.

The function name cited is `multi_tenant_routing_zero_cross_tenant_leakage`. The ACTUAL function name in the codebase (as of 9b4f4154 / just check GREEN baseline) is `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage`. The `test_BC_2_06_017_` infix is part of the function name; it is load-bearing for grep-traceability (SAP-1 / CLAUDE.md §BC naming discipline requires the BC-ID infix in test function names so that `grep test_BC_2_06_017_ crates/` resolves the test to its BC anchor).

The same gap exists in the VP catalog entry for BC-2.06.017 §Verification Properties, which references:

> `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage` — integration test...

Wait — re-reading the VP catalog at v1.9: the first VP entry DOES use the full function name `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage` in the description block. However, EC-017-007 in the §Edge Cases table column "Expected Behavior" uses the truncated form `multi_tenant_routing_zero_cross_tenant_leakage`. That's the citation mismatch.

**Grep-resolve check (adversary):**

A grep of the BC file for `multi_tenant_routing_zero_cross_tenant_leakage` reveals the truncated form in EC-017-007. The full function name `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage` is used in the VP catalog. The truncated cite in EC-017-007 does not grep-resolve to the actual test function.

**Story-side sweep (adversary):**

A corresponding sweep of story v1.13 (S-DEMO-MULTI-TENANT-DTU-001) for all harness-test citations finds 5+ sites referencing test names WITHOUT the `test_BC_2_06_017_` infix in contexts where the full infix is required for grep-traceability:

- AC inline descriptions (multiple AC rows referencing the leakage test)
- Red Gate Test Plan table rows (15 rows total; subset reference the test functions)
- Task-7 implementation description citations

In particular, AC-006 language and the §Red Gate Test Plan AC-006 rows cite `multi_tenant_routing_zero_cross_tenant_leakage` (short form) where the actual function in `crates/prism-dtu-harness/tests/` is `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage`.

Note: the Architecture-Mapping / File Structure sections do NOT use bare test function names; they reference file paths. Those are citation-accurate. The fix is scoped to inline AC text + RGT table rows that cite the short function name.

**Resolution:**

- **product-owner:** BC-2.06.017 v1.9→v1.10 — corrected all harness-test citations in §Edge Cases EC-017-007 and any VP catalog prose that used the truncated form. Added `test_BC_2_06_017_` infix so cites grep-resolve. Citation-accuracy only; no semantic change to any postcondition, invariant, or EC meaning.
- **story-writer:** Story v1.13→v1.14 — comprehensive sweep of ALL AC inline text + §Red Gate Test Plan rows (all 15 rows verified) + Task-7 citations. `grep -n "multi_tenant_routing_zero_cross_tenant_leakage"` applied across full story file; all occurrences confirmed to carry `test_BC_2_06_017_` infix after fix. Volatile commit-SHA references (see OBS-PR4-1 below) removed in the same sweep.

**Grep-clean confirmed post-fix:** no bare `multi_tenant_routing_zero_cross_tenant_leakage` without the `test_BC_2_06_017_` prefix remains in BC or story files.

---

### OBS-PR4-1 [LOW] Volatile Commit-SHA 41d093fe Pinned in Story AC / Architecture-Mapping Prose

**Severity:** LOW
**Status:** CLOSED (story-writer story v1.13→v1.14; volatile SHA removed, function-name anchors retained)

**Finding:**

Story v1.13 (S-DEMO-MULTI-TENANT-DTU-001) §Architecture Mapping prose and AC text contain 4 references to the literal commit SHA `41d093fe`, e.g.:

> "...the integration test was added at commit 41d093fe..."

This violates TD-VSDD-091 (anti-volatile-pin): spec/story narrative must cite function names + behavioral anchors, NOT commit SHAs, which decay on subsequent diffs (each rebase or squash will change the SHA). The correct behavioral anchor is the function name `test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` in `crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs`.

Note: historical SHA references in CHANGELOG rows and BC-INDEX/STORY-INDEX version-history notes ARE permitted (those reference past immutable burst SHAs as audit trail per TD-VSDD-053). The violation is limited to story BODY prose / AC / Architecture-Mapping sections, where the SHA was used as a "proof anchor" for a test that should be cited by function name.

**Resolution:**

story-writer removed all 4 volatile `41d093fe` SHA references from story body prose (AC inline text + Architecture Mapping section). Function-name anchors (`test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` in `crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs`) retained as the load-bearing citation. CHANGELOG history row that recorded the F-PR3-HIGH-001 fix is NOT modified (immutable audit trail).

This fix was applied in the same story-writer sweep as F-PR4-MED-001 (single combined story v1.13→v1.14 amendment).

---

## Checklist Sweep

- [x] **SAP-1** (tracing emission catalog): no new `event_type=` emissions in diff at 41d093fe. CLEAN.
- [x] **SAP-2** (DTU↔TOML schema parity): no sensor TOML spec changes in diff. N/A.
- [x] **INV-PERIMETER-001**: `prism-sensors [dev-dependencies]` += `prism-dtu-harness + prism-dtu-armis + prism-dtu-common` — permitted direction (sensors→DTU is NOT the forbidden direction). `prism-dtu-harness` does NOT gain any new `prism-sensors` dependency. CLEAN.
- [x] **Gate EXPECTED=60**: no change to non-exhaustive gate in 41d093fe. UNCHANGED at 60.
- [x] **POL-32 changelog direction**: BC v1.10 at top; story v1.14 at top. Both DESCENDING post-fix. CLEAN.
- [x] **SID-1** (no ignored-test rationalization): the new `test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` in `prism-sensors/tests/` is NOT `#[ignore]`'d. It runs as a standard integration test with two live in-process ArmisClone instances (no external DTU dependency). Load-bearing. CLEAN.
- [x] **unwrap/expect in production paths**: none introduced in 41d093fe. CLEAN.
- [x] **BC v1.10 + story v1.14 consistent**: after fixes, both cite `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage` with full infix; story has no volatile SHA in body prose; BC EC-017-007 grep-resolves. VERIFIED.
- [x] **SEC-001 through SEC-006**: all closed in prior passes; no new security surface in 41d093fe. CLEAN.
- [x] **TD-VSDD-091**: no remaining volatile commit-SHA in story body/AC/Architecture-Mapping after OBS-PR4-1 fix. CLEAN.

---

## Streak Status

PR-LEVEL streak: **0/3** (findings present at pass start; F-PR4-MED-001 MED required fix; streak resets to 0/3 per BC-5.39.001 D-779)

**NEXT:** PR-LEVEL adversary Pass 5. Full fresh-context adversarial pass on complete PR diff at HEAD 41d093fe (BC v1.10 + story v1.14). Expect CLEAN(strict)=YES if citation sweeps are grep-clean. Need 3 consecutive CLEAN(strict) passes for BC-5.39.001 PR-LEVEL convergence.
