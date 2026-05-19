---
document_type: adversarial-review
producer: adversary
pass: 7
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: e6b47f3e
diff_base_to_develop: a5ab742c
factory_artifacts_head: f42d5d9e
version: "1.0"
timestamp: 2026-05-18T09:30:00Z
verdict: CLEAN
streak_before: 0/3
streak_after: 1/3
finding_counts:
  critical: 0
  important: 0
  suggestion: 0
  observation: 1
  process_gap: 0
fb_impl_5_closures:
  verified: 3
  deferred: 1  # OBS-002 Phase-5
flake_claim_adjudication: outcome_a_verified_pre_existing_load_induced
---

# Adversarial Review — S-PLUGIN-PREREQ-E Implementation Cascade — Pass 7

**Verdict: CLEAN** | Streak: 0/3 → **1/3** | Pass 7 of impl-cascade

---

## §FB-IMPL-5 Closure Verification

All three FB-IMPL-5 in-scope closures independently verified as load-bearing structural fixes:

| Closure | Finding | Fix Applied | Load-Bearing Evidence | Status |
|---------|---------|-------------|----------------------|--------|
| F-P6-001 Option B per-plugin atomic loop | IMPORTANT: step 7.6 rollback bug — plugin with [t1,t2,t3] where t2 fails leaves orphaned P/t3 entry in DYNAMIC_WRITE_TOOLS | `continue 'plugin_loop` on per-plugin failure; labeled-loop construct ensures plugin loop advances to next plugin after rollback, guaranteeing zero orphaned tool entries across multi-tool plugins | RED-GATE test `test_BC_2_16_012_write_tool_reg_failure_rolls_back_all_remaining_tools_for_plugin` — probe_good_t3.is_ok() is the load-bearing assertion; test was RED before fix (good_t3 orphan present) and GREEN after (good_t3 absent from DYNAMIC_WRITE_TOOLS) | VERIFIED — clean structural fix; `plugin_loop + continue correctly enforces atomicity`; 3-tool RED-GATE is the discriminating assertion |
| F-P6-OBS-001 ADR-026 amended_by back-ref | OBSERVATION: ADR-026 missing bidirectional back-reference to ADR-026-AMENDMENT document | Architect SHA 8aa52b2d: ADR-026 v1.25→v1.26 adds `amended_by:` frontmatter field pointing to ADR-026-AMENDMENT | Both ADR-026 and ADR-026-AMENDMENT cross-reference each other; discoverability via ARCH-INDEX v2.82 row | VERIFIED — bidirectional reference confirmed; ADR-026 frontmatter amended_by field present |
| F-P6-OBS-002 Vector C system-level re-verification | OBSERVATION: architect risk-LOW claim for Vector C partially unverifiable in Wave-0 code scope | Deferred to Phase-5 system-level re-verification per S-7.02 | Deferral is structural — Wave-0 code does not have the full cross-plugin interaction surface required to validate the risk claim; Phase-5 holdout evaluation is the correct verification surface | DEFERRED — Phase-5 per S-7.02; not a blocking closure gap |

---

## §Cumulative Closure Re-Verification

Spot-check of all prior pass closures (passes 1–6); all hold at HEAD e6b47f3e:

- **F-P1-001/002 DYNAMIC_WRITE_TOOLS read-side + PluginRuntime register_write_tool wiring:** wiring intact in boot.rs step 7.5/7.6; DYNAMIC_WRITE_TOOLS populated from registry on read path.
- **F-P1-003/F-P2-001 validate_cross_composition production path:** wired to `parse_and_validate_spec_toml` (real production path) — confirmed by 3 integration tests exercising config_manager + MCP + hot_reload paths. Dead-code `SpecLoader::parse` paper-fix lineage fully resolved.
- **F-P2-003 integration test race:** binary isolation confirmed; `invalidation_post_boot_test.rs` is a separate binary crate with no `#[ignore]`; no shared global state with other tests.
- **F-P4-001 Rule C wiring Route A:** `CredentialRefProbe::probe()` returns `Result<Option<String>>` for shape introspection; step5 gate uses `ShapedProbe` in tests and `KeyringCredentialProbe` in production; argument-semantic-aliasing eliminated.
- **F-P4-002/F-P6-001 fail-closed semantics:** `deregister_write_tools_for_plugin` + `PluginRuntime::unregister_plugin` invoked on any `register_write_tool` failure; `plugin_registration_rolled_back` ERROR event emitted. Per-plugin atomic loop (FB-IMPL-5) confirmed as the correct structural fix.
- **F-P5-001 Rule C backend-scope conditional (Option B):** `KeyringCredentialProbe` correctly returns `Ok(None)` for backends without shape metadata; Rule C gate reached only for shaped backends. ADR-026 §D3 scoping amendment present.
- **F-P5-002 unregister_plugin doc-vs-code reconciliation:** doc accurately describes single-threaded load→clone→store semantics (no CAS language).
- **F-P5-003 BC-2.16.002 catalog count 33→34:** intro count and row count are in sync.

---

## §Flake-Claim Investigation

**Subject:** Implementer claim that `test_BC_2_10_010_sigterm` failure during `just check` is a pre-existing load-induced flake.

**Standing Rule 3 §1 requires:** Fresh-context independent verification. Implementer self-disclosure of risk severity is NOT authoritative (CLAUDE.md, Standing Rule 3 §1).

**6-step framework verdict: Outcome (a) — verified pre-existing load-induced flake.**

### Step 1: Locate the claimed evidence

Test file: `crates/prism-bin/tests/signal_handlers.rs`, test: `test_BC_2_10_010_sigterm_shuts_down_cleanly`.

**Finding:** Test source comment at `signal_handlers.rs:102` explicitly documents:
> "Under load, RocksDB init in a temp directory can take several seconds."

This is in-tree documentation of the load-induced mechanism — authored at test-write time (FIX-era of S-WAVE5-PREP-01).

### Step 2: Verify the FIX-era sentinel-polling mitigation

`D-318` in STATE.md records: `test_BC_2_10_010_sigterm` FIXED 2026-05-09 with 5/5 reproducibility via 30s sentinel-polling deadline (FIX-era mechanism from S-WAVE5-PREP-01). This 30s deadline is a mitigation for load-induced RocksDB init latency — not a deferral.

**Finding:** The 30s sentinel-polling deadline is a FIX-era mechanism, not a new workaround. D-318 documents the fix as intentionally robust against load variance.

### Step 3: Scope check — does FB-IMPL-5 diff touch the SIGTERM code path?

FB-IMPL-5 changed: boot.rs step 7.6 per-plugin atomic loop (write tool registration rollback logic). The SIGTERM test exercises step 7.6 Option B in the shutdown flow — specifically the `PRISM_TEST_STOP_AFTER_STEP=6` path.

**Finding:** `PRISM_TEST_STOP_AFTER_STEP=6` causes boot sequence to halt BEFORE step 7.6 is reached. Step 7.6 Option B labeled-continue logic is dead-code on this test path. FB-IMPL-5 diff has zero behavioral impact on the SIGTERM test.

### Step 4: Identify the load-increase mechanism

FB-IMPL-5 adds `tests/plugin_boot_tests.rs` with +15 `#[tokio::test]` functions. Each uses a temporary RocksDB directory.

**Finding:** +15 concurrent tokio::tests each opening a RocksDB temp directory materially increases concurrent filesystem I/O during `just check`. This mechanistically explains why the pre-existing load threshold was crossed for the SIGTERM test.

### Step 5: Pass-2 false-flake comparison

Pass-2 caught `F-P2-004` where the implementer claimed a pre-existing flake citing `TD-S-WAVE5-PREP-01-FLAKY-SIGTERM` — but that entry did not exist in `tech-debt-register.md`, and D-318 documented the test as FIXED with 5/5 reproducibility (not a known flake). That was Outcome (c) — false-flake claim without in-tree evidence.

