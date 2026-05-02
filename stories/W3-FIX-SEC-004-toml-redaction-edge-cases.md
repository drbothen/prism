---
story_id: W3-FIX-SEC-004
title: "prism-customer-config + DTU clones: TOML inline-table redaction and constant-time token comparison"
wave: 3.3
level: "L4"
target_module: prism-customer-config
subsystems: [SS-01, SS-06]
priority: P1
depends_on: []
blocks: []
estimated_days: 1
points: 3
status: draft
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-02T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review-pass3.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass3.md
  - .factory/specs/behavioral-contracts/BC-3.3.004-customer-config-startup-validation.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.3"
phase: 3
behavioral_contracts:
  - BC-3.3.004
  - BC-3.5.001
  - BC-3.5.002
verification_properties: [VP-105, VP-106, VP-107, VP-124, VP-126]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.3.004, BC-3.5.001, BC-3.5.002]
anchor_capabilities: [CAP-009, CAP-036]
anchor_subsystem: ["SS-01", "SS-06"]
tdd_mode: strict
parent_finding: "SEC-P3-001 (M, CWE-209), SEC-P3-002 (M, CWE-209), CR-019 (L), SEC-P3-003 (L, CWE-208)"
# BC status: anchored — all BCs fully authored
---

# W3-FIX-SEC-004: TOML inline-table redaction bypass + constant-time admin token comparison

## Narrative

As a Prism security reviewer, I want the two residual TOML credential-redaction bypass
vectors in `sanitize_error_message` closed and the `X-Admin-Token` comparison hardened
to constant-time, so that (a) a misconfigured inline-table TOML config cannot leak
credential values through `ConfigError` messages, and (b) the DTU admin token comparisons
offer no timing oracle to local attackers.

## Objective

Gate Step D pass-3 (`gate-step-d-security-review-pass3.md`) identified two MEDIUM
findings that were conditions for Phase 5 / production candidacy (SEC-P3-001, SEC-P3-002)
and one LOW finding that is a hygiene improvement for admin token comparisons (SEC-P3-003).
Gate Step C pass-3 identified the same pipe-matching defect (CR-019). All three are
bundled here:

