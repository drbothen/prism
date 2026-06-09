---
document_type: story
story_id: S-DEMO-HARNESS-CLONE-PARITY-001
title: "prism-dtu-harness: Bring In-Process Armis + Claroty Clones to Route Parity with Standalone DTUs (closes F-P6-DEFER-001 / F-P10-LOW-001)"
wave: wave-5-e-demo-fidelity
epic_id: E-DTU-FIDELITY
priority: P2
status: ready
# BC-2.16.013 v1.25 authored by PO (D-989 Phase-A burst) with INV-HARNESS-ROUTE-PARITY invariant.
# S-7.01 gate CLEARED.
version: "1.6"
level: "L3"
producer: story-writer
timestamp: "2026-06-01T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justification:
#   SS-01 (Sensor Adapters) owns prism-dtu-harness per ARCH-INDEX Subsystem Registry v2.115
#   (SS-01 crate list explicitly includes "prism-dtu-harness (planned per ADR-011)").
#   Both router changes (armis.rs + claroty.rs inside prism-dtu-harness) are squarely
#   in the Sensor Adapters subsystem, consistent with sibling story S-DEMO-ARMIS-AQL-001
#   which owns the same harness-clone family and declares subsystems: [SS-01, SS-16].
#   SS-16 (Spec Engine) is included because BC-2.16.013 (the governing BC for this story)
#   is a Spec Engine behavioral contract covering DTU-Parity Verification — the same
#   dual-subsystem scoping as the sibling story and as BC-2.16.013 itself.
#   SS-17 is "WASM Plugin Runtime" (prism-spec-engine AD-019) per ARCH-INDEX v2.115 —
#   it has no ownership relationship to prism-dtu-harness or any DTU crate.
#   (Corrected from SS-17 per LOCAL adversary pass-3 F-P3-HIGH-001 POL-6 violation.)
crates_touched: [prism-dtu-harness]
target_module: prism-dtu-harness
behavioral_contracts:
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — v1.25.
                 # INV-HARNESS-ROUTE-PARITY (added in Wave-5 Phase-A PO burst 2026-06-03):
                 # prism-dtu-harness clones MUST expose the same HTTP route surface as
                 # their corresponding standalone prism-dtu-* crates. Specifically:
                 # - armis::router() MUST include GET /api/v1/search after S-DEMO-ARMIS-AQL-001 merges
                 # - claroty::router() MUST include POST /api/v1/audit_log/get after S-DEMO-CLAROTY-AUDIT-DTU-001 merges
                 # Auth model: Armis→403, Claroty→401. Response envelopes per BC-2.16.013 v1.25.
                 # BC-2.16.013 is active (lifecycle_status: active — auto-promoted at PLUGIN-MIGRATION-001-D merge D-776).
# BC status: BC-2.16.013 is active. S-7.01 gate CLEARED: behavioral_contracts non-empty
# with active BC. Status may transition to ready once AC↔BC bidirectional traces are
# verified at dispatch and both depends_on stories are merged.
verification_properties: []
depends_on:
  - S-DEMO-ARMIS-AQL-001
  # Dependency anchor: prism-dtu-harness::clones::armis::router() must mirror
  # prism-dtu-armis::ClarotyClone after S-DEMO-ARMIS-AQL-001 adds GET /api/v1/search.
  # The standalone clone must merge first so the harness can mirror it. This is a
  # hard ordering dependency, not conceptual relatedness.
  - S-DEMO-CLAROTY-AUDIT-DTU-001
  # Dependency anchor: prism-dtu-harness::clones::claroty::router() must mirror
  # prism-dtu-claroty::ClarotyClone after S-DEMO-CLAROTY-AUDIT-DTU-001 adds
  # POST /api/v1/audit_log/get (Gap-CL-006). The standalone clone must merge first.
