---
document_type: architecture-scoping-correction
version: "1.0"
status: active
producer: architect
timestamp: "2026-06-21T00:00:00Z"
traces_to: [S-DEMO-PRISMQL-ONBOARDING-001-B]
inputs:
  - crates/prism-mcp/src/server.rs
  - crates/prism-mcp/src/safety_envelope.rs
  - .factory/specs/behavioral-contracts/BC-2.10.007-structured-error-responses.md
  - .factory/stories/S-DEMO-PRISMQL-ONBOARDING-001-B-query-engine-l4-errors-normalized-pql.md
routing: story-writer applies edits to 001-B only; no BC amendment required; ci.yml EXPECTED unchanged
---

# Onboarding-001-B normalized_pql Envelope Correction

## Purpose

Resolve CORRECTION-1 flagged during the pre-TDD remove-uncertainty re-validation of
S-DEMO-PRISMQL-ONBOARDING-001-B: the Phase 5 task and File Structure row instruct
adding a typed query-response struct with `normalized_pql: Option<String>` to
`server.rs`, but no such typed struct exists in the codebase. This adjudication
documents the correct mechanism, eliminates the invalid instruction, and fixes the
downstream ripples in the story (risk_mitigations wording, ci.yml coordination note,
Architecture Mapping row).

---

## 1. Confirmed Ground Truth (verified against develop@fc954300)

### 1.1 There is no typed query-response struct in prism-mcp

`PrismServer::query` in `crates/prism-mcp/src/server.rs` builds its response as an
INLINE `serde_json::json!({ "rows", "returned_results", "total_available",
"is_truncated" })` `Value`, passes it to `SafetyEnvelopeBuilder::wrap(...)`, and
returns `CallToolResult::structured(envelope_val)`.

`SafetyEnvelopeBuilder::wrap` accepts `results: serde_json::Value` (the opaque
payload) and returns `ResponseEnvelope`. `ResponseEnvelope` is defined in
`crates/prism-mcp/src/safety_envelope.rs` and is already `#[non_exhaustive]`. It is
NOT a query-specific response struct — it is the generic safety-envelope type shared
by every tool handler.

There is no `QueryResponse`, `QueryResult`, or any other pub struct that represents
the `query` tool's success payload. The only query-related struct in `server.rs` is
`QueryToolParams` (the INPUT parameters, not the output).

This is the established BC-2.10.007 SafetyEnvelope response pattern. It predates
001-A and is not a 001-A regression.

### 1.2 ADR-022 "wiring not redesign" decision

**Decision: add `"normalized_pql"` conditionally to the existing inline `json!{...}`
payload. Do NOT introduce a new typed query-response struct.**

Rationale under ADR-022:
- Introducing a new pub typed struct to hold the query payload would be a REDESIGN of
  the SafetyEnvelope response pattern that every other tool handler uses. It would
  require: (a) replacing the inline `json!` call with a typed struct instantiation,
  (b) adding serde derives + `#[non_exhaustive]` to the new struct, (c) converting it
  to `serde_json::Value` before passing to `SafetyEnvelopeBuilder::wrap` — three
  changes to existing working infrastructure.
- The additive alternative — conditionally inserting `"normalized_pql"` into the
  existing `serde_json::json!({...})` call — requires ZERO changes to the SafetyEnvelope
  pattern, ZERO new pub types, and ZERO new `#[non_exhaustive]` annotations. It is pure
  wiring into an already-established inline `Value` construction site.

ADR-022 "wiring not redesign" clause: adding a key to an existing inline `serde_json::json!`
payload is wiring. Replacing the payload construction strategy with a typed struct is
redesign. The wiring path is canonical.

### 1.3 How "absent-on-error, present-on-success" semantics work with inline Value

The `serde_json::skip_serializing_if = "Option::is_none"` directive is a serde struct
attribute — it has no meaning for an inline `serde_json::json!({...})` call. The
equivalent for a `serde_json::Value` payload is CONDITIONAL KEY INSERTION:

