---
document_type: behavioral-contract
level: L3
version: "1.12"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "fc9d874"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.003: Credential References in Config Resolve to Credential Store Entries

## Description

Sensor configuration entries declare `[[credential_refs]]` blocks whose names are
resolved at query time against the credential store using a **four-tier per-client
priority chain**. The canonical env-var format is
`PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` where `{ID}` is derived from the
org slug using the slug-to-SCREAMING-SNAKE transform defined below.

Resolution is scoped to `client_id` (the org slug), satisfying the credential
isolation invariant (DI-002): credentials for org `acme` can never be resolved
in the context of org `contoso`.

The resolved credential is a `SecretString` that is never logged or serialized.
The `credential_ref` name is validated against `[a-zA-Z0-9_\-\.]+` at config
load time (DI-014).

This convention is the **canonical multi-tenant credential convention for Prism**.
The implementer MUST conform `resolve_credential` and `KeyringCredentialProbe::probe`
to this BC. The v1.2 claim that "code mirrors the chain specified by this BC" was
false — the prior code used a global `{SENSOR}_{REF}` format that is not client-scoped
and does not satisfy DI-002. See Changelog for retraction.

---

## Slug-to-SCREAMING-SNAKE Transform (`{ID}`)

`{ID}` is derived from the org slug (the human-readable identifier, e.g. `demo-org-a`,
`acme`, `acme-corp`). The transform is:

1. Take the org slug exactly as declared in `prism.toml` `[[orgs]]` `org_slug`.
2. Convert to UPPERCASE.
3. Replace every hyphen (`-`) with an underscore (`_`).
4. No other substitution: alphanumerics and underscores pass through unchanged.

The slug alphabet is `[a-zA-Z0-9_-]{1,64}` (validated by `OrgSlug::new`), so step 3
produces a valid SCREAMING_SNAKE_CASE component for every valid slug.

### Worked examples

| `org_slug` (in prism.toml) | `{ID}` (in env var) |
|----------------------------|---------------------|
| `demo-org-a` | `DEMO_ORG_A` |
| `acme` | `ACME` |
| `acme-corp` | `ACME_CORP` |
| `contoso` | `CONTOSO` |

---

## Env-Var Name Derivation

For a given `(org_slug, sensor_id, credential_ref_name)` triple:

- `{ID}` = `slug.to_uppercase().replace('-', '_')`
- `{SENSOR}` = `sensor_id.to_uppercase().replace('-', '_')`
- `{REF}` = `ref_name.to_uppercase().replace('-', '_')`

### Env-var formats

| Tier | Env var name | Purpose |
|------|-------------|---------|
| 1 (highest) | `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE` | File path holding secret (K8s secret-mount pattern). Contents trimmed of leading/trailing whitespace before use. |
| 2 | `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` | Direct env-var value |
| 3 | Keyring: OrgId-keyed key `{org_id_uuid}/{sensor_id}/{ref_name}` via `CredentialStoreOrgId::get_by_org` | **IMPLEMENTED (ADR-034).** OS keyring lookup using OrgId-UUID-keyed namespace (BC-3.2.002 / `namespace_key_by_org_id`). Active only when both `org_id: Option<&OrgId>` and `keyring: Option<&Arc<dyn CredentialStoreOrgId>>` are `Some` in `resolve_credential`. Written via `CredentialStoreOrgId::set_by_org` (OrgId-keyed — the legacy `CredentialStore::set` slug-keyed path is NOT used for Tier 3; see ADR-034 §D3). |
| 4 (lowest) | CRUD store: `credential_status(client_id, sensor_id, credential_name)` → backend lookup | Programmatically configured source |

### Worked examples

| org_slug | sensor_id | ref_name | Tier 2 env var | Tier 1 env var |
|----------|-----------|----------|----------------|----------------|
| `demo-org-a` | `armis` | `secret_key` | `PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_SECRET_KEY` | `PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_SECRET_KEY_FILE` |
| `acme` | `claroty` | `bearer_token` | `PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN` | `PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN_FILE` |
| `acme` | `cyberint-alerts` | `access_token` | `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_ALERTS_ACCESS_TOKEN` | `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_ALERTS_ACCESS_TOKEN_FILE` |
| `acme` | `cyberint-assets` | `access_token` | `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_ASSETS_ACCESS_TOKEN` | `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_ASSETS_ACCESS_TOKEN_FILE` |
| `acme` | `crowdstrike` | `client_id` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_ID` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_ID_FILE` |
| `acme` | `crowdstrike` | `client_secret` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_SECRET` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_SECRET_FILE` |

---

## Preconditions
- A sensor config entry declares one or more `[[credential_refs]]` entries
- The caller supplies a valid `client_id` (org slug) in addition to `sensor_id` and `credential_ref_name`
- The credential store backend (keyring or encrypted file) is accessible

