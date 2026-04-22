# AC-8 Evidence — crates.io Publish

## AC Statement

Given a successful release build, When the release workflow completes, Then
`cargo publish` is invoked for all published crates in dependency-safe order using
`CRATES_IO_TOKEN`.

## Source

`.github/workflows/release.yml` lines 150–169

## Relevant YAML Excerpt

```yaml
  crates-io-publish:
    name: Publish to crates.io
    needs: build-release
    runs-on: ubuntu-latest
    env:
      CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Publish prism-core (first, dependency order)
        run: cargo publish -p prism-core --no-verify
      - name: Publish remaining crates
        run: |
          cargo publish -p prism-ocsf --no-verify
          cargo publish -p prism-query --no-verify
          cargo publish -p prism-spec-engine --no-verify
          cargo publish -p prism-operations --no-verify
          cargo publish -p prism-security --no-verify
          cargo publish -p prism --no-verify
```

## Mapping

`needs: build-release` gates publication behind successful 5-platform builds.
`CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}` is the environment variable
cargo uses for registry authentication; the secret is never printed in logs.
Publish order: `prism-core` first (no workspace deps), then leaf crates.

## Test Assertions Passed

```
ok 1 - AC-8: file exists: release.yml
ok 2 - AC-8: crates-io-publish job defined in release.yml
ok 3 - AC-8: crates-io-publish has 'needs: build-release' gate
ok 4 - AC-8: 'run: cargo publish' is a real step
ok 5 - AC-8: CRATES_IO_TOKEN referenced in release.yml
ok 6 - AC-8: 'prism-core' referenced in publish steps
```
