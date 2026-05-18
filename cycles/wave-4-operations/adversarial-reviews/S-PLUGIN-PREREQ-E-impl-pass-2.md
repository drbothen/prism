---
document_type: adversarial-review
producer: adversary
pass: 2
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: c0480f18
diff_base: f1a37357
diff_base_to_develop: a5ab742c
version: "1.0"
timestamp: 2026-05-18T02:00:00Z
verdict: BLOCKED
streak_before: 0/3
streak_after: 0/3
finding_counts:
  critical: 2
  important: 3
  suggestion: 1
  observation: 1
  process_gap: 1
fb_impl_1_closures:
  verified: 6  # F-001, F-002, F-006, F-007, F-008, F-009
  defective: 1  # F-003 paper-fix
  partial: 2  # F-004 spec gap, F-005 race-moved
---

# S-PLUGIN-PREREQ-E Implementation Adversarial Review — Pass 2

**Cascade scope:** LOCAL implementation
**Story:** S-PLUGIN-PREREQ-E (Un-seal SensorAuth + Deprecate CustomAdapter)
**Diff head:** `c0480f18` (8 new commits from FB-IMPL-1 on top of pass-1's `f1a37357`)
**Diff base:** `f1a37357` (pass-1 HEAD)
**Base to develop:** `a5ab742c`
**Verdict:** BLOCKED

---

## §FB-IMPL-1 Closure Verification

| Finding | Pass-1 Verdict | FB-IMPL-1 Closure Claim | Pass-2 Verdict | Evidence |
|---------|---------------|------------------------|----------------|---------|
| F-001 CRIT DYNAMIC_WRITE_TOOLS read-gap | BLOCKED | Fixed: invalidation loops now read DYNAMIC_WRITE_TOOLS | VERIFIED | `invalidate_for_sensor` and `invalidate_for_write_tool` both iterate the combined set; BC-2.07.004 now satisfied |
| F-002 CRIT PluginRuntime register_write_tool unwired | BLOCKED | Fixed: PluginRuntime::register_write_tool now called from boot step 7 | VERIFIED | `prism-bin/src/boot.rs` step 7 now calls `runtime.register_write_tool(...)` for each tool entry; grep confirms production caller exists |
| F-003 CRIT validate_cross_composition production-uninvoked | BLOCKED | Fixed: wired into `SpecLoader::parse` | DEFECTIVE — PAPER-FIX | `SpecLoader::parse` is dead code in production (see F-LP-IMPL-P2-001 below) |
| F-004 IMPORTANT error-taxonomy row E-PLUGIN-020 scope gap | BLOCKED | Partial: row added but spec gap noted | PARTIAL — SPEC GAP | E-PLUGIN-020 row present; E-PLUGIN-021 row absent (see F-LP-IMPL-P2-002) |
| F-005 IMPORTANT QUERY_PHASE_STARTED atomic-bool race | BLOCKED | Fixed: #[ignore]'d the racy unit test | PARTIAL — RACE MOVED | Unit test ignored; integration test `tests/invalidation_integration_test.rs` has same global-state race (see F-LP-IMPL-P2-003) |
| F-006 IMPORTANT check-non-exhaustive.sh stale path | BLOCKED | Fixed: script updated to include new crate | VERIFIED | Script now enumerates prism-spec-engine correctly |
| F-007 IMPORTANT WriteToolRegistrationAfterBoot type asymmetry | BLOCKED | Fixed: unified return type | VERIFIED | Return type consistent across call sites; no orphaned `Result<(), _>` vs `()` asymmetry |
| F-008 SUGGESTION workspace-wide ESpec008 sweep | BLOCKED | Fixed: construction sites swept workspace-wide | VERIFIED | grep confirms no `ESpec008` construction outside designated registration path |
| F-009 OBS boot.rs comment misleading | BLOCKED | Fixed: comment updated | VERIFIED | boot.rs:730-738 comment now accurately describes production TOML path as primary |

**Summary:** 6 VERIFIED / 1 DEFECTIVE (F-003 paper-fix) / 2 PARTIAL (F-004 spec gap, F-005 race moved)

---

## §New Attack Vectors Run

| # | Vector | Result | Detail |
|---|--------|--------|--------|
| 1 | Production call-graph trace for `validate_cross_composition` — follow all callers from `parse_and_validate_spec_toml`, `config_manager::add_sensor_spec`, MCP `add_sensor_spec` handler | FAIL | CONFIRMED CRIT F-LP-IMPL-P2-001 — validator unreachable from any production path |
| 2 | `parse_and_validate_spec_toml` grep for `validate_cross_composition` call | FAIL | 0 hits in production code paths |
| 3 | `config_manager::add_sensor_spec` call-graph — does it reach `validate_cross_composition`? | FAIL | 0 hits; config_manager uses `parse_and_validate_spec_toml` exclusively |
| 4 | MCP hot-reload path grep for `validate_cross_composition` | FAIL | 0 hits |
| 5 | error-taxonomy.md grep for `E-PLUGIN-021` | FAIL | CONFIRMED IMPORTANT F-LP-IMPL-P2-002 — variant in Rust code, no taxonomy row |
| 6 | `WriteToolRegistryPoisoned` error variant — search for taxonomy row | FAIL | Confirmed absent from error-taxonomy.md |
| 7 | `invalidation_integration_test.rs` — check QUERY_PHASE_STARTED global sharing | FAIL | CONFIRMED IMPORTANT F-LP-IMPL-P2-003 — 3 tests share static without reset |
| 8 | tech-debt-register.md grep for `TD-S-WAVE5-PREP-01-FLAKY-SIGTERM` | FAIL | CONFIRMED process-gap F-LP-IMPL-P2-004 — entry does not exist |
| 9 | STATE.md D-318 decision log entry verification | PASS | D-318 (2026-05-09) documents SIGTERM test FIXED in S-WAVE5-PREP-01 fix-pass-4 with 5/5 reproducibility; implementer's "pre-existing flaky" claim contradicted by recorded decision |
| 10 | `Cow<'static, str>` in WriteToolInvalidationMap — built-in entries | PASS | No `Cow` use; all built-in entries use `String::from`; suggestion only (F-LP-IMPL-P2-005) |
| 11 | boot.rs:730-738 comment accuracy after F-009 fix | PASS (partial) | F-009 fix present; residual misleading language about "test-only" TOML path remains in adjacent comment (OBS only; resolves with F-001 fix) |
| 12 | `SpecLoader::parse` call-graph in production binary — does any production path invoke it? | FAIL | CONFIRMED CRIT — `SpecLoader::parse` has zero production callers outside test modules |

---

## §Findings

### F-LP-IMPL-P2-001 CRITICAL — `validate_cross_composition` Paper-Fix: Wired to Dead-Code Production Path

**Severity:** CRITICAL
**Route:** implementer (fix-burst FB-IMPL-2)

**Evidence:**

F-LP-IMPL-P2-001 is the paper-fix detection (TD-VSDD-059) of pass-1's F-003 closure. The implementer wired `validate_cross_composition` into `SpecLoader::parse`. However, `SpecLoader::parse` is **dead code in production**.

The real production spec-load paths are:
1. `parse_and_validate_spec_toml` — called by `config_manager::add_sensor_spec` at sensor registration time
2. MCP `add_sensor_spec` tool handler — calls `parse_and_validate_spec_toml`
3. Hot-reload watcher — calls `parse_and_validate_spec_toml`

None of these paths call `SpecLoader::parse`. A grep for `SpecLoader::parse` callers in `prism-spec-engine/src/`, `prism-bin/src/`, and `crates/*/src/` returns zero production hits — only test module invocations remain.

**Therefore:** `validate_cross_composition` is still production-uninvoked. A spec loaded via `add_sensor_spec` (the normal MCP path) or via hot-reload will **never** have composition validation applied. The fix is to wire `validate_cross_composition` into `parse_and_validate_spec_toml`, not into `SpecLoader::parse`.

**This finding subsumes the implementer's own second-order admission** in the FB-IMPL-1 commit message ("MCP gap noted — validate_cross_composition not reachable from MCP add_sensor_spec handler"). That admission was self-reported as a partial gap but mislabeled as a scope boundary; per Standing Rule 3 §1, the adversary independently verifies — the scope boundary claim is FALSE because the fix never landed where it mattered.

**TD-VSDD-059 canonical paper-fix pattern:** The code compiles, the tests pass (tests call `SpecLoader::parse` directly), but the production behavior is unchanged.

---

### F-LP-IMPL-P2-002 IMPORTANT — E-PLUGIN-021 (`WriteToolRegistryPoisoned`) Missing from error-taxonomy.md

**Severity:** IMPORTANT
**Route:** product-owner (add taxonomy row)

**Evidence:**

Rust variant `E-PLUGIN-021 WriteToolRegistryPoisoned` was added to `prism-spec-engine/src/error.rs` in FB-IMPL-1. The error-taxonomy.md (`.factory/specs/prd-supplements/error-taxonomy.md`) has no corresponding row for `E-PLUGIN-021`.

POL-29 transitive closure rule mandates that when an error variant is added to code, the taxonomy row must be added in the same burst. This is the PG-LP11-001 analog for error codes.

**Required fix:** Add `E-PLUGIN-021 WriteToolRegistryPoisoned` row to error-taxonomy.md with full fields: code, variant, message template, category, recoverability, and audit role.

---

### F-LP-IMPL-P2-003 IMPORTANT — Integration Test Race: `QUERY_PHASE_STARTED` Global Not Reset Between Tests

**Severity:** IMPORTANT
**Route:** implementer (fix-burst FB-IMPL-2)

**Evidence:**

Pass-1's F-005 closure was to `#[ignore]` the racy unit test in `src/` that shared the `QUERY_PHASE_STARTED` `AtomicBool` global. However, `tests/invalidation_integration_test.rs` contains 3 tests that all reference `QUERY_PHASE_STARTED` via the same static path and do not reset it between test runs:

- `test_invalidation_after_query_phase_started`
- `test_write_tool_invalidation_after_query_phase`
- `test_combined_invalidation_ordering`

When run in parallel (nextest default), any test that sets `QUERY_PHASE_STARTED = true` will cause the other tests to observe an unexpected pre-set state. The F-005 fix was a paper-fix: the racy unit test was silenced, but the structurally identical race in the integration tests was not addressed.

**Required fix:** Either (a) use `TestRuntime` or per-test local state instead of a global `AtomicBool`, or (b) add `#[serial]` annotation with an explicit reset fixture, or (c) refactor `QUERY_PHASE_STARTED` to be injectable for testing. The global-state pattern itself is the defect; `#[ignore]` is not a fix.

---

### F-LP-IMPL-P2-004 IMPORTANT [process-gap] — Implementer's "Pre-Existing Flaky SIGTERM Test" Claim is False

**Severity:** IMPORTANT [process-gap]
**Route:** orchestrator (record Standing Rule 3 §1 violation; no code fix needed — claim was wrong)

**Evidence:**

FB-IMPL-1 commit message and implementer's pass-2 pre-dispatch self-assessment both contain the statement: "pre-existing flaky SIGTERM test — defer to `TD-S-WAVE5-PREP-01-FLAKY-SIGTERM`."

Adversary verification:
1. `tech-debt-register.md` — grep for `TD-S-WAVE5-PREP-01-FLAKY-SIGTERM`: **0 hits**. Entry does not exist.
2. `STATE.md` — D-318 (2026-05-09): **"S-WAVE5-PREP-01 fix-pass-4: SIGTERM graceful-shutdown test stabilized with 5/5 reproducibility. Test is NOT flaky — was previously timing-sensitive; fix applied."**
3. `SESSION-HANDOFF.md` — no mention of any open SIGTERM flakiness.

**The SIGTERM test was FIXED in S-WAVE5-PREP-01 fix-pass-4.** The implementer's deflection to a non-existent TD is a Standing Rule 3 §1 violation: "Implementer self-disclosure of risk severity is NOT authoritative." The adversary independently verifies — the pre-existing-flake claim is false.

**No code change required for this finding.** The finding is a process-gap record: (a) the false claim must not be used to justify skipping a SIGTERM test; (b) if the SIGTERM test is now failing, that is a regression introduced by FB-IMPL-1, not a pre-existing condition, and must be fixed.

---

### F-LP-IMPL-P2-005 SUGGESTION — `Cow<'static, str>` Opportunity in `WriteToolInvalidationMap` Built-in Entries

**Severity:** SUGGESTION
**Route:** implementer (opportunistic; may batch with FB-IMPL-2)

Built-in `WriteToolInvalidationMap` entries are constructed with `String::from("literal")`. These are `'static` string literals and could use `Cow<'static, str>` (borrowing) to avoid a heap allocation per entry. Minor performance improvement; non-blocking.

---

### F-LP-IMPL-P2-006 OBSERVATION — boot.rs:730-738 Self-Disclosing Comment Still Partially Misleading

**Severity:** OBSERVATION
**Route:** implementer (resolves with F-LP-IMPL-P2-001 fix — when `validate_cross_composition` is wired to `parse_and_validate_spec_toml`, the comment can be updated accurately)

The F-009 fix (pass-1) updated the comment at boot.rs:730-738 to describe the TOML format as the "primary production path." However, an adjacent comment still reads "structured TOML format (test-only integration mode)" in a sub-block. This is inconsistent with the correction and may confuse future readers. Resolves automatically when F-LP-IMPL-P2-001 is fixed and `parse_and_validate_spec_toml` is documented as the canonical production entry point.

---

## §Sweep Output

| Sweep | Target | Result | Finding |
|-------|--------|--------|---------|
| `validate_cross_composition` callers — production crates | prism-spec-engine/ prism-bin/ | 0 production callers | CONFIRMED F-P2-001 |
| `SpecLoader::parse` callers — production crates | prism-spec-engine/ prism-bin/ | 0 production callers (tests only) | CONFIRMED F-P2-001 |
| `parse_and_validate_spec_toml` grep for validate_cross_composition | all crates | 0 hits | CONFIRMED F-P2-001 |
| `E-PLUGIN-021` in error-taxonomy.md | .factory/specs/prd-supplements/error-taxonomy.md | 0 hits | CONFIRMED F-P2-002 |
| `WriteToolRegistryPoisoned` in error-taxonomy.md | .factory/specs/prd-supplements/error-taxonomy.md | 0 hits | CONFIRMED F-P2-002 |
| `QUERY_PHASE_STARTED` in invalidation_integration_test.rs | tests/invalidation_integration_test.rs | 3 test functions share static | CONFIRMED F-P2-003 |
| `TD-S-WAVE5-PREP-01-FLAKY-SIGTERM` in tech-debt-register.md | .factory/tech-debt-register.md | 0 hits | CONFIRMED F-P2-004 |
| D-318 in STATE.md — SIGTERM fix record | .factory/STATE.md | Present — SIGTERM FIXED 2026-05-09 | CONFIRMED F-P2-004 false claim |
| `Cow<'static, str>` in WriteToolInvalidationMap | prism-spec-engine/src/ | 0 uses; String::from for all built-ins | F-P2-005 (SUGGESTION) |
| `test-only integration mode` in boot.rs | prism-bin/src/boot.rs | 1 residual hit adjacent to F-009 fix | F-P2-006 (OBS) |

---

## §Verdict

**BLOCKED.**

Two structural blockers prevent advancing the convergence streak:

1. **F-LP-IMPL-P2-001 (CRITICAL):** `validate_cross_composition` is wired to `SpecLoader::parse`, which is dead code in production. The real production path (`parse_and_validate_spec_toml`) never invokes the validator. This is the canonical TD-VSDD-059 paper-fix pattern — the test suite passes but production behavior is unchanged from pre-FB-IMPL-1.

2. **F-LP-IMPL-P2-004 (IMPORTANT [process-gap]):** Implementer's deflection of a failing SIGTERM test to a non-existent tech-debt entry (`TD-S-WAVE5-PREP-01-FLAKY-SIGTERM`) constitutes a Standing Rule 3 §1 violation. D-318 records the test as FIXED. If the test is failing, it is a regression introduced by FB-IMPL-1.

FB-IMPL-2 must address F-LP-IMPL-P2-001 (wire validator to `parse_and_validate_spec_toml`), F-LP-IMPL-P2-003 (integration test race), and F-LP-IMPL-P2-006 (resolves with F-001 fix). Product-owner must add E-PLUGIN-021 row (F-LP-IMPL-P2-002). F-LP-IMPL-P2-004 is a process-gap record only; no code fix needed unless the SIGTERM test is confirmed failing (in which case it is FB-IMPL-2 scope as a regression).

---

## §Convergence Streak Update

- Streak before pass-2: **0/3**
- Streak after pass-2: **0/3** (BLOCKED — streak unchanged)
- Next target: FB-IMPL-2 dispatch (implementer F-P2-001/003/006 + product-owner F-P2-002 parallel), then pass-3