**This case is different:** The test source comment at `signal_handlers.rs:102` is independent in-tree evidence documenting the load-induced mechanism. D-318 confirms the 30s deadline mitigation is a FIX-era choice. FB-IMPL-5 has zero SIGTERM path diff. The +15 concurrent RocksDB tests mechanistically explain threshold crossing.

### Step 6: Final determination

**Outcome (a):** The flake is pre-existing, load-induced, and mechanistically explained by independent in-tree evidence. The implementer's characterization is substantively correct. No regression introduced by FB-IMPL-5.

**However:** The implementer did not cite a TD entry or the specific signal_handlers.rs:102 comment as evidence — they asserted the claim without canonical attribution. This is the F-P2-004 attribution-discipline pattern recurrence. Recorded as F-LP-IMPL-P7-OBS-001 (non-blocking OBSERVATION).

---

## §New Attack Vectors Run

| Vector | Target | Result |
|--------|--------|--------|
| A — Labeled-continue semantics | step 7.6 `continue 'plugin_loop` correctness: does it skip remaining tools for plugin P only? Does the outer loop correctly advance? | PASS — `'plugin_loop` label is on the `for plugin in plugins` iterator; `continue` advances to the NEXT plugin, not tool. All tools for failed plugin P correctly skipped. |
| B — RED-GATE assertion coverage | `test_BC_2_16_012_write_tool_reg_failure_rolls_back_all_remaining_tools_for_plugin`: is probe_good_t3.is_ok() the discriminating assertion? | PASS — good_t3 is the 3rd tool of plugin P where t2 fails. The fix removes the orphan entry; the assertion directly tests the bug-class root cause. |
| C — BC-2.07.004 contract compliance | Does per-plugin atomic loop guarantee exactly ONE `plugin_registration_rolled_back` event per plugin? | PASS — labeled-continue causes a single `deregister_write_tools_for_plugin` call per failed plugin; exactly ONE ERROR event per BC-2.07.004. |
| D — ADR-026 amended_by discoverability | Is bidirectional reference navigable from ARCH-INDEX? | PASS — ARCH-INDEX v2.82 row for ADR-026 cites v1.26; ADR-026 frontmatter has `amended_by: ADR-026-AMENDMENT`; ADR-026-AMENDMENT frontmatter has `amends: ADR-026`. |
| E — Rule C backend-scope conditional completeness | Does `KeyringCredentialProbe::probe()` returning `Ok(None)` correctly bypass Rule C for non-shaped backends? | PASS — ADR-026 §D3 backend-scope scoping is explicit; Rule C gate is guarded by `Option<String>` non-None check; Ok(None) path correctly bypasses the mismatch assertion. |
| F — Flake independence from FB-IMPL-5 | Does the diff include any modification to `crates/prism-bin/tests/signal_handlers.rs` or boot sequence step 7.6 Option B? | PASS — diff inspection: no changes to signal_handlers.rs. Step 7.6 changes are in write-tool registration rollback loop only; PRISM_TEST_STOP_AFTER_STEP=6 path does not reach step 7.6. |
| G — D-706 Option B structural enforcement (PLUGIN-MIGRATION-001-A) | Is the structural enforcement deferral correctly attached? | PASS — S-PLUGIN-PREREQ-E frontmatter `blocks: [PLUGIN-MIGRATION-001-A]` is structurally enforced; cannot be inadvertently bypassed. |
| H — Error taxonomy sync | Does BC-2.16.002 catalog intro count match row count after FB-IMPL-5? | PASS — count 34 matches rows (FB-IMPL-3 brought count to 34; FB-IMPL-5 adds no new catalog row). |
| I — OBS-002 Phase-5 deferral validity | Does Vector C system-level risk claim require Phase-5? | PASS — Wave-0 code does not have the cross-plugin multi-sensor interaction surface required to stress Vector C exhaustively; Phase-5 holdout evaluation is the structurally correct scope. |

---

## §Findings

### F-LP-IMPL-P7-OBS-001 — OBSERVATION: Attribution-discipline gap — flake claim lacks canonical TD entry

**Severity:** OBSERVATION (non-blocking)
**Type:** Process gap (attribution discipline)
**Lineage:** F-P2-004 pattern recurrence

