---
document_type: adr
adr_id: "ADR-056"
title: "PageNumber Pagination Variant — Named-Page (1-Based) Grammar Extension for PaginationConfig"
status: accepted
date: "2026-07-26"
modified: "2026-07-26"
version: "0.1"
producer: architect
subsystems_affected: [SS-06, SS-07, SS-16]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-WAVE-A-CYBERINT-SPEC-001]
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

Injection (both POST-body and GET-URL) occurs only when `page_size > 0`. When
`page_size = 0`, the gate does not fire. This guards the `execute_step` single-request
path, which passes `active_page_size = 0` to signal "no pagination active." The gate
semantics are identical to `OffsetLimit`.

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

### D5 — POL-36 Compliance: No Sensor-Specific Bounds

`PageNumber { page_size: u32 }` encodes no sensor-specific constraints.

The Cyberint Alerts API's `size` field constraints (`minimum: 10`, `maximum: 100`) are NOT
baked into the variant. TOML authors bear responsibility for specifying a `page_size` that
satisfies the sensor API's accepted range. Encoding Cyberint-specific bounds would create
sensor-name-conditional engine behavior, which POL-36
(`generalization_directive_no_sensor_conditional_engine_code`, HIGH) explicitly forbids.

There is no generic cross-sensor page-size bound that would be both correct and safe to
enforce in the engine. A lower bound of `> 0` is enforced by the activation gate (D3), not
as a validation rule — a `page_size = 0` spec simply disables pagination rather than
producing an error. This behavior is intentional (consistency with `OffsetLimit`) and
imposes no sensor-specific logic.

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

- **Activation gate:** Injection occurs only when `page_size > 0`. `page_size = 0` skips
  injection (single-request / non-paginated `execute_step` path).

- **First page, empty `body_template`:** POST body becomes `{"page": 1, "size": page_size}`.

Grounding: ADR-056 §D3 (dispatch shape); `cyberint_alerts_openapi_06.20.2026.json`
`GetAlertsRequest` schema (`page` field minimum 1; `size` field).

### D9 — `#[non_exhaustive]` Gate Count: Unchanged at 92

`PaginationConfig` already carries `#[non_exhaustive]`. The `scripts/check-non-exhaustive.sh`
gate (expected symbol count 92) and `scripts/check-non-exhaustive-per-symbol.py`
(`EXPECTED_COUNT` + `EXPECTED_SYMBOLS` list) count the number of `#[non_exhaustive]`
symbol occurrences in the workspace, not the number of variants within any single enum.
Adding `PageNumber` to `PaginationConfig` does not add a new `#[non_exhaustive]` attribute;
`PaginationConfig` already appears in both scripts' symbol registries.

**The implementing story MUST NOT bump `EXPECTED=92` in `scripts/check-non-exhaustive.sh`
or `EXPECTED_COUNT` in `scripts/check-non-exhaustive-per-symbol.py` for this change.**

External match arms on `PaginationConfig` already require a wildcard `_ => {}` arm due to
the pre-existing `#[non_exhaustive]` annotation. No external callsite migration is needed
solely because this variant is added.

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
2. Extend `build_paged_url_impl` in `pipeline.rs` with the match arm specified in D3.
3. Extend `build_request` in `pipeline.rs` with the POST-body injection block for
   `PageNumber` (D3), following the same structure as the `OffsetLimit` injection block.
4. Extend the `active_page_size` derivation in `execute_impl` (D3).
5. Extend the pagination advance/terminate block in `execute_impl` (D4).
6. Declare `type = "page_number"` and `page_size = 100` in the Cyberint Alerts step
   `[tables.steps.pagination]` block in `cyberint-alerts.sensor.toml`.
7. Do NOT bump `EXPECTED=92` or `EXPECTED_COUNT` in the non-exhaustive gate scripts (D9).
8. Tests must cover the following per SAP-3 (end-to-end from `PipelineExecutor::execute`):
   - POST path: first request body contains `"page": 1` and `"size": 100`; second page
     contains `"page": 2` and `"size": 100`.
   - GET path: first request URL contains `?page=1&size=100`; second page URL contains
     `?page=2&size=100`.
   - Termination: a page returning fewer records than `page_size` ends the loop after
     that page with no additional request.
   - Non-object `body_template` with `PageNumber` POST returns `Err(SpecEngineError)`.
   - `page_size = 0` produces no `page`/`size` injection (activation gate).

### For the product-owner

Author the BC-2.16.002 §Postconditions row specified in D8. This is a separate PO leg
dispatched after this ADR. No BC content is authored here.

### For future sensor specs using named-page pagination

Any sensor that uses 1-based page-number + page-size pagination may now declare
`type = "page_number"` in its step `[tables.steps.pagination]` block. The TOML author
must verify that `page_size` is within the sensor API's accepted range.

---

## §Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 0.1 | FB53a | 2026-07-26 | architect | Initial authoring. Ratifies `PageNumber { page_size: u32 }` grammar extension to `PaginationConfig`. Closes F-WASE-P64-CRIT-003. `wiring_deferred_to: S-WAVE-A-CYBERINT-SPEC-001` added per POL-15 false-positive escape (variant not yet in codebase; three-part Canonical Principle Rule 3 deferral recorded in §Status). |
