---
document_type: adversarial-review
cycle: wave-0-plugin-prereqs
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: 3
type: LOCAL
date: 2026-05-30
feature_head: "d697425b"
clean_strict: false
clean_pr_merge: false
findings_count: 5
findings_by_severity:
  CRIT: 0
  HIGH: 1
  MED: 2
  LOW: 2
  OBS: 0
  PROCESS-GAP: 0
streak_after_pass: 0
target_streak: 3
status: "FIX_BURST_NEEDED — F-LP3-HIGH-001 BackendUnavailable mis-mapped to E-AUTH-005; F-LP3-MED-001/002 doc/comment sweeps; F-LP3-LOW-001 validation order; F-LP3-LOW-002 deferred maintenance"
deferred_findings: ["F-LP3-LOW-002"]
deferred_findings_route: "maintenance follow-up story (NOT this cascade)"
prior_closure_re_verified:
  F-LP2-CRIT-001: "load-bearing (test split tests verified via reverse-tracing acquire_token)"
  F-LP2-HIGH-001: "load-bearing (MockCredentialResolver injection asserts AuthToken content)"
  PO_revert_2707ee69: "verified — BC v1.2 EC-017-005=E-AUTH-006, EC-017-003=E-AUTH-005; resolve_secret.rs:78-81 no empty filter"
---

# S-DTU-CYBERINT-AUTH-FIDELITY-001 — LOCAL Adversarial Review Pass 3

**Date:** 2026-05-30  
**Feature HEAD:** `d697425b`  
**Streak before pass:** 0/3  
**Streak after pass:** 0/3 (NOT CLEAN — fix-burst required)

---

## Findings Table

| ID | Severity | Routing | Anchor | Description |
|----|----------|---------|--------|-------------|
| F-LP3-HIGH-001 | HIGH | implementer + PO | `StaticCookieAuthProvider::acquire_token` `.map_err(...)` + `CredentialResolutionError::BackendUnavailable` | `acquire_token` blanket-labels every resolver `Err` as `E-AUTH-005`, mis-categorizing `BackendUnavailable` as "credential not found" |
| F-LP3-MED-001 | MED | implementer | `crates/prism-dtu-cyberint/src/lib.rs:5` module-doc | Module documentation still advertises `POST /login — cookie-based auth (CookieRoundtrip pattern)` after AC-001 deleted that route |
| F-LP3-MED-002 | MED | implementer | `crates/prism-spec-engine/tests/parity/cyberint.rs:144` | Parity test comment claims "DTU cookie check validates non-empty cyberint_session cookie" — stale narrative after auth-model rewrite |
| F-LP3-LOW-001 | LOW | implementer | `auth_provider.rs::acquire_token` validation block | `is_empty() OR all-whitespace` runs before `len() > 4096`; 5000-byte all-whitespace key is reported as "empty or all-whitespace" rather than "exceeds 4096-byte limit" |
| F-LP3-LOW-002 | LOW | DEFERRED | `crates/prism-bin/tests/plugin_boot_tests.rs` ~lines 167/192/210/489 | 4 `unsafe { std::env::set_var }` sites cite only generic thread-safety in SAFETY comments, do not anchor BC-2.03.009 — deferred to maintenance follow-up |

---

## Finding Details

### F-LP3-HIGH-001 — BackendUnavailable Mis-Mapped to E-AUTH-005

**Severity:** HIGH  
**Routing:** implementer (impl fix) + PO (E-AUTH-007 allocation + BC-2.01.017 v1.2→v1.3 amendment)

**Anchor:**  
- Production code: `crates/prism-spec-engine/src/auth_provider.rs::StaticCookieAuthProvider::acquire_token` — the `.map_err(...)` block labels every resolver `Err` branch with `detail: format!("E-AUTH-005: credential not found: {e}")`
- Error type: `crates/prism-credentials/src/resolution.rs::CredentialResolutionError::BackendUnavailable`

**Description:**  
`acquire_token` maps the entire `Err(e)` path from `resolve_credential()` to a single `E-AUTH-005` error with the message `"E-AUTH-005: credential not found: {e}"`. This conflates two structurally distinct failure modes:

1. `CredentialResolutionError::NotFound` — backend is functional, no credential entry exists for the requested handle. This IS the intended `E-AUTH-005` case per error-taxonomy.md v1.53 §E-AUTH-005: "No credentials in keyring or file backend."
2. `CredentialResolutionError::BackendUnavailable` — the backend itself is unavailable (e.g., `_FILE` env-var points to a path that is unreadable, keyring service is down). This is a backend infrastructure failure, not a "credential not found" condition.

**Error taxonomy reference:** error-taxonomy.md v1.53 §E-AUTH-005 reserves the code for "No credentials in keyring or file backend (backend works, no entry)." Wrapping `BackendUnavailable` in an `E-AUTH-005` wrapper with text "credential not found" gives operators incorrect triage guidance — they will search for a missing credential entry when the actual problem is a broken file path or inaccessible keyring.

**Likely fix:** Allocate `E-AUTH-007` for `BackendUnavailable` in error-taxonomy.md; amend BC-2.01.017 v1.2→v1.3 to add the new error case (EC-017-007 or equivalent); update `acquire_token` to branch on `CredentialResolutionError::BackendUnavailable` → `E-AUTH-007` vs all other `Err` variants → `E-AUTH-005`.

**Novelty:** HIGH. Passes 1 and 2 focused on test-side split (empty vs not-found) and BC narrative correction. This pass is the first to trace the production error-code flow for the file-backend / BackendUnavailable branch.

---

### F-LP3-MED-001 — lib.rs Module Doc Advertises Deleted POST /login Route

**Severity:** MED  
**Routing:** implementer (doc sweep)

**Anchor:** `crates/prism-dtu-cyberint/src/lib.rs:5` module-level documentation block

**Description:**  
The `prism-dtu-cyberint` crate's module doc comment (line 5 or thereabouts) still contains the text `POST /login — cookie-based auth (CookieRoundtrip pattern)`. AC-001 of this story deleted the `POST /login` route and replaced the auth model with `StaticCookieAuthProvider` (direct `Cookie: access_token={token}` injection). The module doc was not swept when `build_router` was edited at `clone.rs:111`.

This is a public-facing crate documentation lie: any developer reading the crate docs will believe the DTU exposes a login endpoint that does not exist.

**Root cause pattern:** Partial-fix discipline gap — `lib.rs` was not included in the sweep after the `build_router` route deletion.

---

### F-LP3-MED-002 — Parity Test Comment Claims Stale cyberint_session Cookie Semantics

**Severity:** MED  
**Routing:** implementer (comment sweep)

**Anchor:** `crates/prism-spec-engine/tests/parity/cyberint.rs:144`

**Description:**  
The parity test at line 144 contains a code comment that reads approximately "DTU cookie check validates non-empty cyberint_session cookie." After the Pass 1 fix-burst rewrote the auth model to use `access_token` (commit `c25bc598` and subsequent implementer work), this comment is a stale narrative — the cookie is now named `access_token`, not `cyberint_session`, and the auth model no longer involves session acquisition.

