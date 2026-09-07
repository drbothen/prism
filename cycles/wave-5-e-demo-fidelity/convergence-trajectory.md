---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-08-19T00:30:00Z
cycle: wave-5-e-demo-fidelity
cascade: OCSF-correctness-claroty-SPEC-adversary
inputs: [STATE.md, decisions-archive-D1789-D2199.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Convergence Trajectory — wave-5-e-demo-fidelity

## OCSF-Correctness Claroty SPEC Adversary Cascade

Cascade started: 2026-08-16 (D-2201 code defect registered in `class_selector.rs`; story decomposition D-2203 produced ROUTING-001 + COERCION-001 + 3 OCSF-FIDELITY stubs).
Current status (as of D-2244 wrap): IN PROGRESS — pass-45 complete; BC-5.39.001 streak 0/3; pass-46 pending.

## Finding Progression

| Pass | Date | Findings | HIGH | MED | LOW | OBS | Streak | Verdict |
|------|------|----------|------|-----|-----|-----|--------|---------|
| 1 | 2026-08-16 | — | — | — | — | — | 0/3 | FIX-BURST D-2205 |
| 2 | 2026-08-16 | — | — | — | — | — | 0/3 | FIX-BURST D-2206 |
| 3 | 2026-08-16 | — | — | — | — | — | 0/3 | FIX-BURST D-2207 |
| 4 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2208 |
| 5 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2209 |
| 6 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2210 |
| 7 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2211 |
| 8 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | 1/3 | CLEAN(strict)=YES D-2212 |
| 9 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | 2/3 | CLEAN(strict)=YES D-2213 |
| 10 | 2026-08-17 | — | — | — | — | — | 0/3 RESET | FIX-BURST D-2214 |
| 11 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | 1/3 | CLEAN(strict)=YES D-2215 |
| 12 | 2026-08-17 | — | — | — | — | — | 0/3 RESET | FIX-BURST D-2216 |
| 13 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | — | CLEAN(strict)=YES D-2217 |
| 14 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | 2/3 | CLEAN(strict)=YES D-2217 |
| 15 | 2026-08-17 | — | — | — | — | — | 0/3 RESET | records micro-burst D-2218 |
| 16 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2219 |
| 17 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2220 |
| 18 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2221 |
| 19 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | 1/3 | CLEAN(strict)=YES D-2222 |
| 20 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | 2/3 | CLEAN(strict)=YES D-2223 |
| 21 | 2026-08-17 | — | — | — | — | — | 0/3 RESET | FIX-BURST D-2224 |
| 22 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2225 |
| 23 | 2026-08-17 | — | — | — | — | — | 0/3 | records micro-burst D-2226 |
| 24 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2227 |
| 25 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2229 |
| 26 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2230 |
| 27 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2231 |
| 28 | 2026-08-17 | — | — | — | — | — | 0/3 | FIX-BURST D-2232 |
| 29 | 2026-08-17 | 0 | 0 | 0 | 0 | 0 | 1/3 | CLEAN(strict)=YES |
| 30 | 2026-08-17 | 1 | 0 | 1 | 0 | 0 | 0/3 RESET | FIX-BURST D-2233 |
| 31 | 2026-08-18 | 1 | 0 | 0 | 1 | 0 | 0/3 | records micro-burst D-2234 |
| 32 | 2026-08-18 | 2 | 0 | 2 | 0 | 0 | 0/3 | FIX-BURST D-2235 |
| 33 | 2026-08-18 | 1 | 0 | 1 | 0 | 0 | 0/3 | FIX-BURST D-2238 (F-P33-MED-001) |
| 34 | 2026-08-18 | 4 | 0 | 0 | 0 | 0 | 0/3 | FIX-BURST D-2239 |
| 35 | 2026-08-18 | 0 | 0 | 0 | 0 | 0 | 1/3 | CLEAN(strict)=YES |
| 36 | 2026-08-18 | 1 | 0 | 0 | 1 | 0 | 0/3 RESET | records micro-burst D-2240 (F-P36-LOW-001) |
| 37 | 2026-08-18 | 0 | 0 | 0 | 0 | 0 | 1/3 | CLEAN(strict)=YES |
| 38 | 2026-08-18 | 0 | 0 | 0 | 0 | 0 | 2/3 | CLEAN(strict)=YES |
| 39 | 2026-08-18 | 1 | 0 | 0 | 1 | 0 | 0/3 RESET | records micro-burst D-2241 (F-P39-LOW-001) |
| 40 | 2026-08-18 | 1 | 1 | 0 | 0 | 0 | 0/3 | FIX-BURST D-2242 |
| 41 | 2026-08-18 | 2 | 0 | 2 | 0 | 0 | 0/3 | FIX-BURST D-2242 |
| 42 | 2026-08-18 | 2 | 1 | 0 | 1 | 0 | 0/3 | FIX-BURST D-2242/D-2243 |
| 43 | 2026-08-18 | 2 | 1 | 1 | 0 | 0 | 0/3 | FIX-BURST D-2243 (F-P43-HIGH-001, F-P43-MED-001) |
| 44 | 2026-08-18 | 1 | 0 | 0 | 0 | 1 | 0/3 | FIX-BURST D-2243 (F-P44-OBS-001) |
| 45 | 2026-08-18 | 1 | 0 | 1 | 0 | 0 | 0/3 | FIX-BURST pending (F-P45-MED-001) |

_Note: Passes 1–28 finding counts not granularly recorded in decisions log — existence of fix-bursts confirms findings were present. Pass 34: 4 findings per Phase Progress shorthand; severity breakdown not separately recorded (included LOW/OBS per D-2239)._

## Trajectory Shorthand

`→p8(C1/3)→p9(C2/3)→p10(R)→p11(C1/3)→p12(R)→p13(C)→p14(C2/3)→p15(R)→…→p19(C1/3)→p20(C2/3)→p21(R)→…→p29(C1/3)→p30(1M)→p31(1L)→p32(2M)→p33(1M)→p34(4)→p35(C1/3)→p36(1L)→p37(C1/3)→p38(C2/3)→p39(1L)→p40(1H)→p41(2M)→p42(1H+1L)→p43(2)→p44(1O)→p45(1M)`

C=CLEAN, R=RESET, H=HIGH, M=MED, L=LOW, O=OBS

## Frozen Perimeters at Key Streak Points

| Event | After Pass | Perimeter |
|-------|------------|-----------|
| Streak 1/3 | p8 | ADR-058 v2.6 / BC-2.16.003 v1.9 / BC-2.16.002 v2.27 / ROUTING-001 v1.11 / COERCION-001 v1.10 |
| RESET (p10) | p10 | D-2214 — streak 2/3→0/3 |
| Streak 1/3 | p11 | ADR-058 v2.7 / BC-2.16.003 v1.9 / BC-2.16.002 v2.27 / ROUTING-001 v1.14 / COERCION-001 v1.13 |
| RESET (p12) | p12 | D-2216 — streak 1/3→0/3 |
| Streak 2/3 | p13+14 | ADR-058 v2.13 / BC-2.16.003 v1.9 / BC-2.16.002 v2.27 / ROUTING-001 v1.16 / COERCION-001 v1.15 |
| RESET (p15) | p15 | D-2218 records micro — streak 2/3→0/3 |
| Streak 2/3 | p19+20 | ADR-058 v2.13 / BC-2.16.003 v1.10 / BC-2.16.002 v2.27 / ROUTING-001 v1.19 / COERCION-001 v1.18 |
| RESET (p21) | p21 | D-2224 — streak 2/3→0/3 |
| Streak 1/3 | p29 | ADR-058 v2.16 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.28 / COERCION-001 v1.28 (approx, post-D-2232) |
| RESET (p30) | p30 | D-2233 — streak 1/3→0/3 |
| Streak 1/3 | p35 | ADR-058 v2.18 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.32 / COERCION-001 v1.31 |
| RESET (p36) | p36 | D-2240 records micro — streak 1/3→0/3 |
| Streak 2/3 | p37+38 | ADR-058 v2.19 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.33 / COERCION-001 v1.32 |
| RESET (p39) | p39 | D-2241 records micro — streak 2/3→0/3 |
| Current frozen | post-p45 | ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34 |

## Per-Pass Details (key passes)

### Pass 8 (2026-08-17) — First CLEAN

**Verdict:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES
**Streak:** 1/3
**Decision:** D-2212

### Pass 9 (2026-08-17) — Second CLEAN

**Verdict:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES
**Streak:** 2/3
**Decision:** D-2213

### Pass 10 (2026-08-17) — First Reset

**Verdict:** FINDINGS — streak RESET 2/3→0/3
**Fix-burst:** D-2214 — ROUTING-001 v1.13→v1.14; COERCION-001 v1.12→v1.13

### Pass 29 (2026-08-17) — Highest Streak Before Disruption

**Verdict:** CLEAN(strict)=YES
**Streak:** 1/3 (after 28-pass fix-burst sequence)
**Note:** DTU-parity scope removed from cascade scope at D-2228; verify §Authority ADR-058 cite only for SAC-2 link validity.

### Pass 33 (2026-08-18) — F-P33-MED-001

**Verdict:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO
**Findings:** 1 MED — `pipeline_result_to_record_batch` §D1 param-threading: `sensor_spec` undefined free variable in ROUTING-001 §AC-012; ADR-058 §D1/§I1 inaccurate
**Fix-burst:** D-2238 — ADR-058 v2.17→v2.18 (§D1 param-threading explicit; §I1 two-step form); ROUTING-001 v1.31→v1.32 (AC-012+RG-024; density 24/12=2.00); COERCION-001 v1.30→v1.31 (sibling pin)

### Pass 34 (2026-08-18) — 4-Finding Pass

**Verdict:** CLEAN(strict)=NO
**Findings:** 4 (F-P34-LOW-002 §Status heading lag; F-P34-OBS-001 ocsf_field_to_arrow_name standardization; + 2 others)
**Fix-burst:** D-2239 — ADR-058 v2.18→v2.19; ROUTING-001 v1.32→v1.33 (§AC-012 call-site enumeration; threading expr corrected); COERCION-001 v1.31→v1.32 (sibling pin)

### Passes 35 / 37 / 38 (2026-08-18) — Three Interrupted CLEANs

**Pass 35:** CLEAN(strict)=YES — streak 1/3
**Pass 36:** 1 LOW (F-P36-LOW-001 volatile-line-cite TD-VSDD-091); records micro-burst D-2240 RESET streak
**Pass 37:** CLEAN(strict)=YES — streak 1/3
**Pass 38:** CLEAN(strict)=YES — streak 2/3
**Pass 39:** 1 LOW (F-P39-LOW-001 §Authority date parenthetical); records micro-burst D-2241 RESET streak
**Note:** Both p36 and p39 resets were records-tier findings only (TD-VSDD-096 micro-burst path).

### Pass 43 (2026-08-18) — HIGH + MED (F-P43-HIGH-001, F-P43-MED-001)

**Verdict:** CLEAN(strict)=NO
**Findings:** 2 — HIGH: §I1 `ocsf_field_to_arrow_name` canonical home ambiguous (prism-spec-engine vs prism-core); MED: Red Gate count stated as 24, should be 25
**Fix-burst:** D-2243 — ADR-058 v2.20→v2.21 (§I1 canonical home → `prism-spec-engine::column_mapping`; §B2 item 1; §G raw_extensions; §Status heading corrected)

### Pass 44 (2026-08-18) — OBS (F-P44-OBS-001)

**Verdict:** CLEAN(strict)=NO (1 OBS)
**Findings:** 1 OBS — raw_extensions `ColumnDescriptor` `col_type=Json / nullable=true` not reflected in §Interpretation A Tier-2
**Fix-burst:** D-2243 (batched with p43) — BC-2.16.003 v1.14→v1.15 (§Interpretation A Tier-2 alignment; EC-016-013-027 NEW)

### Pass 45 (2026-08-18) — MED (F-P45-MED-001)

**Verdict:** CLEAN(strict)=NO (1 MED)
**Findings:** 1 MED — phantom ADR-023 §D3 cite; corrected to `dependency-graph.md §Dependency Rules Rule 2`
**Fix-burst:** D-2243 (batched with p43/44) — ROUTING-001 v1.36→v1.37; COERCION-001 v1.33→v1.34 (sibling pin; input-hash update)

---

_Created by state-manager compact-state burst (D-2244+1). Data source: STATE.md Decisions Log D-2200..D-2244 + Phase Progress OCSF-correctness cascade row. Passes 1–28 granular finding counts not individually recorded; fix-burst existence confirms findings were present._

## Frontmatter Fields (extracted from STATE.md)

<!-- No adversary_pass_* frontmatter fields were present in STATE.md at compaction time (D-2244+1 compact-state burst).
     Pass data above was reconstructed from the Decisions Log (D-2200..D-2244) and Phase Progress Finding Progression column.
     Original field format: adversary_pass_N_findings: "description"
     Original field format: adversary_pass_N_date: "YYYY-MM-DD" -->

---

## E-REL-NOTES LOCAL Cascade (S-REL-CLIFF-001 + S-REL-WRITER-001)

**Target:** Frozen code worktree HEAD b90663a23 → fix-burst landed at 38a662224 (feature/E-REL-NOTES-changelog; NOT pushed; code-only worktree commit by devops — out of scope for factory burst).

**Scope:** git-cliff 2.14.1 two-layer CHANGELOG + release-notes pipeline; cliff.toml, release-prep.yml, RELEASING.md, ADR-063, S-REL-CLIFF-001, S-REL-BETA1-NOTES-001.

| Pass | Date | Findings | HIGH | MED | LOW | OBS | Streak | Verdict |
|------|------|----------|------|-----|-----|-----|--------|---------|
| Pass 1 | 2026-09-07 | 9 (+ post-fix empirical corrections) | 1 | 3 | 2 | 3 | 0/3 | CLEAN(strict): NO — all 9 FIXED; streak begins at 0/3 |
| Pass 2 | 2026-09-07 | 7 | 0 | 2 | 1 | 4 | 0/3 | CLEAN(strict): NO — all 7 FIXED; streak 0/3 |

**Pass 1 finding summary:**

- F-1 HIGH (devops): cliff.toml `commit_groups` non-existent in git-cliff 2.14.1 → native `group_by` attribute; token-free dry-run exit 0 verified.
- F-2 MED (devops): release-prep.yml Step 7a `## [v${VERSION}]` → `## [${VERSION}]` (cliff omits leading `v` in tag-derived headings).
- F-3 MED (devops): RELEASING.md §4 rewritten — two-layer git-cliff path replaces stale manual scaffold.
- F-4 MED (architect): ADR-063 v1.5→v1.6 — author attribution DROPPED per single-author signal.
- F-5 LOW (devops): release-prep.yml stale scaffold comments swept.
- F-6 LOW (devops): synthetic-breaking dry-run captured in `.rel-artifacts/cliff-dryrun-breaking-sample.txt`; Breaking-before-Added ordering verified.
- F-7 OBS (story-writer): S-REL-CLIFF-001 v1.3→v1.4 — Task 2 [remote.github] owner `jmagady`→`drbothen`.
- F-8 OBS (architect): ADR-063 v1.6 — D3 sketch/prose inconsistency resolved: feat bodies dropped, BREAKING footer inline retained (no free-form body paragraphs).
- F-9 OBS (story-writer): S-REL-BETA1-NOTES-001 v1.1→v1.2 — AC-006 dedup guard for pre-existing `## [1.0.0-beta.1]` section; Task 4 renamed detect+reconcile step; acceptance_criteria_count 5→6.

**Post-fix empirical accuracy corrections (ADR-063 v1.6→v1.7; architect + story-writer):**

- DEVIATION-1: Token-free PR-links — `commit.remote.pr_number` and `pr_url` are NOT exposed by git-cliff's Tera context; PR-links are injected via `commit_preprocessors` regex `replace` on the raw commit subject. GITHUB_TOKEN is not required for changelog generation. S-REL-CLIFF-001 Task 2, BCtable, Library GITHUB_TOKEN row, and risk_mitigations updated to v1.5.
- DEVIATION-2: `{ breaking = true }` `commit_parser` grouping NOT effective in git-cliff 2.14.1 for routing to a dedicated "Breaking Changes" header. Filter-based Tera two-part body (detect `BREAKING CHANGE:` trailer → emit `**Breaking Changes:**\n\n` sub-header) is authoritative. The erroneous `{ breaking = true }` entry was REMOVED from ADR-063 §D3 sketch in v1.7.

**TD-VSDD-097 sweep verdict:** Dim-1 CLEAR (ADR-063 has no sibling twin). Dim-2 DISCHARGED (ADR-063 §D3 v1.7 → S-REL-CLIFF-001 Task-2/§Narrative/BCtable swept same fix-burst; devops cliff.toml already aligned on code worktree). Dim-3 CLEAR (ordering MUST anchored to CLIFF-001 AC-006; F-9 AC-006 anchored in BETA1-NOTES).

**Pass 2 finding summary (all FIXED on code HEAD 38a662224→9f09df7a2; devops + architect):**

- MED-1 (devops): RELEASING.md §5 stale `<!-- 0 -->` breaking annotation — non-operational but misleading; replaced with filter-based `just check-changelog-breaking` mechanism description.
- MED-2 (devops + architect): empty-header `--prepend` corrupts CHANGELOG.md — blank `--header ""` causes `git-cliff --prepend` to prepend an empty line and duplicate `##` sections over iterations; resolved via 3-substep flow: (a) header carries full masthead+Unreleased section; (b) Python pre-strip idempotency script removes pre-existing masthead+Unreleased before each --prepend; (c) token-free Python link-ref update step post-prepend. ADR-063 v1.7→v1.8 §D3 DEVIATION-3 documents mechanism; S-REL-CLIFF-001 v1.5→v1.6 Task 2/Task 4/AC-003 swept.
- LOW-1 (devops): cliff.toml comment `# DEVIATION from v1.6` → conforms to §D3 as written; comment updated to reference §D3.
- OBS-1 (devops): `.rel-artifacts/*.txt` test artifact files `git rm`'d + `.gitignore` entry added.
- OBS-2 (devops): RELEASING.md §6 — nothing-to-release recovery path documented (re-run without `--unreleased`, clear partial changes).
- OBS-3 (devops): compare-link `[X.Y.Z]: https://...` ref defs added via link-ref substep in release-prep.yml.
- OBS-4 (devops): stale "scaffold" terminology swept from release-prep.yml comments.

**Post-fix spec corrections (ADR-063 v1.7→v1.8 §D3 DEVIATION-3):**

DEVIATION-3: `git-cliff --prepend --header ""` (empty header) is PROHIBITED — running git-cliff --prepend with an empty header string causes `git-cliff` to emit a blank prepend line, corrupting the CHANGELOG masthead on repeat invocations. The canonical mechanism is a 3-substep flow: (1) header carries masthead+Unreleased block; (2) Python pre-strip script removes pre-existing masthead+Unreleased before each --prepend (idempotency); (3) Python link-ref update step injects compare-link ref defs. Verified idempotent over 3 cycles. S-REL-CLIFF-001 Task 2/Task 4/AC-003 swept to the 3-substep flow; `header=""` prohibition noted.

**TD-VSDD-097 sweep verdict (D-2478):** Dim-1 CLEAR (ADR-063 has no sibling twin). Dim-2 DISCHARGED (ADR-063 §D3 DEVIATION-3 → S-REL-CLIFF-001 Task-2/Task-4/AC-003 swept in same burst; S-REL-WRITER-001 and S-REL-BETA1-NOTES-001 verified clean; cliff.toml/release-prep.yml aligned on worktree). Dim-3 CLEAR (prepend-correctness MUST anchored to CLIFF-001 AC-003 + AC-005).

**Next:** LOCAL adversary pass-3 (re-gate) on frozen code HEAD 9f09df7a2 → 3-CLEAN → BETA1-NOTES-001 CHANGELOG generation + human curation → demo → PR B → (human-auth) merge → (human-auth) tag v1.0.0-beta.1.
