---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 7
target_sha: 8254f075
base_sha: 95d46be2
verdict: BLOCKED-hard
streak: "0/3 → 0/3"
finding_summary: {CRITICAL: 0, HIGH: 4, MEDIUM: 2, LOW: 1, OBS: 0}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7"
idempotency_check: false
producer: adversary (vsdd-factory)
---

# Adversarial Review — Pass-7 — S-PLUGIN-PREREQ-D

## §1 Context

- Target HEAD: factory `8254f075`, develop `95d46be2`
- Streak before pass-7: 0/3 (pass-6 reset via 4 findings missed by pass-5 false-CLEAN)
- Fix-burst-5 closed: F-LP6-MED-001 (Token Budget arithmetic), F-LP6-LOW-002 (changelog wording), F-LP6-LOW-003 (Task 8 column terminology), F-LP6-OBS-004 (AC-9 re-anchored to ADR-023 §C4)
- Pass-7 mandate: rigorous TD-VSDD-059 paper-fix rederivation; POL-20 anchored-regex compliance; semantic-anchor chain audit

## §2 Pass-6 Closure Rederivation

| Finding | Closure Claim | Pass-7 Status | Evidence |
|---------|---------------|---------------|----------|
| F-LP6-MED-001 (Token Budget arithmetic) | rows sum 39,800 = Total; pct 15.5% | CONFIRMED CLEAN | Rows 7,000+12,000+4,000+8,000+3,000+1,000+800+4,000 = 39,800 (matches Total); pct 15.5% |
| F-LP6-LOW-002 (changelog v1.1 BC count notation) | "swap BC-2.17.005 for BC-2.17.007 (7→7 BCs net)" | CONFIRMED CLEAN | Exact phrasing present |
| F-LP6-LOW-003 (Match-Site Inventory column convention) | "Task 8" replaces "AC-8 tasks" | CONFIRMED CLEAN | "Task 8: removed after tests added" present |
| F-LP6-OBS-004 (AC-9 anchor re-routed) | AC-9 cites ADR-023 §C4, not BC-2.17.002 | CONFIRMED CLEAN | AC-9 header references ADR-023 §C4; BC-2.17.002 moved to body as out-of-perimeter note |

All 4 pass-6 closures textually idempotent. No paper-fix regression on closed items.

## §3 POL-20 Anchored-Regex Workspace Sweep

- Sweep `^introduced:.*` matched 236 BC files.
- Sampled 100+ values: every value is `cycle-N` or quoted `"YYYY-MM-DD"`.
- Negative-match probe `^introduced:\s*(?!cycle-|"|[0-9]{4}-)` returned 0 results.
- **POL-20 compliance: VERIFIED — zero violations.**

## §4 Findings

### F-LP7-HIGH-001 — `pipeline.rs` path systematically mis-anchored

- **Severity:** HIGH
- **Scope:** story spec (8+ citations: Architecture Mapping, Purity Classification, File Structure Requirements, Match-Site Inventory, Tasks 6 + 8, Token Budget, Library table)
- **Confidence:** HIGH
- **Evidence:** Story cites `crates/prism-spec-engine/src/plugin/pipeline.rs` in 8+ locations. `Glob('crates/prism-spec-engine/src/**/pipeline*.rs')` returns `crates/prism-spec-engine/src/pipeline.rs` (under `/src/`, NOT under `/src/plugin/`). The cited path does NOT exist.
- **Why it matters:** An implementer following the story will fail to find the file. POLICY 4/5 violation. Worst-anchor systematic gap; survived 6 adversarial passes because no prior pass executed Glob against the actual filesystem.
- **Fix-routing:** story-writer — sweep all citations; canonical path is `crates/prism-spec-engine/src/pipeline.rs`.

### F-LP7-HIGH-002 — `auth_provider.rs` path systematically mis-anchored

- **Severity:** HIGH
- **Scope:** story spec (4+ citations: Architecture Mapping, Purity Classification, File Structure, Match-Site Inventory, Task 5)
- **Confidence:** HIGH
- **Evidence:** `Glob('crates/prism-spec-engine/src/**/auth_provider*.rs')` returns `crates/prism-spec-engine/src/auth_provider.rs` (NOT under `/plugin/`). Story citations use `src/plugin/auth_provider.rs`.
- **Why it matters:** Same blast-radius as HIGH-001. POLICY 4/5 violation.
- **Fix-routing:** story-writer — replace all `src/plugin/auth_provider.rs` with `src/auth_provider.rs`.

### F-LP7-HIGH-003 — BC-2.22.001 does not enumerate plugin-load step; 4 ACs trace to non-existent invariant

