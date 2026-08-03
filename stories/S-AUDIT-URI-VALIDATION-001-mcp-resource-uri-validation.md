---
document_type: story
story_id: "S-AUDIT-URI-VALIDATION-001"
title: "MCP resource URI validation — reject malformed resource URIs in dispatch_read_resource rather than best-effort parsing"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "0.2"
spec_version: "v0.2"
level: ops
producer: story-writer
timestamp: "2026-07-12"
modified: "2026-07-12"
input-hash: ""
inputs:
  - crates/prism-mcp/src/server.rs
  - crates/prism-mcp/src/resources.rs
  - .factory/specs/behavioral-contracts/BC-2.10.008-mcp-resources.md
traces_to: "F-AUD-P24-MED-002"
origin_finding: "F-AUD-P24-MED-002 [implementation gap]"
origin_cascade: "AUDIT-COVERAGE-001 B-hardening; D-1696 (passes 22–25); LOCAL 3-CLEAN converged D-1713 (2026-07-12)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-10]
crates_touched:
  - prism-mcp
target_module: "crates/prism-mcp/src/server.rs"
behavioral_contracts: []
# BC status: pending PO authorship
# F-AUD-P24-MED-002 is an implementation gap in URI validation for MCP resource dispatch.
# BC-2.10.008 (MCP Resources) governs the resources surface but does not explicitly specify
# the behavior for malformed resource URIs (reject vs. best-effort parse).
# PO must either amend BC-2.10.008 with a URI validation invariant or author a companion BC.
# This story's first task (AC-001) specifies that the implementer reads BC-2.10.008 and
# confirms with the PO what the required behavior is before implementing.
# Status must remain draft until a BC is authored and anchored (S-7.01).
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.5
risk: MEDIUM
acceptance_criteria_count: 4
red_gate_tests: 4
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-AUDIT-URI-VALIDATION-001: MCP resource URI validation — reject malformed resource URIs in dispatch_read_resource

## §Origin — [implementation gap] F-AUD-P24-MED-002

**Cascade:** AUDIT-COVERAGE-001 B-hardening; finding surfaced at pass 24
**Session record:** D-1696 (SESSION WRAP passes 22–25; 4 MED findings at pass-24 including F-AUD-P24-MED-002)
**Convergence:** LOCAL 3-CLEAN(strict) D-1713 (2026-07-12); S-7.02 codification gate now due

At pass 24 the adversary found that the MCP resource dispatch in `prism-mcp` uses best-effort
URI parsing: when a client provides a malformed or unrecognized resource URI, the server attempts
to match it against known routes and falls through to an ambiguous error state rather than
returning a structured, well-typed `RESOURCE_NOT_FOUND` or `INVALID_PARAMS` error with
actionable guidance.

The gap: `crates/prism-mcp/src/server.rs` (and/or `resources.rs`) matches resource URIs by
pattern string comparison. A URI that does not start with `prism://` or has an unrecognized
path gets a generic "resource not found" response without validating that the URI was even
structurally well-formed as a `prism://` scheme URI. This means:
- `prism://` (bare scheme, no path) — matched against the table, falls through
- `http://example.com` (wrong scheme) — matched against the table, falls through
- `prism//config/clients` (missing colon) — matched against the table, falls through
- `prism://config/clients///` (triple slash, ambiguous) — matched against the table

The proposed fix: add a URI validation gate at the start of `dispatch_read_resource` (or
equivalent resource-dispatch entry point) that explicitly validates the URI is a well-formed
`prism://` scheme URI before attempting pattern matching. Malformed URIs return a structured
MCP error before reaching the routing table.

## Narrative

As a Prism MCP client (or an AI agent consuming the MCP server), I want malformed resource URIs
to return a clear, structured error response with the URI, the validation failure reason, and
the list of valid URI prefixes, so that debugging malformed resource requests does not require
examining server internals.

## Authority

