---
document_type: story-scope-document
proposed_story_id: S-MAINT-VERIFY-PIPELINE-001
producer: architect
date: 2026-06-10
source: 2026-06-10 full-codebase review package, recommendation ⑭ (human-approved as part of 14-recommendation package)
consolidates_tds: [TD-CICD-001, TD-KANI-001, TD-FUZZ-002, TD-FUZZ-003]
materialized_by: story-writer (full story file authored later; this document is the scoping input)
status: scoped
---

# Story Scope — S-MAINT-VERIFY-PIPELINE-001: Post-Merge Verification Pipeline Redesign

## Problem Statement

Four TDs describe one structural failure: the formal-verification half of the CI
strategy is disconnected from reality, and the gap has *grown* since the TDs were filed.

| TD | Filed state | Current state (2026-06-10, develop@c287b00d) |
|----|------------|---------------------------------------------|
| TD-CICD-001 (P2) | post-merge.yml disabled to `workflow_dispatch` after 7-layer hotfix cascade; 5 architectural defects catalogued | Still disabled. Its 7-checkbox resolution-criteria charter is untouched. |
| TD-KANI-001 (P3) | `cargo kani -p` list scoped to 4 crates (prism-core, prism-spec-engine, prism-security, prism-storage) | **prism-query now carries 4 `#[kani::proof]` files (vp012_depth_limit, vp014_size_limit, vp015_depth_limit, vp025_cache_key) that run in NO CI job anywhere** — post-merge.yml is disabled AND its `-p` list omits prism-query; ci.yml has no Kani job. Only `just kani-local` covers them, manually. |
| TD-FUZZ-002 (P3) | `fuzz_alias_expansion` target never existed; "re-add when alias expansion ships" | Trigger has FIRED in substance: `vp037_alias_fuzz` exists as a `[[bin]]` in fuzz/Cargo.toml but is wired into ZERO workflows (post-merge runs normalize_fuzz/spec_parser/fuzz_injection_scanner; ci.yml + fuzz-nightly.yml run vp021 only). Register text is stale. |
| TD-FUZZ-003 (P3) | `fuzz_template_interpolation` target never existed; "re-add when template interpolation ships" | Trigger has FIRED: the Interpolator (`crates/prism-spec-engine/src/interpolation.rs`, ~566 lines) shipped to production with no fuzz harness at all. |

