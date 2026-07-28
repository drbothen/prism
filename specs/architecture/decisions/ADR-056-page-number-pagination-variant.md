---
document_type: adr
adr_id: "ADR-056"
title: "PageNumber Pagination Variant — Named-Page (1-Based) Grammar Extension for PaginationConfig"
status: accepted
date: "2026-07-26"
modified: "2026-07-27"
version: "0.5"
producer: architect
subsystems_affected: [SS-06, SS-07, SS-16]
supersedes: null
superseded_by: null
amends: null
anchor_stories:
  - S-WAVE-A-CYBERINT-SPEC-001  # §Authority verified: "ADR-056 v0.4" (PageNumber Pagination Variant, wiring_deferred_to designation)
wiring_deferred_to: S-WAVE-A-CYBERINT-SPEC-001
related_adrs: [ADR-028, ADR-053]
related_bcs: [BC-2.16.002]
locked_decisions: []
inputs:
  - crates/prism-spec-engine/src/spec_parser.rs
  - crates/prism-spec-engine/src/pipeline.rs
  - .factory/reference/api-specs/cyberint_alerts_openapi_06.20.2026.json
input-hash: ""
---

# ADR-056: PageNumber Pagination Variant — Named-Page (1-Based) Grammar Extension for PaginationConfig

## Status

Accepted 2026-07-26, v0.1 (FB53a). Closes F-WASE-P64-CRIT-003.

Amended 2026-07-27, v0.2 (FB65). Closes F-WASE-P65-HIGH-002 and F-WASE-P65-HIGH-003.
Two corrections to v0.1: (a) §D9 scoped its `#[non_exhaustive]` claim to external crates
only, but omitted the four in-crate exhaustive match sites within `prism-spec-engine` that
will produce compile errors when `PageNumber` is added — all four are now enumerated in §D10.
(b) §D3/§D5 described `page_size = 0` as a TOML-authorable value that "simply disables
pagination" with "consistency with `OffsetLimit`" — but `validate_sensor_spec` rejects
`OffsetLimit { page_size: 0 }` with `ESpec001` at spec-load time, so the true `OffsetLimit`
parity is spec-load rejection, not silent acceptance. §D3, §D5, and §D10 (new) correct this
and add the `PaginationType::Page` variant decision and the validation obligation.

This is the architecture leg of the `S-WAVE-A-CYBERINT-SPEC-001` Cyberint Alerts spec
remediation. The `PageNumber` variant is the enabling grammar that allows a TOML sensor
spec to declare `type = "page_number"` pagination, unblocking correct multi-page fetches
against the Cyberint Alerts API (`POST /alert/api/v1/alerts`).

The `PageNumber` variant is not yet implemented in the codebase; production wiring is
deferred to story `S-WAVE-A-CYBERINT-SPEC-001` (`status: draft`). This is a legitimate
three-part deferral per Canonical Principle Rule 3: (1) human-authorized — the human
explicitly approved this grammar extension in-session after reviewing a concrete example
(the jointly-unsatisfiable AC-003 / AC-004 conflict and the OpenAPI evidence); (2) concrete
dependency — the variant itself plus its three `pipeline.rs` dispatch sites
(`build_paged_url_impl` new match arm, `build_request` POST-body injection block, and
`execute_impl` `active_page_size` + advance/terminate arms) must be built by the
implementing story; (3) anchored to the specific real story ID `S-WAVE-A-CYBERINT-SPEC-001`.
`accepted` means the architectural decision is binding and governs all future implementation;
it does NOT mean the variant is currently live in any production binary.

---

## Context

### F-WASE-P64-CRIT-003: Jointly Unsatisfiable Pagination Spec

`S-WAVE-A-CYBERINT-SPEC-001` AC-003 declared `type = "offset_limit"` with `page_size = 100`.
AC-004 required the wire contract `page=N&size=M` (from the Cyberint Alerts OpenAPI). These
two requirements are jointly unsatisfiable: the `OffsetLimit` variant emits `offset` and
`limit`, not `page` and `size`. The Cyberint Alerts API has no `offset` or `limit`
parameter — requests using `OffsetLimit` dispatch would send body keys the API ignores;
the API would return page 1 every time; the termination condition (`page_record_count < page_size`)
would exit after one page. Silent truncation of a security alerts feed; CWE-390 class.

### Canonical API Evidence

The `POST /api/v1/alerts` endpoint in `cyberint_alerts_openapi_06.20.2026.json` uses
`GetAlertsRequest` as its request-body schema. Confirmed properties:

