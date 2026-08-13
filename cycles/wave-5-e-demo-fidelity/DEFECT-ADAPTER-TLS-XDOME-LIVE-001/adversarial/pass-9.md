---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 9
phase: LOCAL
frozen_head: "(story v1.8 + code 490b5c831 + spec leg: BC-2.16.002 v2.18 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.21 / ADR-050 v2.3)"
new_feature_head: "fed26d07f"
verdict_strict: "NO"
verdict_pr_merge: "YES"
findings_count: 2
streak_before: 0
streak_after: 0
closed_by: "F-P-LOW-001 [LOW] test doc-comment named sanitize_body_snippet instead of sanitize_body_snippet_bytes — CLOSED via CODE fed26d07f (doc-only; just check 5722 green). F-P-LOW-002 [LOW] story misattributed the MED-1 sanitize test to spec_driven_adapter.rs — CLOSED via story v1.9 (attribution corrected to prism-spec-engine tests + prism-core)."
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 9

**CLEAN(strict): NO | CLEAN(PR-merge): YES (FIRST PR-merge-clean pass)**
**BC-5.39.001 streak: RESET 0/3** (F-P-LOW-001 and F-P-LOW-002 are [LOW]; streak advances only on CLEAN(strict))
**All 2 findings CLOSED (F-P-LOW-001 [LOW] + F-P-LOW-002 [LOW])**

**Context:** Pass-9 ran on the post-pass-8 frozen HEAD (story v1.8 + code 490b5c831). This is the FIRST pass to achieve CLEAN(PR-merge)=YES — zero CRIT/HIGH/MED findings survive. Two [LOW] doc-attribution findings were identified; both are records-tier and were closed via a TD-VSDD-096 records-only micro-burst (code fed26d07f doc-only + story v1.9). The code core was confirmed CRIT/HIGH/MED-clean. CLEAN(PR-merge)=YES does NOT advance the BC-5.39.001 strict streak (which requires CLEAN(strict)=YES, meaning zero findings of any severity).

---

## Code Core Verdict

Pass-9 adversarial review confirmed the frozen HEAD CRIT/HIGH/MED-clean. Comprehensive PASS list confirmed:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS
- §D6 User-Agent sweep: `build_http_client_with_custom_timeout §spec_driven_adapter` + both PluginRuntime builders + `build_http_client_with_timeout §pipeline` — PASS (4 sites)
- Non-2xx body snippet byte-cap: `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes` (`floor_char_boundary`; ≤256 bytes) — PASS
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
- BC-2.16.002 v2.18 §AC-ERR-003 ≤256-byte cap enforced by `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes` — PASS
- AC-ERR-005 scope: 4xx/403 exemplar correctly scoped to Arm-1 HttpRequestFailed path — PASS
- ADR-050 v2.3 §D5 dev-dep http2 "explicit literal declaration" — PASS
- RG-008 failure-message table entry-count 3 (correct post-pass-8 code fix) — PASS

**Convergence trajectory: 5→6→1→3→2→3→1→4→2(LOW)**

---

## Findings

### F-P-LOW-001 [LOW] — Test doc-comment named `sanitize_body_snippet` instead of `sanitize_body_snippet_bytes`

**Description:** A test function doc-comment in the sanitize test module named the function as `sanitize_body_snippet` (the legacy function name, moved to prism-mcp) rather than `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes` (the new prism-core function introduced to close the pass-7 byte-cap finding). The doc-comment was the only incorrect reference; the test itself was correctly exercising `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes`. This is a [LOW] doc-accuracy finding; no behavioral impact.

**Closed by:** CODE commit `fed26d07f` — doc-comment corrected from stale `sanitize_body_snippet` to `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes`. Doc-only change. `just check` 5722 green. New feature HEAD `fed26d07f`.

**Status: CLOSED**

---

### F-P-LOW-002 [LOW] — Story misattributed the MED-1 sanitize test to `spec_driven_adapter.rs`

**Description:** The story's pass-7 closure narrative (within the story file) attributed the MED-1 sanitize byte-cap finding fix to a test in `spec_driven_adapter.rs §spec_driven_adapter`. The test covering `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes` lives in the prism-spec-engine test suite and the prism-core unit tests — not in `spec_driven_adapter.rs`. This mis-attribution would cause a reader reconstructing the test inventory from the story narrative to look in the wrong crate/file. [LOW] doc-attribution finding; no behavioral or test-coverage impact.

**Closed by:** Story v1.8→v1.9 — attribution corrected: test coverage attributed to prism-spec-engine tests + prism-core unit tests (the correct locations). Story-writer self-certified records-only.

**Status: CLOSED**

---

## Closure Summary

Both findings are [LOW] records-tier doc-attribution defects. The code core is CRIT/HIGH/MED-clean. This pass achieves the FIRST CLEAN(PR-merge)=YES milestone in the DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOCAL cascade — zero findings of CRIT/HIGH/MED severity survive.

**CLEAN(strict): NO** (two LOW findings prevent strict-clean)
**CLEAN(PR-merge): YES** (zero CRIT/HIGH/MED findings — FIRST PR-merge-clean pass)
**BC-5.39.001 streak after pass-9: 0/3** (CLEAN(strict)=NO; streak requires three consecutive CLEAN(strict) passes on an unchanged HEAD per BC-5.39.001 §Strict vs PR-Merge Convergence Disambiguation)

**Convergence trajectory: 5→6→1→3→2→3→1→4→2(LOW)**

**New feature HEAD: `fed26d07f`**

**ORCHESTRATOR PAUSED:** Human decision required on LOCAL-gate closure strategy — options include strict-grind (continue LOCAL passes until CLEAN(strict); remaining exposure: two LOW doc-attribution defects in same narrative zone), accept PR-merge-clean (advance to story-level holdout gate on CLEAN(PR-merge)=YES basis), or targeted structural sweep. NEXT: per human decision.
