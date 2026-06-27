---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-23T00:00:00Z
phase: 1a
inputs: [".factory/specs/domain-spec/capabilities.md", ".factory/specs/domain-spec/invariants.md", ".factory/specs/architecture/decisions/ADR-041-prismql-llm-auto-onboarding-4-layer-teaching-surface-for-automatic-agent-query-authoring.md"]
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: draft
introduced: 2026-06-23
modified: 2026-06-27
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.019: E-QUERY-039 Enrich-UDF-Not-Found Plan-Time Gate

## Description

When a PrismQL query — in pipe mode (`| enrich <infusion>(<column>)`) or in SQL mode (`SELECT <infusion>(<column>) FROM ...`) — invokes an enrichment infusion name that is not registered in the process-global `InfusionRegistry`, the query engine rejects the query at plan time (before any sensor fan-out or DataFusion execution) with `E-QUERY-039`. The error payload carries `infusion` (the unrecognized infusion name as written), `available_infusions` (`Vec<String>` of all `InfusionField.name` values in the global `InfusionRegistry`, rendered in the message as a bracket-wrapped comma-joined list), and `did_you_mean` (Levenshtein ≤ 3 match against the global set, if any). This prevents an opaque runtime `E-INT-001` crash (AUDIT-005 root cause) and enables the LLM agent to self-correct the enrichment call in a single retry without human intervention. The gate fires at the same plan-time validation point as E-QUERY-037 (table) and E-QUERY-038 (column).

## Preconditions

1. A PQL query has been submitted containing either:
   - (a) **Pipe mode:** a `| enrich <infusion>(<column>)` stage (parsed successfully with no E-QUERY-001 error), where `EnrichStage.infusion` holds the enrichment function name token; OR
   - (b) **SQL mode:** a `SELECT` projection or `WHERE` clause containing `FuncCall::Scalar { func: ScalarFunc::Unknown(name), args }` — an analyst-defined UDF call that was not resolved to a built-in `ScalarFunc` variant.
2. For pipe mode: the parser has successfully identified an `EnrichStage` node with an `infusion` token.
3. For SQL mode: the Chumsky parser has produced a `FuncCall::Scalar { func: ScalarFunc::Unknown(name) }` node — the `Unknown` escape hatch for enrichment function names not mapped to built-in variants.
4. The `InfusionRegistry` (`prism_spec_engine::infusion`) has been initialized at boot from `{config_dir}/infusions/*.infusion.toml` — its `InfusionRegistryInner.udf_to_infusion` map (keyed by `InfusionField.name`) is available at plan time.
5. This gate is a no-op for queries that contain neither `EnrichStage` nodes nor `ScalarFunc::Unknown` nodes.

## Postconditions

### Gate firing condition

E-QUERY-039 fires when EITHER of the following is true at plan time:
- A pipe-mode query contains an `EnrichStage` node whose `infusion` token is NOT a key in `InfusionRegistry.udf_to_infusion`; OR
- A SQL-mode query contains a `FuncCall::Scalar { func: ScalarFunc::Unknown(name) }` node (in `SELECT` projection or `WHERE`) where `name` is NOT a key in `InfusionRegistry.udf_to_infusion`.

AND:
- The query has already passed the E-QUERY-037 table-availability check (if the table itself is missing, E-QUERY-037 fires first).

The gate fires at plan time, after AST parse and before fan-out and before DataFusion execution. No sensor API call is made for a rejected query.

### E-QUERY-039 error payload shape

```
E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}
```

**Payload fields:**

- `infusion`: the exact enrichment function name token as written in the query (e.g., `"threat_score"`, `"cvss"`)
- `available_infusions`: a `Vec<String>` of ALL `InfusionField.name` values in the process-global `InfusionRegistry` at plan time. When no infusions are registered, this is an empty `Vec` (renders as `[]` in the Display output). This field is ALWAYS present. The Display impl comma-joins the Vec and wraps it in square brackets — e.g., `[threat_score, nvd_cvss]`; an empty Vec renders as `[]`. **Global scope:** `available_infusions` reflects ALL `InfusionField.name` values across the entire `InfusionRegistryInner.udf_to_infusion` map — there is no per-org keying in the current implementation (infusions are shared across all orgs). See §Follow-Up Story Anchor for the per-org scoping deferral.
- `did_you_mean`: `Option<String>`. Present when the Levenshtein distance between `infusion` and the closest `InfusionField.name` in the global registry is ≤ 3 (implementation: `strsim::levenshtein`, same crate used by E-QUERY-037 and E-QUERY-038). When present, contains the single closest-match infusion name from the global set, rendered as `" Did you mean: '{best_match}'?"` (leading space, one candidate — consistent with E-QUERY-037/038 convention). When absent (no match within threshold), the field is `None` (omitted from the Display output — not rendered as empty string). **Org-scoping note:** computed against the global registry; per-org candidate filtering deferred to the §Follow-Up Story Anchor.

