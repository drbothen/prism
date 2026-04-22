---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-22T18:00:00Z
phase: 3
inputs:
  - .factory/stories/S-6.20-dtu-demo-server.md (v1.1)
  - .factory/cycles/phase-3-dtu-wave-1/adversary-S-6.20-spec-review.md (Pass 1)
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md
  - .factory/specs/architecture/decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md
  - .factory/specs/architecture/dtu-assessment.md
  - .factory/policies.yaml
  - .factory/tech-debt-register.md
  - .factory/specs/behavioral-contracts/BC-2.03.009-resolve-secret-env-file.md
  - .factory/stories/S-5.05-config-loading.md
  - .factory/stories/S-1.07-credential-crud.md
  - .factory/stories/S-6.07-dtu-crowdstrike.md
  - .factory/stories/S-6.09-dtu-cyberint.md
  - crates/prism-dtu-common/src/clone.rs
  - crates/prism-dtu-threatintel/src/routes/
  - crates/prism-dtu-nvd/src/routes/dtu.rs
  - crates/prism-dtu-crowdstrike/src/routes/writes.rs
input-hash: ""
traces_to: S-6.20-dtu-demo-server.md
pass: 2
previous_review: adversary-S-6.20-spec-review.md
cycle: phase-3-dtu-wave-1
recommendation: CONDITIONAL
---

