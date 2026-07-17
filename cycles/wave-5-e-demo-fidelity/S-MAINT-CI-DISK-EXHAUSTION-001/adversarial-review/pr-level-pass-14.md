# PR-LEVEL Adversarial Pass 14 — S-MAINT-CI-DISK-EXHAUSTION-001 (PR #224)

- **Pass:** PR-LEVEL pass-14 (fresh-context adversary, TD-VSDD-005; no prior pass reports read)
- **Date:** 2026-07-17
- **Reviewer:** adversary (fresh context)
- **Scope:** Full PR #224 diff vs develop (84062ced) + PR description claims + story spec v0.22 + POL-21/POL-22 Phase A/C + CI workflow semantic correctness
- **Story:** `.factory/stories/S-MAINT-CI-DISK-EXHAUSTION-001-ci-disk-exhaustion-hardening.md` (frontmatter `version: "0.22"` — VERIFIED)

---

## Frozen-HEAD + PR-State Verification

| Check | Expected | Observed | Status |
|-------|----------|----------|--------|
| `git rev-parse --short HEAD` (worktree) | `faf112fd` | `faf112fd` (faf112fd76cf3e3e2e403b8e3d3649aebf375819) | PASS |
| `git status --porcelain` | empty | empty | PASS |
| `gh pr view 224` state | OPEN | OPEN | PASS |
| `gh pr view 224` headRefOid | faf112fd… | `faf112fd76cf3e3e2e403b8e3d3649aebf375819` | PASS |
| Merge base vs develop | 84062ced | `84062ced90e4848b69042f1deaa3dc508d0f74d6` | PASS |
| Diff file list | ci.yml + e2e.yml only | `.github/workflows/ci.yml`, `.github/workflows/e2e.yml` (601 insertions / 25 deletions) | PASS |

No commits, pushes, or file modifications were made to the PR branch or specs during this pass (report file only, per Lesson 65).

---

## Ground-Truth Verification (POL-22 Phase A — every assertion re-executed against worktree)

All 10 story Red Gate patterns re-executed verbatim against the frozen worktree:

| RG | Pattern (target file) | Threshold | Observed | Status |
|----|----------------------|-----------|----------|--------|
| RG-1 | `^\s+- name: Report initial disk space\s*$` (ci.yml) | ≥2 | 2 | PASS |
| RG-2 | `^\s+uses: insightsengineering/disk-space-reclaimer` (ci.yml) | ≥2 | 2 | PASS |
| RG-3 | section-scoped awk `[profile.dev]` → `debug = "line-tables-only"` (.cargo/config.toml) | found | PASS | PASS |
| RG-4 | section-scoped awk `[profile.dev.package."*"]` → `debug = false` (.cargo/config.toml) | found | PASS | PASS |
| RG-5 | `^\s+if ! \( sudo apt-get update && sudo apt-get install` (ci.yml) | ≥12 | 12 | PASS |
| RG-5b | `^\s+sudo tee /etc/apt/apt-mirrors\.txt` (ci.yml) | ≥12 | 12 | PASS |
| RG-6 | `^\s+sudo apt-get install -y build-essential libc6-dev clang libclang-dev\s*$` (ci.yml) | ≥2 | 2 | PASS |
| RG-7 | `^\s+if ! \( sudo apt-get update && sudo apt-get install` (e2e.yml) | ≥1 | 1 | PASS |
| RG-7b | `^\s+sudo tee /etc/apt/apt-mirrors\.txt` (e2e.yml) | ≥1 | 1 | PASS |
| RG-8 | `^\s+- name: Neutralize apt-spy2 sources\.list rewrite` (ci.yml) | ≥2 | 2 | PASS |

Additional Phase A verifications:

- Self-match-proof claims verified for all count-based assertions: assertion lines begin `count=$(grep -cE '…` (whitespace + `count=`), so the `^\s+- name:` / `^\s+uses:` / `^\s+sudo` / `^\s+if ! \(` anchors cannot match the assertion lines themselves. Confirmed correct.
- `|| true` guard present on all 8 `grep -c` count assignments (RG-1, RG-2, RG-5, RG-5b, RG-6, RG-7, RG-7b, RG-8) — v0.21 F-MAINT-P11-LOW-004 fix present in code; zero-match cases produce `count=0` and fall through to the structured `::error::` + `exit 1` path. Fail-loud confirmed.
- verify-workflow-structure summary echo claims 22 total (20 reachability + 2 config-invariant); assertion fail-block count in the job's run block ground-truths at 22; the itemized list resolves to 12 pre-existing reachability + 8 new count-based + 2 AC-003 config-invariant = 22. Arithmetic CORRECT. Empirical: "Verify workflow structure" job PASSED on run 29595664542 (18s).
- Sweep completeness: `grep -nE 'run: sudo apt-get update && sudo apt-get install'` over ci.yml + e2e.yml = **0** residual single-attempt forms.
- 12 wrapper sites distributed across 9 ci.yml jobs (clippy 1, test 3 incl. AC-007, test-no-default-features 3 incl. AC-007, semver-checks 1, fuzz-smoke-vp021 1, perimeter-compile-fail 1, non-exhaustive-violation-compile-fail 1, no-hardcoded-sensors-compile-fail 1, shellcheck-demo-scripts 1) + 1 e2e.yml site — matches PR description claim.
- Hunk-identity spot checks: the clippy-job wrapper carries the clippy-specific Evidence comment line ("clippy is a needs: predecessor blocking pipeline") per the story's clippy snippet; the musl-tools / test-matrix libdbus wrappers carry the runs-29437306537/29438854846 evidence comment per their snippets; AC-007 steps carry the 4-line outer preamble (F-CIDISK-PR1-MED-001). Byte-form spec↔code alignment confirmed at representative sites.
- Ordering constraints verified in ci.yml: preflight BEFORE checkout (both Linux jobs); reclaimer AFTER checkout; neutralization step IMMEDIATELY after reclaimer; ≥25 GB gate after neutralization; AC-007 C toolchain BEFORE Swatinem/rust-cache restore in BOTH Linux workspace-build jobs; AC-004 annotation at job END with `if: failure() && runner.os == 'Linux'` (test matrix) / `if: failure()` (ubuntu-only tndf).
- SHA pinning: `insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2` — full-SHA pinned with version comment; no mutable refs introduced. Reclaimer inputs match spec exactly (`android/dotnet/haskell/docker-images/large-packages: true`, `swap-storage: false`), `continue-on-error: true` present on both reclaimer steps.
- Gate arithmetic: `df -P /` + `awk 'NR==2 { print int($4 / 1024 / 1024) }'` (1K-blocks → GiB) + `AVAIL_GB=${AVAIL_GB:-0}` guard — correct units, correct row, guarded against empty awk output; fails loud with actual free-GB count. AC-004 `USED_PCT` guard likewise correct.
- Fallback failure-mode: diagnostic dumps and `rm -f`/`dpkg --configure -a` are `|| true`-guarded (non-fatal by design); the retry `apt-get update` + `apt-get install` are NOT guarded — canonical-archive failure fails loud. No silent-failure pattern found in the fallback design.
- `.cargo/config.toml` ground truth: `[profile.dev] debug = "line-tables-only"` and `[profile.dev.package."*"] debug = false` present at exact line-start forms; both awk assertions PASS when executed.

## POL-22 Phase C — Named-Entity Existence

