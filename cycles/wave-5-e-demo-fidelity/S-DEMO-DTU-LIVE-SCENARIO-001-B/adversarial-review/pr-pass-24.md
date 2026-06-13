---
document_type: adversarial-review-pass
pass: 24
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 15bedc12
diff: changed (15bedc12 = 1 commit ahead of 0863184a — threatintel test perimeter-claim comment corrected)
date: 2026-06-13
clean_strict: NO
clean_pr_merge: YES
streak_before: 1/3
streak_after: 0/3
findings: 1
novelty: LOW
finding_ids: [BPRL-P24-01]
recorded_by: state-manager (D-1131)
user_decision: prose-correction (structural enforcement); NOT gate-build
---

# PR-LEVEL Adversarial Pass 24 — S-DEMO-DTU-LIVE-SCENARIO-001-B

## Result

**CLEAN(strict): NO** (1 LOW [process-gap] finding)
**CLEAN(PR-merge): YES** (zero CRIT/HIGH/MED findings)
**Streak RESET: 1/3 → 0/3**
**Findings: 1 (BPRL-P24-01 LOW)**
**Novelty: LOW**

## Finding

### BPRL-P24-01 — LOW [process-gap] — False-coverage: INV-PERIMETER-COMPLIANCE-001 enforcement-mechanism claim

**Severity:** LOW  
**Class:** process-gap (false-coverage claim — enforcement mechanism cited does not cover the cited target)  
**Scope:** SPEC-ONLY; code at 0863184a is correct  

**Description:**

Multiple artifacts cited `tests/external/perimeter-violation/` as an enforcement mechanism for `INV-PERIMETER-COMPLIANCE-001` (DTU perimeter: `prism-dtu-threatintel` and `prism-dtu-nvd` must not import `prism-spec-engine`, `prism-sensors`, or `prism-query`). This claim was false:

- `tests/external/perimeter-violation/` is the compile-fail gate established by S-PLUGIN-PREREQ-A (BC-2.11.006) — it covers the **prism-query pub-API perimeter only**. The crate depends on `prism-query` + `prism-core` and contains zero dependency on or knowledge of any `prism-dtu-*` crate.
- The DTU perimeter (`INV-PERIMETER-COMPLIANCE-001`) is enforced **structurally**: `prism-dtu-threatintel/Cargo.toml` and `prism-dtu-nvd/Cargo.toml` declare no dependency on the forbidden crates, so any forbidden `use` statement is an ordinary E0432 compile error caught by the standard workspace build. No separate compile-fail gate is needed or exists for this purpose.

**Affected surfaces (all claiming the perimeter-violation gate enforces the DTU perimeter):**

1. **AC-016 body** in story B v2.15 — stated "The compile-fail gate at `tests/external/perimeter-violation/` enforces..." but the gate covers `prism-query` only.
2. **BC-2.06.020 `INV-PERIMETER-COMPLIANCE-001` body** — Architecture Anchors bullet for `tests/external/perimeter-violation/` described its role ambiguously (implied DTU enforcement).
3. **BC-2.06.020 Architecture Anchors** — `tests/external/perimeter-violation/` bullet lacked explicit scope delimitation.
4. **Threatintel test** `test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate` — RGT row 16 in story B listed crate column as `unit+compile-fail` with an implied reference to the perimeter-violation gate; the test does not actually invoke that gate (and cannot — the gate is a separate crate that does not depend on DTU crates).

**The invariant SEMANTICS are correct.** The DTU perimeter IS held — structurally, via Cargo/E0432. The false claim was in the prose describing the enforcement mechanism, not in the enforcement itself.

