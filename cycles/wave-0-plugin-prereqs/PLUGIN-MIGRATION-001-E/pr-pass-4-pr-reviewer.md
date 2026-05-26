---
type: pr-review
pass: 4
pr: 154
head_sha: 63f95759
base: develop @ f19575ff
reviewer_model: claude-opus-4-7
date: 2026-05-25
story: PLUGIN-MIGRATION-001-E
verdict: APPROVE
findings_blocking: 0
findings_warning: 0
findings_nit: 0
---

# PR #154 Pass-4 Review -- PLUGIN-MIGRATION-001-E

## Review Scope

Pass-4 fresh-context review of PR #154 at HEAD `63f95759`. Focus: verify SEC-008
fix (PluginConfigMap values changed from `String` to `SecretString` for CWE-316
closure), sibling-sweep completeness of the type change, and regression check
against the full PR diff.

68 files changed, +5667 / -152 lines. 37 commits since develop.
CI: 38/38 checks GREEN.

## SEC-008 Fix Verification

### Problem Statement

SEC-008 identified that `PluginConfigMap` was `HashMap<String, String>`. When
`plugin_auth_provider.rs` called `make_host_state(config.clone())`, the `config`
was cloned into an `Arc<PluginConfigMap>` stored in `HostState.config`. The prior
SEC-005 fix added explicit `v.zeroize()` calls on the caller's copy, but the
Arc-cloned copy in `HostState` was NOT zeroized on drop -- credential bytes
(`client_id`, `client_secret`) persisted in heap memory after the `Arc` dropped.

### Fix Assessment

The fix changes `PluginConfigMap` from `HashMap<String, String>` to
`HashMap<String, SecretString>`. `SecretString` (from `secrecy 0.8.0`, which
depends on `zeroize`) implements `Drop` that calls `zeroize()` on the inner
`String`. This means ALL copies of credential values -- caller's copy, the
`Arc::clone` in `HostState.config`, any intermediate copies -- are automatically
zeroized when their last reference is dropped.

This is correct-by-construction: no copy can escape zeroization regardless of
how many Arc clones or HashMap clones exist.

### Changes Verified (4 files, +51/-30)

**1. `plugin/loader.rs` (type alias change)**
- Line 20: `pub type PluginConfigMap = HashMap<String, SecretString>;`
- `secrecy::SecretString` import added at line 9
- Doc comment updated to explain the SEC-008 rationale and AD-017 expose_secret() discipline
- Note added at line 186-187 for test authors about SecretString wrapping
- PASS: type change is correct, doc comment is accurate

**2. `plugin/host_functions.rs` (expose_secret at read site)**
- Lines 278-280: `host_get_config` now calls `.expose_secret().to_owned()` instead of `.cloned()`
- Doc comment (lines 271-277) explains SEC-008/AD-017: expose at the last possible
  moment before handing the plaintext to the WASM guest via the WIT result slot
- `use secrecy::ExposeSecret;` scoped to the function body
- PASS: the expose site is correct -- the plaintext String exists only in the
  `host_get_config` return value, which is consumed by the `register_host_functions`
  closure and dropped after copying into the `Val::String` result slot

**3. `plugin_auth_provider.rs` (SecretString construction + zeroize removal)**
- Lines 155-169: `PluginConfigMap::from([...])` now wraps all values in
  `SecretString::new(...)` -- including `token_endpoint` (non-credential, but
  uniform wrapping avoids per-key sensitivity tracking; negligible overhead)
- Lines 143-154: comment block updated to explain the correct-by-construction
  approach vs the prior explicit-zeroize approach
- Lines 184-185: comment documents that config drops and zeroizes automatically
- The three explicit `config.get_mut("client_id").zeroize()` / `client_secret` /
  `use zeroize::Zeroize` calls are REMOVED -- SecretString handles this
- `config` is no longer `let mut` (was `let mut config` for SEC-005; now `let config`)
- PASS: the removal of explicit zeroize is correct because SecretString handles
  it unconditionally. The `use secrecy::{ExposeSecret, SecretString}` import is
  correctly scoped to the async block

