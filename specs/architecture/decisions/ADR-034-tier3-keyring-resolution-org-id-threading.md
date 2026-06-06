---
document_type: adr
adr_id: "ADR-034"
title: "Tier-3 Keyring Resolution — OrgId Threading via Injected OrgRegistry, OrgId-Keyed Write Reconciliation, and Async Safety"
status: accepted
date: 2026-06-06
author: architect
decision_made_by: human (Option-A selection for S-DEMO-003 scope expansion, 2026-06-06)
supersedes: null
superseded_by: null
related_adrs: ["ADR-032", "ADR-022", "ADR-006"]
related_bcs: ["BC-2.06.003", "BC-2.03.006", "BC-3.2.002", "BC-2.03.007"]
traces_to: "ARCH-INDEX.md"
---

# ADR-034: Tier-3 Keyring Resolution — OrgId Threading via Injected OrgRegistry, OrgId-Keyed Write Reconciliation, and Async Safety

## Status

Accepted. Human approved Option A (full Tier-3 keyring implementation) as S-DEMO-003 scope expansion (2026-06-06). This ADR gates the S-DEMO-003 TDD restart.

## Context

ADR-032 and BC-2.06.003 v1.3 define a four-tier per-client credential resolution chain. Tier 3 is documented as "OS keyring via `CredentialStoreOrgId::get_by_org`" — but was never implemented. `resolve_credential` in `prism-credentials/src/resolution.rs` has a comment at lines 18 and 92 stating "Tier 3 not implemented here, delegated to CRUD store lookup."

Two structural gaps were identified during S-DEMO-003 architecture adjudication (proposal `.factory/proposals/S-DEMO-003-credential-channel-adjudication.md`):

**CRIT-1 (gap):** `resolve_credential` has no Tier-3 keyring branch. After Tier 1/2 env-var misses, it falls directly to Tier-4 CRUD store. The CRUD store is never populated by `prism credential set` (which writes to the keyring directly). Result: every query after `prism credential set` fails with E-AUTH-005.

**CRIT-2 (namespace mismatch):** The current `credential_cli.rs` `handle_credential_set` writes via `CredentialStore::set` → legacy `namespace_key(OrgSlug, ...)` → key `"{slug}/{sensor}/{name}"`. But `CredentialStoreOrgId::get_by_org` reads via `namespace_key_by_org_id(OrgId, ...)` → key `"{org_id_uuid}/{sensor}/{name}"`. These are disjoint keyring namespaces. A write via the legacy slug-keyed path is permanently invisible to the OrgId-keyed read path.

**Design question:** How does Tier 3 in `resolve_credential` obtain the `OrgId` for the keyring key? `resolve_credential`'s current signature is `(client_id: &str, sensor_id: &str, credential_name: &str)` — `client_id` is the org slug string, not an `OrgId`. Two options:

- **Option X:** Add `OrgId` directly to `resolve_credential`'s signature — requires all callers to supply `OrgId`, which means every auth provider must have access to an `OrgRegistry` or pre-resolved `OrgId`.
- **Option Y:** Inject an `Arc<OrgRegistry>` into `resolve_credential` for slug→OrgId lookup — callers continue to pass org slug (no change to `AuthProvider::acquire_token` signature which takes `client_id: &OrgSlug`).

Option Y is chosen. See §Decision.

## Decision

### D1: Resolver Signature — Pre-resolved `OrgId`, Consistent with `prism-credentials` Architecture Compliance Rule

**Architecture constraint:** `crates/prism-credentials/src/trait_.rs:84–85` contains the rule: "`prism-credentials` MUST NOT import `OrgRegistry`. Callers obtain a resolved `OrgId` via `OrgRegistry::resolve(slug)` before calling these methods (ADR-006 §2.3)." Although `OrgRegistry` lives in `prism-core` (already a dep of `prism-credentials`), the rule prohibits importing it — callers own the slug→OrgId resolution. This is a boundary integrity rule to keep `prism-credentials` focused on storage, not identity resolution.

