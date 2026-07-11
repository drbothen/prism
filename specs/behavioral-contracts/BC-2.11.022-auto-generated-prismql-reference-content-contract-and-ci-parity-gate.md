---
document_type: behavioral-contract
level: L3
version: "1.2"
status: active
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: "2026-07-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-045-auto-generated-prismql-reference-resource-grammar-registry-parity-gate.md"
  - ".factory/specs/architecture/decisions/ADR-041-prismql-llm-auto-onboarding-4-layer-teaching-surface-for-automatic-agent-query-authoring.md"
input-hash: "TBD"
traces_to: ["CAP-034"]
extracted_from: null
---

# BC-2.11.022: Auto-Generated `prismql://reference` Content Contract and CI Parity Gate

## Description

The `prismql://reference` MCP resource is no longer served from a static `pql_reference.md` file. It is assembled at request time by `build_reference_content(infusion_registry: Option<&InfusionRegistry>) -> String` in `crates/prism-mcp/src/resources.rs`. Static sections (grammar, modes, BNF, operators, aggregates, temporal, virtual fields, error codes) are Rust `&'static str` constants validated by a CI round-trip gate. The enrichment section is assembled from the live `InfusionRegistry` at request time. A shared `&'static str` example array is the single source of truth: the doc and the test consume the same constant, making it impossible for a documented example to exist without being parse-tested.

## Preconditions

- The `read_resource` handler has been invoked with URI `prismql://reference`
- The MCP server has completed boot (boot step 9: WASM plugins loaded, `InfusionRegistry` populated)
- `InfusionRegistry` is passed as `Some(&registry)` at runtime (or `None` at test time / pre-boot-step-9)

## Postconditions

### Content requirements — every documented section MUST be accurate

| Section | Source of truth | Must include |
|---------|----------------|-------------|
| Mode overview (Filter / SQL / Pipe / SqlPipe) | ADR-043, ADR-046 D7 | Description of all four modes; clarify SQL→Pipe composition; note Filter = bare-predicate sugar |
| SQL mode BNF | `ast.rs` `SqlQuery` | SELECT, FROM, WHERE, GROUP BY, ORDER BY, LIMIT; no pipe-stage keywords |
| Pipe mode BNF + stage list | `pipe_parser.rs` doc-comment | `FROM <table> \| where \| sort \| head \| tail \| stats \| dedup \| fields \| enrich fn(col)` |
| SQL→Pipe composition | ADR-043 §Decision | `SELECT … FROM t \| <stage>` syntax; FORBID-BOTH dual-limit note |
| Operators table | `ast.rs` `Predicate` enum | `=`, `!=`, `>`, `>=`, `<`, `<=`, `IN`, `CONTAINS`/`ICONTAINS`, `=~`/`MATCHES`, `IN CIDR`, `HAS`, `MISSING`, `BETWEEN`, wildcard, `IS NULL`, `IS NOT NULL` |
| Aggregates / stats | `pipe_parser.rs` `stats_stage` | `count`, `sum`, `avg`, `min`, `max`, `percentile`, `distinct_count`; `stats <agg> [by <field>]` |
| Temporal grammar | ADR-044 | `NOW()`, `INTERVAL 'Nh'`, bare duration `24h`, `NOW() - INTERVAL 'Nh'`; note subtraction-only in v1 |
| Virtual fields + scope model | `ast.rs` `VirtualField` enum | `_client`, `_sensor`, `_source_table`, `_source_type`; scope-via-tool-param model (`_safety_flags` retired per BC-2.11.012 v1.7 — now parses as `Expr::Field` → E-QUERY-038) |
| Case sensitivity note | ADR-046 D5 | "All PrismQL keywords are case-insensitive. Convention: UPPER for SQL mode, lowercase for pipe stage names." |
| Column naming note | Design map §GRAMMAR-019 note | "Column names come verbatim from `prism_describe`; use the name as shown; do not construct dot-path names." |
| LIMIT / head / limit equivalence note | Design map §GRAMMAR-002 | "`head N == limit N` in pipe mode; `LIMIT N` is trailing clause in SQL mode; all are case-insensitive." |
| Error code quick-reference | error-taxonomy.md (E-QUERY-NNN subset) | E-QUERY-001 through E-QUERY-040 codes relevant to query authoring |
| Enrichment section | `InfusionRegistry` (live) | **Per-field UDF names** from `InfusionField.name` values (i.e., one entry per `[[infusion.fields]]` TOML declaration), NOT aggregated by `infusion_id`. Call signatures `enrich <name>(<col>)`. Example for a ThreatIntel infusion with fields `threat_score`, `threat_is_known_malicious`, `threat_sources`: the enrichment section lists all three as separate callable functions — NOT a single `threat_intel(col)` aggregate form. The `build_reference_content` implementation MUST iterate `InfusionRegistry.udf_descriptors()` and emit one entry per descriptor's `name` field (the per-field UDF name), NOT per `infusion_id`. Deduplication key is `descriptor.name`, not `descriptor.infusion_id`. Fallback placeholder shown when `InfusionRegistry` is `None`. |