## Postconditions
- The `credential_ref_name` is resolved against the per-client priority chain using `(client_id, sensor_id, credential_ref_name)` as the namespace key
- Resolution follows the four-tier priority chain (Tier 1 highest):
  - **Tier 1:** `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE` — file path; contents trimmed. Hit → return `Ok(secret)` + audit "env". File var set but file missing → hard error (no fall-through to Tier 2).
  - **Tier 2:** `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` — direct env-var value. Hit → return `Ok(secret)` + audit "env". Miss → fall through to Tier 3.
  - **Tier 3 (IMPLEMENTED — ADR-034):** OS keyring via `CredentialStoreOrgId::get_by_org(org_id, sensor_id, ref_name)` — OrgId-keyed key `{org_id_uuid}/{sensor_id}/{ref_name}` (canonical namespace per `namespace_key_by_org_id`; legacy slug-keyed namespace NOT used). `resolve_credential` signature includes `org_id: Option<&OrgId>` and `keyring: Option<&Arc<dyn CredentialStoreOrgId>>`; Tier 3 is active only when both are `Some`. Error semantics per ADR-034 §D4:

    | Condition | Behavior | Error |
    |-----------|----------|-------|
    | `org_id` is `None` or `keyring` is `None` | Fall through to Tier 4 silently | None — Tier 3 skipped |
    | `get_by_org` returns `Ok(None)` / `NoEntry` | Fall through to Tier 4 | None — treat as miss |
    | `get_by_org` returns `Ok(Some(secret))` | Return `Ok(secret)` + audit "keyring" | None |
    | `get_by_org` returns `Err(...)` — backend locked / `NoStorageAccess` / `NoKeyringService` / spawn panic | Hard error — do NOT fall through | `CredentialResolutionError::BackendUnavailable { detail: "E-CRED-008: OS keyring unavailable: {reason}" }` (see error-taxonomy.md E-CRED-008; canonical keyring-unavailable code per ADR-035 §D2 — prior code E-CRED-005 had a collision with `PrismError::CredentialEncryptionError`; ADR-034 §D4 amended by ADR-035 §D5) |

  - **Tier 4:** CRUD store `credential_status` → backend source lookup. Hit → return `Ok(secret)`. Miss → `CredentialResolutionError::NotFound`.
- The resolved credential is available as a `SecretString` that is never logged or serialized
- Resolution is audit-logged (client, sensor, ref name only — never the value)
- Credential values are never included in error detail strings (AD-017; `E-CRED-008` detail is a system error message from the keyring backend, e.g., "access denied" — not a credential value)

## Invariants
- DI-002: Credential isolation per client — `{ID}` segment in env var guarantees per-org namespacing; Tier 3 uses OrgId UUID for rename-stability (BC-3.2.002)
- DI-014: Credential name sanitization — `credential_ref` name is validated against `[a-zA-Z0-9_\-\.]+`

---

## Per-Sensor `[[credential_refs]]` Declarations (Canonical)

Every built-in sensor TOML spec MUST declare a `[[credential_refs]]` block for each
credential name that `resolve_credential` will look up at query time. This enables
boot-step-5 validation (`KeyringCredentialProbe::probe`) to confirm each ref is
resolvable before the first query.

The `name` field in `[[credential_refs]]` is the `credential_ref_name` passed to
`resolve_credential`. It must match exactly what the auth provider passes as the
`credential_name` argument.

| Sensor | `auth_type` | Required `[[credential_refs]]` names | Auth provider |
|--------|------------|--------------------------------------|---------------|
| `armis` | `token_exchange` | `secret_key` | `DeclarativeHttpAuthProvider(TokenExchange)` — passes `"secret_key"` |
| `claroty` | `bearer_static` | `bearer_token` | `BearerStaticCredentialAuthProvider` — passes `"bearer_token"` |
| `cyberint-alerts` | `cookie_roundtrip` | `access_token` | `StaticCookieAuthProvider` — passes `"access_token"` |
| `cyberint-assets` | `cookie_roundtrip` | `access_token` | `StaticCookieAuthProvider` — passes `"access_token"` |
| `crowdstrike` | `oauth2_client_credentials` | `client_id`, `client_secret` | `DeclarativeHttpAuthProvider(Oauth2ClientCredentials)` — resolves both |

TOML declaration format (example for armis):

```toml
[[credential_refs]]
name = "secret_key"
description = "Armis long-lived secret key exchanged for access token via token_exchange"
```

CrowdStrike requires two `[[credential_refs]]` blocks:

```toml
[[credential_refs]]
name = "client_id"
description = "CrowdStrike OAuth2 client ID"

[[credential_refs]]
name = "client_secret"
description = "CrowdStrike OAuth2 client secret"
```

---

## Boot-Step-5 Probe Alignment (`KeyringCredentialProbe`) — v1.8 (F-P14-CRIT-001)

