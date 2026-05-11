---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T08:00:00
phase: 3
inputs: []
input-hash: "02d5618"
traces_to: S-PLUGIN-PREREQ-A-sensorid-newtype.md
pass: 1
previous_review: null
review_level: PR
target_artifact: PR #142 (S-PLUGIN-PREREQ-A)
target_sha: 8dd9a89e
base_sha: c6dd6602
verdict: BLOCKED-hard
streak: 0/3
finding_summary: { critical: 1, high: 2, medium: 3, low: 1, obs: 2 }
prior_passes: 12 LOCAL passes converged 3/3 at HEAD 8b949bba; demo-evidence commit 8dd9a89e added 12 files (verified post-pass-1)
closure_status: closed-by fix-burst-PR1 at worktree ba7d7f6f + factory-artifacts baae27fd (2026-05-11)
crit_001_retraction: F-PR1-CRIT-001 RECLASSIFIED as adversary-tool-false-positive — Glob tool returned 0 files but ls -la confirms 12 demo evidence files exist at /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/docs/demo-evidence/S-PLUGIN-PREREQ-A/ committed in 8dd9a89e (verified via git show --stat 8dd9a89e). The directory and 12 files (AC-1 through AC-11 + INDEX.md) are present on disk and in the commit. Adversary tool error, not a defect.
---

# Adversarial Review: S-PLUGIN-PREREQ-A PR #142 (Pass 1 — PR-Level)

## Finding ID Convention

Finding IDs for this PR-Level pass use the format: `F-PR1-<SEV>-<SEQ>` where SEV is CRIT/HIGH/MED/LOW/OBS and SEQ is a three-digit sequence. Example: `F-PR1-HIGH-001`.

## Part A — Fix Verification (pass >= 2 only)

Not applicable — this is Pass 1.

## Part B — New Findings (or all findings for pass 1)

### Verdict: BLOCKED-hard

Streak: **0/3** (this pass concludes PR-LEVEL streak 0/3).

Per the project policy rubric (POL-10 demo-evidence required, POL-4/6 semantic anchoring), 1 CRITICAL, 2 HIGH, and 3 MEDIUM findings block convergence. Code-level migration is largely solid (all 6 Red Gate tests present at canonical names; all 4 adapter impls return `SensorId::from(...)`; ADR-023 §C1 implemented; perimeter compile-fail gate emits positive coverage assertion per POL-11; zero `SensorType::` production references; cross-crate validator parity proptest exists). The blockers are evidentiary (demo evidence absent — LATER RECLASSIFIED as tool false-positive), structural (subsystem mis-anchor), and a previously-undetected sibling-site sweep gap (shadow `SensorId = String` alias in `prism-query::cache_key` not migrated).

---

### CRITICAL

#### F-PR1-CRIT-001: Demo Evidence Directory Empty (RECLASSIFIED post-pass as adversary-tool false-positive)

- **Severity:** CRITICAL (at time of finding) → RECLASSIFIED
- **Category:** coverage-gap
- **Location:** `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/docs/demo-evidence/S-PLUGIN-PREREQ-A/`
- **Description:** POL-10 requires per-AC demo evidence for all 11 ACs. Adversary Glob tool query for the demo-evidence directory returned 0 files at pass time.
- **Evidence:** Glob tool returned empty result set. Post-pass verification via `git show --stat 8dd9a89e` shows 963 insertions across 12 files in `docs/demo-evidence/S-PLUGIN-PREREQ-A/`: AC-1 through AC-11 individual demo files + INDEX.md. `ls -la` confirms all 12 files on disk. The adversary's Glob tool produced a false-negative on an existing populated directory.
- **Proposed Fix:** N/A — reclassified as adversary tooling defect. Codified as PG-PR1-001 (adversary Glob negatives must be cross-verified before escalating to finding).
- **Post-pass status:** RECLASSIFIED as adversary-tool false-positive per closure annotations.

---

### HIGH