**Pattern A — use `json!` with a conditional value:**
```rust
// In PrismServer::query, after obtaining normalized_pql_str: Option<String>
let mut payload = serde_json::json!({
    "rows": rows,
    "returned_results": result.returned_results,
    "total_available":  result.total_available,
    "is_truncated":     result.is_truncated,
});
if let Some(ref normalized) = normalized_pql_str {
    payload["normalized_pql"] = serde_json::Value::String(normalized.clone());
}
```

**Pattern B — use `serde_json::Map` construction for clarity (equally valid).**

Either pattern produces a JSON object where `"normalized_pql"` is ABSENT (not present,
not null) when the Option is None. The absent-on-error guarantee is naturally enforced
because the success branch is the only code path that populates `normalized_pql_str`:
on the error path, `PrismServer::query` returns early via
`return Ok(prism_error_to_structured_call_result(domain_err))` before the `json!{...}`
payload is constructed.

AC-006 test using `serde_json::Value` deserialization and
`value.get("normalized_pql").is_none()` correctly validates this pattern — the
`serde_json::Value` representation of the response will have no `"normalized_pql"` key
when the field is absent.

### 1.4 Non-exhaustive count impact: +0

Because no new pub struct is introduced:
- No new `#[non_exhaustive]` annotation is added to any type in `prism-mcp`.
- `ci.yml EXPECTED` stays at 82 (the count set by 001-A, which has already merged).
- The "coordinate with 001-A on final EXPECTED count" note in the Phase 5 task is
  now moot — 001-A is merged, EXPECTED is 82, and 001-B adds +0.
- `scripts/check-non-exhaustive.sh` does NOT need updating.

### 1.5 Red Gate test design confirmation

Test 5 (`test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error`) in crate
`prism-mcp` deserializes the response as `serde_json::Value` and asserts:
- On success: `value["structuredContent"]["results"]["normalized_pql"].as_str()` is
  `Some(non_empty_string)` — OR, more precisely, the test should navigate the
  `ResponseEnvelope` structure to the `results` field (which is the `Value` payload
  passed to `wrap`), then call `.get("normalized_pql")`.
- On error: the response has `isError: true` and does NOT reach the payload
  construction code path; the AC-006 assertion `value.get("normalized_pql").is_none()`
  runs against the top-level response object, which will have no such key since the
  error path returns via `prism_error_to_structured_call_result` before the payload
  is built.

Both assertions work correctly against the inline-json payload pattern described in
§1.3. The test design in the story (AC-005, AC-006, Red Gate row 5) requires no
change in semantics — only the implementation mechanism changes from "typed struct
field" to "conditional Value key insertion."

---

## 2. Per-Story Edit List for Story-Writer

The following edits MUST be applied to
`S-DEMO-PRISMQL-ONBOARDING-001-B-query-engine-l4-errors-normalized-pql.md`.

### 2.1 Phase 5 task — "Add normalized_pql field" step

**Current text (in Tasks Phase 5 "normalized_pql Chumsky re-serializer"):**

