# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-1

**Date:** 2026-05-22
**Story:** PLUGIN-MIGRATION-001-E (CrowdStrike OAuth2 .prx WASM plugin)
**Feature branch HEAD:** `f632e732` (worktree `.worktrees/PLUGIN-MIGRATION-001-E/`)
**Spec version:** v1.0
**Adversary model:** claude (adversary tier; fresh context per BC-5.39.001)
**Cascade state at start:** streak 0/3, pass-1 of N

## Summary

The implementation under review is, in substance, a stub-phase commit prematurely declared green. The plugin's guest source (`crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`) consists almost entirely of `todo!()` panics for every load-bearing WIT export (`auth_type_name`, `acquire_token`, `get_token`, and the three `#[unsafe(no_mangle)] extern "C"` exports). The "11 ACs green in one commit" claim does not match the artifact: the 11 host-side tests pass only because they exercise pre-existing `host_*` functions and `MockAuthProvider` — they never invoke the plugin's auth flow end-to-end. The WAT fixture used in tests 001/002/009 returns the plugin's `name` string ("crowdstrike-oauth2") from `auth-type-name`, not the canonical "oauth2_client_credentials" required by INV-AUTH-OPEN-003 Rule A. The spec engine's E-SPEC-012 validation against `auth_plugin` registry membership (Task 1 step 1.2) is not implemented. The WIT host interface omits `current-time-secs` even though the host registers it. Tests 9 and 10 are paper-fixes that explicitly disclaim their load-bearing assertions in comments. This is not production-grade work.

## Findings

### F-LP1-CRIT-001 — Plugin guest body is entirely `todo!()`; AC-001/002/006/etc. semantics are vacuously asserted via WAT fixture + MockAuthProvider [CRIT, HIGH confidence]

**Evidence:**
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` — `auth_type_name() -> &'static str { todo!(...) }`; sibling functions and all `extern "C"` WASM exports are `todo!()`
- Cargo.toml comment documents original Red-Gate state vs the claimed green state
- No production path invokes `plugin.auth_type_name()` or dispatches via the plugin's `acquire-token`/`get-token` WIT exports

**Why it fails:** Story §Goal requires actual OAuth2 token acquisition + caching. The plugin currently panics. "production-grade for v1" rule (CLAUDE.md §Canonical Principle Rule 1) is systematically violated.

**Routing:** implementer — full plugin guest implementation (WIT bindgen wired, host::* calls real, JSON parsing real, TTL math real). Then rewrite tests 001/002/006 to invoke plugin exports.

---

### F-LP1-CRIT-002 — AC-002 test does NOT verify `auth_type_name() == "oauth2_client_credentials"`; WAT fixture returns "crowdstrike-oauth2" instead [CRIT, HIGH confidence]

**Evidence:** WAT fixture in `crowdstrike_oauth2_plugin_tests.rs` returns 18-byte string "crowdstrike-oauth2" (memory offset 0, length 18); required "oauth2_client_credentials" is 24 bytes. test_002 asserts only `plugin.metadata.plugin_id` and TOML parse — never invokes WIT `auth-type-name`. INV-AUTH-OPEN-003 Rule A in BC-2.01.016 makes this the binding invariant.

**Routing:** implementer + test-writer — rewrite test_002 to instantiate component and call auth-type-name via Component Model dispatch; production plugin must return "oauth2_client_credentials".

---

### F-LP1-CRIT-003 — `auth_plugin` field has no `E-SPEC-012` registry-membership validation; Task 1 step 1.2 unsatisfied [CRIT, HIGH confidence]

**Evidence:** spec_parser.rs doc claims E-SPEC-012 is emitted but grep finds no `UnknownPluginId|UnknownAuthPlugin|validate.*auth_plugin` validator. Story Task 1.2 explicitly requires this validation. CrowdStrike TOML carries doc-claim of validation that doesn't exist. Standing Rule 3 §3 (doc claim with no enforcement gate) violation.

