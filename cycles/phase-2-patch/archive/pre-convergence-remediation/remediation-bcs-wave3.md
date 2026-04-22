---
document_type: remediation-manifest
level: ops
version: "1.0"
producer: product-owner
timestamp: 2026-04-20T00:00:00
phase: pre-build-sweep
burst: pre-build-sweep
---

# Remediation Manifest — BC Template Compliance Wave 3

**Date:** 2026-04-20
**Scope:** BC-2.07.*, BC-2.08.*, BC-2.09.*, BC-2.10.*
**Reference:** `.factory/cycles/phase-2-patch/remediation-bcs-wave2.md` (established convention)
**Policy:** Rules A/C/D/E/F (all active BCs); no tombstones in scope

## Summary

| Metric | Count |
|--------|-------|
| Total files in scope | 33 |
| Files edited | 33 |
| Subsystem 2.07 files | 6 |
| Subsystem 2.08 files | 9 |
| Subsystem 2.09 files | 8 |
| Subsystem 2.10 files | 11 (including 1 that already had full modern fields) |
| Tombstone BCs (special treatment) | 0 |
| Active BCs with full remediation | 33 |
| Version 1.0 → 1.1 bumps | 19 |
| Already 1.1+: bumped to next minor | 14 |
| Files skipped | 0 |

## Files Edited by Subsystem

### Subsystem 2.07 (6 files, all active)

All files were at version ≥3.0 (rewrites); bumped to next minor:

- BC-2.07.001-ephemeral-cursor-pagination.md (3.0 → 3.1)
- BC-2.07.002-pagination-token-lifecycle.md (4.0 → 4.1)
- BC-2.07.003-response-cache-ttl.md (4.0 → 4.1)
- BC-2.07.004-cache-invalidation-on-writes.md (3.1 → 3.2; had existing Changelog from Burst 43)
- BC-2.07.005-cache-key-derivation.md (4.0 → 4.1)
- BC-2.07.006-cache-memory-bounds-eviction.md (3.0 → 3.1)

### Subsystem 2.08 (9 files, all active)

- BC-2.08.001-on-demand-connectivity-check.md (1.0 → 1.1)
- BC-2.08.002-auth-validity-check.md (1.0 → 1.1)
- BC-2.08.003-rate-limit-state-detection.md (1.0 → 1.1)
- BC-2.08.004-last-successful-query-timestamp.md (1.0 → 1.1)
- BC-2.08.005-health-mcp-tool.md (1.0 → 1.1)
- BC-2.08.006-health-mcp-resource.md (already at 1.1 with full modern fields; bumped to 1.2; Changelog row appended)
- BC-2.08.007-partial-health-status.md (1.0 → 1.1)
- BC-2.08.008-get-diagnostics-tool.md (1.0 → 1.1; phase 2-patch origin; added missing lifecycle fields)
- BC-2.08.009-diagnostic-resource-templates.md (1.0 → 1.1; phase 2-patch origin; added missing lifecycle fields)

### Subsystem 2.09 (8 files, all active)

- BC-2.09.001-structural-separation.md (1.0 → 1.1)
- BC-2.09.002-provenance-framing.md (1.0 → 1.1)
- BC-2.09.003-suspicious-pattern-detection.md (1.1 → 1.2; had prior remediation at 1.1)
- BC-2.09.004-safety-flag-parallel-fields.md (1.1 → 1.2; had prior remediation at 1.1)
- BC-2.09.005-trust-level-metadata.md (1.0 → 1.1)
- BC-2.09.006-tool-description-security-warnings.md (1.0 → 1.1)
- BC-2.09.007-output-schema-type-safety.md (1.0 → 1.1)
- BC-2.09.008-response-envelope-trust-annotations.md (1.0 → 1.1)

### Subsystem 2.10 (11 files, all active)

