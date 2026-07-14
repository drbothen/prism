---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [14]
feature_head_at_review: 01f6070c
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 1
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 14 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 14 (frozen 01f6070c; fresh-context adversary; PR #222 MCP row-shape null serialization + [H8b] redundancy sweep + threatintel .prx staleness gate + retryable-coverage expansion + 28-explicit-VariantMeta-arms + 117-variant sentinel + ci.yml sidecar diagnostic + scripts/hash-plugin-source.py repo_root anchoring; PR-LEVEL cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

---

## Findings

### F-MCPRS-PRL14-MED-001 [MED][semantic-anchoring] — CLOSED @9e116a01 (fix-burst-25: BC-2.10.007 v1.18→v1.19 PO-ratified + code alignment + load-bearing test)

**Severity:** MED
**Classification:** semantic-anchoring — CursorCapExceeded: sentinel arm commented "internal" (E-STORE section placement) vs. shipped VariantMeta arm using "validation" category / `original_params_valid: false` vs. BC-2.10.007 §Category table silent on CursorCapExceeded (§Category enumerated neither "internal" nor "validation" for this variant); no test asserted the category; discrepancy survived the fix-burst-24 117-arm audit because arm placement in the `Cursor*` group appeared coherent under a visual scan (E-STORE section → "internal" inference seemed locally reasonable without cross-checking the shipped arm)
**Status:** CLOSED — fix-burst-25 @9e116a01

**Finding:** `PrismError::CursorCapExceeded` is the process-wide resource cap for concurrent cursors (`E-STORE-020`). At HEAD 01f6070c, three documents had conflicting claims about its MCP structured-error category:

(1) **117-variant sentinel in `error_category_coverage.rs`:** The arm resides in the `E-STORE` section (the "INTERNAL_ERROR" grouping alongside `TableNotAvailable`, `NoSuchOrg`, `ColumnTypeMismatch`). By placement contract, E-STORE arms → `"internal"` category. The sentinel arm comment reads `"internal"` — correct.

(2) **Shipped VariantMeta arm in `error_mapping.rs`:** The actual production mapping arm produced `category: "validation"`, `original_params_valid: false`. This contradicts the sentinel placement and violates the semantic rule: `"validation"` category is reserved for caller-controllable parameter errors (bad cursor token, invalid page size, malformed query). `CursorCapExceeded` fires when the process exhausts its 200-cursor cap — a non-controllable, process-level resource state. Retrying immediately with the same request does not help; the cap is enforced systemically.

(3) **BC-2.10.007 §Category table:** Neither "validation" nor "internal" was listed for `CursorCapExceeded`. The §Category table was silent on this variant entirely, meaning no declarative specification existed for an adversary or developer to consult.

**Semantic analysis:** `CursorCapExceeded` belongs to the same class as `QueryMemoryBudgetExceeded` and `QueryMaterializationLimitExceeded` — all three are process-wide resource cap enforcements, not caller-controllable parameter errors. The agent strategy when a resource cap is hit is **escalate** (notify operator, reduce concurrency), not **fix-params** (retry with different parameters). `original_params_valid: true` (the parameters are valid; the cap is a process state) and `retryable: false` (cap is enforced until existing cursors close — immediate retry does not help). `"validation"` grouping with caller-controlled `Cursor*` errors (e.g., `CursorExpired`, `CursorTokenUnknown`, `CursorPageSizeInvalid`) was architecturally incorrect.

**Severity rationale:** MED because: (1) the sentinel arm comment was correct but the production mapping arm was wrong — the discrepancy meant the sentinel's documentation contract was broken at the most critical site (the shipped runtime behavior); (2) BC-2.10.007 §Category table was silent, creating a spec vacuum for this variant; (3) no load-bearing test asserted the category, meaning the discrepancy had no automated detection. The finding survived the fix-burst-24 117-arm audit because arm placement in a named section creates a false-positive heuristic — the audit method was visual grouping rather than cross-referencing the shipped `error_mapping.rs` arm output.

**Fix plan — fix-burst-25:** PO ratifies BC-2.10.007 v1.18→v1.19: add `CursorCapExceeded` to §Category table as `"internal"`, `original_params_valid: true`, `retryable: false`, rationale = process-wide resource cap (same class as `QueryMemoryBudgetExceeded`/`QueryMaterializationLimitExceeded`). Code: move `map_prism_error` arm to the `INTERNAL_ERROR` / E-STORE block; add `ec_code_override: Some("E-STORE-020")`; add BC-verbatim suggestion. Add load-bearing test asserting all five observable fields.

**Closure evidence @9e116a01 (fix-burst-25):**

(1) **BC-2.10.007 v1.18→v1.19 (PO-ratified):** `CursorCapExceeded` added to §Category table as category `"internal"`, `original_params_valid: true`, `retryable: false`; rationale documents process-wide cap semantics and agent-escalate strategy (same class as `QueryMemoryBudgetExceeded`/`QueryMaterializationLimitExceeded`; `"validation"` grouping with `Cursor*` caller-controlled errors was wrong).

(2) **`error_mapping.rs` arm moved to INTERNAL_ERROR E-STORE block** (`PrismError::CursorCapExceeded`): category `"internal"`, `original_params_valid: true`, `retryable: false`, `ec_code_override: Some("E-STORE-020")`, suggestion `"Cursor capacity exhausted. Wait for existing cursors to close before retrying."` (BC-2.10.007 §Category v1.19 verbatim). Arm relocated from the `"validation"` block; 28-arm explicit group count unaffected.

(3) **117-variant sentinel comment verified:** The E-STORE section `CursorCapExceeded` arm comment already read `"internal"` — consistent with the fix; no change required to sentinel.

(4) **New load-bearing test `test_BC_2_10_007_cursor_cap_exceeded_category_is_internal` (5 assertions):**
- `category == "internal"` (not "validation")
- `original_params_valid == true` (not false)
- `retryable == false`
- `ec_code_override == Some("E-STORE-020")`
- suggestion contains "Cursor capacity exhausted"

**TD-VSDD-060 sweep:** grep `CursorCapExceeded.*validation\|validation.*CursorCapExceeded` across `crates/` — zero hits post-fix.

**Test verification @9e116a01:** 480/480 prism-mcp GREEN. 261/261 prism-core GREEN.

---

### F-MCPRS-PRL14-LOW-001 [LOW][ci-as-code false-green] — CLOSED @e8db73f4 (fix-burst-25 pt1: committed-.prx validation before build in both CI gates)

**Severity:** LOW
**Classification:** ci-as-code false-green — ci.yml job docblock claimed "committed `.prx` validated by `wasm-tools validate`" but actual flow ran build FIRST (which overwrites the committed `.prx` artifact) THEN ran `wasm-tools validate` on the rebuilt binary, not the committed artifact; the validation step could never catch corruption in the committed file
**Status:** CLOSED — fix-burst-25 pt1 @e8db73f4

**Finding:** Two CI job blocks in `ci.yml` contained the comment "validate committed `.prx` against wasm spec before build" or equivalent, but in both cases the `cargo build` step (which builds and writes the `.prx` artifact) ran BEFORE the `wasm-tools validate` step:

(1) **`wasm32-compile-check` job:** The crowdstrike-oauth2 and threatintel `.prx` binaries are committed sidecar artifacts (ground truth for the wasm spec validation CI gate). The committed files are the ones that ship to users. The validation step was validating the freshly-rebuilt binary (which may or may not match the committed file), not the committed artifact itself. A committed `.prx` that was manually edited or corrupted would pass CI undetected.

(2) **`wasm32-threatintel-staleness-check` job:** Same structural issue — the staleness check validated the rebuilt binary rather than the committed file.

**Severity rationale:** LOW because: (1) the committed `.prx` files are generated artifacts checked in for distribution purposes; a CI pipeline that validates only the rebuilt binary provides false confidence about the integrity of the committed artifact; (2) the issue is not a runtime behavioral defect — the shipped plugin is built fresh; but the CI gate as a documentation-and-validation contract is incorrect.

**Fix plan — fix-burst-25 pt1:** Add committed-`.prx` structural validation steps to BOTH jobs, positioned BEFORE the build step. Add reachability assertions to confirm the validation actually exercises the committed file.

**Closure evidence @e8db73f4 (fix-burst-25 pt1):**

(1) **`wasm32-compile-check` job:** New step added BEFORE `cargo build`: `wasm-tools validate <path-to-committed-crowdstrike-oauth2.prx>` + `wasm-tools validate <path-to-committed-threatintel-lookup.prx>`. Both committed binaries validated against the wasm spec prior to build. Reachability assertion added (sha256 of committed file logged; step fails on non-zero wasm-tools exit).

(2) **`wasm32-threatintel-staleness-check` job:** Same structural fix — committed binary validation before build.

(3) **YAML syntax validated** — both jobs parsed successfully.

---

### F-MCPRS-PRL14-OBS-001 [OBS][defense-in-depth] — CLOSED @e8db73f4 (fix-burst-25 pt1: strip_userinfo + strip_path_from_authority fragment-hardened)

**Severity:** OBS
**Classification:** defense-in-depth — `strip_userinfo` in `prism-mcp/src/security.rs` used `'/'` and `'?'` as authority/path/query boundary sentinels but did not split on `'#'`; an adversarially crafted URL `https://host#user:pw@evil.com/path` would misparse the authority boundary, potentially leaking the `user:pw@evil.com` portion as part of the authority in the stripped output or failing to correctly identify the host boundary
**Status:** CLOSED — fix-burst-25 pt1 @e8db73f4

**Finding:** `strip_userinfo` (VP-050 URL redaction function) strips userinfo (`user:password@`) from URLs before logging. The function identified the end of the authority component by scanning for the FIRST occurrence of `'/'`, `'?'`, or end-of-string. It did not scan for `'#'` (fragment delimiter). Per RFC-3986, a `'#'` character immediately terminates the authority section just as `'/'` does.

An adversarially crafted URL `https://host#user:pw@evil.com/path` would cause `strip_userinfo` to:
1. Scan forward from the end of the scheme (`://`)
2. Find the `#` character before any `/`, `?`, or EOS
3. Interpret the fragment-start `#user:pw@evil.com/path` as part of the authority
4. Incorrectly parse `user:pw` as userinfo in a fragment context

While `#`-containing URLs are rare in API contexts (RFC-3986 forbids fragments in authority), the hardening cost is zero and the vector is theoretically exploitable in adversarial URL injection scenarios (CWE-116).

**Severity rationale:** OBS because: (1) real-world sensor API URLs do not contain `#` fragments; (2) the misparse does not leak credential values to external systems — it affects the logged/stripped representation only; (3) the hardening is a one-line fix with no semantic change for well-formed URLs.

**Closure evidence @e8db73f4 (fix-burst-25 pt1):**

(1) **`strip_userinfo` hardened:** authority-end boundary detection now uses `min('/', '?', '#')` — the earliest of the three delimiters terminates the authority scan. Fragment-containing URLs correctly limit authority parsing at the `#` boundary.

(2) **`strip_path_from_authority` similarly hardened** (companion function, same boundary logic).

(3) **`test_vp050_strip_userinfo_fragment_isolation` (3 cases):**
- `https://host#user:pw@evil` → correctly strips fragment (no userinfo leakage)
- `https://user:pw@host#fragment` → userinfo correctly stripped before fragment
- `https://host/path#user:pw@evil` → path boundary wins; fragment after path not misidentified as authority

(4) **`prop_vp050_fragment_stripped` proptest:** Arbitrary URLs with `#` characters never produce output containing `@` in the authority portion when no real userinfo is present.

---

## SAP-1 Emission Catalog Probe

**PASS.** `crates/` `event_type =` emission sites at HEAD 01f6070c sampled against BC-2.16.002 §Postconditions Canonical Structured Event Catalog — all catalogued. Fix-burst-25 changes (`error_mapping.rs` arm reassignment + CI YAML + `security.rs` fragment hardening) introduced zero new `event_type =` emissions. No BC-2.16.002 catalog row required.

---

## Positive Verifications

- **EC-11-079 single `with_explicit_nulls(true)` chokepoint:** `server.rs` `WriterBuilder` construction confirmed as sole `WriterBuilder` site in `prism-mcp/src/`; null-not-absent contract enforced at one gated location; no new `WriterBuilder` sites introduced by fix-burst-25.
- **5 Red Gate tests:** `test_null_column_is_explicit_null`, `test_non_null_column_not_absent`, `test_absent_column_when_not_in_schema`, `test_all_null_values_explicit`, `test_mixed_null_non_null_row_shape` — all confirmed present and GREEN at 01f6070c.
- **28-arm explicit VariantMeta groups verified:** 12 internal / 3 configuration / 3 validation / 10 upstream_error-explicit as established by fix-burst-22 (OBS-003). `CursorCapExceeded` arm moved from validation group (3 arms) to internal/E-STORE group (now 13 arms); total arm count 28 UNCHANGED (validation group shrinks to 2; internal group grows to 13; net zero).
- **117-variant sentinel coverage:** Post-fix: `CursorCapExceeded` E-STORE section arm comment reads `"internal"` — consistent with production mapping. TD-VSDD-060 sweep: zero `validation.*CursorCapExceeded` or `CursorCapExceeded.*validation` hits in `crates/`.
- **BC-2.11.018 `normalized_pql` conditional-insert wiring:** `pql_metadata.rs` `normalized_pql` field set only when `metadata_include_normalized_pql: true`; confirmed present in branch; no regression from fix-burst-25.
- **CI three-gate staleness design traced:** `wasm32-compile-check` (gate 1: committed-.prx → wasm-tools before build), `wasm32-threatintel-staleness-check` (gate 2: same pattern), and byte-identity post-build comparison (gate 3: committed == rebuilt) — all three gates confirmed present and structurally correct post-fix @e8db73f4.
- **VP-050 redaction walk:** `strip_userinfo` function walk confirmed; fragment-hardened `min('/', '?', '#')` boundary; `strip_path_from_authority` companion hardened; 3 fragment-isolation test cases + proptest all GREEN.

---

## Summary

**CLEAN(strict): NO** (1 MED + 1 LOW + 1 OBS — no severity is zero-finding)
**CLEAN(PR-merge): NO** (1 MED finding — MED blocks both CLEAN(strict) and CLEAN(PR-merge) per BC-5.39.001)

Streak: **0/3** (streak cannot advance; MED finding present).

All 3 findings CLOSED via fix-burst-25:
- **F-MCPRS-PRL14-MED-001 CLOSED @9e116a01 (fix-burst-25 pt2):** BC-2.10.007 v1.18→v1.19 (PO-ratified: CursorCapExceeded category "internal", original_params_valid true, retryable false, ec_code_override E-STORE-020); `error_mapping.rs` arm moved to INTERNAL_ERROR/E-STORE block; `test_BC_2_10_007_cursor_cap_exceeded_category_is_internal` (5 assertions); 480/480 prism-mcp GREEN; 261/261 prism-core GREEN.
- **F-MCPRS-PRL14-LOW-001 CLOSED @e8db73f4 (fix-burst-25 pt1):** ci.yml committed-.prx structural validation steps added to both CI gates before build; reachability assertions added; YAML validated.
- **F-MCPRS-PRL14-OBS-001 CLOSED @e8db73f4 (fix-burst-25 pt1):** `strip_userinfo` + `strip_path_from_authority` fragment-hardened (`'#'` boundary added); 3 fragment-isolation test cases + proptest.

NEW MCP HEAD: @9e116a01 (LOCAL-ONLY; push pending to origin/fix/DEFECT-MCP-ROWSHAPE-NULLS-001).

CASCADE TALLY: 34 passes / 25 fix-bursts. PUSH @9e116a01 PENDING; next = PR-LEVEL pass 15 on frozen @9e116a01 (streak 0/3).