| Field | Type | Constraint | Semantics |
|-------|------|-----------|-----------|
| `page` | integer | `minimum: 1`, `default: 1` | Page number to retrieve (1-based) |
| `size` | integer | `minimum: 10`, `maximum: 100`, `default: 10` | Number of alerts per page |

No `offset` or `limit` field exists on this endpoint. Re-grounding on `offset`/`limit`
is impossible; the API contract forces a grammar extension.

### Why a Grammar Extension, Not an Implementer Decision

A new `PaginationConfig` variant establishes a spec surface (`type = "page_number"` in TOML),
an advance-rule specification, and two dispatch sites in production code (`build_paged_url_impl`
and `build_request`). Leaving these choices to implementer discretion violates Canonical
Principle Rule 1: the implementer would need to invent the variant name, the advance rule,
and the dispatch semantics — all architectural decisions affecting the public TOML schema.

---

## Decision

### D1 — Variant: `PageNumber { page_size: u32 }`

The following variant is ratified for addition to `PaginationConfig` in `spec_parser.rs`:

```
PageNumber { page_size: u32 }
```

**TOML declaration:**

```toml
[tables.steps.pagination]
type = "page_number"
page_size = 100
```

**Serde encoding:** `PaginationConfig` carries `#[serde(tag = "type", rename_all = "snake_case")]`.
The new variant's serde tag is `page_number`, derived automatically from `PageNumber` by
`rename_all = "snake_case"`. No explicit `#[serde(rename = …)]` is needed.

**Naming justification:** `PageNumber` names the pagination mechanism — a 1-based integer
page number used to request a specific page of results. The wire parameter names (`page`,
`size`) are implementation-level constants specific to each API; the variant name is generic
and available to any sensor using 1-based page-number + page-size pagination. `page_size: u32`
follows the existing convention in `OffsetLimit { page_size: u32 }`.

### D2 — 1-Based Counter Encoding (Normative)

The `PageNumber` variant reuses the existing `offset: u32` loop state variable in
`execute_impl` — initialized to `0` before the pagination loop — as a **0-based page index**.
The wire parameter `page` is computed as `offset + 1`.

**Normative first-request value:** When `offset = 0` (first page), the emitted wire value
is `page = 0 + 1 = 1`. This satisfies the Cyberint Alerts API `minimum: 1` and matches
`default: 1`. No API with a `minimum: 1` page constraint can be fed a `page = 0` value
from this variant.

**Advance rule:** `offset += 1`. Each completed page increments the 0-based index by 1.
After the first page, `offset = 1`; the second request emits `page = 2`. This is distinct
from `OffsetLimit`'s `offset += page_size` and MUST NOT be confused with it.

**Overflow guard:** `offset += 1` operates on `u32`. The existing `MAX_PAGES_PER_STEP`
constant in `pipeline.rs` (1,000 pages) causes the loop to abort well before `u32` overflow.
No additional guard is required.

**Rationale for reusing `offset` rather than a distinct counter:** Two options were considered:
(A) reuse `offset` as a 0-based page index — emits `offset + 1`; no signature changes to
`build_paged_url_impl` or `build_request`; (B) introduce a distinct 1-based counter — requires
new state in the loop and new function parameters at all dispatch sites. Option A is chosen
because it has a strictly smaller implementation surface. The `offset` variable already carries
different semantics for different variants (`CursorToken` does not use it at all;
`OffsetLimit` treats it as a record offset). Reusing it as a page index is consistent
with that established pattern, provided the per-variant semantics are documented.

### D3 — Dispatch Shape: POST vs non-POST

`PageNumber` follows the same POST/non-POST split established by `OffsetLimit`
(DRIFT-D850-001 / BC-2.16.002 §Postconditions). The implementing engineer must apply the
following changes to the named symbols.

#### `build_paged_url_impl` — new match arm

```
Some(PaginationConfig::PageNumber { page_size }) => {
    // POST: pagination params go in the request body; URL is returned unchanged.
    // non-POST (GET or other): append ?page=N&size=M as URL query parameters.
    if step.method.eq_ignore_ascii_case("POST") {
        base_url.to_string()
    } else {
        let page = offset + 1;
        let sep = if base_url.contains('?') { '&' } else { '?' };
        format!("{base_url}{sep}page={page}&size={page_size}")
    }
}
```

#### `build_request` — POST body injection block

The existing `OffsetLimit` POST-body injection block is followed by a parallel `PageNumber`
block. The guard condition is:

```
step.method.eq_ignore_ascii_case("POST")
    && matches!(step.pagination, Some(PaginationConfig::PageNumber { .. }))
    && page_size > 0
```

