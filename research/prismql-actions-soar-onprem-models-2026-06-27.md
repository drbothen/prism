# Research: Actions in PrismQL / SOAR Platform + On-Prem Models (C15)

> **SIDE-ANALYSIS item:** C15 — Actions in PrismQL / SOAR platform + on-prem models
> **Mode:** CAPTURE / research only (`do_not_execute`). No live spec/BC/ADR/STATE.md/SESSION-HANDOFF.md modified.
> **Date:** 2026-06-27
> **Type:** general (technology + architecture research, feeds C15 design decision)
> **Author:** research-agent
> **Status:** complete

---

## Scope & Context

Prism today is a **read-only federated security query engine**: PrismQL (Chumsky grammar + DataFusion planner) issues queries, sensor adapters fan out to vendor APIs, results are normalized to OCSF/protobuf and returned to an LLM agent. The user wants to evaluate:

- **(a)** Adding **ACTIONS** to the query language / becoming a **SOAR platform**, structured around an **Action · Recommendation · Observation (ARO)** model.
- **(b)** Running **models on-prem / air-gapped** for agent reasoning + action decisioning, fitting the C7 pluggable AI-opaque `ModelBackend` (candle + ort + wasmtime + tract).

Relevant project anchors (from prompt + project memory):
- Sensor APIs **already have write capabilities**, locked behind a robust feature-flag system (project memory `project_feature_flags.md`).
- **C7** decided a pluggable AI-opaque `ModelBackend` (candle + ort + wasmtime + tract).
- **C10 GAP-Q2** = auditable agent evidence-package + self-QA (Query Workers analog).
- **S3** = server-hosted embedded agent. **C2** = satellite/edge execution. **C16** = masking. **C18** = RBAC. **AD-017** = AI-opaque credentials.

All non-obvious claims are cited. Tool/model/crate versions were verified against registries and official docs (see Research Methods). Where the deep-research model flagged uncertainty, I cross-checked with a second query and the conflict is noted inline.

---

## Q1 — How Query / Detection Languages Express ACTIONS

### Finding: actions are almost always a SEPARATE layer from the query language

The dominant industry pattern is to keep **detection logic in a query/rule layer** and delegate the act stage to a **separate workflow/playbook engine**. Splunk is the notable exception that allows inline action invocation from its query language. [SOAR-1][SOAR-2][SOAR-4][SOAR-11][SOAR-16]

