---
document_type: remediation-manifest
level: ops
version: "1.0"
producer: product-owner
timestamp: 2026-04-20T00:00:00
phase: pre-build-sweep
burst: pre-build-sweep
---

# Remediation Manifest — BC Template Compliance Wave 2

**Date:** 2026-04-20
**Scope:** BC-2.04.*, BC-2.05.*, BC-2.06.*
**Reference:** `.factory/cycles/phase-2-patch/remediation-bcs-wave1.md` (established convention)
**Policy:** Rules A/C/D/E/F (all active BCs)

## Summary

| Metric | Count |
|--------|-------|
| Total files in scope | 36 |
| Files edited | 36 |
| Subsystem 2.04 files | 15 |
| Subsystem 2.05 files | 11 |
| Subsystem 2.06 files | 10 |
| Tombstone BCs (special treatment) | 0 |
| Active BCs with full remediation | 36 |
| Version 1.0 → 1.1 bumps | 24 |
| Files at 1.1+ or 2.0 bumped to next minor | 12 |
| Files skipped | 0 |

## Files Edited by Subsystem

### Subsystem 2.04 (15 files, all active)

- BC-2.04.001-compile-time-cargo-features.md (1.0 → 1.1)
- BC-2.04.002-runtime-per-client-toml-flags.md (1.0 → 1.1)
- BC-2.04.003-hierarchical-flag-resolution.md (was 1.1; bumped to 1.2; changelog row appended)
- BC-2.04.004-two-tier-gate-both-must-pass.md (1.0 → 1.1)
- BC-2.04.005-hidden-tools-pattern.md (was 1.2; bumped to 1.3; changelog row appended)
- BC-2.04.006-list-capabilities-meta-tool.md (1.0 → 1.1)
- BC-2.04.007-three-tier-risk-classification.md (was 1.1; bumped to 1.2; changelog row appended)
- BC-2.04.008-dry-run-default-reversible-writes.md (1.0 → 1.1)
- BC-2.04.009-confirmation-token-request.md (was 1.2; bumped to 1.3; changelog row appended)
- BC-2.04.010-confirmation-token-consumption.md (was 2.0; bumped to 2.1; changelog row appended)
- BC-2.04.011-token-expiry-300s.md (1.0 → 1.1)
- BC-2.04.012-token-content-hash-verification.md (1.0 → 1.1)
- BC-2.04.013-capability-check-audit-logging.md (1.0 → 1.1)
- BC-2.04.014-tools-list-changed-notification.md (was 1.1; bumped to 1.2; changelog row appended)
- BC-2.04.015-write-denied-structured-error.md (1.0 → 1.1)

### Subsystem 2.05 (11 files, all active)

- BC-2.05.001-audit-entry-per-tool-invocation.md (was 1.1; bumped to 1.2; changelog row appended)
- BC-2.05.002-audit-entry-structured-json-format.md (1.0 → 1.1)
- BC-2.05.003-secret-redaction-in-audit-entries.md (1.0 → 1.1)
- BC-2.05.004-write-operation-audit-detail.md (1.0 → 1.1)
- BC-2.05.005-credential-access-audit.md (1.0 → 1.1)
- BC-2.05.006-audit-entry-immutability.md (1.0 → 1.1)
- BC-2.05.007-vector-pipeline-compatibility.md (1.0 → 1.1)
- BC-2.05.008-soc2-iso27001-field-requirements.md (1.0 → 1.1)
- BC-2.05.009-feature-flag-evaluation-audit.md (1.0 → 1.1)
- BC-2.05.010-confirmation-token-audit.md (1.0 → 1.1)
- BC-2.05.011-audit-forwarding-at-least-once.md (1.0 → 1.1; also had missing lifecycle fields)

### Subsystem 2.06 (10 files, all active)

