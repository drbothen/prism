---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-26T00:00:00
phase: 3
inputs: []
input-hash: "901dbbb"
traces_to: prd.md
pass: 2
previous_review: pass-1.md
cycle: phase-3-dtu-wave-2
gate: wave-2-integration-gate
scope: e45159b9..901dbbba (story PRs + Pass 1 fix-PRs)
reviewer: general-purpose-as-adversary (vsdd-factory:adversary tool-binding workaround)
tools_available: Read, Grep, Glob, Bash (all verified working)
prior_pass_findings_closed: 11/16
prior_pass_findings_filed_as_td: 5
novelty_assessment: DECAYING (Pass 2; 5 new findings vs 16 in Pass 1; novelty 0.31)
verdict: FINDINGS_OPEN
total_findings: 5
critical: 0
high: 0
medium: 1
low: 4
date: 2026-04-26
---

# Adversarial Review — Wave 2 Integration Gate, Pass 2

## Summary

Pass 2 has full Bash/Grep/Glob/Read tool access (Pass 1 had Read-only). Verified all 11 Pass-1 closure claims against develop @ `901dbbba`: 10 CLOSED-VERIFIED, 1 CLOSED-WITH-RESIDUAL (escalated as W2-P2-A-002 LOW). Performed all six spec-layer policy sweeps that Pass 1 could not (POL-1, POL-2, POL-6, POL-7, POL-8, POL-9) plus the previously-PASSable POL-5: all PASS. Surfaced 5 NEW findings: 0 CRITICAL, 0 HIGH, 1 MEDIUM (`scan_events` doc-vs-code drift), 4 LOW (RED-comment dust, `case.rs` Kani-derive in CI hotfix, STORY-INDEX historical narrative drift, S-2.08 inherited-BC schema gap). Verdict: FINDINGS_OPEN — driven by the single MEDIUM. After a small W2-FIX-E sweep + TD entries, a Pass 3 confirmation should converge.

## Finding ID Convention

Finding IDs in this pass follow the gate-scoped Pass-2 format: `W2-P2-A-NNN`.

- `W2`: Wave 2 scope prefix
- `P2`: Pass 2
- `A`: Adversarial
- `NNN`: Three-digit sequence

This matches the Pass-1 (`W2-P1-A-NNN`) convention and lets Pass-2 findings be tracked alongside Pass-1 closures within the gate's convergence trajectory.

## Tool verification preamble

Three required tool checks performed; all succeeded.

| Tool check | Command | Outcome |
|---|---|---|
| Bash | `git log --oneline -5` | PASS — 5 commits returned, HEAD = `901dbbba` |
| Glob | `.factory/specs/behavioral-contracts/*.md` | PASS — 200+ BC files enumerated |
| Grep | pattern `BC-2.05` in `.factory/` | PASS — multiple matches in BC-INDEX, stories, evidence reports |

**Conclusion:** Pass 2 has the tool access Pass 1 lacked. POL-1/POL-2/POL-6..POL-9 are now genuinely auditable rather than `NOT-FULLY-VERIFIED`.

---

## Part A — Pass 1 closure verification table

Verification performed against develop @ `901dbbba` (clean working tree).

