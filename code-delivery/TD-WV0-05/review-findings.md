---
pr: 28
story: TD-WV0-05
branch: fix/TD-WV0-05-dtu-route-mounts
reviewer: pr-manager (claude-sonnet-4-6)
---

# Review Findings — TD-WV0-05

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 3 | 1 | 1 | 2 (cosmetic, deferred) |
| 2 | 0 | 0 | 0 | 0 → APPROVE |

## Cycle 1 Findings

### F1 — configure() counter-reset side effect (BLOCKING, FIXED)

**File:** `crates/prism-dtu-threatintel/src/routes/lookup.rs` — rate_limit_after branch
**Category:** code-quality / semantic clarity
**Problem:** The implementer added `state.request_counter.store(0, SeqCst)` as a side effect
of setting `rate_limit_after` in `configure()`. This conflates policy change with state clear.
The real issue was that `dtu_reset_mount.rs` Step 4 (fixture verification lookup) increments
the counter after the Step 3 reset, and Step 5's rate-limit probe then started from counter=1
not counter=0.
**Resolution:** Removed the counter-reset from `configure()`. Refactored `dtu_reset_mount.rs`
Step 5 to call `POST /dtu/reset` a second time (establishing an explicit clean baseline) before
the rate-limit probe. Verified that `ThreatIntelState::reset()` already zeros the counter.
**Commit:** `23a772f8`

### F2 — get_health vs dtu_health naming inconsistency (COSMETIC, DEFERRED)

**File:** `crates/prism-dtu-nvd/src/routes/dtu.rs:26` vs `crates/prism-dtu-threatintel/src/routes/lookup.rs`
**Category:** code-quality / naming consistency
**Problem:** NvdClone names the health handler `get_health` (HTTP-verb prefix style).
ThreatIntelClone uses `dtu_health` (matching the L1 crowdstrike reference).
**Resolution:** Deferred. Will align in a follow-up L2 Clone Template consistency sweep.

### F3 — DTU introspection routes in lookup.rs instead of routes/dtu.rs (COSMETIC, DEFERRED)

**File:** `crates/prism-dtu-threatintel/src/routes/lookup.rs`
**Category:** code-quality / module structure
**Problem:** NvdClone separates DTU introspection routes into `routes/dtu.rs`. ThreatIntelClone
has only `routes/lookup.rs` and the new handlers live alongside business logic.
**Resolution:** Deferred. Suggests creating `crates/prism-dtu-threatintel/src/routes/dtu.rs`
in a follow-up sweep. Track as new tech-debt item.

## CI Fix (post-review)

**Format check failure:** `cargo fmt --check` on CI required the long import line in
`crates/prism-dtu-threatintel/src/clone.rs` to be split across multiple lines.
Fixed in commit `ddc6a827`.

## Final Verdict

APPROVE — all blocking findings resolved. Two cosmetic findings deferred.
