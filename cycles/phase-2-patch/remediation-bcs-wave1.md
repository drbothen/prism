---
document_type: remediation-manifest
level: ops
version: "1.0"
producer: product-owner
timestamp: 2026-04-20T00:00:00
phase: pre-build-sweep
burst: pre-build-sweep
---

# Remediation Manifest — BC Template Compliance Wave 1

**Date:** 2026-04-20
**Scope:** BC-2.01.*, BC-2.02.*, BC-2.03.*
**Reference:** `.factory/cycles/phase-2-patch/template-audit-bcs.md`
**Policy:** Rule B (tombstones), Rules A/C/D/E/F (active BCs)

## Summary

| Metric | Count |
|--------|-------|
| Total files in scope | 39 |
| Files edited | 39 |
| Subsystem 2.01 files | 15 |
| Subsystem 2.02 files | 12 |
| Subsystem 2.03 files | 12 |
| Tombstone BCs (special treatment) | 6 |
| Active BCs with full remediation | 33 |
| Version 1.0 → 1.1 bumps | 28 |
| Files at 1.1+ or 2.0 (no bump needed) | 11 |
| Files skipped | 0 |

## Files Edited by Subsystem

### Subsystem 2.01 (15 files)

**Tombstones (6):**
- BC-2.01.001-single-client-sensor-query.md
- BC-2.01.003-cursor-based-pagination.md
- BC-2.01.009-query-filtering-sorting.md
- BC-2.01.011-cross-sensor-correlation-ocsf-fields.md
- BC-2.01.012-query-fingerprint-validation.md
- BC-2.01.015-response-envelope-structure.md

**Active (9):**
- BC-2.01.002-cross-client-fan-out.md (was v2.0; no bump; changelog row added)
- BC-2.01.004-offset-based-pagination-claroty.md (1.0 → 1.1)
- BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md (1.0 → 1.1)
- BC-2.01.006-cyberint-cookie-auth.md (1.0 → 1.1)
- BC-2.01.007-claroty-bearer-polymorphic-ids.md (1.0 → 1.1)
- BC-2.01.008-armis-bearer-aql.md (1.0 → 1.1)
- BC-2.01.010-partial-failure-handling.md (1.0 → 1.1)
- BC-2.01.013-datasource-trait-adapter-pattern.md (1.0 → 1.1)
- BC-2.01.014-sensor-api-http-503-mid-pagination.md (1.0 → 1.1)

### Subsystem 2.02 (12 files, all active)

All 12 files: 1.0 → 1.1
- BC-2.02.001-ocsf-schema-build-time-loading.md
- BC-2.02.002-dynamic-message-creation.md
- BC-2.02.003-crowdstrike-field-mapping.md
- BC-2.02.004-cyberint-field-mapping.md
- BC-2.02.005-claroty-field-mapping.md
- BC-2.02.006-armis-field-mapping.md
- BC-2.02.007-raw-extensions-preservation.md
- BC-2.02.008-field-alias-resolution.md
- BC-2.02.009-ocsf-version-pinning.md
- BC-2.02.010-enum-value-map-runtime-lookup.md
- BC-2.02.011-normalization-error-handling.md
- BC-2.02.012-ocsf-event-class-selection.md

### Subsystem 2.03 (12 files, all active)

- BC-2.03.001-credential-store-trait.md (1.0 → 1.1)
- BC-2.03.002-keyring-backend.md (1.0 → 1.1)
- BC-2.03.003-encrypted-file-fallback.md (was 1.1; no version bump; changelog row appended)
- BC-2.03.004-namespace-isolation.md (1.0 → 1.1)
- BC-2.03.005-credential-crud-operations.md (was 1.2; no version bump; changelog row appended)
- BC-2.03.006-credential-resolution-at-query-time.md (1.0 → 1.1)
- BC-2.03.007-secret-redaction.md (1.0 → 1.1)
- BC-2.03.008-credential-name-sanitization.md (1.0 → 1.1)
- BC-2.03.009-resolve-secret-env-file.md (1.0 → 1.1)
- BC-2.03.010-credential-access-audit-logging.md (1.0 → 1.1)
- BC-2.03.011-keyring-startup-probe.md (1.0 → 1.1)
- BC-2.03.012-backend-selection-fallback.md (1.0 → 1.1)

## Version Bump Summary

| Category | Count | Files |
|----------|-------|-------|
| 1.0 → 1.1 (active BCs with full remediation) | 28 | See subsystem tables above |
| Already 1.1+ or 2.0 (changelog row appended only) | 11 | BC-2.01.002 (v2.0), BC-2.01.003 (v2.0 tombstone), BC-2.01.001 (v2.0 tombstone), BC-2.01.009 (v2.0 tombstone), BC-2.01.011 (v2.0 tombstone), BC-2.01.012 (v2.0 tombstone), BC-2.01.015 (v2.0 tombstone), BC-2.03.003 (v1.1), BC-2.03.005 (v1.2) |

## Tombstone BCs — Special Treatment

