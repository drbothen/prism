---
document_type: story
story_id: S-ENGINE-TIMEOUT-OVERLAY-WIRE-001
title: "timeout_secs overlay wiring — per-org request timeout via ResolvedSensorSpec provenance flag into per-request reqwest client (ADR-060 §D8.6, deferred)"
level: "L4"
wave: TBD
epic_id: E-XDOME-EXPANSION
priority: P2
status: draft
# BC status: pending PO authorship — behavioral_contracts is empty. Per S-7.01, this story
# MUST remain draft until a product-owner authors and anchors BCs with canonical IDs matching
# BC-\d+\.\d{2}\.\d{3}. No BC covers timeout_secs overlay wiring at this time.
producer: story-writer
timestamp: "2026-08-26T00:00:00Z"
version: "1.0"
modified: "2026-08-26"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md"
input-hash: "1aa2c61"
traces_to: []
points: 5
estimated_days: 1
tdd_mode: strict
subsystems: [SS-01, SS-16]
target_module: prism-bin
crates_touched: [prism-bin, prism-spec-engine]
capabilities:
  - CAP-029
behavioral_contracts: []
# BC status: pending PO authorship (S-7.01 gate — behavioral_contracts: [] blocks status=ready)
verification_properties: []
holdout_scenarios: []
depends_on: [S-ENGINE-LIMIT-EARLY-STOP-001]
# depends_on justification: S-ENGINE-LIMIT-EARLY-STOP-001 establishes the FetchContext
#   and execute_impl patterns this story builds on. ADR-060 §D8.6 explicitly defers this
#   work from the D8 scope. Both must be delivered before this story can run.
blocks: []
acceptance_criteria_count: 0
# acceptance_criteria_count: 0 — draft stub; ACs to be authored when PO writes BCs
risk: LOW
assumption_validations: []
risk_mitigations: []
---

# S-ENGINE-TIMEOUT-OVERLAY-WIRE-001: timeout_secs Overlay Wiring — Per-Org Request Timeout (Deferred, ADR-060 §D8.6)

> **DRAFT STUB — NOT FOR IMMEDIATE DELIVERY.** This story captures the ADR-060 §D8.6
> architectural direction and documents a naming discrepancy in the current overlay.rs WARN
> message. It is non-blocking for S-CLAROTY-VULNS-001 live-green. PO must author BCs before
> this story can be dispatched. Status MUST remain `draft` until `behavioral_contracts:` is
> populated.

## Authority

**ADR-060 §D8.6** is the architectural direction for this story. It specifies:

> "The `timeout_secs` overlay field is accepted but emits `overlay.timeout_secs_ignored` (WARN
> in `overlay.rs`). Wiring it to the reqwest client requires threading the overlay timeout
> through `ResolvedSensorSpec` → caller → `FetchContext` (or creating a per-org client cache
> with the configured timeout). This is architecturally independent of D8 and adds complexity
> to `FetchContext` that would blur the D8 change. Deferring to a separate story
> `S-ENGINE-TIMEOUT-OVERLAY-WIRE-001`. Architectural direction for that story: the caller
> (`spec_driven_adapter.rs`) reads `resolved_spec.provenance.timeout_secs_from_overlay` and,
> when `true`, constructs a fresh reqwest client via a variant of
> `build_http_client_with_custom_timeout` parameterized by the overlay timeout. The
> `PipelineExecutor` receives the correctly-configured client; no change to `FetchContext`
> needed."

**BC**: Pending PO authorship. The relevant prior BC surface involves:
- BC-2.16.014 (DeclarativeHttpAuthProvider — HTTP client inheritance from spec_driven_adapter)
- The `timeout_secs` field in the overlay schema (prism-spec-engine overlay.rs)
- The `provenance.timeout_secs_from_overlay` field in `ResolvedSensorSpec`

No single existing BC covers the per-org client timeout wiring end-to-end. PO must author
a new BC or amend an existing BC before this story can be dispatched.

## Naming Discrepancy (Action Required for Architect)

The current `overlay.rs` WARN message in `crates/prism-spec-engine/src/overlay.rs` reads:

> `event_type = "overlay.timeout_secs_ignored"`, message referencing `"deferred to S-CONFIG-MULTI-TENANT-OVERRIDE-002"`

ADR-060 §D8.6 names this story **S-ENGINE-TIMEOUT-OVERLAY-WIRE-001**, not
`S-CONFIG-MULTI-TENANT-OVERRIDE-002`. There is a naming mismatch between the overlay.rs WARN
message and this story's ID. This discrepancy must be reconciled before delivery — either:
- Update the overlay.rs WARN message to reference `S-ENGINE-TIMEOUT-OVERLAY-WIRE-001`, OR
- Confirm that `S-CONFIG-MULTI-TENANT-OVERRIDE-002` is the intended story and rename this stub.

**No resolution in this stub** — flag to architect and product-owner for naming adjudication
before BC authorship begins.

---

## Narrative

As a multi-tenant operator who has configured a per-org `timeout_secs` overlay for a slow
sensor endpoint,
I want the sensor adapter to construct a fresh reqwest client with the overlay-specified timeout
for that org's requests,
so that the per-org timeout override takes effect at the HTTP level rather than being silently
ignored.

## Background

