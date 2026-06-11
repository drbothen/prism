---
document_type: adr
adr_id: "ADR-038"
title: "E-CFG Runtime Namespace Reconciliation — Canonical E-CFG-100..106 Runtime Codes, PrismError Renumber Map, and Client-Not-Found Variant Split"
status: accepted
date: 2026-06-11
author: architect
decision_made_by: human (approved 2026-06-10, full acceptance of all decisions D1-D6 incl. ClientNotFound variant split D3 and MCP -32602 mapping D4; drift surfaced by PO taxonomy burst at error-taxonomy v1.65; routed via orchestrator per Source-of-Truth Rule 7)
supersedes: null
superseded_by: null
related_adrs: ["ADR-035", "ADR-037"]
related_bcs: ["BC-2.11.001", "BC-2.11.008", "BC-2.11.011", "BC-2.11.013", "BC-2.11.014", "BC-2.10.004", "BC-2.08.008", "BC-2.14.010"]
traces_to: "ARCH-INDEX.md"
subsystems_affected: ["SS-21", "SS-06", "SS-04", "SS-10", "SS-11"]
drift_anchor: "DRIFT-ECFG-TAXONOMY-001"
story_anchor: "fix/review-2026-06-10-query-core (Fix PR 2 cascade)"
---

# ADR-038: E-CFG Runtime Namespace Reconciliation — Canonical E-CFG-100..106 Runtime Codes, PrismError Renumber Map, and Client-Not-Found Variant Split

## Status

ACCEPTED 2026-06-10 v1.1. Human-approved 2026-06-10 (full acceptance — all decisions
D1-D6, explicitly including the ClientNotFound variant split D3 and the MCP `-32602`
mapping D4). Authored by the architect in the 2026-06-10 review cycle, mirroring the
ADR-035 (E-CRED) reconciliation pattern. The implementer work-order (§Blast-Radius
Inventory, implementer section) executes in the **Fix PR 2 cascade** on
`fix/review-2026-06-10-query-core`, where `prism-core/src/error.rs` is already in flight.

Subsystem anchor justification (per Subsystem Registry): SS-21 because `PrismError` and
`CapabilityPath` live in prism-core (Identity & Core Types); SS-06 because the
client-not-found condition is a Client Configuration surface; SS-04 because
`list_capabilities` (prism-security) emits a renumbered code; SS-10 because
`prism-mcp::error_mapping` carries the JSON-RPC mapping; SS-11 because the alias tools
(prism-query) are the highest-volume emitters of the migrated condition.

## Context

### The Drift (DRIFT-ECFG-TAXONOMY-001)

`error-taxonomy.md` v1.8 renumbered the original runtime client-config codes
E-CFG-001..004 to **E-CFG-100..103**, freeing the 001..031 range for Wave 3
customer-config startup-validation semantics (BC-3.3.001..004, ADR-010). ADR-037
(2026-06-10) then retired the entire customer-config surface: all 23 rows in the
E-CFG-000..031 range are now RETIRED with append-only tombstones (error-taxonomy v1.65).

`prism_core::PrismError`, however, was never migrated to the v1.8 numbering. It still
emits the pre-v1.8-style low numbers, and added two codes that were never cataloged at
all. The live code-side namespace as of this ADR:

| Code emitted | Variant | Display | Taxonomy says this number means |
|--------------|---------|---------|--------------------------------|
| E-CFG-001 | `ConfigNotFound { path }` | "E-CFG-001: config file not found: {path}" | RETIRED: missing required field in customers/*.toml (R-CUST-001) |
| E-CFG-002 | `ConfigParseFailed { detail }` | "E-CFG-002: config parse error: {detail}" | RETIRED: org_slug/filename-stem mismatch (R-CUST-002) |
| E-CFG-003 | `ConfigValidationFailed { detail }` | "E-CFG-003: config validation failed: {detail}" | RETIRED: org_id not UUID v7 (R-CUST-003) |
| E-CFG-010 | `ConfigSnapshotStale { current, required }` | "E-CFG-010: config snapshot stale: version {current} < required {required}" | RETIRED: unknown field / deny_unknown_fields (R-CUST-010) |
| E-CFG-020 | `InvalidCapabilityPath { reason }` | "E-CFG-020: invalid capability path: {reason}" | RETIRED: literal credential detected (R-CRED-001..006) |

Every live emission collides with a tombstoned number of entirely different semantics. A
monitoring rule, operator runbook, or AI agent that resolves `E-CFG-001` against the
taxonomy gets "missing required field in customers/{file}" while the process actually
said "config file not found". This is the same defect class ADR-035 resolved for E-CRED
(DRIFT-ECRED-TAXONOMY-001); per Source-of-Truth Rule 7, the SPEC (taxonomy v1.8
renumbering + ADR-037 tombstones) wins and the code realigns.

### The Hidden Second Defect: Variant Misuse for Client-Not-Found

Exhaustive caller survey (this ADR's pre-work) found that the declared variant semantics
and the call-site semantics have themselves diverged:

1. **`PrismError::ConfigNotFound`** is declared as "config file not found" — but it has
   ZERO file-not-found callers. All four production emitters are in
   `prism-query::alias_tools` (the client-scope validation guards in the create_alias /
   list_aliases / delete_alias paths, including the CR-007 no-valid-client-list guard),
   and all four emit it for **"scope references a client ID that is not configured"**,
   stuffing `path: "client:{id}"`. That condition is taxonomy **E-CFG-100** ("Client
   '{client_id}' not found in configuration"), not file-not-found.

2. **`PrismError::ConfigValidationFailed`** has exactly one production emitter — the
   per-client capability lookup in `prism-security::list_capabilities` — and it too
   fires for **"unknown client_id"**, i.e. the E-CFG-100 condition.

3. Eight ACTIVE behavioral contracts pin `E-CFG-001` for the client-not-found condition
   (BC-2.11.001, BC-2.11.008, BC-2.11.011, BC-2.11.013, BC-2.11.014, BC-2.10.004,
   BC-2.08.008, BC-2.14.010), plus test-vectors.md. These BCs froze the **pre-v1.8**
   number for the condition the taxonomy moved to E-CFG-100 in v1.8 — the BCs were never
   swept when the taxonomy renumbered.

4. **`ConfigParseFailed`** and **`ConfigSnapshotStale`** have zero production emitters
   (declared + matched in `error_mapping` only).

5. A third defect surfaced by the same survey: `prism-mcp::error_mapping` maps ALL
   config-family variants (including the client-not-found emissions) to JSON-RPC
   `-32000 INTERNAL_ERROR` with the opaque message "Internal error; see audit log" —
   directly contradicting the eight BCs, which require a structured caller-visible error
   ("Structured error listing valid client IDs" / "Client '{id}' not found"). A wrong
   `client_id` is a caller-parameter error, not an internal failure.

### Why Reconcile Now

`error.rs` is already in flight on `fix/review-2026-06-10-query-core` (Fix PR 2 cascade),
making this the cheapest moment to renumber. The PO taxonomy burst (error-taxonomy v1.65)
annotated the drift with interim NOTEs and explicitly routed adjudication to an
"ADR-035-style reconciliation (architect + implementer)". This ADR is that adjudication.

## Decision

### D1: Canonical Runtime E-CFG Namespace (E-CFG-100..106)

The runtime config-error namespace is the **100-range**, extending the v1.8 block. The
following table is authoritative; every runtime condition has exactly one code, and every
code has exactly one condition.

| Code | Canonical Name / Variant | Condition | Canonical Display Format | Category | Retryable |
|------|--------------------------|-----------|--------------------------|----------|-----------|
| E-CFG-100 | `ClientNotFound` (NEW variant) | Referenced `client_id` is not configured (tool param or scope references unknown client) | `"E-CFG-100: client '{client_id}' not found in configuration"` | configuration | No |
| E-CFG-101 | *(spec-declared; no PrismError variant yet)* | Required field missing from runtime config TOML | `"E-CFG-101: missing required field: {toml_path}"` | configuration | No |
| E-CFG-102 | `ConfigValidationFailed` | Runtime config value/semantic validation failure (type mismatch, invalid value) | `"E-CFG-102: config validation failed: {detail}"` — `{detail}` SHOULD carry `{toml_path}`, expected, and actual | configuration | No |
| E-CFG-103 | `ConfigNotFound` | Config file does not exist at the resolved path | `"E-CFG-103: config file not found: {path}"` | configuration | No |
| E-CFG-104 | `ConfigParseFailed` | TOML structural/parse error on a runtime config surface (e.g., AD-007 hot-reload) | `"E-CFG-104: config parse error: {detail}"` | configuration | No |
| E-CFG-105 | `ConfigSnapshotStale` | ArcSwap config snapshot version below required version (AD-007 hot-reload surface) | `"E-CFG-105: config snapshot stale: version {current} < required {required}"` | transient | Yes — retry acquires a fresh `ArcSwap::load()` snapshot |
| E-CFG-106 | `InvalidCapabilityPath` | Capability path string fails `CapabilityPath::new()` format rules (empty, empty segment, invalid chars, >8 segments, >256 chars) | `"E-CFG-106: invalid capability path: {reason}"` | validation | No |

Message formats are regularized to include the `E-CFG-1NN:` prefix, matching the ADR-035
E-CRED canonical-row convention. The existing E-CFG-100/101/103 taxonomy conditions are
unchanged in meaning; E-CFG-102's message is harmonized to the variant's `{detail}` shape
(see Rationale).

### D2: PrismError Renumber Map

| Old code | Variant | New code | Action |
|----------|---------|----------|--------|
| E-CFG-001 | `ConfigNotFound { path }` | **E-CFG-103** | Renumber doc comment + `#[error]` display. Variant keeps its declared file-not-found semantics. |
| E-CFG-002 | `ConfigParseFailed { detail }` | **E-CFG-104** | Renumber + NEW taxonomy row (zero emitters today; forward-declared for the AD-007 hot-reload surface). |
| E-CFG-003 | `ConfigValidationFailed { detail }` | **E-CFG-102** | Renumber. Semantic match with the v1.8 row ("TOML type or value validation failure"). |
| E-CFG-010 | `ConfigSnapshotStale { current, required }` | **E-CFG-105** | Renumber + NEW taxonomy row. |
| E-CFG-020 | `InvalidCapabilityPath { reason }` | **E-CFG-106** | Renumber + NEW taxonomy row. |
| — | `ClientNotFound { client_id }` (NEW) | **E-CFG-100** | New variant; receives the migrated call sites from D3. |

### D3: Client-Not-Found Variant Split

A new variant `PrismError::ClientNotFound { client_id: String }` with display
`"E-CFG-100: client '{client_id}' not found in configuration"` is added. The five
production call sites that today misuse config-file variants for the client-not-found
condition migrate to it:

- `prism-query::alias_tools` — all four `ConfigNotFound { path: "client:{id}" }` guard
  sites (create / list / delete client-scope validation, including the CR-007 guard)
  → `ClientNotFound { client_id }`.
- `prism-security::list_capabilities` — the per-client capability lookup's
  `ConfigValidationFailed` → `ClientNotFound { client_id }`.

After migration, `ConfigNotFound`, `ConfigParseFailed`, `ConfigValidationFailed`, and
`ConfigSnapshotStale` have zero production emitters. They are RETAINED (not retired):
unlike ADR-035's `KeyringError` (retired for semantic redundancy with E-CRED-008), each
of these covers a real, unique, architecture-anchored condition — the AD-007 ArcSwap
hot-reload surface needs runtime parse/validation/staleness errors that cannot use the
boot-time exit-2 stderr path (BC-2.21.001 boot validation deliberately carries no E-code).

### D4: MCP error_mapping Realignment

- `ClientNotFound` gets an **explicit arm** mapped to `-32602 INVALID_PARAMS` with the
  display string (`format!("{err}")`). This closes the BC contradiction in §Context: a
  wrong `client_id` is caller-resolvable. The arm MUST be added explicitly — `PrismError`
  is `#[non_exhaustive]`, so an unmatched new variant would silently fall through to the
  catch-all and regress to an opaque internal error.
- `ConfigNotFound` / `ConfigParseFailed` / `ConfigValidationFailed` / `ConfigSnapshotStale`
  keep `-32000 INTERNAL_ERROR` (operator-resolvable, not caller-resolvable). Unchanged.
- `InvalidCapabilityPath` keeps `-32602 INVALID_PARAMS`; the `// E-CFG-020:` comment
  renumbers to `// E-CFG-106:`.

### D5: Tombstone Permanence

The retired customer-config numbers **E-CFG-000 through E-CFG-031** (all 23 cataloged
rows: 000..020, 030, 031) are **permanently tombstoned** per POL-1 append-only numbering
(DF-030). They are never reusable for any future condition. After the D2 migration
executes, the number strings `E-CFG-001`, `E-CFG-002`, `E-CFG-003`, `E-CFG-010`, and
`E-CFG-020` refer EXCLUSIVELY to the retired customer-config semantics preserved in the
taxonomy's tombstone rows; no live code path may emit them. Future runtime extension
allocates **E-CFG-107 onward**; the 000..031 range is closed forever.

### D6: BC and Test-Vector Renumber Sweep (E-CFG-001 → E-CFG-100)

The eight active BCs and the test-vector supplement that pin `E-CFG-001` for the
client-not-found condition are renumbered to **E-CFG-100** (pure number sweep; the
contractual condition and response-shape language are already correct):
BC-2.11.001, BC-2.11.008, BC-2.11.011, BC-2.11.013, BC-2.11.014, BC-2.10.004,
BC-2.08.008, BC-2.14.010, and `test-vectors.md` (the "no clients configured" rejection
row). Owner: product-owner (see §Blast-Radius Inventory).

## Rationale

### Why the 100-range (not reclaiming low numbers)

The taxonomy already established the 100-range as the runtime block in v1.8, and ADR-037
tombstoned 000..031 with per-row successor annotations that downstream documents (BC
bodies, hooks, the `validate-error-taxonomy-retirement-annotations.sh` validator) now
depend on. Reusing any low number would violate POL-1 append-only numbering and reopen
the exact ambiguity this ADR closes. Extending 100..103 to 100..106 preserves sequential
density in the runtime block, mirroring ADR-035's "no gaps through 009" principle.

### Why ConfigNotFound → E-CFG-103 (not E-CFG-100)

The variant's declared semantics ("config file not found: {path}") are an exact match for
the v1.8 row E-CFG-103 ("Configuration file not found at {path}", formerly E-CFG-004).
The variant's current CALLERS, however, carry E-CFG-100 semantics — which is why D3
splits the callers out to a new variant instead of renumbering `ConfigNotFound` to 100.
Renumbering the variant to 100 would have blessed the misuse: the next genuine
file-not-found emitter would have found the variant's display lying about its condition.
Code↔condition 1:1 is restored by giving each condition its own variant.

### Why a NEW ClientNotFound variant (not patching display strings)

The client-not-found condition appears in eight active BCs and is the single most
caller-visible config error in the MCP surface. Encoding it as
`ConfigNotFound { path: "client:{id}" }` loses the typed `client_id`, fabricates a fake
"path", and forces the MCP layer to treat a parameter error as an internal error. A
dedicated variant carries the typed field, enables the `-32602` mapping (D4), and gives
the BCs' "Structured error" response shape a truthful source. The BCs' fuller
"listing valid client IDs" enrichment remains a tool-layer response concern owned by the
respective tool implementations — this ADR fixes the code string and the JSON-RPC class,
which are the namespace-level facts.

### Why ConfigValidationFailed → E-CFG-102 with a harmonized message

The v1.8 row's structured message (`"Invalid value for {toml_path}: expected {expected},
got {actual}"`) and the variant's `{detail}` field cover the same condition class
("TOML type or value validation failure" — the row's own description) at different
granularity. Neither has a live emitter after D3 migrates the list_capabilities misuse.
One canonical display must win (ADR-035 principle: one code, one format). The variant's
`{detail}` form wins because it is the shape real call sites can always satisfy; the
structured fields become SHOULD-carry content inside `{detail}`. This is an
architect-adjudicated spec amendment via this ADR — the same vehicle ADR-035 used to
regularize E-CRED-005's three-subtype display.

### Why ConfigParseFailed gets a NEW code (E-CFG-104) instead of mapping to E-CFG-101

E-CFG-101 means "missing required field" — a field-level validation failure. A TOML
structural parse error is a different condition (the document never deserialized; no
field path exists). Mapping parse→101 would recreate the code↔condition mismatch this
ADR exists to eliminate. The taxonomy's 100-range simply never had a parse-error row;
E-CFG-104 fills it.

### Why zero-emitter variants are kept (contrast with ADR-035 D4)

ADR-035 retired `KeyringError` because its condition was IDENTICAL to another code's
(E-CRED-008) — redundancy, not mere zero-caller status. `ConfigParseFailed`,
`ConfigSnapshotStale`, and (post-split) `ConfigNotFound` / `ConfigValidationFailed` each
cover a unique condition anchored to the AD-007 ArcSwap hot-reload architecture, where
runtime config errors cannot use the boot-time exit-code path. Retiring them would force
re-adding them when the hot-reload surface wires up. Kept + cataloged.

### Why E-CFG-105 is Retryable=Yes

Snapshot staleness is self-healing: the condition exists precisely because a newer config
snapshot is already available; re-executing the operation loads it via
`ArcSwap::load()` (AD-007 read discipline). This is the taxonomy's "transient" category.
All other rows remain Retryable=No (operator/caller intervention required).

## Full Migration Mapping Table

| Current emission | Current source | Condition (actual) | Canonical code | Action |
|------------------|----------------|--------------------|----------------|--------|
| `E-CFG-001` via `ConfigNotFound` | alias_tools ×4 guard sites | Client not found | **E-CFG-100** via NEW `ClientNotFound` | Migrate call sites to new variant |
| `E-CFG-001` via `ConfigNotFound` | variant declaration (no file-not-found callers) | Config file not found | **E-CFG-103** | Renumber variant display |
| `E-CFG-002` via `ConfigParseFailed` | variant declaration (zero callers) | Config parse error | **E-CFG-104** | Renumber + new taxonomy row |
| `E-CFG-003` via `ConfigValidationFailed` | list_capabilities ×1 | Client not found | **E-CFG-100** via NEW `ClientNotFound` | Migrate call site to new variant |
| `E-CFG-003` via `ConfigValidationFailed` | variant declaration | Config value validation failed | **E-CFG-102** | Renumber variant display |
| `E-CFG-010` via `ConfigSnapshotStale` | variant declaration (zero callers) | Snapshot stale | **E-CFG-105** | Renumber + new taxonomy row |
| `E-CFG-020` via `InvalidCapabilityPath` | `CapabilityPath::new()` ×5 branches | Invalid capability path | **E-CFG-106** | Renumber + new taxonomy row |
| `E-CFG-001` cited in specs | 8 BCs + test-vectors.md | Client not found | **E-CFG-100** | PO number sweep |

### Canonical Display Strings After Migration

| Code | Display |
|------|---------|
| E-CFG-100 | `"E-CFG-100: client '{client_id}' not found in configuration"` (new variant) |
| E-CFG-102 | `"E-CFG-102: config validation failed: {detail}"` (renumbered from 003) |
| E-CFG-103 | `"E-CFG-103: config file not found: {path}"` (renumbered from 001) |
| E-CFG-104 | `"E-CFG-104: config parse error: {detail}"` (renumbered from 002) |
| E-CFG-105 | `"E-CFG-105: config snapshot stale: version {current} < required {required}"` (renumbered from 010) |
| E-CFG-106 | `"E-CFG-106: invalid capability path: {reason}"` (renumbered from 020) |

## Blast-Radius Inventory

### Owner: implementer (code + test changes — executes in Fix PR 2 cascade, `fix/review-2026-06-10-query-core`)

**`crates/prism-core/src/error.rs`** (already in flight on the fix branch)

| Item | Change |
|------|--------|
| `ConfigNotFound` doc comment + `#[error]` | `E-CFG-001` → `E-CFG-103` |
| `ConfigParseFailed` doc comment + `#[error]` | `E-CFG-002` → `E-CFG-104` |
| `ConfigValidationFailed` doc comment + `#[error]` | `E-CFG-003` → `E-CFG-102` |
| `ConfigSnapshotStale` doc comment + `#[error]` | `E-CFG-010` → `E-CFG-105` |
| `InvalidCapabilityPath` doc comment + `#[error]` | `E-CFG-020` → `E-CFG-106` |
| NEW variant | `ClientNotFound { client_id: String }` with `#[error("E-CFG-100: client '{client_id}' not found in configuration")]` and doc comment citing this ADR + BC-2.10.004 |

**`crates/prism-query/src/alias_tools.rs`**

| Item | Change |
|------|--------|
| Four `ConfigNotFound { path: format!("client:{id_str}") }` guard sites (create CR-007 guard, create validation, list validation, delete validation) | → `ClientNotFound { client_id: id_str.to_string() }` |
| Three doc/inline comments citing `E-CFG-001` | → `E-CFG-100` |

**`crates/prism-security/src/list_capabilities.rs`**

| Item | Change |
|------|--------|
| Per-client lookup `.ok_or_else(\|\| ConfigValidationFailed { .. })` | → `ClientNotFound { client_id }` |
| Doc comment "Returns `PrismError::ConfigValidationFailed`" | → `PrismError::ClientNotFound` / `E-CFG-100` |

**`crates/prism-mcp/src/error_mapping.rs`**

| Item | Change |
|------|--------|
| NEW explicit arm for `ClientNotFound` | `(codes::INVALID_PARAMS, format!("{err}"))` — MUST be explicit; do not let it fall to the non_exhaustive catch-all (would regress to opaque INTERNAL_ERROR and violate the eight BCs) |
| `// E-CFG-020:` comment above `InvalidCapabilityPath` arm | → `// E-CFG-106:` |
| `// E-CFG-*: Config errors → -32000 Internal` arm | Unchanged mapping; arm now covers only the four operator-class variants |
| Unit test | Add/extend mapping test: `ClientNotFound` → `-32602` with message containing `E-CFG-100` |

**`crates/prism-core/tests/ac_5_prism_error_display.rs`**

| Item | Change |
|------|--------|
| `test_ac5_prism_error_display_e_cfg_001` (asserts `"E-CFG-001"` on `ConfigNotFound`) | Rename to `test_ac5_prism_error_display_e_cfg_103`; assert `"E-CFG-103"` |
| NEW test | `test_ac5_prism_error_display_e_cfg_100` asserting `ClientNotFound` displays prefix `"E-CFG-100"` |

**`crates/prism-query/src/tests/alias_tests.rs`**

| Item | Change |
|------|--------|
| Doc comments + assert messages citing `E-CFG-001` (~7 sites across BC-2.11.008/013/014 tests) | → `E-CFG-100` |
| Loose `result.is_err()` assertions on the client-not-found paths | TIGHTEN: match `PrismError::ClientNotFound { .. }` or assert display contains `"E-CFG-100"` (the current is_err() checks would pass on any error; production-grade requires pinning the contract code) |

**`crates/prism-security/tests/bc_2_04_006_test.rs`**

| Item | Change |
|------|--------|
| Header comment "Unknown client_id returns error (`PrismError::ConfigValidationFailed`)" + any variant assertions | → `PrismError::ClientNotFound` |

**TD-VSDD-060 sibling-site sweep (mandatory before commit):**
`rg "E-CFG-001|E-CFG-002|E-CFG-003|E-CFG-010|E-CFG-020" crates/` must return ZERO
matches post-migration — excluding `crates/prism-customer-config`, which is removed
wholesale by ADR-037 via Fix PR 3 (its retired-semantics Display/test matches disappear
with the crate; until Fix PR 3 lands, sweep with `-g '!prism-customer-config/**'`). `rg "ConfigNotFound|ConfigValidationFailed" crates/` must show
only the variant declarations, `error_mapping` arms, and (if any) genuine
file-not-found/validation callers — zero client-not-found callers.
SAP-1 note: no `event_type` emissions change in this migration; no BC-2.16.002 catalog
rows required.

### Owner: product-owner (taxonomy + BC changes — PO handoff per ADR-035 precedent; rows below are verbatim-ready)

**`.factory/specs/prd-supplements/error-taxonomy.md`** (v1.65 → v1.66)

1. Retitle section "CFG-100..103: Runtime Client-Config Errors (pre-Wave 3)" →
   "CFG-100..106: Runtime Config Errors" and replace its intro sentence with: these are
   the live runtime config codes emitted by `prism_core::PrismError` (plus spec-declared
   E-CFG-101); renumbered from E-CFG-001..004 in v1.8; extended to 104..106 per ADR-038.
2. Replace/extend the section table with these rows (verbatim):

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-CFG-100 | broken | configuration | "E-CFG-100: client '{client_id}' not found in configuration" | No | Referenced client not configured (tool param or alias scope names an unknown client). Emitted by `PrismError::ClientNotFound` (ADR-038 D3 variant split). Formerly E-CFG-001 (pre-v1.8). Used in: prism-query alias tools, prism-security list_capabilities. BC-2.10.004, BC-2.11.001/008/011/013/014, BC-2.08.008, BC-2.14.010. |
| E-CFG-101 | broken | configuration | "E-CFG-101: missing required field: {toml_path}" | No | Required field missing from runtime config TOML (formerly E-CFG-002 pre-v1.8). Spec-declared; no `PrismError` variant yet — forward-declared for the AD-007 hot-reload surface (ADR-038). |
| E-CFG-102 | broken | configuration | "E-CFG-102: config validation failed: {detail}" | No | Runtime config type/value/semantic validation failure (formerly E-CFG-003 pre-v1.8). `{detail}` SHOULD carry the toml_path, expected, and actual values. Emitted by `PrismError::ConfigValidationFailed` (renumbered from code-side E-CFG-003 per ADR-038 D2). |
| E-CFG-103 | broken | configuration | "E-CFG-103: config file not found: {path}" | No | Config file does not exist at the resolved path (formerly E-CFG-004 pre-v1.8). Emitted by `PrismError::ConfigNotFound` (renumbered from code-side E-CFG-001 per ADR-038 D2). |
| E-CFG-104 | broken | configuration | "E-CFG-104: config parse error: {detail}" | No | TOML structural/parse error on a runtime config surface (AD-007 hot-reload; boot-time parse failures use the BC-2.21.001 exit-2 path with no E- code). Emitted by `PrismError::ConfigParseFailed` (renumbered from uncataloged code-side E-CFG-002 per ADR-038 D2). |
| E-CFG-105 | broken | transient | "E-CFG-105: config snapshot stale: version {current} < required {required}" | Yes | ArcSwap config snapshot version below required version (AD-007). Retry acquires a fresh `ArcSwap::load()` snapshot. Emitted by `PrismError::ConfigSnapshotStale` (renumbered from uncataloged code-side E-CFG-010 per ADR-038 D2). |
| E-CFG-106 | broken | validation | "E-CFG-106: invalid capability path: {reason}" | No | Capability path string fails `CapabilityPath::new()` format rules: empty string, empty segment, invalid characters, more than 8 segments, or total length > 256 chars. Emitted by `PrismError::InvalidCapabilityPath` (renumbered from uncataloged code-side E-CFG-020 per ADR-038 D2). Used in: prism-core capability, prism-security. |

3. Update the section-level drift NOTE (the "Namespace-collision drift note ... NOT
   adjudicated here" blockquote) to: "Adjudicated per ADR-038 (2026-06-10): live
   `prism_core::PrismError` runtime codes renumbered to E-CFG-100..106; retired
   E-CFG-000..031 numbers permanently tombstoned, never reusable (POL-1); future runtime
   codes allocate E-CFG-107 onward." Update the matching cross-reference sentence in the
   CFG-000/020/030/031 section banner.
4. The per-row interim "NOTE: number collides with live `prism_core::PrismError`..."
   sentences on retired rows E-CFG-001/002/003/010/020 are updated to past-tense
   resolution: "Collision resolved per ADR-038 — live code renumbered to E-CFG-1NN."
5. Changelog row + version bump v1.65 → v1.66 citing ADR-038.

**Behavioral contracts (number sweep E-CFG-001 → E-CFG-100; condition text already correct):**

| File | Sites |
|------|-------|
| BC-2.11.001-query-mcp-tool.md | Error table row "No matching clients/sensors found" |
| BC-2.11.008-create-alias-tool.md | Error table row "Client ID in scope does not exist" |
| BC-2.11.011-cross-client-query-scoping.md | Error table row "No clients match the intersection" |
| BC-2.11.013-list-aliases-tool.md | Error table row + canonical test vector `Err(E-CFG-001)` |
| BC-2.11.014-delete-alias-tool.md | Error table row |
| BC-2.10.004-client-id-parameter-requirement.md | Error table row `code: "E-CFG-001"` |
| BC-2.08.008-get-diagnostics-tool.md | Error table row (tenant_id not in configuration) |
| BC-2.14.010-case-metrics-tool.md | Error table row + edge-case row `E-CFG-001` |
| prd-supplements/test-vectors.md | "no clients configured" rejection row |

Each BC also gains ADR-038 in its normative references where an ADR list exists.

### Owner: architect (this ADR + ARCH-INDEX)

| Item | Change |
|------|--------|
| ARCH-INDEX.md ADR Registry | ADR-038 row added (this burst); v2.122 → v2.123 |

### Explicitly OUT of scope / no change

- `verification-architecture.md` VP-100 row and VP-INDEX retired-VP rows citing
  `E-CFG-020`/`E-CFG-030` — these cite the RETIRED customer-config semantics in
  struck-through historical rows; correct as-is. No active VP cites any live runtime
  E-CFG code (verified against VP-INDEX v1.77).
- BC-3.2.005's `E-CFG-010` citation — retired customer-config semantic
  (deny_unknown_fields), historical context, correct as-is.
- `.factory/cycles/`, `wave-state.yaml`, completed story files (S-3.04, S-5.05, S-1.03,
  etc.) — immutable historical records per ADR-035 precedent; only in-flight work
  re-baselines, and no in-flight story cites the old runtime numbers.

## Consequences

### Positive

- Zero collisions: every live E-CFG emission resolves to exactly one ACTIVE taxonomy row;
  every tombstoned number is emitted by nothing.
- The client-not-found condition — the most caller-visible config error across eight BCs —
  gains a typed variant, a truthful code (E-CFG-100), and a caller-visible `-32602`
  JSON-RPC mapping, closing the latent BC violation where it surfaced as an opaque
  internal error.
- The two previously uncataloged codes (snapshot-stale, capability-path) are cataloged;
  the taxonomy is again the definitive authority for the whole E-CFG namespace.
- Monitoring/runbook rules can match `E-CFG-1` as the live-runtime prefix; everything
  below 100 is historically frozen.

### Negative / Trade-offs

- Five display strings change number. Pre-v1 system with no external customers; blast
  radius bounded to the workspace + spec artifacts enumerated above.
- `ClientNotFound` adds one variant to `PrismError`; external matches on the
  `#[non_exhaustive]` enum absorb it via wildcard arms, but `error_mapping` must add the
  explicit arm or the BC contract silently regresses (called out in the work-order).
- E-CFG-101 remains spec-declared with no variant — a known declared-ahead row, same
  class as E-CRED-009 was in ADR-035 before its implementation.

### Implementer Work-Order Execution Notes

1. Executes in the **Fix PR 2 cascade** on `fix/review-2026-06-10-query-core` (error.rs
   already in flight there; coordinate with the in-flight diff, do not fork a second
   branch for this).
2. TD-VSDD-060 sweep is the exit criterion: zero `E-CFG-001|002|003|010|020` matches in
   `crates/` post-migration, excluding `crates/prism-customer-config` — removed by
   ADR-037 via Fix PR 3 (sweep with `-g '!prism-customer-config/**'` until that PR lands;
   sweep command in §Blast-Radius implementer section).
3. MCP `error_mapping` check: explicit `ClientNotFound` arm + unit test asserting
   `-32602` / `E-CFG-100`; verify no other arm regressed via the existing mapping tests.
4. PO taxonomy/BC burst (v1.66 + 8 BCs + test-vectors) lands in the same cycle so spec
   and code cross the line together; adversary verifies symmetry per SAP-style grep on
   both sides.

### Status as of 2026-06-10

Decision ACCEPTED (human-approved 2026-06-10, full acceptance of all decisions D1-D6
including the D3 ClientNotFound variant split and the D4 MCP `-32602` mapping).
Code migration pending the Fix PR 2 cascade (implementer work-order) and the PO
taxonomy/BC burst (error-taxonomy v1.66 + 8-BC sweep) in the same review cycle; no code
or taxonomy rows were changed by this ADR's authoring burst (ARCH-INDEX registration only).

## Alternatives Considered

- **Option A — adopt code as canonical (rewrite taxonomy to match code):** Rejected. The
  low numbers are tombstoned by ADR-037 with append-only guarantees and per-row successor
  annotations enforced by hooks; reusing them would violate POL-1 and Source-of-Truth
  Rule 7 (spec wins).
- **Option B — renumber `ConfigNotFound` to E-CFG-100 (minimum-churn patch, no new
  variant):** Rejected. It would bless the variant misuse: a "config file not found"
  variant permanently emitting "client not found" semantics, with a fake `path:
  "client:{id}"` payload and no typed client_id. The next genuine file-not-found caller
  would re-create the collision inside one variant.
- **Option C — map `ConfigParseFailed` onto existing E-CFG-101 to avoid a new row:**
  Rejected. "Missing required field" and "document failed to parse" are distinct
  conditions; merging them recreates the code↔condition mismatch this ADR eliminates.
- **Option D — retire the zero-emitter variants (ConfigParseFailed, ConfigSnapshotStale)
  per ADR-035 D4:** Rejected. ADR-035 retired `KeyringError` for semantic REDUNDANCY,
  not zero-caller status alone. These conditions are unique and anchored to the AD-007
  hot-reload surface; retiring them now forces re-adding them later.

## Source / Origin

- DRIFT-ECFG-TAXONOMY-001 — drift anchor (this ADR); surfaced by the PO taxonomy burst,
  error-taxonomy.md v1.65 section drift NOTE (2026-06-10)
- `crates/prism-core/src/error.rs` — `PrismError` E-CFG variants (`ConfigNotFound`,
  `ConfigParseFailed`, `ConfigValidationFailed`, `ConfigSnapshotStale`,
  `InvalidCapabilityPath`) — implementation evidence for the live emissions
- `crates/prism-query/src/alias_tools.rs` — four `ConfigNotFound` client-scope guard
  sites (CR-007 guard + create/list/delete validation)
- `crates/prism-security/src/list_capabilities.rs` — `ConfigValidationFailed` unknown-client
  emission
- `crates/prism-core/src/capability.rs` — `CapabilityPath::new()` five validation branches
- `crates/prism-mcp/src/error_mapping.rs` — config-family `-32000` arm + capability-path
  `-32602` arm
- `crates/prism-core/tests/ac_5_prism_error_display.rs` — `E-CFG-001` display assertion
- `crates/prism-query/src/tests/alias_tests.rs` — BC-2.11.008/013/014 client-not-found tests
- `.factory/specs/prd-supplements/error-taxonomy.md` v1.65 — E-CFG-100..103 active rows,
  E-CFG-000..031 ADR-037 tombstone rows, section drift NOTE
- ADR-035 — E-CRED reconciliation precedent (pattern mirrored here)
- ADR-037 — customer-config retirement that tombstoned the colliding low numbers
- BC-2.11.001/008/011/013/014, BC-2.10.004, BC-2.08.008, BC-2.14.010 — active E-CFG-001
  (client-not-found) citations swept to E-CFG-100 by D6

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.5 | 2026-06-11 | architect | D5-precedent addendum #4 (query-core review cascade pass-6, P6-02/P6-03 adjudication, fifth D5-family instance this cycle — no decision change, status remains ACCEPTED). **P6-02/P6-03:** Three nested-code residuals inside `PrismError::QueryExecutionFailed` ("E-QUERY-034") resolved. (1) E-QUERY-006-in-detail at `materialization.rs::resolve_source_refs` replaced by NEW code **E-QUERY-036** (`PrismError::UnknownSourceTable { source_name: String }`, display `"E-QUERY-036: unknown source table '{source_name}': table is not a registered sensor or internal table. Check spelling or register the sensor in prism.toml."`, MCP `-32602` INVALID_PARAMS — caller-resolvable per D4 principle; explicit arm REQUIRED per `#[non_exhaustive]`). (2) E-ALIAS-001-in-detail at `explain.rs` replaced by the **EXISTING** `PrismError::AliasNotFound` variant — un-nesting eliminates the embedded detail suffix; no new code allocated (existing variant already has the correct display and MCP mapping). (3) E-QUERY-009-in-detail confirmed **LATENT** (no live emission in `materialization.rs` or anywhere in the workspace; the original E-QUERY-009 condition is future-scope wave-5 only) — formal wave-5 deferral recorded as DEFER-EQUERY009-001; no code allocated, no taxonomy row needed at this time. error-taxonomy v1.72→v1.73 carries the new E-QUERY-036 row. Future runtime query-error extension allocates E-QUERY-037 onward. ARCH-INDEX v2.129→v2.130. |
| v1.0 | 2026-06-10 | architect | Initial ADR — E-CFG runtime namespace reconciliation design (canonical E-CFG-100..106, renumber map, ClientNotFound variant split, tombstone permanence, MCP mapping realignment, per-owner work-orders). Mirrors ADR-035 pattern per orchestrator routing of the v1.65 drift NOTE. |
| v1.1 | 2026-06-10 | architect | Status flip proposed → ACCEPTED. Human-approved 2026-06-10 (full acceptance, all decisions D1-D6 incl. ClientNotFound variant split D3 and MCP -32602 mapping D4). §Status, frontmatter `decision_made_by`, and §Consequences "Status as of" updated; implementer + PO work-orders unblocked for the Fix PR 2 cascade. |
| v1.2 | 2026-06-10 | architect | D5-precedent addendum (MCP cascade P2-01, same namespace-hygiene decision family — no decision change, status remains ACCEPTED): E-QUERY-007 tombstone-reuse adjudication. `PrismError::QueryLimitExceeded` shipped displaying tombstoned E-QUERY-007 (assigned by Wave-3 pass ADV-W3MT-P58-CRIT-001, unaware of the Phase 1 merge-into-E-QUERY-008 tombstone). Applied D5 tombstone permanence (POL-1) to the E-QUERY namespace: 007 stays dead; limit-exceeded condition reallocated to **E-QUERY-033** (next sequential free at the namespace tail, mirroring this ADR's E-CFG-107+ rule; gaps 016–019 and write-block reserved codes 027/029 explicitly rejected). error-taxonomy v1.69→v1.70 carries the new row + reinforced tombstone; ARCH-INDEX v2.126→v2.127; implementer work-order flips the display and sweeps all E-QUERY-007 literals (develop + fix/review-2026-06-10-mcp-boot). |
| v1.3 | 2026-06-10 | architect | D5-precedent addendum #2 (QRY cascade P5-02) + Blast-Radius exit-criterion carve-out (QRY cascade P5-05) — no decision change, status remains ACCEPTED. **P5-02:** E-QUERY-003 one-code-three-conditions split. `PrismError::QueryExecutionFailed` ("E-QUERY-003: query execution error: {detail}") shipped for (a) generic DataFusion planning/execution errors, (b) virtual-field injection failures, and (c) syntactic security limits — (c) double-prefixing via embedded `E-QUERY-003: ` details (security.rs guard family). Applied the D5/tail-allocation rule a third time this cycle: generic execution (a)+(b) → **E-QUERY-034** (next sequential free at the namespace tail; verified unallocated workspace-wide); E-QUERY-003 retained exclusively by the security limits via NEW dedicated variant `PrismError::QuerySecurityLimitExceeded { detail }` ("E-QUERY-003: {detail}") — un-nesting eliminates the double-prefix; new variant maps MCP `-32602` INVALID_PARAMS surfaced per this ADR's D4 caller-resolvable principle (closes the latent BC-2.11.006 structured-error violation; explicit arm REQUIRED — `#[non_exhaustive]` fall-through would regress to opaque `-32000`). error-taxonomy v1.70→v1.71 carries the rewritten E-QUERY-003 row + new E-QUERY-034 row + full implementer work-order (QRY branch `fix/review-2026-06-10-query-core`: error.rs display flip + new variant; security.rs ×22 / alias_resolver ×2 / explain ×2 emission-site flips with prefix-drop; `E_QUERY_003` const retirement; error_mapping arm + test; VP-014/VP-015 Kani harness variant-match flips, property text unchanged; literal/test sweep incl. stale engine.rs E-QUERY-003→E-QUERY-033 doc comment). Adjacent flagged for cascade, not adjudicated: embedded E-QUERY-006/E-ALIAS-001 details inside QueryExecutionFailed; code-side `QueryVirtualFieldFailed` E-QUERY-010 collision with taxonomy row 010 (zero emitters). **P5-05:** §Blast-Radius TD-VSDD-060 sweep + §Execution Notes item 2 exit criterion gains the `crates/prism-customer-config` carve-out — the zero-matches criterion was unsatisfiable while the crate exists (267 retired-semantics matches across 5 files); the crate is removed wholesale by ADR-037 via Fix PR 3, so the sweep excludes it (`-g '!prism-customer-config/**'`) until that PR lands. ARCH-INDEX v2.127→v2.128. |
| v1.4 | 2026-06-10 | architect | D5-precedent addendum #3 (MCP cascade P4-05, fourth D5-family instance this cycle — no decision change, status remains ACCEPTED): E-QUERY-011 two-BC code collision. `PrismError::AuditTableAccessDenied` shipped displaying `"E-QUERY-011: Audit table requires audit.read capability..."` — live production emitter (prism-query engine.rs `check_table_access`), MCP `-32002` FORBIDDEN mapping, merged story S-2.03, BC-2.15.011 anchor — while the taxonomy's only 011 row defined the BC-2.16.007 "table removed after config reload" condition with ZERO emitters (implementation-orphaned since Phase 1; implementing stories S-1.12/S-3.13 never cite it). Unlike P2-01/P5-02 this is a two-BC collision, not tombstone-reuse or one-code-N-conditions: BOTH conditions are BC-anchored, so the tail-allocation rule applies to the side WITHOUT the shipped display. E-QUERY-011 retained by the live audit-capability condition (ADR-035 verbatim-shipped-display + dominant footprint); the reload condition RE-HOMED (not retired — BC-2.16.007 still pins the behavior as future scope) to **E-QUERY-035**, next sequential free at the namespace tail (verified unallocated workspace-wide), Message Format harmonized to the BC-2.16.007 pinned text since no shipped display binds. error-taxonomy v1.71→v1.72 carries the new 011 (main QUERY table) + 035 rows, legacy-row deletion, and — folded into the same bump — the P4-04 E-FLAG-001 Message Format regularization to the shipped `DeniedRuntime` reason (ADR-035 PO-pattern; code + BC-2.04.015 agree, taxonomy row was the outlier). **NO implementer work-order for either item** (live displays already match the retained rows; the re-homed condition has no emitters — QRY-branch contention moot). PO follow-up routed via orchestrator: BC-2.16.007 011-citations ×3 → E-QUERY-035; test-vectors.md removed-spec vector → E-QUERY-035. ARCH-INDEX v2.128→v2.129. |
