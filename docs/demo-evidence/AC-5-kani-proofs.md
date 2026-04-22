# AC-5 Evidence — Kani Proofs + Fuzz Corpus (Post-Merge)

## AC Statement

Given a merge to `main`, When the post-merge workflow runs, Then the
`kani-proofs` job executes all 19 Kani proofs with a 300 s timeout and 8192 MB memory
limit, and uploads the report as a GitHub Actions artifact.

## Source

`.github/workflows/post-merge.yml` lines 1–27

## Relevant YAML Excerpt

```yaml
on:
  push:
    branches:
      - main

jobs:
  kani-proofs:
    name: Kani formal verification
    runs-on: ubuntu-latest
    timeout-minutes: 120
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Install kani-verifier
        run: cargo install kani-verifier --locked
      - name: Run Kani proofs
        run: cargo kani --workspace --timeout 300 --mem-limit 8192
      - name: Upload Kani report
        uses: actions/upload-artifact@v4
        with:
          name: kani-report-${{ github.sha }}
          path: kani-report/
          if-no-files-found: ignore
```

## Mapping

- Trigger `branches: [main]` scopes execution to post-merge only — never runs on PRs.
- `--timeout 300` and `--mem-limit 8192` match AC-5 spec exactly.
- `timeout-minutes: 120` provides job-level ceiling (19 proofs * 300 s = 95 min worst-case).
- Artifact upload with SHA-scoped name ensures traceability per commit.

## Test Assertions Passed

```
ok 1  - AC-5: file exists: post-merge.yml
ok 2  - AC-5: post-merge.yml is scoped to main branch
ok 3  - AC-5: kani-proofs job has real 'run: cargo kani' step
ok 4  - AC-5: kani invocation includes --timeout 300
ok 5  - AC-5: kani invocation includes --mem-limit 8192
ok 6  - AC-5: kani-proofs job has timeout-minutes: 120
ok 7  - AC-5: kani-report artifact upload step present (uses: actions/upload-artifact)
ok 8  - AC-5: fuzz target 'fuzz_prismql_parser' invoked
ok 9  - AC-5: fuzz target 'fuzz_alias_expansion' invoked
ok 10 - AC-5: fuzz target 'fuzz_normalize' invoked
ok 11 - AC-5: fuzz target 'fuzz_spec_parser' invoked
ok 12 - AC-5: fuzz target 'fuzz_template_interpolation' invoked
ok 13 - AC-5: fuzz target 'fuzz_injection_scanner' invoked
```
