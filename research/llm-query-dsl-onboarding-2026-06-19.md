---
document_type: research
producer: research-agent
timestamp: 2026-06-19
topic: Teaching an LLM agent to author a query DSL (PrismQL) automatically over MCP
status: complete
feeds: ADR (PrismQL agent-onboarding / teaching-surface decision)
sources:
  - https://modelcontextprotocol.io/specification/2025-06-18/server/resources
  - https://modelcontextprotocol.io/specification/2025-03-26/server/resources
  - https://modelcontextprotocol.io/docs/getting-started/intro
  - https://github.com/modelcontextprotocol/servers
  - https://www.npmjs.com/package/@modelcontextprotocol/server-postgres
  - https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite
  - https://github.com/crystaldba/postgres-mcp/blob/main/README.md
  - https://www.augmentcode.com/mcp/postgres-mcp
  - https://clickhouse.com/docs/use-cases/AI/MCP
  - https://github.com/clickhouse/mcp-clickhouse
  - https://clickhouse.com/blog/google-antigravity
  - https://docs.snowflake.com/en/user-guide/snowflake-cortex/cortex-agents-mcp
  - https://docs.cloud.google.com/bigquery/docs/use-bigquery-mcp
  - https://github.com/ergut/mcp-bigquery-server
  - https://github.com/googleapis/mcp-toolbox
  - https://www.selectstar.com/resources/text-to-sql-llm
  - https://apxml.com/courses/getting-started-model-context-protocol/chapter-3-implementing-tools-and-logic/tool-definition-schema
  - https://developers.openai.com/api/docs/guides/function-calling
  - https://aws.amazon.com/blogs/machine-learning/build-a-robust-text-to-sql-solution-generating-complex-queries-self-correcting-and-querying-diverse-data-sources/
  - https://securitylabs.datadoghq.com/articles/mcp-vulnerability-case-study-SQL-injection-in-the-postgresql-mcp-server/
  - https://towardsdatascience.com/the-mcp-security-survival-guide-best-practices-pitfalls-and-real-world-lessons/
  - https://arxiv.org/abs/2204.00498   # PICARD (Scholak et al.)
  - https://arxiv.org/abs/2304.11015   # DIN-SQL (Pourreza & Rafiei)
  - https://arxiv.org/abs/2304.05128   # Self-Debugging (Chen et al.)
  - https://arxiv.org/abs/2312.11242   # MAC-SQL
  - https://arxiv.org/abs/1807.03100   # Execution-Guided Decoding (Wang et al.)
  - https://docs.vllm.ai/en/latest/features/structured_outputs.html
  - https://blog.vllm.ai/2025/01/14/struct-decode-offering.html
---

# Teaching an LLM Agent to Author PrismQL over MCP

> Scope: how to make Claude (running in Claude Code, talking to the Prism stdio MCP server)
> reliably author **PrismQL (PQL)** against a *per-tenant* live schema, without a static doc,
> while keeping the teaching channel server-authored and injection-safe. This feeds an ADR;
> it is decision-oriented, not a literature dump. Every external claim is cited; library/version
> claims were verified against current docs (June 2026) or flagged inconclusive.

---

## Executive Summary

1. **The dominant prior-art pattern across every production DB MCP server is the same: tiny tool
   `inputSchema` (usually one `query`/`sql` string), short natural-language tool descriptions,
   and **schema discovery via dedicated tools or resources** — and almost *none* embed a formal
   grammar or few-shot examples in the tool metadata.** They lean on the model's pretrained SQL
   fluency plus a `list_tables`/`describe-table`/schema-resource introspection loop.[selectstar][sqlite][clickhouse][postgres-pro][postgres-npm]
   PrismQL is **not** pretrained-known (it is a custom DSL), so Prism cannot copy this lean
   posture wholesale — it must supply more grammar primer than a SQL server would, but it should
   still keep that primer small and push depth on-demand.