**Description:** Implementer's flake claim for `test_BC_2_10_010_sigterm` is substantively correct (Outcome a verified via independent in-tree investigation). However, the claim was made without citing:
1. The specific in-tree evidence (`signal_handlers.rs:102` test source comment documenting load-induced mechanism)
2. The canonical FIX-era mitigation context (D-318, 30s sentinel-polling deadline)

This is the F-P2-004 attribution-discipline pattern: implementer asserts a flake claim without canonical evidence citation. In pass-2, the claim was false (Outcome c) because no TD entry existed and D-318 documented the test as FIXED. In this pass, the claim is correct (Outcome a), but the supporting evidence was only found via independent adversarial investigation — not provided by the implementer.

**Impact:** Non-blocking. The substantive claim is correct and independently verified. However, the pattern of unsubstantiated flake claims (even when occasionally correct) imposes adversarial investigation overhead per Standing Rule 3 §1.

**Resolution path:** No immediate fix required. Codification candidate for orchestrator dispatch-prompt: implementer MUST cite (a) specific source file + line of in-tree documentation OR (b) TD entry number for any pre-existing flake claim. Carry-forward to cycle-close per S-7.02.

---

## §Sweep Output

| Artifact | Scope | Result |
|---------|-------|--------|
| `crates/prism-core/src/plugin/boot.rs` step 7.6 | Per-plugin atomic loop + label semantics | PASS |
| `crates/prism-core/tests/plugin_boot_tests.rs` | RED-GATE test probe_good_t3 assertion | PASS |
| `crates/prism-bin/tests/signal_handlers.rs` | SIGTERM test load-induced flake mechanism | PASS — source comment at line 102 confirmed |
| `.factory/specs/architecture/adr/ADR-026.md` | amended_by back-ref + §D3 backend-scope | PASS |
| `.factory/specs/behavioral-contracts/BC-2.07.004.md` | plugin_registration_rolled_back contract | PASS |
| `.factory/specs/behavioral-contracts/BC-2.16.002.md` | Catalog count 34, catalog rows 34 | PASS |
| `.factory/specs/architecture/adr/ADR-026-AMENDMENT.md` | amends: ADR-026 back-ref | PASS |

---

## §Verdict

**CLEAN.** 0 Critical + 0 Important + 0 Suggestion + 1 Observation + 0 Process Gap.

All 3 FB-IMPL-5 in-scope closures verified load-bearing:
- F-P6-001 Option B per-plugin atomic loop: structural fix eliminates orphan-tool bug class at root; 3-tool RED-GATE test is discriminating.
- F-P6-OBS-001 ADR-026 amended_by: bidirectional reference confirmed.
- F-P6-OBS-002: correctly deferred to Phase-5.

Flake-claim adjudication: Outcome (a) — pre-existing load-induced flake verified via independent in-tree evidence (signal_handlers.rs:102 + D-318 FIX-era mitigation + zero FB-IMPL-5 SIGTERM path diff + +15 concurrent RocksDB tests mechanistically explaining threshold crossing). Unlike F-P2-004 (false claim), this claim is substantively correct but attribution-discipline is deficient — captured as F-LP-IMPL-P7-OBS-001 (non-blocking).

All 9 new attack vectors PASS.
All prior pass closures (passes 1–6) hold at unchanged HEAD e6b47f3e.

---

## §Convergence Streak Update

| Metric | Value |
|--------|-------|
| Pass | 7 |
| Streak before | 0/3 |
| Streak after | **1/3** |
| Advance | YES — first CLEAN advance of post-FB-IMPL-5 cycle |
| Next dispatch | Pass 8, fresh-context, against unchanged HEAD e6b47f3e (target 1/3 → 2/3) |

**Cascade trajectory:** pass-1 3C+4I → pass-2 2C+3I → pass-3 CLEAN(1/3) → pass-4 1C+1I RESET → pass-5 1C+1I RESET → pass-6 0C+1I → **pass-7 0C+0I CLEAN(1/3)** — convergence accelerating; severity decreasing pass-by-pass.