`KeyringCredentialProbe::probe(sensor_id, ref_name, org_registry)` MUST apply the full
three-tier probe order specified below. The prior v1.3–v1.7 clause prescribed the
**legacy non-org-keyed keyring format** (`{sensor_id}/{ref_name}`) as the Tier-3 probe
key. This was a defect: `prism credential set` (S-DEMO-003) writes credentials via
`CredentialStoreOrgId::set_by_org` under the OrgId-keyed namespace
`{org_id_uuid}/{sensor}/{name}` (ADR-034 §D3). A probe that checks only the legacy
key will NEVER find credentials written by `prism credential set`, making the
keyring-only boot path unbootable (F-P14-CRIT-001). Per Source-of-Truth Precedence
(CLAUDE.md §1), ADR-034 §D3/§D5 supersede the prior legacy-only probe clause.

### Precedence Order (canonical as of v1.8)

```
Tier 1/2 env-var wildcard  (highest — checked first)
  → hit on any org: probe succeeds
  → miss all orgs: fall through to Tier 3a

Tier 3a: OrgId-keyed keyring  (canonical — PRIMARY keyring probe)
  → for each registered org, attempt get_by_org(org_id, sensor_id, ref_name)
  → hit on any org: probe succeeds
  → miss all orgs: fall through to Tier 3b (legacy fallback)

Tier 3b: legacy non-org-keyed keyring  (fallback for pre-migration credentials)
  → attempt keyring::Entry::new("prism", "{sensor_id}/{ref_name}").get_password()
  → hit: probe succeeds
  → miss: fall through to not-found

Not found: Err(BootError::CredentialRefInvalid)
```

### Tier-by-Tier Specification

**Tier 1/2 wildcard (unchanged from v1.3):**

Iterate all registered org slugs from `OrgRegistry`. For each org slug, compute
`PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` (Tier 2) and `..._{REF}_FILE` (Tier 1).
If ANY org has the env var set (non-empty for Tier 2; file exists for Tier 1),
the probe succeeds (`Ok(None)`). Rationale: TYPE specs are shared across orgs; at
least one org resolved means the sensor is functional for that org. If NO org resolves,
fall through to Tier 3a.

**Tier 3a — OrgId-keyed keyring probe (PRIMARY — new in v1.8):**

For each registered org in `OrgRegistry`:
1. Resolve `OrgId` for the org slug: `org_registry.resolve(org_slug)` → `Option<OrgId>`.
   If the slug has no OrgId (orphan registry entry), skip this org.
2. Call `keyring_store.get_by_org(&org_id, sensor_id, ref_name).await` — this uses the
   OrgId-keyed namespace `{org_id_uuid}/{sensor_id}/{ref_name}` matching the
   `set_by_org` write path (ADR-034 §D3; `namespace_key_by_org_id`).
3. If `Ok(Some(_))` for any org: discard the value (AD-017), return `Ok(None)` — probe
   succeeds.
4. If `Ok(None)` / `Err(NoEntry)` for all orgs: fall through to Tier 3b.
5. If `Err(BackendUnavailable)` (keyring locked, `NoStorageAccess`, spawn panic): return
   `Err(BootError::CredentialPermissionDenied(...))` — hard error, do NOT fall through.
   Rationale mirrors ADR-034 §D4: a locked keyring at boot indicates misconfiguration;
   silently falling through would hide the operator error.

**Tier 3b — legacy non-org-keyed keyring probe (FALLBACK for pre-migration credentials):**

Attempt `keyring::Entry::new("prism", "{sensor_id}/{ref_name}").get_password()`.

**Rationale for retaining the fallback:** Credentials written by `CredentialStore::set`
(the legacy slug-keyed write path, `namespace_key` form) used the account format
`{sensor_id}/{ref_name}` (without an org component). Operators who wrote credentials
before the S-DEMO-003 migration (ADR-034) would be broken without this fallback.
Retaining it as a secondary fallback (AFTER OrgId-keyed attempt) preserves backward
compatibility while making the OrgId-keyed path the canonical production route.

**Exact legacy key:** `"{sensor_id}/{ref_name}"` — no org component. (The boot.rs doc
comment at the probe implementation may incorrectly state `{org_slug}/{sensor_id}/{ref_name}`;
this BC is the authoritative specification. The implementer MUST align the code and its
doc comment to match the exact key `"{sensor_id}/{ref_name}"`.)

If `Ok(_)`: discard value (AD-017), return `Ok(None)`.
If `Err(NoEntry)`: fall through to not-found.
If `Err(backend error)`: return `Err(BootError::CredentialPermissionDenied(...))` — hard error.

**Not found — all tiers exhausted:**

Return `Err(BootError::CredentialRefInvalid)` citing all three lookup paths so the
operator knows the full remediation surface.

### OrgRegistry and KeyringStore Threading

`KeyringCredentialProbe` requires access to both `OrgRegistry` (for Tier 3a OrgId
resolution) and `Arc<dyn CredentialStoreOrgId>` (for `get_by_org` calls). The probe
struct MUST hold these as injected fields (DI pattern, ADR-022 §C):

```rust
pub struct KeyringCredentialProbe {
    keyring: Arc<dyn CredentialStoreOrgId>,   // NEW in v1.8 — for Tier 3a OrgId probe
}
```

`org_registry` is already threaded into the probe via the `probe(...)` method signature
(unchanged from v1.3). The `keyring` field is injected at construction time.

