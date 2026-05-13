---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 8
target_sha: 0cbb8371
story_content_sha: 479aee14
bc_amendments_sha: 77ba2b0f
base_sha: 95d46be2
verdict: BLOCKED-hard
streak: "0/3 → 0/3"
finding_summary: {CRITICAL: 0, HIGH: 1, MEDIUM: 2, LOW: 1, OBS: 2}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4"
idempotency_check: false
producer: adversary (vsdd-factory; reified by state-manager due to read-only tool profile)
---

# Adversarial Review — Pass-8 — S-PLUGIN-PREREQ-D

## §1 Context

- Target HEAD: factory `0cbb8371` (D-476 + D-477 fix-burst-6 stage 3); develop `95d46be2`.
- Streak before pass-8: 0/3 (pass-7 reset via 7 novel findings).
- Fix-burst-6 closed: F-LP7-HIGH-001 (path mis-anchor pipeline.rs), F-LP7-HIGH-002 (path mis-anchor auth_provider.rs), F-LP7-HIGH-003 (BC-2.22.001 v1.4 plugin-load step 7.5 amendments), F-LP7-HIGH-004 (host_functions.rs Match-Site row), F-LP7-MED-001 (BC-2.17.002 E-PLUGIN-005 30s amendment), F-LP7-MED-002 (Task 9 step numbering), F-LP7-LOW-001 (BC-2.22.001 lifecycle_status Path A).
- Pass-8 mandate: paper-fix rederivation, idempotency, cascade-impact verification per orchestrator directive.

## §2 Pass-7 Closure Rederivation

| Finding | Closure Claim | Pass-8 Status | Evidence |
|---------|---------------|---------------|----------|
| F-LP7-HIGH-001 | 8 sites swept `src/plugin/pipeline.rs` → `src/pipeline.rs` | CONFIRMED CLEAN | Glob confirms only `crates/prism-spec-engine/src/pipeline.rs` exists; only residual reference in changelog row 875 (historical sweep narrative) |
| F-LP7-HIGH-002 | 5 sites swept `src/plugin/auth_provider.rs` → `src/auth_provider.rs` | CONFIRMED CLEAN | Glob confirms only `crates/prism-spec-engine/src/auth_provider.rs` exists; old path only in historical changelog row 875 |
| F-LP7-HIGH-003 | BC-2.22.001 v1.4 plugin-load step 7.5 amendments | CONFIRMED CLEAN | BC-2.22.001 v1.4 §Postconditions (lines 97–117), §Sequencing Invariant with step 7.5 (lines 119–148), §Exit-Code Map plugin rows (lines 159–160), §Pre-Traffic Gate condition 6 (lines 185–189), §Invariants (lines 210–212), §Related BCs BC-2.17.007 (line 292), §Architecture Anchors ADR-023 §C4 (line 298). 35 "plugin" occurrences, all load-bearing. |
| F-LP7-HIGH-004 | host_functions.rs Match-Site row + Task 4 sibling-sweep instruction | CONFIRMED CLEAN | Story line 598 Match-Site Inventory row present with explicit override-defeats-builder rationale; Task 4 line 491 carries TD-VSDD-060 instructions including doc-comment rewrite from "10-second" to "30-second" |
| F-LP7-MED-001 | BC-2.17.002 v1.4 E-PLUGIN-005 timeout 10s → 30s | CONFIRMED CLEAN | BC-2.17.002 line 77 reads `30s per request limit`; v1.4 changelog row at line 139 documents the closure |
| F-LP7-MED-002 | Task 9 step numbering finalized at 7.5 with rationale | CONFIRMED CLEAN | Story Task 9 line 520: "storage = step 7, **plugin-load = step 7.5**, query-engine = step 8, MCP server = step 9 (function `step9_start_mcp_server` retained). Rationale: step 7.5 chosen to avoid cascading renumber..." — single canonical wording |
| F-LP7-LOW-001 | BC-2.22.001 Path A: status:active + lifecycle_status:active | CONFIRMED CLEAN (primary) — sibling-sweep gap surfaces F-LP8-HIGH-001 | BC-2.22.001 v1.4 line 5 `status: active`, line 12 `lifecycle_status: active`; BC-INDEX v4.68 row line 242 active |

All 7 pass-7 closures textually idempotent on their primary targets. No paper-fix regression.

## §3 Filesystem-Grounded Verification

