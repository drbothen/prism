---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-PROF-1: Compliance Profile mechanism — monotone tighten-only named settings bundle"
  - "ADR-PROP-PROF-2: SF-PROF-1 Two-axis model — deployment-profile vs compliance-profile, distinct and never collapsed"
  - "ADR-PROP-PROF-3: SF-PROF-2 Exemption authority — regulatory hard-locks exemptable only by compliance-authority principal"
  - "ADR-PROP-PROF-4: SF-PROF-3 Conformance reporting — boolean gate + itemized drift, no flat score"
  - "ADR-PROP-PROF-5: SF-PROF-4 Custom profile extensibility — open set, Central-authored, tighten-only"
  - "ADR-PROP-PROF-6: Five shipped presets (baseline / soc2 / iso27001 / iec-62443-ot / nerc-cip)"
  - "ADR-PROP-PROF-7: regulatory_class reframe — selector/floor into the profile engine"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Day-2 out-of-band SIDE-ANALYSIS capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/configurable-security-profiles-2026-06-27.md (PRIMARY — two
  perplexity_research sonar-deep-research calls at reasoning_effort=high covering: (1)
  compliance-framework profile representation as data — CIS, DISA STIG/SCAP/OSCAL, NIST OSCAL,
  K8s PSS/PSA, AWS conformance packs/Security Hub, Azure Policy initiatives, OPA/Gatekeeper,
  Sentinel policy sets, InSpec profiles, MS baselines; (2) conformance reporting + tailoring
  within bounds — OSCAL profile resolution, SCAP tailoring, AWS Config/Security Hub compliance,
  Azure Policy compliance states/exemptions, K8s PSA audit/warn) + research/rbac-depth-2026-06-27.md
  (C18 SF-3 deny-on-stale-beyond-N as the primary profile axis) + ADR-PROP-nested-tenancy.md
  (C19 regulatory_class, tighten-only inheritance, closure-table fold).
  This is an out-of-band side-analysis. CAPTURE ONLY. Does NOT modify any live spec,
  ADR-registry artifact (specs/architecture/), BC, story, STATE.md, or SESSION-HANDOFF.md.
  No git operation performed. Real ADR numbers and formal ARCH-INDEX.md rows deferred to
  the morph execution. touches_no_live_artifacts: true
seeded_from:
  - research/configurable-security-profiles-2026-06-27.md
  - research/rbac-depth-2026-06-27.md
  - specs/day2-design-decisions/ADR-PROP-nested-tenancy.md
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md
cross_refs:
  - specs/day2-design-decisions/ADR-PROP-rbac-depth.md (C18 — staleness axes, action-gating axes, masking axes, key-custody axes from C18/C19)
  - specs/day2-design-decisions/ADR-PROP-nested-tenancy.md (C19 — regulatory_class, closure-table fold, tighten-only guardrails, parent-deny-is-final)
  - specs/day2-design-decisions/ADR-PROP-config-management.md (C9 — signed bundles, canary, fast-revert, Config-DB-Authoritative)
  - specs/day2-design-decisions/ADR-PROP-soar-actions-aro.md (C15 / Day-3 — workflow contract surface, required_approver_role)
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-09/11/12/13; AP-ADS-07; PAT-ADS-03/10; INV-ADS-01..08)
  - matured-vision-day2-requirements.md §16.4 (C18 SF-3; C19 regulatory_class; C20 NERC-CIP)
---

# ADR-PROP — Configurable Security / Compliance Profiles: "OT is a Profile We Ship, Not a Code Branch" (C18 SF-3 + C19 regulatory_class generalization)

> **STATUS: DECIDED 2026-06-27 (human).** Full decision record for the Compliance Profile
> mechanism — a monotone (tighten-only), named, versioned, signed, Central-resolved bundle of
> restrictive settings that generalizes C18 SF-3 (revocation-lag / deny-on-stale-beyond-N +
> OT action gating) and C19's `regulatory_class` (nerc_cip / ot_critical force-tighten). The
> correct expression of P-ADS-13 Configurable-Not-Prescriptive for the OT-restrictiveness concern.
> CAPTURE artifact (`do_not_execute: true`). Real ADR numbers and formal ARCH-INDEX.md rows
> deferred to morph execution.

---

## 1 — Context and Scope

The human directive for C18/C19 design resolved cleanly: do NOT hardcode OT-specific
restrictiveness into the product; ship OT / NERC-CIP / SOC2 / ISO27001 / IEC-62443 /
baseline as named PROFILES (data, not code branches) per P-ADS-11 Single-Codebase and
AP-ADS-07 No-Deployment-Model-Code-Forks.

This directive matches universal industry practice: every mature compliance ecosystem ships
its restrictive postures as named, versioned DATA artifacts:
- NIST OSCAL profiles (a profile imports a catalog, selects controls, tailors parameters)
- CIS Benchmark L1/L2 profile-level tags
- Kubernetes Pod Security Standards (`privileged | baseline | restricted`) selected by namespace label
- AWS conformance packs + Security Hub standards
- Azure Policy initiatives (JSON parameter bundles)
- Sentinel policy sets with per-policy enforcement levels
- Chef InSpec profiles with `depends` + `include_controls`
**[VERIFIED-WEB — all rows; see research/configurable-security-profiles-2026-06-27.md §1]**

