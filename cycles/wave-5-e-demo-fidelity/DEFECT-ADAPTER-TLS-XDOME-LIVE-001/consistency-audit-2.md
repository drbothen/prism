---
document_type: consistency-audit
artifact: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
cycle: wave-5-e-demo-fidelity
decision: D-2123
date: 2026-08-13
auditor: architect + product-owner + story-writer (human-directed EXPANDED sweep)
scope: specs + code comment/message strings + test-to-file attributions
axes_checked: 42
findings_total: 9
findings_closed: 9
findings_open: 0
---

# Consistency Audit 2 — DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (EXPANDED)

**Date:** 2026-08-13
**Decision:** D-2123
**Context:** Human-directed EXPANDED coherence audit following D-2122 (LOCAL adversary pass-9 CLEAN(PR-merge)=YES). Scope extended beyond spec-cross-reference to include code comment/message strings and test-to-file attributions. Human chose "targeted sweep, then strict" LOCAL-gate strategy (D-2122 decision).

## Summary

| Axis | Result |
|------|--------|
| Axes checked | 42 |
| Axes CLEAN | 33 |
| Findings (all LOW) | 9 |
| Findings CLOSED | 9 (all — pre-written by architect + product-owner + story-writer) |
| Findings OPEN | 0 |

## Artifact Status

| Artifact | Verdict | Notes |
|----------|---------|-------|
| `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md` | CLOSED 8 findings (DD-1..DD-8) | v1.1→v1.2 |
| `specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md` | CLOSED 1 finding (DD-9) | v1.21→v1.22 |
| `stories/DEFECT-ADAPTER-TLS-XDOME-LIVE-001-live-xdome-https-fails-waf-h1-no-ua.md` | PIN PROPAGATION | v1.9→v1.10 (BC-2.16.014 v1.22; 5 locations) |
| `stories/S-WAVE-A-ENGINE-001-header-scheme-field-rule9-validation-auth-dispatch.md` | PIN PROPAGATION | v3.2→v3.3 (BC-2.16.014 v1.22; 2 locations per POL-23) |
| Canonical story | CLEAN | Verified — no findings |
| `specs/behavioral-contracts/BC-2.16.002` | CLEAN | Verified — no findings |
| `specs/behavioral-contracts/BC-2.08.002` | CLEAN | Verified — no findings |
| `specs/behavioral-contracts/BC-2.01.010` | CLEAN | Verified — no findings |
| `specs/behavioral-contracts/BC-2.01.013` | CLEAN | Verified — no findings |
| `specs/architecture/adr/ADR-050` | CLEAN | Verified at v2.3 — no findings |

## Findings Closed

### DD-1 [LOW] — Design doc RG names did not match delivered test symbols

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** Design doc §8 RG table cited draft test names that diverged from the as-built test symbols delivered in the CODE commits.
**Fix:** v1.1→v1.2 §8 full 12-RG table reconciled to delivered code test symbols.
**Status:** CLOSED

### DD-2 [LOW] — Design doc RG count stale (8 → 12)

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** Design doc §8 header claimed 8 Red Gate tests; story v1.9 carried 12 (RG-001..RG-012 including RG-009/RG-010/RG-011 added in pass-1 burst and RG-012 in pass-2 burst).
**Fix:** v1.2 §8 count reconciled to 12.
**Status:** CLOSED

### DD-3 [LOW] — Design doc sanitize-fn reference stale

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** Design doc cited `sanitize_body_snippet` (draft name); as-built code uses `sanitize_body_snippet_bytes` per BC-2.16.002 v2.18 §T-E01.
**Fix:** v1.2 reference corrected to `sanitize_body_snippet_bytes` (symbol anchor, not line cite).
**Status:** CLOSED

### DD-4 [LOW] — Design doc §D6 infusion scope understated

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** §D6 scope entry for infusion-client UA injection described only `build_http_client_with_timeout §pipeline` but ADR-050 v2.1 §D6 extended scope to include `build_http_client_with_timeout §pipeline` in prism-spec-engine/infusion path (D-2114 OBS-4 closure).
**Fix:** v1.2 §D6 scope corrected to match ADR-050 v2.1 ratified scope.
**Status:** CLOSED

### DD-5 [LOW] — Design doc §9.2 independent-sibling section stale

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** §9.2 referenced S-WAVE-A-ENGINE-001 as "independent sibling" without noting its ADR-050 §D6 dependency discovered in D-2114.
**Fix:** v1.2 §9.2 updated with accurate sibling relationship note.
**Status:** CLOSED

