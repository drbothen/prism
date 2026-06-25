---
document_type: adversarial-review-pass
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
cascade: LOCAL
pass: 2
frozen_head: f03679b2
diff_range: "903c8fcb..f03679b2"
reviewer_perimeter: "per-story (story diff 903c8fcb..f03679b2 + 8 BCs + ADR-043/044/045/046 + error-taxonomy)"
verdict_strict: "CLEAN(strict)=NO"
verdict_pr_merge: "CLEAN(PR-merge)=NO"
pass_outcome: "NOT CLEAN — 1 MED; CLOSED by story-writer (story v1.3→v1.4 BC version-pin correction)"
post_pass_head: f03679b2
streak_after: "0/3 on f03679b2 (story-only fix; code HEAD UNCHANGED; streak preserved per frozen-HEAD rule — code did not change; re-pass on corrected story v1.4 expected CLEAN)"
timestamp: 2026-06-25T10:00:00Z
---

# LOCAL Adversary Pass 2 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Story:** S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
**Cascade:** LOCAL
**Pass:** 2 (on HEAD f03679b2; post-D-1338 fix-burst; reading story v1.3)
**Frozen HEAD reviewed:** `f03679b2`
**Diff perimeter:** story diff `903c8fcb..f03679b2` + 8 BCs (BC-2.11.020/021/022/023, BC-2.10.015/016/017, BC-2.11.002) + ADR-043/044/045/046 + error-taxonomy
**Story version at time of pass:** v1.3

## Verdict

- **CLEAN(strict):** NO
- **CLEAN(PR-merge):** NO
- **Finding count:** 1 MED = 1 total
- **All findings:** CLOSED (story-writer v1.3→v1.4 BC version-pin correction)

## Findings

### MED-1 — POL-23 Sibling-Sweep Gap: Stale BC Version Pins in Story Behavioral Contracts Table

**Severity:** MED
**Policy:** POL-23 (BC version pins in story docs must match current BC frontmatter versions)
**Impact:** Doc-only. No code or runtime impact. The story's Behavioral Contracts table cited BC-2.11.023 at v1.1 (now v1.2 after PO corrected §D1 ParseErrorDetails residual in D-1338 burst) and BC-2.10.015 at v1.0 (now v1.1 after PO corrected OrgRegistry::contains non-existent reference in D-1338 burst). These pins were correct at story v1.3 authoring time but became stale when PO applied BC corrections in the D-1338 factory burst — the story-doc sibling-sweep obligation (POL-23 + TD-VSDD-060) was not applied to the story file in that burst.

**Root cause:** D-1338 burst updated BC-2.11.023 (v1.1→v1.2) and BC-2.10.015 (v1.0→v1.1) via PO BC corrections but did not sweep the story's Behavioral Contracts table for live version-pin cells referencing those BCs.

**Status:** CLOSED — story-writer updated story v1.3→v1.4: BC-2.11.023 cell corrected `v1.1 → v1.2`, BC-2.10.015 cell corrected `v1.0 → v1.1`. §Changelog row v1.4 added. POL-25 sweep confirmed no other live-narrative pins of these BCs at old versions exist outside §Changelog historical rows (TD-VSDD-091-exempt). No AC/scope/code change.

## Positives (PASS — do NOT re-flag)

The following items were verified PASS in Pass 1 (D-1338) and remain PASS on the UNCHANGED code HEAD f03679b2. Re-verification not required unless code changes; listed for completeness and streak continuity.

### Implementation Verified Items

1. **Filter-mode tests load-bearing (SeverityStubAdapter):** `SeverityStubAdapter` pre-seeds 2 HIGH + 3 LOW rows; exact row-count assertions + negative controls present; BC-2.11.023 §D4/AC-011 "verifies rows returned" MET. F-PASS1-HIGH-1 CLOSED f03679b2.

2. **Temporal NOW()/INTERVAL wired in production pipeline:** `inject_now` Step 1a constant-fold wired; plan-pinned plain-string `'<iso>'` per D-1333 Option A; discriminating in/out-of-window temporal tests present with negative controls. BC-2.11.021/ADR-044 SATISFIED.

3. **FORBID-BOTH/E-QUERY-040 fires on 0-row (hoisted Step 1b before fan-out):** FORBID-BOTH gate hoisted before fan-out; fires even when 0 rows returned; data-independence confirmed (EmptyVecAdapter). BC-2.11.020 SATISFIED.

4. **PqlNormalizer emits DataFusion-executable SQL:** `normalize_expr` explicit temporal arms for `Expr::Now/Interval/TimestampArithmetic`; `'_ => String::new()` catch-all removed; structured `E-QUERY` errors for unhandled arms. BC-2.11.021 SATISFIED.

5. **E-QUERY-040 + E-QUERY-036 Display verbatim per error-taxonomy (POL-24):** `#[error]` templates verbatim from error-taxonomy.md pedagogical template; tests assert full substring. F-PASS1-MED-1 CLOSED 8f6bb337.

6. **`mode_bridge_normalized_pql` + `find_first_unquoted_pipe` relocated to `prism-query/error_recovery.rs`:** Relocated from `prism-mcp/error_mapping.rs`; prism-mcp calls via public path; BC-2.11.023 anchors + story File Structure mandate satisfied. OBS-1 CLOSED 616864d0.

7. **SAP-1 PASS:** No new `event_type` values in the diff. All existing `event_type` values in `crates/` remain registered in BC-2.16.002 §Postconditions Canonical Structured Event Catalog.

8. **SAP-2 N/A:** Story does not touch `.prism/specs/sensors/*.toml` or DTU clone routes.

9. **SID-1 PASS:** No `#[ignore]`'d integration test defers a behavior without a unit-test substitute.

10. **Non-exhaustive gate 87/87:** `just check` GREEN on f03679b2. ci.yml EXPECTED=87 at worktree HEAD (reflects S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 additions; not yet reflected in develop EXPECTED=84).

11. **OBS positives (retained from Pass 1):** `Ast::Sql`/`SqlPipe` arms `ok_or_else` structured-error hardening; `create_schedule` scope validation is non-blocking in-memory check (correct per BC semantics).

## Just-Check Evidence

- `prism-query`: 1117/1117 PASS
- clippy: PASS (0 warnings with -D warnings)
- fmt: PASS
- non-exhaustive: 87/87
- Full-workspace DTU harness: 0 failures through 1563/4915 at time of pass; full-suite EXIT pending (all changed crates GREEN)

## Cascade Status After This Pass

- **CLEAN(strict)=NO** (1 MED-1 story-doc drift)
- **CLOSED:** MED-1 story-writer v1.3→v1.4
- **Code HEAD:** f03679b2 UNCHANGED (no code change in this pass)
- **3-CLEAN streak:** 0/3 on f03679b2 (story-only correction; frozen-HEAD rule: code HEAD unchanged; re-pass on corrected story v1.4 expected CLEAN — adversary novelty LOW; all 7 verify items + both probes PASS; this pass found only doc-drift introduced by the D-1338 BC-fix sibling-sweep gap)
- **NEXT:** LOCAL adversary Pass 3 re-run on f03679b2 reading corrected story v1.4 — expect CLEAN(strict)=YES