C18 SF-3 (staleness `N`, OT gating) and C19 `regulatory_class` are two instances of one
mechanism. The clean generalization is a **Compliance Profile**: a monotone (tighten-only)
named, versioned, signed, Central-resolved bundle of restrictive settings axes, parameterized
within profile-declared bounds, riding the existing C9/C18 signed-bundle + canary infrastructure,
with first-class conformance reporting and a workflow-requirements contract surface.

---

## 2 — Decisions

### D-PROF-1 — Compliance Profile = Data, Resolved at Central, Flattened to Satellites

**Decision:** A Compliance Profile is a **named, versioned, signed TOML/JSON document in the
Config-DB** (P-ADS-09). It is authored ONLY at Central, RESOLVED into an effective settings
set at Central plan-time (exactly like C19's effective-config fold), and pushed to satellites
as a FLATTENED signed bundle (PAT-ADS-03). Satellites never compute profile inheritance.

This is the OSCAL "profile resolution → resolved catalog → SSP baseline" pattern applied to
Prism's settings axes, with the additional constraint that resolution is monotone-tighten-only
(OSCAL permits loosening; Prism does not). **[VERIFIED-WEB — OSCAL profile resolution;
GROUNDED-INTERNAL — C19 fold semantics]**

The Rust binary contains a single profile-resolution + enforcement engine. It reads profile
documents from the Config-DB; it has NO `#[cfg(feature="ot")]` branch (AP-ADS-07 forbidden
form). Shipped presets (`baseline`…`nerc-cip`) are **seeded profile documents** distributed
in the product — versioned content, not compiled code paths. This is exactly how K8s ships
PSS definitions, AWS ships sample conformance packs, and Azure ships built-in initiatives.
**[VERIFIED-WEB]**

---

### D-PROF-2 — Profiles Only Tighten (Monotone, Tighten-Only)

**Decision:** A Compliance Profile can only TIGHTEN. Layering
`baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip` is a monotone meet (intersection
toward stricter). This is the C19 guardrail semantics (D-C19-3 hybrid inheritance:
guardrails intersect + parent-deny-is-final) applied to profile settings.

**Resolution semantics for each setting:**
- **Hard-locked value** (`{ lock = V }`): the strictest lock among the ancestry wins; a child
  layer may further lock but never unlock.
- **Tunable range** (`{ min, max, allowed }`): ancestor ranges INTERSECT — the effective
  permitted range can only NARROW as profiles layer; it can never widen.
- **Absent setting** in a child layer: inherits the parent layer's (already-tightened) value.

Prism's profile resolver REJECTS any layer that attempts to widen a parent's lock or range
at authoring time. Out-of-range or out-of-lock author attempts are rejected programmatically
at the Central authoring boundary — NOT warned and accepted. This is a deliberate improvement
over the precedents (AWS/Azure/SCAP frequently leave enforcement to human governance);
it conforms to P-ADS-12 Production-Grade-Default. **[VERIFIED-WEB + MODEL — monotone
constraint is Prism-specific; OSCAL base resolution is VERIFIED-WEB]**

---

### D-PROF-3 — Restrictive Axes (the Profile's Settings Surface)

The axes a Compliance Profile sets values or constraints on:

| Axis | C18/C19 origin | Lock vs tunable example |
|------|---------------|------------------------|
| **Staleness window** (`deny_on_stale_seconds` per action-class) | C18 §5.4 deny-on-stale-beyond-N / D-C18-6 | `baseline`: tunable `[60, 86400]`; `nerc-cip`: OT actions `{ lock = 0 }` |
| **Action-gating strictness** | C18 D-C18-5 / C15 ARO `required_approver_role` | `baseline`: gate destructive only; `nerc-cip`: gate all `ot.*`, `min_approvals = 2` |
| **Masking / bulk-export strictness** | C18 D-C18-3 ABAC; C16 | `baseline`: bulk-export approver-gated; `iec-62443-ot`: `{ lock = "hard_block" }` |
| **Electronic Security Perimeter rules** | C18 §6.1 CIP-005 | `nerc-cip`: ESP rules forced ON |
| **Audit verbosity** | C18 D-C18-7 decision logging | `soc2`: decision-log ON; `nerc-cip`: decision-log + heightened retention floor |
| **Key-custody requirements** | C19 §3.6 / D-C19-8 / C17 | `iec-62443-ot` / `nerc-cip`: force mechanism (d) BYOC remote-op; forbid mechanism (c) |
| **Parent-visibility ceiling** | C19 §3.5 preset ladder P0..P3 | `nerc-cip`: `max_preset { lock = "P0" }` (forces P3 OFF — matches C19 §3.8 regulatory override) |
| **Workflow requirements** (contract surface only; engine = Day-3) | C15 / Day-3 | Dual-auth / SoD / approver-role REQUIREMENTS the Day-3 engine must honor |

---

### D-PROF-4 — Profile Data Model (OSCAL Parameter + Constraint)

**Decision:** Adopt the **OSCAL parameter + constraint model**: a profile setting is either:
1. **Hard-locked** (`{ lock = V }`): the profile pins the value; the tenant admin cannot change it.
2. **Tunable-within-range** (`{ min, max }` / `{ allowed = [...] }`): the profile declares the
   permitted envelope; the tenant picks a value inside it.

Picking outside the envelope is a REJECTION at Central authoring time, not a silent clamp.
**[VERIFIED-WEB — OSCAL `set-parameters` + `constraint`; SCAP `set-value`/`refine-value`;
Azure initiative `allowedValues`]**

**Illustrative profile document (the `nerc-cip` preset):**

```toml
# profile:nerc-cip  (authored at Central, stored in Config-DB per P-ADS-09)
[profile]
id      = "nerc-cip"
version = "1.0.0"                 # SemVer; bundle is version-stamped (PAT-ADS-03)
extends = "iec-62443-ot"          # monotone tighten-only layering
title   = "NERC CIP critical-infrastructure posture"

# --- hard-locked settings (tenant CANNOT loosen) ---
[settings.staleness]
ot_action.deny_on_stale_seconds     = { lock = 0 }            # any staleness hard-denies OT
read_query.deny_on_stale_seconds    = { min = 60, max = 900 } # tunable WITHIN this clamp

[settings.parent_visibility]
max_preset = { lock = "P0" }       # forces P3 OFF — C19 §3.8 regulatory override

[settings.key_custody]
require_mechanism = { lock = "d" }            # BYOC remote-op only (C19 §3.6)
forbid_mechanism  = ["c"]                     # AP-ADS-11 (restated for profile clarity)

[settings.masking]
bulk_export = { lock = "hard_block" }

[settings.audit]
decision_log    = { lock = "on" }
retention_days  = { min = 2555 }              # tunable floor (~7y); tenant may exceed

[settings.esp]
electronic_security_perimeter = { lock = "on" }   # CIP-005

# --- workflow REQUIREMENTS (contract surface only; Day-3 engine implements) ---
[workflow_requirements]
requires_separation_of_duties = true
dual_auth_on                  = ["ot.*"]
min_approvals                 = 2
required_approver_roles       = ["safety_officer", "security_lead"]
break_glass_alerts_to         = ["security_lead", "compliance_officer"]
```

The `{ lock = V }` vs `{ min, max }` pattern is the OSCAL parameter-vs-constraint model.
**[VERIFIED-WEB]**

---

### D-PROF-5 — Two Distinct Named Axes: Deployment-Profile vs Compliance-Profile (SF-PROF-1 DECIDED)

**Decision:** Deployment-profile and Compliance-Profile are **TWO DISTINCT NAMED AXES** sharing
the same signed-bundle + canary mechanism, NEVER collapsed into one selector.

| | **Deployment profile** (P-ADS-11) | **Compliance profile** (this ADR-PROP) |
|---|---|---|
| What it answers | *Where/how does Prism run?* | *How strict must the posture be?* |
| Values | `saas`, `mssp-managed`, `byoc`, `air-gap` | `baseline`, `soc2`, `iso27001`, `iec-62443-ot`, `nerc-cip` |
| Axis level | Process / deployment instance | Tenant NODE (rides C19 closure-table tree) |
| Mechanism | Selects KMS backend, hosting topology — NO code fork (AP-ADS-07) | Monotone tighten-only settings bundle, Central-resolved, flattened |
| Relationship | **Orthogonal.** A `saas` deployment can host a `nerc-cip` tenant; an `air-gap` deployment can run `baseline`. | Orthogonal. |
| Interaction rule | A deployment profile may set a CAPABILITY FLOOR (e.g., `byoc`/`air-gap` makes mechanism (d) available). A compliance profile may DEMAND a capability the deployment profile must provide. A non-satisfiable combination (e.g., `nerc-cip` requires mechanism (d) — only satisfiable on `byoc`/`air-gap`) = **conformance error, not a silent downgrade.** | See interaction rule. |

**Why distinct axes matter:** Collapsing them re-introduces the "OT deployment = OT code"
coupling the directive forbids. P-ADS-13 states: "OT is a shipped Profile, not a fork." Keeping
two distinct axes with a shared mechanism is the correct AP-ADS-07 conformance path.
**[GROUNDED-INTERNAL — ADS v1.1 P-ADS-11 + P-ADS-13 cross-reference]**

**Non-satisfiable combination detection:** The Central authoring engine checks that the combination
of deployment profile + compliance profile is satisfiable at assignment time. Attempting to assign
`profile:nerc-cip` to a `saas` deployment node where mechanism (d) is unavailable produces a
**conformance error** (surfaced in the UI), not a silent downgrade to a weaker profile. The error
is actionable: "nerc-cip requires mechanism (d) BYOC remote-op — upgrade deployment profile to
`byoc` or `air-gap` to satisfy this requirement." Conforms to P-ADS-12 (production-grade: correct
failure behavior, not silent degradation). **[MODEL — synthesized from research §3.4]**

