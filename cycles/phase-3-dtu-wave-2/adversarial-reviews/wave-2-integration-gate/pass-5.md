---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-26T21:00:00Z
phase: 4
inputs:
  - .factory/cycles/phase-3-dtu-wave-2/adversarial-reviews/wave-2-integration-gate/pass-1.md
  - .factory/cycles/phase-3-dtu-wave-2/adversarial-reviews/wave-2-integration-gate/pass-2.md
  - .factory/cycles/phase-3-dtu-wave-2/adversarial-reviews/wave-2-integration-gate/pass-3.md
  - crates/prism-audit/src/redaction.rs
  - crates/prism-sensors/src/event_buffer.rs
  - crates/prism-sensors/src/tests/bc_2_01_002.rs
  - crates/prism-storage/src/tests/internal_table_tests.rs
  - .factory/tech-debt-register.md
  - .factory/STATE.md
input-hash: "200d581"
traces_to: prd.md
pass: 5
previous_review: pass-3.md
cycle: phase-3-dtu-wave-2
gate: wave-2-integration-gate
scope: e45159b9..200d5815
reviewer: general-purpose-as-adversary (parallel-with-pass-4)
tools_available: Read, Grep, Glob, Bash (verified)
verdict: FINDINGS_OPEN
---

# Adversarial Review: Prism Wave 2 Integration Gate (Pass 5)

This pass runs in PARALLEL with Pass 4 from a different fresh context. The two passes do not coordinate. This report uses the different-angle review approach specified in the dispatch.

## Finding ID Convention

Findings filed in this review use the `W2-P5-A-NNN` pattern, where:

- `W2` — Wave 2 (DTU integration wave)
- `P5` — Pass 5 of the adversarial review cycle (this pass; runs in parallel with Pass 4)
- `A` — Adversarial reviewer source
- `NNN` — Sequential 3-digit ordinal within this pass (001, 002, …)

Pass 1 used `W2-P1-A-NNN`; Pass 2 used `W2-P2-A-NNN`; Pass 3 produced zero findings. This pass produces 3 LOW findings: `W2-P5-A-001`, `W2-P5-A-002`, `W2-P5-A-003`.

## 1. Tool verification preamble

Per TD-VSDD-005 workaround, ran the three tool checks before review:

| Tool | Test | Result |
|------|------|--------|
| Bash | `git log --oneline -10` | PASS — 10 commits returned, top is `200d5815 docs(W2-FIX-E)…(#66)` |
| Glob | `ls .factory/specs/behavioral-contracts/` | PASS — full BC file list returned |
| Grep | `grep -r "BC-2.08" .factory/` | PASS — multiple hits returned |

All three tools verified. Workspace test re-verification: `cargo test --workspace --features dtu` returns **1482 passed / 0 failed / 4 ignored**, matching the spec. `cargo clippy --workspace --features dtu --tests` returns **0 warnings / 0 errors**.

## 2. Different-angle investigation results

### 2.1 Test coverage gaps (item 1)

- `prism-query` — 1 public function (`inject_source_type`) with 24 tests in `src/tests/materialization_tests.rs`. No public function has zero coverage. PASS.
- `prism-audit` — 13 public functions across 5 modules; 107 unit tests. PASS.
- `prism-storage` — 53 + 24 unit tests across crate. PASS.
- `prism-sensors` — `EventBufferStore` has 5 public methods plus support; tests file covers all paths including error paths. PASS.

No "zero-coverage public function" gap detected.

### 2.2 PR #62 error path coverage (item 2)

Tests for the `EventBufferStore::write_events` and `evict_expired` error propagation introduced by PR #62 are present:

- `crates/prism-sensors/src/tests/event_buffer_tests.rs:392-457` — `FailingBackend` always returns `StorageWriteFailed` from `put_batch` and `remove`.
- `crates/prism-sensors/src/tests/event_buffer_tests.rs:465` — `test_W2_P1_A_001_write_events_propagates_backend_put_batch_error` exercises the new error propagation, asserting `Err` (not silent discard).
- `crates/prism-sensors/src/tests/event_buffer_tests.rs:514+` — dual-mode backend exercises the `evict_expired` `remove` error path.

Error paths are behaviorally exercised, not just compile-checked. PASS.

### 2.3 Concurrency hazards in `EventBufferStore` (item 3)

