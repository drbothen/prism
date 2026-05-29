---
document_type: cascade-summary
story_id: S-5.01-FOLLOWUP-MCP-BOOT
pr_number: 163
merged_at: "2026-05-29T16:44:42Z"
merged_via_sha: "e898c3c9"
local_passes: 19
local_fix_bursts: 16
pr_level_passes_per_reviewer: 16
pr_level_fix_bursts: 10
ci_status: "40/40 GREEN"
---

# S-5.01-FOLLOWUP-MCP-BOOT Cascade Summary

## Story

prism-mcp: PrismServer — rmcp 1.7, Tool Router, Injection Defense Wiring

## Delivery Timeline

- **Started:** 2026-05-28 (worktree created, stubs written)
- **LOCAL convergence:** Pass 19 (3-CLEAN: passes 17/18/19) per BC-5.39.001 strict
- **PR opened:** HEAD ee36f589
- **PR-LEVEL security convergence:** Pass 15 (3-CLEAN: passes 13/14/15)
- **PR-LEVEL pr-reviewer convergence:** Pass 16 (3-CLEAN: passes 14/15/16)
- **PR merged:** 2026-05-29T16:44:42Z at develop@e898c3c9

## LOCAL Cascade (19 passes, 16 fix-bursts)

**Trajectory:** 2C+4H → 6C+8H+6M → 5C+6H+6M → 2C+3H → 2C+4H → 3H+2M → 3H+2M → CLEAN (streak 1/3) → 1H+1M+1L → 3H+2M → 2H+3M → 2C+2H+3M → 2C+4H → 1C+3H → 1H+1M → 2M → CLEAN → CLEAN → CLEAN

Convergence: 3-CLEAN at passes 17/18/19 per BC-5.39.001 strict (zero findings any severity).

**Key finding classes closed in LOCAL cascade:**
- Initial: PrismServer struct missing subsystem wiring (Arc-DI plumbing gaps)
- Pass 2-3: Tool handler injection defense not wired at every entry point
- Pass 4-6: ResponseEnvelope field schema gaps; MCP error code mapping incomplete
- Pass 7-8: CLEAN streak broken — shutdown handler race condition identified
- Pass 9-11: Concurrent request handling path gaps; validate_text_field sibling-sweep misses
- Pass 12-14: add_sensor_spec path validation gap (pre-cursor to SEC-001); structured error taxonomy alignment
- Pass 15-16: Sibling-sweep cleanups across validate_* helpers
- Pass 17-19: CLEAN (strict) — convergence

## PR-LEVEL Cascade (16 passes per reviewer, 10 fix-bursts)

### Security Reviewer

- **Passes:** 16 total
- **Fix-bursts:** 10
- **Convergence:** 3-CLEAN at passes 13/14/15

**Notable security findings:**
- **SEC-001 (Pass 12) — CRITICAL CWE-22 Path Traversal in add_sensor_spec:** `add_sensor_spec` accepted arbitrary filesystem paths without sanitization. An MCP client could pass `../../etc/passwd` or equivalent path escape sequences. Fixed by adding canonical path validation + `.starts_with(sensor_specs_dir)` containment check. Security caught what LOCAL cascade missed.
- **SEC-002 (Pass 8) — Real production shutdown race:** `serve_with_transport_and_shutdown_inner` natural_close_fut arm was silently masking `JoinError::Panic` from the server task — a panic in the server task would cause a clean exit rather than propagating the error. Caught during CI investigation of a flaky test failure. Fix: distinguish `JoinError::Cancelled` from `JoinError::Panic` and re-panic appropriately.

### PR Reviewer

- **Passes:** 16 total
- **Fix-bursts:** 10 (shared with security fix-bursts)
- **Convergence:** 3-CLEAN at passes 14/15/16

**Notable pr-reviewer findings:**
- **Pass 3 (paper-fix detection, TD-VSDD-059):** Implementer had sentinel-bypassed a finding by doc-commenting rather than fixing the structural issue. pr-reviewer caught the paper-fix pattern per TD-VSDD-059.
- **Pass 8:** Windows CI failure — hardcoded `/tmp/` paths in test fixtures. Fixed by converting to `tempfile::tempdir()` per SID-1 §5 workspace convention.
- **Pass 11:** Sibling-sweep miss across `validate_text_field` extension — four additional callsites had not received the same sanitization applied to the primary fix site.

## CI Results

- **Platform matrix:** Linux x86_64, macOS aarch64, macOS x86_64, Windows x86_64-pc-windows-msvc
- **Total jobs:** 40
- **Result:** 40/40 GREEN at merge
- **Notable:** Windows MSVC was the last platform to go green; hardcoded `/tmp/` path (pass 8 pr-reviewer finding) caused Windows failures until fix was applied.
