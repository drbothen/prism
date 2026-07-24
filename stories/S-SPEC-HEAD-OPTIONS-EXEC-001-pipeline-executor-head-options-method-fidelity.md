---
document_type: story
story_id: S-SPEC-HEAD-OPTIONS-EXEC-001
title: "prism-spec-engine: PipelineExecutor HEAD + OPTIONS method fidelity — build_request arms + bodyless-response handling"
wave: wave-5-f
epic_id: E-SPEC-ENGINE
priority: P2
status: draft
version: "1.1"
updated: "2026-07-24"
level: "L3"
producer: story-writer
timestamp: "2026-06-04T00:00:00Z"
tdd_mode: strict
subsystems: [SS-16]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine) owns PipelineExecutor in `prism-spec-engine/src/pipeline.rs`.
#   `build_request` and response-parsing live in SS-16. This is an executor-fidelity
#   story — no spec parsing, no validation, no query engine involvement.
crates_touched: [prism-spec-engine]
target_module: prism-spec-engine
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# BC-2.16.009 v1.10 (active) defines the 7-method whitelist including HEAD and OPTIONS.
# BC-2.16.002 (active) covers PipelineExecutor method dispatch (postcondition: "method:
# GET or POST as declared" — this must be expanded to reflect all 7 whitelisted methods).
# PO must amend BC-2.16.002 to enumerate all 7 methods and add the bodyless-response
# invariant, then assign behavioral_contracts here before this story can reach `ready`.
# S-7.01 gate: behavioral_contracts MUST be non-empty before status: ready.
verification_properties: []
depends_on: [S-SPEC-HTTP-METHOD-VALIDATION-001]
# Dependency anchor justification:
#   S-SPEC-HTTP-METHOD-VALIDATION-001 establishes the 7-method whitelist in validation.rs
#   (BC-2.16.009 §VR7). This story implements execution fidelity for the two whitelisted
#   methods that currently fall through to the `_ => GET` fallback. Without the prior
#   story landing, operators have no structured error telling them HEAD/OPTIONS are
#   accepted — the execution gap is invisible. This story MUST follow S-SPEC-HTTP-METHOD-
#   VALIDATION-001 to maintain a coherent user experience (validate, then execute faithfully).
blocks: []
points: 5
# Points justification:
#   - Add HEAD + OPTIONS reqwest::Method arms to build_request (mechanical): 0.5 pts
#   - Design bodyless-response handling for HEAD (no body by HTTP spec): 1.0 pts
#   - Design bodyless-response handling for OPTIONS (no body guaranteed by HTTP spec): 0.5 pts
#   - Implement bodyless path in execute_step parse-response logic: 1.0 pts
#   - Unit tests covering HEAD fidelity, OPTIONS fidelity, regression for existing methods: 2.0 pts
#   Total: 5 points
estimated_days: 1.0
risk: MEDIUM
# Risk justification:
#   HEAD and OPTIONS response bodies must be handled carefully:
#   - HEAD responses have a valid Content-Length but NO body bytes (RFC 7231 §4.3.2).
#     Attempting to parse a JSON body from a HEAD response will either block or produce
#     an empty/error read. The executor must short-circuit body parsing for HEAD and
#     return 0 records (status-only check) or an error if 0 records is semantically
#     wrong for the spec's declared columns.
#   - OPTIONS responses may or may not have a body (RFC 7231 §4.3.7). The executor
#     must handle both cases gracefully.
#   - The existing `parse_response` path assumes all methods return JSON bodies; this
#     assumption must be made explicit and gated by method.
#   This is a behavioral change to the pipeline subsystem — regression risk is MEDIUM.
#   Mitigation: keep existing methods unchanged; add method-dispatch gate ONLY for HEAD/OPTIONS.
acceptance_criteria_count: 5
red_gate_tests: 5
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "HEAD response handling: HTTP spec (RFC 7231 §4.3.2) guarantees no body. The executor
    MUST NOT attempt to parse a body from a HEAD response. Design decision (pending PO/architect):
    either (a) return Ok with 0 records and PipelineResult.truncated=false for a successful
    HEAD (headers-only result), or (b) produce an error if the spec table declares columns
    (columns imply row data; HEAD cannot provide it). PO must adjudicate before implementation.
    This design question is the primary risk — the story must NOT proceed to ready without a
    BC that answers it."
  - "OPTIONS response handling: HTTP spec (RFC 7231 §4.3.7) does not guarantee a body.
    Safe default: treat like HEAD (0-record result). If the spec declares an Allow-header
    column, expose it via response header extraction (future enhancement, out of scope here).
    In scope: ensure OPTIONS does not cause a panic/unwrap on body parse."
  - "Regression gate: all 5 existing method arms (GET/POST/PUT/PATCH/DELETE) must be
    unchanged. A regression test must verify each existing method still maps to its
    correct reqwest::Method variant."
  - "reqwest::Method::HEAD exists; reqwest::Method::OPTIONS exists. No new reqwest
    dependency versions needed — reqwest already supports these methods."
  - "The `_ => reqwest::Method::GET` fallback in build_request MUST remain as belt-and-
    suspenders for any method that slips past Rule 7 validation in future code paths.
    The story adds two explicit arms; it does NOT remove the wildcard fallback."