### Structural requirements

- `build_reference_content(None)` must complete synchronously and return a valid string (no `unwrap` on registry)
- When `infusion_registry` is `None`, the enrichment section MUST include: "Call `list_infusions` to see available enrichment functions for your deployment." (placeholder)
- The assembled string MUST NOT include any examples from the old static `pql_reference.md` that do not parse under the current grammar
- Total assembled size: estimated 12–16KB; no upper bound enforced beyond the normal MCP response framing

### CI round-trip gate (ADR-045 D3 — the parity enforcer)

The CI gate has three complementary assertions driven by a **shared `&'static str` example array** (single source of truth for all reference examples):

1. **Positive round-trip gate:** For each positive example in the array, assert `PrismQlParser::parse(example)` returns `Ok(_)`. Any `Err` is a CI failure. This gate catches GRAMMAR-009 / GRAMMAR-011 class of doc-ahead-of-implementation drift.

2. **Negative gate (E-QUERY-040):** For each error example in the array labeled as producing E-QUERY-040 (dual-limit composed query), assert `PrismQlParser::parse_and_plan(example)` returns `Err(PrismError::RedundantRowLimit { .. })`. This keeps the pedagogical FORBID-BOTH error example honest.

3. **Registry-parity gate:** A test builds the enrichment section using a known test `InfusionRegistry` and asserts the rendered infusion names and call signatures exactly match the registry's enumerated capabilities (no additions, no omissions).

The shared example array MUST be a `const REFERENCE_EXAMPLES: &[(ExampleKind, &'static str, &'static str)]` in `crates/prism-mcp/src/resources.rs` where `ExampleKind` is an enum (`Positive`, `NegativeE040`, `NegativeOther`) and each tuple is `(kind, title, pql_snippet)` — the doc and the test both reference this const.

## Invariants

