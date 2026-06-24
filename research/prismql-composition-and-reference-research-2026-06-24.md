---
document_type: research
producer: research-agent
date: 2026-06-24
subject: PrismQL SQL→pipe composition LIMIT semantics (HRG-1) + drift-proof reference generation strategy (HRG-3)
informs: [ADR-043, ADR-045]
status: complete
method: Perplexity perplexity_research (reasoning_effort=high) × 2 + perplexity_ask × 2; web-grounded multi-source synthesis. Cross-language survey of GoogleSQL pipe syntax, PRQL, KQL (Kusto), Splunk SPL, Spark SQL for HRG-1; CI doc-testing patterns (PRQL book, DataFusion, trycmd, Chumsky) for HRG-3.
---

# PrismQL Research: Composition LIMIT Semantics (HRG-1) + Drift-Proof Reference Generation (HRG-3)

Date: 2026-06-24
For: architect (ADR-043 composition, ADR-045 reference generation) → human ratification
Scope: TWO focused external research questions for the PrismQL (PQL) engine.

---

## QUESTION 1 (HRG-1) — Composition LIMIT semantics

**Question.** A composed query `SELECT … FROM t [WHERE p] | enrich fn(c) | limit N` (SQL head + pipe tail) can specify BOTH a SQL `LIMIT` (in the head) AND a trailing `| limit` stage. Behavior options: (a) forbid both (plan-time pedagogical error), (b) pipe `| limit` wins as final cap, SQL `LIMIT` acts as intermediate cap, (c) SQL `LIMIT` wins. Product owner leans toward NOT supporting both.

### What mature hybrid SQL+pipe / dataflow languages actually do

| Language / system | Row-cap constructs | Behavior when caps appear in >1 place |
|---|---|---|
| **GoogleSQL pipe syntax** (BigQuery / ZetaSQL, VLDB 2024 pipe-syntax paper) | standard `LIMIT` clause; pipe `\|> LIMIT count [OFFSET n]` documented as *behaving identically to the standard `LIMIT` clause* | Pipe operator is a **unary relational operator** and a **semantic execution barrier** — everything before `\|>` is treated as a fully-evaluated table. Pipe and standard syntax mix via subqueries; nested limits **compose** (inner caps the rows the outer sees). **No documented rule forbidding multiple limits, and no precedence rule** — composition by position. |
| **PRQL** (`take`, compiles to SQL `LIMIT`) | `take n` transform | Pure pipeline; each transform rewrites the relation from the prior line. Multiple `take` transforms **compose sequentially** — the later/smaller cap determines the final row count; earlier caps still bound intermediate volume. No parse-time error for redundancy. |
| **Kusto / KQL** (`take` / `limit` / `top`) | `take`, `limit` (synonyms, arbitrary rows), `top N by` (sorted) | Tabular pipeline; docs *emphasize operator order is significant*. Multiple caps **compose by position**. Engine-level result truncation (500k rows / 64 MB, configurable via `notruncation`/`truncationmaxrecords`) is a **separate safety layer**, not a precedence rule. No forbiddance of redundant caps. |
| **Splunk SPL** (`head` / `tail`) | `head N`, `tail N` pipeline commands | `\| head 100 \| tail 10` → last 10 of first 100. Pure pipeline composition by position; `head`/`tail` chainable with no restriction; no error for redundancy. |
| **Databricks / Spark SQL** | SQL `LIMIT`; DataFrame `.limit()/.take()/.head()` | Nested `LIMIT` and chained `.limit(100).limit(10)` allowed; Catalyst optimizer may **collapse** consecutive limits to the smallest, but the user-visible semantics are compositional. No precedence rule, no forbiddance. |

### Cross-language verdict

1. **Row caps are universally modeled as ordinary unary operators that compose by pipeline position.** Across all five mature systems researched, the available documentation contains **zero** examples of (a) forbidding redundant caps at parse/plan time, or (c) one syntactic location "winning" over another by special precedence. The effective cap is always "the smallest cap on the path to output, applied in evaluation order" — which in a head→tail flow means **the tail cap is the final cap and the head cap is an intermediate cap (option b)**.
2. **The pipe operator's design ethos is anti-"inside-out".** GoogleSQL's pipe paper explicitly frames `|>` as a top-down semantic barrier intended to *kill* SQL's inside-out evaluation surprise. Letting a SQL-head `LIMIT` silently override a later pipe `| limit` (option c) would resurrect exactly the inside-out semantics pipe syntax exists to eliminate. **Option (c) is contraindicated by every source.**
3. **Important caveat — sources gap.** None of the cited docs explicitly address the *exact* prism construct: a SQL head with a trailing `LIMIT` clause AND a pipe `| limit` *at the same syntactic level after a single FROM*. GoogleSQL's examples never mix a trailing standard `ORDER BY … LIMIT` clause with pipe operators after one `FROM`; they keep pipe-tail operations *as pipe operators only*, and reserve mixing for the subquery boundary. This is a structural hint that **prism's grammar can legitimately decide its own rule here** — there is no entrenched cross-language convention to violate, because mature languages structurally avoid putting two caps at the same level in the first place.

