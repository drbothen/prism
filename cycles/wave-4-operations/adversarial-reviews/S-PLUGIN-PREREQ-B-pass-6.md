---
document_type: adversarial-review
level: LOCAL
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T23:45:00Z
phase: 3
inputs: []
input-hash: "2fe7068"
traces_to: prd.md
pass: 6
previous_review: S-PLUGIN-PREREQ-B-pass-5.md
target_artifact: S-PLUGIN-PREREQ-B
target_sha: 2fe7068c
base_sha: 90d7c80f
verdict: BLOCKED-soft
streak: 0/3
finding_summary: { critical: 0, high: 2, medium: 3, low: 1, obs: 3 }
prior_passes: passes 1-5 + 5 fix-bursts; pass-6 verified fix-burst-5 eager-token closure CLEAN, surfaced novel from P6-A..K dimensions
---

# Adversarial Review: S-PLUGIN-PREREQ-B (Pass 6)

**Verdict:** BLOCKED-soft — streak stays **0/3** (2H + 3M block advancement)
**Target SHA:** 2fe7068c
**Base SHA:** 90d7c80f (develop HEAD, S-PLUGIN-PREREQ-A merged)
**Finding summary:** 0 CRITICAL / 2 HIGH / 3 MEDIUM / 1 LOW / 3 OBS
**Streak:** 0/3 (no advancement; 2H + 3M block)
**Trajectory:** 20→10→4→7→10→9 (non-monotonic; pass-6 found 5 actionable defects in NEW dimensions P6-A..K)

## Finding ID Convention

Finding IDs use the format: `F-LP6-<SEV>-<SEQ>` (LOCAL pass 6, S-PLUGIN-PREREQ-B cascade).

## Part A — Fix Verification (pass 5 closures)

All fix-burst-5 closures verified CLEAN at HEAD 2fe7068c. No paper-closes detected.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP5-LOW-003 | LOW (DESIGN-LEVEL) | RESOLVED | `pipeline.rs:138` now acquires token eagerly at pipeline start. `auth_initial_acquired` / `auth_initial_failed` audit events present. `request_count` semantics correct (HTTP-only, not counting auth roundtrip). BC-2.16.002 v1.5 amendment aligns with implementation. Substantive behavioral change — paper-fix-free confirmed. |

**KUDOs (fix-burst-5 closure verification):**
- Eager-token at pipeline start (`pipeline.rs:138`) is the correct architectural choice — eliminates double-request on first step and accurate `request_count` semantics.
- BC-2.16.002 v1.5 amendment with precondition lifecycle change (lazy→eager) + 3 new postconditions demonstrates complete spec→implementation alignment.
- Two new Red Gate tests + 2 adjusted tests show disciplined Red Gate maintenance (37→39).

**Part A verdict: ALL CLEAN. Fix-burst-5 closures survive fresh-context scrutiny.**

## Part B — New Findings (Pass 6 Dimensions: P6-A through P6-K)

### CRITICAL

_None._

### HIGH

#### F-LP6-HIGH-001: VP-PLUGIN-002 anchor drift — VP-INDEX:168/184 vs story frontmatter

- **Severity:** HIGH
- **Category:** spec-fidelity / verification-gaps
- **Location:** `.factory/specs/verification-properties/VP-INDEX.md:168,184`; `.factory/stories/S-PLUGIN-PREREQ-B.md` frontmatter
- **Description:** VP-INDEX.md at line 168/184 states the anchor for VP-PLUGIN-002 is `PLUGIN-MIGRATION-001-D`, but the story v1.5 frontmatter declares `PREREQ-B` as the anchor for this verification property. The semantic description in VP-INDEX is "Unknown sensor registers without code change" — this describes the open-newtype extensibility property from PREREQ-A, not the HTTP pipeline execution behavior tested in PREREQ-B. The actual test target for PREREQ-B is "PipelineExecutor returns non-empty records against wiremock" (integration-level execution verification). The VP-INDEX description mismatch means a reader cannot determine which story owns which VP, and automated traceability tooling will misroute.
- **Evidence:** VP-INDEX.md lines 168/184 cite `PLUGIN-MIGRATION-001-D` anchor + "Unknown sensor registers" description. Story frontmatter `verification_properties:` lists `VP-PLUGIN-002`. The descriptions are semantically disjoint (extensibility vs execution integration).
- **Proposed Fix:** Correct VP-INDEX.md line 168/184: (1) assign VP-PLUGIN-002 anchor to `PREREQ-B` (not `PLUGIN-MIGRATION-001-D`); (2) update semantic description to match PREREQ-B's actual verification target ("PipelineExecutor returns non-empty records against wiremock").

#### F-LP6-HIGH-002: VP-INDEX.md internal contradiction — VP-PLUGIN-005 dual definition (line 171 vs 187)