---

### D-PROF-6 — regulatory_class Reframe as Profile Selector/Floor (C19 consistency)

**Decision:** C19's `regulatory_class ∈ {standard, nerc_cip, ot_critical}` is REFRAMED as a
**profile selector/floor**:

| `regulatory_class` value | Forced profile floor | Inheritance |
|---|---|---|
| `standard` | `≥ baseline` (no additional forcing) | Normal tighten-only inheritance |
| `ot_critical` | `≥ iec-62443-ot` | The node + all descendants cannot drop below `iec-62443-ot` |
| `nerc_cip` | `≥ nerc-cip` | The node + all descendants cannot drop below `nerc-cip` |

The forced profile is a floor the node and its descendants cannot drop below (tighten-only,
same rule as C19 D-C19-3 guardrails). The *enforcement* lives in the generic profile engine;
the *selection* lives in the `regulatory_class` attribute. This preserves C19 §3.8 exactly
while removing the hardcoding: no "if regulatory_class == nerc_cip { ... }" branches in code.
**[GROUNDED-INTERNAL — C19 §3.8 + PIV-C19-6; VERIFIED-WEB — OSCAL regulatory overlay pattern]**

**C19 ADR-PROP amendment note (Pass 2):** ADR-PROP-nested-tenancy.md §3.8 should be lightly
amended to note that `regulatory_class` now routes through the Compliance Profile engine rather
than being evaluated as a separate code path. The behavioral semantics are identical (can only
tighten); only the implementation path changes. This amendment is a Pass-2 action, NOT authored
here per the HARD BOUNDARY.