- BC-2.10.001-server-handler-implementation.md (1.0 → 1.1)
- BC-2.10.002-tool-registration-via-tool-router.md (2.3 → 2.4; had existing Changelog)
- BC-2.10.003-conditional-tool-registration.md (1.0 → 1.1)
- BC-2.10.004-client-id-parameter-requirement.md (2.1 → 2.2; had existing Changelog from Burst 43)
- BC-2.10.005-notifications-tools-list-changed.md (1.1 → 1.2; had prior remediation at 1.1)
- BC-2.10.006-stdio-transport.md (1.0 → 1.1)
- BC-2.10.007-structured-error-responses.md (1.0 → 1.1)
- BC-2.10.008-mcp-resources.md (1.2 → 1.3; already had full modern fields including Canonical Test Vectors and Verification Properties; Changelog row appended only)
- BC-2.10.009-mcp-prompts.md (1.0 → 1.1)
- BC-2.10.010-graceful-shutdown.md (1.0 → 1.1)
- BC-2.10.011-list-capabilities-meta-tool.md (1.0 → 1.1)

## Version Bump Summary

| Category | Count | Notes |
|----------|-------|-------|
| 1.0 → 1.1 (full remediation) | 19 | BC-2.08.001–005, 007; BC-2.09.001–002, 005–008; BC-2.10.001, 003, 006, 007, 009, 010, 011 |
| Already 1.1+: bumped to next minor | 14 | BC-2.07.001–006 (3.x/4.x → next), BC-2.08.006 (→1.2), BC-2.08.008–009 (→1.1 from 1.0 but with missing lifecycle fields), BC-2.09.003–004 (→1.2), BC-2.10.002 (→2.4), BC-2.10.004 (→2.2), BC-2.10.005 (→1.2), BC-2.10.008 (→1.3) |

## Tombstone BCs

None. All 33 files in scope are active.

## Frontmatter Changes Applied (Rule A)

All files received the following fields (if missing):
- `inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]`
- `input-hash: "[pending-recompute]"`
- `traces_to: ["CAP-NNN"]` (pointing to the capability ID in the BC body)
- `extracted_from: ".factory/specs/prd.md"`

**BC-2.08.006 and BC-2.10.008 exception:** These two files already had `inputs`, `input-hash`, `traces_to`, and `extracted_from` fields from prior targeted work (burst-45, burst-49). Their frontmatter was preserved verbatim; only the version and Changelog were updated.

**BC-2.08.008 and BC-2.08.009 anomaly:** These phase 2-patch files lacked the full lifecycle frontmatter block (`deprecated`, `deprecated_by`, `replacement`, `retired`, `removed`, `removal_reason`) present in all cycle-1 files. These were added during this sweep along with the standard Wave 3 frontmatter additions. Same pattern as BC-2.05.011 in Wave 2.

## Sections Added (Rule C)

All 33 active BCs received the following sections (if missing):

### `## Description`
Synthesized from title, Preconditions, and Postconditions for each BC. Notable synthesis judgment calls:

1. **BC-2.07.003 (Cache with Configurable TTL)** — Description names the two concrete TTL values (60s for alerts, 300s for devices) and calls out the key architectural fact that two syntactically different PrismQL queries sharing the same push-down params share one cache entry — this is the non-obvious cache coherence property that implementers most commonly miss.

2. **BC-2.07.004 (Cache Invalidation)** — Description leads with "synchronously invalidated before the write response is returned" because the ordering guarantee (not just the invalidation fact) is what enforces write-then-read consistency and is the primary implementation constraint.

3. **BC-2.08.008 (get_diagnostics)** — Already had `## Description` from phase 2-patch creation. Preserved verbatim.

4. **BC-2.08.009 (Diagnostic Resource Templates)** — Already had `## Description` from phase 2-patch creation. Preserved verbatim.

5. **BC-2.09.003 (Suspicious Pattern Detection)** — Hook fired on first Write because `## Description` was missing. Added via Edit after hook rejection. Description highlights NFKC normalization as the homoglyph-defeat mechanism, which is the security-critical property not obvious from the section title alone.

6. **BC-2.09.004 (Safety Flag Parallel Fields)** — Same hook scenario as BC-2.09.003. Description highlights the "no per-field parallel fields" constraint since this is the most common implementation mistake (adding `{field}_safety_flag` instead of centralized array).

7. **BC-2.10.004 (Client Scoping)** — Description explicitly names the `__global__` sentinel and its restricted scope (confirm_action only) because this exception to the "always specific client" rule is easy to misimplement.

### `## Canonical Test Vectors`
Placeholder tables with 3-5 scenario rows synthesized from edge cases and key postconditions. All 33 files.