BC-2.10.008 is the closest governing contract for this story. Read it before implementing:
`.factory/specs/behavioral-contracts/BC-2.10.008-mcp-resources.md` (status: `active`).

BC-2.10.008 §Preconditions and §Postconditions govern the `prism://config/clients` and `prism://config/clients/{client_id}/sensors` MCP resources. BC-2.10.008 does not yet specify behavior for malformed URIs — AC-001 requires the product-owner to amend BC-2.10.008 §Invariants with a URI validation invariant and to add E-MCP-RESOURCE-* error cases before implementation begins. The `behavioral_contracts: []` frontmatter reflects that no BC clause currently covers malformed URI rejection; it must be updated post-amendment per this story's AC-001.

**`.factory/specs/prd-supplements/error-taxonomy.md`** governs all E-NNN error code assignments for this project. New E-MCP-RESOURCE-NNN codes introduced by this story must be added to the error taxonomy before the PR merges (see §Architecture Compliance Rules).

The code artifacts to modify: `crates/prism-mcp/src/server.rs` and/or `crates/prism-mcp/src/resources.rs` (SS-10 per `architecture/module-decomposition.md §Subsystem Registry`). See also `architecture/api-surface.md §MCP Resources` for the resource URI registry.

---

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.10.008 | MCP Resources for Client List and Sensor Inventory | v1.12 | Governs `prism://config/clients` and `prism://config/clients/{client_id}/sensors` resources. Does not currently specify behavior for malformed URIs. PO must amend or add a companion clause before this story can advance to ready. |

**Note:** The story is filed against BC-2.10.008 as the closest governing BC. However,
`behavioral_contracts: []` is set in frontmatter because BC-2.10.008 does not yet contain
a clause covering malformed URI rejection. The first task in this story is to surface the
gap to the product-owner for BC amendment.

## Acceptance Criteria

### AC-001 — Product-owner amends BC-2.10.008 (or authors companion BC) with URI validation invariant
(prerequisite before implementation)

**This AC requires PO action before implementation begins.** The product-owner reads
BC-2.10.008 §Invariants and §Error Cases, then authors an invariant and error case covering
malformed `prism://` resource URIs:

Proposed invariant (for PO review/edit):
> INV-MCP-RESOURCE-URI: Every `prism://` resource URI received in `resources/read` must be
> structurally well-formed: (1) scheme MUST be `prism`, (2) path MUST be non-empty and MUST
> start with `/`, (3) path MUST match a registered resource template prefix. URIs failing
> validation MUST return a structured MCP error before pattern matching occurs.

