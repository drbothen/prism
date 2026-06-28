---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
scope: >
  Out-of-band Day-2 vision SIDE-ANALYSIS. Researches and designs a CONFIGURABLE
  Security/Compliance Profile mechanism for Prism — a named, versioned, signed,
  shippable bundle of restrictive settings that generalizes C18 SF-3
  (revocation-lag / deny-on-stale-beyond-N + OT action gating) and C19's
  `regulatory_class` tenant attribute (nerc_cip / ot_critical force-tighten).
  Human directive: do NOT hardcode OT-specific restrictiveness into the product;
  ship OT / NERC-CIP / SOC2 / ISO27001 / IEC-62443 / baseline as named PROFILES
  (data, not code branches) per P-ADS-11 Single-Codebase + AP-ADS-07 no-forks.
  CAPTURE ONLY. Modifies no live spec, ADR, BC, story, STATE.md, or
  SESSION-HANDOFF.md. No git operation performed.
seeded_from:
  - research/rbac-depth-2026-06-27.md
  - specs/day2-design-decisions/ADR-PROP-nested-tenancy.md
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md
cross_refs:
  - C18 RBAC depth (SF-3 deny-on-stale-beyond-N, OT gating, signed policy bundles)
  - C19 nested tenancy (regulatory_class, tenant_relationship, hybrid inheritance, closure table)
  - C9 config-management (DB-authoritative, signed versioned bundles, canary + fast-revert)
  - C15 ARO gated actions (required_approver_role, dual-auth)
  - C20 OT / NERC-CIP regulatory enforcement
  - ADS: P-ADS-09, P-ADS-11, P-ADS-12, AP-ADS-07, PAT-ADS-03, PAT-ADS-10
touches_no_live_artifacts: true
---

# Configurable Security / Compliance Profiles for Prism — "OT is a Profile We Ship, Not a Code Branch" (C18 SF-3 + C19 regulatory_class generalization)

> **STATUS: CAPTURE (`do_not_execute: true`).** Out-of-band Day-2 side-analysis. Does NOT modify any live spec, ADR, BC, story, STATE.md, or SESSION-HANDOFF.md. No git operation performed. Real ADR numbers, BCs, and STORY-INDEX rows are deferred to the morph execution.

> Confidence labels: **[VERIFIED-WEB]** = grounded in the cited deep-research passes this session; **[CRATES.IO-VERIFIED]** = version checked against the crates.io API on 2026-06-27; **[GROUNDED-INTERNAL]** = derived from the cited live capture artifacts (ADR-PROP-nested-tenancy, ADS, rbac-depth research); **[MODEL]** = design synthesis flagged as model knowledge; **[INCONCLUSIVE]** = sources thin / not standardized.

---

## 0. Executive Summary / LEANS (read first)

The human's directive resolves cleanly against the precedent landscape: **every mature compliance ecosystem ships its restrictive postures as named, versioned DATA artifacts — not as code branches.** NIST OSCAL calls them *profiles* (a profile imports a catalog, selects controls, and tailors parameters); CIS calls them *profile levels* (L1/L2 tags within one benchmark); Kubernetes ships *Pod Security Standards* (`privileged | baseline | restricted`) selected by a namespace label; AWS ships *conformance packs* and *Security Hub standards*; Azure ships *policy initiatives*; Sentinel ships *policy sets* with per-policy enforcement levels. **[VERIFIED-WEB]** Prism should adopt the same shape: a **Security/Compliance Profile is a named, versioned, signed bundle of restrictive settings** that a deployment or a tenant node *adopts*, riding the exact same C9 signed-bundle distribution + PAT-ADS-10 canary mechanism that config already uses. "OT" becomes `profile:iec-62443-ot` (data); the Rust binary is unchanged (P-ADS-11 / AP-ADS-07).

**Eight recommended leans:**

