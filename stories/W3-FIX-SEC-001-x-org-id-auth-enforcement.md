---
story_id: W3-FIX-SEC-001
title: "DTU clones: bind OrgId to clone instance — reject mismatched X-Org-Id header"
wave: 3.1
level: "L4"
target_module: prism-dtu-claroty
subsystems: [SS-01]
priority: P0
depends_on: []
blocks: [W3-FIX-SEC-002]
estimated_days: 2
points: 5
status: merged
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-01T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
  - BC-3.5.002
  - BC-3.2.001
verification_properties: [VP-124, VP-125, VP-126]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.5.001, BC-3.5.002, BC-3.2.001]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
---

# W3-FIX-SEC-001: DTU clones — bind OrgId to clone instance, reject mismatched X-Org-Id header

## Narrative

As a Prism security reviewer, I want every DTU clone HTTP handler to derive `OrgId`
from the clone's own registered `instance_org_id` rather than trusting the caller-supplied
`X-Org-Id` header, so that the network harness isolation boundary is cryptographically
meaningful and cross-tenant spoofing returns HTTP 401 rather than silently accessing the
wrong org's state.

## Objective

Gate Step D identified SEC-001 (HIGH, CWE-287/CWE-639, OWASP A01): the `extract_org_id`
helpers in `prism-dtu-claroty`, `prism-dtu-crowdstrike`, and `prism-dtu-cyberint` accept
the `X-Org-Id` (or `X-Prism-Org-Id`) header from the wire with no validation. Any client
that reaches a clone's loopback port can supply an arbitrary UUID and access a different
org's state.

The fix: each clone already knows its `instance_org_id` at startup (it was assigned one
by the harness). Route handlers must use `state.instance_org_id` as the authoritative
org identity and reject any request whose header `OrgId` disagrees with it — returning
HTTP 401 with a structured error body. Armis was noted as needing similar review; apply
the same pattern there as well.

ADR-006 comment at the extraction site explicitly states this is a "structural placeholder
until auth middleware wires validated OrgId into request extensions." This story wires it.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.001 | Harness Logical Isolation Invariants | Postcondition 1 (query OrgA returns only OrgA data), Invariant 2 (concurrent operations do not observe each other) |
| BC-3.5.002 | Harness Network Isolation Invariants | Precondition 3 (routing error observable via 401 when wrong-org request reaches wrong port) |
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key | Invariant 1 (composite key is the exclusive keying scheme), Invariant 2 (isolation is structural) |

## Acceptance Criteria

### AC-001: Same-org request succeeds (traces to BC-3.2.001 postcondition 1)
A request to Claroty clone for OrgA that supplies `X-Org-Id: <OrgA UUID>` receives HTTP 200
with OrgA's device data. The handler derives `OrgId` from the clone state AND verifies the
header matches; correct callers are unaffected.

### AC-002: Cross-org spoofing returns 401 (traces to BC-3.5.002 precondition 3)
A request to OrgA's Claroty clone that supplies `X-Org-Id: <OrgB UUID>` receives HTTP 401
with JSON body `{"error": "org_id mismatch: request does not match this clone instance"}`.
The clone's internal state is not accessed or modified.

### AC-003: Missing header returns 401 (traces to BC-3.5.001 postcondition 1)
A request that omits the `X-Org-Id` header entirely receives HTTP 401. The sentinel UUID
fallback (`00000000-0000-7000-8000-000000000000`) must NOT be accepted as a valid org.

**Auth model A only (Claroty, CrowdStrike).** Cyberint follows auth model B
(multi-org-per-instance routing); missing `X-Prism-Org-Id` defaults to the instance's own
session and returns 200.  Armis uses validate-on-presence (backward compatibility with
50+ pre-existing tests); missing `X-Org-Id` is allowed and returns 200.  See test file
comments for per-clone treatment.

### AC-004: All four DTU clones covered (traces to BC-3.2.001 invariant 1)
The same instance-keyed validation is applied to `prism-dtu-claroty`, `prism-dtu-crowdstrike`,
`prism-dtu-cyberint`, and `prism-dtu-armis`. Each crate's `extract_org_id` (or equivalent)
function is replaced with an instance-binding check using `state.instance_org_id`.

### AC-005: Regression test for HS-003-02 invariant (traces to BC-3.5.002 precondition 3)
A new integration test `test_cross_org_header_rejected` in each affected crate's
`tests/multi_tenant.rs` (or equivalent) demonstrates that credential-mismatch returns
HTTP 401 — not HTTP 200 and not a silent empty response.

### AC-006: Positive paths in existing tests still pass (traces to BC-3.5.001 postcondition 1)
All pre-existing multi-tenant tests that supply the correct `X-Org-Id` continue to pass
without modification to the test side.

## Auth Model Per Clone

Implementation surfaced an architectural tension during the TDD pass (commit `a8209c8c`):
not all DTU clones share the same semantics for a missing `X-Org-Id` header.

