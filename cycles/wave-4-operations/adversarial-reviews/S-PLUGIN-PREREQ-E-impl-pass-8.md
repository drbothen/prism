---
document_type: adversarial-review
producer: adversary
pass: 8
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: e6b47f3e
diff_base_to_develop: a5ab742c
factory_artifacts_head: 443ac6bd
version: "1.0"
timestamp: 2026-05-18T10:30:00Z
verdict: BLOCKED
streak_before: 1/3
streak_after: 0/3
finding_counts:
  critical: 0
  important: 1
  suggestion: 1
  observation: 2
  process_gap: 0
novel_blind_spot: VP_artifact_existence_audit_pass_1_to_7
---

# Adversarial Review — S-PLUGIN-PREREQ-E Implementation Cascade — Pass 8

**Verdict: BLOCKED** | Streak: 1/3 → **0/3 RESET** | Pass 8 of impl-cascade

---

## §FB-IMPL-5 Closure Re-Re-Verification

All three FB-IMPL-5 closures remain load-bearing at unchanged HEAD e6b47f3e:

| Closure | Status | Evidence |
|---------|--------|----------|
| F-P6-001 Option B per-plugin atomic loop (`continue 'plugin_loop`) | HOLDS | `test_BC_2_16_012_write_tool_reg_failure_rolls_back_all_remaining_tools_for_plugin` — probe_good_t3.is_ok() structural assertion still present; labeled-loop construct unchanged |
| F-P6-OBS-001 ADR-026 amended_by back-ref | HOLDS | ADR-026 v1.26 frontmatter `amended_by:` field present; ARCH-INDEX v2.82 row intact |
| F-P6-OBS-002 Vector C Phase-5 deferral | HOLDS | Structural deferral — no Wave-0 scope coverage required; Phase-5 holdout evaluation remains the correct verification surface |

---

## §Cumulative Closure Re-Verification (Passes 1–7)

All prior pass closures spot-checked at HEAD e6b47f3e — all hold:

- **F-P1-001/002 DYNAMIC_WRITE_TOOLS read-side + PluginRuntime register_write_tool wiring:** wiring intact in boot.rs step 7.5/7.6.
- **F-P1-003/F-P2-001 validate_cross_composition production path:** wired to `parse_and_validate_spec_toml` (real production path via config_manager/MCP/hot_reload paths) — confirmed by integration tests.
- **F-P2-002 E-PLUGIN-021 error-taxonomy row:** present in error-taxonomy.md; PO-authored in FB-IMPL-2 scope.
- **F-P2-003 integration test race:** resolved via Cargo separate-binary process isolation; `#[ignore]` removed; test runs in-process correctly.
- **F-P4-001 Rule C CredentialRefProbe::probe() Route A:** `Option<String>` shape introspection present; step5 calls validate_cross_composition on real production path; ShapedProbe injectable for test coverage.
- **F-P4-002 fail-closed Route A deregister_write_tools_for_plugin:** `PluginRuntime::unregister_plugin` + ERROR-level `plugin_registration_rolled_back` event; BC-2.16.002 row 34 catalogued; BC-2.16.012 EC-016-012-004 present.
- **F-P5-001 Rule C backend-scope conditional (Option B):** ADR-026 §D3 + BC-2.01.016 §E-SPEC-014 scope constraint present; KeyringCredentialProbe doc cites D-706.
- **F-P5-002 unregister_plugin doc-vs-code reconciled:** rustdoc accurately describes single-threaded load→clone→store.
- **F-P5-003 BC-2.16.002 intro count 33→34:** intro count matches body row count.
- **F-P6-001 Option B per-plugin atomic loop:** VERIFIED AGAIN above.
- **Pass-7 Outcome (a) flake-claim adjudication:** verified independent in-tree evidence — signal_handlers.rs:102 comment + 30s sentinel-polling + PRISM_TEST_STOP_AFTER_STEP=6 halts before step 7.6; +15 tokio::tests mechanistically explains threshold-crossing. F-P7-OBS-001 attribution-discipline gap (no TD entry) — non-blocking carry-forward unchanged.

---

