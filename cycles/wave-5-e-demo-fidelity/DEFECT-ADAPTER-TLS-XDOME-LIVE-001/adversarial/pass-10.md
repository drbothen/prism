---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 10
phase: LOCAL
frozen_head: "(story v1.10 + code fed26d07f + spec leg: BC-2.16.002 v2.18 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.22 / ADR-050 v2.3)"
new_feature_head: "fed26d07f"
verdict_strict: "NO"
verdict_pr_merge: "YES"
findings_count: 2
streak_before: 0
streak_after: 0
closed_by: "F-P-LOW-001 [LOW] AC-WIRE-001/RG-007 JSON literal spacing — story test assertions used spaced literals (e.g. `\"reachable\": true`) while compact serialization emits `\"reachable\":true`; CLOSED via story v1.11. F-P-OBS-001 [OBS] BC-2.16.002 §Postconditions row-91 error-field disclosure under-stated the sanitized body-snippet path — CLOSED via BC-2.16.002 v2.19 (disclosure amended to name §prism_core::sanitize_body_snippet_bytes)."
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 10

**CLEAN(strict): NO | CLEAN(PR-merge): YES**
**BC-5.39.001 streak: RESET 0/3** (2 findings of LOW/OBS severity prevent strict-clean)
**All 2 findings CLOSED (F-P-LOW-001 [LOW] + F-P-OBS-001 [OBS])**

**Context:** Pass-10 ran on the post-D-2123 frozen HEAD (story v1.10 + code fed26d07f). This is the first strict LOCAL cascade pass after the D-2123 EXPANDED coherence audit. Both findings are records-tier; no content/mechanism/correctness/API-contract defects were found. The finding surface has narrowed to 2 (down from 5→6→1→3→2→3→1→4→2(LOW) in prior passes). BC-5.39.001 streak RESET 0/3 because CLEAN(strict) requires zero findings of any severity.

---

## Code Core Verdict

Pass-10 adversarial review confirmed the frozen HEAD CRIT/HIGH/MED-clean. Comprehensive PASS list confirmed by adversary:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS
- §D6 User-Agent sweep: `build_http_client_with_custom_timeout §spec_driven_adapter` + both PluginRuntime builders + `build_http_client_with_timeout §pipeline` (independent infusion sibling) — PASS (4 sites)
- Non-2xx body snippet byte-cap: `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes` (`floor_char_boundary`; ≤256 bytes) — PASS
- Error source-chain wiring (both emission sites) — PASS
- Error mapping Arm-1 (`HttpRequestFailed`) — PASS
- Error mapping Arm-2 variants (`AuthRefreshFailed`, `CookieAuthFailed`) matched in `map_spec_engine_error §map_spec_engine_error` — PASS
- Error mapping Arm-3 (all other variants) — PASS
- SAP-1: catalog row 91 `fan_out_target_failed` present; zero unregistered `event_type` emissions — PASS
- SAP-3: all 12 Red Gate tests are parser-input or MCP-tool-call-driven; none are synthetic-AST-only — PASS
- All 12 RGTs load-bearing: RG-001..RG-012 each cover a distinct AC — PASS
- POL-7 BC H1 titles: all match BC-INDEX leading rows — PASS
- native-tls absent; all reqwest entries `default-features = false, features = ["rustls-tls"]` per ADR-050 §D3 — PASS
- Wire-shape assertions present on AC-WIRE-001 test path — (see F-P-LOW-001 below)
- BC-2.16.002 v2.18 §AC-ERR-003 ≤256-byte cap enforced — PASS
- AC-ERR-005 scope: 4xx/403 exemplar correctly scoped to Arm-1 HttpRequestFailed path — PASS

**Convergence trajectory: 5→6→1→3→2→3→1→4→2(LOW)→2(LOW+OBS)**

---

## Findings

### F-P-LOW-001 [LOW] — AC-WIRE-001/RG-007 JSON literal spacing vs compact serialization

**Description:** The AC-WIRE-001 test assertions in the story spec (and the corresponding RG-007 task description) cited wire-shape JSON with spaces after colons, e.g. `"reachable": true` and `"auth_valid": false`. Serde's default compact serialization emits `"reachable":true` and `"auth_valid":false` (no space after colon). The actual test implementation already uses compact literals matching serde output, but the story's AC-WIRE-001 prose and RG-007 task description misstated the expected form. A reader writing new assertions from the story spec alone would produce non-matching assertions against the real wire output. [LOW] records-tier accuracy finding.

**Closed by:** Story v1.10→v1.11 — AC-WIRE-001 prose and RG-007 description updated to use compact literal form (`"reachable":true`, `"auth_valid":false`).

**Status: CLOSED**

---

### F-P-OBS-001 [OBS] — BC-2.16.002 row-91 error-field under-disclosed the sanitized body-snippet path

**Description:** The `fan_out_target_failed` WARN catalog row (row 91) in BC-2.16.002 §Postconditions described the `error` field schema as "error description string" without naming the sanitization function applied to the captured body snippet before it is stored in that field. The field carries output from `sanitize_body_snippet_bytes §prism_core::sanitize_body_snippet_bytes` (control-char sanitize + `floor_char_boundary` byte-truncate ≤256 bytes) — the same function governing AC-ERR-003. Without this disclosure, the catalog row is an incomplete contract: a reader cannot verify the injection-safety invariant from the catalog row alone. [OBS] records-tier disclosure gap.

**Closed by:** BC-2.16.002 v2.18→v2.19 — row-91 `error` field schema amended to name `§prism_core::sanitize_body_snippet_bytes` (control-char sanitize + `floor_char_boundary` byte-truncate ≤256 bytes), bringing it to parity with the AC-ERR-003 disclosure level.

**Status: CLOSED**

---

## Closure Summary

Both findings are records-tier (LOW/OBS). The code core is CRIT/HIGH/MED-clean; CLEAN(PR-merge)=YES. CLEAN(strict)=NO because two findings of sub-CRIT/HIGH/MED severity were present. BC-5.39.001 strict streak RESET 0/3.

**TD-VSDD-097 THREE-DIMENSION VERDICTS:**
1. **Sibling pair:** F-P-LOW-001 — story is the sole owner of AC-WIRE-001/RG-007 prose; no spec-sibling artifact carries a copy; CLEAR. F-P-OBS-001 — BC-2.16.002 row-91 has no twin in a sibling BC; AC-ERR-003 in the same BC is its closest relative and already carried the correct sanitize-fn name; CLEAR.
2. **Downstream copy target:** row-91 `error` field schema is not verbatim copy-sourced into any downstream ADR or story task; CLEAR.
3. **Mandate anchor:** no new MUST blocks authored in this records-only burst; CLEAR.

**CLEAN(strict): NO | CLEAN(PR-merge): YES**
**BC-5.39.001 streak RESET 0/3**
**NEXT: strict LOCAL adversary pass 11 on frozen HEAD fed26d07f + story v1.11**