| Clone | Auth Model | Header Name | X-Org-Id Required? | Missing Header | Mismatch Behavior |
|-------|-----------|-------------|-------------------|----------------|-------------------|
| prism-dtu-claroty | A (single-org-per-instance) | `X-Org-Id` | Yes | 401 | 401 org_id mismatch |
| prism-dtu-crowdstrike | A (single-org-per-instance) | `X-Org-Id` | Yes | 401 | 401 org_id mismatch |
| prism-dtu-cyberint | B (multi-org-per-instance routing) | `X-Prism-Org-Id` | No (routing hint) | 200 (defaults to instance_org_id session) | 401 org_id mismatch (session not found for foreign org) |
| prism-dtu-armis | Validate-on-presence (backcompat) | `X-Org-Id` | No (validated only if present) | 200 (guard skipped) | 401 org_id mismatch |

**Auth model A** enforces the header as a strict security gate: absent header is always
rejected with 401.  This is the originally specified behavior for AC-003.

**Auth model B** treats the header as an org routing hint.  Cyberint supports multiple
concurrent orgs per clone instance (BC-3.2.003).  When no header is supplied, the request
falls through to the `instance_org_id` fallback path, which matches the session registered
at login.  When a foreign org UUID is supplied, `is_valid_session` returns false for that
org → 401 with "org_id mismatch".

**Validate-on-presence** was chosen for Armis to preserve backward compatibility with 50+
pre-existing integration tests.  The guard (`if headers.get("x-org-id").is_some()`) is
inserted before `validate_org_id`; when absent the request proceeds normally.  When present,
the full mismatch check applies.

## Tasks

1. Read `crates/prism-dtu-claroty/src/routes/devices.rs` lines 130-145 to understand the
   current `extract_org_id` sentinel pattern.
2. Add `instance_org_id: OrgId` to `ClarotyState` (or confirm it already exists — check
   `clone.rs`) and ensure it is populated from the harness at startup.
3. Replace the `extract_org_id` helper with `validate_org_id(headers, state.instance_org_id)`
   that returns `Result<OrgId, (StatusCode, Json<serde_json::Value>)>` — returning HTTP 401
   if the header is missing, unparseable, or mismatches the instance OrgId.
4. Propagate the `validate_org_id` guard to all route handlers in `prism-dtu-claroty` that
   currently call `extract_org_id`.
5. Repeat steps 2-4 for `prism-dtu-crowdstrike` (`hosts.rs:202-213`).
6. Repeat for `prism-dtu-cyberint` (`alerts.rs:54-60`, header is `X-Prism-Org-Id`).
7. Repeat for `prism-dtu-armis` (review `devices.rs` and apply the same pattern).
8. Add `test_cross_org_header_rejected` tests to each crate's integration test file,
   asserting HTTP 401 on header mismatch (AC-005).
9. Run `cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-cyberint
   -p prism-dtu-armis --all-features` — all tests must pass.
10. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| validate_org_id helper | prism-dtu-claroty | `crates/prism-dtu-claroty/src/routes/devices.rs` | Pure (returns Result) |
| validate_org_id helper | prism-dtu-crowdstrike | `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | Pure |
| validate_org_id helper | prism-dtu-cyberint | `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Pure |
| validate_org_id helper | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/devices.rs` | Pure |
| clone state structs | all four crates | `clone.rs` in each | Effectful (spawns HTTP server) |
| Integration tests | all four crates | `tests/multi_tenant.rs` or equivalent | Effectful (HTTP) |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope because
all four affected crates (`prism-dtu-claroty`, `prism-dtu-crowdstrike`, `prism-dtu-cyberint`,
`prism-dtu-armis`) are client-mode Security Telemetry DTU adapters per the ARCH-INDEX
Subsystem Registry definition of SS-01.

**Dependency anchor justification:** `depends_on: []` — this fix is self-contained; it
requires no other W3-FIX story to land first. `blocks: [W3-FIX-SEC-002]` — SEC-002 (admin
token on reset) is logically downstream because once per-instance OrgId binding is correct,
it is cleaner to implement reset auth against a verified identity context.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `X-Org-Id` header is present but contains a non-UUID string | HTTP 401; `"org_id mismatch"` body; clone state not accessed |
| EC-002 | `X-Org-Id` header contains UUID v4 (not v7) belonging to the correct org | Depends on `instance_org_id` comparison — if the UUIDs match byte-for-byte, accept; do not enforce v7 at this layer (that is OrgRegistry's job at registration). Document this behavior. |
| EC-003 | Sentinel UUID `00000000-0000-7000-8000-000000000000` sent as header | HTTP 401 — the sentinel is not the clone's `instance_org_id` |
| EC-004 | Clone handles the Claroty `reset_for/{org_id}` path-parameter variant | Path param should also be validated against `instance_org_id`; return 403 if mismatched (distinct from header check) |
| EC-005 | Two clones for the same DtuType but different OrgIds running simultaneously | Each clone independently validates against its own `instance_org_id`; no cross-clone interaction |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3 500 |
| BC files (3 BCs) | ~4 000 |
| Four `devices.rs`/`hosts.rs`/`alerts.rs` source files (~150 lines each) | ~3 500 |
| Four `clone.rs` state struct files (~100 lines each) | ~2 000 |
| Existing multi_tenant test files (4 × ~300 lines) | ~5 000 |
| ADR-006 relevant sections | ~1 000 |
| Cargo output + clippy | ~1 000 |
| **Total** | **~20 000** |

Within the 20-30% context window limit. If all four crates require large context, split
into two sub-tasks: claroty+crowdstrike first, then cyberint+armis.

## Previous Story Intelligence

- **S-3.2.01 through S-3.2.04** built per-org state segregation (`(OrgId, String)` keying)
  in the DTU crates. The state-layer isolation is correct; the HTTP-layer gap was introduced
  by leaving `extract_org_id` as a "structural placeholder" stub.
- **S-3.3.04** (PR #103) built the network harness isolation mode that gives each org its own
  TCP listener — but the value of that listener isolation is defeated if the handler trusts the
  caller-supplied org header. This fix completes the security picture.
- **Lesson:** Any `// structural placeholder until auth middleware...` comment is a
  SEC-HIGH-deferred marker. When merging stories that contain such comments, file a
  fix story immediately rather than deferring to the gate review.

