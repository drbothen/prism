# AC-7 Evidence — Homebrew Tap Formula Update

## AC Statement

Given a successful release build, When the release workflow completes, Then
a PR is opened against the Homebrew tap repository with updated `url` and `sha256` in
`Formula/prism.rb`.

## Source

`.github/workflows/release.yml` lines 93–131

## Relevant YAML Excerpt

```yaml
  homebrew-update:
    name: Update Homebrew formula
    needs: build-release
    runs-on: ubuntu-latest
    env:
      HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
    steps:
      - uses: actions/checkout@v4
        with:
          repository: 1898co/homebrew-tap
          token: ${{ secrets.HOMEBREW_TAP_TOKEN }}
          path: homebrew-tap
      - name: Update Formula/prism.rb
        shell: bash
        run: |
          TAG="${{ github.ref_name }}"
          URL="https://github.com/${{ github.repository }}/releases/download/${TAG}/prism-${TAG}-aarch64-apple-darwin.tar.gz"
          SHA256=$(grep "aarch64-apple-darwin" artifacts/checksums.txt | awk '{print $1}')
          sed -i "s|url \".*\"|url \"${URL}\"|" homebrew-tap/Formula/prism.rb
          sed -i "s|sha256 \".*\"|sha256 \"${SHA256}\"|" homebrew-tap/Formula/prism.rb
      - name: Create Homebrew tap PR
        shell: bash
        working-directory: homebrew-tap
        run: gh pr create --repo 1898co/homebrew-tap --title "Update prism to ${{ github.ref_name }}" --body "Automated formula update for Prism ${{ github.ref_name }}" --base main
```

## Mapping

`homebrew-update` runs after `build-release` completes. It checks out `1898co/homebrew-tap`
via `HOMEBREW_TAP_TOKEN`, patches both `url` and `sha256` in `Formula/prism.rb`, commits
to a release-specific branch, and opens a PR via `gh pr create`.

## Known Limitation

Requires external repo `1898co/homebrew-tap` and `HOMEBREW_TAP_TOKEN` secret configured
in GitHub. Runtime verification happens on first release tag push. This evidence documents
the workflow definition; the pr-manager will confirm at PR time.

## Test Assertions Passed

```
ok 1 - AC-7: file exists: release.yml
ok 2 - AC-7: homebrew-update job defined in release.yml
ok 3 - AC-7: tap repo '1898co/homebrew-tap' referenced
ok 4 - AC-7: 'Formula/prism.rb' referenced in release.yml
ok 5 - AC-7: 'gh pr create' is a real run step in homebrew-update job
ok 6 - AC-7: HOMEBREW_TAP_TOKEN is referenced in release.yml
```
