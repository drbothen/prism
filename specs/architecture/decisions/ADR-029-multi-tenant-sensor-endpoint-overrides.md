---
document_type: adr
adr_id: "ADR-029"
title: "Multi-Tenant Sensor Endpoint Overrides — Hybrid Sensor Instance with Per-Org Composition Directory"
status: Proposed
date: "2026-05-23"
modified: "2026-05-23"
version: "1.1"
producer: architect
subsystems_affected: [SS-01, SS-06, SS-16, SS-21]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [PLUGIN-MIGRATION-001-E]
related_adrs: [ADR-010, ADR-022, ADR-028]
related_bcs: []
locked_decisions: ["D-803 Decision 1", "D-803 Decision 2"]
inputs:
  - .factory/research/multi-tenant-sensor-endpoint-overrides-2026-05-23.md
input-hash: ""
wiring_deferred_to: null
---

# ADR-029: Multi-Tenant Sensor Endpoint Overrides — Hybrid Sensor Instance with Per-Org Composition Directory

## Status

Proposed 2026-05-23, v1.0. Locks D-803 architectural-clarification decisions. Will be promoted to ACCEPTED after story `S-CONFIG-MULTI-TENANT-OVERRIDE-001` reaches LOCAL adversarial 3-CLEAN convergence per ADR-021 promotion lifecycle.

---

## Context

### The Endpoint Parameterization Gap

Prism's current sensor spec model is "spec-once, credentials-per-query." A sensor TYPE is declared once in a `<sensor>.sensor.toml` file (with `base_url`, `[[tables]]` schema, `auth_type`, and `rate_limit_hints`). At query time, `(org_id, sensor_id)` is used to look up credentials in the keyring. This correctly handles the **credential** dimension of multi-tenancy.

The **endpoint** dimension is unresolved. `armis.sensor.toml` declares:

```toml
base_url = "${env.ARMIS_INSTANCE_URL}"
```

This resolves from a single environment variable — one Armis endpoint per prism process. In a multi-tenant MSSP deployment, different client organizations run Armis Centrix on different instances: `armis.acme-corp.io` vs `armis.contoso.com`. A single env var cannot serve both.

This gap was discovered during PLUGIN-MIGRATION-001-E architecture-clarification dialogue (D-803, 2026-05-23). The research-agent completed Burst 1/4 (commit `a44d5c2b`) grounding the problem in nine industry tools.

### Sensor Reality: On-Prem vs SaaS

The endpoint parameterization problem is not uniform across sensors:

**Sensors that vary per tenant (on-prem-ish):**
- Armis Centrix: per-customer cloud instance (`<org>.armis.io` or on-prem)
- Claroty: on-premises deployment per client site
- Wazuh: self-hosted manager per MSSP client (some deployments)

**Sensors with a single global endpoint (SaaS):**
- CrowdStrike Falcon: `api.crowdstrike.com` (all tenants, credential-distinguished)
- Cyberint: single SaaS API endpoint

The solution must handle both classes without boilerplate on the SaaS side.

### TOML Array-Merge Constraint

Rust's `config-rs` crate (and `figment`) deep-merges TOML tables but **replaces** TOML arrays wholesale. This is not a library quirk — it is the universal behavior confirmed across Helm, Kustomize, Ansible, and Spring Boot. The prism sensor spec uses `[[tables]]` (TOML array-of-tables) for schema declarations. Any override mechanism that allows per-org files to specify `[[tables]]` would silently replace the entire table array, losing schema definitions from the TYPE spec. This is the footgun that drove a decade of Helm/Kustomize complexity.

---

## Decision Drivers

| Driver | Constraint |
|--------|------------|
| Multi-tenant single-process deployment | No per-tenant prism instances; one prism serves multiple orgs |
| AI-opaque credentials (AD-017) | Endpoint must NOT be embedded in credential store values; operators must grep config to see routing |
| Schema stability | Per-tenant overrides must NOT alter `[[tables]]` definitions; federated query semantics require uniform schema across tenants |
| TOML array REPLACE semantics | config-rs/figment: `[[tables]]` arrays are replaced on merge, never merged element-wise |
| Operator discoverability | "Where does `armis@acme` send queries?" must be answerable without reading the keyring |
| Fail-fast at boot | Per-org override validation occurs at boot; no runtime surprises on first query |
| Backwards compatibility | Existing single-tenant prism deployments continue to work without config changes |
| OrgRegistry coherence | Per-org overlay directories must reference org slugs registered in OrgRegistry (ADR-010) |

