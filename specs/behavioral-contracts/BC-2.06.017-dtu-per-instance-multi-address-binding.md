---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.017"
version: "1.12"
status: active
lifecycle_status: active
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-036"
introduced: "2026-06-09"
modified: "2026-07-17"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-DEMO-MULTI-TENANT-DTU-001]
verifying_vps: []
crates: [prism-dtu-demo-server, prism-dtu-harness, prism-dtu-common, prism-dtu-armis]
inputs:
  - ".factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.016-error-taxonomy-for-override-violations.md"
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "c0f1feb"
traces_to:
  - "CAP-036"
  - "ADR-031"
  - "ADR-029"
extracted_from: null
---

# BC-2.06.017: Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing

## Description

`prism-dtu-demo-server` and `prism-dtu-harness` must each support binding multiple
named DTU clone instances of the same sensor type to distinct socket addresses. This
enables end-to-end verification that the per-org `base_url` overlay mechanism
(BC-2.06.014 / ADR-029) routes tenant A's queries exclusively to instance A and
tenant B's queries exclusively to instance B — satisfying the ADR-031 §D5
DTU=true-DTU fidelity principle for multi-tenant deployments. The multi-address
binding API is strictly additive: the `BehavioralClone::start_on` signature is
unchanged and all existing single-instance callers continue to work without modification.

## Preconditions

**For `prism-dtu-demo-server` multi-instance bind:**
- A `MultiInstanceConfig` value has been constructed with at least one `InstanceEntry`
  (or zero entries, which is valid — see EC-017-002).
- Each `InstanceEntry` carries a non-empty `name: String` and a `bind: SocketAddr`
  (typically `127.0.0.1:0` for ephemeral port selection by the OS).
- `BehavioralClone::start_on(bind, shutdown, tls)` is available for each clone type
  in the configuration (unchanged signature per INV-COMPAT-001).

**For `prism-dtu-harness` `MultiInstanceHarness`:**
- `prism-dtu-harness` does NOT depend on `prism-spec-engine`, `prism-sensors`, or
  `prism-query` (forbidden dependency perimeter per story §Forbidden Dependencies).
- Each `HarnessEntry` in the `entries: Vec<HarnessEntry>` argument carries:
  `org_slug: String`, `sensor_id: String`, and `clone: Box<dyn BehavioralClone>`.
- The calling test has registered each `org_slug` in `OrgRegistry` before invoking
  `SpecLoader::load_all` with the overlay temp directory.

## Postconditions

### Postcondition 1 — Multi-instance bind (demo-server)

Given a `MultiInstanceConfig` with N `InstanceEntry` items (N ≥ 1), when the
multi-instance bind function (`start_instances`) runs:

- Each `InstanceEntry` starts exactly one clone instance via
  `BehavioralClone::start_on(entry.bind, shutdown_tx.subscribe(), tls)`, where
  `shutdown_tx` is a SINGLE shared `broadcast::Sender<()>` owned by the returned
  lifecycle handle (NOT a per-instance channel).
- The OS assigns an ephemeral port for entries specifying `127.0.0.1:0`; the resulting
  `SocketAddr` is captured immediately after bind.
- The function returns `Ok(MultiInstanceServers)` — a lifecycle handle that:
  - Owns the single shared `shutdown_tx: broadcast::Sender<()>` and all N task handles.
  - Exposes `servers.socket_map() -> &HashMap<String, SocketAddr>` where:
    - Each key is `entry.name` (the name string from `InstanceEntry`).
    - Each value is the OS-assigned bound `SocketAddr` for that instance.
    - All N instances are present in the map; no entries are silently dropped.
  - Exposes `servers.admin_token_map() -> &HashMap<String, String>` where:
    - Each key is `entry.name` (the name string from `InstanceEntry`).
    - Each value is the admin token (`clone.admin_token().to_string()`) captured from the
      `BoundInstance` BEFORE the clone is moved into its detached watcher task — same
      ownership pattern as `socket_map`. Once inside `tokio::spawn(async move { … })`,
      the clone is unreachable from the `start_instances` call site; extraction must
      happen in the bind loop before the spawn.
    - All N instances are present in the map; no entries are silently dropped.
  - The `admin_token_map` feeds the `TOKEN_MULTI_FILE` token sidecar written atomically
    by `cmd_start_multi` (tmp+rename, same atomic-write pattern as the URL sidecar; cf.
    GAP-3 sidecar-poll note, S-DEMO-LAUNCHER-CONSOLIDATION-001 Changelog v2.1). Sidecar format:
    `{org_slug: {sensor_id: token}}` — nested JSON mirroring `URL_MULTI_FILE`.
  - Triggers graceful shutdown of ALL instances when either:
    - `servers.shutdown()` is called explicitly, OR
    - the `MultiInstanceServers` value is dropped.
  - Graceful shutdown uses axum's `with_graceful_shutdown` pattern: the shared
    `shutdown_tx` sends a signal, all instances drain in-flight requests, and then
    their bound ports are released. This guarantees no zombie/leaked instances on the
    success path — the success-path analogue of Postcondition 6's "no partial-bound
    zombie instances on bind failure" guarantee, and consistent with EC-017-005
    (which applies to both `MultiInstanceHarness` Drop and `MultiInstanceServers` Drop).
