# canonical pass-17 | adversary=vsdd-factory:adversary fresh-context convergence-candidate | frozen HEAD 828449de | 2026-07-17 | 1 LOW — streak RESET 0/3 | CLEAN PR-merge yes

# Adversarial Review — PR-LEVEL Pass 17 (PR #225, DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001)
**Finding count:** CRIT 0 · HIGH 0 · MED 0 · LOW 1 · OBS 0 · PROCESS-GAP 0

## Verification Statement
Fresh-context at frozen HEAD 828449de: story v0.18 (476 lines), PR body, taxonomy §DEMO v2.55, both BCs, BC-INDEX rows, ARCH-INDEX SS-01/SS-22, STORY-INDEX v2.707, as-built code (cmd_configure, resolve_configure_token, write_token_sidecar_to_path, KillGuard, helpers/mod.rs). All six mandatory axes + fresh probes.

## F-ADMTOK-PR17-LOW-001 — EC-006 mis-attributes a server-origin 401 to the "AC-003 flow" (should be EC-001)
Severity LOW, confidence MEDIUM. Story §Edge Cases EC-006 (line 336): "stale-token 401 from server is surfaced (existing AC-003 flow)". AC-003 (247-258) is exclusively the PRE-POST path (E-DEMO-007, item 3: "NOT proceed to POST"). In EC-006 the token IS resolved (stale-but-present) → cmd_configure POSTs (main.rs:685) → server 401 → surfaced by the non-2xx print-and-exit block (700-705) — precisely the EC-001 mechanism (331), not AC-003. Mutually-exclusive paths. Survived because the v0.18 flow-precision burst swept EC-003/004/005 but not sibling EC-006 (S-7.01 gap; blast radius 1 cell). Fix: replace "(existing AC-003 flow)" with "(existing EC-001 non-2xx status-print-and-exit flow)". Descriptive-accuracy only; code path correct. Routing: product-owner.

## Observations (non-defect)
PR body "KillGuard RAII + disarm-after-wait": code has no explicit disarm flag — guard always SIGKILLs, relies on ESRCH harmless post-wait (td_wv1_04:148-158). Behavior correct; wording loose but defensible. Not a finding.

## Axis Results
1 lattice PASS (story v0.18 == v2.707 row; BC pins agree everywhere; taxonomy v2.55). 2 anchors PASS (SS-01 @154; SS-22 @175 correctly excluded). 3 taxonomy PASS (carve-out coherent with code 1015-1127). 4 mirror-table PASS (header 267==613; row byte-consistent with 621; template matches e_demo_007 @1024-1028). 5 story tables PASS (BC titles match H1s + BC-INDEX; Architecture/FSR/Purity match as-built; SWEEP-MIRROR counts mirror main.rs:618-628). 6 EC end-to-end: 1 LOW (EC-006); EC-003/004/005 caveats verified accurate; EC-001/002/007 accurate (UUID v4 confirmed crowdstrike state.rs:415 + 13 siblings).

## SAP-1
PASS — zero event_type; new debug! carries clone + token_present=true only; AD-017 preserved.

## POL-22 (registry-text)
Phase A PASS; Phase C PASS (all structural tables match as-built; helpers/mod.rs stale reference confirmed removed, grep zero hits).

## CI Status
44/44 pass; all 5 runs success at 828449de.

## Novelty Assessment
LOW — single within-story cross-reference imprecision, direct sibling-sweep gap from the v0.18 burst; narrow blast radius; all other axes clean and coherent; implementation production-grade and test-locked.

## Dual Verdict
CLEAN (strict): no
CLEAN (PR-merge): yes
LOW resets 3-CLEAN(strict) streak to 0/3; does not block PR-merge threshold. Routing: product-owner one-line EC-006 edit; streak re-gates on corrected story.
