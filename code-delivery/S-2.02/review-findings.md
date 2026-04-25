---
document_type: review-findings
story_id: S-2.02
pr_number: 52
reviewer: pr-review-triage
pr_manager: pr-manager
total_cycles: 2
final_verdict: APPROVE
merge_commit: 9de6b3d8a6b44231b42e723210a710ad40a77855
merged_at: "2026-04-25T23:13:38Z"
---

# Review Findings — S-2.02 PR #52

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 5 | 1 | 5 | 0 |
| 2 | 0 | 0 | 0 | 0 -> APPROVE |

## Cycle 1 Findings

### R-001 — BLOCKING (FIXED at 9499ea86)
**Location:** `watchdog.rs` — multiple doc comments
**Issue:** Doc comment on `ResourceWatchdog.budget_bytes` struct field said `"512 MiB = 512 * 1024 * 1024"` but actual production value is `512 * 1_000_000 = 512,000,000` (SI-MB). Additional doc comments in `WatchdogStatus`, `new()`, and `with_probe()` also said "512 MiB".
**Fix:** Corrected all doc comments to say "512 MB SI = 512,000,000 bytes". Added inline ADR reference noting test probe constants use MiB-based math which still lands in correct level buckets. Updated evidence-report.md with actual probe byte values.

### R-002 — NON-BLOCKING (ACCEPTED as deferred)
**Location:** `audit_buffer.rs:135`
**Issue:** `retry_forward_entry` stub uses `#[allow(dead_code)]` with hardcoded `Err("not yet wired")`. Risk of being wired incorrectly in next dispatch.
**Disposition:** Accepted. Function is `pub(crate)`, explicitly annotated, and next-dispatch implementer will see the hardcoded error. Low risk.

### R-003 — NON-BLOCKING (ACCEPTED as deferred)
**Location:** `audit_buffer.rs:97`
**Issue:** `check_and_purge_overflow` materializes full 100K-entry dataset (keys + values) for count check. Could use `scan_range` with keys-only optimization.
**Disposition:** Accepted. Called every 60s per spec; materializes ~15 MB; optimization out of scope for this story.

### R-005 — COSMETIC (FIXED at 9499ea86, subsumed by R-001)
**Location:** `watchdog.rs:137` — `WatchdogStatus.budget_bytes` doc comment said "512 MiB"
**Fix:** Corrected to "512 MB SI = 512,000,000 bytes"

### R-006 — COSMETIC (FIXED at 9499ea86)
**Location:** `docs/demo-evidence/S-2.02/evidence-report.md`
**Issue:** StaticProbe section claimed probe values of 358/486/492 MB; actual constants are 375.8/510.0/519.0 MB.
**Fix:** Updated with a table showing actual byte values, % of SI budget, and watchdog level.

## Cycle 2 Findings

No new findings. R-001 fix verified complete and accurate. APPROVE issued.

## Security Review

**Verdict:** PASS — No Critical, High, or Medium findings.
- Bincode serialization: type-safe
- RocksDB key construction: not an injection surface
- Capability enforcement: correctly deferred to MCP tool layer per architecture
- `Ordering::Relaxed` on QueryId counter: correct
- `retry_forward_entry` stub: no security impact
