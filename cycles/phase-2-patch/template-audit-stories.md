---
document_type: audit-report
audit_target: stories
scope: all S-*.md under .factory/stories/
auditor: story-writer (read-only)
timestamp: "2026-04-20T00:00:00Z"
total_stories: 75
stories_with_gaps: 75
stories_clean: 0
compliance_pct: "0.0%"
reference: STATE.md pre-build sweep scope
---

# Template Compliance Audit — Stories

**Date:** 2026-04-20
**Scope:** All 75 story files under `.factory/stories/S-*.md`
**Mode:** Read-only. No story files modified.
**Reference:** STATE.md pre-build sweep scope (inputs, level, points, blocks, assumption_validations, risk_mitigations; ## Edge Cases, ## Library & Framework Requirements, ## Architecture Compliance Rules)

---

## Executive Summary

| Metric | Value |
|--------|-------|
| Total stories audited | 75 |
| Stories with ANY gap | 75 |
| Stories fully clean | 0 |
| Compliance | 0.0% |
| Malformed frontmatter (unparseable) | 0 |

Every story has at least one gap. The gaps cluster tightly into two categories:

1. **Corpus-wide frontmatter fields never added** — `level`, `inputs`, `points`, `anchor_bcs`, `anchor_capabilities`, `anchor_subsystem` are missing from all 75 stories. These fields exist in the canonical template but were not populated during the Phase 2/3 story writing bursts.

2. **Missing sections in a significant minority** — `## Edge Cases` is absent from 49 stories; `## Dev Notes` (or `## Notes`) absent from 18 stories; `## Architecture Compliance Rules` absent from 13 stories.

No frontmatter is malformed — all 75 files parse cleanly. The `## Changelog` section is present in all 75 stories.

---

## Frequency Table — Missing Frontmatter Fields

| Field | Stories Missing | % of Corpus | Priority |
|-------|----------------|-------------|----------|
| `level` | 75 | 100% | HIGH (STATE.md explicit scope) |
| `inputs` | 75 | 100% | HIGH (STATE.md explicit scope) |
| `points` | 75 | 100% | HIGH (STATE.md explicit scope) |
| `anchor_bcs` | 75 | 100% | HIGH |
| `anchor_capabilities` | 75 | 100% | HIGH |
| `anchor_subsystem` | 75 | 100% | HIGH |
| `assumption_validations` | 62 | 83% | HIGH (STATE.md explicit scope) |
| `risk_mitigations` | 61 | 81% | HIGH (STATE.md explicit scope) |
| `blocks` | 59 | 79% | HIGH (STATE.md explicit scope) |

Notes:
- `assumption_validations` is present in 13 stories (mostly DTU clones S-6.06 through S-6.19 which have it).
- `risk_mitigations` is present in 14 stories.
- `blocks` is present in 16 stories (those with explicit blocking relationships declared at story-write time).
- `input-hash` field is present in all 75 stories but is `null` in all 75 — treated as "needs computation" not "missing field" for this audit, consistent with STATE.md scope.

---

## Frequency Table — Missing Sections

| Section | Stories Missing | % of Corpus | Priority |
|---------|----------------|-------------|----------|
| `## Edge Cases` | 49 | 65% | HIGH (STATE.md explicit scope) |
| `## Dev Notes` (or `## Notes`) | 18 | 24% | MEDIUM |
| `## Architecture Compliance Rules` | 13 | 17% | HIGH (STATE.md explicit scope) |
| `## User Story` / `## Narrative` | 0 | 0% | OK |
| `## Acceptance Criteria` | 0 | 0% | OK |
| `## Tasks` | 0 | 0% | OK |
| `## Library & Framework Requirements` | 0 | 0% | OK |
| `## Changelog` | 0 | 0% | OK |

Notes:
- All 75 stories have `## Narrative` (accepted alias for `## User Story`).
- All 75 stories have `## Acceptance Criteria`, `## Tasks`, `## Library and Framework Requirements`, and `## Changelog`.
- The 18 stories missing `## Dev Notes` generally have a `## Notes` section instead — this is an alias ambiguity; the remedy is normalization to `## Dev Notes`.
- The 13 stories missing `## Architecture Compliance Rules` are concentrated in the DTU clones (S-6.07 through S-6.19).

---

## Malformed Frontmatter — Must-Fix-First List

**None.** All 75 story files have well-formed YAML frontmatter that parses correctly. No must-fix-first items.

---

## Gap Inventory — All 75 Stories with Gaps

Legend: `chg` = changelog present (yes/no) | `ver` = version field value | `pts` = points field value

| File (relative to repo root) | Story ID | Missing Frontmatter Fields | Missing Sections | chg | ver | pts | Notes |
|------------------------------|----------|---------------------------|------------------|-----|-----|-----|-------|
| `.factory/stories/S-0.01-ci-cd-pipeline.md` | S-0.01 | level, inputs, points, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks field present |
| `.factory/stories/S-0.02-developer-toolchain.md` | S-0.02 | level, inputs, points, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks field present |
| `.factory/stories/S-1.01-foundational-types.md` | S-1.01 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-1.02-entity-types.md` | S-1.02 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.2 | MISSING | input-hash=null |
| `.factory/stories/S-1.03-capability-resolution.md` | S-1.03 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-1.04-ocsf-schema-loading.md` | S-1.04 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-1.05-ocsf-field-mapping.md` | S-1.05 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-1.06-credential-store.md` | S-1.06 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-1.07-credential-crud.md` | S-1.07 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.3 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-1.08-feature-flags.md` | S-1.08 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.1 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-1.09-confirmation-tokens.md` | S-1.09 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.1 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-1.10-prompt-injection-defense.md` | S-1.10 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.1 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-1.11-spec-loading.md` | S-1.11 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.1 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-1.12-hot-reload.md` | S-1.12 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.1 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-1.13-sensor-write-specs.md` | S-1.13 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.1 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-1.14-infusion-specs.md` | S-1.14 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-1.15-wasm-runtime.md` | S-1.15 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.2 | MISSING | input-hash=null |
| `.factory/stories/S-2.01-rocksdb-init.md` | S-2.01 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-2.02-audit-buffer-watchdog.md` | S-2.02 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-2.03-decorators-internal-tables.md` | S-2.03 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-2.04-audit-construction.md` | S-2.04 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-2.05-audit-events.md` | S-2.05 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-2.06-datasource-trait.md` | S-2.06 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-2.07-per-sensor-auth.md` | S-2.07 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-2.08-event-tables.md` | S-2.08 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.01-prismql-parser.md` | S-3.01 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.02-query-materialization.md` | S-3.02 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.03-explain-query.md` | S-3.03 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.04-alias-system.md` | S-3.04 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — | yes | 1.1 | MISSING | input-hash=null; all sections present |
| `.factory/stories/S-3.05-pagination-caching.md` | S-3.05 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.2 | MISSING | input-hash=null |
| `.factory/stories/S-3.06-prismql-write-parser.md` | S-3.06 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.07-write-execution.md` | S-3.07 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.08-hidden-columns.md` | S-3.08 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.09-query-profiling.md` | S-3.09 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.10-cost-estimation.md` | S-3.10 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.11-in-query-caching.md` | S-3.11 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.12-column-pruning.md` | S-3.12 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-3.13-dynamic-table-availability.md` | S-3.13 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.3 | MISSING | input-hash=null |
| `.factory/stories/S-4.01-schedule-crud.md` | S-4.01 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.3 | MISSING | input-hash=null |
| `.factory/stories/S-4.02-diff-results-packs.md` | S-4.02 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-4.03-detection-rules.md` | S-4.03 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.2 | MISSING | input-hash=null |
| `.factory/stories/S-4.04-detection-evaluation.md` | S-4.04 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-4.05-alert-generation.md` | S-4.05 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-4.06-case-management.md` | S-4.06 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-4.07-case-metrics.md` | S-4.07 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-4.08-action-delivery.md` | S-4.08 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-5.01-mcp-bootstrap.md` | S-5.01 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.4 | MISSING | input-hash=null |
| `.factory/stories/S-5.02-tool-routing.md` | S-5.02 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-5.03-resources-prompts.md` | S-5.03 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.4 | MISSING | input-hash=null |
| `.factory/stories/S-5.04-sensor-health.md` | S-5.04 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.4 | MISSING | input-hash=null |
| `.factory/stories/S-5.05-config-loading.md` | S-5.05 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.3 | MISSING | input-hash=null |
| `.factory/stories/S-5.06-action-infusion-tools.md` | S-5.06 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.5 | MISSING | input-hash=null |
| `.factory/stories/S-5.07-multi-repo-git-config.md` | S-5.07 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-5.08-diagnostics-logs-cli.md` | S-5.08 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-5.09-external-log-forwarding.md` | S-5.09 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-5.10-audit-trail-forwarding.md` | S-5.10 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.2 | MISSING | input-hash=null |
| `.factory/stories/S-6.01-cli-startup.md` | S-6.01 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-6.02-e2e-smoke-tests.md` | S-6.02 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.2 | MISSING | input-hash=null |
| `.factory/stories/S-6.03-installation.md` | S-6.03 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases | yes | 1.1 | MISSING | input-hash=null |
| `.factory/stories/S-6.04-credential-cli.md` | S-6.04 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; has ## Notes not ## Dev Notes |
| `.factory/stories/S-6.05-migrate-storage.md` | S-6.05 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; has ## Notes not ## Dev Notes |
| `.factory/stories/S-6.06-dtu-common.md` | S-6.06 | level, inputs, points, assumption_validations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+risk_mitigations present |
| `.factory/stories/S-6.07-dtu-crowdstrike.md` | S-6.07 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.08-dtu-claroty.md` | S-6.08 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.09-dtu-cyberint.md` | S-6.09 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.10-dtu-armis.md` | S-6.10 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.11-dtu-slack.md` | S-6.11 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.2 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.12-dtu-pagerduty.md` | S-6.12 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.2 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.13-dtu-jira.md` | S-6.13 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.2 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.14-dtu-threatintel.md` | S-6.14 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.15-dtu-nvd.md` | S-6.15 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.16-dtu-datadog.md` | S-6.16 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.17-dtu-splunk-hec.md` | S-6.17 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.18-dtu-elasticsearch.md` | S-6.18 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |
| `.factory/stories/S-6.19-dtu-otlp.md` | S-6.19 | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes | yes | 1.1 | MISSING | input-hash=null; blocks+assumption_validations+risk_mitigations present |

---

## Top 20 Worst Offenders (by total gap count)

All 75 stories share the same 6 corpus-wide missing frontmatter fields (level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem). The ranking below uses additional missing fields + missing sections as a secondary sort.

| Rank | Story ID | File | Total Missing (FM+Sections) | Missing FM | Missing Sections |
|------|----------|------|-----------------------------|------------|-----------------|
| 1 | S-1.01 | S-1.01-foundational-types.md | 10 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases |
| 1 | S-1.02 | S-1.02-entity-types.md | 10 | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Edge Cases |
| 1 | S-1.03 | S-1.03-capability-resolution.md | 10 | (same as S-1.01) | ## Edge Cases |
| 1 | S-1.04 | S-1.04-ocsf-schema-loading.md | 10 | (same as S-1.01) | ## Edge Cases |
| 1 | S-1.05 | S-1.05-ocsf-field-mapping.md | 10 | (same as S-1.01) | ## Edge Cases |
| 1 | S-1.06 | S-1.06-credential-store.md | 10 | (same as S-1.01) | ## Edge Cases |
| 1 | S-1.14 | S-1.14-infusion-specs.md | 10 | (same as S-1.01) | ## Edge Cases |
| 1 | S-1.15 | S-1.15-wasm-runtime.md | 10 | (same as S-1.01) | ## Edge Cases |
| 1 | S-2.01–S-2.08 | S-2.0[1-8]-*.md (8 stories) | 10 each | (same as S-1.01) | ## Edge Cases |
| 1 | S-3.01–S-3.03, S-3.05–S-3.13 | S-3.0*-*.md (12 stories) | 10 each | (same as S-1.01) | ## Edge Cases |
| 1 | S-4.01–S-4.08 | S-4.0*-*.md (8 stories) | 10 each | (same as S-1.01) | ## Edge Cases |
| 1 | S-5.01–S-5.10 | S-5.0*-*.md (10 stories) | 10 each | (same as S-1.01) | ## Edge Cases |
| 1 | S-6.01–S-6.03 | S-6.0[1-3]-*.md (3 stories) | 10 each | (same as S-1.01) | ## Edge Cases |
| T-14 | S-6.04 | S-6.04-credential-cli.md | 10 | (same as S-1.01) | ## Dev Notes |
| T-14 | S-6.05 | S-6.05-migrate-storage.md | 10 | (same as S-1.01) | ## Dev Notes |
| T-14 | S-0.01 | S-0.01-ci-cd-pipeline.md | 9 | level, inputs, points, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes |
| T-14 | S-0.02 | S-0.02-developer-toolchain.md | 9 | level, inputs, points, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes |
| T-16 | S-6.07–S-6.19 | S-6.07–S-6.19 (13 DTU stories) | 8 each | level, inputs, points, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Architecture Compliance Rules, ## Dev Notes |
| T-17 | S-6.06 | S-6.06-dtu-common.md | 8 | level, inputs, points, assumption_validations, anchor_bcs, anchor_capabilities, anchor_subsystem | ## Dev Notes |
| T-18 | S-1.07–S-1.13, S-3.04 | 8 stories | 9 each | level, inputs, points, blocks, assumption_validations, risk_mitigations, anchor_bcs, anchor_capabilities, anchor_subsystem | — (sections complete) |

---

## Pattern Analysis — Remediation Groups

### Group A: Corpus-Wide (75 stories) — Add 6 frontmatter fields
Fields: `level`, `inputs`, `points`, `anchor_bcs`, `anchor_capabilities`, `anchor_subsystem`

Recommended defaults for batch remediation:
- `level: "implementation"` — all are implementation-phase stories
- `inputs: []` — no inputs tracked yet; check-input-drift will populate
- `points: TBD` — requires story-writer judgment per story; cannot auto-fill
- `anchor_bcs: []` — to be sourced from `behavioral_contracts` field already present
- `anchor_capabilities: []` — to be sourced from BC traceability
- `anchor_subsystem: TBD` — to be derived from `subsystems` or `target_module` field

### Group B: 62 stories — Add `assumption_validations: []`
Stories missing this: all except S-6.06 through S-6.19 (which have it as empty or populated).

### Group C: 61 stories — Add `risk_mitigations: []`
Similar distribution to Group B.

### Group D: 59 stories — Add `blocks` field
16 stories already have `blocks` declared. 59 need it added (value may be `[]` or populated list).

### Group E: 49 stories — Add `## Edge Cases` section
Concentrated in SS-1 through SS-5 product stories. DTU stories (S-6.06+) generally already have an Edge Cases section.

### Group F: 18 stories — Normalize `## Dev Notes`
Stories S-0.01, S-0.02, S-6.04, S-6.05 have a `## Notes` section that needs renaming to `## Dev Notes`. S-6.06 through S-6.19 (DTU clones) are missing the section entirely.

### Group G: 13 stories — Add `## Architecture Compliance Rules`
All 13 are DTU clone stories: S-6.07 through S-6.19.

### Group H: All 75 stories — `input-hash: null` needs computation
After frontmatter remediation, run check-input-drift to compute and populate all input-hash values.

---

## Remediation Dispatch Recommendation

Per Step 2 of Post-Clear Resume Playbook — sub-bursts of ~10 stories to avoid stream idle timeouts:

| Sub-burst | Stories | Primary Work |
|-----------|---------|--------------|
| 2A | S-0.01, S-0.02, S-1.01–S-1.08 (10) | Add 6 corpus-wide FM fields; add ## Edge Cases where missing; add assumption_validations/risk_mitigations/blocks |
| 2B | S-1.09–S-1.15, S-2.01–S-2.03 (10) | Same as 2A |
| 2C | S-2.04–S-2.08, S-3.01–S-3.05 (10) | Same as 2A |
| 2D | S-3.06–S-3.13, S-4.01–S-4.02 (10) | Same as 2A |
| 2E | S-4.03–S-4.08, S-5.01–S-5.04 (10) | Same as 2A |
| 2F | S-5.05–S-5.10, S-6.01–S-6.04 (10) | Same as 2A |
| 2G | S-6.05–S-6.13 (9) | FM fields + add ## Architecture Compliance Rules + normalize ## Dev Notes |
| 2H | S-6.14–S-6.19 (6) | Same as 2G |

After all sub-bursts: state-manager commits. Then check-input-drift (Step 4).
