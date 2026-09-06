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
  - ".factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md"
  - ".factory/stories/S-REL-AGENT-VERSION-001-agent-version-surfaces.md"
input-hash: "dad7dce"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-REL-AGENT-VERSION-001 (HS-032 group). Tests version COHERENCE across both agent-facing surfaces in a SINGLE run using override build (PRISM_BUILD_VERSION=1.0.0-beta.1): the MCP serverInfo.version (Surface A) and the spec-engine outbound HTTP User-Agent (Surface B) must both report '1.0.0-beta.1'. On plain local dev the surfaces legitimately diverge (Surface A=1.0.0-dev, Surface B=0.9.0) — the override build makes the coherence assertion discriminating. Catches the split-wiring defect: Surface A wired but Surface B still uses library version (V_UA=prism/0.9.0 ≠ V_MCP=1.0.0-beta.1), or vice versa. Requires combined MCP + DTU session. Test-writer and implementer must NOT read this file."
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

**The split-wiring defect this scenario catches (with PRISM_BUILD_VERSION=1.0.0-beta.1 override):**
A partial implementation where one surface was migrated but the other was not:
- Surface A correct + Surface B stale → `serverInfo.version = "1.0.0-beta.1"`,
  `User-Agent = "prism/0.9.0"` — incoherent; fails this scenario. This is the PRIMARY
  discriminating FAIL case: Surface A picks up the override (via prism-bin build.rs),
  Surface B ignores it (pipeline.rs still uses CARGO_PKG_VERSION = "0.9.0").
- Surface A stale + Surface B correct → `serverInfo.version = "0.1.0"`,
  `User-Agent = "prism/1.0.0-beta.1"` — incoherent; fails this scenario.
- Both surfaces stale → fails both HS-032-001 and HS-032-002 as well.
- Both surfaces correct and coherent → `serverInfo.version = "1.0.0-beta.1"`,
  `User-Agent = "prism/1.0.0-beta.1"` → PASS.

**BDD supplement:**

**Given** prism is built from the S-REL-AGENT-VERSION-001 story branch with
`PRISM_BUILD_VERSION=1.0.0-beta.1` (override build — see setup step 1)
**And** prism MCP stdio is started with a Claroty DTU configured
**When** an MCP `initialize` handshake is performed AND THEN a PrismQL query is
executed that triggers an outbound HTTP call to the DTU
**Then** `serverInfo.version` from the initialize response equals `V` (some version string)
**And** the `User-Agent` header on the DTU-bound HTTP request also equals `prism/V`
(same `V`)
**And** `V` equals `"1.0.0-beta.1"` (the injected PRISM_BUILD_VERSION override)

---

## Setup Instructions

This scenario runs in the same prism session as HS-032-001 and HS-032-002. Both the MCP
handshake and the outbound HTTP request must come from the SAME prism binary instance so
the version surfaces can be compared. The binary MUST be built with the override to make
the coherence assertion discriminating.

1. Build the prism binary from the S-REL-AGENT-VERSION-001 story branch HEAD commit using
   the SAME override build used for HS-032-001 and HS-032-002:
   ```
   PRISM_BUILD_VERSION=1.0.0-beta.1 cargo build -p prism-bin
   ```
   With this override, a correct implementation gives V_MCP = V_UA = "1.0.0-beta.1".
   A non-migrated Surface B gives V_UA = "prism/0.9.0" ≠ V_MCP = "1.0.0-beta.1" (incoherent).
   On plain local dev without the override, V_MCP = "1.0.0-dev" and V_UA = "prism/0.9.0" by
   design (EC-004) — asserted coherence at "1.0.0-dev" would false-fail the correct
   implementation.

2. Start the Claroty DTU locally (for request-header observation).

3. Start prism in MCP stdio mode with the Claroty sensor configured.

4. **Step A:** Issue the MCP `initialize` request. Capture `result.serverInfo.version`
   from the wire-level JSON response. Call this value `V_MCP`.

5. **Step B:** Issue a PrismQL SELECT query via the MCP `query` tool (same session, same
   prism instance) that triggers an outbound HTTP call to the Claroty DTU. Observe the
   `User-Agent` header on the inbound request at the DTU. Strip the `"prism/"` prefix
   from the User-Agent to get `V_UA`.

6. **Coherence assertion:** Assert `V_MCP == V_UA`.

7. **Value assertion:** Assert both `V_MCP` and `V_UA` equal `"1.0.0-beta.1"`.

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

5. **Value correctness:** Assert both equal `"1.0.0-beta.1"`.
   - If both are `"1.0.0-beta.1"`: record PASS on "correct-value" dimension.
   - If both are the same non-"1.0.0-beta.1" value (e.g., both "1.0.0-dev"): record PARTIAL
     on "correct-value" (coherent but override not picked up; verify build invocation from
     setup step 1 used PRISM_BUILD_VERSION=1.0.0-beta.1).
   - If `V_MCP == "0.1.0"` AND `V_UA` is `prism/0.9.0` (library version): record FAIL on
     "correct-value" (both surfaces stale; HS-032-001 + HS-032-002 already caught individual failures).

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