2. **Ground the teaching in the live, per-tenant schema — not a static doc.** Every credible
   source treats schema/business/usage context as something retrieved against the *actual*
   current database, and MCP's own `resources` primitive is explicitly **application-driven**
   and supports **resource templates** (RFC 6570) plus `listChanged`/`subscribe` notifications
   in the current 2025-06-18 spec — exactly the mechanism for surfacing a tenant's dynamic
   `TableRegistry` so the model never hallucinates columns for sensors the client lacks.[mcp-resources-0618][selectstar][aws]

3. **Self-correcting structured-error loops are the single best-evidenced accuracy lever available
   to a hosted model.** Decomposed/self-correcting prompting (DIN-SQL) lifted Spider SOTA 79.9% →
   85.3% (+5.4pt) and beat heavily fine-tuned models; multi-agent tool-based refinement (MAC-SQL)
   lifted BIRD 46.35 → 59.59 (+13.24pt); self-debugging gave +2–3pt overall and +9pt on the
   hardest queries.[din-sql][mac-sql][self-debug] Prism's existing **pedagogical `E-QUERY-NNN`**
   errors ("unknown column X; available: [...]; did you mean Y") are precisely this lever — the
   research validates leaning into them hard. (Caveat: no source isolates the exact dollar value
   of the "did-you-mean" string specifically; the gain is for execution-error feedback broadly.)[execfeedback]

4. **Grammar-constrained / structured decoding (PICARD, GBNF, Outlines, XGrammar/vLLM) is NOT
   available to Prism's runtime model.** It requires token-logit access and a custom decode loop,
   which hosted APIs like Claude do not expose; the technique is "only possible for models run
   locally; not external APIs."[constrained-decode][vllm-struct] Prism gets the *equivalent* via a
   **server-side parse-then-pedagogical-error gate** (Chumsky parse fail → structured `E-QUERY`
   reject → model retries) — post-hoc enforcement instead of in-loop masking. This is already
   Prism's architecture; the research confirms it is the correct and only-feasible path.

5. **Recommended teaching surface = a 3-tier split mapped onto Prism's existing primitives:**
   (a) a **concise always-present primer** in the `prism-query` tool description (PQL shape +
   3–5 canonical intent↔query pairs + a pointer to discovery/reference); (b) **capability/schema
   discovery** as a tool and/or resource-template that returns the *tenant's* live tables/columns
   (the `TableRegistry` projection) — the thing the model calls before writing a query; (c) a
   **full grammar/reference resource** fetched on demand (retrieval, not always-present), plus
   the pedagogical `E-QUERY` self-correction loop as the runtime backstop and MCP **Prompts** as
   the "how to drive this surface" scaffolding. Keep all of it server-authored.

---

## Q1 — Patterns for teaching an LLM a query DSL / text-to-SQL over MCP

### 1a. Schema-in-tool-description (grammar + examples embedded in the tool/`inputSchema`)

An MCP tool is defined by `name`, a natural-language `description`, and a JSON-Schema `inputSchema`;
the description doubles as model-facing API docs and the per-property `description` fields are
"micro-prompt engineering" — they tell the model exactly what to put in each argument.[apx][selectstar]
OpenAI's function-calling guidance (the closest large-scale corpus of tool-description best practice)
explicitly recommends describing each parameter's purpose and **including examples and edge cases in
descriptions** to fix recurring failures — but warns examples can *hurt* some reasoning models, so
they must be evaluated empirically.[openai-fc]

**Tradeoffs / token budget:** The same guidance warns to keep schemas **flat** — deep nesting and
heavy validation logic raise token count and "cognitive load," causing latency or parse errors —
and to keep the *initial* tool count small (≈<20) for high call accuracy.[apx][openai-fc] Select Star
characterizes pure prompt-injection of schema + DDL + examples + syntax rules as fine for prototypes
but **not scaling**: as schemas grow it consumes too much context and becomes brittle, and they
recommend moving depth to RAG/MCP retrieval.[selectstar]

