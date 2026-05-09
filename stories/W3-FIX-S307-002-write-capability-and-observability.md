---
story_id: W3-FIX-S307-002
title: "prism-query: WriteExecutor Phase 3 Fetch + SQL DML Routing + Write Observability"
wave: 4
target_module: prism-query
subsystems: [SS-11, SS-05]
priority: P0
depends_on: [S-3.02-FOLLOWUP-RUNTIME, W3-FIX-S307-001]
blocks: [S-5.01-FOLLOWUP-MCP-BOOT]
estimated_days: 3
points: 5
risk: MEDIUM
status: draft
document_type: story
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-08T00:00:00Z"
input-hash: "[md5]"
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-CLEANUP-02"
phase: 3
behavioral_contracts: [BC-2.04.007, BC-2.05.004]
verification_properties: []
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-2.04.007, BC-2.05.004]
anchor_capabilities: [CAP-006, CAP-007]
anchor_subsystem: ["SS-11", "SS-05"]
# Decision: Combined 002+003 story — ACs for Phase 3 fetch and SQL DML routing are
# interleaved around the same WriteExecutor call path and WriteCapableTableProvider struct.
# Splitting would force artificial ordering with no parallelism benefit.
inputs:
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/stories/S-3.07-write-execution.md"
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
---

# W3-FIX-S307-002 — prism-query: WriteExecutor Phase 3 Fetch + SQL DML Routing + Write Observability

## Narrative

As the Prism write execution pipeline, I want `WriteExecutor::execute` to perform a real
Phase 3 record fetch (not an empty `vec![]`), SQL DML verbs (`INSERT INTO`, `UPDATE`,
`DELETE FROM`) to route through `WriteCapableTableProvider` into `WriteExecutor` rather
than returning `NotImplemented`, and write operations to emit structured observability
(error codes, tracing fields, audit entries) per the write-audit ordering contract
(AD-016), so that the full write pipeline functions end-to-end and analysts receive
accurate feedback on write operation outcomes.

## Objective

Resolve three write pipeline gaps identified by audit finding F-AUD-D1-06:

1. **Phase 3 fetch gap** (`write_pipeline.rs:349`): Replace `let fetched_records: Vec<...> = vec![];`
   with a real call to `QueryMaterializer::execute(source_query, context)`.

2. **SQL DML routing gap** (`write_table_registration.rs:176/190/205`): Replace
   `DataFusionError::NotImplemented("S-3.07-pending")` in `insert_into`, `delete_from`,
   and `update` with real routing through `WriteExecutor::execute`.

3. **Observability gap**: Add `error_code()` helper method to `WriteError`, structured
   tracing fields on all write outcomes, and audit emission for write operations per
   AD-016 write-audit ordering.

**Architect's note on 002+003 combination:** The ACs for Phase 3 fetch (formerly 002)
and SQL DML routing (formerly 003) are not independent — `insert_into` calls
`WriteExecutor::execute` which calls the Phase 3 fetch. Splitting them would require
a stub in 002 that 003 replaces. Combined scope remains at 5 pts.

No `todo!()`, `unimplemented!()`, or `panic!("stub")` may remain in any of the three
gap sites before this story merges. Per POL-12.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.04.007 | Three-Tier Risk Classification for Operations |
| BC-2.05.004 | Write Operations Log Capability Check and Execution Outcome |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,000 |
| `write_pipeline.rs` (Phase 3 fetch real impl) | ~2,000 |
| `write_table_registration.rs` (insert_into/delete_from/update) | ~3,000 |
| `write_observability.rs` (error_code helper, tracing fields) | ~1,500 |
| BC files (2 BCs) | ~1,000 |
| Integration tests | ~3,000 |
| Total | ~13,500 |

Within the 30% context window budget.

---

## Tasks

1. **Phase 3 fetch real implementation** in `crates/prism-query/src/write_pipeline.rs:349`:

   Replace:
   ```rust
   let fetched_records: Vec<RecordBatch> = vec![]; // STUB
   ```
   With:
   ```rust
   let fetched_records = self
       .materializer
       .execute(&plan.source_query, &context)
       .await?;
   // fetched_records is the RecordBatch slice that the write operation targets.
   // The 10K row cap from the read pipeline applies here (BC-2.11.006 security limits).
   // The batch limit post-fetch check (S-3.07 Task 10) runs next.
   ```
   The `QueryMaterializer` is the same materialization engine implemented by
   `S-3.02-FOLLOWUP-RUNTIME`. `WriteExecutor` must hold an `Arc<QueryMaterializer>`
   (injected at construction or via `WriteExecutor::with_materializer`). If the
   `QueryMaterializer` is not yet injectable at `WriteExecutor::new`, add the field.

