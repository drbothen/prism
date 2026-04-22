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
pass: 34
previous_review: pass-33.md
cycle: phase-2-patch
novelty: MEDIUM
findings: 3
critical: 0
high: 1
medium: 2
low: 0
convergence_counter: 0 of 3
---

# Adversarial Review: Prism (Pass 34)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix from `.factory/current-cycle` — `P3PATCH` for this cycle
- `<PASS>`: Two-digit pass number
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

This cycle uses the shorthand `P3P34-A-[SEV]-[NNN]` per established pass-cycle convention.

## Scope

Full fresh-context review. Burst 34 closure verification (CAP-033 capability-name + test-vectors.md propagation + PRD NFR count). Policy 2, 6, 7, 8, 9 re-sweeps. BC-INDEX v4.10, STORY-INDEX v1.25, test-vectors v2.2 arithmetic. New-axis exploration: case-family tool consistency (CAP-022), error-taxonomy coverage for Phase 3-patch subsystems, api-surface MCP tool registry completeness vs S-5.06.

## Part A — Fix Verification (Burst 34 — 3/3 CLOSED)

| Item | Previous Severity | Status | Notes |
|------|-------------------|--------|-------|
| H-001 capabilities.md CAP-033 `action.execute`→`action.write` | HIGH | RESOLVED | 0 live `action.execute` corpus-wide; capabilities.md:53 consistent on `action.write` L2/L3/L4 |
| M-001 test-vectors.md 5 stale `execute_action` | MEDIUM | RESOLVED | Lines 46/47/48/75 → `fire_action`; line 266 → `crowdstrike_contain_host`; version v2.2; only ref = changelog line 322 |
| M-002 PRD line 471 "16"→"18" NFRs | MEDIUM | RESOLVED | PRD:471 reads "18"; nfr-catalog 18 NFR-NNN headings ✓ |

### Policy Re-sweep Summary

All 5 policies clean: Policy 2 (28 DIs all cited), Policy 6 (subsystem sync), Policy 7 (10 BC H1s match BC-INDEX), Policy 8 (bidirectional AC traces), Policy 9 (VP arithmetic).

### Arithmetic

- BC-INDEX v4.10: 195+6+2=203 ✓; subsystem sums 195 ✓
- STORY-INDEX v1.25: 75 stories; 195 BCs; 39 VPs; wave raw sum 234 ✓
- VP-INDEX v1.3: 20+11+6+2=39 ✓
- test-vectors.md v2.2; nfr-catalog 18 NFRs ✓

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

#### P3P34-A-H-001 — capabilities.md CAP-022 lists 4 non-existent case-mutation MCP tools

**Policy violated:** 6 (domain-spec ↔ implementing spec consistency)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** NEW — same structural class as CAP-033 drift (closed Burst 34) but on case-management family

**Evidence:**
- `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/capabilities.md` line 42: CAP-022 lists `create_case, update_case_status, set_disposition, add_annotation, link_alert_to_case, list_cases, get_case, case_metrics`
- Canonical per BC-2.14.003 H1: unified single `update_case` MCP tool — "Transition State, Set Disposition, Add Annotation"
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/api-surface.md:148` lists `update_case` with params `case_id, status, disposition, annotation`
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/interface-definitions.md:1507` §1.26: "Update Case Tool — update_case"
- `grep "update_case_status|set_disposition|add_annotation|link_alert_to_case"` across specs → only hit is capabilities.md:42

**Why it fails:** Implementer/reviewer consulting CAP-022 will believe Prism exposes 5 separate case-mutation tools; canonical is unified `update_case`. Test-writer crafting holdout tests against CAP-022 would write assertions for non-existent tools.

**Proposed Fix:** Replace CAP-022 tool list with canonical set: `create_case, update_case, acknowledge_alert, list_cases, get_case, case_metrics`.

### MEDIUM

#### P3P34-A-M-001 — error-taxonomy.md missing 18 rows for Phase 3-patch subsystem codes

**Policy violated:** 4 (error code anchoring integrity)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** NEW
**Location:** `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/error-taxonomy.md`

**Description:** error-taxonomy.md defines only seed codes for SS-17/18/19. BC files cite a much larger active set:

- **E-ACTION** family: E-ACTION-002..010 (9 codes, 1 registered)
  - E-ACTION-002 (BC-2.18.005), -003 (BC-2.18.001 dead-letter), -004 (BC-2.18.001 state CF write), -005 (BC-2.18.003 destination error), -006 (BC-2.18.003 unregistered action_id), -007 (BC-2.18.003 manual trigger on non-manual), -008 (BC-2.18.005 delivery fails), -009 (BC-2.18.007 credential env var missing), -010 (BC-2.18.007 missing required field)