| ID | Severity | CWE | One-line description |
|----|----------|-----|----------------------|
| SEC-P3-001 | MEDIUM | CWE-209 | Inline TOML table credentials (`credentials = { bearer_token = "x" }`) not detected by `sanitize_error_message` — outer field name `credentials` has no credential suffix |
| SEC-P3-002 / CR-019 | MEDIUM | CWE-209 | `find_snippet_pipe` uses `line.find(" \| ")` returning the first occurrence; a credential value containing ` \| ` causes false pipe match, offsetting field-name extraction and bypassing redaction |
| SEC-P3-003 | LOW | CWE-208 | `X-Admin-Token` comparison uses short-circuit string equality (`!=`) rather than constant-time comparison in 4 DTU clone admin handlers |

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.3.004 | Customer Config Validation Rejects Invalid Schema at Startup | Postcondition (On any validation failure) 2: all validation errors written to stderr; stdout is empty. Invariant 1: validation completes for all files before OrgRegistry is populated. The `sanitize_error_message` function protects this error output path from leaking credential values. SEC-P3-001/002 are residual bypasses of this protection (CWE-209). |
| BC-3.5.001 | Harness Logical Isolation Invariants | Invariant 3 (failure injection state scoped to target clone); the `admin_token` gate enforces the per-clone identity that supports this invariant. SEC-P3-003 is a theoretical weakening of that gate's timing properties. |
| BC-3.5.002 | Harness Network Isolation Invariants | Precondition 6 (each clone's auth middleware initialized with own `admin_token`); SEC-P3-003 affects the comparison that enforces this precondition for `POST /dtu/reset` and `POST /dtu/configure`. |

## Acceptance Criteria

### AC-001: SEC-P3-001 — `sanitize_error_message` detects inline-table credential values (traces to BC-3.3.004 postcondition 2)

In `crates/prism-customer-config/src/validator.rs`, the `sanitize_error_message` function
is updated to scan ALL ` = ` positions per snippet content line, not only the first
(outermost) assignment.

**Root cause:** For an inline-table snippet line such as:
```
  3 | credentials = { bearer_token = "my-secret", display_name = "ACME" }
```
The existing logic extracts `credentials` as the field name (before the first ` = `) and
checks `is_credential_pattern("credentials")` — which returns false because `credentials`
has no `_token`/`_secret`/`_key`/`_password`/`_pass` suffix. The inner `bearer_token`
is never examined.

**Fix:** After extracting the `content` substring (the part after the ` | ` snippet
separator), perform a whole-line credential scan in addition to the leading-field check:

```rust
// Scan all " = " positions in content, not only the first.
// This catches inline-table credentials: { bearer_token = "x", ... }
let mut search_pos = 0;
while let Some(eq_offset) = content[search_pos..].find(" = ") {
    let abs_eq = search_pos + eq_offset;
    // Find the start of this token (last non-token char before abs_eq)
    let token_start = content[..abs_eq]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|p| p + 1)
        .unwrap_or(0);
    let token = content[token_start..abs_eq].trim();
    if is_credential_pattern(token) {
        // Redact the remainder of this content line
        redacted_lines.push(format!("{prefix}[REDACTED]"));
        goto next_line; // (use flag/break pattern in idiomatic Rust)
    }
    search_pos = abs_eq + 3;
}
```

(Exact implementation may differ; the observable behavior requirement is the postcondition
below.)

After this fix:
- A TOML snippet line containing `credentials = { bearer_token = "my-secret" }` causes
  the entire content line to be replaced with `[REDACTED]` in the sanitized error message.
- A TOML snippet line containing `display_name = "ACME Corp"` is NOT redacted (no
  credential-pattern token before any ` = `).
- Single-line non-inline-table credential lines (`bearer_token = "x"`) continue to be
  redacted as before (existing behavior is preserved; no regression).

Tests: add at least two new test cases to
`crates/prism-customer-config/tests/sec006_toml_multiline_redaction.rs` (or a new sibling
test file `tests/sec_p3_001_inline_table_redaction.rs`):
- `test_inline_table_credential_redacted`: TOML snippet line with
  `credentials = { bearer_token = "secret" }` → redacted.
- `test_inline_table_non_credential_not_redacted`: TOML snippet line with
  `settings = { display_name = "ACME", timeout_seconds = 30 }` → NOT redacted.

### AC-002: SEC-P3-002 / CR-019 — `find_snippet_pipe` anchored to TOML numeric prefix (traces to BC-3.3.004 postcondition 2)

In `crates/prism-customer-config/src/validator.rs`, `find_snippet_pipe` is tightened to
verify that all characters before the ` | ` occurrence consist only of ASCII digits and
spaces (i.e., the TOML 0.8 line-number prefix format).

**Root cause:** `line.find(" | ")` returns the first occurrence of the three-byte sequence
anywhere in the line. A credential value containing ` | ` (e.g., `api_key = "top | secret"`)
causes the function to classify the position inside the value as the snippet separator,
extract a garbage "field name," and fail the credential pattern check — bypassing redaction.

**Fix (exact implementation):**

```rust
fn find_snippet_pipe(line: &str) -> Option<usize> {
    // TOML 0.8 snippet format: "  [digits] | content" or "   | ^^^^^" (caret line)
    // Only classify as a snippet line if everything before " | " is
    // ASCII digits and/or whitespace — the line-number prefix.
    // This prevents false matches when a credential value contains " | ".
    let pos = line.find(" | ")?;
    let prefix = &line[..pos];
    if prefix.chars().all(|c| c.is_ascii_digit() || c == ' ') {
        Some(pos)
    } else {
        None
    }
}
```

After this fix:
- `"  12 | api_key = \"abc | def\""` correctly identifies pipe at offset of `12 |` (before
  `api_key`), extracts `api_key` as the field name, and redacts the line.
- `"api_key = \"top | secret\""` (no digit prefix before any ` | `) → `find_snippet_pipe`
  returns `None` → line is not parsed as a snippet line; the raw-content scan in
  `scan_for_credentials` handles it separately.
- Caret lines `"   | ^^^^^"` (all spaces before ` | `) → matched correctly (spaces-only
  prefix is permitted).

Tests: add at least two new test cases:
- `test_find_snippet_pipe_value_with_pipe_not_confused`: a line
  `"  3 | api_key = \"abc | def\""` → pipe found at the `3 |` position, NOT at the inner
  ` | ` in the value.
- `test_find_snippet_pipe_no_digit_prefix_returns_none`: a line
  `"api_key = \"top | secret\""` → `find_snippet_pipe` returns `None`.

These tests may be unit tests on the private function (using `#[cfg(test)]` module) or
integration tests via `sanitize_error_message` with a crafted input.

### AC-003: SEC-P3-003 — `X-Admin-Token` comparisons use constant-time equality in all 4 DTU clones (traces to BC-3.5.002 precondition 6)

The `X-Admin-Token` string comparison in `POST /dtu/reset` and `POST /dtu/configure`
handlers for all four DTU clones is replaced with a constant-time comparison using the
`subtle` crate's `ConstantTimeEq` trait (or an equivalent constant-time byte-level
equality).