`step5_init_credential_store_with_probe` already receives `org_registry`. The
`KeyringCredentialProbe` is constructed with the same `Arc<KeyringBackend>` instance
that `BootContext.credential_store_org_id` holds (ADR-034 §D5 — same instance,
no state duplication). The production call site in `step5_init_credential_store`:

```rust
step5_init_credential_store_with_probe(
    config,
    config_manager,
    org_registry,
    &KeyringCredentialProbe { keyring: Arc::clone(&keyring_backend) },  // NEW — Tier 3a
).await
```

`CredentialRefProbe::probe` method signature was converted to `async` (via
`#[async_trait]`) as of the pass-14 implementation (commit 0941c0e0) to support
the Tier-3a `get_by_org` await. This IS a method-signature change: all five
implementations (production `KeyringCredentialProbe` and four test doubles in
`tests/bc_2_03_013_credential_init.rs` and `tests/vp153_rule_c_shaped_probe.rs`)
were required to adopt `#[async_trait]` + `async fn probe(...)`, and all call sites
required `.await`. The `org_registry: &OrgRegistry` parameter was already present
from v1.3 (that part is unchanged); the async conversion is the only new signature
element. The async change is correct and necessary: a synchronous `probe` cannot
`.await get_by_org`.

### Error message when not found

```
Credential ref '{ref_name}' for sensor '{sensor_id}' not found in:
  - any client-scoped env var (PRISM_CLIENTS_{ID}_SENSORS_{SENSOR_UPPER}_{REF_UPPER}
    for any registered org),
  - OrgId-keyed OS keyring ({org_id_uuid}/{sensor_id}/{ref_name} for any registered org
    — written via: prism credential set --sensor {sensor_id} --name {ref_name}),
  - legacy keyring key ({sensor_id}/{ref_name}).
To configure (recommended): prism credential set --sensor {sensor_id} --name {ref_name}
To configure (env var): set PRISM_CLIENTS_<ORG_SLUG_UPPER>_SENSORS_{SENSOR_UPPER}_{REF_UPPER}=<value>
(BC-2.06.003 v1.9, BC-2.03.013 TV-03-013-003)
```

---

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Credential not found in any resolution tier | Error citing Tier 2 env var `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` and keyring key format |
| `PrismError::InvalidInput` | `credential_ref` name contains characters outside `[a-zA-Z0-9_\-\.]+` | Error: "Invalid credential reference '{ref}': must match [a-zA-Z0-9_\\-\\.]+" |
| `PrismError::Credential` | Tier 1 `_FILE` env var set but file non-existent or unreadable | Error: "Credential file '{path}' referenced by `...{REF}_FILE` not found or unreadable"; do NOT fall through to Tier 2 — the explicit `_FILE` reference is a misconfiguration |
| `BootError::CredentialRefInvalid` | Step 5 probe: ref absent from all tiers (Tier 1/2 env, Tier 3a OrgId keyring, Tier 3b legacy keyring) for all orgs | Boot aborts exit 2; error cites all three lookup paths (per-client env var, OrgId-keyed keyring key `{org_id_uuid}/{sensor}/{name}`, legacy key `{sensor}/{name}`) |
| `BootError::CredentialPermissionDenied` | Step 5 probe Tier 3a or 3b: keyring backend error (locked, `NoStorageAccess`, `NoKeyringService`, spawn panic) | Boot aborts exit 2; hard error — does NOT fall through; cites E-CRED-008 with backend reason |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-011 | OS keyring locked at startup | `PrismError::Credential` with suggestion to unlock keychain or use env var |
| EC-06-003 | Tier 1 `_FILE` env var set; file content has trailing newline | File content trimmed of leading/trailing whitespace before use as credential value |
| EC-06-004 | Two orgs registered; only one has Tier 2 env var set | Step 5 probe succeeds (at least one org resolves the ref); at query time, the org lacking the env var returns `CredentialResolutionError::NotFound` |
| EC-06-005 | CrowdStrike: `client_id` present but `client_secret` absent | Step 5 probe fails on `client_secret` ref; boot aborts exit 2 |
| EC-06-006 | Boot probe Tier 3a: credential written by `prism credential set` (OrgId-keyed via `set_by_org`); no env vars set; `KeyringCredentialProbe` has the `Arc<dyn CredentialStoreOrgId>` field wired. | Tier 1/2 miss → Tier 3a: `get_by_org(org_id, sensor, ref)` returns `Ok(Some(_))` → probe returns `Ok(None)` → boot step 5 succeeds. This is the canonical keyring-only boot path for the S-DEMO-003 demo setup. (Closes F-P14-CRIT-001.) |
| EC-06-007 | Boot probe Tier 3a miss + Tier 3b hit: credential written via legacy `CredentialStore::set` (slug-keyed, key `"{sensor}/{ref}"`); no env vars; no OrgId-keyed entry. | Tier 1/2 miss → Tier 3a `get_by_org` returns `Ok(None)` for all orgs → Tier 3b: `keyring::Entry::new("prism", "{sensor}/{ref}").get_password()` returns `Ok(_)` → probe returns `Ok(None)` → boot step 5 succeeds. Backward-compatibility path for pre-migration credentials. |
| EC-06-008 | Boot probe: keyring backend unavailable (locked) during Tier 3a `get_by_org` call. | Hard error: `Err(BootError::CredentialPermissionDenied(...))` — does NOT fall through to Tier 3b. Operator must unlock keyring or use Tier 1/2 env vars. |