- `pql_reference.md` (the static file) is RETIRED — `include_str!("pql_reference.md")`, `PQL_REFERENCE_CONTENT`, and `render_pql_reference_resource` are removed from `crates/prism-mcp/src/resources/schema.rs`
- All static section content lives as Rust `&'static str` constants in `resources.rs` — reviewed in PRs as code, not as documentation
- The `InfusionRegistry` reference in `build_reference_content` comes from `Arc<ArcSwap<InfusionRegistry>>` loaded at the time of the `read_resource` call (reload-aware per ADR-042)
- No caching of the assembled reference string — assembled on each `read_resource` call (`prismql://reference` is a cold-path resource fetched once per session)
- The reference is accurate to the grammar at binary build time (static sections) and to the loaded plugins at call time (enrichment section)

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | `build_reference_content` cannot fail — it always returns a valid string | If `InfusionRegistry` is unavailable, the enrichment section shows the placeholder |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-022-001 | `read_resource prismql://reference` before WASM plugins loaded (boot step 8 not yet complete) | `build_reference_content(None)` is called; enrichment section shows placeholder; all other sections are accurate |
| EC-11-022-002 | `read_resource prismql://reference` after hot-reload adds a new infusion | Returns reference with updated enrichment section (reload-aware via `ArcSwap::load()`) |
| EC-11-022-003 | `read_resource prismql://reference` when no infusions are registered (empty registry) | Enrichment section shows "No enrichment infusions are currently registered. Call `list_infusions` to see available functions." |
| EC-11-022-006 | `InfusionRegistry` has infusion `threat_intel` (infusion_id) with three `[[infusion.fields]]` entries: `threat_score`, `threat_is_known_malicious`, `threat_sources`; AND infusion `nvd` (infusion_id) with three entries: `cvss_base_score`, `cvss_severity`, `cvss_vector` | Enrichment section lists SIX entries: `enrich threat_score(col)`, `enrich threat_is_known_malicious(col)`, `enrich threat_sources(col)`, `enrich cvss_base_score(col)`, `enrich cvss_severity(col)`, `enrich cvss_vector(col)`. It MUST NOT list `enrich threat_intel(col)` or `enrich nvd(col)`. The infusion_id (`threat_intel`, `nvd`) is NOT a callable UDF name and must not appear in the reference. (N1 / AUDIT-N1) |
| EC-11-022-004 | CI gate finds a positive example that fails to parse after grammar change | CI FAILS — the broken example must be fixed before merge |
| EC-11-022-005 | CI gate finds the E-QUERY-040 negative example now succeeds (FORBID-BOTH relaxed) | CI FAILS until the example is updated to reflect the new behavior |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Test | Expected Result | Category |
|------|----------------|----------|
| CI: positive gate — `SELECT * FROM crowdstrike_detections \| enrich threat_score(src_ip) \| limit 10` | `PrismQlParser::parse` returns `Ok(Ast::SqlPipe(_))` | CI gate |
| CI: positive gate — `FROM alerts \| where severity = 'HIGH' \| sort time DESC \| head 10` | `PrismQlParser::parse` returns `Ok(Ast::Pipe(_))` | CI gate |
| CI: positive gate — `SELECT * FROM t WHERE timestamp > NOW() - INTERVAL '24h'` | `PrismQlParser::parse` returns `Ok(Ast::Sql(_))` | CI gate |
| CI: negative gate — `SELECT * FROM t LIMIT 5 \| enrich fn(x) \| limit 3` | `plan` returns `Err(PrismError::RedundantRowLimit { sql_limit: 5, pipe_limit: 3 })` | CI gate |
| CI: registry-parity gate — test registry with `{threat_score, cvss_score}` infusions | Rendered enrichment section contains exactly `threat_score(col)` and `cvss_score(col)` signatures | CI gate |
| CI: per-field UDF name gate — test registry with infusion_id `threat_intel` having fields `{threat_score, threat_is_known_malicious}` | Rendered enrichment section contains `threat_score(col)` and `threat_is_known_malicious(col)`; does NOT contain `threat_intel(col)`. Deduplication key is `InfusionField.name`, not `InfusionField.infusion_id`. (N1 / EC-11-022-006) | CI gate |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input (applies to CI gate inputs) | fuzz |

## Related BCs

- **BC-2.11.020** (depends on — SQL→Pipe examples in reference): the reference includes SQL→Pipe composition examples validated by the CI gate
- **BC-2.11.021** (depends on — temporal grammar examples): the reference includes `NOW()` / `INTERVAL` examples validated by the CI gate
- **BC-2.11.023** (depends on — mode-bridge examples): the reference includes Filter/SQL/Pipe mode-bridge examples
- **BC-2.10.014** (supersedes partially): BC-2.10.014 contracts `prismql://reference` as a static resource; this BC supersedes the static-content aspect of that contract

## Architecture Anchors

