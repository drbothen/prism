---
document_type: adr
adr_id: "ADR-061"
title: "Multi-Tenant Cache-Key Isolation via Authoritative OrgSlug Resolution"
status: ACCEPTED
date: "2026-08-27"
modified: "2026-08-28"
version: "1.2"
producer: architect
subsystems_affected: [SS-07, SS-11]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-ENGINE-LIMIT-EARLY-STOP-001
related_adrs: [ADR-006, ADR-008, ADR-034, ADR-039, ADR-060]
related_bcs: [BC-2.01.010, BC-2.11.011, BC-2.07.003]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-061: Multi-Tenant Cache-Key Isolation via Authoritative OrgSlug Resolution

## Status

ACCEPTED v1.2 (amended 2026-08-28; F-R16-P9-MED-001 / F-R16-P10-OBS-001): §D8 field schema
corrected — `org_id` field description changed from stale "8-char prefix for diagnostics" to
full OrgId UUID via `%display`; one-line AD-017 inapplicability note added (org UUID is a
tenant identifier, not a credential). §Alternatives Alt-B AD-017 characterization corrected —
cache-key-miss is the operative rejection ground; AD-017 governs credential values and does not
prohibit tenant identifier emission in operator tracing. No behavioral change to D1–D9.

ACCEPTED v1.1 (amended 2026-08-28; F-R16-P5-HIGH-001): §D3 code example and prose corrected —
non-compiling false-premise synthesis removed; actual infallible `OrgSlug::new` call valid by
construction. §Context Site 3 false "starts with a digit" premise corrected. No behavioral
change to D1, D2, D4–D9.

ACCEPTED v1.0 (2026-08-27) — F-R16-P1-HIGH-001 (ELEVATED to CRITICAL by security review,
CWE-284/CWE-340/CWE-200, OWASP A01): cross-tenant cache-key collision via truncated synthetic
slug in three production sites. Human-approved to fold into S-ENGINE-LIMIT-EARLY-STOP-001.
E-QUERY taxonomy unchanged (skip-with-warn is the fail-closed mechanism; no new error code
allocated). Implementer must remove `crates/prism-core/tests/org_slug_from_uuid_prefix.rs`
(legitimizes the defect pattern).

---

## Context

### Defect Classification

- **CWE-284** — Improper Access Control (primary): tenant data served under a shared cache
  partition key, bypassing the OrgSlug isolation contract.
- **CWE-340** — Generation of Predictable Identifiers: 8-hex prefix of a UUIDv7 encodes
  only the millisecond timestamp component; predictable across concurrent onboarding.
- **CWE-200** — Exposure of Sensitive Information to Unauthorized Actor.
- **OWASP A01** — Broken Access Control.
- **Reachability**: the bare-filter fan-out (Site 1) is triggered by the DEFAULT query path
  (`Ast::Filter` with empty source) — standard bare-predicate queries. The collision is
  **near-certain under MSSP batch provisioning** (multiple orgs onboarded in the same
  ~65-second UUIDv7 timestamp window).

### Three Defect Sites in `materialization.rs`

All three sites are in `prism-query::materialization`. All three feed their result into
`FanOutTarget.client_id` → `derive_response_cache_key` → `CacheKey::derive` (the response
cache partition key for BC-2.07.003).

---

**Site 1 — Bare-Filter Fan-Out Step 3b (CRITICAL)**

```rust
// DEFECT: produces a truncated synthetic client_id
// The in-code comment "no OrgRegistry available in bare-filter test path" is FACTUALLY WRONG.
// mat_ctx.org_registry IS present and populated in production for this path.
for (org_id, _adapter) in adapters {
    let client_id = OrgSlug::new(format!("org-{}", &org_id.to_string()[..8]));
    // ...
}
```

This path NEVER consults `mat_ctx.org_registry`, even when the registry is `Some(reg)` with
full slug mappings. The code comment claiming the registry is unavailable here is incorrect.

