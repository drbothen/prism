---
document_type: vp-tbd-decision-matrix
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00
phase: 2-patch
traces_to: architecture/ARCH-INDEX.md
inputs:
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.17.001 through BC-2.17.006
  - specs/behavioral-contracts/BC-2.18.001 through BC-2.18.009
  - specs/behavioral-contracts/BC-2.19.001 through BC-2.19.005
  - specs/behavioral-contracts/BC-2.08.006
  - specs/behavioral-contracts/BC-2.10.008
input-hash: TBD
---

# VP-TBD Decision Matrix

## Context

Pass-61 (LOW-001) identified 22 BCs across subsystems SS-08, SS-10, SS-17, SS-18, and SS-19
with `VP-TBD` placeholders in their `## Verification Properties` sections. This document
produces a disposition decision for each before Phase 3 implementation begins.

Current VP high-water mark: **VP-039** (all proposed additions use IDs VP-040 and above).

---

## Summary Table

| BC ID | Subsystem | Decision | Action Detail |
|-------|-----------|----------|---------------|
| BC-2.17.001 | SS-17 (WASM Plugin Runtime) | **B: MARK-NONE** | Integration behavior; both VPs are integration tests for wasmtime host-boundary behavior — no pure-function provable invariant distinct from what integration tests cover |
| BC-2.17.002 | SS-17 | **A: ADD-VP** | VP-040: WASI import absence is a linker configuration invariant provable by Kani on the `Linker` build function |
| BC-2.17.003 | SS-17 | **A: ADD-VP** | VP-041: Memory limit boundary (at-limit succeeds, over-limit fails) — proptest on StoreLimits configuration |
| BC-2.17.004 | SS-17 | **B: MARK-NONE** | Integration behavior; epoch-interruption correctness depends on wall-clock timing — integration test is correct vehicle; no pure function to prove |
| BC-2.17.005 | SS-17 | **A: ADD-VP** | VP-042: Hot reload atomicity — failed compile retains old plugin; same pattern as VP-032; proptest |
| BC-2.17.006 | SS-17 | **A: ADD-VP** | VP-043: WIT validation rejects plugin missing required exports; proptest over WIT export sets |
| BC-2.18.001 | SS-18 (Action Delivery) | **A: ADD-VP** | VP-044: At-least-once retry: attempt counter bounded by 5; dead-letter written after exhaustion; Kani |
| BC-2.18.002 | SS-18 | **B: MARK-NONE** | Best-effort / no-retry semantics are behavioral (absence of retry state write); no pure invariant beyond integration test |
| BC-2.18.003 | SS-18 | **B: MARK-NONE** | Fire-and-forget result delivery is integration behavior; correct implementation is the integration test; no provable property separable from the test |
| BC-2.18.004 | SS-18 | **A: ADD-VP** | VP-045: try_acquire (non-blocking) is used — never acquire (blocking); Kani proof on ActionEngine construction |
| BC-2.18.005 | SS-18 | **B: MARK-NONE** | Partial-report assembly is pure glue logic; invariant ("all sections present") covered by existing integration test; no proof harness adds material confidence |
| BC-2.18.006 | SS-18 | **B: MARK-NONE** | Injection-scan-before-interpolation is covered transitively by VP-024 (InjectionScanner detects patterns) and VP-028 (template interpolation never panics); this BC is an integration contract on top of proven components |
| BC-2.18.007 | SS-18 | **A: ADD-VP** | VP-046: Inline credential value produces E-ACTION-001 at load time; never reaches log/error; proptest over spec structures |
| BC-2.18.008 | SS-18 | **B: MARK-NONE** | Audit completeness is an integration concern — every path emits an entry; no provable function invariant distinguishable from integration test coverage |
| BC-2.18.009 | SS-18 | **A: ADD-VP** | VP-047: UUID v7 validation correctness — non-v7 values always dropped, v7 always included; proptest over UUID variants |
| BC-2.19.001 | SS-19 (Infusion Registry) | **A: ADD-VP** | VP-048: N fields → exactly N descriptors; no duplicates in output; Kani on descriptor-count invariant |
| BC-2.19.002 | SS-19 | **A: ADD-VP** | VP-049: Dedup call count = unique-value count; proptest over event sets with repeats |
| BC-2.19.003 | SS-19 | **B: MARK-NONE** | API-backed UDF rejection is a classification lookup (is_api_backed returns bool); already integration-tested; no formal proof gap |
| BC-2.19.004 | SS-19 | **B: MARK-NONE** | Hot reload atomicity for infusions follows the same arc-swap pattern as VP-032; transitively covered; adding a second VP with identical proof shape to VP-032 adds no new verification coverage |
| BC-2.19.005 | SS-19 | **B: MARK-NONE** | Credential redaction is a log-assertion integration test; the invariant is "value absent from string output" — a proptest could cover it but it duplicates the pattern proven for sensors in VP-011/BC-2.03.007 family |
| BC-2.08.006 | SS-08 (Diagnostics) | **B: MARK-NONE** | Read-only MCP resource returning cached data; the timestamp-equals invariant is integration behavior; the credential-absence property is already proven by VP-011 (credential name sanitization) and BC-2.05.003 policy enforced architecturally |
| BC-2.10.008 | SS-10 (MCP Resources) | **A: ADD-VP** | VP-050: MCP resource response never contains API-key-pattern strings; proptest over fabricated config with injected credential-like values |

**Aggregate counts: ADD-VP = 10, MARK-NONE = 12, DEFER = 0**

---

## Per-BC Analysis

### BC-2.17.001 — Plugin Panic Isolation (SS-17)

**Decision: B — MARK-NONE**

The two VP-TBD entries are:
1. "Plugin trap returns Err(Trapped) without terminating host" — integration test
2. "Host process unaffected after concurrent traps" — integration test

Both invariants are correct as integration tests: they require a live wasmtime engine, a real WASM trap fixture, and observation of the host process state. There is no separable pure function to prove. The invariant INV-PLUGIN-001 holds by construction of the wasmtime call wrapper (`catch_unwind` + trap conversion). The BC already specifies the test fixture (`trap_plugin.wat`).

**Replacement frontmatter text:**
```
(none — INV-PLUGIN-001 is enforced by wasmtime host-boundary construction; verified by integration test in tests/plugin_tests.rs; no pure-function invariant to prove formally)
```

---

### BC-2.17.002 — Plugin Sandbox: No Filesystem/Network (SS-17)

**Decision: A — ADD-VP**

The first VP-TBD ("WASI filesystem import causes load rejection") has a provable pure-function invariant: `PluginRuntime::new()` constructs a `Linker` that does NOT include any WASI interfaces. This is a static configuration property of the `Linker` at construction time, which is a function from `wasmtime::Engine` → `wasmtime::component::Linker<HostState>` with no side effects. A Kani proof can verify that the set of linked imports does not contain WASI namespace entries.

The second VP-TBD ("HTTP proxy routes through host reqwest client") is integration behavior — leave as integration test; no VP needed for that half.

**Proposed VP-040:**
- ID: VP-040
- Slug: `vp-040-plugin-linker-no-wasi-imports`
- Method: Kani
- Priority: P1
- Property: `PluginRuntime::build_linker()` produces a Linker whose import namespace set does not contain any `wasi:` prefixed interface name
- Source BC: BC-2.17.002

---

### BC-2.17.003 — Plugin Memory Limit (SS-17)

**Decision: A — ADD-VP**