- All N instances begin serving requests before the function returns.
- Each instance is addressable independently — a request to instance A's `SocketAddr`
  is served by instance A's clone; instance B's `SocketAddr` is served by instance B's
  clone.

### Postcondition 2 — Multi-instance harness (prism-dtu-harness)

Given `MultiInstanceHarness::start(entries).await` with M `HarnessEntry` items (M ≥ 1):

- Each entry starts exactly one clone via `entry.clone.start_on(ephemeral_addr, shutdown, tls)`.
- The function returns `Ok(MultiInstanceHarness)` where:
  - `harness.socket_map()` returns `&HashMap<(String, String), SocketAddr>` (plain `(org_slug, sensor_id)` strings — lightweight test-infra key per U-004 / D-1075; intentionally distinct from the production `OrgKey = (OrgId, DtuType)`, which is NOT used here).
  - Each `(org_slug, sensor_id)` key maps to the OS-assigned `SocketAddr` for that entry.
- The returned `socket_map` can be consumed to construct per-org TOML overlay files
  via the overlay wiring helper
  (`overlay_wiring::write_overlay_temp_dir(&harness, tempdir)`) **without** importing
  `prism-spec-engine` types — the helper writes raw TOML strings only.
  Each written overlay file contains exactly three required fields (raw TOML strings,
  per INV-PERIMETER-001):
  ```toml
  extends = "{sensor_id}"
  instance_id = "{sensor_id}@{org_slug}"
  base_url = "http://{socket_addr}"
  ```
  `extends` is **mandatory** — it binds the overlay to its TYPE spec so that
  `OverlayLoader` can resolve the base sensor configuration; an overlay without
  `extends` fails OverlayLoader validation.
  `instance_id` is **mandatory** and must equal `"{sensor_id}@{org_slug}"` per
  INV-SCALAR-003 (scalar-value uniqueness invariant).
  A `base_url`-only overlay (omitting `extends` and `instance_id`) would fail
  `OverlayLoader::load_overlays` — AC-005 asserts zero OverlayLoader errors.

### Postcondition 3 — Overlay integration end-to-end

Given two harness instances `(acme, armis)` and `(contoso, armis)` started via
`MultiInstanceHarness` with distinct `SocketAddr`s S_A and S_B (S_A ≠ S_B):

- After `overlay_wiring::write_overlay_temp_dir` writes overlay TOML files to a
  temp directory and `SpecLoader::load_all` is called on that directory:
  - `ResolvedSensorSpec` for `(acme, armis)` has `base_url = "http://S_A"`.
  - `ResolvedSensorSpec` for `(contoso, armis)` has `base_url = "http://S_B"`.
  - The two `base_url` values are distinct (S_A ≠ S_B by OS ephemeral port assignment).

### Postcondition 4 — No-cross-tenant-leakage (INV-ISOLATION-001)

Given two harness instances with distinct `SocketAddr`s S_A and S_B, when HTTP requests
addressed to S_A are dispatched, they reach S_A exclusively; zero requests addressed to
S_A reach S_B (symmetrically for S_B). Verification: server-side per-instance request
counter via `GET /dtu/request-count`.

Note: this is the **DISTINCT-LISTENER ISOLATION** proof. The proof that `FanOutTarget`
consumes per-org `base_url` at HTTP dispatch is provided by
`test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` in
`prism-sensors/src/fanout.rs` (S-CONFIG-MULTI-TENANT-OVERRIDE-001); the DTU-grounded
two-instance end-to-end routing proof is
`test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` in
`crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs` (F-PR3-HIGH-001 fix).

**This is the distinct-listener isolation proof at the DTU level.** The end-to-end
`FanOutTarget` routing proof lives in
`prism-sensors/tests/multi_tenant_dtu_routing_integration.rs` (F-PR3-HIGH-001 combined fix).

**INV-ISOLATION-001 Verification Mechanism (prism-dtu-armis):** The no-cross-tenant-leakage
invariant is verified via a server-side per-instance request counter added to
`prism-dtu-armis` as part of the F-P1-HIGH-001 isolation-counter fix. Each `ArmisClone`
instance maintains an `AtomicU64` request counter that is incremented on every received
HTTP request by an outermost axum middleware (`count_request_middleware`). The counter is
exposed as a DTU control-plane route: `GET /dtu/request-count` returns the total number
of requests the instance has received since startup. The isolation proof reads each
instance's actual received-request count via this route and asserts that the wrong-instance
count equals zero — i.e., `requests_received_by(S_B, dispatched_for_acme) == 0` and
`requests_received_by(S_A, dispatched_for_contoso) == 0`. This is why `prism-dtu-armis`
appears in the `crates:` frontmatter array for this BC.

### Postcondition 5 — Single-instance backward-compat (INV-COMPAT-001)

The multi-instance API is additive. After this story merges:

