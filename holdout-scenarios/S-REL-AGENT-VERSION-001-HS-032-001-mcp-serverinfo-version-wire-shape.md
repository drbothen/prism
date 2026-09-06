---
document_type: holdout-scenario
level: L3
id: "HS-032-001"
title: "S-REL-AGENT-VERSION-001: MCP initialize handshake serverInfo.version wire shape — must equal PRISM_VERSION, not stale 0.1.0"
category: "behavioral-correctness"
must_pass: true
priority: P1
epic_id: "E-REL-IDENTITY"
story_source: "S-REL-AGENT-VERSION-001"
version: "1.1"
status: active
used: false
last_evaluated: null
last_eval_satisfaction: null
single_use: true
producer: product-owner
timestamp: "2026-09-05T00:00:00Z"
modified: "2026-09-06"
phase: 3
inputs:
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
  - ".factory/stories/S-REL-AGENT-VERSION-001-agent-version-surfaces.md"
input-hash: "4070bbb"
traces_to: "ADR-064-D4"
behavioral_contracts: []
verification_properties: []
lifecycle_status: active
introduced: "S-REL-AGENT-VERSION-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-REL-AGENT-VERSION-001 (HS-032 group). Tests Surface A wire-level correctness: the MCP initialize handshake serverInfo.version field must contain the product version (1.0.0-beta.1 on an override build using PRISM_BUILD_VERSION=1.0.0-beta.1), NOT the stale hardcoded '0.1.0'. Uses override build so the gate is discriminating: a non-migrated get_info still returns '0.1.0' regardless of the override. This is the most agent-visible version surface — an LLM agent reads serverInfo.version in the handshake to identify the prism server it is connected to. The defect this catches: PrismServer::get_info still returning Implementation::new('prism', '0.1.0') if the with_deps wiring was not applied correctly at boot step 9. Test-writer and implementer must NOT read this file."
---

# HS-032-001: MCP initialize handshake serverInfo.version wire shape

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-REL-AGENT-VERSION-001 (HS-032 group)
**Must Pass:** YES (P1 — blocks story merge)
**Authority:** ADR-064 §D4 §Surface A — `PrismServer::get_info` must return
`Implementation::new("prism", self.product_version)` where `product_version` is the
`env!("PRISM_VERSION")` value threaded from boot step 9.
**Gate:** Story-level holdout gate (HS-032) — runs after LOCAL 3-CLEAN convergence,
before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the **MCP initialize handshake** emits the correct
`serverInfo.version` at the wire level. The prism MCP server's `initialize` response
is the first message an LLM agent receives; its `serverInfo.version` field identifies
which product version the agent is connected to.

**Before this story ships:** `PrismServer::get_info` hardcodes
`Implementation::new("prism", "0.1.0")`, so every initialize response reports
`serverInfo.version = "0.1.0"` regardless of the actual binary version.

**After this story ships:** `get_info` uses `self.product_version`, which is set to
`env!("PRISM_VERSION")` via `PrismServer::with_deps` at boot step 9. On a local/develop
build, `PRISM_VERSION` resolves to `CARGO_PKG_VERSION = "1.0.0-dev"`. On a CI beta.1
build, it resolves to `"1.0.0-beta.1"`.

**The defect this scenario catches:** If the `with_deps` wiring was not applied correctly
(e.g., boot step 9 was not updated to pass `env!("PRISM_VERSION")`), or if `get_info`
still hardcodes the version, the wire output will contain `"0.1.0"` instead of the
product version. This is the exact regression this scenario discriminates.

**BDD supplement:**

**Given** prism is built from the S-REL-AGENT-VERSION-001 story branch
**And** prism MCP stdio is started with a valid prism.toml configuration
**When** an MCP client sends an `initialize` request
**Then** the wire-level JSON response contains `"serverInfo"` with `"version"` equal
to `"1.0.0-beta.1"` (the injected PRISM_BUILD_VERSION override used in this scenario)
**And** the wire-level JSON response does NOT contain `"0.1.0"` in the `serverInfo` block

---

## Setup Instructions