#### F-PR1-HIGH-001: Story Subsystems Array Mis-Anchored

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md` frontmatter `subsystems:`
- **Description:** Story frontmatter `subsystems:` array lists `[SS-01, SS-02, SS-08, SS-16]`. Cross-check against ARCH-INDEX Subsystem Registry v2.40 and the PR diff touched-crates set (prism-core, prism-sensors, prism-query, prism-spec-engine) reveals: SS-02 and SS-08 do not correspond to the actually-touched crates; missing SS-11 (prism-sensors adapter layer) and SS-21 (spec-engine plugin dispatch). The LOCAL cascade adversary anchored to story body + diff but NOT to ARCH-INDEX Subsystem Registry — this class of drift survived 12 LOCAL passes.
- **Evidence:** ARCH-INDEX Subsystem Registry at `/Users/jmagady/Dev/prism/.factory/specs/architecture/ARCH-INDEX.md` v2.40. Story file frontmatter `subsystems: [SS-01, SS-02, SS-08, SS-16]`.
- **Proposed Fix:** Story v1.5 `subsystems:` → `[SS-01, SS-11, SS-16, SS-21]` with per-entry justification recorded in the story changelog.

#### F-PR1-HIGH-002: Shadow `pub type SensorId = String;` in prism-query::cache_key (TRULY NOVEL)

- **Severity:** HIGH
- **Category:** spec-fidelity / purity-boundary-violations
- **Location:** `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/cache_key.rs`
- **Description:** `cache_key.rs` contains `pub type SensorId = String;` — a LOCAL shadow of the canonical `prism_core::SensorId` newtype. This survived all 12 LOCAL adversary passes because the story's dispatch-site enumeration did not include type-alias definition sites (only caller sites). Sibling sites using the shadow type: `cache.rs` (13 sites), `cache_tests.rs` (17 sites), `invalidation.rs` (3 sites), `proofs/vp025_cache_key.rs` Kani proof, and `CacheKey::derive()` API. Hash stability is preserved post-migration via `Borrow<str>` impl on `prism_core::SensorId`.
- **Evidence:** `grep -r 'pub type SensorId' /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/` returns cache_key.rs as the only hit. `git show --stat 8dd9a89e` does not include cache_key.rs in the changed files.
- **Proposed Fix:** Delete `pub type SensorId = String;` from cache_key.rs; replace with `pub use prism_core::SensorId;`. Migrate all 30+ sibling sites in cache.rs, cache_tests.rs, invalidation.rs, vp025_cache_key.rs, and `CacheKey::derive()` API callers.

---

### MEDIUM

#### F-PR1-MED-001: fanout.rs Sentinel String "unknown" Semantically Overloaded

- **Severity:** MEDIUM
- **Category:** code-quality
- **Location:** `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/fanout.rs:409`
- **Description:** The string literal `"unknown"` is used as a sentinel for panic-recovery paths. The string `"unknown"` is also a valid SensorId in some testing contexts, creating ambiguity in log analysis and telemetry.
- **Evidence:** `fanout.rs:409` literal `"unknown"` used in panic-recovery instrumentation path.
- **Proposed Fix:** Rename sentinel to `"internal-panic-recovery"` (semantically reserved string that cannot be confused with a valid sensor name).

#### F-PR1-MED-002: Story §File Structure Requirements Missing cache_key.rs

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md` §File Structure Requirements
- **Description:** Story §File Structure Requirements enumerates dispatch-site groups and expected file mutations but does not list `cache_key.rs` or its sibling cluster. This omission is what allowed F-PR1-HIGH-002 to survive 12 LOCAL passes — the adversary's dispatch-site enumeration derived from the story body was incomplete.
- **Evidence:** Story §File Structure Requirements does not mention cache_key.rs, cache.rs, cache_tests.rs, invalidation.rs, or proofs/vp025_cache_key.rs.
- **Proposed Fix:** Story v1.5 §File Structure Requirements must add cache_key.rs cluster with annotation "workspace-wide type-alias grep required, not just dispatch-site enumeration." Add new AC-12: "`grep -r 'pub type SensorId' workspace` → 0 matches."

#### F-PR1-MED-003: Process-Gap LP-PR1-001 — Identity-Newtype Story §File Structure Derivation

- **Severity:** MEDIUM (process-gap; does not block merge after codification)
- **Category:** missing-edge-cases
- **Location:** Story-writer template for identity-newtype migration stories
- **Description:** The story-writer template derives §File Structure Requirements from dispatch-site enumeration (grep for callers of the old type). This misses shadow type-alias definitions (`pub type X = Y;`) which are NOT callers. The correct derivation must include workspace-wide grep for ALL definition patterns: `pub type SensorId`, `pub struct SensorId`, `type SensorId =`, `pub use ..::SensorId`.
- **Evidence:** F-PR1-HIGH-002 false-negative during 12 LOCAL passes is direct evidence of this gap.
- **Proposed Fix:** Codify as LP-PR1-001 in story v1.5 lessons section. File as TD-VSDD-082 candidate. Non-blocking for merge after codification.

