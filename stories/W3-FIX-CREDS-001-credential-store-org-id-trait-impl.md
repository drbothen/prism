---
story_id: W3-FIX-CREDS-001
title: "prism-credentials: implement CredentialStoreOrgId trait bodies — replace todo!() stubs"
wave: 3.2
level: "L4"
target_module: prism-credentials
subsystems: [SS-03]
priority: P0
depends_on: []
blocks: []
estimated_days: 2
points: 5
status: draft
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-02T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/adversarial-reviews/pass-48.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass2.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review-pass2.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-f-holdout-evaluation.md
  - .factory/specs/behavioral-contracts/BC-3.2.002-per-org-credential-isolation.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.2.002
verification_properties: [VP-3.2.002-01, VP-3.2.002-02, VP-3.2.002-03]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.2.002]
anchor_capabilities: [CAP-004]
anchor_subsystem: ["SS-03"]
tdd_mode: strict
parent_finding: "TD-W3-CREDS-001; gate-step-f-holdout-evaluation.md BC-3.2.002 gap; gate-step-d-security-review-pass2.md SEC-P2-001 NOTE"
# BC status: anchored — BC-3.2.002 fully authored
---

# W3-FIX-CREDS-001: prism-credentials — implement CredentialStoreOrgId trait bodies, replace todo!() stubs

## Narrative

As a Prism credentials subsystem maintainer, I want the `CredentialStoreOrgId` trait
methods `get_by_org`, `set_by_org`, and `delete_by_org` in
`crates/prism-credentials/src/trait_.rs` to have real implementations (not `todo!()`
bodies), so that the holdout cross-org isolation proptest (`bc_3_2_002_org_id_namespace`)
completes rather than hanging, and any backend configured to use OrgId-keyed credential
operations does not panic at runtime.

## Objective

The holdout-evaluator pass-2 (gate-step-f `gate-step-f-holdout-evaluation.md`) confirmed
that BC-3.2.002 ("Per-Org Credential Isolation via OrgId-Keyed Namespace") is
**unimplemented** in `prism-credentials/src/trait_.rs`. The three core trait methods carry
doc comments reading "STUB — `todo!()` pending Red Gate test passage" — a known structural
placeholder from S-3.1.04 that was never promoted to an implementation story.

The consequence is two-fold:
1. The proptest `bc_3_2_002_org_id_namespace::proptest_BC_3_2_002_vp_01_cross_org_isolation`
   hangs (runs `todo!()` → panic → proptest deadlock in the async harness).
2. Any process that calls `get_by_org` / `set_by_org` / `delete_by_org` via a backend
   other than `KeyringBackend` panics unconditionally.

