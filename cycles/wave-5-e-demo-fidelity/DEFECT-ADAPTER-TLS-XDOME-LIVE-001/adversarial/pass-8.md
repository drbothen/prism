---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 8
phase: LOCAL
frozen_head: "(story v1.7 + code f354c9ad8 + spec leg: BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.21 / ADR-050 v2.2)"
new_feature_head: "490b5c831"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 4
streak_before: 0
streak_after: 0
closed_by: "F-1 [LOW] ADR-050 v2.3 (§D5 dev-dep explicit-literal note) + story v1.8 + design-doc (records-only). F-2 [MED] CODE 490b5c831 (RG-008 failure-message entry-count corrected 4→3; message-only; just check 5722 green). F-3 [MED] story v1.8 (AC-ERR-005 generalized to 4xx; 403 exemplar added). F-4 [LOW] BC-2.16.002 v2.18 (T-E01 sanitize-fn reference updated to sanitize_body_snippet_bytes) + story v1.8."
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 8

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (F-2 and F-3 are [MED], which blocks both strict and PR-merge gate)
**All 4 findings CLOSED (F-1 [LOW] + F-2 [MED] + F-3 [MED] + F-4 [LOW])**

**Context:** Pass-8 ran on the post-pass-7 frozen HEAD (story v1.7 + code f354c9ad8). Four doc/citation coherence findings — no logic, security, or correctness issues. The code core was confirmed CRIT/HIGH-clean. F-1 and F-2 are a partial-fix regression of pass-4 F-3 (the entry-count correction at ADR-050 §D5 propagated to the correct count in spec prose but did not sweep the test-message assertion that still asserted the old count, and the dev-dep http2 mechanism description was imprecise). F-3 and F-4 are story-spec precision gaps.

---

## Code Core Verdict

Pass-8 adversarial review confirmed the frozen HEAD CRIT/HIGH-clean. Comprehensive PASS list confirmed:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS
- §D6 User-Agent sweep: `build_http_client_with_custom_timeout §spec_driven_adapter` + both PluginRuntime builders + `build_http_client_with_timeout §pipeline` — PASS (4 sites)
- Non-2xx body snippet byte-cap: `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes` (floor_char_boundary; ≤256 bytes) — PASS
- Error source-chain wiring — PASS
- Error mapping Arm-2 variants (`AuthRefreshFailed`, `CookieAuthFailed`) matched in `map_spec_engine_error §map_spec_engine_error` — PASS
- Error mapping Arm-1 (`HttpRequestFailed`) and Arm-3 (all other variants) — PASS
- SAP-1: catalog row 91 `fan_out_target_failed` present; zero unregistered `event_type` emissions — PASS
- SAP-3: all 12 Red Gate tests are parser-input or MCP-tool-call-driven; none are synthetic-AST-only — PASS
- All 12 RGTs load-bearing: RG-001..RG-012 each cover a distinct AC — PASS
- POL-7 titles: all BC H1 titles match BC-INDEX leading rows — PASS
- No duplicate frontmatter keys — PASS
- Wire-shape assertions present on AC-ERR-001/AC-ERR-005 test paths — PASS
- Security: no credential values in AI context per AD-017 — PASS
- `reqwest` entries: 3 production crates with `default-features = false, features = ["rustls-tls"]` per ADR-050 §D3 — PASS

---

## Findings

### F-1 [LOW] — ADR-050 §D5 dev-dep http2 described as "Cargo feature unification"; it is an explicit literal declaration

**Description:** ADR-050 §D5 contained prose describing `prism-bin`'s dev-dep `http2` entry as arising from "feature unification" (the Cargo mechanism where a feature enabled by one dep enables the same feature in a shared dep). The actual situation is different: `http2` is declared as an explicit literal in `["json", "rustls-tls", "http2"]` in the dev-dep block. This is not a Cargo unification scenario — it is a deliberately written declaration. The story's AC-CARGO-001 and the design doc carried the same imprecise characterization inherited from ADR-050.

**Closed by:** ADR-050 v2.2→v2.3 (§D5 dev-dep note corrected: "explicit literal declaration" replacing "Cargo feature unification"); story v1.7→v1.8 (AC-CARGO-001 updated); `planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md` (prose corrected; no version bump — historical design doc). Pre-written by architect + story-writer.

