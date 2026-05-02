# Red Gate Log — W3-FIX-SEC-004

**Story:** W3-FIX-SEC-004 — TOML inline-table redaction bypass + constant-time admin token comparison  
**Date:** 2026-05-01  
**Agent:** Test Writer  
**Status:** RED GATE VERIFIED — proceed to implementer

---

## Test Files Written

| File | Crate | Tests | Red Gate Status |
|------|-------|-------|----------------|
| `crates/prism-customer-config/tests/sec_p3_001_inline_table_redaction.rs` | prism-customer-config | 6 | 3 FAIL / 3 PASS (guards) |
| `crates/prism-customer-config/tests/sec_p3_002_pipe_anchor.rs` | prism-customer-config | 5 | 2 FAIL / 3 PASS (guards) |
| `crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs` | prism-dtu-claroty | 7 | 7 PASS (behavioral regression guards) |

**Total tests written:** 18  
**Failing (Red Gate confirmed):** 5  
**Passing at Red Gate:** 13 (regression / non-redaction guards, or behavioral contracts already satisfied)

---

## SEC-P3-001 — Inline-Table Credential Redaction (`sec_p3_001_inline_table_redaction.rs`)

```
running 6 tests
test test_AC_003_non_credential_inline_field_visible ... ok
test test_AC_001_single_line_credential_regression ... ok
test test_AC_003_single_level_non_credential_not_redacted ... ok
test test_AC_002_nested_credentials_in_array_table_redacted ... FAILED
test test_AC_001_inline_table_multiple_credential_fields_both_redacted ... FAILED
test test_AC_001_inline_table_credentials_redacted ... FAILED

test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured
```

**Failing tests and reason:**

- `test_AC_001_inline_table_credentials_redacted`: `credentials = { bearer_token = "my-secret-value" }` — `bearer_token` value leaks because `sanitize_error_message` only examines `credentials` (the outer field) against `is_credential_pattern`, which returns false.
- `test_AC_001_inline_table_multiple_credential_fields_both_redacted`: `creds = { api_key = "key-alpha", api_secret = "secret-beta" }` — both inner credential values leak.
- `test_AC_002_nested_credentials_in_array_table_redacted`: `config = { api_password = "pass123-secret" }` inside `[[dtu]]` array table — inner `api_password` not scanned.

**Passing tests (regression / over-redaction guards):**
- `test_AC_003_non_credential_inline_field_visible`: `settings = { display_name = "ACME Corp UI" }` — non-credential inline field correctly not redacted (no false positive).
- `test_AC_003_single_level_non_credential_not_redacted`: Plain non-credential field not redacted.
- `test_AC_001_single_line_credential_regression`: Existing single-line `bearer_token = "classic-secret"` still redacted.

---

## SEC-P3-002 — `find_snippet_pipe` Digit-Prefix Anchor (`sec_p3_002_pipe_anchor.rs`)

```
running 5 tests
test test_AC_002_caret_lines_not_suppressed_by_anchor ... ok
test test_AC_001_raw_source_line_api_key_with_pipe_redacted ... ok
test test_AC_001_single_line_credential_through_anchored_pipe_finder ... ok
test test_AC_001_credential_value_with_pipe_does_not_break_extraction ... FAILED
test test_AC_002_only_digit_prefix_matches_pipe ... FAILED

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured
```

**Failing tests and reason:**

- `test_AC_001_credential_value_with_pipe_does_not_break_extraction`: `api_key = "abc | def-secret"` — the source context emitted without a digit prefix results in the raw-source path being used, but the `" | "` in the value causes `find_snippet_pipe` to misparse when there IS a digit prefix (EC-003). Value `"def-secret"` leaks.
- `test_AC_002_only_digit_prefix_matches_pipe`: `api_secret = "top | secret-value"` — the ` | ` in the value appears in the source context dump; the raw-source fallback should handle the line, but `find_snippet_pipe` incorrectly returns `Some(pos)` for the wrong separator position, causing misparse and leaking `"secret-value"`.