---

## Options Considered

### Option (a): Per-Org Override File at `customers/<org>/<sensor>.sensor.toml`

**Mechanism:** Global TYPE spec at `crates/prism-sensors/specs/<sensor>.sensor.toml`. Per-org overlay at `crates/prism-sensors/specs/customers/<org>/<sensor>.sensor.toml` with all or some fields; merged via config-rs layered sources.

**Industry analog:** Closest to Helm `-f base.yaml -f tenant.yaml` layering.

**Pros:** Low boilerplate for scalar-only overrides; file layout is discoverable.

**Cons:** TOML `[[tables]]` arrays would be REPLACED, not merged. A per-org file containing `[[tables]]` would silently lose all tables not restated. This is the Helm/Kustomize array footgun, documented in research §2.10. Even with a strict "scalars-only" policy, the policy is unenforced at the TOML parser level — an operator who writes `[[tables]]` in an overlay causes a silent schema loss at runtime, not a boot-time error. Requires explicit runtime validation to catch.

**Why not chosen as standalone:** The TOML array hazard requires the same boot-time rejection gate that option (e) provides, but option (a) lacks the instance identity concept that makes the `(org_id, sensor_id)` tuple naturally resolve to a unique resolved spec.

---

### Option (b): Per-Org Inline `[[orgs.sensor_overrides]]` in `prism.toml`

**Mechanism:** All per-org sensor overrides live in `prism.toml` under each `[[orgs]]` block as a nested `[sensor_overrides]` or `[[sensor_overrides]]` table.

**Industry analog:** Resembles Filebeat `instances:` list / Datadog `instances:` YAML list in a single file.

**Pros:** Single-file discoverability; validation centralised.

**Cons:** `prism.toml` grows linearly with (tenants × sensors needing override); operators managing 50+ clients with 4 sensors each get a 200-entry file. Config schema for prism.toml (ADR-010) would require significant extension. The separation of concerns between "org identity config" (prism.toml) and "sensor endpoint config" (spec files) is broken.

**Why not chosen:** Scale and config-schema complexity. Per-org override in prism.toml confounds org identity management with sensor runtime config.

---

### Option (c): Credential-Encoded Base URL (endpoint inside the keyring record)

**Mechanism:** The keyring credential value for `(org, sensor)` contains both the auth token and the `base_url`. The credential resolver returns the endpoint alongside the token at query time.

**Industry analog:** No mainstream analog — every tool surveyed (Telegraf, Datadog, OTel, Vector, Fluent Bit) keeps endpoint in config and credentials in a separate secret store.

**Pros:** Zero new config files; endpoint resolved at credential-lookup time.

**Cons:** Violates AD-017 (AI-opaque credentials) in spirit — operators can no longer grep config to see "where does org X's sensor query go?" The endpoint is hidden in the keyring, which by design does not transit AI context. Breaks operator discoverability ("Where is my data going?" requires reading credentials). No mainstream tool conflates endpoint with credential. Rejected during research phase as an anti-pattern.

**Why not chosen:** AD-017 compliance; operator discoverability; no industry precedent.

---

### Option (d): Sensor Instance ID `armis@acme`, `armis@contoso` (full per-instance specs)

**Mechanism:** Each per-org sensor is a fully independent spec file: `armis@acme.sensor.toml`, `armis@contoso.sensor.toml`. Each restates the complete TYPE schema plus the per-instance `base_url`. Instance identity is `{sensor}@{org}`.

**Industry analog:** Telegraf `[[inputs.http]]` with `alias`; OTel `otlp/tenant_a`; Vector component IDs; Datadog `instances:` list.

**Pros:** No TOML merge complexity; each instance validates independently; maps to industry-standard pattern.