---

### LOW

#### F-PR1-LOW-001: sensor_id.rs should_panic Messages Include Story-Specific Prefix

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-core/src/sensor_id.rs:511` and `:518`
- **Description:** `should_panic(expected = "S-PLUGIN-PREREQ-A: invalid SensorId string")` — the `S-PLUGIN-PREREQ-A:` prefix is story-specific bookkeeping that should not appear in production panic messages.
- **Evidence:** Lines 511 and 518 of sensor_id.rs in the worktree.
- **Proposed Fix:** Shorten to `should_panic(expected = "invalid SensorId string")` at both sites.

---

### OBSERVATIONS

#### OBS-PR1-001: Adversary Glob Negative-Result Verification Protocol Gap (PG-PR1-001)

This pass's F-PR1-CRIT-001 finding was based on a Glob tool returning 0 files for an existing directory. The adversary did not cross-verify with `ls -la` or `git ls-files` before reporting as a blocking finding. This is a process gap — the adversary must cross-verify all directory-existence claims before elevating to CRITICAL.

#### OBS-PR1-002: Subsystem-Anchor-vs-ARCH-INDEX Gap Survived 12 LOCAL Passes (PG-PR1-003)

The LOCAL cascade used story body + PR diff as the adversary's primary context, but did not cross-check `subsystems:` against the ARCH-INDEX Subsystem Registry. This is a methodological gap that allows semantic mis-anchoring to survive many LOCAL passes.

---

## KUDOS

1. **F-PR1-HIGH-002 TRUE NOVELTY:** The cache_key shadow type alias survived 12 LOCAL adversary passes across 7 fix-bursts. The PR-LEVEL adversary's broader diff context (full workspace vs story-scoped view) was the key enabler. This validates the PR-LEVEL pass as a distinct quality gate.
2. **Red Gate 6/6 verified at exact BC-prefixed names** — the 12-pass LOCAL cascade delivered precise naming discipline that the PR-LEVEL adversary independently confirmed.
3. **ADR-023 §C1 implemented cleanly** — the perimeter compile-fail gate is correctly wired with positive-coverage assertion per POL-11.
4. **Adapter dispatch sites: 4/4 adapters return `SensorId::from(...)`** — the core migration is complete and correct.
5. **1,398 tests passing** at HEAD 8dd9a89e confirms the migration introduced zero regressions.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 (reclassified post-pass as tool false-positive) |
| HIGH | 2 |
| MEDIUM | 3 |
| LOW | 1 |
| Observations | 2 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — fix-burst-PR1 required before pass-2
**Readiness:** requires revision — fix-burst-PR1 closes all 6 actionable findings; pass-2 at HEAD ba7d7f6f

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | PR-Level Pass 1 |
| **New findings** | 6 (F-PR1-CRIT-001 reclassified; F-PR1-HIGH-001/002; F-PR1-MED-001/002/003; F-PR1-LOW-001) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (6 new / 6 total; first PR-level pass, all findings are new) |
| **Median severity** | HIGH-MEDIUM (2 HIGH + 3 MED + 1 LOW + 1 CRIT-reclassified) |
| **Trajectory** | LOCAL 14→12→6→4→2→6→4→0→4→0→0→0 CONVERGED → PR-LEVEL pass-1: 6 findings (1 reclassified) |
| **Verdict** | FINDINGS_REMAIN |

---

## Process-Gaps Identified

- **PG-PR1-001:** Adversary Glob negatives must be cross-verified with `ls -la <abs-path>` or `git ls-files | grep <pattern>` before reporting as findings. (Source: F-PR1-CRIT-001 false-positive)
- **PG-PR1-002 = LP-PR1-001:** Story §File Structure Requirements for identity-newtype migration stories MUST include workspace-wide grep for ALL definition patterns, not just dispatch-site enumeration. (Source: F-PR1-HIGH-002)
- **PG-PR1-003:** Per-story adversary prompt MUST cross-check `subsystems:` array against ARCH-INDEX Subsystem Registry AND against PR diff touched-crates set. (Source: F-PR1-HIGH-001)

---

## Absolute-Path Citations

- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/cache_key.rs` — shadow type alias (F-PR1-HIGH-002)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/cache.rs` — 13 sibling sites (F-PR1-HIGH-002)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/cache_tests.rs` — 17 sibling sites (F-PR1-HIGH-002)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/invalidation.rs` — 3 sibling sites (F-PR1-HIGH-002)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/proofs/vp025_cache_key.rs` — Kani proof (F-PR1-HIGH-002)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-query/src/fanout.rs:409` — sentinel string (F-PR1-MED-001)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-core/src/sensor_id.rs:511` — should_panic (F-PR1-LOW-001)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-core/src/sensor_id.rs:518` — should_panic (F-PR1-LOW-001)
- `/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md` — story §File Structure + subsystems (F-PR1-HIGH-001, F-PR1-MED-002)
- `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/docs/demo-evidence/S-PLUGIN-PREREQ-A/` — 12 demo files present (F-PR1-CRIT-001 reclassification evidence)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/ARCH-INDEX.md` — Subsystem Registry (F-PR1-HIGH-001)

