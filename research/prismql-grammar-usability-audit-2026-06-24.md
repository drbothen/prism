---
document_type: research/audit
producer: spec-reviewer
date: 2026-06-24
subject: PrismQL (PQL) grammar usability / consistency / naturalness / learnability
branch: develop
status: complete
method: source-of-truth analysis of crates/prism-query Chumsky grammar + crates/prism-mcp teaching
  surfaces, cross-checked against error-taxonomy.md, with a live parser ground-truth probe
  (temporary test, removed after run) confirming PASS/FAIL of every cited query form.
feeds: PrismQL grammar/teaching-surface remediation story (Wave TBD)
related: .factory/research/demo-pre-flight-audit-2026-06-24.md (BLOCKER-002, DISCOVERABILITY-GAP-001,
  AUDIT-001, AUDIT-004)
---

# PrismQL (PQL) Grammar Usability Audit — 2026-06-24

## 0. Scope, method, and the bar

This is a language-design review. The motivating defect: the demo runbook's own enrichment
example (`FROM t WHERE p | enrich fn LIMIT N`) does not parse — `WHERE`/`LIMIT` are pipe stages
in pipe mode, not SQL clauses. That SQL-clause-vs-pipe-stage confusion is the headline failure
class this audit characterizes, and it turns out to be one instance of a broader pattern: **the
teaching surfaces emit query forms the grammar rejects, and omit the forms the grammar requires.**

The bar applied throughout: *a SQL-literate analyst (or an LLM agent), guided only by Prism's own
teaching surfaces — `prism_describe`, `prismql://reference`, and error messages — writes correct
PrismQL on the first or second try.* Anything short of that is a finding.

### Ground truth (live parser probe)

Every claim below about what parses was confirmed by running the real `PrismQlParser::parse`
against the exact strings. Selected results (discriminant: 0=Filter, 1=Sql, 2=Pipe):

| Query | Source | Result |
|---|---|---|
| `SELECT * FROM crowdstrike_detections \| WHERE severity='CRITICAL' \| ORDER BY time DESC \| LIMIT 10` | `prismql://reference` example (line 137) | **PARSE_ERR** offset 37 `found '\|'` |
| `SELECT * FROM crowdstrike_detections WHERE time > NOW() - INTERVAL '24h' LIMIT 25` | `prismql://reference` example (line 95/127) | **PARSE_ERR** offset 50 `expected NULL` |
| `SELECT COUNT(*) FROM armis_devices WHERE timestamp > NOW() - INTERVAL '1h'` | `build_example_query` autogen default | **PARSE_ERR** offset 53 `expected NULL` |
| `FROM cyberint_alerts WHERE iocs_value IS NOT NULL \| enrich threat_score LIMIT 3` | runbook §5.5 form | **PARSE_ERR** offset 21 `found 'W'` |
| `FROM cyberint_alerts \| where iocs_value IS NOT NULL \| enrich threat_score(iocs_value) \| limit 3` | correct pipe form | **PARSE_OK** (Pipe) |
| `SELECT * FROM armis_devices \| limit 5` | SQL + pipe stage | **PARSE_ERR** offset 28 `found '\|'` |
| `FROM cyberint_alerts \| enrich threat_score \| limit 3` | enrich missing `(col)` | **PARSE_ERR** offset 43 `expected '('` |
| `SELECT * FROM cyberint.alerts LIMIT 3` | dot syntax | **PARSE_OK** (parse), rejected later at plan time (E-QUERY-036) |
| `SELECT * FROM armis_devices WHERE severity IN ('high','critical') LIMIT 50` | autogen severity variant | **PARSE_OK** (Sql) |
| `severity = 'HIGH'` | bare filter mode | **PARSE_OK** (Filter) |

These results anchor the findings; they are not inferred from prose.

### The three modes (actual grammar)

PrismQL has **three** parse modes selected by the first token / structure (`filter_parser.rs`
`parse_with_limits`, `is_pipe_mode`):

- **Filter mode** — `[source |] predicate`, e.g. `severity = 'HIGH'`. No SELECT/FROM, no stages.
- **SQL mode** — `SELECT … FROM … [JOIN] [WHERE] [GROUP BY] [HAVING] [ORDER BY] [LIMIT]`. First token `SELECT`. **No `|` permitted anywhere.** No `enrich`.
- **Pipe mode** — `[FROM] source ('|' stage)*`. First token `FROM`, or leading `|`, or a `|` followed by a stage keyword. Stages are lowercase-conceptually keywords: `where sort head tail limit stats dedup fields join enrich`. **This is the only mode that supports `enrich`.**

Mode is decided by `is_pipe_mode`, which scans for an unquoted `|` immediately followed by one of
`PIPE_STAGE_KEYWORDS = [where, sort, head, tail, stats, dedup, fields, join, enrich, limit]`.

---

## 1. Findings by category

