# Demo Evidence Report — W3-FIX-SEC-004

**Story:** W3-FIX-SEC-004 — TOML inline-table redaction bypass + constant-time admin token comparison
**Anchor BCs:** BC-3.3.004, BC-3.5.001, BC-3.5.002
**Track:** Platform Engineering — Customer Config + Sensor DTU
**Recorded:** 2026-05-01
**Tool:** VHS 0.10.0 (CLI recordings)
**Branch:** feature/W3-FIX-SEC-004
**Implementation commit:** 5740e648

---

## Findings Resolved

| Finding ID | Severity | CWE | Description |
|------------|----------|-----|-------------|
| SEC-P3-001 | MEDIUM | CWE-209 | Inline TOML table credentials not detected by `sanitize_error_message` |
| SEC-P3-002 / CR-019 | MEDIUM | CWE-209 | `find_snippet_pipe` false pipe match on credential value containing ` \| ` |
| SEC-P3-003 | LOW | CWE-208 | `X-Admin-Token` comparison used short-circuit equality in 4 DTU clone admin handlers |

**subtle = "2" added** to workspace `Cargo.toml` and to 4 crate `Cargo.toml` files (armis, claroty, crowdstrike, slack).
**8 handler sites refactored** to `ct_eq`: `dtu_reset` + `dtu_configure` in each of the 4 DTU clones.

---

## Coverage Map

| Demo ID | Acceptance Criterion | BC / Verification Property | Pass/Fail | Artifacts |
|---------|---------------------|---------------------------|-----------|-----------|
| AC-001 | SEC-P3-001: inline-table credential value redacted by `sanitize_error_message` | BC-3.3.004 postcondition 2 / VP-105 | PASS | [gif](AC-001-inline-table-credential-redacted.gif) [webm](AC-001-inline-table-credential-redacted.webm) [tape](AC-001-inline-table-credential-redacted.tape) |
| AC-002 | SEC-P3-002 / CR-019: `find_snippet_pipe` anchored to numeric prefix — pipe in credential value not confused | BC-3.3.004 postcondition 2 / VP-106 | PASS | [gif](AC-002-find-snippet-pipe-robustness.gif) [webm](AC-002-find-snippet-pipe-robustness.webm) [tape](AC-002-find-snippet-pipe-robustness.tape) |
| AC-003 | SEC-P3-003: `subtle::ct_eq` in all 4 DTU clone admin handlers + existing auth tests pass | BC-3.5.002 precondition 6 / VP-107 | PASS | [gif](AC-003-constant-time-admin-token.gif) [webm](AC-003-constant-time-admin-token.webm) [tape](AC-003-constant-time-admin-token.tape) |

**Coverage: 3/3 must-demo criteria recorded**

---

## AC-001 — Inline-Table Credential Redacted (SEC-P3-001)

**Traces to:** BC-3.3.004 postcondition 2, VP-105
**Finding:** MEDIUM CWE-209 — `credentials = { bearer_token = "x" }` not redacted because outer field `credentials` has no credential suffix; inner `bearer_token` was never examined.
**Fix:** `sanitize_error_message` now scans all ` = ` positions per snippet content line. On any matching `is_credential_pattern` token, the entire content line is replaced with `[REDACTED]`.
**Command:** `cargo test -p prism-customer-config --test sec_p3_001_inline_table_redaction -- test_AC_001_inline_table_credentials_redacted 2>&1 | tail -4`
**Expected:** `test result: ok. 1 passed; 0 failed; 0 ignored`
**Observed:** `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s`

**Path coverage:**
- Success path: snippet line `credentials = { bearer_token = "my-secret" }` → entire content replaced with `[REDACTED]`
- Error path: snippet line `settings = { timeout_seconds = 30, display_name = "ACME" }` → NOT redacted (covered by `test_AC_003_non_credential_inline_field_visible`)

**Recording:** `AC-001-inline-table-credential-redacted.gif` / `.webm`

---

## AC-002 — find_snippet_pipe Anchored to Numeric Prefix (SEC-P3-002 / CR-019)

