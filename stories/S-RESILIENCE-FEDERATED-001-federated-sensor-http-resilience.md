---
document_type: story
story_id: S-RESILIENCE-FEDERATED-001
title: "Federated Sensor HTTP Resilience — Per-Sensor TOML Timeouts, Boot-Degraded Mode, Retry-with-Backoff, Availability Cache"
wave: null
target_module: prism-spec-engine
subsystems: [SS-08]
# Subsystem anchor: SS-08 (Sensor Spec Engine) owns the per-sensor TOML config surface,
# plugin HTTP client lifecycle, and boot orchestration. This story's primary scope
# (per-sensor TOML timeout schema, boot-degraded mode, sensor availability cache, and the
# unwired timeout_secs overlay field) all live in prism-spec-engine and prism-bin boot path.
priority: P0
# P0 once scheduled; NOT demo-blocking for T13 (deferred per D-1326 adjudication).
# Will become P0 when scheduled post-demo.
depends_on: []
# No build-order dependency on specific stories. Scheduling depends_on T14/post-demo
# milestone (placeholder): do not dispatch to implementer until T14 demo recording completes
# and BC authorship finishes (Spec-First Gate S-7.01).
blocks: []
estimated_days: null
# TBD — requires full BC decomposition + ADR authorship. Estimated 8-13 pts once scoped.
points: null
level: "L4"
status: draft
# BC status: behavioral_contracts is empty (stub). Status MUST remain draft until
# a product-owner authors BCs (Spec-First Gate S-7.01). DO NOT set status: ready.
# BC status: pending PO authorship
version: "1.0"
updated: "2026-06-24"
producer: story-writer
timestamp: "2026-06-24T00:00:00Z"
input-hash: "TBD"
inputs:
  - ".factory/STATE.md"
  # D-1326 adjudication: root-cause and deferral decision recorded in STATE.md
  - ".factory/SESSION-HANDOFF.md"
traces_to: [D-1326, D-1327]
cycle: "v1.0.0-greenfield"
epic_id: null
# Epic assignment: TBD — day-2 resilience epic. Not yet decomposed.
phase: 2
acceptance_criteria_count: 0
# 0 ACs: stub only. ACs require BC authorship first.
red_gate_tests: 0
tdd_mode: strict
behavioral_contracts: []
# BC status: pending PO authorship
# Spec-First Gate S-7.01: behavioral_contracts MUST be non-empty before status can be set to ready.
verification_properties: []
assumption_validations: []
risk_mitigations: []
crates_touched:
  - prism-spec-engine
  # Per-sensor TOML timeout fields (connect_timeout_secs, request_timeout_secs),
  # sensor availability cache, hot credential reload for static-token sensors,
  # boot-degraded mode logic.
  - prism-bin
  # Boot orchestration: degraded-mode startup path, connectivity diagnostic logging,
  # boot step ordering when sensors are unavailable at startup.
  - prism-sensors
  # Fan-out layer: skip_unavailable flag, per-sensor availability gate before query dispatch.
---

# S-RESILIENCE-FEDERATED-001: Federated Sensor HTTP Resilience

> **STUB — Day-2 Epic Anchor.** This story is a scope-capture stub, NOT a TDD-ready spec.
> Full decomposition (BCs, ADRs, ACs, Red Gate tests) is day-2 work.
> DO NOT dispatch to implementer until BCs are authored and status transitions to ready.

## Deferral Anchor

This story is the deferral anchor for BLOCKER-001 from
S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (AC-019, D-1326 adjudication, 2026-06-24).

**D-1326 adjudication summary:** BLOCKER-001's hang was architecturally misdiagnosed.
The true root cause is that `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS=30` sets a total request
timeout but the plugin HTTP client has no separate `connect_timeout`. When the DTU is
not yet up at Prism start, the plugin waits 30s before failing. PluginKvStore is
in-memory and fresh per `prism start`, so cross-session KV staleness is impossible.

The dead `reset_token_cache` function and its test (removed at code HEAD 3fa69207)
confirmed this — there was no stale token issue to fix. The connect-timeout gap is a
structural resilience concern that requires new TOML schema and ADR work; it is out of
the T13 demo story budget. For the T13 demo, the hang is avoided operationally via the
runbook DTU health-check (demo-pre-flight runbook Fix B).

## Scope (from D-1326/D-1327 Architect Design)

The following capabilities are captured as day-2 scope. Full BC/ADR/story decomposition
is required before implementation. This list is not exhaustive — the architect may expand
or refine during F1 delta-analysis.

1. **Per-sensor TOML-tuneable connect/request timeouts (fail-fast defaults)**
   - New `connect_timeout_secs` and `request_timeout_secs` fields in sensor TOML specs
   - Fail-fast defaults (e.g., connect_timeout=5s, request_timeout=30s)
   - Unify the existing unwired `timeout_secs` overlay field in per-org sensor specs
     with the new per-sensor TOML timeout schema
   - Affected crates: prism-spec-engine (TOML parsing), prism-sensors (HTTP client construction)