- `BehavioralClone::start_on(&mut self, bind: SocketAddr, shutdown: Option<broadcast::Receiver<()>>, tls: Option<Arc<RustlsConfig>> /*cfg(tls)*/ | Option<()> /*cfg(not(tls))*/) -> anyhow::Result<SocketAddr>` — signature unchanged.
- All existing callers of `ArmisClone::start_on(...)`, `ClarotyClone::start_on(...)`,
  etc. compile and behave identically to before.
- All parity tests in S-6.07 through S-6.10 and S-DEMO-002 integration tests pass
  without modification.
- No new required parameters are added to `BehavioralClone::start_on` by this story.

### Postcondition 6 — Multi-error aggregation on bind failure (INV-ERR-003-COMPAT)

If one or more `InstanceEntry` bind operations fail (e.g., EADDRINUSE on a non-zero
bind address), the multi-instance bind function:

- Attempts to bind ALL instances before returning any error.
- Collects all bind failures into a single `Vec<DemoBindError>` where each `DemoBindError`
  carries `instance_name: String` and the underlying `source: std::io::Error`.
- Returns `Err(MultiInstanceBindError::BindFailure(Vec<DemoBindError>))` containing all
  failures; does NOT short-circuit at the first error.
- For instances that DID bind successfully before other failures were detected:
  their tasks are shut down and their sockets released before the error is returned
  (no partial-bound zombie instances).

This is consistent with INV-ERR-003 from BC-2.06.016 (multi-error aggregation for
overlay violations) applied to the bind lifecycle.

### Postcondition 7 — Duplicate key semantics (EC-017-003)

If `MultiInstanceHarness::start` receives two `HarnessEntry` items with the same
`(org_slug, sensor_id)` key, the function returns
`Err(HarnessError::DuplicateKey { org_slug, sensor_id })` immediately, without starting
any clone instances. Silent last-wins behavior is forbidden — duplicate entries indicate
test-code misconfiguration that must be surfaced immediately. The error message must
name the conflicting `(org_slug, sensor_id)` pair verbatim.

The same rule applies to `MultiInstanceConfig`: duplicate `InstanceEntry::name` values
return `Err(MultiInstanceBindError::DuplicateName { name })` before any bind attempts.

## Invariants

### INV-ISOLATION-001 — No Cross-Tenant Leakage (Primary Isolation Invariant)

For any two org slugs O_A ≠ O_B and any sensor type S, given that
`ResolvedSensorSpec(O_A, S).base_url = "http://S_A"` and
`ResolvedSensorSpec(O_B, S).base_url = "http://S_B"` with S_A ≠ S_B:

```
requests_received_by_instance(S_A, query_for_org=O_B) = 0
requests_received_by_instance(S_B, query_for_org=O_A) = 0
```

This invariant holds regardless of query count, concurrency, or ordering. Cross-tenant
leakage is defined as any HTTP request reaching instance X that was dispatched under
an org whose `base_url` resolves to a different instance Y ≠ X.

Scope: enforced by BC-2.06.014 (endpoint resolution correctness) + this BC
(correct per-instance socket addresses supplied to the overlay map) working together.

### INV-COMPAT-001 — Single-Instance Backward Compatibility

The `BehavioralClone::start_on` trait method signature is immutable for this story.
No addendum, wrapper, or default implementation changes are permitted that would alter
the calling convention of existing callers. The multi-instance API is expressed through
new structs (`MultiInstanceConfig`, `InstanceEntry`, `MultiInstanceHarness`,
`HarnessEntry`) and new functions/methods that call `start_on` internally — never by
modifying `start_on` itself.

Scope: enforced by compile-time — existing tests using `start_on` directly must
continue to compile and pass.

### INV-ERR-003-COMPAT — Multi-Error Aggregation on Bind Failure

All bind operations in a multi-instance start sequence are attempted before any error
is returned. The caller receives a complete picture of all failures, not just the first
one. This is the same principle as INV-ERR-003 in BC-2.06.016, applied to the DTU
bind lifecycle.

### INV-PERIMETER-001 — prism-dtu-harness Forbidden Dependencies

`prism-dtu-harness` MUST NOT gain `Cargo.toml` dependencies on `prism-spec-engine`,
`prism-sensors`, or `prism-query`. The overlay wiring helper (`overlay_wiring.rs`)
writes raw TOML strings to a `tempfile::TempDir` — it does NOT import `SensorInstanceOverlay`
or `ResolvedSensorSpec` types. The test that invokes `SpecLoader::load_all` calls it
directly in the test crate (which may depend on `prism-spec-engine`), not via the harness.

### INV-NONEXHAUSTIVE-001 — All New Public Types Are `#[non_exhaustive]`