- **Severity:** HIGH
- **Scope:** story spec ACs 1-4 + BC-2.22.001 file itself
- **Confidence:** HIGH
- **Evidence:** `Grep "plugin" BC-2.22.001-boot-orchestration.md` returns ZERO matches (case-insensitive). BC §Sequencing Invariant enumerates Steps 2 → 3 → 5 → 6 → 7-8 → 9; no plugin-load step. §Pre-Traffic Gate Invariant lists 6 conditions; none mention plugin-load. §Postconditions reference 4 init subsystems + steps 7/8/9; no plugin postcondition. §Exit-Code Map has no plugin row. Yet AC-1/2/3/4 trace to BC-2.22.001's "sequencing invariant," "pre-traffic gate invariant," and "postcondition" for plugin behaviors the BC does not specify.
- **Why it matters:** POLICY 4 violation. After story merges, the implementer's tests cannot be load-bearing against BC-2.22.001 because the BC's text does not specify what the tests are verifying. Production-grade "Pending PO review" anti-pattern (CLAUDE.md): the BC needs amendment in scope.
- **Fix-routing:** product-owner — amend BC-2.22.001 to add plugin-load step in §Sequencing Invariant, gate condition in §Pre-Traffic Gate Invariant, postconditions for PRISM_DISABLE_PLUGIN_LOAD escape valve + unsigned-plugin WARN/audit, and E-PLUGIN-013..016 rows in §Exit-Code Map. Per CLAUDE.md Canonical Principle Rule 4: AI-built-defect-fix-in-scope; do NOT defer.

### F-LP7-HIGH-004 — `host_http_request` `.timeout(Duration::from_secs(10))` clamps the 30s client-builder timeout

- **Severity:** HIGH
- **Scope:** story spec AC-9 + Match-Site Inventory; production code at `crates/prism-spec-engine/src/plugin/host_functions.rs`
- **Confidence:** HIGH
- **Evidence:** `Grep timeout` in `host_functions.rs` line ~153 returns `.timeout(Duration::from_secs(10))`. Line ~30 doc comment: "Enforces a 10-second per-request timeout." Story AC-9 specifies `Client::builder().timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))` where `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS = 30`. Per `reqwest` semantics, `RequestBuilder.timeout(D')` overrides `Client.timeout(D)` — in practice, the per-request value wins. Story Match-Site Inventory does NOT enumerate the line-153 site.
- **Why it matters:** TD-VSDD-059 paper-fix detection. If the implementer follows the story literally and adds `Client::builder().timeout(30)` in boot.rs without removing/updating the per-request override, the **effective request timeout is still 10 seconds**. AC-9's "This is TD-S-PLUGIN-PREREQ-B-005 closure" claim is functionally inert under this implementation path. Adversary independent verification per Standing Rule 3 §1: implementer self-disclosure that AC-9 closes TD-B-005 is **not authoritative** — the per-request clamp is the load-bearing gate.
- **Fix-routing:** story-writer — add `host_http_request` site to Match-Site Inventory with explicit instruction; update Task 4 prose; sibling doc-comment "10-second per-request timeout" must be updated to "30-second" in the same commit (TD-VSDD-060).

### F-LP7-MED-001 — BC-2.17.002 E-PLUGIN-005 cites 10s timeout; story uses 30s; defer-to-future-PO punt violates production-grade default

- **Severity:** MEDIUM
- **Scope:** story spec AC-9 out-of-perimeter note + BC-2.17.002 E-PLUGIN-005 row
- **Confidence:** HIGH
- **Evidence:** BC-2.17.002 row "E-PLUGIN-005 | host::http_request times out (10s per request limit)". Story AC-9 acknowledges gap: "future PO-led story or backlog item should update BC-2.17.002 E-PLUGIN-005 ... No action required for PREREQ-D delivery."
- **Why it matters:** CLAUDE.md Canonical Principle Rule 6 anti-pattern ("Pending architect review → Pick the production-grade default and write the rationale inline"). The story acknowledges a contradiction and explicitly defers via "Cross-doc gap (out-of-perimeter)" framing. Punting to "future PO-led story" with no story ID anchored is an open-ended defer (Rule 3 violation).
- **Fix-routing:** product-owner — amend BC-2.17.002 E-PLUGIN-005 to use 30s (matching ADR-023 §C4 plugin HTTP defaults). Bundle into BC-2.22.001 amendment burst.

### F-LP7-MED-002 — Task 9 ambiguity: "7.5 or new 8, query-engine=new 9" forces implementer choice with ADR-022 §B citation impact

