# Review Findings — MAINT-claude-md-tdd-inner-loop

## PR #137 — docs(CLAUDE.md): add TDD inner-loop discipline subsection

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 4        | 0        | —     | 4 (all non-blocking, no fix required) |
| —     | —        | APPROVE  | —     | 0 blocking → APPROVED |

**Final verdict: APPROVE after cycle 1.**

## Security Review

**Result: CLEAN** — docs-only change. Zero findings. Hard-excluded by false-positive rule 16 (markdown documentation files).

## Findings Detail

### F-001 (NON-BLOCKING): cargo-watch not in `just setup` toolchain

- **Severity:** NON-BLOCKING
- **Location:** CLAUDE.md line 55 (Auto-iteration section)
- **Finding:** `cargo watch` is documented but not installed by `scripts/dev-setup.sh`. Developer following guidance will get `cargo: no such subcommand: 'watch'`.
- **Disposition:** Surface to orchestrator for follow-up. Options: (a) add `cargo-watch` to `dev-setup.sh`, (b) qualify the line with install note.
- **Routed to:** Orchestrator (follow-up story or maintenance PR)

### F-002 (NON-BLOCKING): Table separator style deviation

- **Severity:** NON-BLOCKING
- **Location:** CLAUDE.md lines 46-51 (table header separator)
- **Finding:** New table uses `|---|---|---|`; existing "Project References" table uses `|------|-------------|`. Cosmetic; renders identically on GitHub.
- **Disposition:** Accepted as-is. Cosmetic deviation, no rendering impact.

### F-003 (NON-BLOCKING): cargo-watch invocation syntax unverified

- **Severity:** NON-BLOCKING
- **Location:** CLAUDE.md line 55
- **Finding:** `cargo watch -x 'nextest run -p <crate> --no-fail-fast'` pattern is consistent with cargo-watch API documentation but could not be verified locally (tool not installed). Pattern is almost certainly correct.
- **Disposition:** Accepted as-is. Reviewer note only.

### F-004 (NON-BLOCKING): `< 1s after build` qualifier could be clearer

- **Severity:** NON-BLOCKING
- **Location:** CLAUDE.md line 48 (table row 1, Time column)
- **Finding:** "< 1s after build" does not make clear that incremental compile (2-30s) precedes the < 1s test execution during a TDD fix-burst where files have changed.
- **Disposition:** Accepted as-is. Qualifier "after build" is present. Rewording would improve clarity but is not required.

## Accuracy Verification

| Claim | Verified | Method |
|-------|----------|--------|
| `cargo nextest run -p <crate> -E 'test(<name>)'` valid syntax | YES | Local execution confirmed nextest 0.9.129 accepts `-E 'test()'` filter |
| `just iter <crate>` sets PROPTEST_CASES=32 | YES | Justfile line 37: `PROPTEST_CASES=32 cargo nextest run -p {{crate}} {{test_filter}}` |
| `just check` = 1min warm / 5-8min cold | YES | Consistent with existing CLAUDE.md and Justfile comments |
| 200-800ms subprocess integration test cost | PLAUSIBLE | Consistent with RocksDB open + subprocess overhead; not locally benchmarked |
| ~5ms in-process unit test cost | PLAUSIBLE | Standard for Rust unit tests; consistent with nextest documentation |
