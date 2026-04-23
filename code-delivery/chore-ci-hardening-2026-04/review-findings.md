# Review Findings — chore/ci-hardening-2026-04

## Summary

PR #5: `ci: modernize action versions and replace retired macOS runner`

- Base: develop @ ad95cb5
- Head at merge: 2a3903d (squash commit: 88d46bf0)
- Review cycles: 1 (APPROVE in prior dispatch)
- CI fix iterations: 3 (all in-band, no new review cycle required)

---

## Review Cycle Convergence

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 0        | 0        | 0     | 0 → APPROVE |

No review findings. pr-reviewer approved in cycle 1 (prior dispatch).

---

## CI Fix Log (Step 6)

These failures were surfaced by GitHub Actions CI — not review findings.
All were fixed on the same branch without requiring a new review cycle.

### Fix 1 — commit eb8b90f (prior dispatch)
- **Job:** Cargo deny (license + advisory)
- **Error:** `Unicode-3.0` not in allowlist — transitive dep `icu` v2 series
- **Fix:** Added `Unicode-3.0` to `deny.toml` allow list

### Fix 2 — commit 8ca7ff2
- **Job:** Cargo deny (license + advisory)
- **Error 1:** `prism-dtu-common` has no `license` field in Cargo.toml → flagged as unlicensed
- **Error 2:** `webpki-roots v1.0.7` uses `CDLA-Permissive-2.0` (Mozilla CA root data) → not in allowlist
- **Fix 1:** Added `license = "MIT"` to `crates/prism-dtu-common/Cargo.toml`
- **Fix 2:** Added `CDLA-Permissive-2.0` to `deny.toml` allow list

### Fix 3 — commit 9a13f6c
- **Job:** Cargo deny (license + advisory)
- **Error:** `error: no such command: deny` — `cargo-deny-action@v2` runs checks internally inside Docker; the redundant `run: cargo deny check` step attempted to call cargo-deny on the host runner where it is not installed
- **Fix:** Removed redundant `run` step; passed `command: check` and `arguments: --all-features` as action inputs per EmbarkStudios/cargo-deny-action@v2 interface

### Fix 4 — commit 2a3903d
- **Job:** Semver compatibility
- **Error:** `prism-dtu-common not found in registry (crates.io)` — `cargo-semver-checks check-release` requires a crates.io baseline by default; internal workspace crates are not published
- **Fix:** Changed to `--baseline-rev origin/develop` (git-based diff); added `fetch-depth: 0` to checkout; added `publish = false` to `prism-dtu-common/Cargo.toml`

---

## Final CI Result (run 24761379586)

| Job | Status |
|-----|--------|
| Format check | PASS |
| Clippy (AD-008) | PASS |
| Test (aarch64-apple-darwin) | PASS |
| Test (x86_64-apple-darwin) | PASS |
| Test (x86_64-unknown-linux-gnu) | PASS |
| Test (x86_64-unknown-linux-musl) | PASS |
| Test (x86_64-pc-windows-msvc) | PASS |
| Cargo deny (license + advisory) | PASS |
| Cargo audit (RustSec) | PASS |
| Semver compatibility | PASS |

All 10 jobs: PASS.

---

## Merge

- Squash commit: `88d46bf0a4f7496975ce894e3cd3394d038508e9`
- develop HEAD after merge: `88d46bf0`
- Remote branch `chore/ci-hardening-2026-04`: DELETED
