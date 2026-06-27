---
document_type: story
story_id: S-DEMO-FIDELITY-REMEDIATION-001
title: "Demo Fidelity Code Fixes — T13 Pre-Flight Audit Remediation (2026-06-26)"
wave: null
# Wave assignment: immediate — human directive "fix everything before T13". No wave scheduler needed;
# dispatch as soon as this story reaches status: ready after BC authorship is confirmed complete.
target_module: prism-mcp
# Primary crate is prism-mcp (resources.rs, prompts.rs, prism_describe.rs); prism-query and
# prism-core are secondary crates touched for: E-QUERY-039 net-new implementation (error.rs,
# engine.rs enrichment gate — AST visitor pass, no new InfusionRegistry API), and E-QUERY-037
# gate-ordering fix (table_registry.rs, engine.rs). NOTE: N1-B is NET-NEW (not a routing fix) —
# EnrichUdfNotFound variant + struct do not yet exist anywhere in the workspace (verified
# 2026-06-26 remove-uncertainty pass). E-QUERY-037 map_prism_error arm is CONFIRMED PRESENT
# (v1.2 C1 correction — NOT net-new as stated in v1.1).
subsystems: [SS-10, SS-11]
# Subsystem anchor justifications:
#   SS-10 (MCP Interface) owns the prism-mcp work:
#     - N1: build_reference_content in resources.rs (BC-2.11.022 v1.1) — per-field UDF names
#     - AUDIT-001: build_tables_for_client in prism_describe.rs (BC-2.10.012 v1.4) — sensor-prefixed name
#     - AUDIT-004: render_* functions in prompts.rs (BC-2.10.016 v1.2) — FROM-ready table names
#   SS-11 (Query Execution Engine) owns the prism-query + prism-core work:
#     - N1-B: E-QUERY-039 NET-NEW implementation: EnrichUdfNotFound variant+struct in prism-core/error.rs;
#             plan-time enrichment gate in prism-query/engine.rs (AST visitor, pipe EnrichStage +
#             SQL ScalarFunc::Unknown paths); map_prism_error -32602 net-new arm in error_mapping.rs.
#             NOTE: map_prism_error arm for E-QUERY-037 (TableNotAvailable) is CONFIRMED PRESENT —
#             only the E-QUERY-039 (EnrichUdfNotFound) arm is net-new. BC-2.11.019 v1.3 draft→active
#             at merge (POL-14).
#     - N2: E-QUERY-037 gate-ordering fix located in table_registry.rs (check_availability_gate /
#           is_registered) + engine.rs — NOT materialization.rs only (verified 2026-06-26).
#   Both subsystems are touched; SS-10 is primary (larger scope); SS-11 is co-owner.
priority: P0
# P0: ALL findings targeted by this story are DEMO-BLOCKING under the human directive
# "fix everything before T13" (2026-06-26). The T13 recording cannot proceed with incorrect
# enrichment function names in the reference, silent empty results for dot-syntax, incorrect
# describe names, or prompt bodies that embed invalid FROM queries.
depends_on:
  - S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
  # S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (merged PR #203 develop@7e60df03) must be on develop
  # as a base. This story builds on the MCP server refactor (build_reference_content, prompts.rs
  # prompt body structure, TableRegistry plan-time gates) introduced by PR #203.
  # Dependency anchor: build-order requirement — the code surfaces (resources.rs build_reference_content,
  # materialization.rs E-QUERY-037 gate, prompts.rs render_* functions) introduced by PR #203
  # are the exact functions this story modifies.
blocks: []
estimated_days: 2
# Estimate: 5 targeted code fixes, each well-scoped with a BC and known root cause.
# N1 (resources.rs dedup key change): 0.5d
# N1-B (E-QUERY-039 gate investigation + fix): 0.5d
# N2 (E-QUERY-037 plan-time dot-notation gate ordering): 0.5d
# AUDIT-001 (build_tables_for_client sensor-prefix): 0.25d
# AUDIT-004 (prompts.rs FROM-ready names): 0.25d
points: 10
# Points breakdown (revised v1.2 — C1/I1/I2/S1 corrections; total unchanged from v1.1):
#   BC-2.11.022 v1.1 — N1: fix dedup key in build_reference_content: 2 pts
#   BC-2.11.019 v1.3 — N1-B: NET-NEW E-QUERY-039 implementation:
#     create EnrichUdfNotFound variant + EnrichUdfNotFoundDetails #[non_exhaustive] struct
#     in prism-core/error.rs; plan-time enrichment gate in prism-query/engine.rs (AST visitor,
#     pipe PipeStage::Enrich + SQL ScalarFunc::Unknown paths; derive UDF names from udf_descriptors());
#     map_prism_error -32602 arm (E-QUERY-039 only — E-QUERY-037 arm confirmed-present):
#     4 pts (net-new > 2 pts originally)
#   BC-2.11.001 v1.15 — N2: fix gate ordering across table_registry.rs + engine.rs
#     (NOT materialization.rs only — gate is in check_availability_gate/is_registered): 2 pts
#   BC-2.10.012 v1.4 — AUDIT-001: fix build_tables_for_client emit format: 1 pt
#   BC-2.10.016 v1.2 — AUDIT-004: fix render_* prompt FROM-ready table names: 1 pt
#   Total: 10 pts (N1-B is full net-new implementation, not a routing investigation)
level: "L4"
status: draft
# BC status: 4 active (BC-2.11.001 v1.15, BC-2.11.022 v1.1, BC-2.10.016 v1.2, BC-2.10.012 v1.4)
# + BC-2.11.019 v1.3 draft→active at merge per POL-14. Canonical versions are authoritative
# in the body BC table (§Behavioral Contracts); this comment is a status note only.
# Per Spec-First Gate S-7.01 this story is valid for dispatch as behavioral_contracts is non-empty.
version: "1.9"
updated: "2026-06-27"
producer: story-writer
timestamp: "2026-06-26T00:00:00Z"
input-hash: "TBD"
inputs:
  - ".factory/research/demo-pre-flight-audit-2026-06-26.md"
  - ".factory/research/demo-finding-remediation-plan-2026-06-26.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.022-auto-generated-prismql-reference-content-contract-and-ci-parity-gate.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.019-e-query-039-enrich-udf-not-found.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.016-mcp-prompts-fast-return-guarantee-no-hang.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md"
cycle: "v1.0.0-greenfield"
epic_id: "E-5"
# Epic E-5 (MCP Interface / Query Engine). Remediation story targeting T13 capstone demo fidelity.
phase: 2
acceptance_criteria_count: 10
# 10 ACs: 5 code-fix ACs (one per finding), 2 regression/workspace ACs, 1 demo-evidence AC,
# 1 non-exhaustive compliance AC, 1 SAP-1 structural-event-catalog AC.
# NOTE v1.2: AC-N1B corrected (C1/I1/I2/S1); AC-N2 E-QUERY-037 arm confirmed-present; AC-REG-1 unchanged.
red_gate_tests: 9
# 9 Red Gate tests (v1.2 — count unchanged from v1.1; test assertions corrected per S1):
#   AC-N1: test_bc_2_11_022_n1_per_field_udf_names
#   AC-N1B: test_bc_2_11_019_n1b_infusion_id_as_udf_name (Err(EnrichUdfNotFound(_)) assertion)
#           test_bc_2_11_019_n1b_mcp_maps_to_32602 (map_prism_error -32602 arm)
#   AC-N2: test_bc_2_11_001_n2_dot_notation_from_target_e_query_037 (pipe + SQL + filter regression)
#   AC-AUDIT-001: test_bc_2_10_012_audit_001_sensor_prefixed_table_names
#   AC-AUDIT-004: test_bc_2_10_016_audit_004_no_dot_notation_in_prompts
#   AC-REG-1: just check (workspace gate, not a unit test; verified by exit code 0)
#             scripts/check-non-exhaustive.sh EXPECTED=88 (compile-fail gate via shell script, not a named Rust test)
#   AC-REG-2: test_bc_2_11_022_ci_3tier_gate (existing, updated for per-field UDF parity)
tdd_mode: strict
behavioral_contracts:
  [BC-2.11.001, BC-2.11.022, BC-2.11.019, BC-2.10.016, BC-2.10.012]
