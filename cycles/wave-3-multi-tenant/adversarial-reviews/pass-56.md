---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-07T00:00:00
phase: maintenance
inputs:
  - PR #132 diff (S-3.05 pagination + caching)
  - crates/prism-query/src/cache.rs (1527 lines new)
  - crates/prism-query/src/cache_key.rs (new)
  - crates/prism-query/src/cursor.rs (476 lines new)
  - crates/prism-query/src/engine.rs (modified)
  - crates/prism-query/src/invalidation.rs (new)
  - crates/prism-query/src/lib.rs (modified)
  - crates/prism-query/src/proofs/vp025_cache_key.rs (new)
  - crates/prism-query/src/tests/cache_tests.rs (new)
  - crates/prism-query/src/tests/pagination_tests.rs (new)
  - crates/prism-spec-engine/src/plugin/sandbox.rs (minor field rename)
  - crates/prism-core/src/error.rs (modified)
  - .factory/specs/behavioral-contracts/BC-2.07.001-ephemeral-cursor-pagination.md
  - .factory/specs/behavioral-contracts/BC-2.07.002-pagination-token-lifecycle.md
  - .factory/specs/behavioral-contracts/BC-2.07.003-response-cache-ttl.md
input-hash: "[not-computed]"
traces_to: BC-2.07.001, BC-2.07.002, BC-2.07.003, BC-2.07.004, BC-2.07.005, BC-2.07.006
pass: 56
previous_review: pass-55.md
scope: PR #132 diff-only (feature/S-3.05 pagination + caching — 17 prior local adversary passes)
---

# Adversarial Review: PR #132 — S-3.05 Pagination + Caching (Pass 56)

## Scope Note

This is a targeted PR diff review. The adversary reviewed the diff-only changed files
(cache.rs, cache_key.rs, cursor.rs, engine.rs, invalidation.rs, lib.rs, vp025_cache_key.rs,
cache_tests.rs, pagination_tests.rs, sandbox.rs, error.rs) plus authoritative spec context
(BC-2.07.001–006). Focus areas per dispatch: (1) cursor TTL from `created_at` vs last access;
(2) put_with_ttl eviction-path residual race documentation accuracy; (3) edge cases not covered
by tests; (4) error code monotonicity; (5) moka TTL vs manual expiry stale-serve risk.

This story has had 17 prior local adversary passes. This review targets diff-only blind spots
not caught by local passes.

## Finding ID Convention

Finding IDs use: `ADV-W3MT-P56-<SEV>-<SEQ>`

## Part A — Fix Verification (pass >= 2 only)

Not applicable — this is a fresh PR diff review targeting a new story (S-3.05), not a
continuation of a prior pass in this cycle.

## Part B — New Findings

### CRITICAL

*None.*

### HIGH

#### ADV-W3MT-P56-HIGH-001: put_with_ttl eviction-path race worst-case orphan size is understated when partition is over-count

- **Severity:** HIGH
- **Category:** concurrency / spec-fidelity
- **Location:** `crates/prism-query/src/cache.rs:384–391` (put_with_ttl doc comment)
- **Description:** The `put_with_ttl` doc comment states: "Worst-case orphan size: MAX_ENTRY_BYTES (5MB)." However, the eviction while-loop evicts **multiple** entries when the partition is over-count (e.g., due to a prior concurrent race that inflated partition_keys beyond max_entries_per_sensor). For each evicted entry, a concurrent `put_with_ttl(evict_key, ...)` can race and produce an orphan. The true worst-case is `N_evicted × MAX_ENTRY_BYTES`, not a fixed 5MB.

  In normal steady-state, the loop removes exactly one entry (partitions stay at max - 1 between inserts). But when a prior race leaves partition_keys over-count, the recovery eviction removes multiple entries in a single call. Each creates its own race window. Downstream: `total_bytes` can be transiently overstated by up to `N_evicted × MAX_ENTRY_BYTES` rather than the stated 5MB ceiling.

