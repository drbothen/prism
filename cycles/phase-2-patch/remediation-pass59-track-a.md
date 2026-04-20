---
document_type: remediation-manifest
burst: remediation-pass59
date: "2026-04-20"
producer: story-writer
pass: 59
status: complete
---

# Remediation Manifest — Pass-59 Track A

Pass-59 adversarial review reset convergence counter to 0. This manifest records every
fix applied so pass-60 can re-verify from a clean slate.

---

## Finding HIGH-001: Stem-Only or Missing `specs/` in `inputs:` Paths

**Scope:** ~20 stories with BC paths lacking full slugs or missing the `specs/` directory prefix.
**Fix:** Replaced stem-only paths (e.g. `BC-2.18.001.md`) with full slugged filenames
(e.g. `BC-2.18.001-action-at-least-once-delivery.md`); corrected missing `specs/`
prefix (`.factory/behavioral-contracts/` → `.factory/specs/behavioral-contracts/`).

| File | Version | Changes |
|------|---------|---------|
| `S-4.02-diff-results-packs.md` | 1.1→1.2 | Stem-only slugs on 5 BCs + VP-019 fixed |
| `S-4.03-detection-rules.md` | 1.3→1.4 | Stem-only slugs on 8 BCs + VP-018 fixed |
| `S-4.04-detection-evaluation.md` | 1.1→1.2 | Stem-only slugs on 5 BCs + VP-027 fixed |
| `S-4.05-alert-generation.md` | 1.1→1.2 | Stem-only slug on BC-2.13.005 + VP-028 fixed |
| `S-4.06-case-management.md` | 1.1→1.2 | Stem-only slugs on 9 BC-2.14.xxx files fixed |
| `S-4.07-case-metrics.md` | 1.1→1.2 | Stem-only slugs on 3 BCs fixed |
| `S-4.08-action-delivery.md` | 1.2→1.3 | Stem-only slugs on 9 BC-2.18.xxx files fixed |
| `S-5.03-resources-prompts.md` | 1.4→1.5 | Stem-only slugs on 4 BCs fixed |
| `S-5.05-config-loading.md` | 1.3→1.4 | Missing `specs/` prefix + slugs on 10 BC-2.06.xxx files |
| `S-5.06-action-infusion-tools.md` | 1.6→1.7 | Missing `specs/` prefix + slugs on 4 BCs |
| `S-5.07-multi-repo-git-config.md` | 1.1→1.2 | Missing `specs/` prefix + slugs on 8 BC-2.06.xxx files |
| `S-5.09-external-log-forwarding.md` | 1.2→1.3 | Missing `specs/` prefix + slug on BC-2.10.001 |
| `S-5.10-audit-trail-forwarding.md` | 1.3→1.4 | Missing `specs/` prefix on 7 BC-2.05.xxx + VP-039 slug |
| `S-6.01-cli-startup.md` | 1.2→1.3 | Missing `specs/` prefix + slugs on 4 BCs |
| `S-6.02-e2e-smoke-tests.md` | 1.2→1.3 | Missing `specs/` prefix + slugs on 5 BCs |
| `S-6.03-installation.md` | 1.1→1.2 | Missing `specs/` prefix + slugs on BC-2.10.001 and BC-2.10.006 |
| `S-6.05-migrate-storage.md` | 1.1→1.2 | Stem-only slugs on 3 BCs fixed |

**Stories with no HIGH-001 work needed (already correct in prior passes):**
S-3.03–S-3.13, S-1.02–S-1.05 (fixed in step-5), S-4.01 (had full slugs).

---

## Finding HIGH-002: `anchor_capabilities:` Mis-Anchored

**Scope:** ~19 stories with anchor_capabilities derived incorrectly.
**Fix:** Derived anchor_capabilities as the union of `capability:` fields from BC-INDEX
for every BC in the story's `behavioral_contracts:` array. BC-INDEX is authoritative
over adversary recommendations.

