---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day-2-vision SIDE-ANALYSIS (OUT-OF-BAND; SEPARATE from the live VSDD factory pipeline)
pillar: C9-NARROW — Config-Authority narrowing (refines the C9 broad domain-split toward a human-proposed cut)
scope_fence: >
  Tests ONE hypothesis: narrow Git-authority to ONLY detection content (detection rules +
  saved/scheduled queries + §14.7 hunt recipes); EVERYTHING ELSE (connectors enable/disable AND
  definitions/specs, pushdown descriptors, configure-schema mappings, satellite topology/trust,
  RBAC roles AND assignments, retention, per-tenant overrides, feature flags, credential references)
  is DATABASE-authoritative (Postgres central / SQLite satellite), UI/API-mutated, Git NOT involved.
storage_taxonomy_constraint: "DECIDED — config store = bundled PostgreSQL (central) / embedded SQLite (satellite-edge) per ADR-PROP-storage-engine-taxonomy.md §14.3. NOT reopened."
non_contradiction_reads:
  - research/config-management-depth-2026-06-27.md (C9 — the broad domain-split this NARROWS; Q2 §2.3-2.4 fork; secret-references AD-017; CrowdStrike blast-radius lesson)
  - day2-design-decisions/ADR-PROP-detection-engine-depth.md (C6 — D-C6-2 suppression-as-code mandatory justification + time-box; D-C6-3 shadow/canary/rollback; backtest coverage map)
  - day2-design-decisions/ADR-PROP-prismql-deliverables.md (C8 — L-C8-5 recipe/detection-as-code format Sigma-aligned + semver + CI harness; §14.7 recipe library; D-C8-1 piped surface)
  - CLAUDE.md (AD-007 ArcSwap hot-reload; AD-017 AI-opaque reference-based credentials; #[non_exhaustive]; eat-our-own-dogfood TOML connectors)
# CAPTURE artifact. Tests the human's narrow-cut hypothesis against cited prior art.
# Modifies no live spec/BC/ADR/story/STATE/SESSION-HANDOFF/RESEARCH-INDEX. Not git-added/committed.
# Leans / Verdict are discussion input only — NOT decisions.
---

# Config-Authority NARROW Cut — Detection-Content-Only-in-Git (C9-NARROW)

> **READ FIRST.** Out-of-band side-analysis CAPTURE for the day-2 vision. `do_not_execute: true`.
> Modifies no live artifact; NOT added to RESEARCH-INDEX.md (per the hard boundary on this dispatch).
> Storage-engine choice (Postgres central / SQLite satellite) is SETTLED in §14.3 and treated as a
> fixed premise. The C6/C8 detection decisions are SETTLED inputs this analysis CONSUMES, not reopens.

**The hypothesis under test (the human's proposed cut).** Instead of the broad C9 domain-split
(which put detection rules AND connector-definitions AND pushdown-descriptors AND satellite-trust
AND baseline-RBAC-roles all in the Git-authoritative bucket — see `config-management-depth-2026-06-27.md`
§2.4 LEAN), NARROW Git-authority to **ONLY detection content** = detection rules + (maybe)
saved/scheduled queries + the §14.7 hunt-recipe library. **EVERYTHING ELSE lives in the DATABASE**
(Postgres central / SQLite satellite), DB-authoritative, UI/API-mutated, Git not involved:
connectors (enable/disable AND definitions/specs), pushdown descriptors, configure-schema mappings,
satellite topology/trust, RBAC roles AND assignments, retention, per-tenant overrides, feature
flags, credential references.

**Confidence legend:** [web] = verified web/doc finding with citation · [model-knowledge] = model
knowledge, not independently re-verified this pass · [INCONCLUSIVE] = could not verify.

**Landscape date:** findings current as of 2026-06-27.

---

## 0. Bottom line up front

The narrow cut is **substantially sound and matches mainstream practice — with TWO sharp exceptions
that the broad C9 split got right and the narrow cut gets wrong.**

- **Detection-content-only-in-Git IS the default real-world pattern** across Panther, Elastic,
  Splunk/ESCU, Chronicle/SecOps, SigmaHQ. Connectors, log sources, RBAC, and platform settings live
  in the DB/UI as the *default* across every platform surveyed; GitOps-everything (via Terraform/
  Crossplane) is an *optional overlay*, not the default. The human's instinct is correct and well-attested. [web]
- **The two exceptions where the narrow cut is most likely wrong** are exactly the two the dispatch
  flagged for adversarial scrutiny:
  1. **Satellite TRUST/topology policy in a mutable DB with no review gate** runs against a strong
     security-engineering consensus that *trust and authorization policy should be reviewed code*
     (SPIFFE/SPIRE, Istio AuthorizationPolicy, Teleport, OPA, GitOps-security). This is the highest
     blast-radius item and the weakest part of the narrow cut. [web]
  2. **Connector DEFINITIONS (the dogfood TOML specs)** are *already config-as-code today*. Moving
     them to pure DB-authority discards the dogfood model — but there is a clean reconciling answer
     (**Grafana-provisioning-style: author-as-file, import-to-DB, DB authoritative at runtime**) that
     preserves file-authoring + review without making Git the runtime authority. [web]
- **RBAC ROLE DEFINITIONS** sit in a softer middle: definitions-as-reviewed-code is best practice
  (AWS IAM/Terraform, Vault, Kubernetes RBAC GitOps, OPA), but vendors do NOT mandate it and most
  ship DB/UI role management as default. Assignments-in-DB is uncontested. [web]
- **The narrow cut's biggest WIN is real:** one DB-authoritative model for ~everything + a thin Git
  path for detection content only — no per-key classification sprawl, no broad reconcile loop. The
  residual risk is that §11.2's "versioned change control + audit + rollback" for the DB-authoritative
  majority now rests **entirely** on the Postgres audit table + DB snapshotting — and that is **not a
  full substitute for git-revert for the high-blast-radius items** (trust, connector defs, pushdown). [web]

The honest synthesis: **narrow the Git bucket to detection content as the human proposes, BUT keep a
thin file-authored-import path (not full Git-authority, not pure DB-mutation) for the two high-blast-
radius non-detection items — satellite trust policy and connector/pushdown definitions** — exactly the
Grafana-provisioning reconciling pattern. That is a smaller, cheaper hybrid than the broad C9 split,
and it closes the two security holes the pure narrow cut opens.

---

## 1. Is "detection-content-only in Git, everything-else-in-DB" a recognized pattern?

**Verdict: YES — it is not merely recognized, it is the DEFAULT real-world pattern** for mature
security platforms that have adopted detection-as-code. The deep-research pass surveyed each platform
the dispatch named and found a uniform answer. [web]

| Platform | Git-managed (default) | DB/UI-managed (default) | IaC-everything? |
|---|---|---|---|
| **Panther** | `panther-analysis` repo: Python rule logic + YAML metadata, policies, rule packs ONLY [panther-analysis][panther-DaC] | Log sources + schemas (`Log Sources > Add New Source`, `Configure > Schemas`); connectors/integrations via console API tokens; **RBAC roles via console / GraphQL/REST API** [panther-rbac][panther-logs][dropzone] | No Terraform in `panther-analysis`; API access only |
| **Elastic Security** | `detection-rules` repo: TOML/code rules + test/release pipeline ONLY [elastic-detection-rules] | Fleet agent policies + integrations in Kibana → Elasticsearch DB; **RBAC roles in ES/Kibana (Fleet mgmt currently needs superuser)**; prebuilt-rules Fleet package is a *delivery* mechanism, not config repo [elastic-fleet][elastic-rbac-thread][elastic-prebuilt-thread] | Optional external IaC, beside (not in) detection-rules |
| **Splunk / ESCU** | `security_content` repo: analytic stories, SPL searches, ML, Phantom playbooks via `contentctl` [splunk-security-content] | `inputs.conf` / `indexes.conf` ingestion; **RBAC roles/capabilities via Splunk Web / Cloud Platform** [splunk-inputs][splunk-rbac] | conf files Git-able by admins, but NOT via `security_content` |
| **Chronicle / SecOps** | YARA-L rules + dashboards repo; `content_manager` CLI / Terraform provider pushes rules [chronicle-detection-rules][secops-dac] | Log ingestion / feeds / forwarders / RBAC / parsers via console; **Terraform provider is an optional IaC overlay** beside detection rules [secops-terraform] | Terraform provider exists — explicitly *optional overlay*, alongside |
| **SigmaHQ** | PURE detection content (3000+ YAML rules + translation tooling); no connectors, no RBAC at all [sigmahq] | N/A (not a runtime platform) | N/A |

**Cited conclusion (verbatim synthesis):** *"detections-only in Git, everything-else-in-the-
database/UI/API is not only a recognized model but effectively the default real-world pattern today,
with end-to-end GitOps for connectors and RBAC still emerging and uneven across vendors."* [web]

**The critical nuance for the narrow cut.** The default split these platforms ship is "detections in
Git; **connectors + sources + RBAC + platform settings in DB/UI**." That is *exactly the human's narrow
cut* for those four categories. Where mature platforms push connectors/RBAC into Git, it is via an
*optional Terraform/IaC overlay that sits beside the detection repo*, not the product default. So the
narrow cut is the COMMON pattern; the broad C9 split (connectors + RBAC + pushdown all Git-authoritative
by default) is actually the *less* common, GitOps-everything end of the spectrum. [web]

**Caveat the narrow cut must absorb:** none of these platforms is a *federated, multi-satellite, edge-
trust* system. None of the surveyed platforms manages a **satellite trust topology** the way Prism's
§3.2 dial-home mesh does. The "everything-else-in-DB" default these platforms validate covers
connectors/sources/RBAC/settings — it does NOT cover *trust/topology policy*, because these are
single-control-plane SaaS products with no satellite-trust analog. That gap is precisely why Q3 below
pulls in SPIFFE/SPIRE/Istio/Teleport — the trust-policy prior art lives in the service-mesh / zero-trust
world, not in the SIEM world, and it tells a *different* story (Q3).

**LEAN (Q1):** the narrow cut for **connectors-enable/disable, configure-schema mappings, per-tenant
overrides, feature flags, retention, RBAC assignments** is mainstream and defensible — keep them
DB-authoritative. The narrow cut for **connector DEFINITIONS, pushdown descriptors, satellite trust,
RBAC role definitions** is where the SIEM prior art runs out and the trust/policy-as-code prior art
takes over (Q3). [web]

---

## 2. Saved queries / hunt recipes — Git or DB?

**Verdict: the §14.7 hunt-recipe library belongs in the detection-content Git bucket; ad-hoc saved/
scheduled queries belong in the DB. The natural line is intent + lifecycle, not storage format.** [web]

The deep-research pass found a *consistent* line across Splunk, Elastic, Chronicle, SigmaHQ, and the
community hunt-library ecosystem:

- **Git-worthy (detection-content-grade):** production detection rules, correlation logic, scheduled
  *correlation searches* that function as detectors, and **structured, reusable, MITRE-aligned hunt
  playbooks / recipe libraries.** Exemplars: Splunk Security Content detection YAMLs, Elastic
  detection-rules, Chronicle YARA-L, SigmaHQ rules, **Threat Hunter Playbook (`OTRF/ThreatHunter-
  Playbook`), Target `Threat-Hunting` Jupyter hunt packs**, OSINT-derived rules. These are "long-lived,
  reusable, central to detection posture" and are authored as files, PR-reviewed, CI-tested. [web][threathunter-playbook][target-hunting]
- **DB/UI artifacts (user-grade):** *ad-hoc* saved searches, personal dashboards, Elastic Discover
  sessions + Timelines, Chronicle saved searches (explicitly "saved to the user's account") +
  retrohunt jobs (ephemeral rule executions). "Quick investigative pivots, personal saved searches,
  exploratory hunt sessions." [web][elastic-saved-objects][secops-udm-search][secops-retrohunt]

**The Splunk middle case (decisive for Prism's saved/scheduled queries).** Splunk `savedsearches.conf`
is technically a Git-able file, but the *natural* practice is: **only detection-grade scheduled/
correlation searches get promoted into Git** (via `contentctl` / Security Content); personal/ad-hoc
saved searches stay UI-managed user artifacts. The community even files threads asking how to Git-manage
`savedsearches.conf` precisely because it is NOT the default. [web][splunk-savedsearches-thread]

**Direct answer to "does §14.7 belong in Git with detections?"** YES. The §14.7 recipe library is a
**curated, reusable, MITRE/ATT&CK-aligned hunt-recipe corpus** — the textbook Git-worthy case. This is
*already* the C8 decision: L-C8-5 specifies the recipe format as "query text + Sigma-aligned metadata
block + semver + CI harness, stored in Git under the same repo as detection rules, subject to peer
review and CI validation." [config:C8 L-C8-5] The narrow cut's "detection content = detection rules +
hunt recipes in Git" is fully consistent with C8 and with the surveyed prior art. **No conflict.**

**Where the line falls for Prism's "saved/scheduled queries" (the "(maybe)" in the hypothesis):**
- A **scheduled query that IS a detector** (fires alerts on a schedule) is detection content → Git.
  In Prism's model this is just a detection rule with a schedule; it rides in the detection-content Git
  domain. C6's RBA/scheduled-correlation surface lands here.
- An **ad-hoc / personal saved query** (an analyst's stored investigative pivot, a `FIND` they bookmarked)
  is a user artifact → DB. Routing these through PR-review would be intolerable overhead, exactly the
  "would impose substantial overhead on analysts for trivial exploratory work" the research flags. [web]

**LEAN (Q2):** Git domain = detection rules + scheduled-detector queries + §14.7 recipe library
(reviewed, CI-tested, semver'd — already C8 L-C8-5). DB domain = ad-hoc/personal saved queries,
bookmarked pivots, ephemeral hunt sessions. This split is intent-driven, matches every surveyed
platform, and is already half-decided by C8. The narrow cut is RIGHT here. [web]

---

## 3. What is LOST by moving connector-defs / pushdown / satellite-trust / RBAC-roles into the DB?

This is the adversarial core. The deep-research pass found a **strong, growing security consensus that
high-blast-radius config — connectors, trust policy, RBAC role definitions — should be reviewed code,
even when a database is the runtime store.** The honest consequence of pure DB-authority (no PR-review,
no CI-validation, no diff-history, no git-revert) is enumerated below per item, with the reconciling
answer where one exists. [web]

### 3a. Connector DEFINITIONS (the dogfood TOML specs) — DB-authority breaks dogfood, BUT there is a clean reconcile

**What is lost (cited):** moving connector definitions to pure DB-mutation forfeits, specifically —
(1) **change governance** (no PR/peer-review/segregation-of-duties; the research warns directly that
"directly editing configurations in a UI bypasses static analysis, policy checks, and structured
review, increasing the probability of misconfigurations that grant excessive access or expose
secrets"); (2) **diff history** (DB audit tables record *that* a change occurred, not a human-readable
field-level diff in review context); (3) **rollback** ("database-level rollback mechanisms often
operate at coarse granularity and cannot easily revert only a specific connector without affecting
other tables"); (4) **policy-as-code enforcement** (OPA/Rego can't gate a change that only ever existed
as a DB mutation); (5) **cross-environment consistency** (IaC overlays prevent drift; DB-only invites
it). [web]

**Does DB-authority break the "config-as-code, eat-our-own-dogfood" model?** *For the dogfood claim
specifically, yes — if "dogfood" means "the connector spec is a reviewed file that ships in the repo."*
Prism's CLAUDE.md and `feedback_builtin_sensors_config_driven` memory state connectors ship as TOML
specs *to eat our own dog food*. A pure DB-authoritative connector model means there is no longer a
file artifact under review; the dogfood story collapses into "the UI writes a DB row." [config:CLAUDE.md]

**The reconciling answer — Grafana-provisioning (author-as-file → import-to-DB → DB-authoritative-at-
runtime).** This is the central find of the Q3 pass and the cleanest resolution. **Grafana's
provisioning system is the exact pattern:** YAML files in `provisioning/datasources/` are
**version-controlled, PR-reviewed, CI-validated**, and **consumed by Grafana at startup to populate
its internal DB; the DB is authoritative at runtime, the UI mutates the DB, but Git is the authority on
INTENT.** [web][grafana-provisioning] The same pattern is independently attested by **Vault policies via
Terraform** (policies live in HCL/Git, applied into Vault's backend) and **Kubernetes RBAC GitOps**
(YAML manifests in Git, reconciled into etcd). [web][vault-tf][k8s-rbac-gitops]

**So: can connector specs be authored-as-files and IMPORTED to the DB without Git being the runtime
authority? YES — and that is the reconciling answer.** It preserves dogfood (the spec is still a
reviewed file shipping in the repo), preserves PR-review + CI-validation + diff-history + git-revert-
of-intent, AND keeps the DB authoritative at runtime (so the UI can still toggle/mutate). It is *not*
the broad C9 "Git-authoritative with continuous reconcile loop" — it is the lighter "file-as-bootstrap-
import" half. **This is materially cheaper than the broad C9 reconcile loop and closes the dogfood hole.**

**Honest residual:** the file-import path still needs *something* — a CI gate + an import command +
a "this row was provisioned from `<commit>`" provenance tag on the DB row so an operator can tell
provisioned-config from UI-mutated-config (Grafana's known weakness is exactly that UI edits can
silently diverge from provisioning files). That provenance tag is the minimum viable reconcile.

### 3b. Pushdown descriptors (C3) — coupled to connector definitions, same treatment

Pushdown capability descriptors describe what each connector can evaluate natively — they are
*intrinsic to the connector definition* and change in lockstep with it (a connector that gains a new
queryable field gains a new pushdown capability). They are high-blast-radius in a subtler way: a wrong
pushdown descriptor doesn't crash anything, it **silently produces wrong/empty query results** (the
exact SAP-2 DTU↔TOML parity failure mode in CLAUDE.md — a column declared but unsupported normalizes to
empty data). [config:CLAUDE.md SAP-2]

**LEAN (Q3b):** pushdown descriptors ride with connector definitions in the **same file-authored-import
domain** (3a). They should NOT be pure-DB-mutable for the same reasons connector defs should not, *plus*
the silent-wrong-results hazard makes the CI-validation gate (does the descriptor match the DTU/adapter's
real capability?) more valuable than for any other config class. The narrow cut's "pushdown in pure DB"
is the weakest non-trust item — co-locate with connector defs in the import path.

### 3c. SATELLITE TRUST / TOPOLOGY policy — the highest-blast-radius item; the narrow cut is most likely WRONG here

**This is the one to be adversarial about, and the research is adversarial.** Putting trust policy in a
mutable DB (UI-editable, no PR-review, no diff-history outside the DB) runs against a **strong, multi-
source security-engineering consensus that trust and authorization policy should be reviewed code.** [web]

The prior art (all [web]):
- **SPIFFE/SPIRE** persists registration entries (SPIFFE IDs + selectors that determine *which workloads
  receive trusted identities*) in its datastore, but the ecosystem treats these as governed artifacts;
  *"a wrongly configured registration entry could grant a SPIFFE ID to a workload that should not have
  it, allowing that workload to authenticate to services as if it were a trusted component"* — a
  textbook high-blast-radius-without-review failure. [spiffe-config][spiffe-register]
- **Istio / Cloud Service Mesh AuthorizationPolicy** is a Kubernetes CRD — *authored as YAML, typically
  in Git, PR-reviewed* — and the documented best practice ("start allow-nothing, add ALLOW incrementally")
  *"is a pattern well-suited to code-based management where each added policy is a change set that can be
  reviewed and tested."* The research warns that UI/DB-adjusted policies invite *"ad hoc exceptions for
  'emergency' situations that are never rolled back or properly documented."* [web]
- **Teleport** roles (which embody trust: which clusters/apps/DBs a principal can reach) persist to an
  etcd backend but are routinely managed as config files / Terraform. [teleport-roles][teleport-backends]
- **OPA + GitOps-security** literature *"explicitly advocates enforcing policy as code... to guard
  against misconfigurations and align with Zero Trust assumptions"* — trust/authz declared as code in
  repos, enforced by controllers, *"rather than being manually tweaked in UIs that bypass review."* [web]

**The blast-radius argument, stated plainly (cited):** trust policy decides *which satellites are part
of the trusted fabric and what they can reach.* A misconfigured trust entry made via UI with no review
gate can *"grant a SPIFFE ID to a workload that should not have it"* / *"permit previously denied
communications, enabling lateral movement"* — and *"without a code-based representation and review,
such misconfigurations might go unnoticed until exploited."* This is the single highest-consequence
item in the entire config plane, and it is the one the narrow cut most clearly mis-files. [web]

**Honest counter-weight (the research is fair about this):** vendor docs do NOT *explicitly* label
"trust-in-mutable-DB-without-review" as a named anti-pattern — *"the absence of an explicit anti-pattern
label likely reflects that vendors must support a variety of operational models, including smaller
teams that rely on UIs."* The research's own characterization: it is best described as a *"non-aligned"
pattern relative to current best practices, if not formally an anti-pattern.* So this is a STRONG-
consensus-against, not a unanimous-prohibition. [web]

**LEAN (Q3c — adversarial):** satellite trust/topology policy should NOT be pure-DB-mutable. It belongs
in the **same file-authored-import path** as connector defs (3a) — authored as reviewed files, CI-
validated, diff-tracked, git-revertible-of-intent, imported into the SQLite/Postgres control-plane that
remains runtime-authoritative. This is the item where the broad C9 split's instinct (trust = Git-
authoritative) was *correct* and the pure narrow cut is *wrong*. It also dovetails with Prism's existing
residency-as-structural-reject discipline (D-C2-12 / D-C5-3) — trust policy is the kind of thing you
want reject-at-review, not discover-after-exploit. **Do not let trust topology be a UI-only DB row.**

### 3d. RBAC role DEFINITIONS vs ASSIGNMENTS — definitions soft-belong in reviewed-code; assignments uncontested DB

**Assignments (which user has which role): uncontested DB.** Volatile, HR-driven, high-volume,
directory-integrated. Every source agrees assignments live in the DB/directory. The narrow cut is
unambiguously right here. [web]

**Role DEFINITIONS (the permission sets): best-practice-is-reviewed-code, but NOT mandated.** The
research found *strong-but-indirect* support for definitions-as-code: AWS IAM roles/policies via
Terraform/CloudFormation, Vault policies via Terraform, Kubernetes Role/ClusterRole YAML in Git
(vCluster GitOps explicitly recommends *"organizing RBAC policies in a Git repository to enable version
control"*), OPA/Rego policy-as-code, Boundary roles via Terraform. The asymmetry rationale (cited):
*"role definitions, being relatively stable and high-impact, benefit from the rigor of code-based
management; assignments, being volatile and driven by HR changes, benefit from database-backed,
directory-integrated processes."* [web][k8s-rbac-gitops][vault-tf]

**BUT — the honest caveat the research itself flags:** *"We did not find explicit guidance that states
'role definitions should be in Git and assignments in a database.' The pattern is inferred... the claim
that role definitions should be treated as reviewed code is strongly supported, while the claim that
assignments must live in a database is more a pragmatic observation than a prescriptive consensus."*
AND the SIEM platforms surveyed in Q1 (Panther, Elastic, Splunk) **ship RBAC role management as DB/UI
by default** — none Git-manages role definitions out of the box. [web]

**LEAN (Q3d):** RBAC is the *softest* of the four. Two defensible positions:
- **(Lighter, matches SIEM default)** Keep BOTH role definitions and assignments DB-authoritative
  (the Panther/Elastic/Splunk default). Acceptable IF baseline-role definitions are seeded from a
  reviewed bootstrap file at install (same import path) and changes are audit-logged with analyst
  identity (ADR-051). The blast radius of a role-def change is contained by the fact that Prism's
  permission *surface* is itself defined in reviewed code.
- **(Stricter, matches IAM/zero-trust best practice)** Put baseline role *definitions* in the file-
  import path (3a), assignments in DB. This is what the broad C9 split did ("baseline global RBAC role
  definitions" Git-authoritative).
  
  Either is defensible; the narrow cut's "both in DB" is *acceptable* (unlike trust, which is *not*),
  provided baseline roles are seeded-from-reviewed-file and every role-def mutation is audit-rowed.
  Recommend the lighter option unless a compliance regime (the §11.2 audit requirement) demands role-
  def diff history, in which case use the import path. [web]

---

## 4. The simplification WIN + residual risks

### 4a. What the narrow cut buys (quantified, honestly)

The narrow cut's payoff is real and is the strongest argument *for* it:

- **One authority model for ~everything.** DB-authoritative for connectors-toggle, schema mappings,
  per-tenant overrides, feature flags, retention, RBAC assignments, credential references — UI/API-
  mutated, hot-reloaded via the existing ArcSwap path (AD-007). No bifurcation of the config plane.
- **No per-key classification sprawl.** The broad C9 split's single most expensive design artifact was
  *"an explicit, enumerated, versioned per-key Git-vs-DB classification that must be maintained as
  config keys are added"* (OQ-C9-1). The narrow cut **deletes that artifact almost entirely** — the
  only Git-classified things are detection rules + recipes, which are *already* a separate content
  type with their own repo. This is a genuine, large complexity reduction. [config:C9 OQ-C9-1]
- **No broad reconcile loop.** The broad C9 split required *"a converge-store-to-Git loop with drift
  surfacing... real day-2 implementation, not a wiring change"* (C9 Honest Costs). The narrow cut
  **only detection content reconciles** (and per Q5 below, even that is import-not-reconcile in
  practice). The DB-authoritative majority needs no reconcile loop at all. [config:C9 Honest Costs]
- **Matches mainstream SIEM practice** (Q1) — lower "we invented something weird" risk.

This is a defensible, attractive simplification. The broad C9 split was the GitOps-everything end of
the spectrum; the narrow cut is the mainstream end. **If the two security holes (3c trust, 3a/3b
connector-defs+pushdown) are patched with the thin import path, the net is: a much simpler config plane
than broad-C9, with the high-blast-radius items still protected.**

### 4b. Residual risks (the honest costs of the narrow cut)

- **§11.2 versioned-change-control now rests ENTIRELY on the Postgres audit table + DB snapshotting for
  the DB-authoritative majority — and that is NOT a full substitute for git-revert for high-blast-radius
  items.** The research is explicit: *"a Postgres audit table combined with database snapshots can
  approximate some aspects of Git's functionality... [but] they are not a full substitute for Git revert
  when managing high-blast-radius configuration such as connectors, trust topology, and RBAC role
  definitions."* Specific gaps: DB rollback can't revert *intent that was never valid* (the bad change
  was applied directly to runtime); DB snapshots are coarse-grained (can't revert one connector without
  touching unrelated rows); audit-table inspection is SQL-technical, not human-readable-diff-in-review.
  [web] **→ This is the precise reason 3a/3b/3c argue for the file-import path on the high-blast items:
  it restores git-revert-of-intent for exactly the things the audit table can't safely roll back.**
- **The CrowdStrike "bad config push" lesson now applies to DB-pushed connector/pushdown config that has
  NO git-revert.** The research connects this directly: *"if a security platform pushes connector
  configurations, trust policies, or agent behaviors from a central database to edge nodes... and those
  configurations are not governed by Git with easy revert, a similar [CrowdStrike-style] situation could
  arise... If rollback relies on database snapshots or ad hoc scripts, it may be slow and error-prone."*
  [web] **How rollback is handled for DB-authoritative high-blast items under the pure narrow cut:** the
  only available mechanisms are (i) restore-prior-audited-generation from the Postgres audit table
  (requires the audit table to store full before/after row state, not just a change-event), and (ii)
  DB point-in-time-recovery (coarse, collateral). Neither gives the fast, surgical, reviewed git-revert
  the CrowdStrike post-mortem prescribes. **→ Mitigation:** for connectors/pushdown/trust, the file-
  import path (3a) restores git-revert; for the genuinely-DB-only items (toggles, overrides, flags), the
  blast radius is low enough that audit-table-restore + the existing edge validate-before-swap +
  canary-cohort defense (C9 Q6 three-layer blast defense) is sufficient. The edge fail-closed (a
  satellite that gets bad config keeps its last-good generation and reports DEGRADED, never crashes) is
  the load-bearing safety net and is UNCHANGED by the narrow cut. [config:C9 §6.5]
- **Audit-table completeness becomes a hard requirement, not a nice-to-have.** Under the broad C9 split,
  Git provided who/what/when for the Git-authoritative half. Under the narrow cut, the Postgres audit
  table (§14.3) is the *sole* who/what/when for ~everything. It MUST capture analyst identity (ADR-051),
  timestamp, before/after row state (for restore), and ideally a justification field for high-blast
  changes — i.e. it must reach the audit grade Git would have given for free. This is more weight on the
  audit table than the broad split placed on it. [web][config:CLAUDE.md ADR-051]
- **No CI-validation gate for DB-mutated connector/pushdown config** (unless the import path is adopted).
  The SAP-2 DTU↔TOML parity check (CLAUDE.md) is a *CI gate today* precisely because connector specs are
  files. Pure DB-authority for connector defs would lose that gate — a connector spec edited in the UI
  could declare a column the adapter can't produce, silently breaking normalization. **→ The file-import
  path (3a) preserves the SAP-2 CI gate; pure DB-mutation loses it.** This is a concrete, named
  regression, not a hypothetical. [config:CLAUDE.md SAP-2]

---

## 5. The detection-content Git↔DB mechanic (the thin Git domain)

**Verdict: for Prism's detection-content Git domain, the realistic mechanic is Git-as-one-way-import
(CI pushes content into the runtime store via API), NOT Git-authoritative continuous reconcile —
because that is what EVERY surveyed detection-as-code platform actually ships.** The continuous-reconcile
(Argo-CD/Flux drift-correction) model is *aspirational* in the detection-as-code world, not the default. [web]

**Cited finding (decisive):** *"none of the platforms examined here have implemented a full Argo-CD-style
controller that continuously reconciles the rule database from Git... each relies on discrete deployment
actions (CI pipeline runs, CLI commands, Terraform applies). Drift detection and correction are largely
manual or process-driven."* [web]

| Platform | Git→DB mechanic | Continuous reconcile / drift? |
|---|---|---|
| Panther | Panther Analysis Tool (PAT) CLI → API upload/delete [panther_analysis_tool] | None; CI or manual PAT runs |
| Elastic | `detection-rules` CLI wrapping Kibana rule CRUD APIs (`kibana import-rules`) [elastic-cli] | None; CI/CLI imports; UI edits cause drift |
| Splunk | `contentctl` build → Splunk app → installed [splunk-security-content] | None; CI produces apps; UI edits drift |
| Chronicle/SecOps | GitHub Actions → `content_manager rules update`; Terraform `google_chronicle_rule`; **platform ALSO keeps its own rule version history + UI rollback** [secops-dac] | None continuous; platform-side version history is a *separate* axis from Git |

**Implication for Prism:** the detection-content Git domain should be **Git-as-import** (CI validates
recipes/rules against fixtures per C8 L-C8-5's CI harness, then deploys into the RocksDB/control-plane
detection store via the engine's API), with the runtime store as the live representation. A *light*
drift-surfacing affordance (a "this rule was edited in-console, diverges from Git" flag) is a nice-to-
have, NOT a full reconcile loop. This is materially cheaper than the broad C9 "Git-authoritative with
reconcile loop + drift self-heal" and matches the real-world default. **The narrow cut's thin Git domain
is therefore even thinner/cheaper than the broad split assumed.** [web]

**How the C6 rule lifecycle (shadow/canary/rollback) interacts with the Git domain:**
- **Rule lifecycle status fields exist in code** (Sigma `status: experimental|test|stable`; C8 L-C8-5
  already adopts `status: experimental|stable|deprecated`) — but the research found that **shadow/canary
  are implemented as org conventions + deployment scope + alert-routing tricks, NOT first-class runtime
  modes** in any surveyed platform. [web] Prism's C6 D-C6-3 goes *further* than the prior art here: it
  specifies a *first-class* shadow→canary→production state machine with a per-tenant circuit-breaker +
  CORROBORATION-MASTER-GATE. **That is ahead of the prior art (C6 already says so) and is RUNTIME state,
  not Git state.** The rule's *definition* lives in Git (import); the rule's *rollout state* (shadow/
  canary/production, circuit-breaker open/closed) lives in the DB and is mutated by the runtime control
  loop. These are two different axes and the narrow cut handles them cleanly: **definition in Git-
  import-domain; rollout/lifecycle state in DB.** No conflict with the narrow cut. [config:C6 D-C6-3]
- **Rollback for detection content** is git-revert-then-redeploy (the C6 D-C6-3 "REVERT-TO-PRIOR-VERSION
  = one-click HUMAN action" maps onto git-revert-of-the-rule-file + re-import), complemented by the
  runtime demote-to-shadow auto-action (which is a DB state change, not a Git change). Both coexist:
  fast runtime demote-to-shadow for the circuit-breaker trip; deliberate git-revert for a bad rule
  definition. This is exactly the dual Git-rollback + platform-rollback the research observed in
  Chronicle/SecOps. [web][config:C6 D-C6-3]

**Does suppression-as-code ride in the detection-content Git domain too? — YES, and C6 already decided it does.**
- The research found **suppression-as-code is "emerging, largely aspirational"** in the wider industry —
  Elastic exposes exception lists as TOML in the `detection-rules` project (so it CAN be code), Panther
  ties saved-query filters to rules, Splunk uses lookups/macros, but **the DEFAULT across all platforms
  is UI/DB-created runtime exceptions, not version-controlled suppression artifacts.** [web]
- **C6 D-C6-2 already made suppression-as-code MANDATORY for Prism** (versioned typed object in the
  detection repo, mandatory `justification:`, mandatory `expires_at:` time-box, CI-validated, fire-
  frequency dashboard). [config:C6 D-C6-2] **This is AHEAD of the industry default — and it is exactly
  right that it rides in the detection-content Git domain**, because a suppression is a coverage-reducing
  decision (it can mask a true positive) and is precisely the kind of thing that *must* get PR-review +
  diff-history + expiry-tracking that only the Git domain provides. **The narrow cut is fully consistent
  with C6 D-C6-2: suppression-as-code is detection content → Git.** Putting suppression in the pure-DB
  bucket would directly contradict C6 D-C6-2 and would be a security regression. So suppression-as-code
  is firmly IN the thin Git domain, not in the DB-authoritative majority. [web][config:C6 D-C6-2]

---

## Verdict on the narrow cut (is it sound / where it's risky)

**SOUND, with two patches.** The narrow cut's core thesis — *Git holds detection content (rules + hunt
recipes + suppression-as-code + scheduled-detector queries); the DB holds operational config* — is
**mainstream, well-attested, materially simpler than the broad C9 split, and consistent with the
settled C6/C8 detection decisions.** It is the *common* real-world pattern; the broad C9 split was the
*less-common GitOps-everything* end.

**Where it is RISKY (in descending order of consequence):**

1. **Satellite TRUST/topology policy in a pure mutable DB (no review gate) — HIGHEST RISK, most likely
   wrong.** Strong multi-source security consensus (SPIFFE/SPIRE, Istio, Teleport, OPA, zero-trust/
   GitOps-security) holds that trust/authorization policy should be reviewed code. Blast radius of an
   un-reviewed trust mutation is lateral movement / wrongful trust grant that *"goes unnoticed until
   exploited."* The broad C9 split got this right; the pure narrow cut gets it wrong. **PATCH: trust
   policy → file-authored-import path (reviewed file, CI-validated, git-revertible-of-intent, imported
   into the runtime-authoritative control-plane).** [web]
2. **Connector DEFINITIONS + pushdown descriptors in pure DB — MEDIUM-HIGH RISK.** Breaks the dogfood
   config-as-code model, loses the SAP-2 CI parity gate (silent-wrong-results hazard), loses git-revert
   for high-blast connector changes (CrowdStrike lesson). **PATCH: same file-authored-import path
   (Grafana-provisioning model) — preserves dogfood + SAP-2 gate + git-revert-of-intent, keeps DB
   runtime-authoritative + UI-mutable.** [web]
3. **RBAC role DEFINITIONS in pure DB — LOW-MEDIUM RISK / ACCEPTABLE.** Best practice leans reviewed-code,
   but it is *not* mandated and the SIEM default is DB/UI. Acceptable IF baseline roles are seeded from a
   reviewed bootstrap file and every role-def mutation is audit-rowed. Assignments-in-DB is uncontested. [web]
4. **Everything else (connector toggles, schema mappings, per-tenant overrides, feature flags, retention,
   credential references, RBAC assignments, ad-hoc saved queries) in pure DB — NO RISK / CORRECT.** This
   is exactly the mainstream default and the narrow cut nails it. [web]

---

## Recommended boundary (exactly what is Git vs DB, edge cases resolved)

| Config class | Recommended authority | Rationale |
|---|---|---|
| Detection rules | **Git (import)** | detection-as-code default; C8 L-C8-5 [web] |
| §14.7 hunt-recipe library | **Git (import)** | curated reusable hunt corpus; C8 L-C8-5 [web] |
| Scheduled queries that ARE detectors | **Git (import)** | detection content (a scheduled detection rule) [web] |
| Suppression / exceptions / allowlists | **Git (import)** | C6 D-C6-2 mandatory suppression-as-code (justification + expiry + CI); coverage-reducing → must be reviewed [web][config:C6] |
| **Satellite TRUST / topology policy** | **Git-of-intent → import (NOT pure DB)** | highest blast radius; trust-as-reviewed-code consensus; PATCH #1 [web] |
| **Connector DEFINITIONS (dogfood TOML)** | **Git-of-intent → import (NOT pure DB)** | preserves dogfood + SAP-2 CI gate + git-revert; Grafana-provisioning model; PATCH #2 [web] |
| **Pushdown descriptors (C3)** | **Git-of-intent → import (co-located with connector defs)** | intrinsic to connector def; silent-wrong-results hazard needs CI gate; PATCH #2 [web] |
| RBAC role DEFINITIONS | **DB (seeded from reviewed bootstrap file) — acceptable**; OR import path if §11.2 audit demands diff-history | softest item; SIEM default is DB; best-practice leans reviewed-code [web] |
| Connector enable/disable toggles | **DB** | per-tenant runtime, UI-mutable; mainstream default [web] |
| configure-schema mappings | **DB** | operational, UI-mutable [web] |
| Per-tenant overrides | **DB** | runtime, UI-mutable [web] |
| Feature flags | **DB** | runtime toggle [web] |
| Retention / RETAIN policy | **DB** | operational policy, UI-mutable [web] |
| Credential REFERENCES (not values) | **DB** | reference-based, AI-opaque (AD-017); values never in config [web][config:AD-017] |
| RBAC ASSIGNMENTS | **DB** | volatile, HR-driven; uncontested [web] |
| Ad-hoc / personal saved queries | **DB** | user artifacts, not detection content [web] |

**The unifying shape:** **one DB-authoritative runtime store + a thin Git-of-intent → import path** that
covers (a) detection content [the narrow cut's Git domain] AND (b) the three high-blast-radius non-
detection items [trust, connector defs, pushdown — the PATCH]. The import path is the *same* mechanism
for both — Grafana-provisioning-style author-as-file, CI-validate, import-into-DB, DB-authoritative-at-
runtime, provenance-tag-the-imported-rows. This is **strictly cheaper than the broad C9 split** (no
per-key classification sprawl, no continuous reconcile loop — import not reconcile) while closing the
two security holes the *pure* narrow cut opens. The narrow cut "wins" on simplicity; the patch costs a
small import path it would have needed for detection content anyway.

---

## Consolidated Open Design Questions

| # | Open question | Where it lands |
|---|---|---|
| OQ-NARROW-1 | Confirm the thin Git domain is exactly {detection rules, §14.7 recipes, scheduled-detector queries, suppression-as-code} — and that ad-hoc saved queries are explicitly DB. The line is intent/lifecycle, not storage. | morph ADR (config-authority model) |
| OQ-NARROW-2 | Adopt the file-authored-import (Grafana-provisioning) path for connector defs + pushdown descriptors + satellite trust policy? (The recommended PATCH.) Or accept pure-DB for connector/pushdown and import-path for trust only? | morph ADR (PO + architect + security) — the central fork |
| OQ-NARROW-3 | Satellite trust policy: confirm it is NOT pure-DB-mutable. If DB-stored at runtime, require reviewed-file-of-intent + import + ADR-051 audit + (recommended) a human approval gate on trust mutations. Security-reviewer should weigh in. | morph ADR + security-reviewer |
| OQ-NARROW-4 | RBAC role definitions: lighter (DB, seeded-from-reviewed-file) vs stricter (import path). Decide based on whether §11.2 compliance demands role-def diff-history. | morph ADR |
| OQ-NARROW-5 | Postgres audit table schema MUST capture before/after row state (for restore) + analyst identity (ADR-051) + timestamp + (for high-blast classes) justification. Confirm it reaches audit grade Git would have given. | morph BC + data-engineer |
| OQ-NARROW-6 | DB-authoritative rollback mechanism for the genuinely-DB-only items: audit-table-restore-prior-generation + edge validate-before-swap + canary cohort (C9 §6.5 three-layer defense). Define the restore procedure. | morph ADR |
| OQ-NARROW-7 | Provenance tag on imported DB rows ("provisioned from `<commit>`" vs "UI-mutated") so operators can distinguish import-managed from UI-mutated config (Grafana's known divergence weakness). | morph BC |
| OQ-NARROW-8 | Detection-content Git→DB mechanic = Git-as-import (CI deploy via engine API), NOT continuous reconcile. Confirm a light drift-surfacing flag is sufficient (no Argo-CD-style controller). | morph ADR |
| OQ-NARROW-9 | SAP-2 DTU↔TOML parity CI gate: confirm it survives whichever connector-def authority is chosen (it requires connector specs to remain file-authored to keep the CI gate). | morph (ties CLAUDE.md SAP-2) |

---

## Honest Costs & Caveats

- **The narrow cut is genuinely simpler than the broad C9 split** — this is its real merit and is not
  oversold. Deleting the per-key Git-vs-DB classification artifact and the broad reconcile loop is a
  large complexity reduction. The recommendation does NOT throw this away; it keeps the narrow cut for
  ~everything and adds back a *thin* import path only for the three high-blast items.
- **The recommended "patch" reintroduces a file path for 3 non-detection classes** (trust, connector
  defs, pushdown). Honest cost: this is a smaller version of the broad C9 split's file domain — but it
  is NOT zero. The narrow cut as the human stated it (those three in pure DB) is the *cheapest* option
  and the *least secure* for those three; the patch trades a small import-path cost for closing two
  security holes. The decision is a real PO+architect+security tradeoff (OQ-NARROW-2), not a slam-dunk.
- **Satellite-trust-in-DB is the place to be most careful, and the prior art is service-mesh/zero-trust,
  NOT SIEM.** The Q1 SIEM survey does NOT validate trust-in-DB because SIEMs have no satellite-trust
  analog. The trust-as-reviewed-code consensus is strong but *not* a unanimous vendor-labeled anti-
  pattern (the research is fair about this — it is "non-aligned with best practice," strong-but-not-
  absolute). A security-reviewer pass at morph is warranted (OQ-NARROW-3).
- **Connector-spec-dogfood edge case has a clean answer (Grafana provisioning)** — this is the most
  satisfying find. Pure DB-authority DOES break dogfood; file-import does NOT. The reconciling pattern
  is well-attested (Grafana, Vault-Terraform, K8s-RBAC-GitOps). This is the recommendation's strongest
  leg.
- **Postgres audit + snapshot is NOT a full git-revert substitute for high-blast-radius config** — cited
  directly. This is the load-bearing reason the high-blast items want the import path. For low-blast
  DB-only items (toggles, flags, overrides) the audit-table + edge-fail-closed + canary defense is
  sufficient and the narrow cut is correct.
- **C6/C8 consistency confirmed, not reopened.** Suppression-as-code (C6 D-C6-2), recipe format (C8
  L-C8-5), and the shadow/canary/rollback state machine (C6 D-C6-3) all sit cleanly: definitions/recipes/
  suppressions in the Git-import domain; rollout/circuit-breaker state in the DB. The narrow cut does
  not disturb any settled C6/C8 decision. Rule *definition* (Git) vs rule *rollout state* (DB) is a
  clean two-axis separation.
- **Prior art is incomplete at the federated-satellite-trust seam.** No surveyed platform manages a
  multi-satellite trust topology the way Prism does; the trust guidance is transplanted from service-
  mesh/zero-trust. Prism is partly in unmapped territory on satellite trust — expect bespoke design
  there regardless of which authority is chosen.
- **Leans / Verdict are discussion input only.** The central fork (OQ-NARROW-2: import-path-for-the-
  three-high-blast-items vs pure-DB) is PO + architect + security-reviewer adjudication at morph, not
  decided here.

---

## Citations

**Q1 — detection-content-only-in-Git pattern (per-platform):**
- [panther-analysis] github.com/panther-labs/panther-analysis
- [panther-rbac] docs.panther.com/system-configuration/rbac
- [panther-DaC] panther.com/blog/how-detection-as-code-revolutionizes-security-posture
- [panther-logs] docs.panther.com/data-onboarding/supported-logs
- [dropzone] docs.dropzone.ai/integrations/alert/panther_alert
- [elastic-detection-rules] github.com/elastic/detection-rules
- [elastic-rbac-thread] discuss.elastic.co/t/what-permissions-are-needed-to-manage-fleet-agent-policies-integrations/286650
- [elastic-prebuilt-thread] discuss.elastic.co/t/prebuilt-security-detection-rules-in-policy-or-just-install-assets/312959
- [elastic-fleet] elastic.co/docs/reference/fleet
- [splunk-security-content] github.com/splunk/security_content
- [splunk-inputs] help.splunk.com (inputs.conf reference)
- [splunk-rbac] help.splunk.com (centralized user and role management)
- [chronicle-detection-rules] github.com/chronicle/detection-rules
- [secops-dac] security.googlecloudcommunity.com — Getting Started with Detection-as-Code and Google SecOps (Part 2)
- [secops-terraform] docs.cloud.google.com/chronicle/docs/terraform
- [sigmahq] github.com/sigmahq/sigma ; sigmahq.io/docs/basics/rules.html

**Q2 — saved queries / hunt recipes:**
- [splunk-savedsearches-thread] community.splunk.com — How to maintain savedsearches.conf in code repository
- [elastic-saved-objects] elastic.co/docs/explore-analyze/find-and-organize/saved-objects ; elastic.co/docs/explore-analyze/discover/save-open-search ; elastic.co/docs/api/doc/kibana/group/endpoint-saved-objects
- [secops-udm-search] docs.cloud.google.com/chronicle/docs/investigation/udm-search
- [secops-retrohunt] docs.cloud.google.com/chronicle/docs/detection/run-rule-historical-data
- [threathunter-playbook] github.com/OTRF/ThreatHunter-Playbook
- [target-hunting] github.com/target/Threat-Hunting

**Q3 — connector/trust/RBAC consequences + reconcile patterns:**
- [grafana-provisioning] grafana.com/docs/grafana/latest/administration/provisioning/
- [coralogix-iac] coralogix.com/blog/security-risks-infrastructure-as-code/
- [spiffe-config] spiffe.io/docs/latest/deploying/configuring/
- [spiffe-register] spiffe.io/docs/latest/deploying/registering/
- [teleport-roles] goteleport.com/docs/reference/access-controls/roles/
- [teleport-backends] goteleport.com/docs/reference/deployment/backends/
- [vault-tf] HashiCorp — defining Vault policies with HCP Terraform
- [k8s-rbac-gitops] vCluster — managing Kubernetes RBAC configurations with GitOps
- [argocd-audit] ArgoCD audit-trail guidance (Git commit/PR/sync audit chain)
- [opa] Open Policy Agent / Rego policy-as-code
- (Istio / Cloud Service Mesh AuthorizationPolicy; AWS Terraform prescriptive guidance; Boundary roles — cited inline in the Q3 deep-research corpus)

**Q5 — detection Git→DB mechanic + lifecycle + suppression-as-code:**
- [panther_analysis_tool] github.com/panther-labs/panther_analysis_tool
- [elastic-cli] github.com/elastic/detection-rules/blob/main/CLI.md ; github.com/elastic/detection-rules/releases
- [secops-dac] (as above — rule version history + content_manager + Terraform)
- (Sigma status field; Splunk contentctl CI/CD blog; Elastic exception-list TOML — cited inline in the Q5 deep-research corpus)

**Internal (non-contradiction reads, not web):**
- [config:C9] research/config-management-depth-2026-06-27.md
- [config:C6] day2-design-decisions/ADR-PROP-detection-engine-depth.md (D-C6-2, D-C6-3)
- [config:C8] day2-design-decisions/ADR-PROP-prismql-deliverables.md (L-C8-5)
- [config:CLAUDE.md] CLAUDE.md (AD-007, AD-017, SAP-2, ADR-051, dogfood TOML connectors)

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 | `reasoning_effort=high`, `strip_thinking=true` on all four. (Q1) detection-content-only-in-Git pattern across Panther/Elastic/Splunk/Chronicle/SigmaHQ — default vs IaC-overlay. (Q2) saved queries / scheduled searches / hunt-recipe libraries — Git vs DB; the natural intent/lifecycle line. (Q3) consequences of DB-authority for connectors/pushdown/satellite-trust/RBAC-roles — Grafana-provisioning reconcile, SPIFFE/SPIRE/Istio/Teleport/OPA trust-as-code, AWS-IAM/Vault/K8s RBAC, CrowdStrike rollback, Postgres-audit-vs-git-revert. (Q5) detection Git→DB mechanic (import vs reconcile), rule lifecycle (shadow/canary/status), suppression-as-code. Each returned 70-89KB single-line JSON; read in full via Read (Q1, Q2) and Grep+Read extraction (Q3, Q5 — line exceeded Read's char cap; verdicts, per-platform claims, and full citation URL lists extracted). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — (no ≤2-sentence lookups needed; version-pinning out of scope for this authority-model pass) |
| Context7 | 0 | — (no library-API question this pass; the question is architectural prior art, not API surface) |
| Tavily (all variants) | 0 | — (four deep-research passes + the C9/C6/C8 internal corpus were sufficient; no cross-validation gap surfaced) |
| WebFetch / WebSearch | 0 | — |
| Training data | 2 areas (flagged) | (1) CrowdStrike July-2024 channel-file RCA *mechanism* — the staged-rollout/blast-radius *lesson* is [web]-confirmed in Q3; channel-file-291 / Content-Validator internals are [model-knowledge], inherited from the C9 pass (re-pull RCA if a BC must cite the mechanism). (2) The two-axis "rule definition (Git) vs rule rollout state (DB)" framing for C6 D-C6-3 is synthesis over the cited Q5 finding (shadow/canary are org-conventions/runtime-state) + the settled C6 decision — flagged as analysis, not a new web claim. |

**Total MCP tool calls:** 4 (all `perplexity_research` at `reasoning_effort=high` [PRIMARY]).
**Training data reliance:** low — every load-bearing claim is [web]-cited from the four deep-research
passes or [config:*]-traced to the settled C9/C6/C8/CLAUDE artifacts; the two [model-knowledge]/synthesis
items are explicitly flagged.

**Deviation note (primary-tool mandate):** the non-trivial four-area authority-model question was led
entirely by `perplexity_research` at `reasoning_effort=high` — the mandated default — with NO
`perplexity_ask`/`search`/`reason` and NO Context7/Tavily, because the question is architectural prior-
art synthesis (not library-API or version-pinning), and four high-effort deep passes covered all five
research areas (Q1–Q5; Q4 simplification/residual-risk was answered from the Q3+Q5 corpora plus the C9
internal pass, not a separate call). No high-effort retry/fallback was needed — all four calls succeeded
on first attempt (oversized results read via Read/Grep, not overload failures).