---

**Site 2 — `resolve_source_refs` ALL-scope No-Slug Fallback (CRITICAL)**

```rust
// DEFECT: else branch executes for BOTH registry-absent AND registry-present-slug-missing
let Some(client_slug) = org_registry.as_ref().and_then(|reg| reg.slug_for(&org_id))
else {
    // Comment claims "OrgRegistry absent (test mode) — fall back to test slug"
    // but this also executes when registry IS present and slug_for returns None.
    let synthetic_candidate = format!("org-{}", &org_id.to_string()[..8]);
    // ...
    targets.push(FanOutTarget { client_id: synthetic_slug, ... }); // BUG: should skip
    continue;
};
```

The `else` branch collapses two semantically different conditions into one: (a) registry absent
(acceptable test mode) and (b) registry present but slug missing (configuration inconsistency
in production). Both currently synthesize a truncated slug and push the target.

---

**Site 3 — `"synthetic-unmapped"` Sentinel (CRITICAL — second CRITICAL residual)**

```rust
// DEFECT: collapses ALL orgs with invalid UUID prefix to ONE partition key
OrgSlug::new("synthetic-unmapped")
// used as fallback when the 8-hex synthesis itself fails OrgSlug validation
```

The sentinel `"synthetic-unmapped"` is a **static constant** introduced under the false
premise that `OrgId::to_string()[..8]` could fail `OrgSlug` validation when the hex substring
begins with a digit. This premise is incorrect: the `"org-"` literal prefix always produces
`"org-HHHHHHHH"` where the first character is `'o'`; `ORG_SLUG_PATTERN`
(`^[a-zA-Z0-9_-]{1,64}$`) permits digits in any position, and the `"org-"` prefix satisfies
the pattern unconditionally. The sentinel path is **unreachable by construction** from the
synthesis call. Should the sentinel be reached through any other code path, every affected org
hashes to the same `"synthetic-unmapped"` partition — a **single shared entry across all of
them** and a total cross-tenant collapse for those org IDs.

This sentinel MUST be removed unconditionally. It has no valid use case in either production
or test mode: in production, the registry provides authoritative slugs; in test mode, the
`"org-{8hex}"` synthesis is always valid by construction (all test orgs produce distinct keys).

---

### Downstream Consumers of the Synthetic `client_id`

Both the truncated-hex and sentinel `client_id` values propagate to two consumers:

1. **`derive_response_cache_key`** → `CacheKey::derive(client_id, sensor_id, source_table, params)`.
   Collision → Tenant B reads Tenant A's cached sensor rows.
2. **`SensorSpec.client_id` (deprecated field)** → structured log attribution (audit trail
   shows the wrong org identity).

### Existing Legitimizing Test (Must Be Removed)

`crates/prism-core/tests/org_slug_from_uuid_prefix.rs` tests and asserts that the
`format!("org-{}", &org_id.to_string()[..8])` synthesis pattern is valid. This test
**legitimizes the defect** and must be removed or replaced with a test asserting that
synthesis is ONLY valid when `org_registry.is_none()`.

---

## Decision

### D1 — Cache-Key Identity Invariant

**`derive_response_cache_key` and all `FanOutTarget` construction MUST use a `client_id` that
is a collision-free, persistent, authoritative org identity.** Only two sources are authoritative:

- **(a) `OrgRegistry::slug_for(org_id)`** when `mat_ctx.org_registry` is `Some(reg)` and
  the lookup returns `Some(slug)`.
- **(b) An explicit `OrgSlug` from the caller's `clients` list** when `org_registry` is
  `None` (test/MVP mode with no registry injected).

**The following `client_id` derivations are FORBIDDEN whenever `org_registry: Some(_)`:**
- `format!("org-{}", &org_id.to_string()[..8])` (8-hex truncation)
- `OrgSlug::new("synthetic-unmapped")` (static sentinel)
- Any other synthesized or computed value not sourced from `OrgRegistry::slug_for`