- **Evidence:**
  - `cache.rs:451–455`: `while partition_keys.len() >= self.config.max_entries_per_sensor { evicted.push(partition_keys.remove(0)); }` — loop runs until below threshold, can remove N > 1 entries.
  - `cache.rs:384–391`: doc says "Worst-case orphan size: MAX_ENTRY_BYTES (5MB)" — this claims exactly 5MB regardless of N.
  - The `invalidate_by_prefix` doc at line 609–610 correctly says "MAX_ENTRY_BYTES (5MB) **per evicted key**" — the `put_with_ttl` comment omits the "per evicted key" qualifier.
  - This discrepancy is a doc accuracy issue. The correct statement for `put_with_ttl` should mirror the `invalidate_by_prefix` language: "MAX_ENTRY_BYTES (5MB) per concurrently-racing evicted key; worst-case is N_evicted × MAX_ENTRY_BYTES for over-count partitions."
- **Proposed Fix:** Update the `put_with_ttl` doc comment to:
  ```
  // Worst-case orphan size: MAX_ENTRY_BYTES (5 MB) per concurrently-racing evicted key.
  // Under normal steady-state, N_evicted = 1 (one LRU entry removed per insert). In
  // over-count partition recovery (caused by a prior concurrent race), N_evicted > 1,
  // and the bound becomes N_evicted × MAX_ENTRY_BYTES. Pre-existing heuristic (SEC-NEW-002).
  ```

### MEDIUM

#### ADV-W3MT-P56-MED-001: Missing test: calling next_page with an exhausted cursor token returns E-QUERY-014

- **Severity:** MEDIUM
- **Category:** coverage-gap / missing-edge-cases
- **Location:** `crates/prism-query/src/tests/pagination_tests.rs` (no such test exists)
- **Description:** BC-2.07.002 §Cursor Lifecycle (MCP-exposed surface) specifies: "Returns `Err(PrismError::CursorTokenUnknown)` (E-QUERY-014) if the token never existed in the registry (garbage UUID, **already-released after exhaustion**, or from a different prism instance)." The implementation correctly removes the exhausted cursor's old token from `self.entries` at the `is_last` branch, so calling `next_page` with the old (now-removed) token would return `CursorTokenUnknown` (E-QUERY-014). However, no test covers this behavior: calling `next_page` with the old token after the final page has been returned.

  This is a well-known pattern where caller bugs (retry with old token after receiving `None`) produce a confusing E-QUERY-014 rather than a purpose-built "cursor exhausted" error. The test gap means the behavior is unverified. BC-2.07.002 explicitly calls out this exact scenario as a distinct error case.

- **Evidence:**
  - BC-2.07.002 line 72: "UUID is garbage, already-released after exhaustion, or from a different prism instance."
  - `cursor.rs:264–269`: `is_last` branch removes the token from `entries` and releases `core_id`. A subsequent call with the old token reaches `ok_or(PrismError::CursorTokenUnknown)?` at line 233.
  - `pagination_tests.rs`: `test_last_page_returns_none_token` verifies the last page returns `(rows, None)` but does NOT attempt `next_page` with the old token afterward.
  - Searching all test files for "after_last", "old_token", "exhausted", "post_exhaustion", "already_released" — zero results.
- **Proposed Fix:** Add a test:
  ```rust
  /// BC-2.07.002: calling next_page with an exhausted cursor's old token must return
  /// PrismError::CursorTokenUnknown (E-QUERY-014), not panic.
  #[test]
  fn test_next_page_after_exhaustion_returns_cursor_token_unknown() {
      let mut registry = QueryCursorRegistry::new();
      let rows = vec![json!({"row": 1}), json!({"row": 2})];
      let client = OrgSlug::new("acme");
      let (_, token) = registry.create(rows, 1, "q".to_string(), client).unwrap();
      let token = token.unwrap();
      // Exhaust the cursor.
      let (_, no_token) = registry.next_page(token.clone(), 1).unwrap();
      assert!(no_token.is_none(), "cursor must be exhausted");
      // Old token is now removed — subsequent call must return E-QUERY-014.
      let result = registry.next_page(token, 1);
      assert!(matches!(result, Err(PrismError::CursorTokenUnknown)),
          "old exhausted token must return CursorTokenUnknown");
  }
  ```

