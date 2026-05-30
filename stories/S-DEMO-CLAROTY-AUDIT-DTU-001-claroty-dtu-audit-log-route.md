---
document_type: story
story_id: S-DEMO-CLAROTY-AUDIT-DTU-001
title: "prism-dtu-claroty: Add /api/v1/audit_log/get route for Claroty Audit Log Fidelity (closes Gap-CL-006 / DTU=true-DTU)"
wave: 5
epic_id: E-DTU-FIDELITY
priority: P1
status: draft
# BC status: BC-2.01.013 and BC-2.16.013 are both active (BC-INDEX v5.56).
# S-7.01 gate: behavioral_contracts is non-empty; status may advance to ready once
# implementer confirms DTU route design is consistent with BCs.
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-29T00:00:00Z"
modified: "2026-05-29"
tdd_mode: strict
subsystems: [SS-17, SS-16]
# Subsystem anchor justifications:
#   SS-17 (DTU Clones) owns crates/prism-dtu-claroty; the new audit_log route,
#     ClarotyAuditLogEntry struct, and fixture file are all SS-17 work.
#   SS-16 (Spec Engine) is the consumer of the DTU route — DTU-parity verification
#     under BC-2.16.013 is a spec-engine concern. The spec itself (claroty.sensor.toml)
#     already references /api/v1/audit_log/get; this story makes the DTU serve it.
crates_touched: [prism-dtu-claroty]
target_module: prism-dtu-claroty
capabilities: [CAP-001, CAP-029]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait — the audit_log route is a sensor endpoint; it must be
                 # reachable by the spec-driven pipeline that BC-2.01.013 governs.
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — claroty.sensor.toml
                 # declares a /api/v1/audit_log/get step at Gap-CL-002 fix (develop@72baf413).
                 # This story closes the DTU-side half of the parity gap (Gap-CL-006).
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — parity verification exercises the full pipeline
            # including the audit_log table; VP-148 anchor story is PLUGIN-MIGRATION-001-D.
depends_on:
  - PLUGIN-MIGRATION-001-A  # Must merge first: PLUGIN-MIGRATION-001-A established the
                             # BehavioralClone trait surface and the ClarotyClone scaffolding
                             # that this story extends. PR #156 merged to develop@948a709f.
blocks:
  - S-DEMO-002  # S-DEMO-002 smoke test includes a `FROM claroty_audit_logs LIMIT 10` query;
                # that query returns 404 until this route lands, blocking AC-004 of S-DEMO-002.
# Dependency anchor justifications:
#   depends_on PLUGIN-MIGRATION-001-A: The ClarotyClone `build_router()` and `routes/` module
#   structure were established in PLUGIN-MIGRATION-001-A. Adding a new route requires that
#   scaffolding to be stable. PLUGIN-MIGRATION-001-A is already merged.
#   blocks S-DEMO-002: S-DEMO-002 AC-004 queries claroty_audit_logs; the DTU returns HTTP 404
#   until this route is present. This is a hard runtime dependency, not conceptual relatedness.
points: 5
# Points justification:
#   - Define ClarotyAuditLogEntry struct + GetAuditLogBody + GetAuditLogResponse in types.rs: ~1 pt
#   - Implement routes/audit_log.rs handler (load_fixture + bearer auth check + pagination): ~1 pt
#   - Register route in clone.rs build_router() + mod.rs pub mod: ~0.5 pts
#   - Create fixtures/audit-log.json (5-10 synthetic entries, no real customer data): ~0.5 pts
#   - Red Gate tests (2 unit + 1 integration): ~1.5 pts
#   - BC-2.16.002 catalog row for any new tracing emission: ~0.5 pts
#   Total: 5 points (~1 day of focused TDD work)
estimated_days: 1
risk: LOW
# Risk justification:
#   Pure DTU-only addition. No prism-side code changes needed (claroty.sensor.toml already
#   declares the route at develop@72baf413 per Gap-CL-002 fix). The route pattern is identical
#   to alerts.rs and devices.rs — load fixture, check bearer auth, return JSON envelope.
#   No existing routes are modified. Lowest possible risk class for a DTU story.
assumption_validations: []
risk_mitigations: []
---