### `## Verification Properties`
Cross-referenced from VP-INDEX v1.5. See VP citations table below.

### `## Changelog`
Added to all 33 files. For files that already had a Changelog, a row was appended.

## Verification Properties Cross-References Added

VPs from VP-INDEX v1.5 cited in new `## Verification Properties` sections:

| VP | BCs Referenced |
|----|----------------|
| VP-001 (TenantId rejects invalid characters) | BC-2.10.004 |
| VP-002 (Capability resolution: deny-by-default) | BC-2.10.011 |
| VP-003 (Capability resolution: most-specific-path wins) | BC-2.10.011 |
| VP-004 (Capability resolution: deny overrides allow at same specificity) | BC-2.10.011 |
| VP-020 (Feature flag: compile AND runtime must both permit) | BC-2.10.001, BC-2.10.002, BC-2.10.003, BC-2.10.005 |
| VP-024 (Injection scanner: detects known injection patterns) | BC-2.08.008, BC-2.08.009, BC-2.09.001, BC-2.09.003, BC-2.09.004, BC-2.09.007, BC-2.09.008, BC-2.10.007 |
| VP-025 (Cache key derivation: deterministic) | BC-2.07.003, BC-2.07.005 |
| VP-029 (Cursor cap: rejects at 200 active) | BC-2.07.001, BC-2.07.002 |
| VP-038 (Injection scanner: never panics on arbitrary input strings) | BC-2.08.008, BC-2.08.009, BC-2.09.003, BC-2.09.004 |
| VP-039 (Audit forward watermark: monotonically non-decreasing) | BC-2.10.010 |

BCs with no matching VP (placeholder row used): BC-2.07.004, BC-2.07.006, BC-2.08.001–007, BC-2.09.002, BC-2.09.005–006, BC-2.10.006, BC-2.10.009

## Anomalies

1. **Hook fires on `Write` when `## Description` absent (BC-2.09.003, BC-2.09.004):** The `validate-template-compliance.sh` hook rejected two Write calls because the file body was written without `## Description`. Both files had `## Description` omitted by mistake (these were Wave 1.1 files where the body was not fully rebuilt). Fixed via Edit to insert the section between H1 and Preconditions. All subsequent files were written with `## Description` first in the body.

2. **BC-2.07.004 at version 3.1 (existing Changelog from Burst 43):** Already had a Changelog section from the `P3P41-A-HIGH-001` fix (rename `set_credential` → `configure_credential_source`). A new row was appended; bumped to 3.2.

3. **BC-2.10.002 at version 2.3 (existing Changelog):** Already had a 4-row Changelog from Bursts 43/44/50. A new row was appended; bumped to 2.4.

4. **BC-2.10.004 at version 2.1 (existing Changelog from Burst 43):** New row appended; bumped to 2.2.

5. **BC-2.08.006 and BC-2.10.008 already compliant:** Both files were remediated in prior bursts (45, 49) and already had `inputs`, `input-hash`, `traces_to`, `extracted_from`, `## Description`, `## Canonical Test Vectors`, and `## Verification Properties`. Changelog row appended; version bumped only.

6. **BC-2.08.008 and BC-2.08.009 incomplete frontmatter (phase 2-patch origin):** Both lacked the full lifecycle frontmatter block (`deprecated`, `deprecated_by`, `replacement`, `retired`, `removed`, `removal_reason`) present in all cycle-1 BCs. Added during this sweep. Same pattern as BC-2.05.011 in Wave 2.

7. **No tombstones in Wave 3:** All 33 files are active. The 0-tombstone count matches Wave 2.

## Files Skipped

None. All 33 files in scope were remediated.

## Next Steps

1. **input-hash recompute:** All 33 files have `input-hash: "[pending-recompute]"` (except BC-2.08.006 and BC-2.10.008 which retain their existing `input-hash: ""`). Run `compute-input-hash` (or ask state-manager) on all files to populate actual hash values.

2. **VP-INDEX propagation:** VP citations added to BC bodies do not require VP-INDEX changes — VP-INDEX already lists these VPs; the BC-side `## Verification Properties` section is the new addition. No VP-INDEX update required from this wave.

3. **BC-INDEX verification:** No H1 headings were changed in this wave, so BC-INDEX title drift is not expected. A spot-check is recommended.