**Cons:** High boilerplate — each per-org spec restates the full `[[tables]]` schema. Schema changes (adding a column) require updating every per-org file. Violates the DRY principle for schema definitions. With 50 tenants × 4 sensors = 200 spec files with duplicated schema.

**Why not chosen as standalone:** Schema duplication creates a maintenance burden and drift risk where per-org files diverge from the canonical schema without a validator enforcing consistency.

---

### Option (e): Hybrid — Sensor Instance with Per-Org Composition Directory (CHOSEN)

**Mechanism:** Combines TYPE-level schema from option (d)'s instance concept with option (a)'s directory composition, but with a strict scalar-only policy enforced at boot. The TYPE spec declares the schema; the per-org INSTANCE overlay declares only scalar tunables.

**Industry analogs:** Combines Telegraf TYPE-stable/INSTANCE-parameterized split + OTel named-component identity (`otlp/tenant_a`) + Vector `--config-dir` composition + Fluent Bit `@INCLUDE tenants/*.conf`.

**Pros:** Schema defined once at TYPE; per-org overlays are minimal (often just `base_url`); boot-time rejection of `[[tables]]` in overlays turns the footgun into a hard error; `(org_id, sensor_id)` credential tuple maps cleanly to instance identity `{sensor}@{org}`; SaaS sensors (CrowdStrike) need zero overlay files.

**Cons:** Still requires operators to create per-org files for on-prem sensors (Armis, Claroty). File count grows with tenants, but only for sensors that actually vary per tenant — research §5 documents this as the accepted industry trade-off.

**Why chosen:** Captures the full benefit of industry pattern (instance-per-tenant) while eliminating the schema duplication cost via the TYPE/INSTANCE split. Eliminates the TOML array footgun via hard boot-time rejection.

---

## Decision

**Adopt Option (e): Hybrid Sensor Instance with Per-Org Composition Directory.**

The TYPE/INSTANCE split is the key insight: schema lives at the TYPE, endpoint and scalar tunables live at the per-org INSTANCE overlay. This mirrors Telegraf's `[[inputs.http]] alias` and OTel's `otlp/tenant_a` naming convention while remaining compatible with the `(org_id, sensor_id)` credential tuple prism has already implemented.

### File Layout

```
crates/prism-sensors/specs/
├─ armis.sensor.toml              # TYPE definition: sensor_id, name, auth_type,
│                                 # base_url (default/single-tenant fallback),
│                                 # [[tables]] schema (canonical, shared by all instances)
├─ claroty.sensor.toml
├─ crowdstrike.sensor.toml        # SaaS: base_url is fixed; no per-org overlay needed
├─ cyberint.sensor.toml           # SaaS: base_url is fixed; no per-org overlay needed
└─ customers/
   ├─ .gitkeep                    # dir always present; empty = zero per-org overrides
   ├─ acme/
   │  ├─ armis.sensor.toml        # INSTANCE overlay: base_url = "https://armis.acme-corp.io"
   │  └─ claroty.sensor.toml      # INSTANCE overlay: base_url = "https://claroty.acme.local"
   ├─ contoso/
   │  └─ armis.sensor.toml        # INSTANCE overlay: base_url = "https://armis.contoso.com"
   └─ globex/
      └─ armis.sensor.toml        # INSTANCE overlay: base_url = "https://globex.armis.io"
```

### Instance Overlay Schema

A per-org INSTANCE overlay file is a restricted TOML document. It MUST NOT contain `[[tables]]` blocks. Valid fields:

```toml
# customers/acme/armis.sensor.toml — INSTANCE overlay for armis@acme

# Required: declare which TYPE this instance is an overlay for.
extends = "armis"

# Required: instance identity. Convention: "{sensor_type}@{org_slug}"
instance_id = "armis@acme"

# Required (for on-prem sensors): per-org API endpoint.
base_url = "https://armis.acme-corp.io"

# Optional: per-org rate limit tuning (scalar override).
[rate_limit_hints]
requests_per_second = 5.0

# Optional: per-org request timeout override (scalar).
# timeout_secs = 45

# FORBIDDEN in overlay files (boot-time hard error E-SPEC-021):
# [[tables]]   ← schema lives at TYPE level only
```