When the guard fires, inject `"page": (offset + 1)` and `"size": page_size` as top-level
integer keys into the JSON body, merged onto the interpolated `body_template`. The merge
semantics and error paths are identical to the `OffsetLimit` block:

- Absent or empty `body_template` produces `{"page": 1, "size": page_size}` on the first request.
- If `body_template` interpolates to a non-object JSON value, return
  `Err(SpecEngineError::HttpRequestFailed { sensor_id, step_name, … })` (the EC-002 path
  from BC-2.16.002 §Postconditions).

#### `active_page_size` derivation — extension in `execute_impl`

The `active_page_size: u32` derivation block is extended with an `|`-pattern arm:

```
Some(PaginationConfig::OffsetLimit { page_size: ps })
| Some(PaginationConfig::PageNumber { page_size: ps }) => *ps,
```

This ensures `build_request` receives the correct `page_size` value for body injection.

#### Activation gate

`PageNumber` pagination, like `OffsetLimit`, enforces `page_size > 0` at two layers:

**Spec-load layer:** `validate_sensor_spec` §Category 4 rejects `PageNumber { page_size: 0 }`
with `SpecErrorCode::ESpec001` at spec-load time — the same rejection already applied to
`OffsetLimit { page_size: 0 }`. A TOML author cannot declare `page_size = 0` for either
variant. This is the correct `OffsetLimit` parity; §D5 v0.1's claim that "a `page_size = 0`
spec simply disables pagination" was incorrect (see §D5 and §D10 corrections in this version).

**Runtime sentinel:** `execute_step` (the single-request code path, distinct from the
pagination loop in `execute_impl`) passes `active_page_size = 0` as an integer argument to
`build_request` to signal "no pagination injection active." This `0` value is engine-internal
only — it is never deserialized from a TOML spec and can never reach `execute_impl`'s
`PageNumber` advance/terminate arm. The `page_size > 0` guard in `build_request` fires on
this sentinel, correctly skipping injection on single-request steps.

### D4 — Termination Condition

```
Some(PaginationConfig::PageNumber { page_size }) => {
    let ps = *page_size as usize;
    if page_record_count < ps {
        break;
    }
    offset += 1;
}
```

Termination is identical to `OffsetLimit`: when the page returned fewer records than
`page_size`, the last page has been reached and the loop exits. The advance (`offset += 1`)
is placed after the termination check, consistent with the `OffsetLimit` advance placement.

`ps` cannot be `0` here because `validate_sensor_spec` rejects `PageNumber { page_size: 0 }`
at spec-load time (§D3 spec-load layer; §D10 validation obligation). The `page_record_count <
ps` check where `ps = 0` would be trivially false for any non-empty page, running the loop
to `MAX_PAGES_PER_STEP` (1,000 requests) — that runaway path is unreachable from a validated
spec.

### D5 — POL-36 Compliance: No Sensor-Specific Bounds

`PageNumber { page_size: u32 }` encodes no sensor-specific constraints.

The Cyberint Alerts API's `size` field constraints (`minimum: 10`, `maximum: 100`) are NOT
baked into the variant. TOML authors bear responsibility for specifying a `page_size` that
satisfies the sensor API's accepted range. Encoding Cyberint-specific bounds would create
sensor-name-conditional engine behavior, which POL-36
(`generalization_directive_no_sensor_conditional_engine_code`, HIGH) explicitly forbids.

There is no generic cross-sensor page-size bound that would be both correct and safe to
enforce in the engine beyond `page_size > 0`. The `page_size > 0` lower bound is enforced
at spec-load time by `validate_sensor_spec` §Category 4 (same as `OffsetLimit`): a TOML
author declaring `page_size = 0` receives `SpecErrorCode::ESpec001`. This is the correct
`OffsetLimit` parity — §D5 v0.1 incorrectly stated "a `page_size = 0` spec simply disables
pagination rather than producing an error"; the actual `OffsetLimit` precedent is
spec-load rejection, not silent acceptance. See §D3 (amended) and §D10 for the validation
obligation. Enforcing `page_size > 0` at spec-load time imposes no sensor-specific logic.

### D6 — ADR Vehicle: New ADR-056 (Not an Amendment to ADR-028)

A new ADR is the correct vehicle for four reasons:

1. **Distinct subject.** ADR-028's core subject is "what is the canonical grounding
   reference for TOML spec URL paths and auth_type values." Pagination grammar is orthogonal
   to that question. Adding a pagination grammar decision to ADR-028 would conflate two
   unrelated concerns.

