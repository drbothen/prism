---
story_id: W3-FIX-CODE-002
title: "prism-customer-config + prism-dtu-harness: validation and dispatch hygiene bundle"
wave: 3.1
level: "L4"
target_module: prism-customer-config
subsystems: [SS-01, SS-06]
priority: P0
depends_on: [W3-FIX-SEC-003]
blocks: []
estimated_days: 2
points: 5
status: draft
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-01T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review.md
  - .factory/specs/behavioral-contracts/BC-3.3.001-startup-rejects-st-shared-mode.md
  - .factory/specs/behavioral-contracts/BC-3.3.004-customer-config-startup-validation.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.1.002-audit-entry-org-fields.md
  - .factory/specs/architecture/decisions/ADR-010-customer-config-schema.md
  - .factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.3.001
  - BC-3.3.004
  - BC-3.5.001
  - BC-3.5.002
  - BC-3.1.002
verification_properties: [VP-105, VP-106, VP-124, VP-125]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.3.001, BC-3.3.004, BC-3.5.001, BC-3.5.002, BC-3.1.002]
anchor_capabilities: [CAP-009, CAP-036, CAP-007]
anchor_subsystem: ["SS-01", "SS-06"]
tdd_mode: strict
---

# W3-FIX-CODE-002: prism-customer-config + prism-dtu-harness — validation and dispatch hygiene bundle

## Narrative

As a Prism maintainer, I want the six MEDIUM findings from the Wave 3 integration gate
(CR-003 through CR-006, SEC-006, SEC-007) bundled into a single hygiene story, so that
the remaining gaps in config validation, dispatch exhaustiveness, test-hook performance,
error message redaction, and audit cross-checking are resolved before the Phase 4
holdout evaluation.

## Objective

This story resolves six MEDIUM findings identified in Gate Steps C and D:

| Finding | Crate | Description |
|---------|-------|-------------|
| CR-003 | prism-customer-config | `OrgSlug` regex pattern not validated in structural pass (potential panic path) |
| CR-004 | prism-dtu-harness | `start_clone` uses sequential `if` chains instead of exhaustive `match` |
| CR-005 | prism-customer-config | `validate_all` is `pub` — exposes usability trap with partial configs on duplicate-id error |
| CR-006 | prism-dtu-harness | `poll_test_hook` spins at 10ms loop — replace with `Notify` or 50ms backoff |
| SEC-006 | prism-customer-config | `sanitize_error_message` may not cover multi-line TOML credential values |
| SEC-007 | prism-audit | `org_slug` in audit records not cross-checked against `OrgRegistry::slug_for(org_id)` |

Additionally, finding F-48-M-002 from Pass 47 dispatch asymmetry is captured here (same
files as CR-004).

These are all MEDIUM severity and each is a self-contained change. They are bundled
because they share two crates (`prism-customer-config`, `prism-dtu-harness`) and one
audit cross-check, and no single change is large enough to warrant a standalone story.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.3.004 | Customer Config Validation Rejects Invalid Schema at Startup | R-CUST-002 (slug validation gap — CR-003); Invariant 1 (validate-before-register ordering — CR-005) |
| BC-3.3.001 | Startup Rejects Security Telemetry DTU Type Declared with Shared Mode | General config validation posture; sanitize_error_message redaction (SEC-006) |
| BC-3.5.001 | Harness Logical Isolation Invariants | Dispatch exhaustiveness for all DtuTypes (CR-004); poll_test_hook efficiency (CR-006) |
| BC-3.5.002 | Harness Network Isolation Invariants | Same dispatch correctness requirement |
| BC-3.1.002 | Audit Entry Carries Both org_id and org_slug at Construction Time | Cross-check at write time (SEC-007) |

## Acceptance Criteria

### AC-001: OrgSlug regex validated in structural pass, E-CFG-019 added (traces to BC-3.3.004 R-CUST-002)
After the slug=filename-stem check (R-CUST-002), `validate_structural` also calls
`OrgSlug::new(&config.org_slug)` (or checks against `ORG_SLUG_PATTERN` directly). If the
slug fails the pattern check, `E-CFG-019: InvalidOrgSlugPattern` is pushed to the error
collector with a message identifying the invalid characters. Verified by a test with
`org_slug = "my org"` (contains space) failing validation.

### AC-002: `validate_all` made crate-private (traces to BC-3.3.004 invariant 1)
`pub fn validate_all(...)` is changed to `pub(crate) fn validate_all(...)`. The public
API surface of `prism-customer-config` becomes `load_and_validate` only. No external
crate in the workspace calls `validate_all` directly; confirmed by `cargo build
--workspace` succeeding after the visibility change.