inputs:
  - "crates/prism-spec-engine/src/pipeline.rs"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - ".factory/stories/S-SPEC-HTTP-METHOD-VALIDATION-001-http-method-whitelist-validation-in-sensor-spec.md"
input-hash: null
traces_to: [SEC-002]
# SEC-002 (CWE-440, expected-behavior violation): BC-2.16.009 §VR7 whitelist accepts HEAD
# and OPTIONS but `build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`) silently maps them to GET via `_ => GET`.
# This story closes the CWE-440 gap by implementing faithful execution of whitelisted methods.
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-SPEC-HEAD-OPTIONS-EXEC-001 v1.0 — PipelineExecutor HEAD + OPTIONS Method Fidelity

**Story ID:** S-SPEC-HEAD-OPTIONS-EXEC-001
**Status:** draft
**Version:** v1.0
**Wave:** wave-5-f
**Priority:** P2 (pipeline fidelity — CWE-440 expected-behavior gap; no auth bypass)
**Points:** 5

---

## Origin

PR #172 (S-SPEC-HTTP-METHOD-VALIDATION-001) adversarial + security cascade finding SEC-002
(CWE-440 — expected-behavior violation):

> BC-2.16.009 §VR7 (added in v1.8) now whitelists 7 HTTP methods: GET, POST, PUT, PATCH,
> DELETE, HEAD, OPTIONS. The whitelist is enforced at spec-load time by `validate_step_methods()`
> in `validation.rs`. However, `build_request` only has match arms for GET, POST, PUT, PATCH, DELETE and falls through
> `_ => reqwest::Method::GET` for any other value. A spec author writing `method = "HEAD"` passes
> Rule 7 validation but the HTTP request is silently issued as GET — not HEAD. Same for OPTIONS.
>
> This is a pre-existing gap (the `_ => GET` fallback predates BC-2.16.009 v1.8). It is a
> legitimate feature-ordering deferral per CLAUDE.md Rule 2: closing it requires designing
> bodyless-response handling for HEAD/OPTIONS responses, which is a behavioral change to the
> pipeline subsystem that warrants its own story and BC amendment.

**SEC-002 anchor:** This story closes SEC-002. The gap resolves when this story merges.

**Why a separate story (not an in-PR fix):**
- HEAD responses have no body (RFC 7231 §4.3.2). The existing `parse_response` path assumes
  a JSON body exists; skipping body parsing for HEAD requires a deliberate design decision
  (what does a HEAD step return? 0 records? headers-only? error if columns declared?).
