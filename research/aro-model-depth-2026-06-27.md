# Research: The ARO Model — Action · Recommendation · Observation (C15 Depth Pass)

> **SIDE-ANALYSIS item:** C15 — ARO model as the spine of actions in PrismQL / SOAR. **First-class topic**, deeper than the prior C15 sweep (`research/prismql-actions-soar-onprem-models-2026-06-27.md`, which touched ARO lightly).
> **Mode:** CAPTURE / research only (`do_not_execute`). No live spec/BC/ADR/STATE.md/SESSION-HANDOFF.md modified. No git.
> **Date:** 2026-06-27
> **Type:** general (decision-theory + security-data-model + AI-governance research; feeds C15 design)
> **Author:** research-agent
> **Status:** complete

---

## Scope & Context

Prism today is a **read-only federated security query engine**: PrismQL (Chumsky grammar + DataFusion planner) issues queries, sensor adapters fan out to vendor APIs, results are normalized to OCSF/protobuf and returned to an LLM agent. The user wants a **rigorous ARO (Action · Recommendation · Observation) model** as the spine of C15. This pass goes deep on ARO itself, building on two existing inputs rather than restating them:

1. **Prior C15 research** (`prismql-actions-soar-onprem-models-2026-06-27.md`) — established the three-tier Observation→Recommendation→Action state-machine sketch, the recommend-only-vs-auto-act fork, Tines' 5 risk axes (reversibility, asset importance, confidence, novelty, compliance), and the "don't embed mutations in PrismQL grammar" lean. **This pass takes those as given and deepens the conceptual + data-model + governance grounding.**
2. **Aletheon spike `aros` table** (`/Users/jmagady/Dev/aletheon_2/spike/init-db.sql`) — ONE concrete reference data model: `aro_type` (action|recommendation|observation), `priority` (1-100), `status` (open/acknowledged/resolved/dismissed), dual-audience text (`plain_english` + `technical_details` + `suggested_response`), `source_event_ids[]`, `affected_asset_ids[]`, AI provenance (`confidence`, `model_version`, `reasoning`), ack/resolve workflow. **Treated as one input, NOT the final answer.** Aletheon is OT/asset-centric (Apache AGE graph + pgvector over Claroty/Armis/syslog); Prism is multi-source/multi-entity federated security.

Relevant Prism anchors (from prior C15 + project memory):
- Sensor APIs **already have write capabilities**, behind a robust feature-flag system (`project_feature_flags.md`).
- **C7** = pluggable AI-opaque `ModelBackend` (candle + ort + wasmtime + tract).
- **C10 GAP-Q2** = auditable agent **evidence-package** + self-QA; Query Workers analog produces findings + recommendations and **does NOT auto-act**.
- **C12** = entity model + **mandatory citations**. **C16** = masking. **C18** = RBAC. **AD-017** = AI-opaque credentials. **S3** = server-hosted embedded agent. **C2** = satellite/edge execution.

All non-obvious claims are cited inline. Sources are web-verified via 5 `perplexity_research` deep-research passes (`reasoning_effort=high`); citation groups are listed per-section and consolidated in the Sources section. Date-stamp: findings current **as of June 2026**.

---

## Q1 — Conceptual Lineage of Observation → Recommendation → Action

The Observation→Recommendation→Action progression is **not a novel coinage**; it is the security-platform instantiation of a family of decision/control loops that have been formalized across military strategy, autonomic computing, human-factors engineering, robotics, and control theory. Each contributes a distinct rigor to the ARO taxonomy. [Q1-1 … Q1-19]

### OODA loop (Boyd: Observe–Orient–Decide–Act)

Boyd's loop (1970s–80s, formalized in *Patterns of Conflict* and *The Essence of Winning and Losing*) is the best-known adversarial decision cycle. The popular four-stage rendering understates Boyd's actual diagram, in which **Orientation is central** — it both shapes what is Observed and feeds Decide/Act, mediated by "genetic heritage, cultural tradition, previous experiences, unfolding circumstances, and the processes of analysis and synthesis." [Q1-9][Q1-10][Q1-17] The winning insight is tempo: the side that cycles **faster and more appropriately** gets "inside" the adversary's loop. Cybersecurity practitioners have adopted OODA directly for incident response — Observe (telemetry ingest) → Orient (correlate + enrich + apply analyst expertise) → Decide (prioritize, choose containment) → Act (block/quarantine/reconfigure). [Q1-3][Q1-15]

**Contribution to ARO:** OODA's **Orient** is the crucial intermediate stage that ARO's *Recommendation* tier maps onto — Recommendation is not a mechanical rule-fire but an **interpretive, model-mediated, prediction-bearing** step. OODA also insists Observation is **already shaped by prior orientation** (what you choose to collect/attend to is not neutral), and that **Action is never terminal** — it produces new Observations, closing the loop. This argues that ARO must be modeled as a *cycle*, not a one-shot pipeline, and that Recommendation should carry forecasts of action effects, not just classifications. [Q1-9][Q1-10][Q1-17]

### MAPE-K (IBM autonomic computing: Monitor–Analyze–Plan–Execute over Knowledge)

Kephart & Chess's *Vision of Autonomic Computing* (2003) and IBM's architectural blueprint define the MAPE-K control loop as the core of self-managing systems: **Monitor** (sense + filter), **Analyze** (detect patterns/anomalies, predict), **Plan** (translate analysis into concrete change plans respecting policy), **Execute** (apply via effectors), all over a **shared Knowledge base** (system models, policies, history, learned models). [Q1-4][Q1-11][Q1-14] Modern work applies MAPE-K to smart-factory workflow adaptation and agentic-AI microservice remediation, with AI agents typically slotting into the Analyze (and increasingly Plan) phases. [Q1-5][Q1-13]

**Contribution to ARO:** MAPE-K makes two contributions the other loops do not. **First, it splits the intermediate tier into Analyze vs Plan** — i.e., *situational understanding* (an Observation-comprehension function) is distinct from *generating a recommended change* (the Recommendation function). This suggests ARO's Observation tier should itself carry analytic/comprehension content, and Recommendation is specifically the *plan-generation* output. **Second, the explicit shared Knowledge base** is exactly Prism's C12 entity model + threat intel + asset inventory + policy: it is the semantic glue that makes Observations interpretable, Recommendations consistent, and Actions auditable. MAPE-K's Knowledge base directly motivates linking every ARO record to a shared knowledge layer (entities, evidence, policies) rather than treating each ARO as standalone. [Q1-11][Q1-14]

### Endsley's three-level Situation Awareness (Perception → Comprehension → Projection)

Endsley's canonical SA model (1995) defines SA as: **Level 1 Perception** of elements in the environment; **Level 2 Comprehension** of their meaning relative to goals; **Level 3 Projection** of their near-future status. [Q1-6][Q1-18] SA is a *prerequisite* to decision-making, not the decision itself, and is bounded by attention and working memory; mental models drive comprehension and projection. SOC research increasingly grounds analyst-cognition studies in Endsley's three levels. [Q1-7]

**Contribution to ARO:** Endsley formalizes the most important subtlety for the Observation tier: **an Observation is not raw perception.** A rigorous ARO Observation should explicitly distinguish (1) the *perceived* fact ("host X had 412 failed logins"), (2) the *comprehended* meaning ("consistent with credential-stuffing against a privileged account"), and (3) the *projection* ("likely lateral movement within ~30 min if unmitigated"). The projection layer is the natural *bridge* from Observation to Recommendation — it is simultaneously part of understanding the situation and the basis for what to recommend. [Q1-6][Q1-18] This is a sharper design directive than aletheon's flat `plain_english`/`technical_details` split.

### Sense–Plan–Act / Sense–Decide–Act (robotics)

Classical robotics SPA (dominant through ~1985) is a three-stage cyclic architecture: **Sense** (sensors) → **Plan** (build world model, plan next move) → **Act** (execute). [Q1-8] Sense–decide–act variants emphasize reactive decision rules over rich planning. The deliberation-vs-reaction tradeoff (rich plans but slow vs fast but error-prone) is explicit.

