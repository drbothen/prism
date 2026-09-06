---
document_type: holdout-scenario
level: L3
id: "HS-032-003"
title: "S-REL-AGENT-VERSION-001: product-version coherence — MCP serverInfo.version and outbound User-Agent both report the same PRISM_VERSION"
category: "behavioral-correctness"
must_pass: true
priority: P1
epic_id: "E-REL-IDENTITY"
story_source: "S-REL-AGENT-VERSION-001"
version: "1.0"
status: active
used: false
last_evaluated: null
last_eval_satisfaction: null
single_use: true
producer: product-owner
timestamp: "2026-09-05T00:00:00Z"
modified: "2026-09-05"
phase: 3
inputs:
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
  - ".factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md"
  - ".factory/stories/S-REL-AGENT-VERSION-001-agent-version-surfaces.md"
input-hash: "[pending-recompute]"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-REL-AGENT-VERSION-001 (HS-032 group). Tests version COHERENCE across both agent-facing surfaces in a SINGLE run: the MCP serverInfo.version (Surface A) and the spec-engine outbound HTTP User-Agent (Surface B) must report IDENTICAL version strings. This catches the split-wiring defect: Surface A wired but Surface B still uses library version, or vice versa. Both surfaces must resolve from the same PRISM_VERSION source. Requires combined MCP + DTU session. Test-writer and implementer must NOT read this file."
---

# HS-032-003: product-version coherence across both agent-facing surfaces

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-REL-AGENT-VERSION-001 (HS-032 group)
**Must Pass:** YES (P1 — blocks story merge)
**Authority:** ADR-064 §D4 §Status: "All four agent/tenant-visible
surfaces MUST emit the same PRISM_VERSION after D4 ships." AND ADR-064 §D4 §Purpose:
"v1.0.0-beta.1 MUST self-identify coherently to LLM agents (serverInfo.version) and
to sensor tenants (HTTP user-agent)."
**Gate:** Story-level holdout gate (HS-032) — runs after LOCAL 3-CLEAN convergence,
before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates **coherent product-version identity** across both agent-facing
surfaces in a single prism run. It is a composition test: it only passes if BOTH
HS-032-001 (serverInfo.version) AND the User-Agent surface are correct AND they agree.

The core invariant being tested: both surfaces must resolve from the same `PRISM_VERSION`
source, so any build of prism presents a single, consistent version identity to all
observers (LLM agents via MCP, sensor tenants via HTTP headers).

**The split-wiring defect this scenario catches:**
A partial implementation where one surface was migrated but the other was not:
- Surface A correct + Surface B stale → `serverInfo.version = "1.0.0-dev"`,
  `User-Agent = "prism/0.9.0"` — incoherent; fails this scenario.
- Surface A stale + Surface B correct → `serverInfo.version = "0.1.0"`,
  `User-Agent = "prism/1.0.0-dev"` — incoherent; fails this scenario.
- Both surfaces stale → fails both HS-032-001 and HS-032-002 as well.
- Both surfaces correct and coherent → PASS.

**BDD supplement:**

**Given** prism is built from the S-REL-AGENT-VERSION-001 story branch
**And** prism MCP stdio is started with a Claroty DTU configured
**When** an MCP `initialize` handshake is performed AND THEN a PrismQL query is
executed that triggers an outbound HTTP call to the DTU
**Then** `serverInfo.version` from the initialize response equals `V` (some version string)
**And** the `User-Agent` header on the DTU-bound HTTP request also equals `prism/V`
(same `V`)
**And** `V` equals `"1.0.0-dev"` (the expected PRISM_VERSION on a local develop build)

---

## Setup Instructions

This scenario runs in the same prism session as HS-032-002. Both the MCP handshake
and the outbound HTTP request must come from the SAME prism binary instance so the
version surfaces can be compared.

1. Confirm prism binary is built from the S-REL-AGENT-VERSION-001 story branch HEAD commit.

2. Start the Claroty DTU locally (for request-header observation).

3. Start prism in MCP stdio mode with the Claroty sensor configured.

4. **Step A:** Issue the MCP `initialize` request. Capture `result.serverInfo.version`
   from the wire-level JSON response. Call this value `V_MCP`.

5. **Step B:** Issue a PrismQL SELECT query via the MCP `query` tool (same session, same
   prism instance) that triggers an outbound HTTP call to the Claroty DTU. Observe the
   `User-Agent` header on the inbound request at the DTU. Strip the `"prism/"` prefix
   from the User-Agent to get `V_UA`.

6. **Coherence assertion:** Assert `V_MCP == V_UA`.

7. **Value assertion:** Assert both `V_MCP` and `V_UA` equal `"1.0.0-dev"`.

Note: Steps A and B may be combined with HS-032-001 and HS-032-002 if those scenarios
are run in the same session — their observations suffice for this coherence check.

---

## Behavioral Contract Linkage

| Source | Clause | Scenario Aspect |
|--------|--------|-----------------|
| ADR-064 §D4 §Purpose | "All four agent/tenant-visible surfaces MUST emit the same PRISM_VERSION after D4 ships" | Core coherence invariant |
| ADR-064 §D4 §Surface A | serverInfo.version wiring via self.product_version | Surface A version source |
| ADR-064 §D4 §Surface B + ADR-050 §D6 | User-Agent via env!("PRISM_VERSION") in pipeline.rs | Surface B version source |
| ADR-064 D2 (via D4) | Both surfaces resolve from the same PRISM_VERSION source (build.rs injection) | Single source of truth |

