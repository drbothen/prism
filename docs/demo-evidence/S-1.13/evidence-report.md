# Evidence Report — S-1.13: Sensor Spec Write Endpoints

**Story:** S-1.13 — prism-spec-engine: Sensor Spec Write Endpoints  
**Branch:** feature/S-1.13-sensor-write-specs  
**Date:** 2026-04-22  
**Test suite:** `crates/prism-spec-engine/tests/write_endpoint_tests.rs`  
**Test result:** 28 passed, 1 failed (spec-contradiction-blocked — see AC-5)  
**Policy:** POL-010

---

## AC Coverage Map

| AC | Title | BC | Recording | Status |
|----|-------|----|-----------|--------|
| AC-1 | `registry.get("crowdstrike","contain")` returns `risk_tier=Irreversible`, `batch_limit=10` | BC-2.16.001 | [AC-001-registry-get-contain.gif](AC-001-registry-get-contain.gif) | PASS |
| AC-2 | `pipe_verb="where"` (reserved keyword) returns E-SPEC-011 | BC-2.16.009 | [AC-002-reserved-keyword-e-spec-011.gif](AC-002-reserved-keyword-e-spec-011.gif) | PASS |
| AC-3 | `batch_limit=0 + risk_tier="irreversible"` emits warning, spec loads Ok | BC-2.16.009 | [AC-003-batch-limit-irreversible-warning.gif](AC-003-batch-limit-irreversible-warning.gif) | PASS |
| AC-4 | `WriteTableDescriptor` with `sql_table=crowdstrike_contained_hosts`, `write_only=true` | BC-2.16.001 | [AC-004-write-table-descriptor.gif](AC-004-write-table-descriptor.gif) | PASS |
| AC-5 | At least 10 write verbs across all 4 sensors | BC-2.16.001 | [AC-005-ten-write-verbs-partial.gif](AC-005-ten-write-verbs-partial.gif) | BLOCKED (see below) |

---

## Recording Inventory

### AC-1: Registry get — contain endpoint (BC-2.16.001)

**Tests demonstrated:**
- `test_BC_2_16_001_write_registry_get_crowdstrike_contain` — registry loads and returns the spec
- `test_BC_2_16_001_crowdstrike_risk_tier_irreversible` — risk_tier field is Irreversible
- `test_BC_2_16_001_crowdstrike_batch_limit_ten` — batch_limit field is 10

| File | Type |
|------|------|
| [AC-001-registry-get-contain.gif](AC-001-registry-get-contain.gif) | VHS GIF |
| [AC-001-registry-get-contain.webm](AC-001-registry-get-contain.webm) | VHS WebM |
| [AC-001-registry-get-contain.tape](AC-001-registry-get-contain.tape) | VHS script source |

---

### AC-2: Reserved keyword rejection — E-SPEC-011 (BC-2.16.009, EC-001)

**Tests demonstrated:** All 6 reserved-keyword rejection tests:
- `where`, `sort`, `limit`, `join`, `enrich`, `head` — each returns `E-SPEC-011`

| File | Type |
|------|------|
| [AC-002-reserved-keyword-e-spec-011.gif](AC-002-reserved-keyword-e-spec-011.gif) | VHS GIF |
| [AC-002-reserved-keyword-e-spec-011.webm](AC-002-reserved-keyword-e-spec-011.webm) | VHS WebM |
| [AC-002-reserved-keyword-e-spec-011.tape](AC-002-reserved-keyword-e-spec-011.tape) | VHS script source |

---

### AC-3: Batch limit zero + irreversible warning (BC-2.16.009, EC-003)

**Tests demonstrated:**
- `test_BC_2_16_009_batch_limit_zero_irreversible_emits_warning` — warning present
- `test_BC_2_16_009_batch_limit_zero_irreversible_spec_loads` — result is Ok (no error)

| File | Type |
|------|------|
| [AC-003-batch-limit-irreversible-warning.gif](AC-003-batch-limit-irreversible-warning.gif) | VHS GIF |
| [AC-003-batch-limit-irreversible-warning.webm](AC-003-batch-limit-irreversible-warning.webm) | VHS WebM |
| [AC-003-batch-limit-irreversible-warning.tape](AC-003-batch-limit-irreversible-warning.tape) | VHS script source |

---

### AC-4: WriteTableDescriptor — write_only flag (BC-2.16.001)

