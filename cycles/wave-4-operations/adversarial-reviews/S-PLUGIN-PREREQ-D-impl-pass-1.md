---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-D
pass_id: impl-pass-1
pass_type: LOCAL-implementation
date: 2026-05-14
base_develop: 95d46be2
feature_branch_head_at_pass: 08d084fa
verdict: BLOCKED
streak_before: "0/3"
streak_after: "0/3"
findings_crit: 3
findings_high: 6
findings_med: 7
findings_low: 2
findings_obs: 3
findings_kudo: 2
findings_total_in_perimeter: 18
policies_applied: 18
closure_burst: fix-burst-impl-1
closure_decision: D-548
closure_factory_sha: 9b2b4823
producer: adversary
reified_by: state-manager
---

# S-PLUGIN-PREREQ-D Adversary impl-pass-1 — BLOCKED

> **Note:** This is a structured summary reconstruction. The full adversary report was captured in
> orchestrator conversation context at D-548. This file follows the standard pass-report convention
> per BC-5.39.001 cascade protocol.

## Pass Summary

| Field | Value |
|-------|-------|
| Pass ID | impl-pass-1 (LOCAL implementation cascade, first pass) |
| Date | 2026-05-14 |
| Base develop | 95d46be2 |
| Feature branch HEAD at pass | 08d084fa (Implementer TDD green — D-547) |
| Verdict | **BLOCKED** |
| 3-CLEAN streak | 0/3 → 0/3 |
| Policies applied | 18 (POL-1..POL-15 + POL-20 + POL-22 + BC-5.39.001) |

## Findings

### CRITICAL (3)

**F-IMPL-LP1-CRIT-001** — `run_boot_sequence` did NOT call `plugin_load_step`

- Severity: CRITICAL
- Location: `crates/prism-bin/src/boot.rs` (approximate)
- Policy: POL-15 boot-step registration discipline
- Description: The boot sequence function was wired with correct boot-step infrastructure but `plugin_load_step` was never invoked in the actual call chain. The step was registered but not called — the canonical "registered but not called" anti-pattern that POL-15 targets.
- Impact: Plugin loading never executed at runtime despite appearing wired.
- Closure: `0e0c85d0` — run_boot_sequence wiring + integration test verifying plugin_load_step is called

**F-IMPL-LP1-CRIT-002** — `PrismConfig` missing `plugin_dir` field

- Severity: CRITICAL
- Location: `crates/prism-core/src/config.rs` (approximate)
- Policy: AC-1 contract (plugin directory configuration)
- Description: `PrismConfig` struct lacked the `plugin_dir: PathBuf` field required by AC-1. The boot sequence had no path to resolve the plugin directory. Additionally `#[non_exhaustive]` was missing from `PrismConfig` and related config types, violating CLAUDE.md `#[non_exhaustive]` discipline.
- Impact: AC-1 contract entirely unsatisfied; plugin loading impossible.
- Closure: `0e0c85d0` — PrismConfig.plugin_dir field added + #[non_exhaustive] on PrismConfig family

**F-IMPL-LP1-CRIT-003** — `register_host_functions` was a no-op stub

- Severity: CRITICAL
- Policy: TD-VSDD-059 paper-fix detection
- Description: The function `register_host_functions` existed and was called, but its body was empty or returned immediately without registering any WASM host functions. This is the archetypal paper-fix: the call site is satisfied but the behavioral contract is not. The function must register the 5 specified host functions.
- Impact: All WASM plugins would have no access to host capabilities (HTTP, KV, audit, etc.).
- Closure: `30a7a304` — register_host_functions registers 5 non-stub host functions; WASI-importing component negative-proof test verifies WASI is not accidentally linked

### HIGH (6)

**F-IMPL-LP1-HIGH-001** — BC-2.16.002 catalog not updated for new impl-introduced event_type rows

- Severity: HIGH
- Policy: PG-LP11-001 structured event catalog discipline
- Description: The implementation introduced additional `tracing::*!(event_type=…)` emission sites during TDD that were not present in the factory spec amendments committed during the spec cascade. BC-2.16.002 v1.14 was missing rows for the new impl-phase emission sites.
- Closure: `9b2b4823` (factory in-burst spec amendments) — BC-INDEX v4.76→v4.77; BC-2.16.002 v1.14→v1.15 (3 new catalog rows + `message` field + `sensor_id` correction; total 28→31 rows)

