---
document_type: review-findings
story_id: PLUGIN-MIGRATION-001-E
pr_number: 154
pr_url: https://github.com/drbothen/prism/pull/154
review_start: 2026-05-23
---

# PLUGIN-MIGRATION-001-E PR Review Findings

## PR-Level Cascade Status

PR-LEVEL adversary cascade is SEPARATE from LOCAL cascade per BC-5.39.001. Fresh context, fresh 3-CLEAN streak required.

LOCAL cascade: CONVERGED at pass-12 (3-CLEAN strict, 55 findings closed, 8 fix-bursts).
PR-level cascade: IN PROGRESS — PR #154 created 2026-05-23. Security review and adversary cascade dispatching.

## Convergence Tracking

| Cycle | Date | Reviewer | Findings | Blocking | Fixed | Remaining | CLEAN(strict) | CLEAN(PR-merge) | Streak | Verdict |
|-------|------|----------|----------|----------|-------|-----------|---------------|-----------------|--------|---------|
| 1 | 2026-05-23 | pr-review-triage | 0 | 0 | 0 | 0 | yes | yes | 1/3 | pending pr-reviewer + adversary |

## Cycle 1 Findings

**Date:** 2026-05-23
**Reviewer:** pr-review-triage (PR diff scan)
**Probes applied:** SAP-1 (tracing catalog), SAP-2 (DTU TOML parity), SID-1 (no ignored-test rationalization)

### SAP-1 Result: PASS

New `event_type` values added in this PR:
- `plugin_auth_provider_constructed` — BC-2.16.002 row 36 present (FB-IMPL-3, F-LP3-LOW-001 closure)
- `plugin.auth_token_parse_error` — BC-2.16.002 row 37 present (FB-IMPL-6-CORRECTION, F-LP7-MED-001 closure)

DF-001 (armis.rs `aql_query_execution`/`aql_query_rejected`) pre-existing on develop, out-of-perimeter for this story. Carried forward to Phase-5 system-wide SAP-1 audit.

### SAP-2 Result: N/A

TOML changes in this PR (`crowdstrike.sensor.toml`) are metadata-only: added `auth_plugin = "crowdstrike-oauth2"` field. No column definitions were added, removed, or modified. SAP-2 (DTU↔TOML column parity) probe does not apply.

### SID-1 Result: PASS

All `#[ignore]`'d tests in the PR diff cite `S-PLUGIN-CI-001` as the specific blocking dependency with specific test names. No rationalization deferred without a concrete future story ID.

### Coherence Check: PASS

Files changed:
- `.github/workflows/ci.yml` — wasm32-compile-check CI job + F-LP5-HIGH-001 reachability assertion (on-topic)
- `CLAUDE.md` — BC-5.39.001 strict vs PR-merge disambiguation + SAP-1/SAP-2/SID-1 codification (project docs)
- `Cargo.lock`/`Cargo.toml` — crowdstrike-oauth2-plugin workspace member addition (on-topic)
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/` — new plugin crate (on-topic)
- `crates/prism-spec-engine/src/plugin/` — dispatch_plugin_acquire_token + auth_token_parse_error emission (on-topic)
- `crates/prism-spec-engine/src/plugin_auth_provider.rs` — PluginAuthProvider implementation (on-topic)
- `crates/prism-bin/src/boot.rs` — validate_and_construct_auth_providers step 7.5b (on-topic)
- `docs/demo-evidence/PLUGIN-MIGRATION-001-E/` — 11 AC recordings (on-topic)

No unrelated changes detected.

### Coverage Check: PASS

Test counts:
- 15 Red Gate tests (crowdstrike_oauth2_plugin_tests.rs) + unit tests in lib.rs and mod.rs
- VP-148 parity infrastructure test (AC-008)
- VP-150 401-retry test (AC-006) via WAT fixture + PluginAuthProvider
- BC-2.16.002 row 37 host-side emission test (unconditional, non-#[cfg(test)] gated)

### Known Architectural Gap (Accepted at LOCAL Pass-12)

Production `credential_handle = "sensor:crowdstrike"` in `validate_and_construct_auth_providers` is an opaque handle string, not actual OAuth2 credentials. The story spec §Credential Handling Design documents that host-side credential substitution in `host_http_request` is out-of-scope for this story. The production wiring (keyring resolution + form body injection) is gated on `S-PLUGIN-CI-001` and ADR-028 §D10 co-merge sequence. LOCAL adversary pass-12 accepted this gap. **This is a documentation/architectural note, not a blocking finding for PR merge.**

## Summary

Cycle 1 triage: 0 blocking findings. All probes (SAP-1, SAP-2, SID-1) PASS. One known architectural gap (credential_handle production wiring) documented and accepted at LOCAL cascade. PR is ready for pr-reviewer dispatch and PR-LEVEL adversary cascade.
