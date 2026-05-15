# Review Findings — S-PLUGIN-PREREQ-D

PR: #149 (https://github.com/drbothen/prism/pull/149)
Branch: feature/S-PLUGIN-PREREQ-D

## Convergence Tracking

| Cycle | Reviewer | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|----------|-------|-----------|---------|
| 1 | security-reviewer | 1 CRITICAL | 1 | 1 | 0 | REQUEST_CHANGES |
| 2 | pr-reviewer | 3 | 2 | 3 | 0 | REQUEST_CHANGES |
| 3 | pr-reviewer | 1 | 1 | 1 | 0 | REQUEST_CHANGES (semver-checks fail) |
| 4 | pr-reviewer (e57d0929) | 0 | 0 | — | 0 | APPROVE |

## Cycle 1 — Security Review

- **Finding:** `allowed_urls: None` short-circuit in `host_http_request` allowed all URLs if the allowlist was unset (CRITICAL — ADR-023 §C4 F-CRIT-NEW-002)
- **Route:** implementer
- **Resolution:** Fixed in-scope — `None` case now blocks and audits (403 + WARN). Verified in `test_host_http_request_blocks_when_no_allowlist`.

## Cycle 2 — PR Reviewer Pass 1

- **Finding 1 (BLOCKING):** `PluginRuntime::new()` not Arc-DI wired — constructed without `reqwest::Client` per ADR-022
  - Resolution: `PluginRuntime::new(http_client: reqwest::Client)` signature added; boot wiring passes production client
- **Finding 2 (BLOCKING):** `AuthToken` not zeroized on drop — TD-S-PLUGIN-PREREQ-B-002 still open
  - Resolution: `Zeroizing<String>` wrapper applied; drop test confirms zeroize behavior
- **Finding 3 (ADVISORY → fixed):** BC-2.16.002 Structured Event Catalog missing 3 new event_type rows
  - Resolution: 3 rows added to BC-2.16.002 v1.17

## Cycle 3 — PR Reviewer Pass 2 (semver issue surfaced)

- **Finding 1 (BLOCKING):** `cargo semver-checks --baseline-rev develop` reports 3 breaking changes in `prism-spec-engine` without a version bump — CI gate failing
  - Route: implementer (Option A: bump version)
  - Resolution: `prism-spec-engine 0.7.0 → 0.8.0` in commit `e57d0929`; downstream pins updated; semver-checks 252/252 pass

## Cycle 4 — PR Reviewer Final Pass (on e57d0929)

- Scope: cumulative diff `develop..e57d0929` (56 files + 5 manifest/lockfile files)
- Incremental diff `45ebc198..e57d0929`: 5 files (Cargo.toml x3 + Cargo.lock x2) — version strings only

**Breaking change verification:**
| Change | Justification | Verdict |
|--------|--------------|---------|
| `LoadedPlugin.allowed_urls` new pub field | New field on public struct — external struct-literal construction breaks | ✓ Bump correct |
| `PluginRuntime::new()` 0→1 arg | Signature change — all callers break | ✓ Bump correct |
| `HostState #[non_exhaustive]` | Restricts external pattern matching | ✓ Bump correct |

**Consumer coverage:** prism-bin (reg+dev) ✓, prism-core (dev) ✓, Cargo.lock ✓, non-exhaustive-violation/Cargo.lock ✓. prism-query + prism-sensors use path-only (no pin needed in workspace).

**Verdict: APPROVE** — No source code changes in semver-fix commit. Cumulative diff has no new findings beyond those resolved in prior cycles.

## Additional Issues Resolved During This Dispatch

- **PR description (Action 1):** Corrected inaccurate claim "S-PLUGIN-PREREQ-A, B, C, F all on develop" — PREREQ-F is factory-artifact-only, not a code dependency. Updated on GitHub via `gh pr edit --body-file` and in factory artifact.
- **CI flake (Action 2):** `test_BC_2_10_010_sigterm_causes_graceful_exit_zero` failed during local pre-push hook run under resource contention. Test passes in isolation (2.324s). This is the same class of process-lifecycle flake documented in the test's own doc comment. Prior CI runs (25929899068, 25930452298) show all test suites pass on the remote. Not a regression from the semver bump.
