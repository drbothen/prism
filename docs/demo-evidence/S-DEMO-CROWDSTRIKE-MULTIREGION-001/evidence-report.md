# Evidence Report — S-DEMO-CROWDSTRIKE-MULTIREGION-001

**Story:** S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.4 — CrowdStrike Multi-Region base_url Fidelity  
**Worktree HEAD:** PR #170 / story v1.4 (AC-001 re-recorded during PR-LEVEL pass-1 evidence correction)  
**Branch:** `feature/S-DEMO-CROWDSTRIKE-MULTIREGION-001`  
**Story version:** v1.4 (S-DEMO-CROWDSTRIKE-MULTIREGION-001)  
**Convergence:** LOCAL cascade 3/3 CLEAN  
**Recorded by:** demo-recorder (Step 5 of per-story-delivery)  
**Date:** 2026-06-03

---

## Coverage Table

| AC | Description | Status | Evidence Type | Files |
|----|-------------|--------|---------------|-------|
| AC-001 | crowdstrike.sensor.toml has `base_url = "${env.CROWDSTRIKE_BASE_URL}"` with 4-region comment | DEMONSTRATED | VHS recording | `AC-001-crowdstrike-toml-base-url-env-var.{gif,webm,tape}` |
| AC-002 | eu-1 URL resolves from env var (happy path Red Gate test) | DEMONSTRATED | VHS recording (Red Gate PASS) | `AC-002-eu1-base-url-resolves.{gif,webm,tape}` |
| AC-003 | Unset CROWDSTRIKE_BASE_URL → E-SPEC-024 structured error, not panic | DEMONSTRATED | VHS recording (Red Gate PASS) | `AC-003-unset-env-e-spec-024.{gif,webm,tape}` |
| AC-004 | DTU loopback address resolves from env var (spec-load unit path) | DEMONSTRATED (unit path) + DTU-EXT-001 deferred | VHS recording (Red Gate PASS) + #[ignore] note | `AC-004-dtu-loopback-resolves.{gif,webm,tape}` |
| AC-005 | auth_type and auth_plugin unchanged (D-747 LOCKED) | DEMONSTRATED | Inline in AC-002 Red Gate test (asserts both values) | see AC-002 test body |
| AC-006 | No uncatalogued `event_type` emissions (SAP-1) | DEMONSTRATED | SAP-1 grep — zero new emissions in story-touched files | see SAP-1 section below |

---

## AC-001: crowdstrike.sensor.toml base_url uses `${env.CROWDSTRIKE_BASE_URL}`

**Acceptance Criterion:** `base_url = "${env.CROWDSTRIKE_BASE_URL}"` is present; hardcoded `"https://api.crowdstrike.com"` is removed; 4-region runbook comment is adjacent to the field.

**Demonstrated by:** `AC-001-crowdstrike-toml-base-url-env-var.{gif,webm}`

The recording runs three commands:
1. `grep -A10 region crates/prism-sensors/specs/crowdstrike.sensor.toml | head -14` — shows the 4-region runbook comment and `base_url = "${env.CROWDSTRIKE_BASE_URL}"` in the production TOML.
2. `grep -E '^base_url\s*=' ...` — returns `base_url = "${env.CROWDSTRIKE_BASE_URL}"`, confirming the field is env-var driven.
3. `grep -E '^base_url\s*=.*api\.crowdstrike\.com' ... || echo 'field-not-hardcoded'` — returns `field-not-hardcoded`, confirming the `base_url` FIELD contains no hardcoded URL.

Note: the region runbook comment block (`# us-1 (default): https://api.crowdstrike.com` etc.) intentionally retains all four region URLs for operator reference — `grep -c api.crowdstrike.com` returns `1` because the comment contains that string. The field-discriminating guards above (commands 2 and 3) correctly target the `base_url` FIELD only, not comment lines.

**TOML state confirmed:**
```toml
# CrowdStrike Falcon API region base URLs (set CROWDSTRIKE_BASE_URL to the tenant's region):
#   us-1 (default):  https://api.crowdstrike.com
#   us-2:            https://api.us-2.crowdstrike.com
#   eu-1:            https://api.eu-1.crowdstrike.com
#   gov:             https://api.laggar.gcw.crowdstrike.com
base_url = "${env.CROWDSTRIKE_BASE_URL}"
```

**BC trace:** BC-2.16.013 §Postconditions §1 (CrowdStrike spec authoring fidelity)

---

## AC-002: Spec-load test resolves eu-1 URL when env var is set

**Acceptance Criterion:** Set `CROWDSTRIKE_BASE_URL=https://api.eu-1.crowdstrike.com`; spec loads without errors; `spec.base_url == "https://api.eu-1.crowdstrike.com"`.

**Red Gate test:** `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly`  
**Crate:** `prism-spec-engine`  
**Test result:** PASS (0.024s)

**Demonstrated by:** `AC-002-eu1-base-url-resolves.{gif,webm}` — recording shows the nextest run completing with PASS.

