---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
scope: >
  C19 SF-3 — parent->child visibility under nested tenancy. Refines the open sub-fork
  SF-3 from research/nested-tenancy-2026-06-27.md (base C19 research), which left the
  parent-aggregate-visibility default as a human decision. This pass researches how to
  support the FULLER end of the spectrum the human asked for — configurable visibility
  ranging from "consented derived metrics only" (conservative default) UP TO "full
  transparent subtree visibility, parent sees everything below as if the subtree were
  part of its own tenant" — WITHOUT breaking the ADS isolation/zero-access invariants
  (P-ADS-02 Operator-Zero-Access-At-Rest, P-ADS-03 Derived-Results-Only-At-Central,
  P-ADS-04 Tenant-Keyed-Central-Persistence/Option-3, P-ADS-06 Per-Tenant-Isolation,
  INV-ADS-01/02/03, AP-ADS-01/03/05).
provenance: >
  Day-2 out-of-band SIDE-ANALYSIS input. Cited research pass to inform the human's SF-3
  decision. ANALYSIS/CAPTURE ONLY. Does NOT modify any live spec, ADR, BC, story,
  STATE.md, or SESSION-HANDOFF.md. No git operation performed. Section numbering internal
  to this doc. Framed for ADS conformance — every recommendation is checked against the
  named ADS principles/invariants/anti-patterns.
traces_to:
  - research/nested-tenancy-2026-06-27.md §SF-3 (open sub-fork — parent aggregate-visibility default)
  - research/nested-tenancy-2026-06-27.md Topic 5 (isolation invariants + key custody) / Topic 6 (metering)
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-02/03/04/06, INV-ADS-01/02/03, AP-ADS-01/03/05)
  - project memory AD-017 (credentials never transit AI context), BYOC zero-access thesis
  - D-C2-12 (Central never stores raw sensor data — raw stays at edge/satellite) / INV-ADS-01
  - Option-3 = per-tenant(child)-keyed central cache; OrgId/OrgSlug tenant abstraction
  - C20 cross-link (OT / NERC-CIP regulatory context for parent visibility)
---

# C19 SF-3 — Parent->Child Visibility Under Nested Tenancy (Cited Research, Side-Analysis)

> PROPOSED discussion input. Status: capture. Not a spec, not an ADR, not a vision change.
> This refines **SF-3** from the base C19 research. The ANALYSIS + LEANS section at the end
> states the recommended configurable model and the genuine refined sub-forks for the human.

---

## Executive Summary (~15 lines)

