# HS-007: Cross-Repo Failure Scenarios

**Group:** What happens when a pattern from one repo fails in the unified context
**Date:** 2026-04-13
**Priority:** P1

---

## HS-007-01: MemoryStore Pattern Leaks Into Production Code

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

## HS-007-02: N-Way Collector Duplication Eliminated by Generic Trait

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

## HS-007-03: Cobra's State-Before-Persistence Bug Cannot Recur

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

## HS-007-04: Express's Strict JSON Decoding Replaced with Lenient Parsing

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

## HS-007-05: Bear's Polymorphic JSON IDs Handled in Typed Rust Context

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

## HS-007-06: ServeMyAPI's Path Traversal Prevented in Credential Store

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

## HS-007-07: Tally's Error Code Mapping Unified Across All Tools

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

## HS-007-08: Axiathon's Unbounded Caches Bounded in Prism

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

## State Checkpoint

```yaml
scenario_group: HS-007
title: Cross-Repo Failure
scenarios: 8
priority: P1
repos_covered: [poller-cobra, poller-express, poller-bear, poller-coaster, serveMyAPI, tally, axiathon, mcp-claroty-xdome, ocsf-proto-gen]
status: defined
```