**F-IMPL-LP1-HIGH-002** — AC-4 audit emitter wiring missing (`tracing::warn` ≠ audit channel)

- Severity: HIGH
- Policy: AC-4 durable audit trail
- Description: The implementation used `tracing::warn!` for plugin load audit events instead of routing through the proper audit channel (RocksDB-backed `PluginLoadAuditSink`). `tracing::warn` is volatile (lost on restart); AC-4 requires durable audit storage.
- Closure: `73d72f03` — PluginLoadAuditSink trait + RocksDbPluginAuditSink + PluginRuntime::new_with_audit_sink constructor + durable RocksDB audit test

**F-IMPL-LP1-HIGH-003** — `semver::Version` validation used bespoke `is_valid_semver` instead of `semver::Version::parse`

- Severity: HIGH
- Description: A custom `is_valid_semver` function was implemented rather than using the `semver` crate's `Version::parse`. The custom function missed edge cases and diverged from the semver spec. The `ManifestParseError` (E-PLUGIN-017) was also missing from the error taxonomy.
- Closure: `1d620e63` + `9b2b4823` — `semver::Version::parse` replaces `is_valid_semver`; E-PLUGIN-017 ManifestParseError added to taxonomy + BC-2.16.002 catalog row

**F-IMPL-LP1-HIGH-004** — `semver::Version::parse` not used (duplicate of HIGH-003 resolution path, kept for finding completeness)

- Severity: HIGH
- Closure: `1d620e63` — semver::Version::parse validation

**F-IMPL-LP1-HIGH-005** — `ManifestNotFound` error code E-PLUGIN-018 missing from taxonomy

- Severity: HIGH
- Description: When a plugin directory exists but no manifest file is found, the error `ManifestNotFound` was not registered in the error taxonomy, violating the error taxonomy completeness discipline.
- Closure: `1d620e63` + `9b2b4823` — E-PLUGIN-018 ManifestNotFound added to taxonomy + catalog row

**F-IMPL-LP1-HIGH-006** — `FormatVersionMissing` error code E-PLUGIN-019 missing from taxonomy

- Severity: HIGH
- Description: When a manifest is missing the required `format_version` field, the error `FormatVersionMissing` was not registered in the error taxonomy.
- Closure: `1d620e63` + `9b2b4823` — E-PLUGIN-019 FormatVersionMissing added to taxonomy + catalog row

### MEDIUM (7)

**F-IMPL-LP1-MED-001** — BC-2.16.002 catalog rows missing `message` field

- Severity: MEDIUM
- Description: Several catalog rows in BC-2.16.002 lacked the `message` field that describes the human-readable event message accompanying the structured event_type.
- Closure: `9b2b4823` — `message` field added to all affected rows in BC-2.16.002 v1.15

**F-IMPL-LP1-MED-002** — `sensor_id` field incorrect in catalog rows (was `plugin_id`)

- Severity: MEDIUM
- Description: Several catalog rows used `plugin_id` as the field name where `sensor_id` was the canonical name established by the sensor adapter contract.
- Closure: `1d620e63` (code) + `9b2b4823` (catalog) — sensor_id correction applied in both code and BC-2.16.002 catalog

**F-IMPL-LP1-MED-003** — Audit event emission not verified as durable

- Severity: MEDIUM
- Description: Tests verified that audit events were emitted (via tracing capture) but not that they were durably stored. With the HIGH-002 fix (PluginLoadAuditSink), durability is inherent in the implementation.
- Closure: CLOSED-by-cascade — HIGH-002 (RocksDbPluginAuditSink + durable RocksDB audit test verifies emission durably)

**F-IMPL-LP1-MED-004** — Pipeline request cap branch not exercised by any test

- Severity: MEDIUM
- Description: The `execute_with_max_requests` cap enforcement branch (AC-16 / E-PIPELINE-001 / SpecEngineError::TooManyRequests) was implemented but the boundary condition (cap exceeded) was not exercised by any test, leaving the branch effectively dead code from a test coverage perspective.
- Closure: `c87592e8` — `execute_with_max_requests` variant + wiremock-driven test that triggers the cap + verifies TooManyRequests is returned

