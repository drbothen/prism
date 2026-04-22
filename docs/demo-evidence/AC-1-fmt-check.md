# AC-1 Evidence — Format Gate

## AC Statement

Given a PR that fails `cargo fmt --check`, When the CI workflow runs, Then the
PR gate fails on the format step and the PR cannot be merged.

## Source

`.github/workflows/ci.yml` lines 9–18

## Relevant YAML Excerpt

```yaml
jobs:
  fmt:
    name: Format check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --check
```

## Mapping

`cargo fmt --check` exits non-zero on any formatting deviation; because `fmt` is the first
job in the DAG and downstream jobs (`clippy`, `test`, `deny`, `audit`, `semver-checks`) all
declare `needs: fmt` transitively, a format failure blocks merge via required status checks.

## Test Assertions Passed

```
ok 1 - AC-1: file exists: ci.yml
ok 2 - AC-1: ci.yml has real 'run: cargo fmt --check' step
ok 3 - AC-1: cargo fmt step is not an echo stub
```

## Validation Tools

- `actionlint`: not installed locally — will run on CI runner
- `yamllint`: not installed locally — will run on CI runner
- `python3 yaml.safe_load`: PASS (file is valid YAML)
