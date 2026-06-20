# Remove-Uncertainty Report — S-DEMO-PRISMQL-ONBOARDING-001-B

| Field | Value |
|-------|-------|
| Story | S-DEMO-PRISMQL-ONBOARDING-001-B (Query Engine L4 — E-QUERY-038 gate + pedagogical enrichments + normalized_pql) |
| Pass type | REMOVE-UNCERTAINTY (project rule D-1110), pre-TDD |
| Date | 2026-06-20 |
| Agent | research-agent |
| Codebase ref | develop@f6739764 |
| Primary sources | crates/prism-core, prism-query, prism-mcp; error-taxonomy.md v1.91 (modified 2026-06-19); Cargo.lock; docs.rs/crates.io; Perplexity research |
| Story version after pass | 1.0 → 1.1 (register with state-manager) |

## Summary

- **Uncertainties scanned:** 11
- **Resolved by codebase / taxonomy (primary source):** 8
- **Resolved by external research (docs.rs / crates.io / Perplexity):** 2 (cross-confirmed against Cargo.lock)
- **Inconclusive:** 0
- **Architecture/scope flag (routed to specialist, NOT auto-edited):** 1 (the TableRegistry column-schema gap)
- **Spec edits applied (LOW-RISK):** 8 distinct edits across frontmatter, Previous Story Intelligence, Library/Framework table, pre-flight tasks, Phase 5 task, File Structure table, Token Budget table, changelog + version bump.

The story is in unusually good shape: error-taxonomy.md v1.91 was updated the same week and **already ratifies** E-QUERY-037 and E-QUERY-038 (full Message Format, the `ColumnNotFound` variant signature, ADR-041 L4 references, boxing note, gate ordering). Most of this pass is confirming the story's claims against live code rather than correcting errors. The two material problems found were: (a) factually-wrong + volatile line-number citations for "ast.rs display affordances"; and (b) the TableRegistry column-schema gap (flagged, not fixed).

---

## Uncertainty Register

### U-01 — `PrismError::ColumnNotFound` variant does not yet exist (NEW)
- **Type:** API assumption (new type).
- **Resolution (codebase):** CONFIRMED. `crates/prism-core/src/error.rs` E-QUERY section (lines ~497–826) has NO `ColumnNotFound` variant. Story correctly treats it as net-new. The exact signature `{ column, table, client_id, available_columns: Vec<String>, did_you_mean: Option<String> }` is ALSO pre-ratified in error-taxonomy.md v1.91 E-QUERY-038 row (line 257).
- **Edit:** none needed (story is correct).
- **Source:** prism-core/src/error.rs; error-taxonomy.md:257.

### U-02 — `PrismError` enum `#[non_exhaustive]` discipline
- **Type:** Feature claim ("enum already non_exhaustive — do NOT re-add").
- **Resolution (codebase):** CONFIRMED. `#[non_exhaustive]` is at the enum level (error.rs:98). Story's Architecture Compliance Rule and Phase 1 NOTE are correct.
- **Edit:** none.
- **Source:** prism-core/src/error.rs:97–99.

### U-03 — `result_large_err` boxing precedent (TableNotAvailableDetails)
- **Type:** Stale architecture pattern check.
- **Resolution (codebase):** CONFIRMED. `TableNotAvailableDetails` (error.rs:11–91) is boxed inside `PrismError::TableNotAvailable(Box<…>)` (error.rs:659) exactly as the story references, with the clippy::result_large_err rationale documented inline. The story's guidance ("if ColumnNotFound triggers the lint, box following TableNotAvailableDetails precedent") is accurate.
- **Edit:** none.
- **Source:** prism-core/src/error.rs:11–91, 640–659.

