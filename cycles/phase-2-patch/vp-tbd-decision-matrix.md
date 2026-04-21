---
document_type: vp-tbd-decision-matrix
level: L4
version: "1.0"
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
| 1.0 | 2026-04-20 | architect | Initial decision matrix: 22 BCs analyzed, 10 ADD-VP, 12 MARK-NONE, 0 DEFER |