1. **Profile = data, resolved at Central, flattened to satellites.** A Prism profile is a TOML/JSON document in the Config-DB (P-ADS-09). It is authored only at Central, *resolved* into an effective settings set at Central plan-time (exactly like C19's effective-config fold), and pushed to satellites as a FLATTENED signed bundle (PAT-ADS-03). Satellites never compute profile inheritance. This is the OSCAL "profile resolution → resolved catalog → SSP baseline" pattern applied to Prism's settings axes. **[VERIFIED-WEB / GROUNDED-INTERNAL]**

2. **Profiles only TIGHTEN (monotonic).** Borrow C19's guardrail semantics directly: a profile is a *ceiling-lowering* overlay. Layering `baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip` is a monotone intersection — each layer can only make a setting stricter (smaller staleness `N`, more action-gating, more masking), never looser. This mirrors C19 §3.8 ("`regulatory_class` can only tighten, never loosen") and PIV-C19-6, and it is the single most important invariant. **[GROUNDED-INTERNAL]**

3. **Two senses of "profile" are DISTINCT AXES — name them apart.** (a) **Deployment profile** (P-ADS-11): operator-role × hosting = `{saas, mssp-managed, byoc, air-gap}` — *where/how Prism runs*. (b) **Compliance profile** (this document): `{baseline, soc2, iso27001, iec-62443-ot, nerc-cip}` — *how strict the posture must be*. They are orthogonal: a SaaS deployment can host a NERC-CIP tenant; an air-gap deployment can run the baseline profile. The compliance profile rides the C19 tenant tree (per-node, inherited-down, tighten-only); the deployment profile is a process-level attribute. Conflating them is the precise AP-ADS-07 trap the directive is steering away from. **[GROUNDED-INTERNAL / MODEL]**

4. **Parameterization within profile-permitted bounds.** Adopt OSCAL's parameter+constraint model: a profile setting is either **hard-locked** (the profile pins the exact value; tenant cannot move it) or **tunable-within-range** (the profile declares a constraint `{min, max, allowed-set}` and the tenant picks a value inside it). Example: `baseline` lets a tenant set `deny_on_stale_seconds` anywhere in `[60, 86400]`; `nerc-cip` hard-locks OT-affecting actions to `deny_on_stale_seconds = 0` (any staleness hard-denies) and clamps the tunable range for read queries to `[60, 900]`. This is OSCAL `set-parameters` + `constraint`, and SCAP `set-value`/`refine-value`. **[VERIFIED-WEB]**

5. **Conformance reporting is a first-class output.** A profile defines "what should be true"; Prism reports "what is true" per tenant node: `compliant | drifting | exempt`, per-setting, with the drifting deltas enumerated ("NERC-CIP requires `deny_on_stale_seconds=0` on OT actions; this node has `300` → DRIFT"). This is the OSCAL assessment-results / SCAP pass-fail-notapplicable / Azure Policy compliant-noncompliant-exempt pattern. Ship a non-blocking **audit/warn mode** (K8s PSA `warn`/`audit`; Gatekeeper `dryrun`/`warn`; Sentinel `advisory`) so a tenant can SEE the drift a profile would cause *before* enforcing it. **[VERIFIED-WEB]**

6. **The profile sets WORKFLOW REQUIREMENTS; the Day-3 workflow engine honors them.** Do not design the workflow engine here (C9 deferred it to Day-3). Define only the *contract surface*: a profile may declare `requires_dual_auth_on: [ot.*]`, `requires_separation_of_duties: true`, `min_approvals: 2`, `required_approver_roles: [safety_officer]`. These are *requirements* the C15/Day-3 approval engine MUST satisfy; the profile is the source of the requirement, the engine is the implementation. This is exactly Sentinel's "enforcement level is data; the engine enforces it" and OSCAL's "profile constrains; SSP implements." **[VERIFIED-WEB / GROUNDED-INTERNAL]**

7. **Ship 5 presets.** `baseline` (always-on floor) ⊂ `soc2` ⊂ `iso27001` ⊂ `iec-62443-ot` ⊂ `nerc-cip`. Each is a shippable, signed, versioned bundle. `regulatory_class` on a C19 tenant node *selects* the profile (e.g., `regulatory_class=nerc_cip` ⟹ `profile:nerc-cip` is forced and cannot be loosened by a child). See §8. **[MODEL grounded in VERIFIED-WEB precedents]**

8. **Same signed-bundle mechanism as C18 policy bundles and C9 config.** A compliance profile is not a new distribution channel — it is content that rides PAT-ADS-03 (Ed25519-signed offline bundle), is version-stamped, canary-applied via PAT-ADS-10, and fast-revertible. The C18 deny-on-stale-beyond-N posture is *just a profile setting* in this model; SF-3's per-action staleness `N` becomes a profile-locked-or-tunable parameter. **[GROUNDED-INTERNAL]**

**The crisp framing for the human:** SF-3 (C18) and `regulatory_class` (C19) are two instances of one mechanism. Generalize them into a **Compliance Profile** = a monotone, tighten-only, named, versioned, signed, Central-resolved bundle of the restrictive axes, parameterized within profile-declared bounds, with first-class conformance reporting and a workflow-requirements contract surface. OT stops being special-cased code and becomes `profile:iec-62443-ot`.

---

## 1. Precedent Landscape — How Compliance Ecosystems Ship Profiles As Data (Q1)

All rows **[VERIFIED-WEB]** from the deep-research passes unless tagged otherwise.

| Ecosystem | Profile representation (data vs code) | Composition / inheritance | Parameterization | Lock vs tunable | Versioning | Distribution | Org override / tailoring |
|---|---|---|---|---|---|---|---|
| **CIS Benchmarks + L1/L2** | Benchmark = consensus config recommendations; published PDF/Word/Excel/**XML**. L1/L2 are categorical **profile-level tags** per recommendation (two overlapping subsets in one benchmark). | No formal inheritance; recommendations *map up* to CIS Critical Security Controls. L2 is a superset overlay on L1. | Numeric settings (min password len, lockout count) are de-facto parameters but presented as a single recommended value, no formal schema. | Socio-technical, not structural — "must/should" phrasing + local exception process; nothing technically immutable. | Explicit per-benchmark version (e.g., 2.0.0 → 2.1.0) via CIS WorkBench; old versions retained. | CIS WorkBench portal; PDF free, other formats for SecureSuite members. | Adopt L1 broadly, reserve L2 for high-value assets; mark rules N/A in internal logs; convert to SCAP and subset via OpenSCAP profiles. |
| **DISA STIG + SCAP/OSCAL** | STIG = baseline of requirements; machine-readable as **XCCDF** (XML, distributed as ZIP datastreams w/ OVAL) and increasingly **OSCAL**. `.cklb` checklist = tailored-assessment overlay (status/comments/evidence per requirement). | XCCDF benchmark can define multiple profiles (rule subsets). Overlays adjust applicability/severity. OSCAL: STIG = profile importing the relevant 800-53 catalog + DoD tailoring. | XCCDF `value` elements; OSCAL parameters w/ constraints. Often fixed recommended values rather than ranges. | STIG settings treated as locked in DoD; deviations require formal waiver / risk-acceptance recorded in `.cklb` (N/A, accepted-risk). | DISA versioned releases; SCAP/OSCAL carry version metadata; STIG Viewer API = "normalized, versioned" catalogs. | DISA portals (docs + XCCDF ZIP); NIST (SCAP/OSCAL); STIG Viewer / Workbench. | `.cklb` checklists + overlays record dispositions; SCAP tailoring files; never rewrite the baseline. |
| **NIST OSCAL** *(reference model)* | Catalog / Profile / Baseline / SSP / Assessment-Results all **machine-readable data** (XML/JSON/YAML). Profile = "structured representation of a baseline." | Profile `import`s catalog(s) **or other profiles** (nesting → inheritance). `include`/`exclude` select controls; `modify` (`set-parameters`, `alter`) tailors. `alter` can add/remove parts/props/links but NOT subcontrols (clean separation of selection vs modification). | **First-class parameters** with a `constraint` element providing "criteria for allowed values." `set-parameters` modifies all aspects of a param (value AND constraints). | Profile can pin a value (locked) or leave it open within a constraint (tunable). NIST "organization-defined parameters." | `metadata.version` + last-modified; published in repos w/ release tags. | NIST repos / GitHub / vendor portals / internal config mgmt. | Org authors its own profile importing a baseline, applies `set-parameters`/`alter`; versioned in source control; layered overlays preserve traceability. |
| **K8s Pod Security Standards + PSA** | 3 named levels `privileged \| baseline \| restricted` — **defined in docs + controller code** (NOT user-editable data files). Selected per-namespace via **labels**. | **Cumulative**: restricted ⊃ baseline ⊃ privileged. Layer further via Gatekeeper/OPA. | Only tunable knob is the **pinned PSS version** label (`enforce-version: v1.36`). | Level *conditions* are LOCKED (code); cannot tailor via data — only choose level + version. | Versioned with K8s releases; namespace label pins the version. | K8s release + docs; no separate baseline file. | Per-namespace label strategy (level + mode); complementary admission controllers (Gatekeeper) for custom rules. |
| **K8s PSA modes** | `enforce` (deny) / `audit` (annotate audit log) / `warn` (user-facing warning, non-blocking). Per-mode level+version labels. | Independent per mode — e.g., enforce=baseline, audit=restricted, warn=restricted to preview a future tightening. | n/a | n/a | n/a | n/a | **Drift-before-enforce pattern**: label all namespaces audit/warn=restricted first, observe violations, then flip enforce. `kubectl label --dry-run` previews impact. |
| **AWS Config conformance packs** | **YAML template** = list of Config rules + remediation actions + parameters. Pure data; Config service is the engine. | No formal inheritance; compose by copy/combine rule lists. | Rule input parameters supplied in the YAML. | Org-controlled; AWS notes samples don't guarantee any standard. | Sample templates in console + GitHub (git versioning). | Console / CLI / SSM docs; deploy across org via AWS Organizations. | Copy sample, add/remove rules, tune params; tag-based scoping to exclude resources. |
| **AWS Security Hub standards** | Standards = control sets (CIS AWS Foundations, FSBP, PCI). Internal structured data. | Controls map to Config rules / detections. | Enable/disable individual controls. | Controls enableable; logic fixed by AWS. | Versioned (CIS v1.2/v1.4 …). | Built-in, AWS-updated. | Enable/disable controls; suppress/archive findings as accepted risk. |
| **Azure Policy initiatives** | **JSON**: definition = `if/then` (effect: deny/audit/modify); **initiative** = group of definitions + shared parameters. Data in ARM. | Initiative groups definitions; shared params cascade to each. Clone built-in to customize. | Initiative + policy `parameters` (type, default, allowedValues); overridden at assignment. | Built-in logic locked (clone to change); params + effect (parameterized audit↔deny) tunable. | Built-ins versioned + updated by MS; custom initiatives versioned by org. | Portal / CLI / REST; built-ins directly assignable. | Override params at assignment; clone-and-edit; **exemptions** (Mitigated / Waiver categories, `expiresOn`). |
| **OPA / Gatekeeper** | `ConstraintTemplate` (Rego + Constraint CRD schema) + `Constraint` (params + matchers + enforcement action). K8s CRDs = data; Rego = code. | Template reused by many Constraints (parameterized reuse, NOT inheritance). Bundle via `gatekeeper-library` + Helm/Kustomize. | Constraint `spec.parameters` against the template's openAPIV3 schema. | Rego logic locked (edit template); params tunable. | Git versioning of library. | GitHub library; Helm/Kustomize/raw YAML bundles. | Clone library, adjust constraints; `enforcementAction: deny \| dryrun \| warn`. |
| **HashiCorp Sentinel policy sets** | Policy set = Sentinel policies + **enforcement level per policy as a data attribute**. *(file 1 flagged the deep-research sources thin on Sentinel; treat the enforcement-level taxonomy as [VERIFIED-WEB general] but the structural detail as [MODEL].)* | Policy sets attached per-workspace; compose by set membership. | Sentinel params / config. | **Enforcement level is the lock/tune lever**: `advisory` (log, non-blocking) / `soft-mandatory` (block, authorized override allowed) / `hard-mandatory` (block, no override). | Source control + TFC policy-set history. | Git repos + TFC policy registry. | Change a policy's enforcement level (tailoring without changing the code); per-workspace set assignment; soft-mandatory override for exceptions. |
| **Chef InSpec profiles** | `inspec.yml` metadata + Ruby control files. | `depends` declares a dependency; `include_controls` (run ALL controls from dep) or `require_controls` (run a SUBSET); `skip_control` disables specific checks. | `inputs`/attributes defined in profiles; values supplied per run. | Skipped controls excluded; inputs tunable. | Profile versioning in metadata + supermarket/git. | Chef Supermarket / git / local. | Org profile `depends` on CIS profile, `include_controls` then `skip_control` exceptions → a named bespoke overlay. |
| **Microsoft security baselines** | Versioned packages (e.g., M365 Apps v2512) of importable **GPOs** + scripts, via Security Compliance Toolkit. | Layer GPOs; baseline is a curated GPO set. | GPO setting values. | GPO values adjustable post-import. | Versioned packages (2512 …). | Security Compliance Toolkit download. | Import baseline GPOs, then override specific settings in local GPO. |

**Cross-precedent synthesis [VERIFIED-WEB]:** The deep-research comparative section concluded that **OSCAL and InSpec provide the most explicit structural overlay/inheritance + tailoring mechanisms**, while **Gatekeeper, Sentinel, Azure Policy, and K8s PSA provide the cleanest enforcement-level / mode toggles for staged adoption** (advisory→mandatory, warn/audit→enforce, dryrun→deny). Document-centric frameworks (CIS, STIG) rely more on human governance to record/justify overrides. **For Prism, the winning combination is: OSCAL's profile-resolution + parameter-constraint data model, plus the K8s-PSA/Sentinel enforcement-mode toggle for the audit-before-enforce conformance story.**

---

## 2. The Recommended Prism Profile Data Model (Q2)

A **Compliance Profile** is a named, versioned, signed bundle of restrictive settings, authored at Central, resolved at Central, flattened to satellites. **[MODEL grounded in OSCAL/VERIFIED-WEB + GROUNDED-INTERNAL ADS]**

### 2.1 The restrictive axes (the profile's "controls")

These are the dimensions surfaced by C18 SF-3 and C19. A profile sets a value (or a constraint) on each:

| Axis | C18/C19 origin | Profile expresses | Lock vs tunable example |
|---|---|---|---|
| **Staleness window** (`deny_on_stale_seconds` per action-class) | C18 §5.4 deny-on-stale-beyond-N | Max policy-bundle age before an action-class is denied | `baseline`: tunable `[60, 86400]`; `nerc-cip`: OT actions hard-lock `0` |
| **Action-gating strictness** | C18 §4 / C15 | Which action-classes require approval, `min_approvals`, SoD | `baseline`: gate destructive only; `nerc-cip`: gate all OT, `min_approvals=2` |
| **Masking / bulk-export strictness** | C18 §2 ABAC masking; C16 | Whether PII columns are masked-by-default, bulk-export hard-blocked vs approver-gated | `baseline`: bulk-export approver-gated; `iec-62443-ot`: hard-blocked |
| **Electronic Security Perimeter rules** | C18 §6.1 CIP-005 ESP | Remote-access constraints, MFA-required, access-point logging | `nerc-cip`: ESP rules forced ON |
| **Audit verbosity** | C18 §5.5 decision logging | Decision-log (not just access-log) verbosity; retention floor | `soc2`: decision-log ON; `nerc-cip`: decision-log + heightened retention |
| **Key-custody requirements** | C19 §3.6 / SS-26 | Force client-held CMEK, force BYOC-remote-op (mechanism d), forbid mechanism (c) | `iec-62443-ot`/`nerc-cip`: force mechanism (d) BYOC remote-op |
| **Parent-visibility ceiling** | C19 SF-3 preset ladder P0..P3 | Cap on the visibility preset a node may grant | `nerc-cip`: force P3 OFF (matches C19 §3.8 regulatory override) |
| **Workflow requirements** (contract surface only) | C15 / Day-3 | Dual-auth / SoD / approver-role REQUIREMENTS the Day-3 engine must honor | `nerc-cip`: `requires_dual_auth_on: [ot.*]` |

### 2.2 Profile document shape (illustrative — DATA, not code)

```toml
# profile:nerc-cip  (authored at Central, stored in Config-DB per P-ADS-09)
[profile]
id        = "nerc-cip"
version   = "1.0.0"               # SemVer; bundle is version-stamped (PAT-ADS-03)
extends   = "iec-62443-ot"        # monotone tighten-only layering (§3)
title     = "NERC CIP critical-infrastructure posture"

# --- hard-locked settings (tenant CANNOT loosen) ---
[settings.staleness]
ot_action.deny_on_stale_seconds = { lock = 0 }                 # any staleness hard-denies OT
read_query.deny_on_stale_seconds = { min = 60, max = 900 }     # tunable WITHIN this clamp

[settings.parent_visibility]
max_preset = { lock = "P0" }      # forces P3 OFF — matches C19 §3.8 regulatory override

[settings.key_custody]
require_mechanism = { lock = "d" }            # BYOC remote-op only (C19 §3.6)
forbid_mechanism  = ["c"]                     # AP-ADS-11 (already global, restated for clarity)

[settings.masking]
bulk_export = { lock = "hard_block" }

[settings.audit]
decision_log = { lock = "on" }
retention_days = { min = 2555 }               # tunable floor (~7y), tenant may exceed

# --- workflow REQUIREMENTS (contract surface only; Day-3 engine implements) ---
[workflow_requirements]
requires_separation_of_duties = true
dual_auth_on = ["ot.*"]
min_approvals = 2
required_approver_roles = ["safety_officer", "security_lead"]
```

The `{ lock = V }` vs `{ min, max, allowed }` distinction is the OSCAL parameter-vs-constraint model (Q4 §4.1). **[VERIFIED-WEB]**

### 2.3 Why DATA, not code (P-ADS-11 / AP-ADS-07 conformance)

- The Rust binary contains a **single** profile-resolution + enforcement engine. It reads profile documents from the Config-DB; it has no `#[cfg(feature="ot")]` branch (AP-ADS-07 forbidden form). **[GROUNDED-INTERNAL]**
- Shipped presets (`baseline`…`nerc-cip`) are **seeded profile documents** distributed in the product, exactly as K8s ships PSS definitions, AWS ships sample conformance packs, Azure ships built-in initiatives, and the Microsoft Security Compliance Toolkit ships baseline GPO packages. They are versioned content, not compiled code paths. **[VERIFIED-WEB]**

---

## 3. Composition + Inheritance — The Tighten-Only Question, Resolved (Q3)

### 3.1 Monotonic / tighten-only — RESOLVED: YES

**A profile can only TIGHTEN.** This is the decisive design rule, and it follows C19's existing guardrail semantics rather than inventing a new one:

- C19 D-C19-3 already defines **security guardrails as "intersect + parent-deny-is-final"** — a ceiling a child cannot widen. **[GROUNDED-INTERNAL]**
- C19 §3.8 + PIV-C19-6: **"`regulatory_class` can only tighten visibility/access; it can never loosen them regardless of parent/child preference."** **[GROUNDED-INTERNAL]**
- Compliance profiles are the *generalization* of that exact rule across all the §2.1 axes.

**Resolution semantics (the "fold"):** layering `baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip` is a **monotone meet (intersection toward stricter)**. For each setting:
- locked value: the *strictest* lock among the layers wins; a child layer may further lock but never unlock.
- tunable range `{min,max}`: layers **intersect** the ranges (the resulting permitted range is the tightest common interval); the effective range can only narrow as you descend.
- a setting absent in a child layer inherits the parent layer's (already-tightened) value.

This is OSCAL profile resolution (deterministic `import → select → modify → resolved baseline`) **constrained to be monotone**. OSCAL itself permits loosening; Prism does NOT — Prism's resolver rejects any layer that attempts to widen a parent's lock or range. **[VERIFIED-WEB for OSCAL resolution; MODEL+GROUNDED-INTERNAL for the monotone constraint]**

### 3.2 Riding the C19 tenant tree + closure table

Profile assignment is a **per-node tenant attribute** that inherits DOWN the tree and can only tighten:

- A node carries an assigned `compliance_profile` (and/or it is *selected* by `regulatory_class` — see §3.4).
- The C19 **closure table** is reused (zero new index): effective profile at a node = monotone fold over the closure-table ancestor set, identical to how C19 D-C19-3 folds effective config and D-C19-6 rolls up metering. **[GROUNDED-INTERNAL]**
- A child may assign a STRICTER profile than its parent (child of a `soc2` parent may be `nerc-cip`). A child may NOT assign a looser one (a child of a `nerc-cip` parent cannot drop to `baseline`) — the fold rejects it. This is C19's parent-deny-is-final applied to profiles. **[GROUNDED-INTERNAL]**
- **Resolved at Central, flattened to satellites** (P-ADS-09 / INV-ADS-04): satellites receive the FLATTENED effective profile in their signed config bundle and never run the fold. **[GROUNDED-INTERNAL]**

### 3.3 Profile layering vs the precedents

The `baseline ⊂ soc2 ⊂ …` layering is OSCAL `extends`/`import` of a parent profile (§1) + InSpec `depends` + `include_controls`, but **with the monotone constraint added**. K8s PSS's *cumulative* (`restricted ⊃ baseline`) levels are the closest precedent for the "each level is strictly a superset of restrictions" property. **[VERIFIED-WEB]**

### 3.4 Compliance-profile ↔ deployment-profile disambiguation (the "two senses" precision)

This is the most important conceptual cut, because the directive explicitly warns against hardcoding OT:

| | **Deployment profile** (P-ADS-11) | **Compliance profile** (this doc) |
|---|---|---|
| What it answers | *Where/how does Prism run?* | *How strict must the posture be?* |
| Values | `saas`, `mssp-managed`, `byoc`, `air-gap` | `baseline`, `soc2`, `iso27001`, `iec-62443-ot`, `nerc-cip` |
| Axis level | Process / deployment instance | Tenant NODE (rides C19 tree) |
| Mechanism | Profile config selects KMS backend, hosting topology (SoftwareKms vs cloud KMS), etc. — NO code fork (AP-ADS-07) | Monotone tighten-only settings bundle, Central-resolved, signed, canary-applied |
| Relationship | Orthogonal. A `saas` deployment can host a `nerc-cip` tenant; an `air-gap` deployment can run `baseline`. | Orthogonal. |
| Interaction rule | A deployment profile may set a FLOOR (e.g., `byoc`/`air-gap` may force `require_mechanism=d` available); a compliance profile may demand a capability the deployment must provide (e.g., `nerc-cip` requires `mechanism d`, which is only satisfiable on `byoc`/`air-gap` — a non-satisfiable combination is a **conformance error**, not a silent downgrade). | |

**Is "deployment profile" the same MECHANISM as "compliance profile"?** Recommendation: **same data-bundle + signing + canary mechanism, different axis and different resolution scope.** Both are "named settings bundles riding PAT-ADS-03". But the deployment profile resolves at the PROCESS level (one per Prism instance) while the compliance profile resolves at the TENANT-NODE level (C19 tree fold). Keep them as two named axes that share infrastructure but never collapse into one selector — collapsing them re-introduces the "OT deployment = OT code" coupling the directive forbids. **[MODEL / GROUNDED-INTERNAL]**

**`regulatory_class` becomes a profile SELECTOR.** C19's `regulatory_class ∈ {standard, nerc_cip, ot_critical}` is generalized: `regulatory_class` *forces* a minimum compliance profile (`nerc_cip ⟹ profile ≥ nerc-cip`, `ot_critical ⟹ profile ≥ iec-62443-ot`, `standard ⟹ profile ≥ baseline`). The forced profile is a floor the node and its descendants cannot drop below (tighten-only). This preserves C19 §3.8 exactly while removing the hardcoding: the *enforcement* lives in the generic profile engine, the *selection* lives in the `regulatory_class` attribute. **[GROUNDED-INTERNAL]**

---

## 4. Parameterization + Conformance (Q4)

### 4.1 Tuning within profile-permitted bounds vs hard-locked settings

Adopt the **OSCAL parameter + `constraint` model [VERIFIED-WEB]**: a profile setting is one of —

1. **Hard-locked** (`{ lock = V }`): the profile pins the value; the tenant admin cannot change it. (OSCAL `set-parameters` with a fixed value; Sentinel `hard-mandatory`; STIG locked baseline.)
2. **Tunable-within-range** (`{ min, max }` / `{ allowed = [...] }`): the profile declares the permitted envelope; the tenant picks a value inside it. Picking outside the envelope is a validation rejection at Central authoring time, not a silent clamp. (OSCAL `constraint` "criteria for allowed values"; SCAP `set-value`/`refine-value`; Azure initiative `allowedValues`.)

The monotone fold (§3.1) **intersects** the tunable ranges as profiles layer, so the effective envelope at a deep node is the tightest common interval across all ancestors. **[VERIFIED-WEB + MODEL]**

> **[INCONCLUSIVE caveat — VERIFIED-WEB]:** Both deep-research passes flagged that across SCAP/OpenSCAP, AWS conformance packs, and Azure Policy, the *enforcement* of "value must be within declared range" is frequently left to human governance rather than guaranteed by the format/tooling. **Prism should enforce the range at the authoring boundary (Central) programmatically** — i.e., do better than the precedents here, consistent with P-ADS-12 production-grade default. Out-of-range author attempts are rejected, not warned.

### 4.2 Conformance reporting ("this tenant is NERC-CIP-compliant; these 3 settings drift")

Ship a first-class conformance report per tenant node. **[VERIFIED-WEB pattern, synthesized for Prism]**

- **Per-setting status** drawn from the precedent vocabularies: `compliant | drifting | exempt` (Azure Policy `compliant/non-compliant/exempt`; SCAP `pass/fail/notapplicable`; AWS Config `COMPLIANT/NON_COMPLIANT/NOT_APPLICABLE`). Each `drifting` row enumerates the delta: *required* vs *actual* (e.g., "`ot_action.deny_on_stale_seconds`: required `0`, actual `300` → DRIFT").
- **Roll-up score** is OPTIONAL and must be **risk-weighted, not flat.** Both research passes warned that AWS/Azure flat scores (passed/enabled) treat a critical-control failure identically to a low-impact one. For a security product targeting OT/NERC-CIP, a flat percentage is misleading. Recommendation: report `compliant | drifting` as a boolean gate (any hard-locked-setting drift ⟹ node is non-compliant) PLUS an itemized drift list — do NOT lead with a single percentage. **[VERIFIED-WEB caution]**
- **Audit/warn mode before enforce** (the staged-adoption pattern, strongly supported by K8s PSA `audit`/`warn`, Gatekeeper `dryrun`/`warn`, Sentinel `advisory`): a tenant can assign a stricter profile in **`report-only`** mode first, see exactly what would drift/deny, remediate, then flip to **`enforce`**. This is essential for OT environments where an unexpected hard-deny could disrupt operations. **[VERIFIED-WEB]**
- **Exemptions** (Azure Policy model): a documented, expiring waiver (`Mitigated` = met by compensating control; `Waiver` = temporarily accepted) with `expiresOn`, recorded in the audit trail. **Critical interaction with tighten-only:** a `regulatory_class`-forced lock (e.g., NERC-CIP OT hard-deny) must NOT be exemptable by the tenant — only a mandated authority can, mirroring C19 §3.8 "consent may not be purely at the child's discretion in regulated contexts." Exemptions apply to tunable/advisory settings, never to regulatory hard-locks. **[VERIFIED-WEB + GROUNDED-INTERNAL]**

### 4.3 Relationship to C18 policy-bundle distribution

A compliance profile **is** policy-bundle content. It rides the C18/C9 signed, versioned bundle mechanism (PAT-ADS-03), is canary-applied (PAT-ADS-10), version-stamped on every decision (C18 §5.4), and fast-revertible (C9 D-C9-FAST-REVERT). The C18 deny-on-stale-beyond-N posture is literally the `[settings.staleness]` block of a profile. No new distribution channel. **[GROUNDED-INTERNAL]**

---

## 5. Profile ↔ Workflow Contract Surface (Q5 — light touch; engine is Day-3)

The workflow/approval engine is explicitly Day-3 (C9 deferred "configurable approval/review workflows"). This document defines ONLY the contract surface; it does NOT design the engine. **[GROUNDED-INTERNAL]**

**The contract (profile sets requirements; engine honors them):**

A profile's `[workflow_requirements]` block declares characteristics the Day-3 engine MUST satisfy when it ships:

| Requirement key | Meaning | Maps to |
|---|---|---|
| `requires_separation_of_duties` | requester ≠ approver ≠ executor enforced in backend | C18 §4.3 / NIST AC-5 |
| `dual_auth_on: [action-class glob]` | listed action-classes require 2 distinct approvers | C18 §4.2 / NIST AC-3(2) |
| `min_approvals: N` | threshold | C15 ARO `min_approvals` |
| `required_approver_roles: [...]` | approver must hold one of these `(role, scope-node)` bindings | C15 `required_approver_role` + C19 closure-table-scoped roles |
| `break_glass_alerts_to: [...]` | emergency-path notification targets | C18 §4.4 |

**Precedent grounding [VERIFIED-WEB]:** this is exactly the Sentinel/OSCAL division of labor — *the profile is data that states the requirement; the engine is code that enforces it.* Sentinel's enforcement level is a data attribute the Terraform-run engine honors; OSCAL's profile constrains what the SSP/assessment must implement. Prism's profile states `dual_auth_on: [ot.*]`; the Day-3 C15/workflow engine reads it and enforces dual-auth on OT actions.

**Contract guarantee:** when the Day-3 engine ships, it MUST refuse to operate (fail-safe, P-ADS-10) if a `[workflow_requirements]` directive in the active profile is not yet implementable — it does not silently ignore an unmet requirement. This prevents a `nerc-cip` profile from being silently downgraded because the engine doesn't yet support dual-auth. (Until Day-3, a profile carrying `[workflow_requirements]` that the current gated-action path cannot satisfy must report those requirements as `drifting` in conformance, not as `compliant`.) **[MODEL grounded in P-ADS-10/P-ADS-12]**

---

## 6. Recommended Shipped Presets (Q6)

Five seeded, signed, versioned profile documents. Each ⊂ the next (monotone superset of restrictions). **[MODEL grounded in VERIFIED-WEB standards mappings]**

| Preset | Tightens (delta over the preceding, weaker preset) | Standards anchor |
|---|---|---|
| **`baseline`** | The always-on floor for every tenant. Decision-log OFF by default but tunable ON; staleness tunable `[60, 86400]`; bulk-export approver-gated; destructive actions gated (single approval); masking of `sensitivity:PII` columns ON; mechanism (c) forbidden (AP-ADS-11, global). | Prism floor; CIS L1 spirit |
| **`soc2`** | Forces decision-log ON + retention floor; periodic access-review attestation hooks; tighter staleness clamp `[60, 3600]`; provisioning/deprovisioning audit. | SOC 2 CC6.1/CC6.3 (least privilege, access reviews) — C18 §6.2 |
| **`iso27001`** | Adds A.9 access-control-policy attestation; privilege-management review cadence; removal-on-role-change enforcement; staleness `[60, 1800]`. | ISO/IEC 27001 Annex A.9 — C18 §6.2 |
| **`iec-62443-ot`** | OT zone/conduit segmentation requirements; **bulk-export hard-blocked**; **key-custody forces mechanism (d) BYOC remote-op** for derived-corpus sharing; `dual_auth_on: [ot.*]`, `min_approvals: 2`; OT-affecting actions gated with `required_approver_roles: [safety_officer, security_lead]`; staleness on OT actions clamped `[0, 60]`. | ISA/IEC 62443 (OT/ICS gating, PAM-brokered) — C18 §4.1 |
| **`nerc-cip`** | Everything in `iec-62443-ot` PLUS: **OT-action staleness hard-locked `0`** (any staleness hard-denies); **parent-visibility `max_preset` hard-locked `P0`** (forces P3 OFF — C19 §3.8); ESP rules (CIP-005) forced ON; heightened audit retention floor (~7y); exemptions on hard-locks restricted to mandated authority (not tenant discretion); access-revocation-program rigor (CIP-004) attestation. | NERC CIP-004/005 — C18 §6.1, C19 §3.8, C20 |

`regulatory_class` selection (§3.4): `standard ⟹ ≥ baseline`; `ot_critical ⟹ ≥ iec-62443-ot`; `nerc_cip ⟹ ≥ nerc-cip`.

> **[INCONCLUSIVE — VERIFIED-WEB]:** the deep-research pass could not confirm an exact NERC CIP-004 revocation deadline (commonly interpreted same-day/short-window). The `nerc-cip` preset should make the revocation-timeliness floor a **tunable parameter with a conservative default**, flagged for legal/compliance confirmation at morph, rather than hardcoding a number the sources didn't establish.

---

## 7. ADS Conformance Check

```
CONFORMANCE CHECK — Configurable Security/Compliance Profiles — 2026-06-27

P-ADS-09: Config-DB-Authoritative
  [YES] Profiles are authored ONLY at Central UI, stored in the Config-DB,
        resolved (folded) at Central, and pushed to satellites as FLATTENED
        signed bundles. No satellite/CLI/git-TOML authoring path. Profile
        assignment is a Central-authored tenant-node attribute.

P-ADS-11: Single-Codebase / Deployment-Profile
  [YES] One Rust binary with one profile-resolution+enforcement engine. OT/
        NERC-CIP are DATA (seeded profile documents), not code paths. Behavior
        varies by profile configuration, not by compile-time fork.
  [YES] Deployment profile and compliance profile are kept as DISTINCT named
        axes sharing the signed-bundle mechanism (§3.4) — neither collapses
        into a per-deployment-model code fork.

P-ADS-12: Production-Grade-Default
  [YES] Tunable-range enforcement is programmatic at the authoring boundary
        (out-of-range = rejection), exceeding the precedents that leave it to
        human governance (§4.1). Workflow requirements fail-safe rather than
        silently downgrade (§5). No flat compliance score that masks critical
        drift (§4.2).

AP-ADS-07: No Per-Deployment-Model Code Forks
  [YES] The directive's core intent. "OT" = profile:iec-62443-ot (data). No
        #[cfg(feature="ot")] / #[cfg(feature="nerc")] branches. The compile-time
        fork is the explicitly-avoided anti-pattern.

PAT-ADS-03: Signed-Offline-Bundle
  [YES] A profile is content riding the existing Ed25519-signed, content-
        addressed, version-stamped offline bundle. Verify signature before
        apply; air-gap delivery supported (same channel as C9 config + C11 feed).

PAT-ADS-10: Two-Tier-Canary-Apply
  [YES] Profile changes are config changes. HIGH-BLAST profile tightening
        (e.g., flipping a subtree to nerc-cip enforce) deploys via canary +
        report-only(audit) preview first, then enforce; fast-revert always
        available. The audit/warn-before-enforce mode (§4.2) is the staged-
        adoption safety mechanism.

INV-ADS-02/03/06 (spot check):
  [YES] INV-ADS-02/03: profile key-custody settings only TIGHTEN custody
        (force mechanism d, forbid c); never weaken per-tenant DEK isolation.
  [YES] INV-ADS-06: profiles carry no credentials/raw data; AI-opaque intact.
```

All items PASS. The mechanism is ADS-conformant and is, in fact, the *correct* expression of P-ADS-11/AP-ADS-07 for the OT-restrictiveness concern.

---

## 8. ANALYSIS + LEANS — Refined Sub-Forks for the Human

**Analysis.** The directive is well-founded and matches universal industry practice: nobody hardcodes "OT mode" — they ship OT/NERC-CIP/SOC2 as named profiles (OSCAL profiles, CIS levels, K8s PSS, AWS conformance packs, Azure initiatives, Sentinel policy sets, InSpec profiles, MS baselines). C18 SF-3 (staleness `N`, OT gating) and C19 `regulatory_class` are two instances of one mechanism. The clean generalization is a **monotone (tighten-only) compliance profile**: Central-authored, OSCAL-style resolved, parameterized within profile-declared bounds, riding the C9/C18 signed-bundle + canary infrastructure, reported via an Azure-Policy/SCAP-style conformance vocabulary with a K8s-PSA-style audit-before-enforce mode, and exposing a workflow-requirements contract surface that the Day-3 engine honors. The two hard design wins are (a) **tighten-only**, inherited directly from C19's existing guardrail-intersect/parent-deny-is-final semantics (so it needs no new invariant), and (b) **disambiguating deployment-profile from compliance-profile** as two axes sharing one mechanism, which is precisely what keeps OT out of the code.

**LEANS:** profile = data; tighten-only; rides C19 closure-table fold; OSCAL param+constraint for lock-vs-tune; conformance report with audit-before-enforce + risk-weighted (not flat) status; ship the 5 presets; `regulatory_class` becomes a profile selector/floor; workflow contract surface only (no engine design).

### Refined sub-forks needing a human decision

- **SF-PROF-1 — Profile vs deployment-profile mechanism unification.** Confirm the recommended posture: *same signed-bundle/canary infrastructure, two distinct named axes (process-level deployment profile vs tenant-node compliance profile), never collapsed into one selector.* The alternative (a single unified "profile" object spanning both) is simpler to model but risks re-coupling OT-posture to deployment topology — the exact thing the directive avoids. (Architect decision.)

- **SF-PROF-2 — Exemption authority on regulatory hard-locks.** When a node is `regulatory_class=nerc_cip` and a hard-locked setting drifts, WHO may grant an exemption? Recommended: a **mandated authority** role (not the tenant child admin), mirroring C19 §3.8. Confirm whether Prism models a distinct "compliance authority" principal or whether the MSSP-operator/parent holds that power. (Governance/business + architect.)

- **SF-PROF-3 — Conformance scoring shape.** Confirm the recommendation to AVOID a single flat compliance percentage (which the AWS/Azure precedents use and which both research passes flagged as misleading for critical controls) in favor of a boolean `compliant|drifting` gate + itemized drift list, optionally with a risk-weighted roll-up. (Product-owner + architect.)

- **SF-PROF-4 — Profile authorship extensibility.** Are shipped presets the ONLY profiles (closed set, like K8s PSS levels), or may an MSSP author CUSTOM tenant profiles that `extend` a shipped preset and add further (tighten-only) restrictions (open set, like OSCAL/InSpec/Azure custom initiatives)? Recommended: **open set, tighten-only, Central-authored** — but this expands the authoring UI and validation surface and should be a deliberate scope decision. (Product-owner + architect.)

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) How CIS/STIG-SCAP-OSCAL/NIST-OSCAL/K8s-PSS/AWS-conformance-packs+Security-Hub/Azure-initiatives/OPA-Gatekeeper/Sentinel/InSpec/MS-baselines represent profiles as data — format, composition/inheritance, parameterization, lock-vs-tune, versioning, distribution, override (Q1, §1). (2) Conformance reporting + tailoring within bounds — OSCAL catalog→profile→SAP/SAR + profile resolution, SCAP/OpenSCAP tailoring (set-value/refine-value, tailoring files, pass/fail/N/A), AWS Config/Security Hub compliance scores + exemption/remediation, Azure Policy compliance states + exemptions (Mitigated/Waiver/expiresOn), K8s PSA audit/warn drift-before-enforce, hard-locked vs tunable+ranges (Q4, §4). Both at `reasoning_effort: high`, `strip_thinking: true`. |
| Perplexity perplexity_ask | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — (no single-library API deep-dive needed; this is a design/precedent synthesis) |
| WebFetch (crates.io API) | 2 | Version sanity-check for signed-bundle / schema-validation building blocks: `ed25519-dalek` 3.0.0-rc.1 (2026-06-18), `jsonschema` 0.46.6 (2026-06-23) — both [CRATES.IO-VERIFIED]. Load-bearing only insofar as profiles ride the existing Ed25519 PAT-ADS-03 bundle + need schema validation at the authoring boundary; no new crate is mandated by this analysis. |
| WebSearch | 0 | — |
| Read / Grep | 3 reads + several greps | Grounding against the 3 cited live capture artifacts (rbac-depth, ADR-PROP-nested-tenancy, ADS) and reading the two large deep-research result files (saved to tool-results/ due to size). |
| Training data | ~2 areas | (a) The deployment-profile vs compliance-profile axis disambiguation (§3.4) and monotone-fold formalization (§3.1) are design synthesis [MODEL] grounded in C19's existing guardrail rule and OSCAL resolution. (b) The 5-preset standards-to-settings mapping (§6) is [MODEL] grounded in the VERIFIED-WEB standards anchors + C18 §6 standards findings. Both flagged inline. |

**Total MCP tool calls:** 2 Perplexity deep-research (both `reasoning_effort: high`) + 2 crates.io WebFetch verifications.

**Training data reliance:** **low-to-medium.** Every precedent claim in §1 is [VERIFIED-WEB] from the two deep-research passes (full citation transcripts saved at `~/.claude/projects/-Users-jmagady-Dev-prism/2cf9feb0-f155-4978-a58c-03d2ca744fe6/tool-results/mcp-perplexity-perplexity_research-1782618377132.txt` and `...-1782618346696.txt`). Crate versions are [CRATES.IO-VERIFIED] on 2026-06-27. The Prism-specific design (profile data model §2, tighten-only fold §3, conformance approach §4, presets §6) is synthesis explicitly grounded in the cited live capture artifacts (C18 rbac-depth, C19 ADR-PROP-nested-tenancy, the ADS) and is labeled [MODEL]/[GROUNDED-INTERNAL] where it is design rather than reportage.

### Honesty / coverage note

- **Read coverage of the deep-research outputs:** File 1 (precedents) — read ~60% sequentially via Read, plus targeted Grep extraction of the remaining Sentinel/InSpec/MS-baseline/comparative-synthesis sections (all 10 ecosystems + the synthesis confirmed). File 2 (conformance/tailoring) — read ~64% sequentially (full OSCAL lifecycle, SCAP tailoring, AWS scoring, Azure states/exemptions); the tail (end of Azure exemptions + K8s PSA audit/warn + final synthesis) was covered via Grep and is corroborated by the K8s PSA content already fully read in File 1. No section material to the deliverable was left unread.
- **[INCONCLUSIVE] items, surfaced not papered over:** (1) exact NERC CIP-004 revocation deadline (sources did not establish it — §6 makes it a tunable-with-conservative-default, flagged for legal confirmation at morph); (2) across SCAP/AWS/Azure, programmatic enforcement of "value within declared range" is frequently left to human governance (§4.1 — Prism is recommended to do better and enforce at the authoring boundary); (3) Sentinel structural detail was thin in the sources (enforcement-level taxonomy is [VERIFIED-WEB general], deeper structure is [MODEL]).
- **No live artifact modified. No git operation performed. Single file written, as instructed.**