# S-DEMO-CLAROTY-AUDIT-DTU-001: Add `/api/v1/audit_log/get` Route to prism-dtu-claroty

## Narrative

As a demo operator running the Prism MCP server against DTU clones,
I want the Claroty DTU to serve `POST /api/v1/audit_log/get` with synthetic audit log entries,
so that `FROM claroty_audit_logs LIMIT 10` returns non-empty data during the demo and the
pre-demo smoke test (S-DEMO-002 AC-004) passes.

## Background

`claroty.sensor.toml` (at `crates/prism-sensors/specs/claroty.sensor.toml`) declares:

```toml
[[tables.steps]]
name = "fetch_audit_logs"
method = "POST"
path_template = "/api/v1/audit_log/get"
response_path = "$.audit_log"
```

This was added in the Gap-CL-002 fix at develop@72baf413. The TOML spec is correct; the
DTU is not. `prism-dtu-claroty`'s `build_router()` has no handler for `/api/v1/audit_log/get`,
so the pipeline receives HTTP 404 when executing the `fetch_audit_logs` step.

This story adds the missing route to the DTU. No TOML changes are needed.

Gap-CL-006 is the architect's designation for this specific DTU gap (registered in
POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1 §3 Claroty section).

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | v1.7 | Audit log table is a sensor table; the spec-driven adapter (BC-2.01.013 postcondition) must dispatch to it via the shared pipeline |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | v1.17 | Gap-CL-006 is an open DTU-parity gap under BC-2.16.013; this story closes it by making the DTU serve the endpoint the TOML spec declares |

## Acceptance Criteria

### AC-001: DTU route registration (traces to BC-2.16.013 postcondition §1 DTU-Parity)
`POST /api/v1/audit_log/get` is registered in `ClarotyClone::build_router()`. A request
to this path returns HTTP 200 (not 404) when a valid `Authorization: Bearer` header is present.

### AC-002: Bearer auth enforcement (traces to BC-2.01.013 postcondition §2 auth enforcement)
A request to `POST /api/v1/audit_log/get` without a valid `Authorization: Bearer` header
returns HTTP 401 with `{"error": "missing or invalid bearer token", "code": 401}`, identical
to the pattern established in `routes/alerts.rs::check_bearer_auth`.

### AC-003: Response envelope shape matches TOML `response_path` (traces to BC-2.16.013 postcondition §2 fixture-parity)
The response body is `{"audit_log": [...], "total": N}` where `audit_log` is a JSON array
of `ClarotyAuditLogEntry` objects. The key `audit_log` matches `response_path = "$.audit_log"`
in `claroty.sensor.toml` — the spec-driven pipeline extracts this array without path error.

### AC-004: Synthetic fixture data — non-empty, no real PII (traces to BC-2.16.013 postcondition §3 synthetic-fixture-data / ADR-031 §D2 permitted-divergence #1)
`fixtures/audit-log.json` contains at least 5 synthetic audit log entries. Entries contain
no real customer data. Fields are plausible (action, actor, timestamp, resource, id fields
corresponding to `claroty.sensor.toml` audit_logs columns).

### AC-005: Column parity with TOML `audit_logs` table block (traces to BC-2.16.013 postcondition §2 DTU-TOML-column-parity)
Each `ClarotyAuditLogEntry` response struct field corresponds to a declared column in
`claroty.sensor.toml [[tables]]` block for `table_name = "audit_logs"`:
- `id` (string)
- `action` (string)
- `actor` (string)
- `timestamp` (datetime — ISO 8601 string)
- `resource` (string)

No extra undeclared fields in the response cause DTU-TOML schema divergence. (SAP-2 parity gate.)

### AC-006: Integration via spec-driven pipeline (traces to BC-2.01.013 postcondition §1)
A test that boots `ClarotyClone` against the spec-driven pipeline (or directly via `reqwest`)
issues `POST /api/v1/audit_log/get` and receives a non-empty `audit_log` array. The response
can be passed to `extract_at_path("$.audit_log")` and returns a non-empty `Value::Array`.
This test is the Red Gate for S-DEMO-002 AC-004.

## Red Gate Tests