The `timeout_secs` field can be declared in a sensor overlay (per-org TOML override in
`prism-spec-engine/src/overlay.rs`). Currently, the field is parsed and a WARN is emitted
(`overlay.timeout_secs_ignored`) but the value is never wired into the HTTP client — the
30-second default timeout is always used regardless of the overlay value.

ADR-060 §D8.6 deferred this wiring to avoid cluttering the FetchContext change in
S-ENGINE-LIMIT-EARLY-STOP-001. The architectural direction is clear:

1. `ResolvedSensorSpec` already tracks `provenance.timeout_secs_from_overlay: bool`.
2. In `SpecDrivenSensorAdapter::fetch`, read `resolved_spec.provenance.timeout_secs_from_overlay`.
3. When `true`, construct a fresh reqwest client via a variant of
   `build_http_client_with_custom_timeout` parameterized by the overlay timeout value.
4. Pass this per-request client to `PipelineExecutor`. No `FetchContext` change needed.

**Important:** The per-request client MUST also carry the ADR-059 §D7 h2 window settings
(added by S-ENGINE-H2-LARGE-RESPONSE-001). The variant of `build_http_client_with_custom_timeout`
used here must include all three h2 builder calls — it is a client factory for production
outbound sensor requests, falling under ADR-050 §D6 scope.

## Acceptance Criteria

> N/A — Draft stub. ACs will be authored when PO writes BCs. Placeholder structure:

### AC-001 (placeholder): `timeout_secs` overlay wiring emits no WARN when overlay timeout is wired
### AC-002 (placeholder): Per-org client with overlay timeout carries ADR-059 h2 window settings
### AC-003 (placeholder): Default 30s timeout preserved when no `timeout_secs` overlay is set

## Red Gate Tests

> N/A — No ACs yet. Red Gate list to be authored after BC authorship.

**BC-5.38.001 density check:** N/A — draft stub.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `overlay.rs` timeout_secs field | `crates/prism-spec-engine/src/overlay.rs` | Pure (TOML deserialization) |
| `ResolvedSensorSpec.provenance` | `crates/prism-spec-engine/src/` | Pure (resolved spec struct) |
| Per-request client factory | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (client construction; no I/O) |
| `SpecDrivenSensorAdapter::fetch` wiring | `crates/prism-bin/src/spec_driven_adapter.rs` | Effectful (HTTP request) |

## Purity Classification

> Stub — to be classified when ACs are authored.

- **Pure functions (no I/O, deterministic):** `build_http_client_with_custom_timeout` variant construction (client builder); `provenance.timeout_secs_from_overlay` field read (struct field access).
- **Effectful functions (I/O, network):** `SpecDrivenSensorAdapter::fetch` (HTTP request with per-org client).

## Edge Cases

> Stub — edge cases to be enumerated after BC authorship.

## Token Budget Estimate

> Stub — to be refined when BCs are authored.

## Tasks

> Stub — tasks to be enumerated when PO authors BCs and story moves to ready.

Key implementation direction (for PO reference when authoring BCs):
1. Verify `provenance.timeout_secs_from_overlay: bool` field exists in `ResolvedSensorSpec`
2. In `SpecDrivenSensorAdapter::fetch`: read `resolved_spec.provenance.timeout_secs_from_overlay`
3. When `true`: construct per-request client via `build_http_client_with_custom_timeout(Duration::from_secs(overlay_timeout))` (already includes ADR-059 h2 window settings after S-ENGINE-H2-LARGE-RESPONSE-001 merges)
4. Pass per-request client to `PipelineExecutor::execute` instead of the default 30s client
5. Reconcile naming discrepancy: `S-CONFIG-MULTI-TENANT-OVERRIDE-002` vs this story ID

## Previous Story Intelligence

1. **S-ENGINE-H2-LARGE-RESPONSE-001:** Adds h2 window settings to `build_http_client_with_custom_timeout`. The per-org client variant must ALSO include those settings. This story depends on S-ENGINE-LIMIT-EARLY-STOP-001 which depends on the foundation established by S-ENGINE-H2-LARGE-RESPONSE-001.

2. **S-ENGINE-LIMIT-EARLY-STOP-001:** Establishes the `FetchContext` field pattern. ADR-060 §D8.6 explicitly chose NOT to use `FetchContext` for timeout threading — the timeout is wired at the client-construction level, not via `FetchContext`.

## Architecture Compliance Rules

> Stub — to be extracted from ADRs when BCs are authored. Key rules to enforce:
- New client factory MUST include ADR-059 §D7 h2 window settings (4 MiB, adaptive)
- ADR-050 §D1/D2 (rustls-tls), §D6 (User-Agent) apply to any new client factory

## Library & Framework Requirements

> Stub — to be specified when ACs are defined.

## File Structure Requirements

> Stub — to be defined when ACs are defined.

---

## References

- ADR-060 §D8.6 — Architectural direction: `provenance.timeout_secs_from_overlay` → per-request client
- ADR-059 §D7 — h2 window settings must be included in any new client factory
- ADR-050 §D6 — factory scope; rustls-tls; User-Agent
- `crates/prism-spec-engine/src/overlay.rs` — `overlay.timeout_secs_ignored` WARN source; naming mismatch with this story ID

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-08-26 | story-writer | Initial draft stub — captures ADR-060 §D8.6 architectural direction and overlay.rs naming discrepancy. No ACs, no BCs, no RG tests. Non-blocking. Status must remain draft until PO authors BCs. |