- OPTIONS is similarly ambiguous at the response-parsing level.
- PO must amend BC-2.16.002 to add the bodyless-response invariant before implementation.
- The validation story (S-SPEC-HTTP-METHOD-VALIDATION-001) landed the whitelist. This story
  closes the execution gap. Correct feature ordering per CLAUDE.md Rule 2.

---

## Narrative

As a Prism platform operator, I want sensor spec steps declaring `method = "HEAD"` or
`method = "OPTIONS"` to execute as HEAD or OPTIONS HTTP requests (not silently as GET),
so that the pipeline faithfully executes whitelisted methods and spec authors can rely on
HTTP-method semantics for probe/preflight steps.

---

## Story-Level Goal

After this story merges:

1. `build_request` in `crates/prism-spec-engine/src/pipeline.rs` has explicit match arms
   for `"HEAD"` → `reqwest::Method::HEAD` and `"OPTIONS"` → `reqwest::Method::OPTIONS`,
   with the `_ => reqwest::Method::GET` wildcard fallback retained (belt-and-suspenders,
   not removed).
2. `execute_step` handles bodyless responses for HEAD and OPTIONS: it does NOT attempt to
   parse a JSON body from these responses. The handling design (0-record result vs. error
   for column-declaring steps) is defined in the PO-amended BC-2.16.002 before implementation.
3. Existing methods (GET, POST, PUT, PATCH, DELETE) are unchanged — zero regression.
4. BC-2.16.002 is amended by PO to enumerate all 7 methods in the postcondition that
   currently says "method: GET or POST as declared" and to add the bodyless-response invariant.
5. SEC-002 is formally closed.

---

## Behavioral Contracts

**PENDING PO AUTHORSHIP — S-7.01 gate applies.**

The implementing engineer must NOT start coding until the following BC amendments are complete:

| BC ID | Version | Required Amendment | Role in This Story |
|-------|---------|-------------------|--------------------|
| BC-2.16.002 | current | Expand "method: GET or POST as declared" postcondition to all 7 whitelisted methods; add bodyless-response invariant for HEAD + OPTIONS (design decision by PO: 0-record result vs. error) | build_request arms + execute_step bodyless handling |
| BC-2.16.009 | v1.10 | No amendment needed — §VR7 already whitelists HEAD + OPTIONS | Context for why these methods must be executable |

**Note:** `behavioral_contracts: []` in frontmatter until PO authors the BC-2.16.002
amendment and the BC IDs are propagated here. Status remains `draft` (S-7.01 gate).

---

## Open Design Question (PO must adjudicate before dispatch)

**OQ-001:** When a sensor spec step declares `method = "HEAD"` and the server responds
with a successful 2xx (no body per RFC 7231 §4.3.2), what does `execute_step` return?

Options:
- **Option A (0-record result):** Return `Ok(PipelineResult { records: RecordBatch::empty(...), truncated: false, request_count: 1 })`. The step succeeds with zero data rows. Subsequent steps can use `${this_step.field}` only if they reference non-body fields (e.g., response variables set to empty). Use case: HEAD as a liveness probe.
- **Option B (error if columns declared):** If the table declares one or more columns, return `Err(SpecEngineError::...)` because a HEAD step cannot produce row data to satisfy the schema. If the table declares 0 columns (unusual), return 0-record success. Use case: enforce that HEAD steps are only used for probe tables with no schema.
- **Option C (headers-only result):** Expose response headers as pseudo-columns. Out of scope for this story — requires schema changes. Defer to future story.

**PO must select Option A or Option B (or a hybrid) and amend BC-2.16.002 before dispatch.**

**OQ-002:** Same question for OPTIONS responses (RFC 7231 §4.3.7 — body is optional).

---

## Acceptance Criteria

