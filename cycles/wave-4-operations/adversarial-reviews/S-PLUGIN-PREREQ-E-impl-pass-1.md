---
document_type: adversarial-review
producer: adversary
pass: 1
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: f1a37357
diff_base: "develop a5ab742c"
version: "1.0"
timestamp: 2026-05-18T00:00:00Z
verdict: BLOCKED
streak_before: "0/3"
streak_after: "0/3"
finding_counts:
  critical: 3
  important: 4
  suggestion: 1
  observation: 2
  process_gap: 1
---

# S-PLUGIN-PREREQ-E Implementation Adversarial Review — Pass 1

**Cascade scope:** LOCAL implementation
**Story:** S-PLUGIN-PREREQ-E (Un-seal SensorAuth + Deprecate CustomAdapter)
**Diff head:** `f1a37357` (7 commits ahead of `develop@a5ab742c`)
**Verdict:** BLOCKED

---

## §Attack Vectors Rotated (16 vectors)

1. AC↔implementation correctness — end-to-end behavioral guarantee satisfaction
2. Production call-path wiring — are callable functions actually called in production paths?
3. TD-VSDD-059 paper-fix detection — implementations with tests-only callers, no production wiring
4. Standing Rule 3 §1 — implementer self-disclosure verification via independent grep
5. Standing Rule 3 §4 — Arc-DI plumbing completeness; no placeholder construction
6. Error taxonomy correctness — diagnostic accuracy, operator experience
7. Test isolation — global mutable state, non-determinism risks
8. CI-as-code drift — script vs workflow vs CLAUDE.md authority discrepancies
9. Type system correctness — type mismatches blocking future wiring
10. Scope-narrowness of retirement gates — grep coverage insufficiency
11. Doc comment accuracy — Standing Rule 3 §3 factual correctness
12. BC-2.16.002 catalog coverage — audit log completeness
13. Thread-safety — RwLock poisoning handling
14. Sibling-site sweep (TD-VSDD-060) — callsite completeness
15. Integration test ordering — sequential execution requirements
16. Static variable lifecycle — module-level state across test binary boundaries

---

## §Findings

### F-LP-IMPL-P1-001 — `DYNAMIC_WRITE_TOOLS` is registered-but-never-read

**Severity:** CRITICAL
**Category:** spec↔impl correctness; broken end-to-end behavioral guarantee for AC-9

**Evidence:** `crates/prism-query/src/invalidation.rs:41` defines `static DYNAMIC_WRITE_TOOLS: RwLock<Vec<WriteToolInvalidationMap>>`. `register_write_tool` (line 117) is the sole write site and appends to it. But `CacheInvalidator::invalidate_for_sensor` (line 253) and `invalidate_for_write_tool` (line 288) iterate ONLY over `WRITE_TOOL_INVALIDATION_MAP` (the static `LazyLock`); neither reads `DYNAMIC_WRITE_TOOLS`. Workspace grep: `DYNAMIC_WRITE_TOOLS.read` and `DYNAMIC_WRITE_TOOLS.iter` return zero hits anywhere.

**Why it breaks:** Plugin-registered write tools will be silently ignored at invalidation time — a write operation performed by a plugin tool would NEVER invalidate any cache entry. This is exactly the failure-mode TD-S-PLUGIN-PREREQ-A-003 was supposed to close (BC-2.07.004 write-then-read consistency invariant for plugin tools).

**Story citation:** Task 7 mandates "Update all read-side callers (the invalidation check function) to acquire a read guard instead of dereferencing the LazyLock" — read-side update is missing.

**Suggested route:** orchestrator → implementer (extend `invalidate_for_sensor` and `invalidate_for_write_tool` to read from BOTH `WRITE_TOOL_INVALIDATION_MAP` AND `DYNAMIC_WRITE_TOOLS` under read guard; chain the two iterators).

---

### F-LP-IMPL-P1-002 — PluginRuntime never calls `register_write_tool`

**Severity:** CRITICAL
**Category:** spec↔impl correctness; AC-9 production-wiring gap; Standing Rule 3 §1 violation

**Evidence:** Workspace grep for `register_write_tool` outside `crates/prism-query/src/invalidation.rs` finds ZERO hits in `crates/prism-spec-engine/` or `crates/prism-bin/`. Doc-comment fragments at `crates/prism-spec-engine/src/error.rs:210, 221` reference the function but do NOT call it. Neither `PluginRuntime::load_all_plugins` (in `crates/prism-spec-engine/src/plugin/mod.rs`) nor `plugin_load_step_with_audit` (in `crates/prism-bin/src/boot.rs:898`) iterates plugin manifest write-tool entries.

