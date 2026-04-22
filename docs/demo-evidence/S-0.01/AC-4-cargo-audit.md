# AC-4 Evidence — Cargo Audit + Step Order

## AC Statement

Given a PR that introduces a dependency listed in the RustSec advisory
database, When `cargo audit` runs, Then the PR gate fails and merge is blocked.

## Source

`.github/workflows/ci.yml` lines 62–95

## Relevant YAML Excerpt — Audit Job

```yaml
  audit:
    name: Cargo audit (RustSec)
    needs: deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-audit
        run: cargo install cargo-audit --locked
      - name: Run cargo audit
        run: cargo audit
```

## Step Order Evidence (AD-008 Compliance)

The full gate step order as verified by AC-4 tests:

| Step | Job | Line |
|------|-----|------|
| 1 | fmt (`cargo fmt --check`) | 18 |
| 2 | clippy (`-D warnings`) | 30 |
| 3 | test (5-platform matrix) | 60 |
| 4 | deny (`cargo deny check`) | 71 |
| 5 | audit (`cargo audit`) | 83 |
| 6 | semver-checks | 95 |

This matches `tooling-selection.md` § CI Pipeline Structure exactly.

## Test Assertions Passed

```
ok 1 - AC-4: file exists: ci.yml
ok 2 - AC-4: ci.yml has real 'run: cargo audit' step
ok 3 - AC-4: ci.yml has real 'run: cargo deny check' step
ok 4 - AC-4-order: step order correct — fmt (line 18) before clippy (line 30)
ok 5 - AC-4-order: step order correct — clippy (line 30) before test (line 60)
ok 6 - AC-4-order: step order correct — test (line 60) before deny (line 71)
ok 7 - AC-4-order: step order correct — deny (line 71) before audit (line 83)
ok 8 - AC-4-order: step order correct — audit (line 83) before semver-checks (line 95)
```
