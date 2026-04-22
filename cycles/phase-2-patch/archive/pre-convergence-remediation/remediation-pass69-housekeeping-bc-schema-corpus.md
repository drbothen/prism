---
document_type: remediation-manifest
phase: 2-patch
pass: pass-69-housekeeping
burst: Burst 2B extension
date: 2026-04-20
author: product-owner
topic: BC changelog schema normalization — corpus-wide
---

# Remediation Manifest: BC Changelog Schema Corpus Normalization (pass-69-housekeeping)

## Summary

| Metric | Count |
|--------|-------|
| Total BC files scanned | 203 |
| Already canonical (no change needed) | 69 |
| Converted to canonical | 134 |
| Post-conversion canonical total | 203 |
| Unknown / unresolvable schemas | 0 |
| Anomalies requiring manual fix | 1 |

Canonical schema adopted: `| Version | Burst | Date | Author | Change |`

## Conversion Breakdown by Source Schema Variant

| Source Schema | Count | Transformation Applied |
|---------------|-------|------------------------|
| `Version \| Burst \| Date \| Author \| Changes` (plural) | 60 | Header column renamed `Changes` → `Change`; rows preserved as-is |
| `Version \| Date \| Burst \| Change` (4-col, no Author) | 42 | Columns reordered Date↔Burst; Author column added with value `product-owner` inferred |
| `Version \| Date \| Burst \| Author \| Change` (5-col wrong order) | 30 | Columns reordered: Date↔Burst swap only |
| `Version \| Burst \| Finding \| Change` (anomalous 4-col) | 2 | `Finding` column renamed `Author`; `Date` column added with value `—` |

## Version Bump Protocol

For all 134 converted files:
- Frontmatter `version:` field bumped by minor (+0.1)
- New Changelog row prepended at TOP with:
  `| {new_version} | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col Version | Burst | Date | Author | Change form. |`

## Anomalies

### BC-2.12.011: Row 1.4 had embedded pipe characters in Finding column value

**Issue:** Row `| 1.4 | pass-63-fix | P3P63-A-MED-001 | Aligned...schema (Version | Burst | Finding | Change)...`
contained pipe characters both in the Finding value used as Author AND in the change description. The automated
script's `split("|")` produced more than 4 cells, causing the `len(cells) == 4` guard to fall through and
preserve the original unmodified 4-col row (missing Date `—` column).

**Manual fix:** Row corrected post-script to:
`| 1.4 | pass-63-fix | — | P3P63-A-MED-001 | Aligned...schema (Version \| Burst \| Finding \| Change)...`
Embedded pipes in change description escaped with `\|`.

**BC-2.12.012:** No anomaly — all rows had clean data without embedded pipes.

## Per-File Manifest

### Already Canonical (69 files — no change)

BC-2.04.001 through BC-2.04.015 (15 files), BC-2.05.001 through BC-2.05.011 (11 files),
BC-2.06.001 through BC-2.06.010 (10 files), BC-2.14.001 through BC-2.14.013 (13 files,
excluding BC-2.14.011 which does not exist), BC-2.15.001 through BC-2.15.011 (11 files),
BC-2.16.001 through BC-2.16.010 (10 files).

### Converted: Changes_plural → Change (60 files)