Therefore, `resolve_credential` in `prism-credentials/src/resolution.rs` takes a pre-resolved `OrgId`, not an `OrgRegistry`. The slug→OrgId conversion happens in `PrismCredentialResolver` (in `prism-spec-engine`, which CAN import `OrgRegistry`).

The updated `resolve_credential` signature:

```rust
pub async fn resolve_credential(
    client_id: &str,          // org slug — used for Tier 1/2 env-var name derivation
    sensor_id: &str,
    credential_name: &str,
    org_id: Option<&OrgId>,   // NEW — pre-resolved; None disables Tier 3
    keyring: Option<&Arc<dyn CredentialStoreOrgId>>,  // NEW — None disables Tier 3
) -> Result<SecretString, CredentialResolutionError>
```

`Option<&OrgId>` and `Option<Arc<dyn CredentialStoreOrgId>>` allow callers without Tier-3 capability (e.g., unit tests in `prism-credentials/tests/`) to pass `None` — Tier 3 is skipped gracefully. When both are `Some`, Tier 3 is active.

**`PrismCredentialResolver` in `prism-spec-engine/src/auth_provider.rs`** becomes a struct that holds the `OrgRegistry` and performs slug→OrgId resolution before calling `resolve_credential`:

```rust
pub struct PrismCredentialResolver {
    org_registry: Arc<OrgRegistry>,
    keyring: Arc<dyn CredentialStoreOrgId>,
}

impl PrismCredentialResolver {
    pub fn new(org_registry: Arc<OrgRegistry>, keyring: Arc<dyn CredentialStoreOrgId>) -> Self {
        Self { org_registry, keyring }
    }
}

impl CredentialResolver for PrismCredentialResolver {
    fn resolve<'a>(...) -> Pin<Box<...>> {
        Box::pin(async move {
            // slug→OrgId resolution happens here (in prism-spec-engine, which can import OrgRegistry)
            let org_slug = OrgSlug::new(&client_id)
                .map_err(|_| CredentialResolutionError::NotFound { ... })?;
            let org_id = self.org_registry.resolve(&org_slug);  // Option<OrgId>
            prism_credentials::resolve_credential(
                &client_id, &sensor_id, &credential_name,
                org_id.as_ref(),         // Option<&OrgId>
                Some(&self.keyring),     // Option<&Arc<dyn CredentialStoreOrgId>>
            ).await
        })
    }
}
```

**`CredentialResolver` trait** in `prism-spec-engine/src/auth_provider.rs` is UNCHANGED — the existing `resolve(client_id, sensor_id, credential_name)` signature remains. The OrgId resolution and keyring injection are internal to `PrismCredentialResolver`. The 5 test double implementors (`MockCredentialResolver`, `NotFoundCredentialResolver`, `BackendUnavailableCredentialResolver`, plus any others) require NO signature change. This is a strictly additive change: only `PrismCredentialResolver` changes from unit struct to struct-with-fields.

**`StaticCookieAuthProvider`** is UNCHANGED in its constructor signature for tests. The only change is that `StaticCookieAuthProvider::new(sensor_id)` — the production no-test-override path — now requires `Arc<OrgRegistry>` and `Arc<dyn CredentialStoreOrgId>` to construct a properly-wired `PrismCredentialResolver`:

```rust
pub fn new(
    sensor_id: impl Into<String>,
    org_registry: Arc<OrgRegistry>,
    keyring: Arc<dyn CredentialStoreOrgId>,
) -> Self {
    Self {
        sensor_id: sensor_id.into(),
        resolver: Arc::new(PrismCredentialResolver::new(org_registry, keyring)),
    }
}
```

`StaticCookieAuthProvider::new_with_resolver` is UNCHANGED — tests inject mock resolvers directly and are unaffected.

**`PluginAuthProvider`** in `prism-spec-engine/src/plugin_auth_provider.rs` calls `prism_credentials::resolve_credential` directly at lines 135 and 145. It must be given the same DI fields (`org_registry: Arc<OrgRegistry>`, `keyring: Arc<dyn CredentialStoreOrgId>`), do the slug→OrgId lookup internally, and pass pre-resolved `org_id` and `keyring` to `resolve_credential`. `PluginAuthProvider::new` gains these two fields.