| Test name | Test type | What it gates |
|-----------|-----------|---------------|
| `test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries` | Unit (against DTU HTTP) | AC-003 + AC-004: route serves fixture; envelope shape correct |
| `test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced` | Unit (against DTU HTTP) | AC-002: 401 on missing bearer |
| `test_BC_2_16_013_claroty_audit_logs_dtu_column_parity` | Unit (Rust struct validation) | AC-005: all 5 TOML columns present in ClarotyAuditLogEntry |

The integration test `FROM claroty_audit_logs LIMIT 10` (S-DEMO-002 AC-004) exercises AC-006
and is categorized `#[ignore]` pending full boot wiring (S-DEMO-001 + S-DEMO-002 are blocking
predecessors for the full pipeline integration test). A companion unit test in
`routes/audit_log.rs` using `reqwest` drives the DTU directly without `prism-bin start`.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `ClarotyAuditLogEntry` struct | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |
| `GetAuditLogBody` struct | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |
| `GetAuditLogResponse` struct | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |
| `list_audit_logs` handler | `crates/prism-dtu-claroty/src/routes/audit_log.rs` | Effectful (HTTP handler, loads fixture) |
| `build_router()` — route registration | `crates/prism-dtu-claroty/src/clone.rs` | Effectful (router mutation) |
| `routes/mod.rs` — module export | `crates/prism-dtu-claroty/src/routes/mod.rs` | Pure (module declaration) |
| `fixtures/audit-log.json` | `crates/prism-dtu-claroty/fixtures/` | Static data |

Architecture section references:
- `architecture/module-decomposition.md` §SS-17 DTU Clones (ClarotyClone structure)
- `architecture/dependency-graph.md` §Wave-5 DTU fidelity stories

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Missing `Authorization` header | HTTP 401 `{"error": "missing or invalid bearer token", "code": 401}` — matches `check_bearer_auth` pattern from `routes/devices.rs` |
| EC-002 | Empty `Authorization: Bearer ` (no token value) | HTTP 401 — same as EC-001; `check_bearer_auth` treats empty token as missing |
| EC-003 | Malformed request body (non-JSON) | HTTP 200 with full fixture (body is `Option<Json<GetAuditLogBody>>` — unrecognized body ignored, fixture returned regardless) |
| EC-004 | Body with unknown fields | HTTP 200 with full fixture — permissive deserialization per Claroty API EC-001 in `types.rs` doc comment |
| EC-005 | Route accessed with GET instead of POST | HTTP 405 Method Not Allowed (axum default for unmatched method on registered path) |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,000 |
| `crates/prism-dtu-claroty/src/clone.rs` (existing) | ~6,500 |
| `crates/prism-dtu-claroty/src/types.rs` (existing) | ~4,000 |
| `crates/prism-dtu-claroty/src/routes/alerts.rs` (pattern reference) | ~2,000 |
| `crates/prism-dtu-claroty/src/routes/devices.rs` (auth pattern) | ~4,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (column reference) | ~3,000 |
| BC files (2 BCs: BC-2.01.013, BC-2.16.013) | ~8,000 |
| New files to write (audit_log.rs + fixture + types additions) | ~3,000 |
| **Total estimate** | **~33,500 tokens** |

Well within 20-30% of a 200K context window (~40-60K token budget per story). No split needed.

## Tasks

- [ ] **Task 1: Define types** — Add `ClarotyAuditLogEntry`, `GetAuditLogBody`, and `GetAuditLogResponse`
  to `crates/prism-dtu-claroty/src/types.rs`. Fields must match `claroty.sensor.toml` audit_logs
  columns: `id` (String), `action` (String), `actor` (String), `timestamp` (String, ISO 8601),
  `resource` (String). `GetAuditLogBody` mirrors `GetAlertsBody` — `offset: Option<u32>`,
  `limit: Option<u32>`, permissive fields. Response: `{"audit_log": [...], "total": N}`.

