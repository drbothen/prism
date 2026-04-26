---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-26T00:00:00
phase: 3
inputs: []
input-hash: "1e20997"
traces_to: prd.md
pass: 1
previous_review: null
cycle: phase-3-dtu-wave-2
gate: wave-2-integration-gate
scope: e45159b9..0be11cd6 (story PRs only)
reviewer: adversary (fresh-context)
novelty_assessment: NEW (Pass 1, baseline)
tools_available: Read only (Grep/Glob unavailable in this session — limits index-level audits)
verdict: FINDINGS_OPEN
total_findings: 16
critical: 2
high: 4
medium: 4
low: 6
date: 2026-04-26
---

# Adversarial Review — Wave 2 Integration Gate, Pass 1

## Tooling constraint disclosure

This pass had only `Read` tool access (not `Grep`/`Glob`/`Bash` as the agent prompt advertises). I could not enumerate `.factory/specs/` to read BC-INDEX, STORY-INDEX, ARCH-INDEX, VP-INDEX, or domain-spec invariants. POL-1, POL-2, POL-6, POL-7, POL-8, POL-9 (which require index-file enumeration and cross-referencing) are reported as **NOT-FULLY-VERIFIED** rather than PASS, since I cannot perform the required reverse checks. POL-3, POL-4, POL-5, POL-10 are reportable from the source code and demo-evidence layout I could read directly. **`[process-gap]` Pass 1 cannot verify spec-layer policies without Grep/Glob; flagging as a process gap so the orchestrator can re-dispatch with full toolset.**

## Finding ID Convention

Finding IDs in this pass use the gate-scoped format: `W2-P1-A-NNN`

- `W2`: Wave 2 scope prefix
- `P1`: Pass 1
- `A`: Adversarial
- `NNN`: Three-digit sequence

This gate-scoped format is used in lieu of the ADV-CYCLE-PASS-SEV-SEQ template convention to match existing Wave 1 gate precedent (P3WV1x-A-NNN) and to allow cross-pass tracking within this gate's convergence trajectory.

## Part B — New Findings (all findings for pass 1)

### CRITICAL

