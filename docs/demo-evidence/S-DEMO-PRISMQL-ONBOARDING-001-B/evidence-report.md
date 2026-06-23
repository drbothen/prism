---
story_id: S-DEMO-PRISMQL-ONBOARDING-001-B
title: "PrismQL LLM Auto-Onboarding — Query Engine L4 (E-QUERY-038 Gate + Pedagogical Enrichments + normalized_pql)"
captured_at: "2026-06-22"
revised_at: "2026-06-22"
revision_reason: "OBS-198-1: AC-001 case_a re-captured with clients=[acme] org-scope; OBS-198-2: AC-005 case_a label corrected to match zero-row capture reality; F-198-FRESH-MED-001: AC-003 and AC-006 parse-error cases re-captured after ec_code_override fix — code now E-QUERY-001 (was E-MCP-002); F-198-PRLEVEL-MED-001: red-gate-test-run.txt re-captured after addition of test_BC_2_11_017_ac003_parse_error_structured_code_is_e_query_001 — count is now 49 (was 48); OBS-198-PRLEVEL-1: ast.rs walker comment narrowed to accurately scope string-literal coverage (bare identifiers excluded with rationale)"
product_type: "CLI/MCP (Rust — no browser UI)"
recording_tool: "cargo nextest (Red Gate test output) + cargo run --example (engine-driven JSON captures)"
coverage: "6/6 ACs covered"
---

# Evidence Report — S-DEMO-PRISMQL-ONBOARDING-001-B

## Summary

All 6 acceptance criteria have real engine-driven evidence. Evidence is captured by:
1. Running the Red Gate tests via `cargo nextest` (49 tests pass, all BC_2_11_016 / BC_2_11_017 / BC_2_11_018 tests green)
2. Running the production code paths via a temporary `cargo run --example evidence_capture` that calls the real `PrismServer::query` and `QueryEngine::execute` APIs and captures the actual `serde_json::Value` structured content returned to an LLM agent

No evidence is fabricated. Every JSON payload in the AC evidence files was printed by running production Rust code in this worktree.

## Evidence Artifacts

| Artifact | AC(s) | Description |
|----------|-------|-------------|
| `AC-001-e-query-038-did-you-mean.json` | AC-001 | E-QUERY-038 payload: 3 cases — did_you_mean present, absent, gate ordering |
| `AC-002-di008-org-scoped-columns.json` | AC-002 | DI-008 org-scoped available_columns: acme's columns only, no globex columns |
| `AC-003-pedagogical-enrichments.json` | AC-003 | E-QUERY-001 near_text+reference_pointer; E-QUERY-002 valid_operators_for_type; E-QUERY-003 how_to_fix |
| `AC-004-e-query-037-prism-describe-suggestion.json` | AC-004 | E-QUERY-037 suggestion contains prism_describe (with/without did_you_mean) |
| `AC-005-normalized-pql-present-on-success.json` | AC-005 | normalized_pql present on success + zero-row + canonical form |
| `AC-006-normalized-pql-absent-on-error.json` | AC-006 | normalized_pql absent from all error responses |
| `red-gate-test-run.txt` | All | nextest output: 49/49 pass for BC_2_11_016 / BC_2_11_017 / BC_2_11_018 suites |

## AC Coverage Detail

### AC-001 — E-QUERY-038 gate payload shape

**Evidence file:** `AC-001-e-query-038-did-you-mean.json`

**Red Gate test:** `test_BC_2_11_016_e_query_038_did_you_mean`

Three cases captured from real `PrismServer::query` execution:

**Case a** — Query `SELECT * FROM crowdstrike_alerts WHERE sevrity = 'high'` (Levenshtein-1 typo in WHERE clause), `clients=["acme"]` org-scoped:
- Response contains `"code": "E-QUERY-038"`, MCP error code -32602 (INVALID_PARAMS)
- `"available_columns": ["detection_id", "host_name", "severity"]` — non-empty
- `"did_you_mean": "severity"` — present (Levenshtein distance 1)
- `message` contains `"client 'acme'"` — client_id correctly reflects the acme org scope
- Re-captured: OBS-198-1 fix — original had `client_id: ""` (QueryOptions::default(), no clients scope); now shows `client_id: "acme"` per AC spec

