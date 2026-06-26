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
