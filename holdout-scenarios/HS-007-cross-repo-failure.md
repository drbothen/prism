---
# S-3.6.02 STUB — placeholder anchors, awaiting failing-test phase to validate the new Wave 3 BC mappings
# TODO(S-3.6.02 failing-test): replace behavioral_contracts: [] with [BC-3.6.001, BC-3.6.002, BC-3.5.002]
# TODO(S-3.6.02 failing-test): update phase to 3.A
# TODO(S-3.6.02 failing-test): update timestamp to 2026-04-27T00:00:00Z
document_type: holdout-scenario
level: L3
id: "HS-007"
category: "cross-repo-failure"
must_pass: true
priority: P1
epic_id: "E-3.6"
version: "1.1"
status: draft
producer: stub-architect
timestamp: "2026-04-29T00:00:00Z"
phase: "1b"
inputs: []
input-hash: null
traces_to: prd.md
behavioral_contracts: []
closes_td: []
lifecycle_status: active
introduced: cycle-1
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "STUB — S-3.6.02 Red Gate placeholder. Phase 1b brownfield-repo sub-scenarios retained below; Wave 3 anchors (BC-3.6.001, BC-3.6.002, BC-3.5.002) not yet written. Do not merge until failing-test phase installs correct anchors."
---

# HS-007: Cross-Repo Failure Scenarios (STUB — Wave 3 re-anchor in progress)

**Group:** Per-customer DTU failure isolation; customer A fails, customer B unaffected
**Date:** 2026-04-29
**Priority:** P1
**Status:** STUB — awaiting Wave 3 BC anchor validation (S-3.6.02 failing-test phase)

> **NOTE:** This file is the S-3.6.02 Red Gate stub. The sub-scenarios below are
> placeholder outlines only. They intentionally omit the Wave 3 BC IDs, harness
> module names (`prism-dtu-harness`), `IsolationMode::Network`, and
> `HarnessError::CloneCrashed` references that the acceptance criteria require.
> The failing-test phase will assert their absence (triggering Red Gate) and the
> implementation phase will fill them in.

---

## Scenario

<!-- TODO(S-3.6.02): replace with Wave 3 IsolationMode::Network cross-customer failure scenario -->
<!-- Primary: inject_failure(org_slug_A, DtuType::Claroty, FailureMode::AuthReject);            -->
<!--   org A returns HTTP 401; org B unaffected; clear_failure restores org A.                  -->

**STUB — Wave 3 sub-scenarios pending (S-3.6.02 implementation phase)**

### HS-007-01: Cross-Customer Failure Isolation (STUB)

**Title:** TODO — inject per-org failure on org A; verify org B unaffected in Network harness

**Preconditions:**
- TODO: two orgs registered in `IsolationMode::Network` harness (BC-3.5.002)
- TODO: per-org `FailureLayerShared` state initialized

**Steps:**
1. TODO: call `inject_failure(org_slug_A, dtu_type, FailureMode::AuthReject)` for org A
2. TODO: org A DTU queries return HTTP 401
3. TODO: org B DTU queries return HTTP 200 with valid data
4. TODO: call `clear_failure(org_slug_A, dtu_type)`; org A resumes HTTP 200

**Expected Outcome:**
- TODO: org A isolated failure; org B FailureLayerShared state unchanged
- TODO: cite BC-3.6.001 postconditions 1, 2, 3 (NOT YET WRITTEN — Red Gate)

**BC Anchors:** TODO — [BC-3.6.001, BC-3.5.002] (stub: not yet installed)

---

### HS-007-02: Routing Bug Detection via Network Mode (STUB)

**Title:** TODO — wrong-org credentials to live clone endpoint return HTTP 401

**Preconditions:**
- TODO: org A and org B clones running on separate SocketAddrs (IsolationMode::Network)

**Steps:**
1. TODO: send org A credentials to org B SocketAddr
2. TODO: assert HTTP 401 response (routing bug detectable)

**Expected Outcome:**
- TODO: cross-process credential routing rejected (BC-3.5.002 postcondition 2 — NOT YET WRITTEN)

**BC Anchors:** TODO — [BC-3.5.002] (stub: not yet installed)

---

### HS-007-03: Crash Detection After Auth-Reject (STUB)

**Title:** TODO — inject InternalError; verify HarnessError::CloneCrashed for org A within 1s

**Preconditions:**
- TODO: org A and org B clones running in harness

**Steps:**
1. TODO: inject `FailureMode::InternalError` on org A clone
2. TODO: org A clone panics
3. TODO: harness detects `HarnessError::CloneCrashed` for org A within 1s
4. TODO: org B continues returning HTTP 200