- BC-2.06.001-toml-config-loading.md (1.0 → 1.1)
- BC-2.06.002-per-client-sensor-mapping.md (1.0 → 1.1)
- BC-2.06.003-credential-reference-resolution.md (1.0 → 1.1)
- BC-2.06.004-capability-override-resolution.md (1.0 → 1.1)
- BC-2.06.005-config-validation-multi-error.md (was 1.1; bumped to 1.2; changelog row appended)
- BC-2.06.006-dry-run-validation-mode.md (1.0 → 1.1)
- BC-2.06.007-missing-required-field-errors.md (1.0 → 1.1)
- BC-2.06.008-default-values-and-env-var-override.md (1.0 → 1.1)
- BC-2.06.009-tools-list-changed-on-client-switch.md (was 1.1; bumped to 1.2; changelog row appended)
- BC-2.06.010-client-id-validation.md (1.0 → 1.1)

## Version Bump Summary

| Category | Count | Notes |
|----------|-------|-------|
| 1.0 → 1.1 (full remediation) | 24 | See subsystem tables above |
| Already 1.1+: bumped to next minor | 12 | BC-2.04.003 (→1.2), BC-2.04.005 (→1.3), BC-2.04.007 (→1.2), BC-2.04.009 (→1.3), BC-2.04.010 (→2.1), BC-2.04.014 (→1.2), BC-2.05.001 (→1.2), BC-2.05.011 (→1.1 from 1.0 with missing fields), BC-2.06.005 (→1.2), BC-2.06.009 (→1.2) |

## Tombstone BCs

None. All 36 files in scope are active.

## Frontmatter Changes Applied (Rule A)

All 36 files received the following fields (if missing):
- `inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]`
- `input-hash: "[pending-recompute]"`
- `traces_to: ["CAP-NNN"]` (pointing to the capability ID in the BC body)
- `extracted_from: ".factory/specs/prd.md"`

**BC-2.05.011 anomaly:** This file was created in phase 2-patch and lacked 7 lifecycle
fields (`deprecated`, `deprecated_by`, `modified`, `removal_reason`, `removed`,
`replacement`, `retired`) that all other BCs carry. These were added during this sweep
along with the standard Wave 2 frontmatter additions. The hook caught the partial Edit
(pre-Write) and all fields were added in a single Write.

## Sections Added (Rule C)

All 36 active BCs received the following sections (if missing):

### `## Description`
Synthesized from title, Preconditions, and Postconditions for each BC. Notable synthesis
judgment calls:

1. **BC-2.04.003 (Hierarchical Capability Resolution)** — Description names all three
   relevant invariants (deny-by-default, most-specific-path wins, Deny overrides Allow at
   same specificity) because these are three load-bearing VP targets (VP-002, VP-003,
   VP-004).

2. **BC-2.04.009 (Confirmation Token Generation)** — Description highlights the proactive
   cleanup sweep (expired tokens swept before cap check) because this is the non-obvious
   behavior that prevents `E-FLAG-007` false positives under normal usage patterns.

3. **BC-2.04.010 (Confirmation Token Consumption)** — Description calls out the
   `__global__` sentinel exception explicitly, since the equality-check-not-config-lookup
   distinction is security-relevant and easy to misimplement.

4. **BC-2.05.002 (Structured JSON Format)** — Description summarizes all four `client_id`
   sentinel values because these are commonly misunderstood by implementers working on
   the audit middleware.

5. **BC-2.05.011 (Audit Forwarding)** — Already had a rich `## Description`. No synthesis
   needed; description preserved verbatim.

6. **BC-2.06.009 (Tools List Changed)** — Description explicitly notes this is the
   config-subsystem counterpart of BC-2.04.014 to prevent implementers from treating
   them as duplicates and collapsing them.

### `## Canonical Test Vectors`
Placeholder tables citing `test-vectors.md` supplement, with 3-5 scenario rows synthesized
from edge cases and key postconditions. All 36 files.

### `## Verification Properties`
Cross-referenced from VP-INDEX v1.5. See VP citations table below.

### `## Changelog`
Added to all 36 files. For files that already had a Changelog, a row was appended.

## Verification Properties Cross-References Added

