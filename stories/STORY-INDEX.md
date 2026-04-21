---
document_type: story-index
level: "L4"
version: "v1.41"
status: draft
producer: story-writer
timestamp: 2026-04-21T00:00:00
phase: 2
total_stories: 75
total_bcs_covered: 200
total_vps_assigned: 62
---

# Prism Phase 3 Story Index

## Overview

Phase 3 decomposes the Prism platform into 75 implementation stories spanning 7 parallel
waves. Stories are organized by crate and ordered topologically so that no story begins
before its dependencies are complete.

- **Total stories:** 75 (62 post-Burst-2.75 + 14 new DTU stories: S-6.06 rescoped + S-6.07–S-6.19)
- **Total waves:** 7 (Wave 0 expanded to 16 stories: devops + DTU infrastructure)
- **BCs covered:** 200 (all active BCs per BC-INDEX.md v4.13; 200 active contracts; BC-2.12.011/012 retired in Burst 4b)
- **VPs assigned:** 62 (26 Kani proofs, 28 proptests, 6 fuzz targets, 2 integration tests)
- **Note:** The 7 osquery-inspired stories (S-2.08, S-3.08 through S-3.13) have 0 formal BCs at this stage — they are enhancements derived from the osquery synthesis review.
- **Phase 3 patch Burst 1 (2026-04-16):** Added 5 new stories (S-0.01, S-0.02, S-6.04, S-6.05, S-6.06) and 2 scope expansions (S-6.01 subcommand dispatch, S-2.01 action_state CF) to close gaps identified in the consistency-validator audit.
- **Phase 3 patch Burst 2 (2026-04-16):** Added 4 new stories (S-5.07, S-5.08, S-5.09, S-5.10). 3 scope expansions (S-5.05 scope boundary, S-1.14 BC anchors + infusion_cache CF, S-4.03 IOC file loading). 5 retroactive BC anchor updates (S-1.15 → BC-2.17.*, S-4.08 → BC-2.18.*, S-4.07 → BC-2.14.012 gate resolved, S-4.06 → BC-2.14.013, S-1.14 → BC-2.19.*).
- **Phase 3 patch Burst 2.75 (2026-04-16):** Surgical traceability anchor pass. 4 new BCs committed by product-owner anchored to implementing stories: BC-2.08.008/009 → S-5.08, BC-2.05.011 → S-5.10, BC-2.13.014 → S-4.03. VP-039 (Kani, watermark monotonicity) → S-5.10. All hedge/TBD language removed from the 3 anchored stories. No new stories; story count remains 62.
- **Phase 3 patch Burst 4b (2026-04-16):** Adversary pass 1 fixes. BC count drift corrected in STORY-INDEX Full Story List (S-1.14/15/4.06/4.08). Duplicate BC table headers removed from S-5.08/5.10. BC miswirings corrected in S-5.08, S-5.10, S-6.04. S-1.14 subsystems field updated to [SS-16, SS-19]. Wave BC sums recomputed from scratch (239 raw, 193 unique). S-6.06 endpoints realigned to dtu-assessment.md §3.1–3.4. S-4.07 BC file path corrected. S-2.01 event_buffer/plugin_state rationale added. DTU `dtu = []` workspace feature defined in S-0.02. Layer -1 rationale documented. STORY-INDEX version: 1.4 → 1.5. No new stories; story count remains 62.
- **Phase 3 patch Burst 5b-SW-A (2026-04-16):** Added 13 new DTU stories (S-6.07 through S-6.19) and rescoped S-6.06 from `prism-dtu` (Wave 6) to `prism-dtu-common` (Wave 0). Story count: 62 → 75. VP-033 and VP-036 now anchor to S-6.07 (CrowdStrike clone) as the primary integration-test vehicle for audit and detection BCs.
- **Phase 3 patch Burst 5b-SW-B (2026-04-16):** Adversary pass 2 story-writer fixes. P3P2-C-002: removed retired BC-2.12.011/012 rows from BC Traceability Matrix (retired in Burst 4b; replaced by SS-18 BCs). P3P2-C-003/M-001: S-4.08 frontmatter + AC-8 updated to remove retired BCs and trace to BC-2.18.006. P3P2-H-001: multi-story entries added (S-6.04 to BC-2.03.*, S-6.05 to BC-2.15.001/002/005). P3P2-H-006/M-004: BC-INDEX version pin v4.1 → v4.3, 193 → 192. P3P2-M-002: Wave 4 BC count 47 → 45; raw sum 239 → 237. P3P2-M-006: Wave 5 crate column normalized. P3P2-L-001: Layer -1 renumbered to Layer 0 (devops). P3P2-L-005: S-5.10 cross-crate note added. STORY-INDEX v1.5 → v1.6. Story count 62 → 75.
- **Phase 3 patch Burst 6b (2026-04-16):** Adversary pass 3 story-writer fixes. P3P3-C-001/M-004: VP-033 anchor cleaned from S-2.04, S-6.06; VP-036 anchor cleaned from S-4.04, S-6.06; both VPs now anchor to S-6.07 only. P3P3-H-001: 13 DTU stories subsystems field updated: crate names → SS-IDs (SS-01, SS-08, SS-18, SS-19). P3P3-H-002: R-DTU risk mitigations anchored: R-DTU-005 → S-6.06, R-DTU-008 → S-6.13, R-DTU-009 → S-6.15, R-DTU-010 → S-6.18, R-DTU-011 → S-6.19. P3P3-H-005: S-6.19 log-forwarder crate corrected: prism-operations → prism-mcp. P3P3-M-002: DTU blocks edges added (option B, human-approved): sensor DTUs → S-3.02; action DTUs → S-4.08, S-5.06; infusion DTUs → S-1.14, S-5.06; log-forward DTUs → S-5.09. P3P3-M-003/L-005: S-6.06 filename: dtu-sensor-stubs.md → dtu-common.md. P3P3-M-006: Topological layers integerized (Option B: parallel Test Track dimension). P3P3-M-007: Wave 1 parenthetical: 5 → 3 stories with 0 BCs. P3P3-L-001: S-6.* namespace collision documented. Fidelity taxonomy parenthetical sweep: L[0-4] ([qualifier]) form applied to all 14 DTU story titles and headings. STORY-INDEX v1.6 → v1.7. No new stories; story count remains 75.
- **Phase 3 patch Burst 13 (2026-04-17):** Adversary pass 12 story-level fixes (semantic anchoring audit). S-5.08: removed over-claimed BC-2.10.001/002/003/006/010 from `bcs:` frontmatter — those BCs are implemented by S-5.01 and consumed here via `depends_on: [S-5.01]`; S-5.08 only implements BC-2.08.008 and BC-2.08.009. S-5.08 AC BC traces updated from BC-2.10.* to BC-2.08.008/009. S-5.08 subsystems: [SS-08] → [SS-08, SS-10] (add MCP Interface). S-1.02 subsystems: [SS-14] → [SS-03, SS-11, SS-14] (add Credential Management for CredentialName newtype; add Query Execution for CursorRegistry). S-3.05 subsystems: [SS-07] → [SS-07, SS-11] (add Query Execution for cursor/cache execution-layer ownership). STORY-INDEX v1.9 → v1.10. No new stories; story count remains 75.
- **Phase 3 patch Burst 14 (2026-04-17):** Burst 14: add SS-12 to S-1.02 subsystems (ScheduleId/Scheduler concern previously missing from frontmatter). S-1.02 subsystems: [SS-03, SS-11, SS-14] → [SS-03, SS-11, SS-12, SS-14]. Story body line 36 already cited SS-12 (Scheduling, BC-2.12.*) as a consumer; line 110 defines `ScheduleId(Uuid)` which is a scheduler concern. Frontmatter now consistent with body. STORY-INDEX v1.10 → v1.11. No new stories; story count remains 75.
- **Phase 3 patch Burst 15 (2026-04-17):** P3P14-A3-001: BC-2.10.004 title corrected in S-5.02 BC table: "client_id Parameter on Every Tool (Stateless Model)" → "Client Scoping on Every Tool (Stateless Model)". P3P14-A8-001: BC-INDEX version pins updated v4.3 → v4.5 (two occurrences in STORY-INDEX overview and wave summary). STORY-INDEX v1.11 → v1.12. No new stories; story count remains 75.
- **Phase 3 patch Burst 20 (2026-04-17):** P3P19-A10-001 BC-INDEX version pin v4.5 → v4.6 (lines 24, 63). P3P19-A5-001 BC Traceability Matrix multi-story mapping added for BC-2.05.001/002/003/004/006/008: S-2.04 → S-2.04, S-5.10 (per S-5.10 frontmatter ownership). STORY-INDEX v1.12 → v1.13. No new stories; story count remains 75.
- **Phase 3 patch Burst 21 (2026-04-17):** Un-retired BCs BC-2.04.014, BC-2.06.009, BC-2.10.005 (per user Option A, Config-Reload semantics restored). Story anchors assigned: BC-2.04.014 → S-5.01, BC-2.06.009 → S-5.05, BC-2.10.005 → S-5.01. BC Traceability Matrix +3 rows. S-5.01 bcs: +BC-2.04.014, +BC-2.10.005; S-5.05 bcs: +BC-2.06.009. Active BC count 192 → 195 (pending state-manager). STORY-INDEX v1.13 → v1.14. No new stories; story count remains 75.
- **Phase 3 patch Burst 22 (2026-04-17):** P3P21-A7-H-001/002/003 — S-5.01 body BC table + ACs for BC-2.04.014, BC-2.10.005; S-5.05 body BC table + AC for BC-2.06.009. P3P21-A2-M-002 — BC-INDEX version pins v4.6→v4.7; 192→195 at lines 24/65. STORY-INDEX v1.14 → v1.15. No new stories; story count remains 75.
- **Phase 3 patch Burst 23 (2026-04-17):** P3P22-A3-H-001 Wave 5 BC count 50→48; raw sum 237→235 (propagation of Burst 21 un-retire additions). P3P22-A8-H-002 S-5.08 Full Story List BCs column 7→2 (Burst 13 de-over-claim propagation). P3P22-A2-H-003 S-3.01 body BC table +BC-2.11.006 + AC-8 trace citation. STORY-INDEX v1.15 → v1.16. No new stories; story count remains 75.
- **Phase 3 patch Burst 25 (2026-04-18):** P3P24-A-H-001 S-5.10 AC trace re-anchor to BC-2.05.011: 4 ACs (AC-2, AC-3, AC-4, AC-6) rewired from BC-2.05.001/002/003/004 → BC-2.05.011 postcondition/error-case names (closing finding P3P24-A-H-001, Policies 4 + 8). Frontmatter + body BC table unchanged (already correct from Burst 2.75). STORY-INDEX v1.16 → v1.17. No new stories; story count remains 75.
- **Phase 3 patch Burst 26 (2026-04-19):** P3P25-A-H-001 total_vps_assigned 40→39 (already closed in Burst 26 story-writer pass, recorded here). P3P25-A-M-001/002 S-5.09 BC-2.10.006 removed from frontmatter (stdio mis-anchor; BC-2.10.006 correctly anchored to S-5.01); S-5.09 BCs column 2→1; Wave 5 raw BC count 48→47; raw sum 235→234. P3P25-A-H-004 S-4.03 BC body titles restored. P3P25-A-H-005 S-5.10 +4 ACs. P3P25-A-M-003 S-4.03 +AC-9 for BC-2.13.014. P3P25-A-M-004/L-001 S-4.06 BC titles + burst marker removal. P3P25-A-M-005 S-4.01 BC-2.12.010 title. BC-INDEX version pins v4.7→v4.8. STORY-INDEX v1.17 → v1.18. No new stories; story count remains 75. Unique active BCs unchanged at 195 (BC-2.10.006 still covered by S-5.01).
- **Phase 3 patch Burst 27 (2026-04-19):** Burst 27 closure of 12 pass-26 findings — systematic Wave-1-5 BC title sweep across S-1.08/.09/.14/.15, S-3.02, S-4.02/.03/.04/.05/.06/.07/.08; S-4.03 AC-9 + Task 8a reconciled to BC-2.13.014 SoT; 4 stale [PHASE 3 PATCH] markers stripped; S-4.08 table schema converted to canonical 3-column form. STORY-INDEX v1.18 → v1.19. No new stories; story count remains 75. Frontmatter unchanged: total_bcs_covered=195, total_vps_assigned=39.
- **Phase 3 patch Burst 28 (2026-04-19):** Burst 28 — S-1.14/S-1.15 BC table schema normalized to 2-col canonical; S-1.09 E-FLAG-002→E-FLAG-003 (token expiry code correction); S-2.01/.02 + S-3.03/.04/.05/.07 BC title drift sweep (19 fixes); S-6.01 marker strip. total_bcs_covered and total_vps_assigned unchanged (no frontmatter BC additions or removals). STORY-INDEX v1.19 → v1.20.
- **Phase 3 patch Burst 29 (2026-04-19):** Burst 29 — updated BC-INDEX version pins from v4.8 to v4.10 (pass-28 Observation 1 follow-up). No count changes; purely propagation metadata sync. STORY-INDEX v1.20 → v1.21.
- **Pass-80 F80-002 follow-on (2026-04-21):** BC count sync after CAP-035 re-anchor. BC-INDEX version pins v4.10 → v4.12; active BC count 195 → 200 (lines 24, 73). STORY-INDEX v1.32 → v1.33.
- **Pass-87 remediation F87-002 completion (2026-04-21):** VP-025 relocated from S-3.04 → S-3.05. Full Story List: S-3.04 VPs VP-012,013,025,037 → VP-012,013,037; S-3.05 VPs -- → VP-025. BC Traceability Matrix BC-2.07.005 already correctly mapped to S-3.05 (no change needed). STORY-INDEX v1.34 → v1.35.
- **Pass-89 F89-005 (2026-04-21):** S-5.10 Full Story List BC count 7 → 8 (BC-2.15.004 now anchored to S-5.10 per VP-056 proptest ownership). BC Traceability Matrix BC-2.15.004 row S-2.02 → S-2.02, S-5.10. Wave 5 BC count 55 → 56; wave raw sum 242 → 243. STORY-INDEX v1.36 → v1.37.