### MCP surface

E-QUERY-039 surfaces as MCP `-32602 INVALID_PARAMS` (caller-resolvable — the model supplied an unrecognized enrichment function name; it can retry with a registered name). This is consistent with E-QUERY-037 and E-QUERY-038 mapping.

The error is delivered as a BC-2.10.007 structured error response with:
- `code: "E-QUERY-039"`
- `category: "validation"`
- `severity: "broken"` (query cannot proceed without correction)
- `retryable: false` (without a configuration change or corrected infusion name)
- `suggestion`: when `available_infusions` is non-empty: `"Use one of the registered enrichment functions: {available_infusions}. Call prism_describe('<client_id>') to see pql_hints including available enrichment functions."` When `available_infusions` is empty: `"No enrichment functions are registered. Enrichment is not available in this deployment."`

### PrismError variant

`PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` in `prism-core/src/error.rs`, where `EnrichUdfNotFoundDetails` is a `#[non_exhaustive]` struct carrying:

```rust
pub struct EnrichUdfNotFoundDetails {
    pub infusion: String,
    pub available_infusions: Vec<String>,   // all registered InfusionField.name values; empty Vec when none
    pub did_you_mean: Option<String>,
}
```

**`available_infusions` rendering:** the `Display` implementation MUST comma-join the `Vec<String>` and wrap it in square brackets, e.g., `[threat_score, nvd_cvss]`. When the Vec is empty the rendered form MUST be `[]`. This bracket convention is consistent with `ColumnNotFoundDetails.available_columns` (E-QUERY-038 sibling). The canonical message format the Display MUST produce byte-for-byte is: `"E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}"` where `{available_infusions}` is replaced by the comma-joined Vec content (no additional brackets added — the brackets are part of the format string literal).

The `#[non_exhaustive]` attribute on `EnrichUdfNotFoundDetails` is required per CLAUDE.md `#[non_exhaustive]` discipline (public type in `prism-core`). This adds +1 to the `ci.yml EXPECTED` non-exhaustive gate count (currently 83; increment to 84). External match arms must include a wildcard `_ => {}` arm.

`map_prism_error` in `prism-mcp/src/error_mapping.rs` MUST add an explicit `-32602 INVALID_PARAMS` arm for `PrismError::EnrichUdfNotFound` — it MUST NOT fall through to the `#[non_exhaustive]` catch-all `-32000`.

### Gate ordering

The full plan-time gate sequence is:
1. E-QUERY-001 — parse error (Chumsky cannot parse the query at all)
2. E-QUERY-037 — table not in `TableRegistry` (fires before column/enrichment checks)
3. E-QUERY-038 — column not found in table schema
4. **E-QUERY-039** — enrichment infusion name not registered in `InfusionRegistry` (fires LAST among the plan-time validation gates — after both E-QUERY-037 table check and E-QUERY-038 column check have passed; applies to both pipe-mode `EnrichStage.infusion` and SQL-mode `ScalarFunc::Unknown(name)`)
5. E-QUERY-034 — fallback for DataFusion execution failures

**Implementer note (addresses HIGH-001 from S-DEMO-FIDELITY-REMEDIATION-001 adversary pass):** The code currently fires E-QUERY-039 BEFORE E-QUERY-037 and E-QUERY-038. This is wrong. The implementation MUST perform the checks in the order listed above: table check → column check → enrichment check. The spec ordering is the canonical ordering; the code must be corrected to match it.

### Closes AUDIT-005

AUDIT-005 reported that `cvss(device_cves_first)` — a scalar function-call projection in a SQL `SELECT` statement (`FuncCall::Scalar { func: ScalarFunc::Unknown("cvss"), args: [device_cves_first] }`) — triggered `E-INT-001 "Internal error; see audit log"`, an opaque, unactionable error for MCP clients who cannot see the audit log. The root cause was an unregistered function name reaching DataFusion execution without a plan-time gate.