The memory limit boundary has a clean pure-function invariant: `StoreLimits::new(memory_limit_bytes)` applied to a `Store` must allow allocation at exactly the limit and reject at limit+1. This is a parameterized boundary condition amenable to proptest (and Kani for the exact boundary). INV-PLUGIN-003 states the limit is enforced per-instance — the function `create_store_with_limit(limit_mb: u64) -> Store<HostState>` is pure in the relevant sense (its output's memory budget is deterministic from the input).

The second VP-TBD ("host process memory unaffected") remains an integration test.

**Proposed VP-041:**
- ID: VP-041
- Slug: `vp-041-plugin-memory-limit-boundary`
- Method: Proptest
- Priority: P1
- Property: For any `limit_mb` in 1..=512, a wasmtime Store configured with that limit allows WASM linear memory allocation up to `limit_mb * 1024 * 1024` bytes and returns a trap error at `limit_mb * 1024 * 1024 + 1` bytes attempted
- Source BC: BC-2.17.003

---

### BC-2.17.004 — Plugin CPU Time Limit (SS-17)

**Decision: B — MARK-NONE**

The invariant (epoch interruption fires within deadline + 1s tolerance) depends on wall-clock time and tokio task scheduling — inherently non-deterministic. The epoch ticker is a background async task; its correctness cannot be proven with Kani or proptest in a meaningful way. The BC correctly specifies integration test as the verification vehicle. The one potentially provable invariant (the epoch deadline is set exactly once per call) is too low-level to warrant a VP — it is a static code property reviewable by inspection.

**Replacement frontmatter text:**
```
(none — CPU time limit via epoch interruption is time-dependent; correct implementation is verified by integration test with infinite-loop WAT fixture in tests/plugin_tests.rs AC-3; no provable pure-function invariant)
```

---

### BC-2.17.005 — Plugin Hot Reload Atomic Swap (SS-17)

**Decision: A — ADD-VP**

"Failed reload retains previous working plugin" is structurally identical to VP-032 (hot reload atomicity for sensor specs). Both use arc-swap and the same CI-002 pattern. A proptest can verify: given a `PluginRegistry` with a valid plugin loaded, after attempting to reload with an invalid binary, the registry still returns the old `InstancePre`. This is testable without wasmtime execution — the invariant is about registry state, not WASM execution.

The first VP-TBD ("in-flight disruption") remains integration behavior.

**Proposed VP-042:**
- ID: VP-042
- Slug: `vp-042-plugin-hot-reload-failed-compile-retains-old`
- Method: Proptest
- Priority: P1
- Property: Given a PluginRegistry with a registered valid plugin, invoking `hot_reload(invalid_bytes)` that fails compilation leaves the registry entry unchanged: the old `Arc<LoadedPlugin>` is still returned for the plugin_id
- Source BC: BC-2.17.005

---

### BC-2.17.006 — WIT Interface Validation (SS-17)

**Decision: A — ADD-VP**

WIT validation has a pure-function core: `validate_wit_interface(component: &Component, required_exports: &[&str]) -> Result<PluginType, PluginError>` is deterministic. A proptest can generate synthetic component export sets (valid/missing/wrong-version permutations) and verify that all required exports present → Ok, any missing → Err(E-PLUGIN-001). This does not require a live wasmtime execution loop.

**Proposed VP-043:**
- ID: VP-043
- Slug: `vp-043-plugin-wit-validation-rejects-missing-exports`
- Method: Proptest
- Priority: P1
- Property: For any WASM Component with a strict subset of the required Prism WIT exports, `validate_wit_interface()` returns `Err(PluginError::InvalidInterface)` with a message naming the missing export; for a component with all required exports, returns `Ok(plugin_type)`
- Source BC: BC-2.17.006

---

### BC-2.18.001 — At-Least-Once Delivery with Retry (SS-18)

**Decision: A — ADD-VP**

The retry state machine has a provable invariant: the attempt counter is strictly bounded by 5 and the dead-letter transition fires exactly once after the 5th failure. This is a pure state machine with states {Pending(attempt: u32), DeadLettered} and transitions. Kani can prove:
- From Pending(n), n < 5: transition on failure → Pending(n+1)
- From Pending(5): transition on failure → DeadLettered
- DeadLettered is a terminal state (no further transitions)
- The state machine never reaches attempt count > 5

The persistence aspect (RocksDB write before delay) is integration behavior.

**Proposed VP-044:**
- ID: VP-044
- Slug: `vp-044-action-retry-state-machine-bounded-5-attempts`
- Method: Kani
- Priority: P0
- Property: The action delivery retry state machine never exceeds 5 attempts; the dead-letter transition fires exactly once after the 5th failure; the state is terminal after dead-lettering
- Source BC: BC-2.18.001

---

### BC-2.18.002 — Schedule Best-Effort Delivery (SS-18)

**Decision: B — MARK-NONE**

The "no catch-up on missed tick" invariant is temporal/behavioral: it means the cron scheduler does not write catch-up state on restart. This is a behavioral contract verified by an integration test (restart, observe no catch-up deliveries). There is no pure-function invariant: the absence of state writes is an integration observable, not a formal property amenable to Kani.

**Replacement frontmatter text:**
```
(none — best-effort / no-catch-up semantics are behavioral; verified by integration test in tests/action_tests.rs; no retry state is written by construction — pure absence-of-write not provable formally)
```

---

### BC-2.18.003 — Manual Fire-and-Forget (SS-18)

**Decision: B — MARK-NONE**

Fire-and-forget correctness is integration behavior: the MCP tool returns a result without spawning a retry task. The "no retry state persisted" invariant is an absence property, but it is trivially verified by code inspection and integration test (check that action_state CF has no new writes after a failed manual delivery). There is no separable pure-function property. The second VP-TBD ("non-manual action ID rejected with E-ACTION-007") is a simple lookup error — an integration test for a two-line type check does not warrant a VP.

**Replacement frontmatter text:**
```
(none — fire-and-forget semantics are behavioral; E-ACTION-007 trigger-type check is trivially integration-tested; no pure-function invariant warrants a formal VP)
```

---

### BC-2.18.004 — Schedule Semaphore try_acquire (SS-18)

**Decision: A — ADD-VP**

The critical invariant here is safety-structural: `ActionEngine::fire_schedule` MUST use `try_acquire()` (non-blocking), never `acquire()` (blocking). If `acquire()` is used anywhere, it could deadlock the cron tick loop — a safety-critical failure mode. This is expressible as a Kani proof: prove that the function body of `fire_schedule` calls `semaphore.try_acquire()` (not `semaphore.acquire().await`) by proving that the function does not await on a blocking semaphore acquire. In practice this would be a proptest/integration test that holds all permits and verifies `fire_schedule` returns immediately rather than blocking indefinitely.

**Proposed VP-045:**
- ID: VP-045
- Slug: `vp-045-schedule-semaphore-try-acquire-nonblocking`
- Method: Proptest
- Priority: P0
- Property: When all 16 semaphore permits are held, `ActionEngine::fire_schedule()` returns immediately (within 10ms) without acquiring a permit; it does not block or await on the semaphore
- Source BC: BC-2.18.004

---

### BC-2.18.005 — Partial Report Failure (SS-18)

**Decision: B — MARK-NONE**

The invariant ("all sections present, failed sections have error notes, report always delivered") is verified by an integration test with mock query execution. The assembly function is pure plumbing: it maps a list of `Result<Section, Error>` values to a list of `RenderedSection` values. This is a trivial iterator map; a VP adding formal proof would be over-engineering. The existing integration tests (TV-18-005-partial, TV-18-005-all-fail) are the correct verification vehicle.

**Replacement frontmatter text:**
```
(none — partial-report assembly is a trivial Result→RenderedSection map; invariant verified by integration test TV-18-005-partial and TV-18-005-all-fail in tests/action_tests.rs)
```

---

### BC-2.18.006 — Template Injection Scanning (SS-18)

**Decision: B — MARK-NONE**

This BC is pure plumbing over proven components:
- VP-024 proves `InjectionScanner` detects known injection patterns
- VP-028 proves template interpolation never panics
- VP-038 proves `InjectionScanner` never panics on arbitrary input

The integration-specific contract (scan-before-interpolate, trusted variables bypass scanner) is behavioral glue that does not add a new provable invariant beyond what VP-024/028/038 already cover. The "flag, don't strip" policy is enforced by BC-2.09.004 design.

**Replacement frontmatter text:**
```
(none — injection scanning covered transitively by VP-024 and VP-038; template rendering covered by VP-028; this BC is integration glue over proven components with no additional provable invariant)
```

---

### BC-2.18.007 — Action Credential Opaque Reference (SS-18)

**Decision: A — ADD-VP**

The inline-credential detection has a provable pure-function invariant: `validate_action_credential_fields(spec: &ActionSpec) -> Result<(), E-ACTION-001>` is deterministic. A proptest can generate ActionSpec structures with:
- Reference-form values (`{ source = "env", key = "..." }`) → must return Ok
- Inline string values (any non-reference form) → must return Err(E-ACTION-001) with field name only (not value) in error message

The second VP-TBD ("credential value not present in any log or error output") is a log-assertion integration test — covered by the proptest on the error message content.

**Proposed VP-046:**
- ID: VP-046
- Slug: `vp-046-action-inline-credential-rejection`
- Method: Proptest
- Priority: P0
- Property: For any ActionSpec where a credential field contains an inline string value (non-reference form), `validate_credential_fields()` returns Err(E-ACTION-001); the error message contains the field name and does not contain the field value; reference-form values always return Ok
- Source BC: BC-2.18.007

---

### BC-2.18.008 — Action Delivery Audit Logging (SS-18)

**Decision: B — MARK-NONE**

"Every action outcome produces an audit entry" is a completeness property — it requires enumerating all code paths through ActionEngine and verifying each emits an audit event. This is an integration test concern. The "credential values absent from audit entries" invariant is already policy-enforced by BC-2.05.003 and architecturally by the credential reference model; it is the same pattern as VP-011 (credential name sanitization). Adding a VP here would duplicate existing coverage.

**Replacement frontmatter text:**
```
(none — audit completeness is an integration test concern covering all ActionEngine code paths; credential-absence in audit entries covered by BC-2.05.003 policy + VP-046 credential rejection proving values never enter the system as bare strings)
```

---

### BC-2.18.009 — UUID v7 Validation for alert_ids_quoted (SS-18)

**Decision: A — ADD-VP**

UUID v7 validation is a classic proptest target: the validation function `validate_uuid_v7(value: &str) -> bool` is a pure predicate. A proptest can exhaustively verify:
- Valid UUID v7 strings → always accepted
- UUID v4 strings (valid format, wrong version) → always rejected
- Arbitrary strings (non-UUID payloads including injection patterns) → always rejected
- The invariant "order of valid UUIDs in output = order in input" is also verifiable

**Proposed VP-047:**
- ID: VP-047
- Slug: `vp-047-uuid-v7-validation-rejects-non-v7`
- Method: Proptest
- Priority: P0
- Property: `validate_uuid_v7(s)` returns true only for strings that parse as version-7 UUIDs; returns false for all UUID v4, v1, v6 values, all non-UUID strings (including SQL injection payloads), and empty strings; valid UUIDs in output preserve input order
- Source BC: BC-2.18.009

---

### BC-2.19.001 — Infusion Spec Loading (SS-19)

**Decision: A — ADD-VP**

The core invariant INV-INFUSE-001 ("each `[[infusion.fields]]` entry must produce exactly one `InfusionUdfDescriptor`") is provable by Kani. The function `InfusionRegistry::load_spec(spec: &InfusionSpec) -> Result<Vec<InfusionUdfDescriptor>>` is pure. Kani can prove: `|output.len()| == |spec.fields.len()|` when all fields are valid; and that duplicate UDF names produce a distinct Err(E-INFUSE-002) rather than silently collapsing.

**Proposed VP-048:**
- ID: VP-048
- Slug: `vp-048-infusion-spec-n-fields-n-descriptors`
- Method: Kani
- Priority: P1
- Property: `InfusionRegistry::load_spec()` with N valid, distinct field entries produces exactly N `InfusionUdfDescriptor` objects in the output; duplicate UDF names produce `Err(E-INFUSE-002)` rather than silently merging
- Source BC: BC-2.19.001

---

### BC-2.19.002 — Per-Query Dedup Cache (SS-19)

**Decision: A — ADD-VP**

The dedup invariant ("source calls = unique input count") is a pure property of the cache data structure. A proptest can generate:
- An event set of N rows with K unique IP values (K ≤ N)
- Invoke the dedup-wrapped enrich function
- Assert: enrich_single was called exactly K times
- Assert: dedup cache contains exactly K entries at query end

This is a proptest over the `HashMap`-based dedup logic with a mock `InfusionSource`, entirely independent of DataFusion.

**Proposed VP-049:**
- ID: VP-049
- Slug: `vp-049-infusion-dedup-calls-equal-unique-values`
- Method: Proptest
- Priority: P1
- Property: For any input sequence of N values containing K distinct values (1 ≤ K ≤ N ≤ 10000), the per-query dedup cache results in exactly K calls to `InfusionSource::enrich_single`; the dedup cache contains exactly K entries after processing
- Source BC: BC-2.19.002

---

### BC-2.19.003 — API-Backed UDF Rejection in Detection Rules (SS-19)

**Decision: B — MARK-NONE**

`InfusionRegistry::is_api_backed(udf_name)` is a simple lookup on the registry map. It is not a provable invariant beyond a unit test — there is no subtle logic to verify formally. The BC's invariant ("enforcement point is S-4.03, not prism-spec-engine") means the interesting verification is at the rule validator level, which is already an integration test concern for S-4.03.

**Replacement frontmatter text:**
```
(none — API-backed UDF classification is a trivial registry map lookup; enforcement at S-4.03 rule validator is integration-tested; no pure-function invariant warrants a formal VP)
```

---

### BC-2.19.004 — Infusion Hot Reload Atomicity (SS-19)

**Decision: B — MARK-NONE**

This BC is structurally identical to BC-2.16.005/VP-032 (hot reload atomicity for sensor specs). VP-032 already proves the arc-swap pattern: "failed validation retains old config." Adding VP-050 with an identical proof harness skeleton targeting `InfusionRegistry` instead of `SpecConfig` would be pure duplication. The implementation of CI-002 across all three subsystems (sensor specs, plugins, infusions) will use the same arc-swap wrapper function — a single VP-032 pattern proving the wrapper is sufficient.

Human input flag: If the InfusionRegistry hot reload implementation diverges significantly from the VP-032 arc-swap pattern (e.g., it adds multi-step UDF deregistration/reregistration logic not present in sensor spec reload), a separate VP targeting that additional complexity would be warranted. Recommend human confirm scope before Phase 3.

**Replacement frontmatter text:**
```
(none — CI-002 arc-swap hot reload pattern proven by VP-032; InfusionRegistry uses same arc-swap wrapper; if UDF deregistration logic diverges from VP-032's coverage, revisit during Phase 3)
```

---

### BC-2.19.005 — Infusion Credential Redaction (SS-19)

**Decision: B — MARK-NONE**

This invariant is the same security policy as BC-2.03.007 (sensor credential redaction) and BC-2.18.007 (action credential opaque reference). The pattern is already proven by VP-046 (which proves that inline credential values produce E-ACTION-001 rather than propagating) and the architectural `#[debug = "<redacted>"]` derivation. Adding a separate VP targeting infusion credentials would duplicate VP-046's coverage with identical proof shape. The log-capture integration test (AC-6) is the correct verification vehicle for this specific subsystem.

**Replacement frontmatter text:**
```
(none — credential redaction policy proven architecturally and by VP-046 for the action layer; infusion credential log-capture verified by integration test AC-6 in tests/infusion_tests.rs; no additional formal VP)
```

---

### BC-2.08.006 — Health Status MCP Resource (SS-08)

**Decision: B — MARK-NONE**

The two VP-TBD entries are:
1. "Resource never returns credential values or full API URLs" — already enforced architecturally: the health resource returns `SensorHealthResult` structs which have no credential fields by type design (the type does not contain a credential field). VP-011 (credential name sanitization) plus BC-2.05.003 (credential values never in audit entries) provide the architectural guarantee. No additional formal VP is needed.
2. "`last_checked_at` equals timestamp from most recent check_sensor_health" — this is a cache-read correctness invariant, verifiable only via integration test (read resource, compare to tool invocation timestamp). Not a pure-function property.

**Replacement frontmatter text:**
```
(none — credential-absence guaranteed by SensorHealthResult type design (no credential fields); timestamp-equals-cache invariant is integration behavior; covered by integration test in tests/health_tests.rs)
```

---

### BC-2.10.008 — MCP Resources for Client List and Sensor Inventory (SS-10)

**Decision: A — ADD-VP**

The first VP-TBD ("response never contains API-key-pattern strings") is a proptest target: `render_sensor_resource(config: &ClientSensorConfig) -> SensorInventoryResponse` is a pure function. A proptest can:
- Generate configs with fabricated credential-like values (API key patterns, bearer tokens)
- Invoke the render function
- Assert: the serialized JSON response matches zero occurrences of known API key patterns (UUID-format tokens, Bearer header values, base64-like strings)
- Assert: the API base URL field contains only the host component, not the full URL with path/query

The second VP-TBD ("full API base URL never appears; only host") strengthens the first and is part of the same proptest.

**Proposed VP-050:**
- ID: VP-050
- Slug: `vp-050-mcp-resource-sensor-response-redacts-credentials`
- Method: Proptest
- Priority: P0
- Property: `render_sensor_inventory_resource()` given a ClientSensorConfig containing full API base URLs and credential values produces a response JSON where: (a) no string matching an API key pattern (UUID token, Bearer prefix, base64 32+ chars) appears; (b) the API base URL field contains only the host+port component, not the full URL path or credentials
- Source BC: BC-2.10.008

---

## VP-INDEX Additions

The following 11 rows should be appended to VP-INDEX.md when this matrix is approved. IDs VP-040 through VP-050.

| ID | Property | Module | Method | Priority | Status | Anchor Story |
|----|----------|--------|--------|----------|--------|--------------|
| VP-040 | Plugin Linker excludes all WASI namespace imports | prism-spec-engine | kani | P1 | proposed | S-1.15 |
| VP-041 | Plugin memory limit boundary: at-limit succeeds, over-limit traps | prism-spec-engine | proptest | P1 | proposed | S-1.15 |
| VP-042 | Plugin hot reload: failed compile retains old InstancePre | prism-spec-engine | proptest | P1 | proposed | S-1.15 |
| VP-043 | WIT validation rejects component missing required exports | prism-spec-engine | proptest | P1 | proposed | S-1.15 |
| VP-044 | Action retry state machine: bounded by 5 attempts, dead-letter terminal | prism-operations | kani | P0 | proposed | S-4.08 |
| VP-045 | Schedule semaphore: try_acquire used (non-blocking), never acquire | prism-operations | proptest | P0 | proposed | S-4.08 |
| VP-046 | Action inline credential rejected at load time; value not in error message | prism-operations | proptest | P0 | proposed | S-4.08 |
| VP-047 | UUID v7 validation: non-v7 always rejected, v7 always accepted, order preserved | prism-operations | proptest | P0 | proposed | S-4.08 |
| VP-048 | Infusion spec: N fields produces exactly N UDF descriptors; duplicates error | prism-spec-engine | kani | P1 | proposed | S-1.14 |
| VP-049 | Infusion per-query dedup: source calls = unique value count | prism-spec-engine | proptest | P1 | proposed | S-1.14 |
| VP-050 | MCP sensor resource response redacts credentials and full API URLs | prism-mcp | proptest | P0 | proposed | S-5.03 |

**Updated method totals (if all 11 accepted):**

| Method | Current Count | +New | Total |
|--------|--------------|------|-------|
| Kani | 20 | +2 (VP-040, VP-044) | 22 |
| Proptest | 11 | +8 (VP-041–043, VP-045–047, VP-049–050) | 19 |
| Fuzz | 6 | 0 | 6 |
| Integration test | 2 | 0 | 2 |
| **Total** | **39** | **+10** | **49** |

Note: VP-048 uses Kani (+1 Kani). Retally: Kani = 22, Proptest = 18, Fuzz = 6, Integration test = 2, Total = 48.

Corrected totals:

| Method | Current Count | +New | Total |
|--------|--------------|------|-------|
| Kani | 20 | +3 (VP-040, VP-044, VP-048) | 23 |
| Proptest | 11 | +7 (VP-041–043, VP-045–047, VP-049–050) | 18 |
| Fuzz | 6 | 0 | 6 |
| Integration test | 2 | 0 | 2 |
| **Total** | **39** | **+10** | **49** |

---

## Contentious Decisions — Human Input Requested

### 1. BC-2.19.004 (Infusion Hot Reload Atomicity) — MARK-NONE

Flagged as MARK-NONE because VP-032 proves the same arc-swap pattern. However, the InfusionRegistry hot reload involves an additional step: notifying `prism-query` to deregister old UDFs and register new ones (EC-19-014). This multi-step deregistration/registration sequence is NOT covered by VP-032. If implementation uses a different code path (not the shared arc-swap wrapper), a dedicated VP targeting the UDF re-registration atomicity would be warranted.

**Recommendation:** Confirm during Phase 3 story planning whether `InfusionRegistry::hot_reload` shares code with the VP-032 subject (`SpecConfig::hot_reload`) or has independent logic. If independent → add VP-051 targeting the UDF deregistration step.

### 2. BC-2.17.002 (Plugin Sandbox) — ADD-VP (VP-040, Kani)

The Kani proof target is "Linker's import namespace contains no wasi: entries." This is feasible IF the Linker construction function is deterministic and wasmtime's `Linker` exposes an import enumeration API for Kani to reason over. If wasmtime's `Linker` type is opaque to Kani (e.g., uses unsafe internals), the proof may need to degrade to a proptest (check that loading a WASI-import-containing component fails at instantiate_pre). Recommend Phase 3 story author confirms wasmtime Linker visibility before committing to Kani method. If not feasible as Kani, downgrade to Proptest (integration test with WASI binary fixture).

### 3. BC-2.18.001 (At-Least-Once Retry, VP-044, Kani P0)

This is proposed as P0 Kani. The justification: the retry state machine is a safety-critical delivery guarantee. Concern: if `ActionRetryState` is tightly coupled to tokio task spawning, the Kani proof surface may be limited to the state transition logic only (not the persistence step). The proof harness would target a pure `advance_retry_state(current: RetryState, outcome: DeliveryOutcome) -> RetryState` function extracted from the retry loop. If the implementation does not extract this pure function, the VP feasibility drops. Recommend the Phase 3 story for S-4.08 explicitly extract this pure state-transition function to enable the Kani proof.

---

## MARK-NONE Replacement Text Summary

For BCs receiving MARK-NONE, the VP section content should replace `VP-TBD` rows with:

| BC | Replacement Text |
|----|-----------------|
| BC-2.17.001 | `(none — INV-PLUGIN-001 enforced by wasmtime host-boundary construction; integration test in tests/plugin_tests.rs)` |
| BC-2.17.004 | `(none — epoch interruption correctness is time-dependent; integration test with infinite-loop WAT fixture is correct vehicle)` |
| BC-2.18.002 | `(none — best-effort/no-catch-up semantics are behavioral absence-of-write; integration test in tests/action_tests.rs)` |
| BC-2.18.003 | `(none — fire-and-forget and E-ACTION-007 trigger-type check are integration behaviors; no pure-function invariant)` |
| BC-2.18.005 | `(none — partial-report assembly is a trivial Result→RenderedSection map; integration tests TV-18-005-partial and TV-18-005-all-fail cover this)` |
| BC-2.18.006 | `(none — transitively covered by VP-024, VP-028, VP-038; this BC is integration glue over proven components)` |
| BC-2.18.008 | `(none — audit completeness is an integration test concern; credential-absence covered by BC-2.05.003 + VP-046)` |
| BC-2.19.003 | `(none — is_api_backed() is a trivial map lookup; enforcement at S-4.03 is integration-tested)` |
| BC-2.19.004 | `(none — CI-002 arc-swap pattern proven by VP-032; if UDF deregistration logic diverges, revisit during Phase 3)` |
| BC-2.19.005 | `(none — credential redaction proven architecturally and by VP-046; infusion log-capture verified by integration test AC-6)` |
| BC-2.08.006 | `(none — credential-absence guaranteed by SensorHealthResult type design; timestamp-equals-cache invariant is integration behavior)` |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-04-20 | architect | Pass-74 extension: 33 BCs analyzed (SS-14/15/16), 9 ADD-VP, 23 MARK-NONE, 1 DEFER; VP-051 through VP-059 proposed; BC-2.14.011 confirmed missing |
| 1.0 | 2026-04-20 | architect | Initial decision matrix: 22 BCs analyzed, 10 ADD-VP, 12 MARK-NONE, 0 DEFER |

---

## Pass-74 Extension: SS-14/15/16 (33 BCs)

### File Inventory Note

BC-2.14.011 does not exist on disk. The glob for `BC-2.14.*.md` returns 12 files:
BC-2.14.001 through BC-2.14.010, BC-2.14.012, and BC-2.14.013. BC-2.14.011 is
absent from the repository and is not analyzed here. 32 BCs are analyzed (not 33).

---

### Summary Table

| BC ID | Subsystem | Decision | Action Detail |
|-------|-----------|----------|---------------|
| BC-2.14.001 | SS-14 (Cases) | **B: MARK-NONE** | `case.write` gate covered transitively by VP-002 (deny-by-default capability) + integration test; audit-on-create is an integration observable, not a pure-function invariant |
| BC-2.14.002 | SS-14 | **A: ADD-VP** | VP-051: Case state machine — all 12 valid transitions accept and all invalid/self-transitions reject; Kani; augments VP-005/VP-006 with the full exhaustive transition table |
| BC-2.14.003 | SS-14 | **A: ADD-VP** | VP-052: Disposition-before-status ordering — proptest over single-call disposition+resolve; proves disposition is applied before state-machine check in a pure update-application function |
| BC-2.14.004 | SS-14 | **B: MARK-NONE** | AND-semantics filter logic and truncation metadata are behavioral integration concerns; no pure-function invariant beyond a trivial iterator filter |
| BC-2.14.005 | SS-14 | **B: MARK-NONE** | Metric computation and orphaned-alert handling are integration behavior; MTR/TTI are pure but their proof adds no material gap beyond the determinism invariant stated in the BC |
| BC-2.14.006 | SS-14 | **A: ADD-VP** | VP-053: Resolved case always has non-null disposition — Kani proof on state-machine postcondition |
| BC-2.14.007 | SS-14 | **B: MARK-NONE** | Annotation immutability is a data-layer invariant (RocksDB append-only); system-type MCP rejection is a simple two-value enum check; both are integration tests |
| BC-2.14.008 | SS-14 | **A: ADD-VP** | VP-054: TTR uses first resolution timestamp (never overwritten on reopen); proptest over reopen cycles; pure computation |
| BC-2.14.009 | SS-14 | **B: MARK-NONE** | WriteBatch atomicity is guaranteed by RocksDB; index consistency is an integration test on the persistence layer; no pure-function invariant distinct from integration testing |
| BC-2.14.010 | SS-14 | **B: MARK-NONE** | MTTD/MTTR null-when-no-resolved-cases is a trivial Option/None propagation; cross-client aggregation is integration behavior; both covered by BC-2.14.008 metric determinism invariant |
| BC-2.14.012 | SS-14 | **B: MARK-NONE** | Idempotency (no second write on re-ack) and audit-before-write ordering are integration test concerns; VP-033 covers audit buffer write ordering at the architecture level; no new pure-function invariant |
| BC-2.14.013 | SS-14 | **C: DEFER** | Dedup atomicity under concurrent alerts and alert-before-case ordering require Phase-3 implementation to confirm whether the WriteBatch transaction boundary can be isolated into a pure state function; concurrent alert dedup is a CRDT / CAS pattern, not a simple function |
| BC-2.15.001 | SS-15 (Persistence) | **B: MARK-NONE** | All-16-CFs-present is an integration startup test; exclusive lock enforcement is a RocksDB OS-level contract; neither is a pure-function invariant amenable to Kani |
| BC-2.15.002 | SS-15 | **A: ADD-VP** | VP-055: put_batch atomicity — proptest: batch of N writes where disk-full error is injected mid-batch; all entries must be absent from the store after failure; domain isolation: write to domain A does not appear in domain B |
| BC-2.15.003 | SS-15 | **B: MARK-NONE** | Persist-before-forward ordering is proven transitively by VP-033 (audit buffer RocksDB-write-before-delivery); crash-recovery replay is an integration test; no additional VP |
| BC-2.15.004 | SS-15 | **A: ADD-VP** | VP-056: Audit buffer overflow — newest entries preserved after purge; purge count = entries above threshold; proptest on the purge selection function (oldest-first deletion) |
| BC-2.15.005 | SS-15 | **A: ADD-VP** | VP-057: Crash-recovery denylist — dirty bit with consecutive_crashes=3 triggers denylisting; Kani proof on the state machine: `advance_crash_counter(n) -> denylist_if_n_gte_3` |
| BC-2.15.006 | SS-15 | **B: MARK-NONE** | Override-takes-precedence and watchdog-cannot-be-disabled are TOML config merge logic; an integration test of config loading is correct; VP-014/015 (oversized queries / nesting depth) cover the watchdog's enforcement behavior at the query side |
| BC-2.15.007 | SS-15 | **A: ADD-VP** | VP-058: Memory grace period — two-check policy; proptest: single spike above limit does not terminate; two consecutive checks above limit does terminate; pure policy function on WatchdogState |
| BC-2.15.008 | SS-15 | **B: MARK-NONE** | Consecutive-only counter (intervening success resets) is a pure state machine; but it is the same conceptual pattern as VP-057 (crash counter) already proposed; denylist-survives-restart is integration behavior |
| BC-2.15.009 | SS-15 | **B: MARK-NONE** | Virtual fields queryable in WHERE clause is a DataFusion integration test; decorator fields always-present is a response-assembly property verifiable only via integration test (DataFusion execution context required) |
| BC-2.15.010 | SS-15 | **B: MARK-NONE** | Phase priority ordering (periodic > query-time > config-time) is a merge function over three maps: trivially testable as a unit test but not a formal proof gap; stale-on-failure is a fallback behavioral property |
| BC-2.15.011 | SS-15 | **B: MARK-NONE** | audit.read capability gate covered transitively by VP-002 (deny-by-default capability resolution) + integration test; client_id scoping on internal table scans is an integration test (DataFusion scan predicate pushdown behavior) |
| BC-2.16.001 | SS-16 (Sensor Adapters) | **B: MARK-NONE** | Partial-failure isolation (one bad spec does not block others) is a startup iteration behavior, not a pure function; OCSF field mapping registration is an integration test; VP-023 (sensor spec parser never panics) already covers the critical safety property for this BC's spec parsing path |
| BC-2.16.002 | SS-16 | **B: MARK-NONE** | Fan-out batch concatenation is integration behavior over HTTP; variable scoping (forward references rejected at validation) is covered by BC-2.16.009 and its proposed VP-059 (see below); no additional VP |
| BC-2.16.003 | SS-16 | **B: MARK-NONE** | Coercion-failure non-fatal is a behavioral property (record not dropped); cross-sensor correlation via ocsf_field mapping is an integration test of the OCSF normalizer layer; both are covered by VP-016/017 (OCSF normalization) at the normalization layer |
| BC-2.16.004 | SS-16 | **B: MARK-NONE** | Panic isolation is proven transitively by VP-023 (sensor spec parser never panics) which uses fuzz; the CustomAdapter `catch_unwind` pattern is structurally identical to the WASM plugin panic isolation (BC-2.17.001, MARK-NONE); initial-sensors-use-pure-TOML is an integration test (architectural invariant) |
| BC-2.16.005 | SS-16 | **B: MARK-NONE** | Fail-closed on validation failure is the same arc-swap atomicity pattern as VP-032; MCP notification on schema change is behavioral integration output |
| BC-2.16.006 | SS-16 | **B: MARK-NONE** | In-flight query snapshot stability under reload is proven transitively by VP-032 (hot reload atomicity — failed validation retains old config); no-blocking-on-hot-path is a concurrency invariant verifiable only by integration/load test |
| BC-2.16.007 | SS-16 | **B: MARK-NONE** | In-flight query safety during hot reload is transitively covered by VP-032 and the ArcSwap invariant from BC-2.16.006; cache invalidation for modified/removed sensors is behavioral integration behavior |
| BC-2.16.008 | SS-16 | **B: MARK-NONE** | Atomic write (temp + rename pattern) is an OS-level file system invariant verified by integration test; sensor_spec.write capability gate is covered transitively by VP-002 (deny-by-default) |
| BC-2.16.009 | SS-16 | **A: ADD-VP** | VP-059: All-errors-collected validator — proptest: given a spec with N distinct validation errors, the output contains exactly N errors (no fail-fast); warning-only specs load successfully |
| BC-2.16.010 | SS-16 | **B: MARK-NONE** | Always-visible (no capability gate) is a trivial structural invariant (tool visibility = always); structuredContent format correctness is an integration test of the MCP response serializer |

**Aggregate counts (Pass-74 extension): ADD-VP = 9, MARK-NONE = 22, DEFER = 1**
**Total 32 BCs analyzed (BC-2.14.011 missing from repository).**

---

### Per-BC Analysis

#### BC-2.14.001 — `create_case` MCP Tool (SS-14)

**Decision: B — MARK-NONE**

The two placeholder VPs are:
1. "Verify `case.write` gate enforcement" — covered transitively by VP-002 (capability resolution: deny-by-default). VP-002 proves that the capability system denies any invocation whose capability is not explicitly allowed. The `create_case` gate is a call site for the same proven mechanism. Adding a BC-specific VP would duplicate VP-002's coverage.
2. "Verify audit entry emitted on create" — this is an integration observable. Every audit entry goes through the RocksDB persist-then-forward pipeline proven by VP-033 (audit buffer ordering). The "entry was emitted" assertion requires executing the full tool invocation and observing the audit buffer — an integration test.

**Replacement frontmatter text:**
```
(none — case.write gate covered transitively by VP-002 (deny-by-default capability); audit-on-create ordering covered by VP-033; no pure-function invariant distinct from integration test)
```

---

#### BC-2.14.002 — Case State Transitions (SS-14)

**Decision: A — ADD-VP**

VP-005 proves "exactly 12 valid transitions" and VP-006 proves "no self-transitions." However, BC-2.14.002 contains a richer invariant: the full exhaustive accept/reject table for all (state × state) pairs, including: backward transitions to New/Acknowledged are prohibited, reopen targets only Investigating, and resolved-at semantics on the reopen cycle. VP-005 counts transitions at a high level; it does not prove that the specific invalid transitions (Closed → New, Closed → Acknowledged) are individually rejected with the correct structured error. A Kani proof can enumerate all 25 pairs (5 states × 5 states) and verify exact accept/reject outcomes for each.

**Proposed VP-051:**
- ID: VP-051
- Slug: `vp-051-case-state-machine-exhaustive-transition-table`
- Method: Kani
- Priority: P0
- Property: For every (from_state, to_state) pair in the 5×5 case state matrix, `advance_case_state(from, to)` returns Ok for exactly the 12 valid transitions and Err(E-CASE-004 or E-CASE-005) for all 13 invalid pairs; self-transitions always return Err(E-CASE-005); transitions to New or Acknowledged from any state always return Err(E-CASE-004)
- Source BC: BC-2.14.002
- Note: Complements VP-005 (transition count) and VP-006 (no self-transitions) with per-pair exhaustive coverage

---

#### BC-2.14.003 — `update_case` MCP Tool (SS-14)

**Decision: A — ADD-VP**

The ordering invariant ("disposition is applied before status transition in a single call") is a critical business rule: it is what allows a single `update_case(disposition=FalsePositive, status=Resolved)` call to succeed. If the ordering were reversed, the call would fail (status-before-disposition violates the Resolved precondition). This is a pure-function property: `apply_update_fields(case, update_spec) -> CaseUpdate` where `update_spec.disposition` is applied before `update_spec.status`. A proptest can generate CaseUpdateSpec values with both disposition and status set to Resolved and verify: (a) the call succeeds, (b) the timeline shows DispositionSet before StatusChanged, (c) flipping the order by injecting status-first would fail.

**Proposed VP-052:**
- ID: VP-052
- Slug: `vp-052-update-case-disposition-before-status-ordering`
- Method: Proptest
- Priority: P0
- Property: For any `CaseUpdateSpec` containing both `disposition: Some(d)` and `status: Some(Resolved)`, `apply_update_fields()` applies the disposition update before the status transition; a call with disposition=FalsePositive and status=Resolved succeeds when case has no prior disposition; the same call with status applied first would fail E-CASE-006
- Source BC: BC-2.14.003

---

#### BC-2.14.004 — `list_cases` MCP Tool (SS-14)

**Decision: B — MARK-NONE**

AND-semantics filter combination is a query planner property: given a predicate set, the filter predicate is a conjunction. This is a trivial Rust `Iterator::filter` composition — it is not a formal proof gap. Truncation metadata (`is_truncated`, `total_available`) is a behavioral property of the pagination wrapper around a RocksDB scan, verifiable only by running the scan with a known dataset (integration test). Neither property has a meaningful pure-function proof harness separable from the storage layer.

**Replacement frontmatter text:**
```
(none — AND-filter semantics are trivial iterator conjunction; truncation metadata is integration behavior of the RocksDB scan wrapper; no pure-function invariant warrants a formal VP)
```

---

#### BC-2.14.005 — `get_case` MCP Tool (SS-14)

**Decision: B — MARK-NONE**

"Verify metric computation correctness" — the metric computation invariants (TTR uses first resolution, metrics are non-negative, null propagation) are proven by VP-054 (proposed for BC-2.14.008). BC-2.14.005 is a read-through tool that calls the same metric computation as BC-2.14.008. No additional VP needed.

"Verify orphaned-alert graceful handling" — this is a conditional read: if an alert_id in `source_alert_ids` is absent from the alerts store, return `{deleted: true}`. This is integration behavior (requires a real RocksDB state with a missing key) and is covered by EC-14-017 integration test. No pure-function invariant.

**Replacement frontmatter text:**
```
(none — metric computation invariants covered by VP-054 (BC-2.14.008); orphaned-alert handling is integration behavior; both covered by integration tests in S-4.06 test suite)
```

---

#### BC-2.14.006 — Disposition Assignment (SS-14)

**Decision: A — ADD-VP**

"A resolved case always has non-null disposition" is a state invariant of the case record: `case.status == Resolved → case.disposition.is_some()`. This is structurally identical to the case state machine properties already proven by VP-005/VP-006. A Kani proof can verify: given any `CaseRecord` that has undergone a `Resolved` transition (via `advance_case_state`), the disposition field is Some. This is a postcondition of the state machine transition function. The property is distinct from VP-053 in that it proves the type-level invariant (no null disposition on any resolved case record), not just that the transition function requires a disposition parameter.

**Proposed VP-053:**
- ID: VP-053
- Slug: `vp-053-resolved-case-disposition-non-null`
- Method: Kani
- Priority: P0
- Property: For any `CaseRecord` produced by `advance_case_state(case, Resolved)`, `record.disposition.is_some()` holds; `advance_case_state` returns Err(E-CASE-006) when `case.disposition.is_none()`; no `CaseRecord` can have `status = Resolved` AND `disposition = None`
- Source BC: BC-2.14.006

---

#### BC-2.14.007 — Timeline Annotations (SS-14)

**Decision: B — MARK-NONE**

"Annotation immutability" — annotations are stored as append-only entries in a Vec<Annotation> serialized to RocksDB. Immutability is enforced by the absence of any mutation method — a code review property, not a formal proof property. There is no pure function to prove (you cannot prove a function does not exist).

"System-type MCP rejection" — this is a two-branch match: if annotation type is `status_change` or `alert_link`, return `E-CASE-013`; otherwise proceed. A unit test covers this exhaustively. No formal VP adds material confidence beyond the unit test for a trivial enum check.

**Replacement frontmatter text:**
```
(none — annotation immutability is an append-only data structure property enforced by absence of mutation methods (code review); system-type rejection is a trivial 2-branch enum check covered by unit test; no pure-function invariant warrants a formal VP)
```

---

#### BC-2.14.008 — TTD/TTI/TTR Computation (SS-14)

**Decision: A — ADD-VP**

The reopen-cycle invariant is a pure computation property: `compute_ttr(case)` must use `case.resolved_at` (which is set on first resolution and never overwritten) regardless of how many reopen cycles have occurred. A proptest can generate `CaseRecord` values with multiple reopen cycles (alternating Resolved → Investigating → Resolved) and verify: `compute_ttr(case) == case.resolved_at_first - case.created_at` for all such records. The BC explicitly states "resolved_at is NOT cleared on reopen" — this is a pure invariant of the metric computation function.

The null-propagation property (no resolved cases → null aggregate) is a simpler proptest: an empty Vec<CaseRecord> fed to `compute_mttd/mttdi/mttr_avg` returns `None`.

**Proposed VP-054:**
- ID: VP-054
- Slug: `vp-054-ttr-uses-first-resolution-timestamp`
- Method: Proptest
- Priority: P1
- Property: For any `CaseRecord` with N reopen cycles (N >= 1), `compute_ttr(case)` equals `case.resolved_at - case.created_at` using the FIRST `resolved_at` timestamp; `compute_mttd_avg([])`, `compute_mtti_avg([])`, `compute_mttr_avg([])` all return `None` (not zero) when given an empty input; all metrics are non-negative (floored at Duration::ZERO)
- Source BC: BC-2.14.008

---

#### BC-2.14.009 — Case Persistence (SS-14)

**Decision: B — MARK-NONE**

WriteBatch atomicity is guaranteed by RocksDB's WAL — it is not a property of Prism's code, it is a property of the underlying library. The correct verification vehicle is an integration test that simulates a mid-batch crash and validates that neither the case record nor any index entry is partially written. Index consistency after status transitions is also an integration test (scan old and new index entries after an update). Adding formal VPs here would be over-engineering infrastructure-level storage guarantees.

**Replacement frontmatter text:**
```
(none — WriteBatch atomicity is a RocksDB WAL guarantee, not a Prism code property; index consistency after transitions is an integration test on the storage layer; covered by integration tests in S-1.02 / S-4.06 test suites)
```

---

#### BC-2.14.010 — `case_metrics` MCP Tool (SS-14)

**Decision: B — MARK-NONE**

"MTTD/MTTR null when no resolved cases" is a null-propagation property of the same metric functions proven by VP-054 (BC-2.14.008). VP-054 already covers the `compute_mttd_avg([]) → None` case. No additional VP needed.

"Cross-client aggregation correctness" is an integration test: it requires multiple client RocksDB namespaces and a scan across all of them. Not a pure-function invariant.

**Replacement frontmatter text:**
```
(none — MTTD/MTTR null-propagation covered by VP-054 (BC-2.14.008); cross-client aggregation is integration behavior requiring multi-client RocksDB state)
```

---

#### BC-2.14.012 — `acknowledge_alert` MCP Tool (SS-14)

**Decision: B — MARK-NONE**

"Verify idempotency (no second write on re-ack)" — idempotency here means: if `alert.acknowledged == true`, the tool reads the existing record and returns it without calling `put()`. This is a conditional write omission — a behavioral property verified by integration test (assert RocksDB write count = 1 after two invocations). There is no pure-function invariant: the decision to write or not depends on a RocksDB read result.

"Verify audit-before-write ordering" — covered transitively by VP-033 (audit buffer RocksDB-write-before-delivery) and the architectural DI-016 (write fail-closed) pattern. The `acknowledge_alert` tool follows the same audit-before-mutate contract as all other write tools. No new VP needed.

The BC's VP Anchors section explicitly states: "No dedicated VPs currently. Covered by integration tests in S-4.07 test suite."

**Replacement frontmatter text:**
```
(none — idempotent no-write is behavioral (conditional RocksDB write omission); audit-before-write covered transitively by VP-033 and DI-016 architectural invariant; confirmed by BC's own VP Anchors section)
```

---

#### BC-2.14.013 — Auto-Case-Creation (SS-14)

**Decision: C — DEFER**

The two placeholder VPs are:
1. "Verify dedup atomicity under concurrent alerts" — this requires proving that the RocksDB WriteBatch dedup check + case creation is atomic under concurrent writers. This is a CRDT/CAS problem: the first writer wins because RocksDB's WriteBatch is atomic, but proving correctness under concurrent writers requires either a TLA+ spec of the concurrency protocol or a proptest with a mock atomic compare-and-swap. Phase 3 must clarify whether `CaseDedupRegistry::check_and_create()` is exposed as a pure state-transition function or tightly coupled to RocksDB transactions.
2. "Verify alert-before-case ordering" — requires confirming that `alert.persist()` completes before `case.create()` begins. This is an ordering guarantee about two sequential async operations. Whether this can be proven by Kani (if extracted to a pure sequencing function) or requires an integration test depends on the implementation.

**Phase 3 tracker:**

| Item | BC | Required for VP? | Gate |
|------|-----|-----------------|------|
| Expose `advance_dedup_state(existing_cases, new_alert) -> DedupOutcome` as a pure function | BC-2.14.013 | Yes (VP-060 proposed) | Phase 3 story for S-4.06 |
| Confirm alert-before-case is a single function call sequence vs. separate async tasks | BC-2.14.013 | Yes | Phase 3 story for S-4.06 |

If the Phase 3 implementation exposes a pure dedup state function, propose VP-060: proptest over concurrent alert sequences proving idempotent dedup (same rule+client+open-case window → exactly 1 case created). If implementation uses RocksDB CAS directly without pure function extraction, MARK-NONE and cover with integration test.

---

#### BC-2.15.001 — RocksDB Initialization (SS-15)

**Decision: B — MARK-NONE**

"All 16 CFs present after fresh init" — this is a startup integration test: open RocksDB, enumerate column families, assert count = 16. Not a pure-function invariant; the CF count depends on the RocksDB library call result.

"Exclusive lock enforcement" — RocksDB enforces the database lock via an OS advisory lock file. The Prism code simply calls `DB::open()` and propagates the lock error. There is nothing pure to prove here.

**Replacement frontmatter text:**
```
(none — CF-count-after-init is a startup integration test on RocksDB; exclusive lock is an OS-level advisory lock enforced by RocksDB, not by Prism code; no pure-function invariant)
```

---

#### BC-2.15.002 — Domain KV Operations (SS-15)

**Decision: A — ADD-VP**

Two distinct invariants have pure-function proof surfaces:

1. **put_batch atomicity under simulated write failure**: The `StorageEngine::put_batch` contract is that all entries succeed or all fail. A proptest can implement a `MockStorageEngine` that fails on the Nth write and verify: for any batch of K entries with failure injected at position N < K, the mock verifies zero entries were committed (all-or-nothing). The pure function here is the batch application logic, not RocksDB itself.

2. **Domain isolation**: `write(domain_A, key, value)` produces a state where `read(domain_B, key) == None`. This is a pure property of the `StorageEngine` abstraction: domain operations are namespaced and cannot bleed into other domains. A proptest over the in-memory mock verifies this invariant across all domain pairs.

**Proposed VP-055:**
- ID: VP-055
- Slug: `vp-055-storage-engine-batch-atomicity-and-domain-isolation`
- Method: Proptest
- Priority: P1
- Property: (1) For any `put_batch` where the underlying write fails partway through, zero entries from that batch are readable via `get`; (2) For any pair of distinct domains (A, B), a write to domain A at key K produces `get(domain_B, K) == None`; the MockStorageEngine upholds both invariants across all domain-pair and batch-size combinations
- Source BC: BC-2.15.002

---

#### BC-2.15.003 — Buffered Audit Log Persistence (SS-15)

**Decision: B — MARK-NONE**

"Verify persist-before-forward ordering" — covered transitively by VP-033 (audit buffer: RocksDB write completes before delivery attempt). VP-033 is an integration test that proves the persist-then-forward ordering at the CrowdStrike DTU layer. The same ordering invariant at the audit forwarder level is the same architectural pattern. A second VP targeting the same ordering on the generic audit forwarder would be near-identical to VP-033.

"Verify crash-recovery replay of buffered entries" — crash recovery requires a real process restart and RocksDB state recovery. This is an integration test; there is no pure-function invariant to prove.

**Replacement frontmatter text:**
```
(none — persist-before-forward ordering proven transitively by VP-033 (audit buffer RocksDB-write-before-delivery); crash-recovery replay is a restart integration test; no additional pure-function VP)
```

---

#### BC-2.15.004 — Audit Buffer Overflow (SS-15)

**Decision: A — ADD-VP**

The overflow purge has a pure-function invariant: given a buffer of N entries (N > threshold), the purge function selects the oldest (threshold × 0.9) entries to delete, preserving the newest. This is a pure ordering + selection function over a sorted key-space. A proptest can verify:
- Given N entries sorted by timestamp key, purge produces exactly `N - floor(threshold * 0.9)` deleted entries
- The deleted entries are the N oldest (smallest timestamp keys)
- The retained entries are the N newest (largest timestamp keys)
- The purge-event audit entry is produced as a separate output

The function `compute_purge_targets(entries: &[(key, entry)], threshold: usize) -> Vec<key>` is deterministic and pure.

**Proposed VP-056:**
- ID: VP-056
- Slug: `vp-056-audit-buffer-overflow-purge-preserves-newest`
- Method: Proptest
- Priority: P1
- Property: For any audit buffer of N entries (N > threshold), `compute_purge_targets()` returns exactly the oldest `(N - floor(threshold * 0.9))` entries by timestamp key; the newest `floor(threshold * 0.9)` entries are never in the purge target set; a purge-event record is always included in the output
- Source BC: BC-2.15.004

---

#### BC-2.15.005 — Crash Recovery Dirty Bits (SS-15)

**Decision: A — ADD-VP**

The denylist trigger logic is a pure state machine: `advance_crash_counter(entry: DirtyBitEntry) -> RecoveryAction` where the action is `Denylist` if `entry.consecutive_crashes + 1 >= 3`, else `Warn`. Kani can prove:
- For all `consecutive_crashes` values in 0..=u32::MAX: result is `Denylist` iff `consecutive_crashes >= 2` (i.e., the new value after increment >= 3)
- The denylist action always includes a 86400s expiry
- The recovery action is idempotent (processing the same dirty bit twice produces the same result)

This is structurally identical to the retry state machine proven by VP-044.

**Proposed VP-057:**
- ID: VP-057
- Slug: `vp-057-crash-recovery-denylist-at-three-consecutive-crashes`
- Method: Kani
- Priority: P0
- Property: `advance_crash_counter(entry)` returns `RecoveryAction::Denylist { expiry_seconds: 86400 }` if and only if `entry.consecutive_crashes + 1 >= 3`; for all other values it returns `RecoveryAction::Warn`; the threshold is exactly 3 (not 2 or 4)
- Source BC: BC-2.15.005

---

#### BC-2.15.006 — Resource Watchdog Initialization (SS-15)

**Decision: B — MARK-NONE**

"Override takes precedence over level defaults" — this is a config merge operation: `override.value.unwrap_or(level.default)`. This is trivially proven by a unit test on the `WatchdogConfig::merge` function. No Kani proof adds material confidence over a unit test for a two-branch option merge.

"Watchdog cannot be fully disabled" — the invariant that even the `permissive` level has finite bounds is enforced by hardcoded constants in the level table. This is a code review property (the permissive row has finite values), not a provable invariant. There is no function to call that "disables" the watchdog — the disable path does not exist.

VP-014/VP-015 (oversized queries, nesting depth) cover the watchdog's enforcement at the query-side boundary. No additional VP for initialization is needed.

**Replacement frontmatter text:**
```
(none — override-takes-precedence is a trivial Option::unwrap_or merge, unit test coverage sufficient; watchdog-cannot-be-disabled is a hardcoded constant, code review property; enforcement side covered by VP-014/015)
```

---

#### BC-2.15.007 — Watchdog Query Termination (SS-15)

**Decision: A — ADD-VP**

The memory grace period (two-check policy, DI-027) is a pure state function: `should_terminate_for_memory(state: WatchdogCheckState) -> bool` where `state` contains `consecutive_over_limit: u8`. The function returns `true` iff `consecutive_over_limit >= 2`. A proptest can verify:
- `consecutive_over_limit = 0`: not terminated
- `consecutive_over_limit = 1`: not terminated
- `consecutive_over_limit = 2`: terminated
- `consecutive_over_limit >= 2`: always terminated

This is structurally identical to VP-057 (crash counter) but for the query-level memory violation counter.

**Proposed VP-058:**
- ID: VP-058
- Slug: `vp-058-watchdog-memory-grace-period-two-check-policy`
- Method: Proptest
- Priority: P0
- Property: `should_terminate_for_memory(state)` returns true iff `state.consecutive_over_limit >= 2`; a single check with memory above limit does not terminate (returns false); two consecutive checks above limit do terminate (returns true); the threshold is exactly 2 checks
- Source BC: BC-2.15.007

---

#### BC-2.15.008 — Query Denylisting (SS-15)

**Decision: B — MARK-NONE**

"Consecutive-only counter (intervening success resets)" — this is the same counter-reset pattern as VP-057 (crash recovery consecutive_crashes). The query denylist counter `DenylistCounterState` is structurally identical to `DirtyBitEntry.consecutive_crashes`. Adding VP-058+ for query denylisting would duplicate VP-057's proof shape with different variable names. The BC's edge case EC-15-028 (timeout, success, timeout → counter resets, no denylist) is an integration test that exercises the same RocksDB read-modify-write cycle.

"Denylist survives restart" — requires a real RocksDB restart to verify. Integration test.

**Replacement frontmatter text:**
```
(none — consecutive-only counter is structurally identical to VP-057 (crash-recovery denylist counter); proof shape would be a duplicate; denylist-survives-restart is a restart integration test; covered by integration tests in watchdog test suite)
```

---

#### BC-2.15.009 — Context Decorator Injection (SS-15)

**Decision: B — MARK-NONE**

"Virtual fields queryable in WHERE clause" — this is a DataFusion integration test: register a MemTable with `_sensor` column, execute `WHERE _sensor = 'crowdstrike'`, assert result count. Requires a live DataFusion SessionContext; not a pure-function invariant.

"Decorator fields always present (never null-struct)" — this is an integration property of the response assembly: the `_meta` envelope struct is always serialized with all fields. The "never null-struct" property means the struct is fully initialized; a unit test on `DecoratorFields::default()` covers this. No Kani proof needed for a struct initialization property.

**Replacement frontmatter text:**
```
(none — virtual field queryability requires DataFusion execution context (integration test); decorator-always-present is a struct initialization property (unit test); no pure-function formal invariant)
```

---

#### BC-2.15.010 — Decorator Three-Phase Model (SS-15)

**Decision: B — MARK-NONE**

"Phase priority ordering (periodic > query-time > config-time)" — this is a three-way map merge: `config_map.merge(query_map).merge(periodic_map)` where later wins. This is a trivially testable unit test on the merge function. A proptest would cover it, but the function complexity does not warrant a formal VP — it is a `HashMap::extend()` call with defined precedence.

"Stale-on-failure for periodic phase" — requires injecting a failure in the periodic refresh and observing that the stale cached value is returned. This is a behavioral fallback property verifiable only by integration test (requires a background task and cache state).

**Replacement frontmatter text:**
```
(none — phase priority ordering is a HashMap::extend() merge with defined precedence; unit test coverage sufficient; stale-on-failure is a behavioral fallback property requiring a live background task; integration test is correct vehicle)
```

---

#### BC-2.15.011 — Internal Table Registration (SS-15)

**Decision: B — MARK-NONE**

"audit.read capability gate" — covered transitively by VP-002 (deny-by-default capability resolution). VP-002 proves that any capability not explicitly allowed is denied. The `prism_audit` capability gate is a call site for VP-002's proven mechanism. No additional VP needed.

"client_id scoping on internal table scans" — this is a DataFusion predicate pushdown property: the `TableProvider::scan()` implementation for internal tables applies a `client_id` filter. This requires a live DataFusion planning and execution context; it is an integration test.

**Replacement frontmatter text:**
```
(none — audit.read gate covered transitively by VP-002 (deny-by-default capability); client_id scoping on internal table scans is a DataFusion execution integration test; no additional formal VP)
```

---

#### BC-2.16.001 — Sensor Spec File Loading (SS-16)

**Decision: B — MARK-NONE**

"Partial-failure isolation (one bad spec does not block others)" — this is a startup iteration property: for each spec in the directory, parse it, if it fails skip it and continue. This is a behavioral property of a loop, not a pure function. The `load_all_specs()` function is testable as an integration test with a directory containing mixed valid/invalid files.

"OCSF field mappings registered with normalizer" — this is an integration test of the normalizer registration call site. VP-023 (sensor spec parser never panics on arbitrary TOML) already covers the critical safety property for spec parsing. The OCSF registration itself is a side-effectful operation (modifying the normalizer registry) that cannot be proven in isolation.

**Replacement frontmatter text:**
```
(none — partial-failure isolation is a behavioral loop property (integration test); OCSF field mapping registration is a side-effectful normalizer call; VP-023 covers the critical panic-safety property for spec parsing; no additional formal VP)
```

---

#### BC-2.16.002 — Multi-Step Fetch Pipeline (SS-16)

**Decision: B — MARK-NONE**

"Fan-out batch concatenation" — requires live HTTP mock calls across batch iterations. Integration test.

"Variable scoping (forward references rejected at validation)" — this invariant is specified in BC-2.16.009 (spec file validation, rule 2: Variable Reference Resolution) and proposed VP-059 covers it. BC-2.16.002 is the runtime execution of the pipeline; forward reference rejection happens at validation time (BC-2.16.009). No additional VP needed here.

**Replacement frontmatter text:**
```
(none — fan-out batch concatenation requires HTTP mock integration; forward-reference scoping rejection is covered by VP-059 (BC-2.16.009 validation); no additional formal VP for runtime pipeline execution)
```

---

#### BC-2.16.003 — Column-to-OCSF Mapping (SS-16)

**Decision: B — MARK-NONE**

"Coercion-failure non-fatal (record not dropped)" — this is a behavioral property of the mapping function: on coercion error, the record is included with the failed field moved to `raw_extensions`. VP-017 (OCSF normalization: unmapped fields preserved) covers the `raw_extensions` preservation invariant at the normalizer layer. The coercion-failure fallback in the spec-engine layer is the same semantic: unmapped → raw_extensions. No additional VP.

"Cross-sensor correlation via ocsf_field mapping" — requires two live sensor adapters returning data and a DataFusion JOIN. Integration test of the query engine + OCSF normalizer. VP-016/017 already cover the normalizer's correctness properties.

**Replacement frontmatter text:**
```
(none — coercion-failure record preservation is semantically identical to VP-017 (unmapped fields preserved in raw_extensions); cross-sensor correlation requires full query engine integration test; no additional formal VP)
```

---

#### BC-2.16.004 — Rust Escape Hatch (SS-16)

**Decision: B — MARK-NONE**

"Panic isolation (process survives adapter panic)" — structurally identical to BC-2.17.001 (Plugin Panic Isolation, MARK-NONE). The `catch_unwind` pattern is proven by construction; VP-023 (sensor spec parser never panics on arbitrary TOML) covers the parser-level panic safety. The CustomAdapter `catch_unwind` is a one-line wrapper and warrants an integration test (inject a panicking adapter, verify process survives), not a Kani proof.

"Initial sensors use pure TOML path (no adapter)" — this is an architectural invariant enforced by the absence of `CustomAdapterRegistry::register()` calls for the four initial sensors. A code review property. Not a formal proof.

**Replacement frontmatter text:**
```
(none — panic isolation via catch_unwind is a construction-time guarantee; integration test with panicking fixture is correct vehicle; initial-sensor TOML-only property is a code review invariant; no pure-function formal VP)
```

---

#### BC-2.16.005 — `reload_config` MCP Tool (SS-16)

**Decision: B — MARK-NONE**

"Fail-closed on validation failure (current config retained)" — this is the same arc-swap atomicity pattern proven by VP-032 (hot reload atomicity: failed validation retains old config). VP-032 specifically proves the ArcSwap retention invariant. BC-2.16.005's reload_config is the primary use site for the VP-032 pattern.

"MCP notification sent on schema change" — this is a behavioral output property requiring a live MCP connection and schema diff detection. Integration test.

**Replacement frontmatter text:**
```
(none — fail-closed on validation failure proven transitively by VP-032 (hot reload atomicity); MCP notification on schema change is a behavioral integration test; no additional formal VP)
```

---

#### BC-2.16.006 — ArcSwap Config Access (SS-16)

**Decision: B — MARK-NONE**

"In-flight query snapshot stability under reload" — the ArcSwap load() → Guard relationship ensures in-flight queries hold a reference to the snapshot at query-start. VP-032 proves the arc-swap atomicity invariant. The snapshot stability property is a consequence of `Arc` reference counting semantics in Rust — a type-system guarantee, not a proof target.

"No blocking on query hot path" — wait-freedom of `ArcSwap::load()` on x86_64 is a property of the `arc-swap` library, not of Prism's code. There is nothing in Prism's code to formally prove; the library's own guarantees (documented in its README) provide the coverage. A load test measuring latency is the correct vehicle.

**Replacement frontmatter text:**
```
(none — snapshot stability is a consequence of Arc reference counting (type-system guarantee); wait-free ArcSwap::load() is a library property not provable at the Prism code level; VP-032 covers the ArcSwap swap-side invariant)
```

---

#### BC-2.16.007 — Sensor Spec Hot Reload (SS-16)

**Decision: B — MARK-NONE**

"In-flight query safety during hot reload" — covered transitively by VP-032 and BC-2.16.006 analysis above.

"Cache invalidation for modified/removed sensors" — requires triggering a reload and verifying that subsequent queries to the modified sensor do not return stale cached results. This is a behavioral integration test of the cache invalidation call in the reload path.

**Replacement frontmatter text:**
```
(none — in-flight query safety transitively covered by VP-032 and BC-2.16.006; cache invalidation for modified/removed sensors is a behavioral integration test; no additional formal VP)
```

---

#### BC-2.16.008 — `add_sensor_spec` MCP Tool (SS-16)

**Decision: B — MARK-NONE**

"Atomic write (temp + rename pattern)" — the temp-file-then-rename pattern is an OS-level atomicity guarantee, not a property of Prism's Rust code. The correctness of `std::fs::rename()` atomicity is an OS contract. An integration test that aborts mid-write and checks for no partial files is the correct vehicle.

"sensor_spec.write capability gate" — covered transitively by VP-002 (deny-by-default capability resolution). Same reasoning as BC-2.14.001.

**Replacement frontmatter text:**
```
(none — temp+rename atomicity is an OS-level file system guarantee, not a Prism code property; sensor_spec.write gate covered transitively by VP-002 (deny-by-default capability); no pure-function formal VP)
```

---

#### BC-2.16.009 — Spec File Validation (SS-16)

**Decision: A — ADD-VP**

The all-errors-collected validator (no fail-fast) has a pure-function invariant: `validate_sensor_spec(spec: &SensorSpec) -> ValidationResult` where `ValidationResult` contains a `Vec<ValidationError>`. The function must:
- Not return early on the first error
- Collect all errors before returning
- Return Ok if and only if the error vec is empty

A proptest can generate `SensorSpec` values with N distinct validation errors (e.g., invalid sensor_id + forward variable reference + duplicate column names) and verify that all N errors appear in the output. This is a classic "no-fail-fast" invariant provable by proptest.

The warning-only loading property is also pure: `validate_sensor_spec` returns `Ok(warnings)` (not `Err`) when only warnings are present.

**Proposed VP-059:**
- ID: VP-059
- Slug: `vp-059-spec-validator-all-errors-collected-no-fail-fast`
- Method: Proptest
- Priority: P1
- Property: For any `SensorSpec` with N distinct validation errors (N >= 1), `validate_sensor_spec()` returns `Err(errors)` where `errors.len() == N`; for a spec with only warnings and no errors, returns `Ok(warnings)` (spec accepted); the function never returns early on the first error
- Source BC: BC-2.16.009

---

#### BC-2.16.010 — `list_sensor_specs` MCP Tool (SS-16)

**Decision: B — MARK-NONE**

"Always-visible (no capability gate)" — the tool registration sets `always_visible = true` with no capability check. This is a struct field assignment at registration time — a code review property verifiable by unit test on the tool registration. No Kani proof needed for a boolean field.

"structuredContent format correctness" — requires serializing the response struct to JSON and validating the MCP `structuredContent` field structure. This is an integration test of the MCP response serializer. No pure-function invariant.

**Replacement frontmatter text:**
```
(none — always-visible is a boolean field in tool registration (unit test); structuredContent format correctness is a serialization integration test; no pure-function formal VP)
```

---

### VP-INDEX Additions (Pass-74, for approval)

The following 9 rows should be appended to VP-INDEX.md when this matrix is approved. IDs VP-051 through VP-059.

| ID | Property | Module | Method | Priority | Status | Anchor Story |
|----|----------|--------|--------|----------|--------|--------------|
| VP-051 | Case state machine: exhaustive 5×5 transition table — 12 accept, 13 reject | prism-core | kani | P0 | proposed | S-1.02 |
| VP-052 | update_case: disposition applied before status transition in single-call update | prism-core | proptest | P0 | proposed | S-4.06 |
| VP-053 | Resolved case always has non-null disposition; transition rejects without disposition | prism-core | kani | P0 | proposed | S-4.06 |
| VP-054 | TTR uses first resolution timestamp across reopen cycles; null aggregate when no resolved cases | prism-core | proptest | P1 | proposed | S-4.06 |
| VP-055 | StorageEngine put_batch atomicity and domain isolation (MockStorageEngine) | prism-persistence | proptest | P1 | proposed | S-1.02 |
| VP-056 | Audit buffer overflow purge: oldest entries deleted, newest preserved, purge-event produced | prism-audit | proptest | P1 | proposed | S-5.10 |
| VP-057 | Crash recovery: denylist triggered at consecutive_crashes >= 3; exact threshold | prism-persistence | kani | P0 | proposed | S-1.02 |
| VP-058 | Watchdog memory grace period: single check does not terminate; two consecutive checks do | prism-persistence | proptest | P0 | proposed | S-2.02 |
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok | prism-spec-engine | proptest | P1 | proposed | S-1.11 |

**Updated method totals (Pass-74 additions, if all 9 accepted, on top of current VP-050 baseline):**

| Method | Current (VP-001–050) | Pass-74 +New | Running Total |
|--------|---------------------|--------------|---------------|
| Kani | 23 | +3 (VP-051, VP-053, VP-057) | 26 |
| Proptest | 19 | +5 (VP-052, VP-054, VP-055, VP-056, VP-058, VP-059) | 25 |
| Fuzz | 6 | 0 | 6 |
| Integration test | 2 | 0 | 2 |
| **Total** | **50** | **+9** | **59** |

Note: Proptest count = 19 + 6 (VP-052, VP-054, VP-055, VP-056, VP-058, VP-059) = 25.

---

### Deferred VPs — Phase 3 Tracker

| Item | BC | Proposed VP ID | Required Precondition | Gate |
|------|-----|---------------|----------------------|------|
| Expose `advance_dedup_state()` as pure function | BC-2.14.013 | VP-060 (tentative) | Phase 3 S-4.06 story author must extract a pure `DedupState` transition function from the WriteBatch dedup check | Phase 3 story planning |
| Confirm alert-before-case ordering is a sequenced function call (not concurrent async) | BC-2.14.013 | VP-060 (same) | Phase 3 implementation review | Phase 3 story planning |

---

### Contentious Decisions — Human Input Requested (Pass-74)

#### 1. BC-2.14.002 (Case State Transitions) — ADD-VP (VP-051, Kani, P0)

VP-005 already proves "exactly 12 valid transitions." VP-051 proposes exhaustive per-pair coverage of all 25 (state × state) combinations. There is an argument that VP-051 duplicates VP-005/006. The distinction: VP-005 proves the count (12), VP-006 proves no self-transitions, but neither proves that specific individual transitions (e.g., Closed → New, Closed → Acknowledged) produce the correct error code. VP-051 proves the full table. If the project team considers VP-005+VP-006 sufficient, VP-051 can be MARK-NONE.

#### 2. BC-2.15.002 (Domain KV Operations) — ADD-VP (VP-055, Proptest, P1)

VP-055 targets a MockStorageEngine, not the real RocksDB. The atomicity invariant it proves is only as strong as the mock. If the mock's behavior diverges from RocksDB's (e.g., mock does not simulate WAL), the proof provides false confidence. Recommend: the Phase 3 story for S-1.02 or S-5.01 must implement `MockStorageEngine` faithfully to make VP-055 meaningful. If the mock cannot be made faithful, downgrade to integration test and MARK-NONE.

#### 3. BC-2.14.013 (Auto-Case-Creation) — DEFER

The DEFER decision assumes the Phase 3 implementation can extract a pure `advance_dedup_state` function. If the dedup logic is deeply coupled to RocksDB transactions (compare-and-swap on write), no pure-function extraction is possible and the correct outcome is MARK-NONE with an integration test covering concurrent alert generation. Human decision needed on whether the dedup protocol is worth a TLA+ spec (overkill) or an integration concurrency test (sufficient).

### MARK-NONE Replacement Text Summary (Pass-74)

For BCs receiving MARK-NONE, the VP section content should replace `(placeholder)` rows with:

| BC | Replacement Text |
|----|-----------------|
| BC-2.14.001 | `(none — case.write gate covered transitively by VP-002; audit-on-create covered by VP-033; integration test is correct vehicle)` |
| BC-2.14.004 | `(none — AND-filter semantics are trivial iterator conjunction; truncation metadata is integration behavior; integration test is correct vehicle)` |
| BC-2.14.005 | `(none — metric computation covered by VP-054; orphaned-alert handling is integration behavior)` |
| BC-2.14.007 | `(none — annotation immutability enforced by absence of mutation methods; system-type rejection is trivial 2-branch enum check; unit test sufficient)` |
| BC-2.14.009 | `(none — WriteBatch atomicity is a RocksDB WAL guarantee; index consistency is integration behavior; no Prism code property to prove)` |
| BC-2.14.010 | `(none — null-propagation covered by VP-054; cross-client aggregation is integration behavior)` |
| BC-2.14.012 | `(none — idempotent no-write is behavioral; audit-before-write covered transitively by VP-033 and DI-016; confirmed by BC's own VP Anchors)` |
| BC-2.15.001 | `(none — CF-count-after-init is a startup integration test; exclusive lock is an OS-level RocksDB guarantee)` |
| BC-2.15.003 | `(none — persist-before-forward covered transitively by VP-033; crash-recovery replay is restart integration test)` |
| BC-2.15.006 | `(none — override-precedence is Option::unwrap_or, unit test sufficient; cannot-disable is hardcoded constant, code review; VP-014/015 cover enforcement)` |
| BC-2.15.008 | `(none — consecutive-counter pattern structurally identical to VP-057; denylist-survives-restart is restart integration test)` |
| BC-2.15.009 | `(none — virtual field queryability requires DataFusion execution context; decorator-always-present is struct initialization property; unit test sufficient)` |
| BC-2.15.010 | `(none — phase priority ordering is HashMap::extend() with defined precedence; unit test sufficient; stale-on-failure requires live background task)` |
| BC-2.15.011 | `(none — audit.read gate covered transitively by VP-002; client_id scoping requires DataFusion integration test)` |
| BC-2.16.001 | `(none — partial-failure isolation is behavioral loop property; OCSF registration is side-effectful; VP-023 covers panic-safety for spec parsing)` |
| BC-2.16.002 | `(none — fan-out concatenation requires HTTP mock integration; forward-reference rejection covered by VP-059)` |
| BC-2.16.003 | `(none — coercion-failure preservation semantically identical to VP-017; cross-sensor correlation requires full query engine integration)` |
| BC-2.16.004 | `(none — catch_unwind panic isolation is construction guarantee; initial-sensor TOML-only is code review invariant; VP-023 covers parser panic safety)` |
| BC-2.16.005 | `(none — fail-closed proven transitively by VP-032; MCP notification on schema change is behavioral integration test)` |
| BC-2.16.006 | `(none — snapshot stability is Arc reference counting type-system guarantee; wait-free load() is arc-swap library property; VP-032 covers swap-side)` |
| BC-2.16.007 | `(none — in-flight query safety transitively covered by VP-032 and BC-2.16.006; cache invalidation is behavioral integration test)` |
| BC-2.16.008 | `(none — temp+rename atomicity is OS-level guarantee; sensor_spec.write gate covered transitively by VP-002)` |
| BC-2.16.010 | `(none — always-visible is boolean field at registration; structuredContent format is serialization integration test; unit test sufficient)` |
