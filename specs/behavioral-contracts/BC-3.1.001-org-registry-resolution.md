---
document_type: behavioral-contract
level: L3
version: "0.1"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md]
input-hash: ""
traces_to: .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
origin: greenfield
extracted_from: null
subsystem: SS-06
capability: CAP-009
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.1.001
title: OrgRegistry bijective slug/uuid resolution
wave: 3
related_decisions: [D-041, D-045]
related_adrs: [ADR-006]
inherits_from: null
superseded_by: null
---

# BC-3.1.001: OrgRegistry bijective slug/uuid resolution

## Description

`OrgRegistry` provides the authoritative, in-memory bijective mapping between `OrgSlug` (analyst-facing display identifier) and `OrgId` (internal UUID v7 canonical identity). Both forward resolution (`resolve`) and reverse resolution (`slug_for`) are O(1) in-memory lookups — they never perform filesystem I/O or external calls. The registry is populated at startup from `customers/*.toml` files and is read-only for the lifetime of the process.

## Preconditions

1. `OrgRegistry` has been initialized and fully populated from `customers/*.toml` before any MCP tool dispatch begins.
2. `OrgSlug` values conform to the regex `^[a-zA-Z0-9_-]{1,64}$` (the locked constraint; see ADR-006 §8 open question on 32 vs 64 length).
3. `OrgId` values are UUID v7; UUID v4 is prohibited by the `uuid_v7_newtype!` macro in `prism-core/src/ids.rs`.
4. The bijectivity invariant (BC-3.1.003) holds: every slug maps to exactly one uuid and every uuid maps to exactly one slug.
5. No auto-registration on miss: the registry is read-only at query time.

## Postconditions

1. If `resolve(slug)` returns `Some(id)`, then `slug_for(id)` returns `Some(slug)` (round-trip consistency).
2. If `resolve(slug)` returns `None`, the registry state is unchanged (no side effect, no auto-registration).
3. Returned `OrgId` is always a valid UUID v7; the registry never stores invalid identifiers.
4. No log entry is emitted for routine `None` returns — missing org is a caller responsibility.

## Invariants

1. `resolve` and `slug_for` are pure read operations: they never mutate registry state.
2. The BiMap invariant holds at all times: `∀ slug, id: resolve(slug) = Some(id) ↔ slug_for(id) = Some(slug)`.
3. `OrgRegistry` is initialized before any component that resolves org identity. Startup aborts before the MCP stdio transport is bound if initialization fails.
4. Resolution is O(1) — no filesystem access, no network calls, no blocking I/O on the hot path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `resolve` called with a slug that was never registered | Returns `None`; no panic, no side effect |
| EC-002 | `slug_for` called with an OrgId that was never registered | Returns `None`; logs nothing (orphan detection is caller's responsibility) |
| EC-003 | `resolve` called before registry is initialized | Process startup has already aborted; this state is unreachable in production |
| EC-004 | Slug with maximum-length value (64 chars of `a-z0-9_-`) | Resolves correctly if registered; returns `None` if not |
| EC-005 | Concurrent reads from multiple async tasks | BiMap is behind a shared-read-capable guard; concurrent reads are safe and return consistent results |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.1.001-01 | `resolve("acme-corp")` where `acme-corp` is registered with `OrgId(uuid-A)` | `Some(OrgId(uuid-A))` | Happy path forward resolution |
| TV-3.1.001-02 | `resolve("unknown-org")` — slug not in registry | `None` | Unknown slug; no registration side-effect |
| TV-3.1.001-03 | `slug_for(OrgId(uuid-A))` where uuid-A is registered as `"acme-corp"` | `Some(OrgSlug("acme-corp"))` | Happy path reverse resolution |
| TV-3.1.001-04 | `slug_for(OrgId(uuid-unknown))` — not in registry | `None` | Unknown OrgId; no side-effect |
| TV-3.1.001-05 | `resolve("acme-corp")` returns `Some(id)`; then `slug_for(id)` | `Some("acme-corp")` | Round-trip consistency |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.1.001-01 | Round-trip: `resolve(slug).and_then(\|id\| slug_for(id)) == Some(slug)` for all registered slugs | proptest |
| VP-3.1.001-02 | No-side-effect: calling `resolve` or `slug_for` with any input leaves registry size unchanged | proptest |
| VP-3.1.001-03 | O(1) bound: lookup completes in bounded steps regardless of registry size | kani / manual analysis |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC defines how `OrgRegistry` resolves org identity from startup config, which is the core of client configuration loading and validation. |
| L2 Domain Invariants | n/a (Wave 3 greenfield; no pre-existing DI-NNN for OrgRegistry) |
| Architecture Module | `prism-core` or `prism-orgs` (ADR-006 §8 open question #5) |
| ADR Source | ADR-006 §2.2 (OrgRegistry structure), §2.3 (translation flow), §3.4 (slug squatting) |
| Stories | TBD (filled by story-writer) |

## Related BCs

- BC-3.1.002 — depends on (audit pipeline calls `slug_for` to denormalize slug at write time)
- BC-3.1.003 — composes with (bijectivity invariant is the structural guarantee enabling correct resolution)
- BC-3.1.004 — composes with (duplicate rejection ensures bijectivity precondition holds at registration time)
- BC-3.2.001 — depends on (adapter dispatch calls `resolve` to translate PrismQL slug to OrgId before dispatch)

## Architecture Anchors

- `crates/prism-core/src/ids.rs` — `uuid_v7_newtype!` macro; `OrgId` to be added here (ADR-006 §4 Step 1)
- `crates/prism-core/src/tenant.rs` — `TenantId` to be renamed `OrgSlug`; regex constant `TENANT_ID_PATTERN` → `ORG_SLUG_PATTERN`
- ADR-006 §2.2 — `OrgRegistry` public API surface (`resolve`, `slug_for`, `register`)

## Story Anchor

TBD — implementing story to be assigned by story-writer (Epic E-3.1 Step 1)

## VP Anchors

- VP-3.1.001-01 — round-trip consistency property
- VP-3.1.001-02 — no-side-effect on lookup
- VP-3.1.001-03 — O(1) bound

## Open Questions

- ADR-006 §8 Q1: should the `OrgSlug` regex cap be 32 or 64 characters? Locked at 64 for this dispatch; update this BC's precondition if tightened.
- ADR-006 §8 Q5: crate placement for `OrgRegistry` (`prism-core` vs new `prism-orgs`). Subsystem assignment may shift if a new subsystem is added to ARCH-INDEX.