**Story citation:** Task 7 final bullet — "Wire `PluginRuntime` (already available via PREREQ-D boot wiring) to call `register_write_tool` for each plugin that declares write-tool capabilities in its manifest". Production wiring missing.

**Suggested route:** orchestrator → implementer (wire plugin manifest's write-tool list to `register_write_tool` inside `PluginRuntime::load_plugin` or in `boot.rs` post-load-iteration; this depends on F-001 above being fixed first).

---

### F-LP-IMPL-P1-003 — `validate_cross_composition` never invoked by production spec-load path

**Severity:** CRITICAL
**Category:** spec↔impl correctness; AC-3/3b/3c production effectiveness; paper-fix smell

**Evidence:** `crates/prism-spec-engine/src/spec_parser.rs:859` declares `pub fn validate_cross_composition(...)`. Grep for callers in `crates/`: 8 tests-only hits, ZERO production hits in `crates/prism-spec-engine/src/` or `crates/prism-bin/src/`.

**Why it breaks:** AC-3/3b/3c require runtime rejection at spec-load. The validator exists as a callable function but nothing in `SpecLoader::parse`, `SpecLoader::load_all`, or the credential-validation pipeline (step 5 in `boot.rs:730`) calls it. Real `.sensor.toml` files with composite/multi-cred/mismatched auth_type WILL pass through to runtime undetected — the validators are tests-only.

**Story risk_mitigations** (line 67) explicitly states "credential-validation pass".

**TD-VSDD-059 paper-fix:** validators implemented in isolation; not wired to the load path.

**Suggested route:** orchestrator → implementer (call `validate_cross_composition` from within `SpecLoader::parse` after deserializing `SensorSpec`, OR from `step5_init_credential_store` during credential refs iteration in `boot.rs:730`).

---

### F-LP-IMPL-P1-004 — RwLock poisoning silently masks as "registration-after-boot"

**Severity:** IMPORTANT
**Category:** error taxonomy correctness; misleading diagnostic

**Evidence:** `crates/prism-query/src/invalidation.rs:133`: `.map_err(|_| SpecEngineError::WriteToolRegistrationAfterBoot)?;` — RwLock poisoning collapses into the same error as query-phase-started. Operators get misleading diagnostics.

**Suggested route:** orchestrator → implementer (introduce a distinct error variant `SpecEngineError::WriteToolRegistryPoisoned`, or expand the error type).

---

### F-LP-IMPL-P1-005 — AC-9 third-test depends on global module-state mutation; no reset hook

**Severity:** IMPORTANT
**Category:** test isolation; non-determinism risk on `cargo test` non-nextest runs

**Evidence:** `crates/prism-query/src/invalidation.rs:88` `static QUERY_PHASE_STARTED: AtomicBool = AtomicBool::new(false);` is module-level. Test at line 615 permanently flips it; tests at lines 533 and 563 require `false`. Under `cargo nextest` (one process per test) works; under `cargo test --workspace` (which CLAUDE.md acknowledges as a fallback) all tests share one binary.

**Suggested route:** orchestrator → implementer (add `#[cfg(test)] pub(crate) fn reset_query_phase_for_test()` and invoke from `_happy_path`/`_duplicate_rejected` test setup, OR mark all three tests `#[serial]`).

---

### F-LP-IMPL-P1-006 — `check-non-exhaustive.sh` EXPECTED=30 stale; ci.yml authoritative at EXPECTED=31

**Severity:** IMPORTANT [process-gap]
**Category:** CI-as-code drift; CLAUDE.md cite itself stale

**Evidence:** `scripts/check-non-exhaustive.sh:12` → `EXPECTED=30`; `.github/workflows/ci.yml:579` → `EXPECTED=31`; `CLAUDE.md:209` declares "ci.yml EXPECTED=30 is the authority" (stale).

**Suggested route:** orchestrator → implementer (bump `scripts/check-non-exhaustive.sh` EXPECTED=31; update CLAUDE.md L209 to "ci.yml EXPECTED=31").

---

### F-LP-IMPL-P1-007 — `WriteToolInvalidationMap.tool_name` is `&'static str` but `DuplicateWriteToolRegistration` carries `String`

**Severity:** IMPORTANT
**Category:** type mismatch; runtime allocation; plugin wiring blocker

**Evidence:** `crates/prism-query/src/invalidation.rs:63` `pub tool_name: &'static str;` but plugin manifest data is allocated `String`. Cannot satisfy `&'static str` without `Box::leak`. Standing Rule 3 §4 (wiring not redesign) — the type asymmetry blocks proper Arc-DI plumbing.

**Suggested route:** orchestrator → implementer (change `tool_name` to `String` to match plugin runtime; sibling-sweep `source_ids` to `Vec<String>`; update `WRITE_TOOL_INVALIDATION_MAP` initializer accordingly; part of F-002 wiring).

---

### F-LP-IMPL-P1-008 — `SpecErrorCode::ESpec008` enum variant still constructable from `prism-core` workspace-wide

**Severity:** IMPORTANT
**Category:** AC-5/AC-11 scope-narrowness; semantic vs lexical retirement

**Evidence:** `crates/prism-core/src/error.rs:898` declares `ESpec008,` as a `pub enum SpecErrorCode` variant. Red Gate Test 7 + Test 14 only grep `crates/prism-spec-engine/src/`. Neither test gates against `prism-core`.

**Orchestrator adjudication (D-700 inline):** The variant DECLARATION stays per POL-1 append-only. CONSTRUCTION sweep expands to workspace-wide. Test 14 grep should be expanded.

**Suggested route:** orchestrator → implementer (extend grep gate to all `crates/*/src/`; preserve variant declaration; do NOT touch error.rs).

---

### F-LP-IMPL-P1-009 — Doc comments stale "sealed via SensorAuth"

**Severity:** SUGGESTION (bumped close to important via sibling-sweep TD-VSDD-060)
**Category:** doc drift; Standing Rule 3 §3

**Evidence:** `crowdstrike.rs:4`, `cyberint.rs:4`, `claroty.rs:4`, `armis.rs:4` — all module-level docs claim "sealed via `SensorAuth`" — factually wrong post-AC-1.

**Suggested route:** orchestrator → implementer (sibling-sweep "sealed via `SensorAuth`" → "open trait `SensorAuth` (post S-PLUGIN-PREREQ-E)" in all 4 files; ~5 min in-scope).

---

### F-LP-IMPL-P1-010 — `WriteToolInvalidationMap` Debug derive leaks `plugin_name`

**Severity:** OBSERVATION

AD-017 confirmed clean (plugin_name non-secret per ADR-026 D7). Record for follow-up.

---

### F-LP-IMPL-P1-011 — `register_write_tool` lacks `tracing::info!` on success registration

**Severity:** OBSERVATION [process-gap]
**Category:** BC-2.16.002 catalog coverage — audit log gap; defer to product-owner adjudication.

---

## §Sweep Output

| Sweep | Result |
|---|---|
| `grep -rn 'CustomAdapter\|custom_adapter' crates/` (excl. tests) | 2 files: `tests/bc_2_16_011_test.rs` + `tests/error_taxonomy_annotation.rs` (both retirement gates). Zero `src/` hits. PASS for AC-4. |
| `grep -rn 'private::Sealed\|impl Sealed\|: Sealed' crates/prism-sensors/` | Zero. PASS for AC-1. |
| `grep -rn 'ESpec008' crates/` | 1 hit: `crates/prism-core/src/error.rs:898`. See F-008. |
| `grep -rn 'event_type = "write_tool_registration_after_boot"' crates/prism-query/src/invalidation.rs` | 1 hit (line 121). Field schema matches BC-2.16.002 row 33. PASS for AC-9d. |
| `grep -rn 'mark_query_phase_started' crates/` | 1 production call: `boot.rs:1016` (step 8 first statement, AFTER plugin runtime boot at step 7.5). Sequencing correct. PASS for ADR-022 §B 7.5/8 ordering. |
| `grep -rn 'register_write_tool' crates/ --excl tests` | ZERO production callers. See F-001 + F-002. FAIL for AC-9 production wiring. |
| `grep -rn 'validate_cross_composition' crates/*/src/` | ZERO production callers (8 test-only). FAIL for AC-3/3b/3c production wiring. See F-003. |

---

## §Verdict — BLOCKED

3 critical findings (F-001, F-002, F-003) constitute three independent end-to-end wiring gaps. The implementer delivered isolated callable functions but did NOT integrate them into the production call paths that BC-2.01.016, BC-2.16.011, and BC-2.16.012 demand. Each fix requires non-trivial structural plumbing.

---

## §Convergence Streak Update

- Before: 0/3
- After: 0/3 (BLOCKED — no advance)
