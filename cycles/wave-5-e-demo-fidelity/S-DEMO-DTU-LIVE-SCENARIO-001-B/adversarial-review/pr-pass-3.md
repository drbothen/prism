---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr_number: 185
pass_number: 3
cascade: PR-LEVEL (distinct from LOCAL; LOCAL CONVERGED 3/3 strict @13 passes)
base_develop: "939f36ce"
feature_head_at_review: "4eadb027"
feature_head_after_fix_burst: "13efc875"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-12
authority: BC-5.39.001 D-779
decision: D-1108
---

# PR-LEVEL Adversary Pass 3 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B — Scenario Progression + Enrichment Correlation Live Demo
**PR:** #185 (base develop@939f36ce, head 4eadb027 at review)
**Pass:** PR-LEVEL pass 3 (distinct from LOCAL cascade; LOCAL CONVERGED 3/3 strict at 13 passes)
**Date:** 2026-06-12

## Pass-2 Closure Verification

BPRL-P2-01 MED (cyberint alerts StageMask projection unimplemented; spec-wins per §FSR + BC-2.06.019 PC-4) closed at commit 4eadb027. StageMask projection applied; HTTP-level IOC-filter tests Red→Green. Threats route adjudicated static-fixture out-of-PC-4-scope. Doc comments corrected. VERIFIED LOAD-BEARING at HEAD 4eadb027.

## Adversary Pass 3 Findings

### BPRL-P3-01 — CLAUDE.md #[non_exhaustive] count sentence stale (50 vs ci.yml EXPECTED=52)

**Finding ID:** BPRL-P3-01
**Severity:** MEDIUM
**Category:** Cross-document count propagation (POL-25 count-propagation rule / POL-29 sibling-site sweep)

**Description:** CLAUDE.md §Conventions reads: "50 types currently enforced via the compile-fail gate at `tests/external/non-exhaustive-violation/` (AC-5 of S-PLUGIN-PREREQ-C, expanded by S-DEMO-DTU-LIVE-SCENARIO-001-A AC-014; `ci.yml EXPECTED=50` is the authority — when the gate grows, update BOTH ci.yml and this sentence)."

PR #185 (Story B, commit 2323cf37 diff reviewed at 4eadb027 on the branch) added new public types covered by the non-exhaustive gate. The authoritative ci.yml `EXPECTED` value at the PR head is `52`, yet the CLAUDE.md sentence still reads `EXPECTED=50`. The rule's own text ("when the gate grows, update BOTH ci.yml and this sentence") demands the dual-update. Failure to apply it violates the documented POL-25/POL-29 co-update discipline.

**D-1108 Human Decision (2026-06-12, via orchestrator AskUserQuestion):** CLAUDE.md 50→52 edit RATIFIED for delivery IN THIS PR BRANCH (feature/S-DEMO-DTU-LIVE-SCENARIO-001-B) rather than as a post-merge human-only edit. Rationale: in-PR keeps ci.yml + CLAUDE.md atomic at merge, eliminating the mismatch window between ci.yml update and CLAUDE.md update that the rule explicitly forbids. SUPERSEDES the D-1106 §4 post-convergence step 5 plan ("CLAUDE.md EXPECTED 50→52 = HUMAN edit").

**Closure:** CLOSED by fix-burst (commits 2323cf37 + 13efc875).

- Commit `2323cf37`: CLAUDE.md §Conventions count sentence updated 50→52; provenance sentence extended to include Story B's contributed types; ci.yml `EXPECTED=52` already set in this commit.
- Commit `13efc875` (sibling sweep per TD-VSDD-060/POL-29): `scripts/check-non-exhaustive.sh` EXPECTED 50→52 + provenance line updated; `tests/external/non-exhaustive-violation/src/struct_violations.rs` two doc-comment sites updated 50→52. Workspace-wide grep confirmed zero remaining live `EXPECTED=50` sites (historical 001-A evidence reports intentionally untouched as point-in-time records). Local gate verified `PASS: 52 (expected: 52)`. `just check` PASS (930s).

**Status: CLOSED (load-bearing: new count enforced by gate + all prose sites updated).**

---

### BPRL-P3-OBS-1 — cyberint alerts.rs silent `unwrap_or("ip")` default when `_ioc_type` absent

