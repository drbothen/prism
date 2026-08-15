---
document_type: proposal
status: draft-for-review
producer: architect
date: "2026-08-15"
version: "1.0"
level: L4
phase: architecture
section: query-agent-ergonomics-proposal
timestamp: "2026-08-15T00:00:00Z"
traces_to: ARCH-INDEX.md
inputs: []
input-hash: "d41d8cd"
related_adrs: [ADR-058, ADR-028, ADR-024, ADR-046]
related_bcs: [BC-2.11.001, BC-2.11.016, BC-2.16.003]
related_stories: [S-ADR058-OCSF-ROUTING-001, S-ADR058-OCSF-COERCION-001]
motivation_evidence: "D-2166 / D-2167 — DEFECT-ADAPTER-TLS-XDOME-LIVE-001 live AC-LIVE-001 validation session, 2026-08-15"
---

> **PROPOSAL — NOT ACCEPTED — for human review; no implementation authorized.**
> This document is a design investigation only. It does not modify any behavioral contract,
> ADR, story spec, grammar, or code. All options described are design sketches for human
> evaluation. Implementation authorization requires explicit human approval and a story spec
> authored by the product-owner specialist.

---

# PROPOSAL: LLM-Agent Query Ergonomics — OCSF Naming and Dot-Notation Aliasing

## [Section Content]

This proposal investigates LLM-agent query ergonomics friction observed in live Claroty xDome
demos, specifically the gap between agent training-data identifier priors (BigQuery/SparkSQL
conventions and OCSF-canonical field names) and the PrismQL query surface (flat
`sensor_table` names, vendor-spec or OCSF-flattened column names). See sections 1-8 below.

## 1. Problem Statement

### 1.1 Observed Friction (D-2166)

During the authenticated live Claroty xDome demo (AC-LIVE-001, 2026-08-15), the SOC-analyst
LLM agent required **3 attempts** to produce a working PrismQL query:

| Attempt | Query fragment | Error | Agent behavior |
|---------|----------------|-------|----------------|
| 1 | `FROM claroty.devices` | E-QUERY-037 — table not available; "Did you mean 'claroty_devices'?" | Self-corrected table notation |
| 2 | `SELECT device_id, category, os_type FROM claroty_devices` | E-QUERY-038 — columns not found; listed `uid, device_category, device_type, ...` | Self-corrected column names |
| 3 | `SELECT uid, device_category, device_type FROM claroty_devices` | SUCCESS | — |

The 3-attempt sequence added an observable latency of roughly one full LLM round-trip per
failed attempt before useful results appeared. No data was lost; the agent self-corrected
within the ≤3 retry budget specified in `query_tutorial` Step 3. This is **agent-ergonomics
friction, not a product defect**.

### 1.2 Root Cause: Identifier Mismatch, Not SQL Syntax

Both failures were wrong **identifiers** — not wrong SQL syntax, operators, or grammar:

1. **Dot-notation table name** (`claroty.devices`): The agent applied the BigQuery/SparkSQL
   convention of `sensor.table`. Prism uses flat `sensor_table` (underscore-separated).
2. **OCSF-guessed column names** (`device_id`, `category`, `os_type`): The agent inferred
   column names from training-data priors — likely from OCSF reference schemas or BigQuery
   OCSF-export conventions — rather than reading schema from `prism_describe`. The columns
   it guessed are plausible OCSF names (`device.id`, `category_name`, `os.type`) but do not
   match the Claroty TOML spec column names (`uid`, `device_category`, `device_type`).

Neither failure is surprising: LLM agents trained on public SQL examples will exhibit
BigQuery/SparkSQL table-naming conventions, and an agent reasoning about a "Claroty devices"
table will project OCSF Device object field names from its training data.

---

## 2. OCSF Fidelity Sub-Question: Are Claroty Columns OCSF-Normalized?

> This section answers a specific research question required before any aliasing option can
> be evaluated: **are Claroty columns already OCSF-normalized at the query surface, or are
> they vendor-raw?**

### 2.1 Finding: Currently Vendor-Raw; OCSF Transition In-Flight