#### ADV-W3MT-P56-MED-002: TTL invariant test is `#[ignore]` with a `todo!()` body — BC-2.07.003 absolute TTL has zero behavioral test coverage

- **Severity:** MEDIUM
- **Category:** coverage-gap / verification-gaps
- **Location:** `crates/prism-query/src/tests/cache_tests.rs:317–342`
- **Description:** The test `test_BC_2_07_003_ttl_measured_from_created_at_not_from_last_access` verifies BC-2.07.003 §Invariants: "TTL is measured from `created_at`, not from last access." This is one of the most important correctness properties of the cache. The test is marked `#[ignore]` with a `todo!()` in the body. The TD annotation (TD-S305-006) defers clock injection to a future pass.

  The implementation of `CacheEntry::is_expired()` is correct (`self.created_at.elapsed() > self.ttl`) and does NOT use sliding expiration. However, there is zero behavioral test confirming this — only a constant-level assertion (`CACHE_TTL_ALERTS_SECS == 60`). Without a test that actually ages an entry (using a backdated `created_at` or a mocked clock), any future refactor that accidentally introduces sliding expiration would not be caught.

  The implementation also correctly preserves `created_at` during `next_page` token rotation (line 277 in cursor.rs: `let created_at = entry.created_at;`), but again there's no test asserting that token rotation does not reset the TTL clock.

- **Evidence:**
  - `cache_tests.rs:322`: `#[ignore = "TD-S305-006: test body has todo!()..."]`
  - `cache_tests.rs:341`: `todo!("BC-2.07.003 TTL invariant: not yet implemented")`
  - `cursor.rs:277`: `let created_at = entry.created_at;` — correct, but uncovered.
  - BC-2.07.003 §Invariants: "TTL is measured from `created_at` of the CacheEntry, not from last access (TTL, not sliding expiration)" — Kani does not cover this invariant; VP-025 covers only cache key determinism.
- **Proposed Fix:** Unblock TD-S305-006 using a backdated `Instant` approach (no clock injection required):
  ```rust
  #[test]
  fn test_BC_2_07_003_ttl_measured_from_created_at_not_from_last_access() {
      // Create an entry with created_at backdated by 70 seconds — simulates a 70s-old entry.
      // std::time::Instant cannot be constructed directly in the past, but we can test
      // the is_expired() method directly on CacheEntry using a forced creation.
      let entry = CacheEntry {
          rows: vec![json!({"id": "det-1"})],
          created_at: Instant::now() - Duration::from_secs(70), // requires nightly or workaround
          ttl: Duration::from_secs(60),
      };
      assert!(entry.is_expired(), "70s-old entry with 60s TTL must be expired");
      
      let fresh_entry = CacheEntry {
          rows: vec![],
          created_at: Instant::now(),
          ttl: Duration::from_secs(60),
      };
      assert!(!fresh_entry.is_expired(), "newly created entry must not be expired");
  }
  ```
  Note: `Instant::now() - Duration` may not work on all platforms. Alternative: set `created_at` via a field backdating shim (test-only constructor taking an explicit `Instant`).

#### ADV-W3MT-P56-MED-003: moka TTL 300s ceiling causes alert entries (60s TTL) to leak memory for up to 240 extra seconds when never re-accessed