### AC-003: `start_clone` dispatch consolidated into exhaustive `match` (traces to BC-3.5.001 postcondition 1)
The sequential `if dtu_type == Armis` / `if dtu_type == Claroty` chains in
`clone_server.rs:535-598` are replaced with a single `match dtu_type { ... }` that
exhaustively lists all `DtuType` variants. Adding a new `DtuType` without updating this
`match` produces a compile error. The `_` arm (if used for the generic stub) is
documented explicitly.

### AC-004: `poll_test_hook` uses Notify (or 50ms backoff) (traces to BC-3.5.001 — efficient startup budget)
The 10ms spin loop in `clone_server.rs:783-816` is replaced with either:
(a) `tokio::sync::Notify` — the hook handler notifies; `poll_test_hook` awaits the
notification (preferred), or
(b) `tokio::time::sleep(Duration::from_millis(50))` with a comment explaining the
trade-off (acceptable minimum).
Either approach eliminates the 1,200 wake-ups/second overhead for a 12-clone harness.

### AC-005: `sanitize_error_message` covers multi-line TOML credential patterns (traces to BC-3.3.001 invariant — no credential in errors)
New test cases for `sanitize_error_message` demonstrate that multi-line TOML string
values with credential-pattern field names (e.g., `password = """\nmy-secret-value\n"""`)
are redacted or that a comment explains the known limitation of the current approach.
If the conservative redaction strategy is chosen (redact any line containing a
credential-pattern field name), it must not over-redact non-credential TOML snippets.

### AC-006: Audit entry `org_slug` cross-checked against OrgRegistry at write time (traces to BC-3.1.002 postcondition)
At audit record construction time (`audit_emitter.rs:266-267`), add a debug assertion
(or a log warning in production) that `OrgRegistry::slug_for(req.org_id) ==
Some(&req.org_slug)`. If the check fails in debug mode, panic with a clear message
naming both the expected and actual slug. In production mode, emit a `tracing::warn!`
and continue (audit-must-not-fail semantics per BC-3.1.002).

### AC-007: Network-mode harness dispatch is also correct (traces to BC-3.5.002 postcondition 1)
The exhaustive `match dtu_type` in `start_clone` applies equally when the harness runs in
`IsolationMode::Network` — each DtuType variant starts the correct clone server with the
correct per-org TCP listener. The compile-time exhaustiveness guarantee covers both
isolation modes since `DtuType` dispatch is the same code path.

### AC-008: Each defect has a regression test (traces to all above BCs)
Each of the six sub-fixes has at least one test demonstrating before-fix-fail →
after-fix-pass behavior, added to the appropriate test file or as a new inline test.

## Tasks

### Sub-fix 1 — CR-003: OrgSlug pattern in validator
1. Read `crates/prism-customer-config/src/validator.rs` lines 429-435 (`validate_structural` R-CUST-002).
2. After the slug=stem check, add a pattern validation call. If `OrgSlug::new` is fallible:
   `if OrgSlug::new(&config.org_slug).is_err() { errors.push(ConfigError::InvalidOrgSlugPattern { ... }) }`.
   Define `E-CFG-019` in the `ConfigError` enum.
3. Add a test: `org_slug = "my org"` (space) → `E-CFG-019`.

### Sub-fix 2 — CR-005: validate_all visibility
4. Change `pub fn validate_all` → `pub(crate) fn validate_all` in `validator.rs`.
5. Run `cargo build --workspace` to confirm no external callers break.

### Sub-fix 3 — CR-004 + F-48-M-002: exhaustive match in start_clone
6. Read `crates/prism-dtu-harness/src/clone_server.rs` lines 535-598 (`start_clone`) and
   lines 468-534 (`build_router_for_type`) to understand current dispatch.
7. Replace the sequential `if` chains in `start_clone` with an exhaustive `match dtu_type`.
   Each variant calls the appropriate specialized startup function.
   Add compile-time documentation on the `_ =>` fallback arm if used.
8. Verify that the dispatch in `build_router_for_type` and in `builder.rs` is consistent
   with the new `start_clone` match.

### Sub-fix 4 — CR-006: poll_test_hook backoff
9. Read `crates/prism-dtu-harness/src/clone_server.rs` lines 783-816 (`poll_test_hook`).
10. If `tokio::sync::Notify` is feasible: refactor the signal type to `Arc<Notify>`, notify
    in the hook handler, and await in `poll_test_hook`.
    If Notify refactor is too invasive: change `sleep(10ms)` to `sleep(50ms)` and add a
    comment: "50ms polling; replace with tokio::sync::Notify in a future pass (CR-006)."
