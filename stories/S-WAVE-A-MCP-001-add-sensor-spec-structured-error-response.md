---
document_type: story
story_id: S-WAVE-A-MCP-001
title: "Remap add_sensor_spec ValidationFailed to BC-2.10.007 Structured Error Envelope"
version: "1.0"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P1
points: 5
tdd_mode: strict
target_module: prism-mcp
subsystems: ["SS-10 (MCP)"]
depends_on:
  - S-WAVE-A-ENGINE-001    # Rule 9 must be live so E-SPEC-027 error codes appear in ValidationFailed
                           # responses; this gives the wire-shape tests real E-SPEC codes to assert on
blocks: []
behavioral_contracts:
  - BC-2.16.008
  - BC-2.10.007
verification_properties: []
estimated_days: 2
# BC status: BC-2.16.008 v1.6 contains the authorized contract for this story at §Error Conditions
# (the "Authorized MCP contract (S-WAVE-A-MCP-001)" paragraph). Both BCs are active. No PO amendment
# required before dispatch — the authorization is already in place.
# NOTE: Story ID S-WAVE-A-MCP-001 is FIXED. BC-2.16.008 §Error Conditions v1.6 binds a normative
# MUST to the literal string "S-WAVE-A-MCP-001". Do NOT rename this story.
assumption_validations: []
risk_mitigations: []
---

# S-WAVE-A-MCP-001: Remap add_sensor_spec ValidationFailed to BC-2.10.007 Structured Error Envelope

## CRITICAL — Story ID is Normatively Fixed

**Do not rename this story.** BC-2.16.008 §Error Conditions v1.6 contains this clause:

> "Authorized MCP contract (S-WAVE-A-MCP-001): when S-WAVE-A-MCP-001 is delivered,
> add_sensor_spec MUST emit isError: true + structuredContent.error.code +
> structuredContent.error.errors"

The string `S-WAVE-A-MCP-001` appears in a normative MUST in an active behavioral contract.
Renaming the story would leave a dangling normative reference in BC-2.16.008.

---

## Authority

- **BC-2.16.008 v1.6** — §Error Conditions "Authorized MCP contract" paragraph
- **BC-2.10.007 v1.19** — Wire shape definition for all structured error responses
- **ADR-053 §D6 (Option B)** — Deferred ValidationFailed → BC-2.10.007 remap to this story

---

## Narrative

As an LLM agent using the `add_sensor_spec` MCP tool, I want spec validation failures to
emit `isError: true` with a `structuredContent.error` object (code, message, errors array,
category, retryable) so that my error-handling logic can distinguish spec validation
failures from sensor API errors and act on the specific validation error codes, rather
than receiving a success-shaped JSON object with `"status": "validation_failed"` that
bypasses my error-handling branch.

---

## Current Behavior vs Target Behavior

### Current behavior (pre-S-WAVE-A-MCP-001)

The `ValidationFailed` arm in `server.rs` (function `add_sensor_spec`) currently returns:

```json
{
  "status": "validation_failed",
  "errors": ["E-SPEC-001: base_url must start with http:// or https://", "..."]
}
```

This is a success-shaped response (no `isError: true`). The LLM agent receives it as a
successful tool call result, not as an error. The agent's error-handling branch does not
fire. The `errors` array is present but nested inside a success envelope that an agent
expecting `structuredContent.error` will not parse.

### Target behavior (this story)

```json
{
  "isError": true,
  "content": [
    {
      "type": "text",
      "text": "ERROR: [validation] - Sensor spec validation failed: 2 error(s). Review the errors array and correct the spec."
    }
  ],
  "structuredContent": {
    "error": {
      "code": "E-SPEC-001",
      "message": "Sensor spec validation failed: N error(s).",
      "category": "validation",
      "retryable": false,
      "retry_after_seconds": null,
      "suggestion": "Review the errors array and correct the spec TOML before resubmitting.",
      "source": "prism_mcp",
      "original_params_valid": false,
      "upstream_message": null,
      "errors": ["E-SPEC-001: base_url must start with http:// or https://", "..."]
    },
    "_meta": {
      "trust_level": "internal"
    }
  }
}
```