# BC array propagation (bc_array_changes_propagate_to_body_and_acs):
# BC-2.11.001 — query MCP tool (cited in AC-N2: dot-notation EC-11-067 plan-time gate)
# BC-2.11.022 — prismql://reference content contract (cited in AC-N1: per-field UDF dedup)
# BC-2.11.019 — E-QUERY-039 enrich-UDF-not-found gate (cited in AC-N1B)
# BC-2.10.016 — MCP prompts fast-return + FROM-ready names (cited in AC-AUDIT-004)
# BC-2.10.012 — prism_describe schema discovery tool (cited in AC-AUDIT-001)
# All 5 BCs cited in at least one AC body trace.
verification_properties: [VP-021]
# VP-021 (PrismQL parser never panics on arbitrary input — fuzz) applies to changes in
# materialization.rs E-QUERY-037 gate ordering (N2) and any new plan-time checks.
assumption_validations: []
risk_mitigations: []
crates_touched:
  - prism-core
  # error.rs: CREATE PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>) variant
  # and EnrichUdfNotFoundDetails { infusion: String, available_infusions: Vec<String>,
  # did_you_mean: Option<String> } struct. Both MUST carry #[non_exhaustive].
  # This is a net-new public type — ci.yml EXPECTED increments 87→88; CLAUDE.md sentence
  # + attribution list updated in-scope.
  - prism-mcp
  # resources.rs: build_reference_content function — change deduplication key from
  # descriptor.infusion_id to descriptor.name; update REFERENCE_EXAMPLES enrichment section.
  # tools/prism_describe.rs: build_tables_for_client function — change name field emit from
  # table.table_name.clone() to format!("{sensor_id}_{}", table.table_name).
  # prompts.rs: render_triage_alerts, render_client_overview, render_cross_client_status,
  # render_investigate_host functions — replace all dot-notation FROM references (FROM
  # crowdstrike.alerts, FROM claroty.alerts, etc.) with sensor-prefixed underscore-qualified
  # names (FROM crowdstrike_detections, FROM armis_devices, etc.).
  # error_mapping.rs: map_prism_error function — ADD explicit -32602 INVALID_PARAMS arm for
  # PrismError::EnrichUdfNotFound (E-QUERY-039). This is the ONLY net-new arm.
  # PrismError::TableNotAvailable (E-QUERY-037) arm is CONFIRMED PRESENT at line ~166 with
  # doc block citing "S-3.13 AC-2; BC-2.11.001; error-taxonomy.md E-QUERY-037" — NO CHANGE NEEDED.
  # The E-QUERY-039 arm MUST NOT fall through to -32000.
  - prism-query
  # table_registry.rs: check_availability_gate and/or is_registered functions — add or
  # correct the plan-time TableRegistry::is_registered check so dot-notation FROM targets
  # (e.g., cyberint.alerts) return PrismError::TableNotAvailable (E-QUERY-037) with
  # did_you_mean BEFORE sensor_id_from_table_name dot-extraction routes to the fan-out.
  # Verify whether the current E-QUERY-036 vs E-QUERY-037 path distinction is correct:
  # UnknownSourceTable (E-QUERY-036) is for materialization.rs resolve_source_refs failures;
  # TableNotAvailable (E-QUERY-037) is the availability/registration gate path.
  # engine.rs: locate where table availability gate is surfaced to the caller and confirm
  # it is the correct insertion point for the is_registered pre-check.
  # NOTE: materialization.rs resolve_source_refs currently returns E-QUERY-036 via
  # UnknownSourceTable; do NOT move or remove that path — the N2 fix adds the E-QUERY-037
  # gate in the table_registry / engine layer BEFORE resolve_source_refs is reached.
  # Also add plan-time enrichment gate for EnrichStage.infusion (pipe path) and
  # ScalarFunc::Unknown (SQL path) that returns PrismError::EnrichUdfNotFound (E-QUERY-039).
---

# S-DEMO-FIDELITY-REMEDIATION-001: Demo Fidelity Code Fixes

## Narrative

As a Prism developer preparing the T13 capstone demo recording, I want five targeted
code defects identified in the 2026-06-26 pre-flight re-audit fixed, so that the
`prismql://reference` resource lists the correct callable enrichment UDF names, calling
an unregistered enrichment name returns a self-correcting E-QUERY-039 error (not an
opaque internal error), `FROM cyberint.alerts` returns a pedagogical E-QUERY-037 with
`did_you_mean` (not a silent empty result), `prism_describe` table `name` fields are
FROM-ready sensor-prefixed names, and all MCP prompt bodies embed valid FROM-ready
sensor-prefixed table names that execute without E-QUERY-037.

## Behavioral Contracts

| BC ID | Version | Title |
|-------|---------|-------|
| BC-2.11.001 | v1.15 | `query` MCP Tool Accepts Scoping + PrismQL Query String |
| BC-2.11.022 | v1.1 | Auto-Generated `prismql://reference` Content Contract and CI Parity Gate |
| BC-2.11.019 | v1.3 | E-QUERY-039 Enrich-UDF-Not-Found Plan-Time Gate |
| BC-2.10.016 | v1.2 | MCP Prompts Fast-Return Guarantee — No Indefinite Hang |
| BC-2.10.012 | v1.4 | `prism_describe` Schema Discovery Tool (L2) |

---

## Acceptance Criteria

> NOTE: ACs are ordered by implementation dependency. AC-N2 (prism-query gate ordering)
> should be implemented before AC-AUDIT-004 (prompts), since AUDIT-004 fixes depend on
> knowing the correct FROM-ready table names — which are the same sensor-prefixed names
> that AUDIT-001 and the current TableRegistry already use. Each AC maps to one finding
> from the 2026-06-26 pre-flight audit and one BC postcondition.

---

### Area A — MCP Reference: Correct Enrichment Function Names (N1)

**AC-N1** (traces to BC-2.11.022 v1.1 postcondition — enrichment section per-field UDF names,
EC-11-022-006): `build_reference_content` in `crates/prism-mcp/src/resources.rs` iterates
`InfusionRegistry.udf_descriptors()` and deduplicates by `descriptor.name` (the per-field UDF
name), NOT by `descriptor.infusion_id`. For a live `InfusionRegistry` loaded from
`threatintel.infusion.toml` (infusion_id `threat_intel`, fields `threat_score`,
`threat_is_known_malicious`, `threat_sources`) and `nvd.infusion.toml` (infusion_id `nvd`,
fields `cvss_base_score`, `cvss_severity`, `cvss_vector`), the assembled reference enrichment
section MUST list exactly **six** callable entries: `enrich threat_score(col)`, `enrich
threat_is_known_malicious(col)`, `enrich threat_sources(col)`, `enrich cvss_base_score(col)`,
`enrich cvss_severity(col)`, `enrich cvss_vector(col)`. The strings `threat_intel` and `nvd`
(the infusion_ids, which are NOT callable UDF names) MUST NOT appear in the enrichment section.

**Red Gate test:** `test_bc_2_11_022_n1_per_field_udf_names` — build an `InfusionRegistry`
test fixture with `threat_intel` infusion (fields: `threat_score`, `threat_is_known_malicious`,
`threat_sources`) and `nvd` infusion (fields: `cvss_base_score`, `cvss_severity`,
`cvss_vector`); call `build_reference_content(Some(&registry))`; assert the enrichment section
contains all six per-field names; assert it does NOT contain `threat_intel(` or `nvd(` as
callable fn forms (the N1 regression guard).

---

### Area B — E-QUERY-039: Implement Net-New Enrichment UDF Not Found Gate (N1-B)

> **SCOPE NOTE (v1.2):** N1-B is NET-NEW implementation, NOT a routing investigation.
> `PrismError::EnrichUdfNotFound` and `EnrichUdfNotFoundDetails` do NOT exist anywhere in the
> workspace (zero matches as of 2026-06-26). E-QUERY-039 appears only as a doc table row in
> resources.rs. PR #203 did NOT implement this variant. This AC creates the variant, struct,
> gate, and MCP mapping from scratch per BC-2.11.019 v1.3. BC-2.11.019 promotes draft→active
> at merge (POL-14).
>
> **NO NEW PUBLIC API on `InfusionRegistry`** (I1 correction v1.2): Do NOT add a `udf_names()`
> method to `InfusionRegistry`. The public API already provides `udf_descriptors()` — derive
> the UDF name set from it inline: `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect::<Vec<_>>()`.
> Use this expression everywhere `available_infusions` is populated and everywhere the
> strsim candidate set is built. This keeps the `InfusionRegistry` public API surface minimal
> and the new-#[non_exhaustive]-type count at exactly ONE (EnrichUdfNotFoundDetails), so
> ci.yml EXPECTED increments 87→88 (not 87→89).
>
> **E-QUERY-037 arm in `map_prism_error` is CONFIRMED PRESENT** (C1 correction v1.2): The
> `PrismError::TableNotAvailable(..)` arm already exists in `error_mapping.rs` (line ~166,
> doc block: "Reference: S-3.13 AC-2; BC-2.11.001; error-taxonomy.md E-QUERY-037"). No change
> needed to that arm. ONLY the `EnrichUdfNotFound` arm (E-QUERY-039) is net-new.

**AC-N1B** (traces to BC-2.11.019 v1.3 postconditions — EnrichUdfNotFound variant shape,
gate firing condition for pipe-mode `EnrichStage.infusion` NOT in
`InfusionRegistry.udf_to_infusion`, and SQL-mode `ScalarFunc::Unknown` gate, and MCP -32602
mapping):