---

### D-PROF-7 — Compliance-Profile + C19 Closure-Table Fold

**Decision:** Profile assignment is a **per-node tenant attribute** that inherits DOWN the C19
tenant tree and can only tighten. The C19 closure table is REUSED for profile fold — zero new
index required:

- Effective profile at a node = monotone fold over the closure-table ancestor set (root-to-leaf
  traversal applying tighten-only resolution semantics).
- This is identical to how D-C19-3 folds effective config.
- **Resolved at Central, flattened to satellites** (P-ADS-09 / INV-ADS-04): satellites receive
  the FLATTENED effective profile in their signed config bundle and never run the fold.
- A child may assign a STRICTER profile than its parent (child of a `soc2` parent may be
  `nerc-cip`). A child may NOT assign a looser one (child of a `nerc-cip` parent cannot drop to
  `baseline`) — the fold REJECTS it. This is C19's parent-deny-is-final applied to profiles.
  **[GROUNDED-INTERNAL — C19 D-C19-3; closure-table fold reuse]**

---

### D-PROF-8 — Exemption Authority (SF-PROF-2 DECIDED)

**Decision:** Regulatory **hard-lock exemptions** may be granted ONLY by a distinct "compliance
authority" principal — NOT the tenant child-admin or the MSSP-operator/parent.

- **Tunable / advisory settings** remain tenant-exemptable with documented, expiring waivers
  (Azure Policy model: `Mitigated` = met by compensating control; `Waiver` = temporarily
  accepted risk, with `expiresOn`). Waivers are recorded in the audit trail.
- **`regulatory_class`-forced hard-locks** (e.g., `nerc-cip` OT-action `deny_on_stale = 0`)
  may NOT be exempted by the tenant child-admin. A distinct "compliance authority" role is
  required. This mirrors C19 §3.8: "consent may not be purely at the child's discretion in
  regulated contexts." **[VERIFIED-WEB — Azure Policy exemption model; GROUNDED-INTERNAL —
  C19 §3.8]**
- **Critical interaction with tighten-only:** A waiver on a tunable setting must not allow
  loosening below the ANCESTOR node's locked setting. The fold rejects any effective-setting
  value below the ancestor ceiling even with a waiver (the waiver is a tenant-internal record;
  it does not override the fold). Waivers are for documenting accepted risk on TUNABLE ranges,
  not for bypassing tighten-only semantics.

---

### D-PROF-9 — Conformance Reporting (SF-PROF-3 DECIDED)

**Decision:** Ship first-class per-node conformance reporting. Structure:

1. **Per-setting status:** `compliant | drifting | exempt` per setting
   (Azure Policy `compliant/non-compliant/exempt`; SCAP `pass/fail/notapplicable`)
   Each `drifting` row enumerates the delta: required-vs-actual.
   Example: "`ot_action.deny_on_stale_seconds`: required `0`, actual `300` → DRIFT"

2. **Roll-up:** Boolean gate (any hard-locked-setting drift ⟹ node is non-compliant) PLUS
   itemized drift list. **NOT a single flat compliance percentage.** Both research passes warned
   that AWS/Azure flat scores treat a critical-control failure identically to a low-impact one —
   misleading for an MSSP serving OT/NERC-CIP clients. **[VERIFIED-WEB — explicit caution from
   both deep-research passes]**

3. **Audit/warn mode before enforce** (the staged-adoption pattern): a node can assign a stricter
   profile in `report-only` mode (K8s PSA `audit`/`warn`; Gatekeeper `dryrun`/`warn`;
   Sentinel `advisory`) — sees the drift that would occur, remediates, then flips to `enforce`.
   **Critical for OT environments** where an unexpected hard-deny could disrupt operations.
   **[VERIFIED-WEB — K8s PSA audit-before-enforce pattern]**

4. **Workflow requirements drift:** A profile's `[workflow_requirements]` block that cannot yet
   be satisfied by the current implementation (pre-Day-3 engine) MUST report as `drifting`,
   never as `compliant`. The profile does not degrade silently. **[MODEL — grounded in P-ADS-12
   production-grade fail-safe]**

---

### D-PROF-10 — Profile Authorship Extensibility (SF-PROF-4 DECIDED)

**Decision:** The shipped preset set is **OPEN** (not closed like K8s PSS). MSSPs may author
CUSTOM compliance profiles that:
- Extend a shipped preset (`extends = "iso27001"`, etc.) and add further tighten-only restrictions
- Are Central-authored only (INV-ADS-04; P-ADS-09)
- Still cannot loosen below the base preset or below the INV-ADS invariant floor
- Undergo the same Central authoring-boundary validation (out-of-range = rejection)