## Architecture Compliance Rules

- Do NOT read or forward the raw `X-Org-Id` header value as the routing key. The
  `instance_org_id` from clone state is the only authoritative identity.
- Do NOT use `OrgId::from_uuid` with non-v7 inputs at this layer — use whatever
  constructor avoids panics; comparison against `instance_org_id` is the gate.
- The 401 response body MUST be a JSON object `{"error": "..."}` — not a plain-text
  string — for consistency with other DTU error responses.
- The validation helper MUST be named `validate_org_id` (not `extract_org_id`) to make
  the rename visible in `git diff` and prevent confusion with the old stub.
- Do NOT add the `prism-dtu-common` crate as a new dependency of `prism-dtu-claroty`
  solely for this fix — implement `validate_org_id` locally within each crate's routes
  module. If `prism-dtu-common` already provides a suitable helper, use it; do not
  duplicate if it exists.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| axum | workspace pin | HTTP handler extractors, `StatusCode`, `Json` |
| uuid | workspace pin | UUID parsing for header value |
| serde_json | workspace pin | JSON error body construction |
| prism-core (OrgId) | workspace | `OrgId` type for instance comparison |

No new Cargo dependencies introduced by this story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-claroty/src/routes/devices.rs` | Modify | Replace `extract_org_id` with `validate_org_id`; apply to all handlers |
| `crates/prism-dtu-claroty/src/clone.rs` | Modify (if needed) | Ensure `instance_org_id` is populated and accessible |
| `crates/prism-dtu-claroty/tests/multi_tenant.rs` | Modify | Add `test_cross_org_header_rejected` |
| `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | Modify | Same pattern as Claroty |
| `crates/prism-dtu-crowdstrike/src/clone.rs` | Modify (if needed) | Same as Claroty |
| `crates/prism-dtu-crowdstrike/tests/multi_tenant.rs` | Modify | Add rejection test |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Modify | Replace `X-Prism-Org-Id` extraction with instance-binding check |
| `crates/prism-dtu-cyberint/src/clone.rs` | Modify (if needed) | Populate `instance_org_id` |
| `crates/prism-dtu-cyberint/tests/multi_tenant.rs` | Modify | Add rejection test |
| `crates/prism-dtu-armis/src/routes/devices.rs` | Modify | Apply same pattern |
| `crates/prism-dtu-armis/src/clone.rs` | Modify (if needed) | Populate `instance_org_id` |
| `crates/prism-dtu-armis/tests/multi_tenant.rs` | Modify | Add rejection test |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| validate_org_id helper (claroty/crowdstrike/cyberint/armis) | pure-core | Pure function: takes `&HeaderMap` + `OrgId`, returns `Result`; no I/O |
| Route handlers (devices.rs, hosts.rs, alerts.rs) | effectful-shell | Axum async handlers; perform HTTP I/O and read Arc<State> |
| Integration tests (multi_tenant.rs) | effectful-shell | Spawn real HTTP clones; perform loopback TCP connections |

## Forbidden Dependencies

The fix MUST NOT introduce:
- Any new `crate` dependency that is not already in the affected crate's `Cargo.toml`.
- Any import of production sensor adapter code (prism-claroty, prism-crowdstrike, etc.)
  into the DTU clone crates — DTU clones are test infrastructure and must remain
  self-contained (established in Wave 3 S-3.4.* migration).