Every story contains: narrative, behavioral contracts table, numbered tasks, acceptance
criteria (Given/When/Then), verification properties, and notes. No story exceeds 5
estimated days. No story's estimated context exceeds 30% of the implementing agent's
context window.

---

## Wave Summary

| Wave | Crates | Stories | BCs | Theme |
|------|--------|---------|-----|-------|
| 0 | devops, prism-dtu-common, prism-dtu-threatintel, prism-dtu-nvd | 5 | 0 (infra) | Developer + Test Infrastructure (threat-intel DTUs: must precede wave-1 S-1.14) |
| 1 | prism-core, prism-ocsf, prism-credentials, prism-security, prism-spec-engine, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint, prism-dtu-armis | 19 | 69 (raw; 3 stories with 0 BCs) | Foundation + Pure Domain + Sensor DTUs (precede wave-3 consumers) |
| 2 | prism-storage, prism-audit, prism-sensors, prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-jira | 11 | 30 | Infrastructure + Adapters + Action DTUs (precede wave-4 S-4.08) |
| 3 | prism-query, prism-dtu-datadog, prism-dtu-splunk-hec, prism-dtu-elasticsearch, prism-dtu-otlp | 17 | 28 | Query Engine (incl. write ops + osquery enhancements) + Log-Forwarding DTUs (precede wave-5 S-5.09) |
| 4 | prism-operations | 8 | 45 | Operations |
| 5 | prism-mcp, prism-audit | 10 | 56 | MCP Server + Config + Diagnostics + Log Forwarding + Audit Forwarding |
| 6 | prism-bin | 5 | 15 | Binary + E2E |

Wave 0: devops (S-0.01, S-0.02, no deps) + DTU common (S-6.06, depends on S-0.02) + threat-intel DTUs (S-6.14, S-6.15, depend on S-6.06). S-6.14/S-6.15 must be wave 0 because they block wave-1 S-1.14 (infusion spec loading).
Wave 1: product foundation stories (S-1.01–S-1.15, no product deps beyond S-1.01) + sensor DTUs (S-6.07–S-6.10, depend on S-6.06 wave-0). Sensor DTUs must precede wave-3 consumers S-3.02, S-3.06, S-3.07.
Wave 2: infrastructure+adapters (S-2.01–S-2.08, depend on wave-1) + action DTUs (S-6.11–S-6.13, depend on S-6.06 wave-0). Action DTUs must precede wave-4 S-4.08 and wave-5 S-5.06.
Wave 3: query engine (S-3.01–S-3.13, depend on wave-2) + log-forwarding DTUs (S-6.16–S-6.19, depend on S-6.06 wave-0). Log-forwarding DTUs must precede wave-5 S-5.09.
Waves 4-6 follow in order. All dependency chains are acyclic (validated by topological sort below).
Per-wave BC counts are raw story-BC assignments (sum=243 across all waves: 0+69+30+28+45+56+15).
Some BCs appear in multiple stories (e.g., BC-2.04.001 → S-1.08 AND S-3.07; BC-2.16.001 → S-1.11 AND S-1.13),
so the raw sum exceeds the unique count. Unique active BCs = 200 (per BC-INDEX.md v4.13, 200 active contracts).
Note: DTU stories have 0 BCs. Per user directive Option 2 (DTU-first), product stories that require DTU
clones as test fixtures now have explicit depends_on edges to their DTU prerequisites. DTU stories are
distributed across waves 0-3 based on their earliest product consumer's wave.

**NOTE on wave vs. topological scheduling:** Wave assignments are grouped by crate boundary
for organizational clarity. The topological sort (below) shows that some stories can start
earlier than their wave number suggests — e.g., S-3.01 (Wave 3) and S-2.01 (Wave 2) are
both in topological Layer 2, meaning they can begin as soon as S-1.01 (Layer 1) completes. Teams
pursuing maximum parallelism should schedule by topological layer, not wave number.

---

## Full Story List

