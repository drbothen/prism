---
pass: impl-pass-4
story: S-PLUGIN-PREREQ-D
target_branch: feature/S-PLUGIN-PREREQ-D
target_sha: 51ee7ce5
verdict: BLOCKED
streak_before: 0/3
streak_after: 0/3
findings_total: 2
findings_crit: 0
findings_high: 1
findings_med: 1
findings_low: 0
findings_obs: 1
date: 2026-05-14
recorded_by: D-553
consecutive_single_commit: 58
---

# S-PLUGIN-PREREQ-D Adversary Impl-Pass-4

## Header

- **Pass:** impl-pass-4 (LOCAL adversary cascade per BC-5.39.001)
- **Target:** `feature/S-PLUGIN-PREREQ-D@51ee7ce5`
- **Dispatched:** 2026-05-14
- **Verdict:** BLOCKED
- **3-CLEAN Streak:** 0/3 → 0/3 (BLOCKED; no advance)
- **Prior passes:** impl-pass-1 BLOCKED (18) → fix-burst-impl-1 CLOSED → impl-pass-2 BLOCKED (12) → fix-burst-impl-2 CLOSED → impl-pass-3 BLOCKED (6) → fix-burst-impl-3 CLOSED → **impl-pass-4 BLOCKED (2)**

## Closure Verification — Fix-Burst-Impl-3 Prior Findings

| Finding | Prior Verdict | Pass-4 Verification | Evidence |
|---------|--------------|--------------------|-|
| F-PASS3-CRIT-001 (boot.rs step ordering) | CLOSED (820005e5) | PRODUCTION OK | `plugin_load_step_with_audit` appears before `step7_init_storage` in `run_boot_sequence`; PG-IMPL-LP3-001 dependency-frontier walk PASS |
| F-PASS3-CRIT-002 (Val-type mismatches: U16/Enum/Record) | CLOSED (51ee7ce5) | PRODUCTION OK / TEST PAPER | Val::U16 VERIFIED in host_functions.rs; Val::Enum(String) VERIFIED; single-slot Val::Record VERIFIED; BUT test assertions hand-construct copies of these types rather than exercising them through registered callbacks |
| F-PASS3-CRIT-003 (fabricated story-ID + dispatch test) | CLOSED (51ee7ce5) | PRODUCTION OK / TEST PAPER | Fabricated `S-4.08-manifest-embedding` REMOVED VERIFIED; dispatch test at line 1348-1455 exists BUT uses `linker.instantiate_pre()` (link verification only) + hand-constructs `Val::Record(403)` assertion — does not invoke registered callback via Component Model |
| F-PASS3-HIGH-001 (silent log-level downgrade) | CLOSED (subsumed by CRIT-002 + BC row 32) | PRODUCTION OK / TEST INDIRECT | Val::Enum(String) callback verified; BC-2.16.002 row 32 `plugin_log_level_unrecognized` verified; test indirect (inline-replica pattern) |
| F-PASS3-MED-001 (doc-comment contamination) | CLOSED (subsumed by CRIT-003) | PRODUCTION OK | Fabricated story-ID reference removed from source and tests |
| F-PASS3-MED-002 (schema-violation traps) | CLOSED (51ee7ce5) | PRODUCTION OK / TEST INDIRECT | All 5 callback `_ =>` arms return `Err(wasmtime::Error::msg("schema violation: ..."))` VERIFIED; test indirect (inline-replica pattern does not trigger these traps through Component Model dispatch) |

**Summary:** 6/6 prior closures VERIFIED at production code level. 0 PAPER-FIX-REOPENED (production code is genuinely correct). TEST PAPER pattern persists for 5 of 5 fix-burst-impl-3 tests — these tests verify hand-constructed values, not production dispatch.

## New Findings

### F-PASS4-HIGH-001 — Test Paper-Fix Recurrence (TD-VSDD-059)

**Severity:** HIGH
**Routing:** implementer
**Status:** OPEN

**Location:** `crates/prism-spec-engine/tests/plugin_integration_tests.rs` lines 1078, 1147, 1199, 1256, 1348

**Description:**

