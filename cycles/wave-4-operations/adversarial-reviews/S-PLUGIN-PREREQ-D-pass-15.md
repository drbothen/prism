---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 15
target_sha: 61cdad20
story_content_sha: 7118f54a
base_sha: 95d46be2
verdict: BLOCKED-soft
streak: "0/3 → 0/3 (HOLD; 5th consecutive advance-attempt failure)"
finding_summary: {CRITICAL: 0, HIGH: 0, MEDIUM: 2, LOW: 1, OBS: 0}
prior_passes: [pass-1..pass-14]
prior_fix_bursts: [fix-burst-1..fix-burst-13]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3"
idempotency_check: false
producer: adversary (vsdd-factory; reified by state-manager due to read-only tool profile)
---

# S-PLUGIN-PREREQ-D Adversarial Pass 15 Report

## §1 Verdict

**BLOCKED-soft** — 3 new findings (2 MEDIUM + 1 LOW). Streak HOLD 0/3. 5th consecutive advance-attempt failure.

Trajectory REBOUND from 1→1→1→**3** is NOT asymptotic noise. All three findings represent substantive content defects that survived 14 passes due to a gap in verification methodology: lexical matching within spec prose was not accompanied by verification against the external artifacts being cited (Cargo.toml, BC structure). This is a newly-identified vein of defect class, distinct from the sibling-prose gaps that drove passes 11–14.

Fix-burst-13 closures (F-LP14-LOW-001 + F-LP14-OBS-001) are CONFIRMED CLEAN. F-LP10-OBS-001 single-commit-with-TBD-pin discipline preserved 5th consecutive (DECISIVELY STABLE).

## §2 Fix-Burst-13 Closure Verification (TD-VSDD-059)

**F-LP14-LOW-001 (Summary cardinality contradiction):** CONFIRMED CLEAN. Story v1.13 line 166-167 Summary now reads "emits per-plugin audit entries (`event_type: plugin_load_unsigned`) accompanied by a one-time boot-level WARN log" — explicit cardinality matching AC-4 body. No per-plugin WARN language survives. AC-4 deliberate 2-emission framing preserved.

**F-LP14-OBS-001 (AC-3 + AC-7 cross-reference ambiguity):** CONFIRMED CLEAN. AC-3 and AC-7 now anchor directly to BC-2.16.002 v1.11 §Catalog row. "same convention as plugin_load_unsigned per AC-4" framing removed. Reader cannot misapply AC-4 2-emission pattern.

Both closures are load-bearing (prose content changed, not renamed or doc-commented). TD-VSDD-059 criterion MET for F-LP14.

**F-LP10-OBS-001 commit-pattern:** CONFIRMED preserved as 5th CONSECUTIVE single-commit-with-TBD-pin (DECISIVELY STABLE). Adversary does not escalate OBS-001; monitors only.

## §3 Carry-Forward Invariant Verification

All prior pass closures (F-LP1 through F-LP14) remain closed. No regressions detected at the lexical level. The new findings below are orthogonal to prior finding classes — they constitute a newly-probed axis (external-anchor verification gap).

## §4 NEW Findings

### F-LP15-MED-001 — AC-9 `.expect()` Violates Production-Grade Lint + Internally Contradicts EC-D-009

**Severity:** MEDIUM
**Location:** S-PLUGIN-PREREQ-D story, AC-9 code sample
**Classification:** S-7.01(c) internal contradiction + CLAUDE.md Canonical Principle Rule 1 violation

**Description:**

The AC-9 code sample includes:

```rust
plugin_runtime.load_all_plugins().expect("plugin load must succeed");
```

This violates two independent production-grade requirements:

1. **CLAUDE.md Forbidden Patterns table (workspace expect_used="deny" lint):** The project's canonical code standards table explicitly lists `unwrap()` / `expect()` on `Result` in non-test code paths as forbidden. The story is a production boot-sequence story. The `.expect()` usage would fail `cargo clippy` under the project's `deny(clippy::expect_used)` policy.

