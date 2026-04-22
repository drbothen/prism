---
adr_id: ADR-003
title: "DTU Reset-Lookup Semantics and Fidelity Probe Auth"
document_type: architecture-section
level: ADR
section: decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth
version: "1.0"
status: Accepted
producer: architect
timestamp: 2026-04-22T00:00:00Z
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
input-hash: "572c2a9"
traces_to: ARCH-INDEX.md
resolves: [S-6.07-conflict-ac8-ec003, S-6.07-conflict-ac7-fidelity]
closes_debt: []
flags_debt: TD-WV1-01
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