Severity is **usability impact**: BLOCKER = analyst/agent cannot succeed from teaching surfaces;
MAJOR = high-friction, multi-retry; MINOR = surprising but recoverable; NIT = polish.

Findings are grouped into three remediation buckets at the end (§3): **fix the grammar/code**,
**fix the reference/docs**, **fix the error messages**. Each finding's recommendation states its
bucket.

### A. Consistency

#### GRAMMAR-001 — Two query modes express the same intent, and they cannot be mixed or bridged (BLOCKER)
- **Dimension:** consistency
- **Location:** `filter_parser.rs::parse_with_limits` (mode detection); `sql_parser.rs` (no `|`); `pipe_parser.rs` (no SQL clauses)
- **Bad input / behavior:** `SELECT * FROM armis_devices | limit 5` → PARSE_ERR offset 28. `FROM t WHERE p | limit 3` → PARSE_ERR. The two surface syntaxes for "filter then limit" are mutually exclusive and there is no diagnostic that says "you mixed modes."
- **Root cause:** SQL mode and pipe mode are entirely separate Chumsky grammars chosen by first-token heuristic. SQL mode's clause parser has no `|` production; pipe mode's stage parser has no `WHERE`/`SELECT`/`ORDER BY`/`LIMIT` clause production. A query that starts `SELECT …` is committed to SQL mode for its entire length; a `|` later is an unexpected token with a clause-list expectation message that never mentions pipes.
- **Why it is the headline problem:** The single most valuable feature (`enrich`) lives **only** in pipe mode, but the most natural/SQL-literate entry (`SELECT … FROM …`) commits to SQL mode where `enrich` is impossible. An analyst who knows SQL will reach for SELECT, then cannot add enrichment, and the error never says "switch to pipe mode." Every demo/runbook/prompt drift below is a symptom of this split.
- **Recommendation (grammar/code + error):** Decide the canonical-mode question (see §4). Minimum viable fix without grammar surgery: when SQL mode hits an unexpected `|`, emit a *mode-bridge* diagnostic — "`|` pipe stages are not valid after a SQL `SELECT` query. Rewrite as pipe mode: `FROM <table> | where … | <stage> …`" with a `normalized_pql` rewrite suggestion. Symmetrically, when pipe mode hits an SQL keyword in stage position (`WHERE`/`SELECT`/`ORDER BY`/`LIMIT` uppercase as a *clause*), say "in pipe mode use the lowercase stage `| where …` / `| limit …`, not the SQL clause."

#### GRAMMAR-002 — `LIMIT`/`limit` means three different things depending on mode (MAJOR)
- **Dimension:** consistency
- **Location:** `ast.rs` `SqlQuery.limit: Option<u64>` (SQL clause); `pipe_parser.rs` `limit_stage` and `head_stage` both → `PipeStage::Limit`
- **Bad input / behavior:** In SQL mode `LIMIT 25` is a trailing clause. In pipe mode `| limit 25` is a stage. Also in pipe mode `| head 25` is an *alias* for `| limit 25` (both map to `PipeStage::Limit`), while `| tail 25` is a distinct `PipeStage::Tail`. So "first N rows" has two spellings (`limit`, `head`) in pipe mode and a third placement (trailing clause) in SQL mode.
- **Root cause:** Splunk-style (`head`/`tail`) and SQL-style (`LIMIT`) vocabularies were both adopted to be familiar, but the union is now redundant within one mode and divergent across modes.
- **Recommendation (docs + accept):** Keep `head`/`tail`/`limit` as documented synonyms (this is genuinely friendly) but the reference MUST state explicitly that `head N == limit N` and that SQL-mode `LIMIT` is a trailing clause while pipe-mode `limit` is a stage. Bucket: fix-docs; no grammar change.

#### GRAMMAR-003 — `client_id` is NOT query syntax; it is an out-of-band tool parameter, and nothing says so (MAJOR)
- **Dimension:** consistency / discoverability
- **Location:** `prism_describe.rs` (`client_id` param); query tool schema (`clients: [...]`); `ast.rs::VirtualField::Client` (`_client`)
- **Bad input / behavior:** The prompt asked whether `client_id = "..."` is "a clause, a trailing modifier, or special syntax." Answer: **none** — there is no client-scoping syntax in PrismQL at all. Scope is injected by the executor from the MCP tool's `clients: [...]` parameter (note: parameter is `clients`, plural list, NOT `client_id` — confirmed in the pre-flight audit §1.2). The only in-query handle is the **`_client` virtual field**, usable in projections/predicates (`ast.rs::field_path_to_expr`), e.g. `where _client = 'org-c'`, but that filters within already-scoped data; it does not *set* scope.
- **Root cause:** Correct architecturally (ADR-006 org isolation injected at plan time, DI-008), but completely undocumented in any teaching surface. `_client`/`_sensor`/`_source_table`/`_source_type`/`_safety_flags` virtual fields are invisible.
- **Recommendation (docs):** Add a "Scope & virtual fields" section to `prismql://reference`: state that scope is set by the `clients` tool parameter, NOT in the query string; document the five `_`-prefixed virtual fields and their use in `where`/projection. Bucket: fix-docs.