5 tests introduced by fix-burst-impl-3 use inline-replica match logic rather than dispatching through registered host callbacks:

1. **Line 1078** (`test_F_PASS3_CRIT_002_val_u16_for_wit_u16_status`): Hand-constructs `Val::U16(200)` and `Val::U16(403)` and asserts the production function returns them. Tests the function body directly, not through Component Model dispatch.

2. **Line 1147** (`test_F_PASS3_CRIT_002_val_enum_for_log_level`): Hand-constructs `Val::Enum("info".to_string())` etc. and calls host callback directly with `Val::Enum(...)` params. Does not dispatch through a registered linker function.

3. **Line 1199** (`test_F_PASS3_CRIT_002_val_record_single_slot_writeback`): Hand-constructs expected `Val::Record(...)` shape and asserts against function return. Direct function call, not linker dispatch.

4. **Line 1256** (`test_F_PASS3_MED_002_schema_violation_traps`): Calls callback functions directly with `Val::U32(...)` where `Val::Enum(...)` expected; verifies `Err` returned. Correct behavior verified, but via direct function call not Component Model host-import dispatch.

5. **Lines 1348-1455** (`test_F_PASS3_CRIT_003_component_model_dispatch_allowlist_gate`): This is the closest to genuine — it calls `linker.instantiate_pre()`. However, inspection reveals:
   - `linker.instantiate_pre()` verifies the type signature matches (i.e., the WAT function is linkable) but does NOT invoke the registered callback
   - The assertion at line 1434-1441 hand-constructs `Val::Record(vec![("status", Val::U16(403)), ...])` and asserts the return equals this hand-built record
   - The test does not call `linker.get_func(...)` or `.call()` on an exported function that internally invokes the host import
   - A regression of `Val::U16` → `Val::U32` in the production host callback body would NOT cause this test to fail

**Root cause:** The Component Model dispatch test infrastructure (`wat::parse_str` + `Component::from_binary` + `Linker::new`) is present and used. The gap is that no test calls an EXPORTED function that internally triggers the host import. The test proves the linker can link the types; it does not prove the host function body executes correctly when triggered via Component Model guest-to-host call.

**Implementer self-disclosure** (lines 1342-1346): "Calling the imported http-request through a Component Model function export is covered by the separate unit-level Val tests above." This claim is false: the "unit-level Val tests" are inline-replica tests (items 1-4 above), not Component Model dispatch tests. Per Standing Rule 3 §1, implementer self-disclosure of coverage sufficiency is NOT authoritative.

**Impact:** A Val::U16 → Val::U32 regression in `host_functions.rs` would pass all 5 tests above. The production code would be broken; the test suite would not catch it.

**Fix prescription:**

ADD ONE genuine end-to-end Component Model dispatch test:

```rust
#[test]
fn test_F_PASS4_HIGH_001_component_model_dispatch_http_request_via_export() {
    // WAT that exports a function which calls the host::http-request import
    let wat = r#"
        (component
          (import "host" "http-request" (func $http_req
            (param "method" string) (param "url" string)
            (param "headers" (list (tuple string string)))
            (param "body" (option string))
            (result (record (field "status" u16)
                             (field "headers" (list (tuple string string)))
                             (field "body" (option string))))))
          (func (export "invoke-http") (result u16)
            (let $resp (call $http_req
              (string.const "GET")
              (string.const "https://api.example.com/data")
              (list.new 0)
              (option.none string)))
            (record.get "status" (local.get $resp))))
    "#;
    let component_bytes = wat::parse_str(wat).expect("WAT parse failed");
    let component = Component::from_binary(&ENGINE, &component_bytes)
        .expect("Component compilation failed");
    let mut store = Store::new(&ENGINE, make_host_state_with_allowed_url("https://api.example.com/data"));
    let linker = build_prism_linker(&ENGINE);
    let instance = linker.instantiate(&mut store, &component)
        .expect("Instantiation failed");
    let invoke_http = instance.get_func(&mut store, "invoke-http")
        .expect("export not found");
    let mut results = vec![Val::U16(0)];
    invoke_http.call(&mut store, &[], &mut results)
        .expect("dispatch failed");
    // 200 for allowlisted URL (host::http-request returns actual HTTP response)
    // OR 403 if using mock blocked response
    match &results[0] {
        Val::U16(status) => assert!(*status == 200 || *status == 403,
            "expected u16 status from Component Model dispatch; got {}", status),
        other => panic!("expected Val::U16 from registered http-request callback; got {:?}", other),
    }
}
```