- **Severity:** MEDIUM
- **Scope:** story spec Task 9
- **Confidence:** HIGH
- **Evidence:** Story Task 9: "Renumber subsequent steps in comments (storage=7, plugin-load=7.5 or new 8, query-engine=new 9, etc.)". ADR-022 §B canonical numbering: Step 7 = Storage, Step 8 = QueryEngine, Step 9 = MCP server. boot.rs file header references step 9 = MCP server; function name `step9_start_mcp_server`. BC-2.22.001 §Sequencing Invariant references "Steps 7-8" and "Step 9 MCP server bind — TRAFFIC GATE OPEN."
- **Why it matters:** "Renumber to 8/9/10/11/12" cascades to every existing reference to "step 9 MCP server" across BC-2.22.001 (3 cites), ADR-022 §B, boot.rs file header, STATE.md, and the `step9_start_mcp_server` function name. POLICY 4 violation. Story-writer must pick the numbering scheme in scope per Canonical Principle Rule 6.
- **Fix-routing:** story-writer — pick "step 7.5 plugin-load" to avoid cascading renumber. Document rationale in Task 9 prose.

### F-LP7-LOW-001 — BC-2.22.001 frontmatter `lifecycle_status: draft` contradicts STATE.md / BC-INDEX v4.51 promotion claim; story claims "all BCs are active"

- **Severity:** LOW (pending intent verification)
- **Scope:** BC-2.22.001 frontmatter + story bcs frontmatter comment
- **Confidence:** HIGH for the drift; LOW for severity because BC frontmatter sync is technically state-manager's domain
- **Evidence:** BC-2.22.001 frontmatter `status: draft` + `lifecycle_status: draft`. BC-INDEX row also draft. BC-INDEX history v4.51 line: "BC-2.22.001 v1.0→v1.1 (boot orchestration — first BC under SS-22)" implies promotion per ADR-021 POL-14. BC-INDEX line: "5 BCs promoted draft→active per D-319 ... BC-2.22.001 ... active_contracts 222→227". Story line: "All BCs are active".
- **Why it matters:** Either BC-2.22.001's lifecycle_status is stale (sibling-sweep gap from S-WAVE5-PREP-01 merge) OR BC-INDEX v4.51 incorrectly claimed promotion. POLICY 14 propagation gap. Story's blanket "all BCs are active" is contradicted by BC-2.22.001 frontmatter.
- **Fix-routing:** state-manager — adjudicate whether BC-2.22.001 was supposed to be promoted at S-WAVE5-PREP-01 merge per D-319. Update accordingly.

## §5 Trajectory Analysis

| Pass | Findings | Delta |
|------|----------|-------|
| 1 | 16 | — |
| 2 | 8 | −8 |
| 3 | 6 | −2 |
| 4 | 4 | −2 |
| 5 | 0 (FALSE-CLEAN) | — |
| 6 | 4 | +4 (reset; idempotency catch) |
| 7 | 7 | +3 |

Pass-6 reset broke the geometric-like decline (16→8→6→4) and pass-7 INCREASES findings count 4→7. **Anti-convergence.** The new findings (HIGH-001/002/003/004) are all systematic mis-anchors and paper-fix risks that prior passes did not catch. Fresh-context-compounding-value: pass-7 derived its own understanding rather than inheriting prior passes' assumptions:

1. Filesystem-grounded path validation (no prior pass ran Glob).
2. BC content-vs-trace audit (no prior pass grepped BC-2.22.001 for "plugin").
3. Production code-vs-spec semantic-clamp audit (host_functions.rs `.timeout(10)` clamp).

**Pass-8 prediction:** If story-writer + product-owner + state-manager close all 7 findings, expect 2-4 new findings in pass-8 from second-order effects (BC-2.22.001 amendment shapes new AC traces; sibling-site sweep may expose additional references). True convergence likely 3-4 more passes away. Streak likely to stay at 0/3 through fix-burst-6.

## §6 Verdict & Next Action

**Verdict: BLOCKED-hard** — 4 HIGH-severity findings, including a systematic path mis-anchor that has survived 6 prior passes, a BC ↔ AC semantic chain failure on 4 ACs, and a production-code TD-VSDD-059 paper-fix risk that would make AC-9's closure claim functionally inert.

**Streak: 0/3 → 0/3 (HOLD).** Pass-7 is not CLEAN.

**Recommended next dispatch: fix-burst-6** with multi-agent routing (PO → story-writer → state-manager). Bundle as one .factory/ commit per stage (3 stage commits acceptable; subjects must not use "backfill"/"Stage" theme words).

**After fix-burst-6:** dispatch pass-8 adversary. Do NOT assume convergence — pass-8 will likely surface 2-4 second-order findings from the BC amendment cascade.
