<!-- canonical pass-19, adversary=vsdd-factory:adversary fresh-context, frozen HEAD 9c315608, 2026-07-17, 1 MED + 1 LOW — streak 0/3 -->

# Adversarial Review — PR-LEVEL Pass 19
## S-MAINT-CI-DISK-EXHAUSTION-001 (PR #224, maintenance/ci-disk-hardening → develop)
**Finding count:** 0 CRITICAL · 0 HIGH · 1 MEDIUM · 1 LOW · 0 OBS · 0 PROCESS-GAP

## Frozen-HEAD / PR-State Verification
PR OPEN @ 9c315608, base develop, merge-base 84062ced; CI 4/4 green at HEAD (29615362774/29615362767/29615362869 pull_request + 29615360213 push); diff scope ci.yml + e2e.yml only. All ✓.

## SEC-Commit Verification
SEC-001 (CWE-272) VERIFIED CORRECT & COMPLETE for stated scope: permissions contents:read at ci.yml L86-87 (test) + L353-354 (tndef); e2e already carried it; contents:read breaks nothing (checkout read-only; setup-protoc authenticated read; upload-artifact uses runtime token); 4 green runs confirm. Scope note (not a finding): only the 2 story-modified jobs got blocks — workflow-wide permissions refactor is outside this story per the human-approved SEC scope. SEC-002 (CWE-319) VERIFIED COMPLETE: all 13 printf sites HTTPS-only 2-mirror (ci.yml L64,208,245,284,445,478,834,953,1036,1277,1342,1867; e2e L123); zero residual http:// mirrors (remaining http:// tokens are comment prose or non-apt placeholder env values); renumbering coherent. NO Red-Gate assertion broken (RG-1..RG-8/5b/7b counts re-verified: 2/2/12/12/2/1/1/2; no assertion inspects mirror content). Consistent with 4/4 green.

## F-MAINT-P19-MED-001 — SEC-001 spec-sync incomplete: story scope-contract still forbids the shipped permissions block on test-no-default-features
Severity MEDIUM, confidence HIGH. Routing: product-owner. Story §Architecture Compliance Rules (L1143-1148) and §Forbidden Patterns (L1207) pin an EXHAUSTIVE modification allowlist for the test-no-default-features job (four v0.6 protective steps + neutralization + AC-006 wrapper + AC-007 toolchain) that omits the shipped permissions block (ci.yml L353-354); changelog v0.24 (L1280) inaccurately claims "story does not pin job-level permissions posture so no body edits required." The test matrix job has no analogous exhaustive allowlist — gap specific to tndef. Risk: future fresh-context reviewer flags the shipped security control as out-of-scope and reverts it. Per Source-of-Truth Precedence Rule 7 the spec is amended to match the human-approved code: add a permissions-hardening carve-out to both body sites (or correct the changelog claim). Blast radius: 2 body sites, same document.

## F-MAINT-P19-LOW-001 — Changelog apt mirror+file priority-ordering aside likely inverted (non-load-bearing)
Severity LOW, confidence MEDIUM. Routing: product-owner. Changelog v0.24 (L1280) aside "(higher number = tried first under mirror+file: protocol)" — conventional apt semantics are LOWER number = tried first (runner default lists primary at priority:1). Non-load-bearing: appears only in the changelog aside (not AC-006 prose, YAML, or PR body), and archive vs security serve different suites so behavior is correct regardless. Resolution: delete or correct the parenthetical. Confidence caveat: semantics not authoritatively verifiable read-only; PO to verify.

## AC-005 Dual-Reading Adjudication (PENDING HUMAN RULING — not a finding)
Literal reading SATISFIED: 3 distinct green pull_request run IDs at frozen HEAD (29615362774/29615362767/29615362869) + push run. Distinct-trigger-event reading (F-MAINT-P10-OBS-008): 1/3 — all four runs derive from the single SEC push; two further independent trigger events needed. Both recorded; adjudication deferred to human.

## Version-Pin Lattice
CLEAN — story frontmatter v0.24 == STORY-INDEX row L812 (ready v0.24, D-1814, HEAD 9c315608) == STORY-INDEX changelog v2.703 == PR body (L5,12,199,402-403).

## POL-22 / POL-21
Phase A CLEAN (all cited internal entities resolve; no phantom citations). Phase C CLEAN (STORY-INDEX row text matches frontmatter; behavioral_contracts [] CONFORMING). POL-21 CLEAN.

## SAP-1
N/A — verified by diff-list (workflow YAML only; zero new tracing sites; PR body L397 concurs).

## Partial-Fix Regression Check
SEC-002 propagation COMPLETE (13 story snippet sites + 13 runtime sites HTTPS-only; EC-010/013/015/016 prose updated; no residual current-state http://archive). SEC-001 propagation INCOMPLETE → MED-001. Changelog monotonic v0.24→v0.1 (POL-32).

## Novelty Assessment
MEDIUM — MED-001 is a new class (scope-contract consistency of the SEC sync, not runtime correctness); surfaced because the SEC fix added a job-level key the story's exhaustive step-allowlists never anticipated. Core hardening mechanism stable and converged; SEC-002 + Red-Gate lattice clean. Not a retread.

## Dual Verdict
CLEAN (strict): no
CLEAN (PR-merge): no
Rationale: 1 MEDIUM blocks both gates. Streak 0/3 on frozen 9c315608. Routing: product-owner spec-body fix-burst (§ACR + §Forbidden Patterns carve-out + changelog claim correction + LOW-001 parenthetical). Spec-only — no code push required; frozen-HEAD CI evidence remains valid.