**Scope coverage table (SEC-P3-003) — complete list of call sites to update:**

| Crate | File | Handler | Endpoint | Pattern to replace |
|-------|------|---------|----------|--------------------|
| prism-dtu-armis | `src/routes/dtu.rs:76` | `post_reset` | `POST /dtu/reset` | `provided != Some(state.admin_token.as_str())` |
| prism-dtu-claroty | `src/routes/devices.rs:397` | `dtu_reset` | `POST /dtu/reset` | `provided != Some(state.admin_token.as_str())` |
| prism-dtu-crowdstrike | `src/routes/mod.rs:43` | `dtu_reset` | `POST /dtu/reset` | `provided != Some(state.admin_token.as_str())` |
| prism-dtu-slack | `src/routes/dtu.rs:76` | `post_reset` | `POST /dtu/reset` | `provided != Some(state.admin_token.as_str())` |

The `dtu_configure` handlers in the same files use the same pattern and MUST also be updated
(for consistency; omitting them would leave the configure path vulnerable while only hardening
reset).

Replacement pattern:

```rust
use subtle::ConstantTimeEq;

// Constant-time comparison (SEC-P3-003 / CWE-208): admin_token is a per-clone
// random UUID v4; constant-time prevents timing-based prefix enumeration.
let provided_bytes = provided.unwrap_or("").as_bytes();
let expected_bytes = state.admin_token.as_str().as_bytes();
let token_valid = provided_bytes.ct_eq(expected_bytes).into();
if !token_valid {
    return (StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid admin token"})))
        .into_response();
}
```

`subtle` MUST be added to the `Cargo.toml` workspace dependencies and to the `[dependencies]`
section of each affected crate.

After this fix:
- The four DTU clones perform constant-time byte comparison for `X-Admin-Token`; branch
  timing does not depend on the position of the first differing byte.
- The observable HTTP behavior is identical: mismatched or absent token → 401; matching
  token → next handler step.
- Existing tests (`dtu_reset_auth.rs` AC-001/AC-002/AC-003 per clone) continue to pass
  without modification.

Note on scope: the `subtle` crate comparison applies to both `dtu_reset` AND `dtu_configure`
handlers in each crate. If the `dtu_configure` handler uses the same pattern, it MUST also
be updated in the same PR. Do NOT leave `dtu_configure` using short-circuit equality while
`dtu_reset` uses constant-time equality — the inconsistency is itself a finding surface.

## Tasks

### Part A: Inline-table credential scan (SEC-P3-001)

1. Read `crates/prism-customer-config/src/validator.rs` lines 370-430 — locate the
   `sanitize_error_message` function and the existing `is_credential_pattern` call on
   the extracted field name.
2. Identify the exact code path that processes snippet content lines (the block after
   `find_snippet_pipe` returns `Some(pipe_pos)`).
3. After the existing leading-field check, add the multi-position scan loop (AC-001
   algorithm): iterate all ` = ` positions in `content`, extract the token before each,
   check `is_credential_pattern`. On first match, redact the entire content line and
   break to the next input line.
4. Write `test_inline_table_credential_redacted` in a test file (AC-001).
5. Write `test_inline_table_non_credential_not_redacted` (AC-001).
6. Run `cargo test -p prism-customer-config` — all existing tests pass; two new tests pass.

### Part B: `find_snippet_pipe` anchor fix (SEC-P3-002 / CR-019)

7. Read `crates/prism-customer-config/src/validator.rs` lines 429-435 — locate
   `find_snippet_pipe` and confirm the current `line.find(" | ")` implementation.