- [ ] **Task 2: Create fixture** — Author `crates/prism-dtu-claroty/fixtures/audit-log.json`
  with at least 5 synthetic entries. Each entry: all 5 declared columns populated with
  plausible but synthetic data (no real customer data per AD-017 + ADR-031 §D2 permitted-divergence #1).
  Timestamps in ISO 8601 format (e.g., `"2026-01-15T10:23:45Z"`). IDs as strings.

- [ ] **Task 3: Implement route handler** — Create `crates/prism-dtu-claroty/src/routes/audit_log.rs`.
  Pattern: identical to `routes/alerts.rs` — call `check_bearer_auth`, load fixture via
  `prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "audit-log")`, return
  `json!({"audit_log": entries, "total": total_u32})`. Handler signature:
  `pub async fn list_audit_logs(State(_state): State<Arc<ClarotyState>>, headers: HeaderMap, _body: Option<Json<GetAuditLogBody>>) -> (StatusCode, Json<Value>)`.

- [ ] **Task 4: Register module** — Add `pub mod audit_log;` to
  `crates/prism-dtu-claroty/src/routes/mod.rs`.

- [ ] **Task 5: Register route** — Add to `ClarotyClone::build_router()` in `clone.rs`:
  `.route("/api/v1/audit_log/get", post(audit_log::list_audit_logs))`
  Update the `use crate::routes::{..., audit_log};` import accordingly.

- [ ] **Task 6: Red Gate tests** — Add unit tests to `routes/audit_log.rs` `#[cfg(test)] mod tests`:
  - `test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries` (start ClarotyClone, POST with bearer, assert 200 + non-empty `audit_log` array)
  - `test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced` (POST without bearer, assert 401)
  - `test_BC_2_16_013_claroty_audit_logs_dtu_column_parity` (deserialize fixture into `Vec<ClarotyAuditLogEntry>`, assert all 5 fields present)

- [ ] **Task 7: BC-2.16.002 catalog check** — If any `tracing::*!(event_type = ...)` emission is
  added to the route handler, add a corresponding row to BC-2.16.002 §Postconditions Structured
  Event Catalog in the same commit (SAP-1 discipline). The `alerts.rs` pattern does NOT emit
  a tracing event for normal responses — follow the same pattern (no event = no catalog update needed).

- [ ] **Task 8: Verify TOML parity (SAP-2)** — Before committing, run the SAP-2 probe manually:
  for each column in `claroty.sensor.toml` `[[tables]]` block for `audit_logs`, verify the
  field exists in `ClarotyAuditLogEntry`. Columns: id, action, actor, timestamp, resource.
  All 5 must map 1:1. TOML column in spec with no DTU struct field = P1 CRITICAL finding.

## Previous Story Intelligence

N/A — first story in the E-DTU-FIDELITY epic for the Claroty audit log table. Closely related
to S-DTU-CYBERINT-AUTH-FIDELITY-001 (same epic, same wave) for structural patterns:

1. **Route registration pattern:** Copy `routes/alerts.rs` exactly for the handler structure
   — `check_bearer_auth`, `load_fixture`, `expect()` with SAFETY comments, return JSON envelope.
   Do NOT use `?` propagation in fixture loading (convention: SAFETY allow + expect per
   prism-dtu-claroty existing convention).

2. **Fixture naming convention:** Fixture file name is the hyphenated table name
   (`audit-log.json`), loaded via `load_fixture(env!("CARGO_MANIFEST_DIR"), "audit-log")`.
   Confirm this matches how `prism_dtu_common::load_fixture` builds the path (appends
   `fixtures/<name>.json`).

3. **No `#[serde(deny_unknown_fields)]` on request body:** `GetAlertsBody` and `GetDevicesBody`
   do NOT use `deny_unknown_fields` — the Claroty API is permissive. `GetAuditLogBody` must
   follow the same permissive pattern.

4. **Pagination body fields in DTU:** `GetAlertsBody` has `offset: Option<u32>` and
   `limit: Option<u32>` fields even though the current pipeline sends these as URL params
   (Gap-CL-004). Include these fields in `GetAuditLogBody` so that when S-DEMO-CLAROTY-PAGINATION-001
   lands (POST-body pagination), the DTU already accepts them.

## Architecture Compliance Rules

From `architecture/module-decomposition.md` §SS-17 DTU Clones:

- DTU clones are test infrastructure, not production code. They live in `crates/prism-dtu-*`
  and are not part of the `prism-bin` binary in production mode.
- Every route handler must call `check_bearer_auth` before processing — enforced by the
  existing compile-fail gate pattern (WV1-04-AUTH-ENFORCED).
- Fixture data must be synthetic (no real customer data, AD-017).
- `DtuConfigureBody` uses `#[serde(deny_unknown_fields)]`; route request bodies do NOT.

From `architecture/dependency-graph.md`:

- `prism-dtu-claroty` depends on `prism-dtu-common` for `BehavioralClone`, `load_fixture`,
  `StubConfig`, `FailureMode`. No new dependencies should be added for this story.
- `prism-dtu-claroty` MUST NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query`
  (DTU clones are isolated test infrastructure per ADR-031 §D1).

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `axum` | per `Cargo.toml` workspace pin | Route handler, `State`, `Json`, `HeaderMap` |
| `serde` | per `Cargo.toml` workspace pin | `Deserialize`, `Serialize` on new types |
| `serde_json` | per `Cargo.toml` workspace pin | `json!` macro, `Value` |
| `tokio` | per `Cargo.toml` workspace pin | Async test runtime |
| `prism-dtu-common` | workspace path | `load_fixture`, `BehavioralClone`, `StubConfig` |

Do NOT add `reqwest` as a new direct dependency for tests — the existing test pattern
in `routes/alerts.rs` uses `reqwest` only if it is already present in `[dev-dependencies]`.
Check `crates/prism-dtu-claroty/Cargo.toml` before adding.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| CREATE | `crates/prism-dtu-claroty/src/routes/audit_log.rs` | New route handler module |
| MODIFY | `crates/prism-dtu-claroty/src/routes/mod.rs` | Add `pub mod audit_log;` |
| MODIFY | `crates/prism-dtu-claroty/src/types.rs` | Add `ClarotyAuditLogEntry`, `GetAuditLogBody`, `GetAuditLogResponse` |
| MODIFY | `crates/prism-dtu-claroty/src/clone.rs` | Add route + import in `build_router()` |
| CREATE | `crates/prism-dtu-claroty/fixtures/audit-log.json` | 5+ synthetic entries |

Files MUST NOT be modified:
- `crates/prism-sensors/specs/claroty.sensor.toml` (TOML already correct at develop@72baf413)
- Any file in `crates/prism-spec-engine/` (DTU-only story)
- Any file in `crates/prism-bin/` (DTU-only story)

## Forbidden Dependencies

`prism-dtu-claroty` MUST NOT gain a dependency on:
- `prism-spec-engine` (build MUST fail if this dep appears)
- `prism-sensors` (build MUST fail if this dep appears)
- `prism-query` (build MUST fail if this dep appears)

These are DTU isolation rules per ADR-031 §D1. The existing perimeter-violation compile-fail
gate pattern at `tests/external/perimeter-violation/` is the template for enforcement if a
new gate is warranted.

## Notes for Implementer

1. **No prism-side changes.** `claroty.sensor.toml` already has the correct `path_template`,
   `method`, and `response_path`. This story is DTU-only.

2. **SAP-2 self-check before commit.** Run the SAP-2 probe on your own before submitting:
   for each column in the audit_logs TOML block (id, action, actor, timestamp, resource),
   verify a matching field in `ClarotyAuditLogEntry`. Missing field = P1 CRITICAL that blocks
   merge.

3. **Gap-CL-006 closure.** The story closes Gap-CL-006 from POLLER-DTU-FIDELITY-AUDIT-2026-05-29
   v1.1 §3 Claroty section. Reference the gap ID in the commit message and Red Gate test doc
   comments.

4. **`ClarotyAuditLogEntry.id` type.** Use `String` (not `u32`) — the TOML spec declares
   `column_type = "string"` for `id`. This is consistent with the EC-016-013-004 polymorphic
   ID handling for alerts.

5. **Fixture timestamp format.** Use ISO 8601 with `Z` suffix (e.g. `"2026-01-15T10:23:45Z"`).
   The TOML column declares `column_type = "datetime"` — the pipeline's timestamp normalization
   (ADR-028 §D8) will parse this format.