The pattern is identical to what W3-FIX-CODE-003 did for `KeyringBackend` — that story
(PR #115) confirmed `KeyringBackend::CredentialStoreOrgId` was already complete
(SEC-004 retracted as false positive). This story targets the **trait_ default bodies**
and any remaining stub implementations, using the namespace key pattern
`"{org_id_uuid}/{sensor}/{name}"` already codified in `namespace.rs`.

TD-W3-CREDS-001 was filed in the D-184 cycle-manifest amendment. This story closes it.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.2.002 | Per-Org Credential Isolation via OrgId-Keyed Namespace | Precondition 1 (namespace key format `{org_id_uuid}/{sensor}/{name}`), Postcondition 1 (`get` returns correct cred for matching org), Postcondition 2 (`get` returns `Err(NotFound)` for wrong org), Invariant 1 (namespace key derived from OrgId UUID, never OrgSlug), Invariant 3 (physical separation by namespace string prefix) |

## Acceptance Criteria

### AC-001: `get_by_org` returns credential stored under `{org_id_uuid}/{sensor}/{name}` (traces to BC-3.2.002 postcondition 1)
`CredentialStoreOrgId::get_by_org(&org_id, sensor, name)` returns `Ok(Some(SecretString))`
containing the value that was stored under the namespace key
`"{org_id_uuid}/{sensor}/{name}"` (as produced by `namespace_key_by_org_id`). No
`todo!()` body remains in `trait_.rs`.

### AC-002: `set_by_org` stores under `"{org_id_uuid}/{sensor}/{name}"` namespace (traces to BC-3.2.002 precondition 1)
`CredentialStoreOrgId::set_by_org(&org_id, sensor, name, secret)` stores the value under
the key produced by `namespace_key_by_org_id(org_id, sensor, name)`. A subsequent
`get_by_org` with the same arguments returns the same secret. The format matches the
`namespace_key_by_org_id` helper in `prism-credentials/src/namespace.rs`.

### AC-003: `delete_by_org` removes entry; subsequent get returns None (traces to BC-3.2.002 invariant 3)
After `delete_by_org(&org_id, sensor, name)` returns `Ok(())`, a subsequent
`get_by_org(&org_id, sensor, name)` returns `Ok(None)` or `Err(NotFound)`.
No panic occurs on a double-delete (idempotent).

### AC-004: Cross-org proptest passes — Org A credential not retrievable by Org B (traces to BC-3.2.002 postcondition 2)
The existing proptest `proptest_BC_3_2_002_vp_01_cross_org_isolation` in
`crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs` completes without hanging
or panicking. The test asserts: a credential stored under `(org_id_a, sensor, name)` is
NOT returned by `get_by_org(org_id_b, sensor, name)` for any `org_id_a ≠ org_id_b`.
`cargo nextest run -p prism-credentials` exits 0.

### AC-005: Credential bytes returned as `SecretString` — no leak in Debug (traces to BC-3.2.002 postcondition 4)
`get_by_org` returns `Ok(Some(SecretString))`. The `Debug` / `Display` impl of the return
type does NOT expose the raw secret bytes (redacted as `"[REDACTED]"` or similar). No
credential value appears in any log statement added in this story.

### AC-006: Backwards-compat slug-based methods continue to work (traces to BC-3.2.002 invariant 1)
The deprecated `get` / `set` / `delete` methods keyed by `OrgSlug` (or their
`slug_key`-based equivalents) continue to compile and pass their existing tests. The impl
of the new OrgId-keyed methods must NOT remove or break existing slug-keyed paths — only
additive changes.

## Tasks

1. Read `crates/prism-credentials/src/trait_.rs` in full — locate the
   `CredentialStoreOrgId` trait definition and all `todo!()` method bodies across the
   file.
2. Read `crates/prism-credentials/src/namespace.rs` — extract the exact signature and
   return value of `namespace_key_by_org_id`.
3. Read `crates/prism-credentials/src/keyring_backend.rs` lines covering
   `CredentialStoreOrgId` impl (confirmed complete in W3-FIX-CODE-003 PR #115) — use as
   the authoritative implementation reference for the `get/set/delete_by_org` bodies.
4. Read `crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs` — understand
   the hanging proptest; identify which method call triggers the `todo!()`.
5. Replace `todo!()` bodies in `trait_.rs` for `get_by_org`, `set_by_org`,
   `delete_by_org` using `namespace_key_by_org_id` to construct the key, then
   forwarding to the underlying storage (file or keyring backend) via the same call path
   already used by the slug-keyed methods.
6. If `EncryptedFileBackend` or other backend structs also carry `todo!()` stubs for
   `CredentialStoreOrgId`, apply the same fix to each (check via `grep -rn "todo!" crates/prism-credentials/`).
7. Run `cargo test -p prism-credentials --all-features` — all tests pass, proptest
   no longer hangs.
8. Run `cargo nextest run -p prism-credentials` — confirm 0 failures.
9. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `CredentialStoreOrgId` trait | prism-credentials | `crates/prism-credentials/src/trait_.rs` | Pure (trait definition + default impls) |
| `namespace_key_by_org_id` helper | prism-credentials | `crates/prism-credentials/src/namespace.rs` | Pure (string construction) |
| `KeyringBackend` impl (reference) | prism-credentials | `crates/prism-credentials/src/keyring_backend.rs` | Effectful (OS keyring I/O) |
| `EncryptedFileBackend` impl (if stubbed) | prism-credentials | `crates/prism-credentials/src/encrypted_file_backend.rs` | Effectful (file I/O) |
| proptest suite | prism-credentials | `crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs` | Effectful (async runtime) |

**Subsystem anchor justification:** SS-03 (Credential Management) owns this story's
scope because `prism-credentials` is the credential store crate, which is the canonical
home of SS-03 per the ARCH-INDEX Subsystem Registry. All changes are confined to
`crates/prism-credentials/`.

**Dependency anchor justification:** `depends_on: []` — this is a self-contained stub
implementation; it does not require any other W3.2 fix story. `blocks: []` — no
downstream W3.2 story requires credential OrgId-keying to be complete first (the BC-3.2.002
gap is isolated to this crate).

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `CredentialStoreOrgId` trait methods in `trait_.rs` | pure-core (trait bodies) | String key construction via `namespace_key_by_org_id` is pure; the actual I/O is delegated to the backend impl |
| `namespace_key_by_org_id` helper | pure-core | Pure string construction; no I/O, no side effects |
| `KeyringBackend::get/set/delete_by_org` (reference impl) | effectful-shell | Makes OS keyring syscalls |
| `EncryptedFileBackend::get/set/delete_by_org` (if patched) | effectful-shell | Reads/writes encrypted files on disk |
| Proptest suite `bc_3_2_002_org_id_namespace.rs` | effectful-shell | Spawns async runtime; exercises backend I/O |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `get_by_org(org_id_b, sensor, name)` when only `org_id_a` has a credential for that key | Returns `Ok(None)` or `Err(NotFound)` — never `org_id_a`'s value |
| EC-002 | `delete_by_org` called twice on same key | Second call returns `Ok(())` (idempotent) or `Err(NotFound)` — never panics |
| EC-003 | `set_by_org` called with empty `name` string | Depends on backend constraint; document behavior; do not panic |
| EC-004 | `org_id` is nil UUID (`00000000-0000-0000-0000-000000000000`) | Functions must work correctly — nil UUID is a valid key prefix; isolation still holds |
| EC-005 | `SecretString` constructed from empty bytes | Should construct without panic; `get` returns `Ok(Some(SecretString::from("")))` |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3 500 |
| BC files (1 BC — BC-3.2.002) | ~1 500 |
| `trait_.rs` (full file) | ~2 500 |
| `namespace.rs` (helper functions) | ~800 |
| `keyring_backend.rs` (reference impl, relevant sections) | ~1 500 |
| `encrypted_file_backend.rs` (if also stubbed) | ~1 500 |
| `bc_3_2_002_org_id_namespace.rs` (proptest) | ~1 200 |
| `cargo test` output | ~800 |
| **Total** | **~13 300** |

Comfortably within a single agent context window. If `encrypted_file_backend.rs` is
large, load only the `CredentialStoreOrgId` impl section.

## Previous Story Intelligence

- **W3-FIX-CODE-003** (PR #115): confirmed `KeyringBackend::CredentialStoreOrgId` was
  fully implemented. SEC-004 was retracted as a false positive. This story addresses
  the remaining `todo!()` stubs in `trait_.rs` itself (and any other backend that may
  also have stubs).
- **S-3.1.04** introduced the `CredentialStoreOrgId` trait and `namespace_key_by_org_id`
  helper with the intent of migrating from slug-keyed to UUID-keyed lookups. The trait
  bodies were left as `todo!()` pending the Red Gate test passage — but the Red Gate
  passed without the impl being wired in.
- **Lesson:** A `// STUB — todo!() pending Red Gate test passage` comment is a
  time-bomb: once the gate passes, the stub must be promoted immediately. The
  mechanism to catch this is the holdout evaluator proptest; when a proptest hangs
  rather than fails, the underlying cause is almost always a `todo!()` in an async fn.

## Architecture Compliance Rules

- The namespace key MUST be produced by `namespace_key_by_org_id(org_id, sensor, name)`
  from `crates/prism-credentials/src/namespace.rs`. Do NOT inline the string format —
  the helper is the single source of truth for the format.
- Credential values MUST be wrapped in `SecretString` before being returned from
  `get_by_org`. Do NOT return raw `String` or `Vec<u8>`.
- Do NOT add any new public API surface to `prism-credentials`. The trait methods are
  already declared; only bodies change.
- The slug-keyed `get` / `set` / `delete` methods MUST NOT be removed or broken. They
  are marked `#[deprecated]` but backward compat is required until Wave 4 migration
  (ADR-006 §4 Step 4).
- Do NOT introduce `unsafe` code. Keyring and file I/O are handled by safe abstractions
  (`keyring` crate and `std::fs`).

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| `secrecy` | workspace pin | `SecretString` type for credential wrapping |
| `keyring` | workspace pin | OS keyring backend (already dep in `prism-credentials`) |
| `uuid` | workspace pin | `OrgId` → UUID string conversion |
| `tokio` | workspace pin | Async trait executor |

No new Cargo dependencies introduced by this story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-credentials/src/trait_.rs` | Modify | Replace `todo!()` bodies in `get_by_org`, `set_by_org`, `delete_by_org` |
| `crates/prism-credentials/src/encrypted_file_backend.rs` | Modify (if stubbed) | Apply same OrgId-keyed namespace fix if `todo!()` found via grep |
| `crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs` | No change expected | Proptest already written; should now pass without modification |

## Forbidden Dependencies

- Do NOT add `prism-core` as a new dependency of `prism-credentials` if it is not
  already a dependency. `OrgId` must be imported from the existing dependency chain.
- Do NOT add any external crate not already present in `prism-credentials`'s `Cargo.toml`.