### D2: Tier-3 Implementation in `resolve_credential`

The Tier-3 branch is inserted between the env-var resolution block and the Tier-4 CRUD lookup:

```
Tier 1/2: env-var chain (existing)
  → hit: return Ok(secret) + audit log "env"
  → miss: fall through to Tier 3
  → error: hard error (FILE var set but file missing — no fallthrough)

Tier 3: OS keyring lookup (NEW)
  → if org_id is None or keyring is None: fall through to Tier 4 (Tier 3 disabled)
  → call keyring.get_by_org(org_id, sensor_id, &cred_name).await
  → hit: return Ok(secret) + audit log "keyring"
  → miss (Ok(None) / NoEntry): fall through to Tier 4
  → backend error (keyring locked, unavailable): HARD ERROR → CredentialResolutionError::BackendUnavailable
    (SOUL.md §4: do not silently fall through on a backend failure — see §D4 below)

Tier 4: CRUD store lookup (existing)
```

Note: the `org_id` parameter to `resolve_credential` is `Option<&OrgId>` (pre-resolved by the caller per the architecture compliance rule in `trait_.rs`). The slug→OrgId resolution step happens in `PrismCredentialResolver::resolve` in `prism-spec-engine` (see §D1).

**Implementation detail — `spawn_blocking` requirement:**

`CredentialStoreOrgId::get_by_org` is an async method on `KeyringBackend` that internally uses `tokio::task::spawn_blocking` (see `keyring.rs:262–285`). Calling `keyring.get_by_org(...)` from within `resolve_credential`'s async context is correct — the `spawn_blocking` is already encapsulated inside the trait implementation. No additional `spawn_blocking` wrapping is needed in `resolve_credential` itself.

**Concurrency (ADR-022 §D):** The 8-permit sensor-fetch semaphore gates the entire `resolve_credential` call path. Tier-3 adds one `spawn_blocking` task (internally inside `KeyringBackend::get_by_org`) to the permit-held scope. This is acceptable: the keyring call is O(1) local I/O (no network), typically sub-millisecond on macOS Keychain and Windows Credential Vault. No second semaphore pool is needed.

**Keyring backend instance:** `resolve_credential` accepts `Option<&Arc<dyn CredentialStoreOrgId>>` as an additional parameter — pure DI, maximally testable. This is consistent with approach (a) from D1. The full updated signature (canonical):

```rust
pub async fn resolve_credential(
    client_id: &str,                                      // org slug (Tier 1/2 env-var derivation)
    sensor_id: &str,
    credential_name: &str,
    org_id: Option<&OrgId>,                               // pre-resolved; None → skip Tier 3
    keyring: Option<&Arc<dyn CredentialStoreOrgId>>,      // None → skip Tier 3
) -> Result<SecretString, CredentialResolutionError>
```

Tier 3 is active only when both `org_id` and `keyring` are `Some`. If either is `None`, the function falls through to Tier 4 without error — this is the graceful-degradation path for unit tests and for callers that have not yet been wired with keyring access.

### D3: Namespace Reconciliation — `credential_cli.rs` Must Use `CredentialStoreOrgId::set_by_org`

`handle_credential_set` in `prism-bin/src/credential_cli.rs` currently writes via `CredentialStore::set` using the legacy slug-keyed `namespace_key` (`"{slug}/{sensor}/{name}"`). This MUST change to `CredentialStoreOrgId::set_by_org` using `namespace_key_by_org_id` (`"{org_id_uuid}/{sensor}/{name}"`).

**Required change in `handle_credential_set`:**

1. Load `PrismConfig` from `config_dir/prism.toml` to obtain `[[orgs]]`.
2. Map the resolved org slug to `OrgId` by reading the `org_id` field from `PrismConfig.orgs[n]`. No `OrgRegistry` is needed here — the `prism.toml` file itself contains the `org_id` UUID for each org.
3. Construct `OrgId` from the UUID string: `OrgId::from_str(&org.org_id)?`.
4. Call `CredentialStoreOrgId::set_by_org(&org_id, sensor, &cred_name, value)`.