> `- [ ] Add `normalized_pql: Option<String>` field to query response type in
> `crates/prism-mcp/src/server.rs`:`
> `  - Response type MUST be `#[non_exhaustive]`; add `#[non_exhaustive]` if not
> already present`
> `  - If adding `#[non_exhaustive]` to query response type: increment `ci.yml
> EXPECTED` by 1 more`
> `    (beyond the +3 from 001-A; coordinate with 001-A on final EXPECTED count)`
> `  - Field serialized with `#[serde(skip_serializing_if = "Option::is_none")]` —
> absent on error, not null`
> `  - Field presence rules: PRESENT (non-empty) on every successful execution
> (incl. zero-row,`
> `    partial sensor failure with query-level success); ABSENT on ALL error responses`

**Replace with:**

> `- [ ] Insert `normalized_pql` into the existing inline `json!` payload in`
> `  `PrismServer::query` (`crates/prism-mcp/src/server.rs`), after the rows`
> `  serialization block and before `SafetyEnvelopeBuilder::wrap` is called:`
> `  - Pattern: construct `payload` as `serde_json::json!({...existing keys...})`
> `    then conditionally insert: `if let Some(ref s) = normalized_pql_str {`
> `    payload["normalized_pql"] = serde_json::Value::String(s.clone()); }`
> `  - NO typed query-response struct is needed or permitted — the query handler`
> `    uses an inline `serde_json::Value` payload per the BC-2.10.007 SafetyEnvelope`
> `    pattern (shared by all tool handlers; ADR-022 wiring-not-redesign).`
> `  - `#[serde(skip_serializing_if)]` is a struct attribute and does NOT apply to`
> `    `serde_json::Value`. The correct equivalent is conditional key insertion (above)`
> `    — when the key is absent from the `Value`, it is absent from the JSON output.`
> `  - Field presence rules (UNCHANGED from BC-2.11.018): PRESENT (non-empty) on every`
> `    successful execution (incl. zero-row, partial sensor failure with query-level`
> `    success); ABSENT on ALL error responses. The absent-on-error guarantee is`
> `    enforced structurally — on the error path, `PrismServer::query` returns early`
> `    via `prism_error_to_structured_call_result` before `payload` is constructed.`
> `  - NO `#[non_exhaustive]` annotation is needed (no new pub struct); ci.yml`
> `    EXPECTED stays 82 (set by 001-A which has already merged).`

### 2.2 Phase 6 final gates — ci.yml EXPECTED step

**Current text:**

> `- [ ] If query response type newly marked `#[non_exhaustive]`: update ci.yml
> EXPECTED and`
> `  scripts/check-non-exhaustive.sh; coordinate with 001-A on final combined EXPECTED
> value`

**Replace with:**

> `- [ ] Confirm ci.yml EXPECTED remains 82 — no new `#[non_exhaustive]` pub types`
> `  are added by this story (no new typed response struct; `normalized_pql` is a`
> `  conditionally-inserted `Value` key, not a struct field). The non-exhaustive gate`
> `  is already wired at EXPECTED=82 by the merged 001-A. No coordination needed.`

### 2.3 File Structure row — server.rs entry

**Current text (File Structure Requirements table, server.rs row):**

> `| `crates/prism-mcp/src/server.rs` | Modify | Add `normalized_pql: Option<String>`
> to query response type; add `#[non_exhaustive]` to response type if not already
> present |`

**Replace with:**

> `| `crates/prism-mcp/src/server.rs` | Modify | In `PrismServer::query`, insert`
> `normalized_pql` conditionally into the existing inline `serde_json::json!`
> payload after rows serialization; conditional key insertion (not a struct field —`
> `no typed response struct exists; ADR-022 wiring-not-redesign) |`

### 2.4 File Structure row — ci.yml entry

**Current text (File Structure Requirements table, ci.yml row):**

> `| `ci.yml` | Modify | Increment `EXPECTED` if query response type is newly
> `#[non_exhaustive]` (coordinate with 001-A which already increments to 82 for +3
> types; this sub-story adds +0 or +1 depending on response type state) |`

**Replace with:**

> `| `ci.yml` | No change | EXPECTED remains 82 (set by merged 001-A). No new`
> `#[non_exhaustive]` types added by 001-B (no typed response struct). |`

### 2.5 risk_mitigations — skip_serializing_if bullet

**Current text (risk_mitigations, third bullet):**

> `"normalized_pql MUST be absent (not null, not present) on ALL error responses.
> Implement via`
> `#[serde(skip_serializing_if = 'Option::is_none')] — NOT Option::None serializing
> to null.`
> `AC-006 test uses serde_json::Value deserialization and checks
> value.get('normalized_pql').is_none()."`

**Replace with:**