blocks: []
points: 3
estimated_days: 1
risk: LOW
# Risk justification:
#   The standalone DTU routes being mirrored will already be tested and merged before
#   this story is dispatched (depends_on both predecessors). The harness clone changes
#   are structural additions only — new route registrations with self-contained handlers
#   that serve embedded fixtures (include_str!) and in-process state. No new types needed:
#   the Armis handler builds results from raw Vec<Value> DEVICES_FIXTURE/ALERTS_FIXTURE;
#   the Claroty handler serves the embedded audit-log.json as raw Vec<Value>. Neither
#   handler imports from the standalone DTU crates (harness is intentionally self-contained).
#   (C-8: corrected from false "share or re-export from standalone DTU crates" claim.)
assumption_validations: []
risk_mitigations: []
# Deferred-finding closure:
#   Closes F-P6-DEFER-001 (out-of-perimeter finding: prism-dtu-harness armis clone missing
#   GET /api/v1/search, surfaced during S-DEMO-ARMIS-AQL-001 lane cascade 2026-05-31).
#   Closes F-P10-LOW-001 (out-of-perimeter finding: prism-dtu-harness claroty clone missing
#   POST /api/v1/audit_log/get, surfaced during S-DEMO-CLAROTY-AUDIT-DTU-001 lane cascade
#   2026-05-31).
#   Promoted to goal task per user direction 2026-06-02 (demo goal fidelity).
---

# S-DEMO-HARNESS-CLONE-PARITY-001 v1.6 — Harness In-Process Clone Route Parity

**Story ID:** S-DEMO-HARNESS-CLONE-PARITY-001
**Status:** ready
**Version:** v1.6
**Wave:** wave-5-e-demo-fidelity
**Priority:** P2
**Points:** 3

---

## Origin

Two out-of-perimeter findings were deferred during Wave 5 lane cascades:

- **F-P6-DEFER-001** — Armis harness clone (`prism-dtu-harness::clones::armis`) missing
  `GET /api/v1/search` after S-DEMO-ARMIS-AQL-001 adds the route to the standalone
  `prism-dtu-armis` clone. Surfaced 2026-05-31 during the S-DEMO-ARMIS-AQL-001 cascade;
  deferred out-of-perimeter (armis.rs is not in scope for the standalone-DTU story).

- **F-P10-LOW-001** — Claroty harness clone (`prism-dtu-harness::clones::claroty`) missing
  `POST /api/v1/audit_log/get` after S-DEMO-CLAROTY-AUDIT-DTU-001 adds it to the standalone
  `prism-dtu-claroty` clone (Gap-CL-006). Surfaced 2026-05-31; deferred out-of-perimeter.

Harness clones are the in-process behavioral clones used for multi-tenant harness tests
(S-3.4.01 / BC-3.5.001 / BC-3.5.002). They must remain route-faithful to the standalone
DTU clones per ADR-031 (DTU=true-DTU). After S-DEMO-ARMIS-AQL-001 and
S-DEMO-CLAROTY-AUDIT-DTU-001 merge, the harness clones will diverge from the standalone
clones until this story closes the gap.

Promoted to goal task per user direction 2026-06-02.

---

## Narrative

As a test infrastructure consumer running multi-tenant harness tests,
I want the in-process harness clones for Armis and Claroty to expose the same routes as
their standalone DTU counterparts,
so that multi-tenant tests using `prism-dtu-harness` exercise the same endpoint surface as
the production DTU integration tests — preserving the DTU=true-DTU fidelity guarantee
(ADR-031) across both clone execution models.

---

## Scope

### Armis harness clone — add `GET /api/v1/search`

File: `crates/prism-dtu-harness/src/clones/armis.rs`

- Add `GET /api/v1/search` handler to `router()` mirroring `prism-dtu-armis::ArmisClone`
  per ADR-031 §D8-a.