**4. `tests/crowdstrike_oauth2_plugin_tests.rs` (test value wrapping)**
- Lines 1493-1504: test values wrapped in `SecretString::new(...)` to match the
  new `PluginConfigMap` type
- `use secrecy::SecretString;` added at line 1493
- PASS: test sentinel values ("id", "secret", URL) are correctly wrapped

### Sibling-Sweep Verification (TD-VSDD-060)

Searched all `PluginConfigMap` usage sites across the workspace:

| Site | Type | Status |
|------|------|--------|
| `plugin/loader.rs:20` | Type alias definition | Updated to `SecretString` |
| `plugin/loader.rs:142` | `HostState.config` field type | `Arc<PluginConfigMap>` -- unchanged, inherits new type |
| `plugin/loader.rs:179` | `test_default()` | `PluginConfigMap::new()` -- empty map, no values to wrap |
| `plugin/mod.rs:658` | `dispatch_plugin_acquire_token` param | `&PluginConfigMap` -- unchanged, inherits new type |
| `plugin/mod.rs:816,837,920` | Other dispatch methods | `&PluginConfigMap` param -- unchanged, inherits |
| `plugin/mod.rs:1068,1091,1114` | Test/stub dispatch paths | `_config: &PluginConfigMap` -- unused param, inherits |
| `plugin/host_functions.rs:278-280` | `host_get_config` read site | Updated to `expose_secret()` |
| `plugin/sandbox.rs:66` | `create_store_with_limit` | `PluginConfigMap::new()` -- empty map, no wrapping needed |
| `plugin_auth_provider.rs:156` | Production config construction | Updated to `SecretString::new(...)` |
| `tests/crowdstrike_oauth2_plugin_tests.rs:1494` | Test config construction | Updated to `SecretString::new(...)` |
| `tests/plugin_tests.rs:282,950,978,1014` | Test imports | Import only, no value construction |
| `tests/plugin_integration_tests.rs:132,325` | Test config | `PluginConfigMap::new()` -- empty map |
| `plugins/crowdstrike-oauth2/src/lib.rs:674,682` | Guest MockHost | Different type (`HashMap<String, String>` inside WASM guest sandbox -- NOT host PluginConfigMap) |

All 14 usage sites verified. No site constructs a `PluginConfigMap` with plain
`String` values. The sibling sweep is complete.

### Dependency Verification

- `secrecy = { version = "0.8" }` declared in `crates/prism-spec-engine/Cargo.toml` (line 21)
- `Cargo.lock` resolves to `secrecy 0.8.0` which depends on `zeroize`
- `zeroize = "1"` remains as a direct dependency (still used by `auth_provider.rs:30`
  for `Zeroizing` wrapper on `AuthToken`)
- No orphaned dependencies introduced or removed

## 8-Item Checklist

### 1. Diff Coherence

PASS. All 68 changed files relate to PLUGIN-MIGRATION-001-E scope. The SEC-008 fix
at `63f95759` modifies 4 files (+51/-30 lines), all within the plugin subsystem.
No unrelated changes.

### 2. Description Accuracy

PASS. PR body accurately describes the story scope. The SEC-008 fix commit message
(conventional format, CWE reference, detailed change list) accurately describes the
type-level change and its security rationale. The PR body was written before the
SEC-008 fix and does not mention it, but the commit message serves as the self-
contained description for this incremental change.

### 3. Test Coverage

PASS. The SEC-008 type change is enforced at compile time -- any code that constructs
a `PluginConfigMap` with plain `String` values will fail to compile. The existing
test suite (436 prism-spec-engine tests + 67 prism-bin tests per commit message)
exercises all code paths through the new type. The single test that constructs a
non-empty `PluginConfigMap` (`test_F_LP7_MED_001`) is updated to use `SecretString::new(...)`.

### 4. Demo Evidence

