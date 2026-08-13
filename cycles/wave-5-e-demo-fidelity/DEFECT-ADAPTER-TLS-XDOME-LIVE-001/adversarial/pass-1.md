---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 1
phase: LOCAL
frozen_head: "(story v1.1 + spec leg: BC-2.16.002 v2.14 / BC-2.08.002 v1.4 / BC-2.01.013 v1.16 / error-taxonomy v2.72)"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 5
streak_before: 0
streak_after: 0
closed_by: "code commit ac9563192 + spec-leg (BC-2.16.002 v2.15 / BC-2.08.002 v1.5 / BC-2.01.013 v1.17 / error-taxonomy v2.73 / BC-INDEX v8.99 / story v1.2)"
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 1

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (findings present)
**All 5 findings CLOSED by fix-burst (code commit ac9563192 + spec leg)**

---

## Findings

### MED-1 — Body-snippet sanitization not production-ready
**Severity:** MED | **Owner:** implementer | **Status:** CLOSED
**Closure:** `sanitize_body_snippet` hoisted from story-local to `prism_core` as a shared helper; production fetch pipeline path updated to use it. Code commit ac9563192.

### MED-2 — RG-009 source-chain test gap (error source annotation)
**Severity:** MED | **Owner:** test-writer / implementer | **Status:** CLOSED
**Closure:** RG-009 source-chain assertion test added in code commit ac9563192. BC-2.16.002 v2.15 added §OBS-1 source-chain citation → RG-009 anchor in §Postconditions.

### LOW-1 — RG-007 production-path rework required
**Severity:** LOW | **Owner:** implementer | **Status:** CLOSED
**Closure:** RG-007 production-path reworked in code commit ac9563192 to cover real `fetch_pipeline` invocation path, not test-double-only path.

### LOW-2 — Doc fix (auth-error classification prose)
**Severity:** LOW | **Owner:** spec-steward / product-owner | **Status:** CLOSED
**Closure:** BC-2.08.002 v1.5 corrected persistent-auth-failure → `HttpError{401}` postcondition; EC-08-007/EC-08-008 added. error-taxonomy v2.73 added E-AUTH-002/E-AUTH-004 mapping notes. Code commit ac9563192 fixes `AuthRefreshFailed`/`CookieAuthFailed` → `HttpError{401}` (was `Internal`): **in-scope production defect discovered and fixed**.

### OBS-1 — [process-gap] BC mis-cite in story (EC reference pointed to wrong BC)
**Severity:** OBS / PROCESS-GAP | **Owner:** story-writer / state-manager | **Status:** CLOSED
**Closure:** Story v1.2 corrects BC mis-cite; BC-2.01.013 v1.17 adds EC-01-029 and corrects input-hash. BC-2.01.013 added to story `behavioral_contracts:` frontmatter (bcs: 4→5). BC-INDEX v8.99 pins all three updated BCs.

---

## In-Scope Production Defect Fixed (LOW-2)

`AuthRefreshFailed` and `CookieAuthFailed` error variants were mapped to `HttpError{Internal}` at the MCP boundary. Correct mapping per BC-2.08.002 §Postconditions (EC-08-007/008) is `HttpError{401}` (auth credential failure = unauthenticated, not internal-server-error). Fixed in code commit ac9563192.

---

## Fix-Burst Summary

| Artifact | Change |
|----------|--------|
| code commit ac9563192 | `sanitize_body_snippet` → prism-core; RG-009 source-chain test; RG-007 production-path rework; `AuthRefreshFailed`/`CookieAuthFailed`→`HttpError{401}`; doc fix |
| BC-2.16.002 v2.15 | §OBS-1 source-chain citation → RG-009 anchor |
| BC-2.08.002 v1.5 | persistent-auth-failure → `HttpError{401}` postcondition; EC-08-007/EC-08-008 |
| BC-2.01.013 v1.17 | EC-01-029; input-hash corrected |
| error-taxonomy v2.73 | E-AUTH-002/E-AUTH-004 mapping notes |
| BC-INDEX v8.99 | Leading-pin updates for BC-2.16.002 v2.15 / BC-2.08.002 v1.5 / BC-2.01.013 v1.17 |
| story v1.2 | RG-009/RG-010/RG-011 added (total 11 RGTs); BC-2.01.013 added to `behavioral_contracts:` (bcs: 4→5); density 0.786 |

---

## TD-VSDD-097 Three-Dimension Sweep

**Dim-1 (sibling pair):** BC-2.16.002 amended; its paired BC for auth-surface is BC-2.08.002 — both amended in same burst (CLEAR). BC-2.01.013 (DataSource adapter) amended; no same-capability twin BC exists (CLEAR).

**Dim-2 (downstream copy target):** BC-2.16.002 §OBS-1 source-chain citation is cited in story v1.2 RG-009 anchor — both updated in same burst (CLEAR). error-taxonomy E-AUTH-002/E-AUTH-004 mapping notes are cited in BC-2.08.002 EC-08-007/EC-08-008 — both updated in same burst (CLEAR).

**Dim-3 (mandate anchor):** BC-2.08.002 EC-08-007 `MUST` anchored to `DEFECT-ADAPTER-TLS-XDOME-LIVE-001` AC-010/RG-010 (AuthRefreshFailed→HttpError{401}). EC-08-008 `MUST` anchored to AC-011/RG-011 (CookieAuthFailed→HttpError{401}). BC-2.16.002 §OBS-1 source-chain `MUST` anchored to AC-009/RG-009.

---

## Next Step

Fresh LOCAL adversary pass 2 on updated frozen HEAD (story v1.2 + code commit ac9563192 + spec leg). BC-5.39.001 streak 0/3.