#### GRAMMAR-004 — E-QUERY-036 is the only plan-time "not found" error WITHOUT did_you_mean/available lists (MINOR)
- **Dimension:** consistency (error pedagogy)
- **Location:** `prism-core/src/error.rs` `UnknownSourceTable { source_name: String }` vs `TableNotAvailableDetails` (037), `ColumnNotFoundDetails` (038), `EnrichUdfNotFoundDetails` (039)
- **Bad input / behavior:** E-QUERY-037/038/039 all carry `available_*` lists and a Levenshtein `did_you_mean`. E-QUERY-036 (`UnknownSourceTable`) carries only the bad name and a static "Check spelling or register the sensor" string — no candidate list, no suggestion. The taxonomy documents this asymmetry deliberately (036 is the coarse materialization-layer check; 037 is the rich plan-time gate), but to a user the two are indistinguishable in intent ("table doesn't exist") and the quality of help differs sharply.
- **Recommendation (error):** Give E-QUERY-036 the same `did_you_mean` + `available_tables` treatment as 037/038/039 (the registries are available at that layer too), OR ensure the dot-syntax path (which is the dominant 036 trigger — see GRAMMAR-010) always routes to a suggestion-bearing error. Bucket: fix-error-messages.

### B. Naturalness / ergonomics

#### GRAMMAR-005 — `enrich` requires `fn(column)` parentheses; the natural `enrich fn` form fails with a bare `expected '('` (MAJOR)
- **Dimension:** naturalness
- **Location:** `pipe_parser.rs::enrich_stage` (`.delimited_by(just('('), just(')'))`)
- **Bad input / behavior:** `FROM cyberint_alerts | enrich threat_score | limit 3` → PARSE_ERR offset 43 `expected '('`. The error does not name the missing argument or that enrichment is a function over a column.
- **Root cause:** `EnrichStage { infusion: String, field: FieldPath }` mandates exactly one parenthesized field. This mirrors the UDF call shape (good for consistency with SQL-mode `threat_score(col)`), but the runbook's own shorthand `enrich fn` shows even the authors slip into the paren-less form.
- **Recommendation (error + docs):** Keep the grammar (the `(column)` form is the right design — enrichment must know which column to enrich). Improve the error: "`enrich` requires a column argument: `| enrich <infusion>(<column>)`. Example: `| enrich threat_score(iocs_value)`." And document the canonical form prominently in the reference (currently absent entirely — see GRAMMAR-008). Bucket: fix-error + fix-docs.

#### GRAMMAR-006 — `IS NOT NULL` on JSON-list columns is the load-bearing enrich precondition, but its semantics on list/JSON columns are undocumented (MINOR)
- **Dimension:** naturalness
- **Location:** `ast.rs::Predicate::IsNull`; reference Operators table (Json row: "IS NULL, IS NOT NULL — Structural equality only")
- **Bad input / behavior:** The working enrich queries depend on `where iocs_value IS NOT NULL` / `where device_cves_first IS NOT NULL`. Per the pre-flight audit, `iocs_value` is a JSON-list-as-string and `device_cves_first` is a scalar projection of a list. Whether `IS NOT NULL` means "present", "non-empty list", or "first element exists" is not specified, and the reference only says "Structural equality only" for JSON — which does not explain list behavior.
- **Recommendation (docs):** Document the null-semantics of JSON/list columns explicitly: what `IS NOT NULL` returns for an empty list vs absent field vs `[null]`. This is the gating predicate for the flagship feature; ambiguity here is high-cost. Bucket: fix-docs (and confirm code behavior matches the documented semantics — route to data-engineer if they diverge).

#### GRAMMAR-007 — Rich, genuinely-friendly operators exist but are completely hidden (MAJOR)
- **Dimension:** naturalness / discoverability
- **Location:** `ast.rs::Predicate` (StringOp CONTAINS/ICONTAINS/STARTSWITH/ISTARTSWITH/ENDSWITH/IENDSWITH; Regex `=~`/MATCHES; In; Between; Cidr `IN CIDR`; Has; Missing; Wildcard auto-promotion); `pipe_parser.rs` stats (`distinct_count`, `percentile`, multi-agg `AS`), `dedup`, `sort`, `fields +/-`
- **Bad input / behavior:** PrismQL is far richer than SQL-92 in security-relevant ways: `field CONTAINS "x"`, `field =~ "regex"`, `field IN CIDR "10.0.0.0/8"`, `HAS field`, `MISSING field`, `BETWEEN`, wildcard auto-promotion (`field = "10.0.*"`), `stats distinct_count(x), percentile(latency, 95) by sensor`, `dedup device_id`, `fields - secret_col`. The `prismql://reference` Operators table documents only `= != < <= > >= LIKE NOT LIKE IN IS [NOT] NULL`. Every security-analyst-friendly operator is undiscoverable.
- **Root cause:** Reference doc was written against an early SQL-only conception and never updated as the operator set grew (the AST doc-comment §4 table is the real spec; the reference is a stale subset).
- **Recommendation (docs):** Regenerate the reference Operators section from `ast.rs::Predicate`'s documented operator table (it is already a maintained source of truth). Bucket: fix-docs.

