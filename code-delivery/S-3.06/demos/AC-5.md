# AC-5 — Unknown Write Verb Suggestion (EC-11-063)

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `FROM crowdstrike_hosts | where x = 1 | nonexistent_verb`, when parsed, then
`E-QUERY-023` (unknown write verb) with suggestion listing available write verbs for
the `crowdstrike` source.

## Test Names

- Integration: `test_ac5_unknown_verb_suggestion` (no-panic, public API)
- Unit (pub-crate): `test_BC_2_11_004_parse_pipe_with_write_unknown_verb`
- Error constructor: `test_BC_2_11_004_error_023_message_contains_code_verb_and_suggestions`

## Evidence

### Unit test — E-QUERY-023 assertion (with registry)

```
test test_BC_2_11_004_parse_pipe_with_write_unknown_verb ... ok
```

Query: `FROM crowdstrike_hosts | where x = 1 | nonexistent_verb`
Registry: `["contain", "tag"]`

Assertions:
- `result.is_err()`
- `result.unwrap_err()[0].message.contains("E-QUERY-023")`

### Error constructor test

```
test test_BC_2_11_004_error_023_message_contains_code_verb_and_suggestions ... ok
```

```rust
let err = ParseError::unknown_write_verb(0, "nonexistent", &["contain", "tag"]);
assert!(err.message.contains("E-QUERY-023"));
assert!(err.message.contains("nonexistent"));
```

### Error message format (from error.rs constructor)

```
E-QUERY-023: unknown write verb 'nonexistent_verb' — available verbs: contain, tag
```

When available_verbs is empty:
```
E-QUERY-023: unknown write verb 'VERB' — no write verbs are registered for this sensor
```

### Source path

- Error constructor: `crates/prism-query/src/error.rs` — `ParseError::unknown_write_verb`
- Detection: `crates/prism-query/src/pipe_parser.rs` — `parse_pipe_with_write` (terminal
  identifier that is neither a pipe stage keyword nor a registered write verb)

## Result

PASS — E-QUERY-023 returned with suggestion list for unknown terminal verb.