# S-6.20 Adversarial Review — Pass 2

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix — `WV1` for phase-3-dtu-wave-1
- `<PASS>`: Two-digit pass number (e.g., `P02`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Pass 1 findings retain their original IDs (F-6.20-NN). Pass 2 new findings use `ADV-WV1-P02-<SEV>-<SEQ>` (mapped to F-6.20-P02-* shorthand in this document for cross-reference continuity with Pass 1).

## Part A — Pass 1 Remediation Verification

| Finding | Status | Evidence |
|---------|--------|----------|
| F-6.20-01 SS-18 anchor | FIXED | subsystems: [], anchor_subsystem: null |
| F-6.20-02 7→6 count | FIXED | All "7" replaced in narrative, ACs, File Structure |
| F-6.20-03 fabricated ADR-002 §6 | FIXED | "New policy established by this story" replaces citation |
| F-6.20-04 BehavioralClone::seed() removed | FIXED | Task 6 thin wrapper; explicit "no trait extension" |
| F-6.20-05 two-factor bind | FIXED | Both --bind-any AND env var required |
| F-6.20-06 required-features binary | FIXED | [[bin]] required-features = ["dtu"] |
| F-6.20-07 credential_ref | PARTIAL | env: prefix scheme is undocumented — see F-6.20-P02-H-001 |
| F-6.20-08 AC-7 deterministic | FIXED | JSON-body scope + --deterministic-logging flag |
| F-6.20-09 TD-WV0-05 precheck | FIXED | Task 3 gate + AC-10 + R-DEMO-003 + EC-011 |

Summary: 8 FIXED, 1 PARTIAL, 0 NOT FIXED.

## Part B — New Findings (Pass 2)

### HIGH

**F-6.20-P02-H-001** (ADV-WV1-P02-HIGH-001): `env:` credential_ref scheme prefix is undocumented in S-5.05 / BC-2.03.009. S-5.05 Task 3 uses bare-name convention (`credential_ref = "cs_api_key"` checks `CS_API_KEY_FILE` / `CS_API_KEY`). No `env:` prefix is parsed. AC-6 will fail at credential-resolution time.

**Fix options:** (1) drop `env:` prefix → `credential_ref = 'DEMO_FAKE_CROWDSTRIKE_TOKEN'`; (2) extend S-5.05 with scheme prefix support (requires cross-story coordination); (3) defer AC-6 to follow-up integration story.

### MEDIUM

**F-6.20-P02-M-001** (ADV-WV1-P02-MED-001): AC-9 references a `bind` config field that doesn't exist in DemoConfig schema (Task 2 only has enabled/port/fixture_set/initial_failure_mode/seed/tls). AC-9 "Given" un-satisfiable.

**Fix:** Add `bind` field to DemoConfig OR rewrite AC-9 to match existing mechanisms.

**F-6.20-P02-M-002** (ADV-WV1-P02-MED-002): Seed endpoint serving architecture unspecified. README curls `http://127.0.0.1:17082/dtu/cyberint/seed` (cyberint's port), but each clone's build_router() doesn't expose an extension hook per ADR-002 §6. How does harness insert `/dtu/<clone>/seed` into each clone's HTTP server?

**Fix options:** (1) single harness admin port; (2) harness wraps Router before axum::serve; (3) drop seed endpoint — use each clone's `/dtu/configure` directly.

## Part C — Deferred Findings Re-Evaluation

| Pass 1 | Verdict | Rationale |
|--------|---------|-----------|
| F-6.20-10 onboarding plan | AGREE DEFER | Testable at first Wave 2/3 story |
| F-6.20-11 fixed ports | AGREE DEFER | Conscious deviation from dtu-assessment §4; acceptable |
| F-6.20-12 stop_all vs reset() | **ESCALATE MED→HIGH** | BehavioralClone.reset() doesn't shut down listener. AC-5 un-satisfiable via current trait. |
| F-6.20-13 Bearer x | **DOWNGRADE MED→LOW** | S-6.07 check_auth accepts any non-empty token. AC-2 as written is correct. |
| F-6.20-14 cert path | AGREE DEFER (MED) | Still needs sharpening at implementation |
| F-6.20-15 partial-start cleanup | AGREE (MED) | EC for atomic rollback would help |
| F-6.20-16/17/18 | AGREE DEFER | Estimator / POL-010-scope |
| F-6.20-19 axum+rustls bridge | **ESCALATE LOW→MED** | axum 0.7 needs axum-server with tls-rustls feature |

## Part D — Policy Rubric

All 10 POL-00* policies: PASS or N/A. POL-004 cleared post-F-6.20-01 fix.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 (1 new + 1 escalated from MED) |
| MEDIUM | 4 (2 new + 1 escalated from LOW + 1 carried MED) |
| LOW | 1 (downgraded from MED) |

**Overall Assessment:** pass-with-findings
**Convergence:** findings remain — iterate
**Readiness:** requires revision — 5 fixes required before Pass 3

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 3 (F-6.20-P02-H-001, F-6.20-P02-M-001, F-6.20-P02-M-002) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 3 / (3 + 0) = 1.00 |
| **Median severity** | 3.0 (HIGH=4, MED=2, MED=2 → median MED) |
| **Trajectory** | 9→3 (Pass 1 had 9 findings; Pass 2 has 3 new) |
| **Verdict** | FINDINGS_REMAIN |

## Part E — Novelty Narrative

Pass 2 has 3 new findings (0 CRIT, 1 HIGH, 2 MED) plus escalations of 2 deferred findings and 1 downgrade. CRITICALs eliminated. Active HIGHs reduced from 6 → 2. Not yet converged. Pass 3 recommended after the 5 required fixes.

## Part F — Recommendation

**CONDITIONAL — fix these before implementation:**

1. **F-6.20-P02-H-001**: Align credential_ref syntax with S-5.05 (drop `env:` prefix or amend S-5.05)
2. **F-6.20-12** (escalated HIGH): Specify stop_all teardown mechanism (own JoinHandle or carve-out trait extension)
3. **F-6.20-P02-M-001**: Add `bind` field to DemoConfig or rewrite AC-9
4. **F-6.20-P02-M-002**: Specify admin-listener topology for /dtu/<clone>/seed
5. **F-6.20-19** (escalated MED): Add axum-server 0.7 + tls-rustls to Cargo.toml

After these 5 fixes, Pass 3 verification recommended before worktree creation.

## Part G — Observations

- OBS-P02-001: Admin endpoints disabled when --bind-any creates operability paradox; recommend README documenting seed BEFORE --bind-any
- OBS-P02-002: AC-10 depends on TD-WV0-05 landing (threatintel + nvd /dtu/health); Task 3 protects against this
- OBS-P02-003: input-hash still empty in v1.1 frontmatter; state-manager will populate
- OBS-P02-004: Fresh-context Pass 2 caught Pass 1 error (F-6.20-13 Bearer x was not actually rejected)

## Classification totals

| Bucket | Count |
|--------|-------|
| Pass 2 CRITICAL | 0 |
| Pass 2 HIGH | 1 |
| Pass 2 MEDIUM | 2 |
| Pass 2 LOW | 0 |
| Pass 2 OBSERVATIONS | 4 |
| Pass-1 MED→HIGH escalations | 1 |
| Pass-1 LOW→MED escalations | 1 |
| Pass-1 MED→LOW downgrades | 1 |
| Pass-1 FIXED | 8 |
| Pass-1 PARTIAL | 1 |
| Pass-1 NOT FIXED | 0 |
| **Active blockers for Pass 3** | **2 HIGH + 4 MED** |