This mirrors NIST OSCAL / InSpec / Azure custom initiative / AWS custom conformance pack patterns.
**[VERIFIED-WEB]** It expands the authoring UI and validation surface; this is a deliberate
scope decision, not an accidental one. Custom profiles go through the same sign + version + canary
distribution as shipped presets (PAT-ADS-03 / PAT-ADS-10).

---

### D-PROF-11 — Workflow Contract Surface (Profile Sets Requirements; Day-3 Engine Implements)

**Decision:** A Compliance Profile's `[workflow_requirements]` block STATES requirements;
the Day-3 Workflow Engine (E-WORKFLOW-ENGINE-001 from ADR-PROP-rbac-depth.md §7) IMPLEMENTS them.
The profile does NOT design the engine.

| Requirement key | Meaning | Standards anchor |
|----------------|---------|-----------------|
| `requires_separation_of_duties` | requester ≠ approver ≠ executor enforced in backend | NIST AC-5 |
| `dual_auth_on: [glob]` | listed action-class globs require 2 distinct approvers | NIST AC-3(2) |
| `min_approvals: N` | minimum approval count | C15 ARO `min_approvals` |
| `required_approver_roles: [...]` | approver must hold one of these `(role, scope-node)` bindings | C15 + C19 closure-table-scoped roles |
| `break_glass_alerts_to: [...]` | emergency-path alert targets | C18 §4.4 break-glass |

**Contract guarantee:** When Day-3 engine ships, it MUST refuse to operate (fail-safe, P-ADS-10)
if a `[workflow_requirements]` directive in the active profile is not yet implementable — it does
not silently ignore an unmet requirement. Until Day-3 lands, a profile carrying unsatisfied
`[workflow_requirements]` directives reports those requirements as `drifting` in conformance,
never as `compliant`. **[MODEL — grounded in P-ADS-10/P-ADS-12; Sentinel enforcement-level-is-data
VERIFIED-WEB pattern]**

This is the Sentinel / OSCAL division of labor: the profile is DATA that states the requirement;
the engine is CODE that enforces it. **[VERIFIED-WEB]**

---

### D-PROF-12 — Distribution (rides PAT-ADS-03 + PAT-ADS-10; no new channel)

**Decision:** Compliance Profiles ride the existing signed-bundle + canary mechanism:
- **PAT-ADS-03 Signed-Offline-Bundle:** Profile bundle = Ed25519-signed, content-addressed,
  version-stamped. Satellite verifies signature before applying. Air-gap delivery supported
  (same channel as C9 config + C11 intel feed).
- **PAT-ADS-10 Two-Tier-Canary-Apply:** Profile tightening changes (especially flipping a
  subtree from `soc2` to `nerc-cip` in `enforce` mode) are HIGH-BLAST → deploy with audit/warn
  mode first, then enforce; fast-revert always available.
- **No new distribution channel.** A Compliance Profile is content that rides existing
  infrastructure. **[GROUNDED-INTERNAL — ADS PAT-ADS-03/10; C9 C18 signed bundles]**

---

## 3 — Five Shipped Presets

Each preset is a shippable, signed, versioned profile document. Each is a strict superset of
restrictions compared to the preceding preset (monotone inclusion: `baseline ⊂ soc2 ⊂ iso27001
⊂ iec-62443-ot ⊂ nerc-cip`). **[MODEL grounded in VERIFIED-WEB standards mappings]**

| Preset | Tightens (delta over the preceding preset) | Standards anchor |
|--------|-------------------------------------------|-----------------|
| **`baseline`** | Always-on floor for every tenant. Decision-log OFF by default (tunable ON); staleness tunable `[60, 86400]`; bulk-export approver-gated; destructive actions gated (single approval); `sensitivity:PII` masking ON; mechanism (c) forbidden (AP-ADS-11, restated as global). | Prism floor; CIS L1 spirit |
| **`soc2`** | Forces decision-log ON + retention floor (`[365, ∞]` days tunable); periodic access-review attestation hooks; tighter staleness clamp `[60, 3600]`; provisioning/deprovisioning audit trail. | SOC 2 CC6.1/CC6.3 — least privilege, access reviews **[VERIFIED-WEB]** |
| **`iso27001`** | Adds ISO A.9 access-control-policy attestation; privilege-management review cadence requirement; removal-on-role-change enforcement; staleness `[60, 1800]`. | ISO/IEC 27001 Annex A.9 **[VERIFIED-WEB]** |
| **`iec-62443-ot`** | OT zone/conduit segmentation requirements; **bulk-export hard-locked `hard_block`**; **key-custody forces mechanism (d) BYOC remote-op** for derived-corpus sharing; `dual_auth_on: ["ot.*"]`, `min_approvals: 2`; OT-affecting actions `required_approver_roles: [safety_officer, security_lead]`; staleness on OT actions clamped `[0, 60]`. | ISA/IEC 62443 OT/ICS gating + PAM-brokered sessions **[VERIFIED-WEB]** |
| **`nerc-cip`** | Everything in `iec-62443-ot` PLUS: **OT-action staleness hard-locked `0`** (any staleness hard-denies); **parent-visibility `max_preset` hard-locked `P0`** (forces P3 OFF — C19 §3.8); ESP rules (CIP-005) forced ON; audit retention floor locked to `{ min = 2555 }` days (~7y); exemptions on hard-locks restricted to compliance-authority principal only; access-revocation-program rigor (CIP-004) attestation hook. | NERC CIP-004/005 **[VERIFIED-WEB]** |

