---
document_type: research
title: "PrismQL Case-Sensitive vs Case-Insensitive String Comparison"
topic_slug: prismql-case-sensitivity
date: 2026-06-27
author: vsdd-factory:research-agent
status: complete
research_type: general
feeds_into: architecture-decision (ADR — string-comparison case policy for PrismQL)
related:
  - crates/prism-query/src/pipe_sql_emitter.rs (predicate_to_datafusion_sql)
  - crates/prism-query/src/ast.rs (CompareOp, StringOp, Predicate::In)
  - crates/prism-query/src/filter_parser.rs (ICONTAINS/ISTARTSWITH/IENDSWITH grammar)
  - crates/prism-ocsf/src/enum_map.rs (OCSF canonical captions)
  - ADR-024 (ColumnType canonical naming)
---

# PrismQL Case-Sensitive vs Case-Insensitive String Comparison

> **One-line recommendation:** Keep `=` / `IN` **case-SENSITIVE by default** (least surprise, performance, matches KQL/EQL/Elastic `term`), add an **explicit case-insensitive opt-in** that mirrors PrismQL's *existing* `I*` prefix convention (`IEQ` / `IIN`, lowered to `lower(col) = lower('lit')`), AND **normalize OCSF enum-label fields to canonical casing at the adapter/ingest boundary** so analysts rarely need the opt-in at all. The two together are complementary, not either/or.

---

## 0. Problem Statement & Codebase Grounding

The demo defect: LLM-agent-authored queries like `WHERE severity IN ('HIGH','CRITICAL')` and `status = 'open'` return **zero rows** because the underlying OCSF-normalized data carries Title-case / vendor-native values (`'High'`, `'Critical'`, `'new'`, `'Unresolved'`, `'UNHANDLED'`). PrismQL's `=` and `IN` are case-sensitive exact matches today.

Three facts grounded in the prism codebase materially shape the recommendation:

1. **PrismQL already has a case-insensitivity convention — but only for string-pattern ops.** `StringOp` (`Contains`/`StartsWith`/`EndsWith`) carries a `case_insensitive: bool` flag, surfaced in the grammar as the `I*` keyword prefixes `ICONTAINS` / `ISTARTSWITH` / `IENDSWITH` (`filter_parser.rs:985-993`), and lowered to `lower(field) LIKE lower('%pat%')` in `predicate_to_datafusion_sql` (`pipe_sql_emitter.rs:541-564`). **The `=`/`!=` (`Predicate::Compare`) and `IN` (`Predicate::In`) paths have NO such flag** (`ast.rs:1284-1306`, `1550-1554`). This is the exact gap.

2. **prism already defines canonical OCSF casing.** `crates/prism-ocsf/src/enum_map.rs` maps `severity_id` → Title-case captions: `4 → "High"`, `5 → "Critical"`, `1 → "Informational"`, etc. (lines 60-66). So the canonical/expected string label form for severity in prism is **Title-case**, not the uppercase the agent guessed. The data is "right"; the query casing is "wrong" only relative to an undocumented convention.

3. **The emitter is a single, well-isolated lowering point.** `predicate_to_datafusion_sql(pred: &Predicate)` (`pipe_sql_emitter.rs:506`) is the one place filter-mode predicates become DataFusion SQL, and it already demonstrates the `lower(...)` idiom. There is a sibling op table in the same file (`:743-749`) and the PQL round-trip normalizer (`ast.rs:1977-2016`) that must be swept in lockstep (TD-VSDD-060 sibling-site rule).

**Verified-finding vs model-knowledge legend:** [WEB] = verified via cited web source this session; [CODE] = verified by reading prism source this session; [MODEL] = model knowledge, flagged.

---

## 1. Survey of Case-Insensitive Mechanisms in SQL & Analytics Engines

Five mechanisms exist across mainstream engines. Pros/cons are scored for **two** audiences prism cares about: a human security analyst, and an **LLM agent** that writes queries from a learned grammar.

