# PR Review Findings — W3-FIX-SEC-001 (Cycles 1–2)

**Reviewer:** vsdd-factory:pr-review-triage (fresh-context, cycle 1)
**PR:** #113 — feature/W3-FIX-SEC-001 vs develop
**Date:** 2026-05-01
**Story spec:** .factory/stories/W3-FIX-SEC-001-x-org-id-auth-enforcement.md

---

## Convergence Table

| Cycle | Findings | Blocking | Non-Blocking | Fixed | Remaining |
|-------|----------|----------|--------------|-------|-----------|
| 1 | 2 | 1 | 1 | 0 | 1 blocking |
| 2 | 0 | 0 | 0 | 2 | 0 → **APPROVE** |

---

## Verdict: APPROVE (Cycle 2)

All blocking findings resolved. No new issues found in fix commits. All remaining `extract_org_id` call sites verified as correct (backward-compat else-branches or Cyberint model-B fallback pattern). Implementation is complete and correct for all four DTU clones.

---

## BLOCKING Findings

### REVIEW-001: Claroty tags.rs — add_tag and remove_tag handlers use extract_org_id without validate_org_id guard

**Severity:** BLOCKING
**File:** `crates/prism-dtu-claroty/src/routes/tags.rs:71` (add_tag), `:101` (remove_tag)
**Story ref:** Task 4: "Propagate the `validate_org_id` guard to **all route handlers** in `prism-dtu-claroty` that currently call `extract_org_id`."

**Description:**

The `add_tag` and `remove_tag` handlers in `tags.rs` call the local `extract_org_id` function (which falls back to the sentinel UUID with no validation) without any `validate_org_id` guard. Both handlers mutate `ClarotyState::tag_store` keyed by `(org_id, device_id)`.

```rust
// tags.rs:71 — NO validate_org_id guard
pub async fn add_tag(...) {
    if let Err(err) = check_bearer_auth(&headers) { return err; }
    let org_id = extract_org_id(&headers);  // ← accepts any UUID from wire
    state.add_tag(org_id, &device_id, &body.tag_key);
    ...
}

// tags.rs:101 — NO validate_org_id guard
pub async fn remove_tag(...) {
    if let Err(err) = check_bearer_auth(&headers) { return err; }
    let org_id = extract_org_id(&headers);  // ← accepts any UUID from wire
    state.remove_tag(org_id, &device_id, &tag_key);
    ...
}
```

A caller connected to Org A's Claroty clone port can write or delete tags in Org A's tag store by supplying `X-Org-Id: <OrgA-UUID>` — this is the original SEC-001 attack vector applied to write endpoints. The `list_devices` endpoint is now protected but the two mutation endpoints remain unprotected.

The `tags.rs` doc comment still reads "S-3.2.01 — Multi-tenant stub" and explicitly describes the `extract_org_id` as a "structural placeholder until the middleware layer is wired up." This is the same "structural placeholder" language that was the root cause of SEC-001.

**Required fix:** Apply the same nil-org-guard + `validate_org_id` pattern used in `list_devices` (devices.rs:227-234) to `add_tag` and `remove_tag` in `tags.rs`. Import `validate_org_id` from `devices.rs` (it is already `pub(crate)`).

**Pattern to follow (from devices.rs:227-234):**
```rust
let nil_org = OrgId::from_uuid(Uuid::nil());
let org_id = if state.instance_org_id != nil_org {
    match validate_org_id(&headers, state.instance_org_id) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    }
} else {
    extract_org_id(&headers)
};
```

Note: `add_tag` and `remove_tag` return `(StatusCode, Json<Value>)` directly (not `impl IntoResponse`), so the return type handling will need a small adjustment.

---

## NON-BLOCKING Findings

### REVIEW-002: Test stale comments reference "todo!()" stub — cosmetic only

**Severity:** NON-BLOCKING (cosmetic)
**Files:** `tests/x_org_id_auth.rs` for Claroty and CrowdStrike

Several test assertion messages still reference "validate_org_id is not yet wired into route handlers" (e.g., `claroty/tests/x_org_id_auth.rs:119`). These are copy-paste artifacts from the Red Gate phase and are now factually incorrect since the tests pass. They do not affect correctness.

**Suggestion:** Update assertion messages to remove the "not yet wired" phrasing.

---

## AC Coverage Assessment

| AC | Status | Notes |
|----|--------|-------|
| AC-001 | PASS | test_AC_001_x_org_id_validated_against_bearer_token in all 4 crates |
| AC-002 | PASS | test_AC_002_cross_org_credential_returns_401 + body test in all 4 crates |
| AC-003 | PASS (with per-clone model) | Model A: 401. Model B (Cyberint): 200 default. Validate-on-presence (Armis): 200. Documented. |
| AC-004 | PASS | 30 x_org_id_auth tests across 4 crates |
| AC-005 | PASS | test_cross_org_header_rejected in all 4 crates |
| AC-006 | PASS | pre-existing multi_tenant suite: 0 regressions |