| Platform | Where does the action live? | "Query → decide → act" model |
|---|---|---|
| **Splunk SPL / Enterprise Security** | `sendalert` command can invoke an alert action **inside the SPL pipeline** (e.g. `… \| sendalert block_ip param.ip="$result.src_ip$"`). But the **recommended production pattern** keeps SPL detection-only and attaches **Adaptive Response actions** via correlation-search config (Common Action Model + `cim_actions.py`). [SOAR-1][SOAR-2][SOAR-17] | Detect in SPL; decide via correlation-search thresholds/notable config; act via Adaptive Response (auto on fire, or manual from Incident Review). [SOAR-2][SOAR-17] |
| **Sigma + Sigma correlation** | **No action syntax at all.** Sigma is vendor-neutral detection-only; correlation rules chain detections by `id`/`name` but still stop at "this fired". Actions are 100% downstream platform responsibility. [SOAR-3] | Detect/correlate in YAML → transpile to backend → backend's alerting + SOAR does the acting. |
| **Microsoft Sentinel (KQL)** | KQL analytics rules are detection-only. Acting is **automation rules** (incident-level policy/orchestration) → **Logic Apps playbooks** (connectors, approvals). [SOAR-4][SOAR-11][SOAR-12] | KQL detect → analytics rule decide → automation rule routes → Logic App acts. Logic Apps "Approvals" connector implements human gates. [SOAR-12] |
| **Panther (detection-as-code)** | Detections always emit **signals**; **alerts** are gated by `Create Alert` flag + event thresholds + deduplication. Acting is **external** via alert destinations (email/Slack/**custom webhook** → SOAR). [SOAR-5][SOAR-13] | Detect (Python/declarative) → alert config decides → webhook hands off to external SOAR. |
| **Tines / Shuffle / n8n** | No-code/low-code **workflow ("story"/playbook) engines.** Detection is external (SIEM/EDR feeds a trigger); decide + act live in the workflow graph. [SOAR-7][SOAR-8][SOAR-9] | Trigger ingest → conditional logic → action nodes (HTTP/API/notify/contain). |
| **Torq** | AI-SOC "agentic" hyperautomation; workflow engine + AI agents propose/triage; actions in workflow language with connectors. [SOAR-15] | Agentic triage → workflow act with human oversight. |
| **Cortex XSOAR / Splunk SOAR** | Dedicated **playbook engines** "at the heart" of the platform. Tasks include manual tasks (human gates), conditional tasks, loops. Splunk SOAR's **Prompt block** is the canonical human-in-the-loop checkpoint. [SOAR-16][SOAR-18][SOAR-19] | Incident in → playbook decides/loops → actions via connectors; Prompt block pauses for approval. |

### Safety / approval / idempotency / rollback patterns (what's actually documented vs. left to design)

The deep-research sweep found a consistent gap: **approval gates and RBAC are explicitly documented; idempotency, rollback, dry-run, and two-person control are mostly left to playbook authors / org policy** rather than enforced by the platform. [SOAR-5][SOAR-10][SOAR-14][SOAR-18][SOAR-19]

- **Approval gates / human-in-the-loop (HITL):** Best-documented pattern. Splunk SOAR **Prompt block** assigns a user/role, requires answers to questions, enforces a **required response time (SLA)**, supports external SAML-authenticated approvers, and loops distribution (re-notify every N min until answered). [SOAR-18] XSOAR **manual tasks** serve the same role. [SOAR-19] Logic Apps Approvals connector for Sentinel. [SOAR-12]
- **Risk-tiering of what to auto-act vs. gate (Tines' 5 axes):** **reversibility, asset importance, confidence level, novelty, compliance mandate.** Irreversible/high-consequence actions (disable exec accounts, delete data, isolate prod) require human approval; reversible enrichment/ticketing/high-confidence quarantine can auto-run. [SOAR-14]
- **HITL vs. human-on-the-loop (HOTL):** HITL = workflow cannot proceed without approval; HOTL = autonomous but humans monitor + can override. Tines cites **draft NIST IR 8596** and **CISA/allied agentic-AI guidance**: human oversight required for irreversible actions; AI/LLMs must not make safety decisions autonomously in OT environments. [SOAR-14] StackAI: define agent scope + allowed actions, add guardrail/validation policy layers, use **propose→approve** workflows. [SOAR-20]
- **Rate-limit / blast-radius:** Most concretely realized as Panther **event thresholds + deduplication** (detection can be aggressive internally; alerting/acting is throttled). [SOAR-5] Splunk action-config restart requirements + RBAC capabilities (`list_storage_passwords`) gate credentialed actions. [SOAR-17]
- **Idempotency / rollback / dry-run:** **Not platform-mandated** in the docs reviewed for Splunk, Sentinel, Panther. Idempotent action scripts (e.g. re-adding same IP to blocklist is a no-op) are an implementation discipline. Rollback is a manually-authored compensating step. [SOAR-2][SOAR-10][SOAR-12][SOAR-17]
- **Two-person control:** No platform reviewed encodes it natively; implemented as two sequential approval steps in a playbook or via org policy. [SOAR-14][SOAR-18]
- **Splunk Adaptive Response best practice (transferable to Prism):** *do not* pack choice-routing / multiple operations into one action; instead emit events (`addevent()`/`writeevents()`) that trigger other correlation searches — keep actions discrete + auditable. [SOAR-2]

**Takeaway for Prism:** The mature design is **detection/query language stays declarative; a separate orchestration layer owns actions with explicit approval gates.** Splunk's inline `sendalert` is the *minority* pattern and even Splunk discourages it for production. This strongly informs the PrismQL recommendation below (Q2/Q3).

---

## Q2 — The Action · Recommendation · Observation (ARO) Model

### Industry analogues

The deep research did not find a named "ARO" taxonomy, but the underlying three-tier separation is **exactly the risk-tiering the industry already practices**, just rarely formalized as a vocabulary:

- **Observation ≈ signal/finding.** Panther: detections always produce **signals** regardless of alerting config; signals are the raw "we noticed X". [SOAR-5] Sentinel alerts, Splunk notable events. [SOAR-4][SOAR-17]
- **Recommendation ≈ proposed action awaiting approval.** StackAI **propose→approve** pattern: "agents suggest actions but humans make final decisions." [SOAR-20] Tines HITL checkpoint that assembles full context + presents a **recommended response level** + approve/deny/escalate options. [SOAR-14]
- **Action ≈ executed change.** XSOAR/Splunk SOAR connector tasks that mutate target systems; gated by manual tasks / Prompt blocks. [SOAR-18][SOAR-19]

Prism's **C10 GAP-Q2 Query Workers explicitly do NOT auto-act** (findings + recommendations only). This maps cleanly: Query Workers operate in the **Observation + Recommendation** tiers and never the **Action** tier — which is precisely the conservative, defensible default the industry guidance (NIST IR 8596 draft, CISA agentic-AI) endorses for AI-driven decisioning. [SOAR-14]

### Proposed clean ARO taxonomy for Prism

| Tier | Definition | Mutates external state? | AI agent may emit autonomously? | Audit weight |
|---|---|---|---|---|
| **Observation** | A factual finding derived from federated query data (OCSF-normalized). "Host X had 412 failed logins in 10m." Immutable, evidence-backed. | No | **Yes** (Query Workers, S3 embedded agent) | Evidence package (C10 GAP-Q2) |
| **Recommendation** | A *proposed* Action + rationale + blast-radius estimate + reversibility class. Not executed. "Recommend: isolate host X (reversible, blast=1 host)." | No | **Yes** (Query Workers, agent) | Recommendation record links to Observations |
| **Action** | An executed write against a sensor API (feature-flagged write capability). Always traces to an approved Recommendation (or an explicitly autonomous policy-permitted class). | **Yes** | **Only** if (a) action class is in an explicit auto-act allowlist AND (b) reversibility/confidence/asset gates pass — otherwise requires human approval | Full action audit: who/what/when/approver/result/rollback handle |

**Design principles drawn from the research:**
1. **Observations and Recommendations are AI-safe; Actions are gated by default.** Mirrors Query-Workers-do-not-auto-act + Tines reversibility axis + NIST/CISA. [SOAR-14]
2. **Every Action references the Recommendation(s) and Observation(s) it derives from** — a provenance chain, analogous to Splunk's discrete-action + event-chaining discipline. [SOAR-2] This is the natural extension of C10's evidence package into the act tier.
3. **Recommendation carries machine-readable metadata** so the orchestration layer can apply Tines' 5 axes deterministically: `reversibility ∈ {reversible, compensable, irreversible}`, `blast_radius`, `confidence`, `novelty`, `asset_tier`, `compliance_flags`. [SOAR-14]
4. **The model is a state machine:** Observation → (optional) Recommendation → (gated) Action → Result/Rollback. Each transition is an audited event.

---

## Q3 — SOAR Platform Architecture (what Prism needs to be a credible SOAR)

Synthesizing the platform survey, a credible SOAR built on Prism's existing connector framework + C2 satellite needs these components. **Strong recommendation: do NOT embed actions in PrismQL grammar; build a separate playbook/orchestration layer** — this is the near-universal industry pattern and even Splunk discourages the inline alternative. [SOAR-2][SOAR-16][SOAR-19]

| Component | What it is | Prism mapping / what to build | Source basis |
|---|---|---|---|
| **Playbook / workflow engine** | DAG of tasks: query → condition → action → loop, with manual/conditional tasks | New `prism-orchestration`-style subsystem (analogous to existing prism-operations scheduler/action-delivery, ADR-022 §D 8/8 split). Tasks reference PrismQL queries as inputs, not embed actions in PrismQL. | [SOAR-16][SOAR-19] |
| **Connectors-as-actions** | Reusable, parameterized action plugins with declared inputs/outputs | Extend the existing **spec-driven sensor adapter framework + feature-flagged write capabilities** — a write capability becomes an "Action connector". Use Splunk's Common Action Model discipline: discrete, single-purpose, declared schema. | [SOAR-2][SOAR-13] |
| **Approval gates (HITL)** | Pause-and-wait checkpoint, role-assigned, SLA-bound, audited | A "Prompt"-equivalent task: assign approver role (C18 RBAC), required-response questions, response-time SLA, escalation/re-notify loop, immutable record of what approver saw + decided. | [SOAR-14][SOAR-18] |
| **Case management** | Group observations/recommendations/actions into investigations | Case object linking ARO records + evidence packages (C10). | [SOAR-16][SOAR-19] |
| **Audit** | Immutable trail: what was seen, decided, when, by whom; action results | RocksDB-backed audit domain (existing storage CFs); every ARO transition emits a Canonical Structured Event (BC-2.16.002 catalog). | [SOAR-14][SOAR-18] |
| **Rollback / undo** | Compensating action handle for reversible/compensable actions | Recommendation declares `reversibility` + (where compensable) a paired compensating Action connector. Not auto-provided by any surveyed platform — Prism would differentiate by making it first-class. | [SOAR-2] (gap noted) |
| **Blast-radius control** | Cap scope of an action before it runs | Pre-flight: count affected entities (a read query) and enforce a max threshold per action class, analogous to Panther thresholds. | [SOAR-5][SOAR-14] |
| **Dry-run** | Simulate the action without mutating | Action connectors expose a `dry_run` mode that returns the would-be effect; default for novel/irreversible classes. Not standard in surveyed platforms — a Prism differentiator. | (gap noted) |

---

## Q4 — Write-Action Safety (gating Prism's feature-flagged sensor writes)

Prism's connectors can already write to sensor APIs behind the feature-flag system. To gate **destructive** actions, layer these controls (each maps to a project anchor):

1. **Feature-flag as the master switch** (existing): write capability off by default per sensor/tenant. An Action connector cannot exist for a capability whose write flag is disabled.
2. **RBAC (C18):** approver role + executor role separation. Splunk precedent: credentialed actions require specific capabilities (`list_storage_passwords`). [SOAR-17] Map to C18 roles; **two-person control** = require an approver role distinct from the requesting agent/analyst (two sequential approval tasks). [SOAR-14][SOAR-18]
3. **Approval workflow** (Q3): irreversible/high-blast/low-confidence/novel actions → mandatory HITL Prompt-equivalent with SLA. Reversible high-confidence actions → policy-permitted auto-act allowlist. [SOAR-14]
4. **Dry-run + blast-radius pre-flight:** default dry-run for irreversible classes; enforce affected-entity caps. [SOAR-5]
5. **Idempotency keys:** every Action carries a client-generated idempotency key so retries (network partition, satellite reconnect) don't double-apply. (Implementation discipline — no surveyed platform mandates it; Prism should, given C2 edge/offline queuing in Q6.) [SOAR-2] (gap)
6. **AD-017 credential safety:** action execution uses the **reference-based AI-opaque credential model** — credential *values* never transit AI context (project memory `project_ai_opaque_credentials.md`). The agent emits a Recommendation referencing a credential *handle*; the orchestration layer (not the AI) resolves it at execution time. This is a clean fit: the AI never holds write creds.
7. **C16 masking:** Observation/Recommendation payloads surfaced to the AI are masked; the Action tier (post-approval, in the orchestration layer) operates on unmasked data outside AI context.
8. **Audit (non-negotiable):** every Action emits a structured event (BC-2.16.002 catalog row) with full provenance chain (Q2 principle 2).

---

## Q5 — On-Prem / Air-Gapped Models (version- and license-verified, mid-2026)

All licenses/sizes below are from official model cards / announcements; flagged where deep-research uncertainty was cross-checked. **Bias for Prism:** Apache-2.0 / MIT models are the cleanest fit for **air-gapped, per-tenant, commercial** use (no field-of-use restrictions, no downstream-propagation obligations). [MOD-4][MOD-13][MOD-16][MOD-18][MOD-19]

### General reasoning LLMs (the agent "brain")

| Family | Sizes (open weight) | License | Context | Air-gap fit | Source |
|---|---|---|---|---|---|
| **Qwen3** (Alibaba) | Dense **0.6B/1.7B/4B/8B/14B/32B**; MoE **30B-A3B**, **235B-A22B** | **Apache-2.0** (all open-weight, per Qwen tech report + GitHub + blog; one secondary source contests 235B → resolved in favor of official Apache-2.0) | long-context | **Best** — Apache-2.0 across the board, **Thinking/Non-Thinking modes** (`/think` `/no_think`) ideal for tunable action-reasoning effort | [VERIFY-Qwen3]; [MOD-4][MOD-13] (Qwen2.5 baseline) |
| **Qwen2.5** (Alibaba) | 0.5B/1.5B/3B/7B/14B/32B/72B + Coder + Math | Apache-2.0 **except 3B & 72B** (consult those license files) | 128K in, 8K out, 29+ langs, strong JSON/structured output | Strong; avoid 3B/72B for clean air-gap unless license verified | [MOD-4][MOD-13] |
| **Mistral 3 / Ministral 3** | Dense **3B/8B/14B**; **Large 3** MoE (41B active / 675B total) | **Apache-2.0** (incl. base + instruct + reasoning variants, image understanding) | — | **Best** — fully Apache-2.0, edge sizes (3B/8B) for satellites | [MOD-16][MOD-6] |
| **Mistral NeMo** | 12B | open-weight (confirm card) | 128K | Strong mid-size; NVIDIA NIM packaging | [MOD-7] |
| **OpenAI gpt-oss** | **20B** (3.6B active), **120B** (5.1B active), MoE | **Apache-2.0 + gpt-oss usage policy** (per model card; second query couldn't independently re-confirm Apache-2.0 but the model card explicitly states it) | o200k_harmony tokenizer (~200K) | Strong — designed for agentic tool-use + structured outputs + configurable reasoning effort; usage policy is a contractual overlay | [MOD-18]; [VERIFY-gptoss] (existence + sizes re-confirmed) |
| **Phi-4-mini / Phi-3** (Microsoft) | Phi-4-mini **3.8B**; Phi-3 mini 3.8B / small 7B / medium 14B; Phi-4-multimodal | **MIT** | Phi-4-mini **128K**; Phi-3-mini 8K (128K long-ctx variant) | **Best for edge** — MIT, 4-bit Phi-3-mini ≈ **1.8GB** (phones/secure laptops), function-calling built in | [MOD-8][MOD-9][MOD-19] |
| **Gemma 4** (Google) | small 2B/4B; dense **31B**; MoE **26B-A4B**; unified **12B** multimodal | **Gemma Terms of Use** (open-weight, commercial OK, **but must propagate §3.2 use restrictions to downstream agreements**) | 128K (small) / **256K** (medium) | Good capability, **license overlay heavier** than Apache/MIT — extra care for customer-facing products | [MOD-11][MOD-17] |
| **Llama 3.1 / 3.x / 4** (Meta) | 3.1: **8B/70B/405B**; Llama 4 **Scout** (109B total/16 experts/~17B active, up to **10M ctx**), **Maverick** (~400B total/128 experts/~17B active) | **Llama Community License** (commercial OK, field-of-use restrictions + no-train-competing-model clauses) | 3.1 up to 128K; Llama 4 Scout up to 10M | Capable but **heaviest license** for per-tenant air-gap; verify permitted fields | [MOD-1][MOD-2][MOD-3][MOD-10][MOD-12] |

> **Llama 4 caveat:** Scout/Maverick parameter + context figures come from a secondary comparison source [MOD-10]; the verification query could **not** independently confirm the Scout/Maverick/Behemoth naming or the 10M-token figure against official Meta docs. **Treat Llama 4 specifics as provisional** pending official model-card confirmation.

### Small task-specific models (guardrails, classification, retrieval)

The layered pattern — general LLM "brain" + constellation of small validated models — is the recommended security architecture; small models are easier to calibrate and act as guardrails. [MOD-19][MOD-20][MOD-6]

- **Prompt-injection / jailbreak detection:** **Meta Llama Prompt Guard 2** (~tens of millions of params, e.g. the 86M variant) — front-gate prompts to the reasoning LLM. Critical given Prism's prompt-injection-defense mandate (project memory `project_agent_harness_design.md`). [MOD-20]
- **Content moderation:** **Mistral Moderation 2** (128K ctx, jailbreak detection). [MOD-6]
- **Embeddings / retrieval (RAG over OCSF data):** **Mistral Embed**, **Codestral Embed** (code), Qwen embedding/reranking. [MOD-6][MOD-5]
- **Classification/scoring (e.g. severity, reversibility class):** small dense models (Qwen3 0.6B/1.7B, Phi-3-mini, Ministral 3B) quantized — fast, validatable, cheap, fit satellite CPUs.

### Runtimes (verified versions, June 2026) — direct C7 ModelBackend fit

C7 chose **candle + ort + wasmtime + tract**. Verified current versions:

| Runtime | Verified version (Jun 2026) | Role for Prism | Source |
|---|---|---|---|
| **candle** (`candle-core`) | **0.10.2** (very recent) | Rust-native, memory-safe transformer inference on CPU/GPU; loads HF formats. Primary for Rust-embedded reasoning LLMs. | [VERIFY-crates] |
| **ort** (ONNX Runtime Rust, pykeio) | stable **1.15.1** (wraps ONNX Runtime 1.24); latest **2.0.0-rc.10** (Jun 5 2025) | ONNX-converted task models (classifiers, embeddings, Prompt Guard) across CPU/GPU accelerators. **Note: 2.x still RC** — pin deliberately. | [VERIFY-crates] |
| **tract** (`tract-onnx`) | **0.23.0** (Jun 1 2026) | Tiny, self-contained, no-nonsense ONNX inference — ideal for **small task models on satellites/edge** (CPU-only, minimal deps). | [VERIFY-tract] |
| **wasmtime** | **35.0.0** (~mid-2026; includes `wasmtime-wasi-nn` 35.0.0) | Sandboxed plugin execution; **`wasi-nn`** lets sandboxed Wasm modules run ML inference (load/set_input/compute/get_output) via host backend (OpenVINO today; experimental ONNX) **while staying in the sandbox**. Strong fit for AI-opaque + per-tenant isolation: model runs host-side, module can't escape. | [VERIFY-wasinn][SOAR-wasinn-bca] |

Non-Rust runtimes for heavier/server workloads (informational; not C7's pick but relevant for the central tier): **llama.cpp** (GGUF, CPU/consumer-GPU, full-load-into-RAM), **vLLM** (Python, high-throughput GPU serving, AWQ/GPTQ/FP8/INT4/INT8), **Ollama** (easy offline packaging, 2–5GB footprints), **TGI** (production gRPC/OpenAPI server). [MOD-14][MOD-15]

### Quantization & footprint anchors (for satellite/edge sizing)
- **GGUF** (llama.cpp/LM Studio), **AWQ/GPTQ/INT4/INT8/FP8** (vLLM). [MOD-14][MOD-15]
- **Phi-3-mini 4-bit ≈ 1.8GB** (phone/edge-class). [MOD-19]
- **Gemma 4 examples:** E2B Q4_0 ≈ 2.9GB (mobile ≈ 0.84–1.1GB); E4B Q4_0 ≈ 4.5GB; 31B-dense Q4_0 ≈ 17.5GB (single high-end GPU). [MOD-11]

---

## Q6 — Edge / Satellite Action Execution (C2)

The research surface here is thinner (no surveyed SOAR documents multi-hop offline action queuing in depth), so this is largely a synthesis grounded in the queuing/idempotency gaps the survey *did* surface plus Prism's existing C2 architecture.

- **Central-decide, edge-execute split:** Decisioning (Observation→Recommendation, gating) can run centrally or in the **S3 server-hosted embedded agent**; **Action execution happens at the satellite (C2)** where the sensor API is reachable. The satellite holds an action queue.
- **Offline / air-gap queuing:** When a satellite is partitioned, approved Actions queue durably (RocksDB-backed) and execute on reconnect. This is precisely why **idempotency keys (Q4.5) are mandatory** — reconnect retries must not double-apply. No surveyed platform provides this natively; it's a Prism differentiator and a hard requirement given C2's offline reality.
- **Small task models at the edge:** satellites run **tract**-served quantized classifiers (severity/reversibility scoring) for local triage without central round-trips; the heavy reasoning LLM stays central. [VERIFY-tract][MOD-19]
- **Multi-hop:** approval chains may originate centrally, but the **execution credential resolution (AD-017) must occur at the executing tier**, not transit multiple hops in AI context.

---

## ANALYSIS + LEANS

### Recommended ARO model (lean)
Adopt the three-tier **Observation → Recommendation → Action** state machine from Q2. **Observations and Recommendations are AI-safe and may be emitted autonomously by Query Workers / the S3 agent (matching C10 GAP-Q2's no-auto-act stance).** Actions mutate state, are gated by default, and always carry a provenance chain back to the Recommendation(s)/Observation(s). Recommendations carry machine-readable Tines-style metadata (reversibility, blast_radius, confidence, novelty, asset_tier, compliance_flags) so gating is deterministic. [SOAR-14][SOAR-20]

### PrismQL action grammar sketch (lean: DO NOT embed mutations in the query grammar)
The near-universal industry pattern — and Splunk's own production advice — is that the **query language stays declarative/read-only and actions live in a separate orchestration layer.** [SOAR-2][SOAR-16][SOAR-19] PrismQL should at most gain a **read-only "recommend" projection** (a query can *propose* an Action as data — an ARO Recommendation row), never an inline mutating operator like Splunk's `sendalert`. Sketch:

```
-- Query stays read-only; emits Observations + (optionally) a Recommendation as DATA, never an executed Action.
FROM detections
WHERE failed_logins > 50 BY src_host
EMIT OBSERVATION "excessive_failed_logins"
RECOMMEND ACTION isolate_host(host = src_host)
  WITH reversibility = compensable, blast_radius = count(src_host), confidence = 0.9
-- Execution happens ONLY in the orchestration layer after gating. PrismQL never executes the write.
```

This keeps PrismQL's DataFusion planner pure/read-only (preserving the security perimeter that `tests/external/perimeter-violation/` enforces), and isolates all mutation + approval + audit in a new orchestration subsystem.

### SOAR architecture (lean)
Build a **separate `prism-orchestration` playbook engine** (analogous to the existing prism-operations scheduler/action-delivery, ADR-022 §D) with: connectors-as-actions (= feature-flagged sensor write capabilities wrapped as discrete Common-Action-Model-style plugins), HITL Prompt-equivalent approval tasks (role-assigned, SLA-bound, escalating), case management, immutable audit (BC-2.16.002 catalog), first-class **rollback** (compensating actions) and **dry-run** + **blast-radius caps** (Prism differentiators — the surveyed platforms leave these to authors). [SOAR-2][SOAR-5][SOAR-14][SOAR-18]

### On-prem model recommendations (lean)
- **Reasoning brain (central / S3 agent):** **Qwen3** (8B–32B dense, or 30B-A3B MoE) on **candle** — Apache-2.0, Thinking-mode for tunable action-reasoning, cleanest air-gap/per-tenant license. Alternative: **Mistral 3 / Large 3** (also Apache-2.0). gpt-oss-20B is a strong agentic-tool-use option (Apache-2.0 + usage policy). [VERIFY-Qwen3][MOD-16][MOD-18]
- **Edge / satellite (C2):** **Phi-4-mini (MIT, 3.8B, 128K, function-calling)** or **Ministral 3 3B/8B (Apache-2.0)** on **tract**/**candle**, 4-bit quantized (~1.8–4.5GB) — CPU-only viable. [MOD-8][MOD-9][MOD-19][MOD-16]
- **Guardrails (everywhere):** **Llama Prompt Guard 2** (prompt-injection front-gate) + **Mistral Moderation 2** via **ort**/**tract** — non-negotiable given the prompt-injection-defense mandate. [MOD-20][MOD-6]
- **Embeddings (RAG over OCSF):** **Mistral Embed / Codestral Embed** via ort. [MOD-6]
- **Avoid for clean air-gap unless license re-verified:** Qwen2.5 3B/72B (non-Apache), Gemma (Terms-of-Use propagation burden), Llama 3.x/4 (Community License field-of-use restrictions). [MOD-4][MOD-11][MOD-17][MOD-1]
- **wasmtime wasi-nn** is the strongest fit for AI-opaque + per-tenant isolation: the sandboxed module requests inference; the host backend executes it; the module can't escape the sandbox. [VERIFY-wasinn][SOAR-wasinn-bca]

### Edge-action execution (lean)
Central/S3 decisions, **satellite (C2) execution** with durable RocksDB-backed offline action queues, **mandatory idempotency keys** for reconnect-safe retries, AD-017 credential resolution at the executing tier (never multi-hop in AI context), and tract-served local triage classifiers.

### Genuine sub-forks needing a HUMAN decision
1. **Auto-act vs. recommend-only default.** Leaning **recommend-only by default** (matches C10 Query Workers, NIST IR 8596 draft, CISA agentic-AI guidance). Auto-act, if ever enabled, should be a narrow per-action-class allowlist gated by reversibility+confidence+asset tiers. **Human must ratify whether ANY autonomous action class is permitted in v1, or whether v1 is strictly recommend-only.** [SOAR-14]
2. **Does PrismQL get a read-only `RECOMMEND ACTION` projection at all in v1, or do Recommendations come purely from the agent layer (keeping PrismQL grammar untouched)?** Architectural fork affecting grammar scope + perimeter tests.
3. **Which models ship by default** (Apache-2.0/MIT lean is clear, but the *specific* default brain — Qwen3 vs Mistral 3 vs gpt-oss — and whether to bundle weights or require operator-supplied weights for air-gap) is a product + licensing call.
4. **Approval-workflow design:** two-person control mandatory for which action classes? SLA + escalation policy? This is an org-policy + C18 RBAC design decision.
5. **Rollback/dry-run scope:** first-class compensating-action framework is a Prism differentiator but a non-trivial build — human prioritization call on whether v1 includes it or defers (with explicit future-story anchor per the production-grade default).
6. **Llama 4 specifics unverified** — if Meta models are in scope, a human/architect must confirm Scout/Maverick details against official model cards before relying on them.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Deep multi-source sweep of (a) how query/detection/SOAR platforms express actions + safety patterns (Splunk, Sigma, Sentinel, Panther, Tines, Shuffle, n8n, Torq, XSOAR, Splunk SOAR); (b) on-prem/air-gap open-weight models + runtimes + licenses + quantization. `reasoning_effort=high`. |
| Perplexity perplexity_ask | 3 | Version verification (candle/ort/tract/wasmtime/chumsky); Qwen3/Llama4/gpt-oss confirmation; chumsky status + wasmtime wasi-nn capability. |
| Perplexity perplexity_search | 1 | Raw crates.io version ranking for tract-onnx/wasmtime/chumsky. |
| Context7 | 0 | Not used — questions were ecosystem/platform-level, not single-library API docs. |
| Tavily (all) | 0 | Not needed — Perplexity sources were sufficient and self-citing. |
| WebFetch / WebSearch | 0 | — |
| Training data | 2 areas | Hunters.ai (no official docs surfaced — flagged inline as background only, uncited); general SOAR/AI-architecture framing. Flagged explicitly. |

**Total MCP tool calls:** 6 (2 deep research + 3 ask + 1 search)
**Training data reliance:** low — every non-obvious claim is cited; the two training-data areas (Hunters, general framing) are explicitly flagged and load-bearing claims were version/source-verified. Llama 4 specifics explicitly marked provisional due to inability to confirm against official Meta docs.

---

## Sources

### SOAR / actions-in-query-languages
- [SOAR-1] Splunk Community — pass field value to custom alert action (`sendalert`): https://community.splunk.com/t5/Splunk-Enterprise-Security/How-to-pass-field-value-to-custom-alert-action/m-p/381752
- [SOAR-2] Splunk Dev — Adaptive Response Framework: https://dev.splunk.com/enterprise/docs/devtools/enterprisesecurity/adaptiveresponseframework/
- [SOAR-3] SigmaHQ — Sigma correlation rules specification: https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-correlation-rules-specification.md
- [SOAR-4] Microsoft — create analytics rules (Sentinel KQL): https://learn.microsoft.com/en-us/azure/sentinel/create-analytics-rules
- [SOAR-5] Panther — detections (signals/alerts/thresholds): https://docs.panther.com/detections
- [SOAR-7] Tines — no-code automation for security teams: https://www.tines.com/playbooks/no-code-automation-for-security-teams/
- [SOAR-8] Shuffle (open-source SOAR): https://shuffler.io
- [SOAR-9] n8n: https://n8n.io
- [SOAR-10] Splunk — alert_actions.conf spec (v9.1.0): https://help.splunk.com/en/data-management/splunk-enterprise-admin-manual/9.1/configuration-file-reference/9.1.0-configuration-file-reference/alert_actions.conf
- [SOAR-11] Microsoft — automation rules (Sentinel): https://learn.microsoft.com/en-us/azure/sentinel/create-manage-use-automation-rules
- [SOAR-12] Microsoft — Logic Apps playbooks (Sentinel): https://learn.microsoft.com/en-us/azure/sentinel/automation/logic-apps-playbooks
- [SOAR-13] Panther — custom webhooks integration: https://panther.com/integrations/custom-webhooks
- [SOAR-14] Tines — human-in-the-loop workflows (NIST IR 8596 / CISA refs): https://www.tines.com/blog/human-in-the-loop-workflows-where-intelligent-automation-meets-judgment/
- [SOAR-15] Torq: https://torq.io
- [SOAR-16] Cortex XSOAR — Playbook Development guide: https://docs-cortex.paloaltonetworks.com/r/Cortex-XSOAR/6.x/Cortex-XSOAR-Playbook-Design-Guide/Playbook-Development
- [SOAR-17] Splunk ES 7.x — set up adaptive response actions: https://help.splunk.com/en/splunk-enterprise-security-7/administer/7.2/correlation-searches/set-up-adaptive-response-actions-in-splunk-enterprise-security
- [SOAR-18] Splunk SOAR Cloud — Prompt block (HITL): https://help.splunk.com/en/splunk-soar/soar-cloud/build-playbooks/use-the-playbook-editor-to-create-and-view-playbooks-to-automate-analyst-workflows/require-user-input-using-the-prompt-block-in-your-splunk-soar-cloud-playbook
- [SOAR-19] Cortex XSOAR — playbooks overview: https://xsoar.pan.dev/docs/playbooks/playbooks-overview
- [SOAR-20] StackAI — human-in-the-loop AI agents / approval workflows: https://www.stackai.com/insights/human-in-the-loop-ai-agents-how-to-design-approval-workflows-for-safe-and-scalable-automation
- [SOAR-wasinn-bca] Bytecode Alliance — Using wasi-nn in Wasmtime: https://bytecodealliance.org/articles/using-wasi-nn-in-wasmtime

### Models / runtimes / licenses
- [MOD-1] Meta — Llama 3.1 license: https://developer.meta.com/ai/llama3_1/license/
- [MOD-2] Meta — Llama 3.2 license: https://developer.meta.com/ai/llama3_2/license/
- [MOD-3] Meta — Llama 4 license: https://developer.meta.com/ai/llama4/license/
- [MOD-4] Alibaba — Qwen2.5 announcement (sizes/license): https://www.alibabacloud.com/blog/qwen2-5-a-party-of-foundation-models_601782
- [MOD-5] Alibaba Cloud — Model Studio models (embeddings/reranking): https://www.alibabacloud.com/help/en/model-studio/models
- [MOD-6] Mistral — models overview (generalist/specialist, Moderation 2, Embed): https://docs.mistral.ai/models/overview
- [MOD-7] Mistral — NeMo (12B, 128K): https://mistral.ai/news/mistral-nemo/
- [MOD-8] Microsoft Azure — Phi family (MIT): https://azure.microsoft.com/en-us/products/phi
- [MOD-9] HuggingFace — Phi-4-mini-instruct card (MIT, 3.8B, 128K): https://huggingface.co/microsoft/Phi-4-mini-instruct
- [MOD-10] gpt-trainer — Llama 4 evolution/features comparison (Scout/Maverick figures — SECONDARY, provisional): https://gpt-trainer.com/blog/llama+4+evolution+features+comparison
- [MOD-11] Google — Gemma 4 core docs (sizes/context/memory/quant): https://ai.google.dev/gemma/docs/core
- [MOD-12] HuggingFace — Llama-3.1-8B-Instruct card: https://huggingface.co/meta-llama/Llama-3.1-8B-Instruct
- [MOD-13] HuggingFace — Qwen2.5-7B-Instruct card (Apache-2.0, 128K): https://huggingface.co/Qwen/Qwen2.5-7B-Instruct
- [MOD-14] vLLM — quantization docs: https://docs.vllm.ai/en/latest/features/quantization/
- [MOD-15] llama.cpp — quantize README (GGUF): https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md
- [MOD-16] Mistral — Mistral 3 announcement (Apache-2.0, Large 3 MoE 41B/675B): https://mistral.ai/news/mistral-3/
- [MOD-17] Google — Gemma Terms of Use (§3.2 propagation): https://ai.google.dev/gemma/terms
- [MOD-18] OpenAI gpt-oss — model card (arXiv, Apache-2.0 + usage policy, 20B/120B MoE): https://arxiv.org/pdf/2508.10925.pdf
- [MOD-19] Microsoft — Phi-3 technical report (4-bit mini ≈1.8GB): https://arxiv.org/html/2404.14219v3
- [MOD-20] Meta — Llama Prompt Guard 2 (86M) card: https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-86M

### Version verification (crates / model facts, June 2026)
- [VERIFY-crates] crates.io: candle-core 0.10.2 (https://crates.io/crates/candle-core); ort 1.15.1 stable / 2.0.0-rc.10 (https://crates.io/crates/ort, https://github.com/pykeio/ort)
- [VERIFY-tract] lib.rs — tract-onnx 0.23.0 (Jun 1 2026): https://lib.rs/crates/tract-onnx
- [VERIFY-wasinn] crates.io wasmtime-wasi-nn 35.0.0 + chumsky 0.13.0 (1.0 still alpha): https://crates.io/crates/wasmtime-wasi-nn, https://crates.io/crates/chumsky
- [VERIFY-Qwen3] Qwen3 official (Apache-2.0, dense+MoE 0.6B–235B, Thinking/Non-Thinking): https://qwenlm.github.io/blog/qwen3/, https://github.com/qwenLM/qwen3, https://arxiv.org/html/2505.09388v1
- [VERIFY-gptoss] gpt-oss 20B/120B existence + sizes (deeplearning.ai batch): https://www.deeplearning.ai/the-batch/alibabas-latest-flagship-models-are-open-weights-moe-performers-in-sizes-from-less-than-1b-parameters