> **[INCONCLUSIVE — VERIFIED-WEB, flagged for morph]:** The deep-research pass could not confirm
> an exact NERC CIP-004 revocation deadline (commonly interpreted as same-day / short window;
> sources did not establish an exact number — see OQ-PROF-2). The `nerc-cip` preset makes
> revocation-timeliness a **tunable parameter with a conservative default** rather than
> hardcoding a number the sources did not establish. This requires legal/compliance confirmation
> at morph.

---

## 4 — Invariants

| ID | Invariant |
|---|---|
| **PIV-PROF-1** | A Compliance Profile can only TIGHTEN. The Central authoring engine rejects any profile document that attempts to loosen a setting below its parent preset's lock or range. No runtime overrides. |
| **PIV-PROF-2** | Deployment-profile and Compliance-Profile are NEVER collapsed into a single selector. They share infrastructure (signed bundle + canary) but remain distinct axes with distinct resolution scopes (process vs tenant-node). |
| **PIV-PROF-3** | A non-satisfiable combination (compliance profile demands a capability the deployment profile cannot provide) is a conformance ERROR, not a silent downgrade. The error surfaces at authoring time. |
| **PIV-PROF-4** | `regulatory_class`-forced hard-locks are exemptable only by the compliance-authority principal. Tenant child-admin and MSSP-operator/parent cannot grant exemptions on regulatory hard-locks. |
| **PIV-PROF-5** | Conformance reporting shows a boolean `compliant|drifting` gate plus an itemized drift list. A single flat compliance percentage MUST NOT be the primary conformance output. |
| **PIV-PROF-6** | Unsatisfied `[workflow_requirements]` directives (pending Day-3 engine) report as `drifting`, never `compliant`. The engine does NOT silently ignore them. |
| **PIV-PROF-7** | Profile settings sit ABOVE the INV-ADS invariant floor. The hard invariants (INV-ADS-01..08) are NEVER configurable off — no profile may loosen operator-zero-access-at-rest, per-tenant isolation, no-raw-data-at-Central, or AI-opaque. |
| **PIV-PROF-8** | Custom profiles authored by MSSPs must `extends` a shipped preset and can only add tighten-only restrictions. They undergo the same Central-authoring-boundary validation as shipped presets. |

---

## 5 — Proposed Epic

**E-COMPLIANCE-PROFILES-001** (PROPOSED — not yet in STORY-INDEX): Implement the Compliance
Profile engine.
Covers: profile document schema + validation (TOML/JSON in Config-DB); tighten-only resolver;
`regulatory_class` → profile-selector logic; C19 closure-table fold integration for per-node
effective profile; `{ lock = V }` vs `{ min, max }` enforcement at authoring boundary;
non-satisfiable-combination detection; signed bundle generation + distribution (PAT-ADS-03);
canary-apply integration (PAT-ADS-10); audit/warn `report-only` mode; conformance reporting
per-node (per-setting `compliant|drifting|exempt` + boolean gate + itemized drift list);
waiver/exemption records with `expiresOn`; custom profile authoring path; five shipped presets.

PROPOSED. Not registered in STORY-INDEX. Registration gated on morph execution.

---

## 6 — Open Questions

| ID | Question | Owner | Priority |
|---|---|---|---|
| **OQ-PROF-1** | Exact canary cohort unit for profile tightening: is the canary cohort a percentage of tenant nodes in a subtree, or a dedicated staging tenant? Mirrors the PAT-ADS-10 ambiguity noted in ADS §C.3 (must be resolved before E-COMPLIANCE-PROFILES-001 story decomposition). | architect at morph | P1 |
| **OQ-PROF-2** | NERC CIP-004 revocation deadline: what is the correct numeric floor for `revocation_timeliness` in the `nerc-cip` preset? Sources flagged **[INCONCLUSIVE]**; legal/compliance guidance required. The preset must make this a tunable parameter with a conservative default until confirmed. | legal/compliance at morph | P1 |
| **OQ-PROF-3** | Compliance-authority principal modeling: is the "compliance authority" role a distinct RBAC role in the C18 role model (e.g., `compliance_officer`) or is it an operator-level governance principal separate from the tenant role model? Recommend: a distinct Prism RBAC role `compliance_authority` that only MSSP-operator-level principals can hold (not assignable by tenant child-admin). | product-owner + architect at morph | P1 |
| **OQ-PROF-4** | Custom profile validation tooling: when an MSSP authors a custom profile that `extends` a shipped preset, what is the authoring UX? (a) Raw TOML editor with schema validation; (b) Guided wizard that surfaces only tighten-only knobs within the base preset's declared ranges; (c) Both with different access controls (wizard for tenant-admin, TOML for compliance-authority). Lean: (c). | product-owner + ux-designer at morph | P2 |
| **OQ-PROF-5** | Risk-weighted conformance roll-up: if optional risk-weighted scoring is shipped (beyond the boolean gate + itemized list), what is the weight model? No single flat percentage; but a weighted severity roll-up (CRITICAL drift > HIGH drift > MEDIUM) may be useful for executive dashboards. Flag for ux-designer at morph. | ux-designer at morph | P2 |

---

## 7 — Cross-Wiring with Sibling Features

