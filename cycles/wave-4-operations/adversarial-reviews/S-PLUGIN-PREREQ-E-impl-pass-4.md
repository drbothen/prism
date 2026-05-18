---
document_type: adversarial-review
producer: adversary
pass: 4
cascade_scope: LOCAL implementation
story_id: S-PLUGIN-PREREQ-E
diff_head: 8e4df5bf
diff_base_to_develop: a5ab742c
version: "1.0"
timestamp: 2026-05-18T04:00:00Z
verdict: BLOCKED
streak_before: 1/3
streak_after: 0/3
finding_counts:
  critical: 1
  important: 1
  suggestion: 0
  observation: 2
  process_gap: 1
prior_pass_closures_re_verified:
  verified: 6
  newly_defective: 1  # F-P2-001 SHAPE-argument aliasing
---

# Adversarial Review — S-PLUGIN-PREREQ-E Implementation Pass 4

**Story:** S-PLUGIN-PREREQ-E (Un-seal SensorAuth + Deprecate CustomAdapter)
**Diff head:** `8e4df5bf` (unchanged HEAD — same as pass-3)
**Develop base:** `a5ab742c`
**Verdict:** BLOCKED — streak RESET 1/3 → **0/3**

---

## §Cumulative Closure Re-Verification

Re-verification of all 7 prior closure claims under fresh-context pass-4 (argument-semantic-realism dimension added vs pass-3 callsite-presence dimension only):

| Finding | Prior Verdict | Pass-4 Re-Check | Notes |
|---------|--------------|-----------------|-------|
| F-P1-001 — DYNAMIC_WRITE_TOOLS read-side wiring | VERIFIED CLOSED (pass-3) | VERIFIED CLOSED | read-side wiring confirmed intact at invalidation callsites |
| F-P1-002 — PluginRuntime register_write_tool unwired | VERIFIED CLOSED (pass-3) | VERIFIED CLOSED | production caller present; wiring load-bearing |
| F-P1-003 / F-P2-001 — validate_cross_composition dead-code paper-fix | VERIFIED CLOSED (pass-3) — callsite-presence level | **NEWLY DEFECTIVE — F-LP-IMPL-P4-001** | wired to parse_and_validate_spec_toml CONFIRMED; but BOTH arguments to validate_cross_composition alias auth_type; Rule C equality check is tautologically false in production; paper-fix recurrence at argument-semantic-realism granularity |
| F-P2-002 — E-PLUGIN-021 missing from error-taxonomy.md | VERIFIED CLOSED (pass-3) | VERIFIED CLOSED | row present; message byte-exact |
| F-P2-003 — Integration test race | VERIFIED CLOSED (pass-3) | VERIFIED CLOSED | process isolation intact; no #[ignore] suppression |
| F-P2-004 — [process-gap] false-flake claim | VERIFIED ACKNOWLEDGED (pass-3) | VERIFIED ACKNOWLEDGED | no regression; just check 3664/3664 accepted |
| F-P2-006 — boot.rs residual misleading comment | VERIFIED CLOSED (pass-3) | VERIFIED CLOSED | rewritten comment accurate |

**Summary:** 6 of 7 prior closures hold. F-P2-001 (which was itself the pass-2 re-finding of F-P1-003) is newly defective at argument-semantic-realism granularity. This is a TD-VSDD-059 paper-fix lineage: F-P1-003 (callsite-presence fail) → F-P2-001 (callsite-presence fix at wrong path) → F-LP-IMPL-P4-001 (callsite-argument semantic-aliasing, structurally unreachable in production).

---

## §New Attack Vectors Run

15 attack vectors applied (Vectors A–O):

