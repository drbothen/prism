---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: 2026-06-03
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
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
| 3 | Keyring: key `{org_id_uuid}/{sensor_id}/{ref_name}` via `CredentialStoreOrgId::get_by_org` | OS keyring, OrgId-keyed format (BC-3.2.002 / `namespace_key_by_org_id`) |
| 4 (lowest) | CRUD store: `credential_status(client_id, sensor_id, credential_name)` → backend lookup | Programmatically configured source |

### Worked examples

| org_slug | sensor_id | ref_name | Tier 2 env var | Tier 1 env var |
|----------|-----------|----------|----------------|----------------|
| `demo-org-a` | `armis` | `bearer_token` | `PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_BEARER_TOKEN` | `PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_BEARER_TOKEN_FILE` |
| `acme` | `claroty` | `bearer_token` | `PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN` | `PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN_FILE` |
| `acme` | `cyberint` | `api_key` | `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_API_KEY` | `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_API_KEY_FILE` |
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
  - **Tier 1:** `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE` — file path; contents trimmed
  - **Tier 2:** `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` — direct env-var value
  - **Tier 3:** OS keyring via `CredentialStoreOrgId::get_by_org(org_id, sensor_id, ref_name)` — key `{org_id_uuid}/{sensor_id}/{ref_name}`
  - **Tier 4:** CRUD store `credential_status` → backend source lookup
- The resolved credential is available as a `SecretString` that is never logged or serialized
- Resolution is audit-logged (client, sensor, ref name only — never the value)

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
| `armis` | `bearer_static` | `bearer_token` | `BearerStaticCredentialAuthProvider` — passes `"bearer_token"` |
| `claroty` | `bearer_static` | `bearer_token` | `BearerStaticCredentialAuthProvider` — passes `"bearer_token"` |
| `cyberint` | `api_key` | `api_key` | `StaticCookieAuthProvider` — passes `"api_key"` |
| `crowdstrike` | `oauth2_client_credentials` | `client_id`, `client_secret` | `crowdstrike-oauth2` WASM plugin — resolves both |

TOML declaration format (example for armis):

```toml
[[credential_refs]]
name = "bearer_token"
description = "Armis API bearer token"
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

## Boot-Step-5 Probe Alignment (`KeyringCredentialProbe`)

`KeyringCredentialProbe::probe(sensor_id, ref_name)` MUST apply the full four-tier
per-client priority chain, not the global `{SENSOR}_{REF}` format that the prior
implementation incorrectly used.

### Org-Awareness Design

Step 5 iterates `snapshot.sensor_specs` (TYPE specs, no org context). The probe
must not require an org at this point because TYPE specs are shared across all orgs.

**Specified design:** Probe checks Tier 1 and Tier 2 using a wildcard scan.

For each `(sensor_id, ref_name)` pair, step 5 probes in this order:

1. **Tier 1/2 wildcard:** iterate all registered orgs from `OrgRegistry`. For each
   org slug, compute the per-client Tier 2 env var
   `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` (and the `_FILE` variant). If **any**
   org has the env var set (non-empty for Tier 2; file exists for Tier 1), the probe
   succeeds for this ref. Rationale: at boot time the TYPE spec is shared; if at least
   one org can resolve the ref, the sensor is functional for that org. If NO org resolves
   the ref at step 5, fall through to Tier 3.

2. **Tier 3 (keyring):** attempt `keyring::Entry::new("prism", "{sensor_id}/{ref_name}")`.
   **Note:** The keyring probe uses the **legacy OrgSlug-keyed format**
   `{org_slug}/{sensor_id}/{ref_name}` (via `namespace_key`) for backwards-compatibility
   with credentials stored before BC-3.2.002 OrgId migration. Future stories may migrate
   this to OrgId-keyed probe; for now the boot probe checks the legacy format only.
   If found in keyring, probe succeeds.

3. **Not found in any tier:** return `Err(BootError::CredentialRefInvalid)` with a
   message citing BOTH the per-client Tier 2 env var format AND the keyring key format,
   so operators have actionable remediation options.

### OrgRegistry Threading

Step 5 (`step5_init_credential_store_with_probe`) already receives `config_manager` but
not `OrgRegistry`. The implementer MUST thread `org_registry` (produced at step 3 and
stored in `BootContext`) into `step5_init_credential_store_with_probe` so the probe can
iterate org slugs for Tier 1/2 checks. Signature change:

```rust
pub async fn step5_init_credential_store_with_probe(
    config: &PrismConfig,
    config_manager: &Arc<ArcSwap<ConfigManager>>,
    org_registry: &Arc<OrgRegistry>,   // NEW — thread from BootContext
    probe: &dyn CredentialRefProbe,
) -> Result<Arc<dyn CredentialStore>, BootError>
```

The `KeyringCredentialProbe::probe` signature gains `org_registry`:

```rust
fn probe(
    &self,
    sensor_id: &str,
    ref_name: &str,
    org_registry: &OrgRegistry,        // NEW
) -> Result<Option<String>, BootError>
```

`CredentialRefProbe` trait updated accordingly. All test doubles implement the new
signature.

### Error message when not found

```
Credential ref '{ref_name}' for sensor '{sensor_id}' not found in any client-scoped
env var (PRISM_CLIENTS_{ID}_SENSORS_{SENSOR_UPPER}_{REF_UPPER} for any registered org),
nor in the OS keyring ({sensor_id}/{ref_name}).
To configure:
  - Set PRISM_CLIENTS_<ORG_SLUG_UPPER>_SENSORS_{SENSOR_UPPER}_{REF_UPPER}=<value>
    for each org that uses this sensor, OR
  - Register in keyring: prism credential set {sensor_id} {ref_name}
