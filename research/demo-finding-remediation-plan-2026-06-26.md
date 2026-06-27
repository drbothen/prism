---
document_type: research/plan
producer: architect (triage-and-routing)
timestamp: 2026-06-26
topic: T13 Capstone Demo — Finding Remediation Triage & Routing Plan
status: complete
feeds: T13 demo recording preparation
source_audit: .factory/research/demo-pre-flight-audit-2026-06-26.md
develop_head: f05a9f0e
---

# Demo Finding Remediation Triage & Routing Plan (2026-06-26)

Human directive: "fix everything before T13 recording." Production-grade default applies — no deferrals without explicit human direction + concrete future dependency.

---

## 1. Quick-Reference Routing Table

| Finding | Severity | Root Cause | Spec-or-Code Wrong | Owner | Prerequisite |
|---------|----------|------------|-------------------|-------|--------------|
| N1 — reference wrong enrich fn names | MAJOR | `build_reference_content` deduplicates by `infusion_id` ("threat_intel", "nvd") instead of per-field UDF names | CODE is wrong (reference must show callable UDF names per BC-2.11.022 / ADR-045) | implementer (prism-mcp `resources.rs`) | None |
| N1-B — unknown enrich fn returns internal error not E-QUERY-039 | MAJOR | `threat_intel(iocs_value)` parses, passes to query engine which cannot find the UDF, but error routing goes to catch-all internal error instead of E-QUERY-039 | CODE is wrong (E-QUERY-039 spec-correct per error-taxonomy.md) | implementer (prism-query enrichment gate) | None |
| N2 — dot-syntax now silent-empty (was E-QUERY-036) | MEDIUM | PR #203 added dot-notation fallback in `sensor_id_from_table_name` so `cyberint.alerts` now extracts `cyberint` as sensor prefix, routes to adapter, which fails with AllTargetsFailed (E-SENSOR-030) | CODE regression — prior E-QUERY-036 behavior was spec-correct per BC-2.11.023 / AC-011 | implementer (prism-query `materialization.rs`) | None |
| N3 — E-QUERY-032 vs runbook's E-QUERY-037 | LOW | Runbook Query Block 4 / §5.8 expects E-QUERY-037 for not-registered-sensor; actual behavior returns E-QUERY-032 | RUNBOOK is wrong — E-QUERY-032 is spec-correct per error-taxonomy.md (sensor registered globally but not for org); E-QUERY-037 is for "sensor not configured at all" | product-owner (T13-runbook.md §5.8 + Query Block 4) | None |
| N4 — runbook Step 3.5 uses armis for org-b | MEDIUM | org-b has no armis; runbook Step 3.5 targets `client_id="org-b"` | RUNBOOK is wrong — should be `org-c` (which has armis, confirmed working by audit) | product-owner (T13-runbook.md Step 3.5) | None |
| N5 — runbook §6.3 uses claroty_audit_log singular + log_id | MEDIUM | claroty TOML `table_name = "audit_logs"` → real table is `claroty_audit_logs`; `log_id` column does not exist (real column is `id`) | RUNBOOK is wrong — confirmed by direct TOML inspection | product-owner (T13-runbook.md Step 6.3) | None |
| N6 — list_plugins fast-fails -32003 | OBS | Prior audit claimed it worked; current HEAD now returns -32003 (same as list_infusions). No demo flow depends on it. | Neither — behavior is correct (prism-operations not merged); prior audit observation was incorrect | None — non-blocking observation | None |
| AUDIT-001 — prism_describe returns short table names | MEDIUM | Claroty/cyberint/armis/crowdstrike TOMLs use short `table_name` values ("alerts", "devices", "audit_logs") not sensor-prefixed names. `build_tables_for_client` faithfully reads `table.table_name.clone()`. | PRODUCT DECISION required — see §3.1 | product-owner (contract) + implementer (if field added) | Product decision must precede code fix |
| AUDIT-004 — 4 of 5 prompts embed dot-syntax | MEDIUM | `prompts.rs` hard-codes `FROM crowdstrike.alerts` etc. PR #203 rewrote the server but did not regenerate prompt bodies. | CODE is wrong — prompts must use FROM-ready sensor-prefixed names per the grammar spec | implementer (prism-mcp `prompts.rs`) | N2 fix must land first (or independently — prompts need correct table names regardless) |

---

## 2. Per-Finding Root Cause Analysis

### N1 — prismql://reference documents WRONG enrichment function names

**Root cause (code path):**

