---
document_type: adversarial-review
level: ops
version: "1.0"
status: findings-open
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
cycle: phase-2-patch
pass: 44
previous_review: pass-43.md
novelty: MEDIUM — 3 HIGH findings novel (Burst 44 frontmatter/changelog sync failure on DTU stories; health_check→check_sensor_health drift across 5 stories surviving 44 passes; prism://health/{client_id} vs prism://sensors/health resource URI drift BC↔architecture)
findings_total: 5
findings_crit: 0
findings_high: 3
findings_med: 1
findings_low: 1
findings_observational: 0
previous_pass: 43
convergence_counter: 0
date: 2026-04-19
---

# Adversarial Review: Prism (Pass 44)

## Finding ID Convention

Finding IDs use the format: `P3P44-A-{SEV}-NNN`

- `P3P44`: Cycle prefix (Phase-2-Patch, Pass 44)
- `A`: Part A segment identifier
- `{SEV}`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `NNN`: Three-digit sequence within this pass

---

## Part A — Fix Verification

Verifying pass-43 closures (Burst 44):

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| P3P43-A-HIGH-001 | HIGH | RESOLVED | `trigger_action` → `fire_action` rename confirmed in S-6.11:240, S-6.12:268, S-6.13:280-281; all 3 stories bumped with changelog rows |
| P3P43-A-HIGH-002 | HIGH | RESOLVED | BC-2.10.002 rewritten to v2.2; postcondition, heading, and inventory now defer to api-surface.md as SoT; stale 15-tool count removed |
| P3P43-A-HIGH-003 | HIGH | RESOLVED | `get_credential` → `credential_status` at S-1.07:46; S-1.07 bumped to v1.3 |
| P3P43-A-MED-001 | MEDIUM | RESOLVED | STATE.md internal contradictions resolved; convergence counter synced; headings updated |
| P3P43-A-LOW-001 | LOW | RESOLVED | BC-2.03.005 EC-03-013 reframed as resolution-layer byte handling; bumped to v1.2 |

---

## Part A — Sweep Results (15 dimensions)

All 15 sweep dimensions applied:

1. **VP-INDEX arithmetic** — CLEAN: 20+11+6+2=39; 32+7=39 verified.
2. **BC-INDEX subsystem totals** — CLEAN: 195+6+2=203 verified.
3. **STORY-INDEX Wave Summary** — CLEAN: 0+69+30+28+45+51+15=238; 75 stories; 195 unique active BCs verified.
4. **API-surface Mermaid 28/24 vs actual row counts** — CLEAN: 28 always-visible + 24 capability-gated = 52 total confirmed.
5. **Error-taxonomy 190 codes** — CLEAN: 190 error codes present.
6. **Policy 6 BC subsystem anchoring (sampled)** — CLEAN: BC-2.08.006, BC-2.10.008, BC-2.08.005 verified.
7. **Policy 7 BC H1 title sync (sampled)** — CLEAN: BC-2.08.006, BC-2.10.008 verified.
8. **Policy 8 bidirectional (sampled)** — CLEAN: S-5.01, S-5.04 sampled and confirmed.
9. **Tool-naming `trigger_action` in live prose** — CLEAN: 0 hits confirmed after Burst 44 fixes.
10. **Tool-naming `set_credential` only in changelogs** — CLEAN: no active prose references found.
11. **Tool-naming `execute_action` only in changelogs** — CLEAN: no active prose references found.
12. **Tool-naming `test_infusion`, `update_case_status`, `set_disposition`, `add_annotation`, `link_alert_to_case` absent** — CLEAN: zero hits confirmed.
13. **AI-opaque credentials** — CLEAN: BC-2.03.005 v1.2, entities.md, api-surface all consistent.
14. **DI citations sampled** — CLEAN: DI-018/019/022/023/026/032 all have BC citations.
15. **BC-INDEX→files arithmetic** — CLEAN: 195 active BCs match index arithmetic.

---

## Sweeps Clean (additional spot-checks)

- `trigger_action` in live prose = 0 ✓
- `get_credential` in live prose = 0 ✓
- `set_credential` in live prose = 0 (only changelog) ✓
- `execute_action` in live prose = 0 (only changelog) ✓
- BC-2.10.002 v2.2 structural rewrite clean ✓
- BC-2.03.005 v1.2 EC-03-013 reframe clean ✓
- S-1.07 Task 1 canonical list (configure_credential_source, credential_status, delete_credential, list_credentials) clean ✓
- AI-opaque credentials semantics aligned ✓
- BC-INDEX 195+6+2=203 ✓
- STORY-INDEX Wave sum 238 ✓
- VP-INDEX arithmetic 39 ✓

---

## Part B — New Findings

### CRITICAL

None.

### HIGH

#### P3P44-A-HIGH-001: S-6.11/S-6.12/S-6.13 frontmatter version NOT bumped (Burst 44 regression)

- **Severity:** HIGH
- **Category:** spec-fidelity | policy-violation
- **Policy:** Policy 2 (version bump required when content changes)
- **Location:** S-6.11-dtu-slack.md frontmatter, S-6.12-dtu-pagerduty.md frontmatter, S-6.13-dtu-jira.md frontmatter
- **Description:** All three DTU stories have `version: "1.1"` in their frontmatter despite containing a v1.2 changelog row (added in Burst 44 for the `trigger_action` → `fire_action` rename). The frontmatter version contradicts the changelog and the STATE.md claim that these stories were updated to v1.2.
- **Evidence:** STATE.md POST-BURST-44 checkpoint lists S-6.11/6.12/6.13 as v1.2 bumped. The changelog rows in each story confirm content was changed. Frontmatter `version: "1.1"` was not updated to match. This is a direct Burst 44 regression: the rename was applied to content but the version sync step was skipped.
- **Proposed Fix:** Update frontmatter `version` from `"1.1"` to `"1.2"` in S-6.11, S-6.12, and S-6.13. No additional content changes needed — the changelog rows already exist.

---

#### P3P44-A-HIGH-002: `health_check` tool name drift in 5 stories (canonical: `check_sensor_health`)

- **Severity:** HIGH
- **Category:** interface-gaps | contradictions
- **Policy:** Policy 7 (BC H1 title/body consistency); Policy 4 (spec-fidelity)
- **Location:** S-5.01-mcp-bootstrap.md, S-5.03-resources-prompts.md, S-5.04-sensor-health.md, S-5.06-action-infusion-tools.md (active prose); possibly S-5.01 Task 3 also uses old name
- **Description:** Multiple stories reference `health_check` as the tool name for sensor health checking. The canonical name is `check_sensor_health` per api-surface.md:34, api-surface.md:117, and BC-2.08.005:26. This drift has survived all 44 prior passes because the sweep suite did not include `health_check` as a stale-name candidate.
- **Evidence:** api-surface.md:34 and :117 both use `check_sensor_health`. BC-2.08.005:26 precondition tool list includes `check_sensor_health`, not `health_check`. Note: S-2.01's `RocksDbBackend.health_check()` is an internal method, NOT the MCP tool — that is correct and must not be renamed. The drift is MCP tool layer only.
- **Proposed Fix:** Replace `health_check` → `check_sensor_health` at all affected MCP tool layer sites in S-5.01, S-5.03, S-5.04, S-5.06. Bump each story version with changelog rows. Do NOT rename S-2.01's internal `RocksDbBackend.health_check()` method.

---

#### P3P44-A-HIGH-003: Resource URI drift — BC-2.08.006 and BC-2.10.008 use `prism://health/{client_id}` (templated); api-surface.md uses `prism://sensors/health` (global)

- **Severity:** HIGH
- **Category:** contradictions | interface-gaps
- **Policy:** Policy 4 (spec-fidelity); Policy 7 (cross-document consistency)
- **Location:** BC-2.08.006-health-mcp-resource.md:26, BC-2.10.008-mcp-resources.md:32 vs api-surface.md:207, api-surface.md:245
- **Description:** BC-2.08.006 and BC-2.10.008 define the health MCP resource URI as `prism://health/{client_id}` (a URI template with per-client parameterization). api-surface.md:207 and :245 define it as `prism://sensors/health` (global, no template parameter). These are contradictory: one implies per-client scoping, the other implies global scope.
- **Evidence:** Per deployment model (D-003: per-analyst stdio), each MCP server process serves exactly one analyst. There is no multi-tenant context in which `{client_id}` parameterization adds value within a single process. The global `prism://sensors/health` URI in api-surface.md is architecturally consistent with per-analyst-stdio deployment. The BC templated form introduces unnecessary URI-level complexity and contradicts the authoritative api-surface spec.
- **Resolution Options:**
  - **Case A (recommended):** Reconcile to global `prism://sensors/health` in BCs (matches api-surface.md; consistent with per-analyst-stdio deployment). BCs follow api-surface as SoT.
  - **Case B:** Update api-surface.md to use `prism://health/{client_id}` template and justify per-sensor scoping. Requires AD-003 annotation.
- **Proposed Fix:** Apply Case A: update BC-2.08.006:26 and BC-2.10.008:32 URI template to `prism://sensors/health` (static); bump both BCs with changelog rows noting URI reconciliation.

---

### MEDIUM

#### P3P44-A-MED-001: S-5.01 Task 3 tool count stale (lists 34; canonical api-surface has 52); AC-2 "35+ entries" stale

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Policy:** Policy 4
- **Location:** S-5.01-mcp-bootstrap.md Task 3, AC-2
- **Description:** S-5.01 Task 3 lists 34 tools explicitly. The canonical api-surface.md tool registry has 52 tools (28 always-visible + 24 capability-gated). AC-2 states "35+ entries" which is also stale. BC-2.10.002 v2.2 now correctly defers to api-surface as SoT rather than enumerating a count; S-5.01 should adopt the same framing to avoid recurrence.
- **Evidence:** api-surface.md:113-140 (28 always-visible) + api-surface.md:146-169 (24 capability-gated) = 52 total. BC-2.10.002 v2.2 rewrite in Burst 44 established the correct pattern: cite the registry, not a hardcoded count. S-5.01 still hardcodes the old 34-tool list.
- **Proposed Fix:** Update S-5.01 Task 3 to reference api-surface.md as the canonical tool registry and remove the hardcoded 34-tool list (or reduce to a representative sample with a "see api-surface.md for full list" note). Update AC-2 acceptance criterion to align with actual 52-tool count or to cite api-surface as SoT. Bump S-5.01 with changelog row.

---

### LOW

#### P3P44-A-LOW-001: STATE.md at 200 lines (guideline requires <200); redundant counter statement on two lines

- **Severity:** LOW
- **Category:** ops-hygiene
- **Policy:** STATE.md <200-line dispatch rubric
- **Location:** STATE.md (full file)
- **Description:** STATE.md is at exactly 200 lines, at the guideline boundary. Two lines redundantly state the convergence counter reset (frontmatter line 67 and body line 176 both say "0 of 3 RESET"). Additionally, the STATE.md claim that S-6.11/6.12/6.13 are at v1.2 is incorrect until HIGH-001 is resolved (frontmatter is still v1.1).
- **Evidence:** `wc -l .factory/STATE.md` = 201 lines (including trailing newline). Line 67 and line 176 both carry the RESET annotation. Line 192 in the checkpoint lists S-6.11/6.12/6.13 as "v1.2 (↑ frontmatter sync)" — this claim is false until HIGH-001 is closed.
- **Proposed Fix:** After closing HIGH-001 through MED-001: trim STATE.md to <200 lines by collapsing redundant counter statement on line 176 into the frontmatter value (remove the separate body paragraph or reduce to a one-liner reference). Update the checkpoint version table to accurately reflect post-Burst-45 state.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 3 |
| MEDIUM | 1 |
| LOW | 1 |
| OBSERVATIONAL | 0 |
| **Total** | **5** |

**Overall Assessment:** pass-with-findings
**Convergence:** FINDINGS_REMAIN — convergence counter stays at 0/3; Burst 45 remediation required before pass-45 dispatch
**Readiness:** Not ready for Phase 3 gate; 3 consecutive clean passes required from zero

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 44 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (5 / (5 + 0)) — all findings genuinely new |
| **Median severity** | HIGH (3 HIGH + 1 MED + 1 LOW) |
| **Trajectory** | 29→24→21→7→4→3→2→CLEAN→26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→4→6→2→3→3→12→4→3→3→8→4→3→0→5→**5** |
| **Verdict** | FINDINGS_REMAIN — counter stays 0/3; DTU frontmatter sync failure (Burst 44 regression) + health_check name drift (survived 44 passes) + resource URI BC↔architecture contradiction; Burst 45 required |