### U-04 — error_mapping.rs `-32602` explicit arm before `#[non_exhaustive]` catch-all
- **Type:** API assumption (MCP error mapping).
- **Resolution (codebase):** CONFIRMED. `crates/prism-mcp/src/error_mapping.rs` has explicit `INVALID_PARAMS` (-32602) arms for `QueryParseFailed` (line 28), `TableNotAvailable(..)` (line 140), `UnknownSourceTable` (124), `ClientNotFound` (175), etc., all BEFORE the `-32000` catch-all (line ~385). The `TableNotAvailable` arm is the exact precedent for the new `ColumnNotFound` arm. The pattern of an explicit comment "MUST be explicit: #[non_exhaustive] fall-through would regress to -32000" is already established (line 121–122, 128–129).
- **Edit:** none.
- **Source:** prism-mcp/src/error_mapping.rs:26–185, 385.

### U-05 — E-QUERY-037 plan-time gate exists and is the colocation point
- **Type:** API assumption.
- **Resolution (codebase):** CONFIRMED. `check_table_availability` (engine.rs:1101–1150) is the plan-time E-QUERY-037 gate; it delegates to `registry.check_availability_gate(query_str, org_scope, resolved_spec_map)`. Engine-level test `S-3.13 CRIT-1` (engine.rs:1441+) and SEC-001 org-scope test (engine.rs:1989+) confirm the gate fires E-QUERY-037 before fan-out. E-QUERY-038 colocation here is sound.
- **Edit:** none for the gate location; see U-10 for the method-name correction.
- **Source:** prism-query/src/engine.rs:1101–1150, 1441–1528, 1989–2090.

### U-06 — ColumnType canonical variants for E-QUERY-002 `valid_operators_for_type`
- **Type:** Stale shadow-enum risk (CLAUDE.md ColumnType convention).
- **Resolution (codebase):** CONFIRMED. `prism_core::column::ColumnType` variants are `String / Integer / Float / Boolean / Datetime / Json` (column.rs:19–31) — exactly matching the story AC-003 operator table. The story uses the canonical enum, NOT the retired `prism_spec_engine::types::ColumnType` shadow or the internal `prism_core::types::ColumnType`. No forbidden-pattern reference introduced.
- **Edit:** none.
- **Source:** prism-core/src/column.rs:19–31; CLAUDE.md §ColumnType canonical naming.

### U-07 — `strsim = "0.11"` direct dep + resolved version + API signature
- **Type:** Version pin + API.
- **Resolution (codebase + research):** CONFIRMED. Cargo.toml line 84 `strsim = "0.11"` (D-1163 comment present). Cargo.lock resolves **0.11.1** (Cargo.lock:6516–6517). `strsim::levenshtein(a: &str, b: &str) -> usize` confirmed via crates.io/lib.rs (Perplexity citations [12][15]). NOTE: story cited "line 84" — line numbers are volatile (TD-VSDD-091); softened to "D-1163; resolves 0.11.1 per Cargo.lock."
- **Edit:** APPLIED — Library table + version-pinning note + pre-flight task de-pinned from "line 84" and annotated with confirmed 0.11.1 + signature citation.
- **Source:** prism-query/Cargo.toml:81–84; Cargo.lock:6516–6517; docs.rs/crates.io strsim.

### U-08 — `chumsky` version + whether it provides AST re-serialization
- **Type:** Version pin + critical feature claim (the normalized_pql core risk).
- **Resolution (codebase + research):** CONFIRMED on both. Cargo.toml line 37 `chumsky = "0.12"`; Cargo.lock resolves **0.12.0** (Cargo.lock:1026–1027), the only published 0.12.x release (Perplexity: no 0.12.1 found). Perplexity research + docs.rs confirm chumsky 0.12.0 is **purely a parser-combinator library with NO built-in AST pretty-printer / Display / re-serializer** — so the normalized_pql canonicalizer is genuinely net-new. Independently verified in-repo: a grep for `impl Display`/`to_pql`/`normalize`/`to_canonical` on ast.rs AST nodes returns ZERO matches.
- **Edit:** APPLIED — Library table + version note + risk_mitigation + Previous Story Intelligence updated to cite the confirmed net-new status and chumsky-has-no-reserializer fact.
- **Source:** prism-query/Cargo.toml:37; Cargo.lock:1026–1027; Perplexity research 2026-06-20 (docs.rs/crates.io chumsky); in-repo grep on ast.rs.

