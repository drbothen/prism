# AC-2 Evidence — Clippy -D warnings Gate (AD-008)

## AC Statement

Given a PR that introduces a `clippy::await_holding_lock` violation, When the
CI workflow runs, Then `cargo clippy -D warnings` fails and the PR gate blocks merge.

## Source

`.github/workflows/ci.yml` lines 20–30

## Relevant YAML Excerpt

```yaml
  clippy:
    name: Clippy (AD-008)
    needs: fmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Run clippy
        run: cargo clippy --workspace --all-features -- -D warnings
```

## Mapping

`-D warnings` promotes all clippy warnings (including `await_holding_lock`) to errors,
causing the step to exit non-zero. The `clippy` job is gated by `needs: fmt`; subsequent
jobs (`test`, `deny`, `audit`, `semver-checks`) gate on `clippy`. Any clippy error blocks
the entire chain and prevents merge.

## Test Assertions Passed

```
ok 1 - AC-2: file exists: ci.yml
ok 2 - AC-2: ci.yml has real 'cargo clippy -- -D warnings' step
ok 3 - AC-2: clippy step is not an echo stub
```
