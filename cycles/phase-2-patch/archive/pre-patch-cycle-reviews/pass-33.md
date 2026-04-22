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
pass: 33
previous_review: pass-32.md
cycle: phase-2-patch
novelty: MEDIUM
findings: 3
critical: 0
high: 1
medium: 2
low: 0
convergence_counter: 0 of 3
---

# Adversarial Review: Prism (Pass 33)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix from `.factory/current-cycle` — `P3PATCH` for this cycle
- `<PASS>`: Two-digit pass number
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

This cycle uses the shorthand `P3P33-A-[SEV]-[NNN]` per established pass-cycle convention.

## Scope

Full fresh-context review. Burst 33 closure verification (S-5.06 tool-name rename). Policy 2, 6, 7, 8, 9 re-sweeps. BC-INDEX v4.10, STORY-INDEX v1.25, test-vectors v2.1 arithmetic. New-axis exploration: capability-name consistency (CAP-033), propagation of tool-name rename to adjacent documents, PRD ↔ nfr-catalog supplement count sync.

## Part A — Fix Verification (Burst 33 — 1/1 CLOSED)

All S-5.06 changes verified:

| Item | Previous Severity | Status | Notes |
|------|-------------------|--------|-------|
| M-101 S-5.06 execute_action → fire_action rename (12 occurrences) | MEDIUM | RESOLVED | 0 execute_action refs remain; 12 fire_action refs confirmed; line 51 parenthetical removed; fire_action.rs in File Structure |

**S-5.06 Burst 33 closure: COMPLETE for the story file.** However, the tool-name drift axis has propagation gaps in adjacent documents not swept this burst.

### Policy Re-sweep Summary

- **Policy 2 DI orphan scan:** All 28 active DIs cited by ≥1 BC. CLEAN.
- **Policy 6 ARCH-INDEX sync:** Sampled 4 BCs — subsystem fields match ARCH-INDEX. CLEAN.
- **Policy 7:** Sampled 10 fresh BCs + 3 stories — all H1/body titles verbatim match BC-INDEX. CLEAN.
- **Policy 8 bidirectional:** Sampled 10 stories — all frontmatter BCs have AC traces. CLEAN.
- **Policy 9 VP-INDEX arithmetic:** 39 = 20 Kani + 11 Proptest + 6 Fuzz + 2 Integration. Per-module sums match. CLEAN.

### Arithmetic

- BC-INDEX v4.10: 195 + 6 + 2 = 203 ✓; subsystem sums 195 ✓; P0 166 + P1 29 = 195 ✓
- STORY-INDEX v1.25: 75 stories; BCs covered = 195; VPs = 39; wave sum 234 ✓
- test-vectors v2.1: VP-034 live citations = 0 ✓

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

#### P3P33-A-H-001 — Capability-name drift: CAP-033 uses `action.execute` vs canonical `action.write` (17:3 vote)

**Policy violated:** 6 (architecture/domain-spec consistency with implementing specs)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** NEW — same structural class as pass-32 M-101 but at L2-domain-spec ↔ L3/L4 axis

**Evidence:**
- `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/capabilities.md` line 53: CAP-033 uses `action.execute` 3 times
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/actions.md` lines 532, 533, 620: `action.write` (3 refs)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/api-surface.md` lines 160, 161: `action.write` (2 refs)
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.06-action-infusion-tools.md` lines 51/52/183/192/204/251/315/323/344/349/372/411/504: `action.write` (13 refs)

Vote: 17 canonical refs to `action.write`, 3 stale refs to `action.execute` (all in CAP-033).

**Why it fails:**
- (a) Implementer reading CAP-033 registers `action.execute` in CapabilityResolver; S-5.06 gates tools on `action.write` → capability lookup returns Deny-by-default (DI-003); `fire_action`/`create_action`/`delete_action`/`test_action` unreachable
- (b) TOML author writes `action.execute = allow`; runtime expects `action.write` → write denied
- (c) Two-tier feature flag contract broken at naming layer

**Proposed fix:** Rename `action.execute` → `action.write` in capabilities.md CAP-033 (3 occurrences line 53).

### MEDIUM

#### P3P33-A-M-001 — 5 stale `execute_action` refs in test-vectors.md (Burst 33 scope limited to S-5.06 only)

**Policy violated:** 9 propagation (tool-name propagation across corpus after canonical rename)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** NEW — propagation exposure from narrowly-scoped Burst 33 fix
**File:** `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md`

**Evidence:**
- Lines 46, 47, 48 (TV-001 / EC-05-004 / EC-05-005): "`execute_action` with `parameters.credential_ref = ...`"
- Line 75 (TV-007 BC-2.04.005): "`execute_action`, `create_case`, `update_case`, `delete_rule`, `set_credential` are completely ABSENT"
- Line 266 (Cross-Subsystem Integration): "`execute_action` (contain_host, irreversible=true) with `credential_ref` in params"

Burst 33 renamed 12 occurrences in S-5.06 but did not sweep test-vectors.md which also uses the tool name canonically (or canonically attempted — now stale).

**Why it fails:**
- Line 75 hidden-tools assertion: test-writer verifies `execute_action` absent from tools/list — passes vacuously (never existed), while actual `fire_action` registration goes un-asserted
- Line 266 Cross-Subsystem Vector: treats `execute_action` as wrapper for `contain_host` — but canonical is direct `crowdstrike_contain_host` invocation per api-surface.md:144

**Proposed fix:** Product-owner decides semantic intent per location:
- Lines 46-48: likely `fire_action` (manual action delivery context)
- Line 75: `fire_action` in hidden-tools list
- Line 266: likely `crowdstrike_contain_host` (direct sensor write) or `fire_action` (action pipeline)

Bump test-vectors.md version 2.1 → 2.2.

---

#### P3P33-A-M-002 — PRD claims 16 NFRs; nfr-catalog defines 18

**Policy violated:** Cross-supplement currency (PRD ↔ nfr-catalog sync)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** NEW — new axis not probed in prior passes

**Evidence:**
- `/Users/jmagady/Dev/prism/.factory/specs/prd.md` line 471: "16 non-functional requirements covering five quality dimensions"
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/nfr-catalog.md` lines 15, 26, 36, 46, 55, 64, 74, 84, 93, 102, 112, 120, 129, 138, 147, 157, 165, 175 — 18 `## NFR-NNN:` H2 headings (NFR-001..NFR-018)