The `errors` field is a non-standard extension to BC-2.10.007's standard shape, authorized
by BC-2.16.008 v1.6. It preserves the collect-all semantics established by VP-059 and
ADR-055.

---

## Acceptance Criteria

### AC-001: ValidationFailed emits isError: true
(traces to BC-2.16.008 postcondition "Authorized MCP contract" — add_sensor_spec MUST emit
isError: true for validation failures after S-WAVE-A-MCP-001 delivers)

A wire-level test that submits an invalid TOML via `add_sensor_spec` (e.g., `base_url =
"ftp://x"`) asserts that the MCP response JSON has `isError: true` at the top level.

### AC-002: structuredContent.error.code is present and is an E-SPEC-NNN code
(traces to BC-2.16.008 postcondition — structuredContent.error.code is required)

The response JSON has `structuredContent.error.code` set to the primary spec error code
from the validation failures (e.g., `"E-SPEC-001"` for a base_url violation).

Implementation note: the `code` field should be derived from the first error in the errors
array (strip the trailing message, keep the `E-SPEC-NNN` prefix). If the errors list
contains errors from multiple rules, `code` is the code of the first error; the full list
is in `errors`. The implementer MUST verify the exact code derivation logic against the
error taxonomy and BC-2.16.008 — do not invent a general `"E-SPEC-VALIDATION"` code that
does not exist in the taxonomy.

### AC-003: structuredContent.error.errors array contains all validation errors
(traces to BC-2.16.008 postcondition — structuredContent.error.errors preserves collect-all)

A spec with TWO Rule-1–5 violations (e.g., `base_url = "ftp://x"` AND a dangling variable
reference) produces a response with `structuredContent.error.errors` containing AT LEAST
two entries. Collect-all semantics are preserved end-to-end to the MCP wire level.

### AC-004: Wire-shape negative test — "status": "validation_failed" is ABSENT
(traces to BC-2.10.007 postcondition — all validation errors use the structured error shape)

The wire-level response for a ValidationFailed call does NOT contain a top-level
`"status"` key with value `"validation_failed"`. The old shape is no longer emitted.

At least one test must assert `response["status"]` is absent OR `response.get("status") !=
Some("validation_failed")` in the serialized JSON output (SID-2: composed-output assertion).

### AC-005: BC-2.10.007 fields are complete — all required fields present
(traces to BC-2.10.007 §Complete field specification — all 8 required fields must be present)

A wire-level test asserts that `structuredContent.error` contains all 8 required fields:
`code`, `message`, `category`, `retryable`, `retry_after_seconds`, `suggestion`, `source`,
`original_params_valid`, `upstream_message`. The test must check each field individually.

Field values for ValidationFailed:
- `category`: `"validation"`
- `retryable`: `false`
- `retry_after_seconds`: `null` (not absent — must be explicitly null)
- `source`: `"prism_mcp"`
- `original_params_valid`: `false` (bad spec parameters caused the error)
- `upstream_message`: `null` (no upstream sensor contacted)