E-QUERY-039 gates BOTH AST paths at plan time:
1. **Pipe-mode `EnrichStage.infusion`** — `| enrich cvss(device_cves_first)` is caught when `EnrichStage.infusion == "cvss"` is not in `InfusionRegistry.udf_to_infusion`.
2. **SQL-mode `ScalarFunc::Unknown(name)`** — the actual AUDIT-005 reproducer: `SELECT cvss(device_cves_first) FROM armis_devices` produces `FuncCall::Scalar { func: ScalarFunc::Unknown("cvss") }`, which is caught when `"cvss"` is not in `InfusionRegistry.udf_to_infusion`. This prevents E-INT-001 from surfacing for any unregistered enrichment function name in a SELECT projection or WHERE clause.

## Invariants

- DI-019: This gate fires at plan time, before fan-out, consistent with the security limits principle that problems are caught early.
- DI-004: The query rejection event is included in the `AuditEntry` for the `query` tool call (outcome: `"rejected"`, reason: `"enrich_udf_not_found"`).
- DI-002: `available_infusions` contains no credential values — infusion names are operator-defined `InfusionField.name` identifiers from `*.infusion.toml` spec files; no sensor API response data.

## Error Cases

| Error Code | Condition | Behavior |
|------------|-----------|----------|
| `E-QUERY-039` | Pipe-mode query contains `\| enrich <infusion>(...)` where `infusion` is not registered in `InfusionRegistry`, OR SQL-mode query contains `ScalarFunc::Unknown(name)` in SELECT/WHERE where `name` is not registered | MCP `-32602 INVALID_PARAMS`; structured payload with `infusion`, `available_infusions` (always present, global set), `did_you_mean` (when within distance ≤ 3 of any global infusion name) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-057 | `\| enrich threet_score(ioc_value)` (typo — missing 'a') where `threat_score` is registered in `InfusionRegistry` | `E-QUERY-039` with `infusion: "threet_score"`, `available_infusions: ["threat_score"]` (or all registered names as a Vec), `did_you_mean: "threat_score"` (Levenshtein distance 1); message renders as `available: [threat_score]` |
| EC-11-058 | `\| enrich completely_unknown_udf(col)` where no registered infusion name is within Levenshtein distance 3 | `E-QUERY-039` with `infusion: "completely_unknown_udf"`, `available_infusions: ["threat_score", "nvd_cvss"]` (global set as Vec), `did_you_mean` ABSENT (`None` — omitted from message); message renders as `available: [threat_score, nvd_cvss]` |
| EC-11-059 | No infusions registered in `InfusionRegistry` at all (`available_infusions: Vec::new()`) | `E-QUERY-039` with `available_infusions: []` (empty Vec renders as `[]` in Display), `did_you_mean` absent, `suggestion` using the "not available" form |
| EC-11-060 | Pipe-mode query with `\| enrich` AND a non-existent table | E-QUERY-037 fires first (table not found); E-QUERY-039 does not fire. Gate ordering: table check → column check → infusion check. |
| EC-11-061 | SQL mode query with no `ScalarFunc::Unknown` nodes AND pipe mode with no `EnrichStage` (no enrichment in query) | E-QUERY-039 does not fire; the gate is a no-op. |
| EC-11-062 | Hot reload adds a new infusion between parse and gate check | The gate uses the `InfusionRegistry` snapshot at plan time (consistent with `ArcSwap` hot-reload pattern per ADR-022). If the infusion was added during the in-flight query, the gate may reject it; the next query sees the updated registry. |
| EC-11-063 | SQL-mode `SELECT cvss(device_cves_first) FROM armis_devices` where `cvss` is not registered in `InfusionRegistry` (the AUDIT-005 reproducer) | `E-QUERY-039` with `infusion: "cvss"`, `available_infusions` listing all global infusion names, `did_you_mean` present if any registered name is within Levenshtein ≤ 3 of "cvss"; `E-INT-001` is NOT returned. Gate fires at plan time; DataFusion execution is never reached. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `query("FROM cyberint_alerts \| enrich threat_scor(ioc_value)", clients=["acme"])` where `threat_score` is registered but `threat_scor` is not | `E-QUERY-039` with `infusion: "threat_scor"`, `available_infusions` includes `"threat_score"`, `did_you_mean: "threat_score"` | happy-path (did_you_mean) |
| `query("FROM cyberint_alerts \| enrich completely_unknown_udf(alert_id)", clients=["acme"])` where no infusion name is close | `E-QUERY-039` with `infusion: "completely_unknown_udf"`, `available_infusions` lists all global infusion names, `did_you_mean` absent | no-suggestion |
| `query("FROM cyberint_alerts \| enrich threat_score(ioc_value)")` when no infusions are registered | `E-QUERY-039` with `available_infusions: []` (empty Vec; Display renders as `[]`), `suggestion` uses "not available" form | no-infusions |
| `query("SELECT * FROM cyberint_alerts")` (no enrich call) when `cyberint_alerts` is registered | Successful result rows — E-QUERY-039 does not fire | no-op |
| `query("FROM unknown_table \| enrich threat_score(col)")` when `unknown_table` is not registered | `E-QUERY-037` (not E-QUERY-039) — table gate fires first | gate-ordering |
| MCP error code for E-QUERY-039 | Surfaces as `-32602 INVALID_PARAMS` (not `-32000`) | mcp-mapping |
| `query("SELECT cvss(device_cves_first) FROM armis_devices", clients=["acme"])` where `cvss` is not registered in `InfusionRegistry` | `E-QUERY-039` with `infusion: "cvss"`, `available_infusions` lists global infusion names, `did_you_mean` present if applicable; `E-INT-001` NOT returned | audit-005-repro |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (VP-TBD) | E-QUERY-039 fires at plan time for any unregistered enrichment function name — no sensor API call is made | integration test |
| (VP-TBD) | E-QUERY-039 `available_infusions` contains no strings matching credential patterns | proptest |

