---
document_type: adversarial-review
cycle: wave-0-plugin-prereqs
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: 2
type: LOCAL
date: 2026-05-30
feature_head: "79e3b545"
clean_strict: false
clean_pr_merge: false
findings_count: 4
findings_by_severity:
  CRIT: 1
  HIGH: 1
  MED: 1
  LOW: 0
  OBS: 0
  PROCESS-GAP: 1
streak_after_pass: 0
target_streak: 3
status: "FIX_BURST_DISPATCHED — PO re-adjudicated at 2707ee69 (BC v1.2); implementer follow-on for test split + F-LP2-HIGH-001; SAP-2 TD entry by state-manager (this burst); F-LP2-PG-001 process-gap lesson"
po_re_adjudication_commit: "2707ee69 (BC-2.01.017 v1.2 / BC-INDEX v5.58)"
implementer_follow_on_dispatch: "test split + F-LP2-HIGH-001 — in flight at feature/S-DTU-CYBERINT-AUTH-FIDELITY-001"
---

# LOCAL Adversary Pass 2 — S-DTU-CYBERINT-AUTH-FIDELITY-001

**Feature HEAD at pass:** `79e3b545`
**Date:** 2026-05-30
**Result:** NOT CLEAN (strict: NO, PR-merge: NO)
**Findings:** 4 total — 1 CRIT + 1 HIGH + 1 MED + 1 PROCESS-GAP
**Streak:** 0/3 (reset)
**Novelty:** HIGH — inversion of Pass-1 closure trail

---

## Findings (4 total)

---

### F-LP2-CRIT-001 — BC-2.01.017 v1.1 EC-017-005 rests on a fabricated claim about BC-2.03.006 normalization

**Severity:** CRITICAL
**Category:** Spec correctness / PO adjudication built on fabricated evidence
**BC anchor:** BC-2.01.017 v1.1 §EC-017-005; BC-2.03.006

**Evidence:**

Pass 1 PO adjudication (D-852, commit `4baa0e91`) amended BC-2.01.017 EC-017-005 to mandate `E-AUTH-005` (not-found path) based on the stated rationale: "BC-2.03.006 normalizes empty env-var as not-found → Ok(None) → E-AUTH-005." The adversary independently re-derived the actual `resolve_secret` behavior from source:

```rust
// crates/prism-credentials/src/resolve_secret.rs:78-81
EnvVar { var_name } => {
    let val = std::env::var(var_name).ok();
    Ok(val.map(SecretString::new))
}
```

Behavioral derivation:
- `var_name` unset → `std::env::var` returns `Err` → `.ok()` yields `None` → `Ok(None)` — this is the NOT-FOUND path. ✓
- `var_name` set to `""` → `std::env::var` returns `Ok("")` → `.ok()` yields `Some("")` → `Ok(Some(SecretString("")))` — this is the EMPTY STRING path, NOT not-found.

The fabricated claim: "BC-2.03.006 normalizes empty values as not-found." The actual code does NOT normalize empty strings as not-found. `resolve_secret` returns `Ok(Some(SecretString("")))` for an env-var set to the empty string. The normalization to not-found / `E-AUTH-005` happens ONLY at the consumer layer (`StaticCookieAuthProvider::authenticate`) if the consumer checks for empty strings explicitly.

The PO adjudication note did not quote the actual `resolve_secret` source. The claim was derived from spec narrative (BC-2.03.006 prose) rather than the code. The BC-2.03.006 prose says "normalizes empty or missing credentials as not-found" — but this describes the CONSUMER's responsibility, not the resolver's behavior. The resolver faithfully returns `Ok(Some(""))` for an empty env-var.

**Consequence:** BC-2.01.017 v1.1 EC-017-005 now mandates `E-AUTH-005` for the "empty api_key" case. But the actual call chain for an empty env-var is:
1. `resolve_secret` → `Ok(Some(SecretString("")))`
2. `StaticCookieAuthProvider::authenticate` receives a non-None `SecretString` with empty content
3. If the auth provider checks for empty content → it can return any error code it chooses
4. BC-2.01.017 v1.0 mandated `E-AUTH-006` for this exact case (empty api_key), which is correct: the value IS present (not missing), but IS invalid (empty). E-AUTH-006 = "api_key present but invalid format."

