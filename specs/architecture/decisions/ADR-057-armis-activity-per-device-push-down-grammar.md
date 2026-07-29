---
document_type: adr
adr_id: "ADR-057"
title: "Armis Per-Device Activity Surface — Push-Down Grammar for Parameterized Path Templates"
status: accepted
date: "2026-07-27"
modified: "2026-07-29"
version: "0.7"
producer: architect
subsystems_affected: [SS-06, SS-07, SS-12]
supersedes: null
superseded_by: null
amends: null
anchor_stories:
  - S-WAVE-A-ARMIS-ACTIVITY-001  # §Authority verified: "ADR-057 (accepted 2026-07-27)" — governs ${query.filter.*} push-down grammar for parameterized path templates
related_adrs: [ADR-028, ADR-033, ADR-053, ADR-056]
related_bcs: [BC-2.02.006, BC-2.02.014, BC-2.16.002]
inputs:
  - crates/prism-spec-engine/src/interpolation.rs
  - crates/prism-spec-engine/src/pipeline.rs
  - crates/prism-sensors/specs/crowdstrike.sensor.toml
  - crates/prism-sensors/specs/armis.sensor.toml
  - .factory/stories/S-WAVE-A-ARMIS-ACTIVITY-001-armis-device-activity-toml-surface.md
input-hash: ""
---

# ADR-057: Armis Per-Device Activity Surface — Push-Down Grammar for Parameterized Path Templates

## Status

Accepted 2026-07-27. Resolves the blocking dependency in `S-WAVE-A-ARMIS-ACTIVITY-001`
(architect-confirmation-required gate). Story is UNBLOCKED.

---

## Context

`S-WAVE-A-ARMIS-ACTIVITY-001` needs a `path_template` for the Armis device activity
endpoint, registered in the DTU at `GET /api/v1/devices/:device_id/activity`. The story
was authored with a placeholder `${variable.device_id}` syntax — a PHANTOM namespace
confirmed by a workspace-wide grep returning zero occurrences in `crates/`.

The product-owner correctly escalated three candidate interpretations of what
`${step_name.field}` does when a prior step returns N records:

- **(A)** Per-scalar fan-out — N requests, one per record's field value
- **(B)** Per-batch — one request, array-as-collection
- **(C)** Capability gap — neither is currently supported

This ADR establishes code ground truth, answers the per-step fan-out question, and
determines the correct architecture for the activity surface.

---

## D1 — Code Ground Truth: Multi-Record Prior Step Behavior

### How `store_step_vars` populates inter-step variables

`store_step_vars` (in `crates/prism-spec-engine/src/pipeline.rs`) populates `step_vars`
at the end of each step's page loop. It stores:

1. Fields explicitly named in `variables_produced` — extracted at `$.{var_name}` from the
   raw response body, stored as `"{step_name}.{var_name}"`.
2. The JSONPath-extracted value (from `response_path`) under the last dot-segment of that
   path, stored as `"{step_name}.{last_segment}"`.

For a `fetch_devices` step with `response_path = "$.data.results"` returning N device
objects, `store_step_vars` stores:

```
step_vars["fetch_devices.results"] = Array([{device_id: "d-001", ...}, {device_id: "d-002", ...}, ...])
```

`step_vars["fetch_devices.device_id"]` is **NEVER populated** — `store_step_vars`
does not decompose fields from within individual records of an array. The only automatic
storage is the full extracted value under the response_path last segment.

### How `find_fan_out_array` and `fan_out_batches` behave

`find_fan_out_array` scans template references and returns the first key whose value
in `step_vars` is a JSON Array. For a subsequent step whose template references
`${fetch_devices.results}`:

- The full N-object array is detected as the fan-out source
- `fan_out_batches` chunks it into slices of `fan_out_batch_size` items (default 100)
- Each batch slice is an `Array` of full object values
- In `UrlPath` context, `value_to_string(Array(...))` returns the JSON serialization of
  the array, which `percent_encode` then encodes into a URL segment

