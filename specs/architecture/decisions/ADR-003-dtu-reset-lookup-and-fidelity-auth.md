---
adr_id: ADR-003
title: "DTU Reset-Lookup Semantics and Fidelity Probe Auth"
document_type: architecture-section
level: ADR
section: decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth
version: "1.4"
status: Accepted
producer: architect
timestamp: 2026-04-22T00:00:00Z
amended: 2026-04-24T00:00:00Z
phase: phase-3-dtu-wave-1
inputs:
  - .factory/stories/S-6.07-dtu-crowdstrike.md
  - .factory/specs/architecture/dtu-assessment.md
  - .factory/specs/architecture/decisions/ADR-001-dtu-rate-limit-pattern.md
  - .factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md
  - .factory/tech-debt-register.md
  - crates/prism-dtu-common/src/fidelity.rs (S-6.07 worktree)
  - crates/prism-dtu-crowdstrike/tests/ac_8_reset.rs (S-6.07 worktree)
  - crates/prism-dtu-crowdstrike/tests/edge_cases.rs (S-6.07 worktree)
  - crates/prism-dtu-crowdstrike/tests/fidelity.rs (S-6.07 worktree)
  - crates/prism-dtu-crowdstrike/src/routes/hosts.rs (S-6.07 worktree)
  - crates/prism-dtu-common/src/fidelity.rs (wave-1-gate develop)
  - crates/prism-dtu-{crowdstrike,claroty,cyberint,armis,threatintel,nvd}/tests/ (wave-1-gate develop)
input-hash: "0c00e49"
traces_to: ARCH-INDEX.md
resolves: [S-6.07-conflict-ac8-ec003, S-6.07-conflict-ac7-fidelity]
closes_debt: []
flags_debt: TD-WV1-01
amendments:
  - "#3: FidelityCheck headers field (TD-WV1-01) — 2026-04-24"
  - "#4: Fidelity test filename convention (TD-WV1-02) — 2026-04-24"
  - "#5: /dtu/configure admin token authentication (TD-WV0-07) — 2026-04-24 (wave-1-5/pr-f, 5a2d1c8c)"
runtime_deliverables:
  - prism-dtu-common::FidelityCheck  # headers field added (Amendment #3, TD-WV1-01)
  - prism-dtu-common::FidelityValidator  # header injection loop in run() (Amendment #3)
  - prism-dtu-common::BehavioralClone::admin_token  # new required trait method (Amendment #5, TD-WV0-07)
wiring_deferred_to: null  # All three deliverables confirmed implemented in prism-dtu-common (Wave 1.5 + TD-WV0-07 closure)
---

# ADR-003: DTU Reset-Lookup Semantics and Fidelity Probe Auth

## [Section Content]

## Status

Accepted

## Context

Two spec contradictions in S-6.07 v1.5 were identified as blocking the
implementer from reaching green. Both stem from interactions between the
session-scoped two-step fetch model and the shared `FidelityValidator` in
`prism-dtu-common`, which was designed before S-6.07's auth model was finalized.

### Conflict #1 — AC-8 vs EC-003: Post-Reset Step-2 Lookup Semantics

**AC-8** (as written in S-6.07 v1.5) states:

> Given `reset()` is called, Then the containment store, detection status
> store, and session registry are all cleared; a subsequent
> `GET /devices/entities/devices/v2?ids=h-001` returns the device with
> `containment_status: "normal"` (base fixture state).

**EC-003** states:

> Step 2 called with IDs not in session registry returns 200 with empty
> `resources` array (IDs not found is not an error in the real API).

After `reset()`, the session registry is empty. Therefore any Step-2 request
carrying an `X-DTU-Session-Id` header whose session was cleared by `reset()`
falls directly under EC-003 (empty resources). AC-8's claim that the same
request returns a populated fixture record contradicts EC-003 unless the
request is specifically structured to bypass the session filter.

Inspection of the existing test file `tests/ac_8_reset.rs` in the S-6.07
worktree reveals that the test-writer already resolved this contradiction
in practice by splitting AC-8 into three separate test functions:

1. `ac_8_reset_clears_containment_store` — performs a full step-1 (new session)
   → step-2 round-trip AFTER reset and asserts `containment_status: "normal"`.
   This is a fresh session, not the pre-reset session, so EC-003 does not apply.