**Expected Outcome:**
- TODO: `HarnessError::CloneCrashed` for org A only (BC-3.6.002 — NOT YET WRITTEN)
- TODO: no cross-tenant effect

**BC Anchors:** TODO — [BC-3.6.002] (stub: not yet installed)

## Behavioral Contract Linkage

<!-- TODO(S-3.6.02): update table after failing-test phase confirms BC IDs resolve -->

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| TODO-BC-3.6.001 | postcondition 1 — per-org failure scope | inject_failure affects only org A |
| TODO-BC-3.6.001 | postcondition 3 — idempotent clear | clear_failure restores normal behavior |
| TODO-BC-3.6.002 | postcondition 1 — crash detected within 1s | InternalError clone crash detection |
| TODO-BC-3.5.002 | postcondition 2 — network routing isolation | wrong-org credentials return HTTP 401 |

## Verification Approach

<!-- TODO(S-3.6.02): replace with Wave 3 harness invocation steps -->

**STUB — pending Wave 3 rewrite**

Verification will use `prism-dtu-harness` test harness with `IsolationMode::Network`,
invoking `inject_failure` and `clear_failure` from `prism-dtu-common::layers::failure`.
Audit trail assertions use `prism-audit` RocksDB store.

## Evaluation Rubric

<!-- TODO(S-3.6.02): refine weights after Wave 3 sub-scenario finalization -->

**STUB — pending Wave 3 rewrite**

- **Functional correctness** (weight: 0.4): org A failure injected; org B unaffected
- **Edge case handling** (weight: 0.2): `clear_failure` idempotent (EC-003); 2-org minimum enforced (EC-002)
- **Error quality** (weight: 0.2): `HarnessError::CloneCrashed` returned within 1s for BC-3.6.002
- **Performance** (weight: 0.1): failure injection and detection within NFR thresholds
- **Data integrity** (weight: 0.1): audit trail scoped to org A `OrgId` only; no org B audit event

## Edge Conditions

<!-- TODO(S-3.6.02): confirm EC-001 DTU type once Wave 3 type list is finalized -->

**STUB — pending Wave 3 rewrite**

- EC-001: DTU type substitution — use `DtuType::Claroty` if `DtuType::PagerDuty` not in Wave 3 type list
- EC-002: scenario requires at least 2 orgs; single-org harness insufficient for BC-3.5.002
- EC-003: `clear_failure` called when no failure active returns `Ok(())`; scenario asserts no error

## Failure Guidance

<!-- TODO(S-3.6.02): update HS number and failure message template for Wave 3 scenario -->

**STUB — pending Wave 3 rewrite**

Template: "HOLDOUT LOW: HS-007 (satisfaction: 0.XX) -- per-org failure isolation not working: [describe which org was affected unexpectedly or which BC clause failed]"

## Category: real-world-corpus

<!-- NOTE: HS-007 is cross-customer failure isolation, not a real-world corpus test.   -->
<!-- This section is present for template compliance only; corpus fields are N/A here. -->

**STUB — HS-007 is not a real-world-corpus scenario; section present for template compliance**

| Field | Description |
|-------|-------------|
| corpus_source | N/A — synthetic multi-tenant harness scenario |
| corpus_size | N/A |
| known_edge_cases | See EC-001 through EC-003 above |
| false_positive_threshold | N/A |
| false_negative_threshold | N/A |

---

## Phase 1b Sub-Scenarios (RETAINED — to be removed in S-3.6.02 implementation phase)

> **S-3.6.02 NOTE:** The following sub-scenarios (HS-007-01 through HS-007-08) are Phase 1b
> brownfield-repo content. They will be REMOVED and replaced with the three Wave 3
> sub-scenarios above in the implementation phase.

### Phase 1b HS-007-01: MemoryStore Pattern Leaks Into Production Code

**Title:** Verify no code path uses in-memory-only state in production mode

**Preconditions:**
- Prism configured for production deployment
- StateStore trait with FileStore (production) and MemoryStore (test-only) implementations

**Steps:**
1. Code review / integration test: attempt to instantiate MemoryStore in non-test configuration
2. Verify all production code paths use durable FileStore
3. Simulate FileStore initialization failure
4. Verify Prism does NOT silently fall back to MemoryStore

**Expected Outcome:**
- MemoryStore only available behind `#[cfg(test)]` or test feature flag
- No production code path can accidentally use MemoryStore (compile-time guarantee if possible)
- FileStore initialization failure is a fatal startup error, not a silent fallback
- This prevents the poller-cobra bug: `StateConfig` supports file/memory, Helm sets `file`, but runner hardcodes `MemoryStore`
- Also prevents the poller-express pattern: MemoryStore is the only implementation