For N device objects, this produces a URL path segment like:
`%5B%7B%22device_id%22%3A%22d-001%22%2C...%7D%5D` — a percent-encoded JSON array of
objects. This is not a valid `:device_id` path parameter for
`GET /api/v1/devices/:device_id/activity`.

### What happens with `${fetch_devices.device_id}` in a template

`fetch_devices.device_id` is not inserted into `step_vars` by `store_step_vars` (see
above). When `Interpolator::interpolate` encounters `${fetch_devices.device_id}`:

- It checks `vars` for the key `"fetch_devices.device_id"` — not found
- It then checks if ANY key starts with `"fetch_devices."` — true (from
  `"fetch_devices.results"`)
- Result: `InterpolationError::FieldNotFound` — interpolation aborts and the pipeline
  returns `SpecEngineError::HttpRequestFailed` with `status_code = 0` and detail
  `"path interpolation failed: variable '${fetch_devices.device_id}' not found..."`

No silent empty result: the pipeline errors.

---

## D2 — Precedent Survey: Existing Multi-Step Specs

Every multi-step sensor TOML spec in `crates/prism-sensors/specs/` uses the batch
fan-out pattern exclusively:

| Sensor | Step 1 | Step 2 | Fan-out model |
|--------|--------|--------|---------------|
| CrowdStrike detections | GET IDs → `$.resources` (array of string IDs) | POST `{"ids": ${step1.resources}}` to bulk entities API | Batch: N strings chunked, one POST per 100-ID batch |
| CrowdStrike devices | GET IDs → `$.resources` | POST `{"ids": ${step1.resources}}` | Same |
| CrowdStrike incidents | GET IDs → `$.resources` | POST `{"ids": ${step1.resources}}` | Same |

**None of these specs uses per-record fan-out** (one request per record, scalar field
in URL path). The CrowdStrike pattern works exclusively because:

1. Step 1 returns an array of **scalar string IDs** (not objects)
2. Step 2 sends that array **as a batch body** to a **bulk API endpoint** that accepts
   multiple IDs in one request

The Armis activity endpoint `GET /api/v1/devices/:device_id/activity` has no bulk API
analogue in the DTU — each request is for one device.

---

## D3 — Decision on Fan-Out Question (A / B / C)

**The answer is (C) — per-record fan-out from a prior step is a genuine capability gap.**

The existing fan-out mechanism (batch fan-out to bulk APIs) does NOT produce one HTTP
request per record with a scalar field in the URL path. The engine as-written in
`pipeline.rs execute_impl` and `find_fan_out_array`:

- Stores multi-record steps as a single array value (not decomposed per-record scalars)
- Fans out by chunking the whole array into batches (each batch is a sub-array)
- Inserts the batch sub-array into the template — not individual scalar values

Any sensor pattern requiring "iterate over N prior-step records, issue N requests each
with one scalar field from the current record" cannot be expressed in the current TOML
grammar. This is a capability gap distinct from pagination, batch fan-out, or push-down.

**This capability gap does NOT block `S-WAVE-A-ARMIS-ACTIVITY-001`** — see §D4.

---

## D4 — Resolution for Armis Activity Surface (DECISION)

The Armis device activity endpoint is inherently per-device: `GET /api/v1/devices/:device_id/activity`.
The correct model for this surface is a **filter-required query** using the push-down
grammar already supported by the spec engine:

```
path_template = "/api/v1/devices/${query.filter.device_id}/activity"
```

### Why this works today

`PipelineExecutor.execute_impl` pre-seeds `step_vars` before the steps loop with all
`FetchContext.query_filters` entries under the `query.filter.{key}` namespace. This is
the same mechanism used by `${query.filter.aql}` in the Armis devices/alerts tables and
`${query.filter._fql}` in the CrowdStrike detections table.

`Interpolator::interpolate` resolves `${query.filter.device_id}` as a scalar string
from the pre-seeded `step_vars["query.filter.device_id"]` — no prior step, no fan-out,
no new mechanism required.

### Behavior when no `device_id` filter is provided — Code Gap