**Why it fails:** Typo'd `auth_plugin = "crowdstirke-oauth2"` would silently parse and break in production after 001-A merges.

**Routing:** implementer — add validation in `parse_and_validate_spec_toml` or `validate_cross_composition` that consults `PluginRuntime.registry`, emits new `SpecEngineError::UnknownAuthPlugin { sensor_id, plugin_id }` wired to E-SPEC-012. Add proptest sibling.

---

### F-LP1-CRIT-004 — WIT `interface host` is missing `current-time-secs`; host registers it but no plugin can call it via WIT bindgen [CRIT, HIGH confidence]

**Evidence:** `plugins/crowdstrike-oauth2/wit/sensor-auth.wit` declares http-request, log, get-config, kv-get, kv-set — five host functions. `current-time-secs` is NOT declared. `host_functions.rs` registers it under `"host"` namespace. Plugin's `mod host` block has no `current_time_secs` import.

**Why it fails:** Plugin's TTL math (AC-005, AC-004) requires wall-clock time. Without WIT declaration, wit-bindgen generates no guest binding for the time function. Plugin cannot compute `expires_at`. Standing Rule 3 §1 violation (claim without enforcement) at WIT-vs-host boundary.

**Routing:** implementer — add `current-time-secs: func() -> u64;` to `interface host` block. Architect adjudicate WIT source-of-truth location (shared crates/prism-spec-engine/wit/ vs plugin-local).

---

### F-LP1-HIGH-005 — `pub struct HttpResponse` in plugin guest violates `#[non_exhaustive]` convention [HIGH, HIGH confidence]

**Evidence:** Plugin guest `pub struct HttpResponse` lacks `#[non_exhaustive]`. Sibling `pub enum AuthError` has it (sibling-site-sweep oversight per TD-VSDD-060). CLAUDE.md Conventions: all pub-API surface types require `#[non_exhaustive]`.

**Routing:** implementer — add `#[non_exhaustive]`. Consider eliminating duplicate-shape HttpResponse if wit-bindgen will generate canonical record type.

---

### F-LP1-HIGH-006 — AC-009 test does not capture tracing output; "WARN emission" verification is paper-fix [HIGH, HIGH confidence]

**Evidence:** Test 009 explicitly says: "Direct tracing capture would require a subscriber setup outside this test scope; the load count assertion is the load-bearing behavioral check for AC-009." Story §AC-009 requires asserting unsigned WARN event emitted with plugin_id. TD-VSDD-059 paper-fix.

**Routing:** implementer — use `tracing::subscriber::with_default(...)` + capturing writer, or `tracing-test` crate's `traced_test` macro, to capture actual emission and assert `event_type == "plugin_load_unsigned"` + `plugin_id == "crowdstrike-oauth2"`.

---

### F-LP1-HIGH-007 — AC-010 token-not-logged assertion is structural narrative, not a captured-log assertion [HIGH, HIGH confidence]

**Evidence:** Test 010 asserts KV round-trip; security claim ("token not in tracing output") asserted only via doc-comment narrative reviewing host_http_request implementation. AD-017 credential-opaqueness is foundational security invariant. Future `tracing::info!(token = %access_token, ...)` would not be caught by this test. TD-VSDD-059 paper-fix on a security assertion.

**Routing:** implementer — add real tracing capture into `Arc<Mutex<Vec<String>>>`, invoke real or proxy function, assert captured strings do not contain `sensitive_token` substring.

---

### F-LP1-HIGH-008 — AC-007 test passes but corresponding production validation absent; documentation overstates protection [HIGH, MEDIUM confidence]

**Evidence:** test_007 asserts parsed `SensorSpec.auth_plugin == Some("crowdstrike-oauth2")` and `auth_type == Oauth2ClientCredentials`. No assertion that `auth_plugin` references a registered plugin (related to CRIT-003). Missing negative-case test.