2. **Supersession entanglement.** ADR-028's §D1/§D2/§D5 are superseded by ADR-053.
   An amendment to ADR-028 would be authored against a document with substantial superseded
   sections, increasing the risk of unintentional conflict. New ADR-056 carries no such
   inheritance burden.

3. **Traceability.** A new ADR provides a single locus for `S-WAVE-A-CYBERINT-SPEC-001`
   and future stories that need to reference the `PageNumber` grammar decision.

4. **Precedent check.** ADR-028 §D8 (timestamp grammar extension) was added to ADR-028
   mid-stream because it arose during the same PLUGIN-MIGRATION-001-D cascade that ADR-028
   was recording. The current pagination extension arises in a different cascade
   (Wave-A sensor fidelity remediation); a separate ADR is the correct structure.

`related_adrs: [ADR-028, ADR-053]` captures the relationship. No `amends:` or
`supersedes:` fields are needed; this ADR neither supersedes nor amends either document.

**Bidirectional back-ref:** ADR-028 and ADR-053 do not require amendment to reference
ADR-056 because this ADR extends the grammar without altering any of their decisions.
Future amendments to those ADRs may add ADR-056 to their `related_adrs` if appropriate;
no such addition is required now.

### D7 — CursorToken / CursorPagination Mismatch Investigation

**Finding: Inert today; latent defect for any future Cyberint credentials table.**

The `cyberint_alerts_openapi_06.20.2026.json` defines `CursorPagination` (properties:
`cursor` optional string, `limit` integer min 1 max 100 default 50). This schema is
referenced by `GetCredentialsRequest`, which is used by `POST /api/v1/credentials`.
It is NOT referenced by `POST /api/v1/alerts` or any other endpoint in the file.

Prism's `CursorToken` variant emits pagination state as URL query parameters:
`cursor={encoded_cursor}` on continuation requests, and optionally `page_size={n}` when
`page_size: Some(n)` is declared.

Cyberint's `CursorPagination` expects pagination state as a nested POST body object:
`{"pagination": {"cursor": "...", "limit": N}}`.

The mismatch has two axes:

1. **Transport form:** `CursorToken` uses URL query parameters; `CursorPagination` uses
   a nested POST body object under a `"pagination"` key.
2. **Field naming:** Prism emits `page_size`; Cyberint expects `limit`. The nesting
   structure (`"pagination": {...}`) is also not supported by `CursorToken`.

**Current exposure:** Zero. No prism sensor TOML targets `POST /api/v1/credentials` or
any other Cyberint endpoint that uses `CursorPagination`. The current `cyberint.sensor.toml`
uses `cursor_token` for its alerts table — that spec is superseded and deleted by
`S-WAVE-A-CYBERINT-SPEC-001`, which replaces it with `cyberint-alerts.sensor.toml` using
`type = "page_number"` per this ADR.

**Future routing note for orchestrator:** If a `cyberint-credentials` table is added in a
future story, using `type = "cursor_token"` would silently malfunction: cursor and limit
would be emitted as URL query parameters against an endpoint expecting a POST body. The
nested `"pagination"` wrapper is also not representable in any current `PaginationConfig`
variant. That story's spec author must either extend `CursorToken` with a
POST-body-nested mode or introduce a new variant for nested-object cursor pagination.
This ADR does not address that case; the analysis is recorded here to prevent a future
adversary pass from re-litigating it as a novel finding.

### D8 — Required BC-2.16.002 §Postconditions Row for Product-Owner

Product-owner must add the following row to BC-2.16.002 §Postconditions as a sibling to
the existing "OffsetLimit Pagination Dispatch: POST-body vs GET-URL (DRIFT-D850-001)" row.
The content is specified with sufficient precision to author it without re-deriving the design.

**Row heading:**

```
PageNumber Pagination Dispatch: POST-body vs GET-URL (ADR-056, S-WAVE-A-CYBERINT-SPEC-001)
```

**Row body (verbatim content for product-owner):**

When a step's `PaginationConfig` is `PageNumber`, the transport form of `page` and `size`
parameters is determined by the step's HTTP method:

- **POST:** `build_paged_url_impl` returns the base URL unchanged. `build_request` injects
  `"page"` and `"size"` as top-level integer keys into the POST request body, merged onto
  the interpolated `body_template`. The injected `page` value is the current 0-based loop
  index (`offset`) plus one (`offset + 1`); the first request injects `"page": 1`. The
  injected `size` value equals `page_size`. Merge semantics and error paths are identical
  to `OffsetLimit` POST dispatch: absent or empty `body_template` (`{}`) produces
  `{"page": 1, "size": page_size}` on the first request; a `body_template` that
  interpolates to a non-object JSON value returns `Err(SpecEngineError)` with `sensor_id`
  and `step_name` context (EC-002 path).