- Handler must reuse the existing `check_bearer_auth(&headers, &state.admin_token)` helper.
  Auth semantics (important — differs from standalone's pure-403):
  - Missing or malformed Bearer → HTTP 403 (Armis model per AC-5).
  - Bearer present but token value does not match `state.admin_token` → HTTP 401
    (cross-org credential rejection per BC-3.5.002 postcondition 2).
  - Bearer present and matches → proceed.
  The Red Gate test `test_BC_2_16_013_armis_harness_search_returns_200_with_bearer_403_without` MUST
  send the clone's actual `admin_token` to get 200; an arbitrary "Bearer test-token" will
  yield 401 (token mismatch), not 200. Harness tests must obtain the token via
  `harness.admin_token_for(slug, DtuType::Armis)`.
- Handler must accept `aql: Option<String>` query param; capture via `state.capture_aql()`
  (ArmisHarnessState has `aql_log: Mutex<Vec<String>>` and `capture_aql()` — reuse them).
- Result set: select from the raw Vec<Value> `DEVICES_FIXTURE` or `ALERTS_FIXTURE` embedded
  constants using simple `in:alerts` vs `in:devices` AQL string-matching (matching the same
  `in:<entity>` discriminator logic the standalone uses). Pagination via `page`/`size` query
  params using the same offset-slice pattern already used by `get_devices`/`get_alerts`. The
  harness does NOT implement standalone-identical time-window filtering (`parse_aql_time_bounds`
  and typed DeviceRecord/AlertRecord structs are standalone internals not available here).
- Response envelope: `{"data": {"results": [...], "total": N}}` matching standalone DTU.
- Update `armis.rs` module-doc route table to include the new endpoint.

### Claroty harness clone — add `POST /api/v1/audit_log/get`

File: `crates/prism-dtu-harness/src/clones/claroty.rs`

- Add `POST /api/v1/audit_log/get` handler to BOTH `router()` AND `network_router()`,
  mirroring `prism-dtu-claroty::ClarotyClone` (Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001).
  The Claroty harness has two routers; both must expose the route for full multi-tenant parity
  (BC-3.5.002 / network-isolation harness is BC-3.5.002's consumer).
- In `router()` (logical mode): call `check_bearer_auth(&headers)` which accepts ANY non-empty
  Bearer, returning 401 on missing/malformed.
- In `network_router()` (network mode): follow the existing sibling pattern. Inspection of
  `network_router()` shows that alert/vulnerability routes use the same plain `check_bearer_auth`
  (not a network-mode variant), while only the device list uses `list_devices_network`. Register
  the audit_log handler using plain `check_bearer_auth` — the same convention as the sibling
  `list_alerts`, `list_alerted_devices`, and `list_vulnerabilities` already registered in
  `network_router()`.
- The fixture must be served as an embedded constant (following the harness `include_str!`
  pattern — e.g. `include_str!("../../../prism-dtu-claroty/fixtures/audit-log.json")`).
  Deserialize at request time as `Vec<Value>` and return raw. Do NOT call
  `prism_dtu_common::load_fixture` — that is the standalone DTU's runtime pattern, not the
  harness compile-time embed pattern.
- Response envelope: `{"audit_log": [...], "total": N}` matching standalone DTU.
- Update `claroty.rs` module-doc route table to include the new endpoint in both routers.

### Pagination Parity Scope (intentional design — spec-sanctioned)

Harness `get_search` (Armis) uses `page`/`size` query params only, consistent with all
sibling harness routes (`get_devices`, `get_alerts`). This is intentional:

- **Sibling-route consistency (TD-VSDD-060):** Adding `offset`/`limit` to the Armis search
  handler while sibling routes use `page`/`size` would break within-harness-clone parameter
  consistency. The correct action per TD-VSDD-060 is to use the same convention across all
  sibling routes within the harness.

- **Consumer boundary:** Harness clones are consumed exclusively by isolation/harness tests
  (BC-3.5.001 / BC-3.5.002 consumers). They are NOT consumed by the `PipelineExecutor`.
  The standalone `OffsetLimit` / `PipelineExecutor` pagination convention is a pipeline
  concern that no harness consumer exercises. There is no harness test that would drive
  `offset`/`limit` params to the harness clone.

- **INV-HARNESS-ROUTE-PARITY scope:** AC-002 explicitly scopes Armis harness search parity
  to structural parity (route surface + envelope + auth). AC-002 does NOT require
  byte-identical query-param contracts between harness and standalone. The standalone
  `offset`/`limit` params are a `PipelineExecutor` push-down feature; parity tests verify
  envelope shape and auth behavior, not pagination-param identity.

This design is spec-sanctioned per CLAUDE.md Source-of-Truth Precedence §1: the story
spec governs implementation scope. Implementers must NOT add `offset`/`limit` to the
harness search handler to "match" the standalone — doing so would introduce a sibling
inconsistency that would require a follow-up TD-VSDD-060 sweep across all sibling harness
routes. The current `page`/`size`-only design is correct and final.

---

## Behavioral Contracts

| BC ID | Version | Title | Role in This Story |
|-------|---------|-------|-------------------|
| BC-2.16.013 | v1.25 | Bundled Sensor Spec Authoring and DTU-Parity Verification | INV-HARNESS-ROUTE-PARITY (added Wave-5 Phase-A PO burst 2026-06-03) governs this story's complete scope: harness armis clone MUST include `GET /api/v1/search`; harness claroty clone MUST include `POST /api/v1/audit_log/get`; auth model per sensor (Armis→403, Claroty→401); response envelopes must match standalone DTU. AC-001 through AC-005 implement INV-HARNESS-ROUTE-PARITY. |

---

## Acceptance Criteria

### AC-001: Armis harness clone registers GET /api/v1/search
`crates/prism-dtu-harness/src/clones/armis.rs::router()` includes a handler for
`GET /api/v1/search`. A request carrying the clone's actual `admin_token` as
`Authorization: Bearer {admin_token}` returns 200. A request with no Authorization header
returns 403 (missing Bearer — Armis auth model per BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY).
Note: the harness `check_bearer_auth(&headers, &state.admin_token)` returns 403 for missing/
malformed Bearer and 401 for a present-but-mismatched token (cross-org rejection). Red Gate
tests must therefore send the clone's actual admin token (obtained via
`harness.admin_token_for(slug, DtuType::Armis)`) to obtain 200 — an arbitrary "Bearer test-token"
will yield 401 (mismatch), not 200.
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — armis::router() MUST include
GET /api/v1/search; Armis auth model: 403 on missing/invalid Bearer)