### C. Learnability / discoverability

#### GRAMMAR-008 — `prismql://reference` contains ZERO enrichment / pipe-mode / UDF content (BLOCKER)
- **Dimension:** discoverability
- **Location:** `crates/prism-mcp/src/pql_reference.md` (161 lines, ~6.5KB)
- **Confirmed:** `grep -ci enrich` = 0; `grep -ci 'threat_score|cvss|infusion|UDF'` = 0; only lowercase pipe-stage keyword occurrence is the single (and *wrong*) `| WHERE` example. (Reconfirms demo-pre-flight DISCOVERABILITY-GAP-001.)
- **Bad input / behavior:** An analyst/agent following the documented discovery path — `query` tool description points to `prismql://reference` — cannot find: that pipe mode exists, that `enrich` exists, the names of any enrichment functions, the `(column)` call shape, or any pipe stage beyond the broken `| WHERE` example. The flagship feature is discoverable ONLY by reading parse errors or prior knowledge.
- **Root cause:** Reference predates the pipe/enrich pivot (S-1.14 / S-DEMO-ENRICHMENT-PIVOT) and was never updated.
- **Recommendation (docs):** Add full pipe-mode and enrich grammar to the reference: the three modes, all pipe stages, the `enrich <infusion>(<column>)` form, and a discovery pointer (`list_infusions` / the registry) for *which* infusions are available in a given deployment. Bucket: fix-docs. (Note: `list_infusions` currently hangs — pre-flight BLOCKER-004 — so the doc must give an alternative discovery path until that is fixed.)

#### GRAMMAR-009 — `prismql://reference` BNF and examples are *actively wrong*, not merely incomplete (BLOCKER)
- **Dimension:** discoverability / consistency
- **Location:** `pql_reference.md` §"Clause Grammar (BNF)" lines 43–44, and §"Multi-stage pipeline" example line 137
- **Bad input / behavior:** The reference BNF defines `pipeline ::= query "|" query_stage` where `query_stage ::= where_clause | orderby_clause | limit_clause | select_clause` — i.e. it claims pipe stages are **SQL clauses** (`WHERE`, `ORDER BY`, `LIMIT`, `SELECT`). The grammar does the opposite: pipe stages are lowercase Splunk-style keywords (`where sort limit …`) and SQL clauses are illegal after `|`. The worked example `SELECT * FROM … | WHERE … | ORDER BY time DESC | LIMIT 10` (line 137) **does not parse** (PARSE_ERR offset 37, confirmed). The reference is teaching the exact bug the runbook fell into.
- **Root cause:** The BNF documents an aspirational/incorrect model where SQL and pipe compose. The implemented model is two disjoint grammars (GRAMMAR-001).
- **Recommendation (docs):** Rewrite the BNF to match `pipe_parser.rs` / `sql_parser.rs`. Add a prominent "SQL mode and pipe mode do not mix" callout with the wrong-vs-right pair. This is the single highest-leverage doc fix — it directly caused RUNBOOK-DRIFT-001. Bucket: fix-docs.

#### GRAMMAR-010 — `prism_describe` emits a table name (`alerts`) that the grammar rejects; FROM needs the prefixed name (`cyberint_alerts`) (BLOCKER)
- **Dimension:** discoverability / consistency
- **Location:** `prism_describe.rs` `TableDescriptor.name` (unqualified) vs `SourceRefKind::classify` (FROM expects `sensor_table` underscore form); demo-pre-flight AUDIT-001
- **Bad input / behavior:** `prism_describe` reports `name: "alerts"`; the analyst writes `FROM alerts` or `FROM cyberint.alerts` and gets E-QUERY-036. The correct form `cyberint_alerts` is not the primary token shown. Dot syntax (`cyberint.alerts`) parses fine but is rejected at plan time. So the tool whose entire job is discovery hands you a token the engine won't accept.
- **Root cause:** `prism_describe` surfaces the short table name; the query engine's source classification keys on the `{sensor}_{table}` underscore convention. The `example_query` field DOES use the prefixed form, but the prominent `name` field does not.
- **Recommendation (code):** Have `prism_describe` surface the FROM-ready fully-qualified name (`cyberint_alerts`) as the primary `name`/identifier, or add an explicit `from_name` field and document that FROM uses it. Bucket: fix-code (prism_describe), route to product-owner for the BC-2.10.012 field contract. Highest-impact discoverability fix after GRAMMAR-009.