---

## Closure Annotations (state-manager 2026-05-11)

| Finding | Status | Closing commit | Notes |
|---|---|---|---|
| F-PR1-CRIT-001 | RECLASSIFIED — adversary tool false-positive | N/A | Glob returned 0 but `ls -la` confirmed 12 files exist; commit 8dd9a89e log shows 12 files added in docs/demo-evidence/S-PLUGIN-PREREQ-A/ (963 insertions). Not a defect. Codified as new process-gap PG-PR1-001. |
| F-PR1-HIGH-001 | CLOSED | factory-artifacts baae27fd | Story v1.5 subsystems: [SS-01, SS-02, SS-08, SS-16] → [SS-01, SS-11, SS-16, SS-21]. Per-entry justification recorded. |
| F-PR1-HIGH-002 | CLOSED | worktree ba7d7f6f | `pub type SensorId = String;` deleted from cache_key.rs; replaced with `pub use prism_core::SensorId;`. Found 30+ additional sibling sites in cache.rs (13), cache_tests.rs (17), invalidation.rs (3), vp025_cache_key.rs Kani proof, and `CacheKey::derive()` API. ALL migrated. 1,398 tests pass post-fix. Hash stability preserved via Borrow<str> impl. |
| F-PR1-MED-001 | CLOSED | worktree ba7d7f6f | fanout.rs:409 sentinel renamed "unknown" → "internal-panic-recovery" (semantically reserved). |
| F-PR1-MED-002 | CLOSED | factory-artifacts baae27fd | Story v1.5 added cache_key.rs to §File Structure Requirements + new AC-12 (type-alias inventory grep enforced). |
| F-PR1-MED-003 | CODIFIED | factory-artifacts baae27fd | Process-gap codified as LP-PR1-001 in story v1.5; TD-VSDD-082 filed (see tech-debt-register.md). See PG-PR1-002. |
| F-PR1-LOW-001 | CLOSED | worktree ba7d7f6f | sensor_id.rs:511+518 should_panic shortened to "invalid SensorId string" (dropped story-id prefix). |

## Process-Gaps Codified (state-manager additions)

- **PG-PR1-001 [NEW]:** Adversary Glob negatives must be cross-verified with `ls -la <abs-path>` or `git ls-files | grep <pattern>` before reporting as findings. F-PR1-CRIT-001 false-positive blocked a clean pass-1 verdict that would otherwise have been more accurately classified.
- **PG-PR1-002 [NEW] = LP-PR1-001 in story v1.5:** §File Structure Requirements for identity-newtype migration stories MUST include workspace-wide grep for ALL definition patterns (`pub type <Name>`, `pub struct <Name>`, `type <Name> =`, `pub use ...::<Name>`) not just dispatch-site enumeration. Codified in story v1.5 lessons section.
- **PG-PR1-003 [NEW]:** Per-story adversary prompt MUST cross-check `subsystems:` array against ARCH-INDEX Subsystem Registry AND against PR diff touched-crates set. Currently LOCAL pass adversary anchors to story body + diff, NOT to ARCH-INDEX. Filed as TD-VSDD-083.