**Status: CLOSED**

---

### F-2 [MED] — RG-008 test failure-message assertion still counts 4 entries; entry count was corrected to 3 in pass-4

**Description:** Pass-4 (F-3) correctly reduced the production reqwest entry count from 4 to 3 in ADR-050 §D5. However, the RG-008 Red Gate test's failure-message assertion was not swept in the same burst: the test still asserted the failure message contained "4 production entries" (or equivalent). The failure-message is part of the load-bearing assertion — it documents what the test proves, and a stale count means the test is asserting the wrong invariant. This is a partial-fix regression: the spec side was corrected but the test assertion side was missed. Partial-fix regressions are a recurrence of the TD-VSDD-060 sibling-site sweep discipline.

**Closed by:** Code commit `490b5c831` on branch `feature/DEFECT-ADAPTER-TLS-XDOME-LIVE-001`. The commit updates the RG-008 test failure-message assertion to reflect the correct count of 3 production entries (message-only change; no production code changed). `just check` 5722/5722 tests green after commit.

**Status: CLOSED**

---

### F-3 [MED] — AC-ERR-005 prose asserted `Some(401)` as the specific HTTP status; RG-005 tests 403

**Description:** AC-ERR-005 in the story spec described the expected HTTP error status for a specific auth-failure arm as `Some(401)`. The Red Gate test RG-005 asserts `Some(403)` — which is the actual status returned by the production code. The spec prose was overly specific and wrong. The correct characterization is: auth-failure scenarios surface as HTTP 4xx (4xx family); the specific code (401 vs 403) is implementation-determined and should be documented as an exemplar, not a contract obligation. This inconsistency between spec prose and test assertion means the spec does not accurately describe what is verified.

**Closed by:** Story v1.7→v1.8. AC-ERR-005 generalized to 4xx family contract; 403 documented as the RG-005 exemplar value. Pre-written by story-writer.

**Status: CLOSED**

---

### F-4 [LOW] — Story §T-E01 referenced `sanitize_error` as the function in prism-core; correct symbol is `sanitize_body_snippet_bytes`

**Description:** The story's T-E01 test-evidence row (or equivalent reference in the §Acceptance Criteria section) cited `sanitize_error` as the production function handling the non-2xx body snippet sanitization. After pass-7's fix (code f354c9ad8), the correct production function is `prism_core::sanitize_body_snippet_bytes`. The `sanitize_error` symbol lives in prism-mcp and handles the MCP-level error field — a different contract surface entirely. BC-2.16.002 §T-E01 also carried this stale reference.

**Closed by:** BC-2.16.002 v2.17→v2.18 (§T-E01 reference updated to `sanitize_body_snippet_bytes`); story v1.7→v1.8 (same T-E01 update). Pre-written by product-owner + story-writer.

**Status: CLOSED**

---

## TD-VSDD-097 Three-Dimension Sweep

1. **Sibling pair**: F-2 was a code-commit (RG-008 test assertion); no spec artifacts amended in the code commit. ADR-050 §D5 had one correction (v2.3); its sibling is the story and design-doc — both updated in the same burst. CLEAR.
2. **Downstream copy target**: ADR-050 §D5 is the copy-source for story AC-CARGO-001 and design-doc §D5 references. All three updated in the same burst. BC-2.16.002 §T-E01 is not verbatim-copied downstream. CLEAR.
3. **Mandate anchor**: No new MUST blocks authored. F-3 generalized an obligation (401→4xx); this is a relaxation, not a new mandate. CLEAR.

---

## Adversary Verdict

```
CLEAN(strict): NO
CLEAN(PR-merge): NO
```

BC-5.39.001 streak: RESET 0/3 (F-2 and F-3 are MED, blocking both gates).

Convergence trajectory: 5→6→1→3→2→3→1→4

New feature HEAD after F-2 code closure: `490b5c831`

NEXT: fresh LOCAL adversary pass 9 on new frozen HEAD (story v1.8 + code commit `490b5c831` + spec leg: BC-2.16.002 v2.18 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.21 / ADR-050 v2.3). BC-5.39.001 streak 0/3.