8. Replace the function body with the prefix-validation implementation (AC-002 exact code).
9. Write `test_find_snippet_pipe_value_with_pipe_not_confused` (AC-002).
10. Write `test_find_snippet_pipe_no_digit_prefix_returns_none` (AC-002).
11. Run `cargo test -p prism-customer-config` — all tests pass.
12. Verify the existing `sec006_toml_multiline_redaction.rs` tests all pass (no regression
    from the tightened pipe-finder).

### Part C: Constant-time admin token comparison (SEC-P3-003)

13. Add `subtle = "2"` to the workspace `Cargo.toml` dependencies section (or confirm it
    is already present as a workspace dependency).
14. Add `subtle = { workspace = true }` to the `[dependencies]` of:
    - `crates/prism-dtu-armis/Cargo.toml`
    - `crates/prism-dtu-claroty/Cargo.toml`
    - `crates/prism-dtu-crowdstrike/Cargo.toml`
    - `crates/prism-dtu-slack/Cargo.toml`
15. In each of the four `dtu.rs` / `devices.rs` / `mod.rs` files, add `use subtle::ConstantTimeEq;`
    at the top of the handler module.
16. Replace the `provided != Some(state.admin_token.as_str())` comparison in each
    `post_reset` / `dtu_reset` handler with the constant-time pattern (AC-003 exact code).
17. Apply the same replacement to `dtu_configure` handlers in the same files for consistency.
18. Run `cargo test -p prism-dtu-armis -p prism-dtu-claroty -p prism-dtu-crowdstrike
    -p prism-dtu-slack --all-features` — all existing `dtu_reset_auth.rs` tests pass.
19. Run `cargo clippy --workspace -- -D warnings` — no warnings about the `subtle` usage.

### Part D: Integration