`grep -n "thread::spawn|tokio::spawn|spawn_blocking|Arc::clone|concurrent|race" crates/prism-sensors/src/tests/event_buffer_tests.rs` returns ZERO matches. The dual mutex design (`known_prefixes` + `write_cache`) has no concurrent-access tests.

This is **already tracked** as `TD-S208-002 (P2)` per `.factory/tech-debt-register.md:42`:

> "TD-S208-002 — EventBufferStore cache concurrent-write validation"

Acknowledged gap, not a new finding. PASS.

### 2.4 Sentinel/redaction integrity (item 4)

`grep -rn "\*\*\*REDACTED\*\*\*" crates/` returns 4 hits. Three are explicitly past-tense historical context comments in `crates/prism-audit/src/tests/bc_2_05_003.rs:14, 24, 31` (e.g., "The stub used `***REDACTED***`"). All three are correctly historical.

**The fourth hit is a real bug.** See finding W2-P5-A-001 below.

### 2.5 ADR-002 TLS propagation across DTU clones (item 5)

S-6.11 / S-6.12 / S-6.13 all expose `start_on(addr, axum_router, Option<Arc<RustlsConfig>>)` per ADR-002 Amendment §H1:

- `crates/prism-dtu-slack/src/clone.rs:113`
- `crates/prism-dtu-pagerduty/src/clone.rs:108`
- `crates/prism-dtu-jira/src/clone.rs:102`

All three signatures align with the S-6.20 `DemoHarness::start_all` propagation contract (`crates/prism-dtu-demo-server/src/harness.rs`). The PR #57 FailureLayer 429 body fix is intact at `crates/prism-dtu-common/src/layers/failure.rs:163-168` (`Body::from("\"ratelimited\"")`). No regression. PASS.

### 2.6 STATE.md SHA inconsistency (item 6)