**Repos Tested:** poller-cobra (MemoryStore hardcoded despite FileStore config -- BUG), poller-express (MemoryStore only -- BUG)

---

### Phase 1b HS-007-02: N-Way Collector Duplication Eliminated by Generic Trait

**Title:** Generic DataSource trait handles all sensor/source combinations without duplication

**Preconditions:**
- Prism's `DataSource` trait implemented for all sensor adapters
- At least 2 sensors with multiple data sources (Claroty: 9, Armis: 7)

**Steps:**
1. Add a new data source to Claroty adapter (e.g., new endpoint)
2. Implement only the source-specific parts: endpoint URL, cursor type, field mapping
3. All shared behavior (retry loop, cursor management, enrichment, delivery) inherited from trait default implementations
4. Run full test suite

**Expected Outcome:**
- Adding a new data source requires implementing ~3-5 trait methods, not copying 100+ lines of collection loop
- All 9 Claroty sources and 7 Armis sources share a single generic collection loop
- The generic trait resolves: poller-bear's 9x code duplication (1,367 lines in collector.go), poller-coaster's 7x duplication, poller-express's 2x duplication
- Variation table pattern (from poller-coaster analysis) captured as trait associated types: CursorType, RecordType, ConfigType
- Bug fixes apply uniformly -- no risk of fixing 6 of 7 collectors and missing one

**Repos Tested:** poller-bear (9x duplication), poller-coaster (7x duplication, variation table pattern), poller-express (2x duplication)

---

### Phase 1b HS-007-03: Cobra's State-Before-Persistence Bug Cannot Recur

**Title:** Cursor state never advances before successful persistence

**Preconditions:**
- Prism processing a batch of alerts for Tenant A
- FileStore configured for durable persistence

**Steps:**
1. Prism fetches 50 alerts from CrowdStrike
2. Prism delivers all 50 to sink successfully
3. Prism computes new cursor position
4. Prism attempts atomic write of new cursor to FileStore
5. FileStore write FAILS (disk full, permissions error)
6. Check in-memory cursor state

**Expected Outcome:**
- In-memory cursor state NOT advanced (still at old position)
- On next poll cycle, Prism re-fetches the same 50 alerts (at-least-once delivery)
- This fixes poller-cobra's bug: `alertState = nextState` runs BEFORE `store.Save()`, causing cursor to advance past undelivered alerts on persistence failure
- State update sequence: fetch -> deliver -> persist -> advance in-memory (strictly ordered)
- Error logged: `{ "event": "state_persistence_failed", "tenant": "tenant-a", "sensor": "crowdstrike", "error": "disk full" }`

**Repos Tested:** poller-cobra (state-before-persistence ordering bug)

---

### Phase 1b HS-007-04: Express's Strict JSON Decoding Replaced with Lenient Parsing

**Title:** Unknown JSON fields from sensor APIs do not cause deserialization failure

**Preconditions:**
- Cyberint API has added new fields in a minor version update
- Prism's Cyberint adapter uses `#[serde(deny_unknown_fields)]` (the Rust equivalent of poller-express's `DisallowUnknownFields`)

**Steps:**
1. Prism polls Cyberint alert API
2. Response includes known fields + 3 new unknown fields
3. Prism deserializes response