1. **The market splits cleanly into two visibility philosophies, and the dividing line is the LEGAL-ENTITY boundary, not the technology.** Where parent and child are the *same legal org* (intra-org governance: cloud account hierarchies, Salesforce role hierarchy), parent-sees-all-below is **raw + position-implicit** — granted by hierarchy position, no per-access consent. Where they are *distinct legal entities* (MSSP / cross-tenant: Cortex XDR, the Lighthouses, GDAP), full visibility is **consent-gated** — the child must explicitly delegate/approve [Perplexity DR-1: AWS Organizations docs; Google Cloud IAM; Azure mgmt-group; Salesforce; Palo Alto Cortex XDR admin guide; Microsoft Lighthouse/GDAP docs].
2. **"Parent sees everything below as if it were its own tenant" is a real, shipped pattern — but every shipped instance of the RAW-everything form is intra-org (single legal entity).** AWS `OrganizationAccountAccessRole` (management account assumes into member accounts "as if operating inside them"), Azure management-group Reader inheritance (root-level Reader = read across all child subscriptions), GCP Organization Viewer (cascades to all projects), and Salesforce "View All Data" are all **raw, position-implicit** [DR-1].
3. **The cross-tenant (distinct-legal-entity) precedents that reach raw data DO it consent-first.** Microsoft Sentinel cross-workspace `workspace()`/`union` queries reach RAW logs across up to 20 workspaces — but only over workspaces the customer **delegated via Azure Lighthouse** [DR-1: Sentinel cross-workspace docs]. Azure Lighthouse gives "as-if-native" raw resource visibility, **but only into subscriptions/resource-groups the customer explicitly delegated** [DR-1: Azure Lighthouse concepts]. Cortex XDR's parent gets incident/host/process drill-down across children — gated by a **pairing request the child Admin must Approve**, with a **persistent UI flag in the child console** showing the parent is managing them [DR-1: Cortex XDR multi-tenant pairing].
4. **Microsoft 365 Lighthouse and ConnectWise sit at the DERIVED end** — Lighthouse surfaces posture/incident summaries with up-to-48h load delay (processed, not live-proxy) [DR-1]; ConnectWise SIEM ingests raw logs into a central analytics engine (raw, but MSP-operated) [DR-1].
5. **The crux question — can a parent get FULL raw-row child visibility while the OPERATOR/vendor still has zero at-rest access — is answered YES, but only with strict principal separation.** The parent-tenant is NOT the operator; the invariant under threat is "operator zero at-rest access" (P-ADS-02 / INV-ADS-02), and that survives any of the four mechanisms PROVIDED the operator's identities never hold decrypt rights [Perplexity DR-2: AWS KMS grants; GCP Cloud KMS/EKM; proxy re-encryption; confidential computing].
6. **The four key-custody mechanisms rank cleanly against the two invariants** (never-share-a-DEK; operator-zero-access): (a) **consent-scoped transient decryption under the child key** — PRESERVES operator-zero-access, AMBIGUOUS on never-share-DEK; (b) **re-encrypt results under the parent key** — PRESERVES both (cleanest for never-share-DEK; the parent's copy is its own key); (c) **parent-as-additional-grantee on the child DEK** — BREAKS never-share-DEK (persistent cross-tenant key sharing); (d) **BYOC remote-op + HYOK + TEE** — STRONGEST operator-zero-access guarantee [DR-2].
7. **Mechanism (c) is the one to forbid.** Adding the parent's service account as `cryptoKeyEncrypterDecrypter` (GCP) or a persistent KMS grantee (AWS) on the child DEK is "a straightforward way to enable parent visibility" but it persistently extends decryption rights across tenants and "blurs the semantics of per-tenant key isolation" — directly conflicting with P-ADS-06 / INV-ADS-03 and the GCP multi-tenant baseline ("dedicated service accounts per customer, strongly prohibit service accounts accessing across customer tenants") [DR-2: GCP multi-tenancy guidance].
8. **The cleanest reconciliation for Prism is re-encryption-of-derived-results (mechanism b), with consent-scoped transient decryption (mechanism a) only for the in-context query and never persisted under any key but the child's.** This keeps Option-3 child-keyed cache semantics intact: persisted artifacts a parent sees are re-encrypted under the parent's DEK; the child's at-rest cache stays child-keyed [DR-2 + base C19 Topic 5].
9. **D-C2-12 / INV-ADS-01 is the hard ceiling that the market precedents do NOT respect and Prism must.** Sentinel/AWS/Salesforce "raw everything" includes raw telemetry. Prism's raw sensor data never leaves the edge — so "full transparent subtree visibility" in Prism means **full visibility of the DERIVED corpus the child surfaces to Central** (query outputs, findings, anomaly scores, GraphRAG, conversation history), NOT raw sensor rows. This is a *narrower* "everything" than the cloud precedents, by invariant, and it is the right narrowing.
10. **The configurable model should be a visibility-grant MATRIX, not a single toggle:** axes = {data-class: derived-rows | findings/alerts | config | audit | metering} x {grant-scope: per-child | per-tier-policy | per-data-class} x {default posture}. Default posture = **OFF / opt-in / consent-gated** (Cortex-XDR-grade). It can be configured UP to a "transparent subtree" preset that grants all data-classes across the subtree — legitimate only in the client-managed single-legal-org context.
11. **The tenancy-context gate is mandatory and must be a first-class tenant attribute.** A `tenant_relationship` field (same-legal-entity | managed-client | saas-customer) conditions which presets the UI even OFFERS. "Transparent subtree" preset is offered for same-legal-entity; "consented derived metrics" is the only preset offered for managed-client; SaaS-customer offers none by default.
12. **Make "see everything" SAFE the way the consent-gated precedents do:** child-admin approval (Cortex XDR pairing), a persistent surfaced indicator IN the child console that the parent has visibility (Cortex XDR UI flag), contractual/relationship flag, granular revocation, and a complete child-side audit trail of every parent access [DR-1 + DR-2].
13. **OT / NERC-CIP (C20 cross-link) materially constrains the upper end.** Regulators may treat parent decryption of a child's OT/BES-cyber-system data as a change in who "owns/controls" critical-infrastructure data, potentially extending CIP obligations to the parent; consent may NOT be purely at the child's discretion (a regulator may mandate or forbid the sharing) [DR-2: NERC-CIP general principles — flagged speculative]. The "transparent subtree" preset must be blockable by a regulatory-class tenant attribute.
14. **Audit attribution is the subtle failure mode of mechanism (a)/(c):** when parent and child share/transit one key, KMS logs may not distinguish which tenant initiated a decrypt [DR-2]. Prism must attribute every parent access to the parent principal at the application layer, child-side, independent of KMS log granularity.
15. **Net SF-3 lean:** support the FULL spectrum the human wants, but implement it as **re-encryption-of-derived-results (b) + consent-scoped transient in-query decryption (a)**, gated by a **visibility-grant matrix** that **defaults safe (opt-in/consent)** and is **conditioned by `tenant_relationship`** so the "transparent subtree as if own tenant" preset is reachable ONLY in the client-managed single-legal-org case — never breaking INV-ADS-01/02/03, never using mechanism (c).

---

## Read-coverage / honesty note

- **In-repo grounding (read in full this pass):** `research/nested-tenancy-2026-06-27.md` (the base C19 research — SF-3 origin, Topic 5 key-custody, Topic 6 metering), `specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md` (P-ADS-02/03/04/06, PAT-ADS-01/02, INV-ADS-01/02/03, AP-ADS-01/03/05 read verbatim). Project memory facts (AD-017, BYOC zero-access, D-C2-12, Option-3 child-keyed cache, OrgId/OrgSlug) honored as stated.
- **NOT read this pass:** the live BC/ADR bodies and SS-26 §4–7 DEK-hierarchy detail (carried from base C19 research + ADS summaries, not re-derived from BC text). The C20 OT/NERC-CIP material is cross-linked, not separately researched here beyond what DR-2 surfaced.
- **External sources:** two `perplexity_research` (sonar-deep-research, reasoning_effort=high) deep passes. DR-1 (precedents, 84.8k chars) and DR-2 (key-custody crux, 91.2k chars) both exceeded the single-read token cap. I extracted them via ~40 targeted keyword-windowed greps covering EVERY platform in the question (Flight Control, Cortex XDR, M365 Lighthouse, Azure Lighthouse, Sentinel, Defender XDR, ConnectWise, Kaseya, Okta/Auth0, AWS Organizations/OrganizationAccountAccessRole, GCP Org Viewer, Azure mgmt-group Reader, Salesforce View All) and every mechanism (transient decryption, re-encryption/proxy-re-encryption, parent-as-grantee, BYOC/HYOK/TEE remote-op), plus the comparative-analysis, regulatory-OT, audit/revocation, and explicit-limitation passages. I did NOT paginate the raw trailing citation-URL lists of either file — source IDENTITY is named inline below (vendor docs), source URL is not transcribed. This is a Level-2 partial-coverage flag on URL transcription only; all substantive claims cited were read.
- **Source labels:** `[DR-1: <named vendor source>]` and `[DR-2: <named vendor source>]` reference the two deep-research passes; the originating vendor/source is named inline so each claim is traceable. The deep-research model itself flagged several items as un-verifiable from public docs — those are carried as **[VENDOR-DOC-GAP]** below, not asserted as fact.

---

## Topic 1 — Precedent table: parent-sees-all-below across MSSP / cloud / SaaS

For each: **Raw vs Derived** (what the parent sees) | **Consent vs Implicit** (how it's granted) | **Granularity** (which object classes).

| Platform | Raw vs Derived | Consent vs Implicit | Granularity / objects | Legal-entity context |
|---|---|---|---|---|
| **CrowdStrike Falcon Flight Control** (parent CID -> child CID) | **Raw at detection/host-context level** (detections + host context replicate to parent; whether *arbitrary raw telemetry* is queryable parent-side is **[VENDOR-DOC-GAP]**) [DR-1: CrowdStrike Flight Control datasheet; Stitchflow RBAC writeup] | Leans **position-implicit** — described as a hierarchical "CID Group" architecture with event flow "from the Parent CID"; **no documented child-approval pairing workflow** (contrast Cortex XDR); RBAC adds CID-level scoping [DR-1] | Detections, host context, policy inheritance parent->child, CID-scoped roles | Typically same enterprise / public-sector "whole-of-state"; MSSP usage exists |
| **Palo Alto Cortex XDR** (parent/child tenant) | **Raw incident-level** — central account sees incident timelines, affected hosts, processes, network trajectories, attack chains across children; arbitrary raw-telemetry query breadth **[VENDOR-DOC-GAP]** but raw incident/config access is documented [DR-1: Cortex XDR multi-tenant admin guide] | **Consent-gated** — parent sends "Request for Pairing"; child **Admin must Approve**; Pairing Status stays Pending until approved; **persistent UI flag in child console** shows who manages their config [DR-1] | Incidents, host/process/network drill-down, config, licensing across children | Explicitly MSSP **and** large-org subdivisions (distinct legal entities supported) |
| **Microsoft 365 Lighthouse** | **Derived** — posture insights + incident summaries computed from underlying services (Defender/Intune/Entra); up-to-48h data load = ingested/processed, not live raw proxy [DR-1: M365 Lighthouse FAQ + "What's new"] | **Consent-gated** — DAP/**GDAP** delegated admin; only within delegated scopes [DR-1] | Posture metrics, per-tenant security/business reports, incidents (summarized) | MSP/MSSP, distinct customer tenants |
| **Azure Lighthouse** | **Raw, "as-if-native" resource-level** — provider signs into own tenant, works on customer resources with full resource-level visibility; enables Sentinel cross-workspace raw-log query into delegated workspaces [DR-1: Azure Lighthouse concepts + "My customers"] | **Consent-gated by explicit delegation** — customer delegates specific subscriptions/resource-groups; "Customers" section shows ONLY delegated scopes; visibility grounded in delegation, "not implicit hierarchy" [DR-1] | Resources, config, Log Analytics workspaces (-> raw logs), Activity Logs/audit, dashboards | MSP, distinct customer tenants |
| **Microsoft Sentinel** (multi-workspace) | **RAW logs** — `workspace()` + `union` query `SecurityEvent` etc. across **up to 20 workspaces**, aggregating raw log entries across environments [DR-1: Sentinel cross-workspace docs] | **Consent-gated** — reaches only workspaces the customer delegated (typically via Azure Lighthouse) | Raw security/operational logs, incidents across workspaces | Cross-tenant via delegation |
| **Microsoft Defender XDR** (multi-tenant) | Cross-tenant **incident/case** views (derived-to-semi-raw) [DR-1] | Consent-gated (delegated) | Incidents, cases across tenants | Cross-tenant |
| **ConnectWise SIEM** | **Raw logs** — collects security event logs into a central analytics engine inspectable at log level [DR-1: ConnectWise SIEM marketing; depth **[VENDOR-DOC-GAP]**] | MSP-operated (the MSP runs the multi-tenant platform); per-customer consent is contractual | Raw logs, central analytics across customers | MSP, distinct customers |
| **Kaseya VSA** | Multi-tenant management; raw vs derived depth **[VENDOR-DOC-GAP]** [DR-1] | MSP-operated/contractual | RMM management surface | MSP |
| **Okta (Orgs / Okta Aerial)** | Multi-org admin at an "account layer"; whether Aerial admins see raw system logs/auth events/profiles across orgs vs only push config templates is **[VENDOR-DOC-GAP]** [DR-1: Okta organizations docs] | Centralized at account layer (super-admin); per-org consent model **[VENDOR-DOC-GAP]** | Org administration; config push down | Single Okta customer spanning multiple orgs |
| **Auth0** | Multi-org; specifics **[VENDOR-DOC-GAP]** [DR-1] | **[VENDOR-DOC-GAP]** | — | — |
| **AWS Organizations** (`OrganizationAccountAccessRole`, delegated admin) | **RAW, as-if-inside** — management account assumes the role and gains "broad administrative privileges inside member accounts, as if operating inside those accounts"; raw access to resources/config/logs [DR-1: AWS Organizations docs] | **Position-implicit** — once the account is in the org and the role exists, access does not require per-session member approval; "member account admins do not approve each access session" [DR-1] | Resources, config, logs (raw); delegated-admin per security service (Security Hub/GuardDuty/Macie/Config) | **Same legal org** (consolidated billing entity) |
| **GCP Organization Viewer** | **RAW resource visibility** — Org-node Viewer cascades to all projects/resources; raw resource + log access per IAM intersection [DR-1: GCP IAM/Resource Hierarchy docs] | **Position-implicit** — granted at org node, no per-project approval [DR-1] | Projects, resources, config, logs (per IAM) | **Same legal org** |
| **Azure management-group Reader** | **RAW resource visibility** — Reader at root mgmt-group = read across all child subscriptions' resources/config/dashboards; raw telemetry depends on log-access role [DR-1: Azure mgmt-group docs — partly **[VENDOR-DOC-GAP]**, relies on general knowledge] | **Position-implicit** at mgmt-group level; no per-subscription gating once hierarchy set [DR-1] | Resources, config, dashboards, logs (with log role) | **Same legal org** (single Entra tenant) |
| **Salesforce role hierarchy + "View All"/"Modify All"/"View All Data"** | **RAW record-level** — higher role inherits subordinate record access; "View All Data" = raw access across objects [DR-1: Salesforce sharing model — partly **[VENDOR-DOC-GAP]**] | **Position-implicit** — encoded in role assignment, no per-record consent [DR-1] | Records, config, dashboards/reports (via record access) | **Same legal org** (intra-org governance) |

**Pattern that falls out of the table:** every **raw + position-implicit "sees everything below"** instance is **intra-org (single legal entity)** — AWS Organizations, GCP org Viewer, Azure mgmt-group Reader, Salesforce. Every **cross-legal-entity** instance that reaches raw data does it **consent-first** (Cortex XDR pairing, Azure Lighthouse delegation, Sentinel-over-delegated-workspaces, GDAP). The dividing line the human should design around is the **legal-entity boundary**, not the depth of the tree.

---

## Topic 2 — Tenancy / legal-context distinction (the default driver)

The deep research is explicit: intra-org hierarchies are treated as "trusted root-level accounts [that] can legitimately see everything," whereas cross-entity products "enforce consent-based onboarding and pairing where distinct legal entities are involved" [DR-1]. This maps directly onto Prism's three operating models:

| Prism operating model | Legal-entity relationship | Closest precedent | Natural default visibility |
|---|---|---|---|
| **Client-managed (single org, internal subtree)** | Parent IS the same legal org — the subtree is genuinely "its own" (a corporate parent with BUs/subsidiaries it owns) | AWS Organizations / Azure mgmt-group / Salesforce View All | **Full transparent subtree LEGITIMATE** — position-implicit is defensible here (the org sees its own data). This is the context where the human's "as if its own tenant" end of the spectrum is correct. |
| **MSSP-managed (distinct CLIENTS)** | Children are distinct legal entities (the MSSP manages clients it does not own) | Cortex XDR pairing / Azure Lighthouse / GDAP | **Consent-gated, derived-first** — pairing/delegation REQUIRED; raw access only on explicitly delegated scopes; persistent child-side indicator. Position-implicit raw access is NOT defensible. |
| **SaaS (distinct CUSTOMERS)** | Children are arms-length customers | (none grant parent->customer visibility by default) | **No parent visibility by default** — there is usually no legitimate "parent" with a claim on a SaaS customer's data. |

**Design consequence:** the default posture and the *set of presets the UI offers* must be conditioned on a first-class `tenant_relationship` attribute. The same code path supports all three (single-codebase, P-ADS-11) — only the configuration/default differs. This is the precedent-grounded answer to "what should drive the default": **the legal-entity relationship drives the default, not the tree position.**

---

## Topic 3 — Key-custody reconciliation (THE CRUX)

The question: if a parent gets FULL raw-row visibility into a child, how is that reconciled with per-child DEK + operator-zero-access (P-ADS-02) + Option-3 child-keyed cache? Two invariants are under test:

- **INV-K1 "never share a DEK across tenants"** (P-ADS-06 / INV-ADS-03 cryptographic enforcement; base C19 Topic 5).
- **INV-K2 "operator/vendor zero at-rest access"** (P-ADS-02 / INV-ADS-02). *Critically: the parent-tenant is NOT the operator.* INV-K2 is about the vendor running Central, not about a legitimate parent tenant. So a mechanism can give a parent raw visibility and STILL satisfy INV-K2 as long as the OPERATOR's identities never gain decrypt rights [DR-2].

The deep research evaluated the four mechanisms against both invariants:

| Mechanism | What it does | INV-K1 (never-share-DEK) | INV-K2 (operator-zero-access) | ADS verdict |
|---|---|---|---|---|
| **(a) Consent-scoped transient decryption under the CHILD key** | Parent triggers a query; data decrypts under the child's DEK for the query duration only; parent never receives/stores the DEK; child grants + can revoke; every access audited child-side [DR-2: AWS KMS grants + key/IAM policies; GCP `cryptoKeyEncrypterDecrypter`] | **AMBIGUOUS.** The KMS key stays unique to the child and ciphertext at rest stays child-associated — strict-isolation-preserving in one reading. But "allowing the parent to decrypt with the child's key could be considered a violation" under the strict GCP "no service account across tenants" baseline [DR-2: GCP multi-tenancy guidance] | **PRESERVED** if the operator is excluded from key access; strengthened with confidential computing [DR-2] | **ALLOW for in-query only, never persist.** Use for the live transparent-view query; do not persist the result under the child key for the parent. Pair with strict child-side audit attribution. |
| **(b) Re-encrypt the result under the PARENT key** | Parent reads data encrypted to its OWN key; KMS `ReEncrypt` / proxy-re-encryption transforms ciphertext from child key to parent key inside the KMS boundary [DR-2: AWS KMS ReEncrypt; proxy re-encryption theory] | **PRESERVED (cleanest).** "Re-encrypting a child's data under a parent's key does NOT share the DEK across tenants, because the ciphertext now belongs to the parent's view and is protected under its own key" [DR-2] | **PRESERVED** (operator excluded; transformation inside KMS) | **PREFERRED for any PERSISTED parent-visible artifact.** Caveat: couples threat models + key rotation — if the child rotates its DEK, the parent's re-wrapped copy must be updated or breaks [DR-2: "one key, one purpose"]. |
| **(c) Parent-as-additional-grantee on the child DEK** | Child adds the parent's service account as a persistent KMS grantee / `cryptoKeyEncrypterDecrypter` on the child key [DR-2] | **BROKEN.** "Adding the parent as a grantee on the child's DEK directly extends decryption rights across tenants"; "blurs the semantics of per-tenant key isolation"; conflicts with GCP baseline that "strongly prohibits service accounts accessing across customer tenants" [DR-2] | Preserved re: operator, but the cross-tenant key-sharing is the problem | **FORBID.** This is the mechanism to explicitly bar. It is the easy path and the wrong one. Maps to AP-ADS-05 spirit (cross-tenant access at the key layer) and violates INV-ADS-03. |
| **(d) BYOC remote-op + HYOK + TEE** | Parent runs a bounded query inside the child's own cloud/domain; keys never leave the child (HYOK / GCP EKM); confidential computing (Nitro Enclaves / SGX) restricts data-in-use even from privileged admins [DR-2: GCP EKM; AWS Nitro] | **PRESERVED** (parent never holds keys) | **STRONGEST guarantee** — keys never reside in the operator's administrative domain [DR-2] | **PREFERRED for OT / critical-infra / highest-assurance** client-managed deployments. Heaviest to integrate; latency/availability dependency on external KMS [DR-2]. |

**Answer to "is full visibility as-if-own-tenant achievable while operator/vendor STILL has zero at-rest access?"** — **Yes.** Because parent-tenant != operator, INV-K2 is independent of parent visibility. Full raw-row parent visibility is achievable via (a), (b), or (d) without ever giving the operator decrypt rights. Only mechanism (c) is disqualified, and it is disqualified on INV-K1, not INV-K2 [DR-2].

**Recommended Prism composition (ADS-conformant):**
- **Persisted parent-visible artifacts -> mechanism (b) re-encryption under the parent DEK.** This keeps Option-3 intact: the child's at-rest cache stays child-keyed (P-ADS-04 unchanged); the parent's view is a *separate* artifact re-encrypted to the parent's DEK. No DEK is shared.
- **Live "transparent subtree" query -> mechanism (a) transient decryption under the child key,** scoped to the query, never persisted under the child key for the parent, fully audited child-side.
- **OT / NERC-CIP / highest-assurance client-managed -> mechanism (d) BYOC remote-op.**
- **Mechanism (c) -> explicitly forbidden** (new anti-pattern candidate, see LEANS).
- **D-C2-12 / INV-ADS-01 caps the whole thing:** none of this touches RAW SENSOR DATA, which never leaves the edge. "Everything below" in Prism = the full DERIVED corpus the child surfaces to Central, NOT raw sensor rows. This is a narrower (and correct) "everything" than the AWS/Sentinel/Salesforce precedents, which DO expose raw telemetry [DR-1] — Prism cannot and should not match that breadth.

**Audit-attribution subtlety (do not miss this):** under shared/transit-key mechanisms, "KMS logs will record operations under that key without always distinguishing which tenant initiated them" [DR-2]. Prism must attribute every parent access to the parent principal at the APPLICATION layer, child-side, independent of KMS-log granularity.

---

## Topic 4 — Configurable visibility model (the deliverable)

Per the human's two axes — **(1) WHETHER** a parent can see into a child, **(2) WHAT/how much** — the recommendation is a **visibility-grant matrix**, not a single toggle. This mirrors how the consent-gated precedents work (Lighthouse delegates *specific* scopes; GDAP is *granular*; Cortex XDR pairing is per-child) while permitting the full-transparency upper bound where legitimate.

**Configurable dimensions:**

- **Data-class (WHAT):** `derived_rows` (query outputs) | `findings_alerts` (detections/anomaly scores/advisories) | `config` (effective config view) | `audit` (the child's audit trail) | `metering` (usage/cost rollup — already covered by base C19 Topic 6, runs on metadata, lowest sensitivity).
- **Grant-scope (HOW it's set):** `per_child` (explicit per-child grant, Cortex-XDR-grade) | `per_tier_policy` (a policy that auto-applies to children of a given tier/relationship) | `per_data_class` (toggle each data-class independently).
- **Default posture (the safe floor):** **OFF** for everything except `metering` rollup (metering is metadata, defensible by position per base C19 Topic 6). Every data-class above metering defaults **opt-in / consent-gated**.

**Recommended preset ladder (defaults safe, configurable up):**

| Preset | Data-classes granted | Mechanism | Offered for `tenant_relationship` = |
|---|---|---|---|
| **P0 Metering-only** (default everywhere) | `metering` rollup only (metadata) | metadata aggregation, no decryption | all |
| **P1 Consented derived metrics** (base C19 conservative default) | `findings_alerts` summaries + `metering`; consent-gated | (b) re-encrypt under parent key | managed-client, client-managed |
| **P2 Operational visibility** | `derived_rows` + `findings_alerts` + `config` (read); consent-gated; child-side indicator | (a) transient in-query + (b) for persisted | managed-client (with pairing), client-managed |
| **P3 Transparent subtree ("as if own tenant")** | ALL derived data-classes incl. `audit` across the subtree | (b) persisted + (a) live; OR (d) for OT | **client-managed single-legal-org ONLY** (gated by `tenant_relationship` + non-regulated class) |

- **The matrix defaults safe** (P0/opt-in) and **can be configured up to P3 transparent subtree** — satisfying the human's requirement to support the fuller end of the spectrum.
- **P3 is reachable only when `tenant_relationship = same-legal-entity`** (client-managed single-org), echoing the precedent finding that raw-everything is only legitimate intra-org [DR-1].
- **P3 is blockable by a regulatory-class attribute** (OT/NERC-CIP), per Topic 5.
- Every preset above P0 requires the consent + audit governance in Topic 5.

---

## Topic 5 — Consent / governance / audit (making "see everything" safe)

The consent-gated precedents converge on five controls; Prism should adopt all five for any preset above P0 [DR-1 + DR-2]:

1. **Child-admin approval (pairing).** Copy Cortex XDR: parent requests; child Admin must Approve; status stays Pending until then [DR-1]. For P3 in client-managed single-org, approval may be an org-level administrative grant rather than per-child, but it must still be an explicit, recorded act — not silently position-implicit.
2. **Persistent surfaced indicator in the child.** Copy Cortex XDR's UI flag: the child console must always show that a parent has visibility and who [DR-1]. This is non-negotiable for the trust story and for regulatory defensibility.
3. **Contractual / relationship flag.** The `tenant_relationship` attribute + a recorded contractual basis gates which presets are offered (Topic 2). Distinct legal entities require it; same-legal-org records the internal authorization.
4. **Granular revocation.** Child (or, in regulated contexts, a mandated authority) can revoke any data-class grant; revocation must immediately stop future access. Mechanism (a) revocation = revoke the KMS grant/IAM binding; mechanism (b) = stop producing re-encrypted artifacts (existing parent-keyed copies are the parent's data — define retention/deletion policy explicitly) [DR-2].
5. **Complete child-side audit trail.** Every parent access logged child-side with parent-principal attribution at the application layer (independent of KMS log granularity, Topic 3). Tracks "who accessed what data, when, via which cryptographic operation" [DR-2].

**OT / NERC-CIP (C20 cross-link) — material constraint on the upper end** [DR-2, flagged speculative where it interprets CIP]:
- A parent decrypting a child's OT / Bulk-Electric-System (BES) cyber-system data may be treated by regulators as a **change in who "owns/controls" critical-infrastructure data**, potentially **extending CIP obligations to the parent**.
- **Consent may NOT be purely at the child's discretion** — a regulator may mandate parent oversight OR prohibit certain sharing.
- Data the parent holds under its own key (mechanism b) "must be protected with equivalent or stronger controls" than at the child [DR-2].
- **Design consequence:** a `regulatory_class` tenant attribute (e.g., `nerc_cip`, `ot_critical`) must be able to **force P3 OFF** or **force mechanism (d) BYOC-remote-op** regardless of the parent's wish, and must drive heightened audit. This is the safe default for the OT vertical that Prism's vision targets.

---

## Topic 6 — ADS conformance check (must pass)

| ADS item | Does the recommended model conform? |
|---|---|
| **INV-ADS-01 / D-C2-12** (no raw sensor data at Central) | YES — "everything below" = derived corpus only; raw sensor data never leaves the edge. The model explicitly does NOT replicate the AWS/Sentinel raw-telemetry breadth. |
| **P-ADS-02 / INV-ADS-02** (operator zero at-rest) | YES — parent != operator; mechanisms (a)/(b)/(d) all keep the operator out of decrypt; (c) forbidden. |
| **P-ADS-04 / Option-3** (tenant-keyed central persistence) | YES — child cache stays child-keyed; parent-visible PERSISTED artifacts are re-encrypted under the PARENT DEK (a separate Option-3 artifact), so the cache-key rule from base C19 Topic 5 ("child-keyed") is preserved and the parent's view is its own keyed artifact. |
| **P-ADS-06 / INV-ADS-03 / AP-ADS-05** (per-tenant isolation, no cross-tenant key/data sharing) | YES via (b)/(a-in-query); the ONLY mechanism that violates this is (c), which the model forbids. |
| **AP-ADS-01 / AP-ADS-03** (no raw data / no operator-readable at rest) | YES — no raw at Central; no operator-key at rest. |
| **P-ADS-11** (single codebase, no forks) | YES — one code path; behavior varies by `tenant_relationship` + `regulatory_class` config + the visibility-grant matrix, not by compile-time fork. |

The model is ADS-conformant. The one new architectural artifact it implies is an explicit **anti-pattern for mechanism (c)** (parent-as-grantee on a child DEK) — see LEANS.

---

## ANALYSIS + LEANS

### Synthesis

The human wants the fuller end of the visibility spectrum to be *reachable*. The research says: **it is reachable and shipped — but in the market, the RAW "as if its own tenant" form is exclusively an intra-org (single-legal-entity) pattern, and every cross-legal-entity product that reaches raw data does so consent-first.** Prism can support the full spectrum without breaking any ADS invariant by (1) capping "everything" at the DERIVED corpus (D-C2-12 / INV-ADS-01 — narrower than the cloud precedents, by design), (2) using re-encryption-under-the-parent-key for persisted parent-visible artifacts and transient child-key decryption for live queries, (3) FORBIDDING parent-as-grantee on the child DEK, and (4) gating the "transparent subtree" preset behind a `tenant_relationship = same-legal-entity` attribute with a `regulatory_class` override for OT/NERC-CIP. The operator-zero-access invariant is never at risk because the parent-tenant is not the operator.

### Refined sub-forks for the human (SF-3 decision)

- **SF-3a — Upper bound of the spectrum (the core ask).** Confirm Prism supports a configurable **P3 "transparent subtree" preset** (parent sees all DERIVED data-classes below as if its own), reachable **only when `tenant_relationship = same-legal-entity`** and not regulatory-blocked. *Recommended lean: **YES, support P3, gated by `tenant_relationship`.*** This directly grants the human's request while keeping the cross-legal-entity (MSSP) case consent-gated, matching every market precedent.

- **SF-3b — Persisted-visibility mechanism.** Choose the key-custody mechanism for PERSISTED parent-visible artifacts: (b) re-encrypt under the parent DEK [recommended], vs (a) only transient/non-persisted, vs (d) BYOC-remote-op for the highest tier. *Recommended lean: **(b) re-encryption for persisted + (a) transient for live queries, with (d) reserved for OT/critical-infra.*** This is the only composition that preserves both INV-K1 and INV-K2. **Forbid (c) parent-as-grantee outright** (propose a new ADS anti-pattern, e.g., AP-ADS-11 "Cross-Tenant DEK Grantee").

- **SF-3c — Default posture + matrix vs single toggle.** Confirm the **visibility-grant MATRIX** (data-class x grant-scope x relationship) with **default OFF above metering** (opt-in/consent), vs a simpler single on/off per child. *Recommended lean: **matrix with safe defaults*** — it is what GDAP/Lighthouse/Cortex-XDR granularity demonstrates is necessary for the cross-legal-entity case, and it costs little to also serve the single-org case via the P3 preset.

- **SF-3d — OT / NERC-CIP override (C20 cross-link).** Confirm a `regulatory_class` tenant attribute can **force P3 OFF or force mechanism (d)** and heighten audit, regardless of parent/child preference, because consent may be regulator-constrained for BES/OT data [DR-2]. *Recommended lean: **YES, regulatory_class is an override that can only tighten*** (mirrors the AWS-style "parent guardrail can only restrict" semantic from base C19 SF-1).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | DR-1 (reasoning_effort=high, 84.8k chars): parent-sees-all-below precedents across all 12+ named platforms — raw-vs-derived, consent-vs-implicit, granularity, legal-entity context. DR-2 (reasoning_effort=high, 91.2k chars): cryptographic/architectural key-custody mechanisms (transient decryption / re-encryption / parent-as-grantee / BYOC-remote-op) vs never-share-DEK + operator-zero-access invariants, plus OT/NERC-CIP regulatory implications and audit/revocation governance. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not applicable — architecture/governance pattern research; no single library API in scope. |
| Tavily (any) | 0 | Not needed — two high-effort deep passes plus in-repo ADS/base-research grounding were sufficient; the deep-research model self-flagged its own vendor-doc gaps (carried as [VENDOR-DOC-GAP]), so no false confidence to cross-correct. |
| WebFetch / WebSearch | 0 | — |
| Read (in-repo) | 2 files | Grounding: base C19 research (SF-3 origin, Topic 5 key-custody, Topic 6 metering) + ADS (P-ADS-02/03/04/06, INV-ADS-01/02/03, AP-ADS-01/03/05) read verbatim for conformance framing. |
| Grep (over deep-research output files) | ~40 | Extraction technique: both deep-research results exceeded the single-read token cap (one-line files), so content was extracted via targeted keyword-windowed greps covering every platform and every mechanism named in the question, plus the comparative-analysis, regulatory, audit, and explicit-limitation passages. |
| Training data | 1 area | Mapping precedents/mechanisms onto Prism's specific ADS principles and Option-3 cache semantics (clearly labeled as analysis/lean, not vendor fact). |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated PRIMARY tool, both reasoning_effort=high).
**Training data reliance:** low — every external claim (platform visibility model, consent-vs-implicit gating, the four key-custody mechanisms and their invariant verdicts, KMS ReEncrypt/grant semantics, HYOK/EKM/TEE, NERC-CIP general principles) is sourced to the two deep-research passes with the originating vendor/source named inline. Items the deep-research model could not verify from public docs are explicitly carried as [VENDOR-DOC-GAP] rather than asserted. The Prism-specific mapping (mechanism selection, visibility-grant matrix, `tenant_relationship`/`regulatory_class` gating, ADS conformance verdicts) is labeled analysis/lean.

> **Honesty flag on source completeness:** Level-2 partial-coverage on URL transcription only — I read all substantive analysis, comparison tables, and limitation passages of both deep-research files via windowed greps, but did NOT transcribe the trailing raw citation-URL lists; source identity is named inline so every claim is traceable to a named vendor source. Several precedents (Flight Control raw-telemetry breadth, Okta/Auth0 cross-org log access, Azure mgmt-group / Salesforce specifics, Kaseya depth) are flagged [VENDOR-DOC-GAP] where the deep-research model relied on general knowledge rather than cited vendor docs; the recommendations do NOT depend on those gap items (they rest on the well-documented Cortex XDR / Lighthouse / Sentinel / AWS Organizations / KMS findings). NERC-CIP interpretive claims are flagged speculative per the source.