### AC-006: Existing test(s) expecting "validation_failed" status updated
(traces to BC-2.16.008 postcondition — the as-built contract changes at this story's delivery)

Any existing test in `crates/prism-mcp/` that asserts `response["status"] == "validation_failed"`
is updated to assert `isError == true` and the new structured error shape. The test update
must be in the same commit as the implementation change — no existing test is allowed to
remain asserting the old (pre-S-WAVE-A-MCP-001) wire shape after this story merges.

### AC-007: Other add_sensor_spec result arms are NOT changed
(traces to BC-2.16.008 invariant — only the ValidationFailed arm changes; Added, ConfirmationRequired,
DryRun, WriteError arms are out of scope)

A test verifies that submitting a valid TOML via `add_sensor_spec` (happy path) still
returns a success response with `"status": "added"` (or equivalent current success shape).
The ValidationFailed remap must not affect the Added, ConfirmationRequired, DryRun, or
WriteError arms.

---

## Architecture Mapping

| Component | File | Pure/Effectful | Change |
|-----------|------|---------------|--------|
| `add_sensor_spec()` MCP tool handler | `crates/prism-mcp/src/server.rs` | Effectful (MCP tool dispatch) | Change ValidationFailed arm to call `to_error_data()` (or equivalent) |
| `prism_error_to_structured_call_result` | `crates/prism-mcp/src/error_mapping.rs` | Pure (error mapping) | MAY need a new arm for `PrismError::SpecValidation` variant; TBD — see T-02 |

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.16.008 | v1.6 | §Error Conditions "Authorized MCP contract" paragraph — this story fulfills the authorization |
| BC-2.10.007 | v1.19 | Wire shape definition; §Complete field specification; `errors` array extension is non-standard but authorized by BC-2.16.008 v1.6 |

---

## Implementation Notes

### How to produce isError: true — Option B is the correct approach

**Decision: Option B (direct construction in server.rs). This is not open latitude.**

Two approaches exist; both produce the same BC-2.10.007 wire shape if correctly implemented.
The choice is structural (affects the Rust type system) not behavioral (identical wire output).

**Option A (rejected):** Create a new `PrismError::SpecValidationFailed` variant and route
through `prism_error_to_structured_call_result` in `error_mapping.rs`. Rejected because:
- `AddSensorSpecResult::ValidationFailed` is NOT a `PrismError` — it is a successful
  spec-engine call that found validation errors. Forcing it into `PrismError` creates a
  semantic mismatch in the type system.
- Option A requires: new `PrismError` variant + sentinel test update + dedicated arm in
  `error_mapping.rs`. Three moving parts, any one of which failing causes the BC-2.10.007
  catch-all to fire and produce a different (incorrect) wire shape.
- The `errors` array extension is non-standard and specific to this one response. It should
  NOT propagate through the canonical error-mapping pipeline.

**Option B (chosen):** Construct the structured error JSON directly in the ValidationFailed
arm in `server.rs` and return via `return Err(...)` without a new `PrismError` variant.

**Pre-condition gate (implementer must verify before coding):** Confirm that the MCP
framework's error path supports returning a custom structured JSON body that produces
`isError: true` without calling `to_error_data(PrismError)`. Inspect `rmcp`'s
`CallToolResult` type and the `to_error_data` function signature. If `isError: true` is
ONLY achievable via `to_error_data(PrismError)`, then Option B is infeasible and Option A
must be used instead. Document the finding in the commit message with the specific type or
function that prevents Option B. If Option A is forced, also update this story's architecture
compliance rules section accordingly.

Assuming Option B is viable, the implementation pattern is:

```rust
prism_spec_engine::types::AddSensorSpecResult::ValidationFailed { errors } => {
    let all_errors: Vec<&str> = errors
        .iter()
        .flat_map(|e| e.errors.iter().map(|s| s.as_str()))
        .collect();
    let first_code = all_errors
        .first()
        .and_then(|s| s.split(':').next())
        .unwrap_or("E-SPEC-001");
    let structured = serde_json::json!({
        "code": first_code,
        "message": format!("Sensor spec validation failed: {} error(s).", all_errors.len()),
        "category": "validation",
        "retryable": false,
        "retry_after_seconds": null,
        "suggestion": "Review the errors array and correct the spec TOML before resubmitting.",
        "source": "prism_mcp",
        "original_params_valid": false,
        "upstream_message": null,
        "errors": all_errors,
    });
    // Return via the MCP error path that sets isError: true — exact call TBD from rmcp API
    return Err(/* rmcp error construction from structured */);
}
```

The implementer must fill in the `/* rmcp error construction */` by reading the
`CallToolResult` API in `rmcp`. The `_meta.trust_level: "internal"` field is expected to
be injected by the framework layer, not manually.

### The errors array extension

BC-2.10.007's standard error shape has no `errors` array field. The authorized extension
(`structuredContent.error.errors`) is non-standard but required by BC-2.16.008 v1.6 to
preserve collect-all semantics. When building the error JSON:

```rust
serde_json::json!({
    "code": first_error_code,
    "message": format!("Sensor spec validation failed: {} error(s).", error_count),
    "category": "validation",
    "retryable": false,
    "retry_after_seconds": null,
    "suggestion": "Review the errors array and correct the spec TOML before resubmitting.",
    "source": "prism_mcp",
    "original_params_valid": false,
    "upstream_message": null,
    "errors": all_error_strings,  // Non-standard extension — authorized by BC-2.16.008 v1.6
})
```

The `_meta.trust_level: "internal"` field is added by the MCP framework layer, not
manually — verify this against how other error responses are assembled.

---

## UX / Operator Impact

The `add_sensor_spec` MCP tool's error response shape changes for validation failures.
LLM agent callers that pattern-match on `response.status == "validation_failed"` (the
old shape) will no longer find that key. They should instead check `response.isError == true`.

No human-visible UI change. The LLM agent's error-handling logic may need updating by
consumers of the prism MCP server.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ValidationFailed with zero errors (empty errors vec) | Should not occur (validate_sensor_spec returns Err only with ≥1 error), but if it does, emit errors: [] and code = "E-SPEC-000" or the taxonomy's general spec error code |
| EC-002 | ValidationFailed with errors from multiple different E-SPEC-NNN codes | AC-002: code = first error's code; AC-003: all codes in errors array |
| EC-003 | Valid TOML submitted — happy path | AC-007: response has "status": "added" (or current success shape) — unchanged |
| EC-004 | retry_after_seconds field is absent (not null) in the response | AC-005 assertion catches this — field MUST be null, not absent; per BC-2.10.007 §retry_after_seconds note |
| EC-005 | E-SPEC-027 errors (Rule 9 from ENGINE-001) appear in errors array | The same structured error shape applies; AC-003 covers multi-error cases |

---

## Tasks

### T-01: Read the current ValidationFailed arm in server.rs
**File:** `crates/prism-mcp/src/server.rs` — function `add_sensor_spec()`, ValidationFailed match arm

Read lines around the `ValidationFailed { errors }` arm. Understand:
1. How the `SafetyEnvelopeBuilder::wrap()` path works (success path)
2. How the `return Err(to_error_data(...))` path works (error path — WriteError arm)
3. How `to_error_data()` maps to `isError: true` in the wire shape

Do NOT implement from memory. Read the actual code.

### T-02: Determine Option A vs Option B (TBD at implementation time)
**Scope:** `crates/prism-mcp/src/error_mapping.rs`, `crates/prism-core/src/error.rs`

Determine whether to add a `PrismError::SpecValidationFailed` variant (Option A) or
construct the error JSON directly in server.rs (Option B). The decision depends on:
- Whether `to_error_data()` is the only path that produces `isError: true` in the wire
- Whether Option A requires updating the non-exhaustive gate (CLAUDE.md: EXPECTED=92)
- Whether the sentinel in `tests/error_category_coverage.rs` needs updating

Document the decision in the commit message and in a code comment at the change site.

### T-03: Update existing ValidationFailed tests to assert new shape
**Scope:** grep `crates/prism-mcp/` for `"validation_failed"` in test assertions

For each test asserting the old shape (`"status": "validation_failed"`):
- Update the assertion to check `isError == true` and `structuredContent.error.code` is an E-SPEC-NNN code
- Add wire-shape assertion per CLAUDE.md §Wire-shape assertion discipline

### T-04: Write AC-001/AC-002/AC-003 wire-shape tests
**File:** `crates/prism-mcp/tests/` (or existing MCP integration test file)

Write tests that:
1. Submit invalid TOML via `add_sensor_spec()` (after serialization to MCP CallToolResult)
2. Serialize the result to JSON
3. Assert on the SERIALIZED JSON output — not on pre-serialization Rust structs

Specifically, serialize to `serde_json::Value` and check:
- `response["isError"] == true`
- `response["structuredContent"]["error"]["code"]` matches `"E-SPEC-NNN"` pattern
- `response["structuredContent"]["error"]["errors"]` is a JSON array with ≥1 entry
- `response.get("status")` is None (old field absent)

### T-05: Write AC-005 field completeness test
Write a test that checks each of the 8 required fields in `structuredContent.error` is
present and has the correct type (null vs absent, bool vs string). Per SID-2: at least one
test must assert on the FULL composed error object, not just on component fields.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| `crates/prism-mcp/src/server.rs` (add_sensor_spec function area) | ~3,500 |
| `crates/prism-mcp/src/error_mapping.rs` | ~2,500 |
| `crates/prism-core/src/error.rs` (PrismError enum) | ~2,000 |
| BC-2.10.007 (wire shape spec) | ~2,500 |
| BC-2.16.008 (authority for authorized contract) | ~1,500 |
| Existing MCP tests to update | ~1,500 |
| Error taxonomy (E-SPEC-NNN codes) | ~1,000 |
| Running test output (nextest) | ~1,500 |
| **Total estimate** | **~19,000** |

19,000 tokens is within the 20–30% threshold for a standard 100k-token context. No split required.

---

## Previous Story Intelligence

N/A — first story scoped to prism-mcp in this wave.

Lessons from prism-mcp cascades:
- Wire-shape assertions (CLAUDE.md §Wire-shape assertion discipline, D-1715): MCP tool
  response tests that assert only on Rust structs miss wire-level bugs. Always serialize
  to JSON and assert on the serialized bytes.
- SID-2: The composed output assertion (`"status": "validation_failed"` absent AND
  `"isError": true` present AND full error object populated) must be a single test, not
  three separate partial assertions. Prior escape: message and suggestion were each
  individually asserted but never tested in composition.
- `retry_after_seconds: null` must be explicitly present in JSON — not absent. Use
  `serde(default)` or explicit `Option<u64>` with `#[serde(serialize_always)]` to ensure
  null is emitted, not omitted.

---

## Architecture Compliance Rules

1. **BC-2.10.007 §retry_after_seconds.** Field MUST be `null` (explicitly present) when
   no retry delay applies — not absent. Assert in AC-005.

2. **BC-2.10.007 §Rule 2 catch-all.** If Option A (new `PrismError` variant) is chosen,
   the variant MUST be added to both the sentinel test AND a dedicated arm in
   `prism_error_to_structured_call_result` before the PR merges. The catch-all is
   reserved for FUTURE/unknown variants only (ZERO currently-known variants fall to it as
   of v1.18).

3. **CLAUDE.md §Non-exhaustive gate.** If Option A adds a `PrismError` variant to
   `prism-core` or `prism-spec-engine`, verify the `#[non_exhaustive]` gate applies.
   Current EXPECTED=92; bumping requires updating `scripts/check-non-exhaustive.sh`,
   `CLAUDE.md`, and `scripts/check-non-exhaustive-per-symbol.py` in the same commit.

4. **ADR-055 §D1 collect-all.** The `errors` array extension to the error shape must
   include ALL validation errors from the collect-all pass, not just the first one.

5. **CLAUDE.md §Wire-shape assertion discipline.** Every test covering the ValidationFailed
   arm must assert on the serialized JSON output — not only on Rust struct fields.

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `serde_json` | pinned in workspace `Cargo.toml` | `architecture/dependency-graph.md §External Dependencies` |
| `rmcp` (MCP SDK) | pinned in workspace `Cargo.toml` | same |

No new external dependencies.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-mcp/src/server.rs` | MODIFY | T-01/T-02: change ValidationFailed arm; `return Err(...)` path |
| `crates/prism-mcp/src/error_mapping.rs` | MODIFY (Option A) or NO CHANGE (Option B) | T-02: new PrismError variant arm if Option A chosen |
| `crates/prism-core/src/error.rs` | MODIFY (Option A) or NO CHANGE (Option B) | T-02: new PrismError variant if Option A chosen |
| `crates/prism-mcp/tests/` | MODIFY/ADD | T-03/T-04/T-05: update old tests + new wire-shape tests |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-25 | story-writer | Initial stub; authority: BC-2.16.008 v1.6 + ADR-053 §D6 Option B; normative ID constraint documented; TBD sections marked for implementation-time resolution |
