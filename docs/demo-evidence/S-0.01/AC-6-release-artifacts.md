# AC-6 Evidence — Release Artifacts (5 Platforms + Checksums)

## AC Statement

Given a git tag `v1.0.0` pushed to the repository, When the release workflow
runs, Then 5 platform binaries are built, archived, SHA-256 checksums are computed,
and a GitHub Release is created with all archives and `checksums.txt` attached.

## Source

`.github/workflows/release.yml` lines 1–91

## Relevant YAML Excerpt — Trigger and Build

```yaml
on:
  push:
    tags:
      - 'v*'

jobs:
  build-release:
    strategy:
      fail-fast: false
      matrix:
        include:
          - runner: macos-latest
            target: aarch64-apple-darwin
            archive_ext: tar.gz
          - runner: macos-13
            target: x86_64-apple-darwin
            archive_ext: tar.gz
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            archive_ext: tar.gz
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-musl
            archive_ext: tar.gz
          - runner: windows-latest
            target: x86_64-pc-windows-msvc
            archive_ext: zip
    steps:
      - name: Build release binary
        run: cargo build --release --locked --target ${{ matrix.target }}
      - name: Compute SHA-256 checksum
        shell: bash
        run: |
          if command -v sha256sum >/dev/null 2>&1; then
            sha256sum "${{ env.ARCHIVE }}" >> checksums.txt
          else
            shasum -a 256 "${{ env.ARCHIVE }}" >> checksums.txt
          fi
```

## Relevant YAML Excerpt — GitHub Release Creation

```yaml
  publish-release:
    needs: build-release
    steps:
      - name: Create GitHub Release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: gh release create "${{ github.ref_name }}" --title "Prism ${{ github.ref_name }}" --generate-notes artifacts/release-*/*.tar.gz artifacts/release-*/*.zip checksums.txt
```

## Mapping

Tag trigger `v*` ensures releases never run on PRs or branch pushes. `--locked` ensures
reproducible builds from `Cargo.lock`. SHA-256 computed cross-platform (sha256sum / shasum fallback).
`gh release create` attaches all archives plus `checksums.txt`.

## Test Assertions Passed

```
ok 1  - AC-6: file exists: release.yml
ok 2  - AC-6: release.yml triggers on 'v*' tags
ok 3  - AC-6: release.yml does not trigger on pull_request
ok 4  - AC-6: release matrix target 'aarch64-apple-darwin' present
ok 5  - AC-6: release matrix target 'x86_64-apple-darwin' present
ok 6  - AC-6: release matrix target 'x86_64-unknown-linux-gnu' present
ok 7  - AC-6: release matrix target 'x86_64-unknown-linux-musl' present
ok 8  - AC-6: release matrix target 'x86_64-pc-windows-msvc' present
ok 9  - AC-6: 'cargo build --release --locked' is a real run step
ok 10 - AC-6: SHA-256 checksum step is a real run step
ok 11 - AC-6: checksums.txt referenced in release.yml
ok 12 - AC-6: 'gh release create' is a real run step
```