| File | Source Schema | Action |
|------|---------------|--------|
| BC-2.01.001-single-client-sensor-query.md | Changes_plural | Header renamed; new row v2.4 |
| BC-2.01.002-cross-client-fan-out.md | Changes_plural | Header renamed; new row |
| BC-2.01.003-cursor-based-pagination.md | Changes_plural | Header renamed; new row |
| BC-2.01.004-offset-based-pagination-claroty.md | Changes_plural | Header renamed; new row |
| BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md | Changes_plural | Header renamed; new row |
| BC-2.01.006-cyberint-cookie-auth.md | Changes_plural | Header renamed; new row |
| BC-2.01.007-claroty-bearer-polymorphic-ids.md | Changes_plural | Header renamed; new row |
| BC-2.01.008-armis-bearer-aql.md | Changes_plural | Header renamed; new row |
| BC-2.01.009-query-filtering-sorting.md | Changes_plural | Header renamed; new row |
| BC-2.01.010-partial-failure-handling.md | Changes_plural | Header renamed; new row |
| BC-2.01.011-cross-sensor-correlation-ocsf-fields.md | Changes_plural | Header renamed; new row |
| BC-2.01.012-query-fingerprint-validation.md | Changes_plural | Header renamed; new row |
| BC-2.01.013-datasource-trait-adapter-pattern.md | Changes_plural | Header renamed; new row |
| BC-2.01.014-sensor-api-http-503-mid-pagination.md | Changes_plural | Header renamed; new row |
| BC-2.01.015-response-envelope-structure.md | Changes_plural | Header renamed; new row |
| BC-2.02.001-ocsf-schema-build-time-loading.md | Changes_plural | Header renamed; new row |
| BC-2.02.002-dynamic-message-creation.md | Changes_plural | Header renamed; new row |
| BC-2.02.003-crowdstrike-field-mapping.md | Changes_plural | Header renamed; new row |
| BC-2.02.004-cyberint-field-mapping.md | Changes_plural | Header renamed; new row |
| BC-2.02.005-claroty-field-mapping.md | Changes_plural | Header renamed; new row |
| BC-2.02.006-armis-field-mapping.md | Changes_plural | Header renamed; new row |
| BC-2.02.007-raw-extensions-preservation.md | Changes_plural | Header renamed; new row |
| BC-2.02.008-field-alias-resolution.md | Changes_plural | Header renamed; new row |
| BC-2.02.009-ocsf-version-pinning.md | Changes_plural | Header renamed; new row |
| BC-2.02.010-enum-value-map-runtime-lookup.md | Changes_plural | Header renamed; new row |
| BC-2.02.011-normalization-error-handling.md | Changes_plural | Header renamed; new row |
| BC-2.02.012-ocsf-event-class-selection.md | Changes_plural | Header renamed; new row |
| BC-2.03.001-credential-store-trait.md | Changes_plural | Header renamed; new row |
| BC-2.03.002-keyring-backend.md | Changes_plural | Header renamed; new row |
| BC-2.03.003-encrypted-file-fallback.md | Changes_plural | Header renamed; new row |
| BC-2.03.004-namespace-isolation.md | Changes_plural | Header renamed; new row |
| BC-2.03.006-credential-resolution-at-query-time.md | Changes_plural | Header renamed; new row |
| BC-2.03.007-secret-redaction.md | Changes_plural | Header renamed; new row |
| BC-2.03.008-credential-name-sanitization.md | Changes_plural | Header renamed; new row |
| BC-2.03.009-resolve-secret-env-file.md | Changes_plural | Header renamed; new row |
| BC-2.03.010-credential-access-audit-logging.md | Changes_plural | Header renamed; new row |
| BC-2.03.011-keyring-startup-probe.md | Changes_plural | Header renamed; new row |
| BC-2.03.012-backend-selection-fallback.md | Changes_plural | Header renamed; new row |
| BC-2.08.006-health-mcp-resource.md | Changes_plural | Header renamed; new row |
| BC-2.10.008-mcp-resources.md | Changes_plural | Header renamed; new row |
| BC-2.17.001-plugin-panic-isolation.md | Changes_plural | Header renamed; new row |
| BC-2.17.002-plugin-sandbox-filesystem.md | Changes_plural | Header renamed; new row |
| BC-2.17.003-plugin-memory-limit.md | Changes_plural | Header renamed; new row |
| BC-2.17.004-plugin-cpu-time-limit.md | Changes_plural | Header renamed; new row |
| BC-2.17.005-plugin-hot-reload-atomic-swap.md | Changes_plural | Header renamed; new row |
| BC-2.17.006-plugin-wit-validation.md | Changes_plural | Header renamed; new row |
| BC-2.18.001-action-at-least-once-delivery.md | Changes_plural | Header renamed; new row |
| BC-2.18.002-action-schedule-best-effort.md | Changes_plural | Header renamed; new row |
| BC-2.18.003-action-manual-fire-and-forget.md | Changes_plural | Header renamed; new row |
| BC-2.18.004-action-schedule-semaphore.md | Changes_plural | Header renamed; new row |
| BC-2.18.005-action-partial-report-failure.md | Changes_plural | Header renamed; new row |
| BC-2.18.006-action-template-injection-scan.md | Changes_plural | Header renamed; new row |
| BC-2.18.007-action-credential-opaque-reference.md | Changes_plural | Header renamed; new row |
| BC-2.18.008-action-delivery-audit-logging.md | Changes_plural | Header renamed; new row |
| BC-2.18.009-action-uuid-v7-validation.md | Changes_plural | Header renamed; new row |
| BC-2.19.001-infusion-spec-loading.md | Changes_plural | Header renamed; new row |
| BC-2.19.002-infusion-per-query-dedup.md | Changes_plural | Header renamed; new row |
| BC-2.19.003-infusion-api-backed-rejection.md | Changes_plural | Header renamed; new row |
| BC-2.19.004-infusion-hot-reload-atomicity.md | Changes_plural | Header renamed; new row |
| BC-2.19.005-infusion-credential-redaction.md | Changes_plural | Header renamed; new row |