### D2 — Fail-Closed Policy: Skip-With-Structured-Warn When Registry Present But Slug Missing

When `org_registry: Some(reg)` and `reg.slug_for(org_id)` returns `None`:

**The pipeline MUST skip this org's target and emit a structured `tracing::warn!`.** It MUST
NOT synthesize a slug, use a sentinel, or push a `FanOutTarget` for this org.

```rust
// D2 implementation pattern for resolve_source_refs ALL-scope and Step 3b
match org_registry.as_ref().and_then(|reg| reg.slug_for(&org_id)) {
    Some(slug) => slug,
    None if org_registry.is_some() => {
        // Registry present, no slug for this OrgId — configuration inconsistency.
        // SKIP this org; do NOT synthesize, do NOT use sentinel. (ADR-061 D2)
        tracing::warn!(
            org_id = %org_id,
            event_type = "query.org_slug_resolution_failure",
            "OrgId has no slug mapping in OrgRegistry; skipping fan-out target \
             (ADR-061 D2 fail-closed: no data served under synthetic identity)"
        );
        continue; // next adapter / next org_id in the loop — do NOT push to targets
    }
    None => {
        // Registry absent — test/MVP mode. Synthetic slug is acceptable. (ADR-061 D3)
        // ...
    }
}
```

**Skip-with-structured-warn is fail-closed** because: no data is served under a wrong or
synthesized identity. The constraint is satisfied (AD-017: tenant data does not cross tenant
boundaries) even though the misconfigured org's data is excluded.

**Rationale for skip-with-warn over hard-error:**
In a multi-tenant MSSP deployment, a single misconfigured org's `OrgRegistry` entry must not
block ALL tenants' query results. A hard error would mean that a configuration inconsistency
for org X causes org Y, Z, and W to also receive no data — disproportionate and operationally
harmful in a SOC context. Skip-with-structured-warn allows the other tenants' data to be
returned correctly while surfacing the misconfigured org to the operator via the structured
`query.org_slug_resolution_failure` event. The BC-2.01.010 partial-failure model supports this:
sensor-level and identity-level failures can be accumulated without blocking the full fan-out.

The structured `tracing::warn!` (per SAP-1 / PG-LP11-001 — `event_type` field required)
ensures the skip is **not silent**. The `query.org_slug_resolution_failure` event must appear
in the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions (product-owner
obligation, anchored to S-ENGINE-LIMIT-EARLY-STOP-001).

### D3 — Test/MVP Mode: Synthetic Slug When Registry Is Entirely Absent

When `org_registry: None` (the registry was not injected — test harness or single-tenant
MVP mode), the synthetic-slug path uses the following infallible call:

```rust
// D3: org_registry is None — test/MVP mode.
// Valid by construction: the "org-" literal prefix ensures ORG_SLUG_PATTERN
// (^[a-zA-Z0-9_-]{1,64}$) compliance regardless of the hex characters that follow.
// All hex chars [a-f0-9] are within [a-zA-Z0-9]; total string length is 12 <= 64;
// the first character is always 'o'. No digit-start special case; no fallback branch.
let client_id = OrgSlug::new(format!("org-{}", &org_id.to_string()[..8]));
```

`OrgSlug::new` returns `OrgSlug` directly with embedded validity state (see
`prism_core::tenant::OrgSlug` — infallible constructor that carries an `OrgSlugInner::Valid`
or `OrgSlugInner::Invalid` state; callers call `.unwrap()` or `.expect()` when needed). The
`"org-"` prefix guarantees ORG_SLUG_PATTERN compliance: the result is always `"org-HHHHHHHH"`
(12 chars ≤ 64; first char `'o'`; all hex chars are `[a-f0-9] ⊆ [a-zA-Z0-9]`). No fallback
branch exists — this path is valid by construction.

