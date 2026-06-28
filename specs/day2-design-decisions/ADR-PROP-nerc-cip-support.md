---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human; SF-2 OPEN pending research)"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Day-2 out-of-band SIDE-ANALYSIS capture (C20 — NERC CIP Support synthesis capstone).
  CAPTURE ONLY. Does NOT modify any live spec, ADR-registry artifact (specs/architecture/),
  BC, story, STATE.md, or SESSION-HANDOFF.md. No git operation performed.
  Real ADR numbers and formal ARCH-INDEX.md rows deferred to the morph execution.
  touches_no_live_artifacts: true
seeded_from:
  - research/nerc-cip-support-2026-06-27.md (PRIMARY — three perplexity_research sonar-deep-research calls at reasoning_effort=high covering: (1) CIP standard-by-standard mapping onto a monitoring/query tool; (2) BCSI + cloud/Project-2023-09; (3) CIP-010 + CIP-013 + audit/RSAW/log-retention evidence)
cross_refs:
  - specs/day2-design-decisions/ADR-PROP-entity-masking.md (C16 — RSI clearing house, BCSI first profile)
  - specs/day2-design-decisions/ADR-PROP-backup-recovery.md (C17 — CIP-009 evidence, deferred RSAW export)
  - specs/day2-design-decisions/ADR-PROP-rbac-depth.md (C18 — RBAC at BCSI-category granularity, decision-level audit)
  - specs/day2-design-decisions/ADR-PROP-nested-tenancy.md (C19 — regulatory_class, CIP-002 impact-level/entity/site boundaries)
  - specs/day2-design-decisions/ADR-PROP-compliance-profiles.md (nerc-cip Profile preset, monotone tighten-only chain)
  - specs/day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — edge EACMS-inside-ESP, passive/one-way/data-diode edge mode)
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-09/11/12/13; PAT-ADS-12 + PAT-ADS-17; INV-ADS-01/02/09/10)
  - specs/matured-vision-day2-requirements.md §16.4 (C20 running log)
  - CLAUDE.md §Conventions AD-017 (AI-opaque credentials, no undisclosed remote access)
  - CLAUDE.md §Conventions ADR-022 (Arc-DI wiring contract — no hidden control paths)
  - research/nerc-project-2023-09-cloud-bes-2026-06-27.md (SF-2 concurrent research — DO NOT TOUCH — owned by research-agent)
---

# ADR-PROP — NERC CIP Support: Compliance Posture & Synthesis (C20)

> **STATUS: CAPTURE 2026-06-27 (human decisions; SF-2 OPEN pending research).**
> C20 is the synthesis capstone of the pre-B Day-2 feature track. It assembles and
> cross-wires C2, C16, C17, C18, C19 into a coherent NERC CIP compliance posture.
> SF-1/SF-3/SF-4 and all product invariants are DECIDED. SF-2 (cloud-BES-future
> forward-scaffolding question) is OPEN — a concurrent research pass is in flight
> (`research/nerc-project-2023-09-cloud-bes-2026-06-27.md`); this ADR-PROP will be
> amended once that research resolves. Do not fold SF-2 until that file is complete.
>
> CAPTURE artifact (`do_not_execute: true`). Real ADR numbers and formal
> ARCH-INDEX.md rows deferred to morph execution.

---

## 1 — Context: What "Supporting NERC CIP" Means for Prism

**There is no "NERC CIP certification" for software.** Compliance is the obligation
of the registered entity (utility/operator), not the tool vendor. NERC enforces
against the entity; a tool either (a) is a CIP-in-scope asset that must itself meet
applicable requirements, and/or (b) generates evidence the entity uses to demonstrate
its own compliance. "Prism supports NERC CIP" must be read through both lenses.

### Target posture: "CIP-deployable + CIP-evidence-generating"

Prism's value proposition against NERC CIP splits into two distinct senses:

1. **Deployable-without-breaking-the-entity's-compliance** (CIP-002/005/007/011/013):
   Prism must be classifiable, ESP-fitting, hardenable, BCSI-protecting, and
   supply-chain-clean. An operator deploying Prism does not incur new CIP violations.

2. **Compliance-accelerating evidence generation** (CIP-004/008/009/010 + audit
   evidence): Prism generates the per-requirement artifacts operators otherwise
   assemble by hand. This is the product differentiator.

The target posture is explicitly **NOT "CIP-certified"** — that certification does
not exist for a tool. The marketing claim is "CIP-deployable and CIP-evidence-
generating," which is accurate, auditor-credible, and achievable.

---

## 2 — Key Regulatory Anchor: The CIP-004-7 / CIP-011-3 January-2024 Pivot

This is the single most decision-critical regulatory fact for the Day-2 roadmap.

**NERC Project 2019-02 ("BES Cyber System Information Access Management"),**
FERC docket **RD21-6-000**, letter order accession **20211207-3062** (Dec 7, 2021).

- **CIP-004-7** effective **January 1, 2024** — adds R6 (BCSI access as provisioned
  privilege: authorize / review / revoke, mirroring BCS access itself).
- **CIP-011-3** effective **January 1, 2024** (aligned with CIP-004-7; INFERRED-but-
  strongly-supported by NERC Project 2019-02 package; not independently re-confirmed
  in a second source — **re-verify against live NERC standard page at morph**).

**The conceptual shift:** old CIP-011-2 required identifying and securing specific
servers/shares where BCSI lived ("designated storage location" model). New model:
protect BCSI **wherever it resides** via file-level encryption + permissions + access
management. BCSI access is a *provisioned privilege* under CIP-004-7 R6 + CIP-011-3 R1.

**Net regulatory effect:** encrypted cloud / third-party storage of BCSI is now
explicitly permissible — **provided the registered entity holds the keys and the
provider has zero plaintext access**. This is the regulatory blessing for Prism's
BYOC zero-access central plane (INV-ADS-01/02 + Option-3).

### Enforceable-version snapshot (as of 2026-06-27)

> ALL dates are "as of 2026-06-27" from Perplexity sonar-deep-research passes.
> Re-verify every item against the live NERC standard page before treating as a
> load-bearing architecture gate. NERC standard versions and effective dates change.

| Standard | Subject | Current enforceable version | Effective date | Notes / pending |
|---|---|---|---|---|
| CIP-002 | BES Cyber System categorization (High/Med/Low) | CIP-002-5.1a | in force, stable | No CIP-002-6 found. |
| CIP-003 | Security Management Controls | CIP-003-9 | **April 1, 2026** | Filed Dec 2022, FERC order Mar 16 2023. |
| CIP-004 | Personnel, Training, Access Mgmt, **BCSI access R6** | **CIP-004-7** | **January 1, 2024** | Project 2019-02; docket RD21-6-000. |
| CIP-005 | Electronic Security Perimeter / remote access | CIP-005-7 | **October 1, 2022** | Filed Dec 14 2020, FERC order Mar 18 2021. |
| CIP-007 | System Security Mgmt (patching, ports, logging) | CIP-007-6 | **July 1, 2016** | Inactive date Jun 30 2028; CIP-007-7 expected ~2028. |
| CIP-008 | Incident reporting & response | CIP-008-7 | in force | CIP-008-7.1 (docket RM24-8-000) effective **July 1, 2028**. |
| CIP-009 | Recovery plans for BES Cyber Systems | CIP-009-6 | **July 1, 2016** | Inactive date Jun 30 2028. |
| CIP-010 | Config change mgmt + vuln assessment | CIP-010-4 (→5) | Inactive 04/24/2026; CIP-010-5 effective **July 1, 2028** | Transition window Apr 2026→Jul 2028 is **UNCONFIRMED in detail** — verify NERC implementation plan. |
| CIP-011 | Information protection — **BCSI** | **CIP-011-3** | **January 1, 2024** (INFERRED, aligned w/ CIP-004-7) | Project 2019-02, docket RD21-6-000. |
| CIP-013 | Supply-chain risk management | CIP-013-2 (→3) | CIP-013-2: Oct 1 2022; CIP-013-3 compliance effective date **UNCONFIRMED** (likely 2028) | CIP-013-3 filed Jul 10 2024, FERC order Mar 19 2026. |