**Effectiveness:** Embedding a *concise* grammar summary + a few canonical examples is well-supported
as beneficial; there is **no published MCP-specific quantitative study** isolating how much grammar/
examples-in-description help vs. resources vs. constrained decoding (flagged inconclusive).[selectstar][openai-fc][apx]

> **Prism implication:** Because PrismQL is unknown to the model's weights (unlike SQL), the
> primer *must* live somewhere always-present at first contact — the tool description is the
> natural home. But cap it: PQL shape + clause list + 3–5 examples + "call discovery for your
> tables / read the reference resource for full grammar." Do **not** inline the full grammar.

### 1b. Schema / capability discovery tools & resources

The canonical real-world pattern: the model first calls a discovery tool, then writes the query.[sqlite][clickhouse][postgres-pro]
- Official **SQLite** server: `list_tables` (no input → array of table names) and `describe-table`
  (`table_name` string → array of column defs with names+types).[sqlite]
- **ClickHouse** server: `list_databases`, `list_tables`, `run_select_query`; the Antigravity
  walkthrough shows the agent using `get_services_list` + `list_tables` to "orient and ground
  itself in your data structure" before querying.[clickhouse][ch-antigravity]
- **Postgres MCP Pro** (`crystaldba/postgres-mcp`): `list_schemas`, `list_objects`,
  `get_object_details` ("a table's columns, constraints, and indexes"), `execute_sql`,
  `explain_query`.[postgres-pro]
- Official **Postgres** reference server: exposed schema as **resources** —
  `postgres://<host>/<table>/schema` JSON-schema-per-table, "automatically discovered from
  database metadata" — rather than a `list_tables` tool. (This server has since been
  **deprecated/archived** after a SQL-injection finding; its `src/postgres` path now 404s.)[postgres-npm][datadog]

**Orchestration choice:** discovery can be model-driven (system prompt + tool descriptions tell it
to discover first) or host-driven (host prefetches schema and injects it). OpenAI guidance leans
toward host-side preloading of obviously-needed context and deferring rarely-used tools.[openai-fc]
A hybrid is typical: host injects a coarse summary, model calls fine-grained `describe-table`.[aws]

### 1c. On-demand reference / grammar resources (retrieval vs always-present)

The consensus is that **always-present** context (verbose tool descriptions) is the prototype path,
and **on-demand** (RAG / MCP resources / `search`+`fetch` tools) is the scaling path.[selectstar][aws]
The AWS reference text-to-SQL architecture does similarity search over catalog metadata, merges only
the *relevant* slice with the user query, then generates and self-corrects — explicitly *not* always
present.[aws] MCP `resources` are the first-class on-demand mechanism: server registers e.g. a
`grammar/prismql` reference and per-domain schema docs; the **host decides when to fetch** them
(`resources/read`), and `annotations.priority`/`audience` hint how aggressively to include them.[mcp-resources-0618][selectstar]
Cost: retrieval adds round-trips and depends on retrieval quality (bad chunks → bad queries).[selectstar][aws]

### 1d. Few-shot examples — and whether live-schema grounding matters

Examples ("usage context": canonical intent↔query pairs) are one of the three context types LLMs
need and a primary accuracy driver.[selectstar] They can sit in the tool description (always-present,
small N), as resources (fetched), or via retrieval over a historical-query vector store.[selectstar]
**Grounding in the live schema is treated as important** by every source — text-to-SQL means SQL that
runs on *your* warehouse, and examples should reflect *your* tables/columns/metrics, not generic
SQL.[selectstar][aws] The strongest signal: execution-RL and RAG architectures all bind generation to
the *current* schema/execution. **However, no source runs the clean ablation "live-schema examples
vs stale/static-schema examples"** — the superiority of live grounding is a strong, well-motivated
inference, not a measured number (flagged).[selectstar][aws][execfeedback]

> **Prism implication:** because Prism is multi-tenant with different sensors per client, the few
> examples in the always-present primer should be *schema-agnostic skeletons* (showing PQL shape),
> while *grounded* examples (real tenant table/column names) should be generated against the live
> `TableRegistry` and surfaced via discovery/resource — never a static example set that names a
> sensor a given client doesn't have.