> **Gate-ordering note (BC-2.11.019 v1.3):** E-QUERY-039 fires LAST in the plan-time gate
> sequence. The full ordered sequence is: E-QUERY-001 (parse error) → E-QUERY-037 (table
> availability, `check_availability_gate`) → E-QUERY-038 (column gate) → E-QUERY-039 (enrichment
> UDF not found, this gate). A query with both a dot-notation FROM target AND an invalid
> enrichment name returns E-QUERY-037, NOT E-QUERY-039 — the table gate fires first.
>
> **WHERE-clause note (BC-2.11.019 v1.3 §Precondition 1(b)):** SQL-mode enrichment-validation
> gates `ScalarFunc::Unknown(name)` in SELECT PROJECTION expressions — this is the reachable,
> real-query path. The WHERE-predicate scan via `collect_unknown_scalar_from_predicate` is
> DEFENSIVE / forward-compatible coverage: it honors BC-2.11.019 §Precondition 1(b)'s
> AST-contract ("a WHERE clause containing FuncCall::Scalar{...} must be gated at plan time"),
> but a real SQL query `WHERE udf(col) = v` is currently an **E-QUERY-001 parse error** —
> `build_predicate_parser` (the WHERE grammar, `comparison` atom) parses
> `field_path → compare_op → literal` only; there is no scalar-funcall atom, so the parser
> hits `(` where it expects a compare op and never produces a `ScalarFunc::Unknown` node from
> WHERE text. `ScalarFunc::Unknown` is produced ONLY by the SQL expression parser used for
> SELECT projections (`build_sql_expr_parser`). The WHERE scan is exercised by unit tests via
> programmatic AST construction (`engine::enrich_gate_where_clause_unit_tests`), not reachable
> from real parsed query text today. The pipe `enrich`-keyword form used in a WHERE position
> (e.g., `| WHERE enrich threat_score(col) > 0`) is also an E-QUERY-001 parse error (pipe
> filter grammar has no fn-call atom). Both projection (reachable) and WHERE (defensive) scans
> feed the same validation loop and the same `EnrichUdfNotFound` error type.

**Step 1 — Create the error type** (in `crates/prism-core/src/error.rs`):
- Add variant `EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` to `PrismError`.
- Add `#[non_exhaustive]` struct `EnrichUdfNotFoundDetails { pub infusion: String, pub available_infusions: Vec<String>, pub did_you_mean: Option<String> }`.
  - `available_infusions` is `Vec<String>` (canonical type per BC-2.11.019 v1.3; PO-ratified).
- Both type and variant MUST carry `#[non_exhaustive]`. Increment `ci.yml EXPECTED` 87→88. Update `CLAUDE.md` non-exhaustive sentence + attribution list in the same atomic commit.

**Step 2 — Add plan-time enrichment gate** (in `crates/prism-query/src/engine.rs`) (I2 anchor v1.3):
Add a new plan-time enrichment-validation pass in `crates/prism-query/src/engine.rs`, invoked
BEFORE `check_availability_gate`/fan-out. This pass uses the AST `visit::Visitor` to collect
enrichment function names from BOTH query paths and validates each against the registered
UDF name set (derived from `registry.udf_descriptors()`):
- **Pipe path** — visitor arm collects `EnrichStage.infusion` values from `PipeStage::Enrich` nodes.
- **SQL path** — visitor arm collects `ScalarFunc::Unknown(name)` values from SELECT projection expressions (reachable from real queries via `build_sql_expr_parser`) AND from WHERE clause predicates via `collect_unknown_scalar_from_predicate` (DEFENSIVE / forward-compat coverage per BC-2.11.019 v1.3 §Precondition 1(b) AST-contract; see WHERE-clause note above — a real `WHERE udf(col) = v` is an E-QUERY-001 parse error today; the WHERE scan is exercised by programmatic AST unit tests, not real parsed query text).

Both collection paths are DISTINCT visitor arms but feed the same validation loop and the same
`EnrichUdfNotFound` error type. For each collected name: if `name` is NOT a key in
`InfusionRegistry.udf_to_infusion`, return at plan time:
```rust
Err(PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails {
    infusion: name.to_owned(),
    available_infusions: registry.udf_descriptors().iter().map(|d| d.name.clone()).collect(),
    did_you_mean: strsim_closest(&name, &udf_names_vec),
})))
```
Gate MUST fire BEFORE any fan-out or sensor I/O. No new public methods on `InfusionRegistry`.
Gate ordering: this enrichment-validation pass runs AFTER the table availability gate
(`check_availability_gate` / E-QUERY-037) so that table-availability errors are reported first.

**Step 3 — Add MCP mapping** (in `crates/prism-mcp/src/error_mapping.rs`):
- Add an explicit arm for `PrismError::EnrichUdfNotFound(d)` in `map_prism_error` that returns
  `(codes::INVALID_PARAMS, ...)` with the canonical Display message format (BC-2.11.019 v1.3):
  ```
  E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}
  ```
  Where `{available_infusions}` is the comma-joined `Vec<String>` wrapped in brackets (e.g.,
  `[threat_score, threat_is_known_malicious, threat_sources]`), and `{did_you_mean}` is
  ` Did you mean: '{x}'?` when `did_you_mean` is `Some(x)`, or omitted (empty string) when `None`.
  Full example (no suggestion): `E-QUERY-039: enrichment infusion 'threat_intel' is not registered; available: [threat_score, threat_is_known_malicious, threat_sources, cvss_base_score, cvss_severity, cvss_vector]`
  Full example (with suggestion): `E-QUERY-039: enrichment infusion 'threat_scor' is not registered; available: [threat_score, ...] Did you mean: 'threat_score'?`
- This arm MUST NOT fall through to the `-32000` catch-all.
- The `PrismError::TableNotAvailable(..)` arm (E-QUERY-037) is CONFIRMED PRESENT — do NOT modify or duplicate it.

**Observable behavior**: A pipe-mode query `FROM cyberint_alerts | enrich threat_intel(iocs_value)` where `threat_intel` is an infusion_id (not a per-field UDF name) and therefore NOT a key in `InfusionRegistry.udf_to_infusion`, returns `PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` at plan time, surfaced as MCP `-32602 INVALID_PARAMS` with `code: "E-QUERY-039"`. It MUST NOT return `E-INT-001` "Internal error; see audit log". The `available_infusions: Vec<String>` field MUST list the registered per-field UDF names (e.g., `threat_score`, `threat_is_known_malicious`, `threat_sources`, ...). A `did_you_mean` suggestion is present IF any registered UDF name is within Levenshtein distance 3 of the queried name; `None` is a valid outcome when no registered name is within distance 3 (e.g., `"threat_intel"` vs per-field names like `"threat_score"` may exceed distance 3). The same gate applies to a SQL-mode `ScalarFunc::Unknown("nvd")` in a SELECT projection: it returns E-QUERY-039, NOT E-INT-001.

**Red Gate tests:**

`test_bc_2_11_019_n1b_infusion_id_as_udf_name` — execute a plan-time validation with query
`FROM cyberint_alerts | enrich threat_intel(iocs_value)` where `threat_intel` is NOT registered
as a UDF name in `InfusionRegistry` (only per-field names are registered); assert the result
is `Err(PrismError::EnrichUdfNotFound(_))` with `infusion: "threat_intel"` and
`available_infusions` non-empty (listing the registered per-field UDF names); assert the
result is NOT `E-INT-001` (negative control). Do NOT assert `did_you_mean.is_some()` — assert
on `available_infusions` (always populated) and the error variant/code only, since registered
per-field UDF names are likely > Levenshtein-3 from `"threat_intel"` making `did_you_mean: None`
a valid outcome (S1 relaxation v1.2). Also assert SQL-mode `ScalarFunc::Unknown("nvd")` returns
`Err(PrismError::EnrichUdfNotFound(_))`.

`test_bc_2_11_019_n1b_mcp_maps_to_32602` — call `map_prism_error(PrismError::EnrichUdfNotFound(...))`;
assert the returned MCP error code is `-32602` (INVALID_PARAMS); assert it is NOT `-32000`
(the generic catch-all). This test lives in `crates/prism-mcp/src/error_mapping.rs`
`#[cfg(test)]` module.

---

### Area C — E-QUERY-037: Dot-Notation FROM Target Intercepted at Plan Time (N2)

> **SCOPE NOTE (v1.1, confirmed in v1.2):** N2's gate-ordering fix is located in `crates/prism-query/src/table_registry.rs`
> (`check_availability_gate` / `is_registered`) and `crates/prism-query/src/engine.rs` — NOT
> in `materialization.rs` only. The `TableNotAvailable` variant (E-QUERY-037) already exists
> and is constructed in table_registry.rs. In `materialization.rs`, `resolve_source_refs` calls
> `sensor_id_from_table_name` FIRST and returns `UnknownSourceTable` (E-QUERY-036) on failure
> — a DIFFERENT code path. The N2 fix adds/corrects the `TableRegistry::is_registered` pre-check
> in the table_registry/engine layer so dot-notation strings are caught BEFORE reaching the
> fan-out or resolve_source_refs. The current failure mode is silent E-SENSOR-030 (dot-notation
> string routes to sensor fan-out) rather than the claimed "E-QUERY-036 in materialization.rs."
> Implementer MUST trace the actual call chain via table_registry.rs → engine.rs before writing
> the fix. Do NOT conflate E-QUERY-036 (UnknownSourceTable, materialization.rs) with E-QUERY-037
> (TableNotAvailable, table_registry.rs/engine.rs).