Proposed error case (for PO review/edit):
> `E-MCP-RESOURCE-001`: URI does not start with `prism://` → JSON-RPC `-32602 INVALID_PARAMS`
> with message: "Invalid resource URI scheme. Expected `prism://`, got `<scheme>://`."
> `E-MCP-RESOURCE-002`: URI has well-formed scheme but unrecognized path →
> JSON-RPC `-32002` (or existing resource-not-found code) with message: "Resource not found:
> `<uri>`. Available resource prefixes: [prism://config/clients, prism://sensors/health, ...]"

**Outcome of this AC:** BC-2.10.008 is amended with the PO's ratified version of INV and error
cases. The story's `behavioral_contracts` field is updated with the BC version post-amendment.
No code changes in this AC.

### AC-002 — URI validation gate added at start of resource dispatch
(traces to BC-2.10.008 amended invariant INV-MCP-RESOURCE-URI once authored)

`crates/prism-mcp/src/server.rs` (or `resources.rs` — whichever owns the `resources/read` handler)
gains a URI validation step before the pattern-match routing table:

```rust
fn validate_resource_uri(uri: &str) -> Result<(), rmcp::model::ErrorData> {
    if !uri.starts_with("prism://") {
        return Err(/* E-MCP-RESOURCE-001 structured error */);
    }
    let path = &uri["prism://".len()..];
    if path.is_empty() || !path.starts_with('/') {
        return Err(/* E-MCP-RESOURCE-001 or -002 structured error */);
    }
    Ok(())
}
```

This function is called at the entry point of the resource dispatch before any routing.

Red Gate test: `test_BC_2_10_008_uri_validation_rejects_malformed_scheme` asserts that a
`resources/read` call with URI `http://example.com/clients` returns JSON-RPC `-32602` before
reaching any route handler.

### AC-003 — Malformed URI returns structured error with actionable guidance
(traces to BC-2.10.008 amended error case E-MCP-RESOURCE-001/002 once authored)

A `resources/read` request with a malformed URI returns a structured MCP error response with:
- `isError: true`
- `content[0].text` parseable as JSON containing: `error_code` (string, one of the E-MCP-RESOURCE-*
  codes once authored), `uri` (the submitted URI), `reason` (human-readable), and `valid_prefixes`
  (list of known `prism://` resource prefixes).

Red Gate test: `test_BC_2_10_008_uri_validation_error_body_structured` asserts that the error
response body contains all four required fields.

### AC-004 — Well-formed but unrecognized URIs return consistent resource-not-found error
(traces to BC-2.10.008 amended error case E-MCP-RESOURCE-002 once authored)

A `resources/read` request with a well-formed `prism://` URI that does not match any registered
route (e.g., `prism://nonexistent/path`) returns a structured error distinct from the
malformed-URI error (AC-003), containing the submitted URI and the list of valid resource
prefixes.

Red Gate test: `test_BC_2_10_008_uri_validation_unknown_path_structured_error` asserts the
distinct error shape for a syntactically valid but unrecognized `prism://` URI.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `validate_resource_uri` helper | `crates/prism-mcp/src/server.rs` or `resources.rs` | Pure (string validation, returns Result) |
| `resources/read` dispatch handler | `crates/prism-mcp/src/server.rs` | Effectful (MCP handler, uses `validate_resource_uri`) |
| URI validation unit tests | `crates/prism-mcp/tests/resources.rs` or inline `#[cfg(test)]` | Pure unit tests |

Architecture section references:
- `architecture/api-surface.md` §MCP Resources (resource URI registry)
- `architecture/module-decomposition.md` §SS-10 MCP Interface

**Anchor justifications (POL-4/POL-5):**
- SS-10 owns this story's scope because `prism-mcp/src/server.rs` and `resources.rs` are SS-10
  artifacts per the ARCH-INDEX Subsystem Registry.
- No `depends_on` dependencies: URI validation is self-contained and requires no prerequisite
  story. However, the PO must complete AC-001 (BC amendment) before the implementer begins
  AC-002/003/004.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | URI is `prism://` (bare scheme, empty path) | Returns E-MCP-RESOURCE-001: path is empty after `prism://`; before this story, fell through the routing table |
| EC-002 | URI has `prism://` prefix but contains URL-encoded characters (e.g., `prism://config/clients/%41cme`) | Validation passes the scheme check; routing handles or rejects as resource-not-found; URL-decoding is a separate concern |
| EC-003 | Client sends a resource URI with trailing slash (`prism://config/clients/`) | Routing behavior is unchanged by the validation gate; only scheme and non-empty-path are gated here |
| EC-004 | Multiple concurrent requests with different malformed URIs | Validation is pure and stateless; no contention |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~140 | ~2,000 |
| BC-2.10.008 (MCP resources BC) | ~250 | ~3,500 |
| crates/prism-mcp/src/server.rs (dispatch region) | ~150 | ~2,100 |
| crates/prism-mcp/src/resources.rs (resource routing) | ~150 | ~2,100 |
| crates/prism-mcp/tests/resources.rs (existing tests for context) | ~100 | ~1,400 |
| **Total estimate** | | **~11,100 tokens** |

Fits within a 100k-token agent context window (~11%). No split required.

## Tasks

- [ ] Read BC-2.10.008 §Invariants and §Error Cases; confirm no existing URI validation invariant.
- [ ] Surface AC-001 to the product-owner for BC amendment before writing any code.
- [ ] After PO amends BC-2.10.008: update this story's `behavioral_contracts:` frontmatter with the BC ID + version.
- [ ] Read `crates/prism-mcp/src/server.rs` to identify the `resources/read` dispatch entry point.
- [ ] Write Red Gate tests AC-002, AC-003, AC-004 BEFORE implementing `validate_resource_uri` (TDD strict).
- [ ] Implement `validate_resource_uri` helper and call it from the dispatch entry point.
- [ ] Run `just iter prism-mcp` to confirm GREEN.
- [ ] Run `just check` (full workspace) before declaring done.
- [ ] Update this story's AC traces with the final BC version once PO has amended BC-2.10.008.

## Previous Story Intelligence

N/A — first story targeting MCP resource URI validation. Prior context:
- The audit script's H-section (resource checks) tests the `prism://config/clients` and
  `prism://config/clients/{org}/sensors` routes; those routes were implemented as part of S-5.03.
- BC-2.10.008 was last amended at v1.12 (RECONCILIATION-1B-sensor-inventory-shape-2026-06-18);
  the amendment added `display_name` and reconciled the `SensorConfigEntry` shape.
- No prior story has added URI validation to the MCP resource dispatch.

## Architecture Compliance Rules

- **`#[non_exhaustive]` discipline:** If a new public struct `ResourceUriError` is introduced,
  it requires `#[non_exhaustive]`. Check CLAUDE.md §non_exhaustive discipline and add to
  `scripts/check-non-exhaustive.sh` EXPECTED count + `check-non-exhaustive-per-symbol.py`
  EXPECTED_SYMBOLS list.
- **Error taxonomy:** The E-MCP-RESOURCE-NNN error codes introduced by this story must be added
  to `.factory/specs/prd-supplements/error-taxonomy.md` before the PR merges (PG-LP11-001 principle,
  though this is MCP not tracing). The implementer proposes the codes; PO ratifies as part of BC amendment.
- **No `unwrap()` / `expect()` in production code:** `validate_resource_uri` must use `?` or
  explicit `Err(...)` returns; no panics on invalid URI.
- **TD-VSDD-091:** Cite function names (`validate_resource_uri`, `dispatch_read_resource`), NOT line numbers.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `rmcp` | workspace-pinned | `rmcp::model::ErrorData` for structured error construction |
| `nextest` | workspace-pinned | `just iter prism-mcp` for fast inner loop |

No new dependencies. URI validation uses standard string operations only.

**Forbidden dependencies (build-time enforcement):** `prism-mcp` MUST NOT import `prism-query` or
`prism-sensors` crates. The existing perimeter rules for prism-mcp apply unchanged.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-mcp/src/server.rs` | Modify | Add `validate_resource_uri` call at dispatch entry point; or delegate to `resources.rs` |
| `crates/prism-mcp/src/resources.rs` | Modify (if dispatch logic lives here) | Add `validate_resource_uri` helper function |
| `crates/prism-mcp/tests/resources.rs` | Modify | Add 3 Red Gate tests (AC-002/003/004) |
| `.factory/specs/behavioral-contracts/BC-2.10.008-mcp-resources.md` | Modify (PO action, AC-001) | Add URI validation invariant + error cases |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Modify | Add E-MCP-RESOURCE-001/002 rows |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 0.2 | DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001-R6 | 2026-08-02 | story-writer | Add §Authority section (D-2084 Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001). BC-2.10.008 §Preconditions/§Postconditions cited as governing contract; error-taxonomy.md cited for E-MCP-RESOURCE-NNN codes; architecture/api-surface.md §MCP Resources noted. |
| 0.1 | — | 2026-07-12 | story-writer | Initial story creation. |
