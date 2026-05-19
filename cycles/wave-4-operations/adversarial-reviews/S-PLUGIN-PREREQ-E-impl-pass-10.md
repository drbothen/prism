---
document_type: adversarial-review
producer: adversary
pass: 10
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: 051eab95
diff_base_to_develop: a5ab742c
factory_artifacts_head: 48b504a0
version: "1.0"
timestamp: 2026-05-18T15:00:00Z
verdict: BLOCKED
streak_before: 1/3
streak_after: 0/3
finding_counts:
  critical: 0
  important: 2
  suggestion: 1
  observation: 2
  process_gap: 1
novel_blind_spot: VP_proof_harness_skeleton_pseudocode_drift_+_BC_error_cases_late_addition_gap
---

# Adversarial Review — S-PLUGIN-PREREQ-E Implementation Cascade — Pass 10

**Verdict: BLOCKED** | Streak: 1/3 → **0/3 RESET** | Pass 10 of impl-cascade

---

## §FB-IMPL-6 Closure Re-Re-Verification

All three FB-IMPL-6 closures re-verified at unchanged HEAD 051eab95 (same head as pass-9):

| Closure | Status | Evidence |
|---------|--------|----------|
| F-P8-IMP-001 VP-153 P0 proptest landing (8 proptests) | VERIFIED | `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` EXISTS; 6 proptests cover Rules A+B; `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs` EXISTS; 2 proptests cover Rule C via ShapedProbe injection; both files load-bearing on production paths; VP-153 v0.17 status:active in frontmatter |
| VP-156 P1 sibling-sweep proactive proptest landing (5 proptests) | VERIFIED | `crates/prism-bin/tests/vp156_write_tool_uniqueness.rs` + `vp156_post_boot_uniqueness.rs` EXIST; 5 proptests total; VP-156 v0.19 status:active in frontmatter; DYNAMIC_WRITE_TOOLS uniqueness keying confirmed per BC-2.16.012 |
| VP-INDEX v1.70 sync (both VP-153 + VP-156 rows updated) | VERIFIED | VP-INDEX lines for VP-153 and VP-156 both show status:active; no stale draft markers |

---

## §Cumulative Closure Re-Verification (Passes 1–9)

All prior pass closures re-verified at HEAD 051eab95 — all hold:

- **F-P1-001/002 DYNAMIC_WRITE_TOOLS read-side + PluginRuntime register_write_tool wiring:** intact.
- **F-P1-003/F-P2-001 validate_cross_composition production path:** wired to `parse_and_validate_spec_toml`; integration tests cover config_manager/MCP/hot_reload paths.
- **F-P2-002 E-PLUGIN-021 error-taxonomy row:** `WriteToolRegistryPoisoned` variant row present in error-taxonomy.md.
- **F-P2-003 integration test race:** resolved via separate-binary Cargo process isolation.
- **F-P4-001 Rule C CredentialRefProbe::probe() Route A:** `Option<String>` shape introspection present at step5; ShapedProbe injection exercised by 2 proptests.
- **F-P4-002 fail-closed Route A deregister_write_tools_for_plugin:** `PluginRuntime::unregister_plugin` + ERROR `plugin_registration_rolled_back` event; BC-2.16.002 row 34 catalogued; BC-2.16.012 EC-016-012-004 present.
- **F-P5-001 Rule C backend-scope conditional (Option B):** ADR-026 §D3 + BC-2.01.016 §E-SPEC-014 scope constraint present; KeyringCredentialProbe doc cites D-706.
- **F-P5-002 unregister_plugin doc-vs-code reconciled:** rustdoc accurately describes single-threaded load→clone→store.
- **F-P5-003 BC-2.16.002 intro count 33→34:** intro count matches body row count.
- **F-P6-001 Option B per-plugin atomic loop:** `'plugin_loop` continue construct unchanged; RED-GATE test probe_good_t3.is_ok() assertion present.
- **F-P6-OBS-001 ADR-026 amended_by back-ref:** ADR-026 v1.26 `amended_by:` field present bidirectionally.
- **Pass-7 Outcome (a) flake-claim:** signal_handlers.rs:102 comment + sentinel-polling + PRISM_TEST_STOP_AFTER_STEP=6 evidence unchanged.
- **F-P8-IMP-001 VP-153 proptest existence:** verified above — files exist, proptests load-bearing.
- **Pass-9 zero-finding CLEAN:** all 13 pass-9 vectors (A–M) verified still CLEAN at unchanged HEAD.

---

## §New Attack Vectors Run (Pass 10)