#### GRAMMAR-011 — `build_example_query` and `pql_hints` emit unparseable `NOW() - INTERVAL` SQL as the DEFAULT example (BLOCKER)
- **Dimension:** discoverability / naturalness
- **Location:** `prism_describe.rs::build_example_query` (line 449–450) and `pql_reference.md` §"Datetime Arithmetic" (lines 77–96, 127)
- **Bad input / behavior:** The autogen default example for any table (and the zero-column fallback) is `SELECT COUNT(*) FROM <t> WHERE timestamp > NOW() - INTERVAL '1h'`. This **does not parse** (PARSE_ERR offset 53 `expected NULL`). There is **no `NOW()` or `INTERVAL` support anywhere in the parser** (confirmed: zero matches in `sql_parser.rs`/`filter_parser.rs`/`ast.rs`, zero tests). The reference devotes a whole section to `NOW() - INTERVAL '24h'`. So Prism's *machine-generated, per-table* "here's how to query this" hint is a query the engine rejects, and the reference's time-filtering section is entirely fictional.
- **Root cause:** Relative-time syntax was documented and used in example generation but never implemented in the grammar. The actual grammar has `Duration` literals (`30s`, `24h`, `7d`) and strict RFC-3339 `Timestamp` literals (`ast.rs::TimestampLiteral`, offset required), but no `NOW()` function and no `INTERVAL` keyword.
- **Recommendation (TWO valid paths, pick one — this is a real decision, surface to PO/architect):**
  1. **Implement `NOW()` + relative duration** in the grammar so the documented and autogenerated examples become true (preferred — `NOW() - 24h` is the single most natural security-analyst time filter, and the Duration literal type already exists). Bucket: fix-grammar/code; route to architect (new AST node) + product-owner (BC).
  2. **OR** stop generating/ documenting unsupported syntax: change `build_example_query` to emit only parseable forms (e.g. an absolute RFC-3339 timestamp or no time filter), and delete the Datetime Arithmetic section from the reference. Bucket: fix-code + fix-docs.
  Path 1 is the production-grade default (closes the gap rather than papering over it). Either way, the current state — autogen + reference both shipping invalid syntax — must not stand.

#### GRAMMAR-012 — Enrich function names are not discoverable from any working surface (MAJOR)
- **Dimension:** discoverability
- **Location:** `InfusionRegistry`; `list_infusions` tool (hangs per pre-flight BLOCKER-004); reference (no UDF list)
- **Bad input / behavior:** To write `enrich threat_score(iocs_value)` you must know the literal name `threat_score`. The reference lists no infusions; `list_infusions` hangs; `list_capabilities` returns `client_registered: False` (pre-flight MAJOR-001). The only way to learn `threat_score`/`cvss_base_score` exists is the demo runbook or source. E-QUERY-039 *does* return an `available_infusions` list — but only after you've already guessed a wrong name.
- **Recommendation (code + docs):** Fix `list_infusions`/`list_capabilities` discovery (route to implementer; tracked by pre-flight BLOCKER-004/MAJOR-001), AND have `prism_describe` include a per-client `available_infusions` array alongside `pql_hints` so enrichment is discoverable on the primary discovery call. Bucket: fix-code; route to product-owner for the prism_describe field contract.

#### GRAMMAR-013 — A correct-name enumeration of facts a user must know but cannot discover (MAJOR, summary finding)
- **Dimension:** discoverability
- **Location:** aggregate of GRAMMAR-003/006/007/008/010/011/012
- **Facts a user currently must know out-of-band** (none discoverable from `prism_describe` + `prismql://reference` + errors alone):
  1. Pipe mode exists and is separate from SQL mode (GRAMMAR-009).
  2. `enrich` exists and lives only in pipe mode (GRAMMAR-008).
  3. The names of enrichment infusions (`threat_score`, `cvss_base_score`, …) (GRAMMAR-012).
  4. `enrich` takes `(column)` (GRAMMAR-005/008).
  5. FROM needs the `sensor_table` underscore name, not the `name` field `prism_describe` shows (GRAMMAR-010).
  6. Scope is the `clients` tool param, not query syntax; `_client`/`_sensor` virtual fields exist (GRAMMAR-003).
  7. `NOW()`/`INTERVAL` do NOT work despite being in the reference and autogen examples (GRAMMAR-011).
  8. The rich operators (CONTAINS, =~, IN CIDR, HAS/MISSING, BETWEEN, percentile, distinct_count, dedup, sort, fields±) exist (GRAMMAR-007).
  9. `device_cves_first` is a scalar projection of a CVE list, and `iocs_value` is a JSON-list-as-string — and what `IS NOT NULL` means on them (GRAMMAR-006).
  10. Device-ID format `dev-0196f4b2-{seed}-{N}` (deployment-specific; lower priority, demo-data).
- **Recommendation:** The remediation story should treat "an analyst can author a correct enrich query from teaching surfaces alone" as its acceptance test. Each numbered fact above maps to a sub-finding fix. Bucket: mixed (see §3).

### D. Error-message pedagogy

