---
document_type: demo-evidence-report
product: "prism-spec-engine — CrowdStrike OAuth2 Refresh-on-401 WASM Plugin"
story_id: PLUGIN-MIGRATION-001-E
pipeline_run: "2026-05-23"
demo_type: "cli"
recording_tool: "vhs"
status: complete
---

# Demo Evidence Report — PLUGIN-MIGRATION-001-E

**Story:** CrowdStrike OAuth2 Refresh-on-401 as In-Repo .prx WASM Plugin
**Feature Branch:** `feature/PLUGIN-MIGRATION-001-E`
**HEAD at recording:** `9e412c83`
**LOCAL adversary cascade:** CONVERGED — pass-12, BC-5.39.001 3-CLEAN strict satisfied

---

## Per-AC Demo Recordings

All 11 acceptance criteria have VHS-recorded demos. Each recording shows `cargo nextest run`
output filtered to `PASS|FAIL|Summary` so the PASS status for that AC's Red Gate test is
clearly visible on screen.

| AC | Description | Red Gate Test | Recording (.gif) | Recording (.webm) | Tape | Status |
|----|-------------|---------------|-----------------|-------------------|------|--------|
| AC-001 | Plugin source compiles; manifest + WIT validation passes | `test_PLUGIN_MIGRATION_001_E_001_plugin_compiles_and_manifest_validates` | [AC-001.gif](AC-001-plugin-compiles-and-manifest-validates.gif) | [AC-001.webm](AC-001-plugin-compiles-and-manifest-validates.webm) | [tape](AC-001-plugin-compiles-and-manifest-validates.tape) | recorded |
| AC-002 | `auth_type_name()` returns `oauth2_client_credentials` (INV-AUTH-OPEN-003 Rule A) | `test_PLUGIN_MIGRATION_001_E_002_auth_type_name_returns_oauth2_client_credentials` | [AC-002.gif](AC-002-auth-type-name-returns-canonical-value.gif) | [AC-002.webm](AC-002-auth-type-name-returns-canonical-value.webm) | [tape](AC-002-auth-type-name-returns-canonical-value.tape) | recorded |
| AC-003 | Token acquisition via `POST /oauth2/token` against DTU clone | `test_PLUGIN_MIGRATION_001_E_003_acquire_token_calls_oauth2_token_endpoint` | [AC-003.gif](AC-003-token-acquisition-via-oauth2-token-endpoint.gif) | [AC-003.webm](AC-003-token-acquisition-via-oauth2-token-endpoint.webm) | [tape](AC-003-token-acquisition-via-oauth2-token-endpoint.tape) | recorded |
| AC-004 | Token cached within TTL; subsequent `get_token()` reuses cache (no second HTTP call) | `test_PLUGIN_MIGRATION_001_E_004_token_cached_within_ttl_no_second_request` | [AC-004.gif](AC-004-token-cached-within-ttl-no-second-request.gif) | [AC-004.webm](AC-004-token-cached-within-ttl-no-second-request.webm) | [tape](AC-004-token-cached-within-ttl-no-second-request.tape) | recorded |
| AC-005 | Expired token triggers re-acquisition (cache miss path) | `test_PLUGIN_MIGRATION_001_E_005_expired_token_triggers_reacquisition` | [AC-005.gif](AC-005-expired-token-triggers-reacquisition.gif) | [AC-005.webm](AC-005-expired-token-triggers-reacquisition.webm) | [tape](AC-005-expired-token-triggers-reacquisition.tape) | recorded |
| AC-006 | 401 triggers plugin token refresh + single retry via `PipelineExecutor` (VP-150) | `test_PLUGIN_MIGRATION_001_E_006_401_triggers_plugin_token_refresh_and_retry` | [AC-006.gif](AC-006-401-triggers-plugin-token-refresh-and-retry.gif) | [AC-006.webm](AC-006-401-triggers-plugin-token-refresh-and-retry.webm) | [tape](AC-006-401-triggers-plugin-token-refresh-and-retry.tape) | recorded |
| AC-007 | `crowdstrike.sensor.toml` declares `auth_plugin`; unknown plugin → E-SPEC-012 | `test_PLUGIN_MIGRATION_001_E_007_*` (4 variants) | [AC-007.gif](AC-007-crowdstrike-toml-declares-auth-plugin.gif) | [AC-007.webm](AC-007-crowdstrike-toml-declares-auth-plugin.webm) | [tape](AC-007-crowdstrike-toml-declares-auth-plugin.tape) | recorded |
| AC-008 | VP-148 parity remains GREEN after TOML amendment (plugin path = fixture output) | `test_PLUGIN_MIGRATION_001_E_008_vp148_parity_green_after_toml_amendment` | [AC-008.gif](AC-008-vp148-parity-green-after-toml-amendment.gif) | [AC-008.webm](AC-008-vp148-parity-green-after-toml-amendment.webm) | [tape](AC-008-vp148-parity-green-after-toml-amendment.tape) | recorded |
| AC-009 | Plugin loaded at boot step 7.5; emits `plugin_load_unsigned` WARN (BC-2.22.001) | `test_PLUGIN_MIGRATION_001_E_009_plugin_loaded_at_boot_step_7_5_emits_warn` | [AC-009.gif](AC-009-plugin-loaded-at-boot-step-7-5-emits-warn.gif) | [AC-009.webm](AC-009-plugin-loaded-at-boot-step-7-5-emits-warn.webm) | [tape](AC-009-plugin-loaded-at-boot-step-7-5-emits-warn.tape) | recorded |
| AC-010 | Credential opaqueness — `access_token` not in tracing output (AD-017) | `test_PLUGIN_MIGRATION_001_E_010_token_not_in_tracing_output` | [AC-010.gif](AC-010-credential-opaqueness-token-not-logged.gif) | [AC-010.webm](AC-010-credential-opaqueness-token-not-logged.webm) | [tape](AC-010-credential-opaqueness-token-not-logged.tape) | recorded |
| AC-011 | `just check` workspace-wide GREEN — all 15 story tests pass | `test_PLUGIN_MIGRATION_001_E_*` (15 tests, `--no-fail-fast`) | [AC-011.gif](AC-011-just-check-workspace-green.gif) | [AC-011.webm](AC-011-just-check-workspace-green.webm) | [tape](AC-011-just-check-workspace-green.tape) | recorded |