**Expected Outcome:**
- Deserialization SUCCEEDS -- unknown fields silently ignored (or captured in catch-all)
- This verifies the fix for poller-express's `DisallowUnknownFields` pattern that breaks forward compatibility
- All sensor adapter response types use `#[serde(default)]` for optional fields (ocsf-proto-gen's tolerant parsing pattern)
- Unknown fields optionally captured via `#[serde(flatten)] extra: HashMap<String, Value>` for the unmapped JSON blob
- No sensor API version upgrade causes Prism deserialization failures

**Repos Tested:** poller-express (strict JSON decoding -- DisallowUnknownFields), ocsf-proto-gen (tolerant serde defaults)

---

### Phase 1b HS-007-05: Bear's Polymorphic JSON IDs Handled in Typed Rust Context

**Title:** Claroty API returning IDs as both strings and numbers parsed correctly

**Preconditions:**
- Claroty xDome API returns some device IDs as `"123"` (string) and others as `123` (number)
- Prism's Claroty adapter must handle both forms

**Steps:**
1. Prism fetches devices from Claroty
2. First device has `device_uid: "456"` (string)
3. Second device has `device_uid: 789` (number)
4. Both must be parsed into the same cursor ID type

**Expected Outcome:**
- Both string and number forms parsed without error
- Both normalize to the same internal representation (string, for cursor comparison)
- Custom deserializer handles the polymorphism (serde `deserialize_any` or `untagged` enum)
- Cursor comparison works correctly regardless of original JSON type
- This addresses poller-bear's documented polymorphic JSON handling requirement
- Also handles poller-express's string comparison of numeric asset IDs bug -- IDs compared numerically when appropriate

**Repos Tested:** poller-bear (polymorphic JSON ID handling), poller-express (string comparison of numeric IDs bug)

---

### Phase 1b HS-007-06: ServeMyAPI's Path Traversal Prevented in Credential Store

**Title:** Malicious credential names cannot escape storage directory

**Preconditions:**
- Prism credential store using encrypted file backend
- Attacker controls credential name input

**Steps:**
1. Attempt to store credential with name: `"../../../etc/shadow"`
2. Attempt to store credential with name: `"tenant-a/../../tenant-b/secret"`
3. Attempt to store credential with name: `"normal-credential-name"`

**Expected Outcome:**
- Path traversal attempt 1: REJECTED -- name contains `..`
- Path traversal attempt 2: REJECTED -- name contains path separators
- Normal name: ACCEPTED
- Credential names validated against allowlist: `[a-zA-Z0-9_.-]` only
- This fixes serveMyAPI's CRITICAL path traversal vulnerability (key names used directly as file paths with zero sanitization)
- Validation happens at the service layer, not just transport (fixing serveMyAPI's pattern where Zod validates at transport but CLI bypasses)

**Repos Tested:** serveMyAPI (path traversal vulnerability, validation gap between MCP and CLI)

---

### Phase 1b HS-007-07: Tally's Error Code Mapping Unified Across All Tools

**Title:** All MCP error responses use consistent JSON-RPC error codes

**Preconditions:**
- Prism MCP server running with multiple tool types
- Various error conditions triggerable

**Steps:**
1. Trigger authentication error -> expect specific error code
2. Trigger not-found error -> expect specific error code
3. Trigger rate-limit error -> expect specific error code
4. Trigger internal error -> expect specific error code
5. Verify error mapping is centralized (not per-tool)

**Expected Outcome:**
- All errors mapped through a centralized `to_mcp_error()` function (fixing tally's distributed `to_mcp_err()` pattern)
- Error codes follow JSON-RPC 2.0 standard (mcp-claroty-xdome's typed hierarchy pattern)
- Not ErrorCode(-1) for everything (fixing tally's catch-all pattern)
- Error messages are actionable without leaking sensitive information
- Error types are `#[non_exhaustive]` (tally's pattern, preventing exhaustive matching on public API)

**Repos Tested:** tally (to_mcp_err with ErrorCode(-1) -- improved), mcp-claroty-xdome (typed error hierarchy -> JSON-RPC 2.0 codes)

---

### Phase 1b HS-007-08: Axiathon's Unbounded Caches Bounded in Prism

**Title:** All in-memory caches have size limits and TTL

**Preconditions:**
- Prism running with multiple tenants and sensors
- Caches configured with max entries and TTL

**Steps:**
1. Insert entries until cache reaches max size
2. Verify LRU eviction occurs (oldest entry removed)
3. Insert entry, wait for TTL expiry
4. Verify expired entry not returned
5. Monitor memory usage under sustained load

**Expected Outcome:**
- Cache size bounded: max N entries per (tenant, sensor) cache instance
- LRU eviction when capacity reached
- TTL-based expiry (e.g., 5 minutes, matching mcp-claroty-xdome's default)
- Memory usage stable under sustained load (no unbounded growth)
- This fixes: mcp-claroty-xdome's unbounded caches (5 domain services growing without limit), mcp-claroty-xdome's session accumulation (no expiration), poller-express/coaster's unbounded per-IP rate limiter maps

**Repos Tested:** mcp-claroty-xdome (unbounded caches, session leak), poller-express (unbounded rate limiter map), poller-coaster (unbounded rate limiter map), axiathon (CWE-cited limits as reference)

---

## State Checkpoint (STUB)

```yaml
scenario_group: HS-007
title: Cross-Repo Failure (Wave 3 re-anchor — STUB)
# TODO(S-3.6.02 failing-test): update scenarios count to 3 (Wave 3: HS-007-01, HS-007-02, HS-007-03)
scenarios: 8
priority: P1
# TODO(S-3.6.02 failing-test): replace [] with [BC-3.6.001, BC-3.6.002, BC-3.5.002]
behavioral_contracts: []
# TODO(S-3.6.02 failing-test): replace [] with [TD-HOLDOUT-W2-NNN] if applicable
closes_td: []
status: stub
```