| Mechanism | How it works | Analyst ergonomics | LLM-agent ergonomics | Performance | Verdict for PrismQL |
|---|---|---|---|---|---|
| **`LOWER()`/`UPPER()` normalization** | `lower(col) = lower('lit')` | Verbose but universal; everyone knows it | Highly learnable; deterministic; no engine-specific operator to hallucinate | Loses index/sargability; full-column scan — but prism MemTables are tiny (≤10K rows, BC-2.11.006), so cost is negligible [CODE] | **Lowering target of choice** — already used in prism for `ICONTAINS` [CODE] |
| **`ILIKE` / `~~*`** | Case-insensitive `LIKE` pattern op | Familiar to Postgres users | Pattern semantics (`%`,`_`) leak in even for exact match — agent must escape | DataFusion supports `ILIKE`; `~~*` support is **version-ambiguous** (see §2) | Good for *pattern* ops, wrong tool for exact `=`/`IN` |
| **`COLLATE` (CI/AI collations)** | `col = 'lit' COLLATE ci` or column-level CI collation | Cleanest *if* supported | Agent must know collation names; non-portable | Can be index-backed in mature DBs | **Not viable** — DataFusion does not implement ANSI `COLLATE` for CI matching (§2) [WEB] |
| **Case-insensitive operators (`=~`, `~*`, KQL `=~`, EQL `:`)** | Dedicated operator token | Concise once learned | A single discoverable token is *very* LLM-friendly if documented in grammar/examples | Engine-dependent | **Strong fit** — but pick a token that fits prism's existing `I*` family, not a symbol agents may confuse with regex `=~` (which PrismQL already uses for MATCHES!) [CODE] |
| **Dedicated functions (`equals_ci(a,b)`)** | UDF | Explicit | Learnable but verbose | UDF dispatch cost | Overkill given `lower()` already works |

**SQL-standard default** [WEB/MODEL]: ANSI SQL string `=` semantics are governed by the column's *collation*; the standard itself does not mandate case-insensitive equality. In practice each engine picks a default: PostgreSQL/MySQL-binary/SQLite/DuckDB/**DataFusion are case-SENSITIVE** by default; MySQL's default `_ci` collations and SQL Server's common `CI_AS` collation are case-INSENSITIVE. There is no universal default — which is itself an argument for making PrismQL's choice *explicit and documented*.

**Key cross-engine pattern (verified across 5 SIEM/analytics languages, §3):** the dominant industry design is **case-sensitive exact-equality with an explicit case-insensitive variant marked by a consistent affix** — KQL's `_cs`/`=~`, EQL's `~` suffix and `:` operator, Sigma's `|cased` modifier. PrismQL's existing `I*` *prefix* is the same idea in a different spelling. Consistency with the language's own established convention beats copying any one engine.

---

## 2. What Apache DataFusion Natively Supports

prism executes every filter/pipe/SQL predicate through DataFusion `session_ctx.sql(...)` over in-memory Arrow MemTables [CODE: `materialization.rs:1019, 1144, 1189`]. So the recommendation is bounded by what DataFusion's SQL dialect accepts.

| Feature | DataFusion support | Source |
|---|---|---|
| `lower(str)` / `upper(str)` scalar fns | **Yes**, fully supported (Unicode-aware: `lower('Ångström') → 'ångström'`) | [WEB] DataFusion scalar-functions docs (Context7) |
| `ILIKE` operator | **Yes** | [WEB] DataFusion issue #12637 (Context7/Perplexity) |
| `~~*` (case-insensitive LIKE op) | **Ambiguous** — DataFusion's `operators.md` documents `'datafusion' ~~* 'Dat_F%n'` as case-insensitive LIKE [WEB Context7], but a 2026 Perplexity lookup reported the `~~*` token as *not* in the dialect [WEB]. **Conflict flagged** — verify against the pinned DataFusion version before relying on `~~*`. | [WEB] both cited; conflicting |
| `regexp_match(col, pat, 'i')` case-insensitive flag | **Yes** — `'i'` flag supported | [WEB] DataFusion scalar-functions docs (Context7) |
| ANSI `COLLATE` (CI collations) | **No** — DataFusion does not implement case-insensitive collations; project guidance is to extend the parser/planner rather than use `COLLATE` | [WEB] Perplexity citing DataFusion extending-SQL blog + issue #12637 |