> `"normalized_pql MUST be absent (not null, not present) on ALL error responses.`
> `Mechanism: conditional key insertion into the inline `serde_json::Value` payload`
> `in `PrismServer::query` — when `normalized_pql_str` is `None`, no key is inserted`
> `and the field is absent from the JSON output. `#[serde(skip_serializing_if)]` is`
> `a serde STRUCT attribute and does NOT apply to `serde_json::Value` construction.`
> `The absent-on-error guarantee is structurally enforced: the error path returns via`
> ``prism_error_to_structured_call_result` before the payload `Value` is built.`
> `AC-006 test: deserialize the response as `serde_json::Value` and assert`
> ``value.get(\"normalized_pql\").is_none()` — semantics and test wording UNCHANGED."`

### 2.6 Architecture Mapping row — normalized_pql response envelope field

**Current text (Architecture Mapping table, last row):**

> `| `normalized_pql` response envelope field | SS-10 | prism-mcp (`server.rs` query
> response type) | Effectful (MCP response construction) |`

**Replace with:**

> `| `normalized_pql` response envelope field | SS-10 | prism-mcp (`server.rs``
> ``PrismServer::query` inline payload — conditional `Value` key insertion before`
> ``SafetyEnvelopeBuilder::wrap`; no typed response struct) | Effectful (MCP response`
> `construction) |`

### 2.7 Changelog entry — add correction record

Append a new row to the Changelog table:

> `| 1.3 | NORMALIZED-PQL-ENVELOPE-CORRECTION-2026-06-21 | 2026-06-21 | architect |`
> `CORRECTION-1 adjudication (onboarding-001-B-normalized-pql-envelope-correction.md):`
> `Phase 5 task rewritten to conditional inline `Value` key insertion (not typed struct);`
> `File Structure ci.yml row changed to No-change; server.rs File Structure row clarified;`
> `risk_mitigations skip_serializing_if bullet corrected to conditional-insert pattern;`
> `Architecture Mapping row clarified; ci.yml coordination note removed (moot — 001-A`
> `merged, EXPECTED=82, +0 from 001-B). AC semantics and test wording unchanged. |`

---

## 3. What Does NOT Change

The following story elements are CORRECT as written and must NOT be touched:

- AC-005 and AC-006 behavioral semantics — `normalized_pql` present on success, absent
  on error, not null. The observable behavior is identical regardless of whether the
  field comes from a struct or a `Value` key.
- The test design: `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error`
  in crate `prism-mcp` using `serde_json::Value` deserialization and
  `value.get("normalized_pql").is_none()` — this works correctly against both the typed
  struct pattern and the inline Value pattern.
- Red Gate test table row 5 — test name and crate are correct.
- The token budget estimate row for `server.rs` query response type area (~1,000
  tokens) — the implementer still needs to read that area of server.rs.
- Library and Framework Requirements rows for `serde / serde_json` — the usage
  description "Option<String> with skip_serializing_if" should be updated to
  "conditional Value key insertion" but this is a minor clarification, not a
  correctness issue. Story-writer may update it for precision but it is not blocking.

---

## 4. Summary for Story-Writer

The defect is: the story instructs adding a typed query-response struct with
`normalized_pql: Option<String>` and `#[non_exhaustive]` to `server.rs`. No such
struct exists — the `query` handler constructs its response as an inline
`serde_json::json!({...})` Value and passes it to `SafetyEnvelopeBuilder::wrap`. This
is the BC-2.10.007 pattern shared by all tool handlers.

**Canonical mechanism:** conditional key insertion into the existing inline `json!`
payload. See §1.3 for the Rust pattern.

**Count impact:** +0 new `#[non_exhaustive]` types. `ci.yml EXPECTED` stays 82.

**`serde(skip_serializing_if)` N/A:** that directive is a serde struct attribute and
has no effect on `serde_json::Value` construction. Conditional key insertion is the
correct equivalent.

**BC-2.11.018 semantics preserved:** absent-on-error is structurally enforced by the
early-return error path; present-on-success is enforced by the conditional insert in
the success path. AC-005 and AC-006 remain unchanged.

**ADR-022 wiring-not-redesign:** adding a key to an inline `serde_json::Value` payload
is wiring. Introducing a new typed struct to replace the inline `json!` construction is
redesign. The wiring path is canonical.

**No BC amendment required. No scope change. This is a story-text correction only.**