**Current code behavior (verified 2026-07-27):** `seed_missing_query_filter_vars §seed_missing_query_filter_vars`
(called in `execute_impl §execute_impl`; authority: this ADR §D4 / §D5 — not ADR-033 T1) scans every step's `path_template` and
`body_template` for `${query.filter.*}` references and pre-seeds any absent key with an
empty string via `.or_insert(serde_json::Value::String(String::new()))`. Because
`path_template = "/api/v1/devices/${query.filter.device_id}/activity"` references
`query.filter.device_id`, an absent filter produces:

1. `step_vars["query.filter.device_id"]` is pre-seeded with `""` (empty string) —
   the key is NOT absent; no `InterpolationError::FieldNotFound` fires
2. `Interpolator::interpolate` resolves `${query.filter.device_id}` to `""` successfully
3. The constructed URL path is `/api/v1/devices//activity` — a malformed URL with an
   empty path segment
4. This malformed request is issued upstream. Against the DTU clone (the test boundary
   used by all `S-WAVE-A-ARMIS-ACTIVITY-001` scenarios): `matchit` 0.7.3 (via
   `axum` 0.7.9) matches route `/api/v1/devices/:device_id/activity` with
   `device_id = ""` — the empty path segment between the two `/` separators is
   captured as an empty param by `matchit::tree §NodeType::Param` `split_at(0)` logic
   (verified from matchit-0.7.3 source `tree.rs §NodeType::Param` branch). Axum does
   not normalize double slashes before routing; matchit receives the literal
   `//activity` suffix, finds the first `/` at index 0, and stores `""` as the
   `device_id` param value. `get_device_activity §get_device_activity` then executes
   `state.activity_fixture.iter().filter(|a| a.device_id == "")` → no fixture records
   carry `device_id == ""` → HTTP 200 with `activities: [], total: 0`
   (**silent empty result**). This is the pre-fix DTU behavior contracted in
   BC-2.02.014 EC-014-001 and AC-004 / RG-004. Against the real upstream Armis API,
   behavior depends on their server's empty-segment handling and is not settled from
   static analysis alone; the DTU is the test boundary and its behavior is fully
   determined as above. The Prism engine does NOT produce a structured hard error
   before the request is sent — the failure propagates as a silent empty result
   from the DTU/upstream response, not as a spec-engine-level validation error

**§D4 canonical version note (v0.3–v0.7, for downstream citeability):** The
pre-seed mechanism documented above — `seed_missing_query_filter_vars §seed_missing_query_filter_vars`
inserts an empty-string value for the absent slot, producing the malformed path
`/api/v1/devices//activity` — has been the §D4-documented behavior since v0.3
(2026-07-27, FB74 ITEM 5). Any reference to ADR-057 §D4 claiming
`Interpolator::interpolate §interpolate` would abort on a missing
`query.filter.device_id` key (producing `InterpolationError::FieldNotFound`)
describes RETIRED v0.1/v0.2 text. Current §D4 contains no such claim. The
pre-seed inserts the key BEFORE interpolation runs; `FieldNotFound` does NOT
fire for `${query.filter.*}` slots in the current engine.

This is the **silent malformed-request class** described in CLAUDE.md Standing Rule 3 §2
and SOUL.md §4: an empty interpolation slot in a URL path segment yields a malformed
upstream request rather than a hard error at the engine level.

**Specified/required behavior (contracted in BC-2.02.014):** An absent `device_id`
filter MUST produce a hard error at the Prism spec-engine level before any upstream
request is issued. This contracted behavior satisfies EC-001 from
`S-WAVE-A-ARMIS-ACTIVITY-001`. The implementation obligation belongs to
`S-WAVE-A-ARMIS-ACTIVITY-001` — the code must be brought into alignment with the spec
per the Source-of-Truth Precedence rule (CLAUDE.md §Source-of-Truth Precedence: spec wins
over code; code is brought into alignment via fix or story, not the other way around).