**Current production state (pre-ADR-058 Stage 2):** The query surface uses the TOML `col.name`
values directly as Arrow RecordBatch field identifiers. For Claroty devices, these are:
`uid`, `asset_id`, `device_category`, `device_type`, `risk_score`, `retired`, `ip_list`,
`mac_list`, `network_list`, `vlan_list`, `purdue_level`, `site_name`, `device_subcategory`,
`device_type_family`, `criticality`, `is_online`, `device_name`, `manufacturer`, `model`,
`os_category`.

These are **Claroty xDome API field names** — the same field names used in the `body_template`
POST body (`{"fields": ["uid", "asset_id", ...]}`). They are NOT OCSF field names. The TOML
DOES annotate each column with an `ocsf_field` path (`uid → "device.uid"`, `device_category →
"device.type_category"`, etc.) but these annotations currently only appear as the `description`
field in `prism_describe` output. They have no effect on query column names.

This is a known gap: BC-2.01.013 EC-01-025 marks the ColumnMapper wiring as **NON-CONFORMANT**
— the `column_mapping.rs` mapper was implemented but its output path back to
`pipeline_result_to_record_batch` in `prism-bin` was never wired, so `ocsf_field` declarations
have had no effect on emitted row shapes.

**ADR-058 v2.0 (ACCEPTED, 2026-08-12)** resolves this with an explicit architectural decision:
Interpretation A — OCSF field paths are underscore-flattened and used as Arrow RecordBatch field
names, enabled per-sensor via a TOML flag `ocsf_column_naming = true`. Claroty is the first
target. The two delivery stories are:

| Story | Scope | Status |
|-------|-------|--------|
| S-ADR058-OCSF-COERCION-001 | Stage 1 — coercion integration (`build_column_array` fix, `column_coercion_failure` emission) | `draft` |
| S-ADR058-OCSF-ROUTING-001 | Stage 2 — OCSF routing wiring (`ocsf_column_naming` TOML flag, `pipeline_result_to_record_batch` update, `prism_describe` column name update) | `draft` |

**After ADR-058 Stage 2 ships**, the Claroty query surface will expose OCSF-flattened underscore
names. The devices table will present as:

| Current col.name | ocsf_field | Post-Stage-2 query name |
|-----------------|-----------|------------------------|
| `uid` | `device.uid` | `device_uid` |
| `asset_id` | `device.instance_uid` | `device_instance_uid` |
| `device_category` | `device.type_category` | `device_type_category` |
| `device_type` | `device.type_name` | `device_type_name` |
| `risk_score` | `risk_score` | `risk_score` |
| `retired` | `status_code` | `status_code` |

Columns without `ocsf_field` (`ip_list`, `mac_list`, `vlan_list`, etc.) go to a queryable
`raw_extensions` JSON column per ADR-058 §I2.

### 2.2 Human Attention: Tension with OCSF Normalization Vision

The project memory (`project_core_architecture_insight.md`) states: "sensor adapters emit OCSF+protobuf
shapes per the project vision." The `column_mapping.rs` module header (BC-2.16.003) confirms
this intent. The gap between vision and current implementation was an oversight (ADR-058 §A2
traces its origin). ADR-058 v2.0 is the remediation plan.

**The tension exists and the human should be aware of it as context for this proposal.** Any
design option that builds on top of the current vendor-raw column names (e.g., a manual alias
table pointing `device_id → uid`) will need to be reworked after ADR-058 Stage 2 ships, because
the authoritative column names will change. Options should be evaluated against the
**post-ADR-058-Stage-2 state** as the forward-looking baseline.

### 2.3 Post-Stage-2 OCSF Names vs. Agent Training-Data Priors

Even after ADR-058 Stage 2, the Claroty column names will differ from what an LLM agent
typically guesses:

| Agent's training-data guess | Post-Stage-2 Claroty name | Gap |
|-----------------------------|--------------------------|-----|
| `device_id` | `device_uid` | Close but not equal |
| `category` | `device_type_category` | Different level of specificity |
| `os_type` | `os_category` (in `raw_extensions`) | Different name, different location |
| `risk_level` | `risk_score` | Reasonable match |
| `is_active` | `status_code` | Semantic gap (`status_code` is boolean-valued `retired` flag) |

