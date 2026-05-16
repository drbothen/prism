---
document_type: adversarial-review-pass
pass: 38
cycle: S-PLUGIN-PREREQ-E-spec
date: 2026-05-16
reviewer: adversary
predecessor_pass: 37
predecessor_burst: "FB29 D-646 SHA 89a724cb"
verdict: BLOCKED
finding_count: { CRIT: 0, HIGH: 0, MED: 1, LOW: 1, OBS: 1 }
streak_status: "0/3 stays 0/3 — BLOCKED holds; 9th attempt at 3-CLEAN sequence"
fix_burst: FB30
fix_burst_committed: <state-manager records SHA after commit>
---

# S-PLUGIN-PREREQ-E Spec — Adversarial Review Pass 38

## §1 Summary
BLOCKED. 1 MED + 1 LOW + 1 OBS (carry-forward, non-blocking). Streak 0/3 → 0/3 (BLOCKED holds; 9th attempt at 3-CLEAN sequence reset by pass-36/37/38 each surfacing new findings).

## §2 Methodology
Loaded all 19 spec-package artifacts at FB29-post versions (story v1.14, VP-153 v0.6, VP-INDEX v1.48, STORY-INDEX v2.118, verification-architecture v1.38, verification-coverage-matrix v1.35, all others unchanged). Applied 25 active policies (POL-1..16, 18, 20..27) with emphasis on POL-7, POL-22 Phase A+C, POL-24, POL-26 (changelog cell counts), POL-27 (BC modified-field ISO format). TD-VSDD-091 anti-volatile-pin axis applied. Phase C named-entity verification confirmed ADR-026 contains 0 occurrences of `forbid`/`forbidden` — surfaced F-LP38-MED-001 (FB29 closure 2 introduced a phantom-authority claim).

## §3 Findings

### F-LP38-MED-001 — Story Task 7 misrepresents ADR-026 §D7 stance on OnceLock as "explicitly forbidden"
- **Severity:** MEDIUM
- **Policies:** POL-7 (source-of-truth precedence), POL-22 Phase C (named-entity verification), POL-24 (verbatim discipline applied to normative claims).
- **File:** `.factory/stories/S-PLUGIN-PREREQ-E-unseal-sensor-auth-deprecate-customadapter.md` line 170 (Task 7).
- **Evidence:** Story claimed "OnceLock wrapper is explicitly forbidden by that ADR". ADR-026 §D7 actually uses "not needed" + positive rationale (boot-step 7.5/8 ordering; eager-init simpler; OnceLock::get_or_init panic-pattern avoidance). Lexical sweep `grep -E "forbid|forbidden"` on ADR-026 = 0 matches.
- **Impact:** Phantom-authority gate. Cross-context contamination: "explicitly forbidden" IS correct in BC-2.16.012 EC-016-012-004 (last-writer-wins; ADR-026 D7 strict reject contract via `DuplicateWriteToolRegistration`). FB29 closure borrowed the phrase from that correct context and misapplied it to OnceLock where the ADR has only preference + rationale.
- **Fix routing:** product-owner — rephrase Task 7 to match ADR's actual stance.
- **Closure:** FB30 PO stage — Task 7 line 170 rewritten with rationale-based language (boot-step ordering + panic-pattern). Story v1.14 → v1.15.

### F-LP38-LOW-001 — Task 7 cites volatile line-range "ADR-026 lines 246-259" violating TD-VSDD-091
- **Severity:** LOW
- **Policy:** TD-VSDD-091 anti-volatile-pin (narrative spec content cites semantic anchors, NOT line numbers; ADRs drift).
- **Same site (line 170).** ADR-026 has bumped 12+ times in ~24 hours. Line 246 today is the rationale-start; future inserts above §D7 silently shift the range.
- **Closure:** FB30 PO stage — line range dropped; `§D7` semantic anchor sufficient (naturally absorbed by MED-001 rephrasing).