| File | Version | Old Value | New Value | Note |
|------|---------|-----------|-----------|------|
| `S-4.01-schedule-crud.md` | 1.4→1.5 | `[CAP-016]` | `[CAP-017]` | BC-INDEX for BC-2.13.xxx |
| `S-4.02-diff-results-packs.md` | 1.1→1.2 | `[CAP-016]` | `[CAP-018, CAP-023]` | |
| `S-4.03-detection-rules.md` | 1.3→1.4 | `[CAP-017]` | `[CAP-020, CAP-027]` | BC-2.13.009/010 → CAP-027 |
| `S-4.04-detection-evaluation.md` | 1.1→1.2 | `[CAP-017]` | `[CAP-020, CAP-021]` | |
| `S-4.05-alert-generation.md` | 1.1→1.2 | `[CAP-017]` | `[CAP-020]` | Adversary said CAP-021; BC-INDEX authoritative: CAP-020 |
| `S-4.06-case-management.md` | 1.1→1.2 | `[CAP-018]` | `[CAP-022]` | |
| `S-4.07-case-metrics.md` | 1.1→1.2 | `[CAP-018]` | `[CAP-022]` | |
| `S-4.08-action-delivery.md` | 1.2→1.3 | `[CAP-018]` | `[CAP-033]` | |
| `S-5.01-mcp-bootstrap.md` | 1.5→1.6 | `[CAP-010]` | `[CAP-005, CAP-009, CAP-015, CAP-034]` | |
| `S-5.02-tool-routing.md` | 1.1→1.2 | `[CAP-010]` | `[CAP-005, CAP-009, CAP-034]` | |
| `S-5.03-resources-prompts.md` | 1.4→1.5 | `[CAP-010]` | `[CAP-008, CAP-009, CAP-034]` | |
| `S-5.05-config-loading.md` | 1.3→1.4 | `[CAP-006]` | `[CAP-009]` | |
| `S-5.06-action-infusion-tools.md` | 1.6→1.7 | `[CAP-010]` | `[CAP-007, CAP-030, CAP-031, CAP-033]` | |
| `S-5.07-multi-repo-git-config.md` | 1.1→1.2 | `[CAP-006]` | `[CAP-009]` | |
| `S-5.09-external-log-forwarding.md` | 1.2→1.3 | `[CAP-010]` | `[CAP-034]` | Adversary said [CAP-008, CAP-025]; BC-INDEX authoritative: CAP-034 |
| `S-5.10-audit-trail-forwarding.md` | 1.3→1.4 | `[CAP-005]` | `[CAP-007]` | Adversary said [CAP-007, CAP-025]; BC-INDEX authoritative: CAP-007 |
| `S-6.01-cli-startup.md` | 1.2→1.3 | `[CAP-006, CAP-010]` | `[CAP-009, CAP-034]` | |
| `S-6.02-e2e-smoke-tests.md` | 1.2→1.3 | `[CAP-006, CAP-008, CAP-010]` | `[CAP-008, CAP-009, CAP-034]` | |
| `S-6.03-installation.md` | 1.1→1.2 | `[CAP-010]` | `[CAP-034]` | Adversary said []; BC-INDEX authoritative: CAP-034 |
| `S-6.04-credential-cli.md` | 1.1→1.2 | `[CAP-003]` | `[CAP-004]` | |
| `S-6.05-migrate-storage.md` | 1.1→1.2 | `[CAP-015]` | `[CAP-019, CAP-024]` | |

