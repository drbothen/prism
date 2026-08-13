---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 11
phase: LOCAL
frozen_head: "(story v1.11 + code fed26d07f + spec leg: BC-2.16.002 v2.19 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.22 / ADR-050 v2.3)"
new_feature_head: "a1864d3eb"
verdict_strict: "NO"
verdict_pr_merge: "YES"
findings_count: 2
streak_before: 0
streak_after: 0
closed_by: "F-1 [LOW] §sanitize_body_snippet_bytes doc-comment used a version-citation form referencing §sanitize_body_snippet_bytes but also cited a volatile doc-string paragraph anchored to a PR diff — CLOSED via code commit f3825985c (doc-only rewrite; toolchain-anchored language). F-2 [pre-existing; HUMAN-DIRECTED] pre-existing .expect() in §build_http_client_with_timeout panicked on TLS init failure instead of returning an error — CLOSED via code commits 010694062 (Result-ified; returns Result<Client, String>) + a1864d3eb (E-INFUSE-009 stopgap replaced with dedicated E-INFUSE-015 InfusionError::HttpClientBuildFailed; wired to 3 infusion callers; error-taxonomy v2.74; BC-2.19.001 v2.3; story v1.11→v1.12 RG-013/AC-ERR-006; 95/95 non-exhaustive; just check 5724 green)."
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 11

**CLEAN(strict): NO | CLEAN(PR-merge): YES**
**BC-5.39.001 streak: RESET 0/3** (F-2 was a code change — new frozen HEAD a1864d3eb; streak resets per DRIFT-ORCH-PRLEVEL-PUSH-001)
**2 findings: F-1 [LOW] CLOSED (doc-only f3825985c) + F-2 [pre-existing; HUMAN-DIRECTED] CLOSED (code 010694062 + a1864d3eb)**

**Context:** Pass-11 ran on the post-D-2124 frozen HEAD (story v1.11 + code fed26d07f). This pass found F-1 [LOW] a volatile-citation form in the `§sanitize_body_snippet_bytes` doc-comment (TD-VSDD-091 compliance), and independently surfaced F-2 [pre-existing] a `.expect()` call inside `§build_http_client_with_timeout` that would panic on TLS init failure. F-2 was outside the story's original scope but was HUMAN-DIRECTED for fix per the Canonical Principle. F-1 is doc-only (CLEAN(PR-merge)=YES); F-2 required a code commit. New frozen HEAD: a1864d3eb.

---

## Code Core Verdict

Pass-11 adversarial review confirmed the frozen HEAD CRIT/HIGH/MED-clean on all original story acceptance criteria. Comprehensive PASS list confirmed by adversary:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS
- §D6 User-Agent sweep: `§build_http_client_with_custom_timeout` (spec-driven adapter path) + both PluginRuntime builders + `§build_http_client_with_timeout` (independent infusion sibling, post-F-2 fix now Result-returning) — PASS (4 sites)
- Non-2xx body snippet byte-cap: `§prism_core::sanitize_body_snippet_bytes` (`floor_char_boundary`; ≤256 bytes) — PASS
- Error source-chain wiring (both emission sites) — PASS
- Error mapping Arm-1 (`HttpRequestFailed`) — PASS
- Error mapping Arm-2 variants (`AuthRefreshFailed`, `CookieAuthFailed`) matched in `§map_spec_engine_error` — PASS
- Error mapping Arm-3 (all other variants) — PASS
- SAP-1: catalog row 91 `fan_out_target_failed` present; zero unregistered `event_type` emissions — PASS
- SAP-3: all 13 Red Gate tests (RG-001..RG-013) are parser-input or MCP-tool-call-driven or SID-1 unit-test direct-construction (RG-013 §SID-1 compensating-control for ADR-050-unreachable path) — PASS
- All 13 RGTs load-bearing: RG-001..RG-013 each cover a distinct AC — PASS
- POL-7 BC H1 titles: all 6 BCs match BC-INDEX leading rows — PASS
- native-tls absent; all reqwest entries `default-features = false, features = ["rustls-tls"]` per ADR-050 §D3 — PASS
- BC-2.19.001 v2.3 §Error Conditions E-INFUSE-015 row: `InfusionError::HttpClientBuildFailed` wired at 3 infusion callers; E-INFUSE-009 stopgap retired — PASS
- AC-ERR-006 coverage: RG-013 (`test_infusion_http_client_build_failure_maps_to_e_infuse_015`) asserts Display prefix `"E-INFUSE-015:"`, detail propagation, and non-match against retired E-INFUSE-009 variant — PASS
- BC-2.16.002 v2.19 §AC-ERR-003 ≤256-byte cap enforced — PASS
- AC-ERR-005 scope: 4xx/403 exemplar correctly scoped to Arm-1 HttpRequestFailed path — PASS
- AC-WIRE-001 wire-shape assertions in compact form (`"reachable":true`, `"auth_valid":false`) — PASS

