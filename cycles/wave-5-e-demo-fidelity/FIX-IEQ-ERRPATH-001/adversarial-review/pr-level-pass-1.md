---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [1]
feature_head_at_review: dacb60fa
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 2
  low: 0
  obs: 1
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 1 — FIX-IEQ-ERRPATH-001

---

## Pass 1 (frozen dacb60fa; fresh-context adversary; fix-PR IEQ non-existent column error path; PR-LEVEL cascade begin; streak candidate 1/3 — NOT ADVANCING — 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 3 total (0 CRIT / 0 HIGH / 2 MED / 0 LOW / 1 OBS / 0 PROCESS-GAP)

**STREAK:** 0/3 — NOT CLEAN(strict); 2 MED findings present; streak does not advance. All 3 findings CLOSED same-burst. Fix-burst pushes 7e23a2c2 + 39c8b134; frozen HEAD changes to 39c8b134; streak resets on push per DRIFT-ORCH-PRLEVEL-PUSH-001.

**Code HEAD at review:** dacb60fa (frozen; fix-burst 51f071ff + clippy dacb60fa on top of LOCAL 3-CLEAN HEAD 35117a38; pushed to origin; PR #219 OPEN base develop; 5392/5392 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** NO — 2 MED + 1 OBS findings; strict criterion requires zero findings of any severity

**CLEAN(PR-merge):** NO — 2 MED findings; PR-merge gate requires zero CRIT + HIGH + MED

---

## Findings

### ADV-PR-P1-MED-001 — SAP-1 / PG-LP11-001: column_not_found.rejected catalog row lacked sanitize_for_log annotation

**Severity:** MED
**Classification:** SAP-1 (structured event catalog completeness) / PG-LP11-001

**Finding:** BC-2.16.002 `column_not_found.rejected` Canonical Structured Event Catalog row was missing the `sanitize_for_log` annotation on the `column` field after fix-burst 51f071ff applied `sanitize_for_log` at all 3 emission sites. The sibling `infusion.coercion_failed` row already carries this annotation. The code behavior was correct (sanitization applied at all 3 sites) but the catalog spec description did not reflect the sanitization contract, leaving the catalog in a state where a future implementer reading the spec would not know the field is sanitized.

**Closure:** Product-owner burst — BC-2.16.002 v2.07→**v2.08** (annotation on `column` field in `column_not_found.rejected` catalog row: `sanitize_for_log` applied before emission, matching sibling `infusion.coercion_failed` row; no new event_type; catalog count unchanged 91; POL-30 Fork B description amendment). Companion: BC-2.11.016 v1.21→**v1.22** (§Postconditions "Injection-safety of `column` (MCP-facing payload)" clause); BC-2.11.017 v1.9→**v1.10** (pin-only); BC-2.11.020 v1.14→**v1.15** (pin-only); BC-2.11.004 v1.26→**v1.27** (pin-only); error-taxonomy v2.34→**v2.35** ({column} injection-safety clause). Story-writer pin round: S-DEMO-FIDELITY-REMEDIATION-001 v2.39→**v2.40**, S-DEMO-PRISMQL-ONBOARDING-001-B v2.16→**v2.17**, S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.25→**v1.26**, S-PRISMQL-CASE-INSENSITIVE-001 v1.50→**v1.51**.

**Status:** CLOSED

---

### ADV-PR-P1-MED-002 — TD-VSDD-059: SEC-FIND-001 test was a paper-fix — exercised sanitize_for_log helper only, not emission path

**Severity:** MED
**Classification:** TD-VSDD-059 (paper-fix detection)

**Finding:** The CWE-117 unit test added in fix-burst 51f071ff asserted that `sanitize_for_log` strips control characters correctly but called the sanitization helper function directly rather than exercising the actual `column_not_found.rejected` tracing emission path in production code. A load-bearing test must exercise the actual emission site (`check_query_column_availability` or `check_pipe_stage_columns`) and assert that the sanitized column name appears in the tracing event while the raw control character does not. The doc comment on the helper test claimed it exercised "all 3 column_not_found.rejected sites" which was inaccurate.

**Closure:** Test-writer @**7e23a2c2** — 3 `#[tracing_test::traced_test]` emission-path locks exercising the actual code paths: (1) single-tenant E-QUERY-038 `column_not_found.rejected` emission with control-char column name (`\x01malicious`) — asserts `logs_contain("column_not_found.rejected")` AND `!logs_contain("\x01")`; (2) multi-tenant variant; (3) binding-context (SqlPipe `| where` stage) variant. Helper-test doc-claim comment corrected to clarify scope (helper unit test only). Plus 2 RED payload-injection gates for ADV-PR-P1-OBS-001 closure (injection-safety on MCP-facing payload path). Total 5 new tests. just check 5397/5397 GREEN (+5); non-exhaustive 89/89.

**Status:** CLOSED

---

### ADV-PR-P1-OBS-001 — CWE-116 / AD-017: ColumnNotFoundDetails.column + Display echoed RAW user input to LLM-facing MCP payload

**Severity:** OBS (orchestrator-adjudicated to FIX per AD-017 agent-harness prompt-injection principle)
**Classification:** CWE-116 (Improper Encoding or Escaping of Output); AD-017 (agent-harness prompt-injection defense)

**Finding:** `ColumnNotFoundDetails.column` stored the raw column name string verbatim and `Display` forwarded it to the MCP tool response JSON payload consumed by LLM agents. A malicious or adversarially-crafted column name (e.g., from a sensor schema that echoes user-controlled field names) could contain prompt injection sequences reaching the LLM context. `sanitize_for_log` mitigates the log injection path (CWE-117, closed by 51f071ff) but `ColumnNotFoundDetails.column` is a separate path: it flows into `PrismError::ColumnNotFound` → `error_mapping.rs` → MCP `ErrorData.data` → LLM tool response. The `Display` impl for E-QUERY-038 echoes `{column}` without sanitization on the MCP-facing payload. Scope: all 3 current emission sites + all future callsites that construct `ColumnNotFoundDetails::new`.

**Orchestrator adjudication:** FIX per AD-017 agent-harness prompt-injection principle (CLAUDE.md §Agent Harness Design). Escalated from OBS → production-grade fix required in-burst.

**Closure:** Product-owner spec layer — error-taxonomy v2.34→**v2.35** (injection-safety clause for `{column}` in E-QUERY-038 Display template; sanitize before MCP-facing payload); BC-2.11.016 v1.21→**v1.22** (§Postconditions "Injection-safety of `column` (MCP-facing payload)" clause added); BC-2.11.017 v1.9→**v1.10** (pin-only; injection-safety sibling sync); BC-2.11.020 v1.14→**v1.15** (pin-only; injection-safety sibling sync); BC-2.11.004 v1.26→**v1.27** (pin-only; injection-safety sibling sync). Test-writer 2 RED payload-injection gates @7e23a2c2 (assert MCP-facing `ErrorData` does not contain control characters from column name). Implementer @**39c8b134** — `sanitize_for_log` applied at `ColumnNotFoundDetails::new` chokepoint in `prism-core/src/error.rs`; covers all current 3+ callsites and all future callsites by construction; TD-VSDD-060 sweep: prism-mcp/prism-query test callsites identity-safe (column names are string literals in tests, not user input). CI-FAIL-002 also bundled in same commit: Rust 1.97.0 `clippy::useless_borrows_in_formatting` lint in `materialization.rs:761` (redundant `&` removed; sibling sweep clean). just check 5397/5397 GREEN (5392→5397: +5 tests); non-exhaustive 89/89.

**Status:** CLOSED

---

## Convergence Assessment

**Trajectory:** 6→3→3→2→1→[0]→2→[0]→4(low/obs)→1(med)→1(med)→[0]→[0]→[0] (LOCAL 19 passes, 3-CLEAN on 35117a38) → PR-LEVEL pass 1 on dacb60fa: **3** (0 CRIT / 0 HIGH / 2 MED / 0 LOW / 1 OBS / 0 PROCESS-GAP)

**Novelty:** HIGH — all 3 findings target the two post-LOCAL-convergence commits (51f071ff sanitize_for_log application + dacb60fa clippy fix) that were added after LOCAL 3-CLEAN convergence. ADV-PR-P1-MED-001 SAP-1 catalog annotation gap introduced by 51f071ff; ADV-PR-P1-MED-002 paper-fix test in 51f071ff; ADV-PR-P1-OBS-001 injection-safety gap exposed by the SEC-FIND-001 fix scope (sanitize_for_log fixed log path but left MCP payload path unguarded). Zero findings carry over from LOCAL cascade.

**Pattern:** PR-LEVEL adversary with fresh context found 3 findings all targeting the post-LOCAL-convergence fix-burst. This is the characteristic pattern of LOCAL cascade converging cleanly on the original feature work while a security-focused fix-burst (51f071ff) introduces catalog and test-quality gaps.

**Streak status:** 0/3 — NOT CLEAN(strict) on dacb60fa. All findings CLOSED same-burst by product-owner + test-writer + implementer. New HEAD 39c8b134 pushed (7e23a2c2 + 39c8b134 on top of dacb60fa; just check 5397/5397 GREEN; non-exhaustive 89/89). Streak RESET by push per DRIFT-ORCH-PRLEVEL-PUSH-001. **NEXT: PR-LEVEL adversary pass 2 on new frozen 39c8b134** (streak candidate 1/3; BC-5.39.001 3-CLEAN).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** FAIL at pass start — `column_not_found.rejected` catalog row lacked `sanitize_for_log` annotation on `column` field (ADV-PR-P1-MED-001). CLOSED: BC-2.16.002 v2.07→v2.08 by product-owner (annotation mirrors sibling `infusion.coercion_failed` row).

**SAP-2:** N/A — No sensor TOML spec modifications in this fix cascade.

**TD-VSDD-059 (paper-fix detection):** FAIL at pass start — SEC-FIND-001 test exercised helper not emission path (ADV-PR-P1-MED-002). CLOSED: 3 emission-path tracing locks @7e23a2c2.

**TD-VSDD-060 (sibling-site sweep):** PASS — implementer @39c8b134 confirmed `ColumnNotFoundDetails::new` is the single chokepoint; sweep: prism-mcp/prism-query test callsites identity-safe; no other production callsites unpatched.

**CWE-116 / AD-017 (injection-safety, MCP-facing payload):** FAIL at pass start — OBS-001 found MCP-facing payload unguarded. CLOSED: `sanitize_for_log` at `ColumnNotFoundDetails::new` chokepoint + spec layer @39c8b134; 2 RED payload-injection gates GREEN @7e23a2c2.

**POL-14 (BC auto-promotion):** N/A — no story merge in this pass.

**BC-5.39.001 (3-CLEAN streak):** NOT ADVANCED — pass result NOT CLEAN(strict). Findings CLOSED same-burst; HEAD changed to 39c8b134 on push; streak reset per DRIFT-ORCH-PRLEVEL-PUSH-001.