- **Severity:** HIGH
- **Category:** spec-fidelity / contradictions
- **Location:** `.factory/specs/verification-properties/VP-INDEX.md:171` vs `:187`
- **Description:** VP-PLUGIN-005 appears TWICE in VP-INDEX.md with the SAME alias mapping (`VP-PLUGIN-005 = VP-150`) but different semantic descriptions. Line 171 describes authentication token lifecycle (eager-acquire, refresh on 401, audit events). Line 187 describes plugin step record isolation (intermediate records excluded from PipelineResult). ADR-023 and BC-2.16.002 both confirm line 171's description (token lifecycle) is the correct semantic for VP-PLUGIN-005. Line 187 is either a duplicate entry with wrong content or a mis-numbered VP that should be VP-PLUGIN-006 or similar.
- **Evidence:** VP-INDEX.md contains two rows with alias `VP-PLUGIN-005 = VP-150` at lines 171 and 187. The descriptions are semantically disjoint and cannot both be correct for the same VP. ADR-023 §auth + BC-2.16.002 v1.5 postconditions confirm the token-lifecycle description at line 171 is authoritative.
- **Proposed Fix:** Remove the duplicate at line 187 (or renumber to VP-PLUGIN-006 if it is a distinct VP accidentally assigned the same alias). Verify that BC-2.16.002 v1.5 postconditions map correctly to the surviving VP-PLUGIN-005 entry at line 171.

### MEDIUM

#### F-LP6-MED-001: NullAuthProvider + MockAuthProvider publicly re-exported without feature-gate (P6-A: public-API leakage)

- **Severity:** MEDIUM
- **Category:** security-surface / purity-boundary-violations
- **Location:** `crates/prism-spec-engine/src/lib.rs:89`; `crates/prism-spec-engine/src/auth/providers/pipeline.rs:709`
- **Description:** `lib.rs:89` unconditionally re-exports both `NullAuthProvider` and `MockAuthProvider` as part of the crate's public API. These are test/null-object implementations — `NullAuthProvider` returns an empty `AuthToken` (empty string), and the authentication gate at `pipeline.rs:709` is `if !token.as_str().is_empty()`. A downstream caller that constructs a `NullAuthProvider` for production use will silently skip all authentication, sending unauthenticated HTTP requests to sensor APIs. There is no `#[cfg(test)]` or `#[cfg(feature = "test-support")]` attribute preventing production use. Cargo.toml has no `test-support` feature that downstream crates must opt into.
- **Evidence:** `lib.rs:89` unconditional re-export. `pipeline.rs:709` gate: `if !token.as_str().is_empty()`. No feature flag or cfg attribute on the re-export line.
- **Proposed Fix:** Either: (a) gate exports with `#[cfg(test)]` (prevents use outside test contexts entirely), OR (b) add a `test-support` feature flag and gate `NullAuthProvider`/`MockAuthProvider` exports behind `#[cfg(feature = "test-support")]`. Orchestrator decides (a) vs (b) — if downstream integration tests need `NullAuthProvider`, option (b) is required.

#### F-LP6-MED-002: story frontmatter missing VP-PLUGIN-005 despite 8 body references (P6-J: traceability sweep)

- **Severity:** MEDIUM
- **Category:** spec-fidelity / verification-gaps
- **Location:** `.factory/stories/S-PLUGIN-PREREQ-B.md` frontmatter `verification_properties:` array
- **Description:** The story body references `VP-PLUGIN-005` in 8 distinct locations across acceptance criteria, implementation tasks, and Red Gate test descriptions. However, the frontmatter `verification_properties:` array lists only `VP-PLUGIN-002`. VP-PLUGIN-005 is entirely absent from the frontmatter registry. Automated traceability queries on frontmatter will miss VP-PLUGIN-005 as a PREREQ-B requirement. VP-INDEX back-link traversal via frontmatter will not surface VP-PLUGIN-005. POL-14 post-merge BC/VP promotion workflows that read frontmatter will not promote VP-PLUGIN-005 status.
- **Evidence:** Story body: 8 occurrences of `VP-PLUGIN-005` (grep-confirmed). Story frontmatter `verification_properties: [VP-PLUGIN-002]` — VP-PLUGIN-005 absent.
- **Proposed Fix:** Add `VP-PLUGIN-005` to the story frontmatter `verification_properties:` array alongside existing `VP-PLUGIN-002`.

#### F-LP6-MED-003: `execute_step` sibling not patched — hardcoded empty AuthToken at pipeline.rs:444 (S-7.01 sibling regression)