`MultiInstanceConfig`, `InstanceEntry`, `MultiInstanceHarness`, `HarnessEntry`,
`MultiInstanceBindError`, `HarnessError`, `DemoBindError`, `BindError`,
`MultiInstanceServers`, and any other public struct or enum added by
this story carry `#[non_exhaustive]`. The `ci.yml EXPECTED` count for the
`tests/external/non-exhaustive-violation/` compile-fail gate is incremented by the count
of new non-exhaustive public types.

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `MultiInstanceBindError::BindFailure(Vec<DemoBindError>)` | One or more `InstanceEntry` bind operations fail (e.g., EADDRINUSE on a non-`0` port); `DemoBindError { instance_name: String, source: std::io::Error }` (crate: `prism-dtu-demo-server`) | All bind operations attempted; all failures collected; successful partial binds shut down; `Err(...)` returned with all failures enumerated per INV-ERR-003-COMPAT |
| `MultiInstanceBindError::DuplicateName { name }` | Two `InstanceEntry` values share the same `name` | Returned immediately before any bind attempt; no clone instances started |
| `HarnessError::DuplicateKey { org_slug, sensor_id }` | Two `HarnessEntry` values share the same `(org_slug, sensor_id)` pair | Returned immediately before any clone instances started |
| `HarnessError::BindFailure(Vec<BindError>)` | One or more harness clone bind operations fail; `BindError { org_slug: String, sensor_id: String, source: std::io::Error }` (crate: `prism-dtu-harness`) | Same multi-error aggregation as demo-server path; all failed + successful binds reported; successful binds shut down |
| EC-017-004 + downstream `E-SPEC-022` | Overlay written for `org_slug` not registered in `OrgRegistry` | `SpecLoader::load_all` emits `E-SPEC-022` (BC-2.06.015 / BC-2.06.016); harness test must register all org slugs in `OrgRegistry` before calling `load_all` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-017-001 | EADDRINUSE: one `InstanceEntry` specifies a non-zero port that is already in use | Multi-error aggregation: bind is attempted for all instances; the failing instance contributes a `BindError` to the returned `Vec`; all successfully started instances are shut down; `Err(MultiInstanceBindError::BindFailure(...))` returned |
| EC-017-002 | `MultiInstanceConfig` with zero `InstanceEntry` items | Returns `Ok(MultiInstanceServers)` with an empty `socket_map()` (no instances bound; no tasks spawned). A zero-instance config is a valid no-op. |
| EC-017-003 | Same `(org_slug, sensor_id)` pair appears in two `HarnessEntry` items | `Err(HarnessError::DuplicateKey { org_slug, sensor_id })` — explicit error surfacing misconfiguration; last-wins is forbidden (see Postcondition 7 rationale) |
| EC-017-004 | Overlay TOML written for an org slug not registered in `OrgRegistry` | `SpecLoader::load_all` emits `E-SPEC-022` (BC-2.06.015); this is correct behavior — test code must register all orgs used in overlays. The harness itself does not validate org registration (it is not permitted to import prism-spec-engine) |
| EC-017-005 | `MultiInstanceHarness` or `MultiInstanceServers` dropped while in-flight requests are outstanding (applies to both handles) | Async drop races with in-flight requests; the shutdown signal drains in-flight requests using the shutdown-timeout pattern (per story risk mitigation). All in-flight requests complete or receive connection-closed before the bound port is released |
| EC-017-006 | DTU clone instance crashes mid-test | Subsequent requests to that instance's `SocketAddr` receive `ConnectionRefused` or equivalent error. This is NOT a silent cross-tenant leakage event — the requesting org receives a structured error; INV-ISOLATION-001 is not violated (zero requests can reach a crashed instance) |
| EC-017-007 | Test misconfiguration: org A overlay points to instance B socket | All of org A's requests go to instance B. The leakage test `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage` correctly FAILS — detecting the misconfiguration. This is correct-by-design: the test validates overlay correctness, not the harness |
| EC-017-008 | 10+ named instances in `MultiInstanceConfig` (large multi-tenant scenario) | All instances bind successfully; no hard cap enforced by this BC; memory and bind time increase linearly; test execution time increases proportionally |
| EC-017-009 | Two `InstanceEntry` items with the same name string | `Err(MultiInstanceBindError::DuplicateName { name })` returned before any bind attempt (see Postcondition 7) |

## Canonical Test Vectors