Red Gate test: `test_BC_2_16_013_armis_harness_search_returns_200_with_bearer_403_without`

### AC-002: Armis harness clone AQL routing has structural parity with standalone
`GET /api/v1/search?aql=in:devices` returns a payload where `$.data.results` is a non-empty
array and `$.data.total` is numeric; `GET /api/v1/search?aql=in:alerts` returns a payload
where `$.data.results` contains alert records (not device records). Response envelope is
`{"data": {"results": [...], "total": N}}`. The harness serves results from raw Vec<Value>
DEVICES_FIXTURE / ALERTS_FIXTURE — structural parity with standalone, not byte-identical
field-for-field equality (standalone uses typed DeviceRecord/AlertRecord structs with
org-tag merge and time-window filtering; harness serves raw fixture values without those
standalone-specific features).
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — Armis search response envelope:
`{"data": {"results": [...], "total": N}}`)

Red Gate test: `test_BC_2_16_013_armis_harness_search_aql_in_devices_returns_device_records`

### AC-003: Claroty harness clone registers POST /api/v1/audit_log/get in both routers
`crates/prism-dtu-harness/src/clones/claroty.rs::router()` includes a handler for
`POST /api/v1/audit_log/get`. A request with a valid `Authorization: Bearer {non-empty}`
header returns 200. A request with no/invalid Bearer returns 401 (Claroty auth model per
BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — Claroty returns 401, NOT 403).
Additionally, `claroty.rs::network_router()` also registers `POST /api/v1/audit_log/get`
so that the network-mode isolation harness (BC-3.5.002 consumer) has route parity.
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — claroty::router() MUST include
POST /api/v1/audit_log/get; Claroty auth model: 401 on missing/invalid Bearer)

Red Gate test: `test_BC_2_16_013_claroty_harness_audit_log_returns_200_with_bearer_401_without`

### AC-004: Claroty harness clone audit_log response matches standalone
Response envelope `{"audit_log": [...], "total": N}` where `audit_log` is non-empty and
all 5 TOML-declared columns are present in each entry (id, action, actor, timestamp, resource).
The fixture is served as embedded raw Vec<Value> (the harness embeds via `include_str!` of the
prism-dtu-claroty audit-log.json fixture, NOT via `prism_dtu_common::load_fixture`). The
response shape matches the standalone `prism-dtu-claroty` response per BC-2.16.013 v1.25
INV-HARNESS-ROUTE-PARITY response envelope requirement.
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — Claroty audit_log response:
`{"audit_log": [...], "total": N}`)

Red Gate test: `test_BC_2_16_013_claroty_harness_audit_log_response_envelope_matches_standalone`