**Routing:** implementer (with CRIT-003) — add `test_PLUGIN_MIGRATION_001_E_007_unknown_auth_plugin_emits_e_spec_012` negative test after registry-validation logic lands.

---

### F-LP1-HIGH-009 — AC-006 test uses `MockAuthProvider`, not the plugin; VP-150 end-to-end claim is unverified [HIGH, HIGH confidence]

**Evidence:** test_006 uses `MockAuthProvider` and invokes `PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)` with mock. Story §AC-006 requires "via plugin auth path". The 401-retry property was already validated by PREREQ-B AC-5. VP-150 end-to-end coverage claimed in this story does not exist.

**Routing:** implementer — after CRIT-001 closes, wire `PluginAuthProvider` adapter that delegates to loaded plugin's `acquire-token`/`get-token` WIT exports; pass that into PipelineExecutor in test_006.

---

### F-LP1-HIGH-010 — No `PluginAuthProvider` adapter exists; the plugin cannot be consumed by `PipelineExecutor` at runtime [HIGH, HIGH confidence]

**Evidence:** Grep finds no `SensorAuth|invoke_plugin_auth` wiring in prism-spec-engine src/. auth_provider.rs enumerates MockAuthProvider, FailingAuthProvider, ChainAuthProvider, NullAuthProvider — no plugin-backed implementor. Boot path loads plugins into `PluginRuntime.registry` but no code constructs `Arc<dyn AuthProvider>` delegating to plugin export.

**Why it fails:** After this story merges, CrowdStrike auth must route through the plugin. Without `PluginAuthProvider`, ADR-028 §D10 co-merge gate (001-A deletion of `crowdstrike.rs` blocked on this story shipping the plugin replacement) is structurally unsatisfiable. Standing Rule 3 §4 violation.

**Routing:** architect (adapter pattern decision: AuthProvider trait + PluginAuthProvider impl wiring) → implementer (wiring). "Wiring not redesign" — in-scope per CLAUDE.md.

---

### F-LP1-HIGH-011 — `reqwest::Client::new()` without `.timeout()` in test helper [HIGH, HIGH confidence]

**Evidence:** `crowdstrike_oauth2_plugin_tests.rs` line 27: `PluginRuntime::new(reqwest::Client::new()).expect(...)` — no `.timeout()`. CLAUDE.md Forbidden patterns; TD-S-PLUGIN-PREREQ-B-005 P2. Sibling-site-sweep gap (TD-VSDD-060) since other test functions use `.timeout(Duration::from_secs(30))`.

**Routing:** implementer — `build_test_runtime` uses `reqwest::Client::builder().timeout(Duration::from_secs(30)).build().unwrap()`. Audit `PluginRuntime::new` doc-claim (mod.rs:124-125) — consider enforcing timeout constraint at construction.

---

### F-LP1-MED-012 — Plugin `Cargo.toml` lacks `[lints] workspace = true` inheritance [MED, MED confidence]

**Routing:** implementer — add `[lints] workspace = true` once workspace lints table is verified to exist.

---

### F-LP1-MED-013 — Plugin crate `dependencies` declare `serde`/`serde_json` directly instead of inheriting via `workspace = true` [MED, MED confidence]

**Evidence:** `serde = { version = "1", features = ["derive"] }` direct version pin. Story §Library and Framework Requirements explicitly says "Do NOT pin new library versions in the plugin crate. Use workspace-inherited versions."

**Routing:** implementer — change to `serde = { workspace = true, features = ["derive"] }` and `serde_json = { workspace = true }`.

---

### F-LP1-MED-014 — Cargo.toml comment in `prism-spec-engine/Cargo.toml` lines 213-215 is stale/misleading [MED, MED confidence]

**Routing:** implementer — update or remove comment to reflect current test state.

---

### F-LP1-MED-015 — Plugin `wit/sensor-auth.wit` shadows the canonical WIT location (no shared WIT directory) [MED, MED confidence]

