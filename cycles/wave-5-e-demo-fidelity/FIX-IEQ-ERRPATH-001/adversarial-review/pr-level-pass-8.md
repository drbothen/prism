---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [8]
feature_head_at_review: ddf852bc
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 0
  low: 1
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 8 — FIX-IEQ-ERRPATH-001

---

## Pass 8 (frozen ddf852bc; fresh-context adversary; PR-LEVEL cascade; streak candidate 1/3 — NOT ADVANCING — 0/3 stays 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 1 total (0 CRIT / 0 HIGH / 0 MED / 1 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` confirmed 91 emission sites; BC-2.16.002 v2.08 catalog arithmetic verified. A6 rewrite and C8 rename from pass-7 verified CORRECT (see Pass-7 Verification section below).

**STREAK:** STAYS 0/3 — NOT CLEAN(strict) on frozen ddf852bc (1 LOW finding; spec-only closure; HEAD UNCHANGED; no push; DRIFT-ORCH-PRLEVEL-PUSH-001 N/A). **Next: PR-LEVEL pass 9 on SAME frozen ddf852bc (streak candidate 1/3; NO push before pass 9 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** ddf852bc (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**Code HEAD after closure:** ddf852bc UNCHANGED (spec-only closure — error-taxonomy v2.36 + story S-PRISMQL-CASE-INSENSITIVE-001 v1.55; no Rust code change)

**CLEAN(strict):** NO — 1 LOW finding

**CLEAN(PR-merge):** YES — LOW severity only (below MED threshold)

---

## Findings

### ADV-PR-P8-LOW-001 — Stale live-currency BC pins in error-taxonomy.md E-QUERY-038 row

**Severity:** LOW
**Confidence:** HIGH
**Novelty:** MEDIUM — Same finding class (POL-25 stale BC live-currency pin) as prior passes but surfaced on a different document (error-taxonomy.md E-QUERY-038 row) that prior sweeps had excluded from the sweep perimeter; not a recurrence of the same site.

**Finding:** `error-taxonomy.md` E-QUERY-038 row body text carried THREE live-currency BC-version pins stuck at "BC-2.11.016 v1.21" while the BC had advanced to v1.25 across four consecutive fix bursts (v1.22 D-1635, v1.23 D-1635, v1.24 D-1636, v1.25 D-1637). The three stale live-currency pin sites were:

1. **Gate scope §Preconditions.2 prose**: `"BC-2.11.016 v1.21 §Preconditions.2"` — cites v1.21, which predates the dual-surface class sweep (v1.22–v1.25) that corrected all 14 gate positions; current contract is v1.25.
2. **Full rule citation (DERIVED-COLUMN BINDING RULE section)**: `"BC-2.11.016 v1.21"` — same stale version.
3. **BC anchor at close of the row**: `"BC-2.11.016 v1.21"` — same stale version.

**Root cause:** Four consecutive pass closures (v1.22 through v1.25 bursts) each executed a sweep of BC-2.11.016 live-currency pins across the four carrier stories (S-DEMO-FIDELITY-REMEDIATION-001, S-DEMO-PRISMQL-ONBOARDING-001-B, S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001, S-PRISMQL-CASE-INSENSITIVE-001) but each sweep conclusion incorrectly stated "no live body-text pins in error-taxonomy.md" or omitted error-taxonomy.md from the sweep perimeter entirely. This is a POL-25 oversight: live-currency pins must be bumped in ALL documents that cite the version, not only the four designated carrier stories.

**Orchestrator adjudication:** Intent (b) — genuine POL-25 oversight. The root cause is a sweep-perimeter gap (error-taxonomy.md excluded from the BC-pin sweep template used for passes v1.22–v1.25). Not a methodology deviation.

**Status:** CLOSED (same-burst spec-only closure; error-taxonomy v2.35→v2.36; S-PRISMQL-CASE-INSENSITIVE-001 v1.54→v1.55; HEAD ddf852bc UNCHANGED)

---

## Closure — ADV-PR-P8-LOW-001

**Product-owner burst:** error-taxonomy.md v2.35→**v2.36**

Three live-currency pins in the E-QUERY-038 row body text bumped from v1.21 to v1.25:

1. Gate scope §Preconditions.2 prose: `"BC-2.11.016 v1.21 §Preconditions.2"` → `"BC-2.11.016 v1.25 §Preconditions.2"`
2. Full rule DERIVED-COLUMN BINDING RULE citation: `"BC-2.11.016 v1.21"` → `"BC-2.11.016 v1.25"`
3. BC anchor at row close: `"BC-2.11.016 v1.21"` → `"BC-2.11.016 v1.25"`

Additionally, the eight other `BC-2.11.016 vN.NN` anchors in the E-QUERY-038 §Changelog and §Origin sections were classified as **origin pins** — they record the BC version in force when each scope extension was introduced and MUST NOT be bumped on subsequent BC increments. A collective origin-pin convention note was added to the E-QUERY-038 row preamble:

> *"Named-rule version anchors (e.g., 'BC-2.11.016 v1.6 §Preconditions.2 Gate scope') are origin pins recording which BC version introduced the scope extension; they do not constitute live-currency citations and are not bumped when the BC advances."*

**Story-writer burst:** S-PRISMQL-CASE-INSENSITIVE-001 v1.54→**v1.55**

AC-022 body contained one live pin: `"error-taxonomy.md v2.35 §E-QUERY-002"` → `"error-taxonomy.md v2.36 §E-QUERY-002"`. This was the only live site in this story.

Grep variants checked across S-PRISMQL-CASE-INSENSITIVE-001:
- `error-taxonomy\.md v2\.35` — 1 hit (AC-022 body), fixed
- `error-taxonomy v2\.35` — 0 hits
- backtick-delimited variants — 0 hits
- pipe-sep table cell variants — 0 hits

**Sibling carrier story spot-check (zero stale v2.35 pins):**
- S-DEMO-FIDELITY-REMEDIATION-001: grep `error-taxonomy.*v2\.35` → 0 hits
- S-DEMO-PRISMQL-ONBOARDING-001-B: grep `error-taxonomy.*v2\.35` → 0 hits
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: grep `error-taxonomy.*v2\.35` → 0 hits

**BC regression spot-check (all 4 BC classes — zero stale pins after D-1640 closure):**
- BC-2.11.016 v1.25: stale v1.24 / v1.23 / v1.22 / v1.21 in story or error-taxonomy → 0 hits — CLEAN
- BC-2.11.017 v1.13: stale v1.12 pins → 0 hits — CLEAN
- BC-2.11.020 v1.18: stale v1.17 pins → 0 hits — CLEAN
- BC-2.11.004 v1.30: stale v1.29 pins → 0 hits — CLEAN

**BC-2.16.002 v2.08:** checked independently — no changes needed; stays v2.08. No new event_type values. SAP-1 arithmetic: 91 catalog rows match 91 `rg 'event_type\s*='` emission sites.

---

## Pass-7 Closure Re-Verification (A6 rewrite + C8 rename)

Independent re-verification of pass-7 closures at ddf852bc (fresh-context; not relying on pass-7 adversary's own verification):

**A6 rewrite (BC-2.10.011 v1.6 single-client tri-state model):** Load-bearing assertions confirmed present in `scripts/t13-preflight-audit.py` A6 body. The check FAILs on all required violation conditions per BC-2.10.011 v1.5/v1.6:
1. `capabilities` key absent or not a dict → FAIL
2. `not_registered_tools` key absent → FAIL
3. Legacy `not_implemented` key present → FAIL
4. Any capability entry missing `status` or `resolution_chain` key → FAIL
5. Any capability entry with `status` outside `{enabled, runtime_disabled, compile_time_disabled}` → FAIL

The rewrite is load-bearing per TD-VSDD-059. **CORRECT.**

**C8 rename ("SQL mode executes (ADR-052 §D4 baseline path)"):** The C8 body executes `SELECT 1` to validate SQL mode execution — the ADR-052 §D4 baseline path, not `NOW()`. G7 is the designated `NOW()` test. The title now accurately describes the body. **CORRECT.**

---

## Probe Summary

### Probe 1 — POL-25 live-currency pin sweep: error-taxonomy.md E-QUERY-038 row

Direct read of the full E-QUERY-038 row body text in error-taxonomy.md at the ddf852bc worktree state.

**Method:** Enumerate all `BC-X.NN.NNN vN.NN` anchors in the row; classify each as (a) origin pin or (b) live-currency pin; compare live-currency pins against latest BC versions.

**Result:** Three live-currency pins at v1.21 (BC current: v1.25). Eight additional anchors classified as origin pins (record the BC version that introduced each scope extension; not bumped on increments). **ADV-PR-P8-LOW-001 filed.**

### Probe 2 — SAP-1: Structured Event Catalog completeness + arithmetic

`rg 'event_type\s*=' crates/ --type rust` — 91 emission sites (same as pass-7; no change from A6/C8 Python-only edits). BC-2.16.002 v2.08 catalog: 91 rows. Arithmetic verified. **SAP-1 PASS.**

### Probe 3 — Pass-7 A6/C8 re-verification (TD-VSDD-059)

Independent re-read of A6 and C8 bodies at ddf852bc. Both verified correct (see "Pass-7 Closure Re-Verification" section above). **TD-VSDD-059 PASS.**

### Probe 4 — BC class pin sweep: all 4 BC classes across carrier stories

Spot-check of stale pins for all 4 BC classes (BC-2.11.016 v1.25 / BC-2.11.017 v1.13 / BC-2.11.020 v1.18 / BC-2.11.004 v1.30) across the 4 carrier stories after closure. Zero stale pins. **Class CLEAN.**

### Probe 5 — SAP-2 / SID-1

SAP-2: N/A — no sensor TOML spec modifications in this cascade.
SID-1: N/A — no RED Gate deferrals outstanding in this scope.

### Probe 6 — POL-16/12/13 spot-check

No new public types added (Python-only changes; Rust code UNCHANGED). Non-exhaustive gate unchanged at EXPECTED=89. **PASS.**

---

## Version Summary

**Spec changes this pass (same-burst spec-only closure):**
- error-taxonomy.md v2.35 → **v2.36** (3 live BC-2.11.016 pins bumped v1.21→v1.25; origin-pin convention note added)
- S-PRISMQL-CASE-INSENSITIVE-001 **v1.54 → v1.55** (AC-022 error-taxonomy pin v2.35→v2.36; one live site)

**No code changes. PR HEAD ddf852bc UNCHANGED.**

Versions carrying forward unchanged from D-1639:
- BC-2.11.016 v1.25
- BC-2.11.017 v1.13
- BC-2.11.020 v1.18
- BC-2.11.004 v1.30
- S-DEMO-FIDELITY-REMEDIATION-001 v2.44
- S-DEMO-PRISMQL-ONBOARDING-001-B v2.20
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.29
- BC-2.16.002 v2.08

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 7 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN(strict); streak RESET 1/3 → 0/3]** → same-burst fix pushed @ddf852bc → **PR-LEVEL pass 8 on frozen ddf852bc: 1 finding (0/0/0/1/0/0) [NOT CLEAN(strict); streak stays 0/3]** → same-burst spec-only closure (HEAD ddf852bc UNCHANGED)

**Decay signature:** 3→0→3→1→3→0→1(MED)→1(LOW). Pass-8 finding is a different severity tier (LOW vs MED) and a different document domain (error-taxonomy.md vs t13-preflight-audit.py) from pass-7. The code and spec-logic surfaces remain clean across all 8 passes (zero CRIT/HIGH code-behavior defects in the entire PR-LEVEL cascade).

**Novelty:** MEDIUM — ADV-PR-P8-LOW-001 surfaced a sweep-perimeter gap: error-taxonomy.md was systematically excluded from the BC-pin sweep template used for passes v1.22–v1.25. The origin-pin vs live-currency classification generates a reusable typology note now embedded in the error-taxonomy.md E-QUERY-038 row.

**Streak status:** 0/3 — stays 0/3. Spec-only closure; no push to branch; DRIFT-ORCH-PRLEVEL-PUSH-001 N/A. **NEXT: PR-LEVEL adversary pass 9 on SAME frozen HEAD ddf852bc** (streak candidate 1/3; NO push before pass 9 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — 91 emission sites confirmed; BC-2.16.002 v2.08 catalog complete and unchanged. No new event_type values in this pass scope.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — error-taxonomy v2.36 live-pin corrections are structural (version string updates at specific sites, not doc-comment renames). A6 rewrite (pass-7) independently re-verified load-bearing.

**TD-VSDD-060 (sibling-site sweep):** PASS — error-taxonomy v2.36 changes are confined to E-QUERY-038 row live-currency pins and the origin-pin convention note. No function signatures, constants, or canonical identifiers changed.

**BC-5.39.001 (3-CLEAN streak):** 0/3 — pass-8 result NOT CLEAN(strict). Spec-only closure; HEAD ddf852bc UNCHANGED; streak stays 0/3. Next pass (pass 9) re-gates on same frozen ddf852bc (streak candidate 1/3).