#### GRAMMAR-014 — Mode-mixing parse errors leak the internal clause-expectation list instead of naming the real fix (BLOCKER)
- **Dimension:** error pedagogy
- **Location:** `sql_parser.rs` Chumsky clause parser; `error_recovery::rich_to_parse_error`
- **Bad input / behavior:** `SELECT * FROM t | WHERE …` →
  `parse error at offset 37: found '|' expected '"AS"','"as"', … '"WHERE"','"where"', … '"LIMIT"','"limit"', or end of input`.
  The error dumps the raw Chumsky token-expectation set (every clause keyword, both cases) and never says the two true facts: (a) `|` is not legal in SQL mode, (b) to use pipe stages, start with `FROM … | …`. The runbook author saw exactly this class of error and could not act on it.
- **Root cause:** Default Chumsky `Rich` expectation formatting is surfaced verbatim. There is no mode-aware diagnostic layer.
- **Recommendation (error):** Add a post-parse heuristic: if a SQL-mode parse fails at a `|` token, replace the generic expectation dump with the mode-bridge message from GRAMMAR-001, ideally with a `normalized_pql` field containing the pipe-mode rewrite. Symmetric handling for an uppercase SQL clause appearing in pipe stage position. Bucket: fix-error-messages. Highest-leverage error fix.

#### GRAMMAR-015 — E-QUERY-039 (unknown enrich fn) is excellent; the parse-time enrich errors are not (MINOR)
- **Dimension:** error pedagogy
- **Location:** E-QUERY-039 (`EnrichUdfNotFoundDetails`, plan-time, has `available_infusions` + `did_you_mean`) vs `enrich`-without-parens parse error (GRAMMAR-005)
- **Observation:** When you get the infusion *name* wrong, E-QUERY-039 is exemplary (available list + Levenshtein suggestion + retry hint). But when you get the *shape* wrong (`enrich fn` without `(col)`), you fall back to a bare `expected '('` parse error with none of that pedagogy. The quality cliff is between plan-time (rich) and parse-time (raw Chumsky) errors generally.
- **Recommendation (error):** The remediation should aim to bring parse-time errors for the common pipe-stage shapes up to the plan-time error standard (named fix + example). Bucket: fix-error-messages.

#### GRAMMAR-016 — `normalized_pql` is documented as an error field but is absent in practice (MINOR)
- **Dimension:** error pedagogy
- **Location:** demo-pre-flight §2.2 ("No `normalized_pql` field present in E-QUERY-038"); error-taxonomy E-QUERY-038 row (no `normalized_pql` in field list)
- **Observation:** A `normalized_pql` echo (the canonical re-rendering of the parsed query) would be the ideal teaching aid for mode/shape confusion — it lets the agent see what the engine *thinks* it asked. The prompt explicitly hoped errors would "show a `normalized_pql`." It is not present on the column/table errors.
- **Recommendation (code):** Add `normalized_pql` to structured error responses where a partial/alternative parse is available (especially the mode-bridge case in GRAMMAR-014). Bucket: fix-code; route to product-owner for the error-response contract.

### E. Grammar-reference completeness

(Covered by GRAMMAR-007/008/009/011/012; consolidated gap list.)

#### GRAMMAR-017 — Reference completeness gap list (MAJOR, summary)
- **Dimension:** completeness
- **Location:** `pql_reference.md`
- **Every construct the reference omits or misstates** (vs `ast.rs` / `pipe_parser.rs` / `filter_parser.rs`):
  - **Modes:** filter mode (`severity='HIGH'`) entirely undocumented; pipe mode misdocumented as SQL-clauses-after-pipe (GRAMMAR-009).
  - **Pipe stages:** `sort head tail stats dedup fields join enrich` — all absent; only the wrong `| WHERE` shown.
  - **Operators:** CONTAINS/ICONTAINS/STARTSWITH/ISTARTSWITH/ENDSWITH/IENDSWITH, `=~`/MATCHES regex, `IN CIDR`, HAS, MISSING, BETWEEN, wildcard auto-promotion — all absent.
  - **Aggregates:** `distinct_count`, `percentile(f,p)`, multi-agg with `AS`, `BY` grouping in pipe `stats` — absent (only COUNT/SUM/AVG/MIN/MAX listed, SQL-only).
  - **Literals:** Duration (`30s`/`24h`/`7d`), strict RFC-3339 timestamps (offset required) — undocumented; instead the *unsupported* `NOW()`/`INTERVAL` is documented (GRAMMAR-011).
  - **Enrich/UDFs:** the entire enrichment surface (GRAMMAR-008/012).
  - **Virtual fields & scope:** `_client`/`_sensor`/`_source_table`/`_source_type`/`_safety_flags`; `clients` tool-param scoping (GRAMMAR-003).
  - **Composite sources:** `EVENTS`/`ALERTS`/`DEVICES`/`ASSETS`/`SESSIONS` cross-sensor virtual sources (`ast.rs::CompositeSource`) — undocumented.
  - **Internal tables:** `prism_alerts`/`prism_cases`/etc. (`InternalTable`) — undocumented.
  - **Quoting:** single vs double quotes for strings (both accepted per `Literal::String`) — unstated.