**Case b** — Query `SELECT completely_bogus_col FROM crowdstrike_alerts LIMIT 5` (no match within distance 3):
- `"available_columns": ["host_name", "severity"]` — non-empty
- `did_you_mean` field ABSENT from the JSON (not null, not present — key does not appear)

**Case c** — Query `SELECT * FROM nonexistent_table WHERE bogus_col = 1` (table does not exist):
- Response contains `"code": "E-QUERY-037"` — gate ordering enforced, E-QUERY-038 does NOT fire when table is absent

---

### AC-002 — E-QUERY-038 org-scoped available_columns (DI-008)

**Evidence file:** `AC-002-di008-org-scoped-columns.json`

**Red Gate test:** `test_BC_2_11_016_e_query_038_org_scoped_available_columns`

Multi-tenant fixture: acme has `[severity, acme_only_field]`; globex has `[severity, globex_alert_type]`. Query scoped to `clients=["acme"]`:
- `"available_columns": ["acme_only_field", "severity"]` — acme's columns only
- `"globex_alert_type"` does NOT appear in available_columns — no cross-org leak
- `"message"` contains `"client 'acme'"` — client_id correctly scoped

---

### AC-003 — E-QUERY-001/002/003 pedagogical enrichments

**Evidence file:** `AC-003-pedagogical-enrichments.json`

**Red Gate tests:**
- `test_BC_2_11_017_enrichment_helpers_valid_operators_for_type` (prism-query)
- `test_BC_2_11_017_enrichment_helper_extract_near_text` (prism-query)
- `test_BC_2_11_017_enrichment_helper_how_to_fix_for_security_limit` (prism-query)
- `test_BC_2_11_017_ac003_parse_error_response_carries_near_text` (prism-mcp)
- `test_BC_2_11_017_ac003_type_error_response_carries_valid_operators` (prism-mcp)
- `test_BC_2_11_017_ac003_security_limit_error_carries_how_to_fix` (prism-mcp)

**Case a — E-QUERY-001 parse error** (query `SELCT * FROM crowdstrike_alerts`):
- `"code": "E-QUERY-001"` — correct canonical code (F-198-FRESH-MED-001 fix: was `"E-MCP-002"` before `ec_code_override` was added to the `QueryParseFailed` arm)
- `"near_text": "SELCT"` — the offending token (≤50 chars), present in `sc["error"]`
- `"reference_pointer": "prismql://reference"` — literal string, present in `sc["error"]`

**Case b — E-QUERY-003 security limit** (query > 64KB):
- `"how_to_fix": "Shorten the query. Remove large IN (...) lists or break into multiple queries."` — non-empty actionable string
- `"code": "E-QUERY-003"`

**Case c — E-QUERY-002 type mismatch** (String column `severity` with ordering operator `>`):
- `"valid_operators_for_type": ["=", "!=", "LIKE", "IN", "NOT IN"]` — the STRING-SPECIFIC set
- Ordering operators `>`, `<`, `BETWEEN` absent from the array (not the generic superset)
- `"code": "E-QUERY-002"`

---

### AC-004 — E-QUERY-037 suggestion field with prism_describe reference

**Evidence file:** `AC-004-e-query-037-prism-describe-suggestion.json`

**Red Gate test:** `test_BC_2_11_017_e_query_037_suggestion_prism_describe`

**Case a** — Query `SELECT severity FROM crowdstrike_alert LIMIT 5` (Lev-1 typo for `crowdstrike_alerts`) with `clients=["acme"]`:
- `"suggestion"` contains `"prism_describe('acme')"` — uses client_id, NOT sensor name
- `"suggestion"` contains retry hint: `"If you meant 'crowdstrike_alerts', retry with that table name."`
- `"suggestion"` does NOT contain `prism_describe('crowdstrike')` (sensor name would break LLM self-correction)

