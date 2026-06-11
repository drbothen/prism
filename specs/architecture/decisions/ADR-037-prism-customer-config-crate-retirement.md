---
document_type: adr
adr_id: "ADR-037"
title: "prism-customer-config Crate Retirement — Superseded by Boot Step 3 OrgRegistry (BC-2.21.001), Spec-Engine Per-Org Overlays (ADR-029), and Per-Client Credential Convention (ADR-032)"
status: ACCEPTED
date: "2026-06-10"
version: "1.0"
producer: architect
subsystems_affected: [SS-03, SS-06, SS-21]
supersedes: ADR-010
superseded_by: null
anchor_stories: []
runtime_deliverables:
  - "Delete crates/prism-customer-config/ (entire crate: src/, tests/, Cargo.toml)"
  - "Remove \"crates/prism-customer-config\" from workspace Cargo.toml members"
  - "Reference sweep: zero prism-customer-config / prism_customer_config citations remain in crates/ after removal"
wiring_deferred_to: null
---

# ADR-037: prism-customer-config Crate Retirement — Superseded by Boot Step 3 OrgRegistry (BC-2.21.001), Spec-Engine Per-Org Overlays (ADR-029), and Per-Client Credential Convention (ADR-032)

## Status

ACCEPTED 2026-06-10 v1.0. Human-approved as recommendation ⑨ of the 2026-06-10
full-codebase review package (finding BOOT-01; all 14 recommendations approved as a
package). Crate removal is executed by an implementer fix PR in the same review cycle
(Fix PR 3).

---

## Context