| Vector | Attack Class | Verdict |
|--------|-------------|---------|
| A — validate_cross_composition argument realism | Argument-semantic-aliasing detection | FAIL — F-LP-IMPL-P4-001 CRITICAL |
| B — boot.rs step5 validate_cross_composition call presence | Deferral-target emptiness check | FAIL — F-LP-IMPL-P4-001 CRITICAL (empty deferral target) |
| C — step 7.6 register_write_tool failure semantics | Silent partial-failure detection | FAIL — F-LP-IMPL-P4-002 IMPORTANT |
| D — DYNAMIC_WRITE_TOOLS read-side continuity | F-P1-001 carry-forward hold | PASS |
| E — PluginRuntime register_write_tool production caller | F-P1-002 carry-forward hold | PASS |
| F — E-PLUGIN-021 error-taxonomy row byte-exact match | POL-29 transitive closure | PASS |
| G — Integration test process isolation | Race fix hold | PASS |
| H — BC-2.03.013 Rule B coverage | N>0 compliance | PASS |
| I — error-taxonomy.md line 468 citation | TD-VSDD-091 volatile-pin scan | OBS — F-LP-IMPL-P4-OBS-001 |
| J — boot.rs step8 mark_query_phase_started wiring | Known-deferred todo!() scope check | OBS — F-LP-IMPL-P4-OBS-002 |
| K — validate_cross_composition Rule A / Rule B coverage | Rules other than C | PASS — Rules A/B paths not aliased |
| L — BC-2.01.016 E-SPEC-014 mandate satisfaction | Contract compliance | FAIL — F-LP-IMPL-P4-001 (same finding; CRIT) |
| M — BC-2.07.004 write-then-read consistency | Stale-read risk | FAIL — F-LP-IMPL-P4-002 (same finding; IMPORTANT) |
| N — #[ignore] suppression regression | Test suppression check | PASS |
| O — PluginRuntime loaded state after write-tool failure | Plugin state semantics | FAIL — F-LP-IMPL-P4-002 (same finding; IMPORTANT) |

---

## §Findings

### F-LP-IMPL-P4-001 — CRITICAL

**Finding:** Rule C (E-SPEC-014) structurally dead in production. Both production callsites at `spec_parser.rs:684-690` and `add_sensor_spec.rs:186-193` pass `spec.auth_type.as_str()` as BOTH `expected_shape` and `actual_shape` — the equality check at `spec_parser.rs:947` is tautologically false in production. Unit tests pass by calling the validator directly with mismatched shapes. The comment at `add_sensor_spec.rs:182-185` claims deferral to "step5_init_credential_store_with_probe" but `boot.rs:730-749` step5 contains NO call to `validate_cross_composition` and NO structural-shape comparison. Empty deferral target.

**Evidence:**
- `spec_parser.rs:684-690`: `validate_cross_composition(&spec.auth_type.as_str(), &spec.auth_type.as_str(), ...)` — expected_shape and actual_shape are pointer-equal to the same string
- `add_sensor_spec.rs:186-193`: same aliasing pattern
- `spec_parser.rs:947`: equality check `if expected_shape == actual_shape` is always false only if the two aliased strings differ — but they cannot differ because both are derived from the same `spec.auth_type`
- `boot.rs:730-749` step5: zero calls to `validate_cross_composition`; no structural-shape comparison exists
- Unit tests at direct-invocation level pass because they supply genuinely distinct expected/actual shapes

**Finer-grained lineage:** F-P1-003 (validate_cross_composition never called in production path) → F-P2-001 (wired to wrong path `SpecLoader::parse`) → F-LP-IMPL-P4-001 (wired to correct path `parse_and_validate_spec_toml` but argument aliasing defangs Rule C). This is a TD-VSDD-059 paper-fix recurrence at argument-semantic-realism granularity. Pass-3 verified closure at callsite-presence level; pass-4 applies callsite-argument semantic-realism check and finds aliasing.

**BC impact:** BC-2.01.016 §Error Cases E-SPEC-014 mandate unmet in production. Rule C (cross-composition guard: auth_type shape mismatch between credential reference and auth_type field) is contractually required and structurally unreachable.