2. **`WriteCapableTableProvider::insert_into`** at `write_table_registration.rs:176`:

   Replace `DataFusionError::NotImplemented("S-3.07-pending")` with:
   ```rust
   async fn insert_into(
       &self,
       state: &dyn Session,
       input: Arc<dyn ExecutionPlan>,
       insert_op: InsertOp,
   ) -> Result<Arc<dyn ExecutionPlan>> {
       // Extract WritePlan from the DataFusion LogicalPlan context.
       // Retrieve QueryContext from state extensions.
       let query_ctx = state.config().get_extension::<QueryContext>()
           .ok_or_else(|| DataFusionError::Internal("QueryContext missing from session".into()))?;
       let plan = WritePlan::from_insert(self.endpoint_spec.clone(), input, insert_op)?;
       let executor = self.write_executor.clone();
       // Execute asynchronously; serialize result as a single-row RecordBatch.
       let result = executor.execute(plan, query_ctx.clone()).await
           .map_err(|e| DataFusionError::External(Box::new(e)))?;
       Ok(result_to_execution_plan(result)?)
   }
   ```
   The `result_to_execution_plan` helper serializes `WriteResult` or `WritePreview` as
   a single-row schema (operation_id, dry_run, affected_count, succeeded_count, failed_count).

3. **`WriteCapableTableProvider::delete_from`** at `write_table_registration.rs:190`:
   Same pattern as `insert_into`. SQL `DELETE FROM` always maps to `Irreversible` risk tier
   per AD-022 regardless of endpoint spec (S-3.07 design decision).

4. **`WriteCapableTableProvider::update`** at `write_table_registration.rs:205`:
   Same pattern as `insert_into`. Extract UPDATE SET fields from DataFusion plan; pass as
   `params: HashMap<String, String>` to `WriteExecutor::execute`.

5. **Write observability** — new module `crates/prism-query/src/write_observability.rs`:

   ```rust
   /// error_code() helper: maps PrismError variants to canonical E-XXX-NNN strings.
   pub fn error_code(err: &PrismError) -> &'static str {
       match err {
           PrismError::WriteNotImplemented { .. } => "E-WRITE-001",
           PrismError::FeatureFlagDenied { .. } => "E-FLAG-001",
           PrismError::CapabilityDenied { .. } => "E-FLAG-002",
           PrismError::BatchLimitExceeded { .. } => "E-QUERY-021",
           PrismError::UnboundedWrite { .. } => "E-QUERY-022",
           PrismError::AuditFailure { .. } => "E-AUDIT-001",
           PrismError::TokenNotFound { .. } => "E-FLAG-008",
           PrismError::TokenExpired { .. } => "E-FLAG-003",
           // ... map remaining variants
           _ => "E-WRITE-999",
       }
   }
   ```
   Add structured tracing fields to every write outcome in `write_pipeline.rs`:
   ```rust
   tracing::info!(
       sensor = %plan.sensor_name,
       org_id = %plan.org_id,
       endpoint_id = %plan.endpoint_id,
       affected_count = result.affected_count,
       succeeded_count = result.succeeded_count,
       failed_count = result.failed_count,
       "write completed"
   );
   ```
   On write failure:
   ```rust
   tracing::error!(
       error_code = error_code(&err),
       sensor = %plan.sensor_name,
       "write failed"
   );
   ```

6. **Audit emission for write outcomes** (per AD-016 write-audit ordering, BC-2.05.004):
   Verify that `WriteExecutor::execute` correctly:
   - Emits the INTENT audit entry before any sensor API call (existing in S-3.07, verify not broken).
   - Emits the OUTCOME audit entry after fan-out completes.
   If either is missing from the existing implementation, fix it in this story.
   The audit entry must include `error_code` from the new helper when the write fails.

7. **Integration tests** in `crates/prism-query/tests/write_pipeline_e2e_tests.rs`:
   - Test: `INSERT INTO crowdstrike_detections SET status='acknowledged' WHERE id='X'`
     → routes through `WriteCapableTableProvider::insert_into` → `WriteExecutor::execute`
     → Phase 3 fetches real records (non-empty) → DTU write call succeeds.
   - Test: Phase 3 fetch returns 0 records (empty source) → `WriteResult { affected_count: 0 }`.
   - Test: `DELETE FROM claroty_alerts WHERE id='X'` → classified `Irreversible` → preview returned.
   - Test: `error_code()` returns correct E-NNN string for each error variant.
   - Test: write success emits tracing span with `succeeded_count` field (capture via tracing subscriber).
   - Test: INTENT audit entry is durable before sensor API is called (inject audit failure; verify no sensor call).