**Process-gap note:** Pass 1 closed F-LP1-MED-003 as "N/A — file doesn't exist" because the implementer searched `crates/prism-dtu-cyberint/tests/parity/` (the adversary's cited path) and found no `parity/` directory there. The file actually exists at `crates/prism-spec-engine/tests/parity/cyberint.rs` — a different crate. This is the same wrong-crate-search pattern as F-LP1-LOW-002 (adversary cited `prism-dtu-cyberint/src/auth_provider.rs`; file actually in `prism-spec-engine/src/auth_provider.rs`). Codification candidate for lesson 57 [process-gap].

---

### F-LP3-LOW-001 — Validation Order: Empty/Whitespace Check Before Length Check

**Severity:** LOW  
**Routing:** implementer

**Anchor:** `auth_provider.rs::acquire_token` validation block — `is_empty() || chars().all(|c| c.is_whitespace())` guard precedes `len() > 4096` guard

**Description:**  
The current validation order runs the empty/all-whitespace check before the 4096-byte length check. A 5000-byte all-whitespace key (e.g., 5000 space characters) will match the `is_empty() OR all-whitespace` predicate and be reported with the E-AUTH-006 detail "empty or all-whitespace" rather than "exceeds 4096-byte limit."

Both paths produce E-AUTH-006, so the behavioral contract is correct. However, the `detail` text mis-describes the cause for keys that are simultaneously over-length AND all-whitespace, which complicates operator triage in the edge case.

**Suggested fix:** Reorder the guards: check `len() > 4096` first, then check `is_empty() || all-whitespace`. This ensures the most specific description of the failure is returned.

---

### F-LP3-LOW-002 — Unsafe SAFETY Comment Quality in Plugin Boot Tests (DEFERRED)

**Severity:** LOW  
**Routing:** DEFERRED — maintenance follow-up story (NOT this cascade)  
**Classification:** cross-story; out-of-scope for S-DTU-CYBERINT-AUTH-FIDELITY-001

**Anchor:** `crates/prism-bin/tests/plugin_boot_tests.rs` approximately lines 167, 192, 210, 489

**Description:**  
4 `unsafe { std::env::set_var }` sites in the plugin boot test file cite only generic thread-safety justification in their SAFETY comments (e.g., "tests run in a single thread" or similar). They do not anchor BC-2.03.009 or the resolver-backend BC. This is the sibling class of the F-LP1-LOW-002 closure in this cascade (which addressed `prism-spec-engine/src/auth_provider.rs` unsafe tests), but applies to the prism-bin plugin boot test infrastructure.

**Why deferred:** This file is prism-bin's plugin-boot test infrastructure, not part of the cyberint auth fidelity story. The finding is cross-story in scope. The adversary correctly classifies this as out-of-scope for S-DTU-CYBERINT-AUTH-FIDELITY-001 — it should route to a maintenance follow-up story (wave-gate), not this cascade.

**Deferral anchor:** To be addressed in a maintenance follow-up story. Implementer sibling-sweep advisory from Pass 2 already surfaced this set of sites (D-855 advisory note).

---

## Pass 2 Closure Re-Verification

| Finding | Verification Result |
|---------|---------------------|
| F-LP2-CRIT-001 test split | LOAD-BEARING — `test_static_cookie_auth_provider_missing_credential_returns_e_auth_005` (NotFoundCredentialResolver path) and `test_static_cookie_auth_provider_empty_value_returns_e_auth_006` (MockCredentialResolver::new("") path) both verified via reverse-tracing through `acquire_token` to their respective error codes |
| F-LP2-HIGH-001 MockCredentialResolver injection | LOAD-BEARING — `test_static_cookie_auth_provider_injects_resolved_token_from_credentials` correctly injects MockCredentialResolver and asserts on the AuthToken content returned; the test would fail if the credential plumbing were severed |
| PO revert 2707ee69 | VERIFIED — BC-2.01.017 v1.2 frontmatter confirms EC-017-003 = E-AUTH-005 (not-found path), EC-017-005 = E-AUTH-006 (empty/invalid value path); `resolve_secret.rs:78-81` EnvVar arm has no empty-string filter as correctly documented in lesson 56 |

---

## Standing Probe Results

### SAP-1 — Tracing Emission Catalog Completeness
- Grep: `rg 'event_type\s*=' crates/ --type rust` against feature HEAD `d697425b`
- Result: No new `event_type =` emissions were added in this story's commits. Existing emissions verified against BC-2.16.002. CLEAN.

### SAP-2 — DTU↔TOML Schema Parity
- Cyberint TOML columns verified against `crates/prism-dtu-cyberint/src/types.rs` and route handlers
- The open gap TD-FOLLOWUP-ARRAY-COLUMNTYPE-001 (`Alert::affected_assets`) is already tracked; no new parity gaps found
- CLEAN (existing gap tracked, no new gaps)

### SAP-3 — Injected Value Consumption (Per Lesson 50)
- `StaticCookieAuthProvider::acquire_token` verified: resolved credential IS consumed by injection into the Cookie header at the DTU boundary
- No bypassed-injection pattern found
- CLEAN

### SID-1 — No-Ignored-Test Rationalization Prohibition
- No new behaviors were deferred behind `#[ignore]` flags
- CLEAN

### POL-29 — Version Pin Propagation
- No BC version bumps in this pass's fix-burst scope (PO will bump BC-2.01.017 v1.2→v1.3 in the next dispatch)
- No stale cite-pins introduced
- CLEAN (pre-fix-burst)

### Cross-Document Consistency
- BC-2.01.017 v1.2 is current; the error taxonomy at v1.53 does not yet have E-AUTH-007 (that is the fix-burst target)
- No other cross-document drift found in the in-scope files

---

## Novelty Assessment

**Novelty: HIGH.**

Passes 1 and 2 focused on:
- Pass 1: harness clone auth-model rewrite (access_token vs cyberint_session), route deletions, parity test cleanup
- Pass 2: BC-2.01.017 EC-017-005 fabrication correction (empty value → E-AUTH-006), test split (not-found vs empty), env-var unsafe test refactor

Pass 3 introduces a genuinely new axis: **production error-code taxonomy at the file-backend branch.** The error mis-mapping (F-LP3-HIGH-001) was invisible to Passes 1 and 2 because those passes examined the BC narrative and test-level behavior, not the internal branching of `acquire_token` against the full `CredentialResolutionError` variant space. The `BackendUnavailable` variant only arises when the file-backend path is taken AND the file is unreadable — a branch that standard unit tests with `MockCredentialResolver` or `NotFoundCredentialResolver` do not exercise.

F-LP3-MED-002 also surfaces a genuine wrong-crate-search gap in Pass 1's N/A handling, confirming lesson 57 codification is warranted.

---

## Suggested `adversary-convergence-state.json` Entry

```json
{
  "pass": 3,
  "date": "2026-05-30",
  "clean_strict": false,
  "clean_pr_merge": false,
  "findings_count": 5,
  "findings_by_severity": {"CRIT": 0, "HIGH": 1, "MED": 2, "LOW": 2, "OBS": 0, "PROCESS-GAP": 0},
  "streak": 0,
  "novelty": "HIGH",
  "feature_head_at_pass": "d697425b",
  "key_finding": "F-LP3-HIGH-001 — acquire_token blanket-wraps every resolver Err as E-AUTH-005, mis-labeling BackendUnavailable (file-backend) as not-found",
  "blocked_on": "Fix-burst required before Pass 4 dispatch",
  "in_scope_findings": ["F-LP3-HIGH-001", "F-LP3-MED-001", "F-LP3-MED-002", "F-LP3-LOW-001"],
  "deferred_findings": [
    {
      "finding_id": "F-LP3-LOW-002",
      "category": "cross-story",
      "target": "wave-gate",
      "note": "plugin-boot test unsafe SAFETY-comment quality; sibling-class to closed F-LP1-LOW-002 but out-of-scope for cyberint auth fidelity story"
    }
  ]
}
```

---

## Next-Action Recommendation

1. **Dispatch PO** for E-AUTH-007 allocation in error-taxonomy.md v1.53→v1.54 + BC-2.01.017 v1.2→v1.3 amendment (new EC for BackendUnavailable → E-AUTH-007; retain EC-017-003 = E-AUTH-005 for NotFound, EC-017-005 = E-AUTH-006 for empty/invalid)
2. **Dispatch implementer** for fix-burst:
   - F-LP3-HIGH-001: branch `acquire_token` on `CredentialResolutionError::BackendUnavailable` → E-AUTH-007; all other Err → E-AUTH-005
   - F-LP3-MED-001: sweep `crates/prism-dtu-cyberint/src/lib.rs` module doc — remove `POST /login` reference
   - F-LP3-MED-002: sweep `crates/prism-spec-engine/tests/parity/cyberint.rs:144` — update stale `cyberint_session` comment to `access_token` and current auth model
   - F-LP3-LOW-001: reorder validation — `len() > 4096` check before `is_empty() || all-whitespace` check
3. **Dispatch state-manager** for fix-burst closure persistence
4. **Dispatch Pass 4 LOCAL adversary** against post-fix-burst feature HEAD
5. **F-LP3-LOW-002** — route to maintenance follow-up story at wave-gate; DO NOT hold this cascade for it

---

## CLEAN Status

```
CLEAN (strict): no
CLEAN (PR-merge): no
```

Streak: 0/3. Fix-burst required before Pass 4 dispatch.