---

## 3 — Synthesis Map: C20 Assembles Prior Decisions

C20 does not introduce new architectural components. It shows how decisions already
made in C2/C16/C17/C18/C19 assemble into a coherent CIP compliance posture.

| CIP Standard | Prism Prior Decision | How It Satisfies CIP |
|---|---|---|
| **CIP-011 (BCSI)** | **C16** — RSI Clearing House; BCSI is the first RSI profile; entity-key zero-access at rest (INV-ADS-02); operator holds ciphertext only; edge tokenizing before transit to Central (PAT-ADS-14) | CIP-011-3 R1 information protection: data-centric encryption + entity-controlled keys. "Designated storage location" model replaced by provisioned-access + BYOC zero-access — exactly the Option-3/C16 design. |
| **CIP-009 (Recovery)** | **C17** — recovery-test evidence first-class (D-C17-CIP009); integrity-verified backups; sealed-blob key escrow; post-restore CIP-010 baseline diff (PAT-ADS-15/16) | CIP-009-6 R1/R2/R3: documented recovery plans, integrity verification of backups, periodic recovery testing with **retained evidence**. Prism generates those records. |
| **CIP-004/005/007 (Access + Logging)** | **C18** — Layered RBAC at BCSI-category granularity (PAT-ADS-13); decision-level audit (INV-ADS-09); distributed via `nerc-cip` Compliance-Profile preset (PAT-ADS-12) | CIP-004-7 R4/R5/R6: authorize/review/revoke access, BCSI access as provisioned privilege. CIP-007-6 R4: ≥90-day online log retention + long-term archive, review/alerting. |
| **CIP-002 (Classification boundaries)** | **C19** — `regulatory_class` tenant attribute; nested scopes via closure table; CIP impact-level / registered-entity / site mapped to tenancy boundaries (`ADR-PROP-nested-tenancy.md`) | CIP-002-5.1a: BES Cyber System categorization maps to Prism's tenancy boundary = one CIP entity / one site / one impact-level classification. |
| **CIP-005 (ESP + Passive Edge)** | **C2** — edge nodes are EACMS-inside-ESP; passive/one-way/data-diode edge mode; central plane = zero-access ciphertext + metadata aggregator, never plaintext BCSI, never an EACMS (`ADR-PROP-satellite-mesh.md`) | CIP-005-7: ESP-fitting deployment. Edge node connects inside ESP via EAP. Central plane stays out of the unsettled Project-2023-09 zone (see SF-2 and §OQ-C20-CLOUD-FUTURE). |

**Naming note:** BCSI is the canonical NERC Glossary term. The C16 abstraction
layer uses "RSI (Regulated Sensitive Information)" as the sector-neutral type with
BCSI as the first concrete profile — this is already decided in C16 and is not
reopened here. At the CIP-compliance surface, "BCSI" is the correct audit-recognized
term. Do not bake "BCSI" into Rust type names or API surface; bake "RSI" there and
map BCSI → RSI::BCSI at the profile layer.

---

## 4 — Decisions

### D-C20-SF1 — Build First-Class CIP Audit-Evidence / RSAW-Export Module

**Status: DECIDED (2026-06-27, human).**

Build a first-class CIP audit-evidence / RSAW-export module. This is the
**superset** of C17's deferred RSAW export (D-C17-CIP009 deferred packaging to
C20) and C10's GAP-Q2 evidence-package lean.

**Scope includes:**

1. **Audit substrate** (already decided across C17/C18): tamper-evident,
   long-retention, provenance-tagged audit record. Decision-level audit logs
   (INV-ADS-09), integrity-signed backup records (INV-ADS-10), CIP-007 event
   categories (logons, privilege changes, config changes). This substrate is not
   new — it is the C17/C18 output.