---

## Acceptance Criteria

**AC-1:** Given `WriteExecutor::execute(plan, ctx)` where `plan.source_query` targets a
CrowdStrike sensor, When Phase 3 runs, Then `QueryMaterializer::execute(source_query, ctx)`
is called and returns a non-empty `RecordBatch` (not `vec![]`). The fetched records are
passed to Phase 5 sensor dispatch.
(traces to BC-2.04.007 postcondition — risk-tiered write dispatches real records)

**AC-2:** Given `INSERT INTO crowdstrike_detections SET status='acknowledged' WHERE severity > 3`,
When DataFusion plans the DML and calls `WriteCapableTableProvider::insert_into`, Then the
same 6-phase safety pipeline runs as pipe-mode write (feature flag check, risk tier, dry-run
default, intent log, fan-out, outcome log). No `DataFusionError::NotImplemented` is returned.
(traces to BC-2.04.007 postcondition — SQL DML routes through same safety pipeline)

**AC-3:** Given `DELETE FROM claroty_alerts WHERE rule_id='X'`, When
`WriteCapableTableProvider::delete_from` processes the DML, Then the write is classified
`Irreversible` unconditionally (AD-022 requirement; no spec override for DELETE).
(traces to BC-2.04.007 postcondition — DELETE is always Irreversible)

**AC-4:** Given `UPDATE armis_devices SET acknowledged=true WHERE device_id='X'`, When
`WriteCapableTableProvider::update` processes the DML, Then `WriteExecutor::execute` is
called with the UPDATE params as a `HashMap<String, String>`. No `NotImplemented` error.
(traces to BC-2.04.007 postcondition — UPDATE routes through write executor)

**AC-5:** Given a write operation that fails with `PrismError::FeatureFlagDenied`, When
the observability layer fires, Then `tracing::error!` is emitted with `error_code = "E-FLAG-001"`
as a structured field.
(traces to BC-2.05.004 postcondition — write operations log capability check outcome)

**AC-6:** Given a successful write operation, When the pipeline completes, Then a
`tracing::info!` span is emitted with structured fields: `sensor`, `org_id`, `endpoint_id`,
`affected_count`, `succeeded_count`, `failed_count`.
(traces to BC-2.05.004 postcondition — write execution outcome is audit-logged)

**AC-7:** Given `WriteExecutor::execute` where the audit INTENT write fails, When Phase 5
begins, Then the intent failure results in `E-AUDIT-001` and NO sensor API call is made.
The INTENT entry must be durable before any sensor contact — audit fail-closed invariant.
(traces to BC-2.05.004 postcondition — write audit is fail-closed)

**AC-8:** No `todo!()`, `unimplemented!()`, or `panic!("stub")` may remain in
`write_pipeline.rs:349`, `write_table_registration.rs:176`, `write_table_registration.rs:190`,
or `write_table_registration.rs:205` before merge. Per POL-12.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `WriteExecutor::execute` Phase 3 | `prism-query` (SS-11) | Effectful (HTTP materialization) |
| `WriteCapableTableProvider::insert_into/delete_from/update` | `prism-query` (SS-11) | Effectful |
| Write observability (`error_code`, tracing) | `prism-query` (SS-11) | Mixed |
| Audit emission (INTENT/OUTCOME) | `prism-audit` (SS-05) | Effectful |

Per `architecture/module-decomposition.md`, `prism-query` owns SS-11 (Query Execution)
and consumes SS-05 (Audit Trail) via `AuditEmitter`. Write pipeline observability lives
in `prism-query/src/write_observability.rs`.

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `error_code()` helper | Pure | Deterministic mapping from error variant to string. |
| `WriteCapableTableProvider` DML methods | Effectful | Async write dispatch; audit I/O. |
| Phase 3 fetch | Effectful | HTTP sensor fan-out via QueryMaterializer. |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `WriteCapableTableProvider` DML methods MUST route through `WriteExecutor::execute` | S-3.07 Architecture Compliance Rules + AD-022 | Code review; no direct sensor API calls from TableProvider |
| INTENT audit entry MUST be durable before any sensor API call | AD-016 write-audit ordering + BC-2.05.004 invariant | AC-7 integration test; fail-closed audit injection test |
| `error_code()` MUST reference only error codes in the error taxonomy | project rule (avoid inventing codes) | Code review against `prd-supplements/error-taxonomy.md` |
| `DELETE FROM` SQL DML MUST be classified `Irreversible` unconditionally | AD-022 requirement | AC-3 integration test |
| Phase 3 fetch MUST apply the 10K row materialization cap | BC-2.11.006 (inherited from read pipeline) | Code review; `QueryMaterializer` enforces the cap |

