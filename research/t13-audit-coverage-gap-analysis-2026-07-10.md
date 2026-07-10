---
document_type: research
date: 2026-07-10
status: accepted
author: coverage-gap-analysis
related: [PR-219, t13-preflight-audit, T13-capstone]
decision_anchor: D-1649
---

# T13 Preflight Audit Coverage-Gap Analysis — `scripts/t13-preflight-audit.py` vs develop @ 8ea29823

## Headline answer

**NO — the audit does not exercise the full merged feature set.** It comprehensively covers the PR #214/#216/#217 surfaces it was written for (at develop@f935edb6), but it was committed *as part of* PR #219 without live checks for PR #219's own behaviors, and it has pre-existing structural gaps (INE, joins, SqlPipe, `| stats`, 2 of 5 prompts, most resources, negative temporal paths). The pass-13 adversary observation is confirmed: PR #219 behaviors are verified only by Rust tests (`crates/prism-query/src/engine.rs` mod `drift_ieq_nonexistent_col_errpath_001_tests`, ~60 tests; `crates/prism-mcp/tests/normalized_pql.rs`) and code-trace probes (`.factory/cycles/wave-5-e-demo-fidelity/FIX-IEQ-ERRPATH-001/adversarial-review/pr-level-pass-13.md`, pr-level-pass-6.md, local-pass-19.md), never by the live audit.