**Contribution to ARO:** SPA reinforces the three-tier separation and the **world-model requirement** in the intermediate tier (again pointing at C12/Knowledge). Its most actionable contribution is the **deliberation-vs-reaction tradeoff** mapped to ARO autonomy: simple, high-confidence, reversible Actions can run via a "sense-decide-act" reactive path (low deliberation), while novel/high-impact Actions require the deliberative "sense-plan-act" path (Recommendation + human gate). This tradeoff is the structural justification for *tiered autonomy* (Q4). [Q1-8]

### Cybernetic feedback / control-theory framing

Classical feedback control: a *plant*, *sensors* measuring output, a *controller* computing inputs against a reference/setpoint, *actuators* applying inputs — closed-loop so output adjusts dynamically. The autonomic-computing literature explicitly grounds MAPE-K in control theory. [Q1-4][Q1-11][Q1-14] (Canonical cybernetics texts — Wiener — did not surface in the web sweep; this framing is corroborated via the autonomic-computing sources rather than primary cybernetics. Flagged.)

**Contribution to ARO:** Control theory contributes the **analytic vocabulary for the loop**: stability, robustness, latency, gain, and **disturbance rejection** (the adversary is a disturbance). It frames Observation = measurement, Recommendation = control-law computation, Action = actuation, and adds the discipline that the *setpoint* (desired security state) must be explicit. For Prism this argues that ARO Recommendations should reference a policy/setpoint ("desired state: privileged accounts not accessible from untrusted geos") so that Action effectiveness is measurable against it. [Q1-11][Q1-14]

### Synthesis — what a rigorous ARO taxonomy inherits from each loop

| Loop | Maps to ARO as | Distinct rigor it adds |
|---|---|---|
| **OODA** | Observe→Orient→Decide→Act ≈ Observation→Recommendation→Action, **looped** | Recommendation = model-mediated *prediction* of effects; Observation is selectively shaped; Action is not terminal; **tempo matters** |
| **MAPE-K** | Monitor→(Analyze+Plan)→Execute over **Knowledge** | Split comprehension (Analyze) from recommendation (Plan); mandatory **shared Knowledge base** (≈ C12 entity model + policy) |
| **Endsley SA** | Observation = Perception→Comprehension→**Projection** | Observation is a 3-layer cognitive product; Projection bridges to Recommendation |
| **Sense-Plan-Act** | Sense→Plan→Act, world-model in Plan | Deliberation-vs-reaction tradeoff → justifies **tiered autonomy** |
| **Control theory** | measurement→control-law→actuation, closed-loop | Setpoint/policy must be explicit; adversary = disturbance; loop stability/latency vocabulary |

**Net:** ARO is well-founded — it is the security instantiation of OODA/MAPE-K/SA/SPA. The strongest design directives are: (a) model ARO as a **closed loop** (Action feeds new Observations); (b) treat **Observation as a 3-layer product** (perceive/comprehend/project), not a flat finding; (c) make the **shared Knowledge base (C12)** first-class, not per-ARO; (d) use the **deliberation/reaction tradeoff** to justify tiered autonomy. [Q1-6][Q1-8][Q1-10][Q1-11][Q1-14][Q1-18]

---

## Q2 — The Three Tiers as a First-Class Data Model in Security Practice

A consistent three-tier separation already pervades mature platforms, even where the *vocabulary* differs. The deep sweep found **no platform that names the model "ARO"**, but the underlying detection→recommendation→action separation is near-universal, and each tier carries its own fields, lifecycle, and state machine. [Q2-1 … Q2-16]

### Tier 1 — Observation (signal / finding / alert / story)

The dominant pattern is to separate **raw detections** from **case-worthy alerts**:

- **Panther** explicitly distinguishes **signals** (every rule match) from **alerts** (escalated cases). Toggling `Create Alert` off strips alert-only fields — **Severity, Runbook, Deduplication Period** — proving these belong to the recommendation/case tier, not the raw-detection tier. Signals carry `p_rule_id`, tags, timestamps, correlation IDs. [Q2-5]
- **Microsoft Sentinel** `SecurityAlert` schema is the richest documented Observation model. Detection fields: `AlertName`, `AlertType`, `AlertSeverity`, `CompromisedEntity`, `Entities`, `ConfidenceLevel`, **`ConfidenceScore` (0.0–1.0 real-valued)**, `Tactics`, `Techniques` (MITRE ATT&CK), `StartTime`/`EndTime`/`TimeGenerated`, `ProductName`/`ProviderName`. Crucially it embeds a **`RemediationSteps`** field (the Recommendation tier folded into the finding) and a `Status` field (the lifecycle). `IsIncident` is deprecated/always-false — incidents are modeled separately. [Q2-6]
- **Google SecOps/Chronicle** uses a Unified Data Model (UDM) — `metadata.*`, `target.*`, `actor.*`, nested `user`/`process`/`file` — analogous to OCSF; findings are UDM events flagged by rules. [Q2-7]
- **OCSF** defines specialized finding classes: **Security Finding, Vulnerability Finding, Compliance Finding, Detection Finding**, with standardized fields (`metadata.product.*`, `class_name`, `src_endpoint.ip`, severity, `status_id`). AWS Security Lake maps Security Hub findings into these classes. This is the **vendor-neutral Observation schema** Prism's OCSF normalization should align to. [Q2-8][Q2-16(Q5)]
- **Exabeam** UEBA: each session carries a **`Score`** (aggregate risk) + tags; high scores escalate to stories/alerts (the Observation tier). **Securonix** frames insider risk findings around insider/asset/impact. UEBA shows risk-score → finding → recommendation flow. [Q2-9][Q2-10][Q2-16]

**Observation tier fields (synthesized):** id, type, title, severity, **confidence (calibrated, real-valued)**, affected entities/assets, MITRE tactics/techniques, source rule/detector id, timestamps (start/end/generated), evidence/correlation IDs, status.

### Tier 2 — Recommendation (proposed response / remediation / runbook / task)

The Recommendation tier appears in three forms across platforms — and this is the **least standardized** tier:

1. **Embedded as a field on the finding** — Sentinel `RemediationSteps` (a list of action items); Panther `Runbook` reference on alerts. [Q2-5][Q2-6]
2. **As separate task/workbook objects** — Splunk SOAR **workbooks** define manual tasks/checklists (name, description, owner, due date, status); XSOAR War Room **ML insights suggest next steps**. [Q2-1][Q2-2]
3. **As AI-generated structured plans** — AI-SOC products (Hunters narratives, Dropzone AI, Prophet, "Query Workers" style) produce **findings + recommendations with rationales/evidence but intentionally do NOT auto-act**, deferring execution to humans/SOAR. Their internal schemas are largely **proprietary/undocumented** (flagged: this is the weakest-sourced part of Q2). [Q2 §AI-SOC]
4. **Vulnerability/ASM advisories ARE a recommendation tier** — a CVE advisory is fundamentally a recommendation object (remediation guidance, affected assets, severity, patch availability) coupled to a Vulnerability-Finding observation. [Q2 §ASM]

**Recommendation tier fields (synthesized):** id, links to source Observation(s), proposed action(s), rationale/explanation, target entities/assets, priority, **risk metadata** (reversibility, blast radius, confidence, novelty, asset tier, compliance flags — from prior C15 Tines axes), owner, due/SLA, status (proposed/approved/rejected/implemented/superseded).

### Tier 3 — Action (executed change / connector task / playbook step)

Actions are **most standardized in SOAR**:
- **XSOAR**: War Room command executions + playbook task results, gated by **manual tasks**. [Q2-1]
- **Splunk SOAR**: actions run **against containers** using artifacts as input; each action has params + results; case **status types are grouped New/Open/Closed**; closure can require **mandatory tags**; severity drives SLA (low = 1440-min default resolution). [Q2-2]
- **Tines**: 8 action types; workflow = graph of actions receiving/emitting JSON events; manual actions = HITL recommendation checkpoints. [Q2-3]
- **Torq/Radiant**: agentic AI SOC, action execution via connectors; **data models not publicly documented** (flagged). [Q2-4]