## §4 FB29 Paper-Fix Audit (TD-VSDD-059)
- **Closure 1 (AC-8 4-test enumeration):** ✅ NOT a paper-fix. AC-8 line 235 + Red Gate Tests 7-10 byte-verbatim match.
- **Closure 2 (Task 7 OnceLock + ADR-026 §D7 citation):** ⚠️ NOT a paper-fix in mechanism but INTRODUCED F-LP38-MED-001 (overstrong normative claim not borne out by ADR text). FB30 closes the introduced defect.
- **Closure 3 (VP-153 byte-verbatim sync):** ✅ NOT a paper-fix. Rules A/B/C byte-verbatim match error-taxonomy.md v1.30 E-SPEC-012/013/014.

## §5 Sibling-Sweep Audit (TD-VSDD-060)
- Zone 1 (AC-8 4-test names verbatim): 8 occurrences confirmed (4 in AC-8, 4 in Red Gate 7-10) byte-verbatim ✅
- Zone 2 (ADR-026 §D7 lexical-vs-semantic): §D7 anchor resolves to `### D7` at ADR-026:242 ✅; lines 246-259 contain rationale (mechanically — volatile per TD-VSDD-091); semantic claim "explicitly forbidden" FAILS POL-22 Phase C ❌ → F-LP38-MED-001
- Zone 3 (VP-153 v0.6 sibling pins): VP-INDEX v1.48, verification-architecture v1.38, verification-coverage-matrix v1.35, STORY-INDEX v2.118 row v1.14, BC/HS ID-only refs — all clean ✅; workspace grep on old divergent strings = 0 hits ✅
- Zone 4 (index changelog cell counts POL-26): STORY-INDEX (3-col), VP-INDEX (5-col), verification-architecture (5-col), verification-coverage-matrix (4-col) — all clean ✅
- Zone 5 (BC modified ISO POL-27): all 4 BCs + 4 VPs ISO-formatted ✅
- Zone 6 (behavioral_contracts ↔ body BC table ↔ AC traces POL-8): 5 BCs consistent across frontmatter / body / AC traces / STORY-INDEX cell ✅
- Zone 7 (VP-INDEX → arch named-alias semantic sync POL-9): VP-153/154/155/156 rows match across VP-INDEX / verification-architecture Provable Properties Catalog / verification-coverage-matrix ✅

## §6 Observations

### OBS-LP38-001 [process-gap] — VP-INDEX v1.48 changelog row narrative omits POL-11 citation present in sibling propagation rows
- **Severity:** OBS (non-blocking; codification candidate at cycle-close)
- VP-INDEX row cites only POL-9. verification-architecture v1.38 + verification-coverage-matrix v1.35 rows cite POL-9 + POL-11.
- Recurrence pattern (per story changelog): F-LP15-HIGH-001, F-LP21-HIGH-001, F-LP31-HIGH-001, F-LP32-HIGH-001, F-LP33-HIGH-001, F-LP34-HIGH-001 — 6+ propagation-narrative-asymmetry recurrences across these three artifacts.
- **Routing:** state-manager dispatch template review at cycle-close. Templated 2-clause phrasing ("POL-9 same-burst propagation + POL-11 changelog row recorded") would prevent future single-MED resets.
- Substantive content (version bump + changelog row + propagation) is intact across all three; only narrative phrasing varies. Non-blocking.

## §7 Convergence Trajectory
- Pass-38: BLOCKED (1 MED + 1 LOW + 1 OBS)
- Streak: 0/3 → 0/3 (BLOCKED holds; 9th attempt at 3-CLEAN sequence)
- Pattern: F-LP38-MED-001 was introduced BY FB29 Closure 2 itself — the canonical "fix-burst introduces new defect" recurrence (11+ POL-23 manifestations cataloged in changelog). FB29 dispatch focused on stripping the OnceLock alternative + adding §D7 citation; did not run POL-22 Phase C on the new normative claim.
- Recommendation: Continue cascade. FB30 single-burst PO closure (MED-001 + LOW-001 absorbed together). OBS-LP38-001 process-gap codification deferred to cycle-close (Cycle-Closing Checklist S-7.02). Spec package structurally sound — no pause needed.