| ID | Scenario | Setup | Expected Result |
|----|----------|-------|----------------|
| TV-017-001 | Two armis instances at distinct ephemeral ports | `MultiInstanceConfig { instances: [InstanceEntry { name: "armis-acme", bind: "127.0.0.1:0" }, InstanceEntry { name: "armis-contoso", bind: "127.0.0.1:0" }] }` | `Ok(map)` where `map["armis-acme"] != map["armis-contoso"]`; both are valid loopback `SocketAddr`s |
| TV-017-002 | Zero instances | `MultiInstanceConfig { instances: [] }` | `Ok(MultiInstanceServers)` whose `socket_map()` is empty |
| TV-017-003 | Harness builds per-org socket map | `MultiInstanceHarness::start(vec![HarnessEntry { org_slug: "acme", sensor_id: "armis", clone: ArmisClone::new() }, HarnessEntry { org_slug: "contoso", sensor_id: "armis", clone: ArmisClone::new() }])` | `Ok(harness)` where `harness.socket_map()[("acme", "armis")] != harness.socket_map()[("contoso", "armis")]` |
| TV-017-004 | Zero cross-tenant leakage | Two `ArmisClone` instances at S_A and S_B; per-org overlays pointing to S_A for acme and S_B for contoso; 10 requests dispatched for each org | Instance at S_A receives exactly 10 requests tagged for acme; instance at S_B receives exactly 10 requests tagged for contoso; zero requests cross the tenant boundary |
| TV-017-005 | EADDRINUSE multi-error | `MultiInstanceConfig` with two entries: one `127.0.0.1:0` (succeeds) + one `127.0.0.1:{in_use_port}` (fails) | `Err(MultiInstanceBindError::BindFailure(failures))` where `failures.len() == 1` and `failures[0].instance_name == "the_failing_entry"` |
| TV-017-006 | Duplicate name error | `MultiInstanceConfig { instances: [InstanceEntry { name: "dup", ... }, InstanceEntry { name: "dup", ... }] }` | `Err(MultiInstanceBindError::DuplicateName { name: "dup" })` before any bind attempt |
| TV-017-007 | Duplicate harness key error | `MultiInstanceHarness::start([HarnessEntry { org_slug: "acme", sensor_id: "armis" }, HarnessEntry { org_slug: "acme", sensor_id: "armis" }])` | `Err(HarnessError::DuplicateKey { org_slug: "acme", sensor_id: "armis" })` before any clone started |
| TV-017-008 | Single-instance path unaffected | `ArmisClone::start_on("127.0.0.1:0".parse().unwrap(), Some(rx), None)` called directly (existing pattern; `None` for tls = plain HTTP, `shutdown` is `Option<broadcast::Receiver<()>>`) | Compiles and runs identically to pre-story state; `SocketAddr` returned; no regression |
| TV-017-009 | Overlay TOML integration (Postcondition 3) | `write_overlay_temp_dir(&harness, &tempdir)` produces `customers/acme/armis.sensor.toml` with all three required fields — `extends = "armis"`, `instance_id = "armis@acme"`, `base_url = "http://127.0.0.1:{S_A_port}"` — and `customers/contoso/armis.sensor.toml` with `extends = "armis"`, `instance_id = "armis@contoso"`, `base_url = "http://127.0.0.1:{S_B_port}"` | `SpecLoader::load_all(tempdir.path())` (via `OverlayLoader::load_overlays`, which requires `extends` to resolve the TYPE spec and `instance_id` per INV-SCALAR-003) yields zero errors and `ResolvedSensorSpec` entries for `(acme, armis)` and `(contoso, armis)` with the expected distinct `base_url` values; a `base_url`-only overlay missing `extends`/`instance_id` would produce a non-zero error count (AC-005 failure) |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none yet) | `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage` — integration test that starts two `ArmisClone` instances via `MultiInstanceHarness`, constructs per-org overlays, and asserts distinct-listener TCP isolation: HTTP requests addressed to S_A reach S_A exclusively; zero requests addressed to S_A reach S_B (symmetrically for S_B). Verification via server-side per-instance `GET /dtu/request-count`. This is the canonical INV-ISOLATION-001 **distinct-listener isolation** verification at the DTU harness level. |
| (none yet) | FanOutTarget→base_url routing dispatch proof: provided by `test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` in `prism-sensors/src/fanout.rs` (S-CONFIG-MULTI-TENANT-OVERRIDE-001, PR #155). DTU-grounded two-instance end-to-end routing proof (FanOutTarget + real DTU sockets): `test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` in `crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs` (F-PR3-HIGH-001 fix-burst). These live outside this BC's harness crates by design (INV-PERIMETER-001). |
| (none yet) | Compile-time test: existing single-instance `start_on` callers compile without modification after multi-instance API is added (INV-COMPAT-001 verification). |

## Related BCs

- BC-2.06.014 — Instance Identity Resolution at Fanout: produces the `ResolvedSensorSpec` map from `base_url` values that this BC's overlay wiring helper writes; INV-ISOLATION-001 depends on both BCs cooperating correctly.
- BC-2.06.012 — Per-Tenant Overlay Loading and Merge Semantics: `SpecLoader::load_all` semantics that the overlay wiring helper's TOML output must satisfy.
- BC-2.06.015 — OrgRegistry Cross-Validation at Boot: EC-017-004 org-registration precondition cited here.
- BC-2.06.016 — Error Taxonomy for Override Violations: INV-ERR-003-COMPAT borrows the multi-error aggregation principle from INV-ERR-003.
- BC-3.5.002 — Harness Network Isolation Invariants: CAP-036 canonical network isolation test; this BC's INV-ISOLATION-001 is the end-to-end complement.
- BC-2.01.010 — Partial Failure Handling: fanout errors from unreachable instances attributed per `instance_id` using the `instance_id = "{sensor_id}@{org_slug}"` logging pattern from BC-2.06.014.

## Architecture Anchors

- ADR-029 §At query time: proof that per-org `base_url` reaches `FanOutTarget` HTTP dispatch is delivered by `test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` in `prism-sensors/src/fanout.rs` (S-CONFIG-MULTI-TENANT-OVERRIDE-001, PR #155). The INV-PERIMETER-001 perimeter means this story's harness tests cannot exercise `FanOutTarget` directly; that is by design. The harness tests prove distinct-listener TCP isolation. The cross-layer E2E proof (harness DTU sockets + `FanOutTarget` dispatch) is delivered by `test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` in `crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs` (F-PR3-HIGH-001 fix-burst).
- ADR-029 §D1: `MultiInstanceConfig` / `InstanceEntry` in `prism-dtu-demo-server/src/multi_instance.rs`; `MultiInstanceHarness` in `prism-dtu-harness/src/multi_instance.rs`.
- ADR-031 §D5: parity tests for every sensor MUST assert real API endpoint routing — this BC's overlay integration test is the DTU-level parity assertion for multi-tenant endpoint routing.
- ADR-031 §D7: scope extension — harness clones are in-scope for DTU=true-DTU; `prism-dtu-harness` behavioral clones must exercise real socket routing.
- CLAUDE.md §`#[non_exhaustive]` discipline: INV-NONEXHAUSTIVE-001 codifies the CLAUDE.md requirement that all new public structs in `prism-dtu-*` crates carry `#[non_exhaustive]`.
- CLAUDE.md §reqwest::Client timeout: any new HTTP client in test helpers spawned by this story uses `.timeout(Duration::from_secs(30))`.
- ARCH-INDEX §SS-01 Subsystem Registry: `prism-dtu-demo-server` and `prism-dtu-harness` are SS-01 crates (Sensor Adapters); this BC is authored under SS-01.

## Story Anchor

S-DEMO-MULTI-TENANT-DTU-001

## VP Anchors

(None yet — VP to be authored by test-writer alongside S-DEMO-MULTI-TENANT-DTU-001)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 — this BC defines the API contract for the `MultiInstanceConfig` / `MultiInstanceHarness` components that constitute the multi-address binding extension of the harness. CAP-036 reads: "orchestrates per-customer DTU clone instances with two isolation modes: `Logical` (in-process, org-keyed, microsecond spinup) and `Network` (per-TCP-port, loopback, catches HTTP routing bugs)"; this BC specifies exactly the `Network` isolation mode's per-TCP-port binding API and the no-cross-tenant-leakage invariant that validates it. The "canonical cross-tenant fidelity test (BC-3.5.002) verifies that `devices(OrgA) ∩ devices(OrgB) = ∅` across all registered orgs under `IsolationMode::Network`" directly motivates INV-ISOLATION-001 here. |
| L2 Invariants | DI-008 (client data separation — each org resolves to its own `ResolvedSensorSpec` / `SocketAddr`; zero cross-org HTTP dispatch is the DTU-level enforcement of this invariant) |
| L2 Entities | MultiInstanceConfig, InstanceEntry, MultiInstanceHarness, HarnessEntry, ResolvedSensorSpec, FanOutTarget, BehavioralClone |
| Priority | P2 |
| ADR | ADR-029 (Multi-Tenant Sensor Endpoint Overrides), ADR-031 (DTU=True-DTU Fidelity Principle) |
| Source-of-Truth Precedence | This BC is the canonical specification for multi-instance binding semantics. Story S-DEMO-MULTI-TENANT-DTU-001 §Architecture Mapping sketches are superseded by this BC for contract semantics; the story governs implementation scope only. |

## Flag 2 Decision Notes (PO in-scope resolution)

**Flag 2 from story §New-BC Flags:** "Does BC-2.06.014 need a new postcondition for the
'multi-DTU routing verified end-to-end' scenario?"

**Decision: BC-2.06.014 is NOT amended.**

**Rationale:**

BC-2.06.014 specifies the *production* fanout resolution contract: how the query engine
resolves `(org_id, sensor_id)` → `ResolvedSensorSpec` at dispatch time and uses
`base_url` for HTTP dispatch. Its existing Postcondition Case A already states: "The
sensor adapter's HTTP client uses the overlay `base_url` for this dispatch" and its
Canonical Test Vector "Two-org fanout" describes parallel dispatches to distinct
`base_url`s. BC-2.06.014 is correct and complete at its level of abstraction.

This new BC (BC-2.06.017) is the *test-infrastructure* contract that verifies
BC-2.06.014's production claim against real sockets rather than mock HTTP. The
distinction is:

- BC-2.06.014 specifies WHAT must happen (HTTP dispatch uses overlay `base_url`).
- BC-2.06.017 specifies the TEST APPARATUS that PROVES it happens (real sockets,
  counted requests, INV-ISOLATION-001 assertion).

Adding a "multi-DTU routing verified end-to-end" postcondition to BC-2.06.014 would
conflate the production behavior contract with the test infrastructure contract — a
category error. BC-2.06.017's Postcondition 4 (INV-ISOLATION-001) is the correct home
for the end-to-end verification claim, with BC-2.06.014 cited as a dependency.

The `Related BCs` section of this BC (BC-2.06.017) links back to BC-2.06.014 with the
relationship "depends on: produces the `ResolvedSensorSpec` map that INV-ISOLATION-001
depends upon." That bidirectional reference is sufficient; no amendment to BC-2.06.014
is warranted.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.12 | DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 FIX-BURST-11 SPEC (F-ADMTOK-P12-MED-001) | 2026-07-17 | product-owner | Postcondition 1: corrected phantom §Sidecar-availability anchor (POL-21 violation). "§Sidecar-availability guarantee" was never a real heading in `.factory/`; the S-DEMO-LAUNCHER-CONSOLIDATION-001 launcher story heading is `## Changelog` (no §-sigil), and GAP-3 in that story's Changelog v2.1 is a cwd-path-threading note for demo-run.sh, not an atomic-write guarantee. Replaced with: `same atomic-write pattern as the URL sidecar; cf. GAP-3 sidecar-poll note, S-DEMO-LAUNCHER-CONSOLIDATION-001 Changelog v2.1`. The v1.11 row is NOT rewritten (do not rewrite history). Substance unchanged: admin-token sidecar write is still atomic tmp+rename; the correction is citation-form only. |
| 1.11 | DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 FIX-BURST-1 SPEC (F-ADMTOK-P1-MED-001) | 2026-07-16 | product-owner | Postcondition 1 extended: `MultiInstanceServers` surface now includes `admin_token_map() -> &HashMap<String, String>` accessor (tokens captured per-instance at bind time before the clone is moved into its detached watcher task, same ownership pattern as `socket_map`; once inside `tokio::spawn(async move { … })` the clone is unreachable). Also documents the `TOKEN_MULTI_FILE` admin-token sidecar mechanism: nested `{org_slug: {sensor_id: token}}` JSON, written atomically (tmp+rename) by `cmd_start_multi` from `admin_token_map()`, mirroring `URL_MULTI_FILE` format and the §Sidecar-availability guarantee (GAP-3 from S-DEMO-LAUNCHER-CONSOLIDATION-001). Closing F-ADMTOK-P1-MED-001 (MED: Postcondition 1 did not enumerate `admin_token_map` despite the accessor being mandated by AC-002). |
| 1.10 | F-PR4-MED-001 (PR-LEVEL adversary) | 2026-06-14 | product-owner | Corrected harness-test citation symbols: added the missing `test_BC_2_06_017_` infix to all `multi_tenant_routing` test references (EC-017-007, VP catalog) so they grep-resolve to the actual functions. Citation-accuracy only; no semantic change. |
| 1.9 | F-PR3-HIGH-001 (PR-LEVEL adversary, architect-adjudicated) | 2026-06-14 | product-owner | Narrowed Postcondition 4 + AC-006-anchored invariant + VP-catalog from "FanOutTarget dispatches" framing to the true DISTINCT-LISTENER isolation scope the harness tests actually prove; cross-referenced the FanOutTarget→base_url routing proofs (prism-sensors fanout test_F_LP2_CRIT_001 + new prism-sensors/tests E2E). §Architecture Anchors note added (architect). INV-ISOLATION-001 invariant unchanged; only in-harness verification scope clarified. No product-value reduction (routing proven in prism-sensors). |
| 1.8 | F-PR2-MED-001 (PR-LEVEL adversary, S-DEMO-MULTI-TENANT-DTU-001) | 2026-06-14 | product-owner | Reordered §Changelog table to monotonic-descending (newest-first) per POL-32 — was ascending since v1.0. No content change to any prior row; ordering only. |
| 1.7 | F-P8-HIGH-001 (LOCAL adversary Pass-8, S-DEMO-MULTI-TENANT-DTU-001) | 2026-06-13 | product-owner | crates: array += prism-dtu-armis (the INV-ISOLATION-001 proof depends on the prism-dtu-armis server-side request counter / GET /dtu/request-count route added by the F-P1-HIGH-001 isolation-counter fix). Postcondition 4 verification note added documenting the server-side counter proof mechanism: each ArmisClone instance maintains an AtomicU64 counter incremented by count_request_middleware on every received request; GET /dtu/request-count exposes the count; the isolation proof reads each instance's received-request count and asserts wrong-instance count == 0. Spec↔code scope sync; no contract semantic change. |
| 1.6 | consistency-audit (S-DEMO-MULTI-TENANT-DTU-001 pre-Pass-6) | 2026-06-13 | product-owner | Three spec-code alignment fixes: M-001 Preconditions HarnessEntry fields &str→String (owned, per D-1075/code); M-002 Architecture Anchors removed stale "server.rs" alternative (D-1075: no server.rs); N-001 INV-NONEXHAUSTIVE-001 explicit list completed (added DemoBindError, BindError, MultiInstanceServers — previously catch-all-only). No semantic change. |
| 1.5 | F-P5-MED-003 (LOCAL adversary Pass-5, S-DEMO-MULTI-TENANT-DTU-001) | 2026-06-13 | product-owner | Postcondition 3 + TV-017-009 overlay-format spec gap closed: overlay TOML requires 3 fields (extends, instance_id, base_url), not base_url-only. extends binds overlay to TYPE spec (OverlayLoader requirement); instance_id = {sensor_id}@{org_slug} (INV-SCALAR-003). Spec brought into alignment with the load-bearing overlay_wiring implementation + AC-005 test (which asserts zero OverlayLoader errors). Raw-TOML-string / INV-PERIMETER-001 unchanged. |
| 1.4 | F-P3-MED-001 (LOCAL adversary Pass-3, S-DEMO-MULTI-TENANT-DTU-001) | 2026-06-13 | product-owner | Postcondition 2 socket_map key type swept `(OrgSlug, SensorId)` → `(String, String)` to match U-004/D-1075 architect decision, AC-004, the story Locked API, and the implemented code. Stale newtype prose was never swept when U-004 chose plain test-infra strings. Annotation added: "lightweight test-infra key per U-004 / D-1075; intentionally distinct from the production OrgKey = (OrgId, DtuType)". No semantic change. |
| 1.3 | F-P1-MED-001 (LOCAL adversary Pass-1, S-DEMO-MULTI-TENANT-DTU-001) | 2026-06-13 | product-owner | EC-017-002 + TV-017-002 return-type sweep: Ok(HashMap::new()) → Ok(MultiInstanceServers) with empty socket_map() (v1.2 changed Postcondition 1 return type but missed these two siblings; POL-29 within-FB sweep). No semantic change. |
| 1.2 | D-1075-API-GAP-001 (architect adjudication, S-DEMO-MULTI-TENANT-DTU-001 TDD) | 2026-06-13 | product-owner | Postcondition 1 amended: `start_instances` returns `Ok(MultiInstanceServers)` lifecycle handle (was `Ok(HashMap<String,SocketAddr>)`). Handle owns single shared `shutdown_tx: broadcast::Sender<()>` + all N task handles; `servers.socket_map() -> &HashMap<String, SocketAddr>` accessor exposes bound addresses; `servers.shutdown()` / Drop trigger graceful drain (axum `with_graceful_shutdown`) then port release, eliminating success-path zombie/leaked instances. This is the success-path analogue of Postcondition 6 ("no partial-bound zombie instances on bind failure") and is consistent with EC-017-005 (parenthetical added: "applies to both `MultiInstanceHarness` Drop and `MultiInstanceServers` Drop"). No change to Postconditions 2-7, other ECs, INV-COMPAT-001, INV-ISOLATION-001, INV-ERR-003-COMPAT, INV-PERIMETER-001, or INV-NONEXHAUSTIVE-001. |
| 1.1 | D-1075 (architect reconciliation — remove-uncertainty scan, S-DEMO-MULTI-TENANT-DTU-001 ledger T3 hardening) | 2026-06-09 | product-owner | Two accuracy fixes grounded in real `BehavioralClone::start_on` signature (clone.rs lines 71-84). No semantic or invariant changes. **Amendment 1 (Postcondition 5 / TV-017-008):** Corrected `start_on` prose signature from erroneous `(bind: SocketAddr, shutdown: Receiver<()>, tls: bool)` to actual `(&mut self, bind: SocketAddr, shutdown: Option<broadcast::Receiver<()>>, tls: Option<Arc<RustlsConfig>> / Option<()>) -> anyhow::Result<SocketAddr>`; updated TV-017-008 call site from `start_on(..., false)` to `start_on(..., Some(rx), None)`. INV-COMPAT-001 semantics unchanged — the correction confirms the signature IS already `Option`-typed, not that it changed. **Amendment 2 (Error table / Postcondition 6):** Disambiguated inner aggregate error type names to avoid cross-crate name collision: demo-server uses `DemoBindError { instance_name: String, source: std::io::Error }` in `MultiInstanceBindError::BindFailure(Vec<DemoBindError>)`; harness uses `BindError { org_slug: String, sensor_id: String, source: std::io::Error }` in `HarnessError::BindFailure(Vec<BindError>)`. Variant names (HarnessError::DuplicateKey, HarnessError::BindFailure, MultiInstanceBindError::DuplicateName, MultiInstanceBindError::BindFailure) confirmed correct per architect. |
| 1.0 | D-TBD (S-DEMO-MULTI-TENANT-DTU-001 PO authorship) | 2026-06-09 | product-owner | Initial draft. Resolves S-7.01 Spec-First Gate for S-DEMO-MULTI-TENANT-DTU-001. Covers: MultiInstanceConfig/InstanceEntry demo-server API (Postcondition 1), MultiInstanceHarness harness API (Postcondition 2), overlay TOML integration (Postcondition 3), INV-ISOLATION-001 no-cross-tenant-leakage invariant (Postcondition 4), INV-COMPAT-001 single-instance backward-compat invariant (Postcondition 5), multi-error aggregation on bind failure (Postcondition 6 / INV-ERR-003-COMPAT), and EC-017-003 duplicate-key-returns-error semantics (Postcondition 7). Flag 2 decision: BC-2.06.014 NOT amended — rationale in §Flag 2 Decision Notes. EC-003 decision: error-return on duplicate key (Postcondition 7). |
