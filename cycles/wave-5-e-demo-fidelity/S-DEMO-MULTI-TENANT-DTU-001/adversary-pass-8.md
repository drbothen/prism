---
document_type: adversary-pass-report
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 8
protocol: BC-5.39.001 3-CLEAN-strict
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "YES"
streak_before: 1
streak_after: 0
findings_total: 1
findings_by_severity:
  CRIT: 0
  HIGH: 1
  MED: 0
  LOW: 0
  OBS: 0
all_findings_status: CLOSED
classification: HIGH-SCOPE-DRIFT
date: 2026-06-13
d_anchor: D-1152
spec_versions_at_pass:
  story: v1.10
  bc: v1.7
  bc_index: "6.50"
  story_index: "v2.378"
---

# Adversary Pass 8 — S-DEMO-MULTI-TENANT-DTU-001

## Verdict

- **CLEAN (strict):** NO — 1 HIGH finding (F-P8-HIGH-001). Streak RESET 1/3 → 0/3.
- **CLEAN (PR-merge):** YES — zero CRIT/HIGH/MED findings remain open after fix.

## Finding

### F-P8-HIGH-001 [HIGH — CLOSED] Code-scope expansion into prism-dtu-armis/src not recorded in spec sibling surfaces

**Classification:** Partial-fix regression (S-7.01) — code scope grew in fix-burst for F-P1-HIGH-001 but spec sibling surfaces were not swept.

**Finding:**

The Pass-1 fix-burst for F-P1-HIGH-001 (AC-006/INV-ISOLATION-001 isolation-counter fix) expanded PRODUCTION code into `prism-dtu-armis/src/`:

- `prism-dtu-armis/src/state.rs` — added `request_counter: AtomicU64` field
- `prism-dtu-armis/src/clone.rs` — added `count_request_middleware`, `request_count()` method, `GET /dtu/request-count` route
- `prism-dtu-armis/src/routes/dtu.rs` — added `get_request_count` handler

However, the story's `crates_touched` frontmatter array, Architecture Mapping table, File Structure Reference table, and Token Budget table, and the BC-2.06.017 `crates:` frontmatter array **never recorded prism-dtu-armis as an in-scope crate**. The story actively asserted "src/ never names clone types" without distinguishing between the demo-server/harness `src/` (correct: no clone imports there) and `prism-dtu-armis/src/` (which now contains new production code from this story's fix-burst).

**Is the counter itself a perimeter violation?**

No. The `/dtu/request-count` endpoint is on the `/dtu/*` control plane (not `/api/*` application plane). ArmisClone instrumenting itself for AC-006 isolation proof is entirely within the clone crate's own domain. This is NOT an INV-PERIMETER-001 breach. The counter is legitimate and load-bearing (server-side delta assertion in isolation tests; client-side counters were rejected as paper-fix).

**The defect is spec↔code scope sync, not behavioral correctness.**

A fresh-context reviewer of this story would conclude:
- `crates_touched = [demo-server, harness]` — prism-dtu-armis is NOT in scope
- Architecture Mapping: no row for prism-dtu-armis
- File Structure Reference: no rows for armis/src/*.rs files
- Yet the actual delivery touches 3 armis/src/ files with production-scope changes

This is a Partial-fix Regression (S-7.01): the original finding was fixed correctly in code, but the fix expanded into a crate whose new involvement was never documented in the spec sibling surfaces.

**Fix Applied (D-1152, same burst):**

- **story-writer:** S-DEMO-MULTI-TENANT-DTU-001 v1.9 → v1.10
  - `crates_touched` frontmatter: `prism-dtu-armis` added
  - Architecture Mapping table: new row for prism-dtu-armis (state.rs + clone.rs + routes/dtu.rs)
  - File Structure Reference: new rows for `prism-dtu-armis/src/state.rs`, `prism-dtu-armis/src/clone.rs`, `prism-dtu-armis/src/routes/dtu.rs`
  - AC-006 verification note: clarified "demo-server/harness src/ never names clone types" (only for those crates' src); added note that prism-dtu-armis/src/ is in-scope for the isolation counter
  - Token Budget: updated crate list to include prism-dtu-armis

- **product-owner:** BC-2.06.017 v1.6 → v1.7
  - `crates:` frontmatter array: `prism-dtu-armis` added
  - Postcondition 4 verification-mechanism note: clarified server-side counter lives in ArmisClone (prism-dtu-armis/src/) and is part of the AC-006 isolation proof mechanism; not a perimeter violation (control-plane `/dtu/*` route, clone-own scope)

**No code change. Spec↔code scope sync only.**

**Status:** CLOSED

## Other Axes Checked — All Clean

| Axis | Result |
|------|--------|
| SAP-1 (tracing emission catalog completeness) | CLEAN — no new event_type emissions since Pass-1 |
| Gate 60 (EXPECTED=60, INV-PERIMETER-001) | CLEAN — perimeter-violation gate counts confirmed; DTU crate additions do not affect prism-query perimeter |
| unwrap/expect in story src | CLEAN — code unchanged since Pass-1 commit 9b4f4154 |
| INV-PERIMETER-001 | INTACT — /dtu/request-count is control-plane; ArmisClone is its own crate; no prism-spec-engine/prism-query/prism-sensors import in prism-dtu-armis/src/ |
| Isolation proof load-bearing (not paper-fix) | VERIFIED — AtomicU64 server-side counter + delta assertion (delta_b==1; >1 means cross-tenant leak); not a client-side proxy |
| SAP-2 (DTU↔TOML schema parity) | CLEAN — /dtu/request-count is control-plane only, not in sensor TOML [[tables]] |
| BC-INDEX/STORY-INDEX title+subsystem+version sync | VERIFIED after fix |

## Convergence State After Pass 8

- Streak: 0/3 (RESET from 1/3)
- Next: Pass 9 — need passes 9 + 10 + 11 all CLEAN(strict) for 3/3 convergence
- Code HEAD: unchanged at 9b4f4154 (code stable since Pass-1)
- Story: v1.10 (D-1152)
- BC-2.06.017: v1.7 (D-1152)
- BC-INDEX: v6.50 (D-1152)
- STORY-INDEX: v2.378 (D-1152)
