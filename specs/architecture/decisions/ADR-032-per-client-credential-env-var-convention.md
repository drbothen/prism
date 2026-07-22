---
document_type: adr
adr_id: "ADR-032"
title: "Per-Client Credential Env-Var Convention (Multi-Tenant Correct)"
status: accepted
date: 2026-06-03
author: architect
decision_made_by: human (Option-A selection, 2026-06-03)
supersedes: null
superseded_by: null
subsystems_affected: ["SS-03", "SS-06", "SS-22"]
related_adrs: ["ADR-029", "ADR-022", "ADR-026", "ADR-053", "ADR-054"]
related_bcs: ["BC-2.06.003", "BC-2.03.006", "BC-3.2.002", "BC-2.03.013"]
version: "1.3"
modified: "2026-07-21"
---

# ADR-032: Per-Client Credential Env-Var Convention (Multi-Tenant Correct)

## Status

Accepted (human-selected Option A, 2026-06-03; current version per §Changelog top row).

**Partial amendment pending (ADR-053 acceptance, Wave-A):** ADR-053 §D2 amends the Armis credential-ref name in this ADR's §Implementer Blast Radius tables (`bearer_token` → `secret_key` via `auth_type = "token_exchange"`). ADR-053 §D3 amends the Cyberint credential-ref name (`api_key` → `access_token`) and deletes `cyberint.sensor.toml`, replacing it with a dual-surface split (`cyberint-alerts.sensor.toml` + `cyberint-assets.sensor.toml`). ADR-054 §D2 further amends CrowdStrike `oauth2_client_credentials` handling (native `DeclarativeHttpAuthProvider` replaces the plugin path). This ADR is **not** wholly superseded — the per-client env-var convention itself is unchanged; only the Armis and Cyberint credential-ref rows are affected. At-point annotations on the affected table rows are the reader-facing signal; ADR-053 D5 deferred rewrite manifest is the executing mechanism.

## Context

Prism is an MSSP platform that resolves sensor credentials per `(client_id, sensor_id,
credential_ref_name)` tuple. The credential isolation invariant (DI-002) requires that
credentials for client A can never be resolved in the context of client B.

As of the S-DEMO-002 story, `resolve_credential` computed env-var names as:

```
{SENSOR_UPPER}_{REF_UPPER}         (e.g. ARMIS_BEARER_TOKEN)
{SENSOR_UPPER}_{REF_UPPER}_FILE    (e.g. ARMIS_BEARER_TOKEN_FILE)
```

This format is **not client-scoped**. In a multi-tenant deployment with two orgs —
`acme` and `contoso` — both resolve the same env var `ARMIS_BEARER_TOKEN`, making
it impossible to configure per-org credentials via environment variables. This
directly violates DI-002 for env-var-sourced credentials.

**Issue F-SDEMO002-P-MED-001** surfaced this gap during adversarial review of
S-DEMO-002. Two options were presented to the human:

- **Option A:** Adopt the per-client env-var convention as canonical, bring code into
  alignment with BC-2.06.003.
- **Option B:** Retract the BC to match the global format and add a tech-debt entry
  to address multi-tenancy later.

The human selected **Option A**. This ADR records that decision and defines the
convention precisely so that all affected code sites can be brought into alignment.

## Decision

Prism adopts the **per-client env-var convention** as the canonical credential
environment variable format for all sensor credential resolution. The global
`{SENSOR}_{REF}` format is retired.

### Canonical format

```
PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}
PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE   (K8s secret-mount variant)
```

where:
- `{ID}` = org slug uppercased with hyphens → underscores
  (e.g. `demo-org-a` → `DEMO_ORG_A`, `acme-corp` → `ACME_CORP`)
- `{SENSOR}` = sensor_id uppercased with hyphens → underscores
- `{REF}` = credential ref name uppercased with hyphens → underscores

### Resolution tier order

1. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE` (highest priority; file exists and is readable)
2. `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` (direct env-var value)
3. OS keyring via `CredentialStoreOrgId::get_by_org` (OrgId-UUID key: `{org_id_uuid}/{sensor}/{ref}`)
4. CRUD store `credential_status` → backend source lookup (lowest priority)

### Boot-step-5 org-aware probe

`KeyringCredentialProbe::probe` is extended to accept `org_registry: &OrgRegistry`.
For each `(sensor_id, ref_name)` pair in `snapshot.sensor_specs`, step 5 iterates
all registered orgs (via `OrgRegistry::all_orgs()`) and checks whether ANY org has
Tier 1 or Tier 2 env var set. If at least one org resolves the ref, the probe passes.
If no org resolves the ref, the probe falls through to Tier 3 (keyring legacy format)
before failing.

This design is correct: TYPE specs are shared across orgs; per-org credential absence
at boot time is only an error if ALL orgs lack the ref.

## Rationale

The per-client env-var format satisfies DI-002 (credential isolation invariant): in a multi-tenant MSSP deployment, credentials for different orgs must be independently configurable without cross-contamination. The global `{SENSOR}_{REF}` format cannot satisfy this invariant for env-var-sourced credentials — there is no org discriminant in the variable name, so two orgs sharing a host would resolve the same env var.

This approach mirrors the pattern established by ADR-029 for per-client `base_url` env-vars, making the operator mental model uniform: every per-client config key starts with `PRISM_CLIENTS_{ID}_`.

The human selection of Option A over Option B (retract BC-2.06.003) reflects the correctness-first principle codified in the Canonical Principle: deferring DI-002 compliance for env-var credentials would leave a known invariant violation in the system indefinitely.

## Consequences

### Positive
- DI-002 is satisfied for env-var-sourced credentials: per-org env vars are namespaced
  and cannot cross-contaminate.
- K8s secret mounts can be scoped per-org: `PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN_FILE`.
- Consistent with ADR-029's per-client `base_url` env-var pattern
  (`PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_BASE_URL`), making operator mental model uniform.
- The `_FILE` tier supports K8s secret-mounted credentials without requiring keyring access.

### Negative / Migration cost
- Operators who configured the global format (`ARMIS_BEARER_TOKEN`) must migrate to
  `PRISM_CLIENTS_{ID}_SENSORS_ARMIS_BEARER_TOKEN` per org. This is a breaking
  change in the env-var API. Since Prism is pre-GA, no production migration is required
  — documentation and tooling are updated as part of S-DEMO-002 implementation.
- DTU E2E test env setters in `prism-bin/tests/helpers/mod.rs` and all landing tests
  must be updated to use the per-org format (see Implementer Blast Radius below).

### Neutral
- Keyring (Tier 3) uses the OrgId-UUID key scheme per BC-3.2.002 — no change there.
- CRUD store (Tier 4) is unchanged.

## Implementer Blast Radius

The following files require changes when implementing this ADR (aligned with
BC-2.06.003 v1.3 spec):

### Core resolution logic

| File | Change |
|------|--------|
| `crates/prism-credentials/src/resolution.rs` — `resolve_credential` | Replace `{SENSOR_UPPER}_{REF_UPPER}` env-var construction with `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` per-client format; requires `client_id` (org slug) as input to derive `{ID}` |
| `crates/prism-bin/src/boot.rs` — `KeyringCredentialProbe::probe` | Extend to iterate all orgs from `OrgRegistry`; check Tier 1/2 per-org env vars; update `CredentialRefProbe` trait signature to include `org_registry: &OrgRegistry` |
| `crates/prism-bin/src/boot.rs` — `step5_init_credential_store_with_probe` | Thread `org_registry: &Arc<OrgRegistry>` parameter; pass to probe |
| `crates/prism-bin/src/boot.rs` — `step5_init_credential_store` | Pass `org_registry` through to `step5_init_credential_store_with_probe` |

### Sensor TOML specs (add `[[credential_refs]]`)

| File | Missing refs to add |
|------|---------------------|
| `sensors/armis.sensor.toml` | `[[credential_refs]] name = "bearer_token"` [ADR-053 §D2 SUPERSEDES on acceptance: `name = "secret_key"` via `auth_type = "token_exchange"`; ADR-053 D5 rewrite manifest executes the spec file update] |
| `sensors/claroty.sensor.toml` | `[[credential_refs]] name = "bearer_token"` |
| `sensors/cyberint.sensor.toml` | `[[credential_refs]] name = "api_key"` [ADR-053 §D3 SUPERSEDES on acceptance: `name = "access_token"`; `cyberint.sensor.toml` DELETED + split into `cyberint-alerts.sensor.toml` + `cyberint-assets.sensor.toml`; ADR-053 D5 rewrite manifest executes] |
| `sensors/crowdstrike.sensor.toml` | `[[credential_refs]] name = "client_id"` + `[[credential_refs]] name = "client_secret"` |
| `crates/prism-sensors/specs/armis.sensor.toml` | Same as above (mirror copy) [same ADR-053 §D2 supersession applies] |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Same |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | Same [same ADR-053 §D3 supersession applies; file deleted + split] |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | Same |

### DTU E2E test env-var setters

| File | Old env var | New env var (example org `acme`) |
|------|-------------|----------------------------------|
| `crates/prism-bin/tests/helpers/mod.rs` — `launch_prism_bin` | `ARMIS_BEARER_TOKEN` | `PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN` [ADR-053 §D2 SUPERSEDES on acceptance: credential_ref `bearer_token` → `secret_key`; new env var → `PRISM_CLIENTS_ACME_SENSORS_ARMIS_SECRET_KEY`] |
| `crates/prism-bin/tests/helpers/mod.rs` | `CLAROTY_BEARER_TOKEN` | `PRISM_CLIENTS_ACME_SENSORS_CLAROTY_BEARER_TOKEN` |
| `crates/prism-bin/tests/helpers/mod.rs` | `CYBERINT_API_KEY` | `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_API_KEY` [ADR-053 §D3 SUPERSEDES on acceptance: credential_ref `api_key` → `access_token`; `cyberint` sensor_id splits into `cyberint-alerts`/`cyberint-assets`; new env vars → `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_ALERTS_ACCESS_TOKEN` / `PRISM_CLIENTS_ACME_SENSORS_CYBERINT_ASSETS_ACCESS_TOKEN`] |
| `crates/prism-bin/tests/helpers/mod.rs` | `CROWDSTRIKE_CLIENT_ID` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_ID` |
| `crates/prism-bin/tests/helpers/mod.rs` | `CROWDSTRIKE_CLIENT_SECRET` | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CLIENT_SECRET` |
| `crates/prism-credentials/tests/bc_2_03_006_credential_resolution.rs` | `CROWDSTRIKE_API_KEY`, `CROWDSTRIKE_CLIENT_ID`, `CROWDSTRIKE_CLIENT_SECRET` (all using org `"acme"`) | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_*` per per-client format |
| `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | `CROWDSTRIKE_CLIENT_ID`, `CROWDSTRIKE_CLIENT_SECRET` | Per-client format (test must declare which org is being simulated) |
| `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` | `ARMIS_BEARER_TOKEN` | `PRISM_CLIENTS_{ID}_SENSORS_ARMIS_BEARER_TOKEN` [ADR-053 §D2 SUPERSEDES on acceptance: `SECRET_KEY` suffix → `PRISM_CLIENTS_{ID}_SENSORS_ARMIS_SECRET_KEY`] |

Note: The exact org slug used in test env vars is determined by the test fixture config
(typically `acme` or `demo-org-a` per the prism.toml fixture in that test's config dir).
The implementer MUST read the test's fixture `prism.toml` to derive the correct `{ID}`.

### Unit tests in `resolve_credential` test suite

The unit tests in `bc_2_03_006_credential_resolution.rs` must:
1. Set env vars using the per-client format: `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}`.
2. Pass the org slug as `client_id` to `resolve_credential`.
3. Verify the NOT-FOUND error message cites the per-client env var name.

### bc_2_03_013 and step-5 test suite

All tests using `step5_init_credential_store_with_probe` with a mock `CredentialRefProbe`
must update the mock to implement the new `probe(sensor_id, ref_name, org_registry)` signature.

## Alternatives Considered

**Option B (Retracted):** Keep `{SENSOR}_{REF}` global format, retract BC-2.06.003
per-client requirement, file tech-debt entry. Rejected because: (a) violates DI-002 for
env-var-sourced credentials; (b) the tech-debt model would defer a correctness issue
indefinitely; (c) the human explicitly selected Option A.

## Source / Origin

- **Adversarial finding:** `F-SDEMO002-P-MED-001` — surfaced during adversarial review of S-DEMO-002; `resolve_credential` was computing `{SENSOR_UPPER}_{REF_UPPER}` env-var names with no org discriminant, directly violating DI-002 for env-var-sourced credentials
- **Human adjudication:** Option A selected 2026-06-03; `decision_made_by: human (Option-A selection, 2026-06-03)` recorded in frontmatter
- **Behavioral contracts:** BC-2.06.003 v1.3 — canonical per-client credential convention + slug transform; this ADR brings code into alignment with the BC
- **Code as-built:** `crates/prism-credentials/src/resolution.rs` — `resolve_credential` function (pre-ADR-032 global-format site); `crates/prism-bin/src/boot.rs` — `KeyringCredentialProbe::probe` and `step5_init_credential_store_with_probe` (boot-time credential probe, org-registry threading)

## References

- BC-2.06.003 v1.3 — canonical per-client credential convention + slug transform
- BC-2.03.006 — query-time credential resolution (downstream of this convention)
- BC-3.2.002 — OrgId-keyed keyring (Tier 3 of the resolution chain)
- BC-2.03.013 — boot-step-5 credential probe (probe redesign required)
- ADR-029 — per-client `base_url` env-var pattern (this ADR mirrors that pattern)
- ADR-022 — production runtime wiring (OrgRegistry threading requirement)
- DI-002 — credential isolation invariant (the motivating correctness requirement)

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-07-21 | architect | FIX-BURST 24 (OBS-1): §Status blockquote — "supersedes" corrected to "amends" for partial sub-section changes per project convention (ADR "supersedes" = full ADR replacement; "amends" = partial sub-section change). Three sites in the §Status blockquote: (1) heading "Partial supersession pending" → "Partial amendment pending"; (2) "ADR-053 §D2 supersedes the Armis credential-ref name" → "amends"; (3) "ADR-053 §D3 supersedes the Cyberint credential-ref name" → "amends". "This ADR is not wholly superseded" unchanged — that usage refers to ADR-level lifecycle status (not replaced by another ADR), a correct use of "superseded." POL-29: §Status blockquote sweep — zero remaining "supersedes" misuse of the partial-sub-section class. |
| 1.2 | 2026-07-21 | architect | FIX-BURST 17 (OBS-1): `subsystems_affected` field populated — `[SS-03, SS-06, SS-22]` (Credential Management, Client Configuration, Process Lifecycle); field was empty in v1.1 despite that version's frontmatter adding the field. |
| 1.1 | 2026-07-21 | architect | FIX-BURST 8 (MED-1/OBS-1): §Sensor spec files table — Armis row annotated with ADR-053 §D2 supersession (credential_ref `bearer_token` → `secret_key` via `token_exchange`); Cyberint row annotated with ADR-053 §D3 supersession (credential_ref `api_key` → `access_token`; `cyberint.sensor.toml` DELETED + split into `cyberint-alerts.sensor.toml` + `cyberint-assets.sensor.toml`); mirror-copy rows annotated. DTU E2E test env-var migration table: Armis row and Cyberint row annotated; Armis `bc_2_01_013` row `{ORG_ID}` → `{ID}` (OBS-1). §Consequences/Migration cost: `{ORG_ID}` → `{ID}` (OBS-1). §Unit tests: `{ORG}` → `{ID}` (OBS-1). §Status: partial-supersession-pending note added. Frontmatter: `version` + `modified` fields added; `related_adrs` extended with ADR-053 + ADR-054. POL-29: zero live `{ORG_ID}`/`{ORG}` hits post-fix. |
| 1.0 | 2026-06-03 | architect | Initial ADR — per-client credential env-var convention adopted (human-selected Option A, D-SDEMO002-P-MED-001). |
