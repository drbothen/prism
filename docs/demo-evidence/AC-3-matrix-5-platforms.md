# AC-3 Evidence — 5-Platform Matrix

## AC Statement

Given a PR, When the CI workflow runs, Then all 5 platform matrix jobs
(macOS ARM64, macOS x86_64, Linux glibc, Linux musl, Windows) run in parallel and all
must pass before the PR status check is green.

## Source

`.github/workflows/ci.yml` lines 32–60

## Relevant YAML Excerpt

```yaml
  test:
    name: Test (${{ matrix.target }})
    needs: clippy
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - runner: macos-latest
            target: aarch64-apple-darwin
          - runner: macos-13
            target: x86_64-apple-darwin
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-musl
            install_musl: true
          - runner: windows-latest
            target: x86_64-pc-windows-msvc
```

## Mapping

`fail-fast: false` ensures all 5 legs run to completion independently. All 5 matrix jobs
must report green for the aggregated `test` job to pass. GitHub branch protection requiring
the `test` job enforces that all 5 pass before merge.

## Test Assertions Passed

```
ok 1  - AC-3: file exists: ci.yml
ok 2  - AC-3: matrix target 'aarch64-apple-darwin' present
ok 3  - AC-3: matrix target 'x86_64-apple-darwin' present
ok 4  - AC-3: matrix target 'x86_64-unknown-linux-gnu' present
ok 5  - AC-3: matrix target 'x86_64-unknown-linux-musl' present
ok 6  - AC-3: matrix target 'x86_64-pc-windows-msvc' present
ok 7  - AC-3: runner 'macos-latest' listed under matrix.include
ok 8  - AC-3: runner 'macos-13' listed under matrix.include
ok 9  - AC-3: runner 'ubuntu-latest' listed under matrix.include
ok 10 - AC-3: runner 'windows-latest' listed under matrix.include
ok 11 - AC-3: musl-tools install step present for Linux musl target
ok 12 - AC-3: fail-fast: false is set (all matrix legs run independently)
```
