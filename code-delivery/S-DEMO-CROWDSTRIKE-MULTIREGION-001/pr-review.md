# Final Fresh-Eyes PR Review — PR #170

**Story:** S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.4 — CrowdStrike multi-region `base_url` via `${env.CROWDSTRIKE_BASE_URL}`
**Head reviewed:** `efbcf59b` · **Base:** `develop@b38c1abc` · **Reviewer:** pr-reviewer (fresh-context, different-model cognitive diversity)
**Verdict:** ✅ **APPROVE**

> Supersedes the earlier REQUEST-CHANGES review at head `f283b8a7` (IMPORTANT-001 AC-001 evidence overstatement). That finding is closed and independently re-verified below.

---

## Summary

A tightly-scoped, production-grade change. It replaces the hardcoded us-1 `base_url` in `crowdstrike.sensor.toml` with `${env.CROWDSTRIKE_BASE_URL}`, mirroring the existing Armis/Claroty env-var pattern, and relies on the already-merged S-SPEC-ENV-VAR-001 resolver for E-SPEC-024 fail-closed behavior when the variable is unset. No production Rust code and no DTU code change — consistent with ADR-031 §D8-c (DTU is region-agnostic). The substantive change is 6 lines of TOML plus a 514-line test file (overwhelmingly doc-comments + 4 well-structured tests) and per-AC demo evidence.

The prior REQUEST-CHANGES item (IMPORTANT-001) is **genuinely resolved** on this head, verified against the actual files, not the PR narrative.

---

## Prior IMPORTANT-001 — Resolution Verification (CLOSED)

Original finding: AC-001 demo claimed the hardcoded URL was absent, but `grep -c api.crowdstrike.com` returned 1 (not 0) because the 4-region runbook comment retains all region URLs for operator reference.

Verified on head `efbcf59b`:

- **TOML field (`crowdstrike.sensor.toml:29`):** `base_url = "${env.CROWDSTRIKE_BASE_URL}"` — env-var driven, no hardcoded value. The `api.crowdstrike.com` strings appear ONLY in comment lines 25–28 (the intentional region runbook).
- **AC-001 tape** now uses field-discriminating guards instead of a whole-file grep:
  - `grep -E '^base_url\s*='` → returns the env-var field (proves env-driven)
  - `grep -E '^base_url\s*=.*api\.crowdstrike\.com' || echo 'field-not-hardcoded'` → returns `field-not-hardcoded` (proves the FIELD, not the file, is clean)
- **evidence-report.md** is now honest: it explicitly states "`grep -c api.crowdstrike.com` returns `1` because the comment contains that string. The field-discriminating guards above correctly target the `base_url` FIELD only."

The evidence no longer overstates. Keeping the operator-facing region runbook in the comment is desirable, and the demo now proves the precise claim (field is env-driven) rather than an inaccurate one (string absent from file).

---

## Checklist Results

| # | Item | Result |
|---|------|--------|
| 1 | Diff coherence | PASS — every changed file relates to the story (1 TOML field, 1 test file, 4 AC evidence sets + report). No unrelated changes. |
| 2 | Description accuracy | PASS — PR body matches the diff; mermaid traceability, BC table, region table, and checklist all correspond to actual files. |
| 3 | Test coverage | PASS — 3 non-ignored Red Gate tests cover AC-002 (eu-1 resolve), AC-003 (unset → E-SPEC-024, no panic), AC-004 (DTU loopback spec-load). AC-005 (D-747 LOCKED auth) asserted inline in AC-002 + AC-004. Tests are load-bearing: real `parse_and_validate_spec_toml` calls + assertions on the resolved `base_url`. |
| 4 | Demo evidence | PASS — `evidence-report.md` present; `.gif` + `.webm` per AC-001..AC-004; AC-005/AC-006 covered by inline assertions / SAP-1 grep (appropriate for non-visual ACs). Success (AC-002/004) and error (AC-003 fail-closed) paths both recorded. No `.txt`-only placeholders. |
| 5 | Commit quality | PASS — conventional format, story ID in every subject, clear messages; remediation commits (ecb38d37, 5d4e5603) name the finding IDs they close. |
| 6 | Diff size | PASS — substantive change well under 500 lines; 514-line test file is mostly doc-comments + 4 tests. |
| 7 | Missing changes | PASS — all 6 story ACs accounted for. SID-1 satisfied: full-pipeline DTU test is `#[ignore]`'d with a specific blocking dependency (DTU-EXT-001) AND a specific ungating story (S-6.07) AND a named non-ignored substitute test. |
| 8 | Dependency status | PASS — hard-gate dependency S-SPEC-ENV-VAR-001 (PR #165) is merged to develop. |

---

## Verification of post-prior-review remediations

- **5d4e5603 (inert-assertion fix, ADV-CSMR-PR-P04-OBS-001):** The removed `!contains("must start with http")` assertion was correctly identified as structurally inert — `parse_and_validate_spec_toml` does not call `validate_sensor_spec`, so E-SPEC-001 is unreachable on that path and the assertion could not distinguish correct from incorrect ordering. The replacement is honest: a code comment explaining the unreachability PLUS pointers to the two genuinely load-bearing ordering tests in `env_var_resolution_tests.rs` that exercise BC-2.16.009 §VR6 via the correct SUT. Postconditions 1 (`is_err`) and 2 (error references `CROWDSTRIKE_BASE_URL`) remain load-bearing. This removes a misleading assertion rather than papering over it — the correct call.
- **efbcf59b (AC-003 evidence sync):** AC-003 tape/GIF synced to current head; command targets the correct test name.

---

## SID-1 / SAP-1 / SAP-2 Compliance

- **SID-1:** Compliant. `#[ignore]` on the DTU pipeline test cites DTU-EXT-001 (blocking dependency), S-6.07 (ungating story), and names the non-ignored substitute providing spec-load coverage via the real production load path.
- **SAP-1:** Clean — TOML + test-only diff; no new `event_type =` emissions in production code; no BC-2.16.002 catalog rows required.
- **SAP-2:** Clean — no `[[tables]]` / column changes; only the `base_url` field changed, so no DTU↔TOML schema parity risk.

---

## Findings

| Severity | Category | Finding |
|----------|----------|---------|
| — | — | No BLOCKER, IMPORTANT, or NIT findings. |

The prior IMPORTANT-001 is closed and independently verified. No residual issues.

---

## What I verified (no rubber-stamp)

- Read the cumulative diff (all 15 files) at head `efbcf59b`.
- Confirmed `crowdstrike.sensor.toml:29` field value and that `api.crowdstrike.com` appears only on comment lines (25–28).
- Confirmed the AC-001 tape uses field-discriminating guards and the evidence-report no longer overstates absence.
- Confirmed auth_type/auth_plugin (D-747 LOCKED) unchanged in TOML and asserted in two tests.
- Inspected commit 5d4e5603 to confirm the inert-assertion was honestly removed with documented substitute coverage.
- Confirmed CI gates green on the immediately-prior identical run (fmt, clippy, audit, deny, semver, all 5 target test matrices, non-exhaustive 49/49, perimeter, no-default-features); the re-run triggered by the latest commit is in progress on the same tree that already passed.

---

**Verdict: APPROVE.** Merge-ready pending CI re-run completion.