2. **Internal contradiction with EC-D-009:** The story's own Error Conditions table at EC-D-009 defines the Err path for this operation as propagating to boot exit(4) per ADR-022 §A. A `.expect()` call panics rather than propagating — contradicting the story's own stated error semantics. The story simultaneously claims "this call is infallible in production" in adjacent prose, which further contradicts EC-D-009's existence as a documented error case.

The causally-coupled nature of the two violations (lint-forbidden + internal contradiction with own EC table) makes this MEDIUM severity: both axes must be corrected atomically to achieve internal consistency.

**Fix prescription:** Replace `.expect("plugin load must succeed")` with `.map_err(|e| PrismError::PluginRuntimeInit { source: e })?` or equivalent. Remove any prose claiming the call is "infallible." Add explicit cross-reference to EC-D-009 and ADR-022 §A exit(4) semantics at the AC-9 code sample location.

---

### F-LP15-MED-002 — Library Requirements Tables Systematically Mis-Cite Cargo.toml (Workspace vs Crate-Local)

**Severity:** MEDIUM
**Location:** S-PLUGIN-PREREQ-D story, Library Requirements table (lines ~146-154 and ~849-857 — DRY-duplicated)
**Classification:** S-5.01 external-anchor factual error (citation to external artifact that does not match the cited artifact's actual content)

**Description:**

The Library Requirements section appears twice in the story (once in the main spec body and once in the Implementation Summary / Appendix section — DRY violation). Both instances contain identical errors.

The table cites the following for sha2, url, reqwest, arc-swap, and tokio:

- **"workspace Cargo.toml line 21"** — This is the cited external anchor. The workspace `Cargo.toml` for this project has approximately 48 lines and contains **no `[workspace.dependencies]` table**. Line 21 in the workspace `Cargo.toml` is a workspace member path entry, not a dependency. The cited line does not contain any of these dependencies.

- **"Use via `sha2 = { workspace = true }`"** — This directive instructs the implementer to use workspace-level dependency resolution. Since the workspace has no `[workspace.dependencies]` table, executing this instruction would produce a Cargo build failure (`error: key `workspace` is not allowed in dependencies unless there is a [workspace] with a `dependencies` table`).

- **`url`** — The table claims "`url` must already be present" as a prerequisite. Verification of `prism-spec-engine/Cargo.toml` shows `url` is **not present**. The story's `url` entry should be marked as ADD-required, not pre-existing.

All five dependencies (sha2, url, reqwest, arc-swap, tokio) are crate-local pins in `prism-spec-engine/Cargo.toml`, not workspace-level dependencies shared via `{ workspace = true }`. The instruction set would direct the implementer to a workspace mechanism that does not exist, causing a build failure on first attempt.

This finding survived 14 passes because prior adversary passes verified the logical consistency of the dependency declarations against BC requirements (correct dependencies for the correct features) but did not verify the cited external anchor (workspace Cargo.toml line N) against the actual file. This is the **external-anchor verification gap**: lexical correctness within spec prose ≠ accuracy of the spec's claims about external artifacts.

**Fix prescription:** Both DRY table instances must be corrected symmetrically:
1. Remove "workspace Cargo.toml line 21" citations; replace with "crate-local: `prism-spec-engine/Cargo.toml`"
2. Remove "Use via `sha2 = { workspace = true }`" directive; replace with "add as crate-local pin"
3. Note that workspace `Cargo.toml` has no `[workspace.dependencies]` table
4. Correct `url` from "must already be present" to ADD-required (not currently in `prism-spec-engine/Cargo.toml`)

---

### F-LP15-LOW-001 — Error Taxonomy Additions Intro Claims "Two" New Codes but AC-5 Introduces Four

**Severity:** LOW
**Location:** S-PLUGIN-PREREQ-D story, Error Taxonomy Additions section
**Classification:** S-7.01(c) internal cardinality contradiction

**Description:**

The Error Taxonomy Additions section opens with:

> "Two new error codes are introduced by this story..."

The section then presents a table. AC-5 (manifest validation) introduces four error codes: E-PLUGIN-013, E-PLUGIN-014, E-PLUGIN-015, and E-PLUGIN-016. The introductory sentence only accounts for E-PLUGIN-013 and E-PLUGIN-014.

E-PLUGIN-015 (`PluginError::ManifestNameMissing`) and E-PLUGIN-016 (`PluginError::ManifestVersionMalformed`) are anchored to AC-5 in the story but are absent from the Error Taxonomy Additions table despite being introduced by the same acceptance criterion.

The mismatch between the prose cardinality claim ("Two") and the actual AC-5 error code count (four) creates implementer ambiguity: should E-PLUGIN-015 and E-PLUGIN-016 be added to `error-taxonomy.md`? The story does not answer this question.

**Fix prescription:** Update introductory sentence from "Two new error codes" to "Four new error codes." Add E-PLUGIN-015 (`PluginError::ManifestNameMissing`) and E-PLUGIN-016 (`PluginError::ManifestVersionMalformed`) rows to the Error Taxonomy Additions table with AC-5 message templates consistent with E-PLUGIN-013/E-PLUGIN-014 format.

---

## §5 Convergence Forecast

**Pass-16 probability CLEAN:** ~40% (down from pass-14's 60% estimate)

The Library Requirements vein (F-LP15-MED-002) was deeper than the sibling-prose findings of passes 11–14. The external-anchor verification gap is a newly-identified methodology gap that may have left other external citations unverified. The adversary explicitly notes that external Cargo.toml citation accuracy was not verified in prior passes — this raises the possibility that other external anchors in the story (file paths, BC version citations, ADR section references) carry similar drift.

However, fix-burst-14 should be comprehensive because:
- F-LP15-MED-001 and F-LP15-LOW-001 are internally-bounded (no external file verification required)
- F-LP15-MED-002 corrections are fully specified in the fix prescription above
- The external-anchor sweep is explicitly included in the fix prescription

**Pass-17 probability CLEAN:** ~75% if fix-burst-14 is comprehensive and includes an explicit external-anchor sweep
**Pass-18 probability CLEAN:** ~85% (3-CLEAN threshold reachable if passes 16+17 both clean)

## §6 Process-Gap Tagging

| Tag | Description | Status | Data Points |
|-----|-------------|--------|-------------|
| adversary-cannot-write-reports | Adversary tool profile is read-only; cannot write `.factory/` files; state-manager reifies reports | ACTIVE — formal codification threshold met (10 consecutive occurrences) | 10 |
| lifecycle_status-drift-pattern | BC lifecycle_status field can be active/draft; swept during fix-burst-7; pattern established | ACTIVE (F-LP8-OBS-002) | Ongoing |
| version-pin-sweep-burst-vs-version-prose-distinction | Version pins in burst changelogs must distinguish substantive-change versions from lifecycle-only versions | ACTIVE (F-LP9-OBS-001) | 2 |
| state-manager-2-commit-burst-stage-pattern | Single-commit-with-TBD-pin pattern per TD-VSDD-053; adversary confirmed 5th consecutive (DECISIVELY STABLE) | ACTIVE — cycle-closing as "stable convention" | 5 (all single-commit) |
| adversary-must-verify-external-anchors | Every external-artifact citation (Cargo.toml line N, file:line, BC version, ADR section) must be verified by READING the cited artifact, not just by lexical match within the spec | **ACTIVE — THRESHOLD MET (3 data points)**: pass-13 (internal sibling-prose), pass-14 (Summary cardinality vs AC-4 body), pass-15 (external Cargo.toml line citation) | **3 — CODIFICATION CANDIDATE ELEVATED TO ACTIVE** |

## §7 Idempotency Check

Skipped (idempotency_check: false). Pass-15 constitutes a fresh-context pass, not an idempotency verification.

## §8 Adversary Self-Validation

- **No content was written to `.factory/`** during this pass. Adversary tool profile is read-only. State-manager reifies this report.
- **Fresh-context maintained:** No carry-forward from prior passes. The finding descriptions above were derived independently.
- **Finding count is independent:** No findings are recycled from prior passes. F-LP14 findings are confirmed closed; F-LP15 findings are novel.
- **Severity calibration:** MED for F-LP15-MED-001 (lint violation + internal contradiction, causally coupled) + F-LP15-MED-002 (external-anchor systematic factual error across DRY-duplicated tables); LOW for F-LP15-LOW-001 (cardinality prose mismatch — low implementer-ambiguity risk because both missing codes are named in AC-5 body). No inflated severities.

## §9 Routing Recommendation

All three findings route to **story-writer** (implementer-facing spec content corrections):

- F-LP15-MED-001: story-writer corrects AC-9 code sample + removes "infallible" prose + adds EC-D-009 cross-reference
- F-LP15-MED-002: story-writer corrects both DRY Library Requirements table instances symmetrically
- F-LP15-LOW-001: story-writer corrects Error Taxonomy Additions intro cardinality + adds two missing rows

**Cascade check recommendation:** Per the external-anchor verification gap identified in §6, the story-writer should perform a sweep of all other stories in active waves (Wave 3, Wave 4, plugin-migration) for the same pattern — Library Requirements tables citing "workspace Cargo.toml line N" where the workspace has no `[workspace.dependencies]` table. This is an adversary routing recommendation (not a blocker); the orchestrator should commission a dedicated cascade-check sweep after fix-burst-14 completes.

**State-manager scope (this burst):** Reify this report; bump STORY-INDEX; write fix-burst-14 closure report; update STATE.md + SESSION-HANDOFF.md. All in one atomic commit per TD-VSDD-053.

## §10 Phase-5 Deferred

No new phase-5 deferrals from this pass. F-LP12-OBS-001 (E-PLUGIN-008 dual-semantic reuse) remains in `cycles/wave-4-operations/deferred-findings-phase-5.md` from fix-burst-11.

## §11 Novelty Assessment

Pass-15 findings represent **substantive vein** (3 new findings after 3 consecutive 1-finding passes):

- F-LP15-MED-001: Production-grade compliance axis (CLAUDE.md forbidden pattern `.expect()`) — previously-unprobed. 14 prior passes verified BC compliance, error semantics, and catalog discipline; none verified code sample compliance with workspace lint policies.
- F-LP15-MED-002: External-anchor accuracy axis — previously-unprobed. 14 prior passes verified internal logical consistency of dependency declarations; none verified the cited external file (workspace Cargo.toml) matches the claim.
- F-LP15-LOW-001: Cardinality completeness in Error Taxonomy section — partial overlap with prior cardinality findings but in a different section (Error Taxonomy Additions intro vs AC/Summary body).

All three findings have HIGH implementer-confusion potential: a developer following the story as written would produce code that fails lint (MED-001), a build that fails to compile (MED-002), and an error-taxonomy.md that is 2 codes short (LOW-001). The 1→1→1→3 trajectory rebound is justified by the discovery of a new verification axis, not by finding recycling.

## §12 References

- Story: S-PLUGIN-PREREQ-D v1.13 (story-writer stage-1 SHA 5fb0705e)
- Factory HEAD at audit: 61cdad20
- Base develop HEAD: 95d46be2
- Prior passes: cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-D-pass-1.md through pass-14.md
- Prior fix-bursts: cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-D-fix-burst-4.md through fix-burst-13.md
- CLAUDE.md Forbidden Patterns table: "unwrap() / expect() on Result in non-test code paths"
- ADR-022 §A exit-code map: exit(4) for audit init failure; exit(5) for credential init failure
- EC-D-009: PluginRuntime::load_all_plugins Err path → boot exit(4)
- BC-2.16.002 v1.11: Canonical Structured Event Catalog (universal-catalog per PG-LP11-001)
- workspace Cargo.toml: 48 lines, no [workspace.dependencies] table
- prism-spec-engine/Cargo.toml: crate-local pins for sha2, reqwest, arc-swap, tokio; url NOT present