**Note: BC traces are PLACEHOLDER pending PO authorship of BC-2.16.002 amendment.
All `(traces to BC-S.SS.NNN ...)` clauses below will be populated by PO at BC authorship time.**

### AC-001: HEAD method executes as HEAD (not GET)
Given: A sensor spec with `[[tables.fetch_steps]]` declaring `method = "HEAD"`, and a test
HTTP server that validates the request method.
When: `PipelineExecutor::execute_step` processes the step.
Then: The HTTP request is issued with method `HEAD` (reqwest::Method::HEAD), NOT GET.
The test server must confirm it received a HEAD request, not a GET request.
(traces to BC-2.16.002 — pending PO amendment, future clause)

Red Gate test: `test_BC_2_16_002_head_method_executes_as_head_not_get`

### AC-002: OPTIONS method executes as OPTIONS (not GET)
Given: A sensor spec with `[[tables.fetch_steps]]` declaring `method = "OPTIONS"`, and a
test HTTP server that validates the request method.
When: `PipelineExecutor::execute_step` processes the step.
Then: The HTTP request is issued with method `OPTIONS` (reqwest::Method::OPTIONS), NOT GET.
The test server must confirm it received an OPTIONS request.
(traces to BC-2.16.002 — pending PO amendment, future clause)

Red Gate test: `test_BC_2_16_002_options_method_executes_as_options_not_get`

### AC-003: HEAD bodyless response does not panic and follows BC-2.16.002 design decision
Given: A sensor spec step with `method = "HEAD"` and a server that responds 200 OK with no body.
When: `execute_step` receives the bodyless response.
Then: The step completes without panic or unwrap. The result follows the design selected by
PO in BC-2.16.002 amendment (Option A: 0-record success OR Option B: error if columns declared).
(traces to BC-2.16.002 — pending PO amendment, bodyless-response invariant)

Red Gate test: `test_BC_2_16_002_head_bodyless_response_handled_per_bc`

### AC-004: Existing methods (GET, POST, PUT, PATCH, DELETE) are unchanged — zero regression
Given: Sensor specs using any of the 5 existing methods.
When: `build_request` processes each method.
Then: GET maps to reqwest::Method::GET, POST to POST, PUT to PUT, PATCH to PATCH,
DELETE to DELETE — each identical to pre-story behavior. No test regressions.
(traces to BC-2.16.002 — existing postcondition, no change)

Red Gate test: `test_BC_2_16_002_existing_methods_unchanged_after_head_options_arms_added`

### AC-005: `_ => GET` wildcard fallback is retained
Given: Any future method value that passes validation but has no explicit arm in build_request
(theoretically impossible post-Rule-7, but retained as belt-and-suspenders).
When: `build_request` is called with an unrecognized method string.
Then: The wildcard arm maps to reqwest::Method::GET — no panic, no removal of the fallback.
(traces to BC-2.16.002 — belt-and-suspenders invariant per BC-2.16.009 v1.10 §VR7)

Red Gate test: `test_BC_2_16_002_wildcard_fallback_retained`

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `build_request` match arms must cover all 7 BC-2.16.009 §VR7 whitelist members | BC-2.16.009 v1.10 §VR7 + SEC-002 closure | After this story: GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS each have explicit arms |
| Bodyless-response handling must NOT use `unwrap()`/`expect()` on body parse for HEAD/OPTIONS | CLAUDE.md §Forbidden patterns — no unwrap in critical code paths | Use `Option<Bytes>` or conditional body parsing; gate on response method |
| `_ => reqwest::Method::GET` wildcard MUST remain | BC-2.16.009 v1.10 §VR7 "belt-and-suspenders" clause | Do NOT remove the wildcard — only ADD HEAD and OPTIONS arms before it |
| No `println!` in production code paths | CLAUDE.md §Conventions | Use `tracing::*!` for any diagnostic output |
| reqwest::Method::HEAD and reqwest::Method::OPTIONS MUST be used | Rust reqwest API | No string-based method construction; use the typed constants |
| BC-2.16.002 bodyless-response design decision MUST be resolved before coding | OQ-001/OQ-002 above | S-7.01 gate: story stays `draft` until BC-2.16.002 amended by PO |
| TD-VSDD-091: no volatile line-number citations | Project operational rule | Cite `build_request` and `execute_step` function names as behavioral anchors, not line numbers |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `reqwest` (workspace) | current workspace pin | `reqwest::Method::HEAD` + `reqwest::Method::OPTIONS` constants |
| `prism-spec-engine` (workspace) | current workspace path | `build_request` + `execute_step` in pipeline.rs |
| No new dependencies | — | HEAD/OPTIONS are built-in reqwest::Method constants; no new crate needed |