**Route:** `vsdd-factory:implementer` — derive real `actual_shape` from the resolved credential at step5 credential-introspection time, OR escalate to product-owner if step5 structural-shape derivation requires a sanctioned deferral with explicit future-story anchor per Canonical Principle Rule 3. Implementer self-disclosure of risk severity is NOT authoritative (Standing Rule 3 §1).

**Severity:** CRITICAL — BC-2.01.016 §Error Cases E-SPEC-014 contract unmet in production; no paper-fix acceptable per TD-VSDD-059.

---

### F-LP-IMPL-P4-002 — IMPORTANT

**Finding:** Step 7.6 silent partial-failure. `boot.rs:991-1000` logs `WARN` on `register_write_tool` failure but leaves the plugin loaded and serviceable. Future writes via that plugin produce silent stale reads — BC-2.07.004 §write-then-read consistency is violated. The "n-1 survivor rule" comment in that block misapplies plugin-load semantic (where partial sets are acceptable: load N-1 of N plugins) to write-tool-registration semantic (where a plugin with failed write-tool registration is in an inconsistent state: it is loaded as a read-capable sensor but its write paths are silently broken).

**Evidence:**
- `boot.rs:991-1000`: `register_write_tool` failure path logs WARN and continues; plugin remains in `PluginRuntime` as fully loaded
- Read queries against the plugin succeed; write operations via the unregistered tool silently return stale or empty results
- BC-2.07.004 §write-then-read consistency: "a write followed by a read on the same sensor MUST reflect the write"; violated when write-tool is unregistered but sensor is loaded

**Standing Rule 3 §2 violation:** Silent `Vec::new()` return / partial-failure silent ignore where partial-failure data should propagate — exact pattern.

