---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-07T08:00:00
phase: maintenance
inputs:
  - commit 426bf86f (PR #132 FP25 fix-pass: HIGH-001, MED-001, CQ-001)
  - crates/prism-query/src/cache.rs (post-fix, 3 eviction-doc sites updated)
  - crates/prism-query/src/cursor.rs (exhaustion + TTL expiry paths)
  - crates/prism-query/src/tests/pagination_tests.rs (MED-001 test added)
  - crates/prism-query/src/proofs/vp025_cache_key.rs (12 annotations swept)
  - crates/prism-spec-engine/src/plugin/sandbox.rs (EpochTickerHandle rename)
input-hash: "[not-computed]"
traces_to: BC-2.07.001, BC-2.07.002, BC-2.07.003, BC-2.07.004, BC-2.07.005, BC-2.07.006
pass: 57
previous_review: pass-56.md
scope: Targeted fix-pass verification — commit 426bf86f only (HIGH-001/MED-001/CQ-001 closures)
---

# Adversarial Review: PR #132 FP25 Fix-Pass — S-3.05 (Pass 57)

## Scope Note

This is a targeted fix-pass verification review. The adversary reviewed commit 426bf86f
in full isolation (fresh context, no prior pass context loaded). The four explicit scope
checks requested were:

1. Does the MED-001 test actually exhaust the cursor correctly — 150 rows, page_size=100,
   verify the token used after final page is the token FROM the `create()` call, not from
   `next_page()`?
2. sandbox.rs EpochTickerHandle rename — does the Drop impl still work correctly after
   renaming `_stop` to `stop` and `_thread` to `thread`?
3. Are the 3 orphan-bound documentation sites in cache.rs updated consistently?
4. Any new behavioral divergence between the cursor exhaustion path and the TTL expiry
   path in cursor.rs?

## Finding ID Convention

Finding IDs use: `ADV-W3MT-P57-<SEV>-<SEQ>`

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-W3MT-P56-HIGH-001 | HIGH | RESOLVED | 3 eviction-doc sites in cache.rs updated to "K × MAX_ENTRY_BYTES (50 × 5MB = 250MB)". 2 correctly-bounded single-key sites (force_refresh, remove_entry) left unchanged — see analysis below. |
| ADV-W3MT-P56-MED-001 | MEDIUM | RESOLVED | test_med_001_exhausted_cursor_next_page_returns_token_unknown added. Token provenance verified (see finding analysis). |
| ADV-W3MT-P56-LOW-004 | LOW | RESOLVED | All 14 "RED by design" annotations removed from vp025_cache_key.rs dynamic_tests module. |

### HIGH-001 Resolution Detail

The fix updated exactly 3 eviction-loop functions (`put_with_ttl`, `invalidate_by_prefix`,
`invalidate_by_client`) with the precise K-multiplied bound. Two additional locations that
retain the original "MAX_ENTRY_BYTES (5MB)" phrasing (`force_refresh` line 583, `remove_entry`
line 794) were correctly left unchanged — both are single-key operations where K=1 by
construction:

- `remove_entry` evicts exactly one named key — no loop, no K > 1 possible
- `force_refresh` composes `remove_entry` + `put` on one key — the compound race is also
  bounded at 1 × MAX_ENTRY_BYTES

The 3 updated sites use consistent language: "Worst-case orphan size per evicted key:
MAX_ENTRY_BYTES (5MB). With concurrent racing puts on K evicted keys, total worst-case
orphan inflation is K × MAX_ENTRY_BYTES (bounded by max_entries_per_sensor × MAX_ENTRY_BYTES
= 50 × 5MB = 250MB)."

## Part B — New Findings

### CRITICAL

*None.*

### HIGH

*None.*

### MEDIUM

*None.*

### LOW

#### ADV-W3MT-P57-LOW-001: Commit message says "sweep 12 stale annotations" but 14 were removed

- **Severity:** LOW
- **Category:** code-quality (commit message inaccuracy)
- **Location:** commit 426bf86f commit message body, CQ-001 line
- **Description:** The commit message states "CQ-001 (vp025_cache_key.rs): sweep **12** stale
  'RED by design — todo!()' annotations from dynamic_tests mod". The diff shows 14 `RED by
  design` doc-comment lines removed (verified via `git show 426bf86f -- crates/prism-query/src/proofs/vp025_cache_key.rs | grep "^-.*RED by design" | wc -l` → 14). The prior state had 14 such lines (verified via the parent commit). This is a commit message inaccuracy — the work performed is correct and complete (all RED-by-design annotations removed), but the count stated in the message is wrong.

  This is informational only. The code is correct; the commit message count is off by 2.

- **Evidence:**
  - Commit message: "sweep 12 stale 'RED by design — todo!()' annotations"
  - `git show 426bf86f -- ...vp025_cache_key.rs | grep "^-.*RED by design" | wc -l` → 14
  - `git show 6ab56210^:...vp025_cache_key.rs | grep -c "RED by design"` → 14 (parent state)
  - `git show 426bf86f:...vp025_cache_key.rs | grep -c "RED by design"` → 0 (post-fix)
- **Proposed Fix:** No code change required. Informational. The sweep is complete and correct.

## Scope Check Results

### Scope Check 1: MED-001 test token provenance

**Verdict: PASS — token provenance is correct.**

The test creates a 150-row cursor with page_size=100. With exactly 150 rows:
- `create()` returns rows[0..100] (100 rows) + `token_A`
- `next_page(token_A, 100)` executes the `is_last=true` branch (end=150=total):
  removes `token_A` from the entries map, releases the core cap slot, returns
  (rows[100..149], **None**)

Since `next_page()` returns `None` (not a new_token) for the final page, the only
token ever in scope is `token_A` from `create()`. The test correctly uses `token`
(which IS `token_A`) for the exhausted-cursor call. The test does NOT use any
token returned from `next_page()` — there is none to use. Token provenance is correct.

Specifically: `let (last_page, no_token) = registry.next_page(token.clone(), 100)` — the
`.clone()` preserves `token` for the final call. `no_token.is_none()` confirms no rotation
occurred. `registry.next_page(token, 100)` then reuses `token_A`, which has been removed
from the entries map, correctly triggering `CursorTokenUnknown`.

### Scope Check 2: sandbox.rs Drop impl after EpochTickerHandle rename

**Verdict: PASS — Drop impl is correct after rename.**

Commit 6ab56210 renamed:
- `_stop: Arc<AtomicBool>` → `stop: Arc<AtomicBool>`
- `_thread: Option<JoinHandle<()>>` → `thread: Option<JoinHandle<()>>` (with `#[allow(dead_code)]`)

The Drop impl was updated in the same commit:
```
- self._stop.store(true, std::sync::atomic::Ordering::Relaxed);
+ self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
```

The rename is semantically equivalent: `self.stop.store(...)` correctly signals the
background ticker thread to stop. The `thread` field (previously `_thread`) still holds
the `JoinHandle` for RAII — the `#[allow(dead_code)]` annotation with explanatory comment
correctly documents that the field is intentionally unused (RAII lifetime management only).
Drop does not join the thread — the comment explains this is intentional ("background thread
will stop at next sleep").

The drop path is: `stop` field signals the AtomicBool to `true` → background thread observes
the flag on its next loop iteration and exits. This is the same mechanism as before the rename.
No behavioral change.

### Scope Check 3: 3 orphan-bound sites in cache.rs — consistency check

**Verdict: PASS — all 3 eviction-loop sites updated consistently; 2 single-key sites
correctly left unchanged.**

Updated (eviction-loop functions — K can be > 1):
1. `put_with_ttl` (~line 391-394): "K × MAX_ENTRY_BYTES (bounded by max_entries_per_sensor
   × MAX_ENTRY_BYTES = 50 × 5MB = 250MB)" ✓
2. `invalidate_by_prefix` (~line 612-615): same formula ✓
3. `invalidate_by_client` (~line 698-701): same formula ✓

Not updated (single-key operations — K=1 by construction):
- `force_refresh` (~line 583): "1 × MAX_ENTRY_BYTES (5MB)" — correct; force_refresh
  operates on exactly one key (one remove_entry + one put), single concurrent race possible
- `remove_entry` (~line 794): "MAX_ENTRY_BYTES (5MB)" — correct; single-key remove

The 3 updated sites use identical language. No inconsistency detected.

### Scope Check 4: Behavioral divergence between exhaustion path and TTL expiry path

**Verdict: PASS — no behavioral divergence; both paths are symmetric in cleanup.**

cursor.rs `next_page()` cleanup paths compared:

**Exhaustion path (is_last = true):**
```
self.entries.remove(&token);         // removes from map
self.core_registry.release(core_id); // releases cap slot
return Ok((page, None));             // signals completion with no continuation token
```

**TTL expiry path (entry.is_expired()):**
```
self.entries.remove(&token);         // removes from map
self.core_registry.release(core_id); // releases cap slot
return Err(PrismError::CursorExpired); // signals expiry
```

**Background eviction path (evict_expired):**
```
self.entries.remove(&token);         // removes from map
self.core_registry.release(entry.core_id); // releases cap slot
// no return value (background task)
```

All three paths symmetrically: (1) remove the entry from the map; (2) release the
prism-core cap slot. Post-cleanup, subsequent `next_page` calls with the same token
reach `ok_or(PrismError::CursorTokenUnknown)?` and correctly return E-QUERY-014 in
all cases.

One minor asymmetry is intentional and correct: exhaustion returns `Ok((page, None))`
(the last page IS delivered), TTL expiry returns `Err(PrismError::CursorExpired)` (no
data delivered — the entry is expired before being served). This asymmetry is specified by
BC-2.07.002 and is not a divergence bug.

No behavioral divergence found between the three cleanup paths.

## Policy Rubric Compliance (Fix-Pass Scope)

### POL-1 (append_only_numbering): PASS
No VSDD identifiers renumbered or reused in this fix commit.

### POL-3 (state_manager_runs_last): NOT APPLICABLE
Code-only fix commit, no spec bursts.

### POL-4 (semantic_anchoring_integrity): PASS
MED-001 test cites "BC-2.07.002 §Cursor Lifecycle Advancement" — verified that the cited
section is accurately described (calling next_page on an exhausted token returns
CursorTokenUnknown per BC-2.07.002). No phantom anchors.

### POL-8 (bc_array_changes_propagate_to_body_and_acs): NOT APPLICABLE
No story frontmatter changes.

### POL-9 (vp_index_is_vp_catalog_source_of_truth): NOT APPLICABLE
Fix commit does not create new VPs.

### POL-10 (demo_evidence_story_scoped): NOT APPLICABLE
No demo evidence files.

### POL-11 (index_bump_required_for_index_mutations): NOT APPLICABLE
Fix commit does not mutate index files.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 1 |

**Overall Assessment:** pass-with-findings (informational only — LOW-001 is a commit
message count discrepancy, not a code defect)

**Convergence:** HIGH-001, MED-001, and LOW-004 from pass-56 are all RESOLVED. The
remaining finding (LOW-001) is informational and does not require a code fix. The PR
implementation is correct.

**Readiness:** All 4 explicit scope checks PASS. The FP25 fix-pass correctly addressed
the three PR review findings. No new behavioral issues introduced.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 57 |
| **New findings** | 1 (LOW-001 — commit message count inaccuracy) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1 / (1 + 0) = 1.0 (only 1 finding, informational) |
| **Median severity** | LOW |
| **Trajectory** | 8 (pass-56) → 1 (pass-57) — all open findings from pass-56 resolved |
| **Verdict** | FINDINGS_REMAIN (LOW-001 is informational; code is correct) — recommend merge |