11. Add a test or comment in the existing test that verifies the hook still fires correctly.

### Sub-fix 5 — SEC-006: sanitize_error_message multi-line coverage
12. Read `crates/prism-customer-config/src/validator.rs` lines 327-349 (`sanitize_error_message`).
13. Add test cases for multi-line TOML credential values. If the current logic handles them
    correctly, add the tests as coverage. If not, apply the conservative strategy (redact
    any line in the snippet that contains a field name matching a credential pattern,
    regardless of value format).

### Sub-fix 6 — SEC-007: org_slug cross-check in audit emitter
14. Read `crates/prism-audit/src/audit_emitter.rs` lines 260-275.
15. Inject `OrgRegistry` reference into the emitter construction path (it likely already
    has access — confirm by checking `AuditEmitter::new` signature).
16. Add `debug_assert!` / `tracing::warn!` cross-check at slug write time.
17. Add a test demonstrating that a mismatched slug emits a warning (or panics in test mode).

18. Run full workspace test suite: `cargo test --workspace --all-features`. All pass.
19. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `validate_structural` slug check | prism-customer-config | `crates/prism-customer-config/src/validator.rs` | Pure |
| `validate_all` visibility | prism-customer-config | `crates/prism-customer-config/src/validator.rs` | Pure |
| `start_clone` dispatch | prism-dtu-harness | `crates/prism-dtu-harness/src/clone_server.rs` | Effectful (spawns HTTP server) |
| `poll_test_hook` backoff | prism-dtu-harness | `crates/prism-dtu-harness/src/clone_server.rs` | Effectful (async sleep / await Notify) |
| `sanitize_error_message` | prism-customer-config | `crates/prism-customer-config/src/validator.rs` | Pure |
| Audit slug cross-check | prism-audit | `crates/prism-audit/src/audit_emitter.rs` | Pure (check) + Effectful (warn log) |

**Subsystem anchor justifications:**
- SS-06 owns the `prism-customer-config` sub-fixes (CR-003, CR-005, SEC-006) because
  SS-06 is Client Configuration per ARCH-INDEX.
- SS-01 owns the `prism-dtu-harness` sub-fixes (CR-004, CR-006) because the harness
  is test infrastructure for the Sensor Adapters subsystem per ARCH-INDEX.
- BC-3.1.002 (SEC-007 audit cross-check) is owned by SS-05 (Audit Trail); this story
  touches `prism-audit` which is classified under SS-05. The story `subsystems: [SS-01,
  SS-06]` frontmatter is conservative — the actual scope includes SS-05 for the audit
  sub-fix.

**Dependency anchor justification:** `depends_on: [W3-FIX-SEC-003]` — SEC-003 adds
`E-CFG-018` to the `ConfigError` enum; this story adds `E-CFG-019` to the same enum.
Landing in order avoids conflicting enum variant additions. `blocks: []` — no other
W3-FIX story requires these hygiene fixes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `org_slug = "a"` (1 char, valid) | Passes pattern check; passes stem check (if stem is also "a") |
| EC-002 | `org_slug` is exactly 64 chars with valid `[a-zA-Z0-9_-]` chars | Passes pattern check; valid |
| EC-003 | `org_slug` contains Unicode (e.g., `"acmé"`) | `E-CFG-019`; pattern check fails because `é` is not in `[a-zA-Z0-9_-]` |
| EC-004 | New `DtuType` variant is added to the enum after this fix | `match dtu_type` in `start_clone` becomes a compile error — desired behavior (AC-003) |
| EC-005 | `poll_test_hook` with Notify refactor: hook never fires during test | `poll_test_hook` correctly returns/exits when the clone's shutdown signal fires — not a hang |
| EC-006 | Multi-line TOML credential: `password = """\nmy-secret\n"""` in parse error snippet | Redaction strategy (conservative or pattern-matching) must not reveal `my-secret` in error output |
| EC-007 | `OrgRegistry::slug_for(org_id)` returns `None` (org not registered) | `tracing::warn!` in production; in debug, determine if a panic is appropriate or a warn is safer (favor warn to preserve audit-must-not-fail) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Slug pattern check in `validate_structural` | pure-core | In-memory regex/OrgSlug check; no I/O |
| `validate_all` visibility change | pure-core | Pure API surface change |
| `start_clone` match dispatch | effectful-shell | Spawns async HTTP server tasks |
| `poll_test_hook` Notify await | effectful-shell | Async I/O wait via tokio |
| `sanitize_error_message` | pure-core | String processing; no I/O |
| Audit org_slug cross-check | effectful-shell | Reads `OrgRegistry` (RwLock read); emits tracing warn |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4 000 |
| BC files (5 BCs) | ~7 000 |
| `validator.rs` (prism-customer-config, ~600 lines) | ~4 500 |
| `clone_server.rs` (prism-dtu-harness, ~850 lines) | ~6 000 |
| `audit_emitter.rs` (~300 lines) | ~2 200 |
| New/modified test files | ~1 500 |
| Cargo output | ~800 |
| **Total** | **~26 000** |

