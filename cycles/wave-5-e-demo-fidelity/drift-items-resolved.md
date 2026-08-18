---
document_type: drift-items-resolved
cycle: wave-5-e-demo-fidelity
archived_by: state-manager
archived_at: 2026-06-13
archived_via: hygiene compaction (STATE v7.785→v7.786)
note: "Closed/resolved drift items removed from STATE.md Drift Items table. Open/deferred drift items remain in STATE.md."
---

# Drift Items — Resolved / Closed

> Archived from STATE.md §Drift Items at hygiene compaction (2026-06-13).
> Only CLOSED / RESOLVED / APPLIED / DONE rows are here. Open/deferred rows remain in STATE.md.

| ID | Summary | Resolution | Closed |
|----|---------|------------|--------|
| DRIFT-D850-001 **[RESOLVED D-1059]** | BC-2.16.002 missing explicit postcondition for POST-body vs GET-URL OffsetLimit pagination dispatch | RESOLVED: BC-2.16.002 v1.70 POST-vs-GET pagination clause authored | D-1059 |
| DRIFT-D943-001 **[CLOSED D-958]** | BC-3.5.002 mis-cite in prism-dtu-crowdstrike + prism-dtu-cyberint | RESOLVED: S-MAINT-W3SEC-CITE-SWEEP-001 MERGED PR #169 develop@b38c1abc | D-958 |
| DRIFT-D926-001 **[CLOSED D-1000]** | HTTP-method whitelist validation for env-resolved step.method field | RESOLVED: S-SPEC-HTTP-METHOD-VALIDATION-001 MERGED PR #172 develop@752e407a | D-1000 |
| DRIFT-ECRED-TAXONOMY-001 **[RESOLVED D-1046]** | prism-core E-CRED variant semantics misaligned with error-taxonomy.md | RESOLVED: S-MAINT-ECRED-TAXONOMY-SYNC-001 MERGED PR #175 develop@c603741d | D-1046 |
| DRIFT-ORCH-PRLEVEL-PUSH-001 **[APPLIED D-1065]** | PR-LEVEL fix-bursts MUST be pushed before re-gating | APPLIED — SESSION-HANDOFF §4 + rule #11; DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001 registered for human CLAUDE.md edit | D-1065 |

## Closed Drift Items — archived 2026-06-26 (STATE v7.995 compaction)

_Closed/resolved items moved from STATE.md §Drift Items at compaction 2026-06-26._

| DEFER-CLAUDEMD-BC216002-MISLABEL-001 | [CLOSED @09925bbe D-1178] §Conventions/§Logging/§routing now describe BC-2.16.002 Canonical Structured Event Catalog as a §Postconditions sub-section (H1 = Multi-Step Fetch Pipeline Execution). | CLOSED | 2026-06-15
| DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001 [CLOSED @09925bbe D-1178] | BC-5.39.001 § now carries the DRIFT-ORCH-PRLEVEL-PUSH-001 frozen-HEAD streak rule. | CLOSED | 2026-06-15
| DEFER-CLAUDEMD-FACTORY-PUSH-POLICY-001 [CLOSED @09925bbe D-1178] | CLAUDE.md §Git Workflow now reflects D-1066 standing push authorization for factory-artifacts. | CLOSED | 2026-06-15
| DEFER-CLAUDEMD-NONEXHAUSTIVE-COUNT-001 [RESOLVED-MECHANISM D-1178] | The human delegated CLAUDE.md-edit authority to the orchestrator (2026-06-15). develop's current count stays **60** (the ci.yml gate is a FLOOR `-lt`; develop genuinely has 60 types — premature bump would desync). Count bump now lands PER-STORY AT MERGE under orchestrator-owned CLAUDE.md authority: S-3.13 → 61, S-5.02 → cumulative (orchestrator sets cumulative value + ci.yml EXPECTED at each type-adder's merge). Lane A S-5.02 UNBLOCKED (D-1178). | MECHANISM RESOLVED — count bumps happen at merge per-story | ongoing per merge
| DRIFT-LAUNCHER-ORGSLUG-TRAVERSAL-001 [CLOSED @5cf9e77c D-1181] | `org_slug` CWE-22 traversal: `is_path_safe_slug` charset validation (`[a-zA-Z0-9][a-zA-Z0-9-]*`) added to `MultiOrgDemoConfig::from_str` at parse time; test `test_sec_001_org_slug_path_traversal_rejected` verifies rejection. LAUNCHER story v2.8 EC-010 records closure. | CLOSED | 2026-06-15
| DRIFT-S313-DUPTEST-001 [CLOSED @eafd017a D-1177] | Duplicate/misnamed test `test_BC_2_11_001_e_query_037_mcp_maps_to_invalid_params` in prism-query — implementer renamed at @eafd017a. LOCAL adversary re-pass confirmed CLEAN(strict)=YES → S-3.13 streak now 2/3. | CLOSED — see cycles/wave-5-e-demo-fidelity/blocking-issues-resolved.md | 2026-06-15