PASS. `docs/demo-evidence/PLUGIN-MIGRATION-001-E/` contains `evidence-report.md`
plus 11 ACs x 3 files (`.gif` + `.webm` + `.tape`). 34 files total. All ACs covered
including success paths (AC-001..008, AC-011) and error/security paths (AC-009, AC-010).

### 5. Commit Quality

PASS. 37 commits, all conventional commit format. The SEC-008 commit uses
`fix(prism-spec-engine):` prefix with CWE-316 reference in the subject and a
detailed body explaining the problem, fix, and per-file changes. Story ID
(PLUGIN-MIGRATION-001-E) is present in feature commits.

### 6. Diff Size

Observation (non-finding). +5667 lines total. Size is proportional to scope (new
WASM plugin crate + integration tests + demo evidence + cross-cutting integration).
Acknowledged in pass-3; no change in assessment.

### 7. Missing Changes

PASS. All 11 story ACs remain implemented and tested. The SEC-008 fix does not
remove any functionality -- it strengthens the credential safety guarantee.

### 8. Dependency Status

PASS. All upstream dependencies merged (PREREQ-D #149, PREREQ-E #151, 001-D #153).

## CLAUDE.md Convention Compliance (SEC-008 Delta)

### Credential safety (AD-017)
PASS. The SEC-008 fix strengthens AD-017 compliance: credential values are now
`SecretString` throughout the entire lifecycle -- from `PluginAuthProvider::acquire_token`
construction through `make_host_state` cloning to `host_get_config` exposure. The
`expose_secret()` call in `host_get_config` is documented as the last-possible-moment
exposure site.

### Error handling
PASS. No new `unwrap()` or `expect()` introduced by SEC-008.

### Arc-DI plumbing
PASS. `Arc<PluginConfigMap>` in `HostState.config` is unchanged; the `Arc::clone`
now carries `SecretString` values that auto-zeroize.

### Forbidden patterns
PASS. No forbidden patterns introduced.

## Pass-3 Finding Disposition

### NIT-1 (diff size) from pass-3
Status: CLOSED. This was an observation about PR size (+5643 lines) being above
the 500-line guidance. Pass-3 already justified it as proportional to scope. The
SEC-008 fix adds +51/-30 lines (net +21), bringing the total to +5667/-152. The
additional 21 lines do not change the assessment. This NIT is inherent to the
feature scope and not actionable.

### PR-WRN-1 (SAP-1: plugin_auth_provider_constructed catalog row) from pass-2/3
Status: CLOSED for PR-reviewer purposes. This was carried as a NIT in pass-3
regarding the `event_type = "plugin_auth_provider_constructed"` emission at
`boot.rs:331`. As a PR-reviewer behind the information wall, I cannot verify
BC-2.16.002 catalog content. The emission is `tracing::info!` level for
boot-sequence diagnostics. Pass-3 classified this as NIT (non-blocking) and
I concur -- it is an informational event that does not affect merge readiness.

## Findings

No findings. Zero blocking, zero warning, zero nit.

The SEC-008 fix is architecturally correct, properly sibling-swept, well-documented,
and does not introduce any regressions. The type-level enforcement (`SecretString`
instead of `String`) makes the zeroize guarantee structural rather than procedural --
it is impossible to accidentally skip zeroization of credential values in
`PluginConfigMap` because the type system enforces it.

## Verdict

**APPROVE**

The SEC-008 fix correctly addresses CWE-316 (cleartext storage of sensitive
information in memory) by changing `PluginConfigMap` values from `String` to
`SecretString`. The fix is correct-by-construction: every copy of credential values
(caller copy, Arc clone in HostState, any intermediate) auto-zeroizes on drop via
the `secrecy` crate's `Drop` implementation. The sibling sweep is complete (14
usage sites verified). No regressions introduced.

All prior findings from passes 1-3 are closed. No new findings at any severity.

```
CLEAN (strict): yes  (0 findings at any severity)
CLEAN (PR-merge): yes  (0 BLOCKING, 0 WARNING findings)
```