2. **Dedicated RSAW-aligned evidence export module** (the C20 addition): a module
   that reads the audit substrate and emits per-requirement, standard-aligned
   evidence bundles consumable by CIP auditors and by external GRC tools. Coverage:
   - **CIP-004:** access-review logs (who provisioned/reviewed/revoked BCSI access)
   - **CIP-007:** ≥90-day online log-retrieval attestations + long-term archive records
   - **CIP-009:** recovery-test records + post-restore CIP-010 baseline diffs
   - **CIP-010:** config baseline snapshots, unauthorized-change detection trails,
     15-month vuln-cadence attestations, control-effectiveness reports
   - **CIP-013:** software-integrity verification logs + supply-chain attestations

3. **GRC-consumable output format:** bundles are also parseable by external GRC
   platforms (OSCAL-compatible where applicable; RSAW-structured otherwise).

**Phasing:** The substrate is built within C17/C18 stories. The dedicated export
module is its own story — proposed epic **E-CIP-EVIDENCE-EXPORT-001**. Substrate
first; export module second. Do not block C17/C18 delivery on the export module.

**Ties:** C10 GAP-Q2 evidence-package; C17 D-C17-CIP009 deferred packaging;
C18 decision-level audit (INV-ADS-09); CIP-007-6 R4 90-day retention.

---

### D-C20-SF3 — Lighter CIP Classification by Default (Passive-Read-Only Edge)

**Status: DECIDED (2026-06-27, human).**

Design Prism for the **lighter CIP classification by default**: the default edge
posture is **passive read-only** (no control path → avoids being pulled into full
BES Cyber System or EACMS classification at the heavier CIP weight).

Write/control features (write-capable sensor APIs, C15 ARO actions, C14 active-query
writes) are **FEATURE-FLAGGED OFF by default** so the operator explicitly chooses to
take on the heavier EACMS/BCS CIP weight only when enabling them. This matches:
- The existing Prism feature-flag model (CLAUDE.md project memory: full sensor API
  including writes locked behind robust feature flag system)
- P-ADS-11 Single-Codebase / Deployment-Profile (flags, not forks)
- C14 read-only perimeter as the default
- The existing feature-flag semantics already committed

**PIV-C20-001 — No Hidden Control Paths.**
No hidden control paths that silently escalate CIP classification. A feature that
adds a write or command path MUST reflect that escalation in the deployment profile
documentation and must be gated by an explicit feature flag. Silent capability
escalation violates both this invariant and AD-017.

**Caveat — "Read-Only Is NOT a Safe Harbor":** CIP-002 uses *reasonable
foreseeability*. Passive nodes that aggregate BCSI (configs, topology, baselines)
still appear in ESP documentation and still hold BCSI under CIP-011. Passive
read-only reduces classification risk but does NOT eliminate CIP scope. The operator
must still classify Prism appropriately per CIP-002 given their specific deployment.
This invariant and the caveat must both appear in operator-facing deployment docs.

---

### D-C20-SF4 — Support Both CIP-010 Positionings

**Status: DECIDED (2026-06-27, human).**

Support **both CIP-010 positionings** in the same codebase (P-ADS-11):

1. **Prism as the CIP-010 asset-management-system-of-record:** commissioning config
   snapshot tagged "CIP Baseline," retained history, continuous unauthorized-change
   detection (hash/param compare/topology diff against known-good baseline), 15-month
   vuln-cadence enforcement (per-asset last-assessment-date tracking + alert before
   window closes), control-effectiveness reports.

2. **Prism as a feeder into an existing operator CMDB:** export baselines + change
   records in a structured format consumable by external asset-management systems.

Both positionings are configurable per deployment (P-ADS-13). Go-to-market emphasis
is a human/business decision deferred to morph; see OQ-C20-GTM below.

---

