# PR Manifest: W3-FIX-SEC-005

**Status:** MERGED
**PR:** #125
**Title:** fix(W3-FIX-SEC-005): admin-token uniformity across 5 DTU clones (P7 CR-021/022)
**URL:** https://github.com/drbothen/prism/pull/125
**Merge SHA:** ba3b10c73aac19ffad21928187e1f9c79992d4f9
**Merged At:** 2026-05-02T19:38:05Z
**Merge Strategy:** squash
**Base Branch:** develop
**Head Branch:** fix/W3-FIX-SEC-005-dtu-admin-token-uniformity (deleted post-merge)

---

## Commits Squashed

| SHA | Message |
|-----|---------|
| f30f9789 | test(W3-FIX-SEC-005): red gate — 9 admin-token regression tests across 5 DTU clones |
| 6b93086c | chore(W3-FIX-SEC-005): update Cargo.lock for subtle = "2" in 5 DTU crates |
| 3f371eb4 | fix(W3-FIX-SEC-005): admin-token uniformity across 5 DTU clones — ct_eq + post_reset gate (CR-021/022) |
| 2b62f313 | chore(W3-FIX-SEC-005): demo evidence per POL-010 AC-001..006 |
| fc467937 | fix(W3-FIX-SEC-005): apply ct_eq to ThreatIntel configure handler in lookup.rs (R1-001) |

---

## Findings Closed

| ID | Severity | CWE | Description | Status |
|----|----------|-----|-------------|--------|
| CR-021 | MEDIUM | CWE-863 | post_reset NO admin-token gate — 5 DTU clones | CLOSED |
| CR-022 | LOW | CWE-208 | Non-constant-time `!=` in 5 DTUs' post_configure | CLOSED |

---

## CI Results at Merge

| Check | Result | Duration |
|-------|--------|----------|
| Clippy (AD-008) | pass | 7m26s |
| Format check | pass | 34s |
| Semver compatibility | pass | 2m43s |
| Cargo audit (RustSec) | pass | 40s |
| Cargo deny (license + advisory) | pass | 57s |
| Verify workflow structure | pass | 16s |
| Workspace crate layout (ADR-012) | pass | 13s (×2) |
| Test (aarch64-apple-darwin) | pass | 17m23s |
| Test (x86_64-apple-darwin) | pass | 26m57s |
| Test (x86_64-pc-windows-msvc) | pass | 32m27s |
| Test (x86_64-unknown-linux-musl) | pass | 23m54s |
| Test (x86_64-unknown-linux-gnu) | pass | ~35m |
| Test (no-default-features) | pass | ~30m |

**Total: 14/14 checks passed, 0 failures**

---

## Review Convergence

| Cycle | Blocking Findings | Resolution |
|-------|------------------|------------|
| 1 | 1 (R1-001: ThreatIntel lookup.rs ct_eq missed) | Fixed at fc467937 |
| 2 | 0 | APPROVE |

---

## Worktree

- Path: `/Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-005`
- Status: removed post-merge

---

## Artifacts

- `/Users/jmagady/Dev/prism/.factory/code-delivery/W3-FIX-SEC-005/pr-description.md`
- `/Users/jmagady/Dev/prism/.factory/code-delivery/W3-FIX-SEC-005/security-findings.md`
- `/Users/jmagady/Dev/prism/.factory/code-delivery/W3-FIX-SEC-005/review-findings.md`
- `/Users/jmagady/Dev/prism/.factory/code-delivery/W3-FIX-SEC-005/pr-manifest.md` (this file)
