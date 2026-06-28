---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C15-1: ARO taxonomy — ARO as closed-loop, security instantiation of OODA/MAPE-K/Endsley-SA/SPA; Observation = 3-layer product (perceive/comprehend/project); shared Knowledge base = C12 Prism Context"
  - "ADR-PROP-C15-2: Autonomy model — recommend-only v1; full ladder (advisory → suggested → auto-with-approval → autonomous) designed now; per-action-class, evidence-measured, REVERSIBLE promotion; CISA invariant: system designers set the gates"
  - "ADR-PROP-C15-3: Data model — three typed entities (Observation/Recommendation/Action) over a common base; type-safe in Rust; rich provenance + calibrated confidence + faithfulness enforcement; terminal-vs-reopenable resolution"
  - "ADR-PROP-C15-4: Recommendation sources — agent-layer (S3) AND read-only PrismQL RECOMMEND projection; perimeter compile-fail-tested; source-discriminated provenance"
  - "ADR-PROP-C15-5: AI-recommendation rigor — W3C-PROV provenance, calibrated confidence + conformal sets, per-citation faithfulness check; mandatory from day one"
  - "ADR-PROP-C15-6: SOAR architecture — separate prism-orchestration layer; connectors-as-actions; HITL approval gates; mandatory idempotency keys; write-creds reference-based at execution tier"
  - "ADR-PROP-C15-7: On-prem models — Qwen3/Mistral-class central; Phi-4-mini/Ministral-class edge; Llama Prompt Guard 2 + Mistral Moderation as guardrails; wasmtime wasi-nn for AI-opaque per-tenant isolation"
  - "ADR-PROP-C15-8: Lifecycle — three coupled state machines + Action-approval sub-machine; terminal-vs-reopenable resolution; ack-timeout retrigger; dedup fingerprint + correlation windows + entity-centric grouping"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C15 capture; human-confirmed decisions 2026-06-27 session.
  Primary research basis: research/aro-model-depth-2026-06-27.md (5 sonar-deep-research passes
  at reasoning_effort=high; OODA/MAPE-K/Endsley-SA/SPA conceptual lineage Q1; three-tier data
  models across SOAR/detection/UEBA/ASM/AI-SOC Q2; AI-recommendation provenance/calibration/
  explainability/citation-faithfulness/auditability Q3; autonomy-level taxonomies + HITL/HOTL/
  HOOTL + NIST/CISA gating Q4; lifecycle state machines + dedup/correlation + idempotency +
  linkage Q5).
  Sweep basis: research/prismql-actions-soar-onprem-models-2026-06-27.md (prior C15 sweep —
  query-language action surfacing, SOAR platform architecture, on-prem model survey).
  Aletheon reference: /Users/jmagady/Dev/aletheon_2/spike/init-db.sql (aros table — one input,
  NOT the answer; C15 generalizes it substantially).
  ADS conformance: ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-01..12, PAT-ADS-01..11,
  INV-ADS-01..08, AP-ADS-01..10).
  Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live
  factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §16.4 (C15 decisions log entry)
  - day2-design-decisions/ADR-PROP-s3-agent-runtime.md (S3 agent — Recommendation-generation layer + autonomy gates)
  - day2-design-decisions/ADR-PROP-prism-context.md (C12 — shared Knowledge base; Entity 360; aletheon aros table input; mandatory citations PAT-ADS-08)
  - day2-design-decisions/ADR-PROP-ml-behavior-analytics-depth.md (C7 — ModelBackend pluggable AI-opaque inference; PAT-ADS-05)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — edge execution; offline action queue + idempotency; AD-017 satellite-local credential resolution)
  - day2-design-decisions/ADR-PROP-sso-identity.md (C18 — RBAC + required_approver_role; gate enforcement)
  - day2-design-decisions/ADR-PROP-detection-engine-depth.md (C8 — RECOMMEND inside saved recipe = declarative authoring path)
  - day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (ADS — P-ADS-10 Idempotent-Gated-Actions; AP-ADS-06 Ungated/Non-Idempotent Auto-Actions forbidden; PAT-ADS-11 ARO-Loop; INV-ADS-05/06)
  - research/aro-model-depth-2026-06-27.md (PRIMARY research)
  - research/prismql-actions-soar-onprem-models-2026-06-27.md (sweep research)
  - epics E-SOAR-ACTIONS-001 (proposed), E-ARO-MODEL-001 (proposed)
  - CLAUDE.md (AD-017 AI-opaque credentials; #[non_exhaustive] discipline; Arc-DI plumbing; production-grade default)
---

# ADR-PROP — Actions in PrismQL / SOAR / ARO Model (C15)

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C15
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred
> to the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/aro-model-depth-2026-06-27.md` — five `perplexity_research`
> (sonar-deep-research) calls at `reasoning_effort=high` covering:
> (Q1) OODA/MAPE-K/Endsley-SA/SPA/control-theory conceptual lineage of ARO;
> (Q2) Three-tier separation as first-class data models across SOAR/detection/UEBA/ASM/AI-SOC;
> (Q3) AI-recommendation provenance, confidence calibration, explainability, citation faithfulness;
> (Q4) Autonomy-level taxonomies, HITL/HOTL/HOOTL definitions, NIST/CISA agentic-AI gating;
> (Q5) Lifecycle state machines, dedup/correlation, idempotency, entity linkage.
> Plus `research/prismql-actions-soar-onprem-models-2026-06-27.md` (sweep).
> Plus aletheon `aros` table (`/Users/jmagady/Dev/aletheon_2/spike/init-db.sql`) as one
> concrete reference input.

---

## Context

Prism is today a **read-only federated security query engine**: PrismQL (Chumsky grammar +
DataFusion planner) fans out to vendor APIs, normalizes to OCSF/protobuf, and returns results
to an LLM agent. C10 GAP-Q2 establishes that Query Workers produce findings + recommendations
but do NOT auto-act. The sensor adapter framework already has feature-flagged write capabilities
(`project_feature_flags.md`).

C15 addresses the full ARO (Action · Recommendation · Observation) model: how Prism closes
the security decision loop from detection through recommended response through gated action
execution, while maintaining the read-only PrismQL perimeter, the AI-opaque credential
invariant (AD-017), and production-grade idempotency/reversibility guarantees.

**Build-on inputs that this pass took as given:**
- Prior C15 sweep (`research/prismql-actions-soar-onprem-models-2026-06-27.md`): three-tier
  ARO sketch; do-not-embed-mutations-in-PrismQL lean; Tines 5 risk axes; `prism-orchestration`
  recommendation; on-prem model candidates.
- Aletheon `aros` table: one concrete pragmatic reference — `aro_type` discriminator,
  dual-audience text, AI provenance triplet, ack/resolve workflow. Treated as ONE INPUT,
  not the final answer; generalized substantially in D-C15-3.
- C10, C12, C18, AD-017, C7 ModelBackend — all treated as settled per their capture artifacts.

---

## Decision Ledger

### D-C15-1 — ARO Taxonomy: Closed-Loop, Security Instantiation of OODA/MAPE-K/Endsley-SA/SPA

**DECIDED 2026-06-27 (human).**

ARO is the security platform instantiation of a family of formalized decision/control loops:

| Loop | ARO mapping | Distinct rigor it adds |
|------|------------|----------------------|
| **OODA** (Boyd) | Observe→Orient→Decide→Act ≈ Observation→Recommendation→Action, **looped** | Recommendation = model-mediated prediction of effects; Observation shaped by prior orientation; Action feeds new Observations — tempo matters |
| **MAPE-K** (IBM autonomic computing) | Monitor→(Analyze+Plan)→Execute over **Knowledge** | Splits comprehension (Analyze) from plan-generation (Recommend); mandatory **shared Knowledge base** (= C12 Prism Context + policy) |
| **Endsley SA** | Observation = Perception→Comprehension→**Projection** | Observation is a 3-layer cognitive product; Projection bridges to Recommendation |
| **Sense-Plan-Act** (robotics) | Sense→Plan→Act, world-model in Plan | Deliberation-vs-reaction tradeoff → structural justification for tiered autonomy |
| **Control theory** | measurement→control-law→actuation, closed-loop | Setpoint/policy must be explicit; adversary = disturbance; loop stability/latency vocabulary |

**Architectural consequences:**

1. **ARO is a closed loop.** Action execution is re-observed via the next PrismQL query cycle.
   Explicit `resulting_observation_refs[]` close the OODA loop — this is not optional bookkeeping,
   it is the mechanism that makes the system a genuine feedback control loop. [Q1-10]

2. **Observation = 3-layer product.** An Observation is not a flat finding. Per Endsley, it
   carries three levels: `perceived_fact` (host X had 412 failed logins), `comprehension`
   (consistent with credential-stuffing on a privileged account), and `projection` (likely lateral
   movement within ~30 min if unmitigated). The projection layer bridges naturally to the
   Recommendation tier — it is simultaneously situational understanding and the basis for what to
   recommend. Raw query results (OCSF-shaped) provide the perceived_fact; the S3 agent adds
   comprehension + projection via the C7 ModelBackend. [Q1-18]

3. **Shared Knowledge base (C12) is first-class.** Per MAPE-K, the Knowledge base underpins all
   three tiers — it is not per-ARO state. The C12 entity model + threat intel (C11) + policy
   setpoints (C9/C18) IS the MAPE-K Knowledge base. ARO entity linkage rides the C12 Context graph
   (already decided: two-layer indradb + vector). **No separate substrate.** (Resolves ARO
   sub-fork 6 from the depth research.) [Q1-14]

4. **Deliberation/reaction tradeoff drives tiered autonomy.** Simple, high-confidence,
   reversible Actions can run via a reactive path (short deliberation); novel/high-impact Actions
   require the deliberative Recommendation+gate path. This framing justifies the autonomy ladder
   in D-C15-2. [Q1-8]

---

### D-C15-2 — Autonomy Model: Recommend-Only v1; Design the Full Ladder Now; Enable Post-v1

**DECIDED 2026-06-27 (human).**

**v1 ships:** Observation tier + Recommendation tier ONLY. ALL Actions are HITL-gated. ZERO
autonomous Action in v1. This matches C10 GAP-Q2 (Query Workers do not auto-act), CISA joint
guidance (Apr 2026), NIST RFI on agentic-AI cybersecurity, and EU AI Act human-oversight
requirements. [Q4-10][Q4-11][Q4-14]

**Design the full autonomy ladder now, enable post-v1.** The four autonomy tiers are:

| Tier | Description | Gating mode | v1 ship? |
|------|-------------|-------------|----------|
| **Advisory** | AI generates; analyst sees it passively | HOOTL | Yes |
| **Suggested** | AI generates; analyst confirms before any action | HOTL | Yes |
| **Auto-with-approval** | Workflow pauses; role-bound approver confirms | HITL | v1 only mode for Actions |
| **Autonomous** | No checkpoint; outcome monitored post-hoc | HOOTL for action | Post-v1, per-class |

**Hard CISA invariant (architectural — non-negotiable):** System designers — NOT the agent —
set the autonomy gates. Agents propose; gates authorize. Agents fail-safe + escalate when
uncertain. They do NOT determine what is low-risk. [Q4-10]

**Promotion criteria (per-action-class, evidence-measured, reversible — Dash0 floors):**
- >90% scenario pass rate over a measurement window
- <5% evaluator false-positive rate
- <10% human-override rate
- Measured **per action class** (not org-wide declaration)
- High-risk/OT/safety action classes **may never qualify** for full autonomy
- Promotion is **reversible** — observable false-positive/impact rise rolls the level back [Q4-15]

**OT/safety hard invariant:** Prism sensors include Claroty/Armis (OT/ICS). HITL is
**mandatory** for any Action on OT-segment assets or safety-system entities, regardless of
confidence or reversibility. CISA OT guidance is categorical: AI/LLMs must NOT make safety
decisions autonomously. [Q4-6]

**Conforms:** AP-ADS-06 (ungated/non-idempotent auto-actions forbidden); P-ADS-10
(Idempotent-Gated-Actions); INV-ADS-05 (all actions gated and idempotent).

---

### D-C15-3 — Data Model: Three Typed Entities (Observation/Recommendation/Action) Over a Common Base

**DECIDED 2026-06-27 (human).**

**Three typed entities sharing a common base.** The tiers diverge enough in fields and state
machines (especially Action's mandatory idempotency key, rollback handle, and approval
sub-machine) that three typed entities is cleaner and more type-safe in Rust than a single
`aro_type`-discriminated table. The common-base pattern preserves aletheon's queryable unity.
All types are `#[non_exhaustive]` per CLAUDE.md §Conventions.

**Keep from aletheon's `aros` table:**
- `aro_type` discriminator pattern (as Rust enum discriminator over common base)
- Dual-audience text (operator-plain / analyst-technical / auditor-structured) — validated by
  NISTIR-8312 "Meaning" principle (audience-appropriate explanations) [Q3-2]
- `priority` (numeric), ack/resolve workflow (`acknowledged_by/at`, `resolved_by/at`)
- `source_event_ids[]`, `affected_asset_ids[]` — generalized below

**Generalize from aletheon (OT/asset-centric → Prism multi-source/multi-entity):**
- `affected_asset_ids[]` → `affected_entity_refs[]` typed by entity kind (host, user, account,
  IP, service, cloud-resource, OT-asset, network-zone) per C12 entity model
- `source_event_ids[]` → `source_observation_refs[]` + `evidence_package_id` (C10 GAP-Q2)
- Flat `status` → **terminal-vs-reopenable resolution sub-statuses** (Xpanse model): [Q5-1]
  - Reopenable: `resolved_no_longer_observed`, `resolved_remediated`, `resolved_remediated_auto`
  - Terminal: `resolved_risk_accepted`, `resolved_no_risk`, `resolved_contested_asset`
- `confidence` (single float) → calibrated confidence block (see D-C15-5 below)
- `reasoning` (free text) → structured explanation block (see D-C15-5 below)

**Add (missing from aletheon, required by research):**
- Loop closure: `resulting_observation_refs[]` on Action (Action → new Observations, D-C15-1)
- Observation Endsley-3-layer structure: `perceived_fact`, `comprehension`, `projection`
- Recommendation risk metadata — Tines 5 axes: `reversibility ∈ {reversible, compensable,
  irreversible}`, `blast_radius`, `novelty`, `asset_tier`, `compliance_flags` [Q4-6]
- Autonomy fields on Recommendation/Action: `autonomy_level`, `gate_mode`
  (HITL/HOTL/HOOTL), `required_approver_role` (C18 RBAC)
- Full AI-provenance block on AI-generated Recommendations (see D-C15-5)
- Citations with faithfulness enforcement (see D-C15-5)
- Dedup fields: `dedup_fingerprint` (fnv64a hash over canonical entity+detection-type labels,
  Alertmanager pattern), `correlation_id`, `collapsed_observation_count` [Q5-12]
- Action-tier: `idempotency_key` (mandatory), `rollback_handle`, `executor_identity`,
  `approver_identity`, `dry_run` flag, `result`, `rollback_status` [Q5-8]
- Immutable structured audit trail (who/what/when/where/why, hashed + tamper-evident) [Q3-16]

**Conceptual entity model:**

```
AROBase:
  id, tenant_id (C18), title, priority, status, resolution_substatus
  audience_text: { operator_plain, analyst_technical, auditor_structured }
  affected_entity_refs[], dedup_fingerprint, correlation_id
  lifecycle_audit: { ack_by/at, resolved_by/at, resolution_notes }
  immutable_audit_trail: [{ who, what, when, where, why, hash }]
  created_at, updated_at

Observation extends AROBase:
  perceived_fact, comprehension, projection   // Endsley 3-layer
  severity, calibrated_confidence
  mitre_tactics[], mitre_techniques[]
  source_detector_id, evidence_refs[]
  collapsed_observation_count
  timestamps: { start, end, generated }

Recommendation extends AROBase:
  source_observation_refs[]
  proposed_actions[]
  // Full AI-provenance block (D-C15-5):
  ai_provenance: { model_card_ref, model_version, prompt_version,
                   system_prompt_version, rag_doc_set, gen_config, seed,
                   generating_agent_id, human_reviewer_id }
  // Calibrated confidence block (D-C15-5):
  confidence: { raw, calibrated, calibration_method, ece, calibration_date,
                conformal_set, conformal_coverage_level }
  // Explanation block (D-C15-5):
  explanation: { contributing_features, rules_invoked[], faithfulness_indicator,
                 knowledge_limits_flag, ood_flag }
  // Citations block (D-C15-5):
  citations: [{ evidence_item_ref, faithfulness_flag, faithfulness_score }]
  unsupported_claim_flag
  evidence_package_id  // C10 GAP-Q2
  // Risk metadata (Tines 5 axes):
  risk: { reversibility, blast_radius, novelty, asset_tier, compliance_flags[] }
  autonomy_level, gate_mode, required_approver_role
  // Source provenance discriminator (D-C15-4):
  source_kind: AgentGenerated | DeclarativeRECOMMEND
  rule_or_recipe_ref  // populated if source_kind = DeclarativeRECOMMEND
  author_set_priority // populated if source_kind = DeclarativeRECOMMEND

Action extends AROBase:
  source_recommendation_ref, source_observation_refs[]  // full provenance chain
  action_type, target, parameters
  idempotency_key  // MANDATORY — C2 offline-queue reconnect retry safety
  dry_run: bool
  executor_identity, approver_identity
  timestamps: { requested, approved, executed }
  exec_status: Pending|Scheduled|Running|Succeeded|Failed|RolledBack
  result, rollback_handle
  resulting_observation_refs[]  // loop closure — D-C15-1
  // Approval sub-state-machine:
  approval_state: AwaitingApproval|Approved|Denied|SlaExpired|Escalated
```

**Rust type safety:** The three types are distinct Rust structs sharing a common trait, not a
single enum or `aro_type`-field-tagged struct. Serde `#[non_exhaustive]` on all public types.
Per-tenant isolation via `tenant_id: OrgSlug` on every record (P-ADS-06, INV-ADS-03).

---

### D-C15-4 — Recommendation Sources: Agent-Layer AND Read-Only PrismQL `RECOMMEND` Projection

**DECIDED 2026-06-27 (human). Both sources (option 1+2).**

Two legitimate Recommendation sources, feeding the SAME typed Recommendation entity with
**source-discriminated provenance** (`source_kind` discriminator on Recommendation):

**Source A — S3 AI agent layer:** The S3 agent runtime (ADR-PROP-s3-agent-runtime.md) generates
dynamic Recommendations. These carry the FULL AI-provenance + calibrated-confidence + conformal +
faithfulness block (D-C15-5). Source_kind = `AgentGenerated`.

**Source B — Read-only PrismQL `RECOMMEND` projection:** PrismQL gains a **read-only data
projection** so a detection query or saved recipe can EMIT a Recommendation as DATA — it never
executes an action, it only produces a typed Recommendation record. Source_kind = `DeclarativeRECOMMEND`.

**THREE non-negotiable conditions on Source B:**

1. **`RECOMMEND` is a PURE read-only data projection with ZERO execution/mutation.** The
   DataFusion logical plan containing a `RECOMMEND` projection never enters the write/action path.
   This invariant is enforced by a **perimeter compile-fail test** — the same pattern as the
   existing `tests/external/perimeter-violation/` security-perimeter gates (E0432 pattern from
   S-PLUGIN-PREREQ-A). A test that attempts to wire a `RECOMMEND` result directly to an action
   executor must fail to compile. This is PIV-C15-3.

2. **Source-discriminated provenance.** Agent-generated Recommendations carry the full AI-provenance
   + calibrated-confidence + conformal + faithfulness block (D-C15-5). Declarative `RECOMMEND`
   Recommendations carry RULE/RECIPE provenance + `author_set_priority` (no model confidence, no
   conformal sets — because there is no model). The two sources MUST NOT be mixed up by downstream
   consumers; `source_kind` is not optional.

3. **`RECOMMEND` inside a saved detection recipe (C8) IS the declarative authoring path.** A
   PrismQL detection recipe that includes a `RECOMMEND` projection is how an analyst or detection
   engineer authors a declarative Recommendation — this subsumes any notion of a separate
   "recipe Recommendation field." The query language stays read-only; the Recommendation record
   that materializes is read-only; Actions execute only in the `prism-orchestration` layer.

**PrismQL read-only perimeter invariant upheld.** PrismQL/DataFusion NEVER executes a write.
The `RECOMMEND` projection is syntactic sugar for emitting a typed record into the ARO data
store — it is equivalent to a `SELECT … INTO recommendations` from a planning perspective, not
a mutating action. (Conforms: P-ADS-10; AP-ADS-06.)

---

### D-C15-5 — AI-Recommendation Rigor: Full Day One Including Conformal Prediction

**DECIDED 2026-06-27 (human).**

AI-generated Recommendations require full rigor from day one. These are NOT polish items —
they are correctness/safety requirements under P-ADS-12 (Production-Grade-Default). The
production-grade default lens: a miscalibrated or unfaithful recommendation in a security
product can cause an analyst to apply the wrong remediation to the wrong asset. [Q3-8]

**Required v1 fields on ALL agent-generated Recommendations (mandatory, not optional):**

**W3C-PROV provenance block** (W3C PROV-DM entity/activity/agent model [Q3-4][Q3-16]):
- `model_card_ref` + `model_version` (ties to model card spec [Q3-5])
- `prompt_version` + `system_prompt_version` (NIST GenAI Profile AI 600-1 requirement [Q3-12])
- `rag_doc_set` (the retrieved-context set used for this recommendation)
- `gen_config` (temperature/top-k/max_tokens), `seed` (non-deterministic seeds for
  reproducibility)
- `generating_agent_id`, `human_reviewer_id` (if reviewed before surfacing)

**Calibrated confidence block** (raw model confidence is NOT trustworthy — miscalibrated by
default [Q3-6]):
- `raw_confidence` (model raw output score)
- `calibrated_confidence` + `calibration_method` (`temperature_scaling` | `platt` | `isotonic`)
- `ece` (Expected Calibration Error) + `calibration_date`
- `conformal_set` + `conformal_coverage_level` — conformal prediction gives distribution-free
  uncertainty SETS with a coverage guarantee (e.g. "true attack type is in
  {credential-stuffing, password-spray} with 95% coverage") [Q3-7]. Narrow set = high certainty;
  wide set = low. **Automation thresholds key off calibrated confidence / conformal-set width,
  NEVER raw scores.**

**Explanation block** (NISTIR 8312 four principles of explainable AI [Q3-2]):
- `contributing_features` (key evidence/features driving the recommendation — DARPA XAI
  "Explanation" principle [Q3-3])
- `rules_invoked[]` + `versions` (which detection rules or playbook steps were relevant)
- `faithfulness_indicator` (was this explanation generated consistently with model internals,
  or is it a post-hoc rationalization?)
- `knowledge_limits_flag` / `ood_flag` (NISTIR 8312 "Knowledge Limits" principle — the system
  MUST declare when it is out-of-distribution / uncertain)

**Citations block with mandatory post-hoc faithfulness check** (PAT-ADS-08
Mandatory-Faithful-Citations; the ~57%-unfaithful RAG finding [Q3-8]):
- `citations[]`: each entry maps a factual claim to a source evidence item
  (OCSF event ID / detection rule / asset record from C12 entity model)
- Per-citation: `faithfulness_flag` (VERIFIED | UNVERIFIED | FLAGGED) and `faithfulness_score`
- `unsupported_claim_flag`: if any claim in the recommendation text cannot be mapped to a
  citation in the evidence, this flag is set and the recommendation MUST be flagged before surfacing
- **Mandatory post-hoc statement-vs-citation faithfulness check** before surfacing to any analyst.
  The Output Hardener component (S3 agent runtime, ADR-PROP-s3-agent-runtime.md) MUST run
  this check. An uncited claim is either cited or removed — it is never displayed raw.

**Faithfulness-check mechanism:** Post-hoc statement-vs-citation verification is the minimum
viable enforcement mechanism for v1. The specific implementation (constrained generation vs
token attribution vs post-hoc NLI check) is OQ-C15-5 — deferred to implementation with
explicit story anchor in E-ARO-MODEL-001. The OUTPUT requirement (every displayed claim has a
verified citation or is removed) is NOT deferred.

**Conforms:** PAT-ADS-08 (Mandatory-Faithful-Citations); P-ADS-07 (AI-Opaque); INV-ADS-06.

---

### D-C15-6 — SOAR Architecture: Separate `prism-orchestration` Layer

**DECIDED 2026-06-27 (human). Affirms and extends the prior C15 sweep lean.**

A separate `prism-orchestration` subsystem owns all action execution. PrismQL/DataFusion NEVER
executes writes. This is the near-universal industry pattern (Splunk discourages inline
`sendalert` even in SPL; Sentinel/Tines/XSOAR all separate detection from action). [SOAR-2]

**`prism-orchestration` components:**

| Component | Purpose | Key constraints |
|-----------|---------|----------------|
| Playbook/workflow engine | DAG of tasks: query → condition → Recommendation → Action → loop | Tasks reference PrismQL queries as inputs; NEVER embed actions in PrismQL |
| Connectors-as-actions | Reusable action plugins with declared inputs/outputs/reversibility | Feature-flagged sensor write capabilities; Splunk CAM discipline (discrete, single-purpose, declared schema) |
| HITL approval gate | Pause-and-wait checkpoint, role-assigned (C18), SLA-bound | Splunk SOAR Prompt block pattern; required_approver_role from Recommendation; escalation on SLA expiry |
| Case management | Group Observations/Recommendations/Actions into investigations | Links ARO records + evidence packages (C10 GAP-Q2) |
| Audit subsystem | Immutable trail: what was seen/decided/executed, by whom, with what result | RocksDB-backed audit CF; every ARO transition emits a Canonical Structured Event (BC-2.16.002 catalog — SAP-1 obligation at morph) |
| Rollback/undo | Compensating action handle for reversible/compensable actions | Recommendation declares `reversibility` + paired compensating Action connector; first-class, not manual |
| Blast-radius controller | Per-action blast cap + dry-run enforcement | `dry_run` flag on Action; blast_radius from Recommendation risk metadata |

**Idempotency key invariant (PIV-C15-2):** Every Action carries a **mandatory client-generated
idempotency key**. This is doubly critical given C2 satellite offline-queue reconnect retries —
a reconnect MUST NOT double-apply an Action. Same pattern as PagerDuty `dedup_key`. [Q5-8]

**AI-opaque write-creds invariant (PIV-C15-6 — AD-017 extension to write path):** Write-action
credentials are resolved **reference-based at the execution tier** in `prism-orchestration`.
The AI agent never holds write credentials. The resolution follows the same satellite-local
credential model as C2/AD-017. The agent receives only the action parameters + a credential
reference; the orchestration layer resolves the reference to the actual credential at execution
time. *This is not a new mechanism — it is the existing AD-017 read-credential pattern
applied to the write path.*

**Edge/satellite action execution:** Central-decide / satellite-execute. When an Action targets
an OT-segment or edge-resident asset, the approval gate fires at Central; the approved action
payload (with idempotency key) is delivered to the satellite via the C2 conduit; the satellite
executes using satellite-local credentials (AD-017). For offline scenarios (air-gap), actions
queue in the C2 store-and-forward RocksDB CF with at-least-once + dedup semantics. [ADR-PROP-satellite-mesh.md D-C2-10]

---

### D-C15-7 — On-Prem Models (Plug Into C7 Pluggable AI-Opaque ModelBackend)

**DECIDED 2026-06-27 (human). Final picks via benchmark — OQ-C15-4.**

On-prem model candidates plug into the C7 `ModelBackend` trait (candle/ort/wasmtime/tract).
These are CANDIDATES pending the OQ-C15-4 benchmark; candidates are not locked final picks.

| Role | Candidate models | Rationale |
|------|-----------------|-----------|
| Central reasoning (Recommendation generation) | Qwen3-class, Mistral-class | Instruction-following, tool-use, multilingual; SLM family fits the 512MB budget with quantization |
| Edge reasoning (satellite-local Recommendation) | Phi-4-mini-class, Ministral-class | Smaller footprint for satellite/edge memory envelope; fast inference |
| Guardrails (prompt injection + moderation) | Llama Prompt Guard 2, Mistral Moderation | Classifier-level overhead; hardened for adversarial prompt detection |
| Sandboxed AI-opaque per-tenant isolation | wasmtime wasi-nn | Strongest isolation story: WASM component model + WASI-NN inference API; per-tenant sandbox boundary at the model call level (reuses C4/C7 WASM sandbox pattern) |

**NOTE — Flagged uncertainty:** Llama-4 specifics referenced in research were UNCONFIRMED as
of 2026-06-27 (model availability/API stability); the candidates above use the confirmed Llama
Prompt Guard 2 lineage. Benchmark (OQ-C15-4) must re-verify all candidates against:
- Inference latency within the 200MB per-query budget
- Instruction-following on security-domain prompts
- Quantized-model accuracy retention
- wasmtime wasi-nn SIMD/throughput vs ort native (C7 D-C7-2 notes WASM performance tax)

**Conforms:** PAT-ADS-05 (Pluggable-AI-Opaque-ModelBackend); P-ADS-07 (AI-Opaque); INV-ADS-06.

---

### D-C15-8 — Lifecycle: Three Coupled State Machines + Action-Approval Sub-Machine

**DECIDED 2026-06-27 (human).**

**Three coupled state machines:**

```
Observation lifecycle:
  New → Active → Resolved(terminal|reopenable) ↔ Reopened
  Extended: Dismissed (FP), Suppressed (maintenance/known), Stale (auto-closed unactioned)

Recommendation lifecycle:
  Proposed → Approved | Rejected → Implemented | Superseded | Expired

Action lifecycle:
  Pending → Scheduled → Running → Succeeded | Failed → RolledBack
  Plus: AwaitingApproval → Approved | Denied | SlaExpired → Escalated (approval sub-machine)
```

**Terminal-vs-reopenable resolution (Xpanse model — the single most important lifecycle
lesson):** [Q5-1]
- **Terminal** (governance/policy decision — does NOT reopen even if condition recurs):
  `resolved_risk_accepted`, `resolved_no_risk`, `resolved_contested_asset`
- **Reopenable** (technical disappearance — reopens if condition recurs):
  `resolved_no_longer_observed`, `resolved_remediated`, `resolved_remediated_auto`

Risk-accepted ≠ no-longer-observed. These MUST NOT be conflated. A finding closed as
`resolved_risk_accepted` that later recurs does NOT automatically reopen (the governance
decision still stands); a finding closed as `resolved_no_longer_observed` that recurs DOES
reopen.

**Ack-timeout retrigger (PagerDuty pattern) [Q5-13]:** Ack only halts escalation; if ack
times out without resolution, the Observation retrigers. The `ack-timeout` is configurable
per Recommendation priority tier.

**Task-completion-aware closure (Sentinel pattern) [Q5-2]:** A parent Observation/case is
AUDITED for task completion before closure. Closure while tasks are incomplete is flagged in
the immutable audit trail.

**Dedup / correlation — the "1000 alerts → 10-20 AROs" goal:**
- `dedup_fingerprint`: fnv64a hash over sorted canonical entity+detection-type label set
  (Alertmanager pattern) [Q5-12]
- `correlation_id`: groups Observations from the same sequence/pattern rule into one parent
  (Panther LookbackWindowMinutes-style correlation windows) [Q5-9]
- `collapsed_observation_count`: N raw detections collapsed into one Observation record;
  displayed in the UI so analysts know how many raw signals were deduped
- Entity-centric grouping: multiple Observations about the same entity (same asset/user/IP)
  with the same detection type within the correlation window → one Recommendation [Q5-14]

**Idempotency at all tiers:** `dedup_fingerprint` doubles as idempotency mechanism at the
Observation tier (re-processing the same detection event is a no-op); `idempotency_key` is
mandatory at the Action tier (C2 offline-queue reconnect safety).

---

## Invariants (PIV-C15-*)

| ID | Invariant | Enforced by |
|----|-----------|------------|
| **PIV-C15-1** | PrismQL `RECOMMEND` is a read-only data projection with ZERO execution/mutation capability. | Perimeter compile-fail test (same E0432 pattern as `tests/external/perimeter-violation/`). The test must fail to compile if RECOMMEND result is wired to an action executor. |
| **PIV-C15-2** | Every Action carries a mandatory idempotency key. No Action is constructed without one. | Rust type system: `idempotency_key: IdempotencyKey` is a non-optional field on the Action struct; no `Default` impl; `#[non_exhaustive]` on Action. |
| **PIV-C15-3** | All Actions in v1 are HITL-gated. `autonomy_level = Autonomous` on any Action is a compile-time or startup invariant violation. | Startup assertion + RBAC gate in `prism-orchestration`; `autonomy_level` field validated at action-submission boundary. |
| **PIV-C15-4** | System designers — NOT the agent — set the autonomy gates. The agent cannot set, elevate, or bypass its own `autonomy_level`. | `autonomy_level` and `gate_mode` on Recommendation are set by policy configuration (C9), not by the agent layer. The agent populates `proposed_action` + risk metadata; the orchestration layer applies the configured gate. |
| **PIV-C15-5** | AI-generated Recommendations surfaced to analysts MUST carry verified citations. No claim without a citation in the `citations[]` block; any unverified citation is flagged; unsupported claims are removed by Output Hardener before display. | Output Hardener component in S3 agent runtime (ADR-PROP-s3-agent-runtime.md); `unsupported_claim_flag` set before any surface call. |
| **PIV-C15-6** | Write-action credentials NEVER transit AI context. Resolved reference-based at the execution tier in `prism-orchestration`. | AD-017 extension to write path; credential reference type (not value) in action parameters; satellite-local resolution at execution time. |
| **PIV-C15-7** | OT/safety-segment Actions are HITL-mandatory, regardless of autonomy_level configuration. | `asset_tier = OT_SAFETY` on Recommendation risk metadata triggers HITL gate override in orchestration layer. |

---

## Open Questions (OQ-C15-*)

| ID | Question | Priority | Resolution path |
|----|----------|----------|----------------|
| **OQ-C15-1** | Autonomy-ladder activation criteria: which action classes ever qualify for auto-with-approval or autonomous tier (post-v1)? Which classes are HITL-permanent? | P1 (post-v1, human risk-acceptance gate required) | Evidence-based promotion per Dash0 floors; explicit human sign-off per action class; OT/safety = HITL-permanent |
| **OQ-C15-2** | Conformal-prediction implementation approach for v1: constrained generation vs token attribution vs post-hoc NLI-based faithfulness check? | P1 | Research at story-spec time (E-ARO-MODEL-001); OUTPUT requirement (all claims cited) is not deferred |
| **OQ-C15-3** | `prism-orchestration` playbook DSL design: DAG-of-tasks DSL, YAML-authored vs PrismQL-query-referencing vs visual builder? | P2 | Story-spec time; Tines/XSOAR/Splunk SOAR patterns are the reference |
| **OQ-C15-4** | On-prem model final picks: benchmark Qwen3/Mistral/Phi-4-mini/Ministral/Llama-Prompt-Guard-2/Mistral-Moderation against latency + quality + memory budget + WASM-vs-ort overhead. | P1 | Pre-implementation benchmark per E-SOAR-ACTIONS-001 wave |
| **OQ-C15-5** | Faithfulness-check mechanism detail: which of constrained-gen / token-attribution / post-hoc-statement-vs-citation is most viable for the Output Hardener? | P1 | Research at E-ARO-MODEL-001 story-spec time |

---

## Risk Mitigations

**R: Prompt injection via RECOMMEND projection.** Mitigation: the perimeter compile-fail test
(PIV-C15-1) ensures RECOMMEND is structurally disconnected from action execution at the type
level. Additionally, the Output Hardener validates all AI-generated content before surfacing.
The RECOMMEND projection produces a typed data record — it cannot be coerced into an action call
at the type system level.

**R: Double-action on C2 reconnect.** Mitigation: mandatory `idempotency_key` on every Action
(PIV-C15-2); dedup on the satellite's store-and-forward queue matches the key before executing.

**R: AI-generated Recommendation with hallucinated citations.** Mitigation: mandatory post-hoc
faithfulness check by Output Hardener before any Recommendation surfaces to the analyst
(PIV-C15-5, PAT-ADS-08). The ~57% unfaithful-citation base rate in RAG systems makes this a
correctness invariant, not a quality goal.

**R: Agent elevates its own autonomy.** Mitigation: structural — `autonomy_level` is set by
policy configuration (PIV-C15-4), not by the agent. The agent has no API to modify it.

---

## Proposed Epics

- **E-SOAR-ACTIONS-001** — SOAR/ARO model: `prism-orchestration` layer, connectors-as-actions,
  HITL approval gates, case management, audit subsystem, rollback, blast-radius control
- **E-ARO-MODEL-001** — ARO data model: Rust typed entity structs, lifecycle state machines,
  dedup/correlation, idempotency infrastructure, `RECOMMEND` projection + perimeter test,
  AI-recommendation provenance + calibrated confidence + conformal + faithfulness blocks

Cross-links: C7 (ModelBackend), C8 (detection recipes + RECOMMEND), C10 GAP-Q2
(evidence-package), C12 (Context/Knowledge + entity linkage + faithful citations),
C18 (approver roles/RBAC), C2 (edge execution + offline queue), AD-017, ADS.

---

## ADS Conformance Checklist

```
CONFORMANCE CHECKLIST — ADR-PROP-soar-actions-aro (C15) — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [x] Does every user-interaction path in this feature terminate at Central?
      YES — ARO triage, Recommendation review, Action approval all surface at Central console.
  [x] If the feature involves a satellite, is the satellite strictly headless?
      YES — Action execution at satellite is data-plane only; no satellite login surface.

P-ADS-02: Operator-Zero-Access-At-Rest
  [x] Is every derived result persisted at Central encrypted under tenant-held CMEK (SS-26)?
      YES — ARO records persisted at Central follow PAT-ADS-02 (Tenant-Keyed-Central-Cache);
      OrgSlug-partitioned, SS-26 per-tenant DEK encrypted (P-ADS-04).
  [x] Does the operator have zero read access to persisted data at rest?
      YES — same CMEK guarantee as all Central-cached derived results.

P-ADS-03: Derived-Results-Only-At-Central
  [x] Does the feature transit only DERIVED results from edge to Central?
      YES — Observations are OCSF-normalized (derived) findings; raw sensor data never
      transits to Central. Action payloads carry action parameters + idempotency key +
      credential REFERENCE (not raw credential value).
  [x] If opt-in path where identifiers transit, is consent governance explicitly closed?
      N/A — no new opt-in identifier-transit path introduced.

P-ADS-04: Tenant-Keyed-Central-Persistence
  [x] Does the feature use RocksDB (hot) or Iceberg (cold) — NOT PostgreSQL — for Central
      query result cache?
      YES — ARO records in the Central cache use RocksDB (hot) per the storage taxonomy
      (ADR-PROP-storage-engine-taxonomy.md). NOT PostgreSQL.
  [x] If forensic replay is discussed, does it distinguish output caching from input-level
      Iceberg data-snapshots?
      YES — the ARO store caches derived findings/recommendations/actions (outputs). No
      claim of input-data replay is made; OQ-C8-DATASNAPSHOT governs input-level replay.

P-ADS-06: Per-Tenant-Isolation
  [x] Is all new state per-tenant partitioned?
      YES — `tenant_id: OrgSlug` on every AROBase record; graph entity linkage rides C12
      per-tenant partitioned store (PIV-C12-5).
  [x] Is there any code path that produces cross-tenant joins?
      NO — the dedup/correlation operates within a single tenant's data space.

P-ADS-07: AI-Opaque
  [x] Do all AI/ML components receive feature vectors or masked data only?
      YES — on-prem model backends receive structured evidence context, not raw credentials
      or PII-containing raw OCSF records. C16 masking applies at the ModelBackend boundary
      (PAT-ADS-05).
  [x] Are credentials resolved OUTSIDE the AI reasoning loop?
      YES — PIV-C15-6: write-creds resolved reference-based at the execution tier in
      prism-orchestration; the agent never holds write credential values (AD-017 extended
      to write path).

P-ADS-08: OCSF-Normalize-At-Boundary
  [x] Does every new data source normalize to OCSF at the adapter boundary?
      YES — C15 consumes OCSF-normalized Observations from the existing OCSF pipeline;
      no new raw-schema connectors introduced.
  [x] If source has OCSF version skew, is the version axis declared in the capability descriptor?
      N/A — C15 consumes existing OCSF layer; version governance is C4/C5 responsibility.

P-ADS-09: Config-DB-Authoritative
  [x] Does the feature have any config-authoring path at the satellite or edge?
      NO — autonomy-level configuration, RBAC gate settings, and action class policies are
      authored at Central DB/UI (C9 authority). PIV-C15-4: agent cannot modify these.

P-ADS-10: Idempotent-Gated-Actions
  [x] Do all write/action paths carry idempotency keys?
      YES — PIV-C15-2: mandatory `idempotency_key` on every Action struct; no Action
      constructed without one.
  [x] Is the autonomy gate system-configured, not agent-configured?
      YES — PIV-C15-4: `autonomy_level` and `gate_mode` set by policy config (C9/C18);
      agent has no API to modify. CISA invariant enforced architecturally.

INV-ADS check (all eight):
  [x] INV-ADS-01: No raw sensor data at Central
      ARO records carry OCSF-normalized derived findings; raw sensor data never at Central.
  [x] INV-ADS-02: Operator zero-access at rest
      All Central-persisted ARO records encrypted under tenant-held DEK (SS-26).
  [x] INV-ADS-03: Per-tenant isolation enforced
      `tenant_id: OrgSlug` on every record; dedup/correlation within-tenant only.
  [x] INV-ADS-04: Config authored only at Central
      Autonomy gates and action-class policies authored at Central DB/UI; satellites
      receive them as signed config bundles (C9 PAT-ADS-03).
  [x] INV-ADS-05: Actions gated and idempotent
      All v1 Actions HITL-gated (D-C15-2); mandatory idempotency key (PIV-C15-2).
  [x] INV-ADS-06: AI-opaque
      On-prem models receive structured evidence context; write-creds resolved at
      execution tier; Output Hardener validates before surfacing (PIV-C15-5/6).
  [x] INV-ADS-07: OCSF normalization at all boundaries
      C15 consumes the existing OCSF layer; no new normalization boundary introduced.
  [x] INV-ADS-08: Air-gap deployment is valid reference profile
      `prism-orchestration` layer is embedded single-binary; C7 ModelBackend (candle/ort/
      wasmtime/tract) all air-gap-safe with pre-staged models; satellite action execution
      works air-gap via C2 store-and-forward + idempotency keys.

RESULT: ALL CONFORMANCE ITEMS PASS.
No non-conformances identified. C15 is ADS-conforming at capture.
```

---

*End of ADR-PROP — Actions in PrismQL / SOAR / ARO Model (C15) — 2026-06-27*