### D-C20-CIP013 — CIP-013 Vendor Commitments (Prism-as-Product)

**Status: DECIDED (2026-06-27, human).**

As a CIP-013 supply-chain subject (operators running Prism must include it in their
supply-chain risk-management program), Prism makes the following product commitments.
These are PIV-C20 product invariants — they must hold before any NERC-CIP-regulated
customer deployment.

**PIV-C20-002 — SBOM Published.**
A software bill of materials for the Prism binary and the full Rust dependency tree
must be published with each release. Enables operators to cross-reference their
CVE/KEV feeds against Prism's dependency surface.

**PIV-C20-003 — Cryptographically Signed Releases.**
All Prism releases are signed; hashes are published via a secure channel. Operators
can verify software integrity and authenticity before deployment — directly satisfying
CIP-013-2 R2 (software integrity/authenticity verification).

**PIV-C20-004 — Mature Vulnerability Disclosure + Timely Patch Policy.**
Published CVD/patch policy enabling the operator to answer the NATF ESSCR
(Electric Sector Supply Chain Risk Questionnaire). Patch SLA tiers to be determined
at morph; the commitment is that the policy exists and is public.

**PIV-C20-005 — No Undisclosed or Hard-Coded Remote Access.**
No undisclosed, hard-coded, or non-revocable remote access channel exists in Prism.
Any support, telemetry, or diagnostic channel is:
- Fully documented
- Configurable (operator may disable or restrict)
- Revocable on demand
- Audited (appears in the decision-level audit trail)

This aligns with AD-017 (CLAUDE.md §Conventions: credentials never transit AI
context), ADR-022 (Arc-DI wiring contract: no hidden wiring), and PIV-C20-001
(no hidden control paths). NATF-ESSCR-answerable posture.

---

## 5 — Open Questions

### OQ-C20-CLOUD-FUTURE — SF-2: Cloud-BES Future Scaffolding (OPEN)

**Status: OPEN — concurrent research in flight.**

A concurrent research pass (`research/nerc-project-2023-09-cloud-bes-2026-06-27.md`)
is investigating NERC Project 2023-09 "Risk Management for Third-Party Cloud Services"
and its implications for whether Prism should design forward-scaffolding now for the
speculative "EACMS-in-cloud" future.

**What is already LOCKED (PIV-C20-006 — Entity-Key Zero-Access is a Hard Invariant):**
The central plane NEVER holds plaintext BCSI. The entity holds keys. Zero plaintext
access at Central is required by CIP-011-3 for compliant cloud-BCSI storage and is
independently required by INV-ADS-01/02 and Option-3. This is locked regardless of
Project-2023-09 outcome — it is not the open question.

**The OPEN question:** Do we design forward-scaffolding NOW for the speculative
Project-2023-09 "EACMS-in-cloud" future enablement, or defer until that standard
is enforceable? As of 2026-06-27, Project-2023-09 is in flight, not yet enforceable,
and its specific clauses and effective dates are UNCONFIRMED.

**Lean (provisional, pending research):** Defer speculative forward-scaffolding.
The production-grade principle does not require building on unenforceable speculative
standards; it requires building what exists correctly today. Design the entity-key
zero-access invariant (already done); do not pre-build EACMS-in-cloud abstractions
that may never apply. This is a "defer entire speculative feature" which is
acceptable under the Canonical Principle — it is NOT a within-feature shortcut.

**Resolution path:** Research agent reports findings in
`research/nerc-project-2023-09-cloud-bes-2026-06-27.md`. Architect folds SF-2
by amending this section with the decision. State-manager commits both in one burst.

---

### OQ-C20-DATES — Unconfirmed Standard Dates (Re-verify at Morph)

The following items are UNCONFIRMED or INFERRED and must be re-verified against
the live NERC standard pages before any of them is treated as a load-bearing
architecture gate:

- CIP-011-3 exact effective date (INFERRED Jan 1 2024, aligned with CIP-004-7 per
  Project 2019-02 package — strongly supported but independently unconfirmed)