| Story ID | Title | Crate | BCs | VPs | Days | Depends On |
|----------|-------|-------|-----|-----|------|------------|
| S-0.01 | CI/CD Pipeline and Release Workflow | devops | 0 | -- | 4 | -- |
| S-0.02 | Developer Toolchain Bootstrap | devops | 0 | -- | 3 | -- |
| S-6.06 | DTU Common Infrastructure [W0] | prism-dtu-common | 0 | -- | 4 | S-0.02 |
| S-6.14 | DTU for Threat Intel Aggregator — L2 (stateful) [W0] | prism-dtu-threatintel | 0 | -- | 3 | S-6.06 |
| S-6.15 | DTU for NVD/NIST CVSS API — L2 (stateful) [W0] | prism-dtu-nvd | 0 | -- | 3 | S-6.06 |
| S-6.07 | DTU for CrowdStrike Falcon API — L4 (adversarial) [W1] | prism-dtu-crowdstrike | 0 | VP-033,VP-036 | 5 | S-6.06 |
| S-6.08 | DTU for Claroty xDome API — L4 (adversarial) [W1] | prism-dtu-claroty | 0 | -- | 4 | S-6.06 |
| S-6.09 | DTU for Cyberint API — L2 (stateful) [W1] | prism-dtu-cyberint | 0 | -- | 3 | S-6.06 |
| S-6.10 | DTU for Armis Centrix API — L2 (stateful) [W1] | prism-dtu-armis | 0 | -- | 3 | S-6.06 |
| S-6.11 | DTU for Slack Webhook API — L2 (stateful) [W2] | prism-dtu-slack | 0 | -- | 2 | S-6.06 |
| S-6.12 | DTU for PagerDuty Events API v2 — L3 (behavioral) [W2] | prism-dtu-pagerduty | 0 | -- | 4 | S-6.06 |
| S-6.13 | DTU for Jira REST API v3 — L3 (behavioral) [W2] | prism-dtu-jira | 0 | -- | 5 | S-6.06 |
| S-6.16 | DTU for Datadog Logs API — L2 (stateful) [W3] | prism-dtu-datadog | 0 | -- | 2 | S-6.06 |
| S-6.17 | DTU for Splunk HTTP Event Collector — L2 (stateful) [W3] | prism-dtu-splunk-hec | 0 | -- | 2 | S-6.06 |
| S-6.18 | DTU for Elasticsearch Bulk API — L2 (stateful) [W3] | prism-dtu-elasticsearch | 0 | -- | 3 | S-6.06 |
| S-6.19 | DTU for OTLP/HTTP Log Ingestion — L2 (stateful) [W3] | prism-dtu-otlp | 0 | -- | 3 | S-6.06 |
| S-1.01 | Foundational Types (TenantId, PrismError, StorageDomain) | prism-core | 0 | VP-001 | 2 | -- |
| S-1.02 | Entity Types and State Machines | prism-core | 0 | VP-005,006,011,029,051,055,057 | 2 | S-1.01 |
| S-1.03 | Capability Resolution Engine | prism-core | 0 | VP-002,003,004 | 2 | S-1.01 |
| S-1.04 | OCSF Schema Loading and DynamicMessage | prism-ocsf | 5 | VP-016,022 | 3 | S-1.01 |
| S-1.05 | OCSF Field Mapping and Normalization | prism-ocsf | 7 | VP-017 | 3 | S-1.04 |
| S-1.06 | Credential Store Trait and Backends | prism-credentials | 7 | VP-034,035 | 3 | S-1.01,S-1.02 |
| S-1.07 | Credential CRUD, Resolution, and Security | prism-credentials | 5 | -- | 2 | S-1.06 |
| S-1.08 | Feature Flags (P0 Core) | prism-security | 8 | VP-020 | 3 | S-1.01,S-1.03 |
| S-1.09 | Confirmation Tokens (P1) | prism-security | 6 | VP-007,008,009,010 | 2 | S-1.08 |
| S-1.10 | Prompt Injection Defense | prism-security | 8 | VP-024,038 | 2 | S-1.01 |
| S-1.11 | Spec Loading and Pipeline Execution | prism-spec-engine | 5 | VP-023,VP-059 | 3 | S-1.01 |
| S-1.12 | Hot Reload and Runtime Management | prism-spec-engine | 5 | VP-032 | 2 | S-1.11 |
| S-1.13 | Sensor Spec Write Endpoints | prism-spec-engine | 2 | -- | 2 | S-1.11 |
| S-1.14 | Infusion Spec Loading and UDF Registration | prism-spec-engine | 5 | VP-048,VP-049 | 3 | S-1.11,S-6.14,S-6.15 |
| S-1.15 | WASM Plugin Runtime | prism-spec-engine | 6 | VP-040,VP-041,VP-042,VP-043 | 3 | S-1.11 |
| S-2.01 | RocksDB Initialization and Domain Operations | prism-storage | 3 | -- | 3 | S-1.01 |
| S-2.02 | Audit Buffer and Watchdog | prism-storage | 5 | VP-058 | 2 | S-2.01 |
| S-2.03 | Decorators and Internal Tables | prism-storage | 3 | -- | 2 | S-2.01,S-1.02 |
| S-2.04 | Audit Entry Construction and Compliance | prism-audit | 6 | -- | 3 | S-2.01,S-2.02 |
| S-2.05 | Specialized Audit Events | prism-audit | 4 | -- | 1 | S-2.04 |
| S-2.06 | DataSource Trait and Auth Patterns | prism-sensors | 4 | -- | 3 | S-1.06,S-1.11 |
| S-2.07 | Per-Sensor Auth and Pagination | prism-sensors | 5 | -- | 3 | S-2.06 |
| S-2.08 | Event Table Abstraction and Local Buffering | prism-sensors | 0 | -- | 3 | S-2.06,S-2.01,S-1.11 |
| S-3.01 | PrismQL Parser (Filter + SQL + Pipe) | prism-query | 4 | VP-014,015,021 | 3 | S-1.01 |
| S-3.02 | Query Tool and Materialization | prism-query | 6 | VP-031 | 3 | S-3.01,S-2.06,S-1.04,S-2.01,S-2.03,S-6.08,S-6.09,S-6.10 |
| S-3.03 | Explain and Query Diagnostics | prism-query | 1 | -- | 1 | S-3.02 |
| S-3.04 | Alias System (P1) | prism-query | 5 | VP-012,013,037 | 2 | S-3.02,S-1.08,S-1.09 |
| S-3.05 | Pagination and Caching | prism-query | 6 | VP-025 | 2 | S-3.02 |
| S-3.06 | PrismQL Write Parser Extensions | prism-query | 1 | -- | 2 | S-3.01,S-1.13,S-6.07 |
| S-3.07 | Write Execution Pipeline | prism-query | 5 | -- | 3 | S-3.06,S-3.02,S-1.08,S-1.09,S-2.04,S-6.07 |
| S-3.08 | Hidden Columns | prism-query | 0 | -- | 1 | S-3.02 |
| S-3.09 | Query Performance Profiling | prism-query | 0 | -- | 1 | S-3.02 |
| S-3.10 | Cost Estimation (API Latency-Aware Planner) | prism-query | 0 | -- | 2 | S-3.09,S-3.02 |
| S-3.11 | In-Query Dedup Caching | prism-query | 0 | -- | 1 | S-3.02 |
| S-3.12 | Column Pruning and Field Selection Push-Down | prism-query | 0 | -- | 1 | S-3.02,S-2.06 |
| S-3.13 | Dynamic Table Availability | prism-query | 0 | -- | 1 | S-3.02,S-1.12 |
| S-4.01 | Schedule CRUD and Execution Loop | prism-operations | 5 | VP-026,030 | 3 | S-3.02,S-2.01 |
| S-4.02 | Differential Results and Packs | prism-operations | 5 | VP-019 | 2 | S-4.01 |
| S-4.03 | Detection Rule Loading and Compilation | prism-operations | 8 | VP-018 | 3 | S-3.02,S-1.08,S-2.01 |
| S-4.04 | Detection Evaluation (Single/Correlation/Sequence) | prism-operations | 5 | VP-027 | 3 | S-4.03 |
| S-4.05 | Alert Generation | prism-operations | 1 | VP-028 | 1 | S-4.04 |
| S-4.06 | Case Management | prism-operations | 9 | VP-052,053,054,060 | 3 | S-4.05,S-2.01 |
| S-4.07 | Case Metrics and Acknowledge Alert | prism-operations | 3 | -- | 2 | S-4.06 |
| S-4.08 | Action Delivery Framework | prism-operations | 9 | VP-044,VP-045,VP-046,VP-047 | 3 | S-4.05,S-4.06,S-4.01,S-1.15,S-6.11,S-6.12,S-6.13 |
| S-5.01 | Server Bootstrap and Tool Registration | prism-mcp | 7 | -- | 3 | S-1.08,S-3.02,S-4.01 |
| S-5.02 | Tool Routing, Errors, and Client Scoping | prism-mcp | 3 | -- | 2 | S-5.01 |
| S-5.03 | Resources and Prompts | prism-mcp | 4 | VP-050 | 2 | S-5.02 |
| S-5.04 | Sensor Health Subsystem | prism-mcp | 5 | -- | 2 | S-5.03,S-2.07 |
| S-5.05 | Config Loading and Validation | prism-mcp | 10 | -- | 3 | S-5.01,S-1.06 |
| S-5.06 | Action and Infusion MCP Tools | prism-mcp | 4 | -- | 2 | S-5.01,S-4.08,S-1.14,S-6.11,S-6.12,S-6.13,S-6.14,S-6.15 |
| S-5.07 | Multi-Repo Git Config Subscriptions | prism-mcp | 8 | -- | 4 | S-5.05,S-1.12 |
| S-5.08 | Diagnostics: prism logs CLI + get_diagnostics + Trace IDs | prism-mcp | 2 | -- | 5 | S-5.01,S-5.02,S-5.03 |
| S-5.09 | External Log Forwarding Subsystem | prism-mcp | 5 | VP-061,VP-062 | 4 | S-5.08,S-1.15,S-6.16,S-6.17,S-6.18,S-6.19 |
| S-5.10 | Audit Trail External Forwarding | prism-audit [*] | 8 | VP-039,VP-056 | 3 | S-2.04, S-5.09 |
| S-6.01 | CLI, Startup, and Initialization | prism-bin | 0 | -- | 2 | S-5.01,S-5.05,S-2.01 |
| S-6.02 | End-to-End Integration Smoke Tests | prism-bin | 0 | -- | 2 | S-6.01 |
| S-6.03 | Installation and Distribution | prism-bin | 0 | -- | 1 | S-6.01 |
| S-6.04 | prism credential CLI Subcommand Group | prism-bin | 12 | -- | 3 | S-1.06,S-1.07,S-6.01 |
| S-6.05 | prism migrate-storage CLI Command | prism-bin | 3 | -- | 2 | S-2.01,S-6.01 |

