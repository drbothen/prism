# AC-8: FidelityValidator flags missing required field

## Acceptance Criterion

Given `FidelityValidator::run()` is called with a check expecting field
`"status"` in the response body, When the DTU response body does not contain `"status"`,
Then the `FidelityReport` contains a `FidelityFailure` for that check.

## Test

- File: `crates/prism-dtu-common/tests/ac_8_fidelity_validator.rs`
- Function: `ac_8_fidelity_validator_flags_missing_required_field`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_8_fidelity_validator`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/fidelity.rs`

```rust
Ok(body) => {
    let mut field_failures: Vec<String> = Vec::new();
    for field in &check.required_fields {
        if body.get(field).is_none() {
            field_failures.push(field.clone());
        }
    }
    if field_failures.is_empty() {
        checks_passed += 1;
    } else {
        checks_failed += 1;
        failures.push(FidelityFailure {
            endpoint: check.endpoint.clone(),
            reason: format!(
                "missing required fields: {}",
                field_failures.join(", ")
            ),
        });
    }
}
```

## Test output

```
running 1 test
test ac_8_fidelity_validator_flags_missing_required_field ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Mapping

`FidelityValidator::run` issues HTTP requests to each check's endpoint, verifies the status code, deserializes the JSON body, and checks for each `required_fields` entry using `serde_json::Value::get`; a stub returning `{"result":"ok"}` fails the `"status"` field check and the report records `checks_failed: 1` with a reason containing "status".