**Coverage: 11/11 ACs recorded. 0 skipped. 0 failed.**

---

## Behavioral Contract Traceability

| BC | Title | AC(s) Demonstrated |
|----|-------|--------------------|
| BC-2.01.016 | SensorAuth Open Trait — Plugin-Implementable Auth Contract | AC-002, AC-003, AC-004, AC-005, AC-006, AC-010 |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | AC-011 (workspace GREEN) |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification | AC-007, AC-008, AC-011 |
| BC-2.17.001 | Plugin Panic Isolation / KV Store State | AC-004, AC-005 |
| BC-2.17.006 | Plugin WIT Validation | AC-001 |
| BC-2.17.007 | Plugin Manifest Schema Validation | AC-001 |
| BC-2.22.001 | Boot Orchestration — step 7.5 plugin load | AC-009 |
| VP-148 | DTU parity — TOML+plugin path output matches fixture | AC-008 |
| VP-150 | OAuth2 refresh-on-401 via PipelineExecutor retry path | AC-006 |
| AD-017 | AI-opaque credential model | AC-010 |
| INV-AUTH-OPEN-003 Rule A | auth_type_name() must match TOML auth_type | AC-002 |
| ADR-028 §D2 | auth_type LOCKED; plugin path grounds declaration | AC-007 |

---

## Recording Method Notes

**Product type:** CLI (Rust). VHS used for all recordings per demo-recording protocol.

**Demo strategy:** Each tape runs `cargo nextest run -p prism-spec-engine` filtered to the
specific Red Gate test for that AC. The `grep -E 'PASS|FAIL|Summary'` filter reduces output
to the pass/fail signal — the most legible evidence for PR reviewers.

AC-007 uses `--no-fail-fast` and a broader test filter (`test(PLUGIN_MIGRATION_001_E_007)`)
to capture all 4 test variants (success path + error path + registered plugin + no-plugin-field)
in a single recording.

AC-011 runs all 15 story tests with `--no-fail-fast` and shows `tail -6` (Summary line) to
demonstrate the complete workspace pass in one recording.

**No browser recordings:** This is a WASM plugin story with no web UI surface. All demos are
terminal-based `cargo nextest` runs.

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| VHS | 0.10.0 | installed |
| cargo nextest | workspace pin | installed |
| Font | FiraCode Nerd Font Mono | installed |

---

## PR Embedding Snippet

```markdown
## Demo Evidence — PLUGIN-MIGRATION-001-E

| AC | Description | Demo |
|----|-------------|------|
| AC-001 | Plugin loads — WIT + manifest validated | ![AC-001](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-001-plugin-compiles-and-manifest-validates.gif) |
| AC-002 | auth_type_name() returns oauth2_client_credentials | ![AC-002](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-002-auth-type-name-returns-canonical-value.gif) |
| AC-003 | Token acquisition via POST /oauth2/token | ![AC-003](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-003-token-acquisition-via-oauth2-token-endpoint.gif) |
| AC-004 | KV cache hit within TTL | ![AC-004](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-004-token-cached-within-ttl-no-second-request.gif) |
| AC-005 | Expired token → re-acquisition | ![AC-005](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-005-expired-token-triggers-reacquisition.gif) |
| AC-006 | 401 → plugin refresh + retry (VP-150) | ![AC-006](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-006-401-triggers-plugin-token-refresh-and-retry.gif) |
| AC-007 | TOML auth_plugin field; E-SPEC-012 on unknown | ![AC-007](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-007-crowdstrike-toml-declares-auth-plugin.gif) |
| AC-008 | VP-148 DTU parity GREEN after amendment | ![AC-008](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-008-vp148-parity-green-after-toml-amendment.gif) |
| AC-009 | Boot step 7.5 plugin_load_unsigned WARN | ![AC-009](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-009-plugin-loaded-at-boot-step-7-5-emits-warn.gif) |
| AC-010 | access_token absent from tracing output (AD-017) | ![AC-010](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-010-credential-opaqueness-token-not-logged.gif) |
| AC-011 | All 15 story tests PASS | ![AC-011](docs/demo-evidence/PLUGIN-MIGRATION-001-E/AC-011-just-check-workspace-green.gif) |
```
