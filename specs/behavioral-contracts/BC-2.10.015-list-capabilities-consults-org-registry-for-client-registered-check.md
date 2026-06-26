---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-005"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: "2026-06-26"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-046-three-mode-correctness-filter-sql-pipe-mode-bridge-error-and-execution-validation.md"
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring-arc-di-boot-sequence-and-subsystem-initialization.md"
input-hash: "TBD"
traces_to: ["CAP-005"]
extracted_from: null
---

# BC-2.10.015: `list_capabilities` Consults `OrgRegistry` for `client_registered` Check

## Description

The `list_capabilities` tool's `client_registered` field MUST reflect actual org registration as determined by the authoritative `OrgRegistry` (populated from per-org spec overlays per ADR-006/ADR-029), not by the presence of `[clients.*.capabilities]` write-capability entries in `prism.toml`. The `FeatureFlagEvaluator` receives `Arc<OrgRegistry>` via Arc-DI wiring (ADR-022 §C) and its `client_exists(client_id)` method consults the `OrgRegistry` for org existence independently of write-capability configuration.

## Preconditions

- `list_capabilities` is invoked with a `client_id` (single-client mode) or `null` (all-clients mode)
- The `OrgRegistry` has been populated from per-org spec overlay files (e.g., `~/.config/prism-demo/specs/customers/*.yaml`) during the boot sequence
- `FeatureFlagEvaluator` has been constructed with `Arc<OrgRegistry>` passed via its constructor (ADR-022 §C Arc-DI)

## Postconditions

- `client_registered: true` is returned for any `client_id` whose `OrgId` is present in the `OrgRegistry` — regardless of whether that client has any `[clients.{id}.capabilities]` entries in `prism.toml`
- `client_registered: false` is returned only when the `client_id` is NOT registered in the `OrgRegistry`
- **For demo-provisioned clients**: orgs provisioned via spec overlays (e.g., `demo-setup.sh` populating `~/.config/prism-demo/specs/customers/org-a.yaml`) return `client_registered: true` because `OrgRegistry` is the authoritative source
- The capability matrix (`capabilities` field) continues to reflect write-capability config from `prism.toml` exactly as before — only the `client_registered` check changes data source
- `FeatureFlagEvaluator::new(client_capabilities, org_registry: Arc<OrgRegistry>)` is the constructor signature change — adding `OrgRegistry` is wiring, not redesign (ADR-022 §C wiring contract)

## Invariants