2. `ac_8_reset_clears_session_registry` — asserts that step-2 with the
   PRE-RESET session returns empty resources after `reset()`. This directly
   confirms EC-003 applies to cleared sessions.
3. `ac_8_reset_clears_detection_status_store` — verifies server health after
   reset, not post-reset lookup behavior.

The test-writer's split is semantically correct. The story prose AC-8 is
ambiguous because it conflates two separate invariants without specifying
which session is used in the post-reset GET. The implementation in
`src/routes/hosts.rs` is also correct: it applies the session filter when
`X-DTU-Session-Id` is present and falls back to direct fixture lookup when
the header is absent.

**Root cause:** AC-8 prose omits the session header context of the
post-reset GET. The phrase "base fixture state" implies the intent is to
verify that the write store (containment) was cleared, NOT that the
session filter was bypassed.

### Conflict #2 — AC-7 vs FidelityValidator: Auth Enforcement vs Probe Shape

**AC-7** states:

> Given a request to any auth-required endpoint without an `Authorization`
> header, Then the response is HTTP 401 with
> `{"errors": [{"code": 401, "message": "..."}]}`.

**S-6.07 Task 11 + `tests/fidelity.rs`** require that `FidelityValidator::run`
is called against all 8 endpoints and `checks_failed == 0`.

`FidelityValidator` in `prism-dtu-common` (as implemented in the S-6.07
worktree) sends probes without `Authorization` headers. The `FidelityCheck`
struct has no `headers` field (tracked as TD-WV1-01). Endpoints 1–7 are
auth-required per AC-7. Therefore fidelity probes to endpoints 1–7 will
receive 401, causing all 7 checks to fail — contradicting the
`checks_failed == 0` assertion.

Three resolution options exist:
- **(A)** Add a `headers` field to `FidelityCheck` in `prism-dtu-common`
  so probes can carry a bearer token. Clean long-term fix; requires
  changing shared infrastructure. Tracked as TD-WV1-01.
- **(B)** DTU-side fidelity-probe bypass: the clone's auth middleware
  accepts a special bearer value `"fidelity-probe"` as valid on non-write,
  non-sensitive paths. Local to the clone, zero changes to `prism-dtu-common`.
- **(C)** Scope the fidelity test to unauthenticated endpoints only
  (`/oauth2/token`, `/dtu/health`). Auth-required endpoint shapes are
  covered by the per-AC integration tests which already carry valid
  bearer tokens. Fidelity coverage shrinks but the contradiction disappears
  without any bypass mechanism.

---

## Decision

### Conflict #1: AC-8 Semantics

**Decision:** AC-8 is split into two normative assertions (AC-8a and AC-8b)
that together cover what the original AC-8 intended. EC-003 is NOT amended —
it correctly describes the session-registry miss path that applies to cleared
sessions. The post-reset fixture-state assertion requires a fresh step-1 to
re-register IDs under a new session before performing step-2.

The three-test split already present in `tests/ac_8_reset.rs` is the
authoritative implementation. The story prose must be updated to match.

**Normative replacement text for AC-8 in S-6.07 v1.6:**

```
AC-8a: Given `reset()` is called, Then the containment store, detection
status store, and session registry are all cleared, AND a step-2 request
that carries the PRE-RESET `X-DTU-Session-Id` value returns HTTP 200 with
an empty `resources` array (EC-003 applies: cleared session is a registry
miss).

AC-8b: Given `reset()` is called, When a NEW step-1 (`GET
/devices/queries/devices/v1`) is issued with a fresh `X-DTU-Session-Id`
and the returned IDs are then fetched via step-2 (`GET
/devices/entities/devices/v2`), Then the step-2 response contains host
records with `containment_status: "normal"` (fixture baseline — the
containment store was cleared by `reset()`).
```

EC-003 amendment: none required. EC-003 text is correct and applies
uniformly to: (a) sessions that were never created, (b) sessions that
were evicted from the LRU cache, and (c) sessions that were cleared by
`reset()`. All three produce the same observable behavior: empty resources.

### Conflict #2: Fidelity Probe Auth

**Decision: Option C — scope fidelity checks to unauthenticated endpoints
and DTU introspection endpoints. Auth-required endpoint shapes are
covered by per-AC integration tests.**