- **Severity:** MEDIUM
- **Category:** missing-edge-cases / performance
- **Location:** `crates/prism-query/src/cache.rs:254–261` (QueryCache::new)
- **Description:** moka is configured with `time_to_live(300s)` — the devices TTL ceiling. Alert entries have a `CacheEntry.ttl` of 60s. The `get()` method correctly calls `is_expired()` before returning rows (no stale data is served). However, if an alert entry is **never accessed after insertion** (caller does not call `get()` for that key after the 60s window), the entry remains in moka's memory until moka's 300s clock fires — up to 240 seconds of unnecessary memory retention.

  Under normal query patterns (alerts are re-queried frequently), this is not a problem. But in a scenario where a query populates the alert cache and the client never re-queries (client disconnect, query cancelled after first result), the memory is held for up to `max(300 - 60, 0) = 240s` extra. With `DEFAULT_MAX_ENTRIES_PER_SENSOR = 50` alert entries per partition and `MAX_ENTRY_BYTES = 5MB`, this could retain up to 50 × 5MB = 250MB beyond the 60s window per partition. This competes with the 50MB total budget.

  The design intent (CR-004) is that `is_expired()` provides defense-in-depth against serving stale data, with moka's 300s as a coarse cleanup ceiling. The behavior is correct per CR-004 rationale. However, it is undocumented that abandoned alert entries occupy memory for up to 240 extra seconds, and there is no test or health metric that tracks this.