---

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.003.

### Query-Time Resolution (`resolve_credential`)

| Scenario | org_slug | sensor | ref | Setup / Env | Expected |
|----------|----------|--------|-----|------------|---------|
| Tier 1 file | `acme` | `claroty` | `bearer_token` | `PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN_FILE=/run/secrets/tok` | File contents (trimmed) as `SecretString` |
| Tier 2 direct | `acme` | `claroty` | `bearer_token` | `PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN=abc123` | Env value as `SecretString` |
| Tier 2 hyphen-slug | `demo-org-a` | `claroty` | `bearer_token` | `PRISM_CLIENTS_DEMO_ORG_A_SENSORS_CLAROTY_BEARER_TOKEN=tok` | Env value; slug hyphens → underscores |
| **Tier 3 keyring (OrgId-keyed write→resolution)** | `acme` (org_id: `f47ac10b-58cc-4372-a567-0e02b2c3d479`) | `claroty` | `bearer_token` | `KeyringBackend::set_by_org(org_id, "claroty", "bearer_token", "secret-value")` via `CredentialStoreOrgId`; no env var set; keyring entry at OrgId-keyed key `f47ac10b-58cc-4372-a567-0e02b2c3d479/claroty/bearer_token` | `resolve_credential("acme", "claroty", "bearer_token", Some(&org_id), Some(&keyring))` → `Ok(SecretString("secret-value"))` + audit "keyring". (RG-034-001 per ADR-034.) |
| Tier 3 keyring miss → Tier 4 | `acme` | `claroty` | `bearer_token` | No env var; keyring has no entry for the OrgId-keyed key; CRUD store empty | `CredentialResolutionError::NotFound` — Tier 3 miss falls through to Tier 4, Tier 4 miss → `NotFound`. (RG-034-002 per ADR-034.) |
| Tier 3 None org_id → Tier 4 | `acme` | `claroty` | `bearer_token` | No env var; `org_id: None` passed to `resolve_credential`; CRUD store empty | Tier 3 skipped silently; falls to Tier 4 → `CredentialResolutionError::NotFound` |
| Tier 3 keyring backend error → hard error | `acme` | `claroty` | `bearer_token` | No env var; keyring returns `Err(NoStorageAccess)` | `CredentialResolutionError::BackendUnavailable { detail: "E-CRED-008: OS keyring unavailable: NoStorageAccess" }` — does NOT fall through to Tier 4 (ADR-034 §D4; E-CRED-008 is the canonical keyring-unavailable code per ADR-035 §D2) |
| Not found | `acme` | `claroty` | `bearer_token` | No env var, no keyring | `CredentialResolutionError::NotFound` with Tier 2 env var name in message |
| Tier 1 precedence | `acme` | `claroty` | `bearer_token` | Both `_FILE` and direct env var set | Tier 1 (file) wins; Tier 2 ignored |
| Invalid ref name | any | any | `my key!` | — | `PrismError::InvalidInput`: must match pattern |
| CrowdStrike `client_id` | `acme` | `crowdstrike` | `client_id` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_ID=id-value` | Env value as `SecretString` |
| CrowdStrike `client_secret` | `acme` | `crowdstrike` | `client_secret` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_SECRET=secret-value` | Env value as `SecretString` |

### Boot-Step-5 Probe (`KeyringCredentialProbe::probe`) — v1.8 (F-P14-CRIT-001)

These vectors specifically test the boot probe, which must discover credentials written by `prism credential set` (OrgId-keyed namespace). The probe uses `Arc<dyn CredentialStoreOrgId>` injected at construction — the same `KeyringBackend` instance as `BootContext.credential_store_org_id`.

