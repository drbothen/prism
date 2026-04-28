# Demo Evidence Report — S-3.0.01

**Story:** lefthook: fix pre-commit fmt hook (cargo fmt --all --check)
**Implementation SHA:** 762ab150
**Recorded:** 2026-04-28
**Branch:** feature/S-3.0.01

---

## Coverage Summary

| AC | Description | Recording | Command Shown | Expected Exit | Observed Exit |
|----|-------------|-----------|---------------|---------------|---------------|
| AC-2 | Misformatted .rs file blocks commit | AC-2-fmt-bad.gif / .webm | `cargo fmt --all --check` (the exact command lefthook calls) | non-zero (1) | 1 |
| AC-3 | Clean workspace passes fmt hook | AC-3-fmt-clean.gif / .webm | `lefthook run pre-commit --all-files --command fmt` | 0 | 0 |

---

## AC-2 — Negative Path: misformatted file causes non-zero exit

**File:** `AC-2-fmt-bad.gif` / `AC-2-fmt-bad.webm`
**Tape:** `AC-2-fmt-bad.tape`

**What is shown:**
A standalone Rust file containing `fn bad(){let x=1+2;println!("{}",x);}` (no spacing,
one-liner) is checked with `cargo fmt --all --check` — the exact command configured in
`lefthook.yml`. The command prints the expected diff and exits with `exit_code=1`.

**Fixture approach:** A temporary crate at `/tmp/prism-demo-fmt-bad/` was used so no tracked
workspace files were modified. The fixture is not committed.

**Key output visible in recording:**
- Diff output showing `-fn bad(){...}` vs `+fn bad() { ... }` multi-line form
- `exit_code=1` echo confirming non-zero exit

---

## AC-3 — Positive Path: clean workspace passes fmt hook

**File:** `AC-3-fmt-clean.gif` / `AC-3-fmt-clean.webm`
**Tape:** `AC-3-fmt-clean.tape`

**What is shown:**
`lefthook run pre-commit --all-files --command fmt` runs against the worktree at SHA
762ab150. All workspace crates pass `cargo fmt --all --check`. Lefthook prints
`fmt (N seconds)` with a checkmark and exits `exit_code=0`.

**Key output visible in recording:**
- Lefthook banner: `lefthook v2.1.1  hook: pre-commit`
- `fmt` command line with elapsed time and success indicator
- `exit_code=0` echo confirming zero exit

---

## Notes

- AC-1 (diff verification of `cargo fmt --all --check` in lefthook.yml) is validated by
  code review — no recording needed per task brief.
- AC-4 (stage_fixed decision) is documented in the PR description — no recording needed.
- The `--all-files` flag on lefthook bypasses the staged-file glob filter so the hook
  runs unconditionally, matching the "workspace is clean" scenario for AC-3.