This is near the 20-30% context-window limit for a 200k-token agent. If token pressure
arises, split into two sub-tasks: Sub-burst A = prism-customer-config sub-fixes (CR-003,
CR-005, SEC-006); Sub-burst B = prism-dtu-harness + prism-audit sub-fixes (CR-004,
CR-006, SEC-007).

## Previous Story Intelligence

- **S-3.3.01** established the `ConfigError` enum and E-CFG-NNN taxonomy. E-CFG-019 must
  not conflict with E-CFG-018 (added by W3-FIX-SEC-003). Assign E-CFG-019 only after
  confirming E-CFG-018 is committed.
- **S-3.3.05 and S-3.4.* migrations** established the `start_clone` dispatch pattern
  currently in `clone_server.rs`. The `if`-chain dispatch was introduced during the
  Armis/Claroty specialized startup functions migration; the PR author noted it as "TODO:
  consolidate to match" — this story closes that TODO.
- **S-3.7.01 (archetype catalog):** References `DtuType` — if a new `DtuType` variant
  was added in the Wave 3.7 stories, ensure the new `match` in `start_clone` covers it.

## Architecture Compliance Rules

- `E-CFG-019` MUST follow in sequence after E-CFG-018 (added by W3-FIX-SEC-003). If
  E-CFG-017 exists, use E-CFG-019. If the enum already has a gap, fill the next
  available slot and document it.
- `validate_all` visibility change (`pub` → `pub(crate)`) MUST be verified with
  `cargo build --workspace` before committing — a breakage here means an external crate
  was depending on `validate_all` directly.
- The `match dtu_type` in `start_clone` MUST be annotated with
  `#[deny(unused_variables)]` or ensure the `match` is exhaustive at compile time
  (no `..` catch-all that silently ignores new variants).
- Audit cross-check MUST use `tracing::warn!` in release builds, never `panic!`. The
  audit pipeline must not fail due to a slug mismatch (BC-3.1.002 states slug is
  denormalized at write time for forensic readability, not a hard invariant enforced
  at write time).

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| tokio::sync::Notify | tokio (workspace) | poll_test_hook refactor (preferred) |
| tracing | workspace pin | `warn!` macro for audit cross-check |
| prism-core (OrgSlug, ORG_SLUG_PATTERN) | workspace | Pattern validation in validator |
| prism-core (OrgRegistry) | workspace | slug_for lookup in audit emitter |

No new external Cargo dependencies.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-customer-config/src/validator.rs` | Modify | CR-003: slug pattern check + E-CFG-019; CR-005: validate_all visibility; SEC-006: sanitize test cases |
| `crates/prism-customer-config/src/error.rs` (or inline) | Modify | Add `InvalidOrgSlugPattern` variant E-CFG-019 |
| `crates/prism-dtu-harness/src/clone_server.rs` | Modify | CR-004: exhaustive match in start_clone; CR-006: poll_test_hook backoff |
| `crates/prism-audit/src/audit_emitter.rs` | Modify | SEC-007: org_slug cross-check |
| `crates/prism-customer-config/tests/validation_hygiene_test.rs` | Create or modify existing | Regression tests for CR-003, CR-005, SEC-006 |
| `crates/prism-dtu-harness/tests/dispatch_hygiene_test.rs` | Create or modify existing | Regression test for CR-004 (compile check via build test) |
| `crates/prism-audit/tests/audit_slug_test.rs` | Create or modify existing | Regression test for SEC-007 |

## Forbidden Dependencies

- Do NOT add any new runtime crate dependencies to `prism-customer-config`.
- Do NOT change the public signature of `load_and_validate` — it remains the public
  entry point. Only `validate_all` changes visibility.
- Do NOT use `unwrap()` in the audit cross-check path — `OrgRegistry::slug_for` may
  return `None` and must be handled gracefully.
