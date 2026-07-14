---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [2]
feature_head_at_review: 91c8dc7f
date: 2026-07-13
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 5
  crit: 0
  high: 0
  med: 0
  low: 3
  obs: 2
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 2 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 2 (frozen 91c8dc7f; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate; PR-LEVEL cascade; streak candidate 2/3 — NOT ADVANCING — 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 5 total (0 CRIT / 0 HIGH / 0 MED / 3 LOW / 2 OBS / 0 PROCESS-GAP)

**STREAK:** 0/3 — NOT CLEAN(strict); 3 LOW findings present; streak does not advance. All 5 findings CLOSED fix-burst 15 (branch commits 91c8dc7f→6aab0f67, PUSHED; PR #222 head confirmed 6aab0f67; streak resets on push per DRIFT-ORCH-PRLEVEL-PUSH-001). NEW FROZEN HEAD 6aab0f67.

**Code HEAD at review:** 91c8dc7f (frozen; fix-burst 14 staleness-gate ancestry+version-recut 1.0.0→1.0.1 + builder pattern + ADR-051 v1.5 + timeout 20m; pushed to origin; PR #222 OPEN base develop; 5495/5495 GREEN 21/21 CI checks; non-exhaustive 91/91; CI fully green 2026-07-13)

**CLEAN(strict):** NO — 3 LOW + 2 OBS findings; strict criterion requires zero findings of any severity

**CLEAN(PR-merge):** YES — zero CRIT + HIGH + MED findings present

**SAP-1 result:** PASS — no new event_type emissions without BC-2.16.002 catalog rows

---

## Findings

### F-MCPRS-PRL2-LOW-001 — CI gate integrity: staleness gate never byte-inspects committed .prx artifact

**Severity:** LOW
**Classification:** ci-gate-integrity / staleness-gate completeness

**Finding:** The staleness-gate CI job validates the `.prx` artifact freshness via ancestry rule and manifest-hash sidecar, but NEVER performs a byte-level inspection of the committed `.prx` binary itself. Two distinct attack vectors remain uncovered: (1) **bytes-only substitution** — an attacker replaces the committed `.prx` bytes with a different WASM binary while leaving the sidecar and manifest intact; the ancestry and freshness checks both PASS since the sidecar commit is a valid ancestor of the `.prx` commit and the manifest hash matches. The gate has no proof that `.prx` bytes correspond to the declared manifest inputs. (2) **coordinated manipulation** — a contributor with write access updates both the `.prx` bytes and the sidecar in the same commit, advancing the ancestry relationship; since the gate trusts the sidecar, it cannot distinguish a legitimate rebuild from a coordinated substitution. The README §Residual Risks section documented "bytes-only substitution" as a known gap but underspecified: it cited only one vector, missing the coordinated-manipulation variant.

**Closure:** @a00a8686: README §Residual Risks expanded from one-vector to two-vector enumeration (bytes-only substitution + coordinated manipulation). Accepted-mitigation boundary documented: full byte-comparison (a 4th gate check) was adjudicated infeasible by architect due to cross-platform non-reproducibility (same manifest inputs produce different WASM bytes on Linux vs macOS due to LLVM backend differences; a committed CI-generated `.prx` would never byte-match a developer-built one). Code review at PR creation time is the primary compensating control; future SLSA-level provenance (attested build provenance) noted as the correct long-term mitigation path. Residual risk accepted at current threat model.

**Status:** CLOSED @a00a8686 (README two-vector enumeration + accepted-mitigation boundary)

---

### F-MCPRS-PRL2-LOW-002 — BC category semantic drift: 6 query-engine PrismError variants in upstream_error catch-all contradict BC-2.10.007 semantic rule

**Severity:** LOW
**Classification:** bc-category-semantic-drift / contract fidelity

**Finding:** BC-2.10.007 §Category Decision Rule defines a strict semantic distinction: `"upstream_error"` is reserved for genuine sensor-boundary failures (HTTP timeouts, auth failures, network errors from the sensor adapter). `"internal"` covers Prism-side infrastructure/invariant failures. Six `PrismError` variants were routed through the `upstream_error` catch-all arm in `error_mapping.rs` despite being query-engine failures: `QueryPlanFailed`, `QueryExecutionFailed`, `QueryMaterializationLimitExceeded`, `QueryMemoryBudgetExceeded`, `QueryVirtualFieldFailed`, `QueryDenylisted`. These variants have construction-site evidence placing them on the Prism query-engine path, not the sensor adapter path. An LLM agent receiving `category: "upstream_error"` for a `QueryPlanFailed` error would correctly interpret "fix your sensor parameters" when the real remediation is "check your PQL query syntax." BC-2.10.007 v1.11's catch-all note permitted routing unmapped variants to `upstream_error` but this was semantically correct only for genuinely-unknown third-party errors, not for named query-engine failure variants with known semantics.

**Closure:** BC-2.10.007 v1.11→v1.12 @6bc558d8 (product-owner): 6 variants adjudicated as category `"internal"` with construction-site evidence; decision rule enumeration updated; 3 test vectors added; POL-23 sweep: 22 pin sites updated (S-MCP-E003-SERIALIZATION-MIGRATION-001 v0.1→v0.2 + S-TEST-WIRESHAPE-SWEEP-001 v0.10→v0.11, 22 story-pin sites). Code @6aab0f67 (implementer): shared `VariantMeta` arm pattern — 6 variants routed to `"internal"` category via named arm; RED-first: 5 tests FAIL before @6aab0f67; 4 named category tests GREEN; `SensorHttpError` guard verified untouched (regression lock). BC-2.10.007 v1.12→v1.13 @f749ee4e (implementer-discovered correction): `ec_code_override: None` claim in v1.12 was wrong — Rule 1 (`Display`-prefix redaction) kills the automatic prefix inference when `ec_code_override` is absent; the 6 variants therefore require per-variant nested-match overrides to emit E-QUERY-002/034/005/010/008 and E-WATCHDOG-001; §LOW-002 arm code updated in spec to match shipped mechanism; S-MCP-E003 v0.2→v0.3 + S-TEST-WIRESHAPE-SWEEP v0.11→v0.12 story pins updated.

**Status:** CLOSED @6aab0f67 (code) + BC-2.10.007 v1.13 @f749ee4e (spec correction)

---

### F-MCPRS-PRL2-LOW-003 — CI toolchain supply chain: `just` task runner installed unpinned in 2 CI jobs

**Severity:** LOW
**Classification:** ci-toolchain-supply-chain / dependency pinning

**Finding:** Two CI workflow jobs (`wasm32-compile-check` and `wasm32-threatintel-staleness-check` in `.github/workflows/ci.yml`) install the `just` task runner via `cargo install just` without a version pin. This violates the supply-chain hardening principle: an upstream crates.io release of `just` at an unexpected new version could silently alter task-runner behavior or introduce a dependency conflict that breaks CI non-deterministically. The `just` install is not cached across runs in these two jobs, so every execution fetches and compiles the latest published version. A TD-VSDD-060 sibling sweep identified both jobs as affected; only one was initially in scope.

**Closure:** @a00a8686: `just@1.43.1` pinned explicitly (`cargo install just --version 1.43.1 --locked`) in BOTH `wasm32-compile-check` and `wasm32-threatintel-staleness-check` CI jobs. TD-VSDD-060 sibling sweep confirmed zero remaining unpinned `just` install sites in `.github/workflows/`.

**Status:** CLOSED @a00a8686 (just@1.43.1 pinned in both CI jobs)

---

### F-MCPRS-PRL2-OBS-001 — H8b sweep test: count-only assertion, no POL-24 byte-verbatim lock on message/suggestion content

**Severity:** OBS
**Classification:** test-assertion-completeness / POL-24

**Finding:** The H8b redundancy-sweep test (`test_BC_2_11_007_H8b_redundancy_sweep_catch_all_variants` after the pass-1 rename) asserted `audit_log_count == 1` (POL-24 single-audit-log-entry property) but did not assert the byte-verbatim content of `message` and `suggestion` fields. POL-24 (byte-verbatim lock) requires that tests asserting message/suggestion output use exact string matching, not just structural or count checks. A future refactor that inadvertently changes the message or suggestion text would pass this test silently. The test proved dedup behavior (one audit log entry, not two) but did not lock the exact wording of the user-facing fields.

**Closure:** @6aab0f67: test renamed to `test_BC_2_10_007_H8b_redundancy_sweep_audit_log_once` and structurally split into two groups: (1) query-engine-arm group — 3 variants (`QueryPlanFailed`, `QueryExecutionFailed`, `QueryDenylisted`), each asserting byte-verbatim `message = "Prism query engine failure. Contact Prism operator; see audit log for details."` + `suggestion = None` per BC-2.10.007 §LOW-002; (2) catch-all group — 2 variants (`OcsfNormalizationFailed`, representative catch-all), each asserting byte-verbatim `message = "See audit log for details."` per BC-2.10.007 §H8b. `audit_log_count == 1` property retained in both groups.

**Status:** CLOSED @6aab0f67 (test split + byte-verbatim assertions)

---

### F-MCPRS-PRL2-OBS-002 — README Provenance Anchors table missing current-version row

**Severity:** OBS
**Classification:** documentation-currency / provenance-traceability

**Finding:** The `crates/prism-spec-engine/plugins/threatintel-lookup/README.md` §Provenance Anchors section contained historical version rows but was missing a row for the current plugin version (1.0.1 after the pass-1 fix-burst 14 version re-cut). A reader auditing the plugin's provenance chain would see a gap between the prior version row and the current build, reducing confidence in the audit trail's completeness. The re-cut from 1.0.0 to 1.0.1 was the primary closure mechanism for the staleness-gate MED finding (pass 1 F-MCPRS-PRL1-MED-001), so its provenance record should be documented.

**Closure:** @a00a8686: Provenance Anchors row added for version 1.0.1: date (2026-07-13), re-cut rationale (`staleness-gate ancestry closure — version bump to create distinct commit DAG position from 1.0.0; byte-identical WASM output; re-cut is a CI artifact, not a functional change`), and commit reference (@91c8dc7f). Provenance chain is now continuous from 1.0.0 through 1.0.1.

**Status:** CLOSED @a00a8686 (Provenance Anchors row added)

---

## Summary

| Finding | Severity | Category | Closed At |
|---------|----------|----------|-----------|
| F-MCPRS-PRL2-LOW-001 | LOW | ci-gate-integrity README two-vector enumeration | @a00a8686 |
| F-MCPRS-PRL2-LOW-002 | LOW | bc-category-semantic-drift 6 query-engine variants | @6aab0f67 (code) + BC-2.10.007 v1.13 @f749ee4e |
| F-MCPRS-PRL2-LOW-003 | LOW | ci-toolchain-supply-chain just@1.43.1 pinned 2 jobs | @a00a8686 |
| F-MCPRS-PRL2-OBS-001 | OBS | H8b test split + byte-verbatim audit_log_once | @6aab0f67 |
| F-MCPRS-PRL2-OBS-002 | OBS | README Provenance Anchors current-version row | @a00a8686 |

**Prior CI run (on 91c8dc7f):** 21/21 checks GREEN — including the new staleness-gate test added in fix-burst 14; CI fully validated before pass-2 adversary review.

**New frozen HEAD after fix-burst 15:** 6aab0f67 (pushed; PR #222 OPEN; new CI run pending)

**PR-LEVEL streak:** 0/3 (DRIFT-ORCH-PRLEVEL-PUSH-001 reset on push of 6aab0f67)

**NEXT:** PR-LEVEL pass 3 on frozen 6aab0f67