**Traces to:** BC-3.3.004 postcondition 2, VP-106
**Finding:** MEDIUM CWE-209 — `line.find(" | ")` returns first occurrence anywhere in line; a credential value containing ` | ` offsets field-name extraction and bypasses redaction.
**Fix:** `find_snippet_pipe` validates that all characters before the ` | ` occurrence are ASCII digits or spaces (TOML 0.8 line-number prefix format). Non-digit prefix → returns `None`.
**Command:** `cargo test -p prism-customer-config --test sec_p3_002_pipe_anchor 2>&1 | tail -4`
**Expected:** `test result: ok. 5 passed; 0 failed; 0 ignored`
**Observed:** `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

**Path coverage:**
- Success path (pipe in value not confused): `"  3 | api_key = \"abc | def\""` → pipe at `3 |` position; `api_key` extracted; line redacted
- Error path (no digit prefix returns None): `"api_key = \"top | secret\""` → `find_snippet_pipe` returns `None`
- Edge case: caret lines `"   | ^^^^^"` (spaces-only prefix) → correctly matched
- Regression: all 8 existing `sec006_toml_multiline_redaction.rs` tests continue to pass

**Recording:** `AC-002-find-snippet-pipe-robustness.gif` / `.webm`

---

## AC-003 — Constant-Time X-Admin-Token in 4 DTU Clones (SEC-P3-003)

**Traces to:** BC-3.5.002 precondition 6, VP-107
**Finding:** LOW CWE-208 — `X-Admin-Token` comparison used short-circuit `!=` in 4 DTU clone admin handlers; theoretical timing oracle for local attackers.
**Fix:** `subtle::ConstantTimeEq::ct_eq` applied to byte slices at all 8 handler sites (reset + configure per clone). `subtle = "2"` added as workspace dependency.

**Call sites updated (8 total):**

| Crate | File | Handlers |
|-------|------|---------|
| prism-dtu-armis | `src/routes/dtu.rs` | `post_reset` (line 48) + `dtu_configure` (line 85) |
| prism-dtu-claroty | `src/routes/devices.rs` | `dtu_reset` (line 337) + `dtu_configure` (line 405) |
| prism-dtu-crowdstrike | `src/routes/mod.rs` | `dtu_reset` (line 47) + `dtu_configure` (line 76) |
| prism-dtu-slack | `src/routes/dtu.rs` | `post_reset` (line 43) + `dtu_configure` (line 80) |

**Command (grep):** `grep -rn 'ct_eq' crates/prism-dtu-{armis,claroty,crowdstrike,slack}/src`
**Expected:** 8 lines matching `ct_eq` (2 per clone × 4 clones)
**Observed:** 8 matching lines confirmed

**Command (tests):** `cargo test -p prism-dtu-armis -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-slack --features dtu --test dtu_reset_auth 2>&1 | grep 'test result'`
**Expected:** 4 lines: `test result: ok. 3 passed; 0 failed; 0 ignored`
**Observed:** All 4 suites: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s`

**Path coverage:**
- Success path: code inspection confirms `ct_eq` at all 8 sites
- Error path: existing AC-001/AC-002/AC-003 `dtu_reset_auth` tests cover absent token → 401, correct token → 200, cross-clone token → 401

**Recording:** `AC-003-constant-time-admin-token.gif` / `.webm`

---

## Artifacts Inventory

| File | Type | Size | Purpose |
|------|------|------|---------|
| `AC-001-inline-table-credential-redacted.tape` | VHS script | 1.2 KB | Reproducible recording source |
| `AC-001-inline-table-credential-redacted.gif` | GIF recording | 105 KB | PR embed |
| `AC-001-inline-table-credential-redacted.webm` | WebM recording | 171 KB | Archival |
| `AC-002-find-snippet-pipe-robustness.tape` | VHS script | 1.1 KB | Reproducible recording source |
| `AC-002-find-snippet-pipe-robustness.gif` | GIF recording | 88 KB | PR embed |
| `AC-002-find-snippet-pipe-robustness.webm` | WebM recording | 130 KB | Archival |
| `AC-003-constant-time-admin-token.tape` | VHS script | 1.4 KB | Reproducible recording source |
| `AC-003-constant-time-admin-token.gif` | GIF recording | 313 KB | PR embed |
| `AC-003-constant-time-admin-token.webm` | WebM recording | 1.4 MB | Archival |
| `evidence-report.md` | This file | — | Coverage mapping |

---

## Reproducibility Notes

- All tapes use `Hide/Sleep/Show` for the `cd` setup — the viewer sees only the demo command.
- AC-001 and AC-002 use `Sleep 15s` — `prism-customer-config` builds in under 1 second on warm cache; cold-cache compilation completes well within 15 seconds.
- AC-003 uses `Sleep 3s` for the grep (instant) and `Sleep 45s` for 4-crate `cargo test` (build + link for 4 crates; warm cache runs in ~12 seconds).
- Font: `FiraCode Nerd Font Mono` (confirmed installed at `/Users/jmagady/Library/Fonts/`).
- All recordings produced at 1200×700 with Dracula theme, 20px padding, 14px font.