- **GET (or method absent — defaults to GET):** `build_paged_url_impl` appends
  `?page={offset+1}&size={page_size}` to the URL as query parameters. The `?` vs `&`
  separator is determined by the existing `contains('?')` check, consistent with all
  other non-POST arms of `build_paged_url_impl`.

- **Page counter:** The loop reuses the `offset: u32` state variable as a 0-based page
  index. The wire `page` value is always `offset + 1`. The first request emits `page = 1`.

- **Advance rule:** `offset += 1`. Each page increments the page number by exactly 1.
  The advance is method-agnostic (applies regardless of POST vs GET).

- **Termination:** `if page_record_count < page_size { break }` — identical to `OffsetLimit`.

- **Activation gate:** `build_request` injects `page` and `size` only when `page_size > 0`.
  This guard serves the **engine-internal sentinel case**: `execute_step` (the single-request
  path, distinct from the pagination loop) passes `active_page_size = 0` to `build_request`
  to signal that no pagination injection is active. A TOML spec declaring `page_size = 0` is
  **rejected at spec-load time** by `validate_sensor_spec` §Category 4 with
  `SpecErrorCode::ESpec001` (§D3 spec-load layer; §D10 CE-2 validation obligation) — the
  value `0` cannot reach `build_request` from a validated spec. The `page_size > 0` guard in
  `build_request` is defense-in-depth for the engine-internal sentinel path only.

- **First page, empty `body_template`:** POST body becomes `{"page": 1, "size": page_size}`.

Grounding: ADR-056 §D3 (dispatch shape); `cyberint_alerts_openapi_06.20.2026.json`
`GetAlertsRequest` schema (`page` field minimum 1; `size` field).

**Required BC-2.16.009 §Validation Rule 4 `page_number` row (product-owner must author this in the PO leg):**

Add the following row to BC-2.16.009 §Validation Rule 4 as a sibling to the existing
`offset_limit` row. The error message template uses the step name from the TOML spec step
being validated, consistent with the `offset_limit` parallel.

| Pagination type | Rejection condition | Error code | Error message |
|-----------------|---------------------|------------|---------------|
| `page_number` | `page_size == 0` in `PaginationConfig::PageNumber` | `SpecErrorCode::ESpec001` | `"page_number pagination in step '{step_name}' requires page_size > 0"` |

Grounding: §D10 CE-2 (`validate_sensor_spec` `PageNumber` arm obligation); §D3 spec-load layer.
This rejection mirrors the existing `OffsetLimit { page_size: 0 }` row, which also uses
`SpecErrorCode::ESpec001` per the `OffsetLimit` spec-load-layer precedent.

### D9 — `#[non_exhaustive]` Gate Count: Unchanged at 92

`PaginationConfig` already carries `#[non_exhaustive]`. The `scripts/check-non-exhaustive.sh`
gate (expected symbol count 92) and `scripts/check-non-exhaustive-per-symbol.py`
(`EXPECTED_COUNT` + `EXPECTED_SYMBOLS` list) count the number of `#[non_exhaustive]`
symbol occurrences in the workspace, not the number of variants within any single enum.
Adding `PageNumber` to `PaginationConfig` does not add a new `#[non_exhaustive]` attribute;
`PaginationConfig` already appears in both scripts' symbol registries.

Adding `PaginationType::Page` (§D10) to `PaginationType` also does not add a new
`#[non_exhaustive]` attribute; `PaginationType` already carries `#[non_exhaustive]` and
already appears in the scripts' symbol registries.

**The implementing story MUST NOT bump `EXPECTED=92` in `scripts/check-non-exhaustive.sh`
or `EXPECTED_COUNT` in `scripts/check-non-exhaustive-per-symbol.py` for either of these
changes.**

**External crates only:** Match arms on `PaginationConfig` in crates OTHER THAN
`prism-spec-engine` already require a wildcard `_ => {}` arm due to `#[non_exhaustive]`.
No external callsite migration is needed for those crates. However, `#[non_exhaustive]` has
NO effect inside `prism-spec-engine` (the defining crate). There are four in-crate exhaustive
match sites that will produce compiler errors when `PageNumber` is added — see §D10 for the
complete enumeration and migration obligations.

### D10 — `PaginationType::Page` Variant; In-Crate Exhaustive Match Migration (v0.2 addition)

#### `PaginationType::Page` decision

`PaginationType` (in `prism_spec_engine::types`) is the wire-visible enum carried on
`SensorTableDescriptor.pagination_type`, consumed by LLM agents via the MCP
`list_sensor_specs` tool. Its current variants are `Cursor`, `Offset`, `None`.