---

## Verification Approach

1. Obtain `V_MCP` from the `initialize` response `result.serverInfo.version`.

2. Obtain `V_UA` by stripping `"prism/"` prefix from the observed User-Agent header on
   the outbound DTU request.

3. If either observation is a SETUP-FAILURE (prism did not start, DTU not reachable,
   headers not observable): record SETUP-FAILURE for this scenario.

4. **Coherence assertion:** Compare `V_MCP` and `V_UA`.
   - If `V_MCP == V_UA`: record PASS on "coherence" dimension.
   - If `V_MCP != V_UA`: record FAIL on "coherence" dimension. This is the split-wiring
     defect — one surface was migrated, the other was not.

5. **Value correctness:** Assert both equal `"1.0.0-dev"`.
   - If both are `"1.0.0-dev"`: record PASS on "correct-value" dimension.
   - If both are the same non-"1.0.0-dev" value: record PARTIAL on "correct-value"
     (coherent but unexpected version; investigate fallback chain).
   - If `V_MCP == "0.1.0"` AND `V_UA` is library version: record FAIL on "correct-value"
     (both surfaces stale; coherent but wrong; HS-032-001 + HS-032-002 already caught this).

6. Quote both observed values in the evaluation report.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Both surfaces observable (no setup failure)** (weight: 0.10): Were both `V_MCP`
  and `V_UA` successfully captured?
  Full credit (1.0): both captured. Zero credit (0.0): one or both missing — SETUP-FAILURE.

- **Coherence: V_MCP == V_UA** (weight: 0.55): Do both surfaces report the same version?
  Full credit (1.0): `V_MCP == V_UA` (the core coherence invariant; this is the new
  value this scenario adds over HS-032-001 + HS-032-002 separately).
  Zero credit (0.0): `V_MCP != V_UA` — split-wiring defect; one surface stale.

- **Correct value: both equal "1.0.0-dev"** (weight: 0.35): Are both `"1.0.0-dev"`?
  Full credit (1.0): `V_MCP = V_UA = "1.0.0-dev"`.
  Partial credit (0.5): coherent but unexpected value (not "1.0.0-dev"; investigate build).
  Zero credit (0.0): either stale hardcoded value or empty.

---

## Edge Conditions

- **V_MCP = "0.0.0-test", V_UA = "1.0.0-dev":** Record FAIL on coherence. The "0.0.0-test"
  value indicates `PrismServer::new` (test constructor) was used at boot instead of
  `with_deps`. The with_deps wiring at boot step 9 was not applied.

- **V_MCP = "1.0.0-dev", V_UA = library-version:** Record FAIL on coherence. Surface A
  was correctly wired but Surface B migration was not applied (or build.rs not created).

- **V_MCP = "0.1.0", V_UA = "1.0.0-dev":** Record FAIL on coherence. Surface B was
  correctly migrated but Surface A still hardcodes "0.1.0" in get_info.

- **Both V_MCP and V_UA = "1.0.0-dev" but from different prism instances:**
  Not applicable — both observations must come from the SAME prism instance per setup.
  If HS-032-002 was run in a separate session, re-run this coherence check in a fresh
  combined session.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-032-003 (satisfaction: X.XX) — version identity is incoherent across agent-facing surfaces; check that both Surface A (prism-mcp serverInfo.version via PrismServer::with_deps + boot step 9 env!(\"PRISM_VERSION\")) and Surface B (prism-spec-engine pipeline.rs user-agent via env!(\"PRISM_VERSION\") + crates/prism-spec-engine/build.rs) are fully applied — the surfaces must report the same PRISM_VERSION (ADR-064 D4 §Purpose: all four surfaces must emit the same PRISM_VERSION)"`

Do NOT disclose: the specific version strings observed, which surface was stale, or the
exact coherence mismatch values.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-REL-AGENT-VERSION-001 branch; combined MCP initialize + Claroty DTU session |
| corpus_size | Two version observations from the same binary instance; one coherence assertion |
| known_edge_cases | V_MCP != V_UA (split-wiring; one surface stale); V_MCP = "0.0.0-test" (test constructor at boot) |
| false_positive_threshold | Zero: V_MCP == V_UA == "1.0.0-dev" is an unambiguous pass on a local develop build |
| false_negative_threshold | Zero: V_MCP != V_UA is an unambiguous split-wiring defect |

**Known-good corpus:** prism binary with both Surface A and Surface B correctly wired —
expected: `V_MCP = "1.0.0-dev"`, `V_UA = "1.0.0-dev"`, coherence passes.

**Known-problematic corpus:** prism binary with partial migration (one surface only) —
expected: `V_MCP != V_UA`, coherence fails; the holdout evaluator identifies which
surface carries the stale value.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | s-rel-agent-version-001-holdout-authoring | 2026-09-05 | product-owner | Initial authoring. HS-032 group for S-REL-AGENT-VERSION-001. Coherence test: V_MCP (serverInfo.version) must equal V_UA (User-Agent version suffix) and both must equal "1.0.0-dev" on local dev build. Catches split-wiring defect where one surface is migrated but the other is not. ADR-064 D4 §Purpose ("all surfaces emit same PRISM_VERSION") authority. SINGLE-USE. |