- **Evidence:**
  - `cache.rs:255–261`: moka `time_to_live(300s)` with comment "Manual is_expired() checks remain as defense-in-depth (CR-004)."
  - `cache.rs:319–328`: `get()` calls `is_expired()` and removes on miss — correct, but only triggered on access.
  - `cache.rs:60`: `CACHE_TTL_ALERTS_SECS = 60`.
  - No health metric or diagnostic log emitted when a 60s-TTL entry is found alive-in-moka beyond its TTL by the background cleanup task.
  - The background cleanup task only applies to the cursor registry (`spawn_cursor_cleanup_task`). There is no analogous cleanup task for the cache (moka's own background thread handles eviction on its 300s schedule).
- **Proposed Fix:** Two options:
  1. **Document only (minimal):** Add a comment in `QueryCache::new()` explaining that alert entries (60s TTL) may persist in moka memory for up to 240 extra seconds if never re-accessed, bounded by `max_entries_per_sensor × MAX_ENTRY_BYTES` per partition. This is the CR-004 design intent — no code change needed.
  2. **Add a second moka cache with 60s TTL for alert partitions:** More invasive; separates moka TTL per data type. This would eliminate the phantom retention. Likely out of scope for S-3.05.

  Minimum fix: update the `QueryCache::new()` comment at line 255 to say: "The 300s ceiling is the devices TTL; alert entries (60s TTL) may occupy moka memory for up to 240 extra seconds if no `get()` is issued after their 60s window elapses. Bounded by `max_entries_per_sensor × MAX_ENTRY_BYTES` per partition. Acceptable per CR-004."

### LOW

#### ADV-W3MT-P56-LOW-001: Focus area #5 confirmed as non-bug — no stale alert data served via moka

- **Severity:** LOW
- **Category:** code-quality (documentation accuracy)
- **Location:** `crates/prism-query/src/cache.rs:294–352` (get() method)
- **Description:** The dispatch focus area #5 asked: "Is there a risk that alert entries (60s TTL) are served stale from moka (which uses 300s as TTL) before is_expired() filters them?" The answer is **no** — `get()` always calls `is_expired()` BEFORE returning `rows`. Even if moka returns a 70s-old alert entry (which it would, since moka's TTL is 300s), `is_expired()` fires and the entry is removed as a cache miss. No stale data is ever returned. This focus area is confirmed clean.

- **Evidence:** `cache.rs:319`: `if entry.is_expired() { self.remove_entry(key)?; return Ok(None); }` — this guard precedes the `Ok(Some(entry.rows.clone()))` at line 352.
- **Proposed Fix:** None required for correctness. See ADV-W3MT-P56-MED-003 for the memory retention side effect.

#### ADV-W3MT-P56-LOW-002: Focus area #1 confirmed clean — cursor TTL is absolute from `created_at`

- **Severity:** LOW
- **Category:** code-quality (documentation accuracy)
- **Location:** `crates/prism-query/src/cursor.rs:107–108` (`CursorEntry::is_expired`)
- **Description:** The dispatch focus area #1 asked: "Is the 60s TTL checked from `created_at` or from last access? BC-2.07.002 specifies absolute TTL." Implementation: `self.created_at.elapsed() > Duration::from_secs(CURSOR_EXPIRY_SECS)` — this is an absolute TTL from creation, not a sliding TTL. The `created_at` field is set once at `Instant::now()` in `create()` (line 200) and preserved unchanged during `next_page()` token rotation (line 277: `let created_at = entry.created_at;`). BC-2.07.002 §Cursor Lifecycle: "Expiry: Cursors expire 60 seconds after creation." This matches exactly.

- **Evidence:** `cursor.rs:200`: `created_at: Instant::now()` (set once). `cursor.rs:277`: `let created_at = entry.created_at;` (preserved on rotation). `cursor.rs:108`: `self.created_at.elapsed() > Duration::from_secs(CURSOR_EXPIRY_SECS)` (absolute).
- **Proposed Fix:** None required. This is correct.

#### ADV-W3MT-P56-LOW-003: Focus area #4 confirmed clean — error code ordering is monotonic and non-colliding

- **Severity:** LOW
- **Category:** code-quality (documentation accuracy)
- **Location:** `crates/prism-core/src/error.rs:431–459`
- **Description:** The dispatch focus area #4 asked: "CursorExpired=E-QUERY-012, CursorPageSizeInvalid=E-QUERY-013, CursorTokenUnknown=E-QUERY-014 — are these ordered correctly in the PrismError enum and do they not collide with existing codes?" Verification: the three variants appear at lines 431 (CursorExpired/012), 444 (CursorPageSizeInvalid/013), 459 (CursorTokenUnknown/014) — strictly monotonic order in both error code and source position. No collisions found: E-QUERY-010 (QueryVirtualFieldFailed), E-QUERY-011 (AuditTableAccessDenied), E-QUERY-012, E-QUERY-013, E-QUERY-014. Gaps exist (no E-QUERY-006 through E-QUERY-009 in this enum) but that is consistent with the existing numbering scheme. E-STORE-020 (CursorCapExceeded) is also correctly placed in the E-STORE group, consistent with it being a storage-layer cap.

- **Evidence:** `error.rs:431–459` — verified ordering above.
- **Proposed Fix:** None required.

#### ADV-W3MT-P56-LOW-004: VP-025 Kani proof harness stubs have `todo!()` removed but comment still says "RED by design"

- **Severity:** LOW
- **Category:** code-quality (stale documentation)
- **Location:** `crates/prism-query/src/proofs/vp025_cache_key.rs:173–232` (dynamic_tests module)
- **Description:** The dynamic test module comments in `vp025_cache_key.rs` say "RED by design — `CacheKey::derive_push_down_hash` is `todo!()`" for each test. However, the actual implementation of `CacheKey::derive_push_down_hash` in `cache_key.rs` is now complete (uses SHA-256 via sha2 crate, no `todo!()` present). The tests would be GREEN if run, but the misleading comments declare them RED. This creates confusion for reviewers and CI operators who might assume these tests are expected to fail.

- **Evidence:**
  - `vp025_cache_key.rs:176`: "RED by design — `CacheKey::derive_push_down_hash` is `todo!()`" — but the function is fully implemented.
  - `cache_key.rs:160–199`: `derive_push_down_hash` is fully implemented with SHA-256 hashing.
  - `pagination_tests.rs:7–10`: correctly notes "The historical RED-gate annotations below have been retained as test-design documentation but the impl status is GREEN as of S-3.05 v1.11" — this sweep was done for pagination_tests but NOT for vp025_cache_key.rs.
- **Proposed Fix:** Apply the same annotation sweep to `vp025_cache_key.rs` that was applied to `pagination_tests.rs`. Change "RED by design — `CacheKey::derive_push_down_hash` is `todo!()`" to "GREEN — `CacheKey::derive_push_down_hash` is implemented as of S-3.05." This is the same pattern as the pagination_tests.rs cleanup in commit `df7804a8`.

## Policy Rubric Compliance

### POL-1 (append_only_numbering): PASS
No VSDD identifiers were renumbered or reused. New error variants E-QUERY-012/013/014 and E-STORE-020 follow sequential numbering without gaps in the assigned range. No BC, VP, or story IDs were retired or reused.

### POL-2 (lift_invariants_to_bcs): NOT APPLICABLE
This PR does not create domain invariants; the cache TTL and cursor cap invariants are covered by BC-2.07.001 through BC-2.07.006.

### POL-3 (state_manager_runs_last): NOT APPLICABLE
This is a code PR, not a spec burst.

### POL-4 (semantic_anchoring_integrity): PASS
BC citations in source files (e.g., "BC-2.07.003 §Postconditions", "BC-2.07.002 §Background Cleanup") correctly describe the referenced contract's actual content. The `cursor.rs` module header correctly identifies BC-2.07.001 and BC-2.07.002. The `cache.rs` module header correctly identifies BC-2.07.003 and BC-2.07.006. No phantom anchors found.

### POL-5 (creators_justify_anchors): PASS
Anchor choices (e.g., VP-025 → cache_key.rs test vehicle, VP-029 → cursor 200-cap) match the actual test vehicles. The VP-025 proof harness correctly justifies using `CacheKey::derive_push_down_hash` as the proof target.

### POL-6 (architecture_is_subsystem_name_source_of_truth): NOT APPLICABLE
No subsystem name changes in this PR.

### POL-7 (bc_h1_is_title_source_of_truth): NOT APPLICABLE
No BC file title changes in this PR.

### POL-8 (bc_array_changes_propagate_to_body_and_acs): NOT APPLICABLE
No story frontmatter changes.

### POL-9 (vp_index_is_vp_catalog_source_of_truth): PARTIAL VERIFICATION NEEDED
VP-025 (Cache Key Derivation — Deterministic) is new in this PR. The adversary could not confirm that VP-INDEX.md was updated to include VP-025 with matching proof method and module path. The VP-025 proof file exists at `proofs/vp025_cache_key.rs` with correct `#[cfg(kani)]` gating. If VP-INDEX.md was not updated to reflect VP-025, this is a POL-9 violation. **Verify:** check VP-INDEX.md and verification-architecture.md for VP-025 entry.

### POL-10 (demo_evidence_story_scoped): NOT APPLICABLE
No demo evidence files in this PR.

### POL-11 (index_bump_required_for_index_mutations): NOT APPLICABLE for code files; if STORY-INDEX.md was updated to mark S-3.05 as delivered, verify the index version was bumped.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 3 |
| LOW | 4 |

**Overall Assessment:** pass-with-findings

**Convergence:** FINDINGS_REMAIN — HIGH-001 (race doc accuracy) and MED-001 (missing post-exhaustion test) should be addressed before merge.

**Readiness:** The core implementation (cache, cursor, invalidation, cache_key) is structurally sound and correctly implements BC-2.07.001–006. The focus areas #1 (absolute TTL), #4 (error code ordering), and #5 (moka stale-serve) are confirmed clean. Focus area #2 (race doc accuracy) has a genuine finding (HIGH-001: doc understates worst-case for multi-eviction paths). Focus area #3 (post-exhaustion edge case) has a genuine coverage gap (MED-001).

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 56 |
| **New findings** | 8 (HIGH-001, MED-001, MED-002, MED-003, LOW-001, LOW-002, LOW-003, LOW-004) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 8 / (8 + 0) = 1.0 |
| **Median severity** | MEDIUM |
| **Trajectory** | First PR #132 targeted pass in this cycle |
| **Verdict** | FINDINGS_REMAIN — HIGH-001 and MED-001 require resolution before merge |