**Route:** `vsdd-factory:product-owner` — clarify BC-2.16.012 EC-016-012-004 post-rejection plugin state semantics (what is the correct behavior when a loaded plugin's write-tool registration fails?); OR `vsdd-factory:implementer` — unload plugin if any of its write tools fail registration (fail-closed per production-grade default Rule 1).

**Severity:** IMPORTANT — BC-2.07.004 stale-read risk; Standing Rule 3 §2 violation; "for now" silent ignore is a production-grade default violation.

---

### F-LP-IMPL-P4-OBS-001 — OBSERVATION

**Finding:** `error-taxonomy.md` line 468 cites `invalidation.rs:138-145` (TD-VSDD-091 volatile line-pin); actual construction site is at lines 143-145 per current diff context.

**Severity:** OBSERVATION — TD-VSDD-091 volatile-pin class; non-blocking; does not affect runtime behavior.

**Route:** Cycle-close deferred per S-7.02 (out-of-scope line-pin maintenance).

---

### F-LP-IMPL-P4-OBS-002 — OBSERVATION

**Finding:** `mark_query_phase_started` is wired into step8 which contains `todo!()` at `boot.rs:1059`. Known-deferred per S-3.02-FOLLOWUP-RUNTIME stub-stories.

**Severity:** OBSERVATION — known-deferred per S-3.02-FOLLOWUP-RUNTIME; non-blocking.

**Route:** No action required. Pre-existing intentional stub per wave planning.

---

### F-LP-IMPL-P4-PG-001 — PROCESS-GAP

**Finding:** Pass-3 verified F-P2-001 at callsite-presence level but missed argument-semantic-realism. The adversary skill rubric verifies callsite presence (is `validate_cross_composition` called?) but does not mandate argument-semantic-realism check (are the arguments to the call semantically distinct, as the validator requires?). This gap allowed a structurally dead code path to survive one clean pass.

**Codification target:** Enhance adversary skill rubric to include callsite-argument semantic-realism check alongside callsite-presence check. For any finding involving a "validator now wired," verification must include: (1) callsite presence, (2) call arguments are semantically meaningful (not aliased from same source), (3) execution path to actual semantically distinct inputs exists in production.

**Route:** `vsdd-factory:session-reviewer` at cycle close — codification candidate for adversary skill rubric enhancement.

---

## §Sweep Output

12-target sweep at unchanged HEAD `8e4df5bf`:

| Sweep Target | Check | Result |
|-------------|-------|--------|
| `validate_cross_composition` callsite in `parse_and_validate_spec_toml` | Callsite presence | PASS — callsite present |
| `validate_cross_composition` arguments in `spec_parser.rs:684-690` | Argument semantic-realism | FAIL — both args alias `spec.auth_type.as_str()` |
| `validate_cross_composition` arguments in `add_sensor_spec.rs:186-193` | Argument semantic-realism | FAIL — same aliasing pattern |
| `boot.rs:730-749` step5 | validate_cross_composition call presence | FAIL — absent; deferral target is empty |
| `boot.rs:991-1000` register_write_tool failure path | Silent partial-failure check | FAIL — WARN + continue; plugin stays loaded |
| `DYNAMIC_WRITE_TOOLS` read-side | invalidate_for_sensor/invalidate_for_write_tool | PASS |
| `register_write_tool` production caller | PluginRuntime wiring | PASS |
| `E-PLUGIN-021` error-taxonomy row | Row presence + message | PASS |
| `invalidation_post_boot_test.rs` | Process isolation + no #[ignore] | PASS |
| BC-2.03.013 fixture set | N>0 Rule B coverage | PASS |
| `error-taxonomy.md` line 468 cite | TD-VSDD-091 volatile-pin check | OBS — line-pin drift |
| `boot.rs:1059` step8 | Known-deferred todo!() scope | OBS — intentional stub |

---

## §Verdict

**BLOCKED.**

1 CRITICAL finding (F-LP-IMPL-P4-001) + 1 IMPORTANT finding (F-LP-IMPL-P4-002) found at unchanged HEAD `8e4df5bf`. Both are real bugs missed by pass-3.

**Novelty assessment: HIGH.** The argument-semantic-aliasing class (F-LP-IMPL-P4-001) is new — pass-3 applied callsite-presence verification only; pass-4 applied argument-semantic-realism verification and found aliasing. This is a genuinely new defect axis not previously applied in this cascade. F-LP-IMPL-P4-002 silent partial-failure is also new; pass-3 verified step 7.6 write-tool registration wiring presence but did not verify failure-path semantics.

**Cumulative re-verification:** 6 of 7 prior closures hold. F-P2-001 is newly defective at argument-semantic-realism granularity. The cascade is WORKING — passes 1-3 progressively closed real gaps; pass-4 applies deeper semantic scrutiny and surfaces the next layer. This is not cascade failure; this is cascade value.

---

## §Convergence Streak Update

| Pass | Verdict | Streak Before | Streak After | Notes |
|------|---------|---------------|--------------|-------|
| Pass 1 | BLOCKED | 0/3 | 0/3 | 3C+4I+1S+2Obs+1PG — end-to-end wiring gaps |
| Pass 2 | BLOCKED | 0/3 | 0/3 | 2C+3I+1S+1Obs+1PG — paper-fix + race not fixed |
| Pass 3 | CLEAN | 0/3 | 1/3 | 0C+0I+0M+0L+1S+2Obs — all FB-IMPL-2 closures verified |
| **Pass 4** | **BLOCKED** | **1/3** | **0/3** | **1C+1I+2Obs+1PG — Rule C dead-code arg-aliasing + step 7.6 silent partial-failure; RESET** |

**Streak after pass 4: 0/3 — RESET at penultimate per BC-5.39.001.**

Next: FB-IMPL-3 dispatch — implementer closes F-LP-IMPL-P4-001 (Rule C argument-semantic fix or sanctioned deferral with explicit story anchor) + product-owner consulted on F-LP-IMPL-P4-002 BC-2.16.012 EC-016-012-004 semantics. Then adversary pass-5 fresh-context against updated HEAD.