The `"synthetic-unmapped"` sentinel MUST be removed unconditionally (see §Context Site 3). It
was introduced under a false premise and its synthesis path is unreachable by construction.
Distinct org IDs in test mode produce distinct `"org-HHHHHHHH"` keys, preserving per-org
isolation without any digit-start special casing.

**The `"synthetic-unmapped"` sentinel is removed unconditionally from both Site 1 and Site 2.**

### D4 — Bare-Filter Step 3b Must Consult OrgRegistry (Site 1 Fix)

The bare-filter Step 3b path MUST be updated to apply the D2/D3 dispatch. The fix is a
targeted replacement of the existing `let client_id = OrgSlug::new(format!(...))` with the
registry-first pattern:

```
for each (org_id, sensor_id) in adapter_registry.get_all_for_sensor(sensor_id):
    client_id = match mat_ctx.org_registry:
        Some(reg) => match reg.slug_for(org_id):
            Some(slug) => slug                // D1: authoritative path
            None       => warn+continue       // D2: skip, do not synthesize
        None => synthetic_from_org_id_d3()   // D3: test mode
    push FanOutTarget { client_id, ... }
```

`mat_ctx.org_registry` is already available at the Step 3b callsite via `pub(crate) org_registry`
on `MaterializationContext`. No new field threading is required.

### D5 — `resolve_source_refs` ALL-scope Fallback (Site 2 Fix)

The `else` branch of the `let Some(client_slug) = org_registry.as_ref().and_then(...)` pattern
MUST be split into two arms based on registry presence, replacing the current unified fallback:

```rust
// BEFORE (defect): unified else branch
let Some(client_slug) = org_registry.as_ref().and_then(|reg| reg.slug_for(&org_id))
else {
    // runs for BOTH absent-registry AND present-registry-missing-slug
    /* synthesize and push */ continue;
};

// AFTER (correct): split on registry presence
let client_slug = match org_registry.as_ref().and_then(|reg| reg.slug_for(&org_id)) {
    Some(slug) => slug,
    None if org_registry.is_some() => {
        tracing::warn!(..., event_type = "query.org_slug_resolution_failure", ...);
        continue; // D2: skip
    }
    None => synthetic_from_org_id_d3(org_id), // D3: test mode
};
```

### D6 — SINGLE-BINDING COHERENCE Extension

The SINGLE-BINDING COHERENCE invariant (ADR-060 §D8.8) extends to `client_id`: once a
`client_id` is derived from an authoritative source (D1) or confirmed test-mode synthetic (D3),
it MUST flow unchanged through `FanOutTarget.client_id`, `derive_response_cache_key`, and all
structured log fields for that target. No secondary derivation from `OrgId` is permitted after
the resolution point.

### D7 — Test Removal: `org_slug_from_uuid_prefix.rs`

The implementer MUST remove `crates/prism-core/tests/org_slug_from_uuid_prefix.rs` (or amend
it to assert that `format!("org-{}", org_id.to_string()[..8])` is ONLY valid when
`org_registry.is_none()`). This test currently asserts the defect pattern as correct behavior.
Leaving it in place would cause the adversary to class-close the defect as "tested and valid."

### D8 — SAP-1 / BC-2.16.002 Catalog Row Obligation

The `query.org_slug_resolution_failure` `tracing::warn!` emission (D2) MUST have a
corresponding row in the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions
before the S-ENGINE-LIMIT-EARLY-STOP-001 PR merges. This is a product-owner obligation per
SAP-1 / PG-LP11-001.

Required fields for the catalog row:
- `event_type`: `"query.org_slug_resolution_failure"`
- Fields: `org_id` (full OrgId UUID via `%display` — the 36-char tenant identifier; an `OrgId`
  UUID is a tenant identifier, not a credential; AD-017 governs credential values and does not
  prohibit tenant identifier emission in operator tracing; the full UUID is required to
  unambiguously identify the misconfigured org without risk of the ~65-second UUIDv7
  collision class this ADR was created to close)
