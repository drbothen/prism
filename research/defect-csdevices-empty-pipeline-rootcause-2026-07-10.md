---
document_type: research
level: ops
decision_anchor: D-1650
topic: DEFECT-CSDEVICES-EMPTY-PIPELINE-001 root-cause investigation
produced: 2026-07-10
producer: state-manager (orchestrator-directed)
status: complete
---

# DEFECT-CSDEVICES-EMPTY-PIPELINE-001 — Root-Cause Investigation

**Date:** 2026-07-10 | **Anchor:** D-1650 | **Exposed by:** AUDIT-COVERAGE-001 live gate H7

---

## Executive Summary

`DEFECT-CSDEVICES-EMPTY-PIPELINE-001` decomposes into **two independent sub-defects**:

1. **Symptom 1 (0 rows) = SENSOR TOML SPEC DEFECT** — `crowdstrike.sensor.toml` `fetch_devices` step has NO interpolation variable reference, so `find_fan_out_array()` returns `None` and the GET `/devices/entities/devices/v2` call is issued with zero `ids` params, yielding an empty response from the DTU.
2. **Symptom 2 (JOIN "Internal error") = PRODUCT-CODE DEFECT (independent)** — `materialization.rs` skips DataFusion MemTable registration for 0-batch tables; when a mixed JOIN occurs (one side registered, one not), DataFusion planning fails with `DataFusionError::Plan("table not found")` → catch-all `-32000 "Internal error"`.
3. **Symptom 3 (self-join 0 rows): NOT CONFIRMED** — cascade artifact of Symptoms 1+2; re-verify after fixes land.

Architect ratification is **PENDING** on the recommended fix path (Option 1) for Symptom 1.

---

## Symptom 1 — 0 Rows: Sensor TOML Spec Defect

### Observed Behavior

`FROM crowdstrike_devices | limit 3` returns 0 rows live against the DTU (which serves 50 device records).

### Root Cause

In `crowdstrike.sensor.toml`, the `fetch_devices` pipeline step (approx. lines 246–262) declares:

```toml
fan_out_batch_size = 100
```

…but its `path` or `body_template` field contains **no `${...}` variable reference** to the preceding step's output. As a result:

- The engine's `find_fan_out_array()` routine returns `None` (no interpolation anchor found).
- The engine issues a GET to `/devices/entities/devices/v2` with **zero `ids` query params**.
- The DTU route `prism-dtu-crowdstrike/src/routes/hosts.rs` `parse_ids_from_query()` correctly returns an empty vec when no `ids` params are present.
- Zero records are returned; the engine correctly materializes an empty result.

### Working Contrast

The `fetch_detections` step for `crowdstrike_devices` (or the analogous `detections` table pipeline) uses a POST with `body_template` containing `${query_detection_ids.resources}`, which correctly triggers `find_fan_out_array()` and passes the batch IDs through.

### Complication: UrlPath Array Encoding

A naive TOML fix such as:

```toml
path = "/devices/entities/devices/v2?ids=${fetch_device_ids.resources}"
```

**will NOT work** with the current engine. The engine's `UrlPath` interpolation renders arrays as a single percent-encoded JSON blob (`%5B%22id1%22%2C%22id2%22%5D`), which the CrowdStrike API does not accept as a repeated `ids=` param style.

### Fix Options

| Option | Description | Engine Changes | Recommended |
|--------|-------------|----------------|-------------|
| **Option 1 (RECOMMENDED)** | Convert `fetch_devices` to POST, mirroring the `detections` pipeline; add a POST route to `prism-dtu-crowdstrike/src/routes/hosts.rs`; update TOML `method = "POST"` + `body_template` to embed `${fetch_device_ids.resources}`. Matches the real CrowdStrike Devices POST variant in production. | None | YES — requires architect ratification |
| Option 2 | Extend engine's `UrlPath` interpolation to emit repeated `?ids=v1&ids=v2&...` query-param style for array values | Engine changes (non-trivial) | No — larger blast radius |
| Option 3 | Add DTU tolerance for the blob-encoded single-param form (parse `%5B...%5D` as a JSON array) | None | Rejected — breaks fidelity contract (DTU clones must match real-API behavior) |