- **Recommendation (docs):** Treat the reference as generated-from-or-validated-against the grammar. Ideally add a CI check that every `prismql://reference` example string round-trips through `PrismQlParser::parse` (this would have caught GRAMMAR-009 and GRAMMAR-011 mechanically). Bucket: fix-docs + fix-code (CI gate). Route the CI-gate idea to the architect.

### F. Naming / keyword consistency

#### GRAMMAR-018 — Case convention is inconsistent across modes and undocumented (MINOR)
- **Dimension:** naming
- **Location:** SQL keywords (conventionally UPPER in examples), pipe stage keywords (lowercase in examples / `PIPE_STAGE_KEYWORDS`), but **all keywords are case-insensitive** in the actual parser (`kw_ci`, `text::keyword` with both cases, `eq_ignore_ascii_case`).
- **Observation:** The grammar accepts `WHERE`/`where`/`Where` everywhere (confirmed: `FROM t | WHERE … | LIMIT 3` parses OK in pipe mode). Yet the reference uses UPPER SQL clauses and the runbook author conflated "uppercase WHERE = SQL clause." The convention is purely stylistic, but the *appearance* of UPPER=SQL/lower=pipe is a trap because it is not enforced and not explained.
- **Recommendation (docs):** State the case-insensitivity rule explicitly, and adopt one house style in all examples (recommend: lowercase pipe stages, UPPER SQL keywords) so the visual cue is at least *consistent*, even though the parser does not require it. Bucket: fix-docs.

#### GRAMMAR-019 — Column naming convention (clean identifiers vs source paths) is unstated (NIT)
- **Dimension:** naming
- **Location:** `prism_describe` column lists (e.g. `behaviors_ioc_value`, `device_cves_first`) vs `FieldPath` dot-notation (`device.hostname`)
- **Observation:** Some columns are flattened snake_case (`behaviors_ioc_value`), some imply nesting via dot paths (`device.hostname`). When to use a dot path vs a flat name is not documented; an analyst cannot tell from the name whether `device.hostname` or `device_hostname` is correct without `prism_describe`. (Lower priority since `prism_describe` is authoritative for column names.)
- **Recommendation (docs):** Note that column names come verbatim from `prism_describe`; analysts should not construct dot paths unless `prism_describe` shows them. Bucket: fix-docs.

---

## 2. Findings summary matrix

| ID | Dimension | Severity | Bucket |
|----|-----------|----------|--------|
| GRAMMAR-001 | consistency | BLOCKER | grammar/code + error |
| GRAMMAR-002 | consistency | MAJOR | docs |
| GRAMMAR-003 | consistency/disc. | MAJOR | docs |
| GRAMMAR-004 | error pedagogy | MINOR | error |
| GRAMMAR-005 | naturalness | MAJOR | error + docs |
| GRAMMAR-006 | naturalness | MINOR | docs (+verify code) |
| GRAMMAR-007 | naturalness/disc. | MAJOR | docs |
| GRAMMAR-008 | discoverability | BLOCKER | docs |
| GRAMMAR-009 | discoverability | BLOCKER | docs |
| GRAMMAR-010 | discoverability | BLOCKER | code (prism_describe) |
| GRAMMAR-011 | discoverability | BLOCKER | grammar/code OR code+docs (decision) |
| GRAMMAR-012 | discoverability | MAJOR | code + docs |
| GRAMMAR-013 | discoverability | MAJOR | mixed (summary) |
| GRAMMAR-014 | error pedagogy | BLOCKER | error |
| GRAMMAR-015 | error pedagogy | MINOR | error |
| GRAMMAR-016 | error pedagogy | MINOR | code |
| GRAMMAR-017 | completeness | MAJOR | docs + CI gate |
| GRAMMAR-018 | naming | MINOR | docs |
| GRAMMAR-019 | naming | NIT | docs |

Counts: **6 BLOCKER, 7 MAJOR, 5 MINOR, 1 NIT** (19 findings).
By dimension: consistency 4, naturalness 3, discoverability 6, error pedagogy 4, completeness 1, naming 2 (one finding spans two; GRAMMAR-013 is an aggregate).

---

## 3. Remediation buckets (for the follow-up story)

### Bucket 1 — Fix the grammar / code (engine behavior)
- GRAMMAR-010: `prism_describe` surfaces FROM-ready `sensor_table` name. (product-owner: BC-2.10.012 field; implementer)
- GRAMMAR-011 (if Path 1 chosen): implement `NOW()` + relative-duration in the grammar. (architect: AST node; product-owner: BC; implementer) — OR Path 2 lives in buckets 1+2.
- GRAMMAR-012: surface `available_infusions` in `prism_describe`; fix `list_infusions` hang. (product-owner field contract; implementer; ties to pre-flight BLOCKER-004)
- GRAMMAR-016: add `normalized_pql` to structured errors. (product-owner error contract; implementer)
- GRAMMAR-017 (CI gate): add a test that every `prismql://reference` example round-trips through `PrismQlParser::parse`. (architect/devops)