1. Build the prism binary from the S-REL-AGENT-VERSION-001 story branch HEAD commit
   (not from develop or any prior branch) using a DISTINGUISHING version override so
   that both prism-bin's build.rs resolves to the same injected value, making the
   gate discriminating:
   ```
   PRISM_BUILD_VERSION=1.0.0-beta.1 cargo build -p prism-bin
   ```
   This simulates a release build: prism-bin's build.rs D2-conformant step 1 resolves
   `PRISM_BUILD_VERSION=1.0.0-beta.1` and emits it as `PRISM_VERSION`. That value is
   then threaded through `PrismServer::with_deps` at boot step 9 to Surface A. A
   non-migrated `get_info` that still hardcodes `"0.1.0"` will NOT benefit from the
   override — it remains `"0.1.0"`, making the gate discriminating.

2. Start prism in MCP stdio mode using the binary built in step 1:
   ```
   PRISM_ORG_ID=<org-uuid> prism start --mcp-stdio
   ```
   Use a minimal valid `prism.toml` (no sensor configuration required; this scenario
   only tests the MCP handshake, not sensor query execution).

3. Capture full MCP stdio output from process start through the initialize response.
   The initialize exchange happens immediately on connection — no tool calls needed.

4. Issue an MCP `initialize` request on stdin. Standard JSON-RPC form:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "initialize",
     "params": {
       "protocolVersion": "2024-11-05",
       "capabilities": {},
       "clientInfo": { "name": "holdout-evaluator", "version": "1.0.0" }
     }
   }
   ```

5. Capture the full raw JSON response from stdout.

---

## Behavioral Contract Linkage

| Source | Clause | Scenario Aspect |
|--------|--------|-----------------|
| ADR-064 §D4 §Surface A | `get_info` row: `Implementation::new("prism", self.product_version)` | Primary assertion: serverInfo.version is not "0.1.0" |
| ADR-064 §D4 §Surface A | boot.rs row: `env!("PRISM_VERSION")` passed as final arg to `with_deps` | Wiring chain: product_version reaches get_info via boot step 9 |
| ADR-064 §D4 §Purpose | "An LLM agent reads serverInfo.version in the handshake to identify which prism server it is talking to" | Agent-facing correctness: the version a connected agent sees |

---

## Verification Approach

1. Parse the raw MCP `initialize` response JSON.

2. If the response is a JSON-RPC error object: record SETUP-FAILURE — prism did not
   start or handshake correctly. Do NOT record as behavioral FAIL.

3. Navigate to `result.serverInfo.version` in the response JSON.

4. **Primary assertion:** Assert `serverInfo.version` is NOT `"0.1.0"`.
   - If `serverInfo.version == "0.1.0"`: record FAIL on "stale-version-absent" dimension.
     This is the defect this scenario targets — get_info still returning hardcoded "0.1.0".

5. **Secondary assertion:** Assert `serverInfo.version` equals `"1.0.0-beta.1"`.
   - With `PRISM_BUILD_VERSION=1.0.0-beta.1` set during the cargo build (setup step 1),
     prism-bin's build.rs D2-conformant chain resolves to `"1.0.0-beta.1"` (step 1:
     PRISM_BUILD_VERSION override takes precedence). That value is threaded via
     `PrismServer::with_deps` at boot step 9 to Surface A.
   - If `serverInfo.version == "1.0.0-beta.1"`: record PASS on "correct-product-version" dimension.
   - If `serverInfo.version == "1.0.0-dev"`: record PARTIAL (version wiring is active but
     the PRISM_BUILD_VERSION=1.0.0-beta.1 override was not picked up by prism-bin's build;
     verify that cargo was invoked with the env var set as specified in setup step 1).
   - If `serverInfo.version` is some other non-empty non-"0.1.0" string: record PARTIAL
     (version wiring is active but unexpected value; investigate build env).
   - If `serverInfo.version` is empty or absent: record FAIL.

6. **Wire-level completeness:** Confirm the raw JSON string contains the version value
   as a proper JSON string field (not truncated, not null). Quote the relevant JSON
   fragment in the evaluation report for evidence.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Handshake succeeds (no error)** (weight: 0.15): Does prism respond to initialize
  without a JSON-RPC error?
  Full credit (1.0): valid initialize result received.
  Zero credit (0.0): JSON-RPC error or process crash — record SETUP-FAILURE instead.

- **Stale "0.1.0" absent from serverInfo.version** (weight: 0.50): Is
  `serverInfo.version` anything other than `"0.1.0"`?
  Full credit (1.0): version is not "0.1.0" — wiring is active.
  Zero credit (0.0): version is "0.1.0" — hardcoded value was not replaced.

- **Correct product version present** (weight: 0.35): Is `serverInfo.version` equal
  to `"1.0.0-beta.1"` (the injected PRISM_BUILD_VERSION override)?
  Full credit (1.0): `serverInfo.version == "1.0.0-beta.1"`.
  Partial credit (0.5): non-empty, non-"0.1.0" value present but not "1.0.0-beta.1"
  (wiring active but override not picked up; verify build invocation in setup step 1).
  Zero credit (0.0): empty string, null, or absent field.

---

## Edge Conditions

- **prism fails to start (missing config):** Record SETUP-FAILURE, not behavioral FAIL.
  Try a minimal prism.toml with org_id only — sensor config is not required for handshake.

- **serverInfo is absent from response:** Record FAIL on "correct-product-version"
  dimension — the handshake structure is malformed.

- **serverInfo.version = "0.0.0-test":** Record PARTIAL — this indicates `PrismServer::new`
  (test constructor) was called instead of `PrismServer::with_deps` at boot. The with_deps
  wiring at boot step 9 was not applied.

- **serverInfo.version = "0.9.0":** Record PARTIAL — this indicates PRISM_VERSION resolved
  to the library crate version (from a different crate's CARGO_PKG_VERSION). Unlikely in
  prism-mcp context but would indicate fallback chain confusion.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-032-001 (satisfaction: X.XX) — MCP serverInfo.version contains an unexpected value; check PrismServer::get_info uses self.product_version (not a hardcoded string), PrismServer::with_deps is called at boot step 9 with env!(\"PRISM_VERSION\"), and PrismServer struct has the product_version field (ADR-064 D4 §Surface A injection sites)"`