**Slug resolution for multi-org configs:**

If `--org-slug` is provided, find the matching org in `PrismConfig.orgs` and use its `org_id`. If not provided and `config.orgs.len() == 1`, use the single org. If not provided and `config.orgs.len() > 1`, return error: "Multiple orgs configured in prism.toml — use `--org-slug <slug>` to select one."

**HIGH-3 remediation:** The current `resolve_org_slug` helper (as implemented in the worktree) swallows `prism.toml` read errors and falls back to `"demo-org"`. This is a SOUL.md §4 violation. The correct behavior: if `prism.toml` is missing or unparseable and no `--org-slug` was provided, return a clear error. The demo-org default fallback is removed. The implementer MUST enforce this at TDD time. The unit test `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug` is a required Red Gate test (see §Red Gate Tests).

### D4: Error Semantics — Keyring Backend Error is a Hard Error

BC-2.06.003 Tier-3 row: "OS keyring, OrgId-keyed format (BC-3.2.002 / `namespace_key_by_org_id`)." The error taxonomy for Tier 3:

| Condition | Behavior | Error |
|-----------|----------|-------|
| Keyring entry absent (`NoEntry`) | Fall through to Tier 4 | None — treat as miss |
| `org_registry` is `None` or slug not in registry | Fall through to Tier 4 | None — Tier 3 skipped |
| Keyring backend error (locked, `NoStorageAccess`, `NoKeyringService`, spawn panic) | Hard error — do NOT fall through | `CredentialResolutionError::BackendUnavailable { detail: "E-CRED-003: OS keyring unavailable: {reason}" }` |

**Rationale for hard error on backend failure:** A locked or unavailable keyring indicates the execution environment is misconfigured (e.g., macOS Keychain access denied, Linux libsecret D-Bus unavailable). Silently falling through to Tier 4 would hide a real operator error. SOUL.md §4: do not swallow errors. The error code `E-CRED-003` is a new entry in the error taxonomy (not previously defined — route to product-owner to add to `error-taxonomy.md`, or architect defines it here for downstream use: "OS keyring backend error at Tier 3 resolution time").

**Error code:** ADR-034 originally designated `E-CRED-003` for this error path. However, `E-CRED-003` was already allocated in `error-taxonomy.md` to "Credential decryption failed for ({client_id}, {sensor_id})" (key-material-changed / file-corrupted case). To avoid code collision, `E-CRED-005` is used instead (next free code in the E-CRED namespace after E-CRED-004). `E-CRED-005` is defined as: "OS keyring backend error during Tier-3 credential resolution. The keyring is inaccessible (locked, D-Bus unavailable, spawn panic). Use Tier 1/2 env vars or check keyring access." Wherever this ADR says `E-CRED-003`, read `E-CRED-005`. BC-2.06.003 v1.4 and `error-taxonomy.md` v1.61 both use `E-CRED-005` as the authoritative code. (Source-of-truth precedence: `error-taxonomy.md` + BC-2.06.003 v1.4 supersede this ADR for the specific error code — more-specific artifact wins per CLAUDE.md §Source-of-Truth Precedence.)

### D5: Boot Path Wiring — `PrismCredentialResolver` Construction

`PrismCredentialResolver` becomes non-unit in production. Boot must wire it:

1. `step5_init_credential_store` already produces `Arc<dyn CredentialStore>` (backed by `KeyringBackend`). The same `KeyringBackend` instance also implements `CredentialStoreOrgId`.
2. Boot step 5 must expose the `Arc<KeyringBackend>` (not erased to `Arc<dyn CredentialStore>`) so it can be injected into `PrismCredentialResolver`.
3. The `BootContext` struct gains `credential_store_org_id: Arc<dyn CredentialStoreOrgId>` alongside the existing `credential_store: Arc<dyn CredentialStore>`.
4. At the point where `StaticCookieAuthProvider`, `PluginAuthProvider`, and `BearerStaticCredentialAuthProvider` are constructed (step 9A in `spec_driven_adapter.rs`), the construction path passes `Arc::clone(&ctx.org_registry)` and `Arc::clone(&ctx.credential_store_org_id)` to `PrismCredentialResolver::new(...)`.