**Tests demonstrated:**
- `test_BC_2_16_001_write_table_descriptor_write_only_flag` — `write_only=true`
- `test_BC_2_16_001_write_table_descriptor_sql_table_name` — `sql_table=crowdstrike_contained_hosts`

| File | Type |
|------|------|
| [AC-004-write-table-descriptor.gif](AC-004-write-table-descriptor.gif) | VHS GIF |
| [AC-004-write-table-descriptor.webm](AC-004-write-table-descriptor.webm) | VHS WebM |
| [AC-004-write-table-descriptor.tape](AC-004-write-table-descriptor.tape) | VHS script source |

---

### AC-5: Ten write verbs across four sensors — PARTIAL (BC-2.16.001)

**Status: BLOCKED — spec contradiction**

**Passing sub-tests demonstrated in recording:**
- `test_BC_2_16_001_verbs_for_crowdstrike_returns_four` — PASS (4 verbs: contain, uncontain, update_status, assign)
- `test_BC_2_16_001_all_four_sensors_ten_write_verbs` — FAIL (spec contradiction, see below)

| File | Type |
|------|------|
| [AC-005-ten-write-verbs-partial.gif](AC-005-ten-write-verbs-partial.gif) | VHS GIF |
| [AC-005-ten-write-verbs-partial.webm](AC-005-ten-write-verbs-partial.webm) | VHS WebM |
| [AC-005-ten-write-verbs-partial.tape](AC-005-ten-write-verbs-partial.tape) | VHS script source |

**Blocked test detail:**

`test_BC_2_16_001_all_four_sensors_ten_write_verbs` fails because the test data in
`write_endpoint_tests.rs` assigns `pipe_verb = "tag"` and `pipe_verb = "remove_tag"` to
**both** `claroty` and `armis` (see `claroty_endpoints()` and `armis_endpoints()` helpers,
lines 187–263). The implementation correctly enforces BC-2.16.009 EC-002 (global pipe_verb
uniqueness). The registry rejects `armis` registration with:

```
ESpec009: pipe_verb 'tag' for sensor 'armis' already registered by sensor 'claroty'
          — write verb must be globally unique (BC-2.16.009 EC-002)
ESpec009: pipe_verb 'remove_tag' for sensor 'armis' already registered by sensor 'claroty'
          — write verb must be globally unique (BC-2.16.009 EC-002)
```

**Root cause:** The story spec (Task 7) assigns `tag` and `remove_tag` to both claroty and
armis without acknowledging the EC-002 global uniqueness constraint. The test writer must
rename the armis verbs (e.g., `armis_tag`, `armis_remove_tag` or `label`/`remove_label`)
before this test can pass. The implementation is correct. Fix is in the test data and
optionally the story spec — to be resolved during the PR manager cycle.

---

## Additional Tests Covered (not AC-mapped, error paths)

| Test | Scenario | Result |
|------|----------|--------|
| `test_BC_2_16_009_empty_steps_rejected` | EC-004: empty steps array | PASS |
| `test_BC_2_16_009_record_id_field_uppercase_rejected` | EC-005: uppercase in record_id_field | PASS |
| `test_BC_2_16_009_record_id_field_special_chars_rejected` | EC-005: dash in record_id_field | PASS |
| `test_BC_2_16_009_record_id_field_valid_lowercase` | EC-005: valid `[a-z0-9_]+` field | PASS |
| `test_BC_2_16_009_cross_sensor_verb_uniqueness_collision` | EC-002: second sensor claims "contain" | PASS |
| `test_BC_2_16_009_all_errors_collected_no_fail_fast` | VP-059: all errors collected, no fail-fast | PASS |
| `test_BC_2_16_009_valid_spec_no_errors_no_warnings` | Clean spec produces no errors/warnings | PASS |
| `test_BC_2_16_009_risk_tier_invalid_string_parse_error` | `risk_tier="read"` fails deserialization | PASS |
| `test_BC_2_16_001_registry_is_empty_before_register` | Fresh registry has no descriptors | PASS |
| `test_BC_2_16_009_non_reserved_verb_passes` | Non-reserved verb returns None (no error) | PASS |
| `test_interpolation_record_ids_resolved_in_body_template` | `${record_ids}` resolved to JSON array | PASS |
| `test_interpolation_params_key_resolved` | `${params.KEY}` resolved to provided value | PASS |
| `test_interpolation_params_key_default_used_when_missing` | `${params.KEY\|default:V}` uses default | PASS |
| `test_interpolation_url_context_percent_encodes` | URL context percent-encodes spaces | PASS |