- Audit role: diagnostic / config-gap signal
- Recurrence: per-org per-query (fires once per unresolved org_id per query execution)

### D9 — Red Gate Obligations (Anchored: S-ENGINE-LIMIT-EARLY-STOP-001)

| Gate ID | Description |
|---------|-------------|
| RG-SLUG-001 | `resolve_source_refs` ALL-scope: `org_registry: Some(reg)` + slug missing → `targets` does NOT include an entry for that org; `tracing::warn!` with `event_type = "query.org_slug_resolution_failure"` fired |
| RG-SLUG-002 | `resolve_source_refs` ALL-scope: `org_registry: None` → synthetic slug generated; target IS included |
| RG-SLUG-003 | Bare-filter Step 3b: `org_registry: Some(reg)` + slug missing → target NOT pushed; warn fired |
| RG-SLUG-004 | Bare-filter Step 3b: `org_registry: None` → synthetic slug used; target IS pushed |
| RG-SLUG-005 | Two orgs with UUIDv7 timestamps within the same 65-second window produce DISTINCT cache keys when resolved via a populated `OrgRegistry` (regression: collision defect closed) |
| RG-SLUG-006 | `"synthetic-unmapped"` sentinel is ABSENT from both Site 1 and Site 2 production code paths — compile-time absence enforced by a grep-based test or a `#[deny(unused)]` approach agreed with the implementer |

---

## Rationale

The cache-key derivation uses `OrgSlug` as the tenant partition key because `OrgSlug` is the
canonical, human-readable, and bijectively registry-mapped org identity. `CacheKey::derive`
hashes `(client_id, sensor_id, source_table, params)` where `client_id = OrgSlug::as_str()`.
Any collision in `client_id` directly produces a shared cache entry and cross-tenant data
exposure.

