---
type: pr-review
pass: 2
pr: 154
head_sha: c2ff150e
base: develop @ f19575ff
reviewer_model: claude-opus-4-7
date: 2026-05-25
story: PLUGIN-MIGRATION-001-E
verdict: APPROVE
findings_blocking: 0
findings_warning: 2
findings_nit: 2
---

# PR #154 Pass-2 Review -- PLUGIN-MIGRATION-001-E

## Review Scope

Pass-2 fresh-context review of PR #154 at HEAD `c2ff150e`. Focus: verify pass-1
closure (fix-burst `a759d2b0`), review semver fix (`aaf1b95c`), cargo-deny fix
(`c2ff150e`), and full 8-item checklist evaluation.

68 files changed, +5621 / -149 lines. 35 commits since develop.

## 8-Item Checklist

### 1. Diff Coherence

PASS. All changes relate to PLUGIN-MIGRATION-001-E (CrowdStrike OAuth2 WASM plugin).
The diff includes:
- New plugin crate (`crowdstrike-oauth2`)
- Host-side dispatch (`dispatch_plugin_acquire_token`)
- Boot integration (step 7.5b)
- Validation (`validate_auth_plugin_fields`)
- Error variants (`UnknownAuthPlugin`, `AuthPluginDispatchFailed`, `AuthTokenNotCached`)
- WIT file restructuring (sibling sweep moving types into interface blocks)
- `#[non_exhaustive]` enforcement updates
- CI additions (wasm32-compile-check job)
- Demo evidence (11 ACs)
- Semver fix (InfusionLruCache UnwindSafe) and cargo-deny fix (publish = false)

The WIT restructuring for action/sensor/infusion plugins is a sibling sweep
(F-LP6-MED-002) -- fixing the same top-level-types-outside-interface-block
defect discovered in sensor-auth.wit. This is coherent with the story.

### 2. Description Accuracy

PASS. PR body matches actual changes. Architecture diagram, spec traceability,
test evidence table, convergence statement, and deferred findings all accurately
reflect the diff content. LOCAL convergence at 12 passes / 3-CLEAN strict is
documented with pass-level detail.

### 3. Test Coverage

PASS. Changed lines have comprehensive test coverage:
- 15 story-specific tests (`test_PLUGIN_MIGRATION_001_E_*`)
- 15 unit tests in `crowdstrike-oauth2-plugin/src/lib.rs` (EC-001..006, cache paths, URL encoding)
- 4 boot validation tests (`plugin_boot_tests.rs`)
- 1 tracing emission test (`F_LP7_MED_001`)
- Integration test file: 1572 lines of test code

Edge cases covered: 401, 5xx, 4xx non-401, invalid JSON, missing access_token,
zero expires_in, KV full, cache hit, cache miss, empty cached token, URL-encoded
special characters, missing client_id, missing client_secret.

### 4. Demo Evidence

PASS. `docs/demo-evidence/PLUGIN-MIGRATION-001-E/` contains:
- `evidence-report.md` -- exists
- 11 ACs: each with `.gif` + `.webm` + `.tape` files
- Both success paths (AC-001..008, AC-011) and error/security paths (AC-009 warn, AC-010 credential opaqueness)

### 5. Commit Quality

PASS. Conventional commit format used throughout. Story ID present. Clear messages
describing the specific fix or feature. Fix-burst commits (`a759d2b0`) and follow-up
commits (`aaf1b95c`, `c2ff150e`) are well-labeled.

### 6. Diff Size

WARNING (non-blocking). +5621 lines is above the 500-line threshold. However, this
is justified: the story introduces a complete WASM plugin implementation (1264 lines),
integration tests (1572 lines), boot integration (188 lines), demo evidence artifacts,
and WIT file restructuring. The size is proportional to the feature scope (3-point story
with 11 ACs, a new plugin crate, and cross-cutting boot/validation changes).

### 7. Missing Changes