| Scenario | org_registry | sensor | ref | Keyring State | Expected probe result |
|----------|-------------|--------|-----|---------------|----------------------|
| **TV-BOOT-P-001 — OrgId-keyed entry found (canonical path, EC-06-006)** | `{slug="demo-org", org_id="f47ac10b-…"}` | `claroty` | `bearer_token` | `set_by_org(org_id, "claroty", "bearer_token", _)` called; OrgId-keyed key `f47ac10b-…/claroty/bearer_token` exists. No env vars. No legacy key. | `Ok(None)` — probe succeeds; Tier 3a hit. Credential value discarded (AD-017). Boot step 5 passes. |
| **TV-BOOT-P-002 — Legacy key found, no OrgId entry (EC-06-007, backward compat)** | `{slug="demo-org", org_id="f47ac10b-…"}` | `claroty` | `bearer_token` | Only legacy key `claroty/bearer_token` exists in keyring. No OrgId-keyed key. No env vars. | Tier 1/2 miss → Tier 3a `get_by_org` → `Ok(None)` for all orgs → Tier 3b: `keyring::Entry::new("prism", "claroty/bearer_token")` → `Ok(_)` → `Ok(None)` — probe succeeds. |
| **TV-BOOT-P-003 — All tiers miss → CredentialRefInvalid** | `{slug="demo-org", org_id="f47ac10b-…"}` | `claroty` | `bearer_token` | No env vars. No OrgId-keyed entry. No legacy entry. | `Err(BootError::CredentialRefInvalid(...))` — error message cites env var format, OrgId-keyed keyring format, and legacy keyring format. |
| **TV-BOOT-P-004 — Keyring backend error during Tier 3a (EC-06-008)** | `{slug="demo-org", org_id="f47ac10b-…"}` | `claroty` | `bearer_token` | `get_by_org` returns `Err(NoStorageAccess)`. No env vars. | `Err(BootError::CredentialPermissionDenied(...))` — hard error; does NOT fall through to Tier 3b. |
| **TV-BOOT-P-005 — Tier 2 env var hit (existing behavior, unchanged)** | `{slug="demo-org", org_id="f47ac10b-…"}` | `claroty` | `bearer_token` | No keyring entries. `PRISM_CLIENTS_DEMO_ORG_SENSORS_CLAROTY_BEARER_TOKEN=abc123` set. | `Ok(None)` — Tier 2 hit; probe succeeds without touching keyring. |

---

## Verification Properties

