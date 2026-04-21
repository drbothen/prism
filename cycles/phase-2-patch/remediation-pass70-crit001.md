---
document_type: remediation-manifest
pass: pass-70-fix
issue: CRIT-001
date: 2026-04-20
author: product-owner
files_fixed: 134
---

# CRIT-001 Remediation Manifest — pass-70-fix

## Summary

Fixed 134 BC files containing a malformed changelog row introduced by
pass-69-housekeeping. The row contained unescaped pipe characters in the
description field, causing markdown table parsers to render it as a
10-column row against a 5-column header.

This was a silent edit (content-equivalent): no version bump, no new changelog
row. The schema normalization content itself is unchanged; only the description
text representation was corrected.

## Before / After

**Before (malformed — 10 cells):**
```
| 2.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col Version | Burst | Date | Author | Change form. |
```

**After (corrected — 5 cells):**
```
| 2.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
```

Note: version number varies per file (e.g., 1.2, 1.4, 2.3, 2.4, 3.2, 4.2, etc.) and
was preserved via regex capture group.

## Files Fixed (134 total)

BC-2.01.001-single-client-sensor-query.md
BC-2.01.002-cross-client-fan-out.md
BC-2.01.003-cursor-based-pagination.md
BC-2.01.004-offset-based-pagination-claroty.md
BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md
BC-2.01.006-cyberint-cookie-auth.md
BC-2.01.007-claroty-bearer-polymorphic-ids.md
BC-2.01.008-armis-bearer-aql.md
BC-2.01.009-query-filtering-sorting.md
BC-2.01.010-partial-failure-handling.md
BC-2.01.011-cross-sensor-correlation-ocsf-fields.md
BC-2.01.012-query-fingerprint-validation.md
BC-2.01.013-datasource-trait-adapter-pattern.md
BC-2.01.014-sensor-api-http-503-mid-pagination.md
BC-2.01.015-response-envelope-structure.md
BC-2.02.001-ocsf-schema-build-time-loading.md
BC-2.02.002-dynamic-message-creation.md
BC-2.02.003-crowdstrike-field-mapping.md
BC-2.02.004-cyberint-field-mapping.md
BC-2.02.005-claroty-field-mapping.md
BC-2.02.006-armis-field-mapping.md
BC-2.02.007-raw-extensions-preservation.md
BC-2.02.008-field-alias-resolution.md
BC-2.02.009-ocsf-version-pinning.md
BC-2.02.010-enum-value-map-runtime-lookup.md
BC-2.02.011-normalization-error-handling.md
BC-2.02.012-ocsf-event-class-selection.md
BC-2.03.001-credential-store-trait.md
BC-2.03.002-keyring-backend.md
BC-2.03.003-encrypted-file-fallback.md
BC-2.03.004-namespace-isolation.md
BC-2.03.005-credential-crud-operations.md
BC-2.03.006-credential-resolution-at-query-time.md
BC-2.03.007-secret-redaction.md
BC-2.03.008-credential-name-sanitization.md
BC-2.03.009-resolve-secret-env-file.md
BC-2.03.010-credential-access-audit-logging.md
BC-2.03.011-keyring-startup-probe.md
BC-2.03.012-backend-selection-fallback.md
BC-2.07.001-ephemeral-cursor-pagination.md
BC-2.07.002-pagination-token-lifecycle.md
BC-2.07.003-response-cache-ttl.md
BC-2.07.004-cache-invalidation-on-writes.md
BC-2.07.005-cache-key-derivation.md
BC-2.07.006-cache-memory-bounds-eviction.md
BC-2.08.001-on-demand-connectivity-check.md
BC-2.08.002-auth-validity-check.md
BC-2.08.003-rate-limit-state-detection.md
BC-2.08.004-last-successful-query-timestamp.md
BC-2.08.005-health-mcp-tool.md
BC-2.08.006-health-mcp-resource.md
BC-2.08.007-partial-health-status.md
BC-2.08.008-get-diagnostics-tool.md
BC-2.08.009-diagnostic-resource-templates.md
BC-2.09.001-structural-separation.md
BC-2.09.002-provenance-framing.md
BC-2.09.003-suspicious-pattern-detection.md
BC-2.09.004-safety-flag-parallel-fields.md
BC-2.09.005-trust-level-metadata.md
BC-2.09.006-tool-description-security-warnings.md
BC-2.09.007-output-schema-type-safety.md
BC-2.09.008-response-envelope-trust-annotations.md
BC-2.10.001-server-handler-implementation.md
BC-2.10.002-tool-registration-via-tool-router.md
BC-2.10.003-conditional-tool-registration.md
BC-2.10.004-client-id-parameter-requirement.md
BC-2.10.005-notifications-tools-list-changed.md
BC-2.10.006-stdio-transport.md
BC-2.10.007-structured-error-responses.md
BC-2.10.008-mcp-resources.md
BC-2.10.009-mcp-prompts.md
BC-2.10.010-graceful-shutdown.md
BC-2.10.011-list-capabilities-meta-tool.md
BC-2.11.001-query-mcp-tool.md
BC-2.11.002-prismql-filter-mode.md
BC-2.11.003-prismql-sql-mode.md
BC-2.11.004-prismql-pipe-mode.md
BC-2.11.005-ephemeral-materialization.md
BC-2.11.006-query-security-limits.md
BC-2.11.007-sensor-filter-push-down.md
BC-2.11.008-create-alias-tool.md
BC-2.11.009-alias-resolution.md
BC-2.11.010-explain-query-tool.md
BC-2.11.011-cross-client-query-scoping.md
BC-2.11.012-virtual-fields.md
BC-2.11.013-list-aliases-tool.md
BC-2.11.014-delete-alias-tool.md
BC-2.11.015-explain-alias-tool.md
BC-2.12.001-create-schedule-tool.md
BC-2.12.002-list-schedules-tool.md
BC-2.12.003-delete-schedule-tool.md
BC-2.12.004-schedule-execution-loop.md
BC-2.12.005-differential-result-computation.md
BC-2.12.006-epoch-counter-tracking.md
BC-2.12.007-get-diff-results-tool.md
BC-2.12.008-pack-loading-discovery.md
BC-2.12.009-pack-crud-tools.md
BC-2.12.010-schedule-state-persistence.md
BC-2.12.011-action-at-least-once-delivery.md
BC-2.12.012-action-template-injection-scanning.md
BC-2.13.001-detection-rule-loading.md
BC-2.13.002-single-event-detection.md
BC-2.13.003-correlation-detection.md
BC-2.13.004-sequence-detection.md
BC-2.13.005-alert-generation.md
BC-2.13.006-create-rule-tool.md
BC-2.13.007-list-rules-tool.md
BC-2.13.008-delete-rule-tool.md
BC-2.13.009-rule-to-sql-compilation.md
BC-2.13.010-security-udf-registration.md
BC-2.13.011-three-scope-rule-resolution.md
BC-2.13.012-detection-state-persistence.md
BC-2.13.013-alert-deduplication.md
BC-2.13.014-ioc-file-loading-pattern-store.md
BC-2.17.001-plugin-panic-isolation.md
BC-2.17.002-plugin-sandbox-filesystem.md
BC-2.17.003-plugin-memory-limit.md
BC-2.17.004-plugin-cpu-time-limit.md
BC-2.17.005-plugin-hot-reload-atomic-swap.md
BC-2.17.006-plugin-wit-validation.md
BC-2.18.001-action-at-least-once-delivery.md
BC-2.18.002-action-schedule-best-effort.md
BC-2.18.003-action-manual-fire-and-forget.md
BC-2.18.004-action-schedule-semaphore.md
BC-2.18.005-action-partial-report-failure.md
BC-2.18.006-action-template-injection-scan.md
BC-2.18.007-action-credential-opaque-reference.md
BC-2.18.008-action-delivery-audit-logging.md
BC-2.18.009-action-uuid-v7-validation.md
BC-2.19.001-infusion-spec-loading.md
BC-2.19.002-infusion-per-query-dedup.md
BC-2.19.003-infusion-api-backed-rejection.md
BC-2.19.004-infusion-hot-reload-atomicity.md
BC-2.19.005-infusion-credential-redaction.md

## Sanity Check

- Files with old malformed pattern remaining: 0
- Files with corrected pattern confirmed: 134
- Files where pattern was not found (unexpected): 0

## Version Numbers Observed

The version field in the malformed row was not `1.X` uniformly — it varied
per file based on each BC's changelog history at the time of pass-69:
1.2, 1.3, 1.4, 1.5, 1.6, 2.3, 2.4, 2.6, 3.2, 3.3, 4.2

All were preserved in place; only the description text was corrected.