This is "wiring not redesign" per ADR-022 §C: the `KeyringBackend` already exists; we expose its `CredentialStoreOrgId` impl via the additional `Arc` without replacing any existing implementation.

### D6: Spec Amendment Plan

| Artifact | Change | Owner |
|----------|--------|-------|
| `BC-2.06.003` v1.3 §Tier 3 | Update from "not implemented" comment to "IMPLEMENTED via ADR-034" | product-owner |
| `BC-2.06.003` AC for Tier 3 | Add test vector for Tier-3 keyring resolution (write via `credential_cli`, query succeeds) | product-owner |
| `BC-2.03.007` | Verify `E-CRED-003` error detail does not leak credential value — it contains only `{reason}` from keyring-rs which never includes a secret value; no amendment needed | — |
| `error-taxonomy.md` | Add `E-CRED-003`: "OS keyring backend error at Tier-3 resolution time" | product-owner |
| `S-DEMO-003` AC-005 | Update: write via `CredentialStoreOrgId::set_by_org` (OrgId-keyed, ADR-034 D3); resolver reads via Tier-3 (ADR-034 D2); full end-to-end connectivity is now the contract | product-owner |
| `S-DEMO-003` §Architecture Compliance Rules | Replace `CredentialStore::set(org_id, ...)` row with `CredentialStoreOrgId::set_by_org(org_id, ...)` | product-owner |
| `ARCH-INDEX.md` ADR Registry | Add ADR-034 row | state-manager (post-ADR commit) |

### D7: Points / Effort Re-estimate for Expanded S-DEMO-003

The original S-DEMO-003 was scoped at 5 story points (LOW risk, 1 day). With Option A expansion:

| Addition | Effort |
|----------|--------|
| `resolve_credential` signature change + Tier-3 branch | MEDIUM (1–2 hours; spawn_blocking pattern is established) |
| `CredentialResolver` trait + 5 test-double sibling sweep | LOW (30 min; all boilerplate updates) |
| `PrismCredentialResolver` → struct with fields | LOW (30 min; DI wiring) |
| `StaticCookieAuthProvider` + `PluginAuthProvider` gain `Arc<OrgRegistry>` + `Arc<dyn CredentialStoreOrgId>` | MEDIUM (1 hour; 2 struct changes + callsite sweep) |
| `BootContext.credential_store_org_id` + step 5 wiring | LOW (30 min; additive only) |
| `credential_cli.rs` OrgId-keyed write + HIGH-3 error fix | MEDIUM (1 hour; prism.toml load + org_id extraction) |
| Red Gate tests (see below) | MEDIUM (2 hours; 4 new tests) |
| HIGH-1 (runbook env format), HIGH-2 (shellcheck CI) | LOW (30 min total) |

**Revised estimate:** 8–10 story points, 1.5–2 days. Risk level: MEDIUM (signature blast radius across 3 crates; well-understood pattern).

## Consequences

### Positive
- Tier 3 is now fully implemented and connected, closing CRIT-1 and CRIT-2.
- `prism credential set` is the authoritative end-to-end credential bootstrap path, matching operator UX expectations.
- The DI injection pattern (Option Y: `Arc<OrgRegistry>` in `PrismCredentialResolver`) preserves the existing `AuthProvider` interface — no change to `AuthProvider::acquire_token` signature or callers.
- Keyring calls are wrapped in `spawn_blocking` — no tokio thread pool blocking.
- Test-double injection points are preserved; all 5 `CredentialResolver` test doubles remain injectable.
- `E-CRED-003` gives operators an actionable error when the keyring is unavailable.

### Negative / Cost
- `PrismCredentialResolver` becomes non-unit (struct with 2 fields). Any code that `Arc::new(PrismCredentialResolver)` must change to `Arc::new(PrismCredentialResolver::new(org_registry, keyring))`.
- `StaticCookieAuthProvider::new` gains 2 additional parameters — call sites in spec_driven_adapter.rs must be updated (all in the same crate, bounded blast radius).
- `resolve_credential` gains 2 parameters — callers not using `CredentialResolver` trait must pass them (only `PluginAuthProvider` calls it directly; `StaticCookieAuthProvider` goes through the trait).
- 2 parameters added to `CredentialResolver::resolve` — all 5 test doubles must be updated.