### AC-005: Module-doc route tables updated in both files
Both `armis.rs` and `claroty.rs` module-doc route inventory tables list the new routes.
The claroty.rs module-doc must reflect the route in both the logical-mode route table
(router()) and the network-mode route table (network_router()). No undocumented routes.
Route parity with standalone DTU routes verified by inspection.
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — "route parity is verified by
multi-tenant harness tests (BC-3.5.001/BC-3.5.002 consumers)")

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `armis.rs::router()` — add GET /api/v1/search | `crates/prism-dtu-harness/src/clones/armis.rs` | Effectful (HTTP router mutation) |
| `claroty.rs::router()` — add POST /api/v1/audit_log/get | `crates/prism-dtu-harness/src/clones/claroty.rs` | Effectful (HTTP router mutation) |
| `claroty.rs::network_router()` — add POST /api/v1/audit_log/get | `crates/prism-dtu-harness/src/clones/claroty.rs` | Effectful (HTTP router mutation, network-mode) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters
- `architecture/dependency-graph.md` §Wave-5 DTU fidelity stories

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Armis harness: missing Bearer on /api/v1/search | 403 (matches Armis auth model — `check_bearer_auth` returns 403 for absent/malformed Bearer, 401 for present-but-wrong token) |
| EC-002 | Claroty harness: missing Bearer on /api/v1/audit_log/get | 401 (matches Claroty auth model — `check_bearer_auth` returns 401 for absent/empty Bearer) |
| EC-003 | Armis harness: absent aql param | Returns devices (safe default from DEVICES_FIXTURE raw Vec<Value>) |
| EC-004 | Harness armis search: `in:devices` and `in:alerts` both present in AQL | Devices take precedence (match standalone discriminator: `in:alerts` only when `in:alerts` present AND `in:devices` absent) |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `crates/prism-dtu-harness/src/clones/armis.rs` (existing) | ~2,500 |
| `crates/prism-dtu-harness/src/clones/claroty.rs` (existing) | ~2,500 |
| Standalone DTU route handlers (pattern reference: ~2 files) | ~4,000 |
| BC-2.16.013 v1.25 (INV-HARNESS-ROUTE-PARITY section) | ~3,000 |
| **Total estimate** | **~14,000 tokens** |

Well within 20-30% of agent context window. No split needed.

---

## Tasks (stub — expand at dispatch)

- [ ] **Task 1:** Read `crates/prism-dtu-harness/src/clones/armis.rs` — understand current
  `router()` structure, `ArmisHarnessState` fields (`admin_token`, `aql_log`, `capture_aql()`),
  the `check_bearer_auth(&headers, &state.admin_token)` helper signature, and the raw
  `DEVICES_FIXTURE`/`ALERTS_FIXTURE` constants.
- [ ] **Task 2:** Read `crates/prism-dtu-armis/src/routes/search.rs` (delivered by
  S-DEMO-ARMIS-AQL-001) — understand the `in:alerts` / `in:devices` AQL discriminator logic
  and the `{"data": {"results": [...], "total": N}}` response envelope shape.
- [ ] **Task 3:** Add `GET /api/v1/search` to harness armis `router()`. Write Red Gate tests
  following the reqwest-over-TcpListener idiom in `tests/logical_isolation_test.rs` (NOT
  tower::ServiceExt::oneshot — tower is not a harness dependency). Tests must:
  (a) Obtain the clone's actual `admin_token` (e.g. via `harness.admin_token_for()`) to send
  a valid Bearer and assert 200;
  (b) Send a request with no Authorization header and assert 403;
  (c) Assert `in:devices` AQL → `$.data.results` non-empty array, `$.data.total` numeric.
  Verify RED before implementing.
- [ ] **Task 4:** Implement harness armis search handler using `DEVICES_FIXTURE`/`ALERTS_FIXTURE`
  raw Vec<Value> with `in:alerts` vs `in:devices` string matching and page/size pagination.
  All Red Gate tests GREEN.
- [ ] **Task 5:** Read `crates/prism-dtu-harness/src/clones/claroty.rs` — understand both
  `router()` and `network_router()` constructors, the `check_bearer_auth(&headers)` helper
  (accepts ANY non-empty Bearer, 401 otherwise), and the existing `include_str!` fixture pattern.
- [ ] **Task 6:** Read `crates/prism-dtu-claroty/src/routes/audit_log.rs` (delivered by
  S-DEMO-CLAROTY-AUDIT-DTU-001) — understand the response envelope `{"audit_log": [...], "total": N}`
  and the fixture filename (`audit-log.json`).