**Closure route:** PO must re-adjudicate. The correct resolution for an empty env-var `api_key`:
- Option A (revert to v1.0): BC-2.01.017 EC-017-005 reverts to `E-AUTH-006` (empty value → invalid format, consistent with `resolve_secret` returning `Ok(Some(""))`). `StaticCookieAuthProvider` must check for empty and return `E-AUTH-006`.
- Option B (preserve v1.1): BC-2.01.017 EC-017-005 retains `E-AUTH-005` (not-found semantics), BUT requires that `StaticCookieAuthProvider` convert `Ok(Some(""))` to not-found before returning. The implementer must add an empty-string guard. AND BC-2.03.006 normalization claim must be corrected — the normalization happens at the consumer, not the resolver.

**Adversary recommendation:** Option A is the cleaner contract. The resolver is a pure translation layer; empty env-var → `Ok(Some(""))` is correct behavior. The auth provider's responsibility is to validate the value. BC-2.01.017 v1.0 had the right semantics.

**Reproduction:**
```bash
# Verify resolve_secret returns Ok(Some("")) for empty env-var
# crates/prism-credentials/src/resolve_secret.rs — EnvVar arm
# std::env::var("KEY_SET_TO_EMPTY") where KEY="KEY_SET_TO_EMPTY" set to ""
# returns Ok("") → .ok() = Some("") → Ok(Some(SecretString("")))
```

**CLEAN (strict):** NO
**CLEAN (PR-merge):** NO

---

### F-LP2-HIGH-001 — `unsafe { std::env::set_var }` remains in 3 retained tests after cleanup burst

**Severity:** HIGH
**Category:** Code / SID-1 violation — no-ignored-test rationalization; unsafe env manipulation in tests
**BC anchor:** BC-2.01.017 v1.1; BC-2.03.006
**Code anchor:** `prism-spec-engine/src/auth_provider.rs` test module

**Evidence:**

The Pass 1 cleanup burst (`79e3b545`) refactored 3 tests from `unsafe { std::env::set_var }` to `MockCredentialResolver` / `NotFoundCredentialResolver` injection. The adversary's local-pass-1.md noted F-LP1-LOW-002 as targeting 3 tests in `prism-spec-engine/src/auth_provider.rs`.

The cleanup burst commit message states: "3 unit tests refactored to MockCredentialResolver, TD-VSDD-059 E-AUTH-005 assert added." BUT the adversary independently grepped the file post-cleanup:

```bash
rg 'set_var|unsafe' crates/prism-spec-engine/src/auth_provider.rs
```