### Instance Identity Convention

Instance identity follows the pattern `{sensor_type}@{org_slug}`:
- `armis@acme` — Armis instance for org `acme`
- `claroty@contoso` — Claroty instance for org `contoso`
- `crowdstrike` — no overlay needed; single global SaaS endpoint (bare sensor_id is the instance)

At query time, the `(org_id, sensor_id)` tuple resolves to the instance identity via:
1. `OrgRegistry.slug_for(org_id)` → `OrgSlug` (e.g., `acme`)
2. Instance spec lookup: check for `customers/<org_slug>/<sensor_id>.sensor.toml`; if present, merge scalars from overlay onto TYPE spec
3. If no overlay file exists, use TYPE spec as-is (SaaS sensor or single-tenant deployment)

### Merge Semantics (Deliberately Constrained)

1. **TYPE spec** (`<sensor>.sensor.toml`): loaded once at boot. Registers the sensor TYPE including full `[[tables]]` schema, `auth_type`, default `base_url`, and `rate_limit_hints`.
2. **INSTANCE overlay** (`customers/<org>/<sensor>.sensor.toml`): loaded per (org, sensor) pair. Only scalar fields are merged onto the TYPE spec. The result is a `ResolvedSensorSpec` cached in memory per (org_slug, sensor_id).
3. **Scalar fields eligible for per-org override:** `base_url`, `rate_limit_hints.requests_per_second`, `rate_limit_hints.burst_size`.
4. **Fields NOT overridable:** `auth_type` (security invariant — per-org auth type changes would bypass credential validation), `version`, `sensor_id`, `tables` (schema stability invariant).

### Boot-Time Validation

Step 4 of the boot sequence (`step4_load_sensor_specs`) is extended to:

1. Load all TYPE specs from `<sensor_specs_dir>/*.sensor.toml` (existing behavior).
2. Walk `<sensor_specs_dir>/customers/` subdirectory tree, collecting per-org overlay files.
3. For each overlay file:
   a. Verify `extends` references a loaded TYPE spec (E-SPEC-019: unknown sensor type).
   b. Verify `instance_id` matches the `{sensor}@{org}` convention (E-SPEC-020: malformed instance_id).
   c. Verify the directory name matches a registered `OrgRegistry` slug (E-SPEC-022: unknown org slug).
   d. Reject any overlay containing `[[tables]]` blocks (E-SPEC-021: schema override forbidden).
   e. Verify no unknown scalar fields (E-SPEC-023: unrecognized overlay field).
4. Produce a `ResolvedSensorSpec` for each `(org_slug, sensor_id)` pair found.

Boot fails (exit code 2 per ADR-022 §A) if any overlay fails validation. The fail-fast policy mirrors the existing credential validation at step 5.

---

## Consequences

### Immediate Changes Required

**`SensorSpec` struct (prism-spec-engine/src/spec_parser.rs):**
- No structural change to `SensorSpec` — it continues to represent a fully resolved spec.
- New type: `SensorInstanceOverlay` (parsed from `customers/<org>/<sensor>.sensor.toml`) with fields: `extends: String`, `instance_id: String`, `base_url: Option<String>`, `rate_limit_hints: Option<RateLimitHints>`.
- New type: `ResolvedSensorSpec` wraps a `SensorSpec` with provenance metadata (which fields came from TYPE vs overlay).

**`SpecLoader::load_all` (prism-spec-engine/src/spec_parser.rs):**
- Extended to walk `customers/` subdirectory and return per-(org, sensor) `ResolvedSensorSpec` in addition to TYPE-level `SensorTableDescriptor`.
- Current flat non-recursive scan behavior preserved for the root directory; recursive walk is scoped to `customers/` only.

**Boot step 4 (`prism-bin/src/boot.rs`):**
- `step4_load_sensor_specs` calls the extended `load_all` and validates overlays.
- Validation errors surface as `BootError::ConfigInvalid` (exit code 2).