Rationale for choosing Option C over Option B:

1. **No special tokens in test infrastructure.** A bypass bearer value
   `"fidelity-probe"` is a form of hardcoded credential, even in
   test-only code. It must be documented, enforced to never reach
   production, and maintained across clone upgrades. The maintenance
   surface is non-trivial for 14 DTU clones.

2. **AC-7 enforcement is binary and must be unconditional.** The purpose
   of AC-7 is to verify that the clone faithfully simulates the real
   CrowdStrike API's 401 behavior. Introducing a bypass token — even one
   named `"fidelity-probe"` — makes AC-7 tests vacuous for any probe
   that happens to carry that token.

3. **Full shape coverage already exists via per-AC tests.** Every
   auth-required endpoint has a corresponding AC test that sends
   `Authorization: Bearer dtu-fake-cs-token` and asserts both the status
   code and required response fields. The fidelity validator's value is
   in providing a second, independent shape check — not in duplicating
   the per-AC coverage. For auth-required endpoints, per-AC tests already
   provide that independent check.

4. **Option A (TD-WV1-01) remains the correct long-term fix.** When
   `FidelityCheck` gains a `headers` field, the fidelity test can be
   expanded to cover all 8 endpoints with real bearer tokens. This ADR
   does not close TD-WV1-01.

**Normative fidelity test guidance for S-6.07 Task 11:**

The fidelity test (`tests/fidelity.rs`) MUST cover the following endpoint
set via `FidelityValidator`:

| # | Endpoint | Method | Expected Status | Rationale |
|---|----------|--------|-----------------|-----------|
| 1 | `/oauth2/token` | POST | 200 | Unauthenticated by design |
| 2 | `/dtu/health` | GET | 200 | DTU introspection — no auth required |
| 3 | `/dtu/reset` | POST | 200 | DTU introspection — no auth required |

All auth-required endpoints (detection list, detection summaries, host
list, host details, contain, lift_containment, update_status) are
excluded from the `FidelityValidator` check in this story. Their response
shapes are validated by `tests/ac_1_happy_path.rs` through
`tests/ac_7_auth.rs`, which carry valid auth headers.

The `crowdstrike_dtu_fidelity` test function MUST be updated to reflect
this reduced scope. The `assert_eq!(report.checks_passed, 8, ...)` line
MUST be updated to `assert_eq!(report.checks_passed, 3, ...)`.

When TD-WV1-01 is resolved (adding `headers` to `FidelityCheck`), the
fidelity test SHOULD be expanded to include all 8 endpoints with bearer
token headers, and the count updated accordingly.

---

## Consequences

### For the Implementer (S-6.07)

- The `get_host_details` handler's three-path logic (session-filtered,
  session-miss, no-session header) is architecturally correct as
  implemented. No changes needed.
- `check_auth` in the host and detection route handlers MUST remain
  unconditional — no bypass for `"fidelity-probe"` bearer values.
- No changes to `prism-dtu-common` are required by this ADR.

### For the Test-Writer (S-6.07)

- `tests/fidelity.rs` MUST be updated: remove checks for auth-required
  endpoints 1–7; retain only the three unauthenticated checks listed above.
  Update `checks_passed` assertion from `8` to `3`.
- `tests/ac_8_reset.rs` is already correctly split into three functions.
  No changes needed if the implementation matches the test's expectations.
- `tests/edge_cases.rs` EC-003 tests are correct and unaffected.

### For the Story-Writer (S-6.07 → v1.6)

Replace AC-8 with AC-8a and AC-8b using the normative replacement text
in the Decision section above. Update Task 11 to reference the three-check
fidelity scope. Add a note that TD-WV1-01 tracks the path to restoring
full 8-endpoint fidelity coverage once `FidelityCheck` gains a `headers`
field.

No changes to EC-003, AC-1 through AC-7, or AC-9/AC-10.

### For Future DTU Clone Stories (S-6.08 and beyond)

All future L4-fidelity clones with auth-required endpoints MUST follow
the same fidelity scoping rule established here: `FidelityValidator`
checks are restricted to unauthenticated endpoints and `/dtu/*`
introspection endpoints until TD-WV1-01 is resolved. Per-AC integration
tests carry the auth-required shape coverage.