No VPs in VP-INDEX directly verify credential reference resolution. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Credential management operations") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Credential management operations") per capabilities.md §CAP-009 — this BC describes credential reference resolution (the per-client priority chain for resolving sensor credential refs to `SecretString` values at query time), which is exactly what CAP-009 defines. |
| L2 Invariants | DI-002, DI-014 |
| Priority | P0 |
| Implementing Stories | S-DEMO-002 — `resolve_credential` per-client env-var format alignment + `KeyringCredentialProbe` org-aware probe redesign; S-DEMO-003 — Tier-3 OS-keyring resolution implementation + OrgId-keyed write reconciliation (ADR-034) |
| ADR | ADR-032-per-client-credential-env-var-convention; ADR-034-tier3-keyring-resolution-org-id-threading (Tier-3 implementation decision, source of truth for D1–D5; accepted 2026-06-06; §D4 error code amended from E-CRED-005→E-CRED-008 by ADR-035 §D5); ADR-035-e-cred-namespace-reconciliation (canonical E-CRED-001..010 namespace; E-CRED-008 is the authoritative keyring-unavailable code; accepted 2026-06-07) |
| Related BCs | BC-2.03.006 (query-time resolution), BC-3.2.002 (OrgId-keyed keyring), BC-2.03.013 (boot step 5), BC-2.03.007 (E-CRED-008 no-credential-leak invariant — keyring-backend error detail is system message, not credential value) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.12 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | ADR-053 D2+D3 + ADR-054 D1 amendment: §Per-Sensor table: armis auth_type `bearer_static`→`token_exchange`, credential ref `bearer_token`→`secret_key`, provider `BearerStaticCredentialAuthProvider`→`DeclarativeHttpAuthProvider(TokenExchange)`; cyberint row split into `cyberint-alerts` + `cyberint-assets` (ADR-053 D3 dual-surface), both `cookie_roundtrip` + `access_token` + `StaticCookieAuthProvider`; crowdstrike provider `crowdstrike-oauth2 WASM plugin`→`DeclarativeHttpAuthProvider(Oauth2ClientCredentials)` (ADR-054 D1). §TOML example: armis `bearer_token`→`secret_key` with updated description. §Env-Var Worked examples: armis row updated (ARMIS_BEARER_TOKEN→ARMIS_SECRET_KEY); cyberint row split into cyberint-alerts and cyberint-assets rows (CYBERINT_ALERTS_ACCESS_TOKEN + CYBERINT_ASSETS_ACCESS_TOKEN). §Query-Time Resolution (9 rows): armis+ARMIS→claroty+CLAROTY (re-pointed to preserve four-tier resolution chain examples; claroty still uses bearer_token credential ref — no armis bearer_token reference survives). §Boot-Step-5 Probe TV-BOOT-P-001..005: armis+ARMIS→claroty+CLAROTY throughout. modified date 2026-07-22. |
| 1.11 | S-DEMO-003-merged-PR-176 | 2026-06-08 | state-manager | **POL-14 auto-promotion draft→active (D-1055).** S-DEMO-003 squash-merged PR #176 into develop@a42e3eaf. `status: draft → active` (synced with `lifecycle_status: active` which was already correct per ADR-025 ground truth; the `status:` inconsistency was an ADR-025 drift introduced before the D-1047 durable snapshot). E-CRED-008 emitter + boot Tier-3a `KeyringCredentialProbe::probe` (OrgId-keyed via `get_by_org` per ADR-034 §D3) are now in merged production code. BC H1 title UNCHANGED (POL-7). BC v1.10 → v1.11. |
| 1.10 | S-DEMO-003-F-P16-MED-001 | 2026-06-07 | product-owner | **F-P16-MED-001 — cyberint auth_type drift api_key → cookie_roundtrip (D-747 LOCKED).** §Per-Sensor `[[credential_refs]]` Declarations table, cyberint row: `auth_type` column corrected from `api_key` → `cookie_roundtrip`. Root cause: `api_key` is the credential-ref NAME for cyberint (correct in the `Required [[credential_refs]] names` column), not the auth_type; the auth_type cell was incorrectly populated with the credential name. Canonical source: `crates/prism-sensors/specs/cyberint.sensor.toml:26` declares `auth_type = "cookie_roundtrip"` (D-747 LOCKED; the legacy `bearer_static` label for cyberint was a known latent label bug — `cookie_roundtrip` is the locked canonical value per ADR-028 §D2 / ADR-031 §D3-a). Corroboration: provider column `StaticCookieAuthProvider` (cookie-based) and story Open Question 3 both confirm `cookie_roundtrip`. The `Required [[credential_refs]] names` column (`api_key`), provider column (`StaticCookieAuthProvider`), and all sibling rows (armis `bearer_static`, claroty `bearer_static`, crowdstrike `oauth2_client_credentials`) are unchanged and correct. H1 title UNCHANGED (POL-7). Status remains draft (POL-14). |
| 1.9 | S-DEMO-003-F-P15-HIGH-002 | 2026-06-07 | product-owner | **F-P15-HIGH-002 — Async signature correction.** Corrected §OrgRegistry and KeyringStore Threading: removed false claim that "`CredentialRefProbe` trait and `CredentialRefProbe::probe` method signature are UNCHANGED from v1.3" and "no signature blast radius." The pass-14 implementation (commit 0941c0e0) converted `probe` to `async` via `#[async_trait]` — this IS a method-signature change. All 5 impls (production `KeyringCredentialProbe` + 4 test doubles in `tests/bc_2_03_013_credential_init.rs` and `tests/vp153_rule_c_shaped_probe.rs`) were required to adopt `#[async_trait]` + `async fn probe`, and all call sites required `.await`. The `org_registry: &OrgRegistry` parameter was already present from v1.3 (unchanged). The async conversion is correct and necessary: a synchronous `probe` cannot `.await get_by_org`. The construction prescription (probe and store share ONE `Arc<KeyringBackend>` per ADR-034 §D5) and the `Arc::clone(&keyring_backend)` sharing example in the code block are unchanged and correct. BC H1 title UNCHANGED (POL-7). Status remains draft (POL-14). |
| 1.8 | S-DEMO-003-F-P14-CRIT-001 | 2026-06-07 | product-owner | **F-P14-CRIT-001 — Boot-step-5 probe OrgId-keyed reconciliation.** Full rewrite of §Boot-Step-5 Probe Alignment to fix the defect where the Tier-3 probe used the legacy non-org-keyed key `{sensor_id}/{ref_name}` while `prism credential set` writes via `set_by_org` to the OrgId-keyed key `{org_id_uuid}/{sensor_id}/{ref_name}`. Per Source-of-Truth Precedence (ADR-034 §D3/§D5 supersedes the prior BC clause). Changes: (1) §Boot-Step-5 Probe Alignment fully rewritten: three-tier probe order (Tier 1/2 env wildcard → Tier 3a OrgId-keyed keyring PRIMARY → Tier 3b legacy keyring FALLBACK); (2) Decision documented: legacy fallback RETAINED for backward compatibility with pre-migration credentials, but OrgId-keyed is primary/canonical; (3) `KeyringCredentialProbe` struct gains `keyring: Arc<dyn CredentialStoreOrgId>` field for Tier 3a `get_by_org` calls; (4) Exact legacy key form specified as `"{sensor_id}/{ref_name}"` (no org component) — corrects the doc inconsistency in boot.rs where the doc comment claims `{org_slug}/{sensor_id}/{ref_name}` but code uses `{sensor_id}/{ref_name}`; (5) Error message updated to cite all three lookup paths; (6) Error Cases table: `BootError::CredentialRefInvalid` updated to cite all three paths; added `BootError::CredentialPermissionDenied` for keyring backend errors at Tier 3a/3b; (7) Edge Cases: added EC-06-006 (OrgId-keyed probe success), EC-06-007 (legacy fallback success), EC-06-008 (keyring backend hard error); (8) Canonical Test Vectors: added §Boot-Step-5 Probe section with TV-BOOT-P-001..005; (9) `modified` frontmatter updated to 2026-06-07. |
| 1.7 | S-MAINT-ECRED-TAXONOMY-SYNC-001 | 2026-06-07 | product-owner | F-P17-MED-001: de-pinned stale VP-INDEX version reference in §Verification Properties per TD-VSDD-091; S-MAINT-ECRED-TAXONOMY-SYNC-001. |
| 1.6 | S-MAINT-ECRED-TAXONOMY-SYNC-001 | 2026-06-07 | product-owner | **Wrong-section ADR anchor fix (F-P11-HIGH-001).** Postconditions Tier-3 error-semantics table, backend-error row: `ADR-034 §D5 amended by ADR-035` → `ADR-034 §D4 amended by ADR-035 §D5`. §D4 is "Error Semantics — Keyring Backend Error is a Hard Error" (the correct target); §D5 is "Boot Path Wiring — PrismCredentialResolver Construction" (unrelated). The three pre-existing §D4 cites (lines ~116/~285/~306) were already correct and are unchanged. No content semantics altered. |
| 1.5 | S-MAINT-ECRED-TAXONOMY-SYNC-001 | 2026-06-07 | product-owner | **E-CRED-005 → E-CRED-008 keyring-unavailable code update per ADR-035 §D2 + §Blast-Radius.** Changes: (1) Postconditions Tier-3 table: `BackendUnavailable { detail: "E-CRED-005: OS keyring unavailable: {reason}" }` → `E-CRED-008` — collision resolved: E-CRED-005 was simultaneously assigned to `PrismError::CredentialEncryptionError` (prism-core) and to the keyring-unavailable Tier-3 path (ADR-034 §D4 + this BC). ADR-035 assigns `CredentialEncryptionError` to E-CRED-006 and `KeyringBackendUnavailable` to E-CRED-008. (2) Postconditions invariant note: `E-CRED-005 detail is a system error message` → `E-CRED-008 detail is a system error message` (AD-017 credential-opacity note preserved). (3) Canonical Test Vectors: "Tier 3 keyring backend error → hard error" row output → `E-CRED-008`. (4) Traceability ADR column: added ADR-035 as normative authority; noted ADR-034 §D4 is amended by ADR-035 §D5 for the error code only (all other ADR-034 decisions remain in effect). (5) Traceability Related BCs: BC-2.03.007 parenthetical updated from `E-CRED-005` to `E-CRED-008`. BC H1 title UNCHANGED per POL-7. Draft status UNCHANGED — BC remains anchored to S-DEMO-003. |
| 1.4 | S-DEMO-003-spec-restart | 2026-06-06 | product-owner | **Human Option-A decision (ADR-034):** Tier-3 OS-keyring credential resolution is now IMPLEMENTED per ADR-034 (accepted 2026-06-06). Changes: (1) Tier 3 row in env-var-formats table updated: marked IMPLEMENTED, noted OrgId-keyed namespace (`{org_id_uuid}/{sensor_id}/{ref_name}` via `namespace_key_by_org_id`) is canonical, and that `CredentialStoreOrgId::set_by_org` is required for writes (legacy slug-keyed `CredentialStore::set` path does not feed Tier 3). (2) Postconditions Tier 3 entry updated with ADR-034 §D4 error-semantics table: `org_id/keyring None` → silent fall-through; `get_by_org Ok(None)` → fall-through; `get_by_org Err(...)` → hard `BackendUnavailable` with E-CRED-005 (next free code; ADR-034 names E-CRED-003 but that code was already allocated to "Credential decryption failed" in error-taxonomy.md — collision noted; E-CRED-005 is used in this BC; see ADR-034 Contradiction section). (3) Canonical Test Vectors: fixed existing Tier 3 TV from stale legacy slug-keyed format to correct OrgId-keyed format; added TV for `set_by_org` write→resolution end-to-end connectivity (mirrors RG-034-001); added TV for Tier-3 miss→Tier-4 fall-through (mirrors RG-034-002); added TV for `org_id: None` → silent skip; added TV for backend error → hard `BackendUnavailable`. (4) Traceability: added ADR-034 as normative authority; added S-DEMO-003 as implementing story; added Capability Anchor Justification row per DF-021. (5) `modified` frontmatter updated to 2026-06-06. |
| 1.3 | S-DEMO-002-option-A | 2026-06-03 | architect | **Human Option-A decision**: adopt per-client env-var convention as canonical multi-tenant credential convention. Full content rewrite: (1) Defined slug-to-SCREAMING-SNAKE transform for `{ID}` with worked examples. (2) Specified all four resolution tiers with exact env-var name derivation. (3) Defined per-sensor `[[credential_refs]]` names for all 4 built-in sensors. (4) Specified org-aware boot-step-5 probe design including OrgRegistry threading and revised `CredentialRefProbe` trait signature. (5) **Retraction of v1.2 alignment claim**: the v1.2 entry claimed code "mirrors the chain specified by this BC" — this was false. The prior `resolve_credential` used a global `{SENSOR}_{REF}` format (e.g. `ARMIS_BEARER_TOKEN`) that is NOT client-scoped and violates DI-002. The implementer must bring `resolve_credential` and `KeyringCredentialProbe::probe` into alignment with this BC. Code is NOT the spec; the spec wins (CLAUDE.md Standing Rule 7). |
| 1.2 | S-DEMO-002-traceability | 2026-06-03 | product-owner | **RETRACTED** (see v1.3): incorrectly claimed code mirrors per-client env-var chain. No content changes in that burst; only false traceability claim added. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
