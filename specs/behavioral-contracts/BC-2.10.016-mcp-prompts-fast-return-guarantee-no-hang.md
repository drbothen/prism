---
document_type: behavioral-contract
level: L3
version: "1.2"
status: active
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: 2026-06-26
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-046-three-mode-correctness-filter-sql-pipe-mode-bridge-error-and-execution-validation.md"
input-hash: "TBD"
traces_to: ["CAP-034"]
extracted_from: null
---

# BC-2.10.016: MCP Prompts Fast-Return Guarantee — No Indefinite Hang

## Description

All registered MCP prompts (`triage_alerts`, `investigate_host`, `client_overview`, `cross_client_status`, `query_tutorial`) MUST return a prompt response within 5 seconds of the `prompts/get` request arriving at the MCP server. Prompt render functions (`render_query_tutorial`, `render_investigate_host`, etc.) are synchronous pure functions and return immediately; any hang observed is in the `PromptRouter` dispatch layer or `#[prompt_handler]` macro expansion. This contract defines the observable time-bounded postcondition that the implementer must verify after fixing BLOCKER-003.

## Preconditions

- A `prompts/get` request arrives with a valid prompt name (one of the registered prompts per BC-2.10.009)
- Prompt arguments are provided as required by the prompt's argument schema (e.g., `client_id` for `query_tutorial`, `client_id` + `hostname` for `investigate_host`)

## Postconditions

- The `prompts/get` response is returned within **5 seconds** of request receipt (wall clock), under normal operating conditions
- The response contains the prompt message array with the correct textual content per BC-2.10.009
- The MCP server is NOT blocked for other tool/resource requests while a prompt is being served (no lock held across prompt dispatch)
- Prompt response time is **independent of** any blocked audit channel, in-flight query execution, or tool dispatch lock
- **FROM-ready table names in prompt bodies (AUDIT-004):** All registered prompts that embed PrismQL queries within their message text (e.g., `query_tutorial`, `cross_client_status`) MUST use FROM-ready, sensor-prefixed table names in underscore-qualified form (e.g., `FROM crowdstrike_detections`, `FROM armis_devices`). Prompts MUST NOT use dot-notation table references (e.g., `FROM crowdstrike.detections`) in any embedded PrismQL query. The `render_*` functions that build prompt message content are the enforcement point: they must produce only underscore-qualified table name strings for any FROM clause in example or template queries. Violation: any analyst copying an embedded prompt example query and executing it via the `query` MCP tool MUST get a successful result (not E-QUERY-037 "table not found").

## Invariants

- Prompt render functions (`render_*` in `prompts.rs`) are and remain **synchronous pure functions** — they take parameter values and return `String` or `GetPromptResult`; they do NOT await, lock, or block
- The `PromptRouter` dispatch infrastructure MUST NOT hold any lock that is also held by the tool router or any other MCP server handler during prompt resolution
- **INV-PROMPT-NO-SHARED-LOCK:** There is no shared mutex between prompt dispatch and tool dispatch. The rmcp 1.7 `PromptRoute::new_dyn` closure MUST NOT capture any `Arc<Mutex<…>>` that is also held during tool execution.
- **INV-PROMPT-REQUIRED-ARGS:** For prompts with required arguments (`investigate_host.hostname`), the dispatch machinery MUST NOT hang while waiting for the argument to arrive. The shipped implementation satisfies this via option (a): substitute the literal string `(unknown)` for each missing required argument value and return the rendered prompt as Ok within 5 seconds. Option (b) (returning a structured MCP error) is a valid alternative permitted by this invariant but is NOT what is implemented.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Standard MCP error | Prompt name is unknown | Standard MCP error per BC-2.10.009 — returns within 5 seconds |
| Placeholder substitution (Ok) | Required argument missing from `prompts/get` request | Substitutes the literal string `(unknown)` for each missing required argument value, renders the prompt with the substituted value, and returns Ok within 5 seconds; does NOT hang. No structured MCP error is returned. |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-016-001 | `prompts/get query_tutorial` with `client_id` provided but `goal` absent (optional arg) | Returns prompt without goal context within 5 seconds; `goal` substituted with empty/default |
| EC-10-016-002 | `prompts/get investigate_host` with `client_id` and `hostname` provided | Returns prompt within 5 seconds |
| EC-10-016-003 | `prompts/get investigate_host` with `hostname` argument MISSING | Substitutes `(unknown)` for `hostname`, renders the prompt, and returns Ok within 5 seconds; MUST NOT hang. No structured MCP error is returned (option (a) of INV-PROMPT-REQUIRED-ARGS). |
| EC-10-016-004 | Prompt dispatch concurrent with a long-running `query` tool execution | Prompt returns within 5 seconds regardless of in-flight query state |
| EC-10-016-005 | `prompts/get query_tutorial` where embedded example query uses dot-notation table name (e.g., `FROM crowdstrike.detections`) | MUST NOT occur: the rendered prompt body must use `FROM crowdstrike_detections`. The `render_query_tutorial` function must only emit underscore-qualified FROM-ready table names. If any `render_*` function emits dot-notation in a FROM clause, executing the embedded query via the `query` MCP tool would return E-QUERY-037 ("table not found"), which breaks the tutorial's educational goal. This is the AUDIT-004 regression guard. |
| EC-10-016-006 | `prompts/get cross_client_status` where embedded multi-sensor PrismQL uses dot-notation (e.g., `FROM armis.devices`) | MUST NOT occur: rendered query must use `FROM armis_devices`. All `render_*` functions that include UNION, FROM, or JOIN clauses must exclusively use sensor-prefixed underscore-qualified table names. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `prompts/get query_tutorial` with `client_id: "org-c"` | Prompt message array within 5s; no hang | happy-path |
| `prompts/get investigate_host` with `client_id: "org-c"`, `hostname: "host-001"` | Prompt message array within 5s; no hang | happy-path |
| `prompts/get triage_alerts` with `client_id: "org-c"` | Prompt message array within 5s | happy-path |
| `prompts/get query_tutorial` with `client_id: "org-c"` — scan rendered message text for dot-notation FROM patterns (e.g., regex `FROM\s+\w+\.\w+`) | Zero matches: no embedded query in the rendered body uses dot-notation in a FROM clause. All FROM targets use underscore-qualified names like `crowdstrike_detections`, `armis_devices`. (AUDIT-004 / EC-10-016-005) | AUDIT-004 from-ready guard |
| `prompts/get cross_client_status` with `client_id: "org-c"` — scan rendered message text for dot-notation FROM patterns | Zero matches: all sensor references in the rendered body use underscore-qualified names. (AUDIT-004 / EC-10-016-006) | AUDIT-004 from-ready guard |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| (none allocated) | Prompt returns within 5s | integration test (timing assertion against MCP server) |