Version source: `Cargo.toml` workspace. No independent version pins required.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | Add HEAD + OPTIONS arms to `build_request` match; add bodyless-response gate in `execute_step` |
| `crates/prism-spec-engine/tests/bc_2_16_002_pipeline_method_fidelity.rs` | CREATE | 5 Red Gate tests + bodyless-response coverage; wiremock-based |
| `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md` | READ ONLY (PO amends pre-dispatch) | PO adds bodyless-response invariant + expands 7-method postcondition before implementer writes code |
| `.factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md` | READ ONLY | §VR7 whitelist context — already at v1.10; no amendment needed |

---

## Tasks

1. **BLOCK — PO gate:** Read BC-2.16.002 current postcondition "method: GET or POST as declared".
   Confirm OQ-001 and OQ-002 have been answered by PO in a BC-2.16.002 amendment.
   Do NOT proceed past this task until `behavioral_contracts:` frontmatter is populated.

2. **Read** `crates/prism-spec-engine/src/pipeline.rs` — locate `build_request` function.
   Understand the existing match arms (GET/POST/PUT/PATCH/DELETE) and the `_ => GET` fallback.
   Locate `execute_step` and the body-parse path (where JSON response body is read).

3. **Read** BC-2.16.002 (PO-amended version) — understand the bodyless-response invariant
   and the design decision for HEAD/OPTIONS (Option A or B from OQ-001/OQ-002).

4. **Write stubs:** In `pipeline.rs`, add commented `todo!()` stubs for:
   - `"HEAD" => reqwest::Method::HEAD` arm in `build_request`
   - `"OPTIONS" => reqwest::Method::OPTIONS` arm in `build_request`
   - Bodyless-response gate in `execute_step` (method-dispatch before body read)

5. **Write Red Gate tests** (5 tests listed in ACs above) — all must fail (RED) before
   implementation. Use `wiremock` for HTTP server assertions (same pattern as existing
   `execute_step_tests` in `pipeline.rs`). Tests must assert request METHOD received by server,
   not just client-side configuration.

6. **Implement** HEAD arm: `"HEAD" => reqwest::Method::HEAD`
   Run: `test_BC_2_16_002_head_method_executes_as_head_not_get` — must go GREEN.

7. **Implement** OPTIONS arm: `"OPTIONS" => reqwest::Method::OPTIONS`
   Run: `test_BC_2_16_002_options_method_executes_as_options_not_get` — must go GREEN.

8. **Implement** bodyless-response gate in `execute_step`: before reading/parsing the HTTP
   response body, check if `response.method() == HEAD || response.method() == OPTIONS`
   (or equivalent `reqwest::RequestBuilder`-captured method). Apply the PO-decided design
   (Option A: return 0-record Ok; Option B: return error if columns declared).
   Run: `test_BC_2_16_002_head_bodyless_response_handled_per_bc` — must go GREEN.

9. **Verify regression:** Run `test_BC_2_16_002_existing_methods_unchanged_after_head_options_arms_added`
   and `test_BC_2_16_002_wildcard_fallback_retained` — must go GREEN.