`PageNumber` must NOT fold into `PaginationType::Offset`. A 1-based page-number counter
(`PageNumber`) and a byte-range record offset (`OffsetLimit`) are semantically distinct
pagination mechanisms. Labeling both as `Offset` would produce false information on the
MCP wire surface for any LLM agent reasoning about how a sensor is paginated.

**Decision:** add `PaginationType::Page` to `PaginationType` in `prism_spec_engine::types`.
`sensor_table_descriptor_from_table_spec` maps `PaginationConfig::PageNumber` to
`PaginationType::Page`.

`PaginationType` already carries `#[non_exhaustive]`. External match arms on
`PaginationType` already require a wildcard `_ => {}` arm; adding `Page` requires no
external callsite migration. The `#[non_exhaustive]` EXPECTED gate count remains at 92
(no new `#[non_exhaustive]` attribute is added; see §D9).

#### In-crate exhaustive match sites — compile-error obligations

`#[non_exhaustive]` protects crates EXTERNAL to `prism-spec-engine` only. All four
in-crate sites below will produce compiler errors when `PaginationConfig::PageNumber`
is added; the implementing story MUST migrate them:

**Table 1 — Compile-error sites (all in `prism-spec-engine`):**

| Site | Function symbol | Module path | Required change |
|------|----------------|-------------|-----------------|
| CE-1 | `sensor_table_descriptor_from_table_spec` | `prism_spec_engine::types` | Add `PaginationConfig::PageNumber { .. } => PaginationType::Page` arm |
| CE-2 | `validate_sensor_spec` §Category 4 pagination block | `prism_spec_engine::validation` | Add `PaginationConfig::PageNumber { page_size }` arm rejecting `page_size == 0` with `SpecErrorCode::ESpec001` and message `"page_number pagination in step '{}' requires page_size > 0"` |
| CE-3 | `build_paged_url_impl` | `prism_spec_engine::pipeline` | Add `Some(PaginationConfig::PageNumber { page_size })` arm per §D3 dispatch skeleton |
| CE-4 | `execute_impl` advance/terminate block | `prism_spec_engine::pipeline` | Add `Some(PaginationConfig::PageNumber { page_size })` arm per §D4 termination rule |

**Table 2 — Behavioral-migration sites (no compile error; wildcard/matches! patterns):**

| Site | Function symbol | Module path | Required change |
|------|----------------|-------------|-----------------|
| BM-1 | `execute_impl` `active_page_size` derivation | `prism_spec_engine::pipeline` | Extend `Some(PaginationConfig::OffsetLimit { page_size: ps }) => *ps` with `\| Some(PaginationConfig::PageNumber { page_size: ps }) => *ps` per §D3 |
| BM-2 | `build_request` POST-body injection | `prism_spec_engine::pipeline` | Add parallel `PageNumber` injection block per §D3 `build_request` specification |

The adversary pass (F-WASE-P65-HIGH-002) identified CE-1 and CE-2. CE-3 and CE-4 (both in
`prism_spec_engine::pipeline`) were additional in-crate compile-error sites not enumerated
in ADR-056 v0.1; they are enumerated here as binding migration obligations. BM-1 and BM-2
were already implicit in the §Consequences items in v0.1 but are now named explicitly.

---

## Rationale

### Why POST body injection (not URL parameters) for the POST case?

The Cyberint Alerts API `POST /alert/api/v1/alerts` uses a JSON request body for all
parameters. Injecting `page`/`size` as URL query parameters would produce a correctly
formed URL, but the API would not read them — the wire contract is the POST body.
The POST/GET split mirrors the `OffsetLimit` dispatch established by DRIFT-D850-001 and
is the correct generalization for POST-for-read APIs. Both Claroty xDome (DRIFT-D850-001)
and Cyberint Alerts use POST for bulk read operations with pagination in the body;
`PageNumber` extends this pattern to 1-based page-number semantics.

### Why `page_size: u32` with no built-in validation bounds?

The `OffsetLimit` variant uses `page_size: u32` with no built-in bounds beyond `> 0` for
the activation gate. Consistency with that convention avoids special-casing `PageNumber`.
Sensor-specific bounds (`minimum: 10`, `maximum: 100` for Cyberint) belong in TOML
documentation and spec review, not in the engine's enum definition. Encoding them would
violate POL-36 (D5).

### Why keep the `offset + 1` formula instead of initializing `offset = 1`?