**Forbidden Dependencies:** `prism-query` MUST NOT depend on `prism-mcp` or `prism-bin`.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| datafusion | "53.1" (workspace) | `TableProvider` write methods (insert_into / delete_from / update), `InsertOp`, `Session` |
| tokio | 1.x (workspace) | Async write execution |
| tracing | 0.1.x (workspace) | Structured write-outcome spans |
| prism-audit | workspace | `AuditEmitter` for INTENT/OUTCOME entries |
| prism-core | workspace | `PrismError`, error code mapping |
| prism-query | workspace (self) | `WriteExecutor`, `QueryMaterializer`, `WritePlan` |

**DataFusion write API note:** The exact signature of `insert_into`, `delete_from`, and
`update` in DataFusion 53.1 must be verified at TDD time (inherited flag from S-3.07).
The implementer MUST read the DataFusion 53.1 `TableProvider` trait definition before
implementing the DML methods. If the API differs from the assumed shape above, adjust
to match — do not expand scope; file a W3-FIX-* if needed.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-query/src/write_pipeline.rs` | Modify | Replace Phase 3 `vec![]` stub (line 349) with `QueryMaterializer::execute` call |
| `crates/prism-query/src/write_table_registration.rs` | Modify | Replace `NotImplemented` in insert_into (176), delete_from (190), update (205) |
| `crates/prism-query/src/write_observability.rs` | Create | `error_code()` helper; structured tracing macros for write outcomes |
| `crates/prism-query/src/lib.rs` | Modify | Re-export `write_observability` |
| `crates/prism-query/tests/write_pipeline_e2e_tests.rs` | Create | End-to-end write pipeline integration tests |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Phase 3 fetch: source query returns 0 records | `WriteResult { affected_count: 0, succeeded_count: 0 }` — not an error |
| EC-002 | Phase 3 fetch: source query exceeds 10K row cap | `E-QUERY-021` (batch limit exceeded); no sensor write dispatch |
| EC-003 | `insert_into` called but `QueryContext` not in session state | `DataFusionError::Internal` (not a crash); audit entry not emitted |
| EC-004 | `update` with empty SET clause | Reject at SQL parse time (DataFusion); no `WriteExecutor` invocation |
| EC-005 | Concurrent SQL DML calls (multiple `insert_into` inflight) | Write semaphore (capacity 4) throttles concurrency; beyond-cap calls wait or error |

---

## Previous Story Intelligence

**S-3.07 (Write Execution Pipeline — partial-merge):** This story fills the remaining
stubs that S-3.07 left. Key S-3.07 design decisions carried forward:
- Phase 3 fetch is between Phase 2 (safety) and Phase 4 (dry-run gate). Do not reorder.
- `WriteCapableTableProvider` receives a `WriteEndpointSpec` and `Arc<WriteExecutor>` at construction.
- Partial write failure (some records fail) is `Ok(WriteResult { failed_count > 0 })` — not `Err`.
- The write semaphore (capacity 4) bounds concurrent fan-out per `WriteExecutor` instance.

**S-3.02-FOLLOWUP-RUNTIME (dependency):** Must be merged before this story. This story
calls `QueryMaterializer::execute` which is implemented by S-3.02-FOLLOWUP-RUNTIME. Do
not dispatch this story until S-3.02-FOLLOWUP-RUNTIME is in `merged` state.

**W3-FIX-S307-001 (dependency):** Sensor adapter write overrides must exist before Phase 5
(fan-out) can dispatch to real sensors. W3-FIX-S307-001 must merge before or concurrently.

---

## Dev Notes

- `QueryMaterializer` may need an `execute_for_write` method signature that returns
  `Vec<RecordBatch>` directly (rather than the `Arc<SessionContext>` returned by
  `execute_scheduled`). Check the existing `QueryMaterializer` interface and add a
  write-compatible method if needed — do not break the existing read-path interface.
- The `result_to_execution_plan` helper for returning DataFusion write results must
  return a valid `Arc<dyn ExecutionPlan>`. Use `MemoryExec::try_new(batches, schema, ...)`.
- For `delete_from`: DataFusion may not expose this as a `TableProvider` trait method in
  all versions. If unavailable, wrap the logic in `insert_into` with `InsertOp::Delete`
  flag. Verify at TDD time.

---

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | Bundle-B-Phase-B-1 | 2026-05-08 | story-writer | Initial story creation from ADR-022 §G seed (Story 4). Combined W3-FIX-S307-002 + W3-FIX-S307-003 per architect recommendation — ACs are interleaved across the same code path. |