ADR-002 §8 (Fidelity Validator Test) is implicitly amended by this ADR:
the "one check per AC endpoint shape requirement" guidance applies only
to unauthenticated endpoints when `FidelityCheck` lacks a `headers` field.

---

## Alternatives Considered

### Conflict #1 — Alternative: Keep AC-8 as one assertion, require no-session-header path

The implementation already handles the case where `X-DTU-Session-Id` is
absent — it falls through to direct fixture lookup. We could rewrite AC-8
to use a headerless GET and assert `containment_status: "normal"`. Rejected
because: (1) headerless GETs are the fidelity-probe path, not the
integration-test path; (2) a realistic test must exercise the session
pipeline; (3) splitting into AC-8a/AC-8b exposes both invariants explicitly
and matches the three-function test-writer decomposition already present
in the worktree.

### Conflict #2 — Alternative B: DTU-side fidelity-probe bypass bearer

A special bearer value `"fidelity-probe"` is accepted on non-write paths.
Rejected — see Decision rationale items 1 and 2. Additionally, this sets
a precedent that would need to be replicated in all 13 remaining DTU
clones with auth-required endpoints. Option C is contained to the test
file and costs nothing in the clone implementation.

### Conflict #2 — Alternative A: Add `headers` to `FidelityCheck`

The correct long-term fix. Deferred — TD-WV1-01 tracks this. Implementing
it now would require a `prism-dtu-common` story bump, review cycle, and
cascading test updates across all merged DTU clones. The wave-1 schedule
does not accommodate this scope expansion.

---

## Related

- S-6.07 v1.5 (story being resolved)
- ADR-001 — per-clone rate-limit semantics (independent)
- ADR-002 §8 — fidelity validator test shape (implicitly amended by this ADR)
- TD-WV1-01 — `FidelityCheck.headers` — path to restoring full fidelity coverage
- EC-003 (S-6.07 Edge Cases) — session registry miss semantics (unchanged)
- VP-033, VP-036 — integration tests unaffected; use full auth, not fidelity probes

---

## Amendment #3: FidelityCheck Headers Field (TD-WV1-01)

**Added:** 2026-04-24 (Wave 1.5 remediation burst)

### Context

`FidelityCheck` in `prism-dtu-common/src/fidelity.rs` has no `headers` field.
All six Wave 1 DTU clones work around this gap in one of two ways:

- L4 clones (CrowdStrike, Claroty): scope fidelity probes to unauthenticated
  endpoints only (Option C per original ADR-003 §Conflict-2 Decision). Auth-required
  endpoint shapes are covered exclusively by per-AC integration tests.
- L2 clones (Cyberint, Armis): include unauthenticated error-path checks (401/403
  shapes) in the fidelity validator, and note explicitly in comments that the
  FidelityCheck struct cannot carry headers.

Both workarounds leave a gap: the fidelity validator cannot independently confirm that
authenticated endpoint shapes are correct — it relies entirely on per-AC tests for that
coverage. This is acceptable for L2 clones where authenticated shape is simple, but for
L4 clones (CrowdStrike's 8-endpoint surface, Claroty's full surface) it means the
second-independent-check value of FidelityValidator is not realized for auth-required
routes.

Wave 1.5 is a debt-reduction sprint with no new clone stories competing for bandwidth.
This is the correct time to close TD-WV1-01.

### Decision

Add a `headers: Vec<(String, String)>` field to `FidelityCheck`.

**Chosen type: `Vec<(String, String)>` rather than `HashMap<String, String>`.**

Rationale for ordered Vec over HashMap:
1. HTTP allows duplicate header names (e.g. `Set-Cookie`, `Accept`). A HashMap silently
   drops duplicates. Vec preserves all header instances.
2. Order matters for some security-sensitive headers (e.g. multiple `Authorization`
   headers in malformed requests — relevant for L4 adversarial clone testing).
3. Most callers will construct headers inline with `vec![("Authorization".into(),
   format!("Bearer {token}"))]` — Vec syntax is equally ergonomic.
4. The reqwest builder's `.header(name, value)` call accepts a single pair at a time;
   iterating a Vec is the natural integration pattern.

**Field placement and default:**