**DataFusion-idiomatic conclusion:** `lower()`-normalization is the **unambiguously supported, version-stable, already-in-use** mechanism. `ILIKE` is supported and idiomatic for *pattern* matching. `COLLATE` is out. Given the codebase already emits `lower(field) LIKE lower(...)` for `ICONTAINS`, extending the same `lower(...)` idiom to `=`/`IN` is the **lowest-risk, most-consistent** lowering — it reuses a path already exercised by tests and adds zero new DataFusion-version dependency.

---

## 3. SIEM / Security-Language Conventions + OCSF (the decisive context)

prism is a security tool whose consumers (LLM analyst agents) carry priors from SIEM query languages. The deep-research sweep (19 cited sources) found a consistent dual-mode pattern:

| Language | Free-text search default | Field `=` / exact default | Case-insensitive opt-in | Case-sensitive opt-in |
|---|---|---|---|---|
| **Splunk SPL** | case-**insensitive** | `eval`/`where`/lookup are case-**SENSITIVE** | `lower()`/`upper()`; `case_sensitive_match=false` on lookups | `CASE()` directive |
| **KQL / Kusto** | n/a | `==` case-**SENSITIVE** | `=~` (eq), `in~`, `has`, `contains` (unsuffixed = CI) | `==`, `_cs` suffix (`has_cs`, `contains_cs`) |
| **Elastic EQL** | n/a | `==` case-**SENSITIVE** | `:` operator, `~` suffix (`like~`, `in~`, `endsWith~`) | `==`, `like`, `regex`, `in` |
| **Elasticsearch DSL** | `text` fields CI (analyzer lowercases) | `keyword`+`term` case-**SENSITIVE** | `term` `case_insensitive: true` (since 7.10) | default `term` |
| **Sigma** | field values default **CI** | (compiled per-backend) | default | `|cased` modifier |

Sources: Splunk docs/community [1-8]; Microsoft KQL docs [9-12]; Elastic EQL/DSL [13-16]; Sigma spec [15]. (Full URLs in References.)

**Reading for prism:**
- For **structured field equality** (`severity = ...`, `status = ...`) — the prism use case — the SIEM consensus default is **case-sensitive**, with an explicit, affix-marked CI variant. Sigma is the outlier (CI by default) but Sigma is a *portable rule abstraction*, not an execution language; it defers case to the backend. PrismQL is an execution language closer to KQL/EQL.
- The recurring affix convention (`_cs`, `~`, `|cased`) is what makes the choice **discoverable**. PrismQL already owns the inverse affix (`I`-prefix for insensitive). Reusing it keeps one mental model.

### OCSF: canonical casing & where to normalize