| Feature | Cross-wire point |
|---|---|
| **C18 RBAC Depth** | The restrictive axes (staleness, action-gating, masking, key-custody) originate from C18 decisions. The Compliance Profile is the MECHANISM that makes those axes configurable. C18's authz engine reads the active profile to determine per-action-class staleness thresholds and masking posture. |
| **C19 Nested Tenancy** | `regulatory_class` becomes a profile selector/floor (D-PROF-6). The C19 closure table is reused for the per-node effective-profile fold (D-PROF-7). C19's parent-deny-is-final + guardrails-intersect semantics ARE the tighten-only rule. C19's ADR-PROP will be lightly amended in Pass 2 to note that `regulatory_class` routes through the profile engine. |
| **C9 Config-DB-Authoritative / Bundles** | Profiles ride PAT-ADS-03 signed bundles (D-PROF-12). The C9 bundle delivery, canary apply (PAT-ADS-10), and fast-revert mechanism are reused without modification. No new distribution channel. |
| **C15 ARO / Day-3 Workflow Engine** | Profile `[workflow_requirements]` is the CONTRACT SURFACE the Day-3 engine must honor. Until Day-3, unsatisfied directives report as `drifting`. The Day-3 engine reads the profile to determine which action-classes require dual-auth, SoD, etc. |
| **C20 NERC-CIP** | NERC CIP-004/005 requirements are expressed as the `profile:nerc-cip` preset. C20 scope maps to the NERC-CIP preset definition. Prism stops having a "C20 code branch" — C20 compliance is achieved by assigning the `nerc-cip` profile. |
| **C16 Masking** | The `[settings.masking]` axis of the profile controls C16's bulk-export posture and masking strictness. C16 provides the masking enforcement mechanism; the profile provides the tenant-configurable-within-bounds posture. |

---

## 8 — ADS Conformance Checklist

```
CONFORMANCE CHECKLIST — ADR-PROP-compliance-profiles.md — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [YES] Compliance Profiles are authored ONLY at the Central UI/DB. No satellite,
        CLI, or git-committed TOML authors profile documents. Profile assignment is
        a Central-authored tenant-node attribute. Satellites receive the FLATTENED
        effective profile in their signed config bundle and never author profiles.

P-ADS-02: Operator-Zero-Access-At-Rest
  [YES] Profile documents are metadata (settings / constraints). No client data
        transits in the profile mechanism. Decision logs produced by the profile
        enforcement engine are encrypted under the tenant DEK (PAT-ADS-02).

P-ADS-03: Derived-Results-Only-At-Central
  [YES] The profile mechanism evaluates metadata (setting values, compliance status).
        No raw sensor data transits as part of profile evaluation. Conformance
        reports at Central describe derived compliance status, not raw data.

P-ADS-06: Per-Tenant-Isolation
  [YES] Each tenant node carries its own `compliance_profile` attribute. The
        effective profile fold is per-node (C19 closure table). No cross-tenant
        profile evaluation. Profile settings do not grant cross-tenant data access.

P-ADS-07: AI-Opaque
  [YES] No AI/ML components are introduced by the Compliance Profile mechanism.
        Profile settings are configuration metadata. AI-opaque invariant unaffected.

P-ADS-09: Config-DB-Authoritative
  [YES] Profile documents stored in Config-DB at Central. Profile assignment and
        custom profile authoring = Central UI/DB operations only. INV-ADS-04
        fully preserved.

P-ADS-10: Idempotent-Gated-Actions
  [YES] Profile assignment changes are Central-authored admin operations subject
        to the gated-action path (P-ADS-10). HIGH-BLAST profile tightening (flipping
        a subtree to nerc-cip enforce mode) deploys via PAT-ADS-10 canary + audit/
        warn preview, then enforce; fast-revert always available.

P-ADS-11: Single-Codebase / Deployment-Profile
  [YES] One profile-resolution + enforcement engine in the Rust binary. OT/NERC-CIP
        behaviors are DATA (seeded profile documents), not code paths. No
        #[cfg(feature="ot")] or #[cfg(feature="nerc_cip")] branches.
  [YES] Deployment-profile and Compliance-Profile are kept as DISTINCT named axes
        sharing infrastructure but never collapsed into one selector (PIV-PROF-2).

P-ADS-12: Production-Grade-Default
  [YES] Out-of-range author attempts are REJECTED (not warned) at the authoring
        boundary — exceeds precedents that rely on human governance (§D-PROF-2).
  [YES] Non-satisfiable-combination = conformance error, not silent downgrade
        (D-PROF-5, PIV-PROF-3).
  [YES] Workflow requirements fail-safe (drifting, not compliant) rather than
        silently ignored (D-PROF-11, PIV-PROF-6).
  [YES] No flat compliance percentage that masks critical drift (D-PROF-9, PIV-PROF-5).

P-ADS-13: Configurable-Not-Prescriptive
  [YES] This mechanism IS the correct expression of P-ADS-13 for the OT-restrictiveness
        concern. OT = profile:iec-62443-ot (data). NERC-CIP = profile:nerc-cip (data).
        No #[cfg] fork. Tighten-only + Central-resolved + client-tunable-within-bounds.

AP-ADS-07: No Per-Deployment-Model Code Forks
  [YES] This ADR-PROP eliminates the OT-code-fork risk. Explicitly conforms to AP-ADS-07.

INV-ADS check (all ten):
  [YES] INV-ADS-01: No raw sensor data at Central — profile mechanism is metadata only
  [YES] INV-ADS-02: Operator zero-access at rest — decision logs encrypted under tenant DEK
  [YES] INV-ADS-03: Per-tenant isolation — per-node profile attribute; no cross-tenant evaluation
  [YES] INV-ADS-04: Config authored only at Central — profiles are Central-DB-authoritative
  [YES] INV-ADS-05: Actions gated — profile assignment changes pass through gated-action path
  [YES] INV-ADS-06: AI-opaque — no AI components introduced by this mechanism
  [YES] INV-ADS-07: OCSF normalization unaffected — no new data sources
  [YES] INV-ADS-08: Air-gap valid — signed bundles + PAT-ADS-03 + SoftwareKms all air-gap
                    compatible; profiles function offline after bundle delivery
  [YES] INV-ADS-09: Decision-level audit — C18 compliance-profiles IS the originating source of
                    INV-ADS-09 (D-C18-7 decision-log requirement). Profile enforcement engine
                    logs every authorization decision with policy-bundle-version, attributes
                    considered, and outcome. Fully satisfied.
  [YES] INV-ADS-10: Recoverability preserves operator-zero-access — profile documents are
                    Config-DB artifacts (not primary tenant data); their backup follows the
                    standard PAT-ADS-15/16 sealed-blob escrow path. Crypto-shred does not
                    conflict with this mechanism. Not violated by C18.
                    (INV-ADS-10 was added by C17, which post-dates this document's original
                    capture; satisfiability confirmed above.)

NOTE: PAT-ADS-12 "Configurable Compliance Profile" was added to ARCHITECTURE-DESIGN-SYSTEM.md
Section B in ADS v1.2 (Pass 2, 2026-06-27). This ADR-PROP is the originating feature for that
pattern. The pattern addition is LANDED and complete.
```