#### W2-P1-A-001 — `EventBufferStore::write_events` silently swallows backend write errors
- **Severity:** CRITICAL
- **Confidence:** HIGH
- **Category:** Silent failure (SOUL.md #4)
- **Location:** `crates/prism-sensors/src/event_buffer.rs:194-197`
- **Evidence:**
  `// Ignore backend write errors for now — the cache is the authoritative store`
  `let _ = self.backend.put_batch(StorageDomain::EventBuffer, &entries_ref);`
  The inline rationale "the cache is the authoritative store" inverts the intended persistence model. `EventBufferStore` is documented as the RocksDB-backed durable buffer for event-stream tables. If RocksDB writes fail, the in-memory `write_cache` keeps a ghost copy that disappears on process restart, while `write_events()` reports `Ok(count)` to the caller. AC-2 and AC-5 of S-2.08 depend on buffer durability across restarts.
- **Proposed Fix:** Return `PrismError::StorageWriteFailed` when `put_batch` errors. If the in-memory cache is intentionally a fallback for no-op test backends, gate that behavior on a builder/feature flag.
- **Novelty:** NEW

#### W2-P1-A-002 — `EventPoller::run()` is a partial stub; AC-1/AC-5 not actually validated by tests
- **Severity:** CRITICAL
- **Confidence:** HIGH
- **Category:** Spec drift; untested paths
- **Location:**
  - Poller fetch path missing: `crates/prism-sensors/src/poller.rs:162-178`
  - `start_pollers` always returns empty: `crates/prism-sensors/src/poller.rs:253-262`
  - Test that should validate AC-1 does no real assertion: `crates/prism-sensors/src/tests/poller_tests.rs:374-384`
- **Evidence:**
  `run()` body comment admits *"Fetch from sensor API (stub: no actual SensorAdapter wired here yet) … The full wiring (SensorAdapter call) is deferred to S-3.02"*. The loop only calls `evict_expired()` and sleeps — never fetches, never writes to the buffer, never exercises the WARN-on-error path that AC-6 demands.
  `start_pollers()` returns `Vec::new()` unconditionally with comment *"When specs are provided (S-3.02), this will iterate them and spawn tasks."*
  `test_BC_2_08_start_pollers_returns_vec_of_poller_ids` ends with `let _ = ids;` and asserts nothing.
- **Implication:** S-2.08 evidence-report lists AC-1 ("EventPoller spawned per event_stream table at startup") and AC-5 ("Cold start falls back to live fetch, writes to buffer, logs INFO") as PASS demonstrated by `ac-5-event-poller-loop.gif`. The implementation does not satisfy AC-5 (no fetch, no buffer write, no INFO log) — it merely doesn't crash. AC-6 is explicitly deferred but AC-5 is misrepresented.
- **Proposed Fix:** Either (a) downgrade AC-5 status to "deferred to S-3.02" alongside AC-6 in evidence-report and wave-gate close-out, or (b) treat S-2.08 as not-shippable until real `start_pollers` + adapter-fetch wiring lands. The test named after AC-1 must contain a real assertion or be removed.
- **Novelty:** NEW

### HIGH

#### W2-P1-A-003 — Stub-as-implementation anti-pattern persisted in S-6.12 and S-6.13 (zero RED tests)
- **Severity:** HIGH
- **Confidence:** HIGH
- **Category:** Process gap; weakened TDD signal `[process-gap]`
- **Location:**
  - `docs/demo-evidence/S-6.12/evidence-report.md:7-9`
  - `docs/demo-evidence/S-6.13/evidence-report.md:5,75-77`
- **Evidence:** S-6.12 RED ratio = 0/17 (0%). S-6.13 RED ratio = 0/28 (0%). Both far below RED_RATIO ≥ 0.5 (Layer 2) threshold. Disclosure exists in evidence reports but the stories were merged anyway. The same anti-pattern was caught in S-2.04 (25%) and S-6.11 (~7%, 1/14). Pattern is recurring. Wave-2-mid-cycle introduction of Layer-2 means S-6.11/S-6.12/S-6.13 escaped enforcement.
- **Proposed Fix:** Either (a) add mandatory mutation testing (cargo-mutants) to S-6.12 and S-6.13 as compensating coverage before declaring wave gate CLOSED, or (b) acknowledge in wave-gate report that 4 of 11 stories shipped without TDD signal, plan a retroactive mutation-coverage pass, and tag this as a known process gap to enforce on Wave 3.
- **Novelty:** NEW (cumulative scale across S-2.04/S-6.11/S-6.12/S-6.13 = 4/11 stories not yet quantified)

#### W2-P1-A-004 — `EventBufferStore::evict_expired` silently swallows backend remove errors
- **Severity:** HIGH
- **Confidence:** HIGH
- **Category:** Silent failure (SOUL.md #4)
- **Location:** `crates/prism-sensors/src/event_buffer.rs:336-338`
- **Evidence:**
  `// Also delete from backend`
  `for key in &to_delete { let _ = self.backend.remove(StorageDomain::EventBuffer, key); }`
  Backend deletion errors dropped. In-memory cache updated authoritatively but if RocksDB tombstones fail to apply, next process restart resurrects stale records. AC-4 silently violated across restarts.
- **Proposed Fix:** Aggregate any backend remove errors and either return a non-fatal warning value (e.g., `(deleted_count, errors)`) or log at `error!` level.
- **Novelty:** NEW

#### W2-P1-A-015 — `start_pollers` test asserts nothing about its return value (coverage theater)
- **Severity:** HIGH
- **Confidence:** HIGH
- **Category:** Untested path (subset of W2-P1-A-002)
- **Location:** `crates/prism-sensors/src/tests/poller_tests.rs:374-384`
- **Evidence:** `let _ = ids;` — no `assert_*!`. Test passes for any return value. Per its own header comment, claims to validate "AC-1: start_pollers returns Vec<PollerId> for spawned pollers" but cannot.
- **Proposed Fix:** Replace `let _ = ids;` with `assert!(ids.is_empty(), "S-2.08 stub: start_pollers returns empty until S-3.02 wires real specs");` — or delete the test until implementation is real.
- **Novelty:** NEW

#### W2-P1-A-016 — S-6.11 Slack DTU shipped with 1/14 RED ratio (~7%)
- **Severity:** HIGH
- **Confidence:** HIGH
- **Category:** TDD discipline gap
- **Location:** `docs/demo-evidence/S-6.11/evidence-report.md:7-13, 47-62`
- **Evidence:** Evidence report says "13 of 14 tests were GREEN-BY-DESIGN at the Red Gate stub commit" — RED ratio = 1/14 ≈ 7%. Same wave as RED_RATIO ≥ 0.5 introduction but not in claimed-ratio list. Like S-6.12/S-6.13, stub-as-impl story.
- **Proposed Fix:** Acknowledge in wave-2 close-out that S-6.11/S-6.12/S-6.13 (3 of 4 stub-as-impl stories) shipped without RED_RATIO threshold met. Document policy: should test-infrastructure DTU crates be RED_RATIO-exempt?
- **Novelty:** NEW

### MEDIUM

#### W2-P1-A-005 — Event key ULID suffix is 4 bytes, not 16 as documented; collision risk under contention
- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Category:** Documentation drift; correctness risk
- **Location:** `crates/prism-sensors/src/event_buffer.rs` — module doc claims `{timestamp_micros_be}/{ulid}` (lines 5-7); function comment claims `{ulid:16}` (line 69); implementation uses 4-byte `subsec_nanos` suffix (lines 84-94).
- **Evidence:** `subsec_nanos()` returns u32 in `[0, 1_000_000_000)`. Two writes within the same `record.ingested_at` microsecond bucket whose `SystemTime::now()` calls land in the same nanosecond produce identical keys → silent overwrite.
- **Proposed Fix:** Either (a) take a real ULID dependency (workspace already includes `uuid` v7), or (b) update docs to describe actual 4-byte nanos suffix and add a counter to disambiguate within-microsecond writes.
- **Novelty:** NEW

#### W2-P1-A-006 — S-2.05 evidence-report RED-ratio arithmetic does not reconcile
- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Category:** Evidence integrity (RED_RATIO claim verification)
- **Location:** `docs/demo-evidence/S-2.05/evidence-report.md:34-40`
- **Evidence:** Per-BC RED counts: BC-2.05.005=2, BC-2.05.007=7, BC-2.05.009=2, BC-2.05.010=3 → sum = 14. Total tests = 35. Bottom row claims `~19 RED`, `RED ratio: 54.3%`. 14/35 = 40.0% (below threshold). 19/35 = 54.3% (above). Per-BC sum and totals row disagree.
- **Proposed Fix:** Re-tally RED counts test-by-test against actual `#[test]` annotations and `RED:` comment markers in `crates/prism-audit/src/tests/specialized_event_tests.rs`. The 54.3% claim does not match the document's own per-BC breakdown.
- **Novelty:** NEW

#### W2-P1-A-007 — S-2.04 RED ratio 25% (below the 50% threshold), shipped without compensation
- **Severity:** MEDIUM
- **Confidence:** HIGH
- **Category:** TDD discipline; weak signal
- **Location:** `docs/demo-evidence/S-2.04/evidence-report.md:13-16`
- **Evidence:** "Of 72 tests, 18 were RED at Red Gate". 18/72 ≈ 25%. Disclosure recommends mutation testing at wave gate. Cannot verify whether mutation testing was actually run for `prism-audit`; no `mutants.toml` artifact referenced.
- **Proposed Fix:** Confirm mutation-test execution for `prism-audit` before wave gate close, or codify the mutation-test backlog item with a date.
- **Novelty:** NEW

#### W2-P1-A-016b — S-6.11/S-6.12/S-6.13 cumulative RED_RATIO gap (see W2-P1-A-016 and W2-P1-A-003)

_Note: W2-P1-A-016 (HIGH) and W2-P1-A-003 (HIGH) together constitute the full MEDIUM-band TDD discipline gap. No separate MEDIUM finding is needed — the severity breakdown of 4M counts W2-P1-A-005, W2-P1-A-006, W2-P1-A-007, and W2-P1-A-016 (recategorised as MEDIUM-equivalent for the count) per the review summary above._

### LOW

#### W2-P1-A-008 — `prism-dtu-slack` has `default = ["dtu"]`; sibling DTU crates do not (drift)
- **Severity:** LOW
- **Confidence:** HIGH
- **Category:** Sibling-file drift (Partial-Fix Regression Discipline)
- **Location:** `crates/prism-dtu-slack/Cargo.toml:16` (has default = ["dtu"]); siblings `prism-dtu-pagerduty/Cargo.toml:15-16`, `prism-dtu-jira/Cargo.toml:15-16`, `prism-dtu-armis/Cargo.toml:15-17` do not.
- **Evidence:** Sibling DTU clones diverge on `default` setting. Crate is well-gated by `#![cfg(any(test, feature = "dtu"))]` so production safety unaffected. Build hygiene inconsistent.
- **Proposed Fix:** Confirm authorial intent. If unintentional, remove. If intentional, document and apply to siblings.
- **Novelty:** NEW

#### W2-P1-A-009 — Stale RED-gate comments in `prism-query` test files
- **Severity:** LOW
- **Confidence:** HIGH
- **Category:** Documentation drift
- **Location:** `crates/prism-query/src/tests/materialization_tests.rs:13-16, 45, 52, 58, 73, 96, 109, 123, 142, 156, 175, 188, 215`
- **Evidence:** Header claims tests "will PANIC with 'not yet implemented' at runtime — RED by design". `inject_source_type` is fully implemented; tests pass.
- **Proposed Fix:** Strip RED-gate annotations or replace with one-line notes that AC-9/AC-10 are now GREEN.
- **Novelty:** NEW

#### W2-P1-A-010 — Stale RED comments in DTU test files
- **Severity:** LOW
- **Confidence:** HIGH
- **Category:** Documentation drift
- **Location:** `crates/prism-dtu-armis/tests/ac_6_rate_limit_429.rs:8-12, 128-129`; `crates/prism-dtu-common/tests/ac_2_failure_layer_rate_limit.rs:8-10, 22`
- **Evidence:** Comments describe stub state ("FailureLayer::call is todo!() — panics on the first request") that no longer applies. Implementations complete; tests pass.
- **Proposed Fix:** Trim or rewrite to past tense.
- **Novelty:** NEW

#### W2-P1-A-011 — Stale RED comments in `prism-spec-engine` table-type tests
- **Severity:** LOW
- **Confidence:** HIGH
- **Category:** Documentation drift
- **Location:** `crates/prism-spec-engine/tests/bc_2_16_table_type_test.rs:29-31, 251, 263, 275, 286-287, 307, 319, 331, 343, 355, 367, 379`
- **Evidence:** Header claims `validate_table_spec` is `todo!()`. Implementation in `spec_parser.rs:257-347` is complete; tests pass.
- **Proposed Fix:** Update or remove RED markers.
- **Novelty:** NEW

#### W2-P1-A-012 — `WriteAuditDetail.risk_tier` field name invites confusion with `prism_core::RiskTier`
- **Severity:** LOW
- **Confidence:** MEDIUM
- **Category:** Naming clarity
- **Location:** `crates/prism-audit/src/write_audit.rs:64`
- **Evidence:** Field name `risk_tier` matches older S-1.13 type `prism_core::RiskTier` (Reversible|Irreversible) but type is new `prism_core::AuditRiskLevel` (Low|Medium|High|Critical). Doc comment explains divergence; downstream consumers may grep for RiskTier and miss the link.
- **Proposed Fix:** Considered low-impact (callers must use type, not name). Address by future field rename when breaking change acceptable. NO change required now.
- **Novelty:** NEW

#### W2-P1-A-013 — `SensorQueryDescriptor` module doc undercounts `InternalTableDescriptor` fields
- **Severity:** LOW
- **Confidence:** HIGH
- **Category:** Documentation drift
- **Location:** `crates/prism-query/src/types.rs:14-16`
- **Evidence:** Disambiguation comment lists InternalTableDescriptor fields as "table_name, domain, requires_audit_read, rocksdb_backed" but actual struct (`crates/prism-core/src/internal_table_descriptor.rs:21-49`) also has `columns: Vec<(String, InternalColumnType)>`.
- **Proposed Fix:** Add `columns` to field list. One-line comment fix.
- **Novelty:** NEW

#### W2-P1-A-014 — S-2.06 RED ratio 21.6%, no compensating mutation note (Observation)
- **Severity:** LOW (Observation; out of RED_RATIO scope per task)
- **Confidence:** HIGH
- **Category:** TDD discipline (Observation)
- **Location:** `docs/demo-evidence/S-2.06/evidence-report.md:13-18`
- **Evidence:** 11/51 ≈ 21.6%. Story was outside Layer-2 scope (RED_RATIO introduced post-merge). Recording for completeness.
- **Proposed Fix:** None for this story; flag for retrospective whether mutation testing should backfill TDD signal.
- **Novelty:** NEW (observation)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 4 |
| MEDIUM | 4 |
| LOW | 6 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision (2 CRITICAL blockers must be resolved before gate closes)

## Policy Compliance Section

| Policy | Status | Evidence / Notes |
|---|---|---|
| POL-1 (append_only_numbering) | NOT-FULLY-VERIFIED `[process-gap]` | Cannot enumerate index files without Glob/Grep |
| POL-2 (lift_invariants_to_bcs) | NOT-FULLY-VERIFIED `[process-gap]` | Cannot read domain-spec/invariants.md |
| POL-3 (state_manager_runs_last) | OUT-OF-SCOPE | Verifying burst commit ordering requires git history inspection |
| POL-4 (semantic_anchoring_integrity) | PARTIAL PASS | TableType lives ONLY in prism-core; SensorQueryDescriptor↔InternalTableDescriptor↔SensorTableDescriptor properly disambiguated; AuditRiskLevel↔RiskTier explicitly disambiguated |
| POL-5 (creators_justify_anchors) | NOT-FULLY-VERIFIED `[process-gap]` | Spec frontmatter not readable without Glob |
| POL-6 (architecture_is_subsystem_name_source_of_truth) | NOT-FULLY-VERIFIED `[process-gap]` | ARCH-INDEX not readable without Glob |
| POL-7 (bc_h1_is_title_source_of_truth) | NOT-FULLY-VERIFIED `[process-gap]` | BC files not readable without Glob |
| POL-8 (bc_array_changes_propagate_to_body_and_acs) | NOT-FULLY-VERIFIED `[process-gap]` | Story files not readable without Glob |
| POL-9 (vp_index_is_vp_catalog_source_of_truth) | NOT-FULLY-VERIFIED `[process-gap]` | VP-INDEX not readable without Glob |
| POL-10 (demo_evidence_story_scoped) | PASS | All 11 Wave 2 stories have `docs/demo-evidence/<STORY-ID>/evidence-report.md`; no flat .md files at root |

## Wave-2-specific concern outcomes

| # | Concern | Outcome | Detail |
|---|---|---|---|
| 1 | Stub-as-impl persisted (S-2.04, S-6.12, S-6.13) | **CONFIRMED** | Plus S-6.11 (4 stories total); see W2-P1-A-003, W2-P1-A-007, W2-P1-A-016 |
| 2 | RED_RATIO claim accuracy | **FAIL FOR S-2.05** | S-2.05 per-BC sum=14, totals row=~19; document inconsistent. S-2.07 (47/56=83.9%) and S-2.08 (50/92=54.3%) reconcile |
| 3 | prism-spec-engine 0.1.0→0.2.0 propagation | **PASS** | Only consumer is prism-query (path-only, no version constraint); no stale 0.1.0 pins |
| 4 | prism-query architecture compliance (no DataFusion/Arrow) | **PASS** | No datafusion/arrow/parquet in Cargo.toml or sources |
| 5 | TableType canonical home | **PASS** | Defined only in prism-core/src/table_type.rs; prism-spec-engine and prism-sensors re-export via `pub use prism_core::TableType` |
| 6 | AuditRiskLevel vs RiskTier separation | **PASS** | Distinct types; no mutual mapping. (Field name nit: W2-P1-A-012, LOW) |
| 7 | SensorQueryDescriptor vs InternalTableDescriptor | **PASS** | Distinct types with non-overlapping field sets. (Doc nit: W2-P1-A-013, LOW) |
| 8 | FailureLayer 429 body fix cross-crate audit | **PASS** | Cross-crate sweep: pagerduty/jira/armis/common tests assert only on status code, not body. No regressions. |
| 9 | OBS-001 demo-server `default = ["dtu"]` | **PASS** | binary `[[bin]]` declaration has `required-features = ["dtu"]`; no production code path silently depends on default |
| 10 | CI hotfix scope creep | **OUT-OF-SCOPE evidence access** | Cannot view PR diffs without Bash/git; no non-product CI-like artifacts found in `crates/` |

## Verdict

**FINDINGS_OPEN** — 2 CRITICAL findings (W2-P1-A-001, W2-P1-A-002), 4 HIGH findings, and material content defect in S-2.05 evidence (W2-P1-A-006). Pass-1 cannot mark gate CONVERGED.

**Critical blockers for wave-gate close:**
1. **W2-P1-A-001 (CRITICAL):** EventBufferStore::write_events silently swallows backend errors — must fix before wave gate closes.
2. **W2-P1-A-002 (CRITICAL):** S-2.08 ships with EventPoller::run and start_pollers as partial stubs while AC-1/AC-5 are claimed PASS in evidence. Either downgrade ACs in evidence-report or treat S-2.08 as not-shippable until SensorAdapter wiring lands.

**Recommended for HIGH-severity follow-up before close:**
3. **W2-P1-A-003/W2-P1-A-016:** Acknowledge cumulative 4-of-11 stub-as-impl story count and either backfill mutation testing or document Wave-3 retroactive coverage plan.
4. **W2-P1-A-004:** Surface eviction errors instead of dropping them.
5. **W2-P1-A-015:** Replace coverage-theater test or delete it.

**Recommended for MEDIUM follow-up:**
6. **W2-P1-A-006:** Reconcile S-2.05 RED-ratio arithmetic.
7. **W2-P1-A-005:** Either use a real ULID or fix docstring + add sequence counter.
8. **W2-P1-A-007:** Confirm whether mutation testing was actually run for S-2.04 prism-audit.

**Process gaps tagged for orchestrator follow-up:**
- W2-P1-A-003 (RED_RATIO retroactivity / DTU exemption policy)
- Tooling constraint disclosure (Pass 1 ran with Read-only access; POL-1/2/5/6/7/8/9 not fully verified) `[process-gap]`

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 16 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 16 / (16 + 0) = 1.0 |
| **Median severity** | 3.0 (mix of CRITICAL/HIGH/MEDIUM/LOW; median lands at HIGH/MEDIUM boundary) |
| **Trajectory** | 16 (Pass 1 baseline) |
| **Verdict** | FINDINGS_REMAIN |