**Finding ID:** BPRL-P3-OBS-1
**Severity:** OBSERVATION
**Category:** Fail-closed discipline (CLAUDE.md §Conventions error handling)

**Description:** In `crates/prism-dtu-cyberint/src/routes/alerts.rs`, the field `_ioc_type` (IOC type discriminator) was extracted with `unwrap_or("ip")`, silently defaulting to `"ip"` when the field was absent or unrecognized. This is a fail-open pattern: records with absent or unrecognized IOC type would be returned with a fabricated `"ip"` classification, masking data quality problems and producing incorrect OCSF normalization.

**D-1108 Human Decision (2026-06-12):** Both OBS findings fixed in the same fix-burst rather than adjudicated as by-design.

**Closure:** CLOSED by fix-burst (commit `2323cf37`). The `unwrap_or("ip")` call replaced with a fail-closed match: absent or unrecognized `_ioc_type` → record withheld from response (not emitted with a fabricated default). New test `test_BC_2_06_019_cyberint_ioc_value_without_ioc_type_withheld` added to cover the new fail-closed behavior; `just check` PASS (897s).

**Status: CLOSED (load-bearing: new test exercises the fail-closed path).**

---

### BPRL-P3-OBS-2 — crowdstrike hosts.rs scenario stage-projection silently supersedes operator containment_store at stage<4

**Finding ID:** BPRL-P3-OBS-2
**Severity:** OBSERVATION
**Category:** Undocumented precedence (CLAUDE.md §Conventions doc comment discipline)

**Description:** In `crates/prism-dtu-crowdstrike/src/routes/hosts.rs`, the scenario stage-projection logic silently supersedes the operator-defined `containment_store` value for device records at scenario stage < 4. Per BC-2.06.019 Postcondition-4, devices progress through containment stages deterministically according to scenario stage; the operator-set containment state is overridden without documentation at the overriding call site. A reader unfamiliar with BC-2.06.019 PC-4 would not understand why the `containment_store` field is effectively ignored in the scenario-progression path.

**D-1108 Human Decision (2026-06-12):** Fixed in same fix-burst.

**Closure:** CLOSED by fix-burst (commit `2323cf37`). Doc comment added at the projection call site explicitly citing BC-2.06.019 Postcondition-4 and the precedence rule: "scenario stage-projection supersedes operator containment_store for stage<4 (BC-2.06.019 PC-4; by-design for deterministic demo reproducibility)." `just check` PASS (897s).

**Status: CLOSED (doc comment explains the by-design precedence; load-bearing in conjunction with the pre-existing test coverage).**

---

## Standing Probe Results

### SAP-1 (Tracing emission catalog completeness)

`rg 'event_type\s*=' crates/ --type rust` run against the Story B diff scope. Zero new `event_type =` emissions introduced by Story B changes. **SAP-1: PASS.**

### SAP-2 (DTU↔TOML schema parity)

No sensor TOML files modified in this PR diff. **SAP-2: N/A.**

### POL-22 Phase A (CLAUDE.md canonical principle self-audit)

No rationalization anti-patterns ("MVP", "for now", "good enough", "fix later") present in the fix-burst commits. **POL-22 Phase A: PASS.**

### POL-22 Phase C (No-silencing of findings)

All three findings addressed substantively; none silenced via doc-comment-only or rename. **POL-22 Phase C: PASS.**

---

## Convergence Status

```
CLEAN (strict): no  — BPRL-P3-01 MED + BPRL-P3-OBS-1 OBS + BPRL-P3-OBS-2 OBS present at review HEAD (4eadb027); all fixed in commits 2323cf37 + 13efc875
CLEAN (PR-merge): no  — BPRL-P3-01 MED was blocking at review; fixed before this report is declared closed
```

**Streak after pass 3: 0/3** (BPRL-P3-01 MED finding resets streak per BC-5.39.001 D-779).

**Story B branch HEAD after fix-burst: `13efc875` (= remote; pushed 2026-06-12).**

**NEXT: PR-LEVEL pass 4** — dispatch fresh adversary on PR #185 with updated do-not-reflag list including all BPRL-P1/P2/P3 closures.