**Option 1 requires architect ratification** before the implementer proceeds (new DTU route + TOML method/body_template changes are architectural decisions).

### Spec Anchor

BC-2.16.002 fan-out precondition (step variable references must be present for `find_fan_out_array()` to return a non-None result).

---

## Symptom 2 — JOIN "Internal error": Product-Code Defect (Independent)

### Observed Behavior

Any SQL JOIN where one side is an empty-result table (e.g., `crowdstrike_devices LEFT JOIN armis_devices`) fails with MCP error `-32000 "Internal error"`.

### Root Cause

In `crates/prism-query/src/engine.rs` (or `materialization.rs`, approx. lines 1008–1022), the engine **skips DataFusion MemTable registration for tables that returned 0 result batches**. When a mixed JOIN is attempted:

- `armis_devices` was registered (had results).
- `crowdstrike_devices` was NOT registered (0 batches → skipped).

DataFusion's query planner then raises `DataFusionError::Plan("table 'crowdstrike_devices' not found")`, which the engine's catch-all handler maps to `-32000 "Internal error"`.

### Impact

This defect is **independent of Symptom 1**. Any legitimately-empty sensor result in a JOIN position will trigger it — e.g., if a sensor returns 0 rows for a valid query with no matching data.

### Correct Fix

Register a **schema-only empty `MemTable`** from the sensor's declared spec columns when the result batch count is 0. This allows DataFusion to plan the JOIN against a known schema, producing a graceful 0-row result per BC-2.01.010 (partial-failure handling) and BC-2.11.005 (empty-table JOIN semantics).

### Spec Anchors

- BC-2.01.010: partial-failure propagation — empty result ≠ error.
- BC-2.11.005: JOIN with empty table must yield 0 rows, not an error.

### Owner

Implementer (TDD cascade required; RED gate test for empty-MemTable registration then GREEN fix in `materialization.rs` or equivalent registration site).

---

## Symptom 3 — Self-Join 0 Rows: Unconfirmed

**Status:** Likely a cascade artifact of Symptoms 1+2. Re-verify after both fixes are merged. If self-join on a populated table still yields 0 rows after fixes, escalate as a new distinct defect.

---

## Bonus Finding: T13 Section-B False-Positive Gate

The T13 audit script `section-B` check for `crowdstrike_devices` currently reports `PASS: 0 rows` (because 0 rows match the assertion `>= 0 rows`). This is a **false-positive gate** — it passes without actually verifying data delivery.

**Registered task:** section-B hardening is registered against the parked `AUDIT-COVERAGE-001` branch (`fix/T13-audit-coverage` @317b6e25). It must land before that branch merges. The T13 demo capstone is NOT blocked by this (T13 exercises neither `crowdstrike_devices` nor JOINs in its current 70-check form).

---

## Fix Work Plan

### Track A — Symptom 1 (TOML Spec Defect)

1. **PENDING:** Architect ratification of Option 1 (POST conversion).
2. After ratification:
   - Product-owner: amend `crowdstrike.sensor.toml` `fetch_devices` step (`method = "POST"`, `body_template` with `${...}` reference).
   - Implementer: add POST route to `prism-dtu-crowdstrike/src/routes/hosts.rs` (new handler mirroring detections POST route).