**F-IMPL-LP1-MED-005** — Test name did not match behavioral contract being tested

- Severity: MEDIUM
- Description: A test was named in a way that did not correspond to its behavioral AC, making traceability unclear.
- Closure: `1d620e63` — test renamed + Option A docstring added

**F-IMPL-LP1-MED-006** — WASI linking not verified as absent

- Severity: MEDIUM
- Description: The implementation used a WASM component model approach rather than WASI. Without a negative-proof test, the implementation could accidentally link WASI (which would change security properties).
- Closure: CLOSED-by-cascade — CRIT-003 fix includes WASI-importing component negative-proof test

**F-IMPL-LP1-MED-007** — Empty allowlist entry guard missing at parse and `host_http_request`

- Severity: MEDIUM
- Description: The `allowed_urls` field parser and `host_http_request` handler did not guard against empty string entries in the allowlist (which could create an inadvertent allow-all or confusing behavior).
- Closure: `1d620e63` — empty allowlist entry guard added at parse + host_http_request validation

### LOW (2)

**F-IMPL-LP1-LOW-001** — `#[non_exhaustive]` missing from PrismConfig family

- Severity: LOW
- Description: The `PrismConfig` family of types lacked `#[non_exhaustive]`, violating CLAUDE.md conventions.
- Closure: CLOSED-by-cascade — CRIT-002 fix adds `#[non_exhaustive]` to PrismConfig family

**F-IMPL-LP1-LOW-002** — Tracing field-name cosmetic inconsistency

- Severity: LOW
- Description: Minor inconsistency in tracing event field names (cosmetic; not a semantic contract violation).
- Closure: `1d620e63` — tracing field-name cosmetic fix

### OBS / Process-Gap (3)

**F-IMPL-LP1-OBS-001** — No CI gate verifying BC-INDEX row version matches BC file version

- Type: process-gap
- Description: The BC-INDEX has a row citing BC-2.16.002 at a specific version. When BC-2.16.002 is bumped, the BC-INDEX row should be updated atomically. Currently there is no CI check enforcing this. The F-IMPL-LP1-HIGH-001 finding illustrates the risk: spec amendments had to be applied manually post-implementation.
- Routing: Codification queue item #18; session-reviewer at cycle-close. NOT this burst.

**F-IMPL-LP1-OBS-002** — Boot-step "registered but not called" anti-pattern needs lint

- Type: process-gap
- Description: The CRIT-001 finding (run_boot_sequence does not call plugin_load_step) is a canonical instance of a class that has appeared in prism's history: a step is registered in the boot infrastructure but not invoked. A lint or architectural test that verifies registered boot steps appear in the boot sequence call chain would catch this class mechanically.
- Routing: Codification queue item #19; session-reviewer at cycle-close. NOT this burst.

**F-IMPL-LP1-OBS-003** — Scope-expansion adjudications (3 recorded)

- REJECTED: `iter_module()` behavioral substitution — adversary found the behavioral proof-via-substitution approach for AC-8/AC-11 acceptable; no scope change required.
- ACCEPTED (with caveat): HostState test-helper constructors (`test_with_plugin_id` + `test_with_allowed_urls`) — accepted pattern; story §AC-17 prescription should be amended in a follow-up burst by story-writer to document the `test-helpers` feature gate approach. Routing: story-writer at cycle-close.
- PARTIALLY ACCEPTED: 3 net-new `event_type` emission sites beyond the 9 enumerated in story §Structured Event Catalog Additions — all 3 are appropriate observability sites. Fixed via HIGH-001 (BC-2.16.002 v1.15) and MED-001 (message field).

### KUDO (2)

**K-IMPL-LP1-001** — Excellent structured event catalog discipline in base implementation

The implementer correctly applied PG-LP11-001 by adding the 3 new event_type emission sites discovered during TDD to BC-2.16.002 in the same commit. The in-commit pattern demonstrates internalization of the discipline.

**K-IMPL-LP1-002** — WASM component model approach is architecturally sound

The use of the WASM component model (rather than WASI) for plugin isolation is the correct architectural choice per ADR-022. The implementation demonstrates understanding of the security boundary requirements.

## Scope-Expansion Adjudications

Three implementer-surfaced expansions evaluated:

| Expansion | Decision | Rationale |
|-----------|----------|-----------|
| `iter_module()` behavioral substitution for AC-8/AC-11 | REJECTED (no change needed) | Behavioral proof-via-substitution is acceptable; production behavior unchanged |
| HostState test-helper constructors (`test_with_plugin_id` + `test_with_allowed_urls`) | ACCEPTED with caveat | `#[non_exhaustive]` blocks external functional-update syntax; pattern is production-grade; §AC-17 amendment recommended as follow-up |
| 3 net-new `event_type` emission sites | PARTIALLY ACCEPTED | All 3 are appropriate; addressed via HIGH-001 + MED-001 spec amendments |

## Policy Verification Summary

| Policy | Result | Note |
|--------|--------|------|
| POL-1 (append-only taxonomy) | PASS | New E-PLUGIN codes appended at end of namespace |
| POL-3 (state-manager-last) | PASS | State-manager committed after implementer |
| POL-7 (cross-table sweep) | FAIL (HIGH-001) | BC-2.16.002 not updated for impl-phase emission sites |
| POL-11 (index-bump) | FAIL (HIGH-001) | BC-INDEX row not updated until 9b2b4823 |
| POL-15 (boot-step registration) | FAIL (CRIT-001) | plugin_load_step registered but not called |
| POL-20 (non_exhaustive) | FAIL (CRIT-002) | PrismConfig family missing #[non_exhaustive] |
| POL-22 (anchor verification) | PASS | All external anchors verified |
| BC-5.39.001 (3-CLEAN) | 0/3 BLOCKED | Finding severity CRIT resets streak |
| TD-VSDD-059 (paper-fix) | FAIL (CRIT-003) | register_host_functions was a no-op |

## Carry-Forward OBS Routing

- OBS-001 (codification queue item #18) → session-reviewer at cycle-close
- OBS-002 (codification queue item #19) → session-reviewer at cycle-close
- OBS-003 scope-expansion adjudications → recorded in CYCLE-SNAPSHOT §FIX-BURST-IMPL-1 CLOSURE

## Closure Crosswalk

All 18 in-perimeter findings closed by fix-burst-impl-1 (D-548):

- CRIT-001 → `0e0c85d0`
- CRIT-002 → `0e0c85d0`
- CRIT-003 → `30a7a304`
- HIGH-001 → `9b2b4823` (factory in-burst spec amendments)
- HIGH-002 → `73d72f03`
- HIGH-003 → `1d620e63` + `9b2b4823`
- HIGH-004 → `1d620e63`
- HIGH-005 → `1d620e63` + `9b2b4823`
- HIGH-006 → `1d620e63` + `9b2b4823`
- MED-001 → `9b2b4823`
- MED-002 → `1d620e63` + `9b2b4823`
- MED-003 → CLOSED-by-cascade (HIGH-002)
- MED-004 → `c87592e8`
- MED-005 → `1d620e63`
- MED-006 → CLOSED-by-cascade (CRIT-003)
- MED-007 → `1d620e63`
- LOW-001 → CLOSED-by-cascade (CRIT-002)
- LOW-002 → `1d620e63`

## Next Pass Dispatch Template

**adversary impl-pass-2** (target: streak advance 0/3 → 1/3):

- Feature branch: `feature/S-PLUGIN-PREREQ-D@c87592e8` (or HEAD at dispatch time)
- Base develop: 95d46be2 (unchanged)
- Carry-forward verification: all 18 fix-burst-impl-1 closures must hold under fresh-context
- Priority carry-forward checks:
  - CRIT-001: `run_boot_sequence` calls `plugin_load_step` in its body
  - CRIT-002: `PrismConfig` has `plugin_dir: PathBuf` field + `#[non_exhaustive]`
  - CRIT-003: `register_host_functions` registers 5 non-stub host functions
  - HIGH-002: `PluginRuntime::new_with_audit_sink` wires `PluginLoadAuditSink`
  - HIGH-001/003/005/006: E-PLUGIN-017/018/019 in error-taxonomy.md + BC-2.16.002 catalog
- 3-CLEAN protocol per BC-5.39.001: CLEAN → streak 1/3; BLOCKED → streak 0/3

> **Status as of D-548:** fix-burst-impl-1 CLOSED 18/18. impl-pass-2 dispatch is the next action.