### Neutral
- Boot step 5 exposes `Arc<dyn CredentialStoreOrgId>` alongside `Arc<dyn CredentialStore>` — both point to the same `KeyringBackend` instance; no duplication of state.
- `E-CRED-003` detail string from keyring-rs never contains a credential value (it is a system error message from the keyring backend, e.g., "access denied" or "D-Bus unavailable") — AD-017 compliance is maintained without additional sanitization.

## Red Gate Tests (required — test-writer specification)

The following tests MUST be written as failing Red Gates before any implementation:

### RG-034-001: End-to-end keyring write→resolution connectivity
**File:** `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs`
**Name:** `test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved`
**What:** Create a `KeyringBackend`, call `set_by_org(org_id, "armis", "bearer_token", value)`, then call `resolve_credential(org_slug, "armis", "bearer_token", Some(&registry), Some(&keyring))` with a matching `OrgRegistry` containing the slug→id pair. Assert `Ok(secret)` where `secret.expose_secret() == value`.
**This test is the critical gap closure for CRIT-1 and CRIT-2.**

### RG-034-002: Tier-3 keyring miss falls through to Tier-4
**File:** same test file
**Name:** `test_BC_2_06_003_tier3_miss_falls_through_to_tier4`
**What:** With a `KeyringBackend` that has no entry for the given `(org_id, sensor, name)`, and a CRUD store that also returns `Ok(None)`, assert `CredentialResolutionError::NotFound` (not a `BackendUnavailable`).

### RG-034-003: `resolve_org_slug` errors (no fallback) when prism.toml missing and no `--org-slug`
**File:** `crates/prism-bin/src/credential_cli.rs` `#[cfg(test)] mod tests` or integration test
**Name:** `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug`
**What:** Call `resolve_org_slug(&None, tmp_dir_without_prism_toml)`. Assert `Err(...)` — not `Ok("demo-org")`. HIGH-3 coverage.

### RG-034-004: `handle_credential_set` writes OrgId-keyed keyring entry
**File:** `crates/prism-bin/tests/bc_2_03_007_credential_set_org_id_keyed.rs`
**Name:** `test_handle_credential_set_writes_org_id_keyed_keyring_entry`
**What:** Create a temp dir with a `prism.toml` containing one org with a known `org_id` UUID. Call `handle_credential_set` (piped stdin path). Read back via `KeyringBackend::get_by_org(org_id, sensor, name)`. Assert the entry exists under the OrgId-keyed namespace. Assert it does NOT exist under the legacy slug-keyed namespace (CRIT-2 regression test).

## File Create / Modify List

### Create (new files)
- `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` — RG-034-001, RG-034-002

### Modify (existing files — implementer blast radius)

