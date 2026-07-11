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

---

### Adjudication — Expr::InSubquery projection-position execution (F-CSD-P4-001) — 2026-07-10

**Adjudicator:** architect | **Anchor:** D-1650 | **Finding:** F-CSD-P4-001 (HIGH) raised by LOCAL adversary pass-4

---

#### Background

The implementer, while fixing DEFECT-CSDEVICES-EMPTY-PIPELINE-001 (empty-MemTable pre-registration for Track B), also wrote RED-gate test T2 asserting that a SELECT projection-position `Expr::InSubquery` against a 0-batch table returns `Ok(3 rows, all-false is_known column)`. To make T2 green, they rewrote `PqlNormalizer::normalize_expr` `Expr::InSubquery` arm (ast.rs ~2343–2371) to emit a correlated COUNT subquery:

```
col IN (SQ)  →  (SELECT COUNT(*) FROM (SQ) AS __prism_sq__ WHERE __prism_sq__.<col> = <field>) > 0
```

along with a helper `extract_insubquery_first_col`. The adversary flagged this as an unauthorized semantic rewrite with three issues: (1) NULL semantics divergence, (2) no spec authorization, (3) multi-column arity (F-CSD-P4-002).

---

#### Factual Findings

**1. WHERE-position `Predicate::InSubquery` is unaffected by the rewrite.**

`normalize_predicate` (ast.rs ~2208–2215) handles `Predicate::InSubquery` via a separate arm that passes through unchanged: `format!("{} {not_kw} ({sub})", ...)`. DataFusion's `decorrelate_predicate_subquery` optimizer handles WHERE/HAVING IN-subqueries natively with correct three-valued SQL semantics. The adversary's NULL concern applies exclusively to the `normalize_expr` `Expr::InSubquery` arm (the COUNT rewrite).

**2. NULL semantics divergence is real and analyst-observable.**

Standard SQL three-valued logic for `x IN (S)`:
- Returns `NULL` when `x IS NULL`
- Returns `NULL` when `x ∉ S` AND `S` contains at least one `NULL`
- Returns `TRUE` when `x ∈ S`
- Returns `FALSE` when `x ∉ S` AND `S` contains no `NULL`

The COUNT rewrite always returns `TRUE` or `FALSE`, collapsing the NULL case to `FALSE`. OCSF spec columns are nullable=true (all sensor columns). Queries like `WHERE (hostname IN (SELECT hostname FROM cs_devices)) IS NULL` or `SELECT COUNT(flag_col) FROM t WHERE flag_col IN (SELECT ...)` would produce wrong answers. The agent-harness design goal (project memory `project_agent_harness_design.md`) requires standard SQL semantics for LLM analyst consumption. Option C (documented divergence) fails this goal.

**3. The rewrite was scope expansion beyond DEFECT scope.**

DEFECT-CSDEVICES-EMPTY-PIPELINE-001 Track B required empty-MemTable pre-registration to prevent silent "Internal error" crashes in JOIN positions. The projection-position `Expr::InSubquery` execution capability was not in the root-cause analysis (see §Symptom 2 and §Fix Work Plan Track B above). T2's "DESIRED: Ok(3 rows, all-false)" was written by the implementer to encode the COUNT-rewrite behavior, not derived from a BC requirement.