```rust
#[derive(Debug, Clone)]
pub struct FidelityCheck {
    pub endpoint: String,
    pub method: http::Method,
    pub body: Option<serde_json::Value>,
    pub expected_status: u16,
    pub required_fields: Vec<String>,
    /// HTTP headers to include in the probe request.
    /// Defaults to empty (no custom headers).
    pub headers: Vec<(String, String)>,
}
```

**Backward compatibility:** All existing `FidelityCheck` struct literals in the six
DTU clone test files use named-field syntax (`FidelityCheck { endpoint: ..., method:
..., ... }`). Adding a new field with a `Default` impl breaks these — Rust requires
all fields to be specified in struct literal syntax unless `..Default::default()` is
used. Two migration paths exist:

- **(Preferred)** Implement `Default` for `FidelityCheck` and update every existing
  constructor to append `..Default::default()`. This is mechanical: 6 clone test
  files, ~30 `FidelityCheck` literal sites.
- **(Alternative)** Add a `FidelityCheck::new(...)` constructor that sets
  `headers: vec![]` and migrate callers to use it. Rejected: existing callers use
  struct literal syntax; a constructor would require a larger mechanical refactor.

The preferred path is: implement `Default` for `FidelityCheck` (all string/vec fields
default to empty, method defaults to `http::Method::GET`, expected_status to `200`),
and update every struct literal to add `..Default::default()`.

**Injection in `FidelityValidator::run`:**

In the request builder loop, after body injection and before `req.send()`:

```rust
for (name, value) in &check.headers {
    req = req.header(name.as_str(), value.as_str());
}
```

This integrates cleanly with the existing `reqwest::RequestBuilder` chain.

### Consequences

**Code changes required (Wave 1.5 story):**

1. `crates/prism-dtu-common/src/fidelity.rs` — add `headers` field, implement
   `Default` for `FidelityCheck`, add header injection loop in `FidelityValidator::run`.

2. All six clone fidelity test files — append `..Default::default()` to every existing
   `FidelityCheck` literal to satisfy the new required field. Files affected:
   - `crates/prism-dtu-crowdstrike/tests/fidelity.rs`
   - `crates/prism-dtu-claroty/tests/fidelity.rs`
   - `crates/prism-dtu-cyberint/tests/ac_8_fidelity_validator.rs`
   - `crates/prism-dtu-armis/tests/ac_7_fidelity_validator.rs`
   - Any future clone test files before they ship

3. CrowdStrike fidelity test expansion (optional in Wave 1.5, recommended): once the
   field is available, expand `tests/fidelity.rs` from 3 checks to 11 (all 8
   auth-required endpoints + 3 unauthenticated). Each auth check adds:
   `headers: vec![("Authorization".into(), "Bearer dtu-fake-cs-token".into())]`.
   The comment "When TD-WV1-01 is resolved... expand to all 8 endpoints" is then removed.

4. Claroty fidelity test expansion (optional in Wave 1.5): expand from 10 checks to
   include the 7 currently-401 checks with a valid bearer token to verify the
   authenticated 200/201 shapes.

5. TD-WV1-01 is closed when changes (1) and (2) merge. Expansions (3) and (4) may
   be deferred to Wave 2 if Wave 1.5 capacity is limited — they are improvements, not
   correctness fixes.

**Security note:** The `headers` field is test-only infrastructure (all DTU code is
gated behind `#[cfg(any(test, feature = "dtu"))]`). The bearer tokens used in fidelity
probes are the same fake tokens already used in per-AC integration tests. No new
credential surface is introduced.

**Scope boundary:** This amendment covers `FidelityCheck` and `FidelityValidator` only.
It does not change the `/dtu/configure` auth-bypass model (TD-WV0-07), clone auth
middleware, or any production code path.

### Alternatives Considered

- **Option B (DTU-side bypass bearer):** A hardcoded `"fidelity-probe"` bearer accepted
  by clone auth middleware. Rejected in the original ADR-003 Decision for Conflict #2
  and still rejected here: it makes AC-7 vacuous for any probe carrying that token, and
  it must be propagated to all 14 DTU clones as they ship in Waves 2–3.
- **`HashMap<String, String>` type:** Simpler stdlib type, familiar to most Rust
  programmers. Rejected: silently drops duplicate header names; Vec is more faithful
  to HTTP semantics and costs nothing extra.