| Finding | Severity | Fix-PR / SHA | Verification | Verdict |
|---|---|---|---|---|
| W2-P1-A-001 | CRITICAL | #62 / `48bd45b9` | `crates/prism-sensors/src/event_buffer.rs:201-206` shows `self.backend.put_batch(...).map_err(...)?` propagation; old `let _ = ...` swallow gone; doc comment at line 195-196 retitled to "Errors are propagated — a failed backend write is not recoverable." | **CLOSED-VERIFIED** |
| W2-P1-A-002 | CRITICAL | #64 / `281f1237` + #65 / `901dbbba` | `docs/demo-evidence/S-2.08/evidence-report.md:62-72` documents v1.8 split with AC-5a (PASS) / AC-5b (DEFERRED to S-3.02). Coverage Map rows 89-90 show AC-5a `4/4 PASS (cold-start cases in 8/8 dispatch suite)` and AC-5b `N/A — deferred`. Summary at line 367: `"9 of 11 (AC-5a routing PASS; AC-5b execution + AC-6 deferred to S-3.02 — sensor-adapter wiring required for execution-side)"`. Spec `.factory/stories/S-2.08-event-tables.md` frontmatter `version: "1.8"`; AC-5a (line 195) and AC-5b (line 207) explicitly marked. Changelog row at line 423 ties to PR-FIX-W2-D. | **CLOSED-VERIFIED** |
| W2-P1-A-004 | HIGH | #62 / `48bd45b9` | `crates/prism-sensors/src/event_buffer.rs:354-361` shows a per-key loop that propagates the first `remove` error via `?`. Doc comment at lines 348-353 explicitly notes the partial-state contract (cache flushed, backend may retain stale keys). | **CLOSED-VERIFIED** |
| W2-P1-A-015 | HIGH | #64 / `281f1237` | `crates/prism-sensors/src/tests/poller_tests.rs:386-389` (slightly off the claimed 374-384 range): `assert!(ids.is_empty(), "S-2.08 stub: start_pollers returns empty until S-3.02 wires real specs");`. The vacuous `let _ = ids;` is gone. The earlier `test_W2_P1_A_015_start_pollers_returns_empty_vec` test at lines 365-372 has the same assertion. | **CLOSED-VERIFIED** (note: line range in claim was approximate; semantic intent fully met) |
| W2-P1-A-005 | MEDIUM | #62 / `48bd45b9` | `crates/prism-sensors/src/event_buffer.rs` line 5 module doc, line 9-11 Key suffix note ("4-byte `subsec_nanos` suffix is collision-prone … TD-W2-ULID-001"), line 73 fn doc, lines 76-78 nanos suffix description, line 88 inline comment, line 145 scan_events doc — all consistently use `nanos_be:4` notation. | **CLOSED-VERIFIED** |
| W2-P1-A-006 | MEDIUM | #63 / `de8cd957` | `docs/demo-evidence/S-2.05/evidence-report.md` lines 36-40: BC-2.05.005=2, BC-2.05.007=11, BC-2.05.009=2, BC-2.05.010=4. Sum 2+11+2+4 = 19. Total row line 40: `35 / 19 / 16 / RED ratio: 54.3% (above 50% gate)`. 19/35 = 0.5429 → 54.3% rounded; arithmetic reconciled. | **CLOSED-VERIFIED** |
| W2-P1-A-008 | LOW | #63 / `de8cd957` | `crates/prism-dtu-slack/Cargo.toml:15-17` `[features]` block contains only `dtu = []` and `tls = [...]`; no `default = [...]` line. Cross-checked siblings: only `prism-dtu-demo-server` retains `default = ["dtu"]` (intentional binary). | **CLOSED-VERIFIED** |
| W2-P1-A-009 | LOW | #63 / `de8cd957` | `grep -n "RED:" crates/prism-query/src/tests/materialization_tests.rs` returns zero hits. | **CLOSED-VERIFIED** |
| W2-P1-A-010 | LOW | #63 / `de8cd957` | `grep -n "RED:" crates/prism-dtu-armis/tests/ac_6_rate_limit_429.rs crates/prism-dtu-common/tests/ac_2_failure_layer_rate_limit.rs` returns zero hits in both. | **CLOSED-VERIFIED** |
| W2-P1-A-011 | LOW | #63 / `de8cd957` | `crates/prism-spec-engine/tests/bc_2_16_table_type_test.rs:245` still contains the section banner `// validate_table_spec — AC-7, EC-002 (RED: todo!())` despite `validate_table_spec` being fully implemented (no `todo!()`) at `crates/prism-spec-engine/src/spec_parser.rs:257`. | **CLOSED-WITH-RESIDUAL** — see W2-P2-A-002 below |
| W2-P1-A-013 | LOW | #63 / `de8cd957` | `crates/prism-query/src/types.rs` lines 1-26 module doc disambiguates the three descriptor types and lists `columns` as a field of `InternalTableDescriptor` (line 14) and `SensorTableDescriptor` (line 19). The `SensorQueryDescriptor` doc itself (line 33-43) clarifies it does NOT carry `columns` (only routing context: `table_name`, `table_type`, `rows_from_buffer`). | **CLOSED-VERIFIED** |

**Closure summary:** 10 CLOSED-VERIFIED, 1 CLOSED-WITH-RESIDUAL, 0 REOPENED. The single residual is escalated as a new Pass 2 finding (W2-P2-A-002) and is LOW severity.

---

## Part B — Spec-layer policy verifications (POL-1..POL-9)