**Architectural note — query-param vs path-segment divergence:** `seed_missing_query_filter_vars`
was designed for OPTIONAL filter slots used as URL query parameters (e.g.,
`${query.filter.aql}` → `?aql=` for Armis devices/alerts, and `${query.filter._fql}` →
`?filter=` for CrowdStrike detections). An empty query parameter is semantically safe
for optional filters — the upstream API treats it as "no filter applied." A REQUIRED
filter slot in a URL path segment has a fundamentally different failure mode: an empty
string produces a structurally malformed path (`/devices//activity`). Future sensor specs
that use `${query.filter.*}` in URL path segments with required semantics will produce a
malformed URL on absent filter input under the current engine. Until
`seed_missing_query_filter_vars` is extended to differentiate path-segment from query-param
positions, any such sensor surface requires an engine-level required-filter gate anchored
to a specific BC postcondition in the implementing story. For `armis_device_activity`, that
obligation is anchored in BC-2.02.014 §Postconditions under `S-WAVE-A-ARMIS-ACTIVITY-001`.

### Scope of this table

The `armis_device_activity` surface is a **single-device, filter-required query table**.
Fleet-wide activity queries (iterating all devices without a filter) are out of scope for
Wave-A and require the per-record fan-out capability gap to be filled in a future story.
That future story is not gated, not named, and not required before Wave-A ships.

---

## D5 — TOML Specification for `armis_device_activity`

### Complete `[[tables]]` header (authoritative copy-source)

The `[[tables]]` header block below is the authoritative copy-source for BC-2.02.014
§TOML Contract and `S-WAVE-A-ARMIS-ACTIVITY-001` T-IMPL-01. Every field must be
reproduced verbatim; substituting keys or values not present here produces a
spec-parse failure or a double-prefixed table name.

```toml
[[tables]]
table_name = "device_activity"
ocsf_class = "network_activity"
```

**`table_name` value and the double-prefix trap.** `register_sensor §register_sensor`
in `crates/prism-query/src/table_registry.rs` composes the registered SQL surface
as `format!("{}_{}", spec.sensor_id, table.table_name)`. With `sensor_id = "armis"`:

- `table_name = "device_activity"` → registered as `"armis_device_activity"` ✓
- `table_name = "armis_device_activity"` → registered as `"armis_armis_device_activity"` ✗

All downstream consumers — BC-2.02.014 test vectors (TV-014-001 through TV-014-006),
`S-WAVE-A-ARMIS-ACTIVITY-001` AC-005/AC-007 and RG-004/RG-005/RG-007, and
BC-2.02.006 EC-02-014 — query `FROM armis_device_activity`. The correct value is
`"device_activity"`.

**`ocsf_class` decision — `"network_activity"`.** `TableSpec §TableSpec` requires
`ocsf_class: String` with no `#[serde(default)]`; omission causes a deserialization
error at boot. The correct OCSF class for Armis per-device activity records is
`"network_activity"` (OCSF Class 4001, Network Activity category):

- Armis is a network security platform for IoT/OT devices; its device activity API
  primarily captures network behavior events (connectivity, traffic, protocol activity).
- OCSF Class 4001 provides `device` (the endpoint — mapped to `device_id →
  device.uid`), `activity_name` (action type — mapped to `activity_type →
  activity_name`), and `time` (when — mapped to `timestamp → time`), covering all
  five columns in the schema.
- `"detection_finding"` (OCSF Class 2001) is excluded: it covers security findings
  and policy violations — that is the semantics of the `alerts` table. Activity records
  are raw behavioral events, not evaluated findings.
- `"device"` (OCSF device inventory class) is excluded: activity log ≠ inventory.

Existing armis table precedent: `devices → "device"`, `alerts → "detection_finding"`.
All use the OCSF short-name convention (not numeric class IDs).

**Required TOML fields.** `TableSpec §TableSpec` declares `pub table_name: String` and
`pub ocsf_class: String` with no `#[serde(default)]`. Both are mandatory. The keys
`name` and `sensor_name` are not `TableSpec` fields and will be ignored (or cause a
TOML parse error depending on deny-unknown config). Do not use them.

### Step block (MUST)

The step block for the activity table MUST use this pattern (TOML sub-table of the
`[[tables]]` header above):

```toml
[[tables.steps]]
name = "fetch_device_activity"
method = "GET"
path_template = "/api/v1/devices/${query.filter.device_id}/activity"
response_path = "$.data.activities"
variables_produced = []
```