10. **TD-VSDD-060 sibling sweep:** grep `build_request` and `execute_step` across
    `crates/prism-spec-engine/src/` for any callsites that cache or branch on the method
    string independently of `build_request`. Update them if any assume the old 5-arm set.

11. **Run** `just iter prism-spec-engine` — all tests GREEN.

12. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-SPEC-HTTP-METHOD-VALIDATION-001** (wave-5-e-demo-fidelity; PR #172): This story is
  the direct predecessor. It added `validate_step_methods()` to `validation.rs` implementing
  BC-2.16.009 §VR7 whitelist. The implementer wired Rule 7 into both load paths (`parse_and_
  validate_spec_toml` and `SpecLoader::load_all`) post env-resolver (Rule 6 → Rule 7 ordering).
  Key lesson: both load paths must be covered — do not assume one path subsumes the other.
  The `validate_step_methods` function returns `Vec<(usize, usize, SpecEngineError)>` (table
  index + step index + error), NOT `Vec<SpecEngineError>`.
- **S-PLUGIN-PREREQ-B** (merged): Added `PipelineExecutor::execute` and `execute_step`
  — these are the behavioral ancestors of the methods this story must modify.
- **Pattern for wiremock-based method assertion:** See `execute_step_tests` module in
  `pipeline.rs` (around the `RED GATE verified` comment). Use `wiremock::matchers::method`
  to assert the HTTP method the server receives.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `method = "HEAD"` with a server that returns 200 OK and no body | No panic; bodyless path per OQ-001 PO decision (0-record Ok or column-error) |
| EC-002 | `method = "OPTIONS"` with a server that returns 200 OK and a JSON body | Body is present; treat like a normal response? PO must decide in BC-2.16.002 amendment |
| EC-003 | `method = "HEAD"` step followed by a downstream step using `${head_step.field}` | If head step returns 0 records, downstream variable reference produces empty; spec author responsibility to not use HEAD step variables |
| EC-004 | `method = "OPTIONS"` with a server returning 405 Method Not Allowed | Standard error handling per `execute_step` error path; no special treatment needed |
| EC-005 | `method = "GET"` (existing) after adding HEAD/OPTIONS arms — zero regression | GET arm unchanged; existing tests must still pass |
| EC-006 | `method = "POST"` with body_template after adding HEAD/OPTIONS arms | POST arm unchanged; body_template still sent; existing tests must still pass |
| EC-007 | Unrecognized method `"PROPFIND"` (hypothetical, blocked by Rule 7 in practice) | `_ => GET` wildcard fallback; no panic; belt-and-suspenders invariant |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~5,000 |
| BC-2.16.002 (relevant sections — postconditions + bodyless amendment) | ~5,000 |
| BC-2.16.009 (§VR7 only) | ~2,000 |
| crates/prism-spec-engine/src/pipeline.rs (build_request + execute_step) | ~8,000 |
| S-SPEC-HTTP-METHOD-VALIDATION-001 (predecessor intelligence) | ~2,000 |
| wiremock test patterns (from existing execute_step_tests) | ~3,000 |
| Test outputs (cargo nextest) | ~1,000 |
| **Total estimate** | **~26,000 tokens (~10% of 256K context)** |

Well within budget. Single-story delivery.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-07-24 | story-writer | F-WASE-P52-LOW-001 POL-29 class sweep, burst wave-a-spec-evolution-fix-burst-41: 2 occurrences of stale `PipelineExecutor::build_request` qualifier corrected to free-function citation — first mention (frontmatter comment) uses `build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`), second mention (body blockquote) plain `build_request`. No ACs, BCs, or behavioral semantics changed. |
| 1.0 | 2026-06-04 | story-writer | Initial draft — anchors SEC-002 (CWE-440) from PR #172 adversarial cascade. PO gate required: BC-2.16.002 amendment (OQ-001/OQ-002 bodyless-response design) before dispatch. S-7.01 pending. |