(BC-2.06.003, BC-2.03.013 TV-03-013-003)
```

---

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Credential not found in any resolution tier | Error citing Tier 2 env var `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` and keyring key format |
| `PrismError::InvalidInput` | `credential_ref` name contains characters outside `[a-zA-Z0-9_\-\.]+` | Error: "Invalid credential reference '{ref}': must match [a-zA-Z0-9_\\-\\.]+" |
| `PrismError::Credential` | Tier 1 `_FILE` env var set but file non-existent or unreadable | Error: "Credential file '{path}' referenced by `...{REF}_FILE` not found or unreadable"; do NOT fall through to Tier 2 — the explicit `_FILE` reference is a misconfiguration |
| `BootError::CredentialRefInvalid` | Step 5 probe: ref absent from all tiers for all orgs | Boot aborts exit 2; error cites both per-client env var format and keyring key |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-011 | OS keyring locked at startup | `PrismError::Credential` with suggestion to unlock keychain or use env var |
| EC-06-003 | Tier 1 `_FILE` env var set; file content has trailing newline | File content trimmed of leading/trailing whitespace before use as credential value |
| EC-06-004 | Two orgs registered; only one has Tier 2 env var set | Step 5 probe succeeds (at least one org resolves the ref); at query time, the org lacking the env var returns `CredentialResolutionError::NotFound` |
| EC-06-005 | CrowdStrike: `client_id` present but `client_secret` absent | Step 5 probe fails on `client_secret` ref; boot aborts exit 2 |

---

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.003.

| Scenario | org_slug | sensor | ref | Env var checked | Expected |
|----------|----------|--------|-----|-----------------|---------|
| Tier 1 file | `acme` | `armis` | `bearer_token` | `PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN_FILE=/run/secrets/tok` | File contents (trimmed) as `SecretString` |
| Tier 2 direct | `acme` | `armis` | `bearer_token` | `PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN=abc123` | Env value as `SecretString` |
| Tier 2 hyphen-slug | `demo-org-a` | `armis` | `bearer_token` | `PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_BEARER_TOKEN=tok` | Env value; slug hyphens → underscores |
| Tier 3 keyring | `acme` | `armis` | `bearer_token` | No env var; keyring entry at `acme/armis/bearer_token` | Keyring secret as `SecretString` |
| Not found | `acme` | `armis` | `bearer_token` | No env var, no keyring | `CredentialResolutionError::NotFound` with Tier 2 env var name in message |
| Tier 1 precedence | `acme` | `armis` | `bearer_token` | Both `_FILE` and direct env var set | Tier 1 (file) wins; Tier 2 ignored |
| Invalid ref name | any | any | `my key!` | — | `PrismError::InvalidInput`: must match pattern |
| CrowdStrike `client_id` | `acme` | `crowdstrike` | `client_id` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_ID=id-value` | Env value as `SecretString` |
| CrowdStrike `client_secret` | `acme` | `crowdstrike` | `client_secret` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_SECRET=secret-value` | Env value as `SecretString` |

---

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify credential reference resolution. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-002, DI-014 |
| Priority | P0 |
| Implementing Stories | S-DEMO-002 — `resolve_credential` per-client env-var format alignment + `KeyringCredentialProbe` org-aware probe redesign |
| ADR | ADR-032-per-client-credential-env-var-convention (new, authored alongside this v1.3) |
| Related BCs | BC-2.03.006 (query-time resolution), BC-3.2.002 (OrgId-keyed keyring), BC-2.03.013 (boot step 5) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | S-DEMO-002-option-A | 2026-06-03 | architect | **Human Option-A decision**: adopt per-client env-var convention as canonical multi-tenant credential convention. Full content rewrite: (1) Defined slug-to-SCREAMING-SNAKE transform for `{ID}` with worked examples. (2) Specified all four resolution tiers with exact env-var name derivation. (3) Defined per-sensor `[[credential_refs]]` names for all 4 built-in sensors. (4) Specified org-aware boot-step-5 probe design including OrgRegistry threading and revised `CredentialRefProbe` trait signature. (5) **Retraction of v1.2 alignment claim**: the v1.2 entry claimed code "mirrors the chain specified by this BC" — this was false. The prior `resolve_credential` used a global `{SENSOR}_{REF}` format (e.g. `ARMIS_BEARER_TOKEN`) that is NOT client-scoped and violates DI-002. The implementer must bring `resolve_credential` and `KeyringCredentialProbe::probe` into alignment with this BC. Code is NOT the spec; the spec wins (CLAUDE.md Standing Rule 7). |
| 1.2 | S-DEMO-002-traceability | 2026-06-03 | product-owner | **RETRACTED** (see v1.3): incorrectly claimed code mirrors per-client env-var chain. No content changes in that burst; only false traceability claim added. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