2. **Boot-degraded mode + connectivity diagnostic**
   - Sensor unavailable at boot → degrade gracefully (skip sensor, log diagnostic) rather
     than full startup failure
   - Connectivity diagnostic: human-readable log message identifying which sensor DTU is
     unreachable and what to do (check DTU health, verify URL)
   - Boot-degraded sensors are excluded from fan-out but do not abort the Prism process

3. **Retry-with-backoff for 429/503/504 responses**
   - Transient HTTP errors trigger retry-with-exponential-backoff (configurable max retries)
   - 429 responses: honor `Retry-After` header if present
   - Per-sensor retry budget (does not block other sensors)

4. **`skip_unavailable` flag + per-sensor availability cache**
   - `skip_unavailable: bool` flag (default: false for spec compliance; demo default: true)
   - Availability cache: health-check before query fan-out; cache TTL-based availability state
   - Unavailable sensors return empty results (not error) when `skip_unavailable=true`

5. **Hot credential reload for static-token sensors (Armis/Claroty — G2)**
   - Human-authorized G2 (static-token sensors): reload credentials from keystore without
     requiring `prism restart`
   - CrowdStrike/Cyberint: OAuth2 short-lived tokens already re-acquired by plugin; no change
   - Implementation gated on human authorization of the credential-reload model

6. **Recover-without-restart integration tests**
   - Tests that simulate sensor DTU restart mid-session and verify Prism recovers without
     requiring a full `prism restart`

## Behavioral Contracts

> None yet. BC authorship required before this story can be dispatched.
> Expected BC scope (TBD):
> - BC-2.X.XXX: Per-sensor HTTP timeout configuration
> - BC-2.X.XXX: Boot-degraded mode semantics
> - BC-2.X.XXX: Retry policy for transient sensor errors
> - BC-2.X.XXX: skip_unavailable flag and availability cache contract

## Acceptance Criteria

> None yet. ACs require BC authorship first (Spec-First Gate S-7.01).

## Architecture Mapping

> TBD — requires architect ADR authorship (new TOML schema design, boot-degraded state
> machine, retry policy configuration, availability cache design).

## Edge Cases

> TBD — captured during BC authorship.

## Token Budget Estimate

> TBD — stub only.

## Tasks

> N/A — stub. Full task decomposition requires BC/ADR authorship.

1. **Before dispatch:** PO authors BCs for the 6 scope items above.
2. **Before dispatch:** Architect authors ADR for TOML timeout schema and boot-degraded model.
3. **Before dispatch:** Story-writer decomposes into per-AC TDD stories (may split into
   multiple child stories if points exceed 13).
4. **At dispatch:** Implementer follows standard TDD cascade (red-gate tests → implementation).

## Previous Story Intelligence

**S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (BLOCKER-001/AC-019):**
- The connect-timeout gap was first identified as the BLOCKER-001 root cause in D-1326.
- PluginKvStore is in-memory + fresh per `prism start` — do NOT reopen the KV-staleness
  investigation. That path is confirmed closed.
- The `acquire_token` / `get_token` cache mechanism works correctly for in-session use.
  The issue is purely that the first `client_credentials` request after DTU restart
  takes up to 30s to fail when the DTU is not yet healthy.
- Demo pre-flight runbook Fix B (DTU health-check before `prism start`) is the T13 mitigation.

## Architecture Compliance Rules

> TBD — requires ADR authorship. Expected constraints (preliminary):
> - Per-sensor timeout configuration must be TOML-driven (no hardcoded constants in production).
> - Boot-degraded mode must not silently discard sensors — it must emit a connectivity
>   diagnostic at INFO level.
> - Retry backoff must not block the tokio thread pool (all retry paths are async).
> - `OrgSlug::new_unchecked` MUST NOT appear in any new credential-reload paths (AD-017).

## Library & Framework Requirements

> TBD — version validation required during BC/ADR authorship. Expected dependencies:
> - `reqwest` (existing, workspace): add `.connect_timeout(Duration)` builder call
> - `tokio::time` (existing, workspace): retry timer + backoff logic
> - No new external dependencies expected; confirm during ADR authorship.

## File Structure Requirements

> TBD — requires full scope decomposition. Preliminary file list:
> - `crates/prism-spec-engine/src/sensor_spec.rs` or equivalent TOML parsing module
> - `crates/prism-bin/src/boot.rs` (boot-degraded mode state machine)
> - `crates/prism-sensors/src/fanout.rs` (skip_unavailable gate)
> - New: `crates/prism-sensors/src/availability_cache.rs`

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | story-writer-spec-sync-D1326 | 2026-06-24 | story-writer | Initial stub. Deferral anchor for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 AC-019 (D-1326 adjudication). Scope captured from D-1326/D-1327 architect design. BC authorship pending. |