Do NOT disclose: the specific version string expected, that the defect is "0.1.0", or the
specific boot step number.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary built from S-REL-AGENT-VERSION-001 branch with PRISM_BUILD_VERSION=1.0.0-beta.1 override (see setup step 1) |
| corpus_size | Single MCP initialize handshake; single JSON field assertion |
| known_edge_cases | serverInfo.version = "0.1.0" (stale hardcoded; the defect being fixed); "0.0.0-test" (test constructor path; incorrect boot wiring) |
| false_positive_threshold | Zero: "1.0.0-beta.1" is an unambiguous correct result for the override build (PRISM_BUILD_VERSION=1.0.0-beta.1) |
| false_negative_threshold | Zero: "0.1.0" present in wire output is an unambiguous wiring defect |

**Known-good corpus:** prism binary from this story branch built with `PRISM_BUILD_VERSION=1.0.0-beta.1`, with correct wiring — expected:
`serverInfo.version = "1.0.0-beta.1"` (override build; prism-bin build.rs step 1 resolves PRISM_BUILD_VERSION).

**Known-problematic corpus:** prism binary where `get_info` still returns
`Implementation::new("prism", "0.1.0")` (pre-migration state) — expected:
`serverInfo.version = "0.1.0"` which is the defect signal. The PRISM_BUILD_VERSION
override does NOT fix this defect because the hardcoded literal is never replaced
by the override — making this the discriminating test.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | hs-032-override-build-model-fix | 2026-09-06 | product-owner | HIGH defect fix (F-SRAV-HIGH-001 root cause). Rewrote to override-build model: evaluator builds with PRISM_BUILD_VERSION=1.0.0-beta.1. Expected serverInfo.version changed from "1.0.0-dev" to "1.0.0-beta.1". Gate is now discriminating: non-migrated get_info still returns "0.1.0" ≠ "1.0.0-beta.1"; plain local-dev assertion was non-discriminating (un-migrated Surface B also emits "prism/0.9.0" on local dev — HS-032-002/003 were the failing scenarios; 001 was incidentally also wrong about expected value). Updated §Setup, §Verification, §Rubric, §real-world-corpus, notes. |
| 1.0 | s-rel-agent-version-001-holdout-authoring | 2026-09-05 | product-owner | Initial authoring. HS-032 group for S-REL-AGENT-VERSION-001. MCP initialize handshake wire-shape test: serverInfo.version must not be stale "0.1.0" and must be "1.0.0-dev" on local dev build. Catches get_info hardcoded value + boot step 9 with_deps wiring gap. ADR-064 D4 §Surface A authority. SINGLE-USE. |