The OCSF underscore-flattened names are OCSF-grounded (e.g., `device.uid` → `device_uid`) but
they do not necessarily match what an agent trained on generic OCSF documentation would predict,
because OCSF field paths are not widely present in training corpora at the specificity level that
prism uses them (`device.instance_uid`, `device.type_category`). Agents tend to reason at the
top-level object level (`device.id`, `device.type`) rather than at the exact dot-path level
(`device.uid`, `device.type_name`).

The OCSF fidelity sub-question answer is therefore: **currently vendor-raw; transitioning to
OCSF-flattened via ADR-058 Stage 2 (draft); OCSF-flattened names will be more predictable
than vendor-raw names but will still differ from training-data priors in specific fields.**
This is a material finding: OCSF aliasing (Option A) does not fully solve the ergonomics
problem even after ADR-058 ships.

---

## 3. Option Sketches

> **All options below are design sketches only. No implementation is authorized.**
> Trade-offs are assessed against the post-ADR-058-Stage-2 baseline (OCSF-flattened names).

### Option A: OCSF-Canonical Column Aliasing

**Summary:** Add a mapping layer so that an agent can use standard OCSF field path names
(or their underscore-flattened equivalents) as column names in PrismQL queries. For example,
`SELECT device_uid FROM claroty_devices` would succeed using the OCSF-flattened name AND
`SELECT uid FROM claroty_devices` would also succeed using the pre-flag col.name as a
backward-compatible alias.

**Where it would live:**
The alias layer could be implemented at one of three locations:
1. **Sensor TOML alias table** — a new `[[tables.columns.aliases]]` stanza that declares
   additional accepted names for a column. Processed in `TableRegistry` alongside the primary
   `col.name`. Simplest; no shared infrastructure needed.
2. **Query plan resolver** — during plan construction, the resolver maps unknown column names
   to registered names via a lookup table derived from `ocsf_field` declarations in the
   registered specs. More powerful (cross-sensor); higher blast radius.
3. **OCSF schema registry** — a bundled OCSF schema that maps OCSF class fields to registered
   sensor columns at runtime. Most powerful; not yet built; significant scope.

**Ambiguity / collision risks:**
- `status` is both an OCSF field name (`status`) and a Claroty alerts col.name (`status`) —
  no collision. But if two sensors declare different `ocsf_field = "status"` mappings to
  semantically different columns, the alias becomes sensor-specific and cannot be shared across
  a multi-sensor query. The multi-sensor case requires scoping the alias to `sensor.column`
  qualified form, which is the dot-notation problem again.
- ADR-058 §J (flag-transition name shadowing) already identified that flattened OCSF names can
  collide with existing col.names within the same table (the `device_category` / `device_type`
  collision). Any alias table must enforce the same shadow-free invariant.
- Backward-compatible aliases (where both the old col.name AND the new OCSF-flattened name are
  accepted) require maintaining two code paths indefinitely. This creates schema surface drift
  — `prism_describe` would need to document both names to avoid confusion.

**Maintenance cost:**
Per-sensor alias tables require review every time a TOML column is added or its `ocsf_field` is
amended (as happened with `device_category` in ADR-058 §J3). The OCSF schema registry approach
is fully automatic once built, but it requires the OCSF schema bundle to be kept up to date.

**Interaction with `prism_describe`:**
`prism_describe` currently returns `name: "<col.name>"` (pre-Stage-2) or
`name: "<ocsf_flattened_name>"` (post-Stage-2 with flag=true). If aliases are supported, either
`prism_describe` must list all aliases (increasing response verbosity) or the agent reads only
the canonical name and the alias is silently transparent. The latter leaves the agent without
knowledge of the alias capability, which defeats the discoverability goal.

**Assessment:** Medium-high complexity; deferred to after ADR-058 Stage 2 ships to avoid a
double-migration. The value increases after Stage 2 because OCSF-flattened names are the
authoritative names and the alias mechanism would add backward-compat for the old col.names.
Building this before Stage 2 inverts the dependency order.

---

### Option B: Dot-Notation Table Aliasing

**Summary:** Resolve `claroty.devices` as a transparent alias for `claroty_devices` in the FROM
clause, rather than returning E-QUERY-037.