---

## Edge Case Coverage

| EC | Status | Notes |
|----|--------|-------|
| EC-001 | PASS | Non-UUID → 401 tested in all 4 crates |
| EC-002 | N/A | Story doc notes: comparison is byte-for-byte; v4 UUIDs accepted if they match |
| EC-003 | PASS | Sentinel UUID → 401 tested in all 4 crates |
| EC-004 | PASS | reset_for path param → 403 tested in Claroty (only Claroty has reset_for endpoint) |
| EC-005 | Not tested explicitly | Concurrent clones: each validates against own instance_org_id; no cross-clone state sharing. Implicitly covered by test isolation (each test spawns its own clone on random port). |

---

## Architectural Compliance

| Rule | Status |
|------|--------|
| validate_org_id named (not extract_org_id) | PASS — rename done in all 4 crates |
| 401 response body is JSON {"error": "..."} | PASS — all validate_org_id implementations use serde_json::json! |
| No new Cargo dependencies | PASS — only [[test]] section added to Cargo.toml files |
| validate_org_id implemented locally per-crate | PASS — each crate has its own impl; no cross-crate import for production helpers |
| extract_org_id retained for backward-compat nil-org path | PASS — correct; only used in else branch after nil check |

---

## Code Quality Notes

- The `validate_org_id` in Cyberint `alerts.rs` is `#[allow(dead_code)]` because auth model B implements the check inline in `check_auth`. This is architecturally intentional and documented in the code. Acceptable.
- CrowdStrike `writes.rs` imports `validate_org_id` from `hosts.rs` via `crate::routes::hosts::validate_org_id`. This is a within-crate import — acceptable per the story's "implement locally within each crate's routes module" rule.
- All four `validate_org_id` implementations are identical in structure (missing→401, non-UUID→401, mismatch→401) which is correct. The divergence is only at the call site (guard presence/absence per auth model).

---

## Summary

One BLOCKING finding (REVIEW-001: Claroty tags.rs unguarded write handlers) must be resolved before merge. The BLOCKING finding represents the same class of gap that SEC-001 was filed to fix — two state-mutating endpoints in Claroty accepting wire-supplied org UUIDs without validation.

All other aspects of the implementation are correct and well-structured.

---

_Generated: 2026-05-01 | Reviewer: vsdd-factory:pr-review-triage cycle 1_

---

## Cycle 2 — Verification of REVIEW-001 Fix

**File verified:** `crates/prism-dtu-claroty/src/routes/tags.rs` (commit 17a881c4)

**add_tag fix (line 74-84):**
```rust
let nil_org = OrgId::from_uuid(Uuid::nil());
let org_id = if state.instance_org_id != nil_org {
    match validate_org_id(&headers, state.instance_org_id) {
        Ok(id) => id,
        Err(err) => return err,   // (StatusCode, Json<Value>) — type matches handler return
    }
} else {
    extract_org_id(&headers)
};
```

**remove_tag fix (line 115-125):** identical nil-org-guard pattern. ✓

**Return type check:** `validate_org_id` returns `Result<OrgId, (StatusCode, Json<serde_json::Value>)>`. Both handlers return `(StatusCode, Json<Value>)` where `Value = serde_json::Value`. Types match. ✓

**Import:** `validate_org_id` imported from `crate::routes::devices` (already `pub(crate)`). ✓

**Compilation:** `cargo check -p prism-dtu-claroty --all-features` returned clean. ✓

## Cycle 2 — Remaining extract_org_id Scan (all four src/ dirs)

All 17 remaining call sites categorized:

| File | Line | Category | Verdict |
|------|------|----------|---------|
| claroty/routes/tags.rs:83 | else-branch after nil-org-guard | CORRECT |
| claroty/routes/tags.rs:124 | else-branch after nil-org-guard | CORRECT |
| claroty/routes/devices.rs:233 | else-branch after nil-org-guard | CORRECT |
| crowdstrike/routes/hosts.rs:302 | after validate guard at 294-298 | CORRECT |
| crowdstrike/routes/writes.rs:108 | after validate guard at 102-106 | CORRECT |
| crowdstrike/routes/writes.rs:254 | after validate guard at 248-252 | CORRECT |
| cyberint/routes/auth.rs:36 | model-B login (fallback to instance_org_id) | CORRECT |
| cyberint/routes/dtu.rs:66 | DTU management reset (not sensor data path) | CORRECT |
| cyberint/routes/threats.rs:54 | model-B (is_valid_session enforces org gate) | CORRECT |
| cyberint/routes/alerts.rs:145 | model-B check_auth (is_valid_session gate) | CORRECT |
| cyberint/routes/alerts.rs:212,262,329,374 | model-B data handlers (session-gated) | CORRECT |

No unguarded call sites remain in sensor-data route handlers for model A crates (Claroty, CrowdStrike).

_Cycle 2 completed: 2026-05-01 | Reviewer: vsdd-factory:pr-review-triage_