- CIP-010-4→CIP-010-5 transition behavior in the Apr 2026→Jul 2028 window
- CIP-013-3 compliance effective date (likely 2028; UNCONFIRMED)
- CIP-010 R4 exact cadence (commonly cited ~36 months; UNCONFIRMED from NERC source)
- NERC Project-2023-09 specific clauses, timeline, and effective dates

---

### OQ-C20-CIP009-SELF — Prism's Own CIP-009 Recovery Plan

If Prism is classified as a BCS or EACMS, Prism itself needs a documented recovery
plan (not just the evidence it generates FOR the operator's recovery plan). C17
covers Prism's own recoverability (config-as-data, deterministic re-provision,
integrity-verified backups, restore testing). The open question is whether to
produce a CIP-009-formatted recovery-plan template for Prism operators, and whether
to automate the evidence for that plan from Prism's own backup/restore telemetry.
Resolution at morph (E-CIP-EVIDENCE-EXPORT-001 scope decision).

---

### OQ-C20-GTM — Go-to-Market Emphasis for CIP-010 Positioning

D-C20-SF4 decided to support both CIP-010 positionings (system-of-record AND CMDB
feeder). The business/go-to-market emphasis — which to lead with, which to price
separately, which to demo first — is a human decision deferred to morph.

---

## 6 — PIV-C20 Product Invariants (Summary)

| ID | Invariant | Source |
|---|---|---|
| PIV-C20-001 | No hidden control paths that silently escalate CIP classification | D-C20-SF3 |
| PIV-C20-002 | SBOM published with each release | D-C20-CIP013 |
| PIV-C20-003 | All releases are cryptographically signed; hashes published via secure channel | D-C20-CIP013 |
| PIV-C20-004 | Mature CVD + timely-patch policy published; NATF-ESSCR-answerable | D-C20-CIP013 |
| PIV-C20-005 | No undisclosed/hard-coded/non-revocable remote access channel | D-C20-CIP013 |
| PIV-C20-006 | Central plane NEVER holds plaintext BCSI; entity holds keys (zero-access invariant) | OQ-C20-CLOUD-FUTURE (LOCKED portion), INV-ADS-01/02, Option-3 |

---

## 7 — ADS Conformance Checklist (Section C.2 — C20)

C20 is a synthesis document; it introduces no new modules. All checks pass because
the underlying architecture (C2/C16/C17/C18/C19) already passed their individual
conformance checks. C20 adds the `nerc-cip` Compliance Profile preset (PAT-ADS-12)
and the CIP audit-evidence export pattern (PAT-ADS-17, new — see ADS v1.5).