**AC-N2** (traces to BC-2.11.001 v1.15 postcondition — table availability plan-time check,
EC-11-067: dot-notation in FROM target position): A query `FROM cyberint.alerts` (pipe mode)
or `SELECT * FROM crowdstrike.detections` (SQL mode) where `cyberint.alerts` / `crowdstrike.detections`
is NOT a key in `TableRegistry` (only underscore-qualified names like `cyberint_alerts`,
`crowdstrike_detections` are registered), returns `PrismError::TableNotAvailable` (`E-QUERY-037`)
at plan time with `table: "cyberint.alerts"`, `sensor: "cyberint"`, `did_you_mean: "cyberint_alerts"`
(or `"crowdstrike_detections"` respectively). The `TableRegistry::is_registered` check in
`check_availability_gate` (table_registry.rs) MUST run BEFORE `sensor_id_from_table_name`
dot-notation extraction in the fan-out path — the dot-notation string MUST NOT silently route
to the sensor adapter fan-out (which produces E-SENSOR-030 partial failures). The result is
`isError: true`, NOT `isError: false, returned: 0` with `sensor_errors: ["E-SENSOR-030"]`.
The fix MUST NOT regress BC-2.11.023 / ADR-046 filter-mode dot-notation (source-qualified
filter refs like `crowdstrike_detections | severity='HIGH'` continue to work — filter mode
uses `<table_name> | <predicate>` syntax, NOT `<sensor>.<table>` dot-syntax as a FROM target).
The `map_prism_error` arm for `PrismError::TableNotAvailable` (E-QUERY-037) is CONFIRMED
PRESENT in `error_mapping.rs` (doc block: "Reference: S-3.13 AC-2; BC-2.11.001") — no
change needed to that arm. The implementer should verify it returns `-32602` as expected
(it does) but MUST NOT modify or duplicate it.

**Red Gate test:** `test_bc_2_11_001_n2_dot_notation_from_target_e_query_037` — execute plan
validation on `FROM cyberint.alerts` with `cyberint_alerts` registered in `TableRegistry`
(via check_availability_gate or equivalent table_registry entry point); assert
`Err(PrismError::TableNotAvailable(_))` with `table: "cyberint.alerts"` and
`did_you_mean: "cyberint_alerts"`; assert `isError: true` in the MCP response (not partial
success). Also assert filter-mode `crowdstrike_detections | severity='HIGH'` continues to
parse as `Ast::Filter` and passes TableRegistry validation (regression guard for BC-2.11.023).
Also assert SQL-mode `SELECT * FROM crowdstrike.detections` returns `TableNotAvailable`
(EC-11-067 covers all modes).

---

### Area D — prism_describe: FROM-Ready Sensor-Prefixed Table Names (AUDIT-001)

**AC-AUDIT-001** (traces to BC-2.10.012 v1.4 postcondition — `name` postcondition fully-qualified
FROM-ready token, closes AUDIT-001 + AUDIT-008): `build_tables_for_client` in
`crates/prism-mcp/src/tools/prism_describe.rs` emits `name: format!("{sensor_id}_{}", table.table_name)`
for each table entry, NOT `name: table.table_name.clone()`. For org-c with 4 sensors
(crowdstrike, cyberint, claroty, armis), `prism_describe(org-c)` returns table entries with
distinct, fully-qualified `name` values: `crowdstrike_detections`, `cyberint_alerts`,
`claroty_devices`, `claroty_audit_logs`, `armis_devices`, etc. No two `name` entries are
identical (the disambiguation guarantee). The `example_query` field uses the same
sensor-prefixed name (e.g., `"SELECT COUNT(*) FROM cyberint_alerts WHERE timestamp > NOW() -
INTERVAL '1h'"`). The `pql_hints` array, when non-empty, contains a generic usage hint
(`"Use 'SELECT * FROM <table> LIMIT 25' to query any of the N table(s) above."`) with a
`<table>` placeholder — it does NOT embed table names. The disambiguation guarantee is
entirely in `TableDescriptor.name` and `example_query`; `pql_hints` plays no role in it.

**Red Gate test:** `test_bc_2_10_012_audit_001_sensor_prefixed_table_names` — construct a
`build_tables_for_client` call with a 3-sensor client (cyberint/alerts, claroty/devices,
armis/devices); assert NO two returned `name` fields are identical; assert each `name` equals
`format!("{sensor_id}_{table_name}")`; assert each `example_query` references the same
fully-qualified name (no bare `FROM alerts`).

---

### Area E — MCP Prompts: FROM-Ready Table Names in All Prompt Bodies (AUDIT-004)

**AC-AUDIT-004** (traces to BC-2.10.016 v1.2 postcondition — FROM-ready table names in prompt
bodies, EC-10-016-005 / EC-10-016-006): All five `render_*` functions in
`crates/prism-mcp/src/prompts.rs` (`render_triage_alerts`, `render_client_overview`,
`render_cross_client_status`, `render_investigate_host`, `render_query_tutorial`) MUST NOT
emit dot-notation table references (`FROM crowdstrike.alerts`, `FROM claroty.alerts`,
`FROM armis.devices`, etc.) in any embedded PrismQL query in their message body. Every FROM
clause in a rendered prompt MUST use sensor-prefixed underscore-qualified names that resolve
without error: `crowdstrike_detections`, `armis_devices`, `claroty_devices`,
`claroty_audit_logs`, `cyberint_alerts`. A regex scan `FROM\s+\w+\.\w+` across all five
rendered prompt bodies MUST return zero matches (no dot-notation in FROM target position).

**Red Gate test:** `test_bc_2_10_016_audit_004_no_dot_notation_in_prompts` — call each of the
five `render_*` functions with representative arguments (`client_id: "org-c"` or equivalent);
collect the full rendered message text from all five; assert the regex `FROM\s+\w+\.\w+` has
zero matches across the combined output; assert each rendered body contains at least one
valid FROM reference (e.g., `FROM crowdstrike_detections`) — not merely that dot-notation
is absent, but that correct sensor-prefixed names are present (positive guard).

---

### Regression and Workspace Gate

**AC-REG-1** (traces to BC-2.11.001 v1.15 invariant — DI-019 and DI-008, and
`#[non_exhaustive]` discipline in CLAUDE.md §Conventions): Full workspace `just check` exits
0 after all five code fixes. No existing tests regress.

**REQUIRED (not optional):** The N1-B net-new work introduces `EnrichUdfNotFoundDetails` as
a new `#[non_exhaustive]` public struct in `prism-core/src/error.rs`. This MUST be reflected
in ALL of:
1. `EnrichUdfNotFoundDetails` carries `#[non_exhaustive]` attribute (required by CLAUDE.md discipline).
2. `ci.yml EXPECTED` is incremented from `87` to `88` (the compile-fail gate count).
3. The CLAUDE.md `#[non_exhaustive]` sentence (currently "87 types currently enforced") is
   updated to `88` with `EnrichUdfNotFoundDetails` added to the attribution parenthetical.
4. The perimeter/non-exhaustive compile-fail gate (`tests/external/non-exhaustive-violation/`)
   continues to pass with `EXPECTED=88`.

All four changes MUST land in the same atomic commit as the `EnrichUdfNotFoundDetails` struct
definition. A story is NOT DONE if `ci.yml EXPECTED` still reads `87` after this story merges.

**Red Gate verification:** `just check` exit code 0 (workspace gate). Additionally, the
implementer MUST verify `grep 'EXPECTED=' ci.yml` shows `88` before declaring done.

**AC-REG-2** (traces to BC-2.11.022 v1.1 invariant — CI 3-tier gate): The existing
`REFERENCE_EXAMPLES` CI round-trip gate tests continue to pass: (1) positive examples parse
as `Ok(_)`, (2) E-QUERY-040 negative examples return `Err(PrismError::RedundantRowLimit)`,
(3) registry-parity gate passes with the corrected per-field UDF deduplication. The AC-N1
fix specifically updates the registry-parity assertion to verify per-field names (not
infusion_id aggregate names). No gate test previously passing may regress.

**Red Gate test:** `test_bc_2_11_022_ci_3tier_gate` (existing) — this test MUST continue to
pass and the implementer MUST update its registry-parity sub-assertion to verify per-field
UDF name emission (NOT infusion_id emission) as the N1 regression guard.

---

### Demo Evidence and SAP Compliance

**AC-DEMO-001**: After all five code fixes land on the feature branch, a demo-recorder run
captures evidence for each finding:
- Evidence-N1: `prismql://reference` enrichment section listing six per-field UDF names (`threat_score`, etc.),
  NOT `threat_intel` / `nvd`.
- Evidence-N1B: calling `FROM cyberint_alerts | enrich threat_intel(iocs_value)` returns E-QUERY-039
  with `available_infusions` listing registered per-field names.
- Evidence-N2: `FROM cyberint.alerts` returns E-QUERY-037 with `did_you_mean: "cyberint_alerts"`
  (not a silent 0-row result).
- Evidence-AUDIT-001: `prism_describe(org-c)` returns table names `cyberint_alerts`,
  `claroty_devices`, `armis_devices`, etc. (no bare `alerts` / `devices` collisions).