**Case b** — Query `SELECT severity FROM completely_made_up_table LIMIT 5` (no close match):
- `"suggestion"` contains `"prism_describe"` — pointer present even without a specific retry hint
- `"suggestion"` does NOT contain `"If you meant"` (no retry hint when no close match)

---

### AC-005 — normalized_pql present on success (including zero-row + partial-failure)

**Evidence file:** `AC-005-normalized-pql-present-on-success.json`

**Red Gate tests:**
- `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error`
- `test_BC_2_11_018_normalized_pql_key_present_in_mcp_success_response`
- `test_BC_2_11_018_normalized_pql_present_on_zero_row_mcp_response`
- `test_BC_2_11_018_ec11054_normalized_pql_present_on_partial_failure`

**Case a** — Query `SELECT * FROM crowdstrike_alerts WHERE severity = 'high' LIMIT 10` (zero-row success — no sensor adapters wired in capture fixture):
- `sc["results"]["normalized_pql"]` = `"SELECT * FROM crowdstrike_alerts WHERE severity = 'high' LIMIT 10"`
- Non-empty; contains table name; uppercase keywords; no DataFusion plan nodes
- `total_results: 0` — zero rows by design (no adapters wired; fan-out empty); AC-005 postcondition covers zero-row success explicitly
- Label corrected: OBS-198-2 fix — was "success with results"; relabeled to "success — zero-row result; normalized_pql still present" to match captured reality

**Case b** — Zero-row query:
- `sc["results"]["normalized_pql"]` = `"SELECT severity FROM crowdstrike_alerts WHERE severity = 'nonexistent'"` — present even when 0 rows returned

**Case c** — Lowercase input `select * from crowdstrike_alerts limit 5`:
- `sc["results"]["normalized_pql"]` = `"SELECT * FROM crowdstrike_alerts LIMIT 5"` — keywords canonicalized to uppercase by the Chumsky AST re-serializer

---

### AC-006 — normalized_pql absent on all error responses

**Evidence file:** `AC-006-normalized-pql-absent-on-error.json`

**Red Gate test:** `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error` (combined with AC-005)

Parse error response for query `!!invalid query!!`:
- `"code": "E-QUERY-001"` — correct code after F-198-FRESH-MED-001 fix (was `"E-MCP-002"`)
- `normalized_pql_present_in_response: false` (verified programmatically by checking both `sc["normalized_pql"]` and `sc["results"]["normalized_pql"]`)
- The error payload contains only `"error": {...}` — no `normalized_pql` key at any level
- Error path returns early via `prism_error_to_structured_call_result` BEFORE the `normalized_pql_str` computation in `PrismServer::query`, making absence structurally guaranteed

Cross-references: AC-001-a and AC-004-a payloads also show no `normalized_pql` key for E-QUERY-038 and E-QUERY-037 errors respectively.

## Test Suite Summary

```
49 tests run: 49 passed, 0 failed
Suites covered:
  prism-query::e_query_pedagogical (BC_2_11_016 + BC_2_11_017)
  prism-query ast::bc_2_11_018_normalizer_roundtrip_tests (BC_2_11_018)
  prism-query engine::bc_2_11_016_did_you_mean_determinism_tests (BC_2_11_016)
  prism-mcp::normalized_pql (BC_2_11_016 + BC_2_11_017 + BC_2_11_018)
  prism-mcp error_mapping::tests (BC_2_11_017)
```

Full nextest output: `red-gate-test-run.txt`

## POL-10 Compliance

- All artifacts are under `docs/demo-evidence/S-DEMO-PRISMQL-ONBOARDING-001-B/` (story-scoped subfolder)
- No flat `docs/demo-evidence/*.md` files created
- `evidence-report.md` present with `story_id: S-DEMO-PRISMQL-ONBOARDING-001-B` in frontmatter
- Evidence is from real engine execution, not fabricated