The exact WAT syntax may differ (per wasmtime Component Model WAT encoding); the key requirement is:
- WAT defines an EXPORT that calls the HOST IMPORT
- Test calls the EXPORT via `.call()`
- Test asserts on the return value from the PRODUCTION callback, not a hand-built value

Inline-replica tests (1-4 above) may remain as supplementary unit coverage but cannot be the sole evidence.

---

### F-PASS4-MED-001 — Story Body Sibling-Sweep Gap (TD-VSDD-060)

**Severity:** MED
**Routing:** story-writer
**Status:** OPEN

**Location:** `S-PLUGIN-PREREQ-D.md` §Structured Event Catalog Additions, lines 790 and 954

**Description:**

Story §Structured Event Catalog Additions contains two occurrences of "12 events" / "12 new event types":
- Line 790: "The following 12 events are added to BC-2.16.002 Structured Event Catalog..."
- Line 954: "§Structured Event Catalog Additions (12 new event types across all host callbacks)"

Fix-burst-impl-3 added row 32 (`plugin_log_level_unrecognized`) to BC-2.16.002 v1.17 via factory commit `d8f51552`. This brings the story's total contribution to BC-2.16.002 to **13 events** (rows 20-32 inclusive, 13 events: `plugin_load_started`, `plugin_loaded`, `plugin_load_failed`, `plugin_load_audit_durably_persisted`, `plugin_http_request_dispatched`, `plugin_http_request_blocked`, `plugin_log_emitted`, `plugin_config_fetched`, `plugin_kv_get`, `plugin_kv_set`, `plugin_load_manifest_parse_error`, `plugin_load_manifest_not_found`, `plugin_log_level_unrecognized`).

Story body was not swept when BC-2.16.002 was amended via `d8f51552`. This is a TD-VSDD-060 sibling-sweep failure: when a count-bearing field is changed (BC-2.16.002 row count), all sibling documents citing that count must be swept.

**Fix prescription:**

Story v1.33 → v1.34:
1. Line 790: `"12 events"` → `"13 events"`
2. Line 954: `"12 new event types"` → `"13 new event types"`
3. Append `plugin_log_level_unrecognized` row to the §Structured Event Catalog Additions table (follow existing row format)
4. Update `modified:` / `updated:` frontmatter date
5. Sync STORY-INDEX.md row for S-PLUGIN-PREREQ-D: bump version reference

---

## Process-Gap Candidates (Codification Queue 25 → 26)

### PG-IMPL-LP4-001 — Test Paper-Fix Detector for Fix-Burst-Impl-N Adversary

**Pattern observed (4th recurrence across passes 1-4):** When adversary finding specifies "add a test that exercises X end-to-end through registered callback Y," the implementer closes by writing a test that hand-constructs X's return values and asserts against the hand-built values, rather than invoking a Component Model host function via the linker + export call path.

**Proposed codification:** Adversary dispatch prompt for `fix-burst-impl-N` must include a positive-coverage check question: "For each new test introduced by this fix-burst, does the test invoke the production function via a REGISTERED callback (i.e., through `linker.get_func(...)` + `.call()` on an exported function), or does it construct a copy of the expected return value and compare? If the latter: PAPER FIX — the test would NOT catch a regression in the production callback body."

Implementer self-prompt must include: "Before marking a test as closing 'test that exercises X through registered Y': (a) Does my test call a function that the Component Model registers as a host import? (b) Does the test invoke that function via an exported Component Model function (not directly)? (c) Would a silent type change in the production callback body cause this test to fail?"

**Routing:** session-reviewer at cycle-close adjudication. NOT added to policies.yaml this burst per codification queue routing discipline.

---

## Policy Verification Summary

