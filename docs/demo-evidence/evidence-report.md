# S-0.01 Demo Evidence Report

**Story:** S-0.01 — devops: CI/CD Pipeline and Release Workflow
**Date:** 2026-04-21
**Branch:** `feature/S-0.01-ci-cd-pipeline`
**Implementation commit:** `bd6a04b`
**Test-fix commit:** `a913aa1` (BSD grep portability fix)
**Evidence recorder:** demo-recorder agent

---

## AC Coverage Matrix

| AC | Evidence File | Method | Verdict |
|----|---------------|--------|---------|
| AC-1 | AC-1-fmt-check.md | YAML excerpt + 3 test assertions | SATISFIED |
| AC-2 | AC-2-clippy-D-warnings.md | YAML excerpt + 3 test assertions | SATISFIED |
| AC-3 | AC-3-matrix-5-platforms.md | YAML excerpt + 12 test assertions | SATISFIED |
| AC-4 | AC-4-cargo-audit.md | YAML excerpt + step-order assertions (8 tests) | SATISFIED |
| AC-5 | AC-5-kani-proofs.md | YAML excerpt + 13 test assertions | SATISFIED |
| AC-6 | AC-6-release-artifacts.md | YAML excerpt + 12 test assertions | SATISFIED |
| AC-7 | AC-7-homebrew-tap.md | YAML excerpt + 6 test assertions | SATISFIED |
| AC-8 | AC-8-crates-io-publish.md | YAML excerpt + 6 test assertions | SATISFIED |
| AC-9 | AC-9-no-hardcoded-secrets.md | Secret-pattern grep scan + 9 test assertions | SATISFIED |

---

## Green Gate Verification

**Command:** `bash tests/ci-gate/run.sh`
**Full output:** `ci-gate-run.txt`

```
# S-0.01 Red Gate Summary
# Total:  72
# Passed: 72
# Failed: 0
# Skipped (tool not found): 0
```

**Exit code:** 0

---

## YAML Structural Validation

**Full output:** `yaml-validation.txt`

| File | actionlint | yamllint | python yaml.safe_load |
|------|-----------|---------|----------------------|
| ci.yml | not installed — runs on CI | not installed — runs on CI | VALID |
| post-merge.yml | not installed — runs on CI | not installed — runs on CI | VALID |
| release.yml | not installed — runs on CI | not installed — runs on CI | VALID |

---

## Files in This Evidence Package

```
docs/demo-evidence/
  evidence-report.md           (this file)
  ci-gate-run.txt              (72/72 TAP output, exit 0)
  yaml-validation.txt          (python yaml.safe_load results)
  AC-1-fmt-check.md
  AC-2-clippy-D-warnings.md
  AC-3-matrix-5-platforms.md
  AC-4-cargo-audit.md
  AC-5-kani-proofs.md
  AC-6-release-artifacts.md
  AC-7-homebrew-tap.md
  AC-8-crates-io-publish.md
  AC-9-no-hardcoded-secrets.md
```

---

## Known Limitations

1. **Platform-level merge gate enforcement** (AC-1, AC-2, AC-3, AC-4): Branch protection
   rules that require specific status checks to pass before merge are configured in GitHub
   repository settings, not in workflow YAML. This evidence documents the workflow definitions
   that ENABLE the gate; the pr-manager confirms enforcement is active at PR time.

2. **Homebrew tap formula update** (AC-7): Requires external repo `1898co/homebrew-tap` and
   `HOMEBREW_TAP_TOKEN` secret configured in the GitHub repository. Workflow definition is
   complete; runtime verification happens on first `v*` tag push.

3. **Chocolatey nuspec** (AC-6, referenced in release.yml): `packaging/chocolatey/prism.nuspec`
   is referenced by the release workflow but is a packaging artifact deferred to a separate story.
   The workflow step is correct; the nuspec file is expected to be absent until that story lands.

4. **actionlint / yamllint** not installed in the local dev environment. Both tools will run
   on the CI runner. Python `yaml.safe_load` confirms all three files are syntactically valid
   YAML.