### Converted: Version|Date|Burst|Change (no Author) → canonical (42 files)

| File | Source Schema | Action |
|------|---------------|--------|
| BC-2.03.005-credential-crud-operations.md | Version_Date_Burst_Change_noAuthor | Reordered Date↔Burst; Author added (product-owner); new row |
| BC-2.10.002-tool-registration-via-tool-router.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.10.004-client-id-parameter-requirement.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.001-query-mcp-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.002-prismql-filter-mode.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.003-prismql-sql-mode.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.004-prismql-pipe-mode.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.005-ephemeral-materialization.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.006-query-security-limits.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.007-sensor-filter-push-down.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.008-create-alias-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.009-alias-resolution.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.010-explain-query-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.011-cross-client-query-scoping.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.012-virtual-fields.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.013-list-aliases-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.014-delete-alias-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.11.015-explain-alias-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.001-create-schedule-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.002-list-schedules-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.003-delete-schedule-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.004-schedule-execution-loop.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.005-differential-result-computation.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.006-epoch-counter-tracking.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.007-get-diff-results-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.008-pack-loading-discovery.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.009-pack-crud-tools.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.12.010-schedule-state-persistence.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.001-detection-rule-loading.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.002-single-event-detection.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.003-correlation-detection.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.004-sequence-detection.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.005-alert-generation.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.006-create-rule-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.007-list-rules-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.008-delete-rule-tool.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.009-rule-to-sql-compilation.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.010-security-udf-registration.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.011-three-scope-rule-resolution.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.012-detection-state-persistence.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.013-alert-deduplication.md | Version_Date_Burst_Change_noAuthor | Same |
| BC-2.13.014-ioc-file-loading-pattern-store.md | Version_Date_Burst_Change_noAuthor | Same |

### Converted: Version|Date|Burst|Author|Change (wrong order) → canonical (30 files)

| File | Source Schema | Action |
|------|---------------|--------|
| BC-2.07.001-ephemeral-cursor-pagination.md | Version_Date_Burst_Author_Change | Date↔Burst swap; new row |
| BC-2.07.002-pagination-token-lifecycle.md | Version_Date_Burst_Author_Change | Same |
| BC-2.07.003-response-cache-ttl.md | Version_Date_Burst_Author_Change | Same |
| BC-2.07.004-cache-invalidation-on-writes.md | Version_Date_Burst_Author_Change | Same |
| BC-2.07.005-cache-key-derivation.md | Version_Date_Burst_Author_Change | Same |
| BC-2.07.006-cache-memory-bounds-eviction.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.001-on-demand-connectivity-check.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.002-auth-validity-check.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.003-rate-limit-state-detection.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.004-last-successful-query-timestamp.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.005-health-mcp-tool.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.007-partial-health-status.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.008-get-diagnostics-tool.md | Version_Date_Burst_Author_Change | Same |
| BC-2.08.009-diagnostic-resource-templates.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.001-structural-separation.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.002-provenance-framing.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.003-suspicious-pattern-detection.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.004-safety-flag-parallel-fields.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.005-trust-level-metadata.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.006-tool-description-security-warnings.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.007-output-schema-type-safety.md | Version_Date_Burst_Author_Change | Same |
| BC-2.09.008-response-envelope-trust-annotations.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.001-server-handler-implementation.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.003-conditional-tool-registration.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.005-notifications-tools-list-changed.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.006-stdio-transport.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.007-structured-error-responses.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.009-mcp-prompts.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.010-graceful-shutdown.md | Version_Date_Burst_Author_Change | Same |
| BC-2.10.011-list-capabilities-meta-tool.md | Version_Date_Burst_Author_Change | Same |

### Converted: Version|Burst|Finding|Change (anomalous) → canonical (2 files)

| File | Source Schema | Action | Anomaly |
|------|---------------|--------|---------|
| BC-2.12.011-action-at-least-once-delivery.md | Version_Burst_Finding_Change | Finding→Author; Date=— added; new row | Row 1.4 had embedded pipes; manual fix required post-script. Pipes in change text escaped with \| |
| BC-2.12.012-action-template-injection-scanning.md | Version_Burst_Finding_Change | Finding→Author; Date=— added; new row | Clean — no anomaly |

## Verification

Post-normalization audit run: `203 canonical / 0 non-canonical` across all BC-*.md files (excluding BC-INDEX.md).