| Policy | Verdict | Notes |
|--------|---------|-------|
| POL-14 (BC promotion at merge) | PASS | No promotion needed this burst (adversary-only pass; no spec amendments) |
| POL-15 (boot-step gate ordering) | PASS | `plugin_load_step_with_audit` BEFORE `step7_init_storage` VERIFIED |
| POL-18 (required-features test gate) | PASS | No new test blocks added this burst |
| TD-VSDD-053 (single-commit-per-burst) | PASS | D-553 is 58th consecutive single-commit |
| TD-VSDD-059 (paper-fix detection) | FIRING | F-PASS4-HIGH-001 — 4th paper-fix recurrence; production code is correct but test evidence is paper |
| TD-VSDD-060 (sibling-sweep on count changes) | FIRING | F-PASS4-MED-001 — story body not swept when BC-2.16.002 row count changed |
| BC-5.39.001 (3-CLEAN protocol) | 0/3 | BLOCKED; streak does not advance |

---

## Trajectory Analysis

### 4-Pass Arc

| Pass | Total | CRIT | HIGH | MED | LOW | Streak | Fix-Burst |
|------|-------|------|------|-----|-----|--------|-----------|
| impl-pass-1 | 18 | 3 | 6 | 7 | 2 | 0/3→0/3 | fix-burst-impl-1 CLOSED 18/18 (D-548) |
| impl-pass-2 | 12 | 2 | 3 | 6 | 1 | 0/3→0/3 | fix-burst-impl-2 CLOSED 12/12 (D-550) |
| impl-pass-3 | 6 | 3 | 1 | 2 | 0 | 0/3→0/3 | fix-burst-impl-3 CLOSED 6/6 (D-552) |
| impl-pass-4 | 2 | 0 | 1 | 1 | 0 | 0/3→0/3 | fix-burst-impl-4 ⏳ NEXT |

**Total findings closed (passes 1-3):** 18+12+6 = 36 CLOSED.
**Open (pass-4):** 2 in-perimeter.
**Magnitude decay:** 18→12→6→2 (halving each pass; clean convergence signal).
**CRIT decay:** 3→2→3→0 (oscillated; now zero).

### Assessment

The implementation has genuinely improved across 4 passes. Production code is verified correct for all pass-3 closures. The remaining gap is narrow: test evidence for Component Model dispatch. Fix-burst-impl-4 is the smallest fix-burst in this cascade: 1 dispatch test + 1 story body sweep. After fix-burst-impl-4, impl-pass-5 is expected to advance the streak to 1/3 (optimistic: 0 findings = first CLEAN pass).

---

## Next-Pass Dispatch Template

**Dispatch:** fix-burst-impl-4 (implementer + story-writer parallel)

**Implementer task:**
- ADD ONE genuine Component Model dispatch test for `host::http-request` callback
- Test must: (a) build WAT with exported function that calls host::http-request; (b) instantiate against Prism linker; (c) invoke exported function via `.call()`; (d) assert returned `Val::Record` status field
- Verify test fails if `Val::U16` is changed to `Val::U32` in production code (regression detection proof)
- `just check` must pass (expect baseline 3643 + 1 = 3644)
- No other changes in scope for implementer

**Story-writer task:**
- Story S-PLUGIN-PREREQ-D v1.33 → v1.34
- Bump "12 events" → "13 events" at lines 790 and 954
- Append `plugin_log_level_unrecognized` row to §Structured Event Catalog Additions table
- Sync STORY-INDEX row

**Factory-only changes (state-manager at D-NNN after fix-burst-impl-4 closure):**
- Story v1.34 registration
- STORY-INDEX sync

**After fix-burst-impl-4:** Dispatch adversary impl-pass-5. Target: 0/3 → 1/3 streak advance (first CLEAN pass; convergence optimistic if no new findings introduced).

**Carry-forward for impl-pass-5 adversary:**
- Re-verify F-PASS4-HIGH-001 closure: does the new dispatch test invoke a registered callback via exported Component Model function?
- Re-verify F-PASS4-MED-001 closure: story says "13 events" + table has 13 rows
- Scan for net-new findings introduced by fix-burst-impl-4
- Apply PG-IMPL-LP4-001 positive-coverage check to all new tests