- **E-PLUGIN** family: E-PLUGIN-004..008 (5 codes, 3 registered)
  - E-PLUGIN-004 (BC-2.17.002), -005 (BC-2.17.002), -006 (BC-2.17.003), -007 (BC-2.17.004), -008 (BC-2.17.005)
- **E-INFUSE** family: E-INFUSE-002..005 (4 codes, 1 registered)
  - E-INFUSE-002 (BC-2.19.001), -003 (BC-2.19.001), -004 (BC-2.19.001), -005 (BC-2.19.005)

Total: 18 codes cited in BCs but not registered in central taxonomy.

**Evidence:** Stories cite codes for integration-test assertions (S-4.08, S-5.06, S-6.11). error-taxonomy.md frontmatter declares itself canonical source. Missing rows force test-writers to reconstruct severity/category/retryable metadata from BC bodies.

**Proposed Fix:** Add 18 rows to error-taxonomy.md (9 E-ACTION + 5 E-PLUGIN + 4 E-INFUSE). Source messages and categories from BC bodies.

#### P3P34-A-M-002 — api-surface.md MCP Tool Registry missing 8 of 12 S-5.06 tools

**Policy violated:** 6 (architecture tool registry source-of-truth)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** NEW
**Location:** `/Users/jmagady/Dev/prism/.factory/specs/architecture/api-surface.md` Tool Registry

**Description:** S-5.06 registers 12 MCP tools. api-surface.md documents only 4 (`list_actions`, `action_status`, `fire_action`, `test_action`).

**Evidence:** Missing 8 tools:
- Always-visible: `list_infusions`, `infusion_status`, `list_plugins`, `plugin_status`
- Capability-gated: `reload_infusion` (`infusion.write`), `reload_plugin` (`plugin.write`), `create_action` (`action.write`), `delete_action` (`action.write`)

api-surface.md is architecture-level canonical tool registry. External integrators and test-harness authors rely on it. Missing entries mean tool surface is incomplete and capability-gate declarations absent.

**Proposed Fix:** Add 8 rows to api-surface.md Tool Registry: 4 in Always-Visible table, 4 in Capability-Gated table.

### LOW

None.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision

## Observations

- Tool-name triangle consistency clean for all 25 sampled tools after Bursts 33-34 closures except M-002's 8 missing rows.
- BC H1/subsystem/priority sampled 10 BCs — all verbatim match BC-INDEX.
- DI coverage clean (all 28 active DIs cited).
- Arithmetic integrity intact across BC-INDEX, STORY-INDEX, VP-INDEX.
- L-101 (interface-definitions.md missing Phase 3-patch tools) from pass-32 still deferred.

## Novelty Assessment

**Novelty: MEDIUM.** Three findings at 3 NEW axes:
- H-001: L2 ↔ L3/L4 case-family tool drift (CAP-022) — same structural class as CAP-033 (closed Burst 34), now on case-management
- M-001: L3 ↔ L3-supplement error-taxonomy coverage gap for Phase 3-patch subsystems
- M-002: L3 ↔ L4 api-surface ↔ S-5.06 registry gap (8 of 12 tools missing)

Prior passes anchored on action-delivery edge; pass-34 found case-family, error-registry, and full S-5.06 tool-surface edges.

Trajectory: 26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→4→6→2→3→**3**. Flat at 3. CRIT=0 for 22+ passes.

## Convergence Recommendation

**BLOCK at 0/3.** Mis-anchoring (H-001) never converges per adversary policy; supplement gaps (M-001, M-002) are L3-canonical drift.

**Burst 35 scope:**
1. **H-001** (architect): capabilities.md CAP-022 — replace tool list with canonical set
2. **M-001** (PO): error-taxonomy.md — add 18 rows for E-ACTION/E-PLUGIN/E-INFUSE
3. **M-002** (architect): api-surface.md — add 8 Tool Registry rows

Expected pass-35: 1/3 advance if Burst 35 clean.

## Relevant Files

- `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/capabilities.md` (H-001 target line 42)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.14.003-update-case-tool.md` (H-001 canonical)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/api-surface.md` (H-001 + M-002)
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/error-taxonomy.md` (M-001 target)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.18.001/003/005/007*.md` (M-001 E-ACTION sources)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.17.002/003/004/005*.md` (M-001 E-PLUGIN sources)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.19.001/005*.md` (M-001 E-INFUSE sources)
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.06-action-infusion-tools.md` (M-002 source — 12 tools)
