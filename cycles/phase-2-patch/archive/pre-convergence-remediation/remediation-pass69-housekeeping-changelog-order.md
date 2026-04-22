# Housekeeping #4 (Part 1) — Changelog Row Order Normalization

**Pass:** remediation-pass69-housekeeping  
**Date:** 2026-04-20  
**Author:** story-writer  
**Scope:** All 75 story files at `.factory/stories/S-*.md`  
**Rule:** Changelog rows sorted descending by version (highest at top, 1.0 at bottom)  
**Changes:** Silent cosmetic — no new changelog row added, no version bump, no input-hash recompute

---

## Summary

| Metric | Value |
|--------|-------|
| Total files examined | 75 |
| Already sorted (no change needed) | 0 |
| Files reordered | 75 |

All 75 files required reordering. The dominant broken pattern was that `pass-60-fix` (and in some files `pass-64-fix` or `pass-63-fix`) rows were prepended correctly as the newest entry, but the remaining rows were left in original ascending order (1.0, 1.1, 1.2, ..., N-1) instead of true descending (N-1, N-2, ..., 1.1, 1.0).

---

## Per-File Report

| File | Was Already Sorted | Rows Reordered |
|------|-------------------|----------------|
| S-0.01-ci-cd-pipeline.md | no | 4 |
| S-0.02-developer-toolchain.md | no | 2 |
| S-1.01-foundational-types.md | no | 2 |
| S-1.02-entity-types.md | no | 4 |
| S-1.03-capability-resolution.md | no | 4 |
| S-1.04-ocsf-schema-loading.md | no | 4 |
| S-1.05-ocsf-field-mapping.md | no | 4 |
| S-1.06-credential-store.md | no | 2 |
| S-1.07-credential-crud.md | no | 4 |
| S-1.08-feature-flags.md | no | 2 |
| S-1.09-confirmation-tokens.md | no | 2 |
| S-1.10-prompt-injection-defense.md | no | 2 |
| S-1.11-spec-loading.md | no | 2 |
| S-1.12-hot-reload.md | no | 2 |
| S-1.13-sensor-write-specs.md | no | 2 |
| S-1.14-infusion-specs.md | no | 4 |
| S-1.15-wasm-runtime.md | no | 4 |
| S-2.01-rocksdb-init.md | no | 4 |
| S-2.02-audit-buffer-watchdog.md | no | 2 |
| S-2.03-decorators-internal-tables.md | no | 2 |
| S-2.04-audit-construction.md | no | 4 |
| S-2.05-audit-events.md | no | 2 |
| S-2.06-datasource-trait.md | no | 4 |
| S-2.07-per-sensor-auth.md | no | 2 |
| S-2.08-event-tables.md | no | 2 |
| S-3.01-prismql-parser.md | no | 2 |
| S-3.02-query-materialization.md | no | 4 |
| S-3.03-explain-query.md | no | 4 |
| S-3.04-alias-system.md | no | 4 |
| S-3.05-pagination-caching.md | no | 4 |
| S-3.06-prismql-write-parser.md | no | 4 |
| S-3.07-write-execution.md | no | 4 |
| S-3.08-hidden-columns.md | no | 4 |
| S-3.09-query-profiling.md | no | 4 |
| S-3.10-cost-estimation.md | no | 4 |
| S-3.11-in-query-caching.md | no | 4 |
| S-3.12-column-pruning.md | no | 4 |
| S-3.13-dynamic-table-availability.md | no | 6 |
| S-4.01-schedule-crud.md | no | 6 |
| S-4.02-diff-results-packs.md | no | 4 |
| S-4.03-detection-rules.md | no | 4 |
| S-4.04-detection-evaluation.md | no | 4 |
| S-4.05-alert-generation.md | no | 4 |
| S-4.06-case-management.md | no | 4 |
| S-4.07-case-metrics.md | no | 4 |
| S-4.08-action-delivery.md | no | 4 |
| S-5.01-mcp-bootstrap.md | no | 8 |
| S-5.02-tool-routing.md | no | 4 |
| S-5.03-resources-prompts.md | no | 6 |
| S-5.04-sensor-health.md | no | 4 |
| S-5.05-config-loading.md | no | 4 |
| S-5.06-action-infusion-tools.md | no | 8 |
| S-5.07-multi-repo-git-config.md | no | 4 |
| S-5.08-diagnostics-logs-cli.md | no | 2 |
| S-5.09-external-log-forwarding.md | no | 4 |
| S-5.10-audit-trail-forwarding.md | no | 4 |
| S-6.01-cli-startup.md | no | 4 |
| S-6.02-e2e-smoke-tests.md | no | 4 |
| S-6.03-installation.md | no | 4 |
| S-6.04-credential-cli.md | no | 4 |
| S-6.05-migrate-storage.md | no | 4 |
| S-6.06-dtu-common.md | no | 4 |
| S-6.07-dtu-crowdstrike.md | no | 4 |
| S-6.08-dtu-claroty.md | no | 6 |
| S-6.09-dtu-cyberint.md | no | 6 |
| S-6.10-dtu-armis.md | no | 6 |
| S-6.11-dtu-slack.md | no | 6 |
| S-6.12-dtu-pagerduty.md | no | 6 |
| S-6.13-dtu-jira.md | no | 6 |
| S-6.14-dtu-threatintel.md | no | 6 |
| S-6.15-dtu-nvd.md | no | 6 |
| S-6.16-dtu-datadog.md | no | 6 |
| S-6.17-dtu-splunk-hec.md | no | 6 |
| S-6.18-dtu-elasticsearch.md | no | 6 |
| S-6.19-dtu-otlp.md | no | 6 |