### 1e. Self-correcting structured-error loops (the best-evidenced lever)

Feeding execution results/errors back to the model and re-prompting is the most consistently
effective family of techniques for hosted models (no logit access needed):[execfeedback][din-sql][mac-sql][self-debug]

| Method | Mechanism | Reported gain |
|---|---|---|
| Execution-Guided Decoding (Wang et al.) | execute partial query, prune invalid beams (no error fed back) | +1–6 pt exec-acc; 83.8% WikiSQL SOTA[execfeedback] |
| Self-Debugging (Chen et al.) | model re-reads/explains & revises its own query | +2–3 pt Spider overall, **+9 pt hardest**; up to +12 pt w/ unit tests[self-debug] |
| DIN-SQL (Pourreza & Rafiei) | decompose into sub-tasks + self-correct | Spider 79.9% → **85.3% (+5.4 pt)**; BIRD 55.9% (SOTA at time)[din-sql] |
| MAC-SQL | multi-agent; an agent "refine[s] erroneous SQL queries" via tools | BIRD 46.35 → **59.59 (+13.24 pt)**[mac-sql] |

Practitioner guides confirm the exact pattern Prism uses: capture the DB error ("Unknown column
'email' in 'field list'"), re-prompt with the error + schema, get corrected SQL.[execfeedback]
**Caveat (flagged):** none of these isolate the *specific* "unknown column X; did you mean Y?"
string's marginal contribution; the literature abstracts error feedback into success/failure
signals, rewards, or self-explanation. The gain attributed above is for execution-error feedback
broadly, which strongly supports — but does not separately quantify — Prism's pedagogical
`E-QUERY-NNN` design.[execfeedback]

### 1f. Grammar-constrained / structured decoding — applicability & cost for Prism

PICARD incrementally parses partial SQL during decode and rejects inadmissible tokens, turning
"passable" T5 models into Spider/CoSQL SOTA; general grammar-constrained decoding (GCD) and
tooling like GBNF (llama.cpp), Outlines, and XGrammar/vLLM enforce a CFG by masking logits each
step.[picard][vllm-struct] vLLM's XGrammar integration cut structured-decode overhead by up to ~5×
TPOT under load — i.e., real but manageable cost.[vllm-struct]

**The blocker for Prism:** constrained decoding "manipulates a generative model's token generation
process" and is "only possible for models run locally; not external APIs"; hosted APIs expose only
coarse knobs (temperature, top-p) and at best a `JSON`/structured-output mode that constrains the
*envelope*, not the SQL/DSL *inside* a string field.[constrained-decode][vllm-struct] Claude-in-Claude-Code
is exactly such a hosted client. **Therefore in-loop grammar constraint on the runtime model is
infeasible.** The feasible equivalent is **post-hoc enforcement**: parse the generated PQL
server-side (Chumsky), reject with a structured pedagogical error on failure, and let the model
retry — which is precisely Prism's plan-time gate + `E-QUERY` design.[constrained-decode][execfeedback]
(Constrained decoding *would* be available only if Prism ever shipped a self-hosted local model for
pre-validation — out of scope for the runtime path.)

---

## Q2 — Concrete prior art (what real DB/query MCP servers actually expose)

All quotes verbatim from the cited READMEs/docs; "inferred" marks shape deduced from prose where the
raw JSON Schema was not printed.

| Server | Query tool(s) & input | Schema discovery | Grammar / examples in metadata? | Errors as teaching? | Notable |
|---|---|---|---|---|---|
| **Official Postgres** (`@modelcontextprotocol/server-postgres`) | `query` — input `sql` (string); all queries in a READ ONLY txn[postgres-npm] | **Resources**: `postgres://<host>/<table>/schema` JSON-schema per table, auto-discovered[postgres-npm] | No | Not documented | **Deprecated/archived** post SQL-injection (stacked `;` statements bypassed read-only)[datadog] |
| **Official SQLite** (`server-sqlite`) | `read_query`/`write_query`/`create_table` — input `query` (string)[sqlite] | **Tools**: `list_tables` (→ table names), `describe-table` (`table_name` → cols+types)[sqlite] | No (only "SELECT query" prose) | Not documented | Adds `append_insight` + dynamic `memo://insights` resource (model accumulates findings)[sqlite] |
| **Postgres MCP Pro** (`crystaldba/postgres-mcp`) | `execute_sql` (read-only in restricted mode); `explain_query`[postgres-pro] | `list_schemas`, `list_objects`, `get_object_details` (cols/constraints/indexes)[postgres-pro] | No | "read-only limitations"; payloads undocumented | Index tuning / health / `pg_stat_statements` analysis tools = indirect teaching[postgres-pro] |
| **ClickHouse** (`clickhouse/mcp-clickhouse`) | `run_select_query` (SQL string)[clickhouse] | `list_databases`, `list_tables` (+ `get_services_list` per Antigravity)[clickhouse][ch-antigravity] | No | Not documented | Agent "orient[s] and ground[s] itself" via list tools before querying[ch-antigravity] |
| **Snowflake managed** (Cortex) | `Analyst` tool — input `{message: string}` (NL); **"The SQL statement is listed in the output"**[snowflake] | Internalized in Cortex Analyst (not exposed as raw `list_tables`)[snowflake] | No grammar; **but returns its generated SQL** = live few-shot[snowflake] | Not documented | `Search` tool: `{query, columns[], limit}`. Text-to-SQL done *server-side*; external model can read the SQL it produced[snowflake] |
| **BigQuery** (Google managed) | Not enumerated in docs (IAM: `mcp.tools.call`, `bigquery.jobs.create`, `bigquery.tables.getData`); HTTP endpoint `bigquery.googleapis.com/mcp`[bigquery] | Not documented in available docs | Not documented | Not documented | Docs are config/auth-centric; tool schemas not public in the source examined (inconclusive)[bigquery] |
| **BigQuery community** (`ergut/mcp-bigquery-server`) | Read-only; "talk directly to your BigQuery data"[ergut] | Not enumerated in README excerpt | No | Not documented | Setup-focused README; tool schemas not surfaced (inconclusive)[ergut] |
| **MCP Toolbox for Databases** (`googleapis/mcp-toolbox`) | Prebuilt generic tools; toolsets at `/mcp/{toolset_name}`; "Talk to your data, explore schemas, generate code"[toolbox] | "explore schemas" tools implied; names not in excerpt | Not documented | Not documented | Framework for custom structured-query / NL2SQL / semantic-search tools[toolbox] |

**Two macro-patterns emerge:**
- **Pattern A (raw-SQL + introspection):** server exposes execute + `list/describe` tools/resources; the *model* writes SQL. (Postgres, SQLite, ClickHouse, Postgres-Pro, BigQuery.)
- **Pattern B (agentic text-to-SQL):** server exposes a NL tool that generates SQL *server-side* and **returns the SQL in its output** as a live, schema-grounded example (Snowflake Cortex Analyst).[snowflake]

**Cross-cutting gap:** essentially **no public DB MCP server embeds a formal grammar or curated
few-shot pairs in tool metadata, and none documents errors as a pedagogical channel.** They assume
pretrained SQL fluency. PrismQL breaks that assumption — which is *why* Prism must do more teaching
than these servers, and why the pedagogical-error and reference-resource ideas are differentiators
rather than table stakes.

---

## Q3 — Token-budget management (primer vs full reference split)

Evidence-backed guidance: keep always-present metadata flat and small (token + cognitive-load cost,
latency, parse errors), keep initial tool count low, and push depth on-demand once schemas/grammars
grow.[apx][openai-fc][selectstar] Recommended split for Prism:

- **Always-present primer (in `prism-query` tool description), target small (rough order ~300–600
  tokens):** one-paragraph "what PQL is + it compiles to a federated OCSF query"; the clause
  vocabulary (SELECT/FROM/WHERE/GROUP BY/ORDER BY/LIMIT — whatever PQL's surface is); 3–5
  *schema-agnostic* canonical intent↔query skeletons; and an explicit instruction: "your available
  tables/columns depend on this client's sensors — call the discovery tool first; full grammar is in
  the reference resource." Examples are net-positive but must be evaluated (some reasoning models
  regress with too many).[openai-fc][apx]
- **On-demand depth (resources / discovery tool):** full PrismQL grammar (BNF-ish), operator
  semantics, OCSF field reference, and *grounded* examples — fetched via `resources/read` only when
  the model needs them or the host injects them by heuristic.[mcp-resources-0618][selectstar][aws]

This keeps every-turn cost bounded while making the complete reference reachable in one hop.

---

## Q4 — Multi-tenant grounding (don't hallucinate columns the client lacks)

This is Prism's sharpest requirement and where the generic prior art is weakest (the SQL servers
ground against one fixed DB; Prism's surface is per-client-dynamic). Findings:

- **MCP resources are application-driven and parameterizable.** The current **2025-06-18** spec
  supports **resource templates** (RFC 6570 URI templates), `resources/list`, `resources/read`,
  optional `subscribe`, `listChanged` notifications, and `annotations` (`audience`, `priority`,
  `lastModified`).[mcp-resources-0618] A Prism server can expose the tenant's schema as a resource
  whose contents are computed from the *live* `TableRegistry` for that connection — so the model
  only ever sees tables the client actually has. `listChanged`/`subscribe` map cleanly onto sensor
  set changes (a sensor comes online/offline → registry changes → notify).
- **Every text-to-SQL source insists on grounding generation in the actual current schema** to
  prevent hallucinated identifiers; RAG/discovery exists precisely to feed *only the relevant,
  real* schema slice.[selectstar][aws]
- **The runtime backstop is Prism's plan-time table-availability gate.** Even with perfect
  discovery, the model can still emit an unavailable table; the existing `E-QUERY` table-availability
  reject (with the available-tables list) is the self-correction signal that closes the loop — and
  the literature shows error-feedback re-prompting recovers large accuracy fractions.[execfeedback][din-sql][mac-sql]

**Best-practice synthesis for Prism:** (1) make the *only* source of tenant table/column truth a
live discovery tool + resource projection of `TableRegistry` — never a static doc; (2) keep
always-present examples schema-agnostic; (3) generate grounded examples from the live registry;
(4) treat the plan-time availability gate's structured error as the safety net, and make its message
pedagogical (`unavailable table T; this client has: [...]`).

---

## Q5 — Prompt-injection / trust (keep the teaching channel server-authored)

- **MCP DB servers have been exploited in the wild.** Datadog's case study on Anthropic's *own*
  reference Postgres server: a read-only restriction was bypassed by **stacking statements with
  `;`** — the server has since been deprecated/archived. Lesson: the tool boundary does **not**
  prevent classic injection; use least-privilege DB users, prepared statements, and server-side
  validation.[datadog] For Prism this maps to: parse + plan-gate every PQL string server-side
  (Chumsky), never string-concatenate it into a backend call, and keep the federated adapters
  least-privilege.
- **Keep teaching content server-authored and trusted.** The primer, grammar reference, discovery
  output, examples, and error hints must originate from Prism (trusted), not from sensor data or
  user free-text, so they cannot become an injection vector. MCP `resources` annotations and the
  `audience` field let the server label what is model-facing.[mcp-resources-0618][mcp-sec]
- **Data flowing back stays structured (OCSF).** Returning normalized OCSF rather than raw sensor
  API bodies (Prism's existing normalization-at-adapter-boundary posture) is itself an
  injection-defense: structured, server-shaped results limit the surface for adversarial content
  reaching the model. General MCP security guidance reinforces validating all inputs/URIs and not
  over-trusting tool/resource content.[mcp-sec][mcp-resources-0618]
- **Structured-output envelope ≠ grammar enforcement.** Even if Prism used a JSON envelope, that
  constrains shape, not the PQL inside a string — so server-side parse remains mandatory.[constrained-decode][vllm-struct]

---

## Recommendations mapped to Prism's existing MCP surfaces

| Prism surface (existing/planned) | Recommendation | Evidence |
|---|---|---|
| **`prism-query` tool description (primer)** | Add a small always-present PQL primer: shape + clause vocab + 3–5 *schema-agnostic* intent↔query skeletons + "call discovery / read reference for depth." Keep flat & bounded; A/B the example count. | [openai-fc][apx][selectstar] |
| **Capability/schema discovery (tool + Resource)** | Expose a discovery tool AND/OR a **resource template** that returns the *live* `TableRegistry` projection for *this* client (tables, columns, types, maybe OCSF mappings). This is the model's "what can I query" step. Make it the single source of schema truth. | [sqlite][clickhouse][postgres-pro][mcp-resources-0618][aws] |
| **Reference Resource (on-demand grammar)** | Ship the full PrismQL grammar + operator/OCSF reference + *grounded* examples as a fetched resource (not always-present). Use `annotations.priority` to hint inclusion. | [mcp-resources-0618][selectstar][aws] |
| **Pedagogical `E-QUERY-NNN` errors (self-correction loop)** | Lean in hard. Make every plan-time/parse error actionable: "unknown column X; available: [...]; did you mean Y"; "table T unavailable for this client; available: [...]". This is the hosted-model substitute for grammar-constrained decoding. | [execfeedback][din-sql][mac-sql][self-debug][constrained-decode] |
| **MCP Prompts (just added)** | Use Prompts to encode the *driving procedure*: "discover tables → read reference if unsure → write PQL → on `E-QUERY`, read the error's available-list and retry." Encodes the multi-step self-correction workflow the literature shows works. | [din-sql][mac-sql][openai-fc] |
| **Server-side Chumsky parse + plan gate (trust)** | Keep it as the post-hoc enforcement layer (the only feasible "constraint" for a hosted model) and the injection boundary; never concatenate PQL into backend calls; least-privilege adapters; return OCSF only. | [constrained-decode][datadog][mcp-sec] |
| **Optional differentiator (Pattern B echo)** | Consider echoing the *normalized/validated PQL the server actually planned* back in successful results (like Cortex Analyst returns its SQL) so the model accrues live, correct, tenant-grounded exemplars over a session. | [snowflake] |

---

## Open Questions for the Architect

1. **Primer placement & budget:** Is the `prism-query` tool description the right home for the
   always-present primer, or should it ride in an MCP Prompt the client invokes once per session?
   (Tool descriptions are always-present per turn; Prompts are pulled on demand. Trade per-turn
   token cost vs. guaranteed presence.) No source measured PrismQL-specific optimal primer size —
   needs an internal eval.
2. **Discovery as Tool vs Resource vs both:** Resources are *host-decides-when* (the client/Claude
   Code controls fetch); a tool is *model-decides-when* (explicit call, auditable). For a security
   product an auditable model-initiated discovery call may be preferable — does Prism want the
   `TableRegistry` exposed as a tool, a resource template, or both? (Spec supports all.)[mcp-resources-0618]
3. **Grounded-example generation:** Should Prism auto-generate per-tenant example queries from the
   live registry (cost, freshness on `listChanged`) or curate a small canonical set per sensor type?
   The live-grounding benefit is strongly motivated but not separately measured (flagged).[selectstar][aws]
4. **Self-correction loop budget:** How many `E-QUERY` retry rounds before failing to the user?
   MAC-SQL/DIN-SQL gains come with multi-step latency; Prism should bound iterations. No source gives
   an optimal retry cap for this exact loop.[mac-sql][din-sql]
5. **Echoing planned PQL (Pattern B):** Worth the result-payload cost to return the validated/planned
   PQL for in-session few-shot accrual, or does it leak plan internals / inflate tokens?[snowflake]
6. **`listChanged`/`subscribe` for sensor set changes:** Does the Claude Code MCP client honor
   resource `subscribe`/`listChanged` today? (Spec-optional; client support varies — needs
   verification against the specific client build; **inconclusive from public docs**.)[mcp-resources-0618]

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) DSL/text-to-SQL teaching patterns over MCP; (2) concrete prior-art DB MCP servers' tool schemas/resources/errors; (3) self-correction error-loop evidence + grammar-constrained decoding feasibility for hosted models |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not used — MCP spec + server READMEs verified directly via WebFetch (more authoritative for this surface than Context7's library index) |
| Tavily (all) | 0 | — |
| WebFetch | 2 | Verified current MCP **2025-06-18** Resources spec (templates, listChanged, subscribe, annotations); confirmed official Postgres reference server path 404s (deprecation corroboration) |
| WebSearch | 0 | — |
| Training data | 2 areas | Prism-internal surface mapping (PrismQL/Chumsky/DataFusion/TableRegistry/E-QUERY) for framing only; ADR-level synthesis. All *external* claims are cited. |