TD-FUZZ-002/003 are Rule-3-non-compliant as standalone deferrals (their "future
dependency" conditions have already occurred); TD-KANI-001's risk has grown from
"expand a list someday" to "shipped proofs run nowhere." Consolidating all four into
one human-approved anchor story restores Canonical Principle Rule 3 compliance.

## Scope

1. **Verification manifest (single source of truth).** Author a checked-in manifest
   (e.g., `verification-manifest.toml` or extend Justfile data) enumerating: every
   expected fuzz target (diffed against `fuzz/Cargo.toml` `[[bin]]` entries in CI) and
   every Kani-proof-bearing crate (diffed against `rg -l 'kani::proof' crates/`).
   Workflow drift from the manifest fails fast at a cheap validation step. Closes
   TD-CICD-001 defect 1 (speculative inventory) structurally.
2. **Kani CI coverage.** Re-scope the Kani job command from the stale 4-crate list to
   the manifest-driven crate set (today: prism-core, prism-spec-engine, prism-security,
   prism-storage, **prism-query**). Decide placement: post-merge (re-enabled) vs a
   scheduled kani-nightly job — time-budget math decides (Kani full run is too slow for
   per-PR ci.yml; this matches the existing local/CI asymmetry in ADR-014). Closes
   TD-KANI-001.
3. **Fuzz wiring parity.** Wire `vp037_alias_fuzz` into the scheduled fuzz lane
   (fuzz-nightly.yml pattern: nightly long-run; optional 60s ci.yml smoke per the
   vp021 precedent). Closes TD-FUZZ-002 (and corrects its stale register narrative).
4. **Interpolator fuzz harness.** Author `fuzz_targets/` harness for
   `prism-spec-engine::interpolation` (format-token injection, malformed `{...}`
   syntax, recursion/length edge cases), register it in fuzz/Cargo.toml and the
   manifest, wire it into the same lanes as item 3. Test-writer authors the harness;
   VP registration (new VP or extension) routed to architect during story execution.
   Closes TD-FUZZ-003.
5. **Shared CI infrastructure.** Extract a composite action (checkout + toolchain +
   protoc + libdbus + cache) shared by ci.yml / post-merge.yml / fuzz-nightly.yml.
   Closes TD-CICD-001 defect 3.
6. **Time-budget design.** Compute per-step timeout × parallelism ≤ job
   `timeout-minutes` for every redesigned job; record the math in workflow comments.
   Closes TD-CICD-001 defect 5.
7. **Failure notification.** Failures of post-merge/nightly verification jobs must
   create a GitHub issue (or equivalent visible artifact) — no more silent red runs.
   Closes TD-CICD-001 defect 4.
8. **Re-enable post-merge.** After a green `workflow_dispatch` validation run on
   develop, flip post-merge.yml back to `on: push` (develop + main). Retain
   `workflow_dispatch` escape hatch. Closes TD-CICD-001 top-level. (Defect 2,
   toolchain selection, is addressed by documenting the `RUSTUP_TOOLCHAIN: nightly`
   strategy in the composite action.)

## Out of Scope

- New Kani proofs or new VPs beyond the Interpolator fuzz harness registration.
- cargo-mutants / semgrep CI integration (Phase 6 formal-hardening scope).
- Re-introducing the aspirational `fuzz_prismql_parser` target (TD-FUZZ-001): vp021_parse_fuzz
  already covers parser fuzzing in ci-smoke + nightly; TD-FUZZ-001 disposition (close-as-duplicate
  or fold into the manifest) is adjudicated during story execution, not pre-decided here.

## Acceptance Criteria (sketch — story-writer formalizes)

- AC-1: Verification manifest exists; CI step fails when fuzz/Cargo.toml `[[bin]]` set or kani-proof crate set diverges from it.
- AC-2: A CI job (post-merge or kani-nightly) runs `cargo kani` over ALL proof-bearing crates including prism-query; evidence: green run link.
- AC-3: `vp037_alias_fuzz` executes in at least one scheduled workflow; corpus artifact uploaded.
- AC-4: Interpolator fuzz harness exists, registered, and executes in at least one scheduled workflow; zero crashes on a 30-min run.
- AC-5: Composite action used by ≥2 workflows; no duplicated protoc/toolchain setup blocks remain in the redesigned workflows.
- AC-6: Time-budget table present in each redesigned workflow header comment; sum of step ceilings ≤ job timeout.
- AC-7: Forced-failure test demonstrates the notification path (issue created on red run).
- AC-8: post-merge.yml triggers on push to develop/main and the first post-merge run on develop is green.
- AC-9: TD register updated: TD-CICD-001, TD-KANI-001, TD-FUZZ-002, TD-FUZZ-003 closed with citations to this story (state-manager burst).

## Dependency Notes

- No story dependencies — touches `.github/workflows/`, `fuzz/`, and one new harness file; no production crate logic changes (harness is test-only code).
- Low merge-conflict surface with in-flight Wave 3/4/0 stories; serialize with any other workflow-touching PR.
- Specialists: devops-engineer (workflows, composite action, notification), test-writer (Interpolator harness), formal-verifier (Kani scope validation), adversary (TD-CICD-001 resolution-criteria checklist verification — the TD names architect + adversary + devops as the redesign session roster).
- TD-CICD-001's 7 resolution-criteria checkboxes map onto AC-1..AC-8 and must each be checked at story closure.

## Estimate

8 points (workflow redesign + composite action + 1 new fuzz harness + validation runs;
comparable to PLUGIN-MIGRATION-001-F at 8 pts). Single story — splitting would re-create
the cross-workflow drift this story exists to eliminate.