## §New Attack Vectors Run

Pass-8 rotated to 18 vectors including the novel VP-artifact-existence-audit vector (A-O):

| Vector | Result | Notes |
|--------|--------|-------|
| A. BC semantic contract coverage (Rules A/B/C production paths) | PASS | All three Rules reachable via validate_cross_composition in production; Rule C backend-scoped per D-706 |
| B. Structured event catalog completeness (BC-2.16.002) | PASS | 34 rows present; all event_type fields from implementation catalogued; intro count matches |
| C. Error taxonomy alignment (E-SPEC-012/013/014/E-PLUGIN-021) | PARTIAL — see F-P8-OBS-001 | Rule A rejects at TOML parse (generic error) vs spec-mandated E-SPEC-012 structured variant; observation-level |
| D. Rollback atomicity (F-P6-001 closure re-verification) | PASS | per-plugin atomic loop holds |
| E. Boot sequence wiring completeness (step 7.5/7.6) | PASS | DYNAMIC_WRITE_TOOLS populated; register_write_tool wired |
| F. Production path reachability (validate_cross_composition) | PASS | Wired to parse_and_validate_spec_toml; config_manager + MCP + hot_reload paths covered |
| G. Test isolation (no shared mutable state, no #[ignore] suppression) | PASS | Binary-split isolation confirmed; integration tests run correctly |
| H. Credential safety (AD-017, no credential transit in AI context) | PASS | OrgSlug::new_unchecked test-helpers-feature-gated; credentials reference-based |
| I. Fail-closed semantics on registration failure (BC-2.07.004) | PASS | deregister_write_tools_for_plugin invoked on any register_write_tool failure; F-P4-002 Route A holds |
| J. Arc-DI wiring (ADR-022 §C — no placeholder-construct) | PASS | No Arc::new(SomeThing::placeholder()) anti-pattern in production boot path |
| K. DYNAMIC_WRITE_TOOLS concurrency model | PARTIAL — see F-P8-SUG-001 | RwLock vs ArcSwap (AD-007) divergence; acceptable for write-once-during-boot but not documented |
| L. Spec-doc version consistency (POL-29 family) | PASS | POL-29 v1.28 step 8a/b/c/d/e/f/g/h/i all applied; no stale pins detected |
| M. Red Gate test count alignment (story frontmatter vs actual) | PASS | Red Gate tests present; count consistent with story frontmatter declaration |
| N. VP-INDEX registration and status consistency | PASS (existence-check pending — see below) | VP-INDEX line 183 VP-153 row present; status:draft + P0 + S-PLUGIN-PREREQ-E origin confirmed |
| O. **VP-artifact existence audit** (NEW VECTOR — pass-8 addition) | **FAIL — F-P8-IMP-001** | VP-153 declared status:draft + P0 + S-PLUGIN-PREREQ-E origin in VP-INDEX; story frontmatter `verification_properties:` cites VP-153; VP-153 §Proof Harness Skeleton names target file `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` — workspace grep CONFIRMS file DOES NOT EXIST |

---

## §Findings

### F-LP-IMPL-P8-IMP-001 — IMPORTANT: VP-153 P0 proptest declared but test file absent (novel pass-1-7 blind spot)

**Severity:** IMPORTANT (P0 verification artifact missing)
**Finding ID:** F-LP-IMPL-P8-IMP-001
**Pass:** 8
**Vector:** O — VP-artifact existence audit (NEW)

**Evidence:**

VP-INDEX.md line 183 row:
```
| VP-153 | P0 | proptest | S-PLUGIN-PREREQ-E | draft | BC-2.01.016 Rule A/B/C runtime cross-composition prevention |
```

Story frontmatter `verification_properties:` section lists VP-153.

VP-153 §Proof Harness Skeleton names the target file:
```
crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs
```

Workspace grep result:
```
grep -r "vp153_sensorauth_cross_composition" crates/ → (no output)
find crates/prism-spec-engine/tests/ -name "vp153*" → (no output)
```

**The proptest file does not exist.** The story risk_mitigations section does NOT mark VP-153 as deferred — compare VP-154, which is explicitly deferred to PLUGIN-MIGRATION-001-A per ADR-027 §Verification Property Anchors. VP-153 has no such deferral annotation.

**Why passes 1–7 missed this:** Passes 1–7 audited validator LOGIC (whether validate_cross_composition was wired to production paths, whether Rule C was reachable, whether callsite argument semantics were correct). No prior pass ran a "for each declared VP with origin=S-PLUGIN-PREREQ-E and status=draft, confirm the implementation artifact named in §Proof Harness Skeleton EXISTS in the workspace" check. This is a new attack vector class: VP-artifact-existence-audit.

**Impact:** VP-153's purpose is exhaustive enumeration of Rule A/B/C cross-composition prevention via proptest. The unit Red Gate tests cover ONE example per rule (boundary conditions at specific auth_type values). The proptest's purpose is property-based coverage across the full combinatorial input space. Declaring VP-153 as P0 in VP-INDEX and in story frontmatter creates an implied guarantee that this coverage exists — it does not.

**Per CLAUDE.md Canonical Principle Rule 1** ("No MVP-driven deferrals"): VP-153 has no human-directed deferral annotation (unlike VP-154). Default action per Rule 4 ("AI-built defects are the AI's responsibility to fix") is to land VP-153 in this story scope.

**Remediation required:** FB-IMPL-6 — dispatch test-writer to author `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` per VP-153 §Proof Harness Skeleton specification. VP-153 status:draft → active on merge per POL-14.

---

### F-LP-IMPL-P8-SUG-001 — SUGGESTION: DYNAMIC_WRITE_TOOLS uses RwLock not ArcSwap (AD-007 divergence)

**Severity:** SUGGESTION
**Finding ID:** F-LP-IMPL-P8-SUG-001
**Pass:** 8
**Vector:** K — DYNAMIC_WRITE_TOOLS concurrency model

**Evidence:**

`DYNAMIC_WRITE_TOOLS` is declared as `RwLock<HashMap<...>>` (or equivalent static RwLock). AD-007 establishes ArcSwap as the canonical pattern for config hot-reload (read via `ArcSwap::load()` to avoid Mutex/RwLock blocking on the read path). DYNAMIC_WRITE_TOOLS is populated once during boot (step 7.5/7.6) and read on every tool-dispatch call.

**Assessment:** The RwLock pattern is acceptable for write-once-during-boot because the write phase (boot) completes before the read phase (tool dispatch) begins — no concurrent write/read contention in practice. However, the divergence from AD-007 is undocumented. If a future story modifies DYNAMIC_WRITE_TOOLS at runtime (hot-reload of plugins), the RwLock pattern would introduce read-path blocking under the existing implementation.

**Non-blocking:** Does not violate any current behavioral contract. The divergence should be either (a) documented with a comment citing the boot-time-write-only rationale, or (b) migrated to ArcSwap per AD-007 if the pattern is expected to generalize.

**Remediation (optional):** Add inline comment in source citing write-once-during-boot rationale and AD-007 exception scope. If migrating to ArcSwap, follow AD-007 pattern.

---

### F-LP-IMPL-P8-OBS-001 — OBSERVATION: Rule A E-SPEC-012 variant drift (TOML parse error vs spec-mandated structured error)

**Severity:** OBSERVATION
**Finding ID:** F-LP-IMPL-P8-OBS-001
**Pass:** 8
**Vector:** C — Error taxonomy alignment

**Evidence:**

BC-2.01.016 Rule A specifies that multi-valued `auth_type` fields (comma-separated list where a single auth_type value is expected) must produce `E-SPEC-012` as the structured rejection error. The current implementation rejects multi-valued auth_type at TOML parse time with a generic deserialization error (the TOML serde layer fails on type mismatch before the validator sees the value). The spec-mandated `E-SPEC-012` variant is never emitted for this class of invalid input.

**Impact:** OBS-level — the rejection happens at the correct phase (load time) and correctly prevents the plugin from loading. The user-visible message is less structured than E-SPEC-012 would provide (no error code, no variant-level diagnostic detail). This would surface as a test failure if VP-153 is landed: the proptest would likely include a multi-value auth_type case expecting E-SPEC-012, and the actual error would not match.

**Non-blocking now:** VP-153 proptest landing (FB-IMPL-6) would surface this structurally. Noting now as context for the test-writer.

---

### F-LP-IMPL-P8-OBS-002 — OBSERVATION: BC-2.16.002 catalog scope coverage (cycle-close deferred)

**Severity:** OBSERVATION
**Finding ID:** F-LP-IMPL-P8-OBS-002
**Pass:** 8
**Vector:** B — Structured event catalog completeness

**Evidence:**

BC-2.16.002 Structured Event Catalog has 34 rows as of v1.33. The catalog scope for S-PLUGIN-PREREQ-E events appears complete within the story boundary. However, the catalog's breadth across the full 24-crate workspace has not been audited in this cascade — the catalog was last comprehensively reviewed during the Wave 3 spec cascade. New emission sites added in PREREQ-D (boot wiring) and PREREQ-E (cross-composition validation + rollback events) are catalogued, but the broader workspace audit is a cycle-close item per S-7.02.

**Non-blocking:** Does not affect S-PLUGIN-PREREQ-E merge readiness. Flagged for spec-steward cycle-close review.

---

## §Sweep Output

| Artifact | Check | Result |
|----------|-------|--------|
| crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs | File existence | MISSING — F-P8-IMP-001 |
| VP-INDEX.md line 183 | VP-153 status + origin + P0 declaration | Confirmed present; status:draft; P0; S-PLUGIN-PREREQ-E; no deferral annotation |
| Story frontmatter verification_properties | VP-153 citation | Confirmed — VP-153 listed without deferral |
| VP-154 story frontmatter | Deferral annotation for comparison | PLUGIN-MIGRATION-001-A deferral present — asymmetry with VP-153 confirmed |
| DYNAMIC_WRITE_TOOLS declaration | RwLock vs ArcSwap | RwLock — write-once-during-boot acceptable but undocumented |
| validate_cross_composition E-SPEC-012 emission on multi-value auth_type | Error taxonomy variant coverage | Generic TOML parse error; E-SPEC-012 not emitted |
| BC-2.16.002 row count | 34 rows per intro count | PASS — count matches |
| POL-29 v1.28 propagation | No stale version pins | PASS — no stale pins detected in sweep |

---

## §Verdict

**BLOCKED.** 1 IMPORTANT finding (F-P8-IMP-001 VP-153 P0 proptest test file absent). The VP-INDEX and story frontmatter declare VP-153 as a P0 verification property anchored to this story scope, with an explicit §Proof Harness Skeleton naming the target file. The file does not exist. Per BC-5.39.001 3-CLEAN protocol, any finding resets the streak.

This is a **novel blind spot** from passes 1–7: all prior passes audited validator LOGIC (callsite wiring, argument semantics, production-path reachability) but never ran the "for each declared VP with origin=current-story, confirm the implementation artifact named in §Proof Harness Skeleton exists" check. The pass-8 rotated attack vector O (VP-artifact existence audit) surfaced this gap.

**Cascade trajectory:** pass-3 CLEAN(1/3) → pass-4 RESET → pass-5 RESET → pass-6 0C+1H → pass-7 CLEAN(1/3) → **pass-8 RESET(0/3) — novel blind spot**

**Next step:** FB-IMPL-6 — dispatch test-writer to author VP-153 proptest per §Proof Harness Skeleton. After merge, dispatch pass-9 fresh-context for streak 0/3 → 1/3 retry.

---

## §Convergence Streak Update

| Event | Before | After |
|-------|--------|-------|
| Pass-8 (1 IMPORTANT) | 1/3 | **0/3 RESET** |

**BC-5.39.001 status:** 0 of 3 consecutive clean passes. Streak reset. Three consecutive clean passes required for convergence; count restarts from 0.

**Codification candidate:** VP-artifact-existence-audit as a mandatory adversary attack vector for any story declaring `verification_properties:` with `status: draft` and a §Proof Harness Skeleton naming a target file. Propose adding to adversary rubric at cycle-close per S-7.02.