- **`http::HeaderMap` type:** The canonical HTTP header type from the `http` crate,
  which `prism-dtu-common` already depends on. Rejected: ergonomically awkward for
  struct literal construction in test code; callers would need
  `http::HeaderName::from_static` or `.parse().unwrap()` for every key, which is
  verbose in the inline literal context where `FidelityCheck` is constructed.

---

## Amendment #4: Fidelity Test Filename Convention (TD-WV1-02)

**Added:** 2026-04-24 (Wave 1.5 remediation burst)

### Context

ADR-002 §8 mandates that every L2 clone include a fidelity test file named
`tests/ac_N_fidelity_validator.rs` where N is the last AC number of the story. The
intent was to make the fidelity test discoverable via filename and to bind it to the
final AC. This rule has produced three distinct naming patterns across the six Wave 1
clones:

| Clone | Fidelity file | ADR-002 §8 compliance | Notes |
|-------|-------------|----------------------|-------|
| prism-dtu-crowdstrike (L4) | `tests/fidelity.rs` | Non-compliant | L4 deviation policy applies; ADR-002 allows multi-file split for L4 |
| prism-dtu-claroty (L4) | `tests/fidelity.rs` | Non-compliant | Same L4 deviation |
| prism-dtu-cyberint (L2) | `tests/ac_8_fidelity_validator.rs` | Compliant | Fidelity is the last AC (AC-8) |
| prism-dtu-armis (L2) | `tests/ac_7_fidelity_validator.rs` | Non-compliant per intent | AC-7 slot is fidelity, but AC-7 was originally reset semantics; reset content is in `tests/reset_state_invariants.rs` |
| prism-dtu-threatintel (L2) | None | Non-compliant | Wave 0 crate; ADR-002 postdates it |
| prism-dtu-nvd (L2) | None | Non-compliant | Wave 0 crate; ADR-002 postdates it |

The Armis case (S-6.10) is the specific trigger for TD-WV1-02: the story's AC
numbering ended at AC-7 for reset semantics, and the fidelity AC was added as the
logically-last AC but received the same number (AC-7), displacing the reset content
into `reset_state_invariants.rs`. The result is that `ac_7_fidelity_validator.rs`
exists (correct name) but the file it displaced (`reset_state_invariants.rs`) has a
non-AC-pattern name — creating a two-file anomaly where ADR-002 expects one.

Two options were proposed in TD-WV1-02:

- **(A)** Amend ADR-002 to base fidelity test filename on AC semantic role, not AC
  number: use a fixed name `tests/fidelity_validator.rs` for all clones.
- **(B)** Reserve the last AC slot for fidelity in all DTU stories by convention,
  ensuring the `ac_N_fidelity_validator.rs` pattern always works.

### Decision

**Option A — semantic-role filename: `tests/fidelity_validator.rs` for all clones.**

Rationale:

1. **AC numbering is not stable across story revisions.** When a story is amended
   (as S-6.07 was with AC-8a/AC-8b), AC numbers shift. A filename tied to AC number
   silently becomes stale. A fixed semantic name does not rot.

2. **Option B creates a story-authoring constraint that is hard to enforce.** Requiring
   the last AC slot to always be fidelity forces story authors and the story-writer
   agent to number ACs in a specific order. When a story needs a new AC added after
   fidelity is written (a common review-cycle pattern), the author must either renumber
   or violate the rule. The constraint is low-value and high-friction.

3. **L4 clones already converged on `fidelity.rs`** — CrowdStrike and Claroty both
   use this name naturally, satisfying the L4 multi-file deviation policy. The
   canonical name `fidelity_validator.rs` (L2 standard) and `fidelity.rs` (L4
   permitted variant) can be unified under a single rule with an explicit L4 note.

4. **Discoverable and self-documenting.** The filename `fidelity_validator.rs` is
   semantically unambiguous regardless of which AC it corresponds to. Any developer
   scanning a DTU crate's `tests/` directory immediately identifies the fidelity file.

**Canonical naming rule (replaces ADR-002 §8 filename guidance):**

| Clone fidelity level | Required test filename | Notes |
|----------------------|----------------------|-------|
| L2 (stateful) | `tests/fidelity_validator.rs` | Single file, calls `FidelityValidator::run` |
| L3 (behavioral) | `tests/fidelity_validator.rs` | Single file; may include behavioral dimension checks |
| L4 (adversarial) | `tests/fidelity_validator.rs` | Single file preferred; multi-file split permitted per ADR-002 Deviation Policy if behavioral dimensions warrant it |