The loop state variable `let mut offset: u32 = 0` is unconditional — it precedes the
variant-dispatched code. Changing the initialization value to `1` for `PageNumber` would
require a per-variant initialization branch, complicating the loop setup. The `offset + 1`
formula in the dispatch sites is explicit and clearly documented; it is preferable to
hidden per-variant initialization logic.

---

## Consequences

### For the implementing story (`S-WAVE-A-CYBERINT-SPEC-001`)

1. Add `PageNumber { page_size: u32 }` variant to `PaginationConfig` in `spec_parser.rs`
   with the doc comment pattern established in D1. The `#[non_exhaustive]` attribute on the
   enum already covers this variant; do not add a second attribute.
2. Add `PaginationType::Page` variant to `PaginationType` in `types.rs` (§D10). No new
   `#[non_exhaustive]` attribute needed; `PaginationType` already carries it.
3. Extend `sensor_table_descriptor_from_table_spec` in `types.rs` with arm
   `PaginationConfig::PageNumber { .. } => PaginationType::Page` (§D10 CE-1). This is
   a compile-error site — failure to add this arm prevents the crate from compiling.
4. Extend `validate_sensor_spec` §Category 4 pagination block in `validation.rs` with a
   `PaginationConfig::PageNumber { page_size }` arm that rejects `page_size == 0` with
   `SpecErrorCode::ESpec001` (§D3 spec-load layer; §D10 CE-2). Error message:
   `"page_number pagination in step '{}' requires page_size > 0"`. This is a compile-error
   site AND a behavioral requirement: without it, `page_size = 0` in TOML reaches the
   runaway §D4 termination arm (`ps = 0` → loop to `MAX_PAGES_PER_STEP`).
5. Extend `build_paged_url_impl` in `pipeline.rs` with the match arm specified in §D3
   (§D10 CE-3). This is a compile-error site.
6. Extend `build_request` in `pipeline.rs` with the POST-body injection block for
   `PageNumber` (§D3; §D10 BM-2), following the same structure as the `OffsetLimit` block.
7. Extend the `active_page_size` derivation in `execute_impl` (§D3; §D10 BM-1).
8. Extend the pagination advance/terminate block in `execute_impl` (§D4; §D10 CE-4). This
   is a compile-error site.
9. Declare `type = "page_number"` and `page_size = 100` in the Cyberint Alerts step
   `[tables.steps.pagination]` block in `cyberint-alerts.sensor.toml`.
10. Do NOT bump `EXPECTED=92` or `EXPECTED_COUNT` in the non-exhaustive gate scripts (§D9).
11. Tests must cover the following per SAP-3 (end-to-end from `PipelineExecutor::execute`):
    - POST path: first request body contains `"page": 1` and `"size": 100`; second page
      contains `"page": 2` and `"size": 100`.
    - GET path: first request URL contains `?page=1&size=100`; second page URL contains
      `?page=2&size=100`.
    - Termination: a page returning fewer records than `page_size` ends the loop after
      that page with no additional request.
    - Non-object `body_template` with `PageNumber` POST returns `Err(SpecEngineError)`.
    - `page_size = 0` in TOML produces `SpecErrorCode::ESpec001` at spec-load time (not
      silent unpaginated behavior and not a 1,000-page runaway).
    - `SensorTableDescriptor.pagination_type` for a `PageNumber` table serializes to MCP wire
      value `"Page"` (PascalCase — NOT `"page"`). `PaginationType` carries no `rename_all`
      serde attribute; variant identifiers serialize verbatim, consistent with the existing MCP
      wire vocabulary `"Cursor"` / `"Offset"` / `"None"` (all PascalCase). Test assertion MUST
      use `"Page"`. Adding `rename_all = "snake_case"` to `PaginationType` to produce `"page"`
      would be a breaking change, renaming all three existing wire values to lowercase.
      `S-WAVE-A-CYBERINT-SPEC-001` AC-019 / RG-018 must assert `"Page"`, not `"page"`.

### For the product-owner

Author two BC artifacts using the specifications in §D8:

1. The BC-2.16.002 §Postconditions row specified in §D8 (Row heading + Row body, including the
   corrected activation-gate description).
2. The BC-2.16.009 §Validation Rule 4 `page_number` row specified in the §D8 table above
   (sibling to the existing `offset_limit` row).

Both artifacts are being authored in the concurrent product-owner leg of this same burst
(FB71); no BC content is authored in this ADR. Both MUST be present before
`S-WAVE-A-CYBERINT-SPEC-001` is declared implementation-ready:

- BC-2.16.002 §Postconditions PageNumber Pagination Dispatch row: postcondition authority for
  AC-003 and AC-004 in `S-WAVE-A-CYBERINT-SPEC-001`.