Pass-10 rotated to fresh vectors targeting spec-layer fidelity, error-variant completeness, and cross-ADR compliance — vectors intentionally NOT run in passes 1–9:

| Vector | Result | Notes |
|--------|--------|-------|
| A. VP §Proof Harness Skeleton pseudocode symbol fidelity | **BLOCKED** | See F-LP-IMPL-P10-IMP-001 |
| B. BC §Error Cases exhaustiveness vs late-added error variants | **BLOCKED** | See F-LP-IMPL-P10-IMP-002 |
| C. Cross-ADR compliance matrix (ADR-022 §B, ADR-026 §D, ADR-027 §D) | CLEAN | ADR-022 §B Step 8 Path A, ADR-026 §D3 backend-scope, ADR-027 §D3 Cargo split all satisfied at HEAD 051eab95. |
| D. BC lifecycle propagation (POL-14 post-merge auto-promote) | CLEAN | All BCs in story frontmatter `behavioral_contracts:` show `lifecycle_status: active`; no stale `draft` residual post-merge. |
| E. Holdout file completeness (HS-PREREQ-E-001 + HS-PREREQ-E-002) | CLEAN | Both holdout scenario files present and non-empty; HS-002 v1.4 updated per FB36; HS-001 v1.11 updated per FB75 cascade. |
| F. Build performance regression check | CLEAN | No new proc-macro crates or build.rs additions in FB-IMPL-1..6 diff; cargo-timings report N/A (no incremental build data); no new dependencies added to workspace. |
| G. CI configuration alignment | CLEAN | `.github/workflows/ci.yml` EXPECTED=30 compile-fail gate unchanged; no new CI jobs added without corresponding test coverage. |
| H. Cargo workspace integrity | CLEAN | All new test files added to `crates/prism-spec-engine/tests/` and `crates/prism-bin/tests/` — correct crates per ADR-027 cross-crate dependency direction; no Cargo.toml dep additions without corresponding lockfile entries. |
| I. Tracing field naming convention (PG-LP11-001) | CLEAN | No new `tracing::*!(event_type=…)` sites without BC-2.16.002 catalog row; `plugin_registration_rolled_back` catalogued in row 34. |
| J. Pub API minimalism (non-exhaustive gate) | CLEAN | No new `pub` types added by FB-IMPL-6 diff without `#[non_exhaustive]`; EXPECTED=30 unchanged. |
| K. Error variant completeness in BC §Error Cases sections | **BLOCKED** | See F-LP-IMPL-P10-IMP-002 (BC-2.16.012 §Error Cases sweep) |
| L. AC 100% coverage audit (all 18 ACs + Red Gate tests 1–25 match story task instructions) | CLEAN | 18 ACs accounted for by tasks + Red Gate tests; no AC without corresponding Task or Red Gate coverage. |
| M. POL-29 exhaustive sweep (classes a–i, all 8 steps) | CLEAN | No new version-pin citations introduced by FB-IMPL-6 diff that would require propagation. Proptest files are test-only (`tests/`) and carry no spec version-pin cite. |
| N. D-706 textual fidelity (ADR-026 §D3 + BC-2.01.016 §E-SPEC-014 | CLEAN | Both artifacts cite D-706 as authority per Option B adjudication; KeyringCredentialProbe doc-comment cites D-706 per FB-IMPL-4. |
| O. VP §Proof Harness Skeleton fidelity vs as-built API (FRESH VECTOR) | **BLOCKED** | See F-LP-IMPL-P10-OBS-002 (VP-156 sibling-pattern, process-gap) |

---

## §Findings

### F-LP-IMPL-P10-IMP-001 — IMPORTANT — VP-153 §Proof Harness Skeleton cites non-existent symbols

**File:** `.factory/specs/verification-properties/vp-153-sensorauth-runtime-cross-composition-prevention.md`

**Lines:** 171 and 243 (§Proof Harness Skeleton section)

**Description:** The VP-153 §Proof Harness Skeleton pseudocode cites two symbols that do not exist in the as-built codebase:

1. **Line ~171:** `SpecEngineError::AuthTypeInvalid` — this variant DOES NOT exist. The actual variant added during FB-IMPL-1 is `SpecEngineError::AuthTypeCrossComposition`. A future implementer or spec-reader referencing VP-153 §Proof Harness Skeleton would write code calling a non-existent variant.

2. **Line ~243:** `validate_auth_coherence` — this function DOES NOT exist. The actual function is `SpecLoader::validate_cross_composition`. The §Proof Harness Skeleton contains a stale function name from an earlier planning draft of the story.

**Evidence of wrongness:** Workspace grep confirms:
- `grep -r "AuthTypeInvalid" crates/` → 0 hits
- `grep -r "validate_auth_coherence" crates/` → 0 hits
- `grep -r "AuthTypeCrossComposition" crates/` → multiple hits (correct variant)
- `grep -r "validate_cross_composition" crates/` → multiple hits (correct function)

**Why passes 1–9 missed this:** Prior passes audited the proptest files themselves (load-bearing, correct symbols) and the production code paths. The §Proof Harness Skeleton is a pseudocode documentation section within the VP spec file, not compiled code. No prior attack vector grep-checked VP spec body pseudocode for symbol accuracy against the as-built API.

**Source-of-Truth Precedence (CLAUDE.md Rule 7):** Code-vs-spec: SPEC wins. The VP spec is stale relative to the as-built code. The VP §Proof Harness Skeleton must be brought to code (use `SpecEngineError::AuthTypeCrossComposition` and `SpecLoader::validate_cross_composition`).

**Route:** product-owner / spec-steward (VP-153 §Proof Harness Skeleton correction). Also: session-reviewer codification candidate — VP §Proof Harness Skeleton sections should be attacked by adversary as a standard vector in every cascade.

**Severity:** IMPORTANT (spec documentation drift that would mislead future implementers; not load-bearing on current passing tests, but load-bearing on VP doc accuracy).

---

### F-LP-IMPL-P10-IMP-002 — IMPORTANT — `SpecEngineError::WriteToolRegistryPoisoned` (E-PLUGIN-021) absent from BC-2.16.012 §Error Cases and ADR-026 §D7

**Files:**
- `.factory/specs/behavioral-contracts/bc-2.16.012-plugin-write-tool-registration.md` §Error Cases (lines 93–99) and §Edge Cases (EC-016-012-001..005)
- `.factory/specs/architecture/adr-026-plugin-architecture.md` §D7 runtime_deliverables section

**Description:** The `SpecEngineError::WriteToolRegistryPoisoned` variant (E-PLUGIN-021) was added to the error-taxonomy.md during FB-IMPL-1 (D-707 burst). The BC-2.16.012 §Error Cases section enumerates valid error conditions for the write-tool registration contract. However, `E-PLUGIN-021 WriteToolRegistryPoisoned` does NOT appear in:
- BC-2.16.012 §Error Cases table (lines 93–99)
- BC-2.16.012 §Edge Cases EC-016-012-001..005 (which covers error paths)
- ADR-026 §D7 runtime_deliverables section

**Root cause — POL-29 step 3a class (b) transitive-closure gap:** The error-taxonomy.md row for E-PLUGIN-021 was added by the product-owner during FB-IMPL-1 after the BC-2.16.012 §Error Cases sweep was already complete for that burst. The transitive closure mandate (POL-29 step 8b) covers version-pin propagation but did NOT cover "error variant added to error-taxonomy must be enumerated in all BCs whose error surface it belongs to." This is a class (b) transitive-closure gap: the new variant exists in the error taxonomy, exists as a Rust variant, exists as a `plugin_registration_rolled_back` BC-2.16.002 row 34 event — but the BC governing write-tool registration does not acknowledge that its RwLock poisoning path is a contractually-specified error.

**Severity:** IMPORTANT (BC-2.16.012 §Error Cases incompleteness; a future implementer validating against BC-2.16.012 would not know that RwLock poisoning is a specified error path; ADR-026 §D7 omits the variant from its runtime_deliverables enumeration).

**Route:** product-owner (BC-2.16.012 §Error Cases + §Edge Cases addition for E-PLUGIN-021) + architect (ADR-026 §D7 enumeration of WriteToolRegistryPoisoned).

---

### F-LP-IMPL-P10-SUG-001 — SUGGESTION — BC-2.16.002 catalog bullet label `(v1.21)` vs v1.32 narrative claim "intro updated v1.21→v1.22"

**File:** `.factory/specs/behavioral-contracts/bc-2.16.002-structured-event-catalog.md`

**Line:** 73 (catalog bullet label)

**Description:** The BC-2.16.002 §Canonical Structured Event Catalog intro bullet at line 73 reads:

> `**Canonical Structured Event Catalog (v1.21)**`

The v1.32 §Changelog row (line ~174) contains the narrative claim: "intro updated v1.21→v1.22". However, the literal bullet label at line 73 is still `(v1.21)`.

**TD-VSDD-059 paper-fix classification:** The §Changelog claims the intro label was advanced, but the actual label was not updated — a classic paper-fix where the changelog is the fictional record and the content is the true state.

**POL-30 Fork B Adjudication required:** This requires Fork B canonical rule clarification:
- **Fork B Path A:** Advance the bullet label from `(v1.21)` to `(v1.22)` to match the v1.32 §Changelog narrative claim (the changelog was right, the label is behind).
- **Fork B Path B:** Backout the v1.32 §Changelog claim "intro updated v1.21→v1.22" (the label was never intended to advance; the changelog made a false claim).

Per POL-30, the catalog bullet-version-label tracks catalog-content-version independently of BC frontmatter version. The question is which state is canonical: the label or the changelog claim.

**Severity:** SUGGESTION (non-blocking; does not affect runtime behavior; is a spec accuracy issue under TD-VSDD-059 paper-fix detection discipline).

**Route:** product-owner (POL-30 Fork B adjudication — choose Path A or B and apply atomically).

---

### F-LP-IMPL-P10-OBS-001 — OBSERVATION — Pass-9 evidence-fidelity: VP-156 file path citations were imprecise

**Description:** Pass-9 §FB-IMPL-6 Closure Re-Re-Verification cited VP-156 proptests as `crates/prism-bin/tests/vp156_write_tool_uniqueness.rs` — which is correct. However, in the pass-9 closure report §New Attack Vectors (Vector B), the observation note references `crates/prism-query/tests/` as the location of some VP artifacts — this is an error in the pass-9 report's narrative. The actual VP-156 proptests are in `crates/prism-bin/tests/`, not `crates/prism-query/tests/`. The filenames referenced in the VP-156 §Proof Harness Skeleton are `vp156_write_tool_uniqueness.rs` and `vp156_post_boot_uniqueness.rs`, matching the actual artifacts.

**Impact:** Non-blocking. The underlying claim in pass-9 (VP-156 proptests exist and are load-bearing) holds. The cite-accuracy was imprecise in one pass-9 narrative paragraph. Does not affect convergence streak.

**Severity:** OBSERVATION (cite-accuracy; non-blocking per cascade semantics).

---

### F-LP-IMPL-P10-OBS-002 — OBSERVATION — `[process-gap]` VP-156 §Proof Harness Skeleton also cites stale symbols

**Description:** Sibling-pattern to F-LP-IMPL-P10-IMP-001. VP-156's §Proof Harness Skeleton section also references symbols that do not match the as-built API:
- `reset_for_test()` — does not exist in the current test harness
- `invalidation_map()` — does not exist; the actual data structure is `DYNAMIC_WRITE_TOOLS` accessed via `RwLock`

The VP-156 §Proof Harness Skeleton appears to be from a planning-draft era, like VP-153's §Proof Harness Skeleton.

**Process-gap classification:** This establishes a PATTERN: VP §Proof Harness Skeleton sections are written during spec authoring before implementation, then never updated to match as-built API names. The sections contain pseudocode that becomes stale as the implementation diverges in naming. No enforcement gate exists to detect §Proof Harness Skeleton pseudocode drift vs as-built symbols.

**Codification target:** Either (a) the adversary skill should include a standing VP §Proof Harness Skeleton symbol-fidelity attack vector, OR (b) a pre-commit hook should grep VP §Proof Harness Skeleton sections for function/type names and verify them against the workspace. Route: product-owner (immediate VP-156 §Proof Harness Skeleton correction) + session-reviewer (codification target for cycle-close).

**Severity:** OBSERVATION — process-gap. The VP §Proof Harness Skeleton is documentation, not compiled. The VP-156 proptests themselves use correct symbols. Non-blocking on current tests; IS blocking on VP spec documentation accuracy if a future implementer reads the §Proof Harness Skeleton as implementation guidance.

---

## §Sweep Output

```bash
# Symbol fidelity sweep — F-P10-IMP-001 evidence
grep -r "AuthTypeInvalid" crates/                  # 0 hits
grep -r "validate_auth_coherence" crates/           # 0 hits
grep -r "AuthTypeCrossComposition" crates/          # hits in spec-engine + prism-bin (correct)
grep -r "validate_cross_composition" crates/        # hits in spec-engine + prism-bin (correct)

# Error variant sweep — F-P10-IMP-002 evidence
grep -n "WriteToolRegistryPoisoned\|E-PLUGIN-021" \
  .factory/specs/behavioral-contracts/bc-2.16.012-plugin-write-tool-registration.md
# → 0 hits in §Error Cases or §Edge Cases

grep -n "WriteToolRegistryPoisoned\|E-PLUGIN-021" \
  .factory/specs/architecture/adr-026-plugin-architecture.md
# → 0 hits in §D7

# Confirm variant exists in code + taxonomy
grep -r "WriteToolRegistryPoisoned" crates/           # hits (correct, variant exists)
grep -n "WriteToolRegistryPoisoned\|E-PLUGIN-021" \
  .factory/specs/prd-supplements/error-taxonomy.md    # hit (row present from FB-IMPL-1)

# Catalog bullet label — F-P10-SUG-001
grep -n "Canonical Structured Event Catalog" \
  .factory/specs/behavioral-contracts/bc-2.16.002-structured-event-catalog.md
# line 73: **Canonical Structured Event Catalog (v1.21)**
grep -n "v1.21.*v1.22\|intro updated" \
  .factory/specs/behavioral-contracts/bc-2.16.002-structured-event-catalog.md
# v1.32 changelog row: "intro updated v1.21→v1.22" (contradiction)

# VP-156 §Proof Harness Skeleton stale symbols — F-P10-OBS-002
grep -n "reset_for_test\|invalidation_map" \
  .factory/specs/verification-properties/vp-156-write-tool-uniqueness-enforcement.md
# hits in §Proof Harness Skeleton section (stale; not present in codebase)
grep -r "reset_for_test\|invalidation_map" crates/  # 0 hits (stale in VP, absent in code)
```

---

## §Verdict

**BLOCKED.** Pass-10 found 2 IMPORTANT + 1 SUGGESTION + 2 OBSERVATION (1 process-gap) findings against unchanged HEAD 051eab95 (same head as pass-9 CLEAN).

**Why pass-9 was CLEAN and pass-10 is BLOCKED:** Pass-9 rotated to 13 vectors (A–M) emphasizing property-level verification completeness, cross-test isolation, WASM lifecycle, naming conventions, and commit coherence. Pass-10 rotated to 15 vectors (A–O) including two FRESH vectors: VP §Proof Harness Skeleton pseudocode symbol fidelity (Vector A, Vector O) and BC §Error Cases exhaustiveness for late-added variants (Vector B/K). These vectors were NOT run in passes 1–9. The spec-hygiene drift found at the VP §Proof Harness Skeleton layer (Vectors A and O) was invisible to all body-rendering scans (compilation, proptest, grep of test files) for 9 prior passes because the §Proof Harness Skeleton is a documentation section in a `.md` file, not compiled Rust code.

**Novel blind spot identified:** VP §Proof Harness Skeleton sections are written during spec authoring (pre-implementation) and contain pseudocode with planned API names. When implementation diverges in naming (a natural part of TDD refinement), the §Proof Harness Skeleton pseudocode becomes stale. No prior adversary pass included a vector that grep-checked VP §Proof Harness Skeleton pseudocode for symbol accuracy against the as-built codebase. This is a new defect class surfaced at pass-10.

**Streak assessment:** Streak RESETS 1/3 → 0/3. Per BC-5.39.001, any finding resets the streak. The 2 IMPORTANT findings (VP-153 stale symbols + BC-2.16.012 §Error Cases gap) are both spec-layer fixes, not code fixes — the implementation is correct; the specifications need updating to match the implementation.

---

## §Convergence Streak Update

| Pass | Verdict | Streak | Notes |
|------|---------|--------|-------|
| pass-1 | BLOCKED | 0/3 | 3C+4I+1S+2Obs+1pg |
| pass-2 | BLOCKED | 0/3 | 2C+3I+1S+1Obs+1pg |
| pass-3 | CLEAN | 1/3 | First advance |
| pass-4 | BLOCKED | 0/3 | 1C+1I RESET |
| pass-5 | BLOCKED | 0/3 | 1C+1I |
| pass-6 | BLOCKED | 0/3 | 0C+1I |
| pass-7 | CLEAN | 1/3 | |
| pass-8 | BLOCKED | 0/3 | 0C+1I RESET |
| pass-9 | CLEAN | 1/3 | Perfect zero-finding |
| **pass-10** | **BLOCKED** | **0/3** | **0C+2I+1S+2Obs+1pg RESET — novel VP-skeleton-pseudocode-drift blind-spot** |

Next: FB-IMPL-7 product-owner spec hygiene fixes (VP-153 §Proof Harness Skeleton symbol corrections `AuthTypeInvalid`→`AuthTypeCrossComposition` + `validate_auth_coherence`→`validate_cross_composition`; BC-2.16.012 §Error Cases E-PLUGIN-021 addition; ADR-026 §D7 WriteToolRegistryPoisoned mention; BC-2.16.002 catalog bullet POL-30 Fork B adjudication for F-P10-SUG-001; VP-156 §Proof Harness Skeleton sibling-pattern correction for reset_for_test + invalidation_map stale symbols). Then state-manager closure burst. Then adversary pass-11 fresh-context.