ADR-010 (Customer Config Schema, ACCEPTED 2026-04-28) specified a per-customer
`customers/{org_slug}.toml` file format and three runtime deliverables, all delivered
in the `prism-customer-config` crate during Wave 3 (S-3.3.01 / S-3.3.02, PRs #92 / #97):

1. `prism-customer-config::CustomerConfig` — serde struct for `customers/{org_slug}.toml`
2. `prism-customer-config::load_and_validate` — directory loader + validator
3. `prism-customer-config::boot_org_registry` — OrgRegistry population from loaded configs

When ADR-010 shipped, no production binary existed (the workspace audit D-301 later
confirmed the runtime was functionally absent). The production runtime that subsequently
materialized through ADR-022 / ADR-029 / ADR-032 took a different shape, and the
2026-06-10 full-codebase review (finding BOOT-01) confirmed: **`prism-customer-config`
has zero production dependents.** No crate in the workspace declares a dependency on it
(verified: the only `prism-customer-config` references outside the crate's own directory
are the workspace `members` entry in the root `Cargo.toml`), and no `[[bin]]` target
reaches any of its three deliverables. This is a POL-15
(`runtime_wiring_required_for_accepted_adrs`) violation: an ACCEPTED ADR whose crate-level
deliverables are unreachable from the production binary.

The functional surface ADR-010 specified did not disappear — it was re-delivered through
later, more-specific decisions (Source-of-Truth Precedence Rule 2):

| ADR-010 deliverable | Superseding production mechanism |
|---|---|
| `CustomerConfig` struct / `customers/{org_slug}.toml` org identity (org_id, org_slug, display_name) | `prism.toml` `[[orgs]]` entries in `PrismConfig.orgs`, validated at boot step 3 (BC-2.21.001: UUID-v7 check, kebab-case slug check, bijectivity check) |
| `boot_org_registry` (OrgRegistry population) | `prism-bin` `boot.rs::step3_init_org_registry` (ADR-022 §B step 3; BC-2.21.001; exit 2 on failure) |
| `load_and_validate` (per-customer config discovery + validation) | Spec-engine `OverlayLoader` (`prism-spec-engine/src/overlay.rs`, S-CONFIG-MULTI-TENANT-OVERRIDE-001): per-org `customers/<org_slug>/<sensor_id>.sensor.toml` INSTANCE overlay files discovered, validated, and merged onto TYPE specs at boot step 4 (ADR-029; BC-2.06.012–016) |
| `[[dtu]]` `credential_ref` opaque-reference schemes | ADR-032 per-client credential env-var convention (`PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` + `_FILE`, 4-tier resolution chain) + ADR-034 Tier-3 keyring resolution; AI-opaque principle (AD-017) preserved |
| `[dtu.data]` archetype/scale/seed data-generator parameters | ADR-036 deterministic scenario-progression engine + per-client seeded DTU clone data (BC-2.06.018, `new_with_seed(seed, archetype, org_id)`) |

---

## Decision

`crates/prism-customer-config` is RETIRED. The crate (source, tests, manifest) is deleted
from the workspace, and its workspace `members` entry is removed. ADR-010 is marked
SUPERSEDED by this ADR. No deprecation grace period is required: the crate has never been
published to crates.io and has zero in-tree dependents (same reasoning as ADR-027 D1 for
the `CustomAdapter` removal — atomic same-burst deletion is safe).

Crate removal is executed by an implementer fix PR (Fix PR 3) in the 2026-06-10 review
cycle, with the standard reference sweep: post-removal, `rg 'prism[-_]customer[-_]config'
crates/ Cargo.toml` returns zero matches.

Behavioral contracts BC-3.3.001 through BC-3.3.004 (all currently `draft`, anchored to
the retired crate's stories S-3.3.01/S-3.3.02) require lifecycle disposition by the
product-owner per ADR-021 in the same review cycle: retire each BC, or re-anchor it to
the superseding surface where the contract's intent survives (e.g., BC-3.3.002's
"no credential values in config files" intent is carried by ADR-032/BC-2.06.003;
BC-3.3.004's "collect-all-errors, refuse-to-start" intent is carried by BC-2.21.001 and
the OverlayLoader validation of BC-2.06.012–016). This ADR records the architecture-side
supersession; BC lifecycle records are product-owner-owned.

---

## Rationale

Retaining a zero-dependent crate carries ongoing cost with no benefit: it compiles,
lints, and tests on every CI run; its `CustomerConfig` schema describes a config file
format (`customers/{org_slug}.toml` single-file-per-customer) that the production boot
path never reads, which actively misleads agents and operators reading the workspace
(the production `customers/` directory layout is the ADR-029 per-org overlay
composition directory `customers/<org_slug>/<sensor_id>.sensor.toml` — a different
structure under the same directory name).

Re-wiring the crate into the boot path instead of retiring it was rejected: every one of
its three deliverables has a shipped, BC-anchored, adversarially-reviewed production
replacement (table in §Context). Wiring the crate in would create a second, conflicting
org-configuration mechanism and re-open settled decisions (ADR-029's scalar-only overlay
enforcement, ADR-032's credential tiering, BC-2.21.001's prism.toml org declarations).

POL-15 closure: with the crate deleted, the POL-15 audit finding for ADR-010's
unreachable deliverables is closed structurally — there is no longer an ACCEPTED ADR with
unwired crate-level deliverables on this surface. ADR-010's registry status becomes
SUPERSEDED (POL-15 scopes its check to ACCEPTED ADRs), and ADR-037's own
`runtime_deliverables` are removal actions verified by Fix PR 3.

---

## Consequences

### Positive

- POL-15 violation BOOT-01 closed; the ADR registry again reflects runtime reality.
- Workspace shrinks by one crate (26 → 25 workspace `members` entries, counting the
  nested crowdstrike-oauth2 plugin member); CI compile/test surface reduced.
- The misleading dual meaning of the `customers/` directory is resolved: only the
  ADR-029 overlay layout remains specified as production behavior.
- ADR-010's durable principles survive in their superseding homes: AI-opaque credential
  references (ADR-032/AD-017), deny-unknown-fields validation (ADR-029 overlay
  validation, BC-2.06.013), collect-all-errors startup refusal (BC-2.21.001,
  BC-2.06.015/016).

### Negative / Trade-offs

- The crate's test corpus (slug-pattern, TOML redaction, path-traversal, startup-boot
  tests) is deleted with it. Equivalent coverage exists on the superseding surfaces
  (BC-2.21.001 boot tests in prism-bin; overlay validation tests in prism-spec-engine;
  credential-redaction tests in prism-credentials), but the specific
  `customers/{org_slug}.toml` schema-validation tests have no successor because the
  schema itself is retired.
- If a future wave needs per-customer config beyond scalar sensor overlays (e.g., the
  `[shared_infra]` routing-parameter block from ADR-010 §2.4), it must be re-specified
  fresh against the ADR-029 composition-directory model — ADR-010's schema cannot be
  resurrected by re-adding the crate.

### Status as of 2026-06-10

Decision ACCEPTED (human-approved 2026-06-10, review-package recommendation ⑨).
Crate removal pending Fix PR 3 in the same review cycle; until that PR merges, the crate
remains in the workspace but is governance-retired (no new code may depend on it).