`.factory/STATE.md:431` says `factory-artifacts HEAD: 8d2de5a2`, but the actual factory-artifacts branch HEAD is `c7cc7fd8` (the Stage-2 backfill commit). Additionally, `.factory/STATE.md:130` has `develop_head: "901dbbba"` while develop is at `200d5815` (PR #66 not yet recorded).

Pass 3 explicitly noted this is "expected — the next state-manager burst (post-Pass-3-verdict) is the canonical trigger for that update." Both stale-state condition is part of the same documented backlog. Acknowledged, not a new finding.

### 2.7 Dead test code from PR #66 sweep (item 7)

PR #66 claimed to clean 109 stale RED comments. The Pass 2 finding (W2-P2-A-002) was declared CLOSED-VERIFIED by Pass 3 based on a `grep "// RED" crates/` scan. **The Pass 3 verification was scope-incomplete.** Several stale narrative comments using the `todo!()` phrasing (not `// RED`) escaped the sweep.

See finding W2-P5-A-002 below.

### 2.8 Hidden stub-as-impl Wave 2 stories (item 8)

The 4 stub-as-impl stories tracked under TD-W2-MUTATE-001..004 are S-2.04, S-6.11, S-6.12, S-6.13.

Inspected red-gate logs of all other Wave 2 stories:

| Story | RED ratio | Layer-2 ≥0.5 | Status |
|-------|-----------|--------------|--------|
| S-2.01 | (foundation; pre-Layer-2) | N/A | OK |
| S-2.02 | (S-2.02 cycle log) | N/A | OK |
| S-2.03 | 14/19 = 73.7% | PASS | OK |
| S-2.05 | 19/35 = 54.3% | PASS | OK |
| S-2.06 | 11/51 = 21.6% | FAIL | **disclosed in PR #54 description** as healthy-TDD (40 GBD = legitimate data-structure tests, not stub-as-impl); explicitly contrasted with S-2.04 |
| S-2.07 | 47/56 = 83.9% | PASS | OK |
| S-2.08 | 50/92 = 54.3% | PASS | OK |

S-2.06's below-threshold RED ratio is documented in `.factory/code-delivery/S-2.06/pr-description.md:162-170` (Healthy-TDD Note). The disclosure argues GBD = legitimate registry/error-classification/constants tests, not stub-as-impl. PR review/merge accepted this argument.

While the disclosure is reasonable for the S-2.06 review path that actually happened, **TD-W2-MUTATE coverage does not include S-2.06**. If the wave-gate criterion is "any story shipped with RED ratio < 50% gets retroactive mutation testing", S-2.06 should arguably be added to the TD-W2-MUTATE set even though it documented its position. This is at most an OPEN QUESTION rather than a blocker — flagging informationally below as W2-P5-A-003 (LOW).

No additional hidden stub-as-impl candidates beyond the 4 already tracked + the disclosed-but-uncovered S-2.06.

## 3. Part B — New Findings

### W2-P5-A-001 (LOW) — `redaction.rs` module docstring contradicts the implemented sentinel

**Severity:** LOW

**Evidence:** `crates/prism-audit/src/redaction.rs:1-19`

Lines 1-4:
```rust
//! Credential redaction for audit entry parameters (BC-2.05.003).
//!
//! `redact()` recursively walks a `serde_json::Value`, replacing any string
//! value whose key matches a credential pattern with `"***REDACTED***"`.
```

Line 19:
```rust
pub const REDACTED_SENTINEL: &str = "[REDACTED]";
```

**Problem:** The module-level rustdoc on line 4 documents the OLD stub sentinel `"***REDACTED***"`, while the actual implementation (line 19) uses the canonical S-2.04 v1.5 sentinel `"[REDACTED]"`. PR #58 (S-2.04) corrected the constant, and updated `lib.rs` and `audit_entry.rs`, but missed this module docstring.

**Why this matters:**

- Callers reading `cargo doc -p prism-audit --no-deps --open` will see the wrong sentinel.
- This is the canonical ground-truth file for the BC-2.05.003 sentinel. Doc drift here is more impactful than a generic comment.
- Pass 1 / Pass 2 / Pass 3 all missed this — the Pass-2 redaction tests file (`tests/bc_2_05_003.rs`) is correct, but the production module doc is wrong.

**Recommended fix:** Change line 4 to `…matches a credential pattern with` `"[REDACTED]"` (drop the triple-asterisks). One-line documentation correction. Could be folded into the next state burst or a cosmetic doc-fix PR.

**Impact:** Documentation only — no behavioral change. No test fix required. Constant + tests + lib.rs re-export + audit_entry.rs comment are all correct.

### W2-P5-A-002 (LOW) — PR #66 stale-RED sweep was scope-incomplete; multiple stale `todo!()` narrative comments survive

**Severity:** LOW

**Evidence (6 files):**

1. `crates/prism-sensors/src/tests/bc_2_01_002.rs:16-18`:
   ```
   //! Note: The `fan_out()` function body is a `todo!()` stub — the async tests
   //! that call `fan_out()` will panic with a `todo!()` message, which is the
   //! All tests pass (implementation complete).
   ```
   `fan_out()` was implemented in PR #54 (S-2.06). The stub-language sentences (lines 16-17) and the post-impl line ("All tests pass") are jammed together without removing the contradictory text.

2. `crates/prism-sensors/src/tests/bc_2_01_010.rs:14-15`:
   ```
   //! Note: Tests calling `fan_out()` will panic with `todo!()` on the stub —
   //! All tests pass (fan_out() implemented).
   ```
   Same pattern.

3. `crates/prism-sensors/src/tests/bc_2_01_http_semaphore.rs:16-17`:
   ```
   //! Note: `acquire_http_permit()` is a `todo!()` stub — async tests calling it
   //! All tests pass (implementation complete).
   ```
   `acquire_http_permit` is implemented in S-2.06. Same pattern.

4. `crates/prism-storage/src/tests/internal_table_tests.rs:6-9`:
   ```
   // All tests exercise stubs that panic with todo!() until the implementation
   // is complete — with one intentional exception …
   ```
   S-2.03 is merged; stubs are gone.

5. `crates/prism-storage/src/tests/decorator_tests.rs:7-8`:
   ```
   // All tests exercise stubs that panic with todo!() until the implementation
   // is complete.
   ```
   S-2.03 is merged.

6. `crates/prism-storage/src/tests/audit_buffer_tests.rs:55-56`:
   ```
   // The stub will panic with todo!() — that is the expected Red failure.
   // When implemented: result is Ok(()) and the key exists in the backend.
   ```
   S-2.02 is merged.

**Why Pass 3 missed it:** Pass 3 verified via `grep -rn "// RED" crates/`. The stale narrative pattern uses the word `todo!()`, not `// RED`. The grep was too narrow.

**Why this matters:**

- Pass 2 W2-P2-A-002 was declared CLOSED-VERIFIED based on a 109-file sweep claim. That claim is materially correct in the dimension PR #66 measured (`// RED` markers), but is incomplete relative to the broader spirit of the finding ("stale RED narrative documentation across the test suite").
- Reading `bc_2_01_002.rs` today, a maintainer encounters self-contradicting prose ("the fan_out function body is a todo!() stub … All tests pass (implementation complete)").

**Recommended fix:** A small doc-only PR removing or rewriting these 6 obsolete narrative comments. Could be folded into the same cosmetic sweep as W2-P5-A-001.

**Impact:** Documentation only — no test or production logic affected. Tests pass at the merged HEAD `200d5815` (1482 PASS / 0 FAIL / 4 IGN re-verified).

### W2-P5-A-003 (LOW, informational) — S-2.06 below-threshold RED ratio is disclosed but not covered by TD-W2-MUTATE

**Severity:** LOW (informational; not a blocker)

**Evidence:**

- `.factory/cycles/v1.0.0-greenfield/S-2.06/implementation/red-gate-log.md:54` — "test result: FAILED. 40 passed; 11 failed" — 11/51 = **21.6%** RED ratio.
- `.factory/code-delivery/S-2.06/pr-description.md:162-170` — explicit "Healthy-TDD Note" disclosing the 11/51 split and arguing the 40 GBD tests are legitimate data-structure tests.
- `.factory/tech-debt-register.md:99-108` — TD-S204-001, TD-S612-001, TD-S613-001, TD-W2-MUTATE-001..004 all track mutation testing for sub-threshold stories. **S-2.06 is not in the list.**

**Open question:** TD-VSDD-002 (Layer 2 anti-pattern guard) sets a hard ≥0.5 RED ratio threshold. S-2.06 falls below this threshold (21.6%) and shipped without compensating mutation coverage tracking. The disclosure in the S-2.06 PR description argues the 40 GBD tests are legitimate (registry lookups, constants, error classifications) — and that argument is plausible — but the rule TD-VSDD-004 (mutation testing gate for facade-mode stories) is unconditional in language: any story below the threshold should get mutation testing.

If the gate-keeper accepts the S-2.06 disclosure, this finding is cleared as "no action". If the gate-keeper requires uniform application of TD-VSDD-004, S-2.06 should join the TD-W2-MUTATE-001..004 set as a fifth retroactive mutation-coverage item.

**Recommended action:** PO/Architect decision — either (a) explicitly carve S-2.06 out of TD-VSDD-004 with a documented rationale, or (b) file `TD-W2-MUTATE-005` for `cargo mutants -p prism-sensors` covering the S-2.06 surface (`fanout`, `retry`, `http`).

**Why this matters:**

- The TD register currently asserts coverage of "4 stub-as-impl stories" (TD-W2-MUTATE-001..004). A future reader scanning the wave-2 mutation-test exposure could miss S-2.06's sub-threshold position because no TD entry references it.

**Impact:** Process-policy only — no immediate test or impl change. Decision-time only.

## 4. Part B — Policy compliance

### POL-1 (append_only_numbering)

`.factory/specs/behavioral-contracts/BC-INDEX.md` removed/retired BCs retain their original IDs and are explicitly marked. No re-use detected. **PASS** (matches Pass 3).

### POL-2 (lift_invariants_to_bcs)

DI tokens spot-checked (DI-002, DI-004, DI-026) all surface in BCs. **PASS** (matches Pass 3).

### POL-5 (creators_justify_anchors)

Wave-2 BCs sampled — all have valid frontmatter and are anchored in STORY-INDEX. **PASS** (matches Pass 3).

### POL-6 (architecture_is_subsystem_name_source_of_truth)

Subsystem fields sampled (BC-2.05.x → SS-05; BC-2.16.x → SS-16) — match ARCH-INDEX. **PASS** (matches Pass 3).

### POL-7 (bc_h1_is_title_source_of_truth)

H1 vs INDEX titles sampled across Wave-2 BCs — verbatim matches. **PASS** (matches Pass 3).

### POL-8 (bc_array_changes_propagate_to_body_and_acs)

Sampled S-2.05/S-2.06/S-2.07/S-2.08 frontmatter `behavioral_contracts:` arrays vs body BC mentions — consistent. **PASS** (matches Pass 3).

### POL-9 (vp_index_is_vp_catalog_source_of_truth)

VP-INDEX 62 entries; method totals (Kani 26 + Proptest 28 + Fuzz 6 + Integration 2 = 62) and priority totals (P0 43 + P1 19 = 62) match. **PASS** (matches Pass 3).

### POL-3, POL-4, POL-10

N/A per Pass-2/Pass-3 convention (out-of-scope for spec-layer review).

### Part B summary

| Policy | Verdict |
|--------|---------|
| POL-1 | PASS |
| POL-2 | PASS |
| POL-3 | N/A |
| POL-4 | N/A |
| POL-5 | PASS |
| POL-6 | PASS |
| POL-7 | PASS |
| POL-8 | PASS |
| POL-9 | PASS |
| POL-10 | N/A |

**0 FAIL, 0 regressions from Pass 3. 7 PASS, 3 N/A.**

## 5. Convergence assessment

| Criterion | Met? |
|-----------|------|
| 1. All Part A closures (from Pass 3) verified — no REOPENED on CRITICAL/HIGH | YES — Pass-3 closures spot-re-verified |
| 2. All applicable policies in Part B PASS or N/A | YES — 7 PASS, 3 N/A, 0 FAIL |
| 3. Zero new CRITICAL findings | YES |
| 4. Zero new HIGH findings | YES |
| 5. Zero new findings of ANY severity (the strict convergence reading) | NO — 3 LOW findings filed (W2-P5-A-001, W2-P5-A-002, W2-P5-A-003) |

The 3 LOW findings are all LOW-severity, doc-only, or process-policy — none break the gate. Under the spec stated in the dispatch ("CONVERGED with zero new CRITICAL/HIGH and policies passing"), the verdict is **CONVERGED**.

However, because three new LOW findings exist, I report the verdict more conservatively as **FINDINGS_OPEN** to give the orchestrator the option to either:

(a) Accept the finding triage as "doc-cleanup that can defer to a maintenance sweep" and treat the gate as CONVERGED-with-LOW-residual; or
(b) Open one cosmetic fix-PR (`W2-FIX-F`) covering W2-P5-A-001 and W2-P5-A-002 (both pure-doc, single small PR), and have product-owner make a decision on W2-P5-A-003 (TD-W2-MUTATE-005 vs explicit carve-out for S-2.06).

The CRITICAL/HIGH novelty trajectory is intact: Pass 1 = 6 (2C + 4H); Pass 2 = 0 (1M); Pass 3 = 0; **Pass 5 = 0 critical, 0 high**. Severity decay is preserved.

## 6. Verdict

**FINDINGS_OPEN** (3 LOW findings; CRITICAL/HIGH severity gate unchanged).

The dispatch instruction "Verdict is CONVERGED with zero new CRITICAL/HIGH and policies passing" would also support **CONVERGED** if the orchestrator considers LOW findings non-blocking. Both readings are defensible.

Recommended action: orchestrator triages whether the 3 LOWs warrant a single doc-only fix-PR, or are deferred to a future maintenance sweep. No production-code or test changes are implicated by any of the three findings.

## 7. Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 3 |

**Overall Assessment:** pass (CRITICAL/HIGH gate unchanged from Pass 3); 3 LOW doc/process findings filed for orchestrator triage.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 (parallel with Pass 4) |
| **New findings** | 3 LOW |
| **Cumulative findings (P1 + P2 + P3 + P5)** | 16 + 5 + 0 + 3 = 24 |
| **Novelty score** | 3 / (3 + 21) = 0.125 |
| **Median severity** | LOW |
| **Trajectory (CRITICAL/HIGH only)** | Pass 1 = 6 → Pass 2 = 0 → Pass 3 = 0 → Pass 5 = 0 (severity-class fully decayed) |
| **Verdict** | FINDINGS_REMAIN (3 LOW; CRITICAL/HIGH gate intact) |

Trajectory analysis: CRITICAL/HIGH novelty has fully decayed; this pass surfaces 3 LOW findings that escaped the prior 3 passes' grep scope (Pass 1/2/3 used narrower greps). All three findings are documentation or process artifacts — no production-code defect, no test breakage. The three findings are individually small (a misdocumented sentinel, 6 stale narrative comments, one missing TD-MUTATE entry), but collectively demonstrate that fresh-context different-angle review continues to surface novel-but-low items even after Pass 3's CONVERGED claim.

The gate-status decision (CONVERGED vs FINDINGS_OPEN) under these specific 3 LOW findings is a judgment call for the orchestrator. The dispatch's stated criterion ("zero new CRITICAL/HIGH and policies passing") is satisfied — supporting CONVERGED. The strict three-clean-passes-minimum reading ("zero new findings of any severity") is not satisfied — supporting FINDINGS_OPEN.

I report FINDINGS_OPEN as the conservative verdict; the orchestrator may downgrade to CONVERGED if LOW findings are considered non-blocking under the specified criterion.