**Full nextest output (captured separately):**
```
Nextest run ID 2b153e50-a73a-443f-9250-52c8b4ad90d5 with nextest profile: default
    Starting 3 tests across 37 binaries (526 tests skipped)
        PASS [   0.024s] (1/3) prism-spec-engine::bc_2_16_013_crowdstrike_multiregion test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic
        PASS [   0.024s] (2/3) prism-spec-engine::bc_2_16_013_crowdstrike_multiregion test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly
        PASS [   0.024s] (3/3) prism-spec-engine::bc_2_16_013_crowdstrike_multiregion test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works
────────────
     Summary [   0.025s] 3 tests run: 3 passed, 526 skipped
```

This test also asserts AC-005: `spec.auth_type == AuthType::Oauth2ClientCredentials` and `spec.auth_plugin == Some("crowdstrike-oauth2")` (D-747 LOCKED values).

**BC trace:** BC-2.16.013 §Postconditions §1; BC-2.16.009 EC-009-008

---

## AC-003: Structured E-SPEC error (not panic) when CROWDSTRIKE_BASE_URL is unset

**Acceptance Criterion:** When `CROWDSTRIKE_BASE_URL` is not set, `parse_and_validate_spec_toml` returns `Err(...)` with an error message containing "CROWDSTRIKE_BASE_URL". Must not panic.

**Red Gate test:** `test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic`  
**Crate:** `prism-spec-engine`  
**Test result:** PASS (0.024s)

**Demonstrated by:** `AC-003-unset-env-e-spec-024.{gif,webm}` — recording shows the nextest run completing with PASS.

The test has two load-bearing assertions:
1. Calls `std::env::remove_var("CROWDSTRIKE_BASE_URL")` (ensures absent).
2. Calls `parse_and_validate_spec_toml` — must not panic.
3. Asserts `result.is_err()` — the load is rejected with a structured error, not success. (Postcondition 1)
4. Asserts combined error text contains `"CROWDSTRIKE_BASE_URL"` — the operator knows which env var to configure. (Postcondition 2, E-SPEC-024)

There is no step-5 ordering assertion in this test. The `!contains("must start with http")` assertion was deliberately removed during the pass-4 fix-burst because `parse_and_validate_spec_toml` does NOT call `validate_sensor_spec`; the "must start with http" / E-SPEC-001 error is structurally unreachable on this code path, making any such assertion inert. The BC-2.16.009 §VR6 env-resolver-before-URL-validation ordering invariant IS load-bearing tested, but in the correct SUT scope (`tests/env_var_resolution_tests.rs`, which calls `validate_sensor_spec` directly):
- `test_env_var_resolution_runs_before_url_format_validation`
- `test_env_var_ordering_production_path_absent_var_produces_e_spec_024_not_url_format_error`

**BC trace:** BC-2.16.009 §Validation Rules 6 (env-var resolver) + E-SPEC-024; EC-009-009

---

## AC-004: DTU demo path works when env var points to local DTU address

**Acceptance Criterion:** Set `CROWDSTRIKE_BASE_URL=http://127.0.0.1:<port>`; spec loads with DTU address as base_url; pipeline connects.

**Coverage: TWO-TIER**

### Tier 1: Non-ignored spec-load test (DEMONSTRATED)

**Red Gate test:** `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works`  
**Crate:** `prism-spec-engine`  
**Test result:** PASS (0.024s)

**Demonstrated by:** `AC-004-dtu-loopback-resolves.{gif,webm}`

The test sets `CROWDSTRIKE_BASE_URL=http://127.0.0.1:9999`, loads `crowdstrike.sensor.toml` via `parse_and_validate_spec_toml`, and asserts `spec.base_url == "http://127.0.0.1:9999"`. This proves env-var substitution works for the DTU loopback address without requiring a live DTU.

Also asserts AC-005 values (D-747 LOCKED auth_type and auth_plugin).

### Tier 2: Full DTU pipeline connection (DTU-EXT-001 DEFERRED per SID-1)

The companion test `test_BC_2_16_013_crowdstrike_base_url_dtu_pipeline_connection_succeeds` exercises the full OAuth2 token exchange + detection fetch against a live DTU clone. It is `#[ignore]`'d per SID-1 with the following justification in the test file:

```
#[ignore = "DTU-EXT-001: requires prism-dtu-crowdstrike DTU clone running; \
             ungated after S-6.07 merges. Unit coverage of spec-load behavior \
             provided by test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works."]
```

The `#[ignore]` is justified because:
- The full pipeline requires the `prism-dtu-crowdstrike` clone to be running (an external service dependency).
- The non-ignored Tier 1 test covers the spec-loading behavior that is within this story's scope.
- Per ADR-031 §D8-c: "the DTU is already region-agnostic — it binds to 127.0.0.1:0 and accepts any valid OAuth2 + Bearer flow regardless of base URL." No DTU code changes were required by this story.
- The full pipeline connection test will be ungated after S-6.07 merges.

**BC trace:** BC-2.16.013 §Postconditions §2 (DTU parity; DTU region-agnostic per ADR-031 §D8-c); EC-005