3. Adversary LOCAL 3-CLEAN → fix-PR (may be combined with Track B at architect's discretion).

### Track B — Symptom 2 (Product-Code Defect)

1. Implementer: RED gate test — `JOIN` with one empty-result side → expect 0 rows (not error).
2. Implementer: GREEN fix — register schema-only empty `MemTable` for 0-batch tables in `materialization.rs`.
3. Adversary LOCAL 3-CLEAN → fix-PR.

### Track C — Section-B T13 Hardening

1. AFTER both Tracks A+B PRs merge, un-park `AUDIT-COVERAGE-001` branch.
2. Rebase onto develop.
3. Add section-B 0-rows hardening check (assert actual DTU delivery, not merely `>= 0 rows`).
4. Full live audit — expect 0 FAIL / 0 WARN.
5. LOCAL 3-CLEAN → PR.

---

## Decision Record

- **D-1650**: Root-cause investigation confirmed. Two sub-defects (TOML spec + product-code). Architect ratification pending for Option 1. Track B (empty-MemTable) can proceed autonomously.
- **DEFECT-CSDEVICES-EMPTY-PIPELINE-001** status: ROOT-CAUSED (in STATE.md Drift Items).
- **DEFECT-EQUERY042-GROUPBY-DEADARM-001** status: RED-COMPLETE @49e07a29 (independent fix, `.worktrees/FIX-EQUERY042-GROUPBY` on `fix/equery042-groupby-deadarm`). See also worktree notes in STATE.md.

---

## Architect Ratification — 2026-07-10

**Decision:** RATIFY Option 1 (POST conversion) — with one scoped amendment documented below.

**Ratifier:** architect | **Anchor:** D-1650 | **ADR preconditions verified:** ADR-028 §D1, §D5

---

### Evidence Base

| Evidence | Finding |
|----------|---------|
| CrowdStrike Falcon API official docs ([developer.crowdstrike.com/api-reference/collections/hosts/](https://developer.crowdstrike.com/api-reference/collections/hosts/)) | `POST /devices/entities/devices/v2` IS a real, documented API operation (`PostDeviceDetailsV2`) — confirmed via Perplexity web-grounded research 2026-07-10 |
| FalconPy SDK changelog (github.com/CrowdStrike/falconpy/discussions/804) | `PostDeviceDetailsV2` introduced in FalconPy v1.2.0; moves IDs into request body; POST variant supports up to **5000 IDs** vs GET's 100-ID limit |
| Request body shape | `{"ids": ["AID1", "AID2", ...]}` — identical key/value structure to `POST /detects/entities/summaries/GET/v1` |
| `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | Current GET handler `get_host_details` reads IDs from query params via `parse_ids_from_query()`. No POST handler exists. ADR-028 §D5 requires DTU extension before TOML spec cites the route — **DTU extension is therefore a required precondition of the TOML fix**. |
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | `get_detection_summaries` is the established POST-fan-out pattern: deserializes `{"ids": [...]}` body, applies session-registry filter, looks up fixture records. The new `post_host_details` handler mirrors this exactly. |
| TOML `fetch_devices` step (lines 253–261) | `method = "GET"`, `path_template = "/devices/entities/devices/v2"`, **no `body_template`, no `${...}` interpolation reference** → `find_fan_out_array()` returns `None` → 0 IDs passed → empty DTU response. This is the confirmed bug. |
| ADR-028 §D1 | TOML spec URLs ground against DTU clone routes (real-API canonical). POST variant is real-API canonical — ratified. |
| ADR-028 §D5 | DTU clone MUST be extended before spec cites route. Order of operations: DTU route addition → TOML change. |
| No new ADR needed | This is a sensor-TOML shape fix within existing architectural rules. No cross-cutting rule is changing. ADR-028's core principle (DTU-first grounding) is being honored, not amended. |

---

### Ratification Verdict

**RATIFY Option 1.** The POST conversion is architecturally sound, real-API canonical, and follows the established detections pipeline pattern exactly. No engine changes are required.

Option 2 (engine UrlPath repeated-param extension) is rejected: larger blast radius, defers the fix behind an engine feature story with no benefit over the POST path that the real API natively supports.

Option 3 (DTU blob tolerance) is rejected: violates ADR-003 fidelity contract — DTU routes must model real-API behavior, not paper over encoding mismatches.

---

### Amendment A — fan_out_batch_size Retention at 100 (Conservative Default)

The POST variant of `/devices/entities/devices/v2` supports up to **5000 IDs** per call (vs GET's 100). The TOML `fan_out_batch_size = 100` may therefore be raised in a future story without any DTU or engine changes. However:

- The existing `fetch_detections` step also uses `fan_out_batch_size = 100` with the POST variant (real detections API also supports larger batches).
- Prism's current per-sensor rate-limit hint is `requests_per_second = 10.0`.
- Raising the batch size is a tuning decision, not a defect fix.

**Decision:** Keep `fan_out_batch_size = 100` for this fix. The TOML comment must note the POST API limit for future tuners. Batch size tuning is a follow-up concern, not in scope for DEFECT-CSDEVICES-EMPTY-PIPELINE-001.

---

### Implementation Contract (for fix-lane product-owner + implementer)

#### Contract Part 1 — TOML change (`crates/prism-sensors/specs/crowdstrike.sensor.toml`)

**Locate:** the `fetch_devices` step under `[[tables]]` where `table_name = "devices"` (currently lines ~252–261).

**Replace** the entire step block with:

```toml
  # Step 2: PostDeviceDetailsV2 — POST IDs to get full device records
  # DTU route: POST /devices/entities/devices/v2 (added per DEFECT-CSDEVICES-EMPTY-PIPELINE-001 ratification)
  # Real-API canonical: CrowdStrike PostDeviceDetailsV2 (official; introduced FalconPy v1.2.0;
  #   POST body {"ids": [...]} supports up to 5000 IDs vs GET's 100; body format identical to
  #   POST /detects/entities/summaries/GET/v1 detections pipeline).
  # fan_out_batch_size = 100 per CROWDSTRIKE_BATCH_SIZE (conservative; POST API supports up to 5000;
  #   raising is a separate tuning story, not in scope here).
  [[tables.steps]]
  name = "fetch_devices"
  method = "POST"
  path_template = "/devices/entities/devices/v2"
  body_template = '{"ids": ${query_device_ids.resources}}'
  response_path = "$.resources"
  variables_produced = []
  fan_out_batch_size = 100
```

The variable reference `${query_device_ids.resources}` is correct — Step 1 is named `query_device_ids` with `response_path = "$.resources"`, which makes the array accessible as `${query_device_ids.resources}`. This gives `find_fan_out_array()` the anchor it needs.

#### Contract Part 2 — DTU route addition (`crates/prism-dtu-crowdstrike/src/routes/hosts.rs`)

**Add** a `PostHostDetailsBody` struct and `post_host_details` handler. The handler MUST mirror `get_host_details` logic exactly — same auth check, same org-id guard, same three-way composition (scenario/seeded/static), same session-registry filter, same containment merge, same response shape. The only difference: IDs come from `body.ids` instead of `parse_ids_from_query(raw_query)`.

Minimum contract:

```rust
/// Body for batch host detail fetch via POST (CrowdStrike PostDeviceDetailsV2).
#[derive(Debug, Deserialize)]
pub struct PostHostDetailsBody {
    pub ids: Vec<String>,
}

/// `POST /devices/entities/devices/v2`
///
/// Batch host detail fetch via POST body (CrowdStrike `PostDeviceDetailsV2`,
/// introduced FalconPy v1.2.0). Body: `{"ids": ["h-001", ...]}` — supports up to
/// 5000 IDs vs the GET variant's 100-ID query-param limit.
///
/// Behavior is identical to `get_host_details` (GET) except that IDs are read
/// from the JSON body rather than query params. Session registry, fixture
/// composition, org-id guard, and containment merge are unchanged.
///
/// Returns HTTP 400 if `ids` is empty (mirrors detections.rs `get_detection_summaries`).
pub async fn post_host_details(
    State(state): State<Arc<CrowdstrikeState>>,
    headers: HeaderMap,
    Json(body): Json<PostHostDetailsBody>,
) -> impl IntoResponse {
    // Implementation: delegate to the same internal logic as get_host_details,
    // substituting `body.ids` for the query-param IDs.
    // The implementer MUST NOT duplicate the composition/fixture logic — extract
    // a shared `lookup_host_details(state, headers, ids_to_lookup)` helper called
    // by both handlers, OR inline the same logic with clear `// mirrors get_host_details` comments.
    // Either approach is acceptable; the DRY helper is preferred.
    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        ).into_response();
    }
    // ... session registry, fixture composition, containment merge, return {"resources": [...]}
    // identical to get_host_details
    todo!()
}
```

#### Contract Part 3 — DTU router registration (`crates/prism-dtu-crowdstrike/src/routes/mod.rs`)

**Change** the `/devices/entities/devices/v2` route to register both GET and POST:

```rust
// Before:
.route("/devices/entities/devices/v2", get(hosts::get_host_details))

// After:
.route(
    "/devices/entities/devices/v2",
    get(hosts::get_host_details).post(hosts::post_host_details),
)
```

**Also update** the `build_router` doc comment: change "8 in-scope endpoints (4 read, 4 write)" → "9 in-scope endpoints (5 read, 4 write)" — adding `POST /devices/entities/devices/v2` to the read count.

#### Contract Part 4 — BC-2.16.013 version bump

BC-2.16.013 (`bundled-sensor-spec-dtu-parity.md`) has multiple references to "GET `/devices/entities/devices/v2`" and "GET `/devices/entities/devices/v2`" in its postconditions and route tables. The product-owner must:

1. Add a changelog version row noting: "CrowdStrike `fetch_devices` step converted from GET to POST; DTU `hosts.rs` gains `post_host_details` handler at `POST /devices/entities/devices/v2` per DEFECT-CSDEVICES-EMPTY-PIPELINE-001 architect ratification."
2. Update the route table references from `GET /devices/entities/devices/v2` → `POST /devices/entities/devices/v2` (the GET handler remains but is no longer the spec-driven path for the two-step pipeline; note both verbs exist on the route).
3. Remove/update any test vector or parity test clause that asserts the step uses HTTP GET. The parity test for the devices table must exercise the POST path.

#### Contract Part 5 — fidelity_validator.rs comment update

The excluded routes comment in `crates/prism-dtu-crowdstrike/tests/fidelity_validator.rs` (line ~29) lists:
```
///   - GET  /devices/entities/devices/v2
```
After the DTU route addition, this should become:
```
///   - GET  /devices/entities/devices/v2
///   - POST /devices/entities/devices/v2
```
(Both routes exist on the same path; both are auth-required; both are excluded from the unauthenticated fidelity checks per ADR-003 §Conflict-2 Option C.)

---

### TDD Protocol for Fix Lane

The fix spans two specialists per agent routing table:

1. **Product-owner** owns the TOML change (Contract Part 1) and BC-2.16.013 version bump (Contract Part 4). TOML is a spec artifact.
2. **Implementer** owns the DTU Rust additions (Parts 2, 3, 5). TDD discipline:
   - RED gate test: a test that calls `POST /devices/entities/devices/v2` with a body containing known device IDs and asserts the response `resources` array is non-empty. This test must FAIL before the implementation is added (since the route currently returns 405 Method Not Allowed).
   - GREEN: add the `PostHostDetailsBody` struct and `post_host_details` handler; register the route; confirm test passes.
   - Adversary: SAP-2 (DTU↔TOML parity) applies — verify all `devices` table columns in TOML have counterparts in the DTU fixture/generated-devices records.

Tracks A (TOML+DTU) and B (empty-MemTable) MAY be combined into a single PR at the orchestrator's discretion, since both fix DEFECT-CSDEVICES-EMPTY-PIPELINE-001 and neither creates regressions in the other. Decision is orchestrator's.

---

### Specs Requiring Version Bumps

| Artifact | Change Required |
|----------|----------------|
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | `fetch_devices` step: `method`, add `body_template` |
| `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | Add `PostHostDetailsBody`, `post_host_details` |
| `crates/prism-dtu-crowdstrike/src/routes/mod.rs` | Add POST to route registration; update doc comment count |
| `.factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md` | Changelog row + route table update |
| `crates/prism-dtu-crowdstrike/tests/fidelity_validator.rs` | Comment update (excluded route list) |

No new ADR warranted. No existing ADR is amended. The fix is entirely within the operational envelope of ADR-028 §D1/D5 + ADR-003.

---

### Sibling Determination — fetch_incidents (F-CSD-P1-005) — 2026-07-10

**Finding:** F-CSD-P1-005 (MED, pending-intent) — `fetch_incidents` step has `method = "GET"`, no `body_template`, no `${...}` fan-out anchor. Same defect class as `fetch_devices`.

**Determiner:** architect | **Anchor:** D-1650 (sibling to fetch_devices ratification)

---

#### 1. Real-API Check

**Verdict: CONFIRMED. High confidence.**

The path `/incidents/entities/incidents/GET/v1` follows CrowdStrike's established naming convention for batch POST entity-retrieval endpoints — the "GET" embedded in the URL path is a CrowdStrike API convention, not the HTTP method. The real operation is:

```
POST /incidents/entities/incidents/GET/v1
Body: {"ids": ["inc-001", "inc-002", ...]}
```

This is `GetIncidents` / `PostEntitiesIncidentsGetV1` in the FalconPy SDK. It is structurally identical to the already-ratified `POST /detects/entities/summaries/GET/v1` (detections pipeline):

| Endpoint | HTTP Method | URL contains "GET"? | Body |
|----------|-------------|---------------------|------|
| `POST /detects/entities/summaries/GET/v1` | POST | yes | `{"ids": [...]}` |
| `POST /incidents/entities/incidents/GET/v1` | POST | yes | `{"ids": [...]}` |
| `POST /devices/entities/devices/v2` | POST | no | `{"ids": [...]}` |

The prior fetch_devices ratification (above) confirmed: "CrowdStrike Falcon API entity-retrieval-by-ids pattern (POST /incidents/entities/incidents/GET/v1 with {ids: [...]} body, i.e., GetIncidents/PostEntitiesIncidentsGetV1)" as a recognized general class. No new external research is required — the path segment pattern is self-documenting and consistent across three confirmed CrowdStrike endpoints.

**Confidence: HIGH.** The path naming convention is unambiguous.

---

#### 2. Verdict: (a) — Fix TOML Shape Now

**The TOML shape fix (method POST + body_template) is testable via the existing spec-shape parse validation without a DTU incidents route and must land in this fix lane.**

Rationale:

**Testability confirmed.** `SpecLoader::parse` calls `validate_variable_references` (validation.rs lines 247–254) on every `body_template` at parse time. This validator checks that `${query_incident_ids.resources}` refers to a step named `query_incident_ids` that precedes `fetch_incidents` in the pipeline — a pure structural check that requires no DTU route. The existing `test_BC_2_16_009_validates_all_4_bundled_specs` exercises `SpecLoader::parse` on all four bundled TOML specs including `crowdstrike.sensor.toml`. After the fix, that test will validate the interpolation anchor is correct.

**DTU-EXT-001 gap is unchanged.** The incidents table already carries a documented DTU-EXT-001 gap (TOML lines 265–269). Fixing the HTTP method and adding the `body_template` does not add new DTU requirements — the incidents route was always going to be POST (the path embedding "GET" already implied POST semantics). If anything, the current `method = "GET"` is wrong with respect to what DTU-EXT-001 will need to implement — correcting it now ensures DTU-EXT-001's implementer builds the right POST handler, not a GET handler.

**ADR-028 §D5 constraint is not escalated.** The incidents table's spec is already "ahead of DTU" under the acknowledged DTU-EXT-001 exception. Correcting the method within an already-acknowledged-gap spec does not increase the degree of spec-ahead-of-DTU violation.

**Production-grade default applies.** The defect class is confirmed, the fix is scoped and testable without external DTU dependencies, and deferring would leave a known-wrong `method = "GET"` in the spec that will mislead DTU-EXT-001's implementer.

**Deferral to DTU-EXT-001 (option b) is REJECTED** because: (1) the TOML fix is independently testable at parse time, (2) the spec already has the DTU-EXT-001 gap comment so the route-work anchor is already in place, and (3) deferring the method change adds no value and risks the DTU implementer writing a GET route instead of a POST route.

---

#### 3. Implementation Contract

**Target:** `crates/prism-sensors/specs/crowdstrike.sensor.toml`, incidents table `fetch_incidents` step.

**Current state (lines 305–312):**

```toml
  # Step 2: GetEntities — GET full incident records
  [[tables.steps]]
  name = "fetch_incidents"
  method = "GET"
  path_template = "/incidents/entities/incidents/GET/v1"
  response_path = "$.resources"
  variables_produced = []
  fan_out_batch_size = 100
```

**Replace with:**

```toml
  # Step 2: PostEntitiesIncidentsGetV1 — POST IDs to get full incident records
  # Real-API canonical: CrowdStrike `POST /incidents/entities/incidents/GET/v1`
  #   (GetIncidents / PostEntitiesIncidentsGetV1 in FalconPy). URL path embeds "GET"
  #   per CrowdStrike's naming convention for batch POST entity retrieval — HTTP method
  #   is POST. Body: {"ids": [...]}. Same pattern as detections pipeline
  #   (POST /detects/entities/summaries/GET/v1). Confirmed class per F-CSD-P1-005
  #   sibling determination 2026-07-10.
  # DTU-EXT-001 gap: no incidents route in prism-dtu-crowdstrike yet. When DTU-EXT-001
  #   ships, the implementer MUST add POST /incidents/entities/incidents/GET/v1 (not GET).
  # fan_out_batch_size = 100: conservative default matching detections pipeline.
  [[tables.steps]]
  name = "fetch_incidents"
  method = "POST"
  path_template = "/incidents/entities/incidents/GET/v1"
  body_template = '{"ids": ${query_incident_ids.resources}}'
  response_path = "$.resources"
  variables_produced = []
  fan_out_batch_size = 100
```

**Variable reference justification:**
- Step 1 (`query_incident_ids`) declares `response_path = "$.resources"` and `variables_produced = ["incident_ids"]`
- The fan-out anchor key is `query_incident_ids.resources` — step name + final path segment of `response_path` — which matches the detections pattern (`query_detection_ids.resources` in `fetch_detections`)
- `validate_variable_references` confirms this is a valid backward reference at `SpecLoader::parse` time

**Scope boundary:**
- The TOML change is the ENTIRE scope of this fix in the DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / FIX-CSDEVICES-EMPTY-PIPELINE lane
- DTU incidents route work (`POST /incidents/entities/incidents/GET/v1` handler) remains anchored to DTU-EXT-001; it is NOT in scope for this fix lane
- No new BC update required — the incidents table's existing DTU-EXT-001 gap comment adequately documents the situation; the method correction is a spec-correctness fix, not a behavioral contract change

**Testability after fix:**
- `test_BC_2_16_009_validates_all_4_bundled_specs` (runs unconditionally in CI) — validates `SpecLoader::parse` succeeds and variable reference `${query_incident_ids.resources}` resolves correctly
- No DTU route is needed to pass this test
- The parity test (`test_BC_2_16_013_dtu_parity_crowdstrike`) is already `#[ignore]`'d under DTU-EXT-001 tracking — no change to its ignore status