- **INV-ORG-EXISTENCE-AUTHORITATIVE-SOURCE:** `OrgRegistry` is the authoritative source for org existence; `prism.toml` write-capability config is NOT a registry of org existence. These are two orthogonal concerns that MUST NOT be conflated.
- **INV-ARC-DI-WIRING:** `FeatureFlagEvaluator` receives `Arc<OrgRegistry>` via its constructor, consistent with the Arc-DI wiring contract established by ADR-022 §C. The `Arc<dyn Foo>` plumbing pattern applies: this is wiring, not redesign.
- **INV-CAPABILITY-CHECK-UNCHANGED:** The hierarchical capability resolution logic (compile-tier + runtime-tier, deny-by-default, most-specific-path-wins) is UNCHANGED. Only the `client_exists()` data source changes.
- Path A (adding `[clients.org-*]` entries to `prism.toml` as the fix) is PERMANENTLY REJECTED per HRG-4 (ADR-046 MAJOR-001 ruling). No new code MUST implement Path A.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-AUTH-003` | `client_id` fails format validation (length, charset) | Unchanged from BC-2.10.011 — format validation happens before OrgRegistry lookup |
| (none for unregistered org) | `client_id` passes format validation but is not in `OrgRegistry` | Returns `client_registered: false` in the response body — NOT an error; the tool succeeds with accurate data |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-015-001 | Demo provisioning: org-a/org-b/org-c registered via spec overlays, no `[clients.*]` in `prism.toml` | `list_capabilities(org-a)` returns `client_registered: true` (from OrgRegistry) |
| EC-10-015-002 | `list_capabilities(null)` (all-clients mode) | Returns all orgs registered in `OrgRegistry`; `client_registered: true` for each |
| EC-10-015-003 | Client with write-capability entries in `prism.toml` AND in `OrgRegistry` | `client_registered: true`; capability matrix reflects `prism.toml` entries — both sources are consulted independently |
| EC-10-015-004 | Client with write-capability entries in `prism.toml` but NOT in `OrgRegistry` | `client_registered: false`; capability matrix still shows the write-cap entries (edge case — config inconsistency; OrgRegistry is authoritative for registration) |
| EC-10-015-005 | `OrgRegistry` is empty (no spec overlays loaded) | All clients return `client_registered: false`; capability matrix may still show write-cap config entries from `prism.toml` |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `list_capabilities("org-c")` after demo provisioning via spec overlay | `{ "client_registered": true, "capabilities": { … } }` | happy-path |
| `list_capabilities("org-c")` with empty prism.toml `[clients]` and org-c in OrgRegistry | `client_registered: true` | happy-path |
| `list_capabilities("non-existent-org")` | `{ "client_registered": false }` | boundary |
| `list_capabilities(null)` with 3 orgs in OrgRegistry | All three orgs in response with `client_registered: true` | happy-path |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| (none allocated) | `client_registered` accuracy | integration test (unit test against mock OrgRegistry) |

## Related BCs

- **BC-2.10.011** (amends — list_capabilities meta-tool): this BC specifies the `client_registered` data source change; all other postconditions of BC-2.10.011 remain unchanged
- **BC-2.06.015** (related — OrgRegistry cross-validation at boot): the `OrgRegistry` consulted here is the same registry populated at boot per BC-2.06.015

## Architecture Anchors

- `crates/prism-security/src/feature_flag.rs` — `FeatureFlagEvaluator` (add `Arc<OrgRegistry>` field; change `client_exists` to consult `OrgRegistry`)
- `crates/prism-mcp/src/server.rs` — `list_capabilities` handler (no `tools/list_capabilities.rs` exists; `tools/` contains `config.rs`, `mod.rs`, `operations.rs`, `prism_describe.rs`, `query.rs`, `sensor_health.rs`, `write.rs` only); wires `Arc<OrgRegistry>` through to `FeatureFlagEvaluator` at construction
- `crates/prism-core/src/org_registry.rs` (or equivalent) — `OrgRegistry::slug_exists(&OrgSlug) -> bool`
- ADR-046 §Decision MAJOR-001 ruling (Path B ratified, Path A rejected)
- ADR-022 §C Arc-DI wiring contract

## Story Anchor

TBD

## VP Anchors

(none allocated; covered by integration tests)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| Capability Anchor Justification | CAP-005 ("Feature Flag Evaluation") per capabilities.md §CAP-005 — this BC governs the `client_exists()` check within the `FeatureFlagEvaluator`, which is the implementation of the `list_capabilities` meta-tool that "exposes the full capability matrix per client" as described by CAP-005. The `client_registered` field is part of the capability matrix output. |
| L2 Invariants | DI-003 (deny-by-default feature flags) |
| Priority | P0 |
| Closes findings | MAJOR-001 (`list_capabilities` returns `client_registered: False` for all demo-provisioned orgs) |
| ADR traces | ADR-046 §MAJOR-001 (Path B ratified), ADR-022 §C (Arc-DI wiring) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | PR-203-fix-burst-F-P2R2-LOW-001 | 2026-06-26 | product-owner | Phantom Architecture Anchor correction (F-P2R2-LOW-001 / TD-VSDD-091). §Architecture Anchors: replaced non-existent `crates/prism-mcp/src/tools/list_capabilities.rs (or equivalent handler)` with real `crates/prism-mcp/src/server.rs — list_capabilities handler`. Verified: `tools/` contains only `config.rs`, `mod.rs`, `operations.rs`, `prism_describe.rs`, `query.rs`, `sensor_health.rs`, `write.rs` — no `list_capabilities.rs` exists. Story v1.7 OBS-3 closure (2026-06-25) corrected the same phantom in story file; this BC is the sibling-sweep closure on the spec side. No behavioral semantics changed. |
| 1.1 | LOCAL-adversary-pass1-obs3-closure | 2026-06-25 | product-owner | OBS-3 stale Architecture Anchor (LOCAL pass-1). Corrected `OrgRegistry::contains(client_id: &str) -> bool` to `OrgRegistry::slug_exists(&OrgSlug) -> bool` per D-1110 (the method does not exist; the real API is `slug_exists`). The story spec and implementation already use `slug_exists` correctly; only this BC anchor text was stale. No behavioral semantics changed. |
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md + ADR-046 MAJOR-001 Path B ratification (HRG-4). Closes MAJOR-001. |