There remain `unsafe { std::env::set_var }` calls in the tests that explicitly test the `EnvVar` credential backend path (the `BC-2.03.006 backend coverage` test cited in the cleanup burst's commit note as intentionally retained with a `// SAFETY` justification).

The issue: the retained `unsafe set_var` test is testing the WRONG boundary. The intent (per commit message) is to test BC-2.03.006 backend coverage — i.e., that the production env-var resolution path correctly reaches the `resolve_secret` → `EnvVar` arm. However, this test ALSO depends on EC-017-005 behavior (what error code the auth provider returns for an empty env-var), which is the contested claim in F-LP2-CRIT-001.

If EC-017-005 is reverted to `E-AUTH-006` (PO's Option A for F-LP2-CRIT-001), the TD-VSDD-059 assertion `assert!(err_str.contains("E-AUTH-005"))` added in the cleanup burst will FAIL. This creates a test-spec alignment dependency: the test must be split into two:
- Test A: verify the `EnvVar` backend path is reached (BC-2.03.006 coverage) — inject a valid non-empty env-var, assert Ok result
- Test B: verify the empty-value error code (BC-2.01.017 EC-017-005) — use MockCredentialResolver returning `Ok(Some(""))`, assert the correct error code (whichever PO picks)

**Closure route:** After PO re-adjudicates F-LP2-CRIT-001, implementer must split the retained `unsafe set_var` test into Test A (EnvVar path coverage, no error-code assertion) and Test B (empty-value error-code assertion using MockCredentialResolver). The `unsafe set_var` test can be removed after the split.

**CLEAN (strict):** NO
**CLEAN (PR-merge):** NO

---

### F-LP2-MED-001 — SAP-2 gap: `affected_assets` field in Cyberint Alert DTU has no TOML column (ColumnType::Array not yet supported)

**Severity:** MEDIUM
**Category:** SAP-2 DTU↔TOML schema parity / missing coverage
**BC anchor:** BC-2.01.017; SAP-2 (CLAUDE.md §Standing Adversary Probes)
**Code anchor:** `crates/prism-dtu-cyberint/src/types.rs` — `Alert::affected_assets`; `crates/prism-sensors/specs/cyberint.sensor.toml` — `[[tables]] table_name = "alerts"` block

**Evidence:**

SAP-2 mandatory check: for every column declared in `cyberint.sensor.toml [[tables]]` blocks, verify the column name maps to a DTU field in `crates/prism-dtu-cyberint/src/types.rs`. Adversary also checked the reverse: DTU fields with no TOML column.

In `crates/prism-dtu-cyberint/src/types.rs`:
```rust
pub struct Alert {
    // ... other fields ...
    pub affected_assets: Vec<serde_json::Value>,
}
```

In `crates/prism-sensors/specs/cyberint.sensor.toml` `[[tables]] table_name = "alerts"` block: **no column for `affected_assets`**.

This is a F-in-DTU-with-no-TOML-column finding per SAP-2 §4: "Field in DTU with no TOML column → MEDIUM (missing coverage, not a runtime crash)."

**Root cause:** `prism_core::column::ColumnType` does not have an `Array` variant. Adding `affected_assets: Vec<serde_json::Value>` as a TOML column would require:
- (a) `ColumnType::Array` variant in `prism-core`
- (b) TOML schema parser support for array columns
- (c) Field mapper support for array→DataFusion column conversion
- (d) DataFusion array column type support

This is genuinely cross-component scope requiring architect decision.

**Closure route:** Tech-debt entry (this burst, per orchestrator dispatch). Future story `S-FOLLOWUP-ARRAY-COLUMNTYPE` (stub to be materialized when ColumnType::Array work is scoped). The deferral has a concrete future dependency (ColumnType::Array implementation) — this satisfies CLAUDE.md Rule 3's three requirements: (a) orchestrator concurs with deferral, (b) concrete future dependency named, (c) story anchor named.

**CLEAN (strict):** NO (MED finding)
**CLEAN (PR-merge):** NO (MED finding)

---

### F-LP2-PG-001 — PO adjudication workflow: Pass 1 PO Option A was built on narrative claim without code verification

**Severity:** PROCESS-GAP
**Category:** Process / PO adjudication workflow gap
**BC anchor:** BC-2.01.017; BC-2.03.006
**Source:** D-852 adjudication commit `4baa0e91`

**Evidence:**

The D-852 PO adjudication note (`.factory/cycles/wave-0-plugin-prereqs/S-DTU-CYBERINT-AUTH-FIDELITY-001/po-adjudications/F-LP1-MED-002.md`) states the resolution basis as: "BC-2.03.006 (credential-backend, more specific) normalizes Ok(None) → not-found → E-AUTH-005." This is a spec-narrative derivation, not a code quote.

The F-LP2-CRIT-001 finding above demonstrates that the spec-narrative claim was incorrect: `resolve_secret` returns `Ok(Some(""))` for an empty env-var, NOT `Ok(None)`. The PO did not quote `crates/prism-credentials/src/resolve_secret.rs:78-81` before asserting the normalization claim.

**Rule codification (F-LP2-PG-001):** Future PO adjudications resolving code-vs-spec conflicts MUST include verbatim code quotes from the cited code path — file name + function/line anchor + actual code text. This is the direct prevention pattern for the fabrication class caught here. See lesson 56 in `cycles/wave-0-plugin-prereqs/lessons.md`.

**Orchestrator check:** When reviewing PO adjudication output for "code returns X but BC mandates Y" class findings, scan for direct code quotes. If absent, route back to PO for evidence supplementation before accepting the adjudication.

**CLEAN (strict):** NO (PROCESS-GAP finding)
**CLEAN (PR-merge):** YES (no CRIT/HIGH/MED from PROCESS-GAP alone — but CRIT-001 + HIGH-001 + MED-001 make PR-merge also NO)

---

## Cascade State After Pass 2

### PO Re-adjudication (F-LP2-CRIT-001)

Orchestrator independently verified F-LP2-CRIT-001 against `crates/prism-credentials/src/resolve_secret.rs:78-81` — the empty-string-filter does NOT exist at the resolver layer. PO re-adjudicated and committed revert at `2707ee69`:

- BC-2.01.017 v1.2 — EC-017-005 reverts to `E-AUTH-006` semantics (empty value → invalid format, not not-found)
- BC-INDEX v5.58

F-LP2-CRIT-001 status: CLOSED by PO revert `2707ee69`.

### Implementer Follow-on Dispatch

Dispatched to `feature/S-DTU-CYBERINT-AUTH-FIDELITY-001` worktree. Scope:
1. **Test split** (F-LP2-HIGH-001): split retained `unsafe set_var` test in `prism-spec-engine/src/auth_provider.rs` into (a) EnvVar backend-path coverage test and (b) empty-value error-code test using MockCredentialResolver; align E-AUTH-006 assertion with BC-2.01.017 v1.2.
2. **E-AUTH-006 guard** (F-LP2-HIGH-001 / BC-2.01.017 v1.2): ensure `StaticCookieAuthProvider::authenticate` returns `E-AUTH-006` when resolver returns `Ok(Some(""))` (empty string), not `E-AUTH-005`.

### State-Manager Dispatch (This Burst)

This burst performs:
- Pass 2 report persistence (`adversarial-review/local-pass-2.md`) — this file
- `adversary-convergence-state.json` Pass 2 entry update
- STATE.md v7.540 (frontmatter + decisions log D-854 + current phase steps + session checkpoint)
- Lesson 56 [process-gap] append to `cycles/wave-0-plugin-prereqs/lessons.md`
- TD entry F-LP2-MED-001 `affected_assets` array column gap to `tech-debt-register.md`
- SESSION-HANDOFF.md §ADDENDUM 2026-05-30-PASS-2-FIX-BURST-IN-FLIGHT

---

## Novelty Assessment

**Novelty: HIGH**

Pass 2 did not merely resurface Pass 1 findings in a new form. The CRIT finding (F-LP2-CRIT-001) is a direct INVERSION of a Pass 1 closure: PO's Option A adjudication (D-852, `4baa0e91`) is itself defective because it was based on a fabricated claim about `resolve_secret` behavior. The Pass 1 cascade's PO adjudication — the resolution step for the most-contested finding — was built on incorrect premises that only fresh-context independent code re-derivation could catch. This is a genuine novel finding class: PO fabrication of "normalization behavior" without code verification.

The HIGH finding (F-LP2-HIGH-001) is a derivative consequence: the TD-VSDD-059 assertion added in the cleanup burst (`79e3b545`) asserts the wrong error code because EC-017-005 was wrong. The assertion itself is load-bearing — it will break the next `just check` once the PO revert lands — but fixing it requires the test split.

---

## CLEAN Assessment

**CLEAN (strict):** NO — 4 findings present (1 CRIT + 1 HIGH + 1 MED + 1 PROCESS-GAP)
**CLEAN (PR-merge):** NO — CRIT + HIGH + MED findings all present

**Streak:** 0/3 (reset from Pass 1's position of 0/3 — no change in streak counter, but the findings are of different character)

---

## Next Steps

1. **PO re-adjudication** (F-LP2-CRIT-001) — BC-2.01.017 v1.2 revert. [DISPATCHED — committed `2707ee69`]
2. **Implementer fix-burst** — test split + E-AUTH-006 guard (F-LP2-HIGH-001). [DISPATCHED — in flight at `feature/S-DTU-CYBERINT-AUTH-FIDELITY-001`]
3. **State-manager** — Pass 2 persistence + SAP-2 TD (F-LP2-MED-001) + lesson 56 (F-LP2-PG-001). [THIS BURST — D-854]
4. **D-855 closure burst** (state-manager) — after implementer returns: record test split + feature HEAD update
5. **Pass 3 LOCAL adversary** — dispatch after D-855 against new feature HEAD; verify F-LP2-CRIT-001 closure (BC-2.01.017 v1.2 + E-AUTH-006 guard) + F-LP2-HIGH-001 closure (test split) + SAP-1/SAP-2/SID-1 probes