| Policy | Verdict | Evidence |
|---|---|---|
| **POL-1** append_only_numbering | **PASS** | BC-INDEX.md frontmatter: `total_contracts: 208 (200 active, 6 removed, 2 retired)`; on-disk count of files matches. All 8 currently-tombstoned IDs (BC-2.01.001/.003/.009/.011/.012/.015 — removed; BC-2.12.011/.012 — retired) have on-disk file with frontmatter `status: removed` or `status: retired`. None reused for an active contract. VP-INDEX has 62 sequential VPs (vp-001..vp-062) on disk and 62 rows in INDEX, no retirements. STORY-INDEX `total_stories: 76`, on-disk `S-*.md` files = 76. |
| **POL-2** lift_invariants_to_bcs | **PASS (with explicit retirements)** | `.factory/specs/domain-spec/invariants.md` defines DI-001..DI-032. Per-DI BC citation counts: DI-001..008 all >0; DI-009/.010/.011/.013 all 0 — but each is explicitly tombstoned with `~~DI-NNN~~` strikethrough and "**REMOVED** — No longer applicable …" rationale (cursor refactor). DI-012 onward: all >0. No silent orphans. |
| **POL-5** creators_justify_anchors | **PASS** | Sampled 5 Wave 2 BCs (BC-2.01.013, BC-2.01.014, BC-2.05.005, BC-2.05.007, BC-2.08.001). Each has: `capability:` frontmatter field naming the CAP, `inputs:` listing `.factory/specs/domain-spec/capabilities.md`, `traces_to:` array containing the CAP, and an `## Traceability` table row with `L2 Capability \| CAP-NNN`. No anchor is unjustified. |
| **POL-6** architecture_is_subsystem_name_source_of_truth | **PASS** | ARCH-INDEX Subsystem Registry lists SS-01..SS-20. Sampled BC frontmatter: BC-2.01.013→SS-01, BC-2.05.005→SS-05, BC-2.08.001→SS-08. Sampled Wave 2 stories: S-2.04→[SS-05], S-2.05→[SS-05], S-2.06→[SS-01], S-2.07→[SS-01], S-2.08→[SS-01, SS-16]. Every tag matches the registry verbatim. |
| **POL-7** bc_h1_is_title_source_of_truth | **PASS** | Sampled 10 Wave 2 BCs (BC-2.01.002, .013, .014; BC-2.05.005, .007, .008, .011; BC-2.08.001, .008, .009). H1 heading equals BC-INDEX title column verbatim for all 10. Example: BC-2.05.011 → both render `Audit Forwarding — At-Least-Once Delivery to External Destinations (VP-039 monotonic watermark)`. |
| **POL-8** bc_array_changes_propagate_to_body_and_acs | **PASS (with caveat)** | Sampled S-2.04, .05, .06, .07, .08, S-6.11/12/13. For S-2.04..S-2.07: frontmatter `behavioral_contracts:` and body BC-traceability table contain the same set of BC IDs. Token Budget references those BCs. S-2.08 declares `behavioral_contracts: []` and `anchor_subsystem: null` — intentional: it ships infrastructure only; the body's BC-2.11.005 / BC-2.11.007 references are inherited-deferred to S-3.02 (per AC-5b note + changelog v1.8). S-6.11/12/13 (DTU stories) all carry `behavioral_contracts: []` — DTUs are test-fixture clones with no production BCs. **Caveat:** the S-2.08 body explicitly mentions BC-2.11.005/007 for AC-5b semantics; a strict policy reading might require these to surface in `inherited_bcs` or similar. The current null-array form is consistent with the v1.8 deferral, but is fragile (see W2-P2-A-005 below). |
| **POL-9** vp_index_is_vp_catalog_source_of_truth | **PASS** | VP-INDEX.md has 62 catalog rows; 62 vp-NNN.md files on disk; no skipped numbers. `verification-architecture.md` cites all 62 VP IDs; `verification-coverage-matrix.md` cites all 62. Spot-check VP→Module pairs (VP-001/019/039/050/062): VP-INDEX module column matches verification-architecture module column for every sample. |

---

## Part B — NEW findings (W2-P2-A-NNN)

### MEDIUM

#### W2-P2-A-001 — `scan_events` doc claims "calls `evict_expired` lazily before returning results" but body never calls it
- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Category:** Spec/code drift (doc misrepresents behavior)
- **Location:** `crates/prism-sensors/src/event_buffer.rs:222-277`
- **Evidence:**
  - Doc comment line 223-224: `"Calls `evict_expired` lazily before returning results to prevent stale data from appearing in query results (AC-4)."`
  - `evict_expired` doc comment line 282-283: `"Lazy eviction strategy: called at read time before returning results, and again by the background poller after each ingest cycle."`
  - Function body lines 228-277: no call to `self.evict_expired(...)` anywhere; only the in-memory cache range scan, optional backend fallback, and JSON deserialization.