**4. The F-EQ42-P2-001 tests on develop (PR #220) are preserved under Option A.**

These tests assert `E-QUERY-042` for `SELECT hostname IN (SELECT hostname FROM test_events GROUP BY '<rfc3339>') FROM test_events`. The temporal walker's `check_select_items_raw_temporal` → `check_expr_temporal_pos Expr::InSubquery` arm fires E-QUERY-042 for the subquery's RFC-3339 timestamp in GROUP BY. The temporal walker runs BEFORE normalization and BEFORE any plan-time rejection gate. Because the temporal violation returns `Err(E-QUERY-042)` immediately, the projection-position gate (which runs after temporal checks) is never reached. These tests continue to pass unchanged under Option A.

**5. E-QUERY-043 is the next available code.**

Error taxonomy (`.factory/specs/prd-supplements/error-taxonomy.md`) contains codes through E-QUERY-042. E-QUERY-043 is unallocated.

**6. BC-2.11.003 §Invariants contains a stale claim.**

Line 58: "Subqueries are not supported in v1; nested `SELECT` in `WHERE` or `FROM` returns a parse error with explanation." This is incorrect as of develop@b9cf3f9b (PR #220): `Predicate::InSubquery` (WHERE IN-subquery) parses and EXECUTES successfully via DataFusion. The Error Cases table (line 86) also has a stale `E-QUERY-001 | Subquery detected` row. Both must be reconciled.

---

#### Ruling: Option A — Plan-Time Structured Rejection (E-QUERY-043)

**Decision:** ADOPT Option A. The COUNT rewrite is REJECTED. Projection-position `Expr::InSubquery` returns a new structured E-QUERY-043 error with a clear message directing analysts to the WHERE-clause alternative.

**Justification:**

- The production-grade default (CLAUDE.md) applies: NULL semantics divergence is a correctness defect, not a style preference. LLM analysts relying on standard SQL semantics will get wrong answers on nullable OCSF fields.
- Option B (NULL-semantics-preserving rewrite) requires story-level design: identifying a DataFusion-plannable construct that preserves three-valued logic for `x IN (subquery)` with nullable fields. That is new feature work, not a defect fix. Feature order is the acceptable speed lever.
- Option C (documented divergence) fails the agent-harness design goal.
- A clear E-QUERY-043 error is strictly better than the original silent "Internal error" / `QueryExecutionFailed` — it explains what is unsupported and how to fix the query. The DEFECT goal ("stop crashing silently") is satisfied.
- F-CSD-P4-002 (multi-column arity) is resolved by this ruling: no projection-position IN-subquery execution means the arity concern is moot. No separate fix needed.
- F-EQ42-P2-001 tests are unaffected (temporal check fires before projection gate).
- No new ADR is required: this is a structured-rejection addition within the existing query-gate architecture, not a new cross-cutting semantic decision.

---

#### Implementation Contract

The following contract is binding for the implementer executing this ruling in the `fix/csdevices-empty-pipeline` worktree.

**Contract Item 1 — Revert `normalize_expr` `Expr::InSubquery` arm** (`crates/prism-query/src/ast.rs`)

Revert to develop form (ast.rs ~2343):

```rust
Expr::InSubquery { field, subquery } => {
    let sub = Self::normalize_sql_query(subquery);
    format!("{} IN ({sub})", Self::normalize_field_path(field))
}
```

Remove the entire COUNT-rewrite block (current lines ~2344–2370) including the comment block.

**Contract Item 2 — Delete `extract_insubquery_first_col`** (`crates/prism-query/src/ast.rs`)

Delete the function `extract_insubquery_first_col` (~lines 2452–2467) and its doc comment (~lines 2437–2451). It is no longer needed.

**Contract Item 3 — Add plan-time projection gate** (location: `crates/prism-query/src/materialization.rs` or the engine module that runs temporal checks, **after** `check_temporal_literals` returns `Ok`)

Add a function `check_expr_insubquery_projection(query: &PqlQuery) -> Result<(), PrismError>` that:
1. Walks `query.select.items` — for each `SelectItem::Expr { expr, .. }`, checks whether `expr` contains (directly or recursively) an `Expr::InSubquery { .. }`.
2. If found, returns:
   ```rust
   Err(PrismError::ExprInSubqueryProjectionNotSupported {
       hint: "IN subquery in SELECT projection position is not currently supported. \
              Use a WHERE clause subquery instead: \
              `WHERE field IN (SELECT ...)`.".to_string()
   })
   ```
3. Also walks GROUP BY and ORDER BY expressions for `Expr::InSubquery` and returns the same error (those positions are also unsupported for the same DataFusion reason).
4. Does NOT walk `Predicate::InSubquery` — WHERE/HAVING IN-subqueries are supported.
5. The gate must be called AFTER `check_temporal_literals` (and any other plan-time validation) returns `Ok`, so that temporal errors take precedence (preserves F-EQ42-P2-001 tests).

**Contract Item 4 — Add `ExprInSubqueryProjectionNotSupported` to `PrismError`** (`crates/prism-core/src/error.rs`)

Add the variant to the `PrismError` enum:

```rust
/// E-QUERY-043: `field IN (SELECT ...)` in SELECT projection, GROUP BY, or ORDER BY
/// position. DataFusion 53.1.0 physical planner cannot execute `InSubquery` in
/// scalar expression positions. Use `WHERE field IN (SELECT ...)` instead.
ExprInSubqueryProjectionNotSupported {
    hint: String,
},
```

In the `Display` impl for `PrismError`, add:

```rust
PrismError::ExprInSubqueryProjectionNotSupported { hint } => {
    write!(f, "E-QUERY-043: IN subquery in projection position is not supported. {hint}")
}
```

In the MCP error-mapping function (wherever `map_prism_error` or equivalent is defined), map `ExprInSubqueryProjectionNotSupported` to MCP code `-32602` (INVALID_PARAMS, caller-resolvable) — consistent with the E-QUERY-041/042 mapping pattern.

**Contract Item 5 — Update test T2** (`crates/prism-query/src/tests/defect_csdevices_empty_memtable_tests.rs`)

Test `test_BC_2_11_005_F_CSD_P3_001_T2_expr_insubquery_projection_empty_table_returns_false_col_not_error`:

1. Rename to `test_BC_2_11_005_F_CSD_P3_001_T2_expr_insubquery_projection_returns_e_query_043_not_internal_error`.
2. Change the assertion from:
   ```rust
   assert!(result.is_ok(), "F-CSD-P3-001-T2 / BC-2.11.005: ...")
   ```
   to:
   ```rust
   assert!(
       matches!(
           &result,
           Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
       ),
       "F-CSD-P3-001-T2 / BC-2.11.005: Projection-position IN-subquery must return \
        E-QUERY-043 (ExprInSubqueryProjectionNotSupported), not an internal plan error. \
        The COUNT-rewrite was REJECTED by architect adjudication 2026-07-10 due to NULL \
        semantics divergence (nullable OCSF fields, agent-harness SQL semantics goal). \
        Use WHERE clause subquery form for equivalent filtering. got: {result:?}"
   );
   ```
3. Remove the `total_rows` assertion (since the result is now `Err`, no batches to inspect).
4. Update the test doc comment to reflect the Option A ruling.

**Contract Item 6 — Track B `pre_register_empty_tables` scope clarification**

The Track B fix (extending `pre_register_empty_tables` to recurse into `Predicate::InSubquery` FROM positions) covers:
- WHERE/HAVING `Predicate::InSubquery`: walk the subquery's `from` and nested `Predicate::InSubquery` predicates recursively (fixes T1, T3, T4).
- `Expr::InSubquery` in SELECT items: **NOT required** under Option A. The projection gate returns E-QUERY-043 before DataFusion plans the query, so the table does not need to be pre-registered for projection-position cases. Do NOT add `Expr::InSubquery` SELECT item walking to `pre_register_empty_tables` — it is dead code under Option A and would confuse future maintainers.

---

#### Spec Amendment Contract (Product-Owner work, not implementer)

**BC-2.11.003 `prismql-sql-mode.md` amendments required** (deliver as a single version bump after implementer closes the code):

1. **§Invariants — replace stale subquery claim** (line 58):

   Remove: `"Subqueries are not supported in v1; nested SELECT in WHERE or FROM returns a parse error with explanation"`

   Replace with:
   ```
   - `Predicate::InSubquery` (WHERE/HAVING IN-subquery, e.g. `WHERE field IN (SELECT ...)`) is
     supported — DataFusion's `decorrelate_predicate_subquery` optimizer handles these natively
     with standard SQL three-valued semantics.
   - `Expr::InSubquery` (SELECT projection/GROUP BY/ORDER BY IN-subquery, e.g.
     `SELECT (field IN (SELECT ...)) AS alias FROM t`) is not currently supported —
     returns E-QUERY-043 with a message directing the analyst to the WHERE-clause form.
   - The parser accepts both forms (parse succeeds); rejection is plan-time only.
   ```

2. **§Error Cases — update stale `E-QUERY-001 | Subquery detected` row** (line 86):

   Remove the row: `E-QUERY-001 | Subquery detected | Error: "Subqueries are not supported. Use pipe mode for multi-stage operations."`

   Add new row for E-QUERY-043:
   ```
   | `E-QUERY-043` | `field IN (SELECT ...)` appears in SELECT projection, GROUP BY, or ORDER BY
     position (DataFusion 53.1.0 physical planner cannot execute `InSubquery` in scalar expression
     positions). WHERE/HAVING IN-subqueries are unaffected. |
     `"E-QUERY-043: IN subquery in projection position is not supported.
       Use a WHERE clause subquery instead: WHERE field IN (SELECT ...)."` |
   ```

3. **Frontmatter**: bump `version` to `"1.13"`, update `modified` to `2026-07-10`.

**error-taxonomy.md amendment required:**

Add E-QUERY-043 row to the error taxonomy. Pattern: `E-QUERY-043 | plan-time | query | "E-QUERY-043: IN subquery in projection position is not supported. {hint}" | Yes (caller-resolvable: rewrite as WHERE clause subquery) | ...`.

---

#### Resolution of F-CSD-P4-002 (MED — multi-column arity)

F-CSD-P4-002 identified that `extract_insubquery_first_col` silently uses the first column for `IN (SELECT a, b ...)`. Under Option A, `extract_insubquery_first_col` is deleted and the COUNT rewrite is removed. No projection-position IN-subquery executes. F-CSD-P4-002 is **resolved by ruling** — no separate fix required. Document closure in the adversary cascade record.

---

#### Future Story Authorization

A future story titled "Projection-Position IN-Subquery Execution with Correct Three-Valued SQL NULL Semantics" may implement full execution support when:

1. A DataFusion-plannable equivalent to `x IN (subquery)` with three-valued logic for nullable `x` and subquery containing NULLs is identified and verified against DataFusion 53.x API.
2. An ADR is authored covering the chosen rewrite strategy, NULL semantics preservation proof, and any divergence-from-standard-SQL that is intentionally accepted.
3. BC-2.11.003 is amended to describe the supported form.
4. E-QUERY-043 is removed from the error taxonomy (or scoped to cases that remain unsupported).

Until that story is delivered and merged, E-QUERY-043 is the correct and final behavior for projection-position `Expr::InSubquery`.

---

## Adjudication — E-QUERY-038 second emission source (F-CSD-P20-003) — 2026-07-10

**Finding:** F-CSD-P20-003 (CRITICAL) from LOCAL adversary pass 20 on branch `fix/csdevices-empty-pipeline`, frozen HEAD `7347bb16`.

**Adjudicator:** architect | **Anchor:** pass-20 cascade | **Date:** 2026-07-10

---

### Call-Graph Evidence

The production query path is:

```
MCP handler
  → QueryEngine::execute(query_str, options)     [engine.rs ~L:745]
    → execute_inner(query_str, options)           [engine.rs ~L:762]
      → check_query_column_availability(...)      [engine.rs ~L:879]   ← E-QUERY-038 plan-time gate
      → run_materialization_pipeline(...)         [engine.rs ~L:1034]
        → execute_against_session_with_registry(...)  [materialization.rs ~L:1093]
          → session_ctx.sql(plan_pinned_sql).await    ← DataFusion execution
```

Key observations confirmed by code reading:

1. `execute_inner` (engine.rs L:879) calls `check_query_column_availability` BEFORE calling `run_materialization_pipeline`. If `check_query_column_availability` returns `Err(ColumnNotFound)`, `execute_inner` returns immediately — `run_materialization_pipeline` is never reached.

2. `execute_against_session` is `pub` solely for test access. Its doc comment (materialization.rs L:1118–1122) explicitly states: "Production callers use `run_materialization_pipeline`." A global grep of all non-test `*.rs` files confirms zero production call sites outside `materialization.rs` itself.

3. `execute_against_session_with_registry` is `pub(crate)` and called from one production site only: inside `run_materialization_pipeline` at L:1093 — after the plan-time gate has already passed.

4. `check_query_column_availability` fails-open ONLY when BOTH `resolved_spec_map` AND `table_registry` are `None`. In a production deployment the spec engine always wires `table_registry` (ADR-022). Queries where the table registry is `None` are test-mode legacy paths, not production.

5. The runtime fallback in `execute_against_session_with_registry` at L:1243–1290 (the `session_ctx.sql(...).await.map_err(|e| { ... FieldNotFound → ColumnNotFound ... })` block) can therefore ONLY fire in production for **internal schema anomalies** (pre-registration schema gaps, DataFusion optimizer surprises) — NOT for user-typed unknown columns. User-typo columns are intercepted by the plan-time gate at step 1, before DataFusion is ever invoked.

6. T38 (`test_BC_2_11_012_F_CSD_P19_003_T38_safety_flags_returns_e_query_038_not_datafusion_plan_error`) calls `execute_against_session` directly with a manually constructed `SessionContext` — BYPASSING `execute_inner` and its plan-time gate. It relies on the runtime fallback. The `VirtualField::SafetyFlags` retirement (confirmed in ast.rs L:933–950) is already complete on this branch: `_safety_flags` now parses as `Expr::Field`. The plan-time gate WOULD fire for `_safety_flags` in the production path. T38's direct `execute_against_session` call is the wrong test path.

---

### Decision: Option A — Remove the runtime fallback entirely

The runtime FieldNotFound → ColumnNotFound fallback in `execute_against_session_with_registry` (`Ast::Sql(Select)` arm) must be removed.

**Rationale:**

1. **The fallback is unreachable for user-typo columns in production.** The plan-time gate in `execute_inner` always fires first. When the plan-time gate returns `Err(ColumnNotFound)`, `run_materialization_pipeline` is never called; DataFusion is never executed; the runtime fallback is never reached.

2. **The fallback misclassifies internal schema anomalies as user errors.** If DataFusion returns `FieldNotFound` in production (because the runtime fallback IS reached after the plan-time gate passed), that is an internal schema anomaly — a pre-registration gap, a column-type mismatch in MemTable construction, or a DataFusion optimizer artifact. Converting that to `PrismError::ColumnNotFound` (E-QUERY-038) tells the LLM caller "you typed the wrong column name," which is factually incorrect and defeats the LLM self-correction guarantee (BC-2.11.016 §Design Intent). The correct error for internal schema anomalies is `PrismError::QueryExecutionFailed` with structured logging so the operator can investigate.

3. **BC-2.11.016 defines E-QUERY-038 as exclusively plan-time.** The spec documents three emission sites (single-tenant registry path, multi-tenant spec-map path, binding-context suspension arm). All three are in `check_query_column_availability` / `check_column_availability` in engine.rs. Adding a fourth emission site in the execution layer (materialization.rs) contradicts the specification's design intent and would require a BC amendment — which would be the wrong amendment to make.

4. **T38's test path is incorrect.** T38 bypasses the plan-time gate, tests the runtime fallback, and asserts `ColumnNotFound`. After the `VirtualField::SafetyFlags` retirement, the correct behavior (E-QUERY-038 for `_safety_flags`) is already implemented in the plan-time gate. T38 must be re-pointed to test the gate it actually covers.

**Rejected alternatives:**

- **Option B (spec-compliant runtime fallback):** Would require BC-2.11.016 v1.26 amendment to add a fourth emission site, plus threading `client_id`, `resolved_spec_map`, `org_scope`, and `infusion_registry` through `execute_against_session_with_registry` (none currently present in the function signature) to achieve full payload parity (Levenshtein, sorted/deduped org-scoped columns, correct JOIN table attribution, audit event). All that complexity for a gate that legitimately fires only for internal schema anomalies — and then misrepresents them as user errors. Net effect: more complex, more tests to maintain, wrong semantics.

- **Option C (gate reuse inside `execute_against_session_with_registry`):** Requires `resolved_spec_map`, `options.clients`, `infusion_registry` — none available at the `execute_against_session_with_registry` call site. Moving or duplicating the full gate inside the execution layer would dissolve the plan-time / execution-time separation that BC-2.11.016 is built on.

---

### Per-Agent Directives

#### Product-Owner

No BC-2.11.016 amendment is required. The three existing emission sites (single-tenant registry path, multi-tenant spec-map path, binding-context suspension arm) remain the authoritative and complete list.

Optional (documentation-only, non-blocking): add a `§Design Constraint` note to BC-2.11.016 stating: "E-QUERY-038 is exclusively a plan-time gate. `DataFusionError::SchemaError::FieldNotFound` arising after DataFusion execution has begun is an internal schema anomaly (E-QUERY internal execution error), NOT an E-QUERY-038 condition. Converting runtime FieldNotFound to ColumnNotFound is prohibited." If added, bump the frontmatter version to `v1.26`. This is a documentation clarification, not a behavioral change.

No BC-2.11.012 amendment is required. BC-2.11.012's contract ("queries referencing `_safety_flags` return E-QUERY-038") remains valid — the mechanism by which it is achieved is the plan-time gate, which is the correct mechanism. The test vector T38 is re-pointed in the implementation layer; no spec change is needed.

#### Implementer

**Change 1 — Remove the runtime FieldNotFound → ColumnNotFound fallback:**

File: `crates/prism-query/src/materialization.rs`
Location: `execute_against_session_with_registry`, `Ast::Sql(SqlStatement::Select(sql_query))` arm, at the `session_ctx.sql(&plan_pinned_sql).await.map_err(...)` call.

Remove the entire runtime fallback block that:
- Imports `use datafusion::common::{DataFusionError as DFError, SchemaError};`
- Calls `e.find_root()` and matches `DFError::SchemaError` → `SchemaError::FieldNotFound`
- Constructs `PrismError::ColumnNotFound(Box::new(prism_core::error::ColumnNotFoundDetails::new(col, sql_query.from.source.raw.clone(), "", avail, None)))`

Replace the entire `session_ctx.sql(...).await.map_err(|e| { ... [fallback block] ...})?` with:

```rust
let df = session_ctx.sql(&plan_pinned_sql).await.map_err(|e| {
    tracing::error!(error = %e, "DataFusion SQL planning error");
    PrismError::QueryExecutionFailed {
        detail: "SQL planning error: <redacted; see server logs>".to_string(),
    }
})?;
```

This is the existing non-FieldNotFound error branch — simply remove the FieldNotFound → ColumnNotFound transformation so all DataFusion planning errors consistently surface as `QueryExecutionFailed` with structured logging.

Remove the dead `F-CSD-P19-003` comment block at the removal site. Update the surviving inline comment to note: "DataFusion planning errors at this point are internal schema anomalies (plan-time column gate in execute_inner already handled user-typo columns); surface as QueryExecutionFailed for operator diagnostics."

**Change 2 — Make `check_query_column_availability` pub(crate):**

File: `crates/prism-query/src/engine.rs`
Location: line `fn check_query_column_availability(` (currently bare `fn`, private to module).

Change `fn check_query_column_availability(` to `pub(crate) fn check_query_column_availability(`.

This is required for T38 to call the plan-time gate directly from the test module (`src/tests/defect_csdevices_empty_memtable_tests.rs`).

#### Test-Writer

**Re-point T38** (`test_BC_2_11_012_F_CSD_P19_003_T38_safety_flags_returns_e_query_038_not_datafusion_plan_error` in `crates/prism-query/src/tests/defect_csdevices_empty_memtable_tests.rs`):

The test must be rewritten to call the plan-time gate directly, not `execute_against_session`. Approach:

1. Add `crate::engine::check_query_column_availability` to the test's import list.
2. Build a `crate::table_registry::TableRegistry::new()` and register a minimal CrowdStrike devices sensor spec into it using `registry.register_sensor(&spec)`. The spec needs `sensor_id = "crowdstrike"` and a single table with `table_name = "devices"` and columns: `device_id`, `hostname`, `platform_name`, `status`, `first_seen`, `last_seen` (the canonical six columns). This produces the fully-qualified table `crowdstrike_devices` with a real column list.
3. Call `check_query_column_availability("SELECT _safety_flags FROM crowdstrike_devices", "", None, None, Some(&registry), None)` — no `resolved_spec_map`, no `infusion_registry`, no org scope. This exercises the single-tenant registry fallback path.
4. Assert `Err(PrismError::ColumnNotFound(details))` with:
   - `details.column == "_safety_flags"`
   - `details.table == "crowdstrike_devices"`
   - `details.available_columns` contains the six registered spec columns (sorted: `["device_id", "first_seen", "hostname", "last_seen", "platform_name", "status"]`)
   - `details.client_id == ""` (correct: no explicit client scope, `client_id` defaults to empty string per BC-2.11.016 single-tenant path)
   - `details.did_you_mean == None` (Levenshtein distance from `"_safety_flags"` to each of the six spec columns exceeds 3)
5. Update the test doc comment to reflect: "Re-pointed from `execute_against_session` (runtime fallback) to `check_query_column_availability` (plan-time gate). The `VirtualField::SafetyFlags` retirement (ast.rs L:933–950) means `_safety_flags` now parses as `Expr::Field`; the plan-time gate catches it before DataFusion execution."
6. Remove the `execute_against_session` import from T38's import block if it is no longer needed by any other test in the same file. (Do not remove it if other tests in the file still use it.)

**No new tests required:** The plan-time gate `check_query_column_availability` is already thoroughly tested by the existing BC-2.11.016 test suite. T38 re-pointing closes the specific contract from BC-2.11.012 v1.7 without duplicating that coverage.

---

### Finding Disposition Table

| Finding | Status after Option A | Rationale |
|---------|----------------------|-----------|
| **F-CSD-P20-001** (hardcoded `client_id=""` in runtime fallback) | MOOTED | Runtime fallback removed. In the plan-time gate, `client_id` is correctly derived from `options.clients.first()`. For no-scope queries the empty string is correct per BC-2.11.016. |
| **F-CSD-P20-002** (no `column_not_found.rejected` audit event in runtime fallback) | MOOTED | Runtime fallback removed. The plan-time gate in `check_column_availability` already emits `column_not_found.rejected` per BC-2.11.016 emission-3. No second emission site needed. |
| **F-CSD-P20-003** (this finding — core design question) | RESOLVED by Option A | See rationale above. |
| **F-CSD-P20-004** (T38 payload assertions test the runtime fallback payload, not plan-time payload) | RESOLVED by T38 re-pointing | After re-pointing, T38 asserts the plan-time gate payload, which is fully correct (real spec columns, Levenshtein, correct client_id). The adversary's concern about insufficient assertions is closed by the new assertions specified above. |
| **F-CSD-P20-013** (wrong table attribution under JOINs, unsorted/undeduped `available_columns`, converts ANY `FieldNotFound` including internal planner bugs, no org-scope) | MOOTED | All four sub-issues are properties of the runtime fallback. Fallback removed. The plan-time gate already handles all four correctly (correct table attribution from AST walk, sorted+deduped in `check_column_availability`, only fires for columns that fail the spec-schema check, org-scoped via `resolved_spec_map`). |
