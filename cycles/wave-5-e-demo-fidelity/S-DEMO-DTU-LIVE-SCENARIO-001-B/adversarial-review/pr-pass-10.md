---
document_type: adversarial-review-pass
pass: 10
scope: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: bc0f36c5
date: 2026-06-12
clean_strict: YES
clean_pr_merge: YES
streak: "2/3"
findings_count: 0
---

# PR-LEVEL Adversarial Pass 10 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Result: CLEAN(strict)=YES; CLEAN(PR-merge)=YES. Streak 2/3.**

## Summary

ZERO findings of any severity. Pass 10 is the second consecutive CLEAN(strict) pass in the
PR-LEVEL cascade. No code changes since pass 4 (HEAD bc0f36c5 unchanged). All novel
verification angles probed and verified clean.

## Verification Axes

| Axis | Result | Notes |
|------|--------|-------|
| BC-INDEX row 119 (BC-2.06.019) annotation current | PASS | `ready v2.9` — matches story B current version |
| BC-INDEX row 120 (BC-2.06.020) annotation current | PASS | `ready v2.9 (D-1114 2026-06-12)` — corrected D-1114 |
| BC-2.06.019 v1.7 Route Coverage Table (8 rows, EXHAUSTIVE) | PASS | All 8 rows verified vs PR diff; claroty/alerts.rs EXEMPT on real-API grounds; no gaps |
| BC-2.06.020 v1.2 invariants | PASS | Enrichment correlation BC invariants consistent with implementation |
| E-DEMO-006 verbatim taxonomy↔BC↔code | PASS | `org_id_missing` error taxonomy entry matches BC-2.06.019 PRE-6 and code guard |
| SAP-1 (tracing event_type catalog) | PASS | No new `event_type =` emissions in PR diff; catalog current |
| SAP-2 (DTU↔TOML schema parity) | N/A | No sensor TOML modifications in PR diff |
| Forbidden-pattern sweep | PASS | No `unwrap()`/`println!`/`Client::new()` without timeout/retired ColumnType variants |
| DormantTenant Red Gate test 17 | PASS | Guard logic confirmed load-bearing; test verifies org_id isolation |
| Demo evidence 18/18 ACs | PASS | All 18 acceptance criteria covered by demo evidence at docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/ |
| Frontmatter-body coherence | PASS | BC-2.06.019 + BC-2.06.020 frontmatter changelog rows consistent with body content |
| Story B HEAD = remote | PASS | bc0f36c5 = remote; no code change since pass 4; PR diff identical |
| BC pin consistency (story B) | PASS | BC-2.06.019 v1.7 + BC-2.06.020 v1.2 pins consistent across story body and BC frontmatter |
| Cross-record index consistency | PASS | BC-INDEX, STORY-INDEX, VP-INDEX, ARCH-INDEX all consistent |
| BC-5.39.001 convergence criteria | PASS | CLEAN(strict): zero findings any severity; CLEAN(PR-merge): zero CRIT+HIGH+MED |

## Novel Angles Probed (Pass 10)

**Angle 1 — Scenario-state concurrency (Arc<IncidentTimeline>, no Mutex, pure-function engine)**

The scenario engine uses `Arc<IncidentTimeline>` for shared timeline state across concurrent
sensor adapters. Verified the pure-function architecture: `IncidentTimeline` is constructed
once per scenario initialization and treated as read-only during query execution. No shared
mutable state across concurrent fan-out tasks. No `Mutex<IncidentTimeline>` is needed or
present. The Arc serves reference-counted lifetime management only, not shared mutation.

Result: PASS — concurrency model is correct; no data race potential; consistent with
ADR-022 Arc-DI pattern.

**Angle 2 — Repeated-construction determinism (secondary RNG stream)**

Verified that repeated construction of the scenario engine with identical `(seed, archetype,
org_id, Arc<timeline>, time_anchor)` arguments produces bit-identical output. The generator
uses a seeded RNG with a deterministic secondary stream for synthetic field augmentation
(e.g., `_ioc_value` sentinel stamps). Confirmed no use of `thread_rng()` or wall-clock seeding
in the production code path. ADR-036 time_anchor 5-arg constructor is the canonical entry
point.

Result: PASS — determinism invariant holds; same seed + clock-offset → same timeline across
invocations (BC-2.06.019 PC-3).

**Angle 3 — Stage-boundary saturation arithmetic vs ADR-036/TV vectors**

Examined stage transition logic at boundaries: stage 0→1 (recon→lateral), 1→2
(lateral→exfil), 2→3 (exfil→containment). Verified that saturation arithmetic prevents
integer overflow when `time_anchor` advances past the final stage boundary. Checked against
the test vectors in ADR-036 §Test Vectors: boundary conditions represented and passing.
No off-by-one in `stage_idx` computation relative to stage duration windows.

Result: PASS — saturation arithmetic correct; ADR-036 test vectors match implementation.

**Angle 4 — Cargo.lock unification (chrono only, workspace-resolved)**

Verified that `chrono` appears only once in Cargo.lock — workspace-level resolution per
ADR-022 single-workspace MSRV discipline. No duplicate `chrono` entries at conflicting
patch versions. Story B did not introduce a new per-crate chrono pin that would fragment
the workspace-level resolution.

Result: PASS — Cargo.lock clean; single chrono version; no workspace fragmentation.

**Angle 5 — Required-features test registration and isolation**

DTU-conditional tests under `[[test]]` required-features directives (closed BPRL-P9-01 /
F-P10-01 in LOCAL cascade) verified correctly registered. Tests that require the `dtu-ext`
feature flag are gated; `cargo nextest run` without the feature does not attempt to compile
or link those tests. No harness test that spans a DTU boundary runs without the feature.

Result: PASS — required-features gates correctly isolate DTU-conditional tests; no CI
false-failure risk.

**Angle 6 — Rustdoc text-fenced non-doctest**

Story B introduces several `/// ```text` (non-doctest) rustdoc blocks. Verified all
text-fenced blocks are marked `text` or `ignore`, not bare ` ``` ` (which would be
interpreted as doctests and fail to compile due to missing imports). No bare code fences
present in added rustdoc.

Result: PASS — all new rustdoc code blocks correctly fenced; no unintended doctest
compilation attempted.

## Do-Not-Reflag Confirmation

All items on the pass-10 do-not-reflag list (BPRL-P1 through BPRL-P9, plus all LOCAL
closure items) are confirmed STILL CLOSED. No regression detected.

## Conclusion

PR #185 at HEAD bc0f36c5 is production-grade and fully spec-sanctioned. All novel angles
probed clean. Streak advances to 2/3 under BC-5.39.001.

**NEXT: PR-LEVEL pass 11 (convergence pass — if CLEAN(strict)=YES → streak 3/3 → post-convergence sequence: pr-reviewer APPROVE → security-reviewer MAY PROCEED → CI verify → squash-merge → post-merge burst).**