- `crates/prism-mcp/src/resources.rs` — `build_reference_content` function + static section constants + example array
- `crates/prism-mcp/src/server.rs` — `read_resource` handler (passes live `InfusionRegistry`)
- `crates/prism-mcp/src/pql_reference.md` — RETIRED by this BC
- ADR-045: Auto-Generated `prismql://reference`
- ADR-041 §L3 — Full Grammar Reference Resource (amended by ADR-045)

## Story Anchor

TBD

## VP Anchors

VP-021 (fuzz gate applies to CI gate inputs)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| Capability Anchor Justification | CAP-034 ("MCP Server & Transport") per capabilities.md §CAP-034 — this BC governs an MCP resource (`prismql://reference`) registered and served by the MCP server layer. CAP-034 explicitly describes "MCP resources expose dynamic Prism state" and `prismql://reference` is the canonical PQL grammar reference resource, assembled and served by `PrismServer`. |
| L2 Invariants | DI-019 (query security limits apply to examples used in CI gate) |
| Priority | P0 |
| Closes findings | GRAMMAR-008 (reference has zero enrichment content), GRAMMAR-009 (reference BNF is wrong), GRAMMAR-017 (reference completeness gap); partially closes GRAMMAR-002, GRAMMAR-003 (virtual fields), GRAMMAR-007 (operators), GRAMMAR-018, GRAMMAR-019 (all via reference content requirements) |
| ADR traces | ADR-045 v1.1, ADR-041 v1.2 (amends §L3), ADR-042 (reload awareness) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / F-CSD-P23-001 POL-29 sweep | 2026-07-10 | product-owner | POL-29 exhaustive sweep: §Reference Content Requirements table row "Virtual fields + scope model" updated — `VirtualField` enum enumeration corrected from `_client, _sensor, _source_table, _safety_flags` to `_client, _sensor, _source_table, _source_type`. `_safety_flags` was retired as a virtual-field enum member per BC-2.11.012 v1.7 (F-CSD-P19-003); it now parses as `Expr::Field(fp)` → E-QUERY-038, meaning it is NOT in the `VirtualField` enum. `_source_type` is the correct 4th VirtualField member (sensor-table virtual field; S-2.08 AC-9/AC-10; S-3.02 delivery gap). |
| 1.1 | demo-fidelity-remediation-2026-06-26 | 2026-06-26 | product-owner | **N1 / AUDIT-N1 contract fix (S-DEMO-FIDELITY-REMEDIATION-001):** Enrichment section postcondition amended to explicitly specify that the reference must list per-`[[infusion.fields]]` UDF names (e.g., `threat_score`, `threat_is_known_malicious`, `threat_sources`, `cvss_base_score`, `cvss_severity`, `cvss_vector`) and MUST NOT list infusion_id aggregate names (e.g., `threat_intel`, `nvd`). The `build_reference_content` implementation MUST iterate `InfusionRegistry.udf_descriptors()` using `descriptor.name` (per-field UDF) as both the deduplication key and the emitted function name — not `descriptor.infusion_id`. Added EC-11-022-006 (6-UDF example with threat_intel/nvd infusion_ids) and a CI registry-parity gate test vector to prevent regression. The prismql://reference reference internally contradicting itself (listing `threat_intel(col)` in the "Available enrichment functions" section while using `threat_score(...)` in the example) is a code defect in `build_reference_content`, not a spec ambiguity. The spec required per-field UDF names (via the InfusionRegistry content requirement); this amendment makes that requirement machine-testable. |
| 1.0 | PR-203-post-merge-POL-14 | 2026-06-26 | state-manager | **POL-14 BC auto-promotion: draft → active.** Anchor story S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 squash-merged via PR #203 to develop@7e60df03 (2026-06-26; CI 43/43 green; 9-round PR-LEVEL 3-CLEAN(strict) cascade on frozen HEAD 356e0573). `status: draft → active`. No behavioral change; frontmatter status field only. |
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md + ADR-045 v1.1. Closes GRAMMAR-008/009/017 and partially closes GRAMMAR-002/003/007/018/019 via reference content requirements. CI gate mandates shared example array. |