[WEB, OCSF schema + Datadog OCSF processor docs, GitHub discussion #450]:
- OCSF models categorical fields as an **integer `_id` enum + a string-label sibling** (`severity_id: 4` ↔ `severity: "High"`). The integer `_id` is the canonical, machine-stable representation; the string label is human-facing.
- OCSF string labels have **canonical casing** (Title-case: `Informational/Low/Medium/High/Critical/Fatal/Other`, `Success/Failure`), though the spec stops short of a *normative "MUST"* statement for every label.
- The OCSF ecosystem (Datadog's OCSF processor workflow) **expects vendor-value normalization — including case — at the ingest/adapter boundary**, not at query time. Detection guidance: prefer filtering on `_id` integer fields over free-form string labels.

[CODE] prism **already encodes** these exact Title-case captions in `enum_map.rs` (`4 → "High"`, `5 → "Critical"`). So prism's canonical severity label form is **Title-case** — which is *neither* the `'HIGH'` the agent guessed *nor* something the agent can reliably guess. This is the root cause and it points to a two-pronged fix.

**Adapter-boundary normalization vs query-time CI — the tradeoff:**

| | Normalize at adapter boundary (ingest) | Case-insensitive at query time |
|---|---|---|
| Aligns with OCSF design | **Yes** (OCSF's intended architecture) [WEB] | No (OCSF discourages query-time normalization of labels) |
| Fixes the demo defect | Yes — `severity` becomes uniformly `"High"`, but agent still must guess Title-case | Yes — `severity IEQ 'high'` matches regardless of stored case |
| Cross-sensor consistency | **Yes** — one canonical value per concept across CrowdStrike/Claroty/Armis/Cyberint | No — each sensor's raw casing persists |
| Aggregation correctness (`GROUP BY severity`) | **Yes** — `'High'` and `'HIGH'` collapse | No — fragmented buckets unless CI grouping |
| Performance | Paid once at ingest | Paid per query (negligible at ≤10K rows) [CODE] |
| Discoverability for agent | Still requires knowing canonical case | Self-evident from grammar (`IEQ`/`IIN`) |

**They are complementary, not exclusive.** Adapter-boundary normalization is the *correct primary fix* (it makes the data trustworthy and aggregations correct). Query-time CI is the *ergonomic safety net* for the residual: cross-sensor casing prism hasn't normalized yet, free-form non-enum fields (hostnames, usernames, file paths), and forgiving the agent that types `'high'`.

---

## 4. RECOMMENDATION for PrismQL

### 4.1 Default policy: **case-SENSITIVE** for `=`, `!=`, `IN`

Rationale: (a) matches the SIEM execution-language consensus (KQL `==`, EQL `==`, Elastic `term`) the agent has priors for; (b) matches DataFusion's own default — no surprise between PrismQL semantics and the underlying engine; (c) preserves exact-match precision for fields where case is meaningful (usernames, file paths, registry keys, process names — the masquerading-detection use case from §3); (d) consistent with PrismQL's *current* behavior, so no silent semantic change to existing saved queries.

### 4.2 Opt-in mechanism: extend the existing `I*` prefix family

PrismQL already spells case-insensitivity as an `I`-prefix keyword (`ICONTAINS`/`ISTARTSWITH`/`IENDSWITH`) [CODE]. **Extend the same convention** rather than inventing a new symbol:

- **`IEQ`** (case-insensitive equality) — pipe/filter mode: `severity IEQ 'high'`
- **`IIN`** (case-insensitive membership) — `status IIN ('open','new','unhandled')`
- Optionally **`INE`** for completeness (case-insensitive `!=`).

Why a keyword prefix and **not** a symbolic operator: PrismQL **already uses `=~` for MATCHES (regex)** [CODE: `filter_parser.rs:887`]. Adopting `=~` for case-insensitive equality (as KQL does) would **collide** with prism's regex operator and confuse both analysts and the agent. The `I`-prefix is collision-free and already established.

Lowering (one-line analog of the existing `ICONTAINS` path):
- `IEQ` → `lower(<field>) = lower('<lit>')`
- `IIN` → `lower(<field>) IN (lower('v1'), lower('v2'), ...)`

This is the **unambiguously DataFusion-supported** form (§2), reuses the exact idiom already in `predicate_to_datafusion_sql`, and needs no new DataFusion-version dependency.

### 4.3 Adapter-boundary normalization (the primary, parallel fix)

Independently of the operator work, **normalize OCSF enum-label string fields to their canonical OCSF casing at the adapter/normalizer boundary** (prism-ocsf), reusing the canonical captions already in `enum_map.rs`. This is the OCSF-blessed architecture, fixes aggregation correctness, and gives cross-sensor consistency that no query-time operator can. Scope this as a **product/architecture decision** (see §6) because it touches the normalizer contract (BC-2.02.002 / BC-2.02.010) and every sensor TOML spec's column semantics.

### 4.4 Discoverability for the LLM agent (critical)

A grammar feature the agent never learns about is invisible. Make `IEQ`/`IIN` discoverable by:
- Adding them to the PrismQL grammar reference and the `prism describe` / tool-schema examples the agent consumes (the `PrismDescribeResponse` / pedagogical-example surface added in S-DEMO-PRISMQL-ONBOARDING-001-A/B).
- Including a worked example in the agent-facing docs: *"OCSF severity is Title-case (`'High'`). Use `severity IEQ 'high'` to match regardless of case, or `severity = 'High'` for exact canonical match."*
- Surfacing a **zero-rows-with-a-near-miss hint**: when a string `=`/`IN` filter returns zero rows but a case-insensitive match *would* have matched, emit a pedagogical diagnostic suggesting `IEQ`/`IIN` (this is the highest-leverage agent-ergonomics win and directly prevents the demo failure mode from recurring silently).

---

## 5. Migration / Impact — smallest clean change

Touch points, in dependency order (all reuse existing patterns; sibling-sweep per TD-VSDD-060):

1. **AST** (`ast.rs`): add `case_insensitive: bool` to `Predicate::Compare` (or a new variant) and to `Predicate::In`. Both enums are `#[non_exhaustive]` already — additive, no break. Mirror the existing `StringOp { case_insensitive }` shape. New public fields on `prism-query` types are covered by the `#[non_exhaustive]` gate (CLAUDE.md) — no new types added, so the `EXPECTED=87` count is unaffected.
2. **Grammar** (`filter_parser.rs`): add `IEQ`/`IIN`/`INE` keyword alternatives alongside the existing `ICONTAINS` block (`:985-993`) — a near-verbatim extension. Keywords are already case-insensitive (`kw(...)`), so `ieq` and `IEQ` both parse.
3. **Predicate emitter** (`pipe_sql_emitter.rs`): in `predicate_to_datafusion_sql`, branch `Predicate::Compare`/`Predicate::In` on `case_insensitive`, emitting `lower(lhs) = lower(rhs)` / `lower(field) IN (lower(v), ...)` — the exact idiom already at `:546`. **Sweep the sibling op table at `:743-749`** and the PQL round-trip normalizer (`ast.rs:1977-2016`) so the AST→string round-trip (BC-2.11.018) renders `IEQ`/`IIN` rather than dropping the flag.
4. **Round-trip / normalizer tests**: extend the BC-2.11.018 round-trip and predicate-lowering tests to cover the new ops (TDD red gate first).
5. **Agent-facing docs / `prism describe`**: add `IEQ`/`IIN` examples + the OCSF Title-case note (§4.4).
6. **(Parallel, separate story)** adapter-boundary canonical-case normalization in prism-ocsf.

**Backward-compat:** Fully additive. Existing `=`/`IN` keep case-sensitive semantics — no saved query changes behavior. The `#[non_exhaustive]` enums already force downstream wildcard arms, so adding fields/variants is non-breaking by construction. The only semantic-change candidate — flipping the *default* to case-insensitive — is **explicitly NOT recommended** (would silently alter precision for username/path/process-name filters and diverge from DataFusion's own default).

---

## 6. Decisions the human must make

These are genuine product/architecture calls, not work the AI should silently pick (Canonical Principle: surface DECISIONS, fix WORK):

1. **PRODUCT — adapter-boundary canonical-case normalization scope.** Should prism-ocsf normalize *all* OCSF enum-label string fields (severity, status, activity, disposition, …) to canonical OCSF casing at ingest, or only the demo-critical set (severity, status)? This changes the normalizer contract (BC-2.02.002/.010) and every sensor TOML's column semantics → architect ADR + product-owner BC amendment. Recommended: yes, all enum-label fields, but the *set* is a product call.
2. **PRODUCT — default-case confirmation.** Confirm the recommended **case-sensitive default** for `=`/`IN`. (Flagged because the user's phrasing "support both" could be read as wanting CI-by-default. The research consensus and DataFusion-parity argument favor sensitive-by-default + explicit CI opt-in, but the default is a UX policy call.)
3. **NAMING — opt-in spelling.** `IEQ`/`IIN`/`INE` (recommended, consistent with existing `ICONTAINS`) vs an alternative the human prefers (e.g., a `|ci` modifier à la Sigma, or `~` suffix à la EQL). Recommend `I`-prefix for collision-avoidance with prism's `=~` regex operator.
4. **SCOPE — zero-rows pedagogical hint.** Whether to build the "did you mean case-insensitive?" near-miss diagnostic now or in a follow-up onboarding story. (High agent-ergonomics value; non-trivial to detect cheaply.)

---

## 7. Open / inconclusive items

- **DataFusion `~~*` token support** is genuinely conflicting across two sources this session (Context7 operators doc shows it; a 2026 Perplexity lookup says it's absent). **Inconclusive** — but immaterial to the recommendation, which uses `lower()` (unambiguously supported) rather than `~~*`. Verify against the pinned DataFusion version only if `~~*` is ever chosen.
- OCSF provides **no single normative "MUST" statement** mandating canonical casing for *every* string label [WEB]; the Title-case convention is inferred from the schema + enum captions + ecosystem tooling. prism's own `enum_map.rs` is the authoritative casing for prism regardless.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Mainstream SQL/analytics engines case-insensitivity mechanisms (ILIKE/LOWER/COLLATE/operators), SQL-standard default, pros/cons for analyst + LLM audiences. (2) SIEM/security-language conventions (Splunk SPL, KQL, Elastic EQL/DSL, Sigma) + OCSF enum casing + ingest-vs-query-time normalization. Both run at reasoning_effort=high. |
| Perplexity perplexity_ask | 1 | ≤3-sentence factual confirmation: DataFusion `ILIKE`/`~~*`/`COLLATE` support. |
| Context7 | 2 | resolve-library-id → /apache/datafusion; query-docs for ILIKE/lower/COLLATE/regexp_match case-insensitive support (DataFusion authoritative docs). |
| Read (code) | 6 | materialization.rs, pipe_sql_emitter.rs, ast.rs, filter_parser.rs, enum_map.rs (+ persisted research output). |
| Grep (code) | 5 | predicate emitter op tables, CompareOp/StringOp/Predicate::In definitions, existing `I*` grammar, OCSF normalizer surface. |

**Total MCP tool calls:** 5 (2 perplexity_research + 1 perplexity_ask + 2 Context7).
**Training data reliance:** low — every external claim is web- or doc-cited; SQL-standard collation-default behavior is the only [MODEL]-flagged item and it is corroborated by the engine-by-engine [WEB] survey. All codebase claims are [CODE]-verified by reading prism source this session.

### References (web, verified this session)
- Splunk SPL case behavior: community.splunk.com threads [1][2][3][8]; help.splunk.com search-primer (CASE/TERM) [6][7]; eval comparison fns [4]; sourcetype case-sensitivity [5].
- Microsoft KQL: learn.microsoft.com equals-operator (`==`/`=~`) [9], has-operator [10], has-cs-operator [11], string operators (`_cs` convention) [12].
- Elastic: EQL syntax (`==`/`:`/`~`) [16], Query DSL term-query (`case_insensitive`, text vs keyword) [14], GitHub issue #61883 (EQL case behavior) [13].
- Sigma: sigmahq.io modifiers (`|cased`) [15].
- OCSF: Datadog OCSF processor docs [17], ocsf-schema GitHub discussion #450 (string-sibling normalization) [18], base_event.json captions [19].
- DataFusion: scalar-functions docs (`lower`, `regexp_match 'i'`), operators docs (`~~*`), issue #12637 (ILIKE/COLLATE), extending-SQL blog (2026-01-12) — via Context7 + Perplexity.
