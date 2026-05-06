# AC-2 — Write Stage Not Terminal (EC-11-060)

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `FROM crowdstrike_hosts | contain | where severity >= 3`, when parsed, then
`E-QUERY-024` is returned with message indicating the write stage is not terminal, and
no AST is produced.

## Test Names

- Integration: `test_ac2_write_stage_not_terminal` (no-panic, public API)
- Unit (pub-crate): `test_BC_2_11_004_parse_pipe_with_write_verb_not_terminal`

## Evidence

### Unit test — E-QUERY-024 assertion (parse_pipe_with_write, with registry)

```
test test_BC_2_11_004_parse_pipe_with_write_verb_not_terminal ... ok
```

Query: `FROM crowdstrike_hosts | contain | where severity >= 3`
Registry: `["contain"]`

Assertions:
- `result.is_err()`
- `result.unwrap_err()[0].message.contains("E-QUERY-024")`

### Public API (integration test — no-panic)

```
test test_ac2_write_stage_not_terminal ... ok
```

### Error message format (from error.rs constructor)

```
E-QUERY-024: write stage must be in terminal pipe position —
'contain' at position N is followed by additional stages
```

### Source path

- Error emission: `crates/prism-query/src/error.rs` — `ParseError::write_stage_not_terminal`
- Detection: `crates/prism-query/src/pipe_parser.rs` — `parse_pipe_with_write`

## Result

PASS — E-QUERY-024 returned when write verb appears in non-terminal pipe position.