The `ac_N_fidelity_validator.rs` pattern from ADR-002 §8 is **retired**. The ADR-002
§8 compliance checklist item:

```
[ ] tests/ac_N_fidelity_validator.rs: FidelityValidator used, asserts `checks_failed == 0`
```

is replaced with:

```
[ ] tests/fidelity_validator.rs: FidelityValidator used, asserts `checks_failed == 0`
```

### Consequences

**ADR-002 §8 amendment:** The directory layout example in ADR-002 §1 (which shows
`ac_N_fidelity_validator.rs`) and the compliance checklist are superseded by this
amendment. Future DTU clone stories must reference this amendment rather than ADR-002
§8 alone for the fidelity filename rule.

**Retroactive renames required (Wave 1.5):**

| Current filename | Rename to | Crate | Action |
|-----------------|-----------|-------|--------|
| `tests/ac_8_fidelity_validator.rs` | `tests/fidelity_validator.rs` | prism-dtu-cyberint | Rename |
| `tests/ac_7_fidelity_validator.rs` | `tests/fidelity_validator.rs` | prism-dtu-armis | Rename |
| `tests/fidelity.rs` | `tests/fidelity_validator.rs` | prism-dtu-crowdstrike | Rename |
| `tests/fidelity.rs` | `tests/fidelity_validator.rs` | prism-dtu-claroty | Rename |

The Armis `tests/reset_state_invariants.rs` is NOT renamed — it is correctly named
for its content (reset state invariant tests). It becomes a peer of the new
`fidelity_validator.rs` rather than a displaced artifact.

**Wave 0 crates (ThreatIntel, NVD):** These crates predate ADR-002 and have no
fidelity validator tests. Adding `tests/fidelity_validator.rs` to each is Wave 1.5
work under this amendment. Minimum viable fidelity checks for each:
- `prism-dtu-threatintel`: `/dtu/health` (200), `/dtu/reset` (200), and one
  unauthenticated lookup shape check.
- `prism-dtu-nvd`: `/dtu/health` (200), and one CVE lookup shape check (NVD's
  `/api/2.0/cves/1.0` unauthenticated path).

**`[[test]]` entries in Cargo.toml:** Each renamed file requires a corresponding
rename in the `[[test]]` stanza (name, path, required-features). The mechanical
rename is: change `name = "ac_8_fidelity_validator"` to `name = "fidelity_validator"`
(or equivalent for the other files).

**No impact on per-AC test files:** The rename affects only the fidelity validator
file. All `ac_N_*.rs` tests for business logic AC coverage are unchanged.

**Story-writer agent impact:** The story-writer's ADR-002 §8 guidance cite must be
updated to reference this amendment. The checklist item template must use
`fidelity_validator.rs` not `ac_N_fidelity_validator.rs`.

**New TD items created by this amendment:** None. The retroactive renames are bounded
and mechanical. If they cannot fit in Wave 1.5 capacity, a TD item should be filed
for the Wave 0 crate fidelity additions (ThreatIntel, NVD) as those are new work,
not renames.

### Alternatives Considered

- **Option B (reserve last AC slot):** Forces story authors to number ACs with fidelity
  last. Rejected — see Decision rationale item 2. Additionally, Option B would require
  retroactive renumbering of `reset_state_invariants.rs` in prism-dtu-armis and
  potentially AC renumbering in S-6.10, which is higher-cost than the mechanical file
  rename this option requires.
- **Status quo (accept divergence):** Accept the three naming patterns and document them
  as clan-of-variants. Rejected — the purpose of ADR-002 was to eliminate structural
  drift; accepting a fourth naming pattern for fidelity files contradicts that intent
  and makes the fidelity file location non-deterministic for tooling.
- **`fidelity.rs` (L4 pattern as universal):** Use `fidelity.rs` for all fidelity
  levels rather than `fidelity_validator.rs`. Slightly shorter. Rejected: `fidelity.rs`
  is ambiguous — it could contain fidelity configuration, fidelity scoring logic, or
  fidelity tests. `fidelity_validator.rs` is unambiguous about both content and purpose.

---

## Scope Boundary (Amendments #3 and #4)

