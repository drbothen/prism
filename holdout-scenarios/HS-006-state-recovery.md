# HS-006: State Recovery Scenarios

**Group:** Restart without data loss, cursor forward progress maintained
**Date:** 2026-04-13
**Priority:** P0

---

## HS-006-01: Clean Restart with Cursor Resume

**Title:** Prism restarts and resumes polling from last persisted cursor position

**Preconditions:**
- Tenant A polling CrowdStrike -- cursor at (timestamp: "2026-04-13T10:00:00Z", record_id: "alert-500")
- Tenant A polling Claroty devices -- cursor at (offset: 1000, device_uid: "dev-200")
- Both cursor states persisted to durable files
- Prism process stopped cleanly (SIGTERM)

**Steps:**
1. Prism receives SIGTERM
2. Graceful shutdown: cancel poll loops, drain in-flight deliveries, persist final cursor state
3. Prism process exits
4. Prism restarts
5. Prism loads cursor state for all tenants and sensors
6. CrowdStrike polling resumes from (timestamp: "2026-04-13T10:00:00Z", record_id: "alert-500")
7. Claroty polling resumes from (offset: 1000, device_uid: "dev-200")

**Expected Outcome:**
- No duplicate records fetched -- polling resumes exactly where it left off
- No gap in records -- cursor position accurate to the last successfully delivered batch
- Query fingerprint validated on resume: if config changed between restarts, cursor reset (not stale cursor with new config)
- Cursor loaded from per-tenant directory: `state/tenant-a/crowdstrike-alerts.json`
- Composite cursor supports variable arity: 2-tuple (CrowdStrike), 3-tuple (Claroty audit logs), 2-tuple with type-specific ID (Armis)
- Health endpoint reports READY after first successful post-restart poll

**Repos Tested:** poller-bear (FileStore atomic persistence, cursor resume), poller-coaster (FileStore, query fingerprint validation on resume)

---

## HS-006-02: Crash Recovery with Atomic State Files

**Title:** Prism crashes mid-write and recovers with consistent state

**Preconditions:**
- Tenant A polling Armis -- 7 data sources with independent cursors
- Prism writing cursor state for `armis_alerts` when process is killed (SIGKILL)

**Steps:**
1. Prism completes Armis alert batch delivery
2. Prism begins atomic state write: creates temp file, writes JSON
3. Process killed (SIGKILL) AFTER temp file write but BEFORE rename
4. Prism restarts
5. Prism finds: old state file (pre-batch) intact, orphan temp file on disk