- BC-2.16.009 §Validation Rule 4 `page_number` row: authority for the §D10 CE-2 validation
  obligation exercised by RG-017
  (`test_pagination_page_number_page_size_zero_rejected_at_spec_load`) in
  `S-WAVE-A-CYBERINT-SPEC-001`.

(POL-29 dimension 9c — anchored to `S-WAVE-A-CYBERINT-SPEC-001` AC-003, AC-004, and
RG-017. Closes F-CVA-MED-001.)

### For future sensor specs using named-page pagination

Any sensor that uses 1-based page-number + page-size pagination may now declare
`type = "page_number"` in its step `[tables.steps.pagination]` block. The TOML author
must verify that `page_size` is within the sensor API's accepted range.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 0.5 | FB74 | 2026-07-27 | architect | SAC-2 per-entry verification annotation added to `anchor_stories`. Closes F-CVC-MED-001: S-WAVE-A-CYBERINT-SPEC-001 §Authority now cites "ADR-056 v0.4" (PageNumber Pagination Variant, wiring_deferred_to designation). Single-line `anchor_stories` expanded to multi-line format with per-entry annotation. |
| 0.4 | FB71 | 2026-07-27 | architect | Closes F-CVA-MED-001 (POL-29 dimension 9c). §Consequences "For the product-owner" — unanchored mandate replaced with story-anchored obligation: both BC artifacts are being authored in the concurrent PO leg of FB71 and MUST be present before `S-WAVE-A-CYBERINT-SPEC-001` is declared implementation-ready; anchored to `S-WAVE-A-CYBERINT-SPEC-001` AC-003, AC-004, and RG-017 (§D10 CE-2 coverage). No §D8 content changed. |
| 0.3 | FB70 | 2026-07-27 | architect | Closes F-WASE-P66-HIGH-001, F-WASE-P66-MED-001, F-WASE-P66-LOW-001. HIGH-001: §D10 Consequences item 11 corrected — `PaginationType::Page` wire literal is `"Page"` (PascalCase, no `rename_all` on `PaginationType`), not `"page"`; breaking-change analysis added; `S-WAVE-A-CYBERINT-SPEC-001` RG-018 must assert `"Page"`. MED-001: §D8 activation gate bullet corrected — removes false `page_size = 0` skips-injection claim; replaces with accurate two-layer description: spec-load `ESpec001` rejection (§D3/§D10 CE-2) and engine-internal `active_page_size = 0` sentinel; BC-2.16.009 §Validation Rule 4 `page_number` row specification added to §D8 (sibling to `offset_limit` row); §Consequences updated to instruct PO to author both BC-2.16.002 §Postconditions row AND BC-2.16.009 §Validation Rule 4 row. LOW-001: `## §Changelog` heading corrected to `## Changelog` to match ADR-050 through ADR-055 corpus convention. |
| 0.2 | FB65 | 2026-07-27 | architect | Closes F-WASE-P65-HIGH-002 and F-WASE-P65-HIGH-003. §D3 activation gate amended: two-layer enforcement clarified (spec-load `ESpec001` rejection + engine-internal `active_page_size = 0` sentinel); false "silent-disabling" language removed. §D4 soundness note added: `ps = 0` runaway unreachable from validated spec. §D5 corrected: "consistency with `OffsetLimit`" correctly means spec-load rejection, not silent acceptance; removes false claim that `page_size = 0` spec disables pagination. §D9 scoped to external crates only; forward-references §D10 for in-crate sites. §D10 added (new): `PaginationType::Page` variant decision (agent-observable MCP semantic — must not fold into `Offset`); complete enumeration of four compile-error sites (CE-1 through CE-4) and two behavioral-migration sites (BM-1, BM-2) within `prism-spec-engine`; adversary found CE-1 and CE-2; CE-3 and CE-4 (both in `prism_spec_engine::pipeline`) were additional. §Consequences items renumbered 1–11: new items for `PaginationType::Page`, `sensor_table_descriptor_from_table_spec`, `validate_sensor_spec` spec-load rejection; test list extended with `ESpec001` and `PaginationType::Page` wire assertions. |
| 0.1 | FB53a | 2026-07-26 | architect | Initial authoring. Ratifies `PageNumber { page_size: u32 }` grammar extension to `PaginationConfig`. Closes F-WASE-P64-CRIT-003. `wiring_deferred_to: S-WAVE-A-CYBERINT-SPEC-001` added per POL-15 false-positive escape (variant not yet in codebase; three-part Canonical Principle Rule 3 deferral recorded in §Status). |