[*] S-5.10 is in the `prism-audit` crate — note that all other Wave 5 stories are in `prism-mcp`. This is intentional: audit trail forwarding belongs to the audit subsystem by BC-2.05.011, but the Wave 5 slot reflects its topological dependency on S-2.04 (Wave 2 anchor).

---

## BC Traceability Matrix

Every active BC maps to the story that implements it.

**Retired Contracts (Option A, Burst 4b):** BC-2.12.011 and BC-2.12.012 were retired when SS-18 (Action Delivery) BCs were committed. Their normative replacements are BC-2.18.001 (at-least-once delivery) and BC-2.18.006 (injection flag, don't strip). These retired BCs have been removed from this matrix and from S-4.08 frontmatter.

| BC | Story |
|----|-------|
| BC-2.01.002 | S-2.06 |
| BC-2.01.004 | S-2.07 |
| BC-2.01.005 | S-2.07 |
| BC-2.01.006 | S-2.07 |
| BC-2.01.007 | S-2.07 |
| BC-2.01.008 | S-2.07 |
| BC-2.01.010 | S-2.06 |
| BC-2.01.013 | S-2.06 |
| BC-2.01.014 | S-2.06 |
| BC-2.02.001 | S-1.04 |
| BC-2.02.002 | S-1.04 |
| BC-2.02.003 | S-1.05 |
| BC-2.02.004 | S-1.05 |
| BC-2.02.005 | S-1.05 |
| BC-2.02.006 | S-1.05 |
| BC-2.02.007 | S-1.05 |
| BC-2.02.008 | S-1.05 |
| BC-2.02.009 | S-1.04 |
| BC-2.02.010 | S-1.04 |
| BC-2.02.011 | S-1.05 |
| BC-2.02.012 | S-1.04 |
| BC-2.03.001 | S-1.06, S-6.04 |
| BC-2.03.002 | S-1.06, S-6.04 |
| BC-2.03.003 | S-1.06, S-6.04 |
| BC-2.03.004 | S-1.06, S-6.04 |
| BC-2.03.005 | S-1.07, S-6.04 |
| BC-2.03.006 | S-1.07, S-6.04 |
| BC-2.03.007 | S-1.07, S-6.04 |
| BC-2.03.008 | S-1.06, S-6.04 |
| BC-2.03.009 | S-1.07, S-6.04 |
| BC-2.03.010 | S-1.07, S-6.04 |
| BC-2.03.011 | S-1.06, S-6.04 |
| BC-2.03.012 | S-1.06, S-6.04 |
| BC-2.04.001 | S-1.08, S-3.07 |
| BC-2.04.002 | S-1.08 |
| BC-2.04.003 | S-1.08 |
| BC-2.04.004 | S-1.08 |
| BC-2.04.005 | S-1.08, S-3.07 |
| BC-2.04.006 | S-1.08 |
| BC-2.04.007 | S-1.09, S-3.07 |
| BC-2.04.008 | S-1.09, S-3.07 |
| BC-2.04.009 | S-1.09 |
| BC-2.04.010 | S-1.09 |
| BC-2.04.011 | S-1.09 |
| BC-2.04.012 | S-1.09 |
| BC-2.04.013 | S-1.08 |
| BC-2.04.014 | S-5.01 |
| BC-2.04.015 | S-1.08 |
| BC-2.05.001 | S-2.04, S-5.06, S-5.10 |
| BC-2.05.002 | S-2.04, S-5.10 |
| BC-2.05.003 | S-2.04, S-5.10 |
| BC-2.05.004 | S-2.04, S-5.10 |
| BC-2.05.005 | S-2.05 |
| BC-2.05.006 | S-2.04, S-5.10 |
| BC-2.05.007 | S-2.05 |
| BC-2.05.008 | S-2.04, S-5.10 |
| BC-2.05.009 | S-2.05, S-3.07 |
| BC-2.05.010 | S-2.05 |
| BC-2.05.011 | S-5.10 |
| BC-2.06.001 | S-5.05 |
| BC-2.06.002 | S-5.05 |
| BC-2.06.003 | S-5.05 |
| BC-2.06.004 | S-5.05 |
| BC-2.06.005 | S-5.05 |
| BC-2.06.006 | S-5.05 |
| BC-2.06.007 | S-5.05 |
| BC-2.06.008 | S-5.05 |
| BC-2.06.009 | S-5.05 |
| BC-2.06.010 | S-5.05 |
| BC-2.07.001 | S-3.05 |
| BC-2.07.002 | S-3.05 |
| BC-2.07.003 | S-3.05 |
| BC-2.07.004 | S-3.05 |
| BC-2.07.005 | S-3.05 |
| BC-2.07.006 | S-3.05 |
| BC-2.08.001 | S-5.04 |
| BC-2.08.002 | S-5.04 |
| BC-2.08.003 | S-5.04 |
| BC-2.08.004 | S-5.04 |
| BC-2.08.005 | S-5.03 |
| BC-2.08.006 | S-5.03 |
| BC-2.08.007 | S-5.04 |
| BC-2.08.008 | S-5.08 |
| BC-2.08.009 | S-5.08 |
| BC-2.09.001 | S-1.10 |
| BC-2.09.002 | S-1.10 |
| BC-2.09.003 | S-1.10 |
| BC-2.09.004 | S-1.10 |
| BC-2.09.005 | S-1.10 |
| BC-2.09.006 | S-1.10 |
| BC-2.09.007 | S-1.10 |
| BC-2.09.008 | S-1.10 |
| BC-2.10.001 | S-5.01 |
| BC-2.10.002 | S-5.01 |
| BC-2.10.003 | S-5.01 |
| BC-2.10.004 | S-5.02 |
| BC-2.10.005 | S-5.01 |
| BC-2.10.006 | S-5.01 |
| BC-2.10.007 | S-5.02 |
| BC-2.10.008 | S-5.03 |
| BC-2.10.009 | S-5.03 |
| BC-2.10.010 | S-5.01 |
| BC-2.10.011 | S-5.02 |
| BC-2.11.001 | S-3.02 |
| BC-2.11.002 | S-3.01 |
| BC-2.11.003 | S-3.01 |
| BC-2.11.004 | S-3.01, S-3.06 |
| BC-2.11.005 | S-3.02 |
| BC-2.11.006 | S-3.01, S-3.02 |
| BC-2.11.007 | S-3.02 |
| BC-2.11.008 | S-3.04 |
| BC-2.11.009 | S-3.04 |
| BC-2.11.010 | S-3.03 |
| BC-2.11.011 | S-3.02 |
| BC-2.11.012 | S-3.02 |
| BC-2.11.013 | S-3.04 |
| BC-2.11.014 | S-3.04 |
| BC-2.11.015 | S-3.04 |
| BC-2.12.001 | S-4.01 |
| BC-2.12.002 | S-4.01 |
| BC-2.12.003 | S-4.01 |
| BC-2.12.004 | S-4.01 |
| BC-2.12.005 | S-4.02 |
| BC-2.12.006 | S-4.02 |
| BC-2.12.007 | S-4.02 |
| BC-2.12.008 | S-4.02 |
| BC-2.12.009 | S-4.02 |
| BC-2.12.010 | S-4.01 |
| BC-2.13.001 | S-4.03 |
| BC-2.13.002 | S-4.04 |
| BC-2.13.003 | S-4.04 |
| BC-2.13.004 | S-4.04 |
| BC-2.13.005 | S-4.05 |
| BC-2.13.006 | S-4.03 |
| BC-2.13.007 | S-4.03 |
| BC-2.13.008 | S-4.03 |
| BC-2.13.009 | S-4.03 |
| BC-2.13.010 | S-4.03 |
| BC-2.13.011 | S-4.03 |
| BC-2.13.012 | S-4.04 |
| BC-2.13.013 | S-4.04 |
| BC-2.13.014 | S-4.03 |
| BC-2.14.001 | S-4.06 |
| BC-2.14.002 | S-4.06 |
| BC-2.14.003 | S-4.06 |
| BC-2.14.004 | S-4.06 |
| BC-2.14.005 | S-4.06 |
| BC-2.14.006 | S-4.06 |
| BC-2.14.007 | S-4.06 |
| BC-2.14.008 | S-4.07 |
| BC-2.14.009 | S-4.06 |
| BC-2.14.010 | S-4.07 |
| BC-2.14.012 | S-4.07 |
| BC-2.14.013 | S-4.06 |
| BC-2.15.001 | S-2.01, S-6.05 |
| BC-2.15.002 | S-2.01, S-6.05 |
| BC-2.15.003 | S-2.02 |
| BC-2.15.004 | S-2.02, S-5.10 |
| BC-2.15.005 | S-2.01, S-6.05 |
| BC-2.15.006 | S-2.02 |
| BC-2.15.007 | S-2.02 |
| BC-2.15.008 | S-2.02 |
| BC-2.15.009 | S-2.03 |
| BC-2.15.010 | S-2.03 |
| BC-2.15.011 | S-2.03 |
| BC-2.16.001 | S-1.11, S-1.13 |
| BC-2.16.002 | S-1.11 |
| BC-2.16.003 | S-1.11 |
| BC-2.16.004 | S-1.11 |
| BC-2.16.005 | S-1.12 |
| BC-2.16.006 | S-1.12 |
| BC-2.16.007 | S-1.12 |
| BC-2.16.008 | S-1.12 |
| BC-2.16.009 | S-1.11, S-1.13 |
| BC-2.16.010 | S-1.12 |
| BC-2.17.001 | S-1.15 |
| BC-2.17.002 | S-1.15 |
| BC-2.17.003 | S-1.15 |
| BC-2.17.004 | S-1.15 |
| BC-2.17.005 | S-1.15, S-5.06 |
| BC-2.17.006 | S-1.15 |
| BC-2.18.001 | S-4.08 |
| BC-2.18.002 | S-4.08 |
| BC-2.18.003 | S-4.08, S-5.06 |
| BC-2.18.004 | S-4.08 |
| BC-2.18.005 | S-4.08 |
| BC-2.18.006 | S-4.08 |
| BC-2.18.007 | S-4.08 |
| BC-2.18.008 | S-4.08 |
| BC-2.18.009 | S-4.08 |
| BC-2.19.001 | S-1.14 |
| BC-2.19.002 | S-1.14 |
| BC-2.19.003 | S-1.14 |
| BC-2.19.004 | S-1.14, S-5.06 |
| BC-2.19.005 | S-1.14 |
| BC-2.20.001 | S-5.09 |
| BC-2.20.002 | S-5.09 |
| BC-2.20.003 | S-5.09 |
| BC-2.20.004 | S-5.09 |
| BC-2.20.005 | S-5.09 |

---

## VP Assignment Matrix

| VP | Story | Method | Property (from verification-architecture.md) |
|----|-------|--------|----------------------------------------------|
| VP-001 | S-1.01 | kani | TenantId rejects invalid characters |
| VP-002 | S-1.03 | kani | Capability resolution: deny-by-default |
| VP-003 | S-1.03 | kani | Capability resolution: most-specific-path wins |
| VP-004 | S-1.03 | kani | Capability resolution: deny overrides allow at same specificity |
| VP-005 | S-1.02 | kani | Case state machine: exactly 12 valid transitions |
| VP-006 | S-1.02 | kani | Case state machine: no self-transitions |
| VP-007 | S-1.09 | kani | Confirmation token expiry: expired at boundary (inclusive) |
| VP-008 | S-1.09 | kani | Confirmation token: single-use (consumed rejects second use) |
| VP-009 | S-1.09 | kani | Confirmation token: content hash mismatch rejects |
| VP-010 | S-1.09 | kani | Token cap: store rejects at 100 active tokens |
| VP-011 | S-1.02 | kani | Credential name sanitization: rejects path traversal |
| VP-012 | S-3.04 | kani | Alias depth: rejects composition beyond depth 3 |
| VP-013 | S-3.04 | proptest | Alias cycles: detects and rejects cyclic references |
| VP-014 | S-3.01 | kani | Query security limits: rejects oversized queries |
| VP-015 | S-3.01 | kani | Query security limits: rejects excessive nesting depth |
| VP-016 | S-1.04 | proptest | OCSF normalization: output is valid protobuf |
| VP-017 | S-1.05 | proptest | OCSF normalization: unmapped fields preserved in raw_extensions |
| VP-018 | S-4.03 | proptest | Detection rule validation: rejects invalid rules |
| VP-019 | S-4.02 | proptest | Diff computation: deterministic (same inputs → same output) |
| VP-020 | S-1.08 | kani | Feature flag: compile-time AND runtime must both permit |
| VP-021 | S-3.01 | fuzz | PrismQL parser: never panics on arbitrary input |
| VP-022 | S-1.04 | fuzz | OCSF normalizer: never panics on arbitrary sensor response |
| VP-023 | S-1.11 | fuzz | Sensor spec parser: never panics on arbitrary TOML |
| VP-024 | S-1.10 | proptest | Injection scanner: detects known injection patterns |
| VP-025 | S-3.05 | kani | Cache key derivation: deterministic for same parameters |
| VP-026 | S-4.01 | kani | Splay computation: deterministic per (query, client) |
| VP-027 | S-4.04 | proptest | Alert dedup key: correct per match mode |
| VP-028 | S-4.05 | fuzz | Template interpolation: never panics, handles missing vars |
| VP-029 | S-1.02 | kani | Cursor cap: rejects at 200 active cursors |
| VP-030 | S-4.01 | kani | Schedule/rule count caps: rejects beyond limits |
| VP-031 | S-3.02 | proptest | Required column enforcement: rejects unconstrained queries |
| VP-032 | S-1.12 | proptest | Hot reload atomicity: failed validation retains old config |
| VP-033 | S-6.07 | integration_test | Audit buffer: RocksDB write completes before delivery attempt (exercised by prism-dtu-crowdstrike clone; production story: S-2.04) |
| VP-034 | S-1.06 | proptest | Encryption round-trip: encrypt then decrypt returns plaintext |
| VP-035 | S-1.06 | proptest | Key derivation: same inputs produce same key |
| VP-036 | S-6.07 | integration_test | SessionContext dropped before error propagation and on panic (exercised by prism-dtu-crowdstrike clone; production story: S-4.04) |
| VP-037 | S-3.04 | fuzz | Alias expansion: never panics on arbitrary alias graphs |
| VP-038 | S-1.10 | fuzz | Injection scanner: never panics on arbitrary input strings |
| VP-039 | S-5.10 | kani | Audit forward watermark monotonicity: `Watermark::advance()` never decreases the stored watermark for any destination (proves BC-2.05.011 invariant) |
| VP-040 | S-1.15 | kani | Plugin Linker excludes all WASI namespace imports |
| VP-041 | S-1.15 | proptest | Plugin memory limit boundary: at-limit succeeds, over-limit traps |
| VP-042 | S-1.15 | proptest | Plugin hot reload: failed compile retains old InstancePre |
| VP-043 | S-1.15 | proptest | WIT validation rejects component missing required exports |
| VP-044 | S-4.08 | kani | Action retry state machine: bounded by 5 attempts, dead-letter terminal |
| VP-045 | S-4.08 | proptest | Schedule semaphore: try_acquire used (non-blocking), never acquire |
| VP-046 | S-4.08 | proptest | Action inline credential rejected at load time; value not in error message |
| VP-047 | S-4.08 | proptest | UUID v7 validation: non-v7 always rejected, v7 always accepted, order preserved |
| VP-048 | S-1.14 | kani | Infusion spec: N fields produces exactly N UDF descriptors; duplicates error |
| VP-049 | S-1.14 | proptest | Infusion per-query dedup: source calls = unique value count |
| VP-050 | S-5.03 | proptest | MCP sensor resource response redacts credentials and full API URLs |
| VP-051 | S-1.02 | kani | Case state machine: exhaustive 5×5 transition table — 12 accept, 13 reject |
| VP-052 | S-4.06 | proptest | update_case: disposition applied before status transition in single-call update |
| VP-053 | S-4.06 | kani | Resolved case always has non-null disposition; transition rejects without disposition |
| VP-054 | S-4.06 | proptest | TTR uses first resolution timestamp across reopen cycles; null aggregate when no resolved cases |
| VP-055 | S-1.02 | proptest | StorageEngine put_batch atomicity and domain isolation (MockStorageEngine) |
| VP-056 | S-5.10 | proptest | Audit buffer overflow purge: oldest entries deleted, newest preserved, purge-event produced |
| VP-057 | S-1.02 | kani | Crash recovery: denylist triggered at consecutive_crashes >= 3; exact threshold |
| VP-058 | S-2.02 | proptest | Watchdog memory grace period: single check does not terminate; two consecutive checks do |
| VP-059 | S-1.11 | proptest | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok |
| VP-060 | S-4.06 | proptest | Dedup decision: Link(c.id) iff existing case within window; Create otherwise |
| VP-061 | S-5.09 | proptest | Log Forwarder Min-Level Filter Determinism — level_filter(event, threshold) returns accept iff level_rank(event_level) >= level_rank(threshold); deterministic on every call. Proves BC-2.20.002 postcondition. |
| VP-062 | S-5.09 | proptest | Log Forwarder Queue Cap Bounded — for any enqueue sequence beyond 10 × batch_size entries, queue length never exceeds cap and oldest entry is dropped first (drop-oldest semantics). Proves BC-2.20.003 postcondition. |

---

## Scope Expansions (Phase 3 Patch)

The following existing stories received scope expansions. Implementors MUST read the
scope expansion block (marked `[SCOPE EXPANSION — Phase 3 patch]`) within each story.

| Story | Expansion | Delta |
|-------|-----------|-------|
| S-6.01 | Add Logs/Credential/MigrateStorage to clap Commands enum as placeholders | ~200 lines |
| S-2.01 | Document action_state CF key schema in rocksdb_backend.rs | ~10 lines |
| S-5.05 | Added scope boundary note: git sync / config diff / show --trace commands are S-5.07's scope, not S-5.05 | ~10 lines |
| S-1.14 | BC anchors (BC-2.19.001–005) + infusion_cache CF initialization, per-query LRU struct, TTL eviction policy, hot path read/write integration | ~60 lines |
| S-4.03 | IOC file loading and ioc_match UDF registration: *.ioc parser, IocStore, hot reload, size limits, UDF wiring | ~80 lines |
| Burst 6b: DTU blocks edges added (option B) | All 13 DTU clone stories now have explicit `blocks:` edges to their consumer stories; S-6.06 risk_mitigations anchored; VP-033/VP-036 deduplicated to S-6.07; 13 DTU stories subsystems updated to SS-IDs; fidelity taxonomy parenthetical sweep; S-6.06 filename: dtu-sensor-stubs → dtu-common | ~350 lines across 16 story files |
| step5-option2: DTU-first wave rework (2026-04-20) | User directive Option 2: blocks: edges restored on 13 DTU stories (Step 5 Track A had removed them); reciprocal depends_on edges added to 7 product stories; DTU stories distributed across waves 0-3 (was all wave 0). S-6.14/15 → wave 0; S-6.07-10 → wave 1; S-6.11-13 → wave 2; S-6.16-19 → wave 3. S-6.04/05 unchanged at wave 6. | 20 files across 2 actions |
| Burst 7: Pass-4 fixes + SS-20 re-anchor + taxonomy canonicalization | P3P4-H-001: S-6.19 line 256 `prism-operations` → `prism-mcp`. P3P4-H-003: BC-2.14.013 row added to BC Traceability Matrix (191 → 192 rows). P3P4-L-001: 13 DTU story titles (YAML `title:` + H1 heading) canonicalized to `— L[0-4] ([qualifier])` form; 13 STORY-INDEX Full Story List cells updated to match. SS-20 re-anchor: S-5.09, S-6.16, S-6.17, S-6.18, S-6.19 subsystems [SS-08] → [SS-20] (new subsystem: Observability / Log Forwarding; subsystem count 19 → 20). STORY-INDEX v1.7 → v1.8. | ~80 lines across 15 files |
| Burst 8 | P3P5-L-001: Burst-5b-SW-A summary rows (lines 584–596) canonicalized to L[0-4] (qualifier) form. 13 substitutions: 2 L4, 2 L3, 9 L2. STORY-INDEX v1.8 → v1.9. | ~20 lines |

---

## Retroactive BC Anchor Updates (Phase 3 Burst 2)

The following stories had BC anchor updates applied after their respective BCs were
committed by the product-owner in Burst 1.

| Story | BCs Added | Notes |
|-------|-----------|-------|
| S-1.15 | BC-2.17.001–006 | SS-17 WASM Plugin Runtime BCs now committed; INV-PLUGIN-NNN table updated with BC column |
| S-4.08 | BC-2.18.001–009 | SS-18 Action Delivery BCs now committed; INV-ACTION-NNN table updated with BC column |
| S-4.07 | BC-2.14.012 (gate resolved) | BC-2.14.012 (`acknowledge_alert`) was previously STUB; now fully specified. STUB gate language removed from story. |
| S-4.06 | BC-2.14.013 | Auto case creation BC now committed; Task 9 and AC-11/12/13 added for CRITICAL-severity auto-case behavior |
| S-1.14 | BC-2.19.001–005 | SS-19 Infusion Framework BCs now committed; frontmatter and BC table updated |

---

## Retroactive BC Anchor Updates (Phase 3 Burst 2.75)

Surgical traceability pass after product-owner committed 4 new BCs. No new stories;
no scope changes. Only frontmatter, BC tables, and VP tables updated.

| Story | BCs Added | VP Added | Notes |
|-------|-----------|----------|-------|
| S-5.08 | BC-2.08.008, BC-2.08.009 | -- | Dedicated SS-08 contracts for `get_diagnostics` tool and `prism://diagnostics/*` resource templates now committed. Product-owner flag in notes section replaced with resolved anchor. |
| S-5.10 | BC-2.05.011 | VP-039 | At-least-once forwarding contract committed as BC-2.05.011 (not 009 — those were occupied). Kani watermark monotonicity proof registered as VP-039. Product-owner proposal section replaced with resolved anchor. |
| S-4.03 | BC-2.13.014 | -- | IOC File Loading and Pattern Store contract committed. "No dedicated BC" hedge in Task 8a removed. |

---

## Dependency Graph — New Stories

New dependencies introduced by Phase 3 patch stories:

- S-6.04 depends on: S-1.06, S-1.07, S-6.01
- S-6.05 depends on: S-2.01, S-6.01
- S-0.01, S-0.02: no dependencies (Wave 0 root stories)
- S-5.07 depends on: S-5.05, S-1.12
- S-5.08 depends on: S-5.01, S-5.02, S-5.03
- S-5.09 depends on: S-5.08, S-1.15
- S-5.10 depends on: S-2.04, S-5.09

**Burst 5b-SW-A: DTU Story Dependencies (14 new edges):**
- S-6.06 (prism-dtu-common) depends on: S-0.02 (developer toolchain bootstrap — provides `just integration-test` target)
- S-6.06 blocks: S-6.07, S-6.08, S-6.09, S-6.10, S-6.11, S-6.12, S-6.13, S-6.14, S-6.15, S-6.16, S-6.17, S-6.18, S-6.19 (13 blocking edges — all per-surface DTU clones depend on common infrastructure)
- S-6.07 (prism-dtu-crowdstrike) blocks: S-3.06 (PrismQL write parser integration tests), S-3.07 (write execution integration tests) — CrowdStrike clone is the primary integration-test vehicle for write-path BCs

All chains acyclic:
- S-5.07 gated by S-5.05 (Layer 8) → lands in Layer 9
- S-5.08 gated by S-5.03 (Layer 9) → lands in Layer 10
- S-5.09 gated by S-5.08 (Layer 10) → lands in Layer 11
- S-5.10 gated by S-5.09 (Layer 11) → lands in Layer 12 (Wave 5 by crate boundary; S-5.09 is the binding constraint)
- S-6.06 gated by S-0.02 (product Layer 0) → lands in Test-Track Layer 0 (Wave 0; parallel to all product layers)
- S-6.07–S-6.19 gated by S-6.06 (Test-Track Layer 0) → land in Test-Track Layer 1 (Wave 0; parallel to product layers)
- No cycles introduced. Topological sort confirms acyclicity.

**M-006 decision: Option B (parallel Test Track dimension).** Product layers 0–11 remain
integer and unchanged. DTU stories use a parallel "Test Track" dimension:
  Test-Track Layer 0 = S-6.06 (prism-dtu-common, depends on product Layer 0)
  Test-Track Layer 1 = S-6.07–S-6.19 (per-surface clones, depend on Test-Track Layer 0)
This avoids shifting every product layer and makes the DTU independence from the product
graph explicit.

---

## Topological Order (Dependency Validation)

Topological sort confirms the dependency graph is acyclic. Execution order:

```
Product Layers:
Layer 0 (devops):    S-0.01, S-0.02
Layer 1 (no product deps): S-1.01
Layer 2:             S-1.02, S-1.03, S-1.04, S-1.10, S-1.11, S-3.01, S-2.01
Layer 3:             S-1.05, S-1.06, S-1.08, S-1.12, S-1.13, S-1.14(*), S-1.15, S-2.02, S-2.03
Layer 4:             S-1.07, S-1.09, S-2.04, S-2.06, S-3.06(*)
Layer 5:             S-2.05, S-2.07, S-2.08, S-3.02(*)
Layer 6:             S-3.03, S-3.04, S-3.05, S-3.07(*), S-3.08, S-3.11, S-3.12, S-3.13, S-4.01, S-4.03
Layer 7:             S-3.09, S-4.02, S-4.04, S-5.01
Layer 8:             S-3.10, S-4.05, S-5.02, S-5.05
Layer 9:             S-4.06, S-5.03, S-5.07 (gated by S-5.05 Layer 8), S-6.01
Layer 10:            S-4.07, S-4.08(*), S-5.04, S-5.08 (gated by S-5.03 Layer 9), S-6.02, S-6.03, S-6.04, S-6.05
Layer 11:            S-5.06(*), S-5.09(*) (gated by S-5.08 Layer 10)
Layer 12:            S-5.10 (gated by S-5.09 Layer 11; Wave 5 by crate boundary)

(*) These stories now depend on DTU clones (Option 2). Their product-layer placement is
unchanged because the DTU waves always precede them. DTU dependencies do not lengthen the
critical path — they are satisfied earlier in the schedule than the story's product gating dep.

DTU Test Track (Option 2 — now integrated into product wave schedule):
TT-Layer 0 (DTU common + threat-intel): S-6.06, S-6.14, S-6.15 (wave 0; S-6.06 gates S-6.14/15)
TT-Layer 1 (sensor DTUs): S-6.07, S-6.08, S-6.09, S-6.10 (wave 1; depend on S-6.06)
TT-Layer 2 (action DTUs): S-6.11, S-6.12, S-6.13 (wave 2; depend on S-6.06)
TT-Layer 3 (log-forwarding DTUs): S-6.16, S-6.17, S-6.18, S-6.19 (wave 3; depend on S-6.06)
```

**Topological layer design note (step5-option2):** Per user directive Option 2 (DTU-first),
product stories that require DTU clones as test fixtures now explicitly depend on them.
DTU stories are distributed across waves 0-3. Product layers 0-11 are unchanged because
DTU dependencies do not lengthen the critical path — each DTU completes before its consumer's
other gating dependencies. No cycles introduced.

**IMPORTANT — S-6.* ID namespace note (P3P3-L-001, updated step5-option2):** S-6.* story IDs
span TWO topological tracks. S-6.01–S-6.05 are product Wave 6 stories (prism-bin layer, product
Layer 9–10). S-6.06–S-6.19 are DTU Test Track stories distributed across waves 0-3. Do NOT
assume that all S-6.* stories are in the same wave or layer. The `wave:` frontmatter field in
each story is the authoritative source:
- S-6.06, S-6.14, S-6.15: wave 0 (DTU common + threat-intel; precede wave-1 S-1.14)
- S-6.07, S-6.08, S-6.09, S-6.10: wave 1 (sensor DTUs; precede wave-3 consumers)
- S-6.11, S-6.12, S-6.13: wave 2 (action DTUs; precede wave-4 S-4.08)
- S-6.16, S-6.17, S-6.18, S-6.19: wave 3 (log-forwarding DTUs; precede wave-5 S-5.09)
- S-6.01–S-6.05: wave 6 (product binary; independent of DTU)

Note on DTU→consumer blocking edges (step5-option2, Option 2 / DTU-first): DTU stories have
explicit `blocks:` entries AND their consumer product stories have reciprocal `depends_on:` entries.
Edge set (all restored from Burst 6b, plus Option 2 reciprocal edges):
- S-6.07 → S-3.06, S-3.07 (CrowdStrike: write-parser + write-execution integration tests)
- S-6.08 → S-3.02 (Claroty query integration test)
- S-6.09 → S-3.02 (Cyberint query integration test)
- S-6.10 → S-3.02 (Armis query integration test)
- S-6.11 → S-4.08, S-5.06 (Slack action delivery + MCP tools)
- S-6.12 → S-4.08, S-5.06 (PagerDuty action delivery + MCP tools)
- S-6.13 → S-4.08, S-5.06 (Jira action delivery + MCP tools)
- S-6.14 → S-1.14, S-5.06 (threat intel infusion + MCP tools)
- S-6.15 → S-1.14, S-5.06 (NVD/CVSS infusion + MCP tools)
- S-6.16 → S-5.09 (Datadog log forwarding)
- S-6.17 → S-5.09 (Splunk HEC log forwarding)
- S-6.18 → S-5.09 (Elasticsearch log forwarding)
- S-6.19 → S-5.09 (OTLP log forwarding)

Cycle check (Option 2): DTU stories (S-6.06–S-6.19) depend only on S-0.02 or S-6.06. Product
stories depend on DTU stories only — never the reverse. DTU → product edges flow only forward
(DTU wave ≤ product wave for all edges). No cycles exist.

Dependency chain verification:
- S-6.14/S-6.15 (wave 0) → S-1.14 (wave 1): wave 0 < wave 1. OK.
- S-6.14/S-6.15 (wave 0) → S-5.06 (wave 5): wave 0 < wave 5. OK.
- S-6.07-S-6.10 (wave 1) → S-3.02/S-3.06/S-3.07 (wave 3): wave 1 < wave 3. OK.
- S-6.11-S-6.13 (wave 2) → S-4.08 (wave 4): wave 2 < wave 4. OK.
- S-6.11-S-6.13 (wave 2) → S-5.06 (wave 5): wave 2 < wave 5. OK.
- S-6.16-S-6.19 (wave 3) → S-5.09 (wave 5): wave 3 < wave 5. OK.
All depends_on edges satisfied by earlier-or-equal wave predecessors. No cycles detected.

Notes on story placement:
- S-1.13 (write endpoint specs) lands in Layer 3 — depends only on S-1.11 (Layer 2)
- S-1.14 (infusion specs) lands in Layer 3 — depends only on S-1.11 (Layer 2)
- S-1.15 (WASM plugin runtime) lands in Layer 3 — depends only on S-1.11 (Layer 2)
- S-3.06 (write parser) lands in Layer 4 — depends on S-3.01 (Layer 2) and S-1.13 (Layer 3)
- S-2.08 (event tables) lands in Layer 5 — depends on S-2.06 (Layer 4), S-2.01 (Layer 2),
  and S-1.11 (Layer 2). Gated by S-2.06 as the longest dep chain.
- S-3.07 (write execution) lands in Layer 6 — depends on S-3.06 (Layer 4), S-3.02 (Layer 5),
  S-1.08 (Layer 3), S-1.09 (Layer 4), and S-2.04 (Layer 4). Gated by S-3.02.
- S-3.08 (hidden columns) lands in Layer 6 — depends only on S-3.02 (Layer 5)
- S-3.11 (in-query caching) lands in Layer 6 — depends only on S-3.02 (Layer 5)
- S-3.12 (column pruning) lands in Layer 6 — depends on S-3.02 (Layer 5) and S-2.06 (Layer 4).
  Gated by S-3.02.
- S-3.13 (dynamic table availability) lands in Layer 6 — depends on S-3.02 (Layer 5) and
  S-1.12 (Layer 3). Gated by S-3.02.
- S-3.09 (query profiling) lands in Layer 7 — depends only on S-3.02 (Layer 5) but logically
  positioned here to allow S-3.08/S-3.11/S-3.12/S-3.13 to be wired into it.
- S-3.10 (cost estimation) lands in Layer 8 — depends on S-3.09 (Layer 7) and S-3.02 (Layer 5).
  Gated by S-3.09.
- S-4.08 (action delivery) lands in Layer 10 — depends on S-4.05 (Layer 8), S-4.06 (Layer 9),
  S-4.01 (Layer 6), and S-1.15 (Layer 3). Gated by S-4.06 (Layer 9) as the longest dep chain.
- S-5.06 (action/infusion tools) lands in Layer 11 — depends on S-5.01 (Layer 7), S-4.08
  (Layer 10), and S-1.14 (Layer 3). Gated by S-4.08 as the longest dep chain.

No cycles detected. Wave assignments follow these layers grouped by crate boundary.

---

## Scope Expansions / Retroactive Updates — Burst 5b-SW-A

**Burst 5b-SW-A: DTU Story Addition (2026-04-16)**

SW-A added 14 DTU stories and rescoped S-6.06:

| Change | Detail |
|--------|--------|
| S-6.06 rescoped | Was `prism-dtu` stub in Wave 6 (depends on S-2.07). Now `prism-dtu-common` in Wave 0 (depends on S-0.02). Provides `BehavioralClone` trait, latency/failure injection middleware, fixture loader, `SyslogReceiver`, `WebhookReceiver`, and shared assertion utilities. |
| S-6.07 new | prism-dtu-crowdstrike — L4 (adversarial) clone of CrowdStrike Falcon API. Primary VP-033/VP-036 vehicle. Blocks S-3.06/S-3.07 integration tests. 5 days. |
| S-6.08 new | prism-dtu-claroty — L4 (adversarial) clone of Claroty xDome API. 4 days. |
| S-6.09 new | prism-dtu-cyberint — L2 (stateful) clone of Cyberint API. 3 days. |
| S-6.10 new | prism-dtu-armis — L2 (stateful) clone of Armis Centrix API. 3 days. |
| S-6.11 new | prism-dtu-slack — L2 (stateful) clone of Slack Webhook API. 2 days. |
| S-6.12 new | prism-dtu-pagerduty — L3 (behavioral) clone of PagerDuty Events API v2. 4 days. |
| S-6.13 new | prism-dtu-jira — L3 (behavioral) clone of Jira REST API v3. 5 days. |
| S-6.14 new | prism-dtu-threatintel — L2 (stateful) clone of Threat Intel Aggregator. 3 days. |
| S-6.15 new | prism-dtu-nvd — L2 (stateful) clone of NVD/NIST CVSS API. 3 days. |
| S-6.16 new | prism-dtu-datadog — L2 (stateful) clone of Datadog Logs API. 2 days. |
| S-6.17 new | prism-dtu-splunk-hec — L2 (stateful) clone of Splunk HTTP Event Collector. 2 days. |
| S-6.18 new | prism-dtu-elasticsearch — L2 (stateful) clone of Elasticsearch Bulk API. 3 days. |
| S-6.19 new | prism-dtu-otlp — L2 (stateful) clone of OTLP/HTTP Log Ingestion. 3 days. |
| VP-033 reassigned | From S-2.04 → S-6.07 (integration-test VPs anchor to the DTU crate that exercises them) |
| VP-036 reassigned | From S-4.04 → S-6.07 (same reason) |

All 13 new DTU clones: Wave 0, 0 BCs, priority P0, depends_on: [S-6.06].

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| v1.22 | 2026-04-19 | Burst 30 — comprehensive scripted BC-INDEX-to-story-body title sweep (first of its kind in this cycle). Found 14 title drifts across 5 stories, fixed all. Plus pass-29 specific fixes: S-1.08 em-dash→double-hyphen, S-1.10 BC-2.09.003/.004 title sync, S-1.12 3 backtick adds. Plus [SCOPE EXPANSION — Phase 3 patch] marker strips from S-4.03, S-4.06 (pass-27 L-001 residual close). Trajectory break-out attempt: pass-30 is first candidate for convergence-counter advance in this cycle. |
| v1.23 | 2026-04-19 | Burst 31 — close pass-30 4 findings surgically. S-1.05 line 51 3-col description "Three-tier"→"Four-tier field alias resolution: Prism metadata → Proto descriptor fields → raw_extensions JSON → None" (M-001). S-1.10 +3 ACs (AC-6 BC-2.09.001 structural separation, AC-7 BC-2.09.006 tool description 9-section template, AC-8 BC-2.09.007 OutputSchema) closing Policy-8 orphan gap (M-002). S-1.08 +AC-8 tracing BC-2.04.003 hierarchical resolution (M-003). S-1.10 Task 4 rewritten to centralized _meta.safety_flags array, prohibiting per-field parallel fields (L-001). Total: 3 files, 4 edits (1 title, 4 AC additions, 1 task rewrite). |
| v1.24 | 2026-04-19 | Burst 32 — close pass-31 H-001 systematic Policy 8 sweep (13 BC-level AC-trace gaps across 6 stories) + M-101 S-1.05 Task 6 four-tier propagation fix. +13 ACs total: S-6.04 +AC-9/10/11/12/13 (BC-2.03.002/.003/.004/.005/.010 credential backend/fallback/namespace/file-input/audit); S-5.07 +AC-9/10/11 (BC-2.06.002/.007/.010 sensor mapping/field errors/ID validation); S-4.08 AC-2/3 +INV-ACTION-008 trace + AC-11 for BC-2.18.003 fire-and-forget; S-1.15 +AC-9 BC-2.17.003 memory limit; S-1.09 +AC-7 BC-2.04.007 risk tiers; S-2.04 +AC-6 BC-2.05.006 append-only. S-1.05 Task 6 rewritten to four-tier model per BC-2.02.008 (Prism metadata/Proto descriptor/raw_extensions/None); AC-8 expanded to test all 4 tiers. Policy 8 now clean across all 73 stories. |
| v1.25 | 2026-04-19 | Burst 33 — close pass-32 M-101 MCP tool naming drift. S-5.06 renamed execute_action→fire_action throughout (12 occurrences). Line 51 parenthetical synonymy removed. Rust source filenames also renamed (execute_action.rs→fire_action.rs). Now consistent with canonical name in BC-2.18.003, api-surface.md line 160, actions.md, and S-4.08 AC-11. |
| v1.26 | 2026-04-19 | Burst 38 — close P3P37-A-HIGH-001 + P3P37-A-MED-001. HIGH-001: S-5.06 BC count column 0→4 (4 BCs now owned). MED-001: BC Traceability Matrix co-ownership propagation — BC-2.05.001 adds S-5.06 (was S-2.04, S-5.10); BC-2.17.005 adds S-5.06 (was S-1.15); BC-2.18.003 adds S-5.06 (was S-4.08); BC-2.19.004 adds S-5.06 (was S-1.14). |
| v1.27 | 2026-04-19 | Burst 39 — close P3P38-A-HIGH-001 + P3P38-A-OBS-001. HIGH-001: Wave 5 summary BC count 47→51 (arithmetic regression from Burst 38; S-5.NN rows sum to 51 not 47); comment on line 70 updated to sum=238 (0+69+30+28+45+51+15). OBS-001: changelog row order corrected — v1.25 now precedes v1.26 in ascending version order. |
| v1.28 | 2026-04-19 | Burst 40 (retroactive) + Burst 41 — Corpus-wide Architecture Mapping fill (P3P25-A-L-002): 73 stories bumped v1.0→v1.1 in Burst 40 (subsystem + crate + purity classification added to Architecture Mapping tables). Burst 41 Track 3 (P3P39-A-MED-002): retroactive ## Changelog sections added to 67 stories that were missing audit trail for the v1.0→v1.1 transition. Stories already having changelog sections (S-1.14, S-1.15, S-4.08): unchanged. Track 1 stories (S-4.01, S-4.03, S-5.05, S-5.06, S-5.10): handled separately by Track 1. |
| v1.29 | 2026-04-20 | step5-option2 — Wave schedule reworked per user directive Option 2 (DTU-first). DTU clones S-6.06-S-6.19 distributed across waves 0-3 to precede their product consumers. S-6.04/S-6.05 remain in wave 6. Reciprocal depends_on edges added to 7 product stories (S-1.14, S-3.02, S-3.06, S-3.07, S-4.08, S-5.06, S-5.09). blocks: edges restored on 13 DTU stories. No cycles detected. |
| v1.30 | 2026-04-20 | pass-70-fix — HIGH-003: total_vps_assigned 39→50; VPs assigned count updated to 50 (23 Kani, 19 proptests, 6 fuzz, 2 integration tests). VP-040 through VP-050 added to VP Assignment Matrix and Full Story List VP columns for S-1.14, S-1.15, S-4.08, S-5.03. HIGH-002: verification_properties frontmatter propagated to 4 anchor stories. MED-003: S-4.08 changelog date inversion corrected (v1.0 date 2026-04-19→2026-04-17). |
| v1.31 | 2026-04-20 | pass-77-fix HIGH-002 — VP propagation drift: total_vps_assigned 50→60; VPs assigned count updated to 60 (26 Kani, 26 proptests, 6 fuzz, 2 integration tests). VP-051 through VP-060 added to VP Assignment Matrix. Full Story List VP columns updated: S-1.02 +VP-051/055/057; S-2.02 +VP-058; S-4.06 +VP-052/053/054/060; S-1.11 +VP-059; S-5.10 +VP-056. Story file verification_properties frontmatter propagated for all 5 anchor stories. |
| v1.32 | 2026-04-21 | pass-80-fix F80-004 + F80-008 — F80-004: S-5.09 re-anchored from BC-2.10.001 (SS-10, zero forwarder coverage) to 5 native SS-20 BCs (BC-2.20.001–005); Full Story List S-5.09 BC count 1→5. F80-008: S-5.08 frontmatter subsystems [SS-08, SS-10] → [SS-08, SS-10, SS-20] to match body Architecture Mapping table. Pre-existing Burst 8 table row fixed (missing Delta cell). |
| v1.33 | 2026-04-21 | pass-80-F80-002 follow-on — BC count sync after CAP-035 re-anchor. BC-INDEX version pins v4.10 → v4.12; active BC count 195 → 200. |
| v1.34 | 2026-04-21 | pass-83-F83-001 — VP count sync: total_vps_assigned 60→62; overview 26 proptests→28 proptests; S-5.09 VPs column --→VP-061,VP-062; VP-061 and VP-062 rows added to VP Assignment Matrix (proptest, prism-mcp, P1, BC-2.20.002/003, anchor S-5.09). |
| v1.35 | 2026-04-21 | pass-87 — VP body propagation across 10 stories (S-1.02, S-1.14, S-1.15, S-2.02, S-4.06, S-4.08, S-5.03, S-5.10 + S-3.04 VP-025 removal + S-3.05 re-anchor). |
| v1.36 | 2026-04-21 | pass-88 — F88-001: S-1.02 Task 15 crate path prism-persistence→prism-storage. F88-002: VP-025 catalog row S-3.04→S-3.05. F88-003: S-5.10 BC-2.15.004 added to frontmatter/body/inputs. F88-004: S-5.10 duplicate task 9 renumbered→11. F88-005: S-4.08 Tasks 15-18 renumbered→13-16. F88-006: File Structure rows for VP proof files in 8 stories. F88-007: Library rows (kani/proptest) in 6 stories. F88-009: S-3.04 VP-025 token budget row removed, total ~15300→~14800. F88-010: S-5.03 changelog B-40 duplicate burst disambiguated. F88-011: VP proof task section boundaries added to S-1.14 and S-1.15. |
| v1.37 | 2026-04-21 | pass-89 (retroactive entry). |
| v1.40 | 2026-04-21 | pass-92 F92-002..007 — anchor_capabilities sweep: S-1.09 CAP-005→CAP-006; S-3.04 CAP-015→CAP-016; S-3.05 CAP-015→CAP-011,CAP-014; S-3.07 CAP-004,CAP-005,CAP-007→CAP-005,CAP-006,CAP-007; S-1.12 CAP-029→CAP-029,CAP-030; S-5.10 CAP-007→CAP-007,CAP-025. |
| v1.39 | 2026-04-21 | pass-91 F91-001 — inputs frontmatter VP-path sweep across 10 stories: added 21 VP paths total (S-3.01 +3, S-3.02 +1, S-3.05 +1, S-4.06 +4, S-4.08 +4, S-5.03 +1, S-5.09 +2, S-6.07 +2, S-1.14 +2, S-2.02 +1). |
| v1.38 | 2026-04-21 | pass-90 F90-001 — S-5.10 dependency corrected: depends_on S-2.04→S-2.04,S-5.09. Topological layer updated: S-5.10 removed from Layer 5, added to Layer 12 (gated by S-5.09 Layer 11). Narrative line and dependency graph line updated to match. |
| v1.41 | 2026-04-21 | pass-97 F97-002 — BC-INDEX pin bumped v4.12→v4.13 at lines 24 and 76 (BCs covered note + unique-count comment). |