- Evidence-AUDIT-004: `triage_alerts` prompt body contains `FROM crowdstrike_detections`
  (no `FROM crowdstrike.alerts`).

**AC-SAP-1** (traces to SAP-1 / BC-2.16.002 structured event catalog discipline): If any new
`event_type =` tracing emission is added to fix these findings, a corresponding row MUST be
added to the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions in the same
atomic commit. If the fixes use `?`-propagation instead of new `tracing::*!` emissions, no
catalog row is required (D-765 precedent). The implementer MUST run
`rg 'event_type\s*=' crates/ --type rust` after each fix and confirm every emission has a
catalog row.

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec (v1.8) | ~10,000 |
| BC files (5 BCs) | ~10,000 |
| Source files touched (resources.rs, prompts.rs, prism_describe.rs, error.rs, table_registry.rs, engine.rs, error_mapping.rs) | ~18,000 |
| Research/audit docs (2) | ~6,000 |
| Test files (existing + new — 9 Red Gate tests) | ~6,000 |
| Tool outputs (grep, rg scans, call-chain traces) | ~3,000 |
| **Total estimate** | **~53,000** |

Within the 20-30% context window budget for a Sonnet-class agent context (≈200k tokens).
Story is safe to dispatch as a single story without splitting. (N1-B net-new scope adds ~10k
tokens versus v1.0 estimate; still well within budget.)

---

## Tasks

### N1 — build_reference_content dedup key fix
- [ ] 1. Read `crates/prism-mcp/src/resources.rs`; locate `build_reference_content`;
       identify deduplication logic iterating by `infusion_id`; change dedup key to
       `descriptor.name`; update emitted format to `enrich {name}(col)`.
- [ ] 2. Write Red Gate test `test_bc_2_11_022_n1_per_field_udf_names`; confirm RED.
- [ ] 3. Apply the fix; confirm test GREEN.

### N1-B — E-QUERY-039 net-new implementation
- [ ] 4. Read `crates/prism-core/src/error.rs`; verify `EnrichUdfNotFound` variant DOES NOT
       exist (zero-match prerequisite); add `EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)`
       variant to `PrismError`; add `#[non_exhaustive] pub struct EnrichUdfNotFoundDetails`
       with fields `infusion: String`, `available_infusions: Vec<String>`,
       `did_you_mean: Option<String>`.
- [ ] 5. Increment `ci.yml EXPECTED` from `87` to `88`; update `CLAUDE.md` non-exhaustive
       sentence (87→88) and add `EnrichUdfNotFoundDetails` to the attribution list.
- [ ] 6. Add plan-time enrichment-validation pass in `crates/prism-query/src/engine.rs`
       BEFORE `check_availability_gate`/fan-out; use AST `visit::Visitor` to collect
       enrichment function names — (a) pipe path: `PipeStage::Enrich` nodes → `EnrichStage.infusion`;
       (b) SQL path: `ScalarFunc::Unknown(name)` in SELECT projection expressions (reachable
       from real queries) AND WHERE predicates via `collect_unknown_scalar_from_predicate`
       (DEFENSIVE / forward-compat per BC-2.11.019 v1.3 §Precondition 1(b) AST-contract;
       real `WHERE udf(col)=v` is E-QUERY-001 parse error today; WHERE scan is exercised by
       programmatic AST unit tests, not real parsed query text); these are DISTINCT visitor
       arms but feed the same validation loop. For each collected `name`: if NOT in
       `InfusionRegistry.udf_to_infusion`, build the UDF name vec inline via
       `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect::<Vec<_>>()`
       (do NOT call `udf_names()` — that method does not exist), then return
       `Err(PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails { infusion: name.to_owned(),
       available_infusions: <vec from above>, did_you_mean: strsim_closest(&name, &udf_names_vec) })))`.
       Gate fires BEFORE any fan-out or sensor I/O. No new public methods added to `InfusionRegistry`.
- [ ] 7. Read `crates/prism-mcp/src/error_mapping.rs` `map_prism_error`; add explicit
       `-32602` arm for `PrismError::EnrichUdfNotFound` (E-QUERY-039 — net-new); confirm
       no fall-through to `-32000`. NOTE: the `PrismError::TableNotAvailable` (E-QUERY-037)
       arm is CONFIRMED PRESENT (doc: "S-3.13 AC-2; BC-2.11.001") — do NOT add a duplicate
       or modify it. Only the E-QUERY-039 arm is added here.
- [ ] 8. Write Red Gate test `test_bc_2_11_019_n1b_infusion_id_as_udf_name`; confirm RED.
- [ ] 9. Write Red Gate test `test_bc_2_11_019_n1b_mcp_maps_to_32602`; confirm RED.
- [ ] 10. Apply the plan-time gate + error_mapping.rs fixes; confirm both tests GREEN.

### N2 — E-QUERY-037 dot-notation FROM gate ordering
- [ ] 11. Read `crates/prism-query/src/table_registry.rs`; trace `check_availability_gate`
        and `is_registered`; understand where the availability check fires relative to
        `sensor_id_from_table_name` dot-extraction; determine the correct insertion point
        in engine.rs for the pre-check. Do NOT look at materialization.rs for this fix —
        the gate lives in the table_registry/engine layer.
- [ ] 12. Ensure `TableRegistry::is_registered(table_name_as_written)` check fires in
        `check_availability_gate` BEFORE any fan-out routing; dot-notation strings must
        return `PrismError::TableNotAvailable(...)` (E-QUERY-037) with `did_you_mean`.
- [ ] 13. Write Red Gate test `test_bc_2_11_001_n2_dot_notation_from_target_e_query_037`;
        confirm RED (must cover pipe mode, SQL mode, AND filter-mode regression guard).
- [ ] 14. Apply the gate-ordering fix in table_registry.rs + engine.rs; confirm test GREEN;
        confirm filter-mode `crowdstrike_detections | severity='HIGH'` still passes.

### AUDIT-001 — prism_describe sensor-prefixed table names
- [ ] 15. Read `crates/prism-mcp/src/tools/prism_describe.rs` `build_tables_for_client`;
        change `name: table.table_name.clone()` → `name: format!("{sensor_id}_{}", table.table_name)`;
        update `example_query` grounding to use the same sensor-prefixed name.
- [ ] 16. Write Red Gate test `test_bc_2_10_012_audit_001_sensor_prefixed_table_names`;
        confirm RED.
- [ ] 17. Apply the fix; confirm test GREEN.

### AUDIT-004 — prompts.rs FROM-ready table names
- [ ] 18. Read `crates/prism-mcp/src/prompts.rs`; identify all four affected `render_*`
        functions with dot-notation FROM clauses; determine correct sensor-prefixed names
        from the actual sensor TOML specs (function-name anchor: `render_triage_alerts`,
        `render_client_overview`, `render_cross_client_status`, `render_investigate_host`);
        replace all dot-notation FROM references.
- [ ] 19. Write Red Gate test `test_bc_2_10_016_audit_004_no_dot_notation_in_prompts`;
        confirm RED.
- [ ] 20. Apply the fix; confirm test GREEN.

### Final gates
- [ ] 21. Run `just check` (full workspace); confirm EXIT 0.
- [ ] 22. Verify `grep 'EXPECTED=' ci.yml` shows `88` (not 87).
- [ ] 23. Run `rg 'event_type\s*=' crates/ --type rust`; verify every emission has a
        BC-2.16.002 catalog row (SAP-1 compliance).

---

## Previous Story Intelligence