**Action tier fields (synthesized):** id, links to approved Recommendation + Observation(s) (provenance chain), action type, target, parameters, **idempotency key**, executor identity, approver identity, timestamps (requested/approved/executed), status (pending/scheduled/running/succeeded/failed/rolled-back), result, **rollback handle**.

### Cross-cutting state-machine pattern

All three tiers are connected by an overarching state machine: a new Observation may spawn Recommendations; Recommendations may spawn Actions (gated); Action completion (or all-alerts-resolved, PagerDuty-style) transitions the parent case to resolved. Closure is evaluated not just on disposition but on **task completion** (Sentinel audits "incidents closed while tasks incomplete"). [Q2-2(Q5)] **Findings + recommendations without auto-act is the documented conservative default for AI-SOC products** — exactly Prism's C10 GAP-Q2 stance. [Q2 §AI-SOC]

---

## Q3 — AI-Generated Recommendations: Provenance, Confidence, Explainability

For an AI-generated Recommendation to be **trustworthy + auditable**, it must be treated as a structured decision artifact carrying provenance, calibrated confidence, faithful explanation, linked evidence, and immutable audit metadata. No single standard prescribes the field schema, but NIST + W3C + DARPA + the calibration/conformal literature + RAG-faithfulness research converge on a concrete field set. [Q3-1 … Q3-18] This directly extends C10 GAP-Q2 (evidence-package) and C12 (mandatory citations).

### Decision provenance (W3C PROV + model cards + generative context)

- **W3C PROV-DM** models provenance as **Entities, Activities, Agents** + derivations + bundles. Mapped to an AI Recommendation: the recommendation is an *entity*; model inference is an *activity*; the model instance, the AI system, and the human reviewer are *agents*; each log/threat-intel input is an *entity* with a derivation edge. This produces a **traceable lineage graph** — the natural structure for Prism's evidence package. [Q3-4][Q3-16]
- **Model cards** (Mitchell et al.) document model-level provenance: name, **version**, training data summary, eval metrics, **known limitations/biases**, intended use, and calibration info. Every Recommendation should reference the **model card + version** that produced it. [Q3-5]
- **Generative-AI provenance** (NIST GenAI Profile, AI 600-1): for LLM-produced recommendations, provenance must record the **prompt, system-prompt version, retrieved-context/RAG document set, and generation config** (temperature/top-k), plus any **non-deterministic seeds**, to enable reproducibility. [Q3-12]

### Confidence calibration (why raw model confidence is untrustworthy)

- Raw neural-net confidence is **typically miscalibrated** (overconfident). A model emitting 0.95 may be right far less than 95% of the time. [Q3-6] Reliability diagrams + **Expected Calibration Error (ECE)** quantify this: `ECE = Σ (|B_m|/n)·|acc(B_m) − conf(B_m)|`. [Q3-6]
- **Conformal prediction** gives distribution-free, model-agnostic uncertainty *sets* with a coverage guarantee (e.g., 90%) under exchangeability — e.g. "the true attack type is in {credential-stuffing, password-spray} with 95% coverage." Narrow set = high certainty; broad set = low. This is a far better automation gate than a point confidence. [Q3-7]
- **Implication for ARO:** the Recommendation must carry **raw confidence, calibrated confidence + method (temperature/Platt/isotonic), ECE + last-calibration date,** and optionally a **conformal prediction set + coverage level**. Automation thresholds (Q4) should key off *calibrated* confidence / conformal-set width, never raw scores. [Q3-6][Q3-7]

### Explainability (NIST + DARPA)