- `crates/prism-spec-engine/src/pipeline.rs` exists; `crates/prism-spec-engine/src/auth_provider.rs` exists.
- `crates/prism-spec-engine/src/plugin/pipeline.rs` does NOT exist (Glob: zero results).
- `crates/prism-spec-engine/src/plugin/auth_provider.rs` does NOT exist (Glob: zero results).
- Story file references to `src/plugin/pipeline.rs` / `src/plugin/auth_provider.rs`: only in changelog row 875 (historical sweep narrative). Active spec body: zero references. CONFIRMED CLEAN.
- `plugin_disabled_env`: zero matches in active body (only in changelog row 875). `plugin_load_disabled_via_envvar`: 6 active-content references. CONFIRMED CLEAN.
- `host_functions.rs` line 30 doc-comment still reads "10-second per-request timeout" and line 154 still has `.timeout(Duration::from_secs(10))` — production code in pre-PREREQ-D state. Story Task 4 + Match-Site Inventory row 598 carry explicit closure instructions for these production sites. CONFIRMED — spec is correct; production code closure is implementer's responsibility post-implementation.
- `pub struct HostState` at `crates/prism-spec-engine/src/plugin/loader.rs:101` does NOT carry `#[non_exhaustive]` and has `allowed_urls: Option<Vec<String>>` at line 106 — pre-PREREQ-D state. AC-17 covers this. CONFIRMED.

## §4 POL-20 Anchored-Regex Workspace Sweep

- Anchored regex `^introduced:\s+(?!\"?(cycle-[0-9]+|[0-9]{4}-[0-9]{2}-[0-9]{2})\"?\s*$)` across all 236 BC files: zero matches.
- POL-20 compliance: VERIFIED — zero violations.

## §5 Cascade Impact Verification

- BC-2.22.001 cited in 9 BC/architecture files. ARCH-INDEX line 160 cites generically; module-decomposition.md lines 168, 524, 554 cite generically; ADR-025 cites for historical context. No sibling-sweep gap in these files for v1.4 amendments — they cite at namespace/identity level, not deep-content level.
- BC-INDEX v4.68 row line 242 active/v1.4. CONFIRMED.
- ADR-022 §B (canonical boot sequence) has zero references to plugin-load step 7.5. Per BC-2.22.001 v1.4 §Sequencing Invariant rationale: intentional (avoids cascading renumber). Source-of-truth precedence (ADR-023 supersedes ADR-022 for plugin-load placement) makes this consistent. Surfaced as F-LP8-OBS-001 below — informational, not blocking.

## §6 Findings

### F-LP8-HIGH-001 — Sibling lifecycle_status drift on 6 plugin BCs; story body falsely claims "All BCs are active"

- **Severity:** HIGH (blast radius = 6 BCs + 1 story comment)
- **Confidence:** HIGH
- **Evidence:**
  - Story `S-PLUGIN-PREREQ-D` frontmatter comment line 16: "All BCs are active."
  - BC-2.17.001/002/003/004/006/007 all have `status: draft` + `lifecycle_status: active` (inverted)
  - BC-INDEX rows (lines 215–221): all 6 show `draft` status
- **Why it matters:** F-LP7-LOW-001 closed the SAME drift pattern for BC-2.22.001 via Path A (promoted because BC-INDEX had it as active since v4.51 D-319). For these 6 plugin BCs, the drift is INVERTED: BC-INDEX shows `draft` AND no merge event has promoted them. The BC file `lifecycle_status: active` is the stale value (likely 2026-04-20 v1.1 "Wave-6-pre-build-sweep" pre-POL-14). Plus the story comment falsifies the state for implementers. Path B (correct BC files to draft) is the right adjudication.
- **Fix-routing:** product-owner (BC content amendments on 6 BCs) + story-writer (correct story comment). `[process-gap]` (recurrent — same root cause as F-LP7-LOW-001; codification candidate).

### F-LP8-MEDIUM-001 — BC-2.22.001 `plugin_load_unsigned` level says WARN; story Structured Event Catalog says AUDIT

- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Evidence:** BC-2.22.001 v1.4 line 100: "audit event `plugin_load_unsigned` emitted at WARN with fields `plugin_path` and `plugin_hash` (sha256)". Story Structured Event Catalog line 659 row: `| plugin_load_unsigned | AUDIT | ... |`.
- **Why it matters:** In Rust `tracing` parlance, a single emission has exactly one level. If actual emission is `tracing::warn!(event_type="plugin_load_unsigned", ...)`, then level=WARN; "AUDIT" is a routing characteristic, not a tracing level. Implementer cannot tell whether to emit at WARN or some other level.
- **Fix-routing:** product-owner (clarify BC) → story-writer (sync Catalog row Level column).

### F-LP8-MEDIUM-002 — AC-9 trace omits BC-2.17.002 v1.4 cross-reference now that PO closed cross-doc gap

- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Evidence:** Story AC-9 header line 343: `(traces to ADR-023 §C4 plugin HTTP defaults; closes TD-S-PLUGIN-PREREQ-B-005)` — no BC-2.17.002 reference. Story AC-9 body lines 373–375 closure note hidden: "Closed by BC-2.17.002 v1.4 amendment (fix-burst-6)". BC-2.17.002 v1.4 line 77 now canonically owns the 30s timeout statement.
- **Why it matters:** Fix-burst-5 re-anchored AC-9 to ADR-023 §C4 documenting BC-2.17.002 as out-of-perimeter. Fix-burst-6 closed F-LP7-MED-001 (BC-2.17.002 v1.4 amendment) — BC-2.17.002 is no longer out-of-perimeter. AC-9 trace header is stale; reader checking AC-9's BC anchor sees ADR-023 only, missing the now-canonical BC-2.17.002.
- **Fix-routing:** story-writer (one-line edit to AC-9 trace header).

### F-LP8-LOW-001 — BC-2.17.002 v1.4 `status: draft` + `lifecycle_status: active` divergence (subset of HIGH-001)

- **Severity:** LOW (subset of HIGH-001; logged separately because PO touched BC-2.17.002 in fix-burst-6 stage 1 — drift not caught during PO's own amendment)
- **Confidence:** HIGH
- **Evidence:** BC-2.17.002 line 5 `status: draft`, line 12 `lifecycle_status: active`. PO authored v1.4 amendment but did not address divergence.
- **Why it matters:** Fix-burst-6 was the opportunity to catch this. Adjudication: Path B (BC-INDEX draft, no merge event, set `lifecycle_status: draft`).
- **Fix-routing:** Bundle with F-LP8-HIGH-001.

### F-LP8-OBS-001 — ADR-022 §B does not reference step 7.5 intercalation; cross-doc gap (informational)

- **Severity:** OBS
- **Confidence:** HIGH
- **Evidence:** ADR-022 §B lines 182–228 enumerate Steps 1–8; no step 7.5. ADR-023 §C4 carries intercalation. BC-2.22.001 v1.4 line 124: step 7.5 avoids cascading renumber "by design."
- **Why it matters:** Source-of-truth precedence (ADR-023 supersedes ADR-022 for plugin-load placement) makes this consistent. But operators reading ADR-022 alone will miss the plugin-load step. One-line cross-reference closes discoverability gap without invalidating "no renumber" decision.
- **Fix-routing:** architect (ADR-022 cross-reference, fix-in-scope per Canonical Principle Rule 6). NOT BLOCKING but in-scope.

### F-LP8-OBS-002 — Codification candidate: `lifecycle_status-drift-pattern` pattern is now confirmed across 8 BC files (BC-2.22.001 + 6 plugin BCs + BC-2.17.002) `[process-gap]`

- **Severity:** OBS `[process-gap]`
- **Evidence:** F-LP7-LOW-001 closed BC-2.22.001; F-LP8-HIGH-001 surfaces 6 plugin BCs; F-LP8-LOW-001 surfaces BC-2.17.002. Same root cause: ADR-025 sweep at BC-INDEX v4.62 reset `status:` field while leaving `lifecycle_status:` unchanged across many BCs.
- **Why it matters:** Recurrent process-gap. Codification target: add a state-manager invariant check that `status:` and `lifecycle_status:` are consistent OR explicitly mark when divergence is expected.
- **Fix-routing:** session-reviewer (post-cycle) → orchestrator (codification follow-up at cycle close).

## §7 Trajectory Analysis

Trajectory: 16 → 8 → 6 → 4 → 0 → 4 → 7 → 4.

Pass-7 (7 findings) was anti-convergence. Pass-8 (4 actionable + 2 OBS) returns to decline. Sibling-sweep gaps and cross-doc consistency drifts are the dominant remaining issue class. Predict pass-9 (after fix-burst-7): likely 1–3 findings, mostly observation-class, IF sibling-sweep is executed thoroughly. Convergence reachable in 2–3 more passes if pattern-driven fixes (6-BC sweep) are executed cleanly.

## §8 Verdict & Next Action

- **Verdict: BLOCKED-hard** — 1 HIGH (sibling-sweep on 6 BCs + story falsehood) + 2 MEDIUM + 1 LOW + 2 OBS.
- **Streak: 0/3 → 0/3** (HOLD).
- **Next dispatch: fix-burst-7** — Stage 1A (PO 6-BC sweep + plugin_load_unsigned level adjudication); Stage 1B (architect ADR-022 cross-ref); Stage 2 (story-writer AC-9 trace + Catalog Level + line 16); Stage 3 (state-manager LAST single-commit per TD-VSDD-053). Then pass-9 to attempt streak advance 0/3 → 1/3.
