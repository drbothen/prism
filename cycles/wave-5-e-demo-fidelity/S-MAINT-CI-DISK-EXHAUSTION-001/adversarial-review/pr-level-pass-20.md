# PR-LEVEL Pass 20 — S-MAINT-CI-DISK-EXHAUSTION-001 | adversary=vsdd-factory:adversary fresh-context diff-vs-scope-contract + duration-evidence cross-check | frozen HEAD 9c315608 | 2026-07-17 | 1 HIGH + 1 MED — streak 0/3

# Adversarial Review — PR-LEVEL Pass 20 (S-MAINT-CI-DISK-EXHAUSTION-001, PR #224)
**Finding-count:** CRIT 0 · HIGH 1 · MED 1 · LOW 0 · OBS 0 blocking

## Frozen-HEAD / PR-state verification
PR OPEN @ 9c315608 base develop; run-list 5 completed+success at HEAD (3 pull_request 29615362774/767/869 + 2 push 29615360213/280); all checks pass incl. Verify workflow structure + both Linux Test legs; diff = 2 workflow files only → SAP-1 N/A by diff-list.

## F-MAINT-P20-HIGH-001 — timeout-minutes ceilings tighter than actual frozen-HEAD green durations; sizing comments cite falsified worst-case data
Severity HIGH, confidence HIGH. ci.yml tndef job timeout-minutes: 25 (comment claims "worst-case 10.2 min ≈ 2.5×") vs ACTUAL 22m33s green at this HEAD (push run 29615360280) — ~2m27s (~10%) headroom; a slower cold-cache run false-fails. test matrix timeout-minutes: 45 (comment claims "worst 22 min ≈ 2×") vs ACTUAL x86_64-apple-darwin 35m39s and 30m13s green — worst is 35m39s, "2×" claim false, ~26% headroom; note the hang-bound rationale cites the reclaimer which does NOT run on the macOS legs closest to the ceiling. Reintroduces the flaky-failure class the story exists to eliminate. Values were human-ratified (D-1808) on a now-falsified 10.2-min basis → re-surface to human for re-validation. Routing: implementer (resize with genuine margin vs actual cold-cache worst) + product-owner (correct false comment figures) + human re-validation.

## F-MAINT-P20-MED-001 — tndef exhaustive scope-contract omits the shipped timeout-minutes keys (sibling residue of F-MAINT-P19-MED-001, which fixed only SEC-001)
Severity MED, confidence HIGH. Story §ACR (1143-1150) + §Forbidden Patterns (1209) allowlists enumerate the four v0.6 steps + neutralization + AC-006 + AC-007 + SEC-001 permissions — NOT the pass-14 job-level timeout-minutes: 25 or reclaimer step-level 10 shipped in code. Grep: timeout-minutes appears in story only at EC-001 (1216) and v0.23 changelog (1286). S-7.01 partial-fix gap. Routing: product-owner — extend both allowlist sites + propagate reclaimer timeout into AC-002 snippet, §Tasks bullets, §FSR (parallel to v0.25 SEC-001 carve-out).

## Observations (non-blocking)
AC-005 dual-reading: literal SATISFIED (5 first-attempt green runs, no re-runs, all Linux legs); distinct-trigger reading — one HEAD provides push + PR-sync = 1 event; whether one-HEAD-multi-runs satisfies "three distinct trigger events" is the open human ruling (annotated 1/3). Forward-image coupling: unconditional sources.list truncation pinned to image 20260714.240.1 per EC-016 — adequately scoped, noted only.

## Diff-vs-scope-contract
Every hunk enumerated against ratified carve-outs: all ✅ except the tndef timeout-minutes keys (→ MED-001). fmt/deny/audit untouched (prohibition honored).

## Version-pin lattice
All action SHA-pins consistent (disk-space-reclaimer @dae9fabc v1.1.2 ×2, rust-cache v2.9.1, checkout v6.0.2, dtolnay stable, setup-protoc v3.0.0; documented unused fallback jlumbroso v1.3.1). No floating refs.

## POL-22 A/C
Phase A ✅ (no fabricated anchors; behavioral_contracts [] CONFORMING per W3-FIX-CI-001). Phase C ✅ (frontmatter v0.25/ready/7 ACs/10 RGTs internally consistent; STORY-INDEX row "ready v0.25" matches; POL-13 CONSISTENT). POL-1/32/21/26/27 ✅.

## Novelty
MEDIUM-HIGH — HIGH-001 required cross-referencing sizing comments against actual per-leg run durations in the evidence (an axis no prior pass exercised); MED-001 is pass-19's own sibling-completeness residue. Apt-mirror/neutralization/RG machinery converged and correct.

## Dual Verdict
CLEAN (strict): no
CLEAN (PR-merge): no