**Divergences from adversary recommendations (BC-INDEX authoritative):**
- S-4.05: adversary recommended CAP-021; BC-INDEX shows BC-2.13.013 → CAP-020. Used CAP-020.
- S-5.09: adversary recommended [CAP-008, CAP-025]; BC-INDEX shows BC-2.10.001 → CAP-034. Used CAP-034.
- S-5.10: adversary recommended [CAP-007, CAP-025]; BC-INDEX shows BC-2.05.xxx → CAP-007. Used CAP-007 only (CAP-025 not in BC-INDEX for this story's BCs).
- S-6.03: adversary recommended []; BC-INDEX shows BC-2.10.001 → CAP-034. Used CAP-034.

---

## Finding HIGH-003: DTU Stories Reference `dtu-strategy.md` Instead of `dtu-assessment.md`

**Scope:** 13 DTU stories (S-6.07–S-6.19).
**Fix:** Replaced `.factory/specs/architecture/dtu-strategy.md` with
`.factory/specs/architecture/dtu-assessment.md` in both `inputs:` frontmatter and
all body text references. A side effect of `replace_all` created duplicate
`dtu-assessment.md` lines in `inputs:` (stories already had one correct entry);
deduplicated in this pass.

| File | Version | Body References Fixed |
|------|---------|----------------------|
| `S-6.07-dtu-crowdstrike.md` | 1.3→1.4 | §3.1, §4 body refs; inputs dedup |
| `S-6.08-dtu-claroty.md` | 1.3→1.4 | §3.2 body refs; inputs dedup |
| `S-6.09-dtu-cyberint.md` | 1.3→1.4 | §3.3 body refs; inputs dedup |
| `S-6.10-dtu-armis.md` | 1.3→1.4 | §3.4 body refs; inputs dedup |
| `S-6.11-dtu-slack.md` | 1.4→1.5 | §3.5 body refs; inputs dedup |
| `S-6.12-dtu-pagerduty.md` | 1.4→1.5 | §3.6 body refs; inputs dedup |
| `S-6.13-dtu-jira.md` | 1.4→1.5 | §3.7 body refs; inputs dedup |
| `S-6.14-dtu-threatintel.md` | 1.3→1.4 | §3.8 body refs; inputs dedup |
| `S-6.15-dtu-nvd.md` | 1.3→1.4 | §3.9 body refs; inputs dedup |
| `S-6.16-dtu-datadog.md` | 1.3→1.4 | §3.10 body refs; inputs dedup |
| `S-6.17-dtu-splunk-hec.md` | 1.3→1.4 | §3.11 body refs; inputs dedup |
| `S-6.18-dtu-elasticsearch.md` | 1.3→1.4 | §3.12 body refs; inputs dedup |
| `S-6.19-dtu-otlp.md` | 1.3→1.4 | §3.13 body refs; inputs dedup |

---

## Finding MED-002: Wrong DTU Filenames in remediation-step5-track-a.md

**Scope:** Lines 111–116 of `remediation-step5-track-a.md` listed nonexistent filenames.
**Fix:** Corrected 6 wrong DTU filenames in the table.

| Line | Wrong Filename | Correct Filename |
|------|---------------|-----------------|
| 111 | `S-6.08-dtu-sentinel.md` | `S-6.08-dtu-claroty.md` |
| 112 | `S-6.09-dtu-qradar.md` | `S-6.09-dtu-cyberint.md` |
| 113 | `S-6.10-dtu-defender.md` | `S-6.10-dtu-armis.md` |
| 114 | `S-6.11-dtu-armis.md` | `S-6.11-dtu-slack.md` |
| 115 | `S-6.12-dtu-claroty.md` | `S-6.12-dtu-pagerduty.md` |
| 116 | `S-6.13-dtu-cyberint.md` | `S-6.13-dtu-jira.md` |

---

## Finding MED-004: `anchor_subsystem:` Scalar Instead of Array

**Scope:** S-6.04, S-6.05, S-6.07 had scalar `anchor_subsystem: SS-XX` instead of
YAML array `anchor_subsystem: ["SS-XX"]`.
**Fix:** Converted scalar to array form for all three stories.

| File | Version | Change |
|------|---------|--------|
| `S-6.04-credential-cli.md` | 1.1→1.2 | `SS-03` → `["SS-03"]` |
| `S-6.05-migrate-storage.md` | 1.1→1.2 | `SS-15` → `["SS-15"]` |
| `S-6.07-dtu-crowdstrike.md` | 1.3→1.4 | `SS-01` → `["SS-01"]` |

---

## Finding LOW-003: STORY-INDEX.md `level: L4` Unquoted

**Scope:** `STORY-INDEX.md` line 3 had bare scalar `level: L4`.
**Fix:** Changed to `level: "L4"` (quoted string).

| File | Change |
|------|--------|
| `STORY-INDEX.md` | `level: L4` → `level: "L4"` |

---

## Summary

| Finding | Severity | Files Modified |
|---------|----------|---------------|
| HIGH-001 | HIGH | 17 stories |
| HIGH-002 | HIGH | 21 stories (overlap with HIGH-001) |
| HIGH-003 | HIGH | 13 DTU stories |
| MED-002 | MEDIUM | 1 manifest file |
| MED-004 | MEDIUM | 3 stories (overlap with HIGH-002) |
| LOW-003 | LOW | 1 index file |

Total unique files modified: ~37 story files + 2 support files.

**Convergence status:** Pass-59 findings fully remediated. Pass-60 adversarial review
may proceed against this clean baseline.