- **Correct value: both equal "1.0.0-beta.1"** (weight: 0.35): Are both `"1.0.0-beta.1"`
  (the injected PRISM_BUILD_VERSION override)?
  Full credit (1.0): `V_MCP = V_UA = "1.0.0-beta.1"`.
  Partial credit (0.5): coherent but not "1.0.0-beta.1" (e.g., both "1.0.0-dev"; override not
  picked up — verify build invocation in setup step 1).
  Zero credit (0.0): either stale hardcoded value, empty, or surfaces incoherent.

---

## Edge Conditions

- **V_MCP = "0.0.0-test", V_UA = "prism/1.0.0-beta.1":** Record FAIL on coherence. The
  "0.0.0-test" value indicates `PrismServer::new` (test constructor) was used at boot
  instead of `with_deps`. The with_deps wiring at boot step 9 was not applied.

- **V_MCP = "1.0.0-beta.1", V_UA = "prism/0.9.0":** Record FAIL on coherence. This is
  the PRIMARY discriminating FAIL case: Surface A correctly picks up the override
  (prism-bin build.rs step 1 resolves PRISM_BUILD_VERSION=1.0.0-beta.1, threaded through
  with_deps to get_info), but Surface B migration was not applied — pipeline.rs still uses
  `env!("CARGO_PKG_VERSION")` which resolves to prism-spec-engine's own crate version
  "0.9.0" regardless of the PRISM_BUILD_VERSION override.

- **V_MCP = "0.1.0", V_UA = "prism/1.0.0-beta.1":** Record FAIL on coherence. Surface B
  was correctly migrated (and override was picked up by prism-spec-engine/build.rs), but
  Surface A still hardcodes "0.1.0" in get_info — the with_deps wiring was not applied.

- **Both V_MCP and V_UA = "1.0.0-dev" (override not picked up by either surface):**
  Record PARTIAL on "correct-value" dimension (coherent but unexpected value). Verify that
  the build invocation in setup step 1 correctly set PRISM_BUILD_VERSION=1.0.0-beta.1.
  Both observations must come from the SAME prism instance per setup — if HS-032-001 and
  HS-032-002 were run in separate sessions, re-run this coherence check in a fresh combined
  session with the override build.

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
| corpus_source | prism binary from S-REL-AGENT-VERSION-001 branch, built with PRISM_BUILD_VERSION=1.0.0-beta.1; combined MCP initialize + Claroty DTU session |
| corpus_size | Two version observations from the same binary instance; one coherence assertion + one value assertion |
| known_edge_cases | V_MCP != V_UA (split-wiring; one surface stale); V_MCP = "0.0.0-test" (test constructor at boot); both = "1.0.0-dev" (override not picked up — coherent but PARTIAL) |
| false_positive_threshold | Zero: V_MCP == V_UA == "1.0.0-beta.1" is an unambiguous pass for the override build |
| false_negative_threshold | Zero: V_MCP != V_UA is an unambiguous split-wiring defect |

**Known-good corpus:** prism binary with both Surface A and Surface B correctly wired,
built with `PRISM_BUILD_VERSION=1.0.0-beta.1` — expected: `V_MCP = "1.0.0-beta.1"`,
`V_UA = "prism/1.0.0-beta.1"`, coherence passes.

**Known-problematic corpus:** prism binary with partial Surface B migration (Surface A
wired, Surface B still uses CARGO_PKG_VERSION), built with the same override — expected:
`V_MCP = "1.0.0-beta.1"`, `V_UA = "prism/0.9.0"`, coherence fails. This is the primary
discriminating FAIL case for the coherence scenario.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | hs-032-override-build-model-fix | 2026-09-06 | product-owner | HIGH defect fix (F-SRAV-HIGH-001 root cause). Rewrote to override-build model: evaluator builds with PRISM_BUILD_VERSION=1.0.0-beta.1. Coherence now asserted at "1.0.0-beta.1" (V_MCP == V_UA == "1.0.0-beta.1"). Local-dev plain build was non-discriminating: both surfaces legitimately diverge (Surface A = 1.0.0-dev, Surface B = 0.9.0) — asserted "1.0.0-dev" equality would false-fail correct implementation. §Edge Conditions rewritten: FAIL case is genuine mismatch V_UA="prism/0.9.0" (not the ratified local-dev state). Updated §Scenario split-wiring descriptions, §BDD, §Setup, §Verification, §Rubric, §Edge Conditions, §real-world-corpus, notes. |
| 1.0 | s-rel-agent-version-001-holdout-authoring | 2026-09-05 | product-owner | Initial authoring. HS-032 group for S-REL-AGENT-VERSION-001. Coherence test: V_MCP (serverInfo.version) must equal V_UA (User-Agent version suffix) and both must equal "1.0.0-dev" on local dev build. Catches split-wiring defect where one surface is migrated but the other is not. ADR-064 D4 §Purpose ("all surfaces emit same PRISM_VERSION") authority. SINGLE-USE. |
