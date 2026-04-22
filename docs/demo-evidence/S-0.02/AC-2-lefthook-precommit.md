# AC-2 Evidence: lefthook pre-commit hooks

Story: S-0.02 | Version: 1.4 | Date: 2026-04-21

**AC-2:** Given a developer modifies a `.rs` file and runs `git commit`, When
`lefthook install` has been run previously, Then the pre-commit hook auto-runs
`cargo fmt` on the changed files and re-stages them, then runs `cargo clippy`, and
blocks the commit if clippy produces warnings.

---

## lefthook version

```
/opt/homebrew/bin/lefthook
2.1.1
```

## lefthook.yml content

```yaml
pre-commit:
  parallel: true
  commands:
    fmt:
      glob: "*.rs"
      run: cargo fmt --check {staged_files}
      stage_fixed: true
    clippy:
      glob: "*.rs"
      run: cargo clippy --all-features -- -D warnings

pre-push:
  commands:
    check:
      run: just check
```

## lefthook validate output

```
All good
```

## pre-commit hooks block detail

- `fmt` command: runs `cargo fmt --check {staged_files}` on `*.rs` files; `stage_fixed: true` re-stages
  files that fmt modifies, preventing un-formatted code from being committed
- `clippy` command: runs `cargo clippy --all-features -- -D warnings`; blocks commit on any warning
- `parallel: true` runs fmt and clippy concurrently
- `pre-push` hook runs `just check` (full gate) before pushing

## Test gate result

`ok 1 - lefthook.yml exists` / `ok 2 - lefthook.yml has pre-commit section` / `ok 3 - lefthook.yml commands block contains fmt and clippy entries`
**PASS**