**Passing tests (regression / anchor guards):**
- `test_AC_002_caret_lines_not_suppressed_by_anchor`: Caret lines (`^`) still present in error output.
- `test_AC_001_raw_source_line_api_key_with_pipe_redacted`: Raw source `api_key = "raw-pipe-secret"` handled correctly by raw-source path.
- `test_AC_001_single_line_credential_through_anchored_pipe_finder`: Single-line credential still redacted.

---

## SEC-P3-003 — Constant-Time Admin Token Comparison (`sec_p3_003_constant_time_admin_token.rs`)

```
running 7 tests
test test_AC_002_constant_time_compare_wrong_token_returns_401 ... ok
test test_AC_002_configure_wrong_token_returns_401 ... ok
test test_AC_001_configure_correct_token_returns_200 ... ok
test test_AC_002_absent_token_returns_401 ... ok
test test_AC_001_constant_time_compare_correct_token_returns_200 ... ok
test test_AC_002_wrong_length_token_returns_401 ... ok
test test_AC_002_cross_clone_constant_time_token_returns_401 ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured
```

**Why all 7 pass at the Red Gate:**  
The observable HTTP behavior for the admin-token gate was already implemented in W3-FIX-SEC-002. SEC-P3-003 changes the IMPLEMENTATION (short-circuit `!=` → `subtle::ct_eq`) without changing the observable behavior. These tests are binding regression guards: if the constant-time refactor accidentally inverts the comparison polarity, omits the configure handler, or breaks the cross-clone rejection, these tests catch it.

The implementation requirement (use of `subtle::ConstantTimeEq`) must be verified by code inspection during the implementation phase.

---

## Regression Check

Existing `sec006_toml_multiline_redaction.rs` tests: **5/5 pass** — no regression.

```
running 5 tests
test test_BC_3_3_001_SEC006_single_line_credential_still_redacted ... ok
test test_BC_3_3_001_SEC006_non_credential_field_not_redacted ... ok
test test_BC_3_3_001_SEC006_multiline_password_not_in_error_message ... ok
test test_BC_3_3_001_SEC006_multiline_api_secret_not_in_error_message ... ok
test test_BC_3_3_001_SEC006_multiline_bearer_token_not_in_error_message ... ok

test result: ok. 5 passed; 0 failed
```

---

## Implementer Instructions

Make each failing test pass by implementing exactly the changes described in the story:

### SEC-P3-001: `crates/prism-customer-config/src/validator.rs`

In `sanitize_error_message`, after the existing leading-field check (lines 381–393), add a second loop over all ` = ` positions in `content`. For each position, extract the token immediately before it and call `is_credential_pattern`. On first match, emit `[REDACTED]` for the full content line and continue to the next input line.

Target: make `test_AC_001_inline_table_credentials_redacted`, `test_AC_001_inline_table_multiple_credential_fields_both_redacted`, `test_AC_002_nested_credentials_in_array_table_redacted` pass.

### SEC-P3-002: `crates/prism-customer-config/src/validator.rs`

Replace the `find_snippet_pipe` function body with the prefix-validation implementation from the story (AC-002 exact code): `let pos = line.find(" | ")?;` followed by a check that `line[..pos].chars().all(|c| c.is_ascii_digit() || c == ' ')`.

Target: make `test_AC_001_credential_value_with_pipe_does_not_break_extraction`, `test_AC_002_only_digit_prefix_matches_pipe` pass.

### SEC-P3-003: all four DTU clone crates

Add `subtle = "2"` to workspace `Cargo.toml`, then `subtle = { workspace = true }` to each DTU crate's `[dependencies]`. Replace the `provided != Some(state.admin_token.as_str())` pattern in both `dtu_reset`/`post_reset` AND `dtu_configure`/`post_configure` handlers in all four crates with the `subtle::ct_eq` pattern from the story (AC-003 exact code).

Target: all 7 SEC-P3-003 tests in claroty continue to pass; analogous tests in the other three clones (if written) also pass.

---

## Cargo.toml Changes (already applied by Test Writer)

- `crates/prism-customer-config/Cargo.toml`: added `[[test]]` entries for `sec_p3_001_inline_table_redaction` and `sec_p3_002_pipe_anchor`.
- `crates/prism-dtu-claroty/Cargo.toml`: added `[[test]]` entry for `sec_p3_003_constant_time_admin_token` with `required-features = ["dtu"]`.