## Related BCs

- **BC-2.10.009** (extends — MCP Prompts for Common Workflows): this BC adds the fast-return time-bound guarantee and the no-hang invariant to the prompt registration contract

## Architecture Anchors

- `crates/prism-mcp/src/prompts.rs` — `render_query_tutorial`, `render_investigate_host`, etc. (confirmed synchronous pure functions)
- `crates/prism-mcp/src/prompts.rs` lines 247–300 — `PromptRoute::new_dyn` closures (candidate hang sites per ADR-046 D6)
- `crates/prism-mcp/src/server.rs` or equivalent — `#[prompt_handler]` macro expansion (root cause investigation target per ADR-046 D6)
- ADR-046 D6: BLOCKER-003 investigation protocol

## Story Anchor

TBD

## VP Anchors

(none allocated; timing test is integration test scope)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| Capability Anchor Justification | CAP-034 ("MCP Server & Transport") per capabilities.md §CAP-034 — this BC governs the prompt dispatch behavior of the `PrismServer`. CAP-034 explicitly describes "MCP prompts for common analyst workflows" registered in `prompts/list` and the server's response infrastructure. The fast-return guarantee is a property of the MCP server's prompt routing layer. |
| L2 Invariants | DI-004 (audit completeness does NOT apply to prompt dispatch — prompts are not tool invocations) |
| Priority | P0 |
| Closes findings | BLOCKER-003 (`query_tutorial` and `investigate_host` prompts hang indefinitely) |
| ADR traces | ADR-046 D6 (BLOCKER-003 investigation protocol) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | demo-fidelity-remediation-2026-06-26 | 2026-06-26 | product-owner | **AUDIT-004 contract fix (S-DEMO-FIDELITY-REMEDIATION-001):** Added postcondition requiring all registered prompts that embed PrismQL queries use FROM-ready sensor-prefixed underscore-qualified table names (e.g., `FROM crowdstrike_detections`), never dot-notation (e.g., `FROM crowdstrike.detections`). Added EC-10-016-005 (query_tutorial dot-notation guard) and EC-10-016-006 (cross_client_status dot-notation guard). Added two AUDIT-004 test vectors using regex scan over rendered message text. Enforcement point: `render_*` functions in `crates/prism-mcp/src/prompts.rs` must only emit underscore-qualified FROM-ready table names in embedded PrismQL examples. The implementer fix is to audit all `render_*` string literals for `FROM <sensor>.<table>` patterns and replace with `FROM <sensor>_<table>`. This is a content-correctness amendment; the fast-return (≤5s) guarantee from v1.0–v1.1 is unchanged. |
| 1.1 | PR-203-post-merge-POL-14 | 2026-06-26 | state-manager | **POL-14 BC auto-promotion: draft → active.** Anchor story S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 squash-merged via PR #203 to develop@7e60df03 (2026-06-26; CI 43/43 green; 9-round PR-LEVEL 3-CLEAN(strict) cascade on frozen HEAD 356e0573). `status: draft → active`. No behavioral change; frontmatter status field only. |
| 1.1 | S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 | 2026-06-25 | product-owner | Spec-internal-consistency reconciliation (OBS-2b from PR-LEVEL adversary). INV-PROMPT-REQUIRED-ARGS already sanctioned option (a) placeholder substitution; §Error Cases row and EC-10-016-003 incorrectly stated "Returns structured MCP error" for a missing required arg, contradicting the invariant. Updated §Error Cases, EC-10-016-003, and INV-PROMPT-REQUIRED-ARGS prose to reflect the shipped option-(a) behavior (`(unknown)` substitution, returns Ok within 5s). The no-hang / within-5s guarantee is unchanged. No code change. |
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md + ADR-046 D6. Closes BLOCKER-003. Implementer must investigate `#[prompt_handler]` macro expansion + `PromptRoute::new_dyn` closure before fixing. |