All checklist items PASS.
(Note: INV-ADS-09 was introduced by this document's own D-C18-7; INV-ADS-10 was added by C17
after C18 was originally captured. The "all eight" label in the original capture has been updated
to "all ten" to reflect the current ADS v1.6 invariant set.)

---

## 9 — Pass 2 Flags (status as of 2026-06-27 consistency audit)

The following items were originally flagged for Pass 2 integration. Status is recorded here.

| Item | Target artifact | Status |
|------|----------------|--------|
| **PAT-ADS-12 "Configurable Compliance Profile"** | ARCHITECTURE-DESIGN-SYSTEM.md Section B | EXECUTED — PAT-ADS-12 was added to the ADS in v1.2 (2026-06-27). This ADR-PROP is cited as the originating feature. |
| **INV-ADS-09 "Decision-level audit"** | ARCHITECTURE-DESIGN-SYSTEM.md Section C.1 | EXECUTED — INV-ADS-09 was added to the ADS in v1.2 (2026-06-27), originating from C18 D-C18-7. The conformance checklist in Section C.2 was updated accordingly. |
| **ADR-PROP-nested-tenancy.md §3.8 amendment** | ADR-PROP-nested-tenancy.md | EXECUTED — ADR-PROP-nested-tenancy.md §3.8 contains the correct Compliance-Profile routing language (D-PROF-6 reframe is reflected; `regulatory_class` routes through the profile engine via PAT-ADS-12). |
| **ARCHITECTURE-DESIGN-SYSTEM.md Section E traceability row** | ARCHITECTURE-DESIGN-SYSTEM.md Section E | EXECUTED — `ADR-PROP-compliance-profiles.md` traceability row is present in ADS Section E (added in v1.2). |
| **ARCHITECTURE-DESIGN-SYSTEM.md Section E traceability row** | ARCHITECTURE-DESIGN-SYSTEM.md Section E | EXECUTED — `ADR-PROP-rbac-depth.md` traceability row is present in ADS Section E (added in v1.2). |

---

## 10 — Decision Provenance

| Decision ID | Sub-fork | Human decision |
|-------------|----------|----------------|
| D-PROF-1 | Profile representation | Data in Config-DB; Central-resolved; flattened to satellites |
| D-PROF-2 | Tighten-only monotone | YES — enforced at authoring boundary with programmatic rejection |
| D-PROF-3 | Restrictive axes | Eight axes from C18/C19 (staleness, action-gating, masking, ESP, audit, key-custody, parent-visibility, workflow) |
| D-PROF-4 | Data model | OSCAL parameter + constraint (`{ lock = V }` vs `{ min, max }`) |
| D-PROF-5 | SF-PROF-1 Two-axis model | DECIDED — distinct axes sharing infrastructure; non-satisfiable = error |
| D-PROF-6 | regulatory_class reframe | DECIDED — selector/floor into profile engine; C19 §3.8 semantics preserved |
| D-PROF-7 | C19 closure-table fold | DECIDED — profile fold reuses C19 closure table; zero new index |
| D-PROF-8 | SF-PROF-2 Exemption authority | DECIDED — regulatory hard-locks: compliance-authority principal only; tunable settings: tenant-exemptable with expiring waivers |
| D-PROF-9 | SF-PROF-3 Conformance reporting | DECIDED — boolean gate + itemized drift; NO flat percentage; audit/warn before enforce |
| D-PROF-10 | SF-PROF-4 Custom extensibility | DECIDED — open set, tighten-only, Central-authored; same validation as shipped presets |
| D-PROF-11 | Workflow contract surface | DECIDED — profile states requirements; Day-3 engine implements; fail-safe on unmet requirements |
| D-PROF-12 | Distribution | DECIDED — rides PAT-ADS-03 + PAT-ADS-10; no new channel |