---

## AC-005: auth_type and auth_plugin are unchanged (D-747 LOCKED)

**Acceptance Criterion:** `crowdstrike.sensor.toml` retains `auth_type = "oauth2_client_credentials"` and `auth_plugin = "crowdstrike-oauth2"` (D-747 LOCKED values).

**Demonstrated by:** Inline in AC-002 and AC-004 Red Gate tests. Both tests assert:
```rust
assert_eq!(spec.auth_type, AuthType::Oauth2ClientCredentials, "AC-005 / D-747 LOCKED");
assert_eq!(spec.auth_plugin.as_deref(), Some("crowdstrike-oauth2"), "AC-005 / D-747 LOCKED");
```

**TOML confirmation:**
```toml
auth_type = "oauth2_client_credentials"
auth_plugin = "crowdstrike-oauth2"
```
(Lines 17 and 23 of `crates/prism-sensors/specs/crowdstrike.sensor.toml` — unchanged from PLUGIN-MIGRATION-001-D/E.)

**BC trace:** BC-2.16.013 §Postconditions §1 (CrowdStrike spec authoring fidelity); D-747 LOCKED

---

## AC-006: No uncatalogued tracing event_type emissions (SAP-1)

**Acceptance Criterion:** Zero new `tracing::*!(event_type = ...)` emissions introduced by this story without a BC-2.16.002 catalog row.

**SAP-1 probe result:** CLEAN

Command run:
```bash
rg 'event_type\s*=' crates/prism-spec-engine/src/spec_parser.rs crates/prism-spec-engine/src/add_sensor_spec.rs
```

Result: The only match in `spec_parser.rs` is a doc comment (`/// tracing::warn!(event_type = "timestamp.fallback_to_now"...)`), not an active emission. No new `event_type =` emissions were added by this story.

The story's scope was:
- `crates/prism-sensors/specs/crowdstrike.sensor.toml` (TOML-only change, no Rust code)
- `crates/prism-spec-engine/tests/bc_2_16_013_crowdstrike_multiregion.rs` (test code only — no production emissions)

No new catalog rows required.

**BC trace:** SAP-1 standing probe; BC-2.16.002 §Structured Event Catalog; PG-LP11-001

---

## Files in This Directory

| File | Type | AC | Size |
|------|------|----|------|
| `AC-001-crowdstrike-toml-base-url-env-var.tape` | VHS script | AC-001 | 1.3 KB |
| `AC-001-crowdstrike-toml-base-url-env-var.gif` | Recording | AC-001 | 140 KB |
| `AC-001-crowdstrike-toml-base-url-env-var.webm` | Recording | AC-001 | 157 KB |
| `AC-002-eu1-base-url-resolves.tape` | VHS script | AC-002 | 1.0 KB |
| `AC-002-eu1-base-url-resolves.gif` | Recording | AC-002 | 555 KB |
| `AC-002-eu1-base-url-resolves.webm` | Recording | AC-002 | 2.5 MB |
| `AC-003-unset-env-e-spec-024.tape` | VHS script | AC-003 | 1.0 KB |
| `AC-003-unset-env-e-spec-024.gif` | Recording | AC-003 | 559 KB |
| `AC-003-unset-env-e-spec-024.webm` | Recording | AC-003 | 2.5 MB |
| `AC-004-dtu-loopback-resolves.tape` | VHS script | AC-004 | 1.1 KB |
| `AC-004-dtu-loopback-resolves.gif` | Recording | AC-004 | 552 KB |
| `AC-004-dtu-loopback-resolves.webm` | Recording | AC-004 | 262 KB |
| `evidence-report.md` | This file | all | 11 KB |

---

## Summary

| AC | Demonstrated | Method |
|----|-------------|--------|
| AC-001 | Full | VHS recording: TOML grep shows env-var base_url + 4-region comment; field-discriminating guards confirm `base_url` field is env-var driven (not hardcoded); region runbook comment intentionally retains all 4 region URLs for operator reference |
| AC-002 | Full | VHS recording: Red Gate test PASS — eu-1 URL resolves |
| AC-003 | Full | VHS recording: Red Gate test PASS — unset env → E-SPEC-024, no panic |
| AC-004 | Partial (SID-1 compliant) | VHS recording: Red Gate PASS (spec-load unit path); full DTU connection deferred to DTU-EXT-001 (after S-6.07) |
| AC-005 | Full | Inline in AC-002 + AC-004 test assertions; TOML confirmed |
| AC-006 | Full | SAP-1 grep: zero new event_type emissions in story-touched files |

All 3 Red Gate tests: **PASS** (3/3 in 0.025s)  
Story worktree HEAD: PR #170 / story v1.4 (AC-001 GIFs re-recorded during PR-LEVEL pass-1 evidence correction; volatile HEAD SHA pin dropped per TD-VSDD-091)  
Story version: v1.4 (S-DEMO-CROWDSTRIKE-MULTIREGION-001, established D-946)
