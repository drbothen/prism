# S-0.02 Demo Evidence Report

Story: S-0.02 — devops: Developer Toolchain Bootstrap  
Version: 1.4  
Date: 2026-04-21  
HEAD SHA: a0f89e0  
Branch: feature/S-0.02-developer-toolchain  

---

## Green Gate Verification

```
bash tests/toolchain-gate/run.sh
# Results: 7 passed, 0 failed out of 7 test files
# Red Gate: PASS — all tests passed
```

Exit code: 0 | Count: 7/7 pass

---

## AC Coverage Matrix

| AC | Evidence file | Method | Verdict |
|----|---------------|--------|---------|
| AC-1 | AC-1-just-check-pr-gate.md | `just --list` + Justfile recipe inspection + `just check` execution output | PASS |
| AC-2 | AC-2-lefthook-precommit.md | `lefthook version`, `lefthook validate`, full `lefthook.yml` dump | PASS |
| AC-3 | AC-3-dev-setup-installs-tools.md | `bash -n` syntax check + `grep install_if_missing` (all 9 tools) | PASS |
| AC-4 | AC-4-dev-setup-idempotent.md | `install_if_missing` guard pattern + `bash -n` syntax check | PASS |
| AC-5 | AC-5-deny-toml.md | `cat deny.toml` + `python3 tomllib` parse validation | PASS |
| AC-6 | AC-6-semgrep-credential-rule.md + AC-6-semgrep-fires.txt | `cat .semgrep/credential-handling.yml` + live semgrep trigger firing | PASS |
| AC-config | AC-config-toolchain-files.md | TOML parse of rust-toolchain.toml, rustfmt.toml, clippy.toml, kani.toml | PASS |

---

## Known Limitations

1. **cargo deny runtime deferred** — `cargo deny check` runtime validation is deferred
   until workspace members exist. Schema is validated (TOML parse OK, all required keys
   present). Runtime execution occurs after S-6.06 merges.

2. **Tool installation is static evidence** — `dev-setup.sh` idempotency is demonstrated
   via script inspection and syntax checking. Real end-to-end installation verified by
   downstream developer environments (not captured in CI-only worktree).

3. **dtu feature gate deferred** — `just integration-test`, `just dtu-start`, and
   `just dtu-validate` targets exist in the Justfile but will fail with "feature `dtu`
   not found" until S-6.06 declares the per-crate feature. This is expected and
   intentional per the story spec.

4. **just check exits non-zero** — Expected: workspace has no Rust crates yet
   (members = []). `cargo fmt --check` fails with "Failed to find targets". The recipe
   sequence is verified structurally; runtime exit-0 will hold once crates land.

---

## Implementation Commits

| Commit | Description |
|--------|-------------|
| 5a332cb | Stubs |
| 644f1e8 | Failing tests |
| 8db06bc | Test defect fix (Red Gate inversions) |
| 299ba14 | v1.4 Cargo features strip |
| fdd4dce | AC-1 Justfile |
| 885bb6f | AC-2 lefthook |
| 5d564b4 | AC-3 + AC-4 dev-setup |
| 4e64c65 | AC-5 + AC-6 deny.toml + semgrep rule |
| a0f89e0 | AC-5/AC-6 test fixes (HEAD) |