And the `device_id` column MUST declare push-down eligibility:

```toml
[[tables.columns]]
name = "device_id"
column_type = "string"
ocsf_field = "device.uid"
options = ["INDEX"]
```

`options = ["INDEX"]` declares `device_id` push-down eligible per the BC-2.11.007
push-down taxonomy (REQUIRED / INDEX / ADDITIONAL classification) and future T2
(`classify_predicates §classify_predicates`) integration. The current routing path is
annotation-agnostic: `predicate_tree_to_filter_map §predicate_tree_to_filter_map`
collects all case-sensitive `field = 'string'` equality predicates regardless of column
annotation into the `FilterMap`, which materializes as `FetchContext.query_filters`;
`execute_impl §execute_impl` then pre-seeds `step_vars["query.filter.device_id"]` from
that map (this ADR §D4). ADR-033 T1 is NOT the authority here — ADR-033 T1 governs
datetime time-window extraction into `QueryParams.start_time`/`end_time` only
(authority: `extract_time_window_from_ast §extract_time_window_from_ast`).

Anchor: All three MUSTs above (table header, step block, `device_id` column) resolve in
`S-WAVE-A-ARMIS-ACTIVITY-001` AC-001 / RG-001
(`test_armis_toml_armis_device_activity_table_declared_with_correct_step_block`).
The `device_id` column `options = ["INDEX"]` obligation also resolves in AC-002 / RG-002
(`test_armis_toml_armis_device_activity_device_id_column_has_index_option`). Deferral
is against real story ID `S-WAVE-A-ARMIS-ACTIVITY-001` per Canonical Principle Rule 3.

---

## D6 — Future Capability Gap: Per-Record Fan-Out

For completeness, the per-record fan-out capability (answer A to the original question,
for `${prior_step.field}` references where the prior step returned N objects) would
require:

- A new `for_each: "prior_step_name"` field on `FetchStep` (in `spec_parser.rs`)
- Within a `for_each` step, `${step_name.field}` resolves from the current iteration
  record, not from the pre-stored array
- New execution path in `PipelineExecutor.execute_impl`: when a step declares
  `for_each`, iterate the prior step's stored array, decompose each record's fields
  into a per-iteration `step_vars` copy, and issue one request per record
- New validation rule: `for_each` step may not also declare `fan_out_batch_size`;
  the two mechanisms are mutually exclusive

This is a non-trivial engine extension requiring its own story, ADR amendment, and
BC update. It is not in scope for Wave-A. **No story is created here** — if the
capability is needed for a future sensor surface, a new dedicated story MUST be
proposed to the product-owner at that time, with this ADR as the architecture anchor.

---

## Consequences

**C1 (UNBLOCK):** `S-WAVE-A-ARMIS-ACTIVITY-001` is unblocked. The TOML grammar to
use is `path_template = "/api/v1/devices/${query.filter.device_id}/activity"` with
`device_id` column `options = ["INDEX"]`. Product-owner may now author the BCs;
story-writer may update the story to `status: ready` once BCs are authored and Red Gate
list is enumerated per SAC-1.

**C2 (STORY-WRITER):** `S-WAVE-A-ARMIS-ACTIVITY-001` §Blocking Dependency MUST be
updated to record: (a) the architect resolution date (2026-07-27), (b) the confirmed
grammar (`${query.filter.device_id}`), and (c) a §Authority cite for ADR-057 per SAC-2.
Anchor: `S-WAVE-A-ARMIS-ACTIVITY-001` story update (handoff to story-writer).

**C3 (PRODUCT-OWNER):** BC-2.02.006 EC-02-014 deferral text MUST be updated to reflect:
(a) the resolved grammar (`${query.filter.device_id}`), (b) the single-device
filter-required scope, and (c) the story's unblocked status. The per-record fan-out
capability gap (D6) is NOT resolved — fleet-wide activity remains out of scope.
Anchor: `S-WAVE-A-ARMIS-ACTIVITY-001` (existing story ID, handoff to product-owner).