**Predecessor:** S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (merged PR #203, develop@7e60df03).

**Key lessons from the predecessor cascade (directly applicable here):**

1. **TD-VSDD-091 line-number pins are fragile.** This story uses function-name anchors
   throughout (e.g., `build_reference_content`, `sensor_id_from_table_name`,
   `build_tables_for_client`, `render_triage_alerts`) — never file:line references.
   Verify all code citations use function names, not line numbers, before declaring done.

2. **`build_reference_content` is in `resources.rs`, NOT `resources/schema.rs`.** The
   old static `PQL_REFERENCE_CONTENT` include_str! was in `resources/schema.rs`; the new
   dynamic `build_reference_content` introduced by PR #203 is in `resources.rs`. The
   implementer MUST read and confirm the actual location before writing tests.

3. **SAP-1 tracing catalog discipline.** Every `event_type =` tracing emission must have
   a BC-2.16.002 catalog row. The predecessor had two recurrences of this finding (passes
   1 and 2). This story's fixes are likely to use `?`-propagation without new emissions —
   but the SAP-1 check (`rg 'event_type\s*='`) MUST still be run post-fix.

4. **SAP-2 does NOT apply here.** SAP-2 (DTU↔TOML schema parity) applies to sensor TOML
   spec changes. This story does not modify sensor TOML files — it modifies MCP layer code
   that reads from existing specs. SAP-2 scan is NOT needed.

5. **E-QUERY-039 (N1-B) is NET-NEW, not an investigation.** A 2026-06-26 remove-uncertainty
   pass confirmed that `PrismError::EnrichUdfNotFound` and `EnrichUdfNotFoundDetails` have
   ZERO workspace matches — the variant, struct, plan-time gate, and MCP mapping all need to
   be created from scratch per BC-2.11.019 v1.3. The original remediation plan framed this as
   a "gate should fire / routing fix" but that was based on the incorrect assumption that PR
   #203 implemented E-QUERY-039. It did not. The implementer MUST create the error type first
   (error.rs), then the gate (prism-query/engine.rs), then the MCP mapping (error_mapping.rs),
   in that order. TDD discipline: write `test_bc_2_11_019_n1b_infusion_id_as_udf_name` (RED)
   before adding any gate code.

   **CRITICAL (v1.2):** `InfusionRegistry` has NO `udf_names()` method. The public API is:
   `new`, `load_spec`, `load_spec_with_runtime`, `udf_descriptors`, `enrich_descriptor`,
   `is_api_backed`, `hot_reload`. Do NOT add a new method. Derive the UDF name vector inline:
   `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect::<Vec<_>>()`.
   The E-QUERY-037 `map_prism_error` arm is CONFIRMED PRESENT (doc: "S-3.13 AC-2") — the
   v1.1 story text incorrectly marked it as net-new. Only the E-QUERY-039 arm is net-new.
   The enrichment gate insertion anchor is `engine.rs` (new pass before `check_availability_gate`),
   not an unspecified file to be determined by the implementer.

6. **Forbidden dependencies are unchanged from the predecessor.** `prism-query` MUST NOT
   depend on `prism-mcp`. The E-QUERY-039 gate lives in `prism-query`; its error type is
   in `prism-core`; the MCP mapping is in `prism-mcp/src/error_mapping.rs`. This layering
   is already established by the predecessor story.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and the predecessor story's lessons.
These rules are binding for this story.

1. **Dependency direction:** `prism-query` must NOT import from `prism-mcp`. Error types
   (`PrismError::EnrichUdfNotFound`, `PrismError::TableNotAvailable`) live in `prism-core`.
   MCP mapping (`map_prism_error`) lives in `prism-mcp/src/error_mapping.rs`. This direction
   MUST be preserved.

2. **`#[non_exhaustive]` discipline and InfusionRegistry API surface:** No new public struct
   or enum field may be added without `#[non_exhaustive]` and a corresponding `ci.yml EXPECTED`
   increment. This story DOES introduce exactly ONE new public type: `EnrichUdfNotFoundDetails`
   (for AC-N1B). It MUST carry `#[non_exhaustive]`. `ci.yml EXPECTED` increments 87→88. The
   CLAUDE.md non-exhaustive sentence is updated 87→88 with `EnrichUdfNotFoundDetails` in the
   attribution list. These three changes are REQUIRED — not conditional on "if a new type is needed."

   **No new public methods on `InfusionRegistry`** (I1 v1.2): The UDF name set needed for
   `available_infusions` and strsim candidates MUST be derived from the EXISTING public method
   `udf_descriptors()`: `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect()`.
   Do NOT introduce a `udf_names()` accessor or any other new public method — this keeps the
   new-#[non_exhaustive]-type count at exactly ONE (EnrichUdfNotFoundDetails → EXPECTED 87→88,
   not 87→89).

3. **`InfusionRegistry` reload-awareness:** The `build_reference_content` function receives
   `InfusionRegistry` via `Arc<ArcSwap<InfusionRegistry>>` at request time (per BC-2.11.022
   invariant and ADR-042). The N1 fix (change dedup key) must NOT change the reload pattern.
   No caching of the assembled string.

4. **`TableRegistry` is source of truth for table availability.** `TableRegistry` stores
   only underscore-qualified keys (e.g., `"cyberint_alerts"`). The E-QUERY-037 gate consults
   `TableRegistry::is_registered(table_name_as_written)` first. For N2, the gate ordering
   fix ensures this check precedes `sensor_id_from_table_name` dot-extraction — NOT the
   other way around. ADR-046 filter-mode dot-notation is NOT handled via `TableRegistry`
   (filter-mode uses `<table_name> | <predicate>` syntax; the `<table_name>` in filter
   mode uses underscore-qualified names like `crowdstrike_detections`, not dot-syntax).

5. **`sensor_id_from_table_name` dot-notation extraction must NOT be removed.** This function
   was intentionally extended by PR #203 for BC-2.11.023 filter-mode source refs. The N2
   fix is gate ORDERING only: `TableRegistry::is_registered` first, then fan-out. The
   dot-notation extraction stays in the codebase for its legitimate filter-mode use case.

6. **Forbidden dependencies (module perimeter):**
   - `prism-mcp` must NOT depend on `prism-query` internals (parser types); it accesses
     only the `PrismError` type from `prism-core`.
   - `prism-query` must NOT depend on `prism-mcp`.
   - `crates/prism-core/src/error.rs` is the authoritative `PrismError` location.

---

## Library & Framework Requirements

These version pins are from the `Cargo.lock` at develop HEAD `7e60df03` (authoritative).
The implementer MUST use these exact versions — no drift.

| Crate | Version | Use in this story |
|-------|---------|------------------|
| `strsim` | per Cargo.lock (same version used by E-QUERY-037/038) | `did_you_mean` Levenshtein computation in E-QUERY-039; already in use — no new dep |
| `tokio` | per Cargo.lock | async test harness for timing assertions (AC-REG-2 if needed) |
| `datafusion` | per Cargo.lock | DataFusion plan-time API used in materialization.rs gate; no version change |
| `serde_json` | per Cargo.lock | JSON shape assertions in prism_describe tests |

No new dependencies are introduced by this story. All fixes modify existing code paths using
already-present library calls.

---

## File Structure Requirements