**Fanout resolution (`prism-sensors` fanout path):**
- `FanOutTarget` already carries `org_id`; the fanout engine resolves `(org_id, sensor_id)` → `ResolvedSensorSpec` at dispatch time.
- `SensorSpec.base_url` used by adapters comes from the resolved spec (overlay wins if present).

**`CredentialResolver`:**
- No changes. Credential lookup continues by `(org_id, sensor_id)` tuple, independent of endpoint resolution.

**`OrgRegistry`:**
- No changes. Boot step 3 populates OrgRegistry from `customers/*.toml` per ADR-010; step 4 validates overlay directories against the populated registry.

### New Error Codes

The following error codes are added to `.factory/specs/prd-supplements/error-taxonomy.md` (drafted by product-owner in Burst 3; see BC-2.06.016 for canonical definitions). **Note:** E-SPEC-018 is already allocated to `TimestampParseFailure` (ADR-028/BC-2.16.013). The codes below reflect the final allocation shifted +1 from ADR-029 v1.0 draft (E-SPEC-018–022 → E-SPEC-019–023). BC-2.06.016 §INV-ERR-005 and §Source-of-Truth Precedence govern the implementer.

| Code | Condition | Severity |
|------|-----------|----------|
| E-SPEC-019 | Per-org overlay `extends` references an unknown sensor TYPE | FATAL (boot) |
| E-SPEC-020 | `instance_id` does not match `{sensor}@{org}` convention | FATAL (boot) |
| E-SPEC-021 | Per-org overlay contains `[[tables]]` blocks (schema override forbidden) | FATAL (boot) |
| E-SPEC-022 | Overlay directory `customers/<slug>/` references unknown org slug | FATAL (boot) |
| E-SPEC-023 | Overlay file contains unrecognized scalar field | FATAL (boot) |

### Existing Sensor TYPE Specs

All four existing sensor TYPE specs (`armis.sensor.toml`, `claroty.sensor.toml`, `crowdstrike.sensor.toml`, `cyberint.sensor.toml`) require one change: the `base_url` default value for on-prem sensors should be documented as "single-tenant default; override in `customers/<org>/` for multi-tenant deployments."

CrowdStrike and Cyberint require no overlay mechanism — their `base_url` values are already fixed global SaaS endpoints.

---

## Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| Stale per-org overlay after global TYPE schema bump | MEDIUM | Boot validator: emit `WARN` if overlay file mtime is older than TYPE file mtime (pattern from Datadog Agent drift detection) |
| `[[tables]]` footgun in overlay silently loses schema | HIGH → MITIGATED | Boot-time hard reject: E-SPEC-021. No runtime exposure possible |
| Overlay references nonexistent org slug (typo in directory name) | MEDIUM | Boot-time cross-check against OrgRegistry (E-SPEC-022). Fail-fast at boot |
| AI agent or analyst confuses `armis` (TYPE) with `armis@acme` (INSTANCE) in queries | LOW | PrismQL parser rejects unscoped sensor refs when multiple instances exist ("ambiguous sensor reference — use `armis@<org>`") |
| Per-org overlay files multiply linearly with tenants | LOW | Accepted industry trade-off (Telegraf, Datadog accept it); mitigated by `prism org add` CLI scaffolding |
| `prism config show --sensor armis@acme` needs provenance rendering | LOW | `prism config show` renders effective merged spec with per-field provenance; pattern from `helm get values` |
| Single-tenant deployments see no behavioral change | LOW | If `customers/` directory is empty or absent, step 4 loads zero overlays; TYPE spec used as-is |

---

## Behavioral Contracts to be Drafted (Handoff to Product-Owner)

The following BCs are needed in the `SS-06` (Client Configuration) subsystem. Product-owner drafts these in Burst 3:

1. **BC-6.XX.001 — Per-Tenant Overlay Loading and Merge Semantics:** Postcondition: for each `customers/<org>/<sensor>.sensor.toml` found at boot, a `ResolvedSensorSpec` is produced by merging overlay scalars onto TYPE spec. Tables, auth_type, and sensor_id are inherited from TYPE unchanged.