- **Implication:** AC-4 ("expired buffered records do not appear in query results") relies on the poller calling `evict_expired` (which it does at `poller.rs:176`). But during the cold-start window before any poller cycle has run, or when the poller has been canceled but `scan_events` is still callable, expired records could be returned by `scan_events` despite the doc's promise. Even if functionally compensated by the poller cadence, the doc-vs-code drift is a contract-trust hazard introduced post-Pass-1 (the W2-FIX-A doc realignment did not touch this area).
- **Proposed Fix:** Either (a) actually invoke `self.evict_expired(...)` at the top of `scan_events` (matching the doc), or (b) edit both doc comments to truthfully reflect "eviction is performed by the background poller; `scan_events` does not evict on read" and link the AC-4 obligation to the poller's `evict_expired` call site.
- **Novelty:** NEW (not in Pass 1; the W2-FIX-A doc realignment focused on the suffix/ULID claim, missed this neighbor doc)

### LOW

#### W2-P2-A-002 — Stale `(RED: todo!())` section banner in BC-2.16 table-type test, plus broad RED-comment dust elsewhere
- **Severity:** LOW
- **Confidence:** HIGH
- **Category:** Stale documentation; W2-FIX-C scope gap
- **Location:**
  - `crates/prism-spec-engine/tests/bc_2_16_table_type_test.rs:245` — `// validate_table_spec — AC-7, EC-002 (RED: todo!())` (production code at `prism-spec-engine/src/spec_parser.rs:257` is fully implemented)
  - 110 total `// RED:` or `/// RED:` lines across `crates/`, including:
    - `crates/prism-audit/src/tests/specialized_event_tests.rs` (~20 RED annotations referring to `to_vector_json`, `emit_*` which all GREEN per S-2.05 evidence-report)
    - `crates/prism-sensors/tests/test_cyberint.rs` (8 RED annotations referring to `CyberintAdapter::new` which is implemented in S-2.06)
    - `crates/prism-sensors/tests/test_claroty.rs` (10 RED annotations)
    - `crates/prism-sensors/tests/test_pagination.rs` (RED on `OffsetCursor::advance`)
