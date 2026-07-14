---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [11]
feature_head_at_review: 6b2a7c8e
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 4
  crit: 0
  high: 1
  med: 1
  low: 1
  obs: 1
  process_gap: 1
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 11 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 11 (frozen 6b2a7c8e; fresh-context adversary; PR #222 MCP row-shape null serialization + [H8b] redundancy sweep + threatintel .prx staleness gate + retryable-coverage expansion + 28-explicit-VariantMeta-arms + 117-variant sentinel + ci.yml sidecar diagnostic + scripts/hash-plugin-source.py repo_root anchoring; PR-LEVEL cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

---

## Findings

### F-MCPRS-PRL11-HIGH-001 [HIGH][semantic-anchor/paper-fix/cross-cascade] — CLOSED-CROSS-BRANCH (orchestrator adjudication 2026-07-14)

**Severity:** HIGH (semantic-anchor / paper-fix / cross-cascade)
**Classification:** semantic-anchor — BC-2.11.019 (v1.16) §Injection-safety claim without load-bearing test on this branch
**BC:** BC-2.11.019 v1.16

**Finding:** BC-2.11.019 (v1.16) §Injection-safety postcondition claims `sanitize_for_log` is applied at `EnrichUdfNotFoundDetails::new` construction time and cites a load-bearing test `test_f_pqlfn_p7_low_001_enrich_udf_infusion_cc_stripped_at_construction`. On the MCP branch (which is develop-based), `crates/prism-core/src/error.rs` stores `infusion.into()` verbatim and the cited test does not exist. The adversary cannot distinguish "the fix lives on another branch" from "the fix was never applied" without cross-branch verification.

**Impact:** If PR #222 merges before the PQL fix PR, the `EnrichUdfNotFoundDetails` constructor on develop would store the infusion string verbatim — BC-2.11.019 §Injection-safety unenforced until the PQL PR lands. Severity HIGH because the BC carries an injection-safety postcondition (CWE-117/AD-017 class) and the citing test is the load-bearing proof.

**Resolution — CLOSED-CROSS-BRANCH (orchestrator adjudication 2026-07-14):** Verified that both the production fix (`error.rs:89 sanitize_for_log(&infusion.into())`) and the cited load-bearing test (`crates/prism-core/src/tests/test_enrich_udf_not_found_display.rs`) exist on `fix/DEFECT-PQL-FNCALL-LHS-001`, landed at commit 3e0d3585 (fix-burst-6, F-PQLFN-P7-LOW-001 SEC-001 sanitize parity). The defect was not introduced by PR #222 — the develop status quo (before either defect fix) does not have the constructor; the fix was authored on the PQL lane for full context-coupling reasons. Not a regression introduced by this branch.

**MERGE-GATE NOTE (record verbatim — both merge gates must surface this):** If PR #222 merges before the PQL fix PR, BC-2.11.019 §Injection-safety remains unenforced on develop until the PQL PR lands. This must be surfaced at BOTH merge gates: (1) the human reviewing PR #222 merge must be informed that BC-2.11.019 §Injection-safety enforcement rides the PQL PR; (2) the human reviewing the PQL PR merge must confirm both the constructor fix and the load-bearing test are present in the squash-merge. Added to closed classes for MCP passes 12+.

---

### F-MCPRS-PRL11-MED-001 [MED][POL-23 version-cell drift] — CLOSED (story v0.18; TD-VSDD-060 residual grep clean)

**Severity:** MED
**Classification:** POL-23 version-cell drift — story body BC table cell stale pin
**BC:** BC-2.10.007 v1.18

**Finding:** `S-TEST-WIRESHAPE-SWEEP-001` body §Behavioral Contracts reference table line ~139 carried `| BC-2.10.007 | v1.13 |` — the cell was pinned at v1.13 and was missed by both the v0.13 sweep (which updated 12 prose citation sites) and the v0.17 sweep (which updated 12 prose sites to v1.18 via the D-1751 POL-23 round). All other BC-2.10.007 references in the file correctly read v1.18 after the v0.17 sweep; this one markdown table-cell variant in the §Behavioral Contracts body table was overlooked.

**Impact:** MED because a story contributor consulting the §Behavioral Contracts table for implementation guidance would see v1.13 for BC-2.10.007 and consult a 5-version-stale specification, missing the 28-explicit-arm categorization, §Rule 2 catch-all future-only amendment, 117-variant sentinel, and dedicated safety arm for SafetyContextContamination/SafetyDataExfiltration. Spec accuracy defect at the story implementation-guidance layer.

**Resolution (story v0.17→v0.18; TD-VSDD-060 residual grep clean):** Story-writer updated the body BC table cell at line ~139 from `v1.13` to `v1.18`. TD-VSDD-060 residual grep confirms zero remaining live-prose stale `v1.13`/`v1.14`/`v1.15`/`v1.16`/`v1.17` BC-2.10.007 pins outside the §Changelog section. Story version bumped 0.17→0.18. Historical §Changelog rows referencing `v1.17` as a destination (the v0.17 row) left untouched per TD-VSDD-091.

---

### F-MCPRS-PRL11-LOW-001 [LOW][ci-invariant-precision] — CLOSED @c82f30ba (two-case structured diagnostic)

**Severity:** LOW
**Classification:** ci-invariant-precision — ci.yml sidecar ancestor-check diagnostic incorrect for byte-identical rebuild edge case
**BC:** N/A (CI infrastructure hygiene)

**Finding:** The ci.yml sidecar ancestor-check diagnostic emitted a single message for both failure modes: (a) the plugin was not rebuilt at all (run `just build-plugin-threatintel-infusion`), and (b) the plugin was rebuilt but produced a byte-identical binary (the .prx was not re-committed because the content didn't change). Case (b) produces the same "not rebuilt" symptom in the CI check because no new .prx commit appears, but the guidance in case (a) ("run `just build-plugin-threatintel-infusion`") would deadlock a developer in case (b): they run the command, get an identical binary, commit-amend, and the check fires again.

**Impact:** Developer ergonomics defect — a byte-identical rebuild causes a false CI failure with misleading remediation guidance. LOW because the check itself is correct (the .prx staleness gate fires correctly); only the diagnostic message is imprecise.

**Resolution @c82f30ba:** Implementer added a two-case structured diagnostic to ci.yml: (a) plugin not rebuilt → `run just build-plugin-threatintel-infusion`, and (b) byte-identical rebuild → `amend the sidecar commit with a .prx touch / commit the .prx and sidecar together`. The check logic is unchanged (architect Option c design preserved). YAML validated.

---

### F-MCPRS-PRL11-OBS-001 [OBS][hardening][process-gap] — CLOSED @c82f30ba (repo_root anchoring + fail-loud guard)

**Severity:** OBS (hardening / process-gap)
**Classification:** hardening — scripts/hash-plugin-source.py path resolution silently produced wrong hash from non-root cwd
**BC:** N/A (CI toolchain hygiene)

**Finding:** `scripts/hash-plugin-source.py` used `git ls-files` to enumerate tracked source files, then resolved relative paths against the current working directory (`CWD`). When invoked from any directory that is not the repository root, `git ls-files` enumerates repo-root-relative paths (e.g., `crates/prism-spec-engine/plugins/threatintel-lookup/src/lib.rs`), but the path resolver joined them against `CWD` — producing non-existent absolute paths. Each non-existent path was silently skipped; the script printed the SHA-256 of the empty input string (`sha256("")` = `e3b0c44298fc1c149afb...`). The committed sidecar hash `ac5bf335ea7b2a30...` was correct (script was run from repo root at commit time), but any CI invocation or developer invocation from a subdirectory would silently produce a wrong hash, causing spurious staleness-gate failures or, worse, silently passing with an empty-corpus hash.

**Impact:** Silent wrong-hash failure mode. The staleness gate is only as reliable as the hash script. An attacker who knows about the non-root cwd behavior could invoke the script from a non-root path to produce a trivially-colliding empty hash. Severity: OBS (practical exploit risk LOW given developer-tooling context; the structural gap is real and exploitable).

**Resolution @c82f30ba:** Repo_root anchoring via `git rev-parse --show-toplevel`; `ls-files cwd=repo_root`; all files resolved as `repo_root/rel_path`. Added fail-loud guard: if tracked files are non-empty but zero files were hashed, the script exits 1 with an explicit error. Verified: root and subdirectory invocations both produce `ac5bf335ea7b2a3036062f2ca1a2f188ab99388e8c39627bd81595bbfb131945` matching the committed sidecar (hash output unchanged for same inputs; anchoring fix is transparent to correct-cwd callers).

---

## SAP-1 Emission Catalog Probe

**PASS.** All ~89 `crates/` `event_type =` emissions at HEAD 6b2a7c8e map to BC-2.16.002 §Postconditions Canonical Structured Event Catalog. The two documented exemptions (`credential_access` and `boot.audit.initialized`) carry explicit catalog notes. No new `event_type` emissions introduced by the branch relative to develop@5f1b5771.

---

## Positive Verifications

- **BC-2.10.007 v1.18 28-arm enumeration:** All 28 explicit VariantMeta arms verified against `error_mapping.rs`; enumeration complete; no phantom arms.
- **117-variant sentinel:** Compile-time `const _: ()` assertion verified present in `error_mapping.rs`; fires on new variant addition without match update.
- **`with_explicit_nulls(true)` single chokepoint:** `server.rs:1953-1955` confirmed as the sole WriterBuilder construction site; 6 regression tests lock the null-not-absent behavior.
- **[H8b] no audit-log-suffix residue:** Zero `audit_log_suffix` / `_audit` column name occurrences in the branch diff; H8b redundancy sweep complete.
- **Retryable whitelist boundary test:** 7 transient + 6 permanent response codes verified in test; 501 (Not Implemented) confirmed in permanent whitelist.
- **.prx staleness gate CI wiring:** CI job wiring for `build-plugin-threatintel-infusion` staleness gate verified present; ancestor-check step confirmed.
- **EC-11-079/080 renumbering propagation:** 30+ sites confirmed updated across `crates/` and `.factory/`; no collision with existing EC-11-NNN assignments.

---

## Summary

**CLEAN(strict): NO** (1 HIGH cross-cascade + 1 MED + 1 LOW + 1 OBS[process-gap])
**CLEAN(PR-merge): NO** (1 HIGH finding — cross-branch adjudicated CLOSED but still counts against PR-merge strict gate per BC-5.39.001)

Streak: **0/3** (stays 0/3; HIGH/MED/LOW/OBS findings prevent strict-clean and PR-merge-clean advancement per BC-5.39.001).

All 4 findings closed: F-MCPRS-PRL11-HIGH-001 CLOSED-CROSS-BRANCH via orchestrator adjudication (fix at 3e0d3585 on PQL lane); F-MCPRS-PRL11-MED-001 CLOSED via story v0.17→v0.18 (BC table cell v1.13→v1.18); F-MCPRS-PRL11-LOW-001 CLOSED @c82f30ba (two-case ci.yml diagnostic); F-MCPRS-PRL11-OBS-001 CLOSED @c82f30ba (repo_root anchoring + fail-loud guard). Branch final HEAD: @c82f30ba (fix-burst-23; push pending). Cascade tally: 31 passes / 23 fix-bursts. Streak 0/3 on @c82f30ba (push from fix-burst-23 resets per DRIFT-ORCH-PRLEVEL-PUSH-001 when executed). PR-LEVEL pass 12 dispatched on frozen @c82f30ba after push.
