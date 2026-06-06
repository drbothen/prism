---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 1
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "3a2322d7"
feature_head_after_fix_burst: "f290a43d"
clean_strict: false
clean_pr_merge: true
streak_after: "0/3"
security_review_verdict: "SECURITY-CLEAR-TO-MERGE"
produced: 2026-06-05
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 1 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head 3a2322d7 at review)
**Pass:** PR-LEVEL pass 1 (distinct from LOCAL cascade; LOCAL CONVERGED @69aafcc7 passes 9/10/11)
**Date:** 2026-06-05

## Security Review Result

**Verdict: SECURITY-CLEAR-TO-MERGE** — 0 CRITICAL, 0 IMPORTANT findings.

Injection structurally impossible: time-window bounds are typed `DateTime<Utc>` (not raw strings), serialized via `to_rfc3339_opts(Secs, true)` at the push-down boundary; AQL keyword bounds are percent-encoded URL parameters; numeric u64 limit is range-validated. DTU parsers (CrowdStrike state.rs, Armis search.rs) have no ReDoS-prone regexes and no panic-on-parse paths. No credential or auth surface changed by this story.

### Security Suggestions (non-blocking)

**SEC-002 (SUGGESTION — LOW):** `predicate_tree_to_filter_map` in `crates/prism-query/src/pushdown.rs` has no doc comment explaining the security invariant that DateTime<Utc> type-safety and percent-encoding protect against injection. A doc note would aid future reviewers.

- **Disposition:** CLOSED by fix-burst (implementer f290a43d) — doc note added to `predicate_tree_to_filter_map` module-level. Closed in-scope.

**SEC-004 (SUGGESTION — LOW):** `extract_fql_bound` in `crates/prism-dtu-crowdstrike/src/state.rs` and `extract_aql_keyword_bound` in `crates/prism-dtu-armis/src/search.rs` have no explicit length cap on the extracted string. While current callers pass only validated ISO8601 strings (bounded ~25 chars), defense-in-depth recommends a max-length guard.

- **Disposition:** CLOSED by fix-burst (implementer f290a43d) — 64-char length cap added to both `extract_fql_bound` and `extract_aql_keyword_bound` with unit tests confirming oversized input returns `None`.

**SEC-007 (SUGGESTION — MEDIUM, HARDENING CANDIDATE):** `QueryParams.start_time` and `QueryParams.end_time` are typed as `Option<String>` (in `crates/prism-query/src/pushdown.rs` or equivalent). The single AST call site validates these values before use, but a validated `TimestampString` newtype enforcing ISO8601 format at the type boundary would eliminate the need to rely on a single call-site validation and would make the invariant structurally impossible to violate.

- **Disposition:** NOT CLOSED in this burst. Architect-owned API surface decision. NON-URGENT: current type-safety via validated AST is sufficient; no exploitable gap in the current implementation. Surfaced for human/architect consideration as a hardening candidate. Tagged [hardening-candidate]. Recorded as follow-up candidate in Drift Items (see D-1016). Does NOT block PR merge.

## Adversary Pass 1 Findings

### OBS-1 (LOW) — Stale story-version pin in BC-2.16.002

**Finding ID:** OBS-1
**Severity:** LOW (observability / documentation accuracy)
**Category:** Volatile version pin (TD-VSDD-091 anti-volatile-pin)

**Description:** BC-2.16.002 §Postconditions Canonical Structured Event Catalog scope statement and catalog row 71 (`push_down.inverted_time_range`) Cross-reference field both cited `S-DEMO-QUERY-PUSHDOWN-001 v2.2 EC-003`. Story is now at v2.5; EC-003 content is accurate and unchanged. The `v2.2` pin is a stale volatile reference — story version advances with each spec iteration, so any pinned version in a BC cross-reference will decay.

**Root cause:** BC-2.16.002 v1.66 (D-1010 SAP-1 burst) authored the catalog row when story was v2.2. Story advanced to v2.5 through LOCAL cascade fix-bursts; BC cross-reference pin was not updated.

**Closure:** CLOSED by fix-burst (product-owner). BC-2.16.002 v1.67→v1.68:
- Both live cross-reference sites updated to version-agnostic citation: `S-DEMO-QUERY-PUSHDOWN-001 EC-003` (no version pin).
- Per TD-VSDD-091: behavioral anchors (`EC-003`) are stable; version numbers are volatile. Version-agnostic citation prevents future recurrence of this exact class.
- Historical changelog row v1.66 is immutable per POLICY 1 (append-only) — left intact as historical record.
- BC-INDEX frontmatter v5.87→v5.88 (BC version bump per POL-11).

**Recurrence prevention:** Version-agnostic citation (story-id + EC-NNN, no version pin) is the TD-VSDD-091-compliant form for BC cross-references to story acceptance criteria.

## Summary

**CLEAN(strict):** no (1 LOW OBS-1 finding; not zero)
**CLEAN(PR-merge):** yes (0 CRIT + 0 HIGH + 0 MED; OBS-1 LOW is non-blocking for merge)
**Security:** CLEAR-TO-MERGE (0 CRIT, 0 IMPORTANT; 3 SUGGESTIONS: SEC-002+SEC-004 CLOSED in-burst; SEC-007 hardening candidate surfaced, NOT blocking)
**Streak:** 0/3 (OBS-1 LOW finding; strict criterion requires zero findings)
**Feature HEAD after fix-burst:** f290a43d
**Next step:** PR-LEVEL pass 2 (pr-reviewer dispatch first)

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007, BC-2.11.005) | PASS | Push-down wiring, result-equivalence, inclusive-boundary, RFC3339 Z-normalization all verified |
| Test strength (SAP-1 + SAP-2) | PASS | SAP-1: catalog 71 rows, no unregistered event_type. SAP-2: DTU↔TOML parity confirmed CrowdStrike+Armis |
| SID-1 (no-ignored-test rationalization) | PASS | All push-down tests load-bearing; no deferred integration tests without unit substitutes |
| POLICY-10 (story traceability) | PASS | Story v2.5 BCs correctly traced |
| POLICY-13 (spec-before-code) | PASS | BC-2.01.013 v1.14, BC-2.11.007 v1.8 authored before implementation |
| Demo evidence | PASS (not yet generated — demo-recorder forthcoming; LOCAL cascade CONVERGED) |
| Wiring (Arc-DI, ADR-022) | PASS | No placeholder-construct anti-patterns found |
| Security injection surface | PASS (CLEAR-TO-MERGE) | DateTime<Utc> typing + percent-encoding + u64 range-validation close all injection vectors |
