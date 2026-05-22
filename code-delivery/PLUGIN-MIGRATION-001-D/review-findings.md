---
document_type: review-findings
story_id: PLUGIN-MIGRATION-001-D
pr_number: 153
review_start: 2026-05-22
---

# PLUGIN-MIGRATION-001-D PR Review Findings

## Convergence Tracking

| Cycle | Date | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|------|----------|----------|-------|-----------|---------|
| 1 | 2026-05-22 | 1 | 0 | 0 | 1 (suggestion) | APPROVE (no blocking) |

## Cycle 1 Findings

### Finding R1-001: CrowdStrike `base_url` hardcoded to single-region endpoint

- **Severity:** SUGGESTION
- **Category:** description / coherence
- **Location:** `crates/prism-sensors/specs/crowdstrike.sensor.toml:18`
- **Description:** `base_url = "https://api.crowdstrike.com"` is hardcoded to the US-1 region. The story spec (AC-001, Task 3) specifies the value should be `"https://api.{cloud_region}.crowdstrike.com"` (parameterized by region). The existing Rust adapter uses `cloud_region` to derive the URL dynamically. However, the spec grammar's `Interpolator` only handles `${step_name.field}` cross-step references — it does NOT support `${env.VAR}` interpolation in `base_url` at this story's scope. The spec parser stores `base_url` as a plain `String` with no env-resolution pass.
- **Assessment:** This is an intentional implementation choice by the implementer (per AC-001: "if spec grammar doesn't support it, use a canonical placeholder that DTU test can override via config injection"). The value `"https://api.crowdstrike.com"` is the canonical US-1 production endpoint, not a placeholder like `https://example.com`. The DTU test path overrides `spec.base_url` directly for testing. Multi-region parameterization (`${env.CROWDSTRIKE_BASE_URL}`) is a follow-up concern when env-variable resolution for `base_url` is implemented in the `prism-query` wiring layer (S-3.02 scope).
- **Suggested Fix:** Add an inline comment documenting the single-region limitation and the follow-up story for env-based URL parameterization. Update PR description to note this explicitly.
- **Route:** pr-manager edits PR description note (not a code blocker); SUGGESTION only.
- **Status:** OPEN (suggestion, no blocker)

## Summary

| Finding | Severity | Category | Route | Status |
|---------|----------|----------|-------|--------|
| R1-001 | SUGGESTION | coherence | pr-manager / description note | OPEN |

## Verdict: APPROVE

Zero blocking findings. One suggestion-level observation about CrowdStrike's single-region `base_url` which is an intentional implementation choice given the spec grammar's current scope and AC-001's fallback clause.

The PR is clean:
- 13/13 ACs covered by tests (AC-007..010 correctly `#[ignore]`-tagged with concrete story anchors)
- 9 Red Gate tests pass unconditionally
- Workspace: 3724/3724 GREEN
- Security review: CLEAN
- LOCAL impl cascade: CONVERGED (12 passes, 3/3 clean)
- No hardcoded dispatch match arms introduced (BC-2.16.012 INV-SPEC-PARSER-OPEN-001)
- E-SPEC-017/018 implemented with `#[non_exhaustive]` per CLAUDE.md conventions
- `reqwest::Client` timeout discipline: all test instances use `.timeout(30s)`
- `OrgSlug::new()` (not `new_unchecked`) in all test code
- Two `expect()` calls in production pipeline.rs are guarded unwraps with invariant documentation
- `chrono` dep added with `default-features = false` (F-LP3-MEDIUM-002 closure)
- DTU crates added as dev-dependencies only (not production deps — correct per architectural boundary)

**APPROVE — proceed to CI wait and merge.**
