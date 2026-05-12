---
fix_burst: 10
story: S-PLUGIN-PREREQ-B
addresses_pass: 10
worktree_head_before: f5746553
worktree_head_after: 01df68cd
factory_sha_at_close: <will be this commit>
findings_closed: 3 actionable (2 MED + 1 LOW) + 1 deferred-as-TD (F-LP10-LOW-001)
findings_closure_class: 1 sibling-sweep gap (with BROAD sweep), 1 silent-data-corruption (Object class), 1 test-helper visibility
tests_added: 2 (story red_gate_tests 47→49)
discipline: TDD + TD-VSDD-059 paper-fix detection + TD-VSDD-060 BROAD sibling sweep
bonus_fix: 1 (spec_parser.rs:383 defensive char_indices, discovered during BROAD sweep)
date: 2026-05-11
---

# Fix-Burst 10 — Closure Report (S-PLUGIN-PREREQ-B)

**Worktree HEAD after burst:** `01df68cd`
**factory-artifacts HEAD:** (this commit)
**Pass addressed:** Pass-10 (BLOCKED-soft, 2 MED + 2 LOW + 3 OBS)
**Streak position:** Pass-10 was 0/3; pass-11 dispatch will target 1/3.

---

## Findings Closed

### F-LP10-MED-001 — Byte-slice anti-pattern in validation.rs:126 — CLOSED (with BROAD sibling sweep)

- **Test added:** `test_BC_2_16_009_validation_handles_multibyte_utf8_base_url_without_panic` (bc_2_16_009_test.rs)
- **Production change (primary):** validation.rs — new helper `truncate_at_char_boundary(s: &str, max_chars: usize) -> &str` (line 22+); base_url truncation at line 126 now uses helper.
- **TD-VSDD-059 load-bearing:** Pre-fix PANICS `byte index 200 is not a char boundary; it is inside '🎯' (bytes 197..201)`. Post-fix returns `Err(ValidationError { .. })` cleanly. Pre-fix code would FAIL the no-panic assertion.
- **BROAD sibling sweep results** (mandated by F-LP10-MED-001 itself — fix-burst-9 sweep was scoped too narrowly to pipeline.rs only):

| File:Line | Match | Classification |
|-----------|-------|----------------|
| validation.rs:126 (old) | `&spec.base_url[..spec.base_url.len().min(200)]` | **FIXED** — uses truncate_at_char_boundary |
| pipeline.rs:897 | `&s[..idx]` where idx from char_indices().nth(100) | **EXEMPT** — char-boundary-safe by construction |
| spec_parser.rs:383 | `&toml_input[..span.start.min(toml_input.len())]` | **FIXED (defensive bonus)** — char_indices()-based newline counter; semantics identical, immune to future toml span encoding changes |
| validation.rs:22+ (new helper) | `&s[..idx]` from char_indices().nth(max_chars) | **EXEMPT** — this IS the fix pattern |
| validation.rs:273 | `table.steps[..si]` | **EXEMPT** — si is usize index into Vec<FetchStep>, not a byte index into a &str |
| proofs/plugin_wit_validation.rs:187 | `&["name","version"][..]` | **EXEMPT** — static literal slice, no UTF-8 byte indexing |