### Recommendation for HRG-1

**Forbid both simultaneously (option a) — plan-time pedagogical rejection — is the defensible and recommended choice for prism, even though pure composition (option b) is what other engines do.** Rationale:

- **The product owner's lean is well-founded and not in conflict with precedent.** The cross-language norm is *composition*, but the cross-language norm is also that languages **structurally never place two caps at the same level** — GoogleSQL keeps the pipe tail as pure pipe operators; the only place two caps coexist is across a subquery boundary where the nesting makes intent unambiguous. Prism's composed form deliberately puts a SQL-clause `LIMIT` and a pipe `| limit` at the *same* level with no nesting to disambiguate intent. That is precisely the ambiguous construct the other languages avoid by design.
- **PrismQL is explicitly teaching-oriented (the `prismql://reference` MCP resource teaches LLM agents).** For a pedagogical DSL consumed by an AI agent, a clear plan-time error ("you specified a row cap twice — put exactly one `limit` at the end of your pipeline") is far more instructive and far less surprising than silently dropping the head limit's value. Silent composition trains the agent that redundant caps are fine; an error trains the correct idiom.
- **Forbidding is the strictly safer default and is reversible.** If you forbid-both in v1, you can later relax to compositional (option b) without breaking any existing valid query. If you ship compositional (option b) in v1, tightening to forbid-both later is a breaking change. Under the production-grade default, ship the constraint that preserves the most future optionality.

**If the human prefers permissiveness over pedagogy, the fallback is option (b), NOT option (c).** Option (b) (pipe-tail `| limit` is the final cap, SQL-head `LIMIT` is an intermediate cap on the head before enrichment) is the only permissive option consistent with every researched language. Option (c) (SQL `LIMIT` wins) has **no precedent** and contradicts the pipe-operator design ethos — do not ship it.

### Implementation / UX implication (forbid-both path)

- **Where to reject:** plan-time (after parse, during logical planning), not parse-time — the parser should accept the grammar so the error can be *semantic and specific* rather than a generic syntax error.
- **Pedagogical error message (suggested):** an `E-QUERY-NNN` code, e.g.
  > `E-QUERY-0XX: redundant row limit. This query caps rows in two places: a SQL `LIMIT N` in the head and a `| limit M` pipe stage. PrismQL requires exactly one row cap. Put a single `| limit` at the end of the pipeline (recommended for composed queries), or use `LIMIT` only in pure SQL-mode queries.`
- **Teach the idiom in `prismql://reference`:** the composition section should show the canonical form (one cap, at the tail) and explicitly call out that doubling the cap is an error — this also gives the HRG-3 round-trip gate a negative example to assert *does not* plan.
- **DataFusion note:** since both forms ultimately become a DataFusion `LIMIT`, forbidding-both also sidesteps having to decide how to lower two `LIMIT`s into one DataFusion plan node — the plan-time rejection is cheaper to implement correctly than the composition lowering.

---

## QUESTION 2 (HRG-3) — Drift-proof reference-doc generation for `prismql://reference`