### DD-6 [LOW] — Design doc maintenance-note header absent

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** Design doc lacked a maintenance-note header explaining that the document was authored at design time (before delivery) and §8 RG table was reconciled post-delivery.
**Fix:** v1.2 maintenance-note header added at document top.
**Status:** CLOSED

### DD-7 [LOW] — Design doc adr_amendment frontmatter pin stale

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** `adr_amendment` frontmatter field cited an internal draft reference rather than the ratified `ADR-050 v2.1` pin (as per ARCH-INDEX v2.301 record of D-2114 ADR-050 v2.0→v2.1 ratification).
**Fix:** v1.2 `adr_amendment` pin corrected to `ADR-050 v2.1`.
**Status:** CLOSED

### DD-8 [LOW] — Design doc v1.1 changelog row absent

**Artifact:** `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md`
**Root cause:** The v1.1 changes (DD-1 through DD-7 closure) had no corresponding frontmatter changelog row, violating POL-1 changelog discipline.
**Fix:** v1.2 changelog rows added for v1.1 and v1.2.
**Status:** CLOSED

### DD-9 [LOW] — BC-2.16.014 INV-014-007 delegation description inaccurate

**Artifact:** `specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md`
**Root cause:** INV-014-007 §Invariants body described the `DeclarativeHttpAuthProvider` client delegation as a single-path operation; the as-built code has two delegation paths (internal-construction via `build_http_client_with_custom_timeout` per ADR-050 §D5, and the `new_for_test()` clock-seam path per ADR-054 §D4 Option (b)) with distinct semantics for production vs test context.
**Fix:** v1.21→v1.22 INV-014-007 delegation claim corrected to accurately describe both paths; no behavioral contract semantics changed.
**Status:** CLOSED

## TD-VSDD-097 Three-Dimension Sweep

| Dimension | Verdict |
|-----------|---------|
| Sibling pair | CLEAR — design doc has no spec-sibling artifact; BC-2.16.014 INV-014-007 correction is scoped to this contract only; BC-2.01.014 (n/a) and ADR-054 §D4 remain authoritative and were not contradicted |
| Downstream copy target | CLEAR — design doc §D6/§8 sections are not verbatim copy-sources for any active downstream artifact; BC-2.16.014 INV-014-007 prose is not transcribed into any ADR or story task verbatim |
| Mandate anchor | CLEAR — no new MUST blocks authored; all changes are accuracy corrections to existing prose, not new normative obligations |

## POL-7 / POL-23 / POL-29 Compliance

- **POL-7** (bc_h1_is_title_source_of_truth): all BC references use canonical H1 titles — CLEAN
- **POL-23** (pin propagation): BC-2.16.014 v1.22 propagated to story (5 locations) and S-WAVE-A-ENGINE-001 (2 locations) in same burst — CLEAN
- **POL-29** (sibling sweep): three-dimension sweep reported above — CLEAN

## Artifacts Confirmed CLEAN (No Findings)

The following artifacts were explicitly checked and produced no findings:

- `stories/DEFECT-ADAPTER-TLS-XDOME-LIVE-001-live-xdome-https-fails-waf-h1-no-ua.md` canonical story spec — all BC pins current, RG list matches code, AC prose matches delivered behavior
- `specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline-execution.md` — v2.18 current; catalog row 91 and §T-E01 references verified
- `specs/behavioral-contracts/BC-2.08.002-http-error-classification.md` — v1.6 current; EC-08-006/007/008 verified
- `specs/behavioral-contracts/BC-2.01.010-all-targets-failed.md` — v1.6 current; per-target logging postcondition verified
- `specs/behavioral-contracts/BC-2.01.013-sensor-error-normalization.md` — v1.18 current; EC-01-029 AuthRefreshFailed/CookieAuthFailed path verified
- `specs/architecture/adr/ADR-050-reqwest-rustls-tls-backend.md` — v2.3 current; §D5 3-entry count correct; "explicit literal declaration" phrasing correct; all 12 RG names load-bearing in code
- All 12 RG test symbols verified present in delivered code (crates/prism-spec-engine + crates/prism-sensors + crates/prism-bin)
- No duplicate frontmatter keys in any swept artifact
- POL-7 BC title canonicality — CLEAN across all swept references