These amendments cover:
- `FidelityCheck` struct field addition and `FidelityValidator` header injection
- DTU fidelity test filename convention and retroactive renames
- ADR-002 §8 compliance checklist update

These amendments do NOT cover:
- `/dtu/configure` unauthenticated access (TD-WV0-07 — see Amendment #5 below)
- Production auth middleware or credential handling
- Wave 2+ clone stories (they adopt the new rules from their inception)
- The reset-lookup semantics in Conflict #1 (unchanged from original ADR-003)

---

## Amendment #5: `/dtu/configure` Admin Token Authentication (TD-WV0-07)

**Date:** 2026-04-24
**Status:** Accepted
**Resolves:** TD-WV0-07

### Decision

`POST /dtu/configure` on every DTU clone MUST require a valid `X-Admin-Token`
header. The token value is a per-instance UUID v4 generated at clone construction
time and accessible via the new `BehavioralClone::admin_token()` trait method.
Requests missing the header, or presenting an incorrect token, receive HTTP 401
with `{"error": "missing or invalid X-Admin-Token"}`.

### Rationale

**Loopback-only is not a security guarantee.** During integration test runs,
multiple loopback processes may coexist (other test binaries, cargo test threads,
stray server tasks from prior test runs that haven't been reaped). Any of these
can POST to `/dtu/configure` on any port they discover or enumerate. Without
authentication, a concurrent test run on the same machine could accidentally
reconfigure a DTU clone mid-test, producing non-deterministic failures that are
extremely difficult to diagnose.

A shared-secret token ensures only the test harness that started the clone (which
holds the token via `clone.admin_token()`) can reconfigure it mid-run. This is
analogous to how real API management systems protect admin endpoints regardless of
network-layer restrictions.

### Implementation

1. **`BehavioralClone` trait** (`crates/prism-dtu-common/src/clone.rs`): new
   required method `fn admin_token(&self) -> &str`.

2. **Each clone struct**: `admin_token: String` field initialized via
   `uuid::Uuid::new_v4().to_string()` in `new()`. Token stored in both the clone
   struct (for `admin_token()` impl) and the clone's state struct (for handler
   access via `Arc<State>`).

3. **Each `/dtu/configure` handler**: checks
   `headers.get("x-admin-token").and_then(|v| v.to_str().ok())` against
   `state.admin_token`. Returns 401 if missing or wrong; proceeds to payload
   validation if correct.

4. **All 12 existing `td_wv0_04` configure tests** (2 per clone) and all other
   integration tests calling `/dtu/configure`: updated to include
   `.header("X-Admin-Token", clone.admin_token())`.

5. **All fidelity_validator.rs tests** that probe `/dtu/configure`: updated to
   pass the admin token via `FidelityCheck::headers` (Amendment #3 field).

6. **New `td_wv0_07_configure_requires_admin_token.rs`** per clone (18 tests
   total, 3 per clone): no-token → 401, wrong-token → 401, correct-token → 200.

7. **`prism-dtu-demo-server::ClonePair`**: exposes `admin_token()` method
   delegating to `BehavioralClone::admin_token()` for demo-server tests.

### Backward Compatibility

All existing configure callers in the test suite were updated in the same PR.
There are no external callers outside the test suite; `/dtu/configure` is a
test-harness-only endpoint not exposed in production builds.

### Rejected Alternatives

- **No authentication on loopback**: Rejected — loopback does not prevent other
  test processes from discovering and calling the endpoint.
- **IP allowlist (only 127.0.0.1)**: Rejected — axum/hyper do not expose the
  remote address to route handlers without additional middleware; the effort is
  comparable to token auth but with weaker guarantees.
- **Static compile-time token**: Rejected — would be the same across all clones
  in a test run; a rogue configure to clone A would work on clone B. Per-instance
  tokens prevent cross-clone pollution.

### Scope

This amendment covers:
- All 6 Wave 1 DTU clones (crowdstrike, claroty, cyberint, armis, nvd, threatintel)
- `BehavioralClone` trait extension
- All integration tests calling `/dtu/configure`

This amendment does NOT cover:
- `/dtu/reset` or `/dtu/health` (remain unauthenticated — harness calls reset
  after each test; health is a liveness probe)
- Wave 2+ clones (they adopt this pattern from inception via the updated trait)
- Production auth middleware or credential handling