## Follow-Up Story Anchor

Per-org `InfusionRegistry` scoping (filtering `available_infusions` to org-visible set via ADR-039 Option B pattern) is deferred. Until then, `available_infusions` enumerates the process-global set of all `InfusionField.name` values regardless of requesting org. This is correct for shared-infusion deployments (current deployment model) and carries a CWE-200 information-disclosure risk that is LOW today (all orgs share the same infusion set) and escalates to MEDIUM when per-org infusion configs are introduced (one org's infusion names become visible to another org's error responses).

**Required follow-up story:** "Per-org InfusionRegistry scoping — E-QUERY-039 `available_infusions` org-filter" — applies the ADR-039 `filter_to_org_visible` pattern to the `InfusionRegistry` lookup, filtering `available_infusions` to infusions visible to the requesting org's `OrgId`. Must also update `did_you_mean` to compute against the org-filtered candidate set.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC defines a new plan-time validation gate in the PQL query engine that rejects queries referencing unregistered enrichment functions. CAP-015 is the authoritative capability for the PQL query engine including its validation gates, error responses (`E-QUERY-NNN` codes), and plan-time checks. The E-QUERY-039 enrich-function-not-found gate is a plan-time check in the query engine layer, exactly what CAP-015 governs. |
| L2 Invariants | DI-002, DI-004, DI-019 |
| ADR | ADR-041 v1.2 — allocates E-QUERY-039 in the L4 pedagogical error suite; closes AUDIT-005 |
| Architecture Module | SS-11 (Query Execution) |
| Priority | P1 |

## Related BCs