## Extracted from STATE.md on 2026-08-18 (D-2237 compact-state burst)
| ~~DRIFT-ADR049-FIGURE-001~~ [RESOLVED D-1490] | ADR-049 §Context/§Consequences figure inconsistency — **RESOLVED**: architect reconciled figures to profiling-report-sourced per-call values in ADR-049 v1.1 (2026-07-02). | DONE | RESOLVED D-1490 |
| ~~DRIFT-ADR049-D6-HASH-001~~ [RESOLVED D-1490] | ADR-049 D6 "hash validation" prose — **RESOLVED**: architect corrected §D6 in ADR-049 v1.1 (2026-07-02): metadata validation clarified (not native-code blob signing); AD-001 trust-domain boundary added. | DONE | RESOLVED D-1490 |
| ~~DRIFT-ADR031-STATUS-001~~ [RESOLVED D-1892] | ADR-031 status confirmed accepted by architect: frontmatter `status: accepted` + `version: "1.4"` (D-1892 burst OBS-1). ARCH-INDEX row updated PROPOSED v1.3 → ACCEPTED v1.4. An accepted ADR can be partially superseded by ADR-053 §D3 — resolved. | DONE | RESOLVED D-1892 |
| ~~DRIFT-DEMO-CONFIGURE-ADMINTOKEN-001~~ [CLOSED D-1795; demo-functionality, pre-existing] | `configure` subcommand POSTs `/dtu/configure` without `X-Admin-Token` → 401. RESOLVED: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 story v0.5 @1647e999 + code @26e623c9; LOCAL pass-5 DISPATCHED; pass-6 IN FLIGHT. | CLOSED D-1795 2026-07-16 | RESOLVED D-1795 2026-07-16 |
| ~~DRIFT-AUDIT-RUNBOOK-LITERALS-001~~ [CLOSED D-1795] | T13 pre-flight audit WARN-1 (2026-07-08): runbook Step 3.1a literal values mismatch scenario seed data. RESOLVED: VERDICT A CORRECT PASSTHROUGH — BC-2.02.013 v1.9→v1.10 EC-02-029 cyberint status passthrough ratified; runbook Step 3.1a literals fixed @5635cd7f. D-1795 CLOSED. | DONE | RESOLVED D-1795 2026-07-16 |
| ~~DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001~~ [CLOSED D-1647; T13 audit WARN-2; resolved at PR #219 merge (2026-07-10T01:54:58Z)] | T13 pre-flight audit WARN-2 (2026-07-08): IEQ operator on non-existent column surfaces opaque 'Internal error' — RESOLVED: FIX-IEQ-ERRPATH-001 cascade (33 passes); E-QUERY-038 gate at all 14 positions including IEQ/IIN/INE + SqlPipe + pipe-stages. BC-2.11.016 v1.25. PR #219 squash-merged develop@8ea29823 2026-07-10T01:54:58Z. | DONE | RESOLVED D-1647 |
| ~~DRIFT-AUDIT-SCRIPT-UNCOMMITTED-001~~ [CLOSED D-1647; housekeeping; resolved at PR #219 merge] | scripts/t13-preflight-audit.py extended from 62 to 70 items was uncommitted in develop working tree — RESOLVED: strict predecessor verified (46 superseded lines, zero unique content); devops discarded stale uncommitted copy at PR merge. Audit script 70 checks live on develop@8ea29823 (2026-07-10T01:54:58Z). | DONE | RESOLVED D-1647 |
| ~~DEFECT-EQUERY042-GROUPBY-DEADARM-001~~ [CLOSED D-1655; PR #220; MERGED 2026-07-10] | prism-query `check_expr_temporal_pos` only matched `Literal::RawTemporalLiteral`; `GROUP BY`/`ORDER BY` positions yielded `Literal::Timestamp` → E-QUERY-042 GroupBy/OrderBy arms never fired. FIXED: `Literal::Timestamp` arm added to GroupBy+OrderBy in `check_expr_temporal_pos` (ADR-052 §D4 v1.11 arms 6+7); 15 new tests; prism-query 1502/1502. Full cascade: LOCAL 5-pass + PR-LEVEL 3-pass ALL CLEAN(strict) on frozen 7db0b1ba. PR #220 squash-merged develop@b9cf3f9b 2026-07-10. | CLOSED — MERGED develop@b9cf3f9b D-1655 2026-07-10 | D-1655 2026-07-10 |
| ~~DEFECT-CSDEVICES-EMPTY-PIPELINE-001~~ [CLOSED D-1690; PR #221; MERGED 2026-07-11] | ROOT-CAUSED (D-1650). Option 1 RATIFIED by architect (D-1652): POST conversion; PostDeviceDetailsV2 real-API-canonical; fan_out_batch_size=100 retained. LOCAL 38-pass 3-CLEAN @bc7d8f36 + PR-LEVEL 4-pass 3-CLEAN @912e862d + security APPROVE (2 LOW fixed) + pr-reviewer APPROVE + CI 100% green. PR #221 squash-merged develop@5f1b5771 (normal squash-merge, no --admin; human-authorized). Research: `research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md`. | CLOSED — MERGED develop@5f1b5771 D-1690 2026-07-11 | D-1690 2026-07-11 |
| ~~DRIFT-AC005-DISTINCT-TRIGGER-RULING-001~~ [RESOLVED D-1829; human-decision; D-1808] | AC-005 acquisition rule RESOLVED: Human ruled literal-reading satisfied (3 distinct green pull_request run IDs at frozen HEAD d412defe per pass-25 §AC-005 Dual-Reading; distinct-trigger-events interpretation not required). PR #224 MERGED @0f9857dd. | DONE | RESOLVED D-1829 2026-07-18 |
| ~~DRIFT-ADMINTOKEN-PR225-MERGE-GATE-001~~ [RESOLVED D-1841] | PR #225 merge gate satisfied: adversary F-ADMTOK-PR22 CLEAN(strict)=yes (D-1838); security delta-confirm #2 APPROVE NEW-001 CLOSED (D-1838); pr-reviewer APPROVE (D-1839); human merge auth (D-1840); PR #225 SQUASH-MERGED @277b7844 2026-07-18T16:10:23Z (D-1841). LANE 3 CLOSED. | DONE | RESOLVED D-1841 2026-07-18 |
| ~~DRIFT-AUDIT-COVERAGE-001-RUNBOOK-ENV-BRIDGE-001~~ [RESOLVED D-1870 2026-07-18] | T13-capstone-demo-runbook.md §1.6 Pre-Flight Audit (Go/No-Go Gate) authored in v1.12 — PRISM_THREATINTEL_BASE_URL/PRISM_NVD_BASE_URL env-bridge documented; BASE_URL>PORT>default precedence; PRISM_BIN override; PIPESTATUS[0]/$? exit-code discipline; 106-check matrix; DEMO-READY YES/NO verdict. ALL T13 PRECONDITIONS CLOSED. | DONE | RESOLVED D-1870 2026-07-18 |
| ~~DRIFT-SDEMO004-INPUTS-BC32001-001~~ [RESOLVED D-1801; spec-drift; D-1795] | S-DEMO-004 story.inputs cites BC-3.2.001-multi-tenant-isolation.md — RESOLVED: story-writer corrected inputs path to BC-3.2.001-per-org-sensor-data-isolation.md during S-DEMO-004 v1.15 (D-1801 2026-07-17). | DONE | RESOLVED D-1801 2026-07-17 |
| ~~SEC-002~~ [CLOSED D-2013 as side-effect of SEC-001 fix; CWE-390 MEDIUM] | EC-009-046 load-time rejection (added for SEC-001 CWE-20/CWE-74) also closes the SEC-002 cookie-name-empty-string edge case. No separate fix required. | DONE | RESOLVED D-2013 2026-07-24 |