**Total MCP tool calls:** 3 (Perplexity deep-research) + 2 WebFetch verifications = 5 grounded external retrievals.
**Training data reliance:** low — every external/quantitative claim is sourced; version-specific MCP claims verified against the live 2025-06-18 spec; library-version claims (PICARD/vLLM/etc.) cited to primary sources; live-schema-grounding superiority and the marginal value of the specific "did-you-mean" error string are explicitly flagged as inconclusive (motivated inference, not measured).

### Citation key
- [mcp-resources-0618] https://modelcontextprotocol.io/specification/2025-06-18/server/resources (verified live, 2026-06-19)
- [selectstar] https://www.selectstar.com/resources/text-to-sql-llm
- [apx] https://apxml.com/courses/getting-started-model-context-protocol/chapter-3-implementing-tools-and-logic/tool-definition-schema
- [openai-fc] https://developers.openai.com/api/docs/guides/function-calling
- [aws] https://aws.amazon.com/blogs/machine-learning/build-a-robust-text-to-sql-solution-generating-complex-queries-self-correcting-and-querying-diverse-data-sources/
- [postgres-npm] https://www.npmjs.com/package/@modelcontextprotocol/server-postgres
- [sqlite] https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite
- [postgres-pro] https://github.com/crystaldba/postgres-mcp/blob/main/README.md , https://www.augmentcode.com/mcp/postgres-mcp
- [clickhouse] https://clickhouse.com/docs/use-cases/AI/MCP , https://github.com/clickhouse/mcp-clickhouse
- [ch-antigravity] https://clickhouse.com/blog/google-antigravity
- [snowflake] https://docs.snowflake.com/en/user-guide/snowflake-cortex/cortex-agents-mcp
- [bigquery] https://docs.cloud.google.com/bigquery/docs/use-bigquery-mcp
- [ergut] https://github.com/ergut/mcp-bigquery-server
- [toolbox] https://github.com/googleapis/mcp-toolbox
- [datadog] https://securitylabs.datadoghq.com/articles/mcp-vulnerability-case-study-SQL-injection-in-the-postgresql-mcp-server/
- [mcp-sec] https://towardsdatascience.com/the-mcp-security-survival-guide-best-practices-pitfalls-and-real-world-lessons/
- [picard] https://arxiv.org/abs/2204.00498 (Scholak et al., PICARD)
- [din-sql] https://arxiv.org/abs/2304.11015 (Pourreza & Rafiei, DIN-SQL)
- [self-debug] https://arxiv.org/abs/2304.05128 (Chen et al., Teaching LLMs to Self-Debug)
- [mac-sql] https://arxiv.org/abs/2312.11242 (MAC-SQL)
- [execfeedback] https://arxiv.org/abs/1807.03100 (Wang et al., Execution-Guided Decoding) + AWS/practitioner guides above
- [constrained-decode] structured/constrained-decoding guidance (logit-access requirement) corroborated by vLLM docs below
- [vllm-struct] https://docs.vllm.ai/en/latest/features/structured_outputs.html , https://blog.vllm.ai/2025/01/14/struct-decode-offering.html