Per Rule B, tombstones received minimal treatment:
- Added `inputs`, `input-hash: "[pending-recompute]"`, `traces_to: []`, `extracted_from: "[tombstone]"` frontmatter
- Added `removal_reason` to frontmatter where previously null
- Added required template stub sections (`## Preconditions`, `## Postconditions`, `## Invariants`, `## Edge Cases`, `## Canonical Test Vectors`, `## Verification Properties`, `## Traceability`, `## Changelog`) with tombstone notices
- Existing `## Description` and tombstone notice body preserved verbatim

**Hook compliance note:** The `validate-template-compliance.sh` hook checks all `behavioral-contract` files against the template and requires H2 sections regardless of `lifecycle_status: removed`. Tombstone stubs were added to satisfy the hook while preserving the minimal-treatment principle — stub content explicitly notes "Tombstone — no preconditions/postconditions/etc. apply."

## Description Synthesis — Judgment Calls

The following active BCs required multi-sentence synthesis judgment because the existing body contained dense or multi-faceted preconditions/postconditions:

1. **BC-2.01.005 (CrowdStrike OAuth2)** — Two-step fetch pattern (QueryV2 + PostEntities batching) is architecturally significant; description was crafted to highlight the per-page HTTP call count implication for latency budgets, which is noted in Postconditions but could be missed by readers skimming.

2. **BC-2.02.001 (OCSF schema build-time loading)** — Compressed the "why build-time" rationale (no runtime network dependency, schema consistency per release) into the description, as this is the primary value proposition of this approach vs. runtime schema loading.

3. **BC-2.02.008 (Four-tier field alias resolution)** — The four-tier priority system needed a concise description that named all tiers and the determinism invariant, since this is a load-bearing resolution algorithm used across the query engine.

4. **BC-2.03.003 (AES-256-GCM encrypted file)** — Key derivation details (HKDF-SHA256, 32-byte salt, 96-bit nonce, file format layout) are complex; description summarized the security-relevant properties at a level useful for reviewers without duplicating the postconditions verbatim.

5. **BC-2.03.005 (Credential CRUD operations)** — The create-vs-update asymmetry (create is immediate; update requires confirmation token) is the critical behavioral distinction; description was written to make this asymmetry immediately apparent.

## Verification Properties Cross-References Added

VPs from VP-INDEX v1.5 cited in new `## Verification Properties` sections:

| VP | BCs Referenced |
|----|----------------|
| VP-011 (credential name sanitization) | BC-2.03.001, BC-2.03.002, BC-2.03.004, BC-2.03.005, BC-2.03.006, BC-2.03.007, BC-2.03.008 |
| VP-016 (OCSF normalization: valid protobuf) | BC-2.02.001, BC-2.02.002, BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006, BC-2.02.009, BC-2.02.011, BC-2.02.012 |
| VP-017 (OCSF normalization: unmapped fields) | BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006, BC-2.02.007 |
| VP-022 (OCSF normalizer: no panics) | BC-2.02.001, BC-2.02.002, BC-2.02.011 |
| VP-034 (encryption round-trip) | BC-2.03.001, BC-2.03.003 |
| VP-035 (key derivation deterministic) | BC-2.03.003 |

BCs with no matching VP (placeholder row used): BC-2.01.002, BC-2.01.004, BC-2.01.006, BC-2.01.007, BC-2.01.008, BC-2.01.010, BC-2.01.013, BC-2.01.014, BC-2.02.008, BC-2.02.010, BC-2.03.009, BC-2.03.010, BC-2.03.011, BC-2.03.012

## Anomalies

1. **Hook fires on Edit (not Write only):** The `validate-template-compliance.sh` hook fires on every `Edit` tool call, not just `Write`. This caused a false-positive warning when the frontmatter was updated before stub sections were added on BC-2.01.001. Subsequent tombstones were remediated using single `Write` calls to avoid multi-step hook triggers.

2. **BC-2.01.002 version 2.0 (active):** BC-2.01.002 is version "2.0" but `lifecycle_status: active` — it is a rewrite (not a tombstone). No version bump applied; changelog row appended only per Rule D.

3. **BC-2.03.004 EC numbering collision:** BC-2.03.004 has edge case `EC-03-009` which collides with an EC number also used in BC-2.03.003. Both files were preserved as-is per Rule F (do not alter existing content beyond missing structure). This pre-existing numbering collision is noted for future EC-index consolidation.

4. **No tombstones in 2.02 or 2.03:** All 12 files in 2.02 and all 12 files in 2.03 are active. The tombstone count (6) is entirely within 2.01.

## Files Skipped

None. All 39 files in scope were remediated.

## Next Steps

1. **input-hash recompute:** All files have `input-hash: "[pending-recompute]"`. Run `compute-input-hash` (or ask state-manager) on all 39 files to populate actual hash values.
2. **VP-INDEX propagation:** VP citations added to BC bodies do not require VP-INDEX changes — VP-INDEX already lists these VPs; the BC-side Verification Properties section is the new addition. No VP-INDEX update required from this wave.
3. **BC-INDEX update:** BC-INDEX titles should be verified against the H1 headings of these files (no H1 changes were made, so drift is unlikely, but a spot-check is recommended).