- [ ] **Task 7:** Add `POST /api/v1/audit_log/get` to harness claroty BOTH `router()` AND
  `network_router()`. Write Red Gate tests following the reqwest-over-TcpListener idiom in
  `tests/logical_isolation_test.rs`. Tests must assert 200 + non-empty `audit_log` array +
  401 on missing Bearer. Verify RED before implementing.
- [ ] **Task 8:** Implement harness claroty audit_log handler using `include_str!` of the
  `prism-dtu-claroty/fixtures/audit-log.json` fixture, parsed as raw `Vec<Value>`. Do NOT
  call `prism_dtu_common::load_fixture` or import types from `prism-dtu-claroty`.
  All Red Gate tests GREEN.
- [ ] **Task 9:** Update module-doc route tables in both files (armis.rs and claroty.rs;
  the claroty.rs table should reflect both `router()` and `network_router()` routes). SAP-1
  sweep: `rg 'event_type\s*=' crates/ --type rust`.
- [ ] **Task 10:** `just check` — final pre-push gate.

---

## Previous Story Intelligence

N/A — first story in E-DTU-FIDELITY touching `prism-dtu-harness`. Related predecessors
that establish the patterns to mirror:
- S-DEMO-ARMIS-AQL-001 — delivers standalone `GET /api/v1/search` (must merge before dispatch)
- S-DEMO-CLAROTY-AUDIT-DTU-001 — delivers standalone `POST /api/v1/audit_log/get` (must merge)

Read those stories' delivered source files before writing any harness code.

---

## Architecture Compliance Rules

- Harness clones mirror standalone DTU route surfaces (ADR-031 DTU=true-DTU)
- Auth model per sensor: Armis→403, Claroty→401 (not interchangeable)
- `prism-dtu-harness` must NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query`
- New `event_type` emissions require BC-2.16.002 catalog row (SAP-1)
- No `println!` in production code

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `axum` | **0.7 (pinned literally in `crates/prism-dtu-harness/Cargo.toml`, NOT .workspace)** | Route handler, `State`, `Json`, `HeaderMap`, `Query` |
| `serde_json` | 1 (workspace) | `json!` macro, response envelopes, `Vec<Value>` fixture deserialization |
| `tokio` | 1 (workspace) | Async test runtime |
| `reqwest` | 0.12 (workspace) | Red Gate test HTTP client — use over TcpListener per `tests/logical_isolation_test.rs` idiom |
| `prism-dtu-common` | workspace path | `FailureMode` and other shared helpers (NOT `load_fixture` — harness embeds fixtures via `include_str!`, not `load_fixture`) |

Version note: `axum = "0.7"` is pinned literally in `crates/prism-dtu-harness/Cargo.toml` (not via
`.workspace = true`). The new routes have no path params so the axum 0.7 `:param` vs 0.8 `{param}`
colon-syntax migration risk does not apply to them. Do not upgrade the axum pin — pin must stay "0.7".

---

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-dtu-harness/src/clones/armis.rs` | Add GET /api/v1/search handler + route registration + module-doc update |
| MODIFY | `crates/prism-dtu-harness/src/clones/claroty.rs` | Add POST /api/v1/audit_log/get handler + route registration + module-doc update |

---

## Forbidden Dependencies

`prism-dtu-harness` MUST NOT gain a dependency on:
- `prism-spec-engine` (build MUST fail if this dep appears)
- `prism-sensors` (build MUST fail if this dep appears)
- `prism-query` (build MUST fail if this dep appears)
- `prism-dtu-claroty` (build MUST fail if this dep appears — harness is intentionally
  self-contained per module docstring; `ClarotyAuditLogEntry`/`GetAuditLogBody` and other
  types from `prism-dtu-claroty` MUST NOT be imported into the harness. The Claroty
  harness audit_log handler serves raw `Vec<Value>` from an embedded `include_str!` fixture
  and needs no typed struct from the standalone crate.)
- `prism-dtu-armis` (build MUST fail if this dep appears as a direct crate dependency —
  `DEVICES_FIXTURE`/`ALERTS_FIXTURE` reference the armis crate's `fixtures/` directory
  via `include_str!` path strings, which is a compile-time file inclusion, not a crate dep;
  this is the existing and correct pattern in armis.rs)