- **Severity:** MEDIUM
- **Category:** spec-fidelity / security-surface
- **Location:** `crates/prism-spec-engine/src/pipeline.rs:436-483` (`execute_step` function); specifically line 444
- **Description:** The v1.5 eager-token fix was applied to `PipelineExecutor::execute` (pipeline.rs:138 — the primary entry point). However, `execute_step`, a sibling `pub` function at pipeline.rs:436-483, was not patched. `execute_step` at line 444 constructs `AuthToken::new("")` — a hardcoded empty `AuthToken`. This is the pre-fix lazy/null pattern. The docstring at line 415 confirms `execute_step` is used by PREREQ-D plugin-runtime contexts. This is a textbook S-7.01 sibling-fix regression: the fix was applied to the primary callsite (`execute`) but not to the sibling public API (`execute_step`). PREREQ-D callers using `execute_step` will send unauthenticated HTTP requests (empty AuthToken → skip-auth gate at line 709 fires). BC-2.16.002 v1.5 precondition requiring eager token acquisition does not apply to `execute_step` paths.
- **Evidence:** `pipeline.rs:138` patched (eager acquire). `pipeline.rs:444` unpatched: `AuthToken::new("")`. Function signature is `pub`. Docstring line 415: "used by PREREQ-D plugin-runtime contexts."
- **Proposed Fix:** Apply the same eager-token acquisition pattern to `execute_step`. Since `execute_step` likely needs to accept an existing token (not re-acquire), the fix may involve threading `AuthToken` as a parameter or extracting shared auth initialization into a helper called by both `execute` and `execute_step`. Orchestrator must decide threading approach before implementer dispatches.

### LOW

#### F-LP6-LOW-001: BC-2.16.002 input-hash unchanged through v1.4 and v1.5 amendments (P6-J: pending intent verification)

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `.factory/specs/behavioral-contracts/BC-2.16.002.md` frontmatter `input-hash:`
- **Description:** BC-2.16.002 was amended from v1.4 to v1.5 (lazy→eager precondition change + 3 new/amended postconditions). The frontmatter `input-hash:` field appears unchanged between v1.4 and v1.5. If the v1.5 amendment was driven by the same PRD/story source artifacts as v1.4 (i.e., only the interpretation changed), the hash should remain the same — correct behavior. If new input artifacts drove the v1.5 amendment (the adversary finding F-LP5-LOW-003 report, the orchestrator decision), the input-hash must reflect those new inputs.
- **Evidence:** BC-2.16.002 v1.4 and v1.5 frontmatter `input-hash:` field — unchanged value. Amendment scope: 3 substantive postcondition changes.
- **Proposed Fix:** State-manager confirms whether input-hash update is required for v1.5 amendment context. If new input artifacts drove the amendment, recompute and update. If the PRD source is unchanged, mark this finding CLOSED as non-defect.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 3 |
| LOW | 1 |
| OBS | 3 |

**Overall Assessment:** block
**Convergence:** findings remain — fix-burst-6 required
**Readiness:** requires revision (2H + 3M must close before streak can advance)

**fix-burst-6 scope (parallel dispatch recommended):**
- **Implementer track:** F-LP6-MED-001 (feature-gate NullAuth/Mock — orchestrator decides (a) cfg(test) vs (b) test-support feature); F-LP6-MED-003 (execute_step eager-token threading — orchestrator decides approach before dispatch)
- **Product-owner track:** F-LP6-HIGH-001 (VP-INDEX anchor + description correction); F-LP6-HIGH-002 (VP-PLUGIN-005 duplicate row removal); F-LP6-MED-002 (story frontmatter VP-PLUGIN-005 addition)
- **State-manager track:** F-LP6-LOW-001 intent verification (BC-2.16.002 input-hash); acknowledge OBS-LP6-001/002/003

**Observations (Non-Blocking):**

**OBS-LP6-001:** NullAuth eager-call emits `auth_initial_acquired` with empty token — log-semantic ambiguity (event name implies success but empty token is indistinguishable from zero-length credential in log output). Non-blocking. Recommend docstring annotation in NullAuthProvider.

**OBS-LP6-002:** `PipelineExecutor::execute` has zero production callsites (pub but test-only). No compile-time perimeter preventing Wave 1 wire-in regression. Expected at PREREQ-B stage; PREREQ-D wires the boot.rs callsite. Recommend filing TD for production callsite assertion after PREREQ-D lands.

**OBS-LP6-003:** Test asserts no `auth_refresh_triggered` via `calls()==1` proxy rather than direct tracing capture. Indirect assertion will false-alarm if parallel fan-out calls `acquire_token` multiple times for non-refresh reasons. Non-blocking for single-pipeline model. Recommend direct tracing capture in future proptest hardening pass (per TD-S-PLUGIN-PREREQ-B-006).

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 6 |
| **New findings** | 9 (2H + 3M + 1L + 3O — all from dimensions not covered in passes 1-5) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 9/9 = 1.0 (all genuinely new) |
| **Median severity** | MEDIUM (3.0 on 1-5 scale) |
| **Trajectory** | 20→10→4→7→10→9 |
| **Verdict** | FINDINGS_REMAIN — 2H + 3M block streak advancement; fix-burst-6 required |
