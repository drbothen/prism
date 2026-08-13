---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 7
phase: LOCAL
frozen_head: "(story v1.7 + code b7e4cb215 + spec leg: BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.21 / ADR-050 v2.2)"
new_feature_head: "f354c9ad8"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 1
streak_before: 0
streak_after: 0
closed_by: "CODE f354c9ad8 — new prism_core::sanitize_body_snippet_bytes (control-char sanitize + str::floor_char_boundary byte-truncate → valid UTF-8 ≤256 bytes); read_non_2xx_body §read_non_2xx_body uses it; prism-mcp sanitize_error §sanitize_error UNCHANGED; load-bearing test test_BC_2_16_002_f1_non_2xx_body_byte_cap_multibyte_utf8 (300-byte input → asserts ≤256 bytes); just check 5722/5722 green"
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 7

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (F-1 is [MED] which blocks both strict and PR-merge gate)
**F-1 CLOSED via CODE commit f354c9ad8**

**Context:** Pass-7 was preceded by a consistency-validator sweep (D-2119, 24 axes clean, 4 findings closed) which front-loaded records drift. Pass-7 ran on the post-D-2119 frozen HEAD (story v1.7 + code b7e4cb215) and found only this one substantive code-contract gap.

---

## Code Core Verdict

Pass-7 adversarial review confirmed the frozen HEAD CRIT/HIGH-clean. Comprehensive PASS list confirmed:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS
- §D6 User-Agent sweep: `build_http_client_with_custom_timeout §spec_driven_adapter` + both PluginRuntime builders + `build_http_client_with_timeout §pipeline` — PASS (4 sites)
- Error source-chain wiring (`sanitize_body_snippet §prism_core::sanitize_body_snippet`, non-2xx body capture path) — PASS
- Error mapping Arm-2 variants (`AuthRefreshFailed`, `CookieAuthFailed`) matched in `map_spec_engine_error §map_spec_engine_error` — PASS
- Error mapping Arm-1 (`HttpRequestFailed`) and Arm-3 (all other variants) — PASS
- SAP-1: catalog row 91 `fan_out_target_failed` present; zero unregistered `event_type` emissions — PASS
- SAP-3: all 12 Red Gate tests are parser-input or MCP-tool-call-driven; none are synthetic-AST-only — PASS
- All 12 RGTs load-bearing: RG-001..RG-012 each cover a distinct AC — PASS
- POL-7 titles: all BC H1 titles match BC-INDEX leading rows — PASS
- BC/ADR version pins current per POL-23 (BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.21 / ADR-050 v2.2) — PASS
- No duplicate frontmatter keys — PASS
- Wire-shape assertions present on AC-ERR-001/AC-ERR-005 test paths — PASS
- Security: no credential values in AI context per AD-017 — PASS
- `reqwest` entries: 3 production crates with `default-features = false, features = ["rustls-tls"]` per ADR-050 §D3 — PASS

---

## Findings

### F-1 [MED] — Non-2xx body snippet capped at 256 CHARS; BC-2.16.002 contract mandates ≤256 BYTES

**Description:** The `read_non_2xx_body §read_non_2xx_body` function in prism-spec-engine capped the captured error body snippet by character count (256 chars) rather than byte count (256 bytes). The governing contract BC-2.16.002 §AC-ERR-003 / §EC-003 / §T-E01 explicitly mandates ≤256 **bytes**. For ASCII-only responses the two limits coincide, but for multibyte UTF-8 responses (e.g., error bodies from international APIs including xDome endpoint errors in Chinese, Arabic, or full-width characters) the char-based cap can emit a snippet that exceeds the 256-byte wire budget. This is a code-vs-spec deviation. Per CLAUDE.md §Source-of-Truth Precedence rule 7 (code-vs-spec: spec wins), the code must be corrected.

**Note on prism-mcp `sanitize_error §sanitize_error`:** The prism-mcp `sanitize_error §sanitize_error` function is char-based by design and governs a different contract surface (the MCP-level tool response error field, not the sensor error body snippet). It is correctly UNCHANGED.

**Closed by:** Code commit `f354c9ad8` on branch `feature/DEFECT-ADAPTER-TLS-XDOME-LIVE-001`.
Fix: new `prism_core::sanitize_body_snippet_bytes` — control-char sanitize step (retains printable ASCII and UTF-8 continuations, strips C0/C1 controls) THEN byte-truncate on the nearest valid char boundary below 256 bytes via `str::floor_char_boundary`, yielding a valid UTF-8 string guaranteed ≤256 bytes. `read_non_2xx_body §read_non_2xx_body` now delegates to `sanitize_body_snippet_bytes` instead of the prior char-capped path.
Load-bearing test: `test_BC_2_16_002_f1_non_2xx_body_byte_cap_multibyte_utf8` — constructs a 300-byte multibyte UTF-8 body (100 three-byte CJK characters), calls `sanitize_body_snippet_bytes`, asserts the result is valid UTF-8 AND byte length ≤ 256. `just check` 5722/5722 tests green on feature branch after fix.

**Status: CLOSED**

---

## Adversary Verdict

```
CLEAN(strict): NO
CLEAN(PR-merge): NO
```

BC-5.39.001 streak: RESET 0/3 (F-1 is MED, blocks both gates).

Convergence trajectory: 5→6→1→3→2→3→1

New feature HEAD after F-1 code closure: `f354c9ad8`

NEXT: fresh LOCAL adversary pass 8 on new frozen HEAD (story v1.7 + code commit `f354c9ad8` + spec leg: BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.21 / ADR-050 v2.2). BC-5.39.001 streak 0/3.