---

## References

- ADR-031 §D8-a — Armis AQL endpoint fidelity (harness parity obligation for GET /api/v1/search)
- ADR-031 §D7 — Harness clones in-scope for DTU=true-DTU; governs the Claroty audit_log
  harness route-parity obligation; explicitly records the Claroty audit_log harness gap as HIGH
  and co-scopes its closure with S-DEMO-CLAROTY-AUDIT-DTU-001 (PRIMARY authority for the
  harness claroty route-parity requirement)
- ADR-031 §D1-c — DTU MUST register exactly the real API's endpoints; no extra routes, no
  missing routes (route-existence fidelity — binding)
- ADR-031 §D2 — Permitted Divergences (exhaustive: rate-limit cooldowns, credential format,
  TLS, persistence semantics); synthetic fixture data only; §D2 is NOT the route-parity
  authority and does NOT govern route existence for the Claroty audit_log
- Gap-CL-006 — Claroty audit_log route gap (closed by S-DEMO-CLAROTY-AUDIT-DTU-001); harness
  clone must mirror the standalone POST /api/v1/audit_log/get per INV-HARNESS-ROUTE-PARITY
- ADR-031 §D1 — DTU clone isolation (scope extension in §D7 explicitly covers harness clones)
- S-DEMO-ARMIS-AQL-001 — standalone Armis AQL route (depends on)
- S-DEMO-CLAROTY-AUDIT-DTU-001 — standalone Claroty audit_log route (depends on)
- BC-3.5.001 / BC-3.5.002 — harness isolation contracts (candidate BCs, pending PO)
- F-P6-DEFER-001 — deferred finding: harness armis missing /api/v1/search
- F-P10-LOW-001 — deferred finding: harness claroty missing /api/v1/audit_log/get

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.6 | 2026-06-08 | story-writer | LOCAL adversary fix F-RC2C-LOW-001 (spec-only). Claroty audit_log §References anchor corrected: §D2 was incorrectly headlined as the route-parity authority while the body asserted §D7 — a self-contradiction. Rewritten so §D7 (harness-scope extension, PRIMARY) + §D1-c (endpoint-existence fidelity, PRIMARY) are the leading authorities; §D2 retained as explicitly-scoped secondary for synthetic fixture data only with a note that it is NOT the route-parity authority. Armis §D8-a anchor left intact (correct). TD-VSDD-060 sweep: §D2 appears only in the corrected §References block and in the historical v1.5 changelog row — no other sites found. `behavioral_contracts` array untouched, all AC↔BC trace lines untouched, INV-HARNESS-ROUTE-PARITY references untouched, S-7.01 gate remains CLEARED, status remains ready. |
| 1.5 | 2026-06-08 | story-writer | LOCAL adversary fixes F-RC3-MED-001 + F-RC1-LOW-001 (spec-only). F-RC3-MED-001: corrected Claroty audit_log architecture anchor — replaced incorrect `ADR-031 §D8-b` (which governs Claroty trailing-slash / Gap-CL-001, a different concern) with correct `Gap-CL-006 + ADR-031 §D2 + INV-HARNESS-ROUTE-PARITY` in §References; Armis §D8-a anchor left intact (verified correct). F-RC1-LOW-001: added `### Pagination Parity Scope` subsection under §Scope documenting that harness `get_search` uses `page`/`size` pagination only (not `offset`/`limit`) intentionally — consistent with all sibling harness routes (TD-VSDD-060 sibling-consistency), harness clones are not consumed by PipelineExecutor, and INV-HARNESS-ROUTE-PARITY scopes to structural parity (AC-002). Spec-only edit — `behavioral_contracts` array untouched, all AC↔BC trace lines untouched, INV-HARNESS-ROUTE-PARITY references untouched, S-7.01 gate remains CLEARED, status remains ready. |
| 1.4 | 2026-06-08 | story-writer | LOCAL adversary pass-3 F-P3-HIGH-001 (POL-6 subsystem correction SS-17→SS-01+SS-16) + Red Gate test-name reconciliation. (1) `subsystems: [SS-17]` corrected to `[SS-01, SS-16]`: SS-17 is "WASM Plugin Runtime" per ARCH-INDEX v2.115 — it has no ownership of prism-dtu-harness or any DTU crate; SS-01 ("Sensor Adapters") explicitly lists prism-dtu-harness per ARCH-INDEX v2.115; SS-16 included to match sibling story S-DEMO-ARMIS-AQL-001 dual-scoping and BC-2.16.013 provenance. (2) Subsystem anchor justification comment rewritten with correct subsystem names and ARCH-INDEX v2.115 citation. (3) Architecture Mapping section reference corrected: `§SS-17 DTU Clones` → `§SS-01 Sensor Adapters`. (4) All five Red Gate test-name references updated to codebase convention `test_BC_2_16_013_<name>` (4 AC references + 1 inline mention in §Scope). Spec-only edit — `behavioral_contracts` array untouched, all AC↔BC trace lines untouched, INV-HARNESS-ROUTE-PARITY references untouched, S-7.01 gate remains CLEARED, status remains ready. |
| 1.3 | 2026-06-08 | story-writer | remove-uncertainty corrections C-1..C-8 (D-1061). Implementation-scope refinements only — `behavioral_contracts` array untouched, all AC↔BC trace lines untouched, INV-HARNESS-ROUTE-PARITY references untouched, S-7.01 gate remains CLEARED, status remains ready. C-1: corrected fixture idiom — harness embeds via include_str!, NOT load_fixture. C-2: corrected Armis search data source — raw Vec<Value> DEVICES_FIXTURE/ALERTS_FIXTURE, no typed structs, no time-window filtering. C-3: clarified Armis auth semantics — check_bearer_auth(&headers, &state.admin_token) gives 403 for missing Bearer, 401 for present-but-wrong token; Red Gate test must send actual admin_token. C-4: expanded Claroty scope — POST /api/v1/audit_log/get must be registered in BOTH router() and network_router(); network_router() uses plain check_bearer_auth per sibling alert/vuln route convention. C-5: Red Gate test idiom corrected — reqwest-over-TcpListener (tests/logical_isolation_test.rs pattern), NOT tower::ServiceExt::oneshot. C-6: axum pin corrected — "0.7" pinned literally in Cargo.toml, NOT .workspace. C-7: AC-002 softened to structural parity ($.data.results non-empty array, $.data.total numeric, in:alerts selects alerts) — not byte-identical field-for-field equality. C-8: Forbidden Dependencies expanded — prism-dtu-claroty MUST NOT be added; harness audit_log handler uses raw Vec<Value> from embedded fixture, no typed struct import. |
| 1.2 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; BC-2.16.013 v1.25 active (PO authored D-989, INV-HARNESS-ROUTE-PARITY); depends_on S-DEMO-ARMIS-AQL-001 (merged PR #168) + S-DEMO-CLAROTY-AUDIT-DTU-001 (merged PR #167) BOTH SATISFIED; S-7.01 gate CLEARED. |
| 1.1 | 2026-06-03 | story-writer | Wave-5 Phase-A BC-array propagation burst (D-989). PO authored BC-2.16.013 v1.25 with INV-HARNESS-ROUTE-PARITY invariant governing this story's full scope. Propagated into story: (1) `behavioral_contracts: []` → `[BC-2.16.013]`; status stays draft (depends_on stories both merged, but AC↔BC traces need dispatch-time verification). (2) Added §Behavioral Contracts table with BC-2.16.013 v1.25 + INV-HARNESS-ROUTE-PARITY role. (3) ACs rewritten from INV-HARNESS-ROUTE-PARITY: AC-001 (armis GET /api/v1/search, 403 on missing Bearer), AC-002 (AQL routing, response envelope), AC-003 (claroty POST /api/v1/audit_log/get, 401 on missing Bearer), AC-004 (response envelope), AC-005 (module-doc). Red Gate test names added. (4) Token budget updated to include BC-2.16.013 v1.25 read. Version bump 1.0 → 1.1. |
| 1.0 | 2026-06-01 | story-writer | Initial stub. Captures scope (armis.rs + claroty.rs route additions), gating (depends_on S-DEMO-ARMIS-AQL-001 + S-DEMO-CLAROTY-AUDIT-DTU-001), and finding closure (F-P6-DEFER-001 + F-P10-LOW-001). Status draft pending PO BC authorship per S-7.01. |