**C4 (CAPABILITY GAP RECORD):** Per-record fan-out from a prior multi-record step
(`for_each` semantic) is confirmed as a capability gap in the current engine. It is
not required for Wave-A. If a future sensor surface requires it, this ADR §D6 is the
architecture anchor for the implementing story.

**C5 (ADR INDEX):** State-manager MUST add ADR-057 to ARCH-INDEX.md with the standard
ADR row format in the same burst that commits this file. Anchor: standard state-manager
burst obligation per factory-dispatcher TD-VSDD-053.

**C6 (IMPLEMENTER — engine alignment, pre-ship obligation):** `execute_impl`'s
`seed_missing_query_filter_vars` currently pre-seeds absent `${query.filter.*}` slots with
empty string regardless of whether the slot occupies a query-param or path-segment position.
For `armis_device_activity`, an absent `device_id` filter produces the path
`/api/v1/devices//activity` — a malformed URL issued silently to the upstream API, not a
hard engine error. The implementing story `S-WAVE-A-ARMIS-ACTIVITY-001` must add a
required-filter gate that fires before any upstream request is issued when the required
`device_id` slot is absent, consistent with BC-2.02.014 §Postconditions. Anchor:
`S-WAVE-A-ARMIS-ACTIVITY-001` / BC-2.02.014.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.7 | 2026-07-29 | architect | FB88 — three-concern correction. CONCERN A: §D5 `[[tables]]` header added as authoritative copy-source for BC-2.02.014 §TOML Contract and `S-WAVE-A-ARMIS-ACTIVITY-001` T-IMPL-01. `table_name = "device_activity"` (not `"armis_device_activity"`) — `register_sensor §register_sensor` composes `format!("{}_{}", spec.sensor_id, table.table_name)` = `"armis_device_activity"` with the value above; the double-prefix trap documented inline. `ocsf_class = "network_activity"` (OCSF Class 4001) — selected for Armis IoT/OT network behavioral events; rationale in §D5. `name`/`sensor_name` are not `TableSpec §TableSpec` fields; both keys absent from corrected header. CONCERN B: §D4 step 4 corrected — the "likely 404/error" claim removed. Ground truth from `matchit::tree §NodeType::Param` `split_at(0)` logic (matchit-0.7.3 source verified): empty segment between `devices/` and `activity` is captured as `device_id = ""`; `get_device_activity §get_device_activity` filters `activity_fixture` for `device_id == ""` → empty → HTTP 200 with `activities: [], total: 0`. This is the pre-fix DTU behavior contracted in BC-2.02.014 EC-014-001. The outcome is fully determined from static analysis; no empirical pin required. CONCERN C: §D4 canonical version note added — explicitly states that the `seed_missing_query_filter_vars §seed_missing_query_filter_vars` pre-seed mechanism (not `Interpolator::interpolate §interpolate`) has been the §D4-documented behavior since v0.3; `FieldNotFound` does NOT fire for `${query.filter.*}` slots; retired v0.1/v0.2 text is explicitly superseded. POL-29 9a: ADR-056 (pagination twin) carries no `[[tables]]` header content or `register_sensor` composition claim — no sweep needed; `query-engine.md` mentions `{sensor_id}_{source}` format in passing, consistent with the correction. CLEAR. 9b: §D5 complete `[[tables]]` header block is the authoritative copy-source; marked as such. The step block and column block are sub-tables of this header; all three are copy-safe as a unit. 9c: All three MUSTs in §D5 anchored to `S-WAVE-A-ARMIS-ACTIVITY-001` AC-001/RG-001 (header + step block) and AC-002/RG-002 (INDEX column); no unanchored MUSTs introduced. SAC-2: `anchor_stories` carries `[S-WAVE-A-ARMIS-ACTIVITY-001]`; `S-WAVE-A-ARMIS-SPEC-001` §Authority was not checked (out of scope for this ADR's surface). Ratification: corrections repair statement accuracy (factual omission, incorrect empirical claim); no ratified design decision is changed. |
| 0.6 | 2026-07-28 | architect | FB81 §D4 self-miss correction (POL-29 9a). The FB81 §D5 fix did not sweep the sibling §D4. §D4 retained "`seed_missing_query_filter_vars §seed_missing_query_filter_vars` (called in `execute_impl §execute_impl` per ADR-033 T1)" — the identical wrong attribution removed from §D5. §D4 now reads "(called in `execute_impl §execute_impl`; authority: this ADR §D4 / §D5 — not ADR-033 T1)". Full ADR-033 sweep: 4 hits — frontmatter `related_adrs` (valid: §D5 still references ADR-033 for scope-boundary explanation), §D4 wrong attribution (fixed this row), §D5 scope-boundary statement (legitimate), v0.5 changelog row (legitimate historical record). `related_adrs: [ADR-028, ADR-033, ...]` retained: ADR-033 remains a meaningful related ADR because §D5 explicitly names it to state the scope boundary (datetime extraction only). |
| 0.5 | 2026-07-28 | architect | FB81 — §D5 ADR-033 T1 mis-citation corrected. The sentence "options = ['INDEX'] is required... via the push-down extraction path (ADR-033 T1 convention for INDEX-declared columns)" was the copy-source for a wrong causal claim that propagated to BC-2.02.014, BC-2.02.006, and S-WAVE-A-ARMIS-ACTIVITY-001. Accurate replacement: `options = ["INDEX"]` declares push-down eligibility per BC-2.11.007 taxonomy and future T2 (`classify_predicates §classify_predicates`); the CURRENT routing is annotation-agnostic via `predicate_tree_to_filter_map §predicate_tree_to_filter_map` (collects all equality predicates regardless of annotation) → `FetchContext.query_filters` → `execute_impl §execute_impl` pre-seed loop (this ADR §D4). ADR-033 T1 governs datetime time-window extraction only (`extract_time_window_from_ast §extract_time_window_from_ast`). Sites 2–5 (BC-2.02.014, BC-2.02.006, S-WAVE-A-ARMIS-ACTIVITY-001 AC-002/RG-002) fixed by product-owner and story-writer in same FB81 burst. |
| 0.4 | 2026-07-27 | architect | FB76 — (1) Unanchored `MUST` in §D4 architectural note (future sensor specs) replaced with anchored guidance: obligation for `armis_device_activity` is anchored in BC-2.02.014 / `S-WAVE-A-ARMIS-ACTIVITY-001`; future surfaces must do the same in their implementing story. (2) C6 added: engine-alignment pre-ship obligation for the implementer, anchored to BC-2.02.014 / `S-WAVE-A-ARMIS-ACTIVITY-001`. POL-29 dimension 9c: all MUSTs now have explicit story+BC anchors. |
| 0.3 | 2026-07-27 | architect | FB74 ITEM 5 — §D4 "Behavior when no device_id filter is provided" corrected. False claim ("falls through pre-seed path without inserting; hits FieldNotFound") replaced with accurate code ground truth: `seed_missing_query_filter_vars` pre-seeds any absent `${query.filter.*}` path-template slot with empty string; absent `device_id` → `/api/v1/devices//activity` malformed URL silently sent upstream. Required behavior contracted in BC-2.02.014; implementation obligation anchored to `S-WAVE-A-ARMIS-ACTIVITY-001`. Architectural note added: query-param vs path-segment divergence (empty query param = safe for optional filters; empty path segment = malformed URL). BC-2.02.014 added to `related_bcs`. |
| 0.2 | 2026-07-27 | architect | FB74 SAC-2 anchor_stories update: S-WAVE-A-ARMIS-ACTIVITY-001 v1.1 §Authority cites "ADR-057 (accepted 2026-07-27)"; promoted to verified `anchor_stories` entry with per-entry annotation. Stale verified-empty comment annotation removed (POL-29 dimension 9b — no longer accurate). |
| 0.1 | 2026-07-27 | architect | Initial authoring. Resolves S-WAVE-A-ARMIS-ACTIVITY-001 blocking dependency (architect-confirmation-required gate). Code ground truth: store_step_vars stores full array only; find_fan_out_array batches arrays; per-record fan-out is (C) capability gap. Resolution: push-down grammar ${query.filter.device_id} unblocks the story. Per-record fan-out mechanism documented as D6 future gap. |