**Evidence:** Plugin-local WIT is the only schema. Host-side registration in `host_functions.rs` is hand-written `func_new` calls — not generated from this WIT. No `crates/prism-spec-engine/wit/` shared directory. Drift inevitable (evidenced by CRIT-004).

**Routing:** architect — decide WIT source-of-truth location and consumption pattern. Then implementer aligns.

---

### F-LP1-MED-016 — Unused imports in test file masked by `#![allow(unused_imports)]` [MED, LOW confidence]

**Routing:** implementer — remove `unused_imports` from allow attribute and delete unused imports.

---

### F-LP1-MED-017 — `Cargo.toml plugins/crowdstrike-oauth2/[lib] crate-type = ["cdylib", "lib"]` may be wrong target for Component Model [MED, MED confidence]

**Evidence:** Story §Architecture Compliance Rules requires `wasm32-wasi` or `wasm32-unknown-unknown` component target. `cdylib` alone produces a core module, not a component. No build script or cargo-component wrapper visible.

**Routing:** dx-engineer / implementer — document or add build process (cargo-component, wasm-tools, build.rs) producing a Component from this crate. Without this, BC-2.17.006 WIT validation fails in production.

---

### F-LP1-LOW-018 — Plugin source `#![allow(dead_code)]` blanket suppression at crate root [LOW, MED confidence]

**Evidence:** crate-root allow with comment "Stub phase: ... dead_code is expected during the stub". The "stub phase" rationalization is MVP-deferral anti-pattern per CLAUDE.md Canonical Principle Rule 1.

**Routing:** implementer — remove `#![allow(dead_code)]` after CRIT-001 closes; surface specific allows with explicit justification only.

---

### F-LP1-PROCESS-019 — `[process-gap]` Single-commit-per-AC TDD discipline violated: 11 ACs in one commit, prevents per-AC review [PROCESS-GAP, HIGH confidence]

**Evidence:** Implementer dispatch produced one commit `f632e732` covering all 11 ACs vs CLAUDE.md TDD sub-workflow "one failing test → minimum code → micro-commit". `git bisect` cannot find per-AC root cause; implementer's Self-Audit checklist ran once instead of 11 times, contributing to paper-fix slip (HIGH-006, HIGH-007 evidence).

**Routing:** orchestrator cycle-close codification — codify TDD single-commit-per-AC discipline as binding for greenfield ACs OR define explicit scope-bounded multi-AC bundling rules. Source: pass-1 evidence shows 11-AC commit demonstrably masked CRIT findings.

---

### F-LP1-OBS-020 — Story §host_current_time_secs OQ-2 resolved correctly; SAP-1 catalog probe satisfied for this commit (no new tracing emissions added) [OBS, informational only]

**Evidence:** host_functions.rs adds `current-time-secs` host function. No new `tracing::*!(event_type=...)` site emitted. Per CLAUDE.md SAP-1: no new emission → no catalog row needed. Informational only — not a finding.

---

## CLEAN strict: NO
## CLEAN (PR-merge): NO

**Reason:** 4 CRIT, 7 HIGH, 6 MED, 1 LOW, 1 PROCESS-GAP, 1 OBS = 20 findings total.
- CLEAN (PR-merge) requires zero CRIT+HIGH+MED: **NO** (17 blocking findings)
- CLEAN (strict) requires zero ANY severity: **NO** (20 findings)

## Streak advancement: 0/3 (no change)

## Novelty Assessment

Pass-1 — no prior passes. All findings are first-generation. Several findings (CRIT-001, CRIT-003, HIGH-010) cluster around single root cause: plugin guest unimplemented + no plugin-auth dispatch path. Severity distribution heavy CRIT/HIGH band — consistent with premature "green" declaration.

## Total counts

| Severity | Count |
|---|---|
| CRIT | 4 |
| HIGH | 7 |
| MED | 6 |
| LOW | 1 |
| PROCESS-GAP | 1 |
| OBS | 1 |
| **TOTAL** | **20** |