### U-09 — FALSE + volatile "ast.rs:681 / ast.rs:1099 display affordances"
- **Type:** Stale/volatile citation (TD-VSDD-091) AND factual error.
- **Resolution (codebase):** The story's risk_mitigation, Previous Story Intelligence, Token Budget, Phase 5 task, and File Structure table all claimed "partial display affordances / raw display strings at ast.rs lines 681, 1099." VERIFIED FALSE: ast.rs line ~681 is a doc-comment inside the `SourceRef` struct; line ~1099 is a doc-comment inside the `TimestampLiteral` struct. Neither is a Display impl or "raw display string." There are no display affordances anywhere in ast.rs. These citations would have misdirected the implementer into hunting for non-existent leverage. Also a TD-VSDD-091 anti-volatile-pin violation (raw line numbers in narrative spec).
- **Edit:** APPLIED — all five citations retracted and replaced with the verified "ZERO Display impls; build net-new" guidance + a behavioral-anchor framing.
- **Source:** prism-query/src/ast.rs:670–694, 1090–1109; grep (no Display matches).

### U-10 — `Arc<dyn TableRegistry>` / `filter_to_org_visible()` API claims
- **Type:** API assumption (type shape + method names).
- **Resolution (codebase):** Two corrections. (a) `TableRegistry` is a **concrete struct** (`#[non_exhaustive]`, table_registry.rs:66–72), NOT a trait — the engine receives `Option<&TableRegistry>` (engine.rs:1131), not `Arc<dyn TableRegistry>`. (b) The single method `filter_to_org_visible()` does NOT exist; the actual crate-private helpers are `filter_to_org_visible_sensors()` (table_registry.rs:565) and `filter_to_org_visible_tables()` (table_registry.rs:609). The org-scope plumbing the gate uses is `check_availability_gate(query_str, org_scope, resolved_spec_map)`.
- **Edit:** APPLIED — Previous Story Intelligence corrected on both points; gate/method names made accurate.
- **Source:** prism-query/src/table_registry.rs:66–72, 565, 609; engine.rs:1129–1150.

### U-11 — E-QUERY-001 variant name / Display format
- **Type:** API assumption.
- **Resolution (codebase):** The live variant is `PrismError::QueryParseFailed { offset: usize, detail: String }` with Display `"E-QUERY-001: query parse error at offset {offset}: {detail}"` (error.rs:499–501). The taxonomy prose Message Format says "at position {pos}: {message}" (taxonomy:228) — minor prose drift, not blocking; the story's enrichment is additive and does not touch the Display. Noted in the pre-flight task so the implementer sources `near_text` from the chumsky error span, not from a `{pos}`/`{message}` shape that does not exist.
- **Edit:** APPLIED — pre-flight task annotated.
- **Source:** prism-core/src/error.rs:499–501; error-taxonomy.md:228.

---

## ARCHITECTURE FLAG (routed to specialist — NOT auto-edited into ACs)

**FLAG-001 — TableRegistry has no column-level schema; E-QUERY-038 `available_columns` source is ambiguous.**

