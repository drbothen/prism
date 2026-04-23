# Evidence Report — S-1.05: prism-ocsf Field Mapping and Normalization

**Story:** S-1.05
**Branch:** feature/S-1.05-ocsf-field-mapping
**Policy:** POL-010
**Date:** 2026-04-23
**Rebased onto:** origin/develop (HEAD 94033a69 — S-1.15 merged)
**Post-rebase SHA:** 2c4b8202
**Test state:** 68 passed; 4 pre-existing RED GATE stubs (S-1.04 scope, require ocsf-proto-gen); 1 ignored

---

## Coverage Summary

| AC | Title | BC | Recording | Error Path |
|----|-------|----|-----------|------------|
| AC-1 | CrowdStrike severity "High" → severity_id=4; detection_id → finding_info.uid | BC-2.02.003 | `AC-001-crowdstrike-severity-mapping.gif` | Out-of-range severity → severity_id 99 (included) |
| AC-2 | CrowdStrike behaviors[0].tactic → attacks[0].tactic.name | BC-2.02.003 | `AC-002-crowdstrike-behaviors-tactic.gif` | Covered by AC-1 scope |
| AC-3 | Cyberint Unix timestamp 1710498600 → 2024-03-15T10:30:00Z | BC-2.02.004 | `AC-003-cyberint-unix-timestamp.gif` | All 3 valid formats demonstrated |
| AC-4 | Cyberint "not-a-date" → Err(OcsfTimestampParseError) with field+raw | BC-2.02.004, BC-2.02.011 | `AC-004-cyberint-bad-timestamp-error.gif` | Inherent (full error path demo) |
| AC-5 | Claroty integer id=42 → device.uid="claroty:42"; unknown type → Err | BC-2.02.005 | `AC-005-claroty-integer-id-uid.gif` | Unknown record type → OcsfUnknownRecordType (included) |
| AC-6 | Armis device no timestamps → current-time fallback (never fails) | BC-2.02.006 | `AC-006-armis-timestamp-fallback.gif` | fallback chain: last_seen → created_at → timestamp → now (all exercised) |
| AC-7 | Custom vendor field "custom_vendor_field"="xyz" preserved in extensions | BC-2.02.007 | `AC-007-unmapped-fields-extensions.gif` | All-unmapped-fields and original-name invariant (included) |
| AC-8 | AliasResolver four-tier: PrismMetadata / ProtoField / RawExtension / Absent | BC-2.02.008 | `AC-008-alias-resolver-four-tiers.gif` | OOB array index → Absent; tier precedence order verified |
| AC-9 | Missing detection_id → OcsfNormalizationFailed with source_id + field name | BC-2.02.011 | `AC-009-normalization-error-context.gif` | Inherent (full error path demo) |
| AC-10 | VP-017 proptest: no input fields silently dropped across 10,000+ cases | VP-017, BC-2.02.007 | `AC-010-vp017-proptest-no-silent-drop.gif` | Proptest shrinking handles failure cases |

---

## RED GATE Note

Four tests in `bc_2_02_002_normalizer` and `proptest_normalizer` remain failing by design —
they are S-1.04 scope RED GATE stubs that require `ocsf-proto-gen` (real OCSF descriptor
pool) to pass. This is pre-existing from before this story's branch was cut and is unchanged
by the S-1.05 implementation. These 4 tests are NOT in scope for S-1.05.

---

## File Index

### VHS Recordings

| File | AC | Format |
|------|----|--------|
| `AC-001-crowdstrike-severity-mapping.gif` | AC-1 | GIF (embed) |
| `AC-001-crowdstrike-severity-mapping.webm` | AC-1 | WebM (archival) |
| `AC-001-crowdstrike-severity-mapping.tape` | AC-1 | VHS source |
| `AC-002-crowdstrike-behaviors-tactic.gif` | AC-2 | GIF |
| `AC-002-crowdstrike-behaviors-tactic.webm` | AC-2 | WebM |
| `AC-002-crowdstrike-behaviors-tactic.tape` | AC-2 | VHS source |
| `AC-003-cyberint-unix-timestamp.gif` | AC-3 | GIF |
| `AC-003-cyberint-unix-timestamp.webm` | AC-3 | WebM |
| `AC-003-cyberint-unix-timestamp.tape` | AC-3 | VHS source |
| `AC-004-cyberint-bad-timestamp-error.gif` | AC-4 | GIF |
| `AC-004-cyberint-bad-timestamp-error.webm` | AC-4 | WebM |
| `AC-004-cyberint-bad-timestamp-error.tape` | AC-4 | VHS source |
| `AC-005-claroty-integer-id-uid.gif` | AC-5 | GIF |
| `AC-005-claroty-integer-id-uid.webm` | AC-5 | WebM |
| `AC-005-claroty-integer-id-uid.tape` | AC-5 | VHS source |
| `AC-006-armis-timestamp-fallback.gif` | AC-6 | GIF |
| `AC-006-armis-timestamp-fallback.webm` | AC-6 | WebM |
| `AC-006-armis-timestamp-fallback.tape` | AC-6 | VHS source |
| `AC-007-unmapped-fields-extensions.gif` | AC-7 | GIF |
| `AC-007-unmapped-fields-extensions.webm` | AC-7 | WebM |
| `AC-007-unmapped-fields-extensions.tape` | AC-7 | VHS source |
| `AC-008-alias-resolver-four-tiers.gif` | AC-8 | GIF |
| `AC-008-alias-resolver-four-tiers.webm` | AC-8 | WebM |
| `AC-008-alias-resolver-four-tiers.tape` | AC-8 | VHS source |
| `AC-009-normalization-error-context.gif` | AC-9 | GIF |
| `AC-009-normalization-error-context.webm` | AC-9 | WebM |
| `AC-009-normalization-error-context.tape` | AC-9 | VHS source |
| `AC-010-vp017-proptest-no-silent-drop.gif` | AC-10 | GIF |
| `AC-010-vp017-proptest-no-silent-drop.webm` | AC-10 | WebM |
| `AC-010-vp017-proptest-no-silent-drop.tape` | AC-10 | VHS source |
| `FULL-SUITE-all-mappers.gif` | All | GIF |
| `FULL-SUITE-all-mappers.webm` | All | WebM |
| `FULL-SUITE-all-mappers.tape` | All | VHS source |

---

## Verification

All VHS recordings produced `.gif` and `.webm` outputs via `vhs 0.10.0`.
Font: `FiraCode Nerd Font Mono`. Theme: `Catppuccin Mocha`.

```
$ cargo test -p prism-ocsf -- tests::mapper_tests tests::alias_tests tests::proptest_extensions 2>&1 | grep "test result"
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 63 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 72 filtered out
```

```
$ cargo test -p prism-ocsf 2>&1 | grep "test result"
test result: FAILED. 68 passed; 4 failed; 1 ignored
# (4 failures are S-1.04 RED GATE stubs, pre-existing, out of S-1.05 scope)
```