**Expected Outcome:**
- Old state file is valid (rename is atomic on POSIX -- either happened or didn't)
- Orphan temp file ignored or cleaned up on startup
- Cursor position reflects the last COMPLETED atomic write (not the interrupted one)
- Some records may be re-fetched (between last successful persist and crash) -- this is acceptable (at-least-once delivery)
- Forward progress invariant prevents cursor regression: new cursor must be >= old cursor
- State updated AFTER successful persistence (fixing poller-cobra's ordering bug where in-memory state advances before store.Save())

**Repos Tested:** poller-bear (atomic write: temp -> fsync -> rename), poller-coaster (atomic file persistence), poller-cobra (state-before-persistence ordering bug -- fixed)

---

## HS-006-03: Config Change Detection via Query Fingerprint

**Title:** Changed polling config detected on restart, cursor reset to avoid stale data

**Preconditions:**
- Tenant A polling CrowdStrike with FQL filter: `status: "new"`
- Cursor at position (T100, alert-500)
- Query fingerprint stored: SHA-256 of `sorted(query_params) + "|" + limit`

**Steps:**
1. Operator changes FQL filter to `status: ["new", "in_progress"]`
2. Prism restarts with new config
3. Prism loads cursor for Tenant A CrowdStrike
4. Prism computes new query fingerprint from updated config
5. New fingerprint does not match stored fingerprint

**Expected Outcome:**
- Fingerprint mismatch detected -- cursor state considered invalid
- Cursor RESET to initial position (start from beginning with new query)
- This is the correct behavior: old cursor was based on different query semantics
- Log entry: `{ "event": "query_fingerprint_mismatch", "tenant": "tenant-a", "sensor": "crowdstrike", "action": "cursor_reset" }`
- New fingerprint stored after first successful poll with new config
- Fingerprint computation is order-independent for config fields (sorted before hashing)

**Repos Tested:** poller-cobra (query fingerprint: SHA-256 of query params), poller-express (fingerprint: sorted field names + limit), poller-bear (fingerprint: sorted fields + limit), poller-coaster (fingerprint: AQL query + limit)

---

## HS-006-04: Forward Progress Invariant Prevents Cursor Regression

**Title:** Prism rejects any attempt to move cursor backward

**Preconditions:**
- Tenant A CrowdStrike cursor at (timestamp: T100, record_id: "alert-500")
- API returns a batch where all records have timestamps <= T100

**Steps:**
1. Prism polls CrowdStrike and receives records with timestamps all <= T100
2. Prism computes new cursor from batch: (T90, "alert-480") -- earlier than current
3. Forward progress check compares new cursor against current

**Expected Outcome:**
- Cursor NOT regressed -- remains at (T100, "alert-500")
- Forward progress violation logged as warning (not fatal error)
- Prism resolves the inconsistency between pollers: uniform error handling (not 3/7 sentinel + 4/7 plain as in poller-coaster)
- Composite cursor comparison uses lexicographic ordering: timestamp first, then record_id as tiebreaker
- Empty batch (no records) does not affect cursor position

**Repos Tested:** poller-cobra (forward progress enforcement), poller-coaster (inconsistent error handling: 3/7 sentinel, 4/7 plain -- unified), poller-bear (9 cursor comparators)

---

## HS-006-05: Batch Receipt Audit Trail Survives Restart

**Title:** Batch receipts provide audit trail for delivered data across restarts

**Preconditions:**
- Tenant A Claroty adapter has delivered 150 batches (receipt history bounded to 100 most recent)
- Receipts persisted alongside cursor state

**Steps:**
1. Prism has 100 batch receipts stored (most recent 100 of 150)
2. Prism restarts
3. Prism loads receipts from state file
4. Prism resumes polling and adds new receipts
5. New receipt count exceeds max (configurable) -- oldest receipts trimmed

**Expected Outcome:**
- All 100 receipts survive restart (persisted in state file)
- Each receipt contains: `{ "version": "...", "count": N, "first_id": "...", "last_id": "...", "cursor_applied": {...} }`
- After adding receipt 101, oldest receipt (receipt 51) trimmed -- bounded to configurable max
- Receipts per (tenant, sensor, data_source) -- independent per source
- Operators can audit: "What was the last successful batch for Tenant A's Claroty alerts?"

**Repos Tested:** poller-bear (batch receipts bounded to 100), poller-coaster (configurable max receipts, receipt auditing)

---

## HS-006-06: Multi-Tenant State Recovery After System-Wide Restart

**Title:** All tenants resume correctly from independent cursor positions after full system restart

**Preconditions:**
- 3 tenants, each with 2 sensors, each sensor with multiple data sources
- All have been polling and have established cursor positions
- System-wide restart (e.g., K8s node drain)

**Steps:**
1. All 3 tenants actively polling across 6+ total sensor adapters
2. Prism receives SIGTERM (K8s graceful shutdown)
3. Graceful shutdown: drain in-flight requests, persist all cursor states, close all connections
4. Health server shut down with 5s grace period
5. Prism fully stopped
6. Prism restarts on new node
7. All tenants' cursor states loaded from persistent storage (PVC)
8. All sensors resume polling from their respective positions

**Expected Outcome:**
- All cursor states recovered -- no data re-fetch beyond the last persisted batch
- All query fingerprints validated -- no stale cursors
- All credential files re-read from secret mounts
- OAuth2 tokens re-acquired (not cached across restarts -- file-based only)
- Health endpoints report per-tenant, per-sensor readiness as each sensor completes its first post-restart poll
- Startup credential validation (ping) runs for all tenants before entering poll loop
- Total recovery time reasonable for the number of tenants/sensors (parallel startup, not sequential)

**Repos Tested:** all pollers (state persistence patterns), poller-cobra (health server shutdown never called -- fixed), poller-express (no signal handling -- fixed)

---

## State Checkpoint

```yaml
scenario_group: HS-006
title: State Recovery
scenarios: 6
priority: P0
repos_covered: [poller-cobra, poller-express, poller-bear, poller-coaster]
status: defined
```