---

## Alternatives Considered

### Alt 1: Wire prism-customer-config into the boot path (satisfy POL-15 by wiring)

Rejected. All three deliverables have shipped, BC-anchored replacements (§Context table).
Wiring would introduce a second org-configuration mechanism conflicting with BC-2.21.001
(prism.toml `[[orgs]]`) and ADR-029 (overlay composition directory), and would re-open
the credential-reference design settled by ADR-032/ADR-034. Cost is high; benefit is nil.

### Alt 2: Keep the crate as dormant/reference code with a deprecation annotation

Rejected. A `#[deprecated]` grace period serves external consumers; this crate has none
(never published, zero in-tree dependents). Dormant code that describes a non-production
config schema is an active hazard for agents grounding on the workspace (the
2026-06-10 review demonstrated exactly this confusion). Same-burst deletion follows the
ADR-027 D1 precedent.

### Alt 3: Retire the crate but leave ADR-010 status ACCEPTED

Rejected. POL-15 audits ACCEPTED ADRs for deliverable reachability; leaving ADR-010
ACCEPTED with deleted deliverables would re-raise the same finding every audit cycle.
Marking ADR-010 SUPERSEDED (bidirectional `supersedes`/`superseded_by` links per ADR
lifecycle) records the truth: the decision was implemented, then replaced.

---

## Source / Origin

- **Finding BOOT-01**, 2026-06-10 full-codebase review package; human approved
  recommendation ⑨ (RETIRE) on 2026-06-10 as part of the 14-recommendation package.
- **Implementation evidence (zero dependents):** `rg 'prism-customer-config'` across the
  workspace matches only `crates/prism-customer-config/**` itself and the root
  `Cargo.toml` workspace `members` entry — no `[dependencies]` declaration in any other
  crate; no `use prism_customer_config` outside the crate.
- **Superseding surfaces (as-built):**
  `crates/prism-bin/src/boot.rs::step3_init_org_registry` (BC-2.21.001 org validation +
  registration); `crates/prism-spec-engine/src/overlay.rs` `OverlayLoader` (ADR-029,
  BC-2.06.012–016, story S-CONFIG-MULTI-TENANT-OVERRIDE-001); ADR-032 credential
  env-var convention; ADR-036 / BC-2.06.018 seeded data generation.
- **Antecedent ADR:** ADR-010 (superseded by this ADR); delivered by S-3.3.01/S-3.3.02
  via PRs #92/#97 during Wave 3.
- **Policy:** POL-15 `runtime_wiring_required_for_accepted_adrs`
  (`.factory/policies.yaml` id 15).

---

## Related ADRs

| ADR | Relationship |
|-----|-------------|
| **ADR-010** | Superseded by this ADR (bidirectional link). Schema, loading lifecycle, and runtime deliverables retired with the crate. |
| **ADR-007** | Partially amended by this ADR (ADR-007 v0.16, bidirectional): §2.4 customer-config validation rules, the §2.1/§3.1 BC-3.3.001 startup guard, and the §7 OQ-1 `allow_shared_override` escape hatch are superseded/mooted with the ADR-010 schema. ADR-007 remains ACCEPTED — its classification, mode semantics, `DTU_DEFAULT_MODE` registry (shipped in prism-core), and §2.5/BC-3.2.005 immutability are unaffected. |
| **ADR-029** | Superseding surface for per-customer sensor configuration (overlay composition directory). |
| **ADR-032 / ADR-034** | Superseding surface for credential references (per-client env-var convention; Tier-3 keyring resolution). |
| **ADR-022** | Boot sequence owner — §B step 3 (OrgRegistry init) is the production home of org registration. |
| **ADR-027** | Precedent for same-burst retirement of never-published, zero-dependent code. |
| **ADR-036** | Superseding surface for per-customer deterministic data-generation parameters. |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-10 | architect | Initial authorship — BOOT-01 closure per human-approved 2026-06-10 review package recommendation ⑨. Supersedes ADR-010; crate removal mandated via Fix PR 3; BC-3.3.001..004 disposition routed to product-owner per ADR-021. Same-burst follow-up (pre-commit fold): §Related ADRs row added for ADR-007 partial amendment (ADR-007 v0.16 — §2.4 / BC-3.3.001 guard / OQ-1 superseded; ADR-007 remains ACCEPTED). |