| Entity | Verified where | Status |
|--------|---------------|--------|
| W3-FIX-CI-001 | `.factory/stories/W3-FIX-CI-001-ci-wall-clock-optimization.md` (merged PR #112) | EXISTS |
| S-MAINT-REQWEST-RUSTLS-GATE-001 | `.factory/stories/S-MAINT-REQWEST-RUSTLS-GATE-001-ci-reqwest-rustls-tls-enforcement.md` | EXISTS |
| D-1780 / D-1791 / D-1795 / D-1796 / D-1797 | STATE.md / STORY-INDEX.md decision rows | EXIST |
| DRIFT-CI-STDBOOL-001 | STATE.md + SESSION-HANDOFF.md + STORY-INDEX.md | EXISTS |
| S-7.01 (Semantic Anchoring Audit gate) | Pervasive project entity: SESSION-HANDOFF.md, BC-INDEX.md, ARCH-INDEX.md, ADR-023, 29 STORY-INDEX references | EXISTS |
| EC-001..EC-016, F-CIDISK-*/F-MAINT-* IDs | internal to story §Edge Cases / §Changelog | EXIST (internally consistent) |
| CI run IDs 29394488318 / 29399778005 / 29404746333 / 29437306537 / 29438854846 / 29531645116 / 29531648104 / 29540085270 / 29541731247 / 29541732821 / 29543743638 | cited consistently across story + PR body; spot-checked evidence runs below | CONSISTENT |

POL-21: all live-body §X anchors (`§Tasks`, `§Forbidden Patterns`, `§Implementation Notes`, `§File Structure Requirements`, `§Architecture Compliance Rules`, `§Edge Cases`) resolve to actual `## §…` headings. §ACR/§FSR abbreviations appear only in historical §Changelog rows (POL-32 exempt). No phantom anchors.

## Story Governance Checks

- POL-13: STORY-INDEX row shows **ready v0.22**; story frontmatter `status: ready`, `version: "0.22"` — CONSISTENT.
- POL-32: §Changelog v0.22 → v0.1 strictly descending, no duplicates, no skips — PASS.
- `modified: "2026-07-16"` matches newest changelog row v0.22 (2026-07-16); factory commit 6a377bad authored 2026-07-16 22:32:26 -0500 — date CORRECT in commit-local time.
- Frontmatter `acceptance_criteria_count: 7` = AC-001..AC-007; `red_gate_tests: 10` = RG-1..RG-8 + RG-5b + RG-7b — both CORRECT.
- POL-6/POL-7/POL-8: `subsystems: []` + `behavioral_contracts: []` CONFORMING per PO Option-B adjudication (W3-FIX-CI-001 precedent); no BC table required; no POL-14 promotion at merge.
- §Architecture Compliance Rules carve-outs respected: fmt/deny/audit jobs untouched by diff; clippy/semver-checks/fuzz/perimeter/non-exhaustive/no-hardcoded/shellcheck jobs modified ONLY at their apt install step (wrapper + spec-mandated step rename + `if:` guard, all specified byte-exact in AC-006 snippets); test-no-default-features `PROPTEST_CASES`/`RUSTFLAGS`/test-invocation/cache lines unchanged.
- §Forbidden Patterns: `CARGO_PROFILE_DEV_DEBUG` absent from ci.yml; `swap-storage: false` retained; no floating action refs; no docker prune; no direct `rm -rf /usr/share/dotnet`.
- SAP-1: **N/A-verified-by-diff-list** — diff touches only `.github/workflows/ci.yml` and `.github/workflows/e2e.yml`; zero `crates/**/*.rs` changes; zero `event_type =` additions.
- POL-10/POL-12/POL-16: N/A (no demo-evidence requirement for this CI-toolchain maintenance story; no production Rust code; no tests outside verify-workflow-structure).

## AC Satisfaction Matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC-001 | SATISFIED | Preflight step first in both Linux jobs, before checkout; RG-1 = 2; CI green |
| AC-002 | SATISFIED | SHA-pinned reclaimer ×2 with exact inputs + `continue-on-error: true`; neutralization step ×2 (RG-8 = 2); ≥25 GB gate ×2 with correct `df -P` arithmetic + guard |
| AC-003 | SATISFIED | Section-scoped awk assertions 3+4 in verify-workflow-structure; ground-truthed PASS against `.cargo/config.toml`; no `CARGO_PROFILE_DEV_DEBUG` anywhere in ci.yml |
| AC-004 | SATISFIED | `if: failure()` annotation at END of both Linux jobs; USED_PCT guard; ::warning:: with evidence-run citation |
| AC-005 | **PENDING-CI (2/3 green + run-3 in progress)** | See below |
| AC-006 | SATISFIED | 12 ci.yml wrapper sites + 1 e2e.yml site; five-operation fallback byte-form; RG-5/5b/7/7b at threshold; 0 residual single-attempt forms |
| AC-007 | SATISFIED | C toolchain step ×2, before rust-cache restore in both jobs, via AC-006 wrapper; RG-6 = 2 |

### AC-005 run-3 CI evidence status

| Run | Event | HEAD | Status |
|-----|-------|------|--------|
| 29544970679 | push | faf112fd | completed / **success** (2026-07-17T00:30:27Z) |
| 29544972231 | pull_request | faf112fd | completed / **success** (2026-07-17T00:30:29Z) |
| 29595664542 (run-3, PR-reopen trigger 2026-07-17) | pull_request | faf112fd | **in_progress** (created 2026-07-17T16:18:47Z; Verify-workflow-structure/fmt/shellcheck/perimeter-sync/deep-recursion/WASM32 already PASS; clippy + Test matrix pending) |

Per dispatch instruction, run-3 in-progress is recorded as **PENDING-CI**, not a finding. All three run IDs are distinct GitHub Actions runs on the frozen HEAD faf112fd triggered by distinct events (push, pull_request.synchronize, pull_request.reopened) — consistent with the F-MAINT-P10-MED-004 / OBS-008 valid-run adjudication.

---

## Findings

### F-MAINT-P14-LOW-001 — PR #224 description carries three stale/incorrect claims (LOW)

- **File+anchor:** PR #224 description — header paragraph ("Story v0.21"), Architecture Changes ("18 commits over develop@84062ced"), Adversarial Review §PR-LEVEL Cascade table (ends "PR-12 … PENDING").
- **Description:** (a) The description states "Story v0.21"; the story is at **v0.22** (spec-only recon fix-burst, D-1797, factory commit 6a377bad). (b) The description claims "18 commits over develop@84062ced"; ground truth is **17** (`git rev-list --count 84062ced..HEAD` = 17; GitHub API `commits | length` = 17). (c) The PR-LEVEL cascade table stops at "PR-12 PENDING", while STORY-INDEX D-1796/D-1797 record pass-12 CLEAN(strict) 1/3 and pass-13 (2 LOW + 1 OBS spec-side, streak reset 0/3) as already completed.
- **Evidence:** `gh pr view 224 --json body`; `git rev-list --count 84062ced..HEAD` → 17; `gh pr view 224 --json commits -q '.commits | length'` → 17; story frontmatter `version: "0.22"`; STORY-INDEX line 812 + v2.696 changelog row.
- **Impact:** PR description is a reviewed claim surface (AC-005 evidence recording lives here); stale/incorrect metadata degrades audit fidelity. No code impact.
- **Proposed routing:** pr-manager — description-only refresh (does not change HEAD; no frozen-HEAD reset).

### F-MAINT-P14-LOW-002 — AC-002 residual prose contradicts the v0.20 step ordering and the ratified gate form (LOW)

- **File+anchor:** story §Acceptance Criteria → AC-002, sentence "A verification step immediately after the reclaim step confirms at least 25 GB free via `df -h`." (immediately before the gate snippet).
- **Description:** Two imprecisions in one v0.1-era sentence that v0.20/v0.4 sweeps did not update: (a) since v0.20, the "Neutralize apt-spy2 sources.list rewrite" step sits BETWEEN the reclaimer and the gate — AC-002's own later text mandates "Both neutralization steps MUST appear immediately after the reclaimer step and before the ≥25 GB gate", so the gate is no longer "immediately after the reclaim step"; internal contradiction within the same AC. (b) "via `df -h`" — the ratified gate (v0.4 F-CIDISK-P1-LOW-001) checks via `df -P /` 1K-block arithmetic; `df -h` appears only in the failure diagnostic branch. The implemented ci.yml follows the correct (later, more-specific) ordering and form — this is a spec-prose-only defect (POL-25-class propagation miss).
- **Evidence:** story lines ~295-297 vs lines ~233-288 (neutralization ordering mandate) and the gate snippet itself (`df -P /`); ci.yml implements reclaimer → neutralize → gate.
- **Proposed routing:** product-owner — spec-only sentence fix + POL-29 sibling sweep of "immediately after the reclaim step" phrasing.

### F-MAINT-P14-LOW-003 — EC-001 "hangs" coverage claim not delivered by the mechanism; reclaimer hang is unbounded below the 360-min default (LOW)

- **File+anchor:** story §Edge Cases EC-001 (description column: "action fails **or hangs**"; expected-behavior column: "`continue-on-error: true` — the job proceeds to the ≥25 GB gate regardless of reclaimer exit code").
- **Description:** `continue-on-error: true` only governs a step's failure *outcome*; a **hung** reclaimer produces no outcome, the step never completes, the ≥25 GB gate never runs, and the job stalls until the GitHub default 360-minute job timeout (the `test` and `test-no-default-features` jobs carry no `timeout-minutes` — pre-existing). EC-001's expected behavior is therefore semantically incorrect for the hang half of its own description. The failure mode is rare and eventually visible (job timeout failure), so impact is bounded — but the spec claim is falsifiable as written. Fix options: (a) amend EC-001 to scope the continue-on-error mitigation to exit-code failures and document the 360-min default bound for hangs; and/or (b) ratify a step-level `timeout-minutes:` on the two reclaimer steps (step-level timeout + continue-on-error together WOULD deliver the claimed behavior; note this is a code change → frozen-HEAD streak reset, and requires a carve-out/snippet amendment since the reclaimer step form is spec-governed byte-exact).
- **Evidence:** ci.yml jobs `test` (line 78) and `test-no-default-features` (line 329) have no `timeout-minutes`; reclaimer steps have no step-level `timeout-minutes`; GitHub Actions semantics: continue-on-error does not cancel a running step.
- **Proposed routing:** product-owner (EC-001 wording) + orchestrator adjudication on whether step-level timeout hardening is taken in-scope now (code push) or ratified as a documented bound (spec-only).

---

## Adjudicated no-action items independently re-verified (not findings)

- Reclaimer `continue-on-error` masking of exit-code failures: EC-009 / F-CIDISK-PR3-OBS-004 ratified trade-off; ≥25 GB gate is the load-bearing authoritative check — confirmed present and correct in both jobs.
- AC-007 step-position asymmetry between the two Linux jobs: within the spec's ordering contract (F-CIDISK-PR3-OBS-003); both instances verified before rust-cache restore.
- AC-006 ≥12 threshold exactness: BY DESIGN (F-CIDISK-PR3-OBS-002); observed count is exactly 12.
- `http://archive.ubuntu.com/ubuntu/` at priority:1 in the fallback: plain-HTTP apt is signature-verified (Release/InRelease GPG); https mirrors at priority 2/3; acceptable.
- Step renames + `if: runner.os == 'Linux'` additions in the seven carve-out jobs: mandated byte-exact by the AC-006 snippets themselves; not a carve-out violation.
- "core.sh ~line 112" citation: pinned to immutable SHA dae9fabcb8 — not a TD-VSDD-091 volatile pin.
- Two AC-005 evidence runs sharing one push (push + pull_request events): distinct run IDs from distinct GitHub events on distinct runner allocations; consistent with the F-MAINT-P10-OBS-008 adjudication text.

---

## Verdict

```
CLEAN (strict): no   [3 findings: 0 CRIT / 0 HIGH / 0 MED / 3 LOW / 0 OBS / 0 PROCESS-GAP]
CLEAN (PR-merge): yes [zero CRIT+HIGH+MED]
```

**BC-5.39.001 streak impact:** any finding resets/holds the streak — streak remains **0/3** on frozen faf112fd. All three findings are spec-prose or PR-description layer; none require a code push to close (F-MAINT-P14-LOW-003 option (b) would push code and reset the frozen HEAD — orchestrator's call).

**AC-005 run-3:** PENDING-CI — run 29595664542 in progress on faf112fd at pass time (2/3 green: 29544970679 push + 29544972231 pull_request, both success on faf112fd).
