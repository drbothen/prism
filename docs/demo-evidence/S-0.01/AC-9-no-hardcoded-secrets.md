# AC-9 Evidence — No Hardcoded Secrets

## AC Statement

No secrets (tokens, API keys) appear in GitHub Actions logs — all sensitive
values are referenced via `secrets.VARNAME` and GitHub Actions automatically masks them.

## Source

All three workflow files scanned: `ci.yml`, `post-merge.yml`, `release.yml`

## Evidence: Secret References Use `secrets.VARNAME`

All secrets in workflow files use the `${{ secrets.VARNAME }}` expression pattern,
which GitHub Actions automatically redacts from log output.

| Secret | Workflow | Reference |
|--------|----------|-----------|
| `GITHUB_TOKEN` | release.yml | `${{ secrets.GITHUB_TOKEN }}` (auto-provided, no config needed) |
| `HOMEBREW_TAP_TOKEN` | release.yml | `${{ secrets.HOMEBREW_TAP_TOKEN }}` |
| `CHOCOLATEY_API_KEY` | release.yml | `${{ secrets.CHOCOLATEY_API_KEY }}` |
| `CRATES_IO_TOKEN` | release.yml | `${{ secrets.CRATES_IO_TOKEN }}` |

## Grep Scan — No Hardcoded Patterns

The test scans for the following patterns and finds zero matches:
- `ghp_` (GitHub PAT prefix)
- `AKIA` (AWS key prefix)
- `-----BEGIN` (PEM private key)
- Any string matching `token: [a-zA-Z0-9]{20,}` (literal token assignment)

## Test Assertions Passed

```
ok 1 - AC-9: file exists: ci.yml
ok 2 - AC-9: file exists: post-merge.yml
ok 3 - AC-9: file exists: release.yml
ok 4 - AC-9: 'secrets.HOMEBREW_TAP_TOKEN' is referenced in a workflow file
ok 5 - AC-9: 'secrets.CHOCOLATEY_API_KEY' is referenced in a workflow file
ok 6 - AC-9: 'secrets.CRATES_IO_TOKEN' is referenced in a workflow file
ok 7 - AC-9: no obvious hardcoded secret pattern in ci.yml
ok 8 - AC-9: no obvious hardcoded secret pattern in post-merge.yml
ok 9 - AC-9: no obvious hardcoded secret pattern in release.yml
```