Files to modify (v1.2 — E-QUERY-039 net-new in prism-core + engine.rs gate; E-QUERY-037 map_prism_error arm confirmed-present, no change):

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-core/src/error.rs` | MODIFY (NET-NEW variant+struct) | N1-B Step 1: add `PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` variant; add `#[non_exhaustive] pub struct EnrichUdfNotFoundDetails { pub infusion, pub available_infusions, pub did_you_mean }` |
| `crates/prism-mcp/src/resources.rs` | MODIFY | N1: change `build_reference_content` dedup key from `desc.infusion_id` to `desc.name`; add test `test_bc_2_11_022_n1_per_field_udf_names` in `#[cfg(test)]` module |
| `crates/prism-mcp/src/error_mapping.rs` | MODIFY (E-QUERY-039 arm net-new; E-QUERY-037 arm confirmed-present, no change) | N1-B Step 3: add explicit `-32602` INVALID_PARAMS arm for `PrismError::EnrichUdfNotFound` (E-QUERY-039) — this is the ONLY net-new arm; MUST NOT fall through to `-32000` catch-all. `PrismError::TableNotAvailable` (E-QUERY-037) arm is CONFIRMED PRESENT (doc: "S-3.13 AC-2; BC-2.11.001") — no modification needed. |
| `crates/prism-query/src/table_registry.rs` | MODIFY | N2: in `check_availability_gate` and/or `is_registered`, ensure `TableRegistry::is_registered(table_name_as_written)` check runs BEFORE any fan-out routing — so dot-notation strings like `cyberint.alerts` return `PrismError::TableNotAvailable` (E-QUERY-037) with `did_you_mean`; trace actual call chain first |
| `crates/prism-query/src/engine.rs` | MODIFY | N2: ensure the table availability gate from table_registry.rs is wired at the correct engine layer entry point (where the plan-time check fires before fan-out is dispatched) |
| `crates/prism-query/src/engine.rs` (enrichment gate) | MODIFY | N1-B Step 2: add plan-time enrichment-validation pass BEFORE `check_availability_gate`/fan-out in `engine.rs`; use AST `visit::Visitor` to collect enrichment names from BOTH paths — (a) pipe: `PipeStage::Enrich` → `EnrichStage.infusion`; (b) SQL: `ScalarFunc::Unknown(name)` in SELECT projection expressions (reachable from real queries via `build_sql_expr_parser`) AND WHERE predicates via `collect_unknown_scalar_from_predicate` (DEFENSIVE / forward-compat per BC-2.11.019 v1.3 §Precondition 1(b) AST-contract; real `WHERE udf(col)=v` is E-QUERY-001 parse error — `build_predicate_parser` has no scalar-funcall atom; WHERE scan is exercised by programmatic AST unit tests); validate each against `registry.udf_descriptors()` (NO new public API on `InfusionRegistry` — derive names inline); unknown name → `PrismError::EnrichUdfNotFound`; function-name anchor required (TD-VSDD-091). |
| `crates/prism-mcp/src/tools/prism_describe.rs` | MODIFY | AUDIT-001: change `build_tables_for_client` emit from `name: table.table_name.clone()` → `name: format!("{sensor_id}_{}", table.table_name)`; add test |
| `crates/prism-mcp/src/prompts.rs` | MODIFY | AUDIT-004: replace all dot-notation FROM refs in `render_triage_alerts`, `render_client_overview`, `render_cross_client_status`, `render_investigate_host` with sensor-prefixed underscore-qualified names; add test |
| `ci.yml` | MODIFY | AC-REG-1: increment `EXPECTED=87` → `EXPECTED=88` (one new #[non_exhaustive] type: EnrichUdfNotFoundDetails) |
| `CLAUDE.md` | MODIFY | AC-REG-1: update non-exhaustive sentence count 87→88; add `EnrichUdfNotFoundDetails` to attribution parenthetical |

Files NOT to modify:
- BC files (PO owns them — this story reads BCs, never modifies them)
- `.factory/STATE.md` (state-manager owns it)
- `crates/prism-query/src/materialization.rs` — resolve_source_refs returns E-QUERY-036 (UnknownSourceTable) via a DIFFERENT path; N2 fix is in table_registry.rs/engine.rs, not here. Do NOT add E-QUERY-037 logic to materialization.rs.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `build_reference_content` | `crates/prism-mcp/src/resources.rs` | Pure (takes `Option<&InfusionRegistry>`, returns `String`) |
| `build_tables_for_client` | `crates/prism-mcp/src/tools/prism_describe.rs` | Pure (takes spec data, returns `Vec<TableDescriptor>`) |
| `render_triage_alerts` / `render_client_overview` / `render_cross_client_status` / `render_investigate_host` | `crates/prism-mcp/src/prompts.rs` | Pure (synchronous render functions per BC-2.10.016 invariant) |
| `PrismError::EnrichUdfNotFound` variant + `EnrichUdfNotFoundDetails` struct (NET-NEW) | `crates/prism-core/src/error.rs` | Pure (data type, no I/O) |
| E-QUERY-039 plan-time enrichment gate (NET-NEW) — pipe: `PipeStage::Enrich` → `EnrichStage.infusion`; SQL: `ScalarFunc::Unknown` in SELECT projection expressions (reachable from real queries) AND WHERE predicates via `collect_unknown_scalar_from_predicate` (DEFENSIVE / forward-compat per BC-2.11.019 v1.3 §Precondition 1(b) AST-contract; real `WHERE udf(col)=v` is E-QUERY-001 parse error — `build_predicate_parser` has no scalar-funcall atom; WHERE scan exercised by programmatic AST unit tests) | `crates/prism-query/src/engine.rs` (new validation pass before `check_availability_gate`; no new public API on `InfusionRegistry` — names derived from existing `udf_descriptors()`) | Pure (takes `&InfusionRegistry`, returns `Result<_, PrismError>`) |
| E-QUERY-037 plan-time availability gate — `check_availability_gate` / `is_registered` | `crates/prism-query/src/table_registry.rs` | Pure (gate check, no I/O) |
| E-QUERY-037 gate wiring point | `crates/prism-query/src/engine.rs` | Pure (plan-time orchestration) |
| `map_prism_error` (E-QUERY-039 arm NET-NEW; E-QUERY-037 arm CONFIRMED PRESENT — no change) | `crates/prism-mcp/src/error_mapping.rs` | Pure (mapping function) |

---

## UX References

N/A — this is a server-side correctness story with no UI component. The demo-recorder will
capture evidence of the fixed MCP tool output and prompt rendering as per AC-DEMO-001.

---

## Dependencies

| Type | Story | Reason |
|------|-------|--------|
| `depends_on` | S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 | build_reference_content, materialization.rs gate structure, and prompts.rs prompt dispatch — all introduced by PR #203 — are the code surfaces being modified. Dependency anchor: these code surfaces must exist (merged) before this fix story can be written and tested against them. |
| `blocks` | (none) | No subsequent story depends on these fixes; they unblock the T13 demo recording (human-scheduled milestone, not a story dependency). |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | N1: `InfusionRegistry` has zero infusions registered at reference request time | `build_reference_content(Some(&registry))` returns enrichment section with "No enrichment functions are currently registered for your deployment." (not six UDF entries). Test: `test_bc_2_11_022_some_empty_registry_placeholder` (new, covers the Some(empty) path) must pass. |
| EC-002 | N1-B: Calling `threat_intel(iocs_value)` where `threat_intel` IS registered as a UDF name (hypothetical future; currently it is NOT) | E-QUERY-039 does NOT fire — the gate only fires when the name is absent from `udf_to_infusion`. This edge case is the negative control for the fix — confirm the test fixture correctly registers only per-field names, not the infusion_id. |
| EC-003 | N2: Filter-mode query `crowdstrike_detections | severity='HIGH'` continues to work after gate-ordering fix | Gate-ordering fix must NOT alter filter-mode behavior. `Ast::Filter` path does not use `FROM <table>` syntax; the TableRegistry check in the fix applies only to the `FROM`-target path (SQL/Pipe/SqlPipe modes). |
| EC-004 | N2: `SELECT * FROM crowdstrike.detections` (SQL mode, dot-notation) also returns E-QUERY-037 | The fix applies to all modes (SQL, Pipe, SqlPipe) per EC-11-067 in BC-2.11.001 v1.15. The Red Gate test must cover both pipe and SQL mode. |
| EC-005 | AUDIT-001: `prism_describe(org-c)` for a sensor with a table name already containing an underscore (hypothetical, e.g., `audit_logs`) | `format!("{sensor_id}_{}", table.table_name)` produces `"claroty_audit_logs"` — correct. No double-underscore issue since `sensor_id` is a simple identifier (`claroty`) and `table_name` is the TOML value (`audit_logs`). |
| EC-006 | AUDIT-004: `render_query_tutorial` (the one clean prompt) must not be inadvertently broken | The fix targets only the four affected prompts. `render_query_tutorial` already uses `<sensor_table>` placeholder syntax (no hardcoded table names) — MUST NOT be modified. |

---

## Estimated Complexity

**8 story points.**

Root causes are all confirmed, code paths are known, and BCs are in place. The implementation
is a set of targeted, surgical code changes:
- N1: one-line dedup key change in `build_reference_content` + test
- N1-B: net-new error type (error.rs) + AST-visitor enrichment gate in engine.rs (pipe+SQL paths, derive names from udf_descriptors()) + map_prism_error -32602 arm (E-QUERY-039 only) + tests
- N2: gate ordering resequence in `run_materialization_pipeline` + test (with filter-mode regression guard)
- AUDIT-001: one-line format change in `build_tables_for_client` + test
- AUDIT-004: string replacement across four `render_*` functions + test (requires reading sensor TOML specs for correct table names)

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.9 | med-1-re-correction-where-clause-code-verified-2026-06-27 | 2026-06-27 | story-writer | MED-1 re-correction: AC-N1B WHERE note aligned to code-verified reality (over-corrected in v1.8). `build_predicate_parser` has no scalar-funcall atom → real `WHERE udf(col)=v` is E-QUERY-001 parse error; `collect_unknown_scalar_from_predicate` WHERE scan is DEFENSIVE/forward-compat (programmatic AST), honoring BC-2.11.019 §Precondition 1(b) AST-contract; SQL projection is the reachable gated path. `ScalarFunc::Unknown` is produced ONLY by `build_sql_expr_parser` (SELECT projections), not by the WHERE predicate grammar. Five locations corrected: (1) AC-N1B WHERE-clause note block quote; (2) AC-N1B Step 2 SQL path bullet; (3) Tasks step 6(b); (4) File Structure table engine.rs enrichment gate row; (5) Architecture Mapping E-QUERY-039 gate row. Matches the implementing test docstring in `crates/prism-query/src/tests/bc_2_11_019_n1b_test.rs` (~lines 355-366). Version bump 1.8→1.9. |
| 1.8 | med-1-where-clause-note-correction-2026-06-27 | 2026-06-27 | story-writer | MED-1: corrected AC-N1B WHERE-clause note — SQL-mode `ScalarFunc::Unknown` gating covers projection AND WHERE per BC-2.11.019 v1.3 §Precondition 1(b) (the WHERE scan is required+implemented via `collect_unknown_scalar_from_predicate`, not "defensive/unneeded"); only the pipe `enrich`-keyword WHERE form is an E-QUERY-001 parse error. Five locations fixed: (1) AC-N1B WHERE-clause note block quote (lines ~249-261); (2) AC-N1B Step 2 SQL path bullet; (3) Tasks step 6(b); (4) File Structure table engine.rs enrichment gate row; (5) Architecture Mapping E-QUERY-039 gate row. Incorrect assertions "no WHERE-clause scan is needed" and "projection-arm scan is COMPLETE coverage" removed. Version bump 1.7→1.8. |
| 1.7 | low-1-ec001-exhaustive-claim-audit-2026-06-27 | 2026-06-27 | story-writer | LOW-1 + EXHAUSTIVE whole-story claim audit: corrected EC-001 phantom string `"No enrichment infusions are currently registered."` → actual code string `"No enrichment functions are currently registered for your deployment."` (resources.rs ~line 1519, Some(empty) path); updated EC-001 test cite from `test_bc_2_11_022_none_registry_placeholder` (covers None path) → `test_bc_2_11_022_some_empty_registry_placeholder` (covers the Some(empty) path). Second inaccuracy found and fixed: frontmatter Red Gate test comment listed phantom test `test_non_exhaustive_count_87_to_88` — this test does not exist; the non-exhaustive gate is a compile-fail crate run via `scripts/check-non-exhaustive.sh EXPECTED=88`, not a named Rust `fn test_*`; corrected to `scripts/check-non-exhaustive.sh EXPECTED=88 (compile-fail gate via shell script, not a named Rust test)`. All other claims across every section verified accurate against feature worktree code: AC-N1 (resources.rs dedup key, per-field UDF names, test name), AC-N1B (EnrichUdfNotFoundDetails struct fields, map_prism_error -32602 arm, Display template, gate ordering, test names), AC-N2 (check_availability_gate / is_registered function names, TableNotAvailable variant, udf_to_infusion field), AC-AUDIT-001 (build_tables_for_client format string, pql_hints generic hint text), AC-AUDIT-004 (render_* function names, FROM-ready table names, regex), AC-REG-1/REG-2/DEMO-001/SAP-1, §Edge Cases (EC-002 through EC-006), §Red Gate Tests table (all other names confirmed present), §File Structure, §Library Requirements, §Architecture Mapping, §Dev Notes, §Previous Story Intelligence. No further inaccuracies found. Token Budget label 1.6→1.7. Version bump 1.6→1.7. |
| 1.6 | obs-1-ac-prose-accuracy-audit-2026-06-27 | 2026-06-27 | story-writer | OBS-1 + AC-prose accuracy audit: corrected AC-AUDIT-001 phantom `pql_hints[0]` "This client has N tables:" string claim — that string does not exist in `build_pql_hints`; the actual non-empty `pql_hints[0]` is `"Use 'SELECT * FROM <table> LIMIT 25' to query any of the N table(s) above."` (a generic usage hint with `<table>` placeholder, no embedded table names). The disambiguation guarantee is in `TableDescriptor.name` + `example_query`, not in `pql_hints`. Full AC-prose-vs-code accuracy sweep (AC-N1, AC-N1B, AC-N2, AC-AUDIT-004, AC-REG-1, AC-REG-2, AC-DEMO-001, AC-SAP-1): all other AC prose matches code — no further inaccuracies found. Token Budget label 1.5→1.6. Version bump 1.5→1.6. |
| 1.5 | med-1-exhaustive-all-forms-bc-cite-audit-2026-06-27 | 2026-06-27 | story-writer | Exhaustive all-forms BC version-cite audit (Pass MED-1): residual compact-form `(v1.14/v1.1/v1.3/v1.4/v1.2)` at frontmatter line 69 contained stale `v1.14` for BC-2.11.001 (canonical v1.15) — missed by prior prefixed-grep sweeps that searched `BC-2.11.001 v1.14` but not the parenthesized slash-joined form. Redundant compact version enumeration removed from frontmatter comment (drift risk; body BC table is canonical); replaced with accurate status note: 4 active BCs + BC-2.11.019 draft→active at merge per POL-14. Descriptor accuracy fix: prior comment said "all 5 BCs are active" — BC-2.11.019 is `status: draft` (confirmed against BC frontmatter). All prefixed-form cites verified correct (zero stale). Re-grep confirms ZERO live stale `v1.14` or stale compact-form cites remain (changelog rows excepted, TD-VSDD-091 exempt). Token Budget version label updated 1.4→1.5. Version bump 1.4→1.5. |
| 1.4 | pass-5-bc-version-cite-sweep-2026-06-27 | 2026-06-27 | story-writer | Comprehensive BC version-cite sweep (Pass-5 MED-1/MED-2): BC-2.11.001 v1.14→v1.15 (5 sites: frontmatter comment line 62, BC table line 175, AC-N2 header trace line 338, AC-REG-1 trace line 415, EC-004 table line 748); BC-2.11.019 v1.2→v1.3 (2 residual sites the v1.3 "throughout" claim missed: AC-N1B scope note line 222, Previous Story Intelligence line 596). Token Budget version label updated 1.3→1.4. Frontmatter version bumped 1.3→1.4. BC-2.11.022 v1.1, BC-2.10.016 v1.2, BC-2.10.012 v1.4 verified current — no changes needed. POL-29 version-cite recurrence break. |
| 1.3 | po-e-query-039-reconciliation-2026-06-27 | 2026-06-27 | story-writer | HIGH-005 changelog reorder (POL-32 monotonic_descending violation — rows were 1.0→1.2→1.1; reordered to strict descending 1.3→1.2→1.1→1.0). Sync to PO E-QUERY-039 reconciliation: (1) canonical Display message template added to AC-N1B Step 3 — EXACTLY `E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}` (bracket-wrapped comma-joined Vec<String>; did_you_mean = ` Did you mean: '{x}'?` when Some, omitted when None); (2) available_infusions confirmed as `Vec<String>` (PO-ratified canonical type — already matched struct definition; now explicit in AC text and observable behavior); (3) enrich-LAST gate ordering added as explicit callout in AC-N1B: E-QUERY-039 fires LAST (E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039); (4) WHERE-clause note added: SQL-mode ScalarFunc::Unknown gate covers projections only — WHERE-clause enrichment calls are E-QUERY-001 parse errors at the grammar level; the projection-arm scan is complete coverage; defensive visitor arm noted for programmatic AST; (5) BC-2.11.019 version references updated v1.2→v1.3 throughout (BC table, AC-N1B header trace, frontmatter I2 anchor comment, crates_touched comment, BC status comment). LOCAL Pass-1 fix-burst closures noted for record: CRIT-001 (prompt table names), HIGH-001 (gate ordering), HIGH-002 (Display template), HIGH-003 (WHERE scan), OBS-1 (tie-break), OBS-2 (doc-comment). Version bump 1.2→1.3. |
| 1.2 | pre-tdd-api-mismatch-corrections-2026-06-26 | 2026-06-26 | story-writer | Four internal-API mismatch corrections found before TDD delivery. C1 (HIGH): E-QUERY-037 `map_prism_error` arm CONFIRMED PRESENT in error_mapping.rs (~line 166, doc "S-3.13 AC-2; BC-2.11.001") — v1.1 wrongly marked it net-new; corrected throughout (AC-N1B, AC-N2, File Structure table, Tasks, Architecture Mapping). I1 (HIGH): `InfusionRegistry.udf_names()` does NOT exist — only `udf_descriptors()` exists; corrected gate code to derive UDF names inline via `udf_descriptors().iter().map(|d| d.name.clone()).collect()` throughout; no new public method added (keeps EXPECTED increment at exactly 87→88). I2 (MED-HIGH): enrichment-gate insertion point pinned to `engine.rs` (new AST-visitor pass before `check_availability_gate`; pipe arm: `PipeStage::Enrich` → `EnrichStage.infusion`; SQL arm: `ScalarFunc::Unknown` in projections; both distinct visitor arms, same validation loop); "implementer determines exact file" language removed. S1 (LOW): Red Gate test for N1-B relaxed — `did_you_mean.is_some()` assertion removed; test must assert `available_infusions` non-empty and error variant/code only (registered per-field UDF names likely > Levenshtein-3 from "threat_intel" so `did_you_mean: None` is valid). Points/Red Gate test count/token budget UNCHANGED from v1.1 (10 pts / 9 tests / ~53k tokens). |
| 1.1 | remove-uncertainty-scope-correction-2026-06-26 | 2026-06-26 | story-writer | Scope corrections from post-materialization remove-uncertainty pass. THREE HIGH findings resolved: (1) N1-B re-scoped net-new — EnrichUdfNotFound variant + EnrichUdfNotFoundDetails struct do NOT exist in workspace (zero matches); AC-N1B now requires creating error.rs variant + plan-time gate (prism-query) + map_prism_error -32602 arm (error_mapping.rs) from scratch; story-investigation framing removed; BC-2.11.019 promotes draft→active at merge (POL-14). (2) N2 gate-ordering fix relocated from materialization.rs to table_registry.rs (check_availability_gate / is_registered) + engine.rs; materialization.rs resolve_source_refs is a DIFFERENT code path (E-QUERY-036, not E-QUERY-037); AC-N2 scope note + scope-note re-written accordingly. (3) AC-REG-1 amended: previously incorrectly stated no new #[non_exhaustive] types; now REQUIRES EnrichUdfNotFoundDetails with #[non_exhaustive]; ci.yml EXPECTED 87→88; CLAUDE.md sentence updated. map_prism_error E-QUERY-037 arm also changed from conditional to net-new. crates_touched expanded: + prism-core/src/error.rs, + prism-query/src/table_registry.rs, + prism-query/src/engine.rs. Points: 8→10. Red Gate tests: 7→9. Token budget: ~43k→~53k. |
| 1.0 | demo-fidelity-remediation-2026-06-26 | 2026-06-26 | story-writer | Initial story. Materializes the 5 code-fix ACs from the 2026-06-26 pre-flight audit remediation plan. Traces to BC-2.11.001 v1.14 (EC-11-067 N2), BC-2.11.022 v1.1 (EC-11-022-006 N1), BC-2.11.019 v1.2 (N1-B), BC-2.10.012 v1.4 (AUDIT-001), BC-2.10.016 v1.2 (AUDIT-004). 10 ACs; 7 Red Gate tests; 8 pts; P0; depends_on S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001. |
