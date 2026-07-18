# Adversarial Review — F-ADMTOK-PR22 (Second Accelerated Re-gate)

**Namespace:** F-ADMTOK-PR22 · **Frozen HEAD:** `dac830d1` (fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001) · **Date:** 2026-07-18 · **Authority:** D-1837 human-authorized one-time second accelerated re-gate · **Scope:** delta `5c9458d6…dac830d1` (+34/−0, `crates/prism-dtu-demo-server/src/main.rs` only)

## Top-line Counts

- CRIT: 0
- HIGH: 0
- MED: 0
- LOW: 0
- OBS: 0
- PROCESS-GAP: 0

**No findings of any severity.**

## Delta-Focus Verification (NEW-001, CWE-20)

All delta-focus bullets confirmed against `/Users/jmagady/Dev/prism/.worktrees/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001/crates/prism-dtu-demo-server/src/main.rs`:

- **Guard ordering:** `if name.is_empty() { anyhow::bail!("configure: clone name must not be empty"); }` is the FIRST statement in `validate_clone_name` (lines 620-625), strictly ahead of the `name.chars().all(...)` charset check (lines 626-629). Confirmed.
- **Load-bearing test:** `test_validate_clone_name_rejects_empty` (delta lines 37-48) calls the production fn `validate_clone_name("")`, asserts `expect_err`, then asserts the error string `contains("must not be empty")`. Not tautological — it exercises the real code path and would fail if the guard were removed (vacuous-truth: `"".chars().all(pred)` returns `true`). LOAD-BEARING annotation present and correctly documents the vacuous-truth regression.
- **POL-24 byte match:** Code message `"configure: clone name must not be empty"` is byte-identical to story EC-008 v0.21 (line 338) and §Changelog v0.21 (line 443). Confirmed verbatim.
- **Net-additive:** diff is +34/−0. `sanitize_clone_name`, the charset gate, `resolve_configure_url` ordering (validate-before-URL/sidecar/tracing at line 648), AD-017 `token_present=true` placeholder, and the 10s timeout rationale are all untouched by the delta.
- **F-ADMTOK-PR21-OBS-001 closure:** `test_validate_clone_name_accepts_valid` (main.rs lines 861-877) contains exactly the as-built literals `crowdstrike`, `cyberint`, `armis`, `org-a-crowdstrike`, `org-b-cyberint`, `sensor_name-v2`, `Sensor42` — matches story EC-008 v0.21 verbatim. Prior OBS closed.

## Mandatory Axis Results

1. **Version-pin lattice — PASS.** Story frontmatter `version: "0.21"` (line 9); STORY-INDEX `version: "v2.712"` (line 4) with ADMINTOKEN row pinned **draft v0.21** (line 828); PR body pins v0.21 throughout; BC-3.6.001 `version: "0.8"`; BC-2.06.017 `version: "1.12"`; error-taxonomy `version: "2.55"`. All six pins consistent across story ↔ index ↔ BCs ↔ taxonomy ↔ PR body.
2. **Anchors vs registry (POL-4/6/7) — PASS.** `subsystems: [SS-01]` correctly anchored (SS-01 Sensor Adapters owns prism-dtu-demo-server per ARCH-INDEX v2.193:154; SS-22 re-anchor was F-ADMTOK-PR4-HIGH-001, closed v0.16). BC H1 titles match: BC-3.6.001 = "Per-Org Failure Injection"; BC-2.06.017 H1 (line 40) = "Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing" = story body table verbatim.
3. **Error-taxonomy consistency — PASS.** Empty-string `anyhow::bail!` is correctly NOT an E-DEMO code — same rationale as the charset rejection (argument-validation gate, not runtime state-resolution). E-DEMO-007 remains the sole runtime error in §DEMO (taxonomy line 606). Story EC-008 (line 338), §Changelog v0.21 (line 443), and code all agree.
4. **Mirror-table byte-identity — PASS (spot).** Delta touches no sweep tables. PR-body per-site reconciliation table (146 total: 131 same-line + 7 dynamic + 8 FidelityCheck) matches story §Root Cause table and the main.rs SWEEP-MIRROR block (counts 447/131/6/8). Unchanged by delta.
5. **Story tables vs as-built (POL-22 C) — PASS.** New guard and test resolve to real symbols at cited locations. Architecture Mapping / Purity Classification tables remain accurate (validate helpers already covered under main.rs effectful-shell rows).
6. **EC end-to-end — PASS.** EC-008 v0.21 fully verified: three locked tests enumerated (`rejects_invalid`, `accepts_valid`, `rejects_empty`) all present in source; accepts-literal list matches as-built exactly; empty-string message byte-matches. EC-001..007 unchanged by the delta.

## SAP-1, POL-22 A/C, AD-017, CI

- **SAP-1:** Delta adds zero `tracing::*!` calls and zero `event_type=` fields. No new catalog rows required. PASS.
- **POL-22 Phase A:** All cited primary artifacts read and confirmed to carry the claimed pins/titles. PASS.
- **POL-22 Phase C:** `validate_clone_name`, `sanitize_clone_name`, `test_validate_clone_name_rejects_empty`, `test_validate_clone_name_accepts_valid` all resolve to real symbols in the worktree checkout. PASS.
- **AD-017:** Delta introduces no credential logging; the empty-string guard emits no structured fields. PASS.
- **CI:** 45/45 checks green on `dac830d1` (runs 29637532367 / 29637534214 / 29637534235 / 29637534247). PASS.

Known-accepted items (bc_2_06_018_seeding Red Gate ×3, DEMO_ORG_UUID_B dead_code, DRIFT-HARNESS-ADMIN-TOKEN-CT-001 CWE-208) were not re-flagged, per instruction.

## Novelty Assessment

**Novelty: NONE.** The delta is a single, minimal, correctly-ordered empty-string guard with a genuine load-bearing regression test locking the Rust vacuous-truth semantics. It fully closes NEW-001 (CWE-20). No new attack surface, no spec drift, no anchor drift, no taxonomy drift. The prior first-re-gate findings (F-ADMTOK-PR21-OBS-001; NEW-001) are both verified FIXED, not re-flagged.

## MANDATORY DUAL VERDICT

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```

The delta `5c9458d6…dac830d1` is production-grade and merge-ready. The second accelerated re-gate is satisfied on frozen HEAD `dac830d1`. Per BC-5.39.001, note this is a delta-scoped one-time exception pass (D-1837), not a streak-advancing strict-CLEAN against a full-scope cascade; the merge gate rests on this re-gate outcome per the PR body's D-1827/D-1837 narrative.