infusion subdir (cache.rs, enrich_descriptor.rs, loader.rs, mod.rs, plugin_bridge.rs, sources/*, udf.rs): zero `[..]` patterns — clean.

### F-LP10-MED-002 — find_fan_out_array Object-stringification silent corruption — CLOSED

- **Test added:** `test_BC_2_16_002_fanout_invalid_source_type_emits_structured_event_for_object` (pipeline_http_integration.rs)
- **Production change:** pipeline.rs `find_fan_out_array` (Strategy A — narrowest blast radius). After array-vars collection, second sweep detects Object-typed step_vars referenced by templates and emits structured `tracing::warn!` with `event_type = "fanout_invalid_source_type"`, `step_name`, `var_name`, `actual_type = "Object"`.
- **TD-VSDD-059 load-bearing:** Pre-fix code emits no warn → log buffer empty. Test assertion `log_output.contains("fanout_invalid_source_type")` FAILS. Post-fix: warn fires with structured fields. Pipeline executes (warn surfaces ambiguity; doesn't fail-fast — backward compat preserved).
- **Sibling sweep:** value_to_string callers in interpolation.rs at lines 118 and 249 — both run AFTER find_fan_out_array's Object-warn, so no additional silent-corruption sites. Strategy A places the warn at the earliest detection point.

### F-LP10-LOW-002 — MockAuthProvider.token pub-mut — CLOSED

- **Production change:** auth_provider.rs:148 — `pub token: String` → private + `pub fn token(&self) -> &str` getter. Also: MockAuthProvider.call_count and FailingAuthProvider.call_count converted to private + `.calls()` getter (broader visibility hygiene).
- **No dedicated Red Gate test needed** — visibility change. Sibling-sweep confirmation: search for `.token =` write-access sites in tests/ returned ZERO matches. Compilation succeeds with private field, all 282 tests pass.
- **Pending-intent-verification disposition:** No test mutates .token after construction. The field WAS gratuitously public; visibility tightening is the correct fix, no Mutex<String> needed.

### F-LP10-LOW-001 — Inconsistent #[non_exhaustive] discipline — DEFERRED (TD-S-PLUGIN-PREREQ-B-016 filed)

- **Justification (substantial, per user mandate):**
  1. PREREQ-C plans to extend SensorSpec/FetchStep with new fields. Adding `#[non_exhaustive]` mid-PREREQ-B without the field additions risks "decorate now, extend later" pattern losing track of the discipline.
  2. Crate version is 0.5.0 — pre-1.0, no SemVer commitment, so the SemVer-surface concern is theoretical.
  3. Discipline best applied as single audit pass when field additions land, not piecemeal.
- **TD filed:** TD-S-PLUGIN-PREREQ-B-016 in tech-debt-register.md (this commit).

---

## Discipline Verification

- [x] TD-VSDD-059 paper-fix detection per finding (Red Gate FAIL captured for MED-001 and MED-002; LOW-002 is visibility change with sibling-sweep confirmation)
- [x] TD-VSDD-060 BROAD sibling sweep per F-LP10-MED-001 (full crate, not pipeline.rs only)
- [x] Bonus fix surfaced by BROAD sweep: spec_parser.rs:383 defensive char_indices conversion
- [x] `just check-fast` clean (clippy --all-features 0 warnings + layout + fmt)
- [x] Full suite: 282 tests pass (280 → 282, +2 Red Gate)
- [x] Single worktree commit per TD-VSDD-053: `fix(prism-spec-engine): close pass-10 findings (F-LP10-M001+M002+L002)`

---

## Files Modified

### Worktree (committed by implementer as 01df68cd)
- `crates/prism-spec-engine/src/validation.rs` — truncate_at_char_boundary helper + base_url truncation fix
- `crates/prism-spec-engine/src/spec_parser.rs` — char_indices newline counter (bonus defensive fix)
- `crates/prism-spec-engine/src/pipeline.rs` — Object-type warn in find_fan_out_array
- `crates/prism-spec-engine/src/auth_provider.rs` — token/call_count visibility + getters
- `crates/prism-spec-engine/tests/bc_2_16_009_test.rs` — Red Gate test for F-LP10-MED-001
- `crates/prism-spec-engine/tests/pipeline_http_integration.rs` — Red Gate test for F-LP10-MED-002

### Factory-artifacts (this commit)
- `.factory/code-delivery/S-PLUGIN-PREREQ-B/adversarial-review/fix-burst-10.md` (this file)
- `.factory/tech-debt-register.md` (v2.12 → v2.13 — TD-016 added)
- STATE.md, SESSION-HANDOFF.md, story, STORY-INDEX (per work items below)

---

## Recommendation

Dispatch LOCAL adversary pass-11 against worktree HEAD `01df68cd`. Target streak: 0/3 → 1/3. Scope: closure verification of fix-burst-10's 3 actionable + 1 deferred-as-TD + NEW dimensions disjoint from P5..P10.

**Verdict:** fix-burst-10 COMPLETE. Pass-11 dispatch authorized.
