---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 6
phase: LOCAL
frozen_head: "(story v1.5 + code commits e21b0cdc3 / dff20e910 / 8f6b5e131 / 67638ce07 + spec leg: BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.010 v1.6 / ADR-050 v2.2)"
new_feature_head: "b7e4cb215"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 3
streak_before: 0
streak_after: 0
closed_by: "TD-VSDD-096 records-only micro-burst — story v1.5→v1.6 (F-1 [MED] AC-ERR-001/AC-ERR-005 scope paragraphs corrected to Arm-2 variant-matching mechanism) / code b7e4cb215 (F-2 [LOW] sanitize module-header doc corrected; F-3 [LOW] RG-008 header-table row corrected) / STORY-INDEX v2.783→v2.784"
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 6

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (F-1 is [MED] which blocks both strict and PR-merge gate)
**All 3 findings CLOSED via TD-VSDD-096 records-only micro-burst**

---

## Code Core Verdict

Pass-6 adversarial review confirmed the frozen HEAD CRIT/HIGH-clean. Comprehensive PASS list confirmed:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS
- §D6 User-Agent sweep: `build_http_client_with_custom_timeout §spec_driven_adapter` + both PluginRuntime builders + `build_http_client_with_timeout §pipeline` — PASS (4 sites)
- Error source-chain wiring (`prism_core::sanitize_body_snippet §sanitize_body_snippet`, non-2xx body capture) — PASS
- Error mapping Arm-2 variants (`AuthRefreshFailed`, `CookieAuthFailed`) matched in `map_spec_engine_error §map_spec_engine_error` — PASS
- Error mapping Arm-1 (`HttpRequestFailed`) and Arm-3 (all other variants) — PASS
- SAP-1: catalog row 91 `fan_out_target_failed` present; zero unregistered `event_type` emissions — PASS
- SAP-3: all 12 Red Gate tests are parser-input or MCP-tool-call-driven; none are synthetic-AST-only — PASS
- All 12 RGTs load-bearing: RG-001..RG-012 each cover a distinct AC — PASS
- POL-7 titles: all BC H1 titles match BC-INDEX leading rows — PASS
- ADR-050 v2.2 pin: story §Behavioral Contracts table, AC-CARGO-001, and design-doc all cite v2.2 — PASS
- No duplicate frontmatter keys — PASS
- Wire-shape assertions present on AC-ERR-001/AC-ERR-005 test paths — PASS
- Security: no credential values in AI context per AD-017 — PASS
- `reqwest` entries: 3 production crates with `default-features = false, features = ["rustls-tls"]` per ADR-050 §D3 — PASS

---

## Findings

### F-1 [MED] — AC-ERR-001 and AC-ERR-005 scope paragraphs describe wrong mechanism

**Description:** The §Acceptance Criteria entries AC-ERR-001 and AC-ERR-005 contained prose stating that persistent-auth errors (`AuthRefreshFailed`, `CookieAuthFailed`) "surface as `HttpRequestFailed{401}`." This is factually incorrect. These errors are matched in `map_spec_engine_error §map_spec_engine_error` Arm 2 as distinct named variants — they do NOT pass through `HttpRequestFailed`. The prose contradiction would mislead implementers and test-writers into writing tests against the wrong arm, yielding tests that pass against a wrong implementation.

**Closed by:** story v1.5→v1.6 (story-writer, pre-written before this burst). AC-ERR-001 and AC-ERR-005 scope paragraphs corrected to describe Arm-2 variant-matching mechanism. RG-010 and RG-011 task bullets updated to Arm-2 variant-matching.

**Status: CLOSED**

---

### F-2 [LOW] — Stale sanitize module-header doc

**Description:** The `prism_core::sanitize_body_snippet §sanitize_body_snippet` module-level doc comment described a pre-refactor function signature that no longer matched the production implementation after the source-chain hoisting in commit ac9563192 (pass-1 fix-burst).

**Closed by:** code commit `b7e4cb215` on branch `feature/DEFECT-ADAPTER-TLS-XDOME-LIVE-001` (comment-only change; `just check` green). New feature HEAD `b7e4cb215`.

**Status: CLOSED**

---

### F-3 [LOW] — Stale RG-008 header-table row

**Description:** The RG-008 test-inventory header table row cited a stale pre-refactor description that no longer matched the test's actual assertion scope after the RG-007/RG-008 production-path rework in commit 67638ce07 (pass-2 fix-burst).

**Closed by:** code commit `b7e4cb215` (same commit as F-2; comment-only; `just check` green).

**Status: CLOSED**

---

## Adversary Verdict

```
CLEAN(strict): NO
CLEAN(PR-merge): NO
```

BC-5.39.001 streak: RESET 0/3 (F-1 is MED, blocks both gates).

Convergence trajectory: 5→6→1→3→2→3

New feature HEAD after F-2/F-3 code closure: `b7e4cb215`

NEXT: fresh LOCAL adversary pass 7 on new frozen HEAD (story v1.6 + code commit `b7e4cb215` + spec leg: BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.010 v1.6 / ADR-050 v2.2).