The defect arose because two sites were written before `OrgRegistry` was injected into
`MaterializationContext`, or were written for single-tenant test paths and never updated when
multi-tenancy was added. The in-code comment at Site 1 ("no OrgRegistry available in bare-filter
test path") is factually wrong in the current codebase — the registry IS present in the
production `MaterializationContext` construction.

Skip-with-structured-warn (D2) is preferred over hard-error because a single misconfigured
org must not block all other tenants' query results. The partial-failure model
(BC-2.01.010) is the established mechanism for per-sensor and per-org skip semantics in
this engine. The skip is not silent — the structured `query.org_slug_resolution_failure`
warn event surfaces the configuration gap to the operator through the observability pipeline.

The `"synthetic-unmapped"` sentinel (Site 3) is categorically different from the truncation
defect: it collapses multiple distinct orgs into a single cache partition rather than just
creating pairwise collisions. This is a total cross-tenant collapse risk for affected org IDs
and must be eliminated, not just suppressed.

---

## Consequences

### Positive
- Tenant data isolation (AD-017) is restored for all three defect sites.
  `derive_response_cache_key` always receives an authoritative `OrgSlug`.
- The `"synthetic-unmapped"` sentinel is removed; no code path can accidentally serve all
  affected orgs' data under a single shared cache key.
- The `resolve_source_refs` comment ("skip this target") now accurately describes the
  production behavior (D2: skip + warn).
- Bare-filter queries in production are correctly partitioned by org.

### Negative / Trade-offs
- Queries against a system where any org in the registry has no slug mapping will skip that
  org's data and emit a warn. Operators must ensure all registered org adapters have
  corresponding `OrgRegistry` slug entries. This is a configuration requirement, not a code
  change.
- `org_slug_from_uuid_prefix.rs` is removed. If that test was also covering other behavior,
  the implementer must migrate those other assertions to a separate test.

---

## Alternatives Considered

**Alt-A: Hard-Error (E-QUERY-043) for registry-present-slug-missing** — Considered but
rejected. A hard error blocks all tenants from receiving query results if one tenant's
`OrgRegistry` entry is missing. In a multi-tenant MSSP SOC deployment, blocking all tenants
because of one misconfiguration is operationally disproportionate and creates a denial-of-service
risk. Skip-with-warn is fail-closed (no wrong data served) while allowing other tenants to
receive correct results. The structured warn event ensures the gap surfaces to operators.

**Alt-B: Use full OrgId UUID string as synthetic slug** — Rejected. A full UUID string is not
an authoritative `OrgSlug` — it differs from the registry slug for the same org, causing a
query with an explicit client list to miss the cache entry created by the ALL-scope path. This
cache-key-miss is the operative rejection reason. (The v1.0 text also cited an AD-017 concern
about exposing org UUIDs in structured log events; that characterization was imprecise — `OrgId`
UUIDs are tenant identifiers, not credentials; AD-017 governs credential values and does not
prohibit tenant identifier emission in operator tracing; the D2 `tracing::warn!` correctly
emits `org_id = %org_id` as a full UUID.)

**Alt-C: Amend an existing ADR** — The existing multi-tenant ADRs (ADR-006, ADR-008, ADR-034)
address different subsystems. ADR-060 addresses the same pipeline function but a different
mechanism (early-stop pagination). A new ADR provides a clean decision record for the
cache-key isolation invariant without polluting an unrelated ADR's decision space.

---

## Source / Origin

F-R16-P1-HIGH-001 (LOCAL adversary cascade round-16 pass-1, 2026-08-27), elevated to CRITICAL
by security review (CWE-284/CWE-340/CWE-200, OWASP A01). Three production sites in
`prism-query::materialization` synthesize `client_id` from a truncated `OrgId` substring or
a static sentinel, bypassing the `OrgRegistry` that is present and populated in production.
Two orgs onboarded within the same ~65-second UUIDv7 timestamp window collide on a single
cache key; the sentinel collapses all affected orgs to one partition. Human-approved to fold
the fix into S-ENGINE-LIMIT-EARLY-STOP-001 (2026-08-27).

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-08-28 | architect | F-R16-P9-MED-001 / F-R16-P10-OBS-001: §D8 `org_id` field description corrected from stale "8-char prefix for diagnostics" to full OrgId UUID via `%display`; AD-017 inapplicability note added (OrgId UUID is a tenant identifier, not a credential; full UUID required for unambiguous misconfigured-org identification; emitting it in operator tracing is acceptable under AD-017). §Alternatives Alt-B AD-017 characterization corrected — cache-key-miss is the operative rejection ground; v1.0 AD-017 cite was imprecise given D2 already emits `org_id = %org_id` as full UUID. No behavioral change to D1–D9. |
| 1.1 | 2026-08-28 | architect | F-R16-P5-HIGH-001: §D3 corrected — non-compiling false-premise synthesis example replaced with actual infallible `OrgSlug::new` call valid by construction; §Context Site 3 false "starts with a digit" premise corrected (the `"org-"` literal prefix ensures ORG_SLUG_PATTERN compliance unconditionally; sentinel path is unreachable by construction); §D3 prose "x-prefix form resolves digit-start collision" false rationale removed. No behavioral change to D1, D2, D4, D5, D6, D7, D8, or D9. Closes F-R16-P5-HIGH-001. |
| 1.0 | 2026-08-27 | architect | Initial — three defect sites (Site 1: Step 3b bare-filter; Site 2: resolve_source_refs ALL-scope fallback; Site 3: "synthetic-unmapped" sentinel). D1 cache-key identity invariant; D2 fail-closed skip-with-structured-warn policy (justification vs hard-error); D3 test-mode synthetic preservation; D4 Site 1 fix; D5 Site 2 fix; D6 SINGLE-BINDING COHERENCE extension; D7 test-removal obligation; D8 SAP-1 catalog row obligation; D9 Red Gate gates anchored to S-ENGINE-LIMIT-EARLY-STOP-001. Severity CRITICAL (CWE-284/CWE-340/CWE-200, OWASP A01). Closes F-R16-P1-HIGH-001. |