- **NISTIR 8312 — four principles of explainable AI:** (1) **Explanation** (provide evidence/reasons), (2) **Meaning** (understandable to the target audience — analyst vs operator vs auditor; this validates aletheon's `plain_english`/`technical_details` dual-audience split), (3) **Explanation Accuracy** (the explanation must faithfully reflect actual model behavior — *not* a plausible post-hoc story), (4) **Knowledge Limits** (the system must say when it is out-of-distribution / uncertain). [Q3-2]
- **DARPA XAI** adds interactive **explanation dialogue** — the analyst can ask "why this?", request alternatives, explore hypotheticals — backed by the provenance record. [Q3-3]
- **NIST AI RMF (AI 100-1)** + bias guidance (SP 1270) treat explainability/interpretability + transparency + accountability as core trustworthiness characteristics. [Q3-1][Q3-11]

### Evidence linkage + citation faithfulness (ties directly to C12 mandatory citations)

- **Critical finding:** in RAG systems, **up to 57% of citations can be *unfaithful*** — the cited document does not actually support the statement; the model post-rationalizes citations after generating the answer. **Correctness ≠ faithfulness.** [Q3-8] This is the single most important caveat for C12's mandatory-citation requirement: a citation field is worthless unless faithfulness is *enforced or measured*.
- Mitigations: constrain generation to retrieved content, token-level attribution, post-hoc statement-vs-citation checking, and recording **per-citation faithfulness flags/scores**. [Q3-8]
- **Forensic chain-of-custody** (NIST SP 800-61r3, SP 800-86, SP 800-92; AWS logging guidance): AI conclusions relying on digital evidence must fit existing evidence-admissibility practice — evidence-package IDs, collection times, hashes, transformation logs. [Q3-13][Q3-14][Q3-17][Q3-18]

### Auditability (who / what / when / where / why + reproducibility)

Immutable, tamper-evident audit capturing **who** (model id+version, human approver, automated responder), **what** (recommendation type, action taken), **when** (generation/review/execution timestamps), **where** (affected asset/segment), **why** (rationale + evidence + rules/policy invoked). **Reproducibility**: given same inputs+model+prompt+config+seed, the decision should reproduce. [Q3-1][Q3-10][Q3-13][Q3-14][Q3-16][Q3-17][Q3-18]

### Mandatory AI-Recommendation field set (Q3 synthesis)

| Group | Fields | Source |
|---|---|---|
| **Provenance** | model name+**version** (→ model card), prompt + system-prompt version, RAG/retrieved-doc set, generation config, seed, generating-agent id, human-reviewer id | [Q3-4][Q3-5][Q3-12][Q3-16] |
| **Confidence** | raw confidence, **calibrated** confidence + method, ECE + calibration date, optional conformal set + coverage | [Q3-6][Q3-7] |
| **Explanation** | human-readable rationale (audience-tagged), key contributing features/evidence, rules/playbooks invoked + versions, **explanation-faithfulness indicator**, **knowledge-limits / OOD flag** | [Q3-2][Q3-3][Q3-8] |
| **Evidence/citation** | evidence-package id, per-citation list mapped to evidence items, **per-citation faithfulness flag/score**, "unsupported-claim" flag, chain-of-custody (hashes, collection times, transforms) | [Q3-8][Q3-13][Q3-18] |
| **Audit** | who/what/when/where/why, immutable+hashed, reproducibility inputs | [Q3-1][Q3-14][Q3-16] |

---

## Q4 — Human-in-the-Loop Gating + Tier Promotion + Autonomy Levels

How does an Observation get promoted to a Recommendation, and a Recommendation to an Action? Through **graded autonomy + risk-based gating**. There is **no SAE J3016-equivalent standard for "SOC autonomy"** (a genuine gap), but multiple convergent autonomy scales + explicit gating criteria exist. [Q4-1 … Q4-16]

### Autonomy-level taxonomies (SAE-style, applied to security)

- **SentinelOne Autonomous SOC Maturity Model (L0–L4):** L0 Manual → L1 Rules-Based (SOAR enrichment/ticketing) → L2 AI-Assisted (ML triage/prioritization, human-central) → **L3 Partial Autonomy** (agentic AI performs *lower-risk* response actions, humans supervise higher-impact) → L4 High Autonomy (aspirational, AGI-dependent, not imminent). **Industry today is L1–L3.** [Q4-1]
- **WatchGuard Security Operations Maturity Model (L0 Minimal → L4 Optimized):** couples autonomy to process+staffing; even "Optimized" assumes a staffed SOC — autonomy augments, not replaces. [Q4-3]
- **Dash0 six levels of agentic engineering** (building on **Cloud Security Alliance "Autonomy Levels for Agentic AI"** and ASDLC.io L1–L5): assisted → human-reviewed → evidence-approved → selective auto-merge → mostly autonomous → "dark factory." **Key transferable idea: promotion is a *measurement problem* per surface, never an org-wide declaration; high-risk surfaces may NEVER qualify for full autonomy.** [Q4-15]
- **NVIDIA Agentic Autonomy framework** ties autonomy levels to required security controls. [Q4-16]
- **Gartner (via Torq):** AI will **augment, not replace** analysts for the foreseeable future. [Q4-4]

The **SAE J3016 template** (levels defined by division of human/machine labor + operating conditions + fallback expectations) is the conceptual reference, but no security-specific standard is ratified — organizations assemble per-workflow. [Q4-15]

### The spectrum: advisory → suggested → auto-with-approval → autonomous

This maps **cleanly onto ARO**:
- **Observation tier** → automation is uncontroversial (collect/detect/correlate autonomously; HOOTL is fine). [Q4-2][Q4-3]
- **Recommendation tier** → AI *generates* recommendations autonomously, but humans validate (suggested/advisory). This is the C10 GAP-Q2 / Query-Workers zone. [Q4-1][Q4-7]
- **Action tier** → the contentious tier; gated by **reversibility, asset importance, confidence, novelty, compliance** (the 5 Tines axes from prior C15). [Q4-6]

### HITL / HOTL / HOOTL definitions + when each applies

- **HITL (human-in-the-loop):** workflow **pauses at a checkpoint** and cannot proceed without human approval. Required for **irreversible / high-impact / safety-critical / compliance-sensitive / novel** actions. [Q4-6][Q4-8]
- **HOTL (human-on-the-loop):** system acts autonomously, humans **monitor + can override** post-hoc. Appropriate for **reversible, lower-impact, well-understood** actions (e.g., auto-quarantine high-confidence phishing — releasable later). [Q4-6]
- **HOOTL (human-out-of-the-loop):** no real-time human; relies on design-time constraints + post-hoc audit. Acceptable only for **routine, safe-by-design, tightly-constrained** actions (the ABS-brakes analogy). [Q4-13]

### Gating criteria + confidence thresholds

- **Microsoft** documents **confidence-threshold escalation architectures**: high-confidence → autonomous; low-confidence/ambiguous → routed to human reviewer, asynchronous approval, time-bound SLA. [Q4-12]
- **Tines** worked example: phishing with multiple corroborating signals → **auto-quarantine** (high confidence + reversible); single ambiguous signal → human review. Same action, **different gate by asset tier** (MFA-reset auto for standard user, *review* for privileged admin). **Novelty** triggers a checkpoint (automation shouldn't act on scenarios it wasn't designed for). [Q4-6]
- **Reversibility is the safety net:** lower-confidence autonomy is acceptable for reversible actions; irreversible actions need high confidence **and** HITL regardless. [Q4-6]

### Promotion criteria between tiers (evidence-based, reversible)

- **Dash0 promotion floors** (transferable to SOC workflows): **>90% scenario pass rate, <5% evaluator false-positive rate, <10% human-override rate** over a recent window, measured **per surface**; high-risk surfaces may never promote. [Q4-15]
- **Start with more checkpoints than you think you need, remove based on outcome evidence** (Tines). [Q4-6]
- **Promotion must be reversible** — if a promoted workflow's false-positive/business-impact rises, roll the autonomy level back (CISA + Tines). [Q4-6][Q4-10]

### NIST + CISA agentic-AI guidance (authoritative)

- **NIST RFI on agentic-AI cybersecurity** calls for controls at both **model level** (robustness, adversarial training) and **agent-system level** (identity/IAM for agents, action logging, human oversight); a NIST AI-cybersecurity-framework profile is in progress. [Q4-9]
- **CISA + allied agencies joint guidance (Apr 2026, "Careful Adoption of Agentic AI Services"):** maintain **HITL checkpoints for high-impact/irreversible actions**; **do NOT grant agents broad/unrestricted access** to sensitive data/systems; start with **low-risk use cases**; embed agentic-AI security in *existing* frameworks (secure-by-design, defense-in-depth, least privilege, continuous monitoring); agents must **fail-safe and escalate when uncertain**; **the decision about when human approval is required must be made by system designers, not delegated to the agent.** [Q4-10][Q4-11][Q4-14]
- **CISA OT guidance** (via Tines) is categorical: **AI/LLMs must NOT make safety decisions autonomously; HITL must be maintained** in OT. Directly relevant given aletheon's OT origin and any Prism OT-adjacent sensors. [Q4-6]
- **EU AI Act** (via IBM): high-risk AI must enable effective human oversight (manual operation, intervention, override, real-time monitoring) by competent, trained, empowered persons. [Q4-8]

**Net for ARO:** Recommendation-tier autonomy is *safe and expected* (matches C10 GAP-Q2); Action-tier autonomy is **gated by default**, promotion is **per-action-class, evidence-measured, and reversible**, and **system designers (not the agent) set the gates** — a hard CISA constraint that should be an architectural invariant in Prism. [Q4-10][Q4-15]

---

## Q5 — Lifecycle + State Machine + Dedup/Correlation + Linkage

### Canonical lifecycle state machine (convergent across platforms)

The near-universal pattern: **New/Open → Acknowledged/Assigned → In-Progress/Active → Resolved/Closed**, with **Reopened** transitions, plus extended terminal/policy states. [Q5-1][Q5-11][Q5-13][Q5-15]

- **Cortex Xpanse** is the richest documented model and the best blueprint: 4 base statuses **New → In Progress → Resolved → Reopened**, with **resolution sub-statuses split into TERMINAL vs REOPENABLE**:
  - **Terminal** (never reopens, even if condition persists): `Resolved – Contested Asset`, `Resolved – Risk Accepted`, `Resolved – No Risk`. These encode **governance/policy decisions**, not technical disappearance.
  - **Reopenable** (reopens if condition recurs): `Resolved – No Longer Observed`, `Resolved – Remediated Automatically`, generic `Resolved`.
  - System-only statuses (`No Longer Observed`, `Remediated Automatically`) cannot be set manually; custom statuses are supported. [Q5-1]
- **Sentinel:** incident **New / Active / Closed**, with a **two-tier state machine** — incident status + per-**task** status (Created/In-Progress/Completed/Deleted). Closure should coincide with task completion (auditable). [Q5-2][Q5-11]
- **PagerDuty:** incident **triggered → acknowledged → resolved**; ack only *halts escalation* (ack-timeout **retriggers** = notification reopen without new identity); alerts are simpler (triggered/resolved); **incident resolves when all its alerts resolve.** [Q5-13][Q5-8]
- **Opsgenie:** open/acknowledged + **snooze** (timed notification suppression that reverts to open+unacked on expiry). [Q5-5]
- **OCSF** abstracts these via standardized `status_id` enums (open/in-progress/closed-style) on base events — the vendor-neutral status layer Prism should map to. [Q5-6][Q5-15]

**Extended states to model:** **Dismissed/No-Risk** (false positive), **Suppressed** (real but intentionally silenced — maintenance/known-issue), **Risk-Accepted** (governance), **Expired/Stale** (auto-closed unactioned). The terminal-vs-reopenable distinction is the **single most important lifecycle design lesson** for ARO: closing because a human accepted the risk is semantically different from closing because the condition vanished, and only the latter should reopen. [Q5-1][Q5-10][Q5-12]

### Dedup / correlation — the "1000 alerts → 10–20 AROs" goal

This is exactly aletheon's stated goal and is a *solved-pattern* in practice. [Q5-9][Q5-10][Q5-12]

- **Prometheus Alertmanager:** **fingerprint = fnv64a hash over sorted labels** (the dedup key); **grouping** by shared labels (e.g., service) collapses N alerts → 1 notification; **silencing** (matcher-based) + **inhibition** (suppress lower-priority when higher-priority for same service is firing). [Q5-12][Q5-7]
- **PagerDuty `dedup_key`:** doubles as **correlation key AND idempotency mechanism** — an event with an existing open incident's `dedup_key` attaches to it (no new incident) and drives the incident to acknowledged (suppressing re-notification). [Q5-8][Q5-13]
- **Panther correlation rules:** group OR sequence rules over a **`LookbackWindowMinutes`** dedup window; collapse multi-step patterns into one correlation signal/alert. Window-sizing formulas: if rule-rate `R ≤ T_max` (max signal timespan), set window `W ≈ 2·T_max + L` (ingest latency); if `R > T_max`, `W ≈ T_max + R + L`. [Q5-9]
- **Sentinel alert grouping:** scheduled rules group alerts into incidents by entity/rule identity; **>150 alerts → spawn a new incident** (manageability cap). [Q5-14]
- **Rapid7 alert-fatigue guidance:** shift from volume-based to **risk-based** detection; consolidate+correlate across tools; prioritize by **asset criticality + business impact**; automate repetitive triage. [Q5-10]

**For ARO:** N Observations collapse into far fewer Recommendations via (a) a **dedup fingerprint** (hash over canonical entity+detection-type labels), (b) **correlation windows** (group/sequence over time), and (c) **entity-centric grouping** (multiple Observations about the same asset/entity → one Recommendation). This is the mechanism behind aletheon's 1000→10-20 aspiration. [Q5-9][Q5-12][Q5-14]

### Idempotency

At-least-once delivery is the norm; **idempotency keys** (PagerDuty `dedup_key`, Alertmanager fingerprint) make reprocessing the same event a no-op. **For Prism this is doubly critical at the Action tier given C2 satellite offline-queue reconnect retries** (prior C15 Q4.5/Q6) — every Action MUST carry a client-generated idempotency key so reconnect retries don't double-apply. [Q5-8][Q5-12]

### Linkage (entities / evidence / actions / case)

- Observations link to **entities/assets** (Sentinel `Entities`, OCSF `src_endpoint`, Xpanse assets) — Prism's C12 entity model. [Q5-1][Q5-11]
- Incidents/cases carry **tasks** with their own lifecycle; closure audited against task completion. [Q5-2]
- Evidence linkage + chain-of-custody (Q3) ties Observations→Recommendations→Actions into a traceable case. [Q5-2][Q5-11]

---

## Q6 — Data-Model Synthesis: A Rigorous ARO Model for Prism

Synthesizing Q1–Q5 and incorporating aletheon's `aros` table as one input. **Verdict on aletheon's schema: a solid pragmatic starting point, but OT/asset-centric and under-specified on provenance, lifecycle nuance, autonomy, and the loop.** Below: keep / generalize / add.

### Keep from aletheon's `aros` table

- **Single unified table with `aro_type` discriminator** (action|recommendation|observation) — pragmatic and queryable; matches the conceptual unity of the loop. (Alternative: 3 tables. See sub-fork below.)
- **`priority`** (numeric), **`status`** (open/acknowledged/resolved/dismissed) — aligns with canonical lifecycle.
- **Dual-audience text** (`plain_english` + `technical_details` + `suggested_response`) — **validated directly by NISTIR 8312's "Meaning" principle** (audience-appropriate explanations). [Q3-2] Keep, but generalize the audience labels (operator/analyst/auditor) and make audience a tag rather than fixed columns.
- **`source_event_ids[]`, `affected_asset_ids[]`** — evidence + entity linkage; keep but generalize `asset` → `entity` (C12).
- **AI provenance triplet** `confidence` / `model_version` / `reasoning` — keep but **substantially expand** (Q3).
- **Ack/resolve workflow** (`acknowledged_by/at`, `resolved_by/at`, `resolution_notes`) — keep.

### Generalize (aletheon is OT/asset-centric; Prism is multi-source/multi-entity)

- `affected_asset_ids[]` → **`affected_entity_refs[]`** typed by entity kind (host, user, account, IP, service, cloud-resource, OT-asset…) per C12. Prism federates many sources, not just Claroty/Armis.
- `source_event_ids[]` → **`source_observation_refs[]` + `evidence_package_id`** (C10 GAP-Q2). Distinguish the *Observations* a Recommendation derives from vs the raw *evidence items*.
- Flat `status` → **terminal-vs-reopenable resolution sub-statuses** (Xpanse model): `resolved_no_longer_observed` / `resolved_remediated` / `resolved_remediated_auto` (reopenable) vs `resolved_risk_accepted` / `resolved_no_risk` / `resolved_contested` (terminal). [Q5-1]
- `confidence` (single float) → **raw + calibrated confidence + method + ECE + optional conformal set** (Q3). [Q3-6][Q3-7]
- `reasoning` (free text) → **structured explanation** (contributing features, rules/playbooks invoked + versions, faithfulness indicator, knowledge-limits/OOD flag) (Q3). [Q3-2][Q3-8]

### Add (missing from aletheon, required by the research)

- **The loop / linkage edges:** explicit refs Observation→Recommendation→Action (provenance chain), and **Action→resulting-Observation** (close the OODA loop, Q1). [Q1-10]
- **Observation 3-layer structure** (Endsley): `perceived_fact` / `comprehension` / `projection`. [Q1-18]
- **Recommendation risk metadata** (Tines 5 axes): `reversibility ∈ {reversible, compensable, irreversible}`, `blast_radius`, `novelty`, `asset_tier`, `compliance_flags`. [Q4-6]
- **Autonomy + gating** on Recommendation/Action: `autonomy_level` (advisory/suggested/auto-with-approval/autonomous), `gate_mode` (HITL/HOTL/HOOTL), `required_approver_role` (C18). [Q4-6][Q4-10][Q4-12]
- **Full provenance** (W3C PROV mapping): model card ref+version, prompt+system-prompt version, RAG doc set, generation config, seed. [Q3-4][Q3-12]
- **Citations with faithfulness** (C12 + the 57%-unfaithful finding): per-citation evidence map + faithfulness flag/score + unsupported-claim flag. [Q3-8]
- **Dedup/correlation:** `dedup_fingerprint` (hash over canonical entity+detection-type), `correlation_id`, `collapsed_observation_count`. [Q5-9][Q5-12]
- **Action-tier specifics:** `idempotency_key` (mandatory, C2-critical), `rollback_handle`, `executor_identity`, `approver_identity`, `dry_run` flag, action result. [Q5-8] + prior C15.
- **Immutable audit:** hashed who/what/when/where/why + reproducibility inputs. [Q3-16]

### Proposed ARO entity model (conceptual)

**Shared base (all three tiers):** `id`, `aro_type`, `tenant_id` (C18 multi-tenant), `title`, `priority`, `status` + `resolution_substatus`, `created_at`/`updated_at`, audience-tagged text, `affected_entity_refs[]`, `dedup_fingerprint`, `correlation_id`, lifecycle audit (ack/resolve who/at), immutable structured audit trail.

**Observation extends base:** `perceived_fact`, `comprehension`, `projection`, `severity`, `calibrated_confidence`, MITRE tactics/techniques, `source_detector_id`, `evidence_refs[]`, `collapsed_observation_count`, timestamps (start/end/generated).

**Recommendation extends base:** `source_observation_refs[]`, `proposed_actions[]`, **full AI-provenance block** (model card+version, prompt/system-prompt version, RAG set, gen config, seed), **confidence block** (raw/calibrated/ECE/conformal), **explanation block** (features, rules, faithfulness, knowledge-limits), **citations[]** (evidence-mapped + faithfulness), **risk metadata** (reversibility/blast/novelty/asset-tier/compliance), `autonomy_level`, `gate_mode`, `required_approver_role`, evidence_package_id.

**Action extends base:** `source_recommendation_ref`, `source_observation_refs[]` (full chain), `action_type`, `target`, `parameters`, **`idempotency_key`**, `dry_run`, `executor_identity`, `approver_identity`, timestamps (requested/approved/executed), exec status (pending/scheduled/running/succeeded/failed/rolled-back), `result`, `rollback_handle`, `resulting_observation_refs[]` (loop closure).

**State machines:** Observation: New→Active→Resolved(terminal|reopenable)→Reopened. Recommendation: Proposed→Approved|Rejected→Implemented|Superseded|Expired. Action: Pending→Scheduled→Running→Succeeded|Failed→Rolled-back; (Approval sub-state-machine: Awaiting-Approval→Approved|Denied|SLA-Expired→Escalated). [Q5-1][Q5-13][Q2-2]

---

## Q7 — How ARO Surfaces in PrismQL + the Agent + the SOAR Layer

Building on the prior C15 lean (DO NOT embed mutations in PrismQL grammar; keep the planner read-only to preserve the `tests/external/perimeter-violation/` security perimeter):

- **Observations = query/detection results.** PrismQL queries (DataFusion planner over OCSF-normalized federated data) **already produce Observations** — they are the query result set, OCSF-shaped. An Observation is a materialized, evidence-backed query finding. This requires **no grammar change**; it's a typed projection/sink of existing results. The Endsley 3-layer structure (perceive/comprehend/project) is produced by the **agent enriching** the raw query result, not by the SQL planner. [Q1-18]
- **Recommendations = AI-suggested responses, emitted by the S3 agent (and Query Workers).** These live in the **agent layer**, not the grammar — matching C10 GAP-Q2 (Query Workers produce findings + recommendations, no auto-act) and the AI-SOC documented norm. [Q2 §AI-SOC][Q4-1] PrismQL at most gains a **read-only `RECOMMEND` projection** (a query *emits a Recommendation as DATA*, never an executed Action) — sub-fork below.
- **Actions = gated connector writes, executed ONLY in a separate orchestration layer** (the prior C15 `prism-orchestration` recommendation), wrapping feature-flagged sensor write capabilities as discrete Common-Action-Model-style connectors, with HITL approval gates, idempotency keys, dry-run, blast-radius caps, rollback, and immutable audit. PrismQL/DataFusion never executes a write. [prior C15][Q4-10]
- **The shared Knowledge base (MAPE-K/C12)** — entity model + evidence packages + policy/setpoints — underpins all three tiers, and is where dedup/correlation collapse N Observations → fewer Recommendations. [Q1-14][Q5-9]
- **The loop:** Action results feed back as new Observations (executed change is re-observed via the next query) — Prism is an OODA/MAPE-K loop where PrismQL is the Monitor/Observe sensor, the agent is Orient/Analyze+Plan/Recommend, and the orchestration layer is Decide-gate/Execute/Act. [Q1-10][Q1-14]

---

## ANALYSIS + LEANS

### Proposed ARO taxonomy (lean)
ARO is well-founded as the security instantiation of OODA / MAPE-K / Endsley-SA / Sense-Plan-Act, framed by control theory. Adopt the **closed-loop three-tier model** with these rigor upgrades over a flat scheme: **Observation = 3-layer product** (perceive/comprehend/project, Endsley); **Recommendation = AI-safe, autonomously-emittable, never-auto-executing** (matching C10 GAP-Q2 + the documented AI-SOC norm); **Action = gated-by-default, provenance-chained, idempotent, reversible-where-possible**, executed only in a separate orchestration layer. A **shared Knowledge base (C12 entities + evidence + policy/setpoint)** underpins all three (MAPE-K). [Q1-10][Q1-14][Q1-18][Q4-1]

### Data model (lean)
Generalize aletheon's `aros` table: keep the `aro_type` discriminator + dual-audience text (NISTIR-8312-validated) + ack/resolve workflow; generalize asset→entity (C12) and event→observation+evidence; and **add the four things aletheon lacks**: (1) **rich AI provenance** (W3C-PROV-mapped: model card+version, prompt, RAG set, gen config, seed); (2) **calibrated confidence + conformal sets** (raw confidence is untrustworthy); (3) **citations with per-citation faithfulness** (the 57%-unfaithful RAG finding makes naked citation fields dangerous — this is the key C12 caveat); (4) **terminal-vs-reopenable resolution sub-statuses** (Xpanse model — risk-accepted ≠ no-longer-observed). Add risk metadata (Tines 5 axes), autonomy/gate fields, dedup fingerprint + correlation, and Action idempotency/rollback. [Q3-2][Q3-4][Q3-6][Q3-7][Q3-8][Q5-1]

### Tier-promotion / autonomy model (lean)
No ratified SOC-autonomy standard exists, but the convergent pattern is clear: **Observation = autonomous (HOOTL ok); Recommendation = autonomous-generate-but-human-validate (advisory/suggested); Action = gated by default** with the Tines 5 axes. Promotion between Action autonomy tiers is an **evidence-based, per-action-class, reversible** decision (Dash0 floors: >90% pass, <5% evaluator FP, <10% override; high-risk surfaces may never promote). **Hard CISA invariant for Prism: system designers — not the agent — decide when human approval is required; agents fail-safe and escalate; no broad agent access to sensitive systems.** OT-adjacent sensors: HITL mandatory for safety decisions (CISA OT). [Q4-6][Q4-10][Q4-15]

### Lifecycle / state-machine (lean)
Three coupled state machines (Observation / Recommendation / Action) plus an Action-approval sub-machine, with **terminal-vs-reopenable resolution** (Xpanse), task-completion-aware closure (Sentinel), and ack-timeout retrigger (PagerDuty). Dedup/correlation collapses N Observations → fewer Recommendations via fingerprint hashing (Alertmanager fnv64a-over-labels), correlation windows (Panther lookback formulas), and entity-centric grouping (Sentinel) — the mechanism behind aletheon's "1000→10-20" goal. **Idempotency keys are mandatory at the Action tier** given C2 offline-queue reconnect retries. [Q5-1][Q5-2][Q5-8][Q5-9][Q5-12][Q5-13]

### PrismQL + agent surfacing (lean)
PrismQL planner stays **read-only** (preserve the security perimeter). Observations = OCSF-shaped query results; the agent adds the Endsley comprehension/projection layers. Recommendations live in the **S3 agent / Query Workers layer** (C10 GAP-Q2), at most surfaced as a read-only `RECOMMEND` data-projection in PrismQL. Actions execute **only** in a separate `prism-orchestration` layer (HITL gates, idempotency, dry-run, rollback, audit). The whole thing is an OODA/MAPE-K loop. [prior C15][Q1-14][Q2 §AI-SOC]

### Genuine sub-forks needing a HUMAN decision

1. **One unified `aros` table (aletheon-style, `aro_type` discriminator) vs three typed entities (Observation/Recommendation/Action).** Lean: the tiers diverge enough in fields/state-machines (esp. Action's idempotency/rollback/approval) that **three typed entities sharing a common base** is cleaner and more type-safe in Rust — but a single table is simpler to query and matches aletheon. **Architect/data-engineer call.**
2. **Autonomy-level default for v1.** Lean: **strictly recommend-only** (Observation + Recommendation tiers, zero autonomous Action) — matches C10 GAP-Q2, CISA, NIST, EU AI Act. Whether *any* autonomous Action class is permitted in v1 (even reversible/high-confidence) is a **human risk-acceptance decision**. [Q4-10]
3. **Does PrismQL get a read-only `RECOMMEND` projection in v1, or do Recommendations come purely from the agent layer** (keeping the grammar/perimeter untouched)? Grammar-scope + perimeter-test fork. Lean: **agent-layer only for v1**; defer grammar `RECOMMEND` unless there's a concrete demand. [prior C15]
4. **Confidence rigor scope for v1.** Calibration (ECE/temperature scaling) + conformal prediction are non-trivial to build. Lean: **ship calibrated confidence + faithfulness flags as required fields from day one** (they are correctness/safety, not polish, per production-grade default), but **conformal-prediction sets** could be a later enhancement with a concrete story anchor. **Human prioritization call.** [Q3-6][Q3-7]
5. **Citation-faithfulness enforcement mechanism.** Given the 57%-unfaithful RAG finding, C12's mandatory citations need an *enforcement/measurement* mechanism (constrained generation / token attribution / post-hoc check). Lean: **post-hoc statement-vs-citation faithfulness check is the minimum viable enforcement**; which mechanism is an architect/research call. [Q3-8]
6. **Aletheon graph/vector reuse.** Aletheon uses Apache AGE (graph) + pgvector for entity relationships + semantic search. Whether Prism's C12 entity model adopts a graph + vector substrate (vs RocksDB column families) for ARO entity linkage is an **architect call** outside this pass's scope — flagged because aletheon's schema implies it.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 5 | Deep multi-source sweeps (`reasoning_effort=high`, `strip_thinking=true`) on: (Q1) OODA/MAPE-K/Endsley-SA/Sense-Plan-Act/control-theory conceptual lineage; (Q2) finding/recommendation/action as first-class data models across SOAR/detection/UEBA/ASM/AI-SOC; (Q3) AI-recommendation provenance/calibration/explainability/citation-faithfulness/auditability; (Q4) autonomy-level taxonomies + HITL/HOTL/HOOTL + NIST/CISA agentic-AI gating; (Q5) lifecycle state machines + dedup/correlation + idempotency + linkage. |
| Perplexity perplexity_ask | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | Not used — questions were conceptual/standards/platform-level, not single-library API docs. |
| Tavily (all) | 0 | Not needed — the 5 deep-research passes were self-citing with authoritative primary sources (NIST, W3C, DARPA, SAE-derived, vendor docs, OCSF, academic). |
| WebFetch / WebSearch | 0 | — |
| Read (local) | 2 | Prior C15 research + aletheon `init-db.sql` (`aros` table) as build-on inputs. |
| Training data | 1 area | General framing/cross-linking of the loops; every load-bearing claim is web-cited. Flagged: AI-SOC vendor *internal schemas* (Hunters/Dropzone/Prophet/Torq/Radiant) are proprietary/undocumented — those specifics are inferred-pattern, flagged inline; primary cybernetics texts (Wiener) did not surface, control-theory framing corroborated via autonomic-computing sources. |

**Total MCP tool calls:** 5 (all `perplexity_research`, `reasoning_effort=high`)
**Training data reliance:** low — all five research questions answered via deep web research with authoritative primary sources; every non-obvious claim cited; gaps (no SOC-autonomy standard; proprietary AI-SOC schemas; missing primary cybernetics) flagged explicitly rather than papered over.

---

## Sources

### Q1 — Conceptual loops (OODA / MAPE-K / Endsley SA / SPA / control theory)
- [Q1-1] Marine Corps Association — OODA Loop for Strategy: https://www.mca-marines.org/gazette/ooda-loop-for-strategy/
- [Q1-2] GeekBoss — Boyd, Patterns of Conflict: https://geekboss.com/blog/boyd-patterns-of-conflict
- [Q1-3] PacGenesis — OODA Loop in Cybersecurity (Defender's Playbook): https://pacgenesis.com/what-is-the-ooda-loop-in-cybersecurity-a-defenders-playbook/
- [Q1-4] ScienceDirect — Autonomic Computing (topic overview): https://www.sciencedirect.com/topics/computer-science/autonomic-computing
- [Q1-5] Malburg et al. — MAPE-K Loops for Adaptive Workflow Mgmt (PDF): https://www.wi2.uni-trier.de/shared/publications/2023_MalburgEtAl_MAPEK_Loops.pdf
- [Q1-6] Endsley — Toward a Theory of Situation Awareness in Dynamic Systems (Human Factors): https://journals.sagepub.com/doi/10.1518/001872095779049543
- [Q1-7] ScienceDirect — Situation Awareness in SOCs (systematic review): https://www.sciencedirect.com/science/article/pii/S0167404822004618
- [Q1-8] Wikipedia — Sense Plan Act: https://en.wikipedia.org/wiki/Sense_Plan_Act
- [Q1-9] Col John Boyd digital archive: https://www.coljohnboyd.com
- [Q1-10] Boyd — The Essence of Winning and Losing (PDF): https://slightlyeastofnew.com/wp-content/uploads/2010/03/essence_of_winning_losing.pdf
- [Q1-11] Kephart & Chess — The Vision of Autonomic Computing (IEEE): https://ieeexplore.ieee.org/document/1160055/
- [Q1-12] Endsley — SA measurement (SAGAT) (SAGE): https://journals.sagepub.com/doi/10.1177/154193128803200221
- [Q1-13] Autonomic Microservice Mgmt via Agentic AI + MAPE-K (arXiv): https://arxiv.org/html/2506.22185v1
- [Q1-14] IBM — Architectural Blueprint for Autonomic Computing (PDF): https://users.cs.fiu.edu/~sadjadi/Teaching/Autonomic%20Grid%20Computing/CIS-6612-Summer-2006/AC-Blueprint-WhitePaper-V7.pdf
- [Q1-15] Google Cloud — AI boosting OODA-loop impact on cybersecurity: https://cloud.google.com/transform/lightning-fast-decision-making-how-ai-can-boost-ooda-loop-impact-on-cybersecurity
- [Q1-16] The Decision Lab — The OODA Loop: https://thedecisionlab.com/reference-guide/computer-science/the-ooda-loop
- [Q1-17] Slightly East of New — An Orientation for IOHAI (Boyd's orientation): https://slightlyeastofnew.com/2020/11/13/an-orientation-for-iohai/
- [Q1-18] ScienceDirect — SA validity assessment: https://www.sciencedirect.com/science/article/pii/S0003687026001043
- [Q1-19] ScienceDirect — (SOC decision cycles): https://www.sciencedirect.com/science/article/pii/S0963868717304353

### Q2 — Three tiers as data models (SOAR / detection / UEBA / ASM / AI-SOC)
- [Q2-1] Cortex XSOAR — War Room in an investigation: https://docs-cortex.paloaltonetworks.com/r/Cortex-XSOAR/8/Cortex-XSOAR-SaaS-Documentation/Use-the-War-Room-in-an-investigation
- [Q2-2] Splunk SOAR — Playbook API / Containers: https://help.splunk.com/en?resourceId=SOAR_PlaybookAPI_Containers
- [Q2-3] Tines — Actions docs: https://www.tines.com/docs/actions/
- [Q2-4] Torq: https://torq.io
- [Q2-5] Panther — Signals: https://docs.panther.com/detections/signals
- [Q2-6] Microsoft Sentinel — Security alert schema: https://learn.microsoft.com/en-us/azure/sentinel/security-alert-schema
- [Q2-7] Google SecOps — Unified Data Model (UDM): https://security.googlecloudcommunity.com/community-blog-42/new-to-google-secops-unified-data-model-udm-3958
- [Q2-8] AWS Security Lake — OCSF: https://docs.aws.amazon.com/security-lake/latest/userguide/open-cybersecurity-schema-framework.html
- [Q2-9] Exabeam — UEBA primer: https://www.exabeam.com/explainers/ueba/what-ueba-stands-for-and-a-5-minute-ueba-primer/
- [Q2-10] Securonix — risk-management approach to insider threat: https://www.securonix.com/blog/using-a-risk-management-approach-to-build-your-insider-threat-program/
- [Q2-11] Cortex XSOAR — Incident management: https://docs-cortex.paloaltonetworks.com/r/Cortex-XSOAR/8.6/Cortex-XSOAR-On-prem-Documentation/Incident-management
- [Q2-12] XSOAR — incident lifecycle: https://xsoar.pan.dev/docs/incidents/incident-xsoar-incident-lifecycle
- [Q2-13] Splunk SOAR — manage status/severity/resolution: https://help.splunk.com/en/splunk-soar/soar-cloud/use-soar-cloud/get-started-using-splunk-soar-cloud/manage-the-status-severity-and-resolution-of-events-in-splunk-soar-cloud
- [Q2-14] SentinelOne — OCSF analyst experience: https://www.sentinelone.com/blog/simplifying-the-security-analyst-experience-with-open-cybersecurity-schema-framework-ocsf/
- [Q2-15] Splunk SOAR — understanding artifacts: https://help.splunk.com/splunk-soar/soar-cloud/develop-apps/python-playbook-api-reference/overview/understanding-artifacts
- [Q2-16] Exabeam — user profile / session score: https://docs.exabeam.com/en/cloud-delivered-advanced-analytics/all/user-guide/153664-get-to-know-a-user-profile.html

### Q3 — AI recommendation provenance / calibration / explainability / citations / audit
- [Q3-1] NIST AI RMF 1.0 (AI 100-1, PDF): https://nvlpubs.nist.gov/nistpubs/ai/nist.ai.100-1.pdf
- [Q3-2] NISTIR 8312 — Four Principles of Explainable AI (PDF): https://nvlpubs.nist.gov/nistpubs/ir/2021/nist.ir.8312.pdf
- [Q3-3] DARPA — Explainable AI (XAI) program: https://www.darpa.mil/research/programs/explainable-artificial-intelligence
- [Q3-4] W3C — PROV Overview: https://www.w3.org/TR/prov-overview/
- [Q3-5] Mitchell et al. — Model Cards (arXiv): https://arxiv.org/abs/1810.03993
- [Q3-6] ICLR blogposts 2025 — Calibration (ECE, reliability diagrams): https://iclr-blogposts.github.io/2025/blog/calibration/
- [Q3-7] Angelopoulos & Bates — Gentle Intro to Conformal Prediction (arXiv): https://arxiv.org/abs/2107.07511
- [Q3-8] "Correctness is not Faithfulness in RAG" (ACM, 57% unfaithful citations): https://dl.acm.org/doi/10.1145/3731120.3744592
- [Q3-9] Swimlane — AI SOC: https://swimlane.com/blog/ai-soc/
- [Q3-10] Cybersecurity Tribe — Building trust into automated cybersecurity decisions: https://www.cybersecuritytribe.com/articles/building-trust-into-automated-cybersecurity-decisions
- [Q3-11] NIST SP 1270 — Bias in AI (PDF): https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.1270.pdf
- [Q3-12] NIST AI 600-1 — Generative AI Profile (PDF): https://nvlpubs.nist.gov/nistpubs/ai/NIST.AI.600-1.pdf
- [Q3-13] NIST SP 800-61r3 — Incident handling (PDF): https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-61r3.pdf
- [Q3-14] NIST SP 800-92 — Log management: https://csrc.nist.gov/pubs/sp/800/92/final
- [Q3-15] CyberProof — Google SecOps modern threat detection/response: https://www.cyberproof.com/security-operations-center/google-secops-modern-threat-detection-and-response/
- [Q3-16] W3C — PROV-DM (data model): https://www.w3.org/TR/prov-dm/
- [Q3-17] AWS — Logging strategies for security incident response: https://aws.amazon.com/blogs/security/logging-strategies-for-security-incident-response/
- [Q3-18] NIST SP 800-86 — Integrating forensic techniques into IR (PDF): https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-86.pdf

### Q4 — Autonomy levels / HITL-HOTL-HOOTL / NIST + CISA gating
- [Q4-1] SentinelOne — Autonomous SOC Maturity Model (L0–L4): https://www.sentinelone.com/blog/introducing-the-autonomous-soc-maturity-model/
- [Q4-2] Splunk — Security automation: https://www.splunk.com/en_us/blog/learn/security-automation.html
- [Q4-3] WatchGuard — Security Operations Maturity Model: https://www.watchguard.com/wgrd-news/blog/security-operations-maturity-model-ii-what-it
- [Q4-4] Torq — AI for security operations (Gartner augment-not-replace): https://torq.io/blog/ai-for-security-operations/
- [Q4-5] Torq — Hyperautomation: https://torq.io/hyperautomation/
- [Q4-6] Tines — Human-in-the-loop workflows (5 axes; NIST IR 8596; CISA OT): https://www.tines.com/blog/human-in-the-loop-workflows-where-intelligent-automation-meets-judgment/
- [Q4-7] Radiant Security — agentic AI SOC: https://radiantsecurity.ai
- [Q4-8] IBM — Human-in-the-loop (EU AI Act oversight): https://www.ibm.com/think/topics/human-in-the-loop
- [Q4-9] Inside Cybersecurity — NIST RFI on agentic-AI cybersecurity: https://insidecybersecurity.com/share/17654
- [Q4-10] Industrial Cyber — CISA + partners agentic-AI security guidance: https://industrialcyber.co/ai/cisa-and-partners-release-agentic-ai-security-guidance-to-protect-critical-infrastructure-outline-mitigation-action/
- [Q4-11] Crowell — US + allied first joint guidance on securing agentic AI: https://www.crowell.com/en/insights/client-alerts/american-and-allied-cyber-agencies-issue-first-joint-guidance-on-securing-agentic-ai
- [Q4-12] Microsoft Learn — Design human-in-the-loop approval workflows (confidence-threshold escalation): https://learn.microsoft.com/en-us/training/modules/aaai-design-human-in-loop-approval-workflows/
- [Q4-13] Institute for Future Conflict (USAFA) — "Please Stop Saying Human-in-the-Loop" (HOOTL / DoD 3000.09): https://ifc.usafa.edu/articles/please-stop-saying-human-in-the-loop
- [Q4-14] CISA et al. — Careful Adoption of Agentic AI Services (PDF, Apr 2026): https://media.defense.gov/2026/Apr/30/2003922823/-1/-1/0/CAREFUL%20ADOPTION%20OF%20AGENTIC%20AI%20SERVICES_FINAL.PDF
- [Q4-15] Dash0 — Six Levels of Agentic Software Engineering (CSA/ASDLC autonomy scales; measurement-based promotion): https://www.dash0.com/knowledge/the-six-levels-of-agentic-software-engineering
- [Q4-16] NVIDIA — Agentic Autonomy Levels and Security: https://developer.nvidia.com/blog/agentic-autonomy-levels-and-security/

### Q5 — Lifecycle / dedup / correlation / idempotency / linkage
- [Q5-1] Cortex Xpanse — Alert Status (terminal vs reopenable resolutions): https://docs-cortex.paloaltonetworks.com/r/Cortex-XPANSE/2/Cortex-Xpanse-Expander-User-Guide/Alert-Status
- [Q5-2] Microsoft Sentinel — Audit & track tasks (incident + task two-tier lifecycle): https://learn.microsoft.com/en-us/azure/sentinel/audit-track-tasks
- [Q5-3] Splunk ES — Customize notable event settings: https://help.splunk.com/en/splunk-enterprise-security-7/administer/7.2/incident-review-and-investigations/customize-notable-event-settings-in-splunk-enterprise-security
- [Q5-4] PagerDuty — Incident Response Lifecycle for DevOps: https://www.pagerduty.com/resources/digital-operations/learn/incident-response-lifecycle-for-devops/
- [Q5-5] Opsgenie — Snooze an alert: https://support.atlassian.com/opsgenie/docs/snooze-an-alert/
- [Q5-6] Query.AI — Definitive Guide to OCSF Mapping (status_id): https://www.query.ai/resources/blogs/definitive-guide-to-open-cybersecurity-schema-framework-ocsf-mapping/
- [Q5-7] Prometheus developers — alert fingerprint (fnv64a over sorted labels): https://groups.google.com/g/prometheus-developers/c/24qpU9QfOL0
- [Q5-8] PagerDuty — Send Alert Event (dedup_key as correlation + idempotency): https://developer.pagerduty.com/docs/send-alert-event
- [Q5-9] Panther — Correlation Rules (LookbackWindowMinutes formulas): https://docs.panther.com/detections/correlation-rules
- [Q5-10] Rapid7 — Alert Fatigue in cybersecurity: https://www.rapid7.com/fundamentals/alert-fatigue-cybersecurity/
- [Q5-11] Microsoft Sentinel — Investigate cases (incident New/Active/Closed): https://learn.microsoft.com/en-us/azure/sentinel/investigate-cases
- [Q5-12] Prometheus — Alertmanager (dedup/grouping/silencing/inhibition): https://prometheus.io/docs/alerting/latest/alertmanager/
- [Q5-13] PagerDuty — Incidents (lifecycle, ack-timeout retrigger): https://support.pagerduty.com/main/docs/incidents
- [Q5-14] Microsoft Sentinel — Create analytics rules (alert grouping, 150-alert cap): https://docs.azure.cn/en-us/sentinel/create-analytics-rules
- [Q5-15] OCSF — base_event.json (status attributes): https://github.com/ocsf/ocsf-schema/blob/main/events/base_event.json

### Build-on inputs (local)
- Prior C15 research: `/Users/jmagady/Dev/prism/.factory/research/prismql-actions-soar-onprem-models-2026-06-27.md`
- Aletheon spike `aros` table: `/Users/jmagady/Dev/aletheon_2/spike/init-db.sql`