2. **BC-6.XX.002 — Per-Tenant Overlay Boot Validation (Scalar-Only):** Postcondition: any overlay containing `[[tables]]` blocks triggers E-SPEC-021 and aborts boot (exit 2). Any overlay whose `extends` value does not match a loaded TYPE triggers E-SPEC-019 and aborts boot.

3. **BC-6.XX.003 — Instance Identity Resolution at Fanout:** Postcondition: for a query targeting `(org_id, sensor_id)`, the fanout engine resolves the `ResolvedSensorSpec` at dispatch time; `base_url` in the resolved spec is the overlay value if present, otherwise the TYPE default.

4. **BC-6.XX.004 — Org-Registry Cross-Validation at Boot:** Postcondition: every `customers/<slug>/` directory whose `<slug>` does not correspond to a registered `OrgRegistry` entry triggers E-SPEC-022 and aborts boot (exit 2).

5. **BC-6.XX.005 — Error Taxonomy Entries for Override Violations:** Defines E-SPEC-019 through E-SPEC-023 with canonical messages, field context, and recoverable/non-recoverable classification.

BC IDs are provisional (`6.XX`); product-owner assigns canonical sequential IDs consistent with the BC-INDEX.

---

## Story Stub for Handoff to Story-Writer

**Suggested story ID:** `S-CONFIG-MULTI-TENANT-OVERRIDE-001`

**Scope:** One story covering:
- `SensorInstanceOverlay` and `ResolvedSensorSpec` types in prism-spec-engine
- `SpecLoader::load_all` extension for `customers/` traversal + overlay merge
- Boot step 4 extension with overlay validation (E-SPEC-019 through E-SPEC-023)
- Fanout resolution wiring: `(org_id, sensor_id)` → `ResolvedSensorSpec`
- Test fixtures: `customers/acme/armis.sensor.toml` + `customers/contoso/armis.sensor.toml`
- Red Gate: existing single-tenant Armis integration tests pass unchanged
- `customers/.gitkeep` scaffold added to `crates/prism-sensors/specs/customers/`

**Wave placement:** Wave 0 / prereq parallel to `S-PLUGIN-CI-001`. Both unblock multi-tenant prism deployments. Dependency: after ADR-029 accepted, before PLUGIN-MIGRATION-001-F.

**Follow-up story:** `S-CONFIG-MULTI-TENANT-OVERRIDE-002` — `prism config show --sensor <instance_id>` provenance rendering (lower priority; operators can inspect files directly).

---

## Follow-Up Actions

- `crates/prism-sensors/specs/customers/.gitkeep` — add to repo so directory is tracked
- Error taxonomy: add E-SPEC-019 through E-SPEC-023 (product-owner Burst 3; E-SPEC-018 already allocated to TimestampParseFailure per ADR-028/BC-2.16.013)
- CI fixtures: per-tenant overlay files for Armis and Claroty (test-writer, `S-CONFIG-MULTI-TENANT-OVERRIDE-001`)
- `armis.sensor.toml` and `claroty.sensor.toml` TYPE specs: update `base_url` comments to document per-org override path (implementer, same story)
- `prism config show` CLI: provenance-aware rendering (story `S-CONFIG-MULTI-TENANT-OVERRIDE-002`)

---

## Changelog

| Version | Pass | Date | Author | Change |
|---------|------|------|--------|--------|
| 1.0 | D-803 | 2026-05-23 | architect | Initial proposal. Locks D-803 Decisions 1 + 2. ADR-029 registered in ARCH-INDEX v2.101. |
| 1.1 | D-806 | 2026-05-23 | state-manager (POL-29 within-FB sibling-sync) | E-SPEC code range shifted from 018–022 to 019–023. PO caught E-SPEC-018 collision with already-allocated `TimestampParseFailure` (ADR-028/BC-2.16.013) during BC drafting (BC-2.06.016). PO shifted BCs to 019–023; state-manager swept all 14 ADR-029 occurrences of the old draft codes in the same atomic burst to eliminate ADR-vs-BC drift. Source-of-Truth Precedence Rule #3 (BC-2.06.016) governs implementer. |