20. Run `cargo test --workspace --all-features` — all tests pass.
21. Run `cargo clippy --workspace -- -D warnings` — zero new warnings.
22. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `sanitize_error_message` inline-table scan | prism-customer-config | `crates/prism-customer-config/src/validator.rs:370-430` | Pure (string transformation; no I/O) |
| `find_snippet_pipe` prefix validation | prism-customer-config | `crates/prism-customer-config/src/validator.rs:429-435` | Pure (string predicate; no I/O) |
| Armis `post_reset` + `dtu_configure` CT-eq | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/dtu.rs:76` | Pure (comparison change; no I/O) |
| Claroty `dtu_reset` + `dtu_configure` CT-eq | prism-dtu-claroty | `crates/prism-dtu-claroty/src/routes/devices.rs:397` | Pure (comparison change; no I/O) |
| CrowdStrike `dtu_reset` + `dtu_configure` CT-eq | prism-dtu-crowdstrike | `crates/prism-dtu-crowdstrike/src/routes/mod.rs:43` | Pure (comparison change; no I/O) |
| Slack `post_reset` + `dtu_configure` CT-eq | prism-dtu-slack | `crates/prism-dtu-slack/src/routes/dtu.rs:76` | Pure (comparison change; no I/O) |

**Subsystem anchor justification:**
- SS-06 (Client Configuration) owns the `prism-customer-config` changes (SEC-P3-001,
  SEC-P3-002): `sanitize_error_message` and `find_snippet_pipe` live in the customer
  config validation crate, which belongs to the Client Configuration subsystem per the
  ARCH-INDEX Subsystem Registry.
- SS-01 (Sensor Adapters) owns the DTU clone admin-token changes (SEC-P3-003): the four
  affected crates (`prism-dtu-armis`, `prism-dtu-claroty`, `prism-dtu-crowdstrike`,
  `prism-dtu-slack`) are Sensor Adapter subsystem crates. Both subsystems appear in the
  story's `subsystems:` frontmatter field.

**Dependency anchor justification:** `depends_on: []` — all three items are self-contained;
none requires any other W3.3 story. `blocks: []` — no downstream story depends on these
fixes before proceeding.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `sanitize_error_message` inline-table scan | pure-core | String transformation; no filesystem I/O; no network; no shared mutable state |
| `find_snippet_pipe` prefix validation | pure-core | String predicate function; purely functional; no I/O |
| DTU admin-token constant-time comparison | pure-core | Replaces one comparison operator with another; no I/O; no state mutation |
| Test functions (both sets) | effectful-shell | Spawn HTTP test servers or invoke pure functions in a test harness; marked `#[tokio::test]` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TOML snippet containing `credentials = { bearer_token = "secret" }` appears in parse error | Entire content line replaced with `[REDACTED]` in sanitized error; the outer `credentials` field name that previously passed the check is now caught by the inner scan |
| EC-002 | TOML snippet containing `settings = { timeout_seconds = 30, display_name = "ACME" }` | NOT redacted; neither `timeout_seconds` nor `display_name` match credential patterns |
| EC-003 | TOML snippet line `"  3 \| api_key = \"abc \| def\""` | `find_snippet_pipe` returns offset of `3 \|` (before `api_key`); inner ` \| ` in value ignored; `api_key` extracted; line redacted |
| EC-004 | Non-snippet line `"api_key = \"top \| secret\""` (no digit prefix) | `find_snippet_pipe` returns `None`; line handled by `scan_for_credentials` raw path (not affected by this fix) |
| EC-005 | Caret line `"   \| ^^^^^"` following a redacted snippet line | `find_snippet_pipe` returns the pipe position correctly (prefix is spaces only, which `is_ascii_digit() \|\| c == ' '` accepts); caret line is not itself a credential line |
| EC-006 | `dtu_reset` receives `X-Admin-Token` with correct value but wrong byte length | Constant-time comparison: `ct_eq` on slices of different length returns `Choice(0)` (false) without leaking which byte first differed |
| EC-007 | `dtu_configure` receives absent `X-Admin-Token` header | `provided_bytes` is `"".as_bytes()` (empty slice); CT comparison to non-empty `admin_token` returns false; HTTP 401 returned |
| EC-008 | Existing `dtu_reset_auth.rs` tests after the constant-time refactor | All 12 tests (AC-001/002/003 × 4 clones) pass without modification; behavioral contract unchanged |
| EC-009 | `subtle` crate not yet in workspace Cargo.toml | Task 13 adds it; `cargo check` confirms compilation before test run |
| EC-010 | Inline-table with multiple credential fields: `creds = { api_key = "a", api_secret = "b" }` | First matched token (`api_key`) triggers redaction of the full content line; both secrets are hidden; second credential field need not be individually matched once the line is already redacted |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5 500 |
| BC files (3 BCs) | ~6 500 |
| `validator.rs` lines 350-445 (sanitize_error_message + find_snippet_pipe) | ~1 200 |
| `tests/sec006_toml_multiline_redaction.rs` (existing tests, ~80 lines) | ~800 |
| `prism-dtu-armis/src/routes/dtu.rs` (lines 40-100, reset + configure) | ~700 |
| `prism-dtu-claroty/src/routes/devices.rs` (lines 380-420) | ~500 |
| `prism-dtu-crowdstrike/src/routes/mod.rs` (lines 35-55) | ~300 |
| `prism-dtu-slack/src/routes/dtu.rs` (lines 40-100) | ~700 |
| `Cargo.toml` workspace deps section (~30 lines) | ~300 |
| 4 crate `Cargo.toml` files (~10 lines each) | ~400 |
| New test files (4 inline-table + pipe tests, ~30 lines each) | ~600 |
| `cargo test` + `cargo clippy` output | ~1 000 |
| **Total** | **~18 500** |

Fits in a single agent context window. Load only the specific line ranges listed above
for source files; do not load entire crate sources.

## Previous Story Intelligence