**Convergence trajectory: 5→6→1→3→2→3→1→4→2(LOW)→2(LOW+OBS)→2(LOW+pre-existing)**

---

## Findings

### F-1 [LOW] — §sanitize_body_snippet_bytes doc-comment volatile-citation form

**Description:** The doc-comment on `§prism_core::sanitize_body_snippet_bytes` used a volatile-citation form that combined a symbol anchor with a differing form seen in a prior PR diff context. The doc language referred to implementation details in a form that could anchor to a transient diff line rather than a durable symbol. Per TD-VSDD-091 (TD-VSDD-092 L9), record-tier doc-comments on canonical functions must use section/symbol/anchor cites only. This finding is records-tier doc-citation only; no behavioral defect. CLEAN(PR-merge) verdict is unaffected.

**Closed by:** Code commit f3825985c — doc-only rewrite of the `§sanitize_body_snippet_bytes` doc-comment to use toolchain-anchored language (function + behavior description, no volatile form). No functional change.

**Status: CLOSED**

---

### F-2 [pre-existing; HUMAN-DIRECTED] — §build_http_client_with_timeout .expect() on TLS init failure

**Description:** The `§build_http_client_with_timeout` function in `prism-spec-engine` used `.expect("Failed to build HTTP client")` on the `reqwest::ClientBuilder::build()` call. Under the workspace-wide `rustls-tls` mandate (ADR-050 D3), this path is effectively unreachable in production, but it is an unconditional panic if reached (e.g., misconfigured TLS backend, `native-tls` accidentally activated). The production-grade default (CLAUDE.md §Canonical Principle) forbids `.expect()` on `Result` in production code paths; the correct path is to propagate the error as a typed variant. This finding was outside the original DEFECT-ADAPTER-TLS-XDOME-LIVE-001 scope but was HUMAN-DIRECTED for fix.

**Closed by:**
1. Code commit 010694062 — `§build_http_client_with_timeout` return type changed from `Client` to `Result<Client, String>`; the `.expect()` replaced with `?` propagation (returning the `reqwest` error as a `String`).
2. Code commit a1864d3eb — three infusion callers updated to handle `Result<Client, String>`; E-INFUSE-009 stopgap mapping replaced with dedicated `E-INFUSE-015` (`InfusionError::HttpClientBuildFailed { detail }`); error-taxonomy v2.73→v2.74 (E-INFUSE-015 row); BC-2.19.001 v2.2→v2.3 (§Error Conditions E-INFUSE-015 row); story v1.11→v1.12 (RG-013/AC-ERR-006 added; BC-2.19.001 added → 6 BCs; density 13/15=0.867); 95/95 non-exhaustive gate; just check 5724 green.

**Status: CLOSED**

---

## Closure Summary

F-1 is records-tier (LOW). F-2 is a pre-existing production-code defect (`.expect()` in a non-test path) closed via HUMAN-DIRECTED in-scope fix per the Canonical Principle. CLEAN(PR-merge)=YES (F-1 doc-only does not affect PR-merge gate). CLEAN(strict)=NO because F-2 required a code change that resets the frozen HEAD to a1864d3eb. BC-5.39.001 streak RESET 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001 (new commit pushed to branch).

**TD-VSDD-097 THREE-DIMENSION VERDICTS:**
1. **Sibling pair:** F-1 — `§sanitize_body_snippet_bytes` doc-comment has no spec-sibling in a parallel BC or story; CLEAR. F-2 — BC-2.19.001 §Error Conditions has no spec-sibling BC with an identical infusion-load §Error Conditions surface; CLEAR.
2. **Downstream copy target:** E-INFUSE-015 row in BC-2.19.001 §Error Conditions is not verbatim copy-sourced into any downstream ADR or story task (error-taxonomy.md is the source; BC-2.19.001 cites it); CLEAR.
3. **Mandate anchor:** No new MUST blocks authored in this burst. AC-ERR-006 is an acceptance criterion, not a normative MUST in a BC. CLEAR.

**CLEAN(strict): NO | CLEAN(PR-merge): YES**
**BC-5.39.001 streak RESET 0/3**
**New frozen HEAD: a1864d3eb**
**NEXT: strict LOCAL adversary pass 12 on frozen HEAD a1864d3eb + story v1.12**