PASS. All story ACs covered in the diff:
- AC-001: Plugin loads + WIT validation
- AC-002: auth_type_name returns canonical value
- AC-003: Token acquisition via OAuth2
- AC-004: TTL cache hit
- AC-005: Expired token re-acquisition
- AC-006: 401 refresh+retry
- AC-007: TOML auth_plugin field
- AC-008: VP-148 parity GREEN
- AC-009: Boot step 7.5 plugin_load_unsigned WARN
- AC-010: Credential opaqueness
- AC-011: just check GREEN

### 8. Dependency Status

PASS. All upstream dependencies merged:
- S-PLUGIN-PREREQ-D: merged (PR #149)
- S-PLUGIN-PREREQ-E: merged (PR #151)
- PLUGIN-MIGRATION-001-D: merged (PR #153)

## Focus Area Assessment

### CLAUDE.md Convention Compliance

PASS with notes:

- **`#[non_exhaustive]`**: Applied to all new public types: `AuthError`, `HttpResponse`,
  `PluginType`, `LoadedPlugin`, `PluginAuthProvider`. CI gate updated from 32 to 33.
  Compile-fail test crate updated with `v33_loaded_plugin()`.

- **Error handling**: No `unwrap()`/`expect()` on `Result` in production code paths.
  Two `unwrap()` calls exist at lines 373/378 in `url_encode()` but these are on
  `Option<char>` from `char::from_digit()` with values mathematically guaranteed to be
  0..15 (valid hex digits). The function is also `#[cfg(any(target_arch = "wasm32", test))]`.
  This is acceptable -- the unwrap cannot panic.

- **Credential safety**: `PluginAuthProvider` Debug impl shows only structural IDs
  (plugin_id, sensor_id, token_endpoint), never credential values. Test
  `test_debug_shows_structural_ids_not_credentials` asserts `!debug_str.contains("client_secret")`.
  Credentials are resolved from `prism_credentials` at dispatch time only (AD-017).

- **HTTP client timeout**: All `reqwest::Client::builder()` calls use `.timeout(Duration::from_secs(30))`.

- **Structured event catalog (SAP-1)**: Two new `event_type` emissions:
  1. `plugin_auth_token_parse_error` (mod.rs:1160) -- documented as BC-2.16.002 row 37
  2. `plugin_auth_provider_constructed` (boot.rs:331) -- informational boot event

### InfusionLruCache UnwindSafe (aaf1b95c)

PASS. The safety argument is sound:
1. `tokio::sync::Mutex` does not poison on panic (unlike `std::sync::Mutex`)
2. `lru::LruCache<String, LruCacheEntry>` holds only plain data (`serde_json::Value`, `u64`)
3. The `capacity: usize` field has no invariants

The root cause is correctly identified: adding `prism-credentials` pulls
`tokio = { features = ["full"] }` which enables `parking_lot`, changing
`batch_semaphore::Semaphore` internals and transitively removing the auto-derived
`UnwindSafe`. The explicit impl with documented safety rationale is the correct fix.

### cargo-deny publish = false (c2ff150e)

PASS. `publish = false` is correct for `prism-spec-engine` -- it is an internal crate
not intended for crates.io publication. The `cargo-deny` wildcard source ban was failing
because `prism-spec-engine` was technically publishable and its new dep `prism-credentials`
was not from crates.io. `publish = false` makes the crate private, excluding it from the
source ban check. This is consistent with the plugin crate which also has `publish = false`.

## Findings

### Finding 1

| Field | Value |
|-------|-------|
| Severity | WARNING |
| Category | coverage |
| File | `crates/prism-bin/src/boot.rs:331` |
| Finding | New `event_type = "plugin_auth_provider_constructed"` emission in boot.rs may not have a corresponding BC-2.16.002 catalog row. The PR description mentions BC-2.16.002 v1.41 added `plugin.auth_token_parse_error` (row 37), but does not mention `plugin_auth_provider_constructed`. Per SAP-1 standing probe, every `event_type` emission requires a catalog row. |
| Suggestion | Verify BC-2.16.002 has a row for `plugin_auth_provider_constructed`. If absent, add one in a follow-up commit or the next story touching BC-2.16.002. This is a diagnostic/informational event (not an error audit event), so it is lower severity than a missing error-path emission, but SAP-1 makes no distinction. |

### Finding 2

| Field | Value |
|-------|-------|
| Severity | WARNING |
| Category | coverage |
| File | `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs:373,378` |
| Finding | Two `unwrap()` calls in `url_encode()` on `char::from_digit()` results. While mathematically safe (input is always 0..15), the workspace clippy config has `unwrap_used = "deny"`. These survive because the function is `#[cfg(any(target_arch = "wasm32", test))]` and the WASM target is not linted by the host-target clippy job. The wasm32-compile-check CI job runs `cargo check` (not clippy), so these are invisible to all lint gates. |
| Suggestion | No immediate action required -- these are mathematically infallible. However, for consistency with the workspace lint policy, consider replacing with `.expect("hex digit 0..15 is always valid")` or `// SAFETY: ...` comment + `#[allow(clippy::unwrap_used)]` on the specific lines. This can be addressed in a future story (e.g., S-PLUGIN-CI-001 which adds the full wasm32 build pipeline). |

### Finding 3

| Field | Value |
|-------|-------|
| Severity | NIT |
| Category | coherence |
| File | `crates/prism-bin/tests/plugin_boot_tests.rs:1326` |
| Finding | The test `test_plugin_auth_provider_construction_production_api` passes `credential_handle = "sensor:crowdstrike"` as the `sensor_id` parameter to `PluginAuthProvider::new`. After ADR-028 section D11 Option C, the second parameter is `sensor_id` (not `credential_handle`). The variable name `credential_handle` is a holdover from the pre-Option-C design. |
| Suggestion | Rename the local variable from `credential_handle` to `sensor_id` for clarity. The code is functionally correct (it is just a String passed to `new`), but the naming is misleading given that Option C explicitly removed the credential_handle concept. |

### Finding 4

| Field | Value |
|-------|-------|
| Severity | NIT |
| Category | size |
| File | Multiple |
| Finding | PR is +5621 lines (exceeds 500-line guidance). Acknowledged and justified in checklist item 6 above. |
| Suggestion | No action needed. The size is proportional to the feature (new crate + integration tests + demo evidence). |

## Pass-1 Closure Verification

The fix-burst at `a759d2b0` is the primary closure commit for pass-1 findings. Based
on the diff review:

1. **ADR-028 section D11 Option C credential substitution**: Fully implemented.
   `PluginAuthProvider` resolves credentials via `prism_credentials::resolve_credential`
   and injects into `PluginConfigMap`. No `credential_handle` string in WIT params.

2. **`#[non_exhaustive]` on PluginType**: Applied. CI gate incremented to 33.

3. **Component Model export reflection**: `extract_component_exports()` added in
   discovery.rs using `Component::component_type().exports()` reflection API.

4. **KV store Arc sharing (F-LP2-CRIT-001)**: `kv_store` field on `LoadedPlugin`,
   cloned in `make_host_state`. Shared across dispatch calls.

5. **UnknownAuthPlugin boot validation (F-LP2-CRIT-002)**: `validate_and_construct_auth_providers`
   extracted as testable function. Exit code 2 tested.

6. **URL encoding for form body**: `url_encode()` function added with test coverage
   for special characters.

7. **Semver fix (aaf1b95c)**: `InfusionLruCache` `UnwindSafe` impl with safety argument.

8. **cargo-deny fix (c2ff150e)**: `publish = false` added to `prism-spec-engine`.

All pass-1 findings appear to be addressed. No regressions detected in the fix-burst
or follow-up commits.

## Verdict

**APPROVE**

The PR implements a well-structured WASM plugin for CrowdStrike OAuth2 authentication
with comprehensive test coverage (15+ story tests, 15 unit tests, 4 boot tests),
proper error handling, credential safety (AD-017), `#[non_exhaustive]` compliance,
and 11/11 AC demo evidence recordings. The fix-burst and follow-up commits cleanly
address semver and cargo-deny CI regressions.

Two WARNING-level findings (SAP-1 catalog completeness for `plugin_auth_provider_constructed`
and `unwrap()` in wasm32-only `url_encode`) are non-blocking. Both can be addressed in
follow-up stories.

```
CLEAN (strict): no  (2 WARNING + 2 NIT)
CLEAN (PR-merge): yes  (0 BLOCKING findings)
```