**User decision:** Correct the prose to accurately describe structural Cargo/E0432 enforcement. Do NOT build a new compile-fail gate for the DTU perimeter — Cargo dependency declarations ARE the enforcement mechanism (the DTU perimeter is invisible to ordinary compilation only for the `prism-query` pub-API perimeter where a library's Cargo.toml correctly declares the dep but the usage pattern violates the API boundary — the DTU situation is different; the forbidden dep is simply absent from Cargo.toml, making any violation a standard E0432).

## Closure

**Implementer** (feature-branch commit `15bedc12`):

- Corrected `test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate` test comment in `crates/prism-dtu-threatintel/tests/...` — removed claim that the perimeter-violation gate enforces the DTU perimeter; replaced with structural Cargo/E0432 framing. Test name kept. Passthrough assertions unchanged.
- `just check` PASS at `15bedc12`.

**Product-Owner** (BC-2.06.020 v1.5→v1.6):

- `INV-PERIMETER-COMPLIANCE-001` body corrected: explicit statement that the `tests/external/perimeter-violation/` gate covers `prism-query` pub-API perimeter (BC-2.11.006) ONLY and does NOT reference DTU crates; DTU perimeter is Cargo-structural.
- Architecture Anchors `tests/external/perimeter-violation/` bullet corrected: describes its actual scope (prism-query only) and explicitly states that the DTU perimeter is Cargo-structural.
- Invariant SEMANTICS (no forbidden imports in threatintel/nvd) UNCHANGED.

**Story-writer** (story B v2.15→v2.16):

- AC-016 prose corrected: structural Cargo enforcement described accurately; `tests/external/perimeter-violation/` correctly scoped to `prism-query` perimeter (BC-2.11.006) only and noted as unrelated to DTU perimeter.
- Architecture Compliance row for perimeter corrected.
- Phase-6 gate item for DTU perimeter check corrected (structural `cargo build` framing, not compile-fail gate reference).
- RGT row 16 `crate` column: removed `+compile-fail` annotation (the test is unit only; structural perimeter verified separately by `cargo build`); `type` column: `unit+compile-fail` → `unit`.
- BC-2.06.020 pin v1.5→v1.6.

**PIVOT-003** (v1.7→v1.8):

- BC-2.06.020 pin v1.5→v1.6 at §Behavioral Contracts BC table row and §Token Budget row.

**Feature HEAD → `15bedc12`** (= remote after push). Diff changed by comment fix.

## Clean Angles Confirmed (re-verified at 0863184a, still valid)

The adversary independently confirmed these angles remain closed at the diff state entering pass 24:

1. **Feature-graph closure** — D-1117 AC-019 end-to-end Cyberint CVE↔NVD pivot chain: `CyberintClone::new_with_scenario` 6-arg wiring; `catalog.device_cves` cyclic assignment; `NvdState::lookup_and_count` resolves all catalog CVEs. PASS.
2. **SAP-1** — no new `event_type =` values in diff without BC-2.16.002 catalog rows. PASS.
3. **Structural perimeter (INV-PERIMETER-COMPLIANCE-001)** — `prism-dtu-threatintel/Cargo.toml` and `prism-dtu-nvd/Cargo.toml` declare no dependency on `prism-spec-engine`, `prism-sensors`, or `prism-query`. The invariant IS held. The false-coverage finding was in the prose description of HOW it is held, not in whether it is held.
4. **All BPRL-P1..P23 + DRIFT-1/2/3 closures** — re-confirmed at 0863184a; all still closed at 15bedc12 (the comment fix does not touch any of those surfaces).

## Convergence

CLEAN(strict)=NO — streak RESET 1/3→0/3.  
CLEAN(PR-merge)=YES.  
User directed: prose-correction over gate-build. Feature HEAD advanced to `15bedc12`.  
**PR-LEVEL pass 25 NEXT** at HEAD `15bedc12` — re-materialize diff (`gh pr diff 185`; diff changed by commit `15bedc12`; do NOT reuse `/tmp/pr185-pass20.diff`).

## Do-Not-Reflag (carry forward + new addition)

BPRL-P24-01 CLOSED — DTU perimeter (`INV-PERIMETER-COMPLIANCE-001`) is enforced STRUCTURALLY via Cargo/E0432: `prism-dtu-threatintel/Cargo.toml` and `prism-dtu-nvd/Cargo.toml` declare no dependency on the forbidden crates; any violation is a standard E0432 compile error in the workspace build. The `tests/external/perimeter-violation/` compile-fail gate covers the **prism-query pub-API perimeter only** (BC-2.11.006) and correctly does NOT reference DTU crates. **DO NOT re-raise "DTU perimeter gate missing", "no compile-fail test for DTU perimeter", or "tests/external/perimeter-violation/ should cover DTU crates" — user ratified prose-correction over gate-build; structural Cargo enforcement is adequate for this perimeter.**
