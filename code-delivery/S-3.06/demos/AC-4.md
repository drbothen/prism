# AC-4 — Filter Mode Rejects Write Verbs (EC-11-064)

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `severity_id >= 4 | contain`, when parsed in filter mode (expression begins with
a predicate, no FROM keyword), then a parse error is returned indicating writes are not
permitted in filter mode.

## Test Names

- Integration: `test_ac4_filter_mode_write_rejected` (no-panic, public API)
- Integration (gap-fill): `test_BC_2_11_004_filter_mode_rejects_tag_verb_with_args`
- Integration (gap-fill): `test_BC_2_11_004_filter_mode_rejects_acknowledge_verb`
- Integration (gap-fill): `test_BC_2_11_004_filter_mode_field_named_contain_is_not_rejected`
- Unit (pub-crate): `test_BC_2_11_004_reject_write_verbs_in_filter_with_verb_in_input`
- Unit (pub-crate): `test_BC_2_11_004_reject_write_verbs_in_filter_clean_input_ok`
- Unit (pub-crate): `test_BC_2_11_004_reject_write_verbs_in_filter_empty_registry_always_ok`
- Unit (pub-crate): `test_BC_2_11_004_filter_rejection_case_insensitive`

## Evidence

### Unit test — direct rejection (with registry)

```
test test_BC_2_11_004_reject_write_verbs_in_filter_with_verb_in_input ... ok
```

`reject_write_verbs_in_filter("severity_id >= 4 | contain", &registry)` with
registry containing `["contain"]` returns `Err`.

### Unit test — clean input is accepted

```
test test_BC_2_11_004_reject_write_verbs_in_filter_clean_input_ok ... ok
```

`reject_write_verbs_in_filter("severity_id >= 4 AND status = 'active'", &registry)`
returns `Ok(())`.

### Unit test — empty registry never rejects

```
test test_BC_2_11_004_reject_write_verbs_in_filter_empty_registry_always_ok ... ok
```

Per `INV-FILTER-EMPTY-REGISTRY`: with no verbs registered, filter mode cannot reject
anything — no verbs are known to be write verbs.

### Unit test — case insensitive rejection

```
test test_BC_2_11_004_filter_rejection_case_insensitive ... ok
```

`CONTAIN` (uppercase) after `|` is still rejected when `contain` is in the registry.

### Integration gap-fill

```
test test_BC_2_11_004_filter_mode_rejects_tag_verb_with_args ... ok
test test_BC_2_11_004_filter_mode_rejects_acknowledge_verb ... ok
test test_BC_2_11_004_filter_mode_field_named_contain_is_not_rejected ... ok
```

The last test confirms `contain = 1` (field comparison, no pipe) does NOT trigger
E-QUERY-010 — the grammar-level rejection only fires when a verb appears after `|`.

### Architecture compliance

The rejection is at the grammar level via `reject_write_verbs_in_filter` — a
substring scan for registered write verbs in the input string before Chumsky parsing
begins. Not a post-parse semantic check.

## Result

PASS — Filter mode hard-rejects write verbs at grammar level; clean inputs are accepted.