### Bucket 2 — Fix the reference / docs (`prismql://reference`, prompts)
- GRAMMAR-009 (rewrite BNF + add SQL-vs-pipe callout) — highest leverage.
- GRAMMAR-008 (add pipe + enrich grammar).
- GRAMMAR-007/017 (regenerate Operators + completeness sections from `ast.rs`).
- GRAMMAR-002 (head==limit; clause-vs-stage placement).
- GRAMMAR-003 (scope + virtual fields section).
- GRAMMAR-006 (JSON/list `IS NOT NULL` semantics).
- GRAMMAR-011 (Path 2: delete Datetime Arithmetic / fix autogen).
- GRAMMAR-018/019 (case-insensitivity rule + column-naming note).
- Also regenerate `triage_alerts` / `cross_client_status` prompt bodies off the real table registry (pre-flight AUDIT-004; dot-syntax → underscore).

### Bucket 3 — Fix the error messages
- GRAMMAR-014: mode-bridge diagnostic on `|` in SQL mode (and SQL-clause in pipe stage). Highest-leverage error fix.
- GRAMMAR-001: same mode-bridge messaging.
- GRAMMAR-005/015: `enrich` shape error names the `(column)` fix with example; raise parse-time pipe-stage errors to the plan-time pedagogy standard.
- GRAMMAR-004: give E-QUERY-036 the `did_you_mean` + `available_tables` treatment (parity with 037/038/039).

---

## 4. Recommendation on the SQL-mode-vs-pipe-mode consistency question

**The dual-mode design is worth keeping, but the modes must be bridged — the current "two
disjoint grammars selected by first-token, with no diagnostic when you cross the streams" is the
root cause of the headline defect and of GRAMMAR-001/009/014.**

Reasoning:
- **Pipe mode is the canonical mode and should be presented as primary.** It is the only mode that
  supports `enrich` (the flagship feature), it composes naturally (security analysts think in
  pipelines — Splunk/SPL, KQL, `|`-chained shells), and it is the mode every working enrichment
  query uses. The reference, `prism_describe` hints, and prompts should lead with pipe mode.
- **SQL mode is worth retaining as a familiarity on-ramp** for the simple `SELECT … FROM … WHERE …
  LIMIT` case — it lowers the barrier for SQL-literate analysts and is already implemented, tested,
  and used throughout the demo for basic reads. Removing it would be a regression in
  approachability for the 80% "just show me rows" case.
- **But the boundary must stop being a silent foot-gun.** Two concrete bridges, in priority order:
  1. **(Bucket 3, do first, cheap)** Mode-aware error: any SQL-mode parse that dies on `|` returns
     the pipe-mode rewrite as `normalized_pql` plus a one-line explanation. This alone would have
     prevented RUNBOOK-DRIFT-001 and is achievable without grammar surgery.
  2. **(Bucket 1, larger, higher payoff)** Make the modes actually composable: allow pipe stages
     to follow a `SELECT … FROM …` head (lower the SQL SELECT into a pipe source so
     `SELECT … FROM t | enrich fn(c) | limit N` parses). This is the form the reference *already
     documents* (GRAMMAR-009) and the form analysts intuitively reach for. If feasible, it
     converts the reference's current lie into the truth and eliminates the entire mode-mixing
     finding class. Route the feasibility call to the architect (it touches the SQL/pipe AST
     boundary and the pipe-to-SQL emitter `pipe_sql_emitter.rs`).

**Net:** keep both modes; make pipe mode canonical in all teaching surfaces; add the mode-bridge
error immediately (Bucket 3); and seriously evaluate true SQL→pipe composition (Bucket 1) so the
documented-and-intuitive `SELECT … | …` form becomes real rather than a parse error. Do **not**
ship the remediation with the modes still silently disjoint — that leaves the foot-gun loaded.

---

## 5. Note on what is genuinely good (constructive balance)

- The **plan-time error taxonomy (E-QUERY-037/038/039)** is exemplary: org-scoped `available_*`
  lists + Levenshtein `did_you_mean` + retry hints + explicit gate ordering. The remediation should
  hold parse-time and E-QUERY-036 errors to this same bar, not lower this one.
- The **operator set itself** (CIDR membership, regex, CONTAINS family, HAS/MISSING, percentile,
  dedup, fields±) is well-chosen for security analysis and more expressive than SQL-92. The problem
  is purely that it is hidden, not that it is wrong.
- **Strict RFC-3339 timestamp parsing** (offset required, no silent-UTC) is a correct, safe choice.
- The **`#[non_exhaustive]` + constructor discipline** on AST types is clean and forward-compatible.

The grammar is good. The teaching surfaces lie about it. Fix the lies and bridge the modes.