**Current state:** E-QUERY-037 already handles dot-notation by detecting `SourceRefKind::External`
source refs and returning `TableNotAvailable { did_you_mean: " Did you mean: 'claroty_devices'?" }`.
The error response is well-formed and the agent self-corrected in a single attempt in the demo.
Dot-notation is detected in the `check_availability_gate` in `table_registry.rs` before any
fan-out occurs — it is a plan-time gate, not a runtime cost.

**Grammar / parser impact:**
The PrismQL Chumsky parser already parses `claroty.devices` as `SourceRefKind::External { sensor: "claroty", table: "devices" }`. The `check_availability_gate` then rejects it (EC-11-067,
BC-2.11.001 §HIGH-1 which removed an earlier SqlPipe exemption). Accepting it would require
changing the gate from rejection to transparent underscore-translation.

ADR-046 §§A–C establishes the three-mode correctness model (filter, SQL, pipe). Dot-notation
acceptance would change observable behavior in all three modes and would require a BC-2.11.001
amendment (the current BC §HIGH-1 explicitly forbids silent translation).

**Collision with qualified column refs:**
In standard SQL, `table.column` is a qualified column reference. DataFusion SQL uses `.` as the
qualifier separator. If `claroty.devices` is accepted as a table name, and an agent later writes
`claroty.devices.uid` as a qualified column reference, the parser would need to resolve the
ambiguity. The current grammar avoids this by rejecting dot-notation tables entirely.

**Why it may not be needed:**
The dot-notation failure in the demo was a 1-attempt fix: the agent got E-QUERY-037, read the
"Did you mean 'claroty_devices'?" suggestion, and corrected its table name. The E-QUERY-037
mechanism is working correctly. Unless demo evidence shows the agent repeatedly failing to
self-correct dot-notation (which the single-session sample does not show), the current
guidance-via-error is sufficient.

**Assessment:** Not recommended unless multiple demos confirm agents consistently fail to
self-correct dot-notation despite the "Did you mean?" suggestion. The single-session evidence
is insufficient to justify the BC amendment and grammar change required.

---

### Option C: Do Nothing (Rely on Existing Error-Guided Self-Correction)

**Summary:** The existing system already provides sufficient agent steering. No product change
is needed for the observed ergonomics friction.

**Evidence that the current approach works:**
- E-QUERY-037 returns a `did_you_mean` field with the correct underscore table name (implemented,
  tested, BC-2.11.001 §AC-3).
- E-QUERY-038 returns an `available_columns` field listing ALL valid column names for the table
  (BC-2.11.016 §EC-11-041).
- `prismql://reference` resource §3 says explicitly: "Column names come verbatim from
  `prism_describe` — do not construct dot-path names."
- `query_tutorial` prompt Step 1 says: "Call `prism_describe` with client_id='{client_id}' to
  discover which tables and columns are available before writing any query."
- The agent in the demo self-corrected within 3 attempts — matching the designed retry budget.
- The demo produced the correct answer. No data was lost or misrepresented.

**Why this may be sufficient for a SOC analyst harness:**
An agent that self-corrects in ≤3 attempts before succeeding is following the error-guided
self-correction loop that PrismQL was designed for. The 2 failed queries add roughly 2 LLM
API round-trips plus 2 near-zero-cost plan-time rejections. At typical LLM API latencies
(1–3s per call), this is a 2–6s overhead. For a SOC analyst session with multiple multi-second
queries, this is not a dominant cost.

**Why this may not be sufficient at scale:**
- A system prompt that mandates `prism_describe` FIRST (Option D) would eliminate both failures
  at zero product cost. The 2 failed queries were avoidable given what the system prompt already
  says — the agent simply did not read `prismql://reference` before its first attempt.
- If the demo is a rehearsed showcase to non-technical stakeholders, a 3-attempt sequence before
  a successful answer is a visible friction point regardless of correctness.
- Once ADR-058 Stage 2 ships and column names change (`uid → device_uid`), any existing LLM
  agent that was trained or conditioned on the pre-Stage-2 column names will face the same
  3-attempt pattern again unless proactively steered.

**Assessment:** Technically sufficient for the single observed session. Not robust to ADR-058
Stage 2 column-name churn or to agents that skip the `query_tutorial` / `prismql://reference`
pre-read.

---