**Question.** The MCP resource `prismql://reference` teaches grammar + sensor tables + enrichment UDFs. The current static doc drifted (documented syntax that doesn't parse). Options: (a) static Rust string constants for grammar/operator sections next to the grammar + runtime assembly of the infusions section from the registry + a CI gate that fails if any documented example doesn't round-trip through the parser; (b) build-time codegen deriving the reference from grammar/AST at compile time.

### What the research surfaced (2024–2026 practice)

**Grammar-as-source-of-truth doc generation:**
- EBNF/ANTLR → docs and **railroad/syntax-diagram generators** (Rust `railroad` crate; `GuntherRademacher/rr`) can render a grammar into navigable diagrams. Strong for *structural visualization*, weak as the *prose teaching surface* and useless for *per-deployment dynamic content* (UDF registry).
- **tree-sitter** grammars are increasingly used as a single source for editors + docs, but tree-sitter is a *separate* grammar artifact — adopting it for docs would mean maintaining a grammar that is NOT the Chumsky parser, reintroducing exactly the drift class HRG-3 is trying to eliminate (two grammars to keep in sync).
- **Combinator parsers (Chumsky) ARE the grammar.** Key finding: with a parser-combinator like Chumsky, the parser code *is* the grammar specification — there is no separate EBNF artifact to drift from. This means **codegen-from-grammar (option b) has nothing clean to codegen *from*** other than the parser source itself, which is not a tractable AST-to-docs extraction target. This significantly weakens option (b) for prism specifically.

**Example-validation-in-CI patterns (the actual anti-drift mechanism):**
- **Rust doctests** (`cargo test --doc`): examples in `///` doc comments execute in CI. The canonical Rust anti-drift primitive.
- **"Every documented example must round-trip through the parser" gate:** the standard, confirmed pattern is a plain `#[test]` that iterates a **table of example strings** and asserts each parses (and optionally pretty-prints back to itself). This is the highest-signal, lowest-cost drift gate.
- **PRQL book (directly on point):** the PRQL book's PRQL snippets are **compile-tested in CI** — code blocks are extracted and run through the `prqlc` compiler as documentation tests (changelog: "Add a documentation test for prql-compiler"). This is the gold-standard precedent: a teaching DSL book whose every example is proven to compile.
- **DataFusion / sqlparser-rs:** documented SQL examples are exercised via the SQL API in tests/doctests rather than left as prose.
- **`trycmd` / `snapbox`:** snapshot-test CLI example invocations and their output — useful if `prismql://reference` examples are also surfaced as CLI usage, and gives "input parses successfully + output matches snapshot" in one gate.
- **`mdbook test` / `skeptic`:** compile-test fenced code blocks in mdBook/Markdown docs. Relevant only if the reference is *also* shipped as an mdBook; for an MCP resource served from Rust, in-crate `#[test]` round-trip is the more direct gate.

**Runtime-assembled vs build-time-generated (the per-deployment dimension):**
- The enrichment **infusions/UDF set varies by deployment** (registry-driven). A `build.rs` codegen snapshot would bake in the *build host's* registry and be **wrong for any deployment with a different infusion set** — i.e., build-time codegen *cannot* correctly produce the dynamic section and would itself become a new drift source.
- **Runtime assembly from the live registry** is the only correct way to render the infusions section, because it reflects *this deployment's* actual capabilities at serve time. The static grammar/operator sections do not change per deployment and are cheap to keep as constants next to the grammar.

### Recommendation for HRG-3

**Hybrid: static Rust constants for grammar/operator sections (co-located with the parser) + runtime assembly of the infusions section from the live registry + a CI round-trip gate (option a, the hybrid). Reject pure build-time codegen (option b).** Rationale:

- **Build-time codegen (b) is structurally wrong for prism on two counts:** (1) Chumsky-as-grammar gives codegen no clean grammar artifact to derive from; (2) the per-deployment UDF registry means a build-host snapshot is wrong at runtime. Option (b) would *reintroduce* drift, not eliminate it.
- **Runtime assembly is mandatory for the dynamic half.** The infusions section MUST be assembled at serve time from the registry the deployment actually loaded — this is the same registry the planner uses, so it cannot drift from what actually resolves.
- **Static constants are correct for the static half** because the grammar/operator surface is deployment-invariant, and co-locating the constants in the same module as the Chumsky grammar keeps them physically adjacent to the thing they describe (proximity reduces drift; the CI gate proves it).
- **The CI gate is the load-bearing no-drift guarantee** and the part that fixes the original failure (documented syntax that didn't parse).

### Concrete CI-gate mechanism (the part that guarantees no-drift)

A single in-crate Rust test that is the authority. Two complementary assertions:

1. **Positive round-trip gate (catches the original failure):** maintain the reference's example queries as a **table of `&'static str` constants** (one table, the source of truth for examples). A `#[test]` iterates the table and asserts **every example parses through the actual Chumsky parser** (and plans, where planning is cheap/pure). Any documented example that no longer parses → CI red. This is exactly the PRQL-book pattern and directly prevents "documented syntax that doesn't parse."

2. **Negative gate (catches the inverse drift + ties to HRG-1):** assert that example strings the reference says are *errors* (e.g., the double-`limit` query from HRG-1, if forbid-both is chosen) **fail to plan with the documented error code**. This keeps the pedagogical error examples honest.

3. **Registry-parity gate for the dynamic section:** a `#[test]` that builds the reference's infusions section from a known test registry and asserts the rendered UDF names/signatures exactly equal the registry's enumerated capabilities — so the runtime assembler can't silently diverge from the registry schema.

4. **Wire the example table as the single source:** the same `&'static str` example table feeds BOTH the rendered `prismql://reference` resource AND the round-trip test. Because the doc and the test consume the *same* constant, a doc example physically cannot exist without being parse-tested. This is the structural guarantee — not "remember to test the docs," but "the docs and the test are the same array."

**Why not `mdbook test`/`skeptic`/`trycmd` as the primary gate:** those are excellent when the artifact is an mdBook or CLI transcript. `prismql://reference` is an MCP resource rendered from Rust, so an in-crate `#[test]` over the shared example table is more direct, faster, and has zero extra toolchain. Keep `trycmd`/doctests as a secondary layer if/when the same examples appear in CLI help or `///` docs.

---

## Bottom-line recommendations

- **HRG-1 (ADR-043 composition):** **Forbid both caps (option a)** — plan-time `E-QUERY-NNN` pedagogical error directing the user to a single tail `| limit`. Aligns with the product owner's lean; well-founded because mature languages structurally avoid same-level dual caps; reversible toward composition later. **If the human wants permissiveness instead, use option (b) (pipe-tail wins, SQL head = intermediate cap) — never option (c).**
- **HRG-3 (ADR-045 reference generation):** **Hybrid (option a)** — static Rust constants for grammar/operator sections co-located with the Chumsky grammar + runtime assembly of the infusions section from the live registry + a CI gate built on a **shared `&'static str` example table** that (1) round-trips every positive example through the real parser, (2) asserts documented-error examples fail with the right code, and (3) checks infusions-section parity against the registry. **Reject build-time codegen (option b)** — Chumsky-as-grammar and the per-deployment registry make it both unworkable and a new drift source.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) HRG-1: cross-language LIMIT/take/head composition + forbid-vs-precedence in GoogleSQL pipe syntax, PRQL, KQL, Splunk SPL, Spark; (2) HRG-3: grammar-as-source-of-truth doc gen, example-validation-in-CI, runtime-vs-build-time docs, Chumsky CI gates. Both run at `reasoning_effort=high`. |
| Perplexity perplexity_ask | 2 | Confirmed two load-bearing claims: PRQL book compile-tests its examples via prqlc doctest harness; standard Rust round-trip/table-of-examples + trycmd/snapbox gate pattern. |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — (Chumsky/DataFusion API specifics not needed; research was design-pattern level, verified via Perplexity sources incl. docs.rs/chumsky, datafusion.apache.org) |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 1 area | Splunk `head`/`tail` exact semantics partially from model knowledge where the SPL cheat-sheet snippet was thin — flagged as such in the source synthesis; corroborated by the pipeline-composition pattern shared across the other four languages. |

**Total MCP tool calls:** 4 (2 deep research + 2 ask)
**Training data reliance:** low — both questions answered primarily from web-grounded multi-source synthesis with explicit citations; the single training-data lean (SPL command detail) is non-load-bearing for the recommendation (Splunk corroborates, does not drive, the cross-language verdict).

### Key sources

HRG-1: GoogleSQL pipe syntax reference (docs.cloud.google.com/bigquery/.../pipe-syntax; github.com/google/googlesql/blob/master/docs/pipe-syntax.md); VLDB 2024 pipe-syntax paper coverage (simonwillison.net/2024/Aug/24/pipe-syntax-in-sql/, HN 41338877); ZetaSQL query syntax (beam.apache.org/.../zetasql/query-syntax); PRQL book (prql-lang.org/book, /tutorial/relations.html); KQL operators + query limits (learn.microsoft.com/kusto/query/best-practices, /concepts/query-limits, learn-common-operators); Splunk SPL cheat sheet (splunk.com/.../splunk-cheat-sheet).

HRG-3: railroad crate (crates.io/crates/railroad); rr generator (github.com/GuntherRademacher/rr); tree-sitter grammar authoring (jonashietala.se/blog/2024/03/19/lets_create_a_tree-sitter_grammar/); mdBook preprocessors + test (rust-lang.github.io/mdBook/for_developers/preprocessors.html, /cli/test.html); DataFusion SQL API (datafusion.apache.org/library-user-guide/using-the-sql-api.html); sqlparser (docs.rs/sqlparser); chumsky (docs.rs/chumsky); trycmd (crates.io/crates/trycmd); skeptic (users.rust-lang.org/t/.../2163); PRQL repo + changelog (github.com/PRQL/prql, prql-lang.org/book/project/changelog.html).