Audit script scope analyzed: 70 checks — A1–A22 (MCP protocol), B1–B10 + 5 dynamic (sensor tables), C1–C8 (query modes), D1–D5 (scenario), E1–E6 (enrichment), F1–F6 (error taxonomy), G1–G8 (PRs #214/#216/#217 regressions).

## 1. Coverage classification table

| # | Feature (source) | Status | Evidence / what's missing |
|---|---|---|---|
| 1 | MCP boot + protocol handshake | **COVERED** — A1 | |
| 2 | tools/list surface | **PARTIAL** — A2 | EXPECTED_TOOLS asserts only 6 of the **14 implemented** tools (server.rs has 54 `#[tool]` sites, 14 real, 40 runtime -32003 stubs). `explain_query`, `reload_config` (BC-2.16.007), alias suite (`create/list/delete/explain_alias`), `confirm_action`, `add_sensor_spec`, `list_sensor_specs`, `validate_config` never asserted present nor exercised |
| 3 | prompts | **PARTIAL** — A4/A16/A17/A18 | `prompts.rs` registers **5** prompts; audit asserts/exercises only 3. `client_overview` and `cross_client_status` never listed in EXPECTED_PROMPTS nor prompt_get-exercised (the exact hang/arg-validation class A17/A18 guard against) |
| 4 | resources | **PARTIAL** — A3/A12 | Only `prismql://reference`. Never read: `prism://config/clients`, `prism://sensors/health`, templates `prismql://schema/{client_id}`, `prism://schema/{sensor}/{table}`, `prism://config/clients/{client_id}/sensors`; subscribe/unsubscribe path untouched |
| 5 | `list_capabilities` tri-state (D-1162/D-1312, BC-2.10.011) | **COVERED** — A5/A6 | |
| 6 | `prism_describe` (BC-2.10.012, ONBOARDING-001-A) | **COVERED** — A7–A11 | |
| 7 | `check_sensor_health` (S-5.04) | **COVERED** — A22 | (`get_diagnostics` stub not probed — trivial) |
| 8 | 6 sensors × tables × 3 orgs | **COVERED** — B1–B10 + 5 dynamic | Correctly avoids asserting `crowdstrike_incidents`/`cyberint_incidents` rows (no DTU routes — verified) |
| 9 | Multi-client isolation | **COVERED** — B10, A11 | |
| 10 | Multi-client fan-out in ONE query (`clients: [a,c]`) | **NOT COVERED** | Every audit call passes a single client; `cross_client_status` prompt implies multi-client use live. `sensor_errors` partial-failure field (BC-2.11.005/011) never asserted |
| 11 | SQL mode | **COVERED** — C1, C4–C6, F5 | |
| 12 | Pipe mode (where/fields/sort/limit) | **COVERED** — C2/C3/C7 | |
| 13 | **SqlPipe mode** (BC-2.11.020) | **NOT COVERED** | Docstring claims it (item 3); zero checks issue a `SELECT … FROM t \| stage` query |
| 14 | `\| stats` (position 11) | **NOT COVERED** | No live stats query anywhere (runbook gap too) |
| 15 | `\| dedup` / `\| head` / `\| tail` (GRAMMAR-REMEDIATION-001) | **NOT COVERED** | |
| 16 | JOIN queries (SQL position 5) | **NOT COVERED** | No JOIN anywhere in script or runbook, despite guaranteed `device_id` 0..49 overlap between `crowdstrike_devices` and `armis_devices` per org |
| 17 | Scenario stage 4 data + IOC stamping | **COVERED** — D1–D5 | |
| 18 | Determinism ("determinism verified", docstring item 4) | **NOT COVERED** | No repeated-run row comparison exists |
| 19 | Enrichment `threat_score`/`threat_is_known_malicious`/`cvss_base_score`/`cvss_severity` on scalar companions | **COVERED** — E1–E6, G8 | |
| 20 | Enrichment `threat_sources`, `cvss_vector` UDFs | **NOT COVERED** | Runbook uses them; ThreatIntel/NVD DTUs guarantee values (`["virustotal"]`, `CVSS:3.1/…`) |
| 21 | Runbook list-column enrich forms (`enrich threat_score(iocs_value)`, Steps 3.2/3.4/6.2) | **NOT COVERED** | Post-ADR-051 scalar-input rule these likely return score 0 / mismatch live — runbook v1.7 drift risk |
| 22 | E-QUERY-032/-037/-039 | **COVERED** — F1–F4, F6, A13–A15 | |
| 23 | **E-QUERY-038 SQL mode** | **COVERED** — F5 | Code-only assertion; no payload check |
| 24 | **E-QUERY-038 pipe mode (PR #219, the original DRIFT shape)** | **NOT COVERED** | F5 is SQL-only. `FROM t \| where nonexistent IEQ 'x'` never issued. If regressed → live "Internal error" |
| 25 | **E-QUERY-038 filter mode (position 7, PR #219)** | **NOT COVERED** | Filter syntax `table \| predicate` never issued |
| 26 | **E-QUERY-038 SqlPipe stages (positions 9–14, PR #219)** | **NOT COVERED** | |
| 27 | **did_you_mean + available_columns payload (BC-2.11.016)** | **NOT COVERED** | F5 passes on code alone. The `query_tutorial` prompt *teaches agents to self-correct with these fields* — live absence breaks the demo's self-correction beat. Both survive in `content[0].text` ("Did you mean: '…'?" / "available: […]") so the existing parser can assert them |
| 28 | **HEAD-JOIN fail-open + 6 suspension rules (PR #219)** | **NOT COVERED** | Only Rust tests (EC-11-070–076) + adversary code-trace. Observable contract: joined query + unknown bare column → **E-QUERY-034/"Internal error"**, never E-QUERY-038 (spec-sanctioned FP-001) |
| 29 | **CWE-116/117 sanitized column echo (PR #219)** | **NOT COVERED** | Only Rust tests (`test_sec_find_001_cwe117…`, med002 emission trio, obs001 payload pair) |
| 30 | IEQ happy path + OCSF Title-case | **COVERED** — G1, G3, G6 | |
| 31 | IIN | **COVERED** — G2, G3 | |
| 32 | **INE (PR #217, part of the 27 ACs)** | **NOT COVERED** | Zero live INE query in script *or* runbook — headline operator with no end-to-end evidence |
| 33 | Unicode case-folding hardening (PR #217 CWE-117 scope) | **NOT COVERED** | Rust tests only |
| 34 | SQL-mode IEQ rejection → E-QUERY-001 | **COVERED** — G4 | |
| 35 | E-QUERY-002 typed guidance | **PARTIAL** — G5 (permanent WARN) | Via `severity_id` which doesn't exist in cyberint_alerts. **Now deterministically exercisable** post-#219 via `armis_devices.risk_score` (integer column, plan-time pipe-mode gate) |
| 36 | Temporal RFC-3339 positive path (ADR-052 §D4, PR #214) | **COVERED** — G7, C8 | |
| 37 | **E-QUERY-041 negative path** (date-only literal → pedagogical error) | **NOT COVERED** | G7's comment documents it but never asserts it |
| 38 | **E-QUERY-042** (temporal literal in invalid position) | **NOT COVERED** | |
| 39 | Typed enrichment Int64 (ADR-051, PR #216) | **COVERED** — E1/E6/G8 | |
| 40 | Guardrails: E-QUERY-003 (64KB/depth), E-QUERY-033 (limit>1000), E-QUERY-040 (dual limit), E-QUERY-010/015 (write/reserved-source rejection) | **NOT COVERED** | All demo-reachable per error-taxonomy.md v2.36; none probed |
| 41 | `normalized_pql` success-path field (BC-2.11.018) | **NOT COVERED** | Never asserted present |

## 2. Ranked gap list (by live-demo embarrassment potential)

1. **[CRITICAL] Pipe/filter-mode E-QUERY-038 regression (rows 24–25).** This is the *exact* shape that produced "Internal error" in the Jul-08 audit (PARTIAL-1/G5 → FIX-IEQ-ERRPATH-001). The fix merged 24h ago with zero live verification; an analyst typo in the flagship pipe mode is the single most likely live keystroke.
2. **[CRITICAL] did_you_mean / available_columns payload (row 27).** The demo's "agent self-corrects from pedagogical errors" beat depends on these; a serialization regression is invisible to every current check.
3. **[HIGH] INE operator (row 32).** Shipped headline feature of PR #217; a presenter typing `severity INE 'low'` live has no preflight safety net.
4. **[HIGH] Runbook list-column enrich drift (row 21).** T13 runbook v1.7 Steps 3.2/3.4/6.2 use `iocs_value`/`behaviors_ioc_value` (JSON-list columns); ADR-051 D4 made typed UDFs scalar-input. Presenter running the runbook verbatim likely gets threat_score=0 or a mismatch. Needs an audit probe AND a runbook amendment.
5. **[HIGH] E-QUERY-041 negative temporal (row 37).** A bare `'2026-07-09'` date is the most natural analyst input; the pedagogical rejection is the selling point and is unverified live.
6. **[HIGH] Unexercised prompts `client_overview` / `cross_client_status` (row 3).** Same hang/arg-validation risk class the script already guards for the other three prompts.
7. **[MED] E-QUERY-002 now exercisable via `armis_devices.risk_score` (row 35)** — converts G5's permanent WARN into a deterministic PASS/FAIL.
8. **[MED] JOIN positive path + HEAD-JOIN fail-open (rows 16, 28).** Fail-open means a joined typo yields a *deliberately* opaque error — the demo team must know this cliff exists; and nobody has ever proven a join returns rows live.
9. **[MED] SqlPipe mode + E-QUERY-040 (rows 13, 40)** — docstring claims coverage it doesn't have.
10. **[MED] `| stats` (row 14)** — the natural "SOC metrics" demo query shape, zero live evidence.
11. **[MED] Multi-client fan-out + sensor_errors (row 10).**
12. **[MED] Guardrail probes E-QUERY-003/-033 + CWE-117 echo (rows 29, 40)** — prompt-injection/DoS defense showcase, currently asserted nowhere.
13. **[LOW] tools/list & resources completeness, explain_query, determinism, normalized_pql, dedup/head/tail (rows 2, 4, 15, 18, 41).**

## 3. Proposed new checks (Section H + amendments)

All queries verified against the DTU data contract (org-a = healthy/seed 100 CS+Armis; org-b = healthy/seed 150 Claroty+Cyberint; org-c = compromised/seed 200, all 4 sensors, Stage 4 guaranteed since `scenario_start_secs` = 1782214754 is in the past — Stage 4 is terminal/absorbing).

**Parser extension note (`parse_envelope`):** the current parser reads only `content[0].text`. H2's payload anchors ("Did you mean:", "available: [") survive there verbatim (they are part of the `ColumnNotFoundDetails` Display message), but asserting the machine-readable `structuredContent.error.did_you_mean` / `available_columns` fields requires extending `parse_envelope` to also return `resp["result"]["structuredContent"]["error"]` — recommended (one-time change, benefits H2/H16; note the field path is `structuredContent.error.*`, field name is `code` not `error_code`, and there is no `details.*` key in this codebase's envelope).

**Amendments to existing checks:** A2 EXPECTED_TOOLS grows 6 → 14 (see H15); A4 EXPECTED_PROMPTS grows 3 → 5 (see H13); G5 is retired-or-demoted once H6 lands (deterministic E-QUERY-002 path via `armis_devices.risk_score` replaces the structurally-unexercisable `severity_id` probe).

| ID | Query / call (client) | Assertion | Data-contract grounding |
|---|---|---|---|
| **H1** | `FROM crowdstrike_detections\n\| where nonexistent_column_xyz IEQ 'high'\n\| limit 5` (org-c) | `error_code == "E-QUERY-038"`; FAIL hard if message contains "Internal error" or E-QUERY-034, or rows returned | Table registered for org-c; column absent from crowdstrike.sensor.toml. Original DRIFT shape |
| **H1b** | Filter mode: `crowdstrike_detections \| nonexistent_column_xyz IEQ 'high'` (org-c) — note NO `where` keyword (syntax per `test_DRIFT_IEQ_filter_mode_nonexistent_col_yields_e_query_038`, engine.rs:9022) | Same as H1 (position 7) | Same |
| **H2** | `SELECT sevrity FROM crowdstrike_detections LIMIT 5` (org-c) | E-QUERY-038 AND text contains `"Did you mean: 'severity'"` AND `"available: ["` | `severity` is a declared column; Levenshtein("sevrity","severity")=1 ≤ 3; suggestion is max-1, lexicographic tie-break, deterministic |
| **H3** | `FROM crowdstrike_detections\n\| where severity INE 'medium'\n\| limit 20` (org-c) | rows ≥ 1; every returned severity == `"Critical"`; zero `"Medium"` rows | Seed-200 detections are exactly 5 Critical (idx 0–4) + 15 Medium; INE case-folds both sides |
| **H4** | `FROM claroty_audit_logs\n\| where timestamp > '2020-01-01'\n\| limit 3` (org-c) | `error_code == "E-QUERY-041"`; message contains `"RFC-3339"` | `claroty_audit_logs.timestamp` is datetime-typed (claroty.sensor.toml); static 5-row fixture always present |
| **H5** | `SELECT severity, COUNT(*) FROM crowdstrike_detections GROUP BY '2026-07-01T00:00:00Z'` (org-c) | `error_code == "E-QUERY-042"` (temporal literal in GROUP BY arm) | ADR-052 §D4 7-arm dispatch; implementer should confirm shape against the E-QUERY-042 unit tests in prism-query before pinning |
| **H6** | `FROM armis_devices\n\| where risk_score IEQ 'high'\n\| limit 5` (org-c) | `error_code == "E-QUERY-002"`; message contains `"does not support operator"`. Do NOT assert a sibling suggestion (risk_score has no OCSF sibling) | `armis_devices.risk_score` is integer-typed in armis.sensor.toml — this **retires G5's permanent WARN** |
| **H7** | `SELECT d.device_id, a.risk_score FROM crowdstrike_devices d JOIN armis_devices a ON d.device_id = a.device_id LIMIT 5` (org-c) | no error; rows ≥ 1; risk_score numeric | `dev-0196f4b2-200-{0..49}` identical in both tables (full overlap, same slug+seed) |
| **H8** | `SELECT totally_unknown_col FROM crowdstrike_devices d JOIN armis_devices a ON d.device_id = a.device_id LIMIT 5` (org-c) | Must NOT return rows; must NOT be E-QUERY-038 (HEAD-JOIN bare-ref fail-open is spec-sanctioned FP-001); PASS if E-QUERY-034 / "Internal error" / -32000 controlled rejection; FAIL on rows, hang, or process exit | BC-2.11.016 v1.25 suspension rule 6 (per-reference scoping, EC-11-074/075/076); locks the accepted trade-off so a future change is noticed |
| **H9** | SqlPipe: `SELECT device_id, severity FROM crowdstrike_detections \| where severity IEQ 'critical' \| limit 5` (org-c) | rows ≥ 1; all severity == "Critical" | SqlPipe head + position-9 stage; same seed-200 guarantee as H3 |
| **H10** | `SELECT device_id FROM crowdstrike_detections LIMIT 5 \| limit 3` (org-c) | `error_code == "E-QUERY-040"` (FORBID-BOTH, ADR-043 D4) | Pure grammar; no data dependency |
| **H11** | `FROM crowdstrike_detections \| stats count() as cnt by severity` (org-c) — implementer must lift exact stats grammar from `prismql://reference` / prism-query parser tests before pinning | 2 buckets; {Critical: 5, Medium: 15} | Seed-200 exact counts guaranteed |
| **H12** | `query` with `clients: ["org-a","org-c"]`, `FROM crowdstrike_detections \| limit 40` | no error; rows contain device/alert IDs with BOTH `-100-` and `-200-` segments; `results.sensor_errors == []` | org-a: 5 detections, org-c: 20; all IDs embed the seed segment (all orgs share slug `0196f4b2` — never assert distinct slugs) |
| **H13** | `prompt_get("client_overview", {client_id:"org-c"})` and `prompt_get("cross_client_status", {})` with 5s timeout | each returns < 3s with ≥ 1 message; also add both names to A4's EXPECTED_PROMPTS (now 5) | prompts.rs registers 5 static prompts |
| **H14** | `resources/read` on `prism://config/clients`, `prism://sensors/health`, `prismql://schema/org-c` | clients doc lists org-a/org-b/org-c; health JSON parses; schema doc contains `cyberint_alerts` | All three URIs statically registered / template-backed (resources.rs, resources/schema.rs) |
| **H15** | Extend A2 EXPECTED_TOOLS to the 14 implemented tools (add: explain_query, reload_config, create_alias, list_aliases, delete_alias, explain_alias, confirm_action, add_sensor_spec, list_sensor_specs, validate_config); plus live `explain_query` call on `FROM crowdstrike_detections \| limit 5` (org-c) | all present in tools/list; explain_query returns non-error plan | server.rs `#[tool_router]`; no compile-time gating exists — all 54 tools always listed, 14 implemented |
| **H16** | `SELECT "badcolumn" FROM crowdstrike_detections LIMIT 3` (org-c; embed literal U+0001 via json.dumps) | FAIL if any raw control char (U+0000–U+001F, U+007F) appears in `content[0].text`; PASS if E-QUERY-038 echoing `'badcolumn'` (stripped, not escaped — `sanitize_for_log`, prism-core error.rs:2013) OR E-QUERY-001 parse rejection | CWE-116/117 chokepoint `ColumnNotFoundDetails::new`; either rejection layer is safe, control-char leakage is the only FAIL |
| **H17** | `query` args `{query:"FROM crowdstrike_detections \| limit 5", clients:["org-c"], limit:1001}` | E-QUERY-033 (or -32602 param rejection); FAIL if executes | BC-2.11.001 ceiling 1000 |
| **H18** | Pipe query with a ~70KB `IN (…)` literal list (org-c) | E-QUERY-003 OR protocol -32602 oversize — either controlled rejection PASSes; FAIL on success/hang/crash | BC-2.11.006 64KB/depth-64/32-stage limits; MCP input-length guard may legitimately fire first |
| **H19** | `FROM cyberint_alerts \| where iocs_value_first IS NOT NULL \| enrich threat_sources(iocs_value_first) \| limit 3` and `FROM armis_devices \| where device_cves_first IS NOT NULL \| enrich cvss_vector(device_cves_first) \| limit 3` (org-c) | threat_sources non-null containing `virustotal`; cvss_vector startswith `CVSS:3.1/` | Hash IOC → sources `["virustotal"]` (ThreatIntel DTU state.rs); CVE-9999-* → vector `CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N` (NVD DTU fixtures + scenario entries) |
| **H20** | Runbook-drift probe: run T13 runbook Step 3.2 verbatim (`\| enrich threat_score(iocs_value)` — JSON-list column) (org-c) | PASS only if threat_score ≥ 75; expected current outcome is FAIL/0-score → drives a T13-capstone-demo-runbook.md v1.8 amendment to `iocs_value_first` forms **before** the live demo | ADR-051 D4 scalar-input rule vs runbook v1.7 Steps 3.2/3.4/6.2 |
| **H21** | Determinism: run `FROM crowdstrike_detections \| sort detection_id \| limit 20` twice (org-c) | byte-identical row sets | Seeded ChaCha20 generators + fixed anchors (org-a/b anchor 2026-01-01T00:00:00Z; org-c anchor 1782214754); makes docstring item 4 true |
| **H22** | On any successful pipe query (reuse C2 body) | `results` contains key `normalized_pql` (BC-2.11.018; key absent only on unparseable path) | Envelope contract, safety_envelope.rs |

## 4. Never-assert guardrails (ADV-PR-P11-HIGH-001 class, verified against DTU generators)

1. No rows in `crowdstrike_incidents` / `cyberint_incidents` — no DTU routes exist (crowdstrike routes dir has only detections/hosts/oauth/writes, DTU-EXT-001; cyberint has only alerts/threats/dtu, EC-016-013-002).
2. No specific cyberint severity/status distributions beyond set-membership (`severity ∈ {low,medium,high,critical}`, `status ∈ {open,acknowledged,closed}`) + org-c alert rows 0–2 guaranteed `severity = "high"`. Cyberint status values are vendor-native lowercase with no OCSF caption match — pass-through is correct (BC-2.02.013 RG-021).
3. No hardcoded scenario-catalog IOC/CVE literal values — assert format (64-hex hash, `10.x.x.x`, `malicious-<u32>-<i>.example.com`, `CVE-9999-NNNNN`) + cross-table equality (e.g. `cyberint_alerts.iocs_value_first == crowdstrike detection-0 behaviors_ioc_value_first`) instead.
4. No distinct org slug prefixes — all three org UUIDs share first 4 bytes, so slug = `0196f4b2` for ALL orgs; isolation lives solely in the `-100-` / `-150-` / `-200-` seed segment of generated IDs.
5. No claroty alert `severity` column (removed, Gap-CL-005) and no claroty `device_id` column (coherence key exists in records but is not declared in claroty.sensor.toml).
6. No per-org `claroty_audit_logs` differences — static 5-row fixture (`crates/prism-dtu-claroty/fixtures/audit-log.json`) shared by all orgs, independent of seed/scenario.
7. No `ioc_type` / `ioc_value_singleton` / `alert_data_url` values in cyberint (never emitted → always NULL); no `behaviors_ioc_*` values outside org-c detection 0 (the only IOC-stamped detection).
8. No armis alert statuses other than `"UNHANDLED"`, claroty alert statuses other than `"Unresolved"`, crowdstrike detection statuses other than `"new"`.

## 5. Non-script findings

- **(a) T13 runbook v1.7 list-column enrich drift:** `.factory/objectives/T13-capstone-demo-runbook.md` Steps 3.2/3.4/6.2 use JSON-list columns (`iocs_value`, `behaviors_ioc_value`) as typed-UDF inputs; ADR-051 D4 made typed enrichment UDFs scalar-input (`*_first` companions). The runbook needs a v1.8 amendment to the scalar forms before the live demo, regardless of the H20 probe outcome.
- **(b) Pipe-FROM + bare SQL LIMIT inconsistency:** `docs/DEMO-RUNBOOK.md` §5 states the form `FROM crowdstrike_detections LIMIT 5` is rejected, while the T13 checklist (§5.3/§5.4) presents it as a working form. Adjudicate and align the two documents before someone types it live.

## Appendix — key sources

- Audit script: `/Users/jmagady/Dev/prism/scripts/t13-preflight-audit.py` (1,813 lines, 70-item COVERAGE_MATRIX, authored at develop@f935edb6, committed within PR #219)
- PR #219 gate: `crates/prism-query/src/engine.rs` (`check_query_column_availability` @2489, `check_pipe_stage_columns` @3344, HEAD-JOIN per-reference scoping @2777–2838); `crates/prism-core/src/error.rs` (`ColumnNotFoundDetails`, `sanitize_for_log` @2013); `crates/prism-mcp/src/error_mapping.rs` (`StructuredErrorFields` @590, `build_structured_error_response` @824)
- Spec: `.factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md` v1.25 (14 positions, 6 suspension rules)
- MCP surface: `crates/prism-mcp/src/server.rs` (54 `#[tool]` sites, 14 implemented / 40 runtime -32003 stubs, no compile-time gating), `crates/prism-mcp/src/prompts.rs` (5 prompts), `crates/prism-mcp/src/resources.rs` + `resources/schema.rs` (3 static + 3 template resources)
- Data contract: `scripts/demo.toml`, `crates/prism-dtu-{crowdstrike,armis,claroty,cyberint,threatintel,nvd,demo-server,common}/src/`, `crates/prism-sensors/specs/*.sensor.toml`
- Taxonomy: `.factory/specs/prd-supplements/error-taxonomy.md` v2.36 (32 E-QUERY codes)
- Prior audits: `.factory/research/demo-comprehensive-preflight-audit-2026-07-08.md` (68/70 PASS; PARTIAL-1 → FIX-IEQ-ERRPATH-001), `-2026-07-03.md` (OBS-1 lineage → ADR-051 → PR #216)
- Adversary probes: `.factory/cycles/wave-5-e-demo-fidelity/FIX-IEQ-ERRPATH-001/adversarial-review/pr-level-pass-13.md`, `pr-level-pass-6.md`, `local-pass-19.md`