VPs from VP-INDEX v1.5 cited in new `## Verification Properties` sections:

| VP | BCs Referenced |
|----|----------------|
| VP-001 (TenantId rejects invalid characters) | BC-2.06.010 |
| VP-002 (Capability resolution: deny-by-default) | BC-2.04.003, BC-2.06.004 |
| VP-003 (Capability resolution: most-specific-path wins) | BC-2.04.003, BC-2.06.004 |
| VP-004 (Capability resolution: deny overrides allow at same specificity) | BC-2.04.003 |
| VP-007 (Confirmation token expiry: expired at boundary inclusive) | BC-2.04.009, BC-2.04.010, BC-2.04.011 |
| VP-008 (Confirmation token: single-use enforcement) | BC-2.04.009, BC-2.04.010, BC-2.05.010 |
| VP-009 (Confirmation token: content hash mismatch rejects) | BC-2.04.010, BC-2.04.012 |
| VP-010 (Token cap: store rejects at 100 active tokens) | BC-2.04.009 |
| VP-020 (Feature flag: compile AND runtime must both permit) | BC-2.04.001, BC-2.04.002, BC-2.04.004, BC-2.04.015 |
| VP-033 (Audit buffer: RocksDB write completes before delivery attempt) | BC-2.05.001 |
| VP-039 (Audit forward watermark: monotonically non-decreasing) | BC-2.05.011 |

BCs with no matching VP (placeholder row used): BC-2.04.005, BC-2.04.006, BC-2.04.007,
BC-2.04.008, BC-2.04.013, BC-2.04.014, BC-2.05.002, BC-2.05.003, BC-2.05.004, BC-2.05.005,
BC-2.05.006, BC-2.05.007, BC-2.05.008, BC-2.05.009, BC-2.06.001, BC-2.06.002, BC-2.06.003,
BC-2.06.005, BC-2.06.006, BC-2.06.007, BC-2.06.008

## Anomalies

1. **BC-2.05.011 incomplete frontmatter (phase 2-patch origin):** This BC was created
   in phase 2-patch (2026-04-16) rather than cycle-1 and lacked the full lifecycle
   frontmatter block present in all other BCs (`deprecated`, `deprecated_by`, `modified`,
   `removal_reason`, `removed`, `replacement`, `retired`). The hook caught this during the
   partial Edit attempt. Resolved via single Write with complete frontmatter.

2. **BC-2.05.011 hook fire on partial Edit:** Per Wave 1 lesson, the
   `validate-template-compliance.sh` hook fires on Edit tool calls. An initial Edit to
   BC-2.05.011 (adding only frontmatter) triggered the hook warning before `## Canonical
   Test Vectors` and `## Verification Properties` were present. File was then remediated
   with a full Write call. No other BCs required multiple attempts.

3. **No tombstones in Wave 2:** All 36 files are active. The 0-tombstone count differs
   from Wave 1 (6 tombstones in 2.01).

4. **BC-2.04.010 at version 2.0 (active):** Like BC-2.01.002 in Wave 1, this file was
   a full rewrite (v2.0) but `lifecycle_status: active`. Bumped to 2.1 with changelog
   row appended per Rule D.

5. **BC-2.06.005 had existing Changelog:** This file was already at v1.1 with a Changelog
   section from `deferred-cleanup-track-1` burst. Changelog row appended; bumped to 1.2.

## Files Skipped

None. All 36 files in scope were remediated.

## Next Steps

1. **input-hash recompute:** All 36 files have `input-hash: "[pending-recompute]"`. Run
   `compute-input-hash` (or ask state-manager) on all files to populate actual hash values.

2. **VP-INDEX propagation:** VP citations added to BC bodies do not require VP-INDEX
   changes — VP-INDEX already lists these VPs; the BC-side `## Verification Properties`
   section is the new addition. No VP-INDEX update required from this wave.

3. **BC-INDEX verification:** No H1 headings were changed in this wave, so BC-INDEX title
   drift is not expected. A spot-check is recommended.
