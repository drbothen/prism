---
document_type: fix-burst-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
burst_number: 5
date: 2026-05-23
feature_head_before: 639d89e1
feature_head_after: 7702ea78
develop_head_baseline: f19575ff
findings_addressed: [F-LP6-MED-001, F-LP6-MED-002, F-LP6-LOW-001]
findings_closed: 3
findings_deferred: 0
micro_commits: 2
workspace_test_count: 3762
just_check_status: PASS
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-6.md
  - .factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md
input-hash: "[live-fb-5]"
---

# PLUGIN-MIGRATION-001-E — FB-IMPL-5

**Date:** 2026-05-23
**Feature HEAD before:** `639d89e1`
**Feature HEAD after:** `7702ea78`
**Develop baseline:** `f19575ff`
**Findings addressed:** F-LP6-MED-001 + F-LP6-MED-002 + F-LP6-LOW-001
**Micro-commits:** 2 (68017bfa + 7702ea78)
**Workspace tests:** 3762 GREEN (+2 from FB-IMPL-4's 3760)
**just check:** PASS

## F-LP6-MED-001 — EC-002 spec-test fidelity drift

### Closure (production-grade default = both sides)

Commit `7702ea78`:
- Added `test_acquire_token_EC_002_invalid_json_returns_response_parse` in `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` `#[cfg(test)] mod tests`. Uses `push_http_response(200, "this is not JSON {[")`; asserts `matches!(err, AuthError::ResponseParse(_))` AND non-empty detail string. Exercises the `serde_json::from_str` branch the story spec EC-002 actually names.
- Renamed `test_acquire_token_EC_002_non_2xx_returns_response_parse` → `test_acquire_token_non_2xx_returns_response_parse`. Added doc comment explaining this is the separate defense-in-depth status-check branch (not EC-002).
- Story spec `/Users/jmagady/Dev/prism/.factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md` bumped v1.1 → v1.2. EC-002 "Red Gate Test" cell updated to cite `test_acquire_token_EC_002_invalid_json_returns_response_parse`. Defense-in-depth footnote added after the EC table documenting the non-2xx test.

### Anti-paper-fix evidence

Both code paths now have load-bearing variant-matching tests with distinct named scenarios. A regression flipping `AuthError::ResponseParse(e.to_string())` to `AuthError::Internal(e.to_string())` on JSON parse failure would fail `test_acquire_token_EC_002_invalid_json_returns_response_parse`. A regression flipping the non-2xx error variant would fail `test_acquire_token_non_2xx_returns_response_parse`. Spec-test drift closed.

## F-LP6-MED-002 — sibling-WIT-file sweep

### Closure

Commit `68017bfa`:
- `crates/prism-spec-engine/wit/prism-sensor-plugin.wit`: moved `log-level`, `http-response` into the `host` interface; moved `page-result` into a new `sensor-plugin` interface; world renamed to `sensor-plugin-world` for clarity.
- `crates/prism-spec-engine/wit/prism-infusion-plugin.wit`: moved `log-level`, `http-response` into `host`; moved `enrichment` type into a new `infusion-plugin` interface; world renamed to `infusion-plugin-world`.
- `crates/prism-spec-engine/wit/prism-action-plugin.wit`: moved `log-level`, `http-response` into `host`; moved `alert-context`, `case-context`, `report-context`, `action-result` into a new `action-plugin` interface; world renamed to `action-plugin-world`.

Each file's restructure mirrors the proven FB-IMPL-4 pattern applied to `sensor-auth.wit`.

### Intent verification result

The 3 sibling WIT files are NOT docs-only stubs. Each contains realistic function signatures per the SS-17 WASM Plugin Runtime roadmap (AD-019 plugin types). The structural sweep IS in-scope work, not premature design.

### Anti-paper-fix evidence

No `wit_bindgen::generate!` consumer exists for these 3 files yet, so no test exercises the fix today. However, the structural-coverage axis (pass-5) established that wit-bindgen 0.51+ rejects top-level type declarations at compile time. The instant a future story authors a sensor/infusion/action plugin via wit-bindgen, the `wasm32-compile-check` CI job (FB-IMPL-4) will exercise the same structural-validity gate that caught F-LP5-HIGH-001. The defect class is now structurally extinct in this directory.

## F-LP6-LOW-001 — EC-004 zero-`expires_in` uncovered

### Closure

Commit `7702ea78` (same commit as F-LP6-MED-001):
- Added `test_acquire_token_EC_004_zero_expires_in_defaults_to_1799`. Body: `r#"{"access_token": "tok-abc", "token_type": "bearer", "expires_in": 0}"#`. Asserts `expires_at == now + 1769` (1799 default - 30 saturating_sub leeway).
- Story spec EC-004 "Red Gate Test" cell now cites BOTH `test_acquire_token_EC_004_missing_expires_in_defaults_to_1799` AND `test_acquire_token_EC_004_zero_expires_in_defaults_to_1799`.

### Anti-paper-fix evidence

A regression removing the `.filter(|&v| v > 0)` filter in `acquire_token` would compile clean but fail `test_acquire_token_EC_004_zero_expires_in_defaults_to_1799` (the zero case would propagate, producing `expires_at = now`, immediately stale-cached). The "infinite token refresh loop" failure mode named in the pass-6 finding is now structurally prevented.

## Workspace verification

`just check`: PASS. 3762 tests run, 3762 passed (+2 from FB-IMPL-4's 3760). 25 skipped (the pre-existing `#[ignore]`'d set). `#[non_exhaustive]` count 32 (unchanged — no API surface changes). Zero warnings. Zero clippy errors.

## Files modified

- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` (2 new tests + 1 rename + doc comment)
- `crates/prism-spec-engine/wit/prism-sensor-plugin.wit`
- `crates/prism-spec-engine/wit/prism-infusion-plugin.wit`
- `crates/prism-spec-engine/wit/prism-action-plugin.wit`
- `.factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md` (v1.1 → v1.2 + EC-002 + EC-004 test refs + defense-in-depth footnote)

## Streak attempt impact

FB-IMPL-5 closes all 3 pass-6 findings. Next adversary pass (pass-7) attempts streak 0/3 → 1/3. The EC-test-vs-spec-fidelity axis introduced in pass-6 plus the structural-coverage axis from pass-5 are now both standing probes for pass-7+.