- BC-2.11.001 — depends on: `query` MCP tool is the entry point; E-QUERY-039 is a new plan-time error condition for pipe-mode and SQL-mode queries invoking unregistered enrichment functions
- BC-2.11.016 — sibling: E-QUERY-038 column-not-found gate; same gate ordering, same `strsim::levenshtein` pattern
- BC-2.11.017 — sibling: E-QUERY pedagogical enrichments for existing codes; E-QUERY-039 follows the same pedagogical payload pattern
- BC-2.10.012 — composes with: `prism_describe` pql_hints should advertise available infusion functions; agents who call `prism_describe` first can avoid E-QUERY-039
- BC-2.10.014 — composes with: `prismql://reference` must include the `| enrich` syntax in its BNF; agents who read the reference can write correct enrich queries
- BC-2.19.001 — depends on: defines `InfusionRegistry`, `InfusionField.name` (authoritative enrichment-function-name namespace), and `udf_to_infusion` (the gate's plan-time lookup map)

## Architecture Anchors

- `architecture/decisions/ADR-041` §L4 — pedagogical self-correction loop; E-QUERY-039 extends the pattern established by E-QUERY-037/038
- `architecture/decisions/ADR-039` — org-scoped enumeration pattern; E-QUERY-039 currently uses global scope (see §Follow-Up Story Anchor for org-filter deferral)
- `prism_spec_engine::infusion::InfusionRegistry` — `udf_descriptors()` / `udf_to_infusion` is the plan-time lookup map for the E-QUERY-039 gate; `InfusionField.name` is the authoritative enrichment function-name namespace

## Story Anchor

S-DEMO-PRISMQL-ONBOARDING-001-C — implementing story (to be authored by story-writer in this burst).

## VP Anchors

VP assignments TBD — assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | S-DEMO-FIDELITY-REMEDIATION-001-HIGH-002-HIGH-004-canonical | 2026-06-27 | product-owner | **HIGH-002 + HIGH-004 closure from LOCAL adversary Pass 1 on S-DEMO-FIDELITY-REMEDIATION-001: three-way message drift and available_infusions type contradiction resolved.** **(HIGH-002) Message format canonicalized:** §payload-shape prose heading changed from "enrichment function '...' is not registered" to the verbatim canonical Display format `"E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}"` — "infusion" (not "function", not "UDF"), "is not registered" (not "not found"), brackets around `{available_infusions}` (parity with E-QUERY-038). The canonical message is the error-taxonomy.md row (POL-24 SOT), now also amended to add brackets (error-taxonomy v2.01). **(HIGH-004) `available_infusions` type corrected String→Vec<String>:** `EnrichUdfNotFoundDetails.available_infusions` changed from `String // comma-separated; "" when none` to `Vec<String>` matching E-QUERY-038 sibling (`ColumnNotFoundDetails.available_columns: Vec<String>`) and the pre-existing taxonomy emitter clause. The BC was wrong; the taxonomy emitter was already correct. Rendering rule added: Display comma-joins the Vec and wraps in brackets; empty Vec renders as `[]`. EC-11-059 updated: `available_infusions: ""` → `available_infusions: []` (empty Vec). no-infusions canonical test vector updated to `available_infusions: []`. `did_you_mean` description clarified: `Option<String>`, rendered as `" Did you mean: '{x}'?"` (leading space) when `Some`; omitted (not empty string) when `None` — consistent with E-QUERY-037/038. **(Gate ordering confirmation + HIGH-001 implementer note):** §Gate ordering prose amended: E-QUERY-039 fires LAST (after E-QUERY-037 and E-QUERY-038); explicit implementer note added that the code fires E-QUERY-039 FIRST (adversary HIGH-001) and MUST be corrected to match spec ordering — spec is the canonical ordering, code is wrong. The gate ordering itself (table→column→enrich) was always correct in the BC; no semantic change, only explicit statement added. |
| 1.2 | onboarding-001-C-sr-006-ec-renumber-2026-06-23 | 2026-06-23 | product-owner | SR-006 EC-11 namespace collision fix: renumbered all edge-case IDs in this BC from EC-11-046..053 range (which collided with committed BC-2.11.017 046..050 and BC-2.11.018 051..056) into the free range EC-11-057..063. Exact map: EC-11-046→057, 047→058, 048→059, 049→060, 050→061, 052→062, 053→063. Semantic content of every edge case unchanged — ID-only fix. Changelog references to old IDs in v1.1 entry are historical and left intact per append-only audit trail policy. |
| 1.1 | onboarding-001-C-sr-resolution-2026-06-23 | 2026-06-23 | product-owner | SR-001–SR-005 architect-adjudicated revisions: (SR-001) renamed `udf_name`→`infusion` and `available_udfs`→`available_infusions` throughout; bound to `EnrichStage.infusion` (pipe) / `ScalarFunc::Unknown(name)` (SQL) and `InfusionField.name` namespace in `InfusionRegistry`; updated `EnrichUdfNotFoundDetails` struct fields accordingly. (SR-002) replaced per-org `available_udfs` scoping with process-global `InfusionRegistry` enumeration; removed DI-008 from §Invariants; removed EC-11-051 (per-org isolation edge case) and the org-isolation Canonical Test Vector; added §Follow-Up Story Anchor for per-org scoping deferral with CWE-200 risk note. (SR-003) extended gate to cover BOTH AST paths — pipe-mode `EnrichStage.infusion` AND SQL-mode `FuncCall::Scalar { ScalarFunc::Unknown(name) }` (the actual AUDIT-005 reproducer); rewrote §Description, §Preconditions, §Gate firing condition, and §Closes AUDIT-005 to cover both paths; removed the conditional implementer-MUST paragraph; added EC-11-053 (SQL-mode AUDIT-005 reproducer) and `audit-005-repro` Canonical Test Vector. (SR-004) updated §Traceability ADR citation to `ADR-041 v1.2 — allocates E-QUERY-039 in the L4 pedagogical error suite; closes AUDIT-005`; removed "pending ADR-041 amendment" hedge. (SR-005) clarified `did_you_mean` as computed against the FULL global `InfusionField.name` set; added org-scoping note matching §Follow-Up Story Anchor. |
| 1.0 | onboarding-001-C-spec-burst-2026-06-23 | 2026-06-23 | product-owner | Initial draft — E-QUERY-039 enrich-UDF-not-found plan-time gate; closes AUDIT-005 from onboarding-discoverability-audit-2026-06-23.md |