### Option D: Stronger Up-Front Steering (Mandate prism_describe Before First Query)

**Summary:** Modify the harness system prompt or agent workflow to require the agent to call
`prism_describe` BEFORE writing any PQL query for a given table. This is a harness/prompt change
(zero product code change) but would eliminate both observed failure modes.

**Current state:**
`query_tutorial` Step 1 says "Call `prism_describe` ... before writing any query." The
`prismql://reference` §3 says the same. These are recommendations, not enforced constraints.
The demo agent did not read either resource before its first query.

**How it would work:**
The SOC-analyst agent harness (system prompt or tool-use loop) would include an explicit
constraint: before calling the `query` tool for a client/table pair for the first time in a
session, the agent MUST first call `prism_describe`. This can be framed as a workflow rule
rather than a product enforcement.

**Variants:**
- **Soft (recommended):** Rewrite `query_tutorial` Step 1 to be imperative: "MUST call
  `prism_describe` before any query." Add a similar note to `prismql://reference` §2 as the
  first bullet. No code change; text change only.
- **Hard (product-enforced):** The `query` MCP tool handler checks whether `prism_describe` has
  been called for the client_id in the current session. If not, it returns an advisory response
  (not an E-QUERY error) directing the agent to call `prism_describe` first. Requires session
  state tracking for the `query` handler.

**Minimal cost / immediate applicability:**
Option D (soft) is a documentation edit only, can be applied without any story or spec change,
and has zero blast radius. It would have eliminated both the dot-notation attempt AND the
OCSF-guessed column attempt in the demo.

**Limitation:**
This is a harness change, not a product capability. It does not help agents that bypass
`query_tutorial` and `prismql://reference`, or agents orchestrated by a third-party harness
that does not enforce the call order. The "hard" variant requires product work but is bounded.

**Assessment:** Recommended as a low-cost mitigation that can be applied immediately,
complementary to any product-level option. The soft variant requires only a text edit to
`prompts.rs` and `resources.rs`. The hard variant is a separate story scoped as a workflow
guardrail, not a query-language change.

---

## 4. Trade-Offs Matrix

| Dimension | Option A (OCSF aliasing) | Option B (dot-notation) | Option C (do nothing) | Option D (stronger steering) |
|-----------|--------------------------|------------------------|----------------------|------------------------------|
| Eliminates demo failure mode 1 (dot-notation) | No direct effect | Yes | No | Yes (soft) |
| Eliminates demo failure mode 2 (column names) | Partial (post-Stage-2 + residual gaps) | No | No | Yes (if pre-read enforced) |
| Requires grammar/BC change | No | Yes (BC-2.11.001 §HIGH-1 amendment) | No | No (soft) / No (hard) |
| Requires product code change | Yes (significant) | Yes (moderate) | No | No (soft) / Small (hard) |
| Works before ADR-058 Stage 2 | Needs rework after Stage-2 transition | Works now | Works now | Works now |
| Maintenance cost | High (alias tables per sensor, shadow checks) | Low (one grammar gate change) | None | Low |
| Risk of new confusion | Medium (two names for same column, schema drift) | Medium (SQL qualified-ref ambiguity) | None | Low |
| Applicable to all 4 sensors | Yes (per-sensor work) | Yes | N/A | Yes |

---

## 5. Recommendation

The architect's recommendation, **for human review and approval**:

**Short-term (apply immediately, zero product risk):** Option D soft — amend `query_tutorial`
Step 1 and `prismql://reference` §2 to use imperative language ("MUST call `prism_describe`
before any query for a new table"). This is a text edit to `prompts.rs` and `resources.rs`.
It directly addresses both observed failure modes without any grammar, BC, or schema change.
It should be treated as a maintenance edit, not a story.

**Medium-term (after ADR-058 Stage 2 ships):** Revisit Option A in the context of the new
OCSF-flattened column names. At that point, the authoritative column names will be stable and
well-defined. A backward-compat alias mechanism (accepting the old col.name AND the new
OCSF-flattened name) would smooth the transition for agents that learned the pre-Stage-2 names.
The TOML alias table variant (sub-option A1) is the lowest-complexity starting point; the query
plan resolver variant (A2) is better for multi-sensor scenarios.

**Not recommended now:** Option B (dot-notation acceptance) requires amending BC-2.11.001
§HIGH-1 which was deliberately written to REJECT silent translation. The existing error guidance
is working. Option C (do nothing) is technically correct but fragile to ADR-058 churn.

---

## 6. Open Questions for Human Review

1. **Stage 2 timing**: ADR-058 Stage 2 (S-ADR058-OCSF-ROUTING-001) is `draft`. Is the
   Claroty column-name churn (e.g., `uid → device_uid`) a blocker for the North Star demo, or
   is the demo running against the current pre-Stage-2 schema? The answer determines whether
   Option D soft is sufficient for the demo or whether Option A is needed sooner.

2. **Does Option D soft require a story?** The architect recommends it as a maintenance edit
   to existing resources. If the human agrees it's a text-only change with no behavioral
   contract modification, it can be dispatched directly without a story spec.

3. **OCSF aliasing scope after Stage 2**: If the human approves pursuing Option A post-Stage-2,
   the TOML alias table (sub-option A1) or query plan resolver (sub-option A2) approach needs
   a scope decision before a story can be authored. The key design question: should old col.names
   (`uid`, `device_category`) remain as query aliases after Stage 2, or should the transition
   be a clean break?

4. **Does the 3-attempt self-correction pattern appear in other sensors?** The demo used Claroty.
   CrowdStrike, Armis, and Cyberint are on Interpretation B (col.names) indefinitely until their
   Stage 2 stories land. If SOC-analyst demos run against those sensors before their Stage 2,
   the same 3-attempt pattern may recur.

5. **Schema evolution and the "did you mean?" mechanism**: After ADR-058 Stage 2 ships for
   Claroty, the `did_you_mean` Levenshtein logic in E-QUERY-038 will compute distances against
   OCSF-flattened names. An agent guessing `device_id` will get `did_you_mean: "device_uid"`
   (Levenshtein distance 3). Is that guidance sufficient, or should Option D hard (mandatory
   `prism_describe` check in the `query` handler) be scoped as a follow-up story?

6. **Does aliasing undermine the "schema is authoritative, read prism_describe" model?** If
   `device_id` silently resolves to `device_uid`, the agent may never learn the canonical
   name and the alias becomes a permanent crutch. This weakens the self-correction mechanism
   that is currently working. The human should weigh the ergonomics gain against the
   discoverability principle.

---

## 7. Supporting Evidence

| Evidence | Location | Relevance |
|----------|----------|-----------|
| Live demo 3-attempt sequence | D-2166/D-2167, STATE.md `current_step` | Motivates this proposal |
| TOML col.name vs ocsf_field gap | ADR-058 §A (column_mapping.rs dead path) | Root cause of OCSF fidelity gap |
| OCSF routing decision (Interpretation A) | ADR-058 §B2 v2.0 | Architecture direction for column names |
| OCSF-flattened name table for Claroty devices | ADR-058 §E2 and §J3 | Post-Stage-2 column names |
| Flag-transition shadow rule | ADR-058 §J2 | Constraint on any alias mechanism |
| E-QUERY-037 dot-notation handling | `table_registry.rs` `check_availability_gate`, BC-2.11.001 | Option B already works via error guidance |
| E-QUERY-038 available_columns | BC-2.11.016 §EC-11-041 | Current column error guidance |
| query_tutorial Step 1 | `prompts.rs` | Existing "call prism_describe first" recommendation |
| prismql://reference §3 | `resources.rs` | "Column names come verbatim from prism_describe" |
| prism_describe returns ocsf_field as description | `tools/prism_describe.rs` `ColumnDescriptor.description` | Agent can already read OCSF mapping from schema output |

---

## 8. What This Proposal Explicitly Does NOT Do

- Does not modify any behavioral contract (BC-NNN)
- Does not modify any ADR
- Does not modify any grammar or parser
- Does not create any story spec
- Does not touch code, sensor TOMLs, or test files
- Does not claim any option is accepted or authorized for implementation
- Does not pre-empt ADR-058 Stage 2 or alter its scope

---

*Produced by architect agent per human directive. All findings based on code/spec reads as of
2026-08-15. State-manager to commit after human acknowledges receipt.*