- **Evidence:** Pass 1 listed only the 4 files cleaned by W2-FIX-C (PR #63). The cleanup was scoped — Pass 1 did not enumerate the broader codebase. With Grep available in Pass 2, the residual count is ~110.
- **Implication:** RED comments referring to no-longer-extant `todo!()`s are misleading to future readers and to AI agents reading test files for context. They imply work-in-progress where the work has shipped.
- **Proposed Fix:** File W2-FIX-E (sweep) or escalate as TD-W2-RED-DUST-001 to remove all `// RED:` / `/// RED:` annotations whose subject is no longer `todo!()`.
- **Novelty:** NEW (broader-scope residual of W2-P1-A-009/.010/.011)

#### W2-P2-A-003 — CI hotfix #45 (`7903da15`) touched `crates/prism-core/src/case.rs` — product-code creep beyond `.github/` and `fuzz/`
- **Severity:** LOW
- **Confidence:** HIGH
- **Category:** Process — gate-scope adherence
- **Location:** `crates/prism-core/src/case.rs:50` (added `#[cfg_attr(kani, derive(kani::Arbitrary))]` to `CaseStatus`)
- **Evidence:** Mission's gate criterion: "Verify CI hotfixes (#44–#50) touched only `.github/workflows/` and `fuzz/Cargo.toml` — no product code creep." `git show --stat 7903da15` shows two files: `.github/workflows/post-merge.yml` (4 lines) and `crates/prism-core/src/case.rs` (1 line). The 1-line edit is gated on `cfg(kani)` and has zero effect on production builds, but it IS a product-code mutation in a CI hotfix.
- **Implication:** Strict reading of the gate flags it; pragmatic reading accepts it (Kani proofs require the trait, change is conditional). Filing as LOW for process-traceability; recommend explicit ADR or policy update if such Kani-derive additions are routine.
- **Proposed Fix:** Either (a) accept and document in policy that `#[cfg_attr(kani, ...)]` derives are a permitted CI-hotfix exception, or (b) require future Kani-derive additions to ride a normal product PR. Either way, surface explicitly.
- **Novelty:** NEW (Pass 1 had no Bash/Grep to enumerate hotfix file scopes)

#### W2-P2-A-004 — DI count drift — historical narrative bullets in STORY-INDEX cite "62 stories" / "75 stories" while frontmatter and disk both show 76
- **Severity:** LOW
- **Confidence:** MEDIUM
- **Category:** Documentation hygiene; stale narrative
- **Location:** `.factory/stories/STORY-INDEX.md` frontmatter `total_stories: 76` (correct); narrative bullets in burst-history scattered in body include `"story count remains 62"`, `"story count 62 → 75"`, `"story count remains 75"`. Disk count: 76 `S-*.md` files.
- **Evidence:** `grep -c "^| S-" STORY-INDEX.md` = 103 (matrix rows); `total_stories: 76` (frontmatter); `ls .factory/stories/S-*.md | wc -l` = 76. The historical bullets are point-in-time records and are not technically wrong (they describe past states), but they are easy to misread as current.
- **Implication:** Future maintainers or AI agents skimming the index for the current count may pull a stale number from the body. Pass 1 could not see this with Read-only access to specific files.
- **Proposed Fix:** Add a one-line "Current count: 76 stories" callout at the top of STORY-INDEX above the burst-history; or prefix each historical bullet with the date so readers don't mistake them for current state.
- **Novelty:** NEW

#### W2-P2-A-005 — S-2.08 frontmatter declares `behavioral_contracts: []` while body cites BC-2.11.005/007 in AC-5b inherited-defer note
- **Severity:** LOW
- **Confidence:** MEDIUM
- **Category:** Spec frontmatter / body coherence (POL-8 boundary case)
- **Location:** `.factory/stories/S-2.08-event-tables.md:20` (`behavioral_contracts: []`) vs lines 207-217 (AC-5b body referencing `BC-2.11.005`, `BC-2.11.007`) and changelog line 423.
- **Evidence:** The intent of `behavioral_contracts: []` is "this story implements no new BCs." That is true for S-2.08 — it ships infrastructure that BC-2.11.005/.007 will exercise in S-3.02. But future tooling that scrapes frontmatter to build a "story → BC" graph will miss the inherited-deferred reference. There is no `inherited_bcs:` or `deferred_bcs:` field in the schema to express this nuance.
- **Implication:** When S-3.02 lands, if no one updates a "what S-2.08 set up" trace, the structural-foundation contribution of S-2.08 to BC-2.11.005/007 is invisible to graph queries.
- **Proposed Fix:** Either (a) add an optional `inherited_bcs:` frontmatter field across the schema and populate `[BC-2.11.005, BC-2.11.007]` on S-2.08, or (b) populate `behavioral_contracts: [BC-2.11.005, BC-2.11.007]` and tag them with status "deferred" via a parallel field. Capture as TD if not addressed in this gate.
- **Novelty:** NEW

---

## Cross-cuts checked (no findings)

The following Pass-2-specific concerns were investigated and did NOT yield findings:

1. **Did fix-PR #62 introduce a regression downstream of `write_events` / `evict_expired`?** Search `grep -rn "write_events\|evict_expired" crates --include='*.rs'` shows the only non-test caller of `evict_expired` is `crates/prism-sensors/src/poller.rs:176`, which already wraps the call in `if let Err(e) = ... { tracing::warn!(...); }`. Error propagation breakage is structurally impossible. `write_events` has no production callers wired yet (called only via tests + S-3.02 future wiring). No regression introduced.
2. **AC-5 split semantic preservation across evidence-report v1.8 + S-2.08 spec v1.8 + companion S-3.02 v1.7.** The phrase "AC-5a (routing PASS)" appears in evidence-report.md line 70 and S-2.08 spec line 195 with consistent semantics ("route_table_query returns ColdStartFallback for EventStream with no buffered data"). AC-5b inherited tag at S-2.08 line 217 cross-references "S-3.02 v1.7 as: 'AC-X: Fulfills inherited S-2.08 AC-5b — cold-start...'". Verified S-3.02 spec by grep: matches.
3. **CI hotfix file-scope sweep.** PRs #44, #46, #47, #48 touched only `.github/workflows/` (#46 also `release.yml`). PR #45 touched `case.rs` (recorded as W2-P2-A-003). PR #49 touched `fuzz/Cargo.toml`. PR #50 touched `.github/workflows/post-merge.yml`. PR #51 (`8eafb7b7`, OBS-001) touched `prism-dtu-demo-server/Cargo.toml` — but #51 is OUTSIDE the #44-#50 mission window so not a violation.
4. **Are AC-5a's 4 dispatch tests real?** `crates/prism-sensors/src/tests/table_dispatch_tests.rs` exists; spot-check confirms 8 test functions covering the full PointInTime/EventStream × buffered/cold-start matrix as referenced.
5. **Did the W2-FIX-D evidence-report v1.8 amendment break the AC numbering invariant?** Coverage Map renames AC-5 → AC-5a + AC-5b; total ACs effectively 11. Summary line 367 says "9 of 11 (AC-5a routing PASS; AC-5b execution + AC-6 deferred)". Arithmetic: AC-1, .2, .3, .4, .5a, .7, .8, .9, .10 PASS = 9; AC-5b, .6 deferred = 2; 9+2 = 11. Consistent.

---

## Verdict

**FINDINGS_OPEN** — convergence criteria evaluation:

| Criterion | Met? | Reason |
|---|---|---|
| (a) all Pass 1 closures verified | YES (with one residual) | 10/11 CLOSED-VERIFIED, 1 CLOSED-WITH-RESIDUAL — the residual is itself a LOW finding (W2-P2-A-002) |
| (b) zero new CRITICAL findings | YES | 0 new CRITICAL; 1 MEDIUM, 4 LOW |
| (c) all 9 policies PASS or N/A | YES | POL-1, POL-2, POL-5, POL-6, POL-7, POL-8 (with caveat), POL-9 — all PASS. POL-3, POL-4, POL-10 are out-of-scope for this mission's Part B menu but were marked PASSable in Pass 1. |

The verdict is **FINDINGS_OPEN** because of W2-P2-A-001 (MEDIUM, doc-vs-code drift in `scan_events`) — convergence requires zero open MEDIUM-or-higher findings unless explicitly re-classified as TD by the orchestrator. The 4 LOW items (W2-P2-A-002..005) are individually small but collectively suggest a sweep PR or batched TD entry.

**Recommended next actions:**
1. **W2-FIX-E (one PR):** fix `scan_events` doc OR add a real `evict_expired` call (pick one); strip the BC-2.16 RED banner; choose `kani::Arbitrary` policy stance for `case.rs`.
2. **TD-W2-RED-DUST-001:** sweep all 110 stale `// RED:` annotations across `crates/`.
3. **TD-W2-S2.08-INHERITED-BCS:** decide whether to add `inherited_bcs` schema field; defer if not.

After W2-FIX-E lands, a Pass 3 confirmation should suffice (no further policy/closure work required).

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 5 |
| **Duplicate/variant findings (Pass 1 reopens)** | 0 |
| **Pass 1 closures verified** | 10 CLOSED-VERIFIED + 1 CLOSED-WITH-RESIDUAL (residual escalated as W2-P2-A-002 LOW) |
| **Novelty score** | 5 / (5 + 11) = 0.31 |
| **Median severity** | LOW (1 MEDIUM, 4 LOW; median lands at LOW) |
| **Trajectory** | Pass 1 = 16 findings → Pass 2 = 5 new findings (decay 11/16 = 69%) |
| **Verdict** | FINDINGS_REMAIN (1 MEDIUM blocks convergence; 4 LOW recommended for sweep) |

Trajectory analysis: novelty has decayed sharply (1.0 → 0.31). Severity has decayed even more sharply (Pass 1 had 2 CRITICAL + 4 HIGH; Pass 2 has 0 CRITICAL + 0 HIGH). The MEDIUM is a doc-vs-code drift discovered only because Pass 2 had Bash/Grep tools to follow callers. One more Pass (Pass 3) after a small W2-FIX-E PR is expected to converge to zero new findings.

## Appendix — Evidence cache

All evidence was gathered against develop @ `901dbbba7605d42ac71a44d1bfc3491a00c23869`, working tree clean, on 2026-04-26.

Key file:line citations are inlined in each finding. Counts:
- BCs on disk: 208 (matches frontmatter)
- VPs on disk: 62 (matches frontmatter)
- Stories on disk: 76 (matches frontmatter)
- Stale `// RED:` lines in `crates/`: 110 (Pass 1 closed 4 files of unknown larger scope)
- DI orphans in invariants.md: 4, all explicitly REMOVED with rationale
- Wave 2 BCs sampled for POL-5/.7: 10
- Wave 2 stories sampled for POL-6/.8: 8
