# Evidence Report — S-1.04: OCSF Schema Loading and DynamicMessage

**Story ID:** S-1.04  
**Branch:** feature/S-1.04-ocsf-schema-loading  
**Base commit:** 76837cf  
**Test suite:** 36 passed, 0 failed, 1 ignored  
**Recorded:** 2026-04-22  
**Toolchain:** VHS 0.10.0, Rust (prism-ocsf test harness)

---

## Coverage Map

| AC | BC | Description | Recording | Status |
|----|-----|-------------|-----------|--------|
| AC-1 | BC-2.02.009 | `ocsf_version()` returns compile-time pinned semver | [gif](AC-1-ocsf-version-pin.gif) / [webm](AC-1-ocsf-version-pin.webm) | PASS |
| AC-2 | BC-2.02.001 | `DescriptorPool` singleton loads all OCSF event class descriptors | [gif](AC-2-pool-loading.gif) / [webm](AC-2-pool-loading.webm) | PASS |
| AC-3 | BC-2.02.002 | `normalize("crowdstrike","detection",json)` returns `DynamicMessage` | [gif](AC-3-normalize-dynamic-message.gif) / [webm](AC-3-normalize-dynamic-message.webm) | PASS |
| AC-4 | BC-2.02.010 | `display_name("severity_id", 4)` returns `Some("High")` | [gif](AC-4-enum-map-lookup.gif) / [webm](AC-4-enum-map-lookup.webm) | PASS |
| AC-5 | BC-2.02.010 | `display_name("severity_id", 99)` returns gracefully (no panic) | [gif](AC-5-enum-map-unknown.gif) / [webm](AC-5-enum-map-unknown.webm) | PASS |
| AC-6 | BC-2.02.012 | `select("crowdstrike","detection")` returns `Ok(2004)`, not deprecated 2001 | [gif](AC-6-selector-crowdstrike.gif) / [webm](AC-6-selector-crowdstrike.webm) | PASS |
| AC-7 | BC-2.02.012 | `select("claroty","alert")` returns `Ok(2004)` | [gif](AC-7-selector-claroty.gif) / [webm](AC-7-selector-claroty.webm) | PASS |
| AC-8 | BC-2.02.012 | `select("vendor_x","unknown_type")` returns `Err(OcsfUnknownEventClass)` | [gif](AC-8-selector-unknown-err.gif) / [webm](AC-8-selector-unknown-err.webm) | PASS |
| AC-9 | VP-016 | Proptest: all `Ok` normalize outputs survive protobuf encode/decode round-trip | [gif](AC-9-vp016-proptest.gif) / [webm](AC-9-vp016-proptest.webm) | PASS |
| AC-5 (deferred) | BC-2.02.002 | `class_uid = 2004` field value set in `DynamicMessage` | [placeholder](AC-5-deferred-class-uid-field-population.md) | DEFERRED (S-1.05) |

---

## Deferred Test Note

`test_BC_2_02_002_normalized_message_has_class_uid_2004` is marked `#[ignore]`
in the test suite. It verifies that the `class_uid` field inside the returned
`DynamicMessage` is set to `2004`. This requires sensor-specific field mappers
from S-1.05. The test is present and will be un-ignored when S-1.05 ships.
See [AC-5-deferred-class-uid-field-population.md](AC-5-deferred-class-uid-field-population.md).

---

## BC Test Coverage

| BC | Tests | Tape |
|----|-------|------|
| BC-2.02.001 (pool loading) | `test_BC_2_02_001_pool_contains_detection_finding_descriptor`, `test_BC_2_02_001_pool_contains_all_83_event_class_descriptors`, `test_BC_2_02_001_pool_populated_without_network_access` | AC-2 |
| BC-2.02.002 (normalization) | `test_BC_2_02_002_crowdstrike_detection_produces_dynamic_message`, `test_BC_2_02_002_empty_json_produces_dynamic_message`, `test_BC_2_02_002_unknown_sensor_returns_err`, `test_BC_2_02_002_malformed_input_does_not_panic`, `test_BC_2_02_002_known_sensor_returns_typed_error_or_ok`, `test_BC_2_02_002_vp016_dynamic_message_round_trips` | AC-3 |
| BC-2.02.009 (version pin) | `test_BC_2_02_009_ocsf_version_is_nonempty`, `test_BC_2_02_009_pinned_version_is_semver`, `test_BC_2_02_009_invariant_version_immutable_across_calls` | AC-1 |
| BC-2.02.010 (enum map) | `test_BC_2_02_010_severity_id_4_returns_high`, `test_BC_2_02_010_severity_id_99_returns_none`, `test_BC_2_02_010_severity_id_99_returns_other`, `test_BC_2_02_010_unknown_value_returns_formatted_string`, `test_BC_2_02_010_severity_id_canonical_values`, `test_BC_2_02_010_activity_id_canonical_values`, `test_BC_2_02_010_invariant_display_name_never_panics` | AC-4, AC-5 |
| BC-2.02.012 (selector) | `test_BC_2_02_012_crowdstrike_detection_returns_2004`, `test_BC_2_02_012_crowdstrike_incident_returns_2005`, `test_BC_2_02_012_cyberint_alert_returns_2004`, `test_BC_2_02_012_claroty_alert_returns_2004`, `test_BC_2_02_012_claroty_device_returns_5001`, `test_BC_2_02_012_claroty_vulnerability_returns_2002`, `test_BC_2_02_012_armis_device_returns_5001`, `test_BC_2_02_012_armis_alert_returns_2004`, `test_BC_2_02_012_armis_audit_log_returns_3001`, `test_BC_2_02_012_claroty_audit_log_returns_3001`, `test_BC_2_02_012_unknown_pair_returns_err`, `test_BC_2_02_012_invariant_no_deprecated_2001_in_any_mapping`, `test_BC_2_02_012_invariant_select_is_deterministic`, `test_BC_2_02_012_rejects_empty_sensor`, `test_BC_2_02_012_rejects_empty_record_type` | AC-6, AC-7, AC-8 |
| VP-016 (proptest) | `test_VP_016_normalize_produces_ok_for_valid_inputs`, `prop_normalize_output_is_valid_protobuf` | AC-9 |

---

## Files in This Directory

```
docs/demo-evidence/S-1.04/
  AC-1-ocsf-version-pin.tape
  AC-1-ocsf-version-pin.gif
  AC-1-ocsf-version-pin.webm
  AC-2-pool-loading.tape
  AC-2-pool-loading.gif
  AC-2-pool-loading.webm
  AC-3-normalize-dynamic-message.tape
  AC-3-normalize-dynamic-message.gif
  AC-3-normalize-dynamic-message.webm
  AC-4-enum-map-lookup.tape
  AC-4-enum-map-lookup.gif
  AC-4-enum-map-lookup.webm
  AC-5-enum-map-unknown.tape
  AC-5-enum-map-unknown.gif
  AC-5-enum-map-unknown.webm
  AC-6-selector-crowdstrike.tape
  AC-6-selector-crowdstrike.gif
  AC-6-selector-crowdstrike.webm
  AC-7-selector-claroty.tape
  AC-7-selector-claroty.gif
  AC-7-selector-claroty.webm
  AC-8-selector-unknown-err.tape
  AC-8-selector-unknown-err.gif
  AC-8-selector-unknown-err.webm
  AC-9-vp016-proptest.tape
  AC-9-vp016-proptest.gif
  AC-9-vp016-proptest.webm
  AC-5-deferred-class-uid-field-population.md
  evidence-report.md
```
