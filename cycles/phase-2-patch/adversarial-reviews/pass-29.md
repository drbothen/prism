---
document_type: adversarial-review
level: ops
version: "1.0"
status: findings-open
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
pass: 29
previous_review: pass-28.md
cycle: phase-2-patch
novelty: MEDIUM
findings: 5
critical: 0
high: 2
medium: 2
low: 1
previous_pass: 28 (5 findings: 2 HIGH, 2 MED, 1 LOW — all 5 closed Burst 29)
convergence_counter: 0 of 3
---

# Pass 29 — Whack-a-mole pattern confirmed: title-sync sweep reveals missed BCs despite Burst 29 targeted fix

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix from `.factory/current-cycle` — `P3PATCH` for phase-2-patch
- `<PASS>`: Two-digit pass number (`P29`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

This cycle uses short-form IDs (`P3P29-A-H-001` etc.) consistent with all prior passes in this cycle.

## Scope

Fresh-context review. Burst 29 closure verification for 5 pass-28 findings. Observation-1 follow-up. Comprehensive BC-INDEX title-sync audit across Wave 1-6 stories. BC-INDEX arithmetic integrity. VP-INDEX arithmetic and architecture propagation (Policy 9). Policy 2 orphan scan. Policy 8 bidirectional check. Policy 6 ARCH-INDEX sync. Fresh BC spot-check (BC-2.03/.08/.09/.17).

## Part A — Fix Verification (Burst 29 Closure)

All 5 pass-28 findings closed. No regressions.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| P3P28-A-H-001 (S-1.09) | HIGH | RESOLVED | UUID v7→crypto-random, cap→E-FLAG-007, expiry→E-FLAG-003; S-1.09 lines 51, 62, 148-149 correct |
| P3P28-A-H-002 (S-3.04) | HIGH | RESOLVED | 4 MCP tool names backticked; S-3.04 lines 46, 50, 51, 52 all backticked |
| P3P28-A-M-001 (S-2.01) | MEDIUM | RESOLVED | BC-2.15.005 title includes "Operation"; line 47 matches BC-INDEX v4.10 verbatim |
| P3P28-A-M-002 (test-vectors.md) | MEDIUM | RESOLVED | VP-034 removed; v2.1; TV-001 "integration only"; line 4 version 2.1; lines 49, 297 integration only; VP-034 only in changelog |
| P3P28-A-L-001 (S-3.07) | LOW | RESOLVED | AC-9 traces to BC-2.04.005; lines 320-329 present |
| Observation-1 | observation | RESOLVED | STORY-INDEX v4.10 pin; lines 24, 72 pin v4.10 |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

#### P3P29-A-H-001 — S-1.10 body BC table shows factually wrong title for BC-2.09.004

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Policy violated:** 7 (`bc_h1_is_title_source_of_truth`)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.10-prompt-injection-defense.md` line 41
- **Novelty:** NEW — pre-existing drift not caught in prior sweeps
- **Description:** S-1.10 BC table lists the old factually-wrong title for BC-2.09.004. The old title was explicitly retired in BC-INDEX v4.6 nine versions ago.
- **Evidence:**
  - S-1.10 line 41: `| BC-2.09.004 | Safety Flag Parallel Fields (Flag, Don't Strip) |`
  - BC-2.09.004 H1 (file line 23): `Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)`
  - BC-INDEX line 122: `Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)`
  - BC-INDEX v4.6 changelog line 441 explicitly states: "BC-2.09.004: 'Safety Flag Parallel Fields (Flag, Don't Strip)' → 'Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)' (BC body unambiguous: centralized array, no per-field parallel fields; **old BC-INDEX title was factually wrong**)"
- **Proposed Fix:** Update S-1.10 line 41 title to BC-INDEX verbatim: `Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)`

---

#### P3P29-A-H-002 — S-1.10 body BC-2.09.003 missing "with NFKC Normalization" qualifier

- **Severity:** HIGH
- **Category:** spec-fidelity, security-surface
- **Policy violated:** 7, 4
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.10-prompt-injection-defense.md` line 40
- **Novelty:** NEW
- **Description:** S-1.10 BC table drops the NFKC Normalization qualifier from BC-2.09.003 title. This qualifier is security-sensitive: without Unicode canonicalization before regex, homoglyph bypass attacks defeat the scanner.
- **Evidence:**
  - S-1.10 line 40: `| BC-2.09.003 | Suspicious Pattern Detection via Regex |`
  - BC-2.09.003 H1 and BC-INDEX line 121: `Suspicious Pattern Detection via Regex with NFKC Normalization`
  - BC-INDEX v4.6 changelog line 440: `Added "with NFKC Normalization"`
- **Proposed Fix:** Update S-1.10 line 40 to: `| BC-2.09.003 | Suspicious Pattern Detection via Regex with NFKC Normalization |`

### MEDIUM

#### P3P29-A-M-001 — S-1.12 body BC table omits backticks on 3 MCP tool names (same class as pass-28 H-002)

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Policy violated:** 7
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.12-hot-reload.md` lines 38, 41, 42
- **Novelty:** NEW — preemptive sweep missed Spec Engine story
- **Description:** Three MCP tool names in S-1.12's BC table lack backticks, inconsistent with BC-INDEX v4.10 formatting and the fix applied to S-3.04 in Burst 29.
- **Evidence:**
  - Line 38: `| BC-2.16.005 | reload_config MCP Tool — ... |` — `reload_config` unbackticked
  - Line 41: `| BC-2.16.008 | add_sensor_spec MCP Tool — ... |` — `add_sensor_spec` unbackticked
  - Line 42: `| BC-2.16.010 | list_sensor_specs MCP Tool — ... |` — `list_sensor_specs` unbackticked
  - BC-INDEX v4.10 lines 206, 209, 211 all use backticks
- **Proposed Fix:** Backtick the 3 tool names: `` `reload_config` ``, `` `add_sensor_spec` ``, `` `list_sensor_specs` ``

---

#### P3P29-A-M-002 — S-1.08 BC-2.04.004 uses em-dash where BC-INDEX uses double-hyphen

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Policy violated:** 7
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.08-feature-flags.md` line 41
- **Novelty:** NEW
- **Description:** S-1.08 uses an em-dash (—) in the BC-2.04.004 title where BC-INDEX and the BC file both use a double-hyphen (--). The adjacent row (BC-2.04.005, line 42) correctly uses `--`, creating internal inconsistency.
- **Evidence:**
  - S-1.08 line 41: `| BC-2.04.004 | Two-Tier Gate — Both Compile-Time and Runtime Must Permit Operation |` (em-dash)
  - BC file H1 line 23 + BC-INDEX line 71: `Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation` (double hyphen)
- **Proposed Fix:** Replace em-dash with `--` on line 41.

### LOW

#### P3P29-A-L-001 — 8 stories use non-canonical 3-col BC table schema (pre-existing baseline, not a regression)

- **Severity:** LOW
- **Category:** spec-fidelity (loosely — schema variation)
- **Policy violated:** 7 (loosely)
- **Location:** S-1.01/.02/.03/.04/.05/.06, S-6.04/.05, S-5.07/.08/.09/.10, S-4.08
- **Novelty:** RE-CONFIRMED pre-existing pattern
- **Description:** Multiple stories use 3-column BC table variants (`| BC | Clause | Description |`, `| BC ID | Clause | Description |`, `| BC ID | Title | Clause/Invariant |`) rather than the canonical 2-column `| BC-ID | Title |` schema. Title column absent makes Policy 7 verbatim-match unverifiable on those rows. Not a convergence blocker. Optional post-v1.0 standardization.
- **Proposed Fix:** Defer to post-v1.0 batch `/vsdd-factory:conform-to-template` pass.

## Observations

1. **BC-INDEX arithmetic clean:** 195 active (166 P0 + 29 P1) + 6 removed + 2 retired = 203 ✓
2. **Policy 9 (VP-INDEX) clean:** 39 VPs (20 Kani + 11 Proptest + 6 Fuzz + 2 Integration); coverage-matrix per-tool sums match; VP-039 propagated to both architecture anchor docs ✓
3. **Policy 2 orphan scan clean:** All 32 active DIs have at least one BC citation; DI-009/010/011/013 properly tombstoned ✓
4. **Policy 8 bidirectional (10 stories sampled):** S-5.01, S-5.05, S-4.06, S-1.08, S-1.10, S-1.12, S-2.01, S-3.04, S-3.07, S-4.08 — all frontmatter/body/AC coherent ✓
5. **Policy 6 ARCH-INDEX sync (6 BCs sampled):** BC-2.05.011→SS-05, BC-2.11.001→SS-11, BC-2.13.014→SS-13, BC-2.17.001→SS-17, BC-2.19.001→SS-19, BC-2.15.001→SS-15 — all verbatim ✓
6. **test-vectors.md v2.1 structural integrity:** Frontmatter matches official template; body has all required sections; 10 TVs; traceability matrix correct ✓
7. **Systematic sweeps consistently miss 1–3 stragglers:** Burst 28 did 19-fix sweep but missed S-1.08/1.10/1.12. Pattern persists across passes. **Recommend scripted BC-INDEX-to-story-body title-diff validator** to prevent whack-a-mole.

## Novelty Assessment

**NOVELTY: MEDIUM.** Findings are NEW but **same class** as Burst 28's closed findings (title drift, backticks, dash variants). True novelty would be findings in areas not previously swept. BC-INDEX arithmetic, VP-INDEX arithmetic, Policy 2/6/8/9 all clean.

The BC-2.09.004 finding (H-001) is especially high-impact: old title explicitly documented as factually wrong in BC-INDEX v4.6 changelog (line 441) nine versions ago, yet persists in S-1.10. Body BC tables were never systematically re-synced after v4.6 title corrections.

Trajectory: 26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→**5** (flatlined). CRIT=0 streak holds.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 2 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (counter holds at 0/3)
**Readiness:** requires revision — Burst 30 scope: S-1.10 lines 40+41 title fix; S-1.12 3 backticks; S-1.08 em-dash; comprehensive scripted BC-title sweep STRONGLY RECOMMENDED to break whack-a-mole pattern

## Recommended Burst 30 Scope

1. **H-001/H-002 (S-1.10):** Replace 2 BC titles with verbatim BC-INDEX values (lines 40, 41)
2. **M-001 (S-1.12):** Backtick 3 MCP tool names (lines 38, 41, 42)
3. **M-002 (S-1.08):** Em-dash → double-hyphen on BC-2.04.004 (line 41)
4. **L-001:** Defer to post-v1.0 schema standardization pass
5. **Process improvement (STRONGLY RECOMMENDED):** Script a comprehensive BC-INDEX-to-story-body title-diff tool. Without this, we may keep finding 1-3 title drifts per pass forever. Expected post-Burst-30: 0 findings → counter 1/3 (IF Burst 30 also runs comprehensive scripted sweep).

## Relevant Files

- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-INDEX.md` (v4.10)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.09.003-suspicious-pattern-detection.md`
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.09.004-safety-flag-parallel-fields.md`
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.04.004-two-tier-gate-both-must-pass.md`
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.08-feature-flags.md` (M-002 line 41)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.10-prompt-injection-defense.md` (H-001, H-002 lines 40, 41)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.12-hot-reload.md` (M-001 lines 38, 41, 42)
- `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` (v1.21 verified lines 24, 72)
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` (v2.1)