- **W3-FIX-CODE-002** (PR #120 / SEC-006 closure): implemented `in_multiline_cred` state
  machine for triple-quoted credentials. SEC-P3-001/002 are residual bypass vectors in the
  same `sanitize_error_message` function — the triple-quote case was fixed but the inline-table
  and pipe-value cases were not covered by the initial scope.
- **W3-FIX-SEC-002** (PR #119 / SEC-NEW-001 closure): added `X-Admin-Token` gate to all four
  DTU clone `dtu_reset` handlers. SEC-P3-003 is a theoretical hardening of that same
  comparison — the gate is functionally correct but uses short-circuit equality. The `subtle`
  crate addition follows the same pattern used in other Rust projects for token comparison
  hardening.
- **Lesson:** When implementing a security fix (SEC-006 redaction), write test cases that
  explicitly probe the bypass scenarios identified in the security review, not just the
  happy-path fix. SEC-P3-001 and SEC-P3-002 both represent bypass cases that were noted in
  the pass-3 security review's "bypass analysis" subsection but not included in the
  acceptance criteria of the original SEC-006 story.

## Architecture Compliance Rules

- The inline-table scan in `sanitize_error_message` MUST NOT change the behavior for
  single-level (non-inline-table) credential lines — those are the primary fix from SEC-006
  and must continue to work correctly. Run the full `sec006_toml_multiline_redaction.rs`
  test suite to verify no regression.
- `find_snippet_pipe` MUST accept caret lines (`"   | ^^^^^"`) — all-spaces prefix is
  a valid TOML 0.8 snippet prefix format. The `is_ascii_digit() || c == ' '` predicate
  correctly handles this.
- `find_snippet_pipe` MUST reject lines where the first ` | ` occurrence is preceded by
  alphabetic characters — these are field-value lines, not snippet lines.
- The `subtle::ConstantTimeEq` comparison MUST be applied to both `dtu_reset` AND
  `dtu_configure` handlers in each affected crate. Applying it to only one creates an
  asymmetry that is itself a future finding surface.
- Do NOT use `secrecy::ExposeSecret` as the primary comparison mechanism — `subtle::ct_eq`
  on byte slices is the correct primitive. `secrecy` may be used elsewhere in the codebase
  for secret storage but is not the right tool for comparison.
- `subtle = "2"` MUST be a workspace dependency (added to the root `Cargo.toml` `[workspace.dependencies]`
  section); individual crate `Cargo.toml` files use `subtle = { workspace = true }`.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `subtle` | `"2"` (new workspace dep) | `ConstantTimeEq` trait for admin token byte comparison (SEC-P3-003) |
| `axum` | workspace pin | Handler extractors in DTU admin routes |
| `serde_json` | workspace pin | `json!()` macro in 401 response bodies |

No other new Cargo dependencies.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-customer-config/src/validator.rs` | Modify | Lines 370-435: inline-table scan + `find_snippet_pipe` anchor fix (AC-001, AC-002) |
| `crates/prism-customer-config/tests/sec_p3_001_inline_table_redaction.rs` | Create | 2+ new tests for inline-table redaction (AC-001) |
| `crates/prism-customer-config/tests/sec_p3_002_pipe_anchor.rs` | Create | 2+ new tests for `find_snippet_pipe` anchor (AC-002) |
| `crates/prism-dtu-armis/src/routes/dtu.rs` | Modify | Line ~76: constant-time comparison in `post_reset` and `dtu_configure` (AC-003) |
| `crates/prism-dtu-claroty/src/routes/devices.rs` | Modify | Line ~397: constant-time comparison in `dtu_reset` and `dtu_configure` (AC-003) |
| `crates/prism-dtu-crowdstrike/src/routes/mod.rs` | Modify | Line ~43: constant-time comparison in `dtu_reset` and `dtu_configure` (AC-003) |
| `crates/prism-dtu-slack/src/routes/dtu.rs` | Modify | Line ~76: constant-time comparison in `post_reset` and `dtu_configure` (AC-003) |
| `crates/prism-dtu-armis/Cargo.toml` | Modify | Add `subtle = { workspace = true }` to `[dependencies]` |
| `crates/prism-dtu-claroty/Cargo.toml` | Modify | Add `subtle = { workspace = true }` to `[dependencies]` |
| `crates/prism-dtu-crowdstrike/Cargo.toml` | Modify | Add `subtle = { workspace = true }` to `[dependencies]` |
| `crates/prism-dtu-slack/Cargo.toml` | Modify | Add `subtle = { workspace = true }` to `[dependencies]` |
| `Cargo.toml` (workspace root) | Modify | Add `subtle = "2"` to `[workspace.dependencies]` |

## Forbidden Dependencies

- Do NOT use `ring` or `openssl` for the constant-time comparison — `subtle` is the
  correct minimal dependency for this purpose.
- Do NOT add `subtle` directly to crate `Cargo.toml` with a pinned version; use the
  workspace dependency pattern so all four crates share the same pin.
- Do NOT modify `sanitize_error_message` to redact ALL lines containing ` = ` — only
  lines where a token before some ` = ` matches `is_credential_pattern`. Redacting
  non-credential fields degrades the usefulness of TOML parse error diagnostics.
- Do NOT change the `is_credential_pattern` function's matching logic (suffix list) as
  part of this story — extending the suffix list is a separate concern that requires
  its own review.
- Do NOT remove or weaken the existing `in_multiline_cred` state machine for triple-quoted
  credentials introduced in SEC-006 / W3-FIX-CODE-002.