- **Finding:** `TableRegistry` stores ONLY table names (`registered: HashSet<String>`) and a `sensor_by_table` reverse map (table_registry.rs:67–72). It exposes `is_registered`, `registered_tables`, `registered_sensor_ids`, `did_you_mean`, `did_you_mean_for_tables` — but **no column-listing method**. The story (and the BC-2.11.016 / taxonomy E-QUERY-038 text) say `available_columns` is "sourced ENTIRELY from TableRegistry" / "from operator TOML specs → TableRegistry." Neither is literally satisfiable today: columns live in `prism_spec_engine` `TableSpec.columns: Vec<ColumnSpec>` (field `ColumnSpec.name`), reachable at the gate via the `resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec>` parameter already threaded into `check_availability_gate` — NOT via `TableRegistry`.
- **Why it matters:** AC-001 and AC-002 both assert behavior over `available_columns` org-scoped per `(table, OrgId)`. The implementer needs a ratified data path: either (a) add a per-`(table, OrgId)` column API to `TableRegistry` (new struct field + populate from spec at register time), or (b) read columns from `resolved_spec_map` inside the E-QUERY-038 gate (the org-scope filter the gate already does for E-QUERY-037). Option (b) is the lower-blast-radius path and aligns with how E-QUERY-037's org-scoping already works, but it is a design decision that touches the gate signature and the injection-safety story (AC-002 / DI-008), so it is NOT a LOW-RISK auto-edit.
- **Routing:** architect (data path decision) → story-writer (AC/narrative wording reconciliation if the "ENTIRELY from TableRegistry" phrasing must change). The story narrative now carries a clearly-labeled CRITICAL ARCHITECTURE FLAG paragraph pointing here; ACs were left untouched.
- **Source:** prism-query/src/table_registry.rs:66–72, 207–397; prism-spec-engine column_mapping.rs:11,27,56 (TableSpec.columns / ColumnSpec.name); engine.rs:1129–1150 (resolved_spec_map threading).

---

## Spec Edits Applied (LOW-RISK only)

1. `risk_mitigations[0]` — net-new normalizer claim re-grounded (develop@f6739764, grep evidence, chumsky-no-reserializer); ast.rs:681/1099 citation retracted.
2. Previous Story Intelligence (S-3.13 block) — `Arc<dyn TableRegistry>` → concrete struct; `filter_to_org_visible()` → `_sensors/_tables`; gate name `check_availability_gate`; strsim 0.11.1 + signature citation; "line 84" de-pinned. Added the CRITICAL ARCHITECTURE FLAG paragraph (pointer to FLAG-001).
3. Previous Story Intelligence (ast.rs block) — retracted the false "lines 681/1099 raw display strings" text; replaced with verified zero-Display-impls guidance.
4. Library & Framework Requirements table — confirmed resolved versions (strsim 0.11.1, chumsky 0.12.0, datafusion 53.1.0, ariadne 0.4.1) with citations; chumsky row notes no built-in re-serializer.
5. Version-pinning note — rewritten with Cargo.lock-verified pins + research citation.
6. Pre-flight tasks — chumsky 0.12.0 parse-output note + E-QUERY-001 variant-name correction (`QueryParseFailed { offset, detail }`); Cargo.toml task de-pinned from "line 84."
7. Phase 5 + Token Budget + File Structure rows — ast.rs "display affordances at 681/1099" retracted everywhere.
8. Version bump 1.0 → 1.1 + changelog row.

No AC text, BC reference, scope, points, or Red Gate test name was changed.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | chumsky 0.12.0 AST-reserialization capability + error/span API; strsim 0.11.x levenshtein signature; version-publication confirmation (reasoning_effort=medium) |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 1 (resolve only) | resolved chumsky library ID (`/websites/rs_chumsky_chumsky`); query-docs not needed — Perplexity + Cargo.lock + in-repo usage already settled the API facts |
| Tavily * | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read (codebase, primary truth) | 7 | story spec; error-taxonomy.md (QUERY section); prism-core/error.rs; prism-query/Cargo.toml; engine.rs (gate); table_registry.rs; ast.rs (681/1099 + Display check) |
| Grep (codebase) | 6 | E-QUERY code inventory; TableRegistry API surface; ColumnType variants; ast.rs Display impls; error_mapping.rs arms; Cargo.lock version pins |
| Training data | 1 area | general Rust thiserror / serde `skip_serializing_if` idiom — flagged: low reliance; all version + API specifics verified against registry/Cargo.lock, not training data |

**Total MCP tool calls:** 2 (1 perplexity_research + 1 context7 resolve)
**Training data reliance:** low — every version number was read from Cargo.lock; every API/type/variant name was read from the live codebase; the one genuinely-external claim (chumsky has no re-serializer) was cross-confirmed by Perplexity (docs.rs/crates.io) AND an in-repo grep.