| File | Change |
|------|--------|
| `crates/prism-credentials/src/resolution.rs` | Add Tier-3 branch; new signature with `org_registry: Option<&OrgRegistry>` + `keyring: Option<&Arc<dyn CredentialStoreOrgId>>` |
| `crates/prism-credentials/src/lib.rs` | Re-export `resolve_credential` with updated signature |
| `crates/prism-spec-engine/src/auth_provider.rs` | `CredentialResolver::resolve` trait: add `org_registry: Option<&OrgRegistry>` param; update `PrismCredentialResolver` from unit struct to struct with `org_registry` + `keyring` fields; update `PrismCredentialResolver::new`; update 5 test double impls; update `StaticCookieAuthProvider` to hold `org_registry: Arc<OrgRegistry>`; update `StaticCookieAuthProvider::new` + `new_with_resolver` |
| `crates/prism-spec-engine/src/plugin_auth_provider.rs` | `PluginAuthProvider` gains `org_registry: Arc<OrgRegistry>` + `keyring: Arc<dyn CredentialStoreOrgId>`; update `new()`; update two `resolve_credential` callsites (lines 135, 145) |
| `crates/prism-bin/src/boot.rs` | `BootContext` gains `credential_store_org_id: Arc<dyn CredentialStoreOrgId>`; step 5 exposes `Arc<KeyringBackend>` alongside `Arc<dyn CredentialStore>`; wire into `BootContext` |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Auth provider construction sites (step 9A) gain `Arc::clone(&ctx.org_registry)` + `Arc::clone(&ctx.credential_store_org_id)` parameters |
| `crates/prism-bin/src/credential_cli.rs` | `handle_credential_set`: replace `CredentialStore::set` with `CredentialStoreOrgId::set_by_org`; load prism.toml + map slug→OrgId from `PrismConfig.orgs`; remove demo-org fallback from `resolve_org_slug`; add error on multi-org without `--org-slug` |
| `crates/prism-bin/tests/bc_2_03_007_credential_set_org_id_keyed.rs` | New test: RG-034-004 |
| `scripts/demo-setup.sh` (if exists) | Use `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` format — HIGH-1 remediation |
| `.github/workflows/ci.yml` | Add shellcheck step for `scripts/demo-*.sh` — HIGH-2 remediation |
| `error-taxonomy.md` (`prd-supplements`) | Add `E-CRED-003` entry — route to product-owner |
| `BC-2.06.003` | Update Tier-3 description — route to product-owner |
| `S-DEMO-003` story spec AC-005 + §Architecture Compliance — route to product-owner |

## Alternatives Considered

**Option X (add `Arc<OrgRegistry>` directly to `resolve_credential`):** Rejected because `prism-credentials/src/trait_.rs:84–85` prohibits `prism-credentials` from importing `OrgRegistry`. Although `prism-core` (which contains `OrgRegistry`) is already a dependency, the architecture compliance rule mandates that callers pre-resolve `OrgSlug→OrgId` before calling credential store methods. Passing `OrgRegistry` into `resolve_credential` would violate this rule.

**Option Z (add `OrgId` directly to `resolve_credential` without pre-resolution responsibility):** Initial draft used `Option<&OrgRegistry>` but the architecture compliance rule requires pre-resolution. The final design uses `Option<&OrgId>` (pre-resolved in `PrismCredentialResolver` in `prism-spec-engine`) + `Option<&Arc<dyn CredentialStoreOrgId>>`. This correctly satisfies the compliance rule while centering the slug→OrgId resolution in the one place that can import `OrgRegistry`.

**Inject `Arc<OrgRegistry>` into the module via a global / thread-local:** Rejected. Global mutable state violates AD-008 (pure core / effectful shell) and is incompatible with test isolation (thread-local CREDENTIAL_STORE in `crud.rs` already causes isolation warnings).

**Transient `KeyringBackend` construction inside `resolve_credential`:** Rejected. Constructing a `KeyringBackend` (which loads a `CredentialIndex` from disk) inside a per-credential-resolution call is an unnecessary I/O cost. The boot path already has a single `KeyringBackend` instance; it should be injected, not re-constructed.

## References

- ADR-032 §Resolution tier order (Tier 3 specified but not previously implemented)
- ADR-022 §C (wiring not redesign — guides BootContext expansion)
- ADR-006 §OrgId/OrgSlug identity (OrgRegistry as bijective map)
- BC-2.06.003 v1.3 §Tier 3 (OrgId-keyed keyring key format)
- BC-3.2.002 (namespace_key_by_org_id format)
- SOUL.md §4 (do not swallow errors — motivates D4 hard-error on backend failure)
- AD-013 (tokio multi-threaded runtime — motivates spawn_blocking requirement)
- `crates/prism-credentials/src/resolution.rs` lines 18, 92 (the unimplemented Tier-3 comment)
- `crates/prism-credentials/src/keyring.rs` lines 248–285 (`CredentialStoreOrgId::get_by_org`)
- `crates/prism-credentials/src/namespace.rs` lines 40–42 (`namespace_key_by_org_id`)
- `.factory/proposals/S-DEMO-003-credential-channel-adjudication.md` (CRIT-1, CRIT-2 evidence)