`build_reference_content` in `crates/prism-mcp/src/resources.rs` (lines ~1483-1506) calls `registry.udf_descriptors()`, which returns one `InfusionUdfDescriptor` per `[[infusion.fields]]` entry. Each descriptor carries:
- `name` = per-field UDF name (e.g., `threat_score`, `cvss_base_score`)
- `infusion_id` = aggregate infusion identifier (e.g., `threat_intel`, `nvd`)

The reference builder deduplicates by `infusion_id` and emits `enrich {infusion_id}(col)` — producing `enrich threat_intel(col)` and `enrich nvd(col)`. These are infusion IDs, not callable UDF names. The callable names in the DataFusion UDF registry are the per-field `name` values: `threat_score`, `threat_is_known_malicious`, `threat_sources`, `cvss_base_score`, `cvss_severity`, `cvss_vector`.

**Evidence:** `threatintel.infusion.toml` has `infusion_id = "threat_intel"` with three `[[infusion.fields]]` entries named `threat_is_known_malicious`, `threat_score`, `threat_sources`. `nvd.infusion.toml` has `infusion_id = "nvd"` with three entries named `cvss_base_score`, `cvss_severity`, `cvss_vector`.

**Spec-vs-code verdict:** CODE is wrong. BC-2.11.022 (prismql://reference contract) and ADR-045 §A require the reference to accurately document the callable syntax. The reference self-contradicts: its own example two sections later correctly uses `| enrich threat_score(src_ip)` (the correct per-field UDF name), but the "Available enrichment functions" list shows `threat_intel(col)` and `nvd(col)` (the infusion IDs).

**Fix:** Change the deduplication key in the reference builder from `desc.infusion_id` to `desc.name` (the UDF name), and emit `enrich {name}(col)` for each unique UDF name. This produces the full list: `threat_is_known_malicious`, `threat_score`, `threat_sources`, `cvss_base_score`, `cvss_severity`, `cvss_vector`.

**Routing:** implementer, `crates/prism-mcp/src/resources.rs` `build_reference_content` function.

---

### N1-B — Calling an unregistered enrichment name returns internal error, not E-QUERY-039

**Root cause:** When `threat_intel(iocs_value)` is written in a query, the parser accepts it (it's a valid identifier). The enrichment gate (E-QUERY-039, `PrismError::EnrichUdfNotFound`) must fire at plan time when the UDF name is not in `InfusionRegistry.udf_to_infusion`. The audit reports the actual error is `[upstream_error] - Internal error; see audit log` — meaning the call reaches the execution layer and fails with a generic internal error rather than returning E-QUERY-039.

**Spec-vs-code verdict:** CODE is wrong. Error-taxonomy.md E-QUERY-039 (line 258) specifies: "Fires at plan time (after E-QUERY-038 column check), before any sensor API or infusion HTTP call is made." The gate must intercept unknown enrichment names before execution. BC-2.11.019 is the BC anchor.

**Fix:** The enrichment-not-found gate needs to be verified for the case where an infusion_id is used as the function name rather than a UDF name. The `InfusionRegistry.udf_to_infusion` map is keyed by UDF name (e.g., `threat_score`), not by infusion_id (e.g., `threat_intel`). Calling `threat_intel(x)` would not be found in `udf_to_infusion`, so E-QUERY-039 SHOULD fire — unless the gate itself is incomplete or bypassed. This needs implementer investigation: either the gate is not wired for the pipe-mode path, or the infusion_id is somehow being recognized as a valid key via a different path.

**Routing:** implementer, prism-query enrichment gate (`E-QUERY-039` plan-time check), with investigation of the code path where `threat_intel(x)` bypasses the gate.

---

### N2 — Dot-syntax `FROM cyberint.alerts` now silent-empty instead of E-QUERY-036

**Root cause (code path):**

PR #203 extended `sensor_id_from_table_name` in `crates/prism-query/src/materialization.rs` (lines 1588-1596) with a dot-notation fallback: when the underscore-split path fails, try splitting by `.` to extract the sensor component. This was added for BC-2.11.023 / AC-011 (filter-mode source refs use dot notation).

The result: `cyberint.alerts` now successfully extracts `cyberint` as the `SensorId`, passes `is_sensor_registered` check (cyberint IS registered globally), enters the fan-out path, but the adapter fails to fetch because no table named `cyberint.alerts` exists in the cyberint DTU. All targets fail → E-SENSOR-030 (AllTargetsFailed), surfaced as a partial failure in `sensor_errors` with `isError: false` and 0 rows.

**Prior behavior:** `cyberint.alerts` passed `sensor_id_from_table_name` → None (the dot prefix check didn't exist), hit the E-QUERY-036 unknown-source-table path, returned E-QUERY-036 with did-you-mean "cyberint_alerts".

**Spec-vs-code verdict:** This is a spec-gated regression. BC-2.11.023 / AC-011 explicitly adds dot-notation support for Filter-mode source refs (e.g., `crowdstrike.detections | severity = 'HIGH'`). The dot-notation fallback in `sensor_id_from_table_name` was intentional for that case. But the side-effect is that `FROM cyberint.alerts` in SQL/Pipe mode now silently passes sensor extraction instead of returning the pedagogical E-QUERY-036.

The question is: should `FROM cyberint.alerts` return E-QUERY-036 (did-you-mean hint) or silently empty via E-SENSOR-030?

Per error-taxonomy.md E-QUERY-036: "Query references a table name whose sensor prefix is not registered in the adapter registry, OR whose prefix string fails sensor_id_from_table_name validation." The sensor prefix (`cyberint`) IS registered — so E-QUERY-036's technical trigger condition is not met by the current code. However, `cyberint.alerts` is NOT a valid registered table in any sensor spec. The pedagogical use-case is preserved by E-QUERY-037 (TableRegistry plan-time check) — if `cyberint.alerts` is not in the `TableRegistry`, E-QUERY-037 should fire before reaching `resolve_source_refs`.

**Critical path check:** E-QUERY-037 fires at plan time via `TableRegistry::is_registered()` check on the query plan's source refs. If `cyberint.alerts` (the literal dot-syntax string) is not in the registry, E-QUERY-037 fires. If `TableRegistry` stores tables by sensor-prefixed underscore names only (`cyberint_alerts`), then `cyberint.alerts` will not be found → E-QUERY-037 fires with did-you-mean "cyberint_alerts". But the audit shows E-SENSOR-030, not E-QUERY-037 — meaning the TableRegistry check is NOT intercepting `cyberint.alerts`. Either (a) the TableRegistry also accepts dot-notation entries (because the filter-mode change added them), or (b) the E-QUERY-037 gate is bypassed for this path.

**Routing decision:** This is an implementer investigation + fix. The desired end state is: `FROM cyberint.alerts` → E-QUERY-037 with "Did you mean 'cyberint_alerts'?" at plan time (pedagogically correct, matches the audit's pre-regression behavior, no silent empty). The fix involves either: (a) teaching the E-QUERY-037 gate to detect dot-notation table names and map them to the did-you-mean suggestion, or (b) preventing dot-notation from passing `sensor_id_from_table_name` for non-Filter-mode queries.

**Routing:** implementer, with investigation of whether E-QUERY-037 can intercept dot-notation table names. The fix must NOT break BC-2.11.023 / AC-011 Filter-mode dot-notation support.

---

### N3 — Runbook Query Block 4 / §5.8 expects E-QUERY-037 for not-registered-sensor

**Root cause:** The runbook was written before E-QUERY-032 was introduced (or before the distinction between "table globally unknown" vs "sensor not registered for this org" was codified). The actual behavior — E-QUERY-032 for org-scoped sensor authorization failure — is spec-correct per error-taxonomy.md E-QUERY-032: "sensor exists in the registry under a different org's key; the requesting org simply has no adapter for it."

E-QUERY-037 fires when the sensor/table is globally absent from TableRegistry. E-QUERY-032 fires when the sensor is registered but not for this org. The runbook's expectation of E-QUERY-037 is factually incorrect — the sensor IS configured (in the demo fleet), just not for org-a.

**Spec-vs-code verdict:** RUNBOOK is wrong. The current code behavior is spec-correct.

**Fix:** Update runbook Query Block 4 / §5.8 to expect E-QUERY-032 for the `FROM claroty_devices LIMIT 5 client_id="org-a"` case. Note: the runbook Step 4.1 prose also explains this as "Prism fails at plan time" with pedagogical E-QUERY-037 — the prose explanation should also be updated to describe E-QUERY-032 as the authorization-check result.

**Routing:** product-owner, T13-runbook.md §5.8 and Query Block 4 (Step 4.1).

---

### N4 — Runbook Step 3.5 uses armis for org-b (org-b has no armis)

**Root cause:** Simple wrong-org reference in the runbook. The org topology (from `demo.toml` and confirmed by audit §1.5):
- org-b: claroty + cyberint ONLY (no armis)
- org-c: crowdstrike + armis + claroty + cyberint

The query `FROM armis_devices | enrich cvss_base_score(device_cves_first) ... client_id="org-b"` returns E-QUERY-032 because armis is not registered for org-b.

**Spec-vs-code verdict:** RUNBOOK is wrong. Changing `client_id="org-b"` to `client_id="org-c"` makes the query work (confirmed by audit: org-c armis returns cvss_base_score 8.1, HIGH).

**Routing:** product-owner, T13-runbook.md Step 3.5.

---

### N5 — Runbook §6.3 uses claroty_audit_log (singular) and log_id column

**Root cause:** The claroty sensor TOML at `crates/prism-sensors/specs/claroty.sensor.toml` declares `table_name = "audit_logs"` — which, with the `claroty_` prefix, means the FROM-ready name is `claroty_audit_logs` (plural). The runbook uses `claroty_audit_log` (singular). Furthermore, the real columns in the claroty audit_logs table are `[action, actor, id, resource, timestamp]` — there is no `log_id` column; `id` is the identifier column.

**Note:** The prior 2026-06-24 audit incorrectly stated the singular form was correct. The current TOML source of truth (confirmed by direct inspection) shows plural `audit_logs`.

**Spec-vs-code verdict:** RUNBOOK is wrong — code and TOML spec agree on plural.

**Fix:** Update runbook Step 6.3 to `FROM claroty_audit_logs LIMIT 20` and use column `id` (not `log_id`) in any column-specific references.

**Routing:** product-owner, T13-runbook.md Step 6.3.

---

### N6 — list_plugins now fast-fails -32003

**Root cause:** Prior audit (2026-06-24) observed `list_plugins` working. Current HEAD has `list_plugins` returning -32003 "Feature not yet available: plugin management (prism-operations not merged)" — the same fast-fail pattern as `list_infusions`.

**Spec-vs-code verdict:** Both states are acceptable; prism-operations is not merged. The current behavior is correct. The prior audit observation was either a transient quirk or a version difference. No demo flow references `list_plugins`.

**Action:** Record as resolved-by-observation. No code or runbook change required.

---

### AUDIT-001 — prism_describe returns short table names not FROM-ready names

**Root cause (code path):**

`build_tables_for_client` in `crates/prism-mcp/src/tools/prism_describe.rs` emits `name: table.table_name.clone()` where `table_name` comes from the resolved sensor spec — i.e., the `table_name` field in the sensor TOML. Direct inspection confirms:
- `cyberint.sensor.toml`: `table_name = "alerts"` (not `cyberint_alerts`)
- `claroty.sensor.toml`: `table_name = "audit_logs"` (not `claroty_audit_logs`)
- `claroty.sensor.toml`: `table_name = "devices"` (not `claroty_devices`)

So the code faithfully reads the TOML and returns the short `table_name`. The `TableRegistry` however registers tables under the `sensor_id + "_" + table_name` (sensor-prefixed) convention for query routing. This means the `prism_describe` response `name` field and the FROM-clause requirement diverge.

For org-c with 4 sensors, `prism_describe` returns:
- `alerts` (from cyberint, claroty, and crowdstrike potentially share this — 3 entries all named "alerts")
- `devices` (from armis and claroty)
- `detections` (crowdstrike)
- `audit_logs` (claroty)
- `incidents` (multiple)

A naive analyst copying any of these `name` values into a FROM clause gets E-QUERY-037 or E-SENSOR-030.

**This is a product decision requiring human input.** Two options exist:

**Option A — Fix prism_describe to emit sensor-prefixed names.** Change `build_tables_for_client` to emit `name: format!("{}_{}", sensor_id, table.table_name)` instead of `table.table_name.clone()`. This makes `name` FROM-ready. The runbook's expected output in Step 1.2 already shows this format (`"crowdstrike_detections"`, `"armis_devices"`, etc.) — so Option A is what the runbook expected to be spec-correct. This would require no TOML changes.

**Option B — Add a `from_name` field to `TableDescriptor`.** Keep `name` as the short TOML table_name (preserve backward compat), add a new `from_name: String` field that carries the sensor-prefixed name. `pql_hints` then instructs "use the `from_name` value in FROM clauses." This is a larger API change.

**Spec-vs-code verdict:** The runbook's Step 1.2 expected output (which product-owner owns) already expects `name: "crowdstrike_detections"` — meaning Option A matches the existing spec intent. The code does not match the spec. CODE is wrong relative to the runbook's expected output (which is the product spec for this behavior).

**Routing:** product-owner must confirm whether the existing runbook Step 1.2 JSON example is the authoritative contract (in which case Option A is the fix), or whether a new `from_name` field is desired. Once decided, implementer applies the fix to `build_tables_for_client`.

**HUMAN INPUT REQUIRED:** Confirm Option A (fix `name` to be sensor-prefixed) vs Option B (add `from_name` field). The production-grade default is Option A since that's what the runbook spec shows, but the product-owner should confirm before implementation to avoid a BC change.

---

### AUDIT-004 — 4 of 5 MCP prompts embed dot-syntax table names

**Root cause:** `crates/prism-mcp/src/prompts.rs` contains hard-coded prompt bodies for `triage_alerts`, `client_overview`, `cross_client_status`, and `investigate_host`. These prompts were authored before the grammar remediation PR #203 changed the accepted table name format. The prompts use `FROM crowdstrike.alerts`, `FROM claroty.alerts`, etc. — dot-syntax that the query engine does not accept as a FROM target (produces E-SENSOR-030 after N2 regression, was E-QUERY-036 before).

Confirmed affected lines in prompts.rs: lines ~320-321, ~354-356, ~383-384, ~418.

**Spec-vs-code verdict:** CODE is wrong. The prompts must embed queries that the analyst can execute. Correct FROM-ready table names are: `crowdstrike_detections`, `crowdstrike_alerts` (if applicable), `claroty_alerts` → `claroty_devices` / `claroty_audit_logs`, `armis_devices`, `cyberint_alerts`. Prompt bodies should use the actual sensor-prefixed table names from the sensor TOML specs.

**Dependency on AUDIT-001:** If AUDIT-001 is fixed via Option A (prism_describe emits sensor-prefixed names), the prompts should match. The prompts also reference non-existent tables (e.g., `crowdstrike.alerts` vs the actual table `crowdstrike_detections`). The implementer fixing AUDIT-004 must consult the actual sensor TOML `table_name` values and sensor IDs to construct valid FROM-ready names.

**Note:** AUDIT-001 fix is a prerequisite only if Option B (add `from_name`) is chosen, since the prompt fix must use the same format as prism_describe. If Option A is chosen, both can be fixed in the same burst.

**Routing:** implementer, `crates/prism-mcp/src/prompts.rs`. The fix is regenerating the prompt bodies with correct sensor-prefixed table names.

---

## 3. Key Product Decisions Requiring Human Input

### 3.1 AUDIT-001 — prism_describe `name` field format

The only genuine product decision in this batch.

**Question:** Should `TableDescriptor.name` emit sensor-prefixed names (`crowdstrike_detections`) or short names (`detections`)?

**Context:**
- The runbook Step 1.2 expected JSON already shows sensor-prefixed names → this is the existing spec intent
- The actual TOML `table_name` values are short names (e.g., `alerts`, `detections`)
- The `TableRegistry` uses sensor-prefixed names for query routing
- Changing `name` is a breaking change to the `PrismDescribeResponse.tables[].name` field

**Recommendation (production-grade default):** Option A — fix `build_tables_for_client` to emit `format!("{sensor_id}_{table_name}")` as the `name` value. This makes prism_describe self-consistent with the query engine, matches the runbook spec, and eliminates the primary analyst confusion without adding a new API field.

**The human must confirm this before implementation begins.** The implementer MUST NOT proceed on AUDIT-001 or AUDIT-004 without this confirmation.

---

## 4. Recommended Fix Sequence

The following ordering minimizes re-work and allows independent parallel dispatch where possible.

### Phase 1 — Runbook fixes (no code dependency, can proceed immediately)

All three are product-owner tasks on `T13-capstone-demo-runbook.md`:

1. **N3** — Update Query Block 4 / §5.8 expected error code from E-QUERY-037 → E-QUERY-032 for sensor-not-registered-for-org cases
2. **N4** — Change Step 3.5 `client_id="org-b"` → `client_id="org-c"`
3. **N5** — Change Step 6.3 `FROM claroty_audit_log` → `FROM claroty_audit_logs`, and `log_id` → `id`

These are purely doc fixes and block nothing. Target: bump runbook to v1.5.

### Phase 2 — Product decision (blocks AUDIT-001 + AUDIT-004 code fixes)

4. **AUDIT-001 decision** — human confirms Option A (sensor-prefixed `name`) or Option B (new `from_name` field). This is a 1-question decision, not a research task.

### Phase 3 — Code fixes (implementer, can batch into one PR)

Once the AUDIT-001 decision lands, all code fixes can be batched:

5. **N1** — Fix `build_reference_content` to emit per-field UDF names (deduplicate by `desc.name` not `desc.infusion_id`)
6. **N1-B** — Verify and fix E-QUERY-039 gate for unknown enrichment function calls
7. **N2** — Fix dot-syntax detection: `FROM cyberint.alerts` must return E-QUERY-037 (not E-SENSOR-030); implement dot-syntax detection at the TableRegistry plan-time check level
8. **AUDIT-001** — Fix `build_tables_for_client` per the product decision (Option A or B)
9. **AUDIT-004** — Regenerate prompt bodies in `prompts.rs` with sensor-prefixed FROM-ready table names

N1 and N1-B are independent of AUDIT-001 and can be dispatched immediately without waiting for Phase 2.
N2 and AUDIT-004 are also independent of AUDIT-001 but should be coordinated since both touch the same user-facing table naming.

### Parallel dispatch option

If the AUDIT-001 decision comes quickly:
- N1 + N1-B can be dispatched to implementer RIGHT NOW (no dependency)
- N3 + N4 + N5 can be dispatched to product-owner RIGHT NOW (no dependency)
- AUDIT-001 decision → AUDIT-001 + AUDIT-004 + N2 dispatched as a single implementer burst

---

## 5. ADR / BC Change Assessment

No new ADR or BC is required for any of these fixes. All fixes are conformance repairs to existing specs:

| Finding | Spec Refs | Change Required? |
|---------|-----------|-----------------|
| N1 | BC-2.11.022, ADR-045 §A | No — fix code to match existing spec |
| N1-B | BC-2.11.019, E-QUERY-039 | No — fix code to match existing spec |
| N2 | BC-2.11.023, E-QUERY-036/037 | No — restore spec-correct behavior |
| N3 | E-QUERY-032, E-QUERY-037 | No — fix runbook to match code (code is correct) |
| N4 | Demo topology invariants | No — fix runbook wrong-org reference |
| N5 | TOML source of truth | No — fix runbook to match TOML |
| AUDIT-001 | BC-2.10.012 (prism_describe contract) | Product-owner must confirm new `name` format; no ADR required if treated as bug-fix conformance to Step 1.2 expected output |
| AUDIT-004 | BC-2.10.015/016/017 (prompt contracts) | No — fix prompt content to use valid FROM-ready table names |

If the product-owner decides Option B (add `from_name` field), a minor BC-2.10.012 amendment is needed to add the new field to the `TableDescriptor` contract. That amendment is still product-owner scope (field definition) + implementer scope (implementation).

---

## 6. N6 Disposition

**list_plugins fast-fails -32003 — CLOSED as non-blocking observation.**

Current behavior is correct. `list_plugins` was not a demo-critical tool; the prior audit's observation that it "worked" may have reflected a transient state or a version difference before an intermediate PR. No action required.

---

## 7. Summary: What Blocks T13 Recording?

Under the production-grade default (human directive: fix everything):

| Fix | Blocks Recording? | When Available |
|-----|------------------|----------------|
| N1 (reference fn names) | Blocks (analyst gets wrong fn names from reference) | Phase 3, now dispatchable |
| N1-B (internal error vs E-QUERY-039) | Blocks (opaque error if analyst follows reference) | Phase 3, now dispatchable |
| N2 (dot-syntax pedagogical regression) | Blocks (AUDIT-004 prompts use dot-syntax → silent empty) | Phase 3 after AUDIT-001 decision |
| N3 (runbook error code) | Blocks recording accuracy | Phase 1, now dispatchable |
| N4 (wrong org) | Blocks (Step 3.5 fails if followed verbatim) | Phase 1, now dispatchable |
| N5 (wrong table name) | Blocks (Step 6.3 fails if followed verbatim) | Phase 1, now dispatchable |
| AUDIT-001 (describe name format) | Blocks (analyst copies describe name into FROM → fails) | Phase 2+3 |
| AUDIT-004 (prompt dot-syntax) | Blocks (Claude following prompts gets 0 rows) | Phase 3 after AUDIT-001 decision |

The fastest path to a clean recording: dispatch N3/N4/N5 (product-owner) and N1/N1-B (implementer) simultaneously, get the AUDIT-001 product decision immediately, then dispatch N2 / AUDIT-001 / AUDIT-004 as a single implementer burst.