Likely post-16 additions: NFR-017 (Cache Bounds per DI-018) and NFR-018 (Token Store Cap per DI-015).

**Proposed fix:** Update PRD line 471 "16" → "18". Note NFR-017/.018 in PRD changelog.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (H-001 blocks counter advance)
**Readiness:** requires revision (Burst 34 — 3 surgical fixes before pass-34)

## Observations

1. All 3 findings are GENUINELY NEW (not retreads).
2. Arithmetic integrity intact across BC-INDEX / STORY-INDEX / VP-INDEX / verification-coverage-matrix.
3. All 5 policy sweeps (2, 6, 7, 8, 9) pass clean.
4. PRD line 60 BC count matches BC-INDEX v4.10 exactly.
5. test-vectors.md v2.1 VP-034 live citations = 0 (only in changelog).
6. No other tool-name drift detected in 25-tool triangle spot-check across BC/arch/story.
7. Corpus-wide `execute_action` grep: 0 in source code or architecture; only drift is test-vectors.md (M-001) and CAP-033 (capability-name drift, different axis).

## Novelty Assessment

**Novelty: MEDIUM.** Three findings:
- H-001: New axis (capability-name drift) — same structural class as pass-32 M-101 but at L2-domain ↔ L3/L4 level
- M-001: Propagation exposure — Burst 33 scoped narrowly to S-5.06, test-vectors.md retention surfaced
- M-002: New axis (PRD ↔ nfr-catalog supplement pair) not probed before

Trajectory: 26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→4→6→2→**3**. Small uptick from 2→3 but findings are surgical and decay pattern continues (well below mid-cycle highs). CRIT=0 for 21+ passes. HIGH=1 breaks 2-pass zero-HIGH streak but H-001 is same-class as M-101.

## Convergence Recommendation

**BLOCK at 0/3.**

**Burst 34 scope (small, surgical, single-burst clearable):**
1. **H-001:** `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/capabilities.md` line 53 CAP-033 — rename `action.execute` → `action.write` (3 occurrences). Architect scope.
2. **M-001:** `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` lines 46/47/48/75/266 — product-owner decides semantic intent (likely `fire_action` or `crowdstrike_contain_host`). Bump v2.1 → v2.2.
3. **M-002:** `/Users/jmagady/Dev/prism/.factory/specs/prd.md` line 471 "16" → "18" + PRD changelog note NFR-017/.018. Product-owner scope.

Estimated pass-34 outcome: advance to 1/3 if Burst 34 lands cleanly.

## Relevant Files

- `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/capabilities.md` (H-001 CAP-033 line 53)
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` (M-001 lines 46/47/48/75/266)
- `/Users/jmagady/Dev/prism/.factory/specs/prd.md` (M-002 line 471)
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/nfr-catalog.md` (ground truth: 18 NFRs)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/api-surface.md` (canonical `action.write`)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/actions.md` (canonical `action.write`)
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.06-action-infusion-tools.md` (canonical `action.write` + `fire_action` — Burst 33 verified)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.18.003-action-manual-fire-and-forget.md` (canonical `fire_action`)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-INDEX.md` (v4.10 arithmetic clean)
- `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` (v1.25 arithmetic clean)
