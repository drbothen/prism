## Summary

Closes **TD-VSDD-058**: two latent bugs in `.github/workflows/fuzz-nightly.yml` that mirror the `fuzz-smoke` fixes that landed in PR #127.

Without this fix, **tonight's scheduled 02:00 UTC nightly fuzz run would fail at protoc-NotFound** — the same failure mode as CI run 25444145941 (fuzz-smoke, pre-PR-#127). This maintenance PR prevents that outage.

**Discovery context:** PR #127 adversary pass-14 lens-7 sibling-layer check (2026-05-06). Non-blocking for PR #127 merge but flagged for immediate post-merge maintenance. Filed as TD-VSDD-058 at that time.

---

## Changes (single file, single commit `3b2a1a29`)

### Bug 1: Missing protoc install step

`prism-ocsf` is in `vp021_parse_fuzz`'s transitive build graph. Its `build.rs` invokes `prost-build`, which requires the `protoc` binary at build time. Without the install step, the nightly build fails immediately with `protoc not found`.

**Fix:** Added `arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b # v3.0.0` step after `rust-cache` and before `cargo-fuzz` install, mirroring all 6 sibling jobs in `ci.yml` (including the `fuzz-smoke-vp021` job fixed in PR #127 commit `230aa700`) and the SHA-pinned action already in use.

**Placement:** After `rust-cache`, before `cargo-fuzz` install — consistent with the ordering convention established across all sibling jobs.

### Bug 2: Insufficient timeout-minutes ceiling

PR #127 commit `30f6fc07` added `opt-level=3` dev-profile entries for the sha2/aes-gcm/argon2/blake2 crypto stack in `.cargo/config.toml`. This extends cold sanitizer-instrumented build time from ~5 min to ~10-15 min.

**Budget under prior 45-min ceiling:**
- Cold build (sanitizer-instrumented, opt-level=3 crypto): ~10-15 min
- Fuzz run (`-max_total_time=1800`): 30 min
- Checkout + toolchain + cache restore + cleanup: ~3-5 min
- **Total:** ~43-50 min — leaves only minutes of margin, intermittent failures expected

**Fix:** Bumped `timeout-minutes: 45` → `timeout-minutes: 60`, providing ~14 min headroom.

---

## Sibling-Layer Protocol

This fix mirrors the sibling-layer check methodology from PR #127 pass-14. When a class of bug is found and fixed in one CI job (`fuzz-smoke`), all sibling jobs are inspected for the same root cause. Both issues here are direct siblings of the `fuzz-smoke` fixes in PR #127.

---

## Traceability

```
TD-VSDD-058
  └─ fuzz-smoke root cause (PR #127 commit 230aa700) → sibling: fuzz-nightly protoc-NotFound
  └─ crypto opt-level=3 build-time increase (PR #127 commit 30f6fc07) → sibling: fuzz-nightly timeout margin
```

---

## Test Evidence

This is a CI configuration fix. Functional correctness is verified by:
1. The existing fuzz target `vp021_parse_fuzz` (unchanged)
2. CI workflow validation — ci.yml jobs (fmt, clippy, tests) pass on this branch
3. fuzz-nightly itself only runs on schedule/`workflow_dispatch`, not on PR push — expected and correct per workflow design

---

## Security Review

N/A — CI YAML change only. No product code, no credentials, no secrets handling. Action SHAs are pinned and match all sibling jobs:
- `arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b # v3.0.0` — same pin used in all 6 sibling ci.yml jobs

---

## Risk Assessment

- **Blast radius:** Zero product code. Single workflow file, CI-only change.
- **Revert risk:** Trivially revertible. `git revert 3b2a1a29`.
- **Performance impact:** None (CI runtime only; expected nightly run time unchanged, ceiling raised).
- **If not merged:** Tonight's 02:00 UTC nightly fuzz run fails at build phase (protoc-NotFound).

---

## Pre-Merge Checklist

- [x] Single file change (`.github/workflows/fuzz-nightly.yml`)
- [x] protoc install step SHA matches all 6 sibling ci.yml jobs
- [x] Step placement: after rust-cache, before cargo-fuzz install
- [x] Timeout bump documented with rationale (pass-14 lens-7 discovery context)
- [x] No demo evidence needed (CI config, not a user-facing story)
- [x] No AI attribution (per project policy)
- [x] CI gates: ci.yml jobs pass on PR branch
- [x] TD-VSDD-058 closure: both stated bugs addressed
