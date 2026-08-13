---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 12
phase: LOCAL
frozen_head: "(story v1.12 + code a1864d3eb + spec leg: BC-2.16.002 v2.19 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.22 / BC-2.19.001 v2.3 / ADR-050 v2.3 / error-taxonomy v2.74)"
new_feature_head: "a1864d3eb (UNCHANGED — records-only burst; no code commits)"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 1
streak_before: 0
streak_after: 0
closed_by: "F-P-MED-001 [MED] BC-2.19.001 §Error Conditions E-INFUSE-015 row stated 'load_spec_with_runtime (2 sites)' — omitted §load_spec entirely and miscounted §load_spec_with_runtime. Authoritative source (code HEAD a1864d3eb + error-taxonomy v2.74): E-INFUSE-015 fires from §load_spec (1 site), §load_spec_with_runtime (1 site), §hot_reload (1 site) — 1/1/1 not 0/2/1. CLOSED: BC-2.19.001 v2.3→v2.4 (§Error Conditions E-INFUSE-015 firing-path corrected to all 3 callers 1/1/1); story v1.12→v1.13 (AC-ERR-006 prose + §Files-to-Modify caller list + §Behavioral-Contracts scope cell all updated to enumerate all 3 paths; BC-2.19.001 v2.4 pin; full-story sweep zero residuals). Code HEAD a1864d3eb UNCHANGED."
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 12

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (records-only burst; code HEAD a1864d3eb unchanged; streak resets due to MED finding)
**1 finding: F-P-MED-001 [MED] CLOSED (BC-2.19.001 v2.4 + story v1.13; records-only; TD-VSDD-096)**

**Context:** Pass-12 ran on the post-D-2125 frozen HEAD (story v1.12 + code a1864d3eb). This pass found a single MED-severity records-tier finding: E-INFUSE-015 firing-path enumeration in BC-2.19.001 §Error Conditions was incomplete and miscounted. The BC stated "load_spec_with_runtime (2 sites)" — omitting §load_spec entirely and double-counting §load_spec_with_runtime. Authoritative source (code + error-taxonomy v2.74): all 3 infusion callers fire the path with 1 site each. Story v1.12 AC-ERR-006, §Files-to-Modify caller list, and §Behavioral-Contracts scope cell each had the same 2-site under-count. All corrected via TD-VSDD-096 records-only micro-burst. Code HEAD a1864d3eb unchanged. CLEAN(PR-merge)=NO because MED severity.

---

## Finding F-P-MED-001 [MED] — CLOSED

**ID:** F-P-MED-001
**Severity:** MED
**Class:** Records-tier enumeration error (firing-path under-count in BC + story)
**Status:** CLOSED (BC-2.19.001 v2.4 + story v1.13; pre-written by product-owner + story-writer)

**Description:**

BC-2.19.001 §Error Conditions E-INFUSE-015 row stated firing path as "load_spec_with_runtime (2 sites)". This was wrong in two ways:
1. §load_spec omitted entirely — it is the first caller in `§spec_driven_adapter` that calls `§load_spec`, which internally calls `§build_http_client_with_timeout`.
2. "2 sites" for load_spec_with_runtime is a miscounting — there is exactly 1 site for each of the 3 callers.

Authoritative source: code HEAD a1864d3eb + error-taxonomy v2.74 which correctly states "3 infusion callers wired". The correct enumeration is §load_spec (1 site) + §load_spec_with_runtime (1 site) + §hot_reload (1 site) = 1/1/1.

Story v1.12 carried the same error in three places:
- AC-ERR-006 prose: stated "load_spec_with_runtime 2 call sites"
- §Files-to-Modify caller list: omitted §load_spec
- §Behavioral-Contracts scope cell: stated "2-site" language

**Fix applied:**
- BC-2.19.001 v2.3→v2.4: §Error Conditions E-INFUSE-015 firing-path corrected to "§load_spec + §load_spec_with_runtime + §hot_reload (1/1/1 sites each)"
- Story v1.12→v1.13: AC-ERR-006 prose + §Files-to-Modify + §Behavioral-Contracts scope cell all updated to enumerate all 3 callers with 1/1/1 site count; BC-2.19.001 v2.4 pin propagated

**Full-story sweep:** Zero residuals. All other references to the firing-path in story v1.13 confirmed correct post-fix.

---

## Code Core Verdict

Pass-12 confirmed code HEAD a1864d3eb is UNCHANGED. All pass-11 code-core verdicts carry forward (no new code introduced). Code is CRIT/HIGH/MED-clean on all original story acceptance criteria:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS (unchanged)
- §D6 User-Agent sweep: all 4 sites — PASS (unchanged)
- Non-2xx body snippet byte-cap via §prism_core::sanitize_body_snippet_bytes — PASS (unchanged)
- Error source-chain wiring — PASS (unchanged)
- Error mapping Arm-1/Arm-2/Arm-3 — PASS (unchanged)
- E-INFUSE-015 InfusionError::HttpClientBuildFailed wired to 3 callers — PASS (unchanged; finding F-P-MED-001 was a documentation error, not a code correctness issue)
- SAP-1: catalog row 91 `fan_out_target_failed` present; zero unregistered `event_type` emissions — PASS (unchanged)
- SAP-3: all 13 RGTs load-bearing — PASS (unchanged)
- 95/95 non-exhaustive; 5724 tests green — PASS (unchanged)

---

## TD-VSDD-097 Three-Dimension Sweep

**Dimension 1 — Sibling pair:** BC-2.19.001 §Error Conditions E-INFUSE-015 has no spec-sibling with an identical infusion-load surface (the three infusion callers are all in one BC; no twin exists). Story is sole owner of AC-ERR-006. **CLEAR.**

**Dimension 2 — Downstream copy target:** The E-INFUSE-015 firing-path text in BC-2.19.001 §Error Conditions is not verbatim copy-sourced into any downstream artifact (not transcribed by orchestrator or agent legs into ADRs, ARCH-INDEX, or other BCs). **CLEAR.**

**Dimension 3 — Mandate anchor:** No new MUST blocks authored in this pass. The firing-path enumeration correction is a factual accuracy fix — it removes an under-count but does not introduce new normative obligations. **CLEAR.**

---

## Convergence Trajectory

Pass 1: 5 findings | Pass 2: 6 | Pass 3: 1 | Pass 4: 3 | Pass 5: 2 | Pass 6: 3 | Pass 7: 1 | Pass 8: 4 | Pass 9: 2(LOW) | Pass 10: 2(LOW+OBS) | Pass 11: 2(F-2 HUMAN-DIRECTED) | Pass 12: 1(MED)

Full trajectory: 5→6→1→3→2→3→1→4→2(LOW)→2(LOW+OBS)→2(F-2 HUMAN-FIX)→1(MED)

BC-5.39.001 streak: 0/3. NEXT: strict LOCAL adversary pass-13 on frozen HEAD a1864d3eb + story v1.13.