```
CONFORMANCE CHECKLIST — ADR-PROP-nerc-cip-support.md (C20) — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [YES] Every user-interaction path terminates at Central.
  [YES] Satellites (edge nodes inside ESP) are strictly headless.

P-ADS-02: Operator-Zero-Access-At-Rest
  [YES] All derived results at Central encrypted under tenant-held CMEK (INV-ADS-02).
        PIV-C20-006 extends this: BCSI is never plaintext at Central.

P-ADS-03: Derived-Results-Only-At-Central
  [YES] C16 clearing house enforces masking before Central transit; BCSI
        tokens (not plaintext) transit the conduit.

P-ADS-04: Tenant-Keyed-Central-Persistence
  [YES] All Central-cached results use RocksDB/Iceberg with per-tenant DEK.
        PostgreSQL is control-plane only.

P-ADS-06: Per-Tenant-Isolation
  [YES] CIP-002 tenancy boundaries map to C19 per-OrgId scope isolation.

P-ADS-07: AI-Opaque
  [YES] C16 clearing house + dual-index ensures AI backends never see plaintext
        BCSI. PIV-C20-001 extends: no hidden paths that escalate classification.

P-ADS-08: OCSF-Normalize-At-Boundary
  [YES] All sensor adapters normalize at boundary; no trusted-source exemption.

P-ADS-09: Config-DB-Authoritative
  [YES] nerc-cip Compliance Profile is Central-authored + signed-bundle distributed.

P-ADS-10: Idempotent-Gated-Actions
  [YES] Write/control features feature-flagged off by default (D-C20-SF3).
        All write paths carry idempotency keys (P-ADS-10 / INV-ADS-05).

INV-ADS check (all ten):
  [YES] INV-ADS-01: No raw BCSI at Central (C16 clearing house + PIV-C20-006)
  [YES] INV-ADS-02: Operator zero-access at rest (Option-3 + PIV-C20-006)
  [YES] INV-ADS-03: Per-tenant isolation (C19 closure table, OrgId partition)
  [YES] INV-ADS-04: Config authored only at Central (C9 + nerc-cip Profile)
  [YES] INV-ADS-05: Actions gated and idempotent (D-C20-SF3 feature flags)
  [YES] INV-ADS-06: AI-opaque (C16 clearing house, AD-017, PIV-C20-005)
  [YES] INV-ADS-07: OCSF normalization at all boundaries (no exemption)
  [YES] INV-ADS-08: Air-gap deployment valid (PAT-ADS-03 signed bundles; offline RSAW
        evidence bundles carry hashes for offline verification)
  [YES] INV-ADS-09: Authorization decisions logged at decision-resolution time (C18
        decision-level audit; CIP-004-7 R6 BCSI-access log generated by this substrate)
  [YES] INV-ADS-10: Backups CMK-encrypted; sealed-blob key escrow; crypto-shred erasure
        (C17 PAT-ADS-16 + INV-ADS-10; CIP-011-3 entity-held-key zero-access aligns)
```

**Conformance result:** PASS. C20 is the synthesis capstone — it introduces the
`nerc-cip` Compliance Profile preset and the CIP evidence-export pattern; no new
INV-ADS violations introduced.

---

## 8 — Cross-Wiring Index

| Feature | Connection |
|---|---|
| C2 (Satellite Mesh) | Edge nodes are EACMS-inside-ESP; passive/one-way/data-diode edge mode; central = zero-access ciphertext aggregator, never an EACMS |
| C10 GAP-Q2 | Evidence-package lean now resolved: D-C20-SF1 decides build CIP audit-evidence export module (E-CIP-EVIDENCE-EXPORT-001) |
| C14 (Read-Only Perimeter) | Default read-only matches D-C20-SF3 lighter-classification-by-default |
| C15 (ARO Loop) | Write/control features feature-flagged off by default (D-C20-SF3); EACMS-class weight opt-in only |
| C16 (RSI / Entity Masking) | CIP-011 BCSI spine; BCSI = first RSI profile; entity-key zero-access; clearing house is the structural enforcement of CIP-011-3 R1 |
| C17 (Backup & Recovery) | CIP-009 recovery evidence first-class (D-C17-CIP009); RSAW export packaging consolidated here (E-CIP-EVIDENCE-EXPORT-001); integrity-verified backups + post-restore baseline diff |
| C18 (RBAC Depth) | RBAC at BCSI-category granularity; decision-level audit (INV-ADS-09); distributed via `nerc-cip` Compliance Profile preset |
| C19 (Nested Tenancy) | CIP-002 impact-level / registered-entity / site boundaries → C19 tenancy boundaries; `regulatory_class` maps to Profile selector |
| Compliance Profiles (nerc-cip preset) | Five-preset chain: baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip. nerc-cip is the highest-restriction shipped preset; tighten-only from iec-62443-ot |
| Option-3 / SS-26 | Entity-held-key zero-access for BCSI at Central = the CIP-011-3 compliant cloud-BCSI storage path |
| AD-017 | No undisclosed remote access = PIV-C20-005; AI-opaque = INV-ADS-06 |
| ADR-022 | Arc-DI wiring contract; no hidden wiring = no hidden control paths (PIV-C20-001) |
