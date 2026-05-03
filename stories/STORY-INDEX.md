---
document_type: story-index
level: "L4"
version: "v1.92"
status: draft
producer: state-manager
timestamp: 2026-05-02T23:30:00
phase: 3
total_stories: 129
total_active_bcs: 222
# 230 total registered (222 active + 6 removed + 2 retired) — stories cover active BCs only
total_vps_assigned: 136
---

# Prism Phase 3 Story Index

## Overview

Phase 3 decomposes the Prism platform into 113 implementation stories spanning 7 parallel
waves. Stories are organized by crate and ordered topologically so that no story begins
before its dependencies are complete.

- **Total stories:** 129 (76 through Wave 2 + 37 Wave 3 Multi-Tenant stories: S-3.0.01/02 + S-3.1.01–07 + S-3.2.01–08 + S-3.3.01–06 + S-3.4.01–05 + S-3.5.01 + S-3.6.01/02 + S-3.7.00–05 + 3 E-3.5 devx merged: W3-FIX-WIN/LEFTHOOK/CI-001 + 6 Wave 3.1 fix stories: W3-FIX-SEC-001/002/003 + W3-FIX-CODE-001/002/003 + 1 Wave 3.1 impl-phase story: S-3.1.06-ImplPhase + 2 Wave 3.2 fix stories: W3-FIX-CREDS-001 + W3-FIX-CODE-004 + 2 Wave 3.3 fix stories: W3-FIX-SEC-004 + W3-FIX-CODE-005 + 2 Wave 3.4 fix stories: W3-FIX-SEC-005 + W3-FIX-CODE-006)
- **Total waves:** 7 (Wave 0 expanded to 16 stories: devops + DTU infrastructure)
- **BCs covered:** 230 total registered (222 active per BC-INDEX.md v4.27; 200 Wave 1-2 BCs + 22 new Wave 3 BCs: BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001; at v0.2+ draft status; BC-3.3.004 is a distinct contract from BC-3.3.001 per PO rename in Phase 3.A consistency-validator pass)
- **VPs assigned:** 136 (30 Kani proofs, 77 proptests, 4 unit_tests, 6 fuzz targets, 19 integration tests)
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
- **S-6.20 scope expansion (2026-04-22):** Added S-6.20 (prism-dtu-demo-server: Unified Multi-Clone Demo Harness). Closes DTU design-review gaps: no multi-clone launcher; only S-6.09 had a demo_server bin; static fixtures; plain HTTP only. Wave 1 stories 19 → 20; total stories 75 → 76. Wave Summary Wave 1 crates adds prism-dtu-demo-server; Wave 1 "0 BCs" story count 3 → 4. No new BCs or VPs (harness is infrastructure only). STORY-INDEX v1.42 → v1.43.
- **Wave 1 gate Pass 10 remediation (2026-04-23):** P3WV1J-A-M-001: BC-INDEX version pin corrected from v4.13 → v4.14 (lines 24, 77). STATE.md `bc_index_version: "4.14"` is the authoritative source; STORY-INDEX pin was not updated when the v4.13→v4.14 bump occurred. No story count or BC/VP count changes. STORY-INDEX v1.43 → v1.44.
- **Wave 2 S-2.02 post-merge (2026-04-25):** S-2.02 status updated to MERGED (PR #52, 9de6b3d8, 25 tests, 2 review cycles). OBS-001 (demo-server dtu feature default-enabled) recorded. STORY-INDEX v1.44 → v1.45.
- **Wave 2 S-2.03 post-merge (2026-04-25):** S-2.03 status updated to MERGED (PR #53, f13b5c76, 19 tests, 1 review cycle). 3 spec-vs-impl deviations logged as TD-S203-001/002/003 (D-015). STORY-INDEX v1.45 → v1.46.
- **Wave 2 parallel batch post-merge (2026-04-25):** S-2.04, S-2.06, S-6.11, S-6.12, S-6.13 status updated to MERGED (PRs #58/54/57/55/56; +183 tests; develop 0b194cb4). Stub-as-impl anti-pattern disclosed for S-2.04/S-6.12/S-6.13; 4 vsdd-factory prevention layers queued (TD-VSDD-001..004); D-016..D-019 logged. STORY-INDEX v1.46 → v1.47.
- **Wave 2 S-2.05 post-merge (2026-04-26):** S-2.05 status updated to MERGED (PR #59, c828e8af, 35 tests, 1 review cycle; RED_RATIO 54.3% — first Wave-2 story to satisfy Layer 2 Red Gate density check). Anti-precedent guard (Layer 1 inlined in stub-architect prompt) confirmed working. TD-S205-001 registered: unify 3 interim context types into prism_core::QueryContext in v1.4. D-020 + D-021 logged. 9 of 11 Wave-2 stories complete. STORY-INDEX v1.47 → v1.48.
- **Wave 2 S-2.07 post-merge (2026-04-26):** S-2.07 status updated to MERGED (PR #60, 26d0954b, 56 tests, 1 review cycle; RED_RATIO 83.9%; healthy TDD 7 micro-commits; anchor BCs: BC-2.01.004/005/006/007/008). BC-2.01.005 batch-size non-conflict resolved (D-022). 5 test bug fixes documented as correctness fixes (D-023). 10 of 11 Wave-2 stories complete; S-2.08 remaining. STORY-INDEX v1.48 → v1.49.
- **Wave 2 S-2.08 post-merge — WAVE 2 CLOSED (2026-04-26):** S-2.08 status updated to MERGED (PR #61, 0be11cd6, 92 tests, 1 review cycle; RED_RATIO 54.3%; 50 RED + 42 GBD; v1.4→v1.5→v1.6 PO reconciliation; prism-query crate created; prism-spec-engine 0.1.0→0.2.0; anchor BCs: BC-2.16.x; TD-S208-001/002 registered; D-024..D-028 logged). Wave 2 CLOSED — 11/11 stories merged; workspace baseline 1043 → 1480 (+437 tests); develop f13b5c76 → 0be11cd6. Wave 2 integration gate triggered 2026-04-26. STORY-INDEX v1.49 → v1.50.
- **Wave 2 integration gate Pass-1 adversary spec amendment (2026-04-26):** S-2.08 spec retroactively amended v1.6→v1.7: AC-5 reclassified DEFERRED-to-S-3.02 (W2-P1-A-002 fix). Implementation structural-only (EventPoller construction, CancellationToken, diagnostics); SensorAdapter wiring deferred to S-3.02. No BC changes. STORY-INDEX v1.50 → v1.51.
- **W2-FIX-W2-D AC-5 split refinement (2026-04-26):** S-2.08 spec v1.7→v1.8: AC-5 split into AC-5a (cold-start ROUTING, IN-SCOPE PASS — `route_table_query()` returns `RouteDecision::ColdStartFallback`, 4 RED→GREEN tests in `table_dispatch_tests`) and AC-5b (cold-start EXECUTION, DEFERRED to S-3.02 — requires SensorAdapter wiring per BC-2.11.005/007). The v1.7 deferral was too aggressive; S-2.08 does credit the routing decision. S-3.02 spec v1.6→v1.7: AC-5b inherited as AC-9 (cold-start execution — live fetch via SensorAdapter + EventBufferStore write + INFO log); "Inherited Deferrals from Wave 2" section added; S-2.08 added to inputs. STORY-INDEX v1.51 → v1.52.
- **W2-P2-A-005 schema-hygiene fix (2026-04-26):** S-2.08 spec v1.8→v1.9: clarifying note added to AC-5b body explaining that BC-2.11.005/.007 are cited as deferral rationale only (owned by S-3.02); `behavioral_contracts: []` is correct per VSDD convention (implementation-owned BCs only). Option 1 resolution — no schema change, no new frontmatter fields. STORY-INDEX v1.52 → v1.53.
- **W2-P2-A-004 historical-narrative reconciliation (2026-04-26):** Counts cited in changelog entries prior to "S-6.20 scope expansion (2026-04-22)" (e.g., "story count remains 62" in Bursts 2.75/4b, "Story count: 62 → 75" in Burst 5b-SW-A, "Story count 62 → 75" in Burst 5b-SW-B) are accurate point-in-time snapshots recorded when those bursts ran. Current authoritative total is 76 (frontmatter: `total_stories: 76`; established in v1.43 when S-6.20 was added). Historical entries are not updated retroactively per change-log policy.
- **W2-FIX-G frontmatter sync (2026-04-26):** W2-FIX-G executed; 11 Wave 2 story files status synced draft → merged (WGCV-W2-001 CRITICAL closed); S-2.01 row annotated [MERGED PR #43 0d24ab79 2026-04-24 +24t] (WGCV-W2-002 HIGH closed). Closes WGCV-W2-001 + WGCV-W2-002. STORY-INDEX v1.53 → v1.54.
- **Wave 3 Multi-Tenant story registration (2026-04-27):** Added 35 new Wave 3 Multi-Tenant DTU stories (S-3.0.01/02, S-3.1.01–07, S-3.2.01–07, S-3.3.01–05, S-3.4.01–05, S-3.5.01, S-3.6.01/02, S-3.7.00–05) all at `status: draft` — NOT ready for implementation; pending Phase 3.A spec convergence + human approval. 22 new BCs (BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001) at v0.2 PROPOSED. 2 new CAPs (CAP-036 Multi-Tenant DTU Test Harness; CAP-037 Workspace Crate Layout Convention). Story count 76 → 111. BC count 200 → 222. Pre-compact handoff for post-compact convergence run. STORY-INDEX v1.54 → v1.55. NOTE: the original v1.55 entry recorded 16 stories and 21 BCs — both were undercounts; corrected to 35 stories and 22 BCs in v1.56 (BLOCK-2 + BLOCK-4 + BC-3.3.001→BC-3.3.004 propagation pass).
- **DRIFT-7 fix (2026-04-27):** BC-INDEX version pin updated v4.15 → v4.16 (matches BC-INDEX after Burst 2 NEW-1 fixes). No content changes. STORY-INDEX v1.56 → v1.57.
- **C-3/C-4/C-2/C-5 spec-reviewer fixes (2026-04-27):** Added 2 new stories: S-3.3.06 (prism-spec-engine reload_config mode-change detection — BC-3.2.005 invariant 4 + EC-006; 3 pts; depends S-3.3.02) and S-3.2.08 (prism-query CrowdStrike session ID org-scoping — BC-3.2.003 + D-048; 5 pts; depends S-3.1.06, S-3.2.03). C-2 propagation: S-3.3.01 updated — `allow_shared_override` removal, AC-017 added (E-CFG-010 rejection), ADR-007 §7 OQ-1 deferral reference added, `tdd_mode: strict` added. C-5 capability re-anchoring: `anchor_capabilities` updated in 9 stories — S-3.3.02 CAP-009→CAP-038; S-3.2.05/06/07 CAP-009→CAP-040; S-3.4.05 CAP-009→CAP-040; S-3.7.00–05 CAP-009→CAP-039. `tdd_mode` added to S-3.2.05/06/07, S-3.3.01/02. BC Traceability Matrix: BC-3.2.003 += S-3.2.08; BC-3.2.005 += S-3.3.06. Story count 111 → 113. STORY-INDEX v1.57 → v1.58.
- **DRIFT-1 fix (2026-04-27):** E-3.2 header story count 7→8 (S-3.2.08 was added in Step 2; header parenthetical was missed at that time). STORY-INDEX v1.58 → v1.59.
- **Adversary Pass 2 story-side fixes (2026-04-27):** M-001: total_vps_assigned 62→136; overview VP breakdown updated to 136 (30 Kani / 77 proptest / 4 unit_test / 6 fuzz / 19 integration) per VP-INDEX v1.13. m-004: S-3.0.02 scope reconciled with ADR-007 §2.3 — centralized `DTU_DEFAULT_MODE: &[DtuRegistryEntry]` registry in prism-core replaces per-crate constants; title + crate column corrected in STORY-INDEX; story file rewritten. STORY-INDEX v1.60 → v1.61.
- **Pass 16 story-side fixes (2026-04-27):** M-16-002: S-1.01 Full Story List title updated "TenantId" → "OrgSlug [TenantId legacy alias]" per ADR-006. M-16-003: S-3.1.01 + S-3.1.03 subsystems SS-06→SS-21 (prism-core owns OrgId/OrgRegistry per D-047/ARCH-INDEX SS-21). m-16-001: BC body table titles corrected to Title Case per BC-INDEX canonical form across all affected Wave 3 stories (S-3.1.01–07, S-3.2.01–07, S-3.3.01–02). STORY-INDEX v1.61 → v1.62.
- **M-32-001 fix burst (2026-04-28):** S-3.0.02 v0.3 → v0.4: subsystems [SS-01, SS-06] → [SS-21] (sibling-fix gap from D-116/D-117 CAP-040 SS-21 propagation — implementing story carried stale consumer-subsystem annotation; prism-core = SS-21 per ARCH-INDEX convention). Convention alignment with S-3.1.01/S-3.1.03. D-119. STORY-INDEX v1.62 → v1.63.
- **M-33-001 fix burst (2026-04-28):** STORY-INDEX line 552 — VP Assignment Matrix VP-001 Property column "TenantId rejects invalid characters" → "OrgSlug rejects invalid characters" per verification-architecture.md v1.21 source-of-truth (line 127). Residual M-14-002 OrgSlug-rename propagation (M-14-002 landed 19 passes ago; STORY-INDEX VP Assignment Matrix Property column was the last unswept location of the OrgSlug rename chain). D-120. STORY-INDEX v1.63 → v1.64.
- **M-34-001 fix burst (2026-04-28):** STORY-INDEX prose changelog — append missing v1.63 → v1.64 entry for M-33-001 fix (was added to tabular changelog only by Pass 33 burst). Bookkeeping completeness; no content change to spec artifacts. STORY-INDEX v1.64 → v1.65.
- **m-38-001 fix burst (2026-04-28):** S-3.5.01 v1.2 → v1.3: line 228 "all 6 subsystems are affected" → "all 7 subsystems are affected" (sibling-fix gap from Pass 27 m-27-001 v1.2 changelog over-claim that only patched line 57; line 228 in Subsystem Anchor Justification section was missed; survived 11 passes P27-P37). D-125. STORY-INDEX v1.65 → v1.66.
- **m-41-001 fix burst (2026-04-28):** S-3.5.01 v1.3 → v1.4: lines 57 + 228 stale paraphrase "all 7 subsystems" → "all 22 workspace crates regardless of their primary subsystem affiliation" per BC-3.7.001 v0.8 canonical framing. NEW DEFECT CLASS: stale-paraphrase-of-BC-canonical-framing. COMPREHENSIVE 6-class sweep performed across all BC-drift sub-classes — zero additional residues. D-128. STORY-INDEX v1.66 → v1.67.
- **m-42-001 fix burst (2026-04-28):** S-3.0.01 v0.1 → v0.2 + S-3.0.02 v0.4 → v0.5: frontmatter epic_id "E-Quick" → "E-3.0" to match STORY-INDEX canonical Wave 3 epic naming (E-3.X form). NEW DEFECT CLASS: frontmatter-vs-index field-value drift (8th this cycle, orthogonal to BC-drift). EXTENDED proactive sweep across Wave 3 frontmatter epic_id + status vs STORY-INDEX columns — zero additional VALUE_DRIFT hits. D-129. STORY-INDEX v1.67 → v1.68.
- **m-43-001 fix burst (2026-04-28):** S-3.0.01 v0.2 → v0.3: line 146 body cell text "first story in E-Quick" → "first story in E-3.0" (sibling propagation from m-42-001 frontmatter fix). NEW SUB-AXIS: intra-file body-prose-vs-frontmatter. ESCALATION NOT TRIGGERED — finding within recently-swept frontmatter-vs-index family. D-130. STORY-INDEX v1.68 → v1.69.
- **Pass 44 fixes (2026-04-28):** L-44-001 wave-state.yaml legacy `waves.wave_3` block removed (Path 1; D-040 canonical top-level block supersedes). O-44-001 STORY-INDEX changelog tabular block (lines 867-876) reordered ascending per v1.27 OBS-001 convention. User direction: continue Option A + commission Option C linter independently. D-131. STORY-INDEX v1.69 → v1.70.
- **Phase 3.A APPROVED (2026-04-28):** User approved Phase 3.A at Step 5 human approval gate. ADR-006..ADR-012 transitioned PROPOSED → ACCEPTED. 3 Wave 4+ TDs filed (audit query/replay, log forwarding, alerting workflows). Wave 3 implementation cleared to begin per D-045. Q1 scope+3 TDs; Q2-Q5 all approved. D-136. STORY-INDEX v1.70 → v1.71.
- **Wave 3 integration gate hygiene burst (2026-05-01):** 6 Wave 3.1 fix stories registered (W3-FIX-SEC-001/002/003 + W3-FIX-CODE-001/002/003; 24 pts). E-3.5 epic-view expanded from 1 story to 10 stories (S-3.5.01 + 3 merged devx fixes + 6 new fix stories). W3-FIX-WIN-001 epic_id corrected E-3.3 → E-3.5 (F-48-H-003). Total stories 113 → 119. STORY-INDEX v1.72 → v1.73. D-183.
- **W3-FIX-G state hygiene burst (2026-05-01):** Wave 3 integration gate step-e consistency-validator CONDITIONAL_FAIL remediation. (1) 37 Wave 3 MT story rows in Epic-view tables and Full Story List annotated with `[MERGED PR #NNN SHA DATE +Nt]` (closes WGCV-W3-002). (2) 3 W3-FIX devx stories (W3-FIX-WIN-001, W3-FIX-LEFTHOOK-001, W3-FIX-CI-001) registered in Full Story List section (closes WGCV-W3-003 index portion). (3) BC-INDEX version pin updated v4.17 → v4.26 (two occurrences: overview line 25 and Wave Summary line 105; closes WGCV-W3-005). (4) STORY-INDEX version bumped v1.71 → v1.72; timestamp updated 2026-04-27 → 2026-05-01; producer updated story-writer → state-manager. Note: S-3.2.03 was already annotated [MERGED] in prior burst; remaining 36 story status flips executed as part of same W3-FIX-G burst. D-182. STORY-INDEX v1.71 → v1.72.
- **W3.1 state hygiene burst (2026-05-02):** Wave 3.1 fix wave CLOSED (5 PRs merged #113-#117). (1) S-3.1.06-ImplPhase registered in E-3.1 epic table + Full Story List (PR #117 cda17ed4 2026-05-02). (2) MERGED annotations added to 5 W3.1 stories: W3-FIX-SEC-001 (PR #113 59803de3), W3-FIX-SEC-003 (PR #114 a68d1748), W3-FIX-CODE-003 (PR #115 bbe79480), W3-FIX-CODE-001 (PR #116 702d10b5), S-3.1.06-ImplPhase (PR #117 cda17ed4). (3) BC columns updated from (TBD) to actual BC IDs from story frontmatter. (4) BC Traceability Matrix updated: BC-3.1.001/002/003/004 += S-3.1.06-ImplPhase; BC-3.2.001 += W3-FIX-SEC-001; BC-3.2.002 += W3-FIX-CODE-003; BC-3.3.001/004 += W3-FIX-SEC-003; BC-3.5.001/002 += W3-FIX-SEC-001 + W3-FIX-CODE-001; BC-3.6.001 += W3-FIX-CODE-001. Total stories 119 → 120. STORY-INDEX v1.73 → v1.74. D-184.
- **W3.2 fix wave story-writer burst (2026-05-02):** Filed 2 new Wave 3.2 fix stories. (1) W3-FIX-CREDS-001 (prism-credentials CredentialStoreOrgId trait body impl; BC-3.2.002; 5 pts) registered in E-3.5 epic table + Full Story List. (2) W3-FIX-CODE-004 (pass-49 cleanup bundle: CR-010..015 + SEC-P2-002/006 + BC-3.5.002 timing; BC-3.5.001/002 + BC-3.6.001 + BC-3.3.004 + BC-3.2.001; 5 pts) registered in E-3.5 epic table + Full Story List. (3) BC Traceability Matrix: BC-3.2.002 += W3-FIX-CREDS-001; BC-3.6.001 += W3-FIX-CODE-004; BC-3.5.001/002 += W3-FIX-CODE-004; BC-3.3.004 += W3-FIX-CODE-004; BC-3.2.001 += W3-FIX-CODE-004. Total stories 120 → 122. STORY-INDEX v1.74 → v1.75. D-185.
- **W3.2 state hygiene burst (2026-05-02):** Wave 3.2 fix wave CLOSED (4 PRs merged #118-#121). MERGED annotations added to 4 W3.2 stories: W3-FIX-CODE-004 (PR #118 618ad644), W3-FIX-SEC-002 (PR #119 f89e7044), W3-FIX-CODE-002 (PR #120 a7f0d374), W3-FIX-CREDS-001 (PR #121 9d04235d). BC columns updated from deferred placeholders to actual anchor_bcs: W3-FIX-SEC-002 → BC-3.5.001,BC-3.2.001; W3-FIX-CODE-002 → BC-3.3.001,BC-3.3.004,BC-3.2.005. Story file status: draft → merged for all 4 W3.2 story files. STORY-INDEX v1.75 → v1.76. D-186.
- **W3.3 state hygiene burst (2026-05-02):** Pass-50 integration gate state hygiene. (1) +Nt placeholders resolved for all W3-FIX-* stories and S-3.1.06-ImplPhase: SEC-001 +12t, SEC-002 +12t, SEC-003 +3t, CODE-001 +2t, CODE-002 +31t, CODE-003 +3t, CODE-004 +14t, CREDS-001 +7t, S-3.1.06-ImplPhase +6t. (2) MERGED annotations added to Full Story List for W3-FIX-SEC-002, W3-FIX-CODE-002, W3-FIX-CREDS-001, W3-FIX-CODE-004 (gaps from W3.2 burst). (3) E-3.5 epic header corrected: (10 stories) → (12 stories). (4) BC Traceability Matrix: BC-3.2.001 += W3-FIX-SEC-002; BC-3.2.005 += W3-FIX-CODE-002; BC-3.3.001 += W3-FIX-CODE-002; BC-3.3.004 += W3-FIX-CODE-002; BC-3.5.001 += W3-FIX-SEC-002; BC-3.5.002 += W3-FIX-SEC-002. (5) total_stories 122 → 125 (actual enumeration; 3 devx W3-FIX-* stories counted in overview but omitted from prior tally). STORY-INDEX v1.76 → v1.77. D-187.
- **W3.3 fix wave CLOSED (2026-05-02):** Wave 3.3 fix wave delivery complete (2 PRs merged). (1) W3-FIX-SEC-004 (PR #122 4e053105) + W3-FIX-CODE-005 (PR #123 e4be29ae) registered in E-3.5 epic table + Full Story List with MERGED annotations. (2) Story files status: draft → merged for both. (3) E-3.5 epic header: (12 stories) → (14 stories). (4) BC Traceability Matrix: BC-3.2.001 += W3-FIX-CODE-005; BC-3.3.004 += W3-FIX-SEC-004; BC-3.5.001 += W3-FIX-SEC-004 + W3-FIX-CODE-005; BC-3.5.002 += W3-FIX-SEC-004 + W3-FIX-CODE-005; BC-3.6.001 += W3-FIX-CODE-005. (5) total_stories 125 → 127; overview updated. Pass-51 gate dispatch queued. STORY-INDEX v1.77 → v1.78. D-188.
- **W3.4 fix wave story authoring (2026-05-02):** Filed 2 new Wave 3.4 fix stories. (1) W3-FIX-SEC-005 (5-DTU admin-token uniformity: cyberint+jira+nvd+pagerduty+threatintel × post_configure ct_eq + post_reset admin gate = 10 sites; BC-3.5.001,BC-3.5.002; 5 pts; P1) registered in E-3.5 epic table + Full Story List at status: planned. (2) W3-FIX-CODE-006 (CR-023: Armis get_device_activity + get_device_risk org-id guard test coverage; BC-3.5.001; 2 pts; P3) registered in E-3.5 epic table + Full Story List at status: planned. (3) E-3.5 epic header: (14 stories) → (16 stories); wave annotation updated Wave 3.1–3.3 → Wave 3.1–3.4. (4) BC Traceability Matrix: BC-3.5.001 += W3-FIX-SEC-005 + W3-FIX-CODE-006; BC-3.5.002 += W3-FIX-SEC-005. (5) total_stories 127 → 129; overview updated. W3.4 dispatch next. STORY-INDEX v1.78 → v1.79. D-189 (pass-51 not-clean decision already in STATE.md).
- **W3.4-G hygiene burst (2026-05-02):** W3.4 fix wave CLOSED (2 PRs merged). (1) MERGED annotations + test counts added: W3-FIX-SEC-004 [MERGED PR #122 4e053105 2026-05-02 +18t], W3-FIX-CODE-005 [MERGED PR #123 e4be29ae 2026-05-02 +14t], W3-FIX-SEC-005 [MERGED PR #125 ba3b10c7 2026-05-02 +21t], W3-FIX-CODE-006 [MERGED PR #124 981e17d4 2026-05-02 +6t] — in both E-3.5 epic table and Full Story List. (2) WGCV3-P3-007 CLOSED: W3-FIX-CODE-002 epic-view BC column corrected from `BC-3.3.001,BC-3.3.004,BC-3.2.005` to `BC-3.3.001,BC-3.3.004,BC-3.5.001,BC-3.5.002,BC-3.1.002` to match story frontmatter SoT. (3) BC Traceability Matrix: BC-3.1.002 += W3-FIX-CODE-002; BC-3.2.005 -= W3-FIX-CODE-002 (error from D-186 anchor_bcs mismatch); BC-3.5.001 += W3-FIX-CODE-002; BC-3.5.002 += W3-FIX-CODE-002. STORY-INDEX v1.79 → v1.80. D-192.

Every story contains: narrative, behavioral contracts table, numbered tasks, acceptance
criteria (Given/When/Then), verification properties, and notes. No story exceeds 5
estimated days. No story's estimated context exceeds 30% of the implementing agent's
context window.

---

## Wave Summary

| Wave | Crates | Stories | BCs | Theme |
|------|--------|---------|-----|-------|
| 0 | devops, prism-dtu-common, prism-dtu-threatintel, prism-dtu-nvd | 5 | 0 (infra) | Developer + Test Infrastructure (threat-intel DTUs: must precede wave-1 S-1.14) |
| 1 | prism-core, prism-ocsf, prism-credentials, prism-security, prism-spec-engine, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint, prism-dtu-armis, prism-dtu-demo-server | 20 | 69 (raw; 4 stories with 0 BCs) | Foundation + Pure Domain + Sensor DTUs + Demo Harness (precede wave-3 consumers) |
| 2 | prism-storage, prism-audit, prism-sensors, prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-jira | 11 | 30 | Infrastructure + Adapters + Action DTUs (precede wave-4 S-4.08) |
| 3 | prism-query, prism-dtu-datadog, prism-dtu-splunk-hec, prism-dtu-elasticsearch, prism-dtu-otlp | 17 | 28 | Query Engine (incl. write ops + osquery enhancements) + Log-Forwarding DTUs (precede wave-5 S-5.09) |
| 4 | prism-operations | 8 | 45 | Operations |
| 5 | prism-mcp, prism-audit | 10 | 56 | MCP Server + Config + Diagnostics + Log Forwarding + Audit Forwarding |
| 6 | prism-bin | 5 | 15 | Binary + E2E |

Wave 0: devops (S-0.01, S-0.02, no deps) + DTU common (S-6.06, depends on S-0.02) + threat-intel DTUs (S-6.14, S-6.15, depend on S-6.06). S-6.14/S-6.15 must be wave 0 because they block wave-1 S-1.14 (infusion spec loading).
Wave 1: product foundation stories (S-1.01–S-1.15, no product deps beyond S-1.01) + sensor DTUs (S-6.07–S-6.10, depend on S-6.06 wave-0) + demo harness (S-6.20, depends on S-6.06/S-6.07/S-6.08/S-6.09/S-6.10/S-6.14/S-6.15 all merged). Sensor DTUs must precede wave-3 consumers S-3.02, S-3.06, S-3.07. S-6.20 blocks nothing and is dispatachable immediately (all deps merged 2026-04-22).
Wave 2: infrastructure+adapters (S-2.01–S-2.08, depend on wave-1) + action DTUs (S-6.11–S-6.13, depend on S-6.06 wave-0). Action DTUs must precede wave-4 S-4.08 and wave-5 S-5.06.
Wave 3: query engine (S-3.01–S-3.13, depend on wave-2) + log-forwarding DTUs (S-6.16–S-6.19, depend on S-6.06 wave-0). Log-forwarding DTUs must precede wave-5 S-5.09.
Waves 4-6 follow in order. All dependency chains are acyclic (validated by topological sort below).
Per-wave BC counts are raw story-BC assignments (sum=243 across all waves: 0+69+30+28+45+56+15).
Some BCs appear in multiple stories (e.g., BC-2.04.001 → S-1.08 AND S-3.07; BC-2.16.001 → S-1.11 AND S-1.13),
so the raw sum exceeds the unique count. Unique active BCs = 222 (per BC-INDEX.md v4.27, 222 active contracts: 200 Wave 1-2 + 22 Wave 3).
Note: DTU stories have 0 BCs. Per user directive Option 2 (DTU-first), product stories that require DTU
clones as test fixtures now have explicit depends_on edges to their DTU prerequisites. DTU stories are
distributed across waves 0-3 based on their earliest product consumer's wave.

**NOTE on wave vs. topological scheduling:** Wave assignments are grouped by crate boundary
for organizational clarity. The topological sort (below) shows that some stories can start
earlier than their wave number suggests — e.g., S-3.01 (Wave 3) and S-2.01 (Wave 2) are
both in topological Layer 2, meaning they can begin as soon as S-1.01 (Layer 1) completes. Teams
pursuing maximum parallelism should schedule by topological layer, not wave number.

---

## Wave 3 — Multi-Tenant DTU Stories

> **Status: ALL `draft` — NOT ready for implementation.** Phase 3.A spec convergence (3 clean adversary passes + consistency-validator + spec-reviewer + drift check) and human approval required before any Wave 3 story may be dispatched. Per D-045.

### Pre-Wave-3 Quick Fix-PRs (E-3.0)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.0.01 | lefthook: fix pre-commit fmt hook (cargo fmt --all --check) [MERGED PR #73 6696e374 2026-04-28 +1t] | E-3.0 | (none) | Platform Engineering | 1 | -- |
| S-3.0.02 | prism-core: register DTU_DEFAULT_MODE registry (10-entry DtuRegistryEntry slice) per ADR-007 §2.3 [MERGED PR #74 373baf78 2026-04-28 +17t] | E-3.0 | BC-3.2.005 | Platform Engineering | 2 | -- |

### E-3.1: OrgId/OrgSlug Split + Translation Layer (8 stories)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.1.01 | prism-core: declare OrgId(Uuid v7) newtype via uuid_v7_newtype! macro [MERGED PR #81 39125a3e 2026-04-29 +11t] | E-3.1 | BC-3.1.001 | Application Development | 1 | -- |
| S-3.1.02 | workspace: rename TenantId → OrgSlug across all crates [MERGED PR #93 8532d204 2026-04-29 +0t] | E-3.1 | BC-3.1.001 | Application Development | 3 | S-3.1.01 |
| S-3.1.03 | prism-core: implement OrgRegistry (bijective BiMap, resolve/slug_for/register) [MERGED PR #94 3e961bd1 2026-04-29 +35t] | E-3.1 | BC-3.1.001,BC-3.1.003,BC-3.1.004 | Application Development | 5 | S-3.1.01,S-3.1.02 |
| S-3.1.04 | prism-credentials: migrate credential namespace key from OrgSlug to OrgId [MERGED PR #95 f139238e 2026-04-29 +18t] | E-3.1 | BC-3.2.002 | Application Development | 3 | S-3.1.01,S-3.1.02,S-3.1.03 |
| S-3.1.05 | prism-spec-engine: scope sensor specs per OrgId (resolve slug at user-facing surface) [MERGED PR #98 5e323edd 2026-04-29 +18t] | E-3.1 | BC-3.1.001 | Application Development | 3 | S-3.1.01,S-3.1.02,S-3.1.03 |
| S-3.1.06 | prism-sensors: migrate adapter constructors and fan-out dispatch to OrgId [MERGED PR #99 c2dc67b2 2026-04-30 +17t] | E-3.1 | BC-3.2.001,BC-3.2.004 | Application Development | 5 | S-3.1.01,S-3.1.02,S-3.1.03,S-3.1.04,S-3.1.05 |
| S-3.1.06-ImplPhase | prism-sensors: complete adapter OrgId binding (S-3.1.06 Task 4 follow-on) [MERGED PR #117 cda17ed4 2026-05-02 +6t] | E-3.1 | BC-3.1.001,BC-3.1.002,BC-3.1.003,BC-3.1.004 | Application Development | 8 | -- |
| S-3.1.07 | prism-audit: add org_id + org_slug to AuditEntry; SHA-256 aql_hash [MERGED PR #96 fd39e94c 2026-04-29 +18t] | E-3.1 | BC-3.1.001,BC-3.1.002 | Application Development | 5 | S-3.1.01,S-3.1.02,S-3.1.03 |

### E-3.2: Multi-Tenant DTU State Segregation (8 stories)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.2.01 | prism-dtu-claroty: Multi-tenant state segregation — (OrgId, String) re-keying [MERGED PR #86 214a9780 2026-04-29 +17t] | E-3.2 | BC-3.2.001,BC-3.2.003 | Application Development | 5 | S-6.08 |
| S-3.2.02 | prism-dtu-armis: Multi-tenant state segregation — (OrgId, String) re-keying [MERGED PR #88 65cb3269 2026-04-29 +11t] | E-3.2 | BC-3.2.001 | Application Development | 5 | S-6.10 |
| S-3.2.03 | prism-dtu-crowdstrike: Multi-tenant state segregation — containment + detection store re-keying (D-048) [MERGED PR #85 5f087c8f 2026-04-29 +14t] | E-3.2 | BC-3.2.001,BC-3.2.003 | Application Development | 5 | S-6.07 |
| S-3.2.04 | prism-dtu-cyberint: Multi-tenant state segregation — alert_store + session_store re-keying [MERGED PR #87 48c407f3 2026-04-29 +15t] | E-3.2 | BC-3.2.001,BC-3.2.003 | Application Development | 5 | S-6.09 |
| S-3.2.05 | prism-dtu-slack: Shared-mode OrgId ingress tagging [MERGED PR #89 df59b0d0 2026-04-29 +7t] | E-3.2 | BC-3.2.004,BC-3.2.005 | Application Development | 3 | S-6.11 |
| S-3.2.06 | prism-dtu-pagerduty: Shared-mode OrgId ingress tagging [MERGED PR #90 7deb7fd7 2026-04-29 +8t] | E-3.2 | BC-3.2.004,BC-3.2.005 | Application Development | 3 | S-6.12,S-3.2.05 |
| S-3.2.07 | prism-dtu-jira: Shared-mode OrgId ingress tagging [MERGED PR #91 9c1ecec0 2026-04-29 +8t] | E-3.2 | BC-3.2.004,BC-3.2.005 | Application Development | 3 | S-6.13,S-3.2.05 |
| S-3.2.08 | prism-query: scope CrowdStrike pagination session IDs per OrgId (D-048) [MERGED PR #102 5ec44bdd 2026-04-30 +28t] | E-3.2 | BC-3.2.003 | Application Development | 5 | S-3.1.06,S-3.2.03 |

### E-3.3: Customer Config Schema + Harness (6 stories)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.3.01 | prism-customer-config: TOML schema, parser, and startup validator [MERGED PR #92 7e5cc790 2026-04-29 +46t] | E-3.3 | BC-3.3.001,BC-3.3.002,BC-3.3.003,BC-3.3.004 | Application Development | 8 | S-1.06 |
| S-3.3.02 | OrgRegistry boot from customers/*.toml at startup [MERGED PR #97 5b38103e 2026-04-29 +18t] | E-3.3 | BC-3.1.003,BC-3.1.004,BC-3.3.004 | Application Development | 5 | S-3.3.01 |
| S-3.3.03 | prism-dtu-harness: logical isolation mode + crash detection + failure injection [MERGED PR #101 7245b783 2026-04-30 +47t] | E-3.3 | BC-3.5.001,BC-3.6.001,BC-3.6.002 | Application Development | 13 | S-3.3.01,S-3.3.02,S-6.06 |
| S-3.3.04 | prism-dtu-harness: network isolation mode (per-port, real HTTP) [MERGED PR #103 7ad3c3cd 2026-04-30 +19t] | E-3.3 | BC-3.5.002 | Application Development | 8 | S-3.3.03 |
| S-3.3.05 | prism-dtu-harness: builder ergonomics, per-test overrides, and documentation [MERGED PR #104 7666fd9b 2026-04-30 +19t] | E-3.3 | BC-3.5.001,BC-3.5.002,BC-3.6.001 | Application Development | 5 | S-3.3.04 |
| S-3.3.06 | prism-spec-engine: reload_config detects and warns on DTU mode changes without applying them [MERGED PR #100 f3b14691 2026-04-30 +17t] | E-3.3 | BC-3.2.005 | Application Development | 3 | S-3.3.02 |

### E-3.4: Test Migration to Harness (5 stories)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.4.01 | Migrate prism-dtu-claroty tests to prism-dtu-harness [MERGED PR #107 a724f94e 2026-04-30 +62t] | E-3.4 | BC-3.5.001,BC-3.5.002 | Application Development | 5 | S-3.3.05,S-6.08 |
| S-3.4.02 | Migrate prism-dtu-armis tests to prism-dtu-harness [MERGED PR #108 eee5f8ec 2026-04-30 +63t] | E-3.4 | BC-3.5.001,BC-3.5.002 | Application Development | 5 | S-3.3.05,S-6.10 |
| S-3.4.03 | Migrate prism-dtu-crowdstrike tests to prism-dtu-harness [MERGED PR #109 28722c47 2026-04-30 +63t] | E-3.4 | BC-3.5.001,BC-3.5.002 | Application Development | 5 | S-3.3.05,S-6.07 |
| S-3.4.04 | Migrate prism-dtu-cyberint tests to prism-dtu-harness [MERGED PR #111 2c77deeb 2026-04-30 +63t] | E-3.4 | BC-3.5.001,BC-3.5.002,BC-3.6.001 | Application Development | 5 | S-3.3.05,S-6.09 |
| S-3.4.05 | Migrate prism-dtu-slack/pagerduty/jira tests to prism-dtu-harness (shared-mode) [MERGED PR #110 881cf01e 2026-04-30 +62t] | E-3.4 | BC-3.2.004,BC-3.3.001,BC-3.5.001 | Application Development | 5 | S-3.3.05,S-6.11,S-6.12,S-6.13 |

### E-3.5: src/ Convention Sweep + devx Fix Wave (Wave 3.1–3.4) (16 stories)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.5.01 | Workspace src/ convention sweep — check-crate-layout.sh + CI gate + CRATE-LAYOUT.md [MERGED PR #82 c4287aef 2026-04-29 +36t] | E-3.5 | BC-3.7.001 | Platform Engineering | 3 | -- |
| W3-FIX-WIN-001 | prism-dtu-harness: cross-platform fix for drop_releases_ports test (Windows winsock) [MERGED PR #105 ea90c9ee 2026-04-30 +0t] | E-3.5 | BC-3.5.001 | Platform Engineering | 2 | -- |
| W3-FIX-LEFTHOOK-001 | Pre-push lefthook gate tuning — proptest case reduction, audit/deny CI-only, semver-checks pre-tag [MERGED PR #106 7418f269 2026-04-30 +0t] | E-3.5 | (none) | Platform Engineering | 2 | -- |
| W3-FIX-CI-001 | CI wall-clock optimization — cargo-nextest, per-platform PROPTEST_CASES, mold linker [MERGED PR #112 a3bd5a0f 2026-04-30 +0t] | E-3.5 | (none) | Platform Engineering | 3 | -- |
| W3-FIX-SEC-001 | DTU clones: bind OrgId to clone instance — reject mismatched X-Org-Id header [MERGED PR #113 59803de3 2026-05-01 +12t] | E-3.5 | BC-3.5.001,BC-3.5.002,BC-3.2.001 | Security Engineering | 5 | -- |
| W3-FIX-SEC-002 | /dtu/reset admin token authentication [MERGED PR #119 f89e7044 2026-05-02 +12t] | E-3.5 | BC-3.5.001,BC-3.2.001 | Security Engineering | 3 | W3-FIX-SEC-001 |
| W3-FIX-SEC-003 | prism-customer-config: path canonicalization + E-CFG-018 SpecPathTraversal rejection [MERGED PR #114 a68d1748 2026-05-01 +3t] | E-3.5 | BC-3.3.001,BC-3.3.004 | Security Engineering | 3 | -- |
| W3-FIX-CODE-001 | prism-dtu-harness: per-DtuType failure scoping and honest Drop semantics [MERGED PR #116 702d10b5 2026-05-01 +2t] | E-3.5 | BC-3.5.001,BC-3.5.002,BC-3.6.001 | Application Development | 5 | -- |
| W3-FIX-CODE-002 | prism-customer-config: config validation hardening + dispatch hygiene [MERGED PR #120 a7f0d374 2026-05-02 +31t] | E-3.5 | BC-3.3.001,BC-3.3.004,BC-3.5.001,BC-3.5.002,BC-3.1.002 | Application Development | 5 | W3-FIX-SEC-003 |
| W3-FIX-CODE-003 | prism-credentials: implement KeyringBackend::CredentialStoreOrgId — replace todo!() stubs [MERGED PR #115 bbe79480 2026-05-01 +3t] | E-3.5 | BC-3.2.002 | Application Development | 3 | -- |
| W3-FIX-CREDS-001 | prism-credentials: implement CredentialStoreOrgId trait bodies — replace todo!() stubs [MERGED PR #121 9d04235d 2026-05-02 +7t] | E-3.5 | BC-3.2.002 | Application Development | 5 | -- |
| W3-FIX-CODE-004 | prism-dtu-harness/sensors/config: pass-49 hygiene bundle — CR-010..015, SEC-P2-002/006, BC-3.5.002 timing [MERGED PR #118 618ad644 2026-05-02 +14t] | E-3.5 | BC-3.5.001,BC-3.5.002,BC-3.6.001,BC-3.3.004,BC-3.2.001 | Application Development | 5 | -- |
| W3-FIX-SEC-004 | prism-customer-config + DTU clones: TOML inline-table redaction and constant-time token comparison [MERGED PR #122 4e053105 2026-05-02 +18t] | E-3.5 | BC-3.3.004,BC-3.5.001,BC-3.5.002 | Security Engineering | 3 | -- |
| W3-FIX-CODE-005 | DTU harness + Armis/CrowdStrike: sibling poll-backoff propagation and missing org-id guards [MERGED PR #123 e4be29ae 2026-05-02 +14t] | E-3.5 | BC-3.5.001,BC-3.5.002,BC-3.2.001,BC-3.6.001 | Application Development | 5 | -- |
| W3-FIX-SEC-005 | 5-DTU admin-token uniformity — constant-time comparison + post_reset gate (cyberint/jira/nvd/pagerduty/threatintel) [MERGED PR #125 ba3b10c7 2026-05-02 +21t] | E-3.5 | BC-3.5.001,BC-3.5.002 | Security Engineering | 5 | -- |
| W3-FIX-CODE-006 | Armis activity/risk endpoint org-id guard test coverage (CR-023 closure) [MERGED PR #124 981e17d4 2026-05-02 +6t] | E-3.5 | BC-3.5.001 | Application Development | 2 | -- |

### E-3.6: HS-006/HS-007 Holdout Refresh (2 stories)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.6.01 | HS-006 multi-tenant state recovery holdout refresh — re-anchor to Wave 3 BCs [MERGED PR #83 36a40f59 2026-04-29 +5t] | E-3.6 | BC-3.2.001,BC-3.2.003,BC-3.5.001,BC-3.6.001,BC-3.6.002 | Application Development | 2 | -- |
| S-3.6.02 | HS-007 multi-tenant cross-repo failure holdout refresh — re-anchor to Wave 3 BCs [MERGED PR #84 73d1c348 2026-04-29 +5t] | E-3.6 | BC-3.5.001,BC-3.5.002,BC-3.6.001,BC-3.6.002 | Application Development | 2 | -- |

### E-3.7: Multi-Tenant Data Generator (6 stories)

| Story ID | Title | Epic | BCs Anchored | Track | Pts | Depends On |
|----------|-------|------|--------------|-------|-----|------------|
| S-3.7.00 | Schema derivation: Armis (armis-sdk-go) + CrowdStrike (gofalcon) → Rust types [MERGED PR #75 79f67c93 2026-04-29 +25t] | E-3.7 | BC-3.4.002,BC-3.4.003 | Application Development | 5 | -- |
| S-3.7.01 | Archetype catalog + GenOpts API (prism-dtu-common generator module, D-056) [MERGED PR #76 0bb7735d 2026-04-29 +39t] | E-3.7 | BC-3.4.001,BC-3.4.002,BC-3.4.003 | Application Development | 5 | -- |
| S-3.7.02 | Claroty fixture generator — all 8 archetypes from poller-bear specs.json [MERGED PR #79 6a333785 2026-04-29 +24t] | E-3.7 | BC-3.4.001,BC-3.4.002,BC-3.4.003,BC-3.4.004 | Application Development | 5 | S-3.7.01 |
| S-3.7.03 | Cyberint fixture generator — all 8 archetypes from 4 poller-express specs [MERGED PR #77 c7a6f4df 2026-04-29 +35t] | E-3.7 | BC-3.4.001,BC-3.4.002,BC-3.4.003,BC-3.4.004 | Application Development | 5 | S-3.7.01 |
| S-3.7.04 | Armis fixture generator — all 8 archetypes from S-3.7.00 derived schemas [MERGED PR #78 45732009 2026-04-29 +37t] | E-3.7 | BC-3.4.001,BC-3.4.002,BC-3.4.003,BC-3.4.004 | Application Development | 5 | S-3.7.00,S-3.7.01 |
| S-3.7.05 | CrowdStrike fixture generator — all 8 archetypes, 2-step pagination, OAuth2 [MERGED PR #80 89fa8dea 2026-04-29 +37t] | E-3.7 | BC-3.4.001,BC-3.4.002,BC-3.4.003,BC-3.4.004 | Application Development | 5 | S-3.7.00,S-3.7.01 |

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
| S-6.20 | Unified Multi-Clone Demo Harness [W1] | prism-dtu-demo-server | 0 | -- | 3 | S-6.06,S-6.07,S-6.08,S-6.09,S-6.10,S-6.14,S-6.15 |
| S-6.11 | DTU for Slack Webhook API — L2 (stateful) [W2] [MERGED PR #57 6fd20860 2026-04-25 +14t] | prism-dtu-slack | 0 | -- | 2 | S-6.06 |
| S-6.12 | DTU for PagerDuty Events API v2 — L3 (behavioral) [W2] [MERGED PR #55 13579505 2026-04-25 +17t] | prism-dtu-pagerduty | 0 | -- | 4 | S-6.06 |
| S-6.13 | DTU for Jira REST API v3 — L3 (behavioral) [W2] [MERGED PR #56 81adf74a 2026-04-25 +28t] | prism-dtu-jira | 0 | -- | 5 | S-6.06 |
| S-6.16 | DTU for Datadog Logs API — L2 (stateful) [W3] | prism-dtu-datadog | 0 | -- | 2 | S-6.06 |
| S-6.17 | DTU for Splunk HTTP Event Collector — L2 (stateful) [W3] | prism-dtu-splunk-hec | 0 | -- | 2 | S-6.06 |
| S-6.18 | DTU for Elasticsearch Bulk API — L2 (stateful) [W3] | prism-dtu-elasticsearch | 0 | -- | 3 | S-6.06 |
| S-6.19 | DTU for OTLP/HTTP Log Ingestion — L2 (stateful) [W3] | prism-dtu-otlp | 0 | -- | 3 | S-6.06 |
| S-1.01 | Foundational Types (OrgSlug [TenantId legacy alias], PrismError, StorageDomain) | prism-core | 0 | VP-001 | 2 | -- |
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
| S-2.01 | RocksDB Initialization and Domain Operations [MERGED PR #43 0d24ab79 2026-04-24 +24t] | prism-storage | 3 | -- | 3 | S-1.01 |
| S-2.02 | Audit Buffer and Watchdog [MERGED PR #52 9de6b3d8 2026-04-25 +25t] | prism-storage | 5 | VP-058 | 2 | S-2.01 |
| S-2.03 | Decorators and Internal Tables [MERGED PR #53 f13b5c76 2026-04-25 +19t] | prism-storage | 3 | -- | 2 | S-2.01,S-1.02 |
| S-2.04 | Audit Entry Construction and Compliance [MERGED PR #58 ab1f57b2 2026-04-25 +72t] | prism-audit | 6 | -- | 3 | S-2.01,S-2.02 |
| S-2.05 | Specialized Audit Events [MERGED PR #59 c828e8af 2026-04-26 +35t RED_RATIO=54.3%] | prism-audit | 4 | -- | 1 | S-2.04 |
| S-2.06 | DataSource Trait and Auth Patterns [MERGED PR #54 0b194cb4 2026-04-25 +51t] | prism-sensors | 4 | -- | 3 | S-1.06,S-1.11 |
| S-2.07 | Per-Sensor Auth and Pagination [MERGED PR #60 26d0954b 2026-04-26 +56t RED_RATIO=83.9%] | prism-sensors | 5 | -- | 3 | S-2.06 |
| S-2.08 | Event Table Abstraction and Local Buffering [MERGED PR #61 0be11cd6 2026-04-26 +92t RED_RATIO=54.3% prism-query-crate-created **WAVE-2-CLOSED** spec-v1.9-W2-P2-A-005-schema-hygiene-fix] | prism-sensors, prism-query | 0 | -- | 3 | S-2.06,S-2.01,S-1.11 |
| S-3.01 | PrismQL Parser (Filter + SQL + Pipe) | prism-query | 4 | VP-014,015,021 | 3 | S-1.01 |
| S-3.02 | Query Tool and Materialization [spec-v1.7-inherited-AC-5b-from-S-2.08] | prism-query | 6 | VP-031 | 3 | S-3.01,S-2.06,S-1.04,S-2.01,S-2.03,S-6.08,S-6.09,S-6.10 |
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
| S-4.01 | Schedule CRUD and Execution Loop [v1.10 ADR-013] | prism-operations | 5 | VP-026,030 | 3 | S-3.02,S-2.01 |
| S-4.02 | Differential Results and Packs [v1.7 ADR-015] | prism-operations | 3 | VP-019 | 2 | S-4.01 |
| S-4.03 | Detection Rule Loading and Compilation [v1.9 ADR-015] | prism-operations | 8 | VP-018 | 3 | S-3.02,S-1.08,S-2.01 |
| S-4.04 | Detection Evaluation (Single/Correlation/Sequence) [v1.8 ADR-015] | prism-operations | 5 | VP-027 | 3 | S-4.03 |
| S-4.05 | Alert Generation [v1.10 ADR-016] | prism-operations | 4 | VP-028 | 1 | S-4.04 |
| S-4.06 | Case Management [v1.13 ADR-017,ADR-019] | prism-operations | 9 | VP-052,053,054,060 | 3 | S-4.05,S-2.01 |
| S-4.07 | Case Metrics and Acknowledge Alert [v1.8 ADR-017] | prism-operations | 3 | -- | 2 | S-4.06 |
| S-4.08 | Action Delivery Framework [v1.19 ADR-016,ADR-019] | prism-operations | 9 | VP-044,VP-045,VP-046,VP-047,VP-137,VP-144 | 3 | S-4.05,S-4.06,S-4.01,S-1.15,S-6.11,S-6.12,S-6.13 |
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
| S-3.0.01 | lefthook: fix pre-commit fmt hook (cargo fmt --all --check) [MERGED PR #73 6696e374 2026-04-28 +1t] | devops | 0 | -- | 1 | -- |
| S-3.0.02 | prism-core: register DTU_DEFAULT_MODE registry (10-entry DtuRegistryEntry slice) per ADR-007 §2.3 [MERGED PR #74 373baf78 2026-04-28 +17t] | prism-core | 1 | -- | 1 | -- |
| S-3.1.01 | prism-core: declare OrgId(Uuid v7) newtype via uuid_v7_newtype! macro [MERGED PR #81 39125a3e 2026-04-29 +11t] | prism-core | 1 | -- | 1 | -- |
| S-3.1.02 | workspace: rename TenantId → OrgSlug across all crates [MERGED PR #93 8532d204 2026-04-29 +0t] | workspace | 1 | -- | 2 | S-3.1.01 |
| S-3.1.03 | prism-core: implement OrgRegistry (bijective BiMap, resolve/slug_for/register) [MERGED PR #94 3e961bd1 2026-04-29 +35t] | prism-core | 3 | -- | 3 | S-3.1.01,S-3.1.02 |
| S-3.1.04 | prism-credentials: migrate credential namespace key from OrgSlug to OrgId [MERGED PR #95 f139238e 2026-04-29 +18t] | prism-credentials | 1 | -- | 2 | S-3.1.01,S-3.1.02,S-3.1.03 |
| S-3.1.05 | prism-spec-engine: scope sensor specs per OrgId (resolve slug at user-facing surface) [MERGED PR #98 5e323edd 2026-04-29 +18t] | prism-spec-engine | 1 | -- | 2 | S-3.1.01,S-3.1.02,S-3.1.03 |
| S-3.1.06 | prism-sensors: migrate adapter constructors and fan-out dispatch to OrgId [MERGED PR #99 c2dc67b2 2026-04-30 +17t] | prism-sensors | 2 | -- | 3 | S-3.1.01,S-3.1.02,S-3.1.03,S-3.1.04,S-3.1.05 |
| S-3.1.07 | prism-audit: add org_id + org_slug to AuditEntry; SHA-256 aql_hash [MERGED PR #96 fd39e94c 2026-04-29 +18t] | prism-audit | 2 | -- | 3 | S-3.1.01,S-3.1.02,S-3.1.03 |
| S-3.2.01 | prism-dtu-claroty: Multi-tenant state segregation — (OrgId, String) re-keying [MERGED PR #86 214a9780 2026-04-29 +17t] | prism-dtu-claroty | 2 | -- | 3 | S-6.08 |
| S-3.2.02 | prism-dtu-armis: Multi-tenant state segregation — (OrgId, String) re-keying [MERGED PR #88 65cb3269 2026-04-29 +11t] | prism-dtu-armis | 1 | -- | 3 | S-6.10 |
| S-3.2.03 | prism-dtu-crowdstrike: Multi-tenant state segregation — containment + detection store re-keying [MERGED PR #85 5f087c8f 2026-04-29 +14t] | prism-dtu-crowdstrike | 2 | -- | 3 | S-6.07 |
| S-3.2.04 | prism-dtu-cyberint: Multi-tenant state segregation — alert_store + session_store re-keying [MERGED PR #87 48c407f3 2026-04-29 +15t] | prism-dtu-cyberint | 2 | -- | 3 | S-6.09 |
| S-3.2.05 | prism-dtu-slack: Shared-mode OrgId ingress tagging [MERGED PR #89 df59b0d0 2026-04-29 +7t] | prism-dtu-slack | 2 | -- | 2 | S-6.11 |
| S-3.2.06 | prism-dtu-pagerduty: Shared-mode OrgId ingress tagging [MERGED PR #90 7deb7fd7 2026-04-29 +8t] | prism-dtu-pagerduty | 2 | -- | 2 | S-6.12,S-3.2.05 |
| S-3.2.07 | prism-dtu-jira: Shared-mode OrgId ingress tagging [MERGED PR #91 9c1ecec0 2026-04-29 +8t] | prism-dtu-jira | 2 | -- | 2 | S-6.13,S-3.2.05 |
| S-3.2.08 | prism-query: scope CrowdStrike pagination session IDs per OrgId (D-048) [MERGED PR #102 5ec44bdd 2026-04-30 +28t] | prism-query | 1 | VP-084 | 2 | S-3.1.06,S-3.2.03 |
| S-3.3.01 | prism-customer-config: TOML schema, parser, and startup validator [MERGED PR #92 7e5cc790 2026-04-29 +46t] | prism-customer-config | 3 | -- | 3 | S-1.06 |
| S-3.3.02 | OrgRegistry boot from customers/*.toml at startup [MERGED PR #97 5b38103e 2026-04-29 +18t] | prism-customer-config | 3 | -- | 2 | S-3.3.01 |
| S-3.3.03 | prism-dtu-harness: logical isolation mode + crash detection + failure injection [MERGED PR #101 7245b783 2026-04-30 +47t] | prism-dtu-harness | 3 | -- | 5 | S-3.3.01,S-3.3.02,S-6.06 |
| S-3.3.04 | prism-dtu-harness: network isolation mode (per-port, real HTTP) [MERGED PR #103 7ad3c3cd 2026-04-30 +19t] | prism-dtu-harness | 1 | -- | 3 | S-3.3.03 |
| S-3.3.05 | prism-dtu-harness: builder ergonomics, per-test overrides, and documentation [MERGED PR #104 7666fd9b 2026-04-30 +19t] | prism-dtu-harness | 3 | -- | 2 | S-3.3.04 |
| S-3.3.06 | prism-spec-engine: reload_config detects and warns on DTU mode changes [MERGED PR #100 f3b14691 2026-04-30 +17t] | prism-spec-engine | 1 | VP-094 | 1 | S-3.3.02 |
| S-3.4.01 | Migrate prism-dtu-claroty tests to prism-dtu-harness [MERGED PR #107 a724f94e 2026-04-30 +62t] | prism-dtu-claroty | 2 | -- | 2 | S-3.3.05,S-6.08 |
| S-3.4.02 | Migrate prism-dtu-armis tests to prism-dtu-harness [MERGED PR #108 eee5f8ec 2026-04-30 +63t] | prism-dtu-armis | 2 | -- | 2 | S-3.3.05,S-6.10 |
| S-3.4.03 | Migrate prism-dtu-crowdstrike tests to prism-dtu-harness [MERGED PR #109 28722c47 2026-04-30 +63t] | prism-dtu-crowdstrike | 2 | -- | 2 | S-3.3.05,S-6.07 |
| S-3.4.04 | Migrate prism-dtu-cyberint tests to prism-dtu-harness [MERGED PR #111 2c77deeb 2026-04-30 +63t] | prism-dtu-cyberint | 3 | -- | 2 | S-3.3.05,S-6.09 |
| S-3.4.05 | Migrate prism-dtu-slack/pagerduty/jira tests to prism-dtu-harness (shared-mode) [MERGED PR #110 881cf01e 2026-04-30 +62t] | prism-dtu-slack/pd/jira | 3 | -- | 2 | S-3.3.05,S-6.11,S-6.12,S-6.13 |
| S-3.5.01 | Workspace src/ convention sweep — check-crate-layout.sh + CI gate + CRATE-LAYOUT.md [MERGED PR #82 c4287aef 2026-04-29 +36t] | workspace | 1 | -- | 2 | -- |
| S-3.6.01 | HS-006 multi-tenant state recovery holdout refresh — re-anchor to Wave 3 BCs [MERGED PR #83 36a40f59 2026-04-29 +5t] | prism-dtu-harness | 5 | -- | 1 | -- |
| S-3.6.02 | HS-007 multi-tenant cross-repo failure holdout refresh — re-anchor to Wave 3 BCs [MERGED PR #84 73d1c348 2026-04-29 +5t] | prism-dtu-harness | 4 | -- | 1 | -- |
| S-3.7.00 | Schema derivation: Armis (armis-sdk-go) + CrowdStrike (gofalcon) → Rust types [MERGED PR #75 79f67c93 2026-04-29 +25t] | prism-dtu-armis/crowdstrike | 2 | -- | 3 | -- |
| S-3.7.01 | Archetype catalog + GenOpts API (prism-dtu-common generator module, D-056) [MERGED PR #76 0bb7735d 2026-04-29 +39t] | prism-dtu-common | 3 | -- | 3 | -- |
| S-3.7.02 | Claroty fixture generator — all 8 archetypes from poller-bear specs.json [MERGED PR #79 6a333785 2026-04-29 +24t] | prism-dtu-claroty | 4 | -- | 3 | S-3.7.01 |
| S-3.7.03 | Cyberint fixture generator — all 8 archetypes from 4 poller-express specs [MERGED PR #77 c7a6f4df 2026-04-29 +35t] | prism-dtu-cyberint | 4 | -- | 3 | S-3.7.01 |
| S-3.7.04 | Armis fixture generator — all 8 archetypes from S-3.7.00 derived schemas [MERGED PR #78 45732009 2026-04-29 +37t] | prism-dtu-armis | 4 | -- | 3 | S-3.7.00,S-3.7.01 |
| S-3.7.05 | CrowdStrike fixture generator — all 8 archetypes, 2-step pagination, OAuth2 [MERGED PR #80 89fa8dea 2026-04-29 +37t] | prism-dtu-crowdstrike | 4 | -- | 3 | S-3.7.00,S-3.7.01 |
| W3-FIX-WIN-001 | prism-dtu-harness: cross-platform fix for drop_releases_ports test (Windows winsock) [MERGED PR #105 ea90c9ee 2026-04-30 +0t] | prism-dtu-harness | 0 | -- | 0.5 | -- |
| W3-FIX-LEFTHOOK-001 | Pre-push lefthook gate tuning — proptest case reduction, audit/deny CI-only, semver-checks pre-tag [MERGED PR #106 7418f269 2026-04-30 +0t] | devops | 0 | -- | 0.5 | -- |
| W3-FIX-CI-001 | CI wall-clock optimization — cargo-nextest, per-platform PROPTEST_CASES, mold linker [MERGED PR #112 a3bd5a0f 2026-04-30 +0t] | devops | 0 | -- | 1 | -- |
| S-3.1.06-ImplPhase | prism-sensors: complete adapter OrgId binding (S-3.1.06 Task 4 follow-on) [MERGED PR #117 cda17ed4 2026-05-02 +6t] | prism-sensors | 4 | -- | 2 | -- |
| W3-FIX-SEC-001 | DTU clones: bind OrgId to clone instance — reject mismatched X-Org-Id header [MERGED PR #113 59803de3 2026-05-01 +12t] | prism-dtu-harness,prism-dtu-* | 3 | -- | 1 | -- |
| W3-FIX-SEC-002 | /dtu/reset admin token authentication [MERGED PR #119 f89e7044 2026-05-02 +12t] | prism-dtu-harness | 2 | -- | 0.5 | W3-FIX-SEC-001 |
| W3-FIX-SEC-003 | prism-customer-config: path canonicalization + E-CFG-018 SpecPathTraversal rejection [MERGED PR #114 a68d1748 2026-05-01 +3t] | prism-customer-config | 2 | -- | 0.5 | -- |
| W3-FIX-CODE-001 | prism-dtu-harness: per-DtuType failure scoping and honest Drop semantics [MERGED PR #116 702d10b5 2026-05-01 +2t] | prism-dtu-harness | 3 | -- | 1 | -- |
| W3-FIX-CODE-002 | prism-customer-config: config validation hardening + dispatch hygiene [MERGED PR #120 a7f0d374 2026-05-02 +31t] | prism-customer-config | 3 | -- | 1 | W3-FIX-SEC-003 |
| W3-FIX-CODE-003 | prism-credentials: implement KeyringBackend::CredentialStoreOrgId — replace todo!() stubs [MERGED PR #115 bbe79480 2026-05-01 +3t] | prism-credentials | 1 | -- | 0.5 | -- |
| W3-FIX-CREDS-001 | prism-credentials: implement CredentialStoreOrgId trait bodies — replace todo!() stubs [MERGED PR #121 9d04235d 2026-05-02 +7t] | prism-credentials | 1 | -- | 2 | -- |
| W3-FIX-CODE-004 | prism-dtu-harness/sensors/config: pass-49 hygiene bundle — CR-010..015, SEC-P2-002/006, BC-3.5.002 timing [MERGED PR #118 618ad644 2026-05-02 +14t] | prism-dtu-harness,prism-sensors,prism-customer-config,prism-dtu-armis,prism-dtu-cyberint | 5 | -- | 3 | -- |
| W3-FIX-SEC-004 | prism-customer-config + DTU clones: TOML inline-table redaction and constant-time token comparison [MERGED PR #122 4e053105 2026-05-02 +18t] | prism-customer-config,prism-dtu-* | 3 | -- | 1 | -- |
| W3-FIX-CODE-005 | DTU harness + Armis/CrowdStrike: sibling poll-backoff propagation and missing org-id guards [MERGED PR #123 e4be29ae 2026-05-02 +14t] | prism-dtu-harness,prism-dtu-armis,prism-dtu-crowdstrike | 5 | -- | 2 | -- |
| W3-FIX-SEC-005 | 5-DTU admin-token uniformity — constant-time comparison + post_reset gate (cyberint/jira/nvd/pagerduty/threatintel) [MERGED PR #125 ba3b10c7 2026-05-02 +21t] | prism-dtu-cyberint,prism-dtu-jira,prism-dtu-nvd,prism-dtu-pagerduty,prism-dtu-threatintel | 2 | -- | 1 | -- |
| W3-FIX-CODE-006 | Armis activity/risk endpoint org-id guard test coverage (CR-023 closure) [MERGED PR #124 981e17d4 2026-05-02 +6t] | prism-dtu-armis | 1 | -- | 0.5 | -- |

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
| BC-3.1.001 | S-3.1.01, S-3.1.02, S-3.1.03, S-3.1.05, S-3.1.07, S-3.1.06-ImplPhase |
| BC-3.1.002 | S-3.1.07, S-3.1.06-ImplPhase, W3-FIX-CODE-002 |
| BC-3.1.003 | S-3.1.03, S-3.3.02, S-3.1.06-ImplPhase |
| BC-3.1.004 | S-3.1.03, S-3.3.02, S-3.1.06-ImplPhase |
| BC-3.2.001 | S-3.1.06, S-3.2.01, S-3.2.02, S-3.2.03, S-3.2.04, S-3.6.01, W3-FIX-SEC-001, W3-FIX-SEC-002, W3-FIX-CODE-004, W3-FIX-CODE-005 |
| BC-3.2.002 | S-3.1.04, W3-FIX-CODE-003, W3-FIX-CREDS-001 |
| BC-3.2.003 | S-3.2.01, S-3.2.03, S-3.2.04, S-3.2.08, S-3.6.01 |
| BC-3.2.004 | S-3.1.06, S-3.2.05, S-3.2.06, S-3.2.07, S-3.4.05 |
| BC-3.2.005 | S-3.0.02, S-3.2.05, S-3.2.06, S-3.2.07, S-3.3.06 |
| BC-3.3.001 | S-3.3.01, W3-FIX-SEC-003, W3-FIX-CODE-002 |
| BC-3.3.002 | S-3.3.01 |
| BC-3.3.003 | S-3.3.01 |
| BC-3.3.004 | S-3.3.01, S-3.3.02, W3-FIX-SEC-003, W3-FIX-CODE-002, W3-FIX-CODE-004, W3-FIX-SEC-004 |
| BC-3.4.001 | S-3.7.01, S-3.7.02, S-3.7.03, S-3.7.04, S-3.7.05 |
| BC-3.4.002 | S-3.7.00, S-3.7.01, S-3.7.02, S-3.7.03, S-3.7.04, S-3.7.05 |
| BC-3.4.003 | S-3.7.00, S-3.7.01, S-3.7.02, S-3.7.03, S-3.7.04, S-3.7.05 |
| BC-3.4.004 | S-3.7.02, S-3.7.03, S-3.7.04, S-3.7.05 |
| BC-3.5.001 | S-3.3.03, S-3.3.05, S-3.4.01, S-3.4.02, S-3.4.03, S-3.4.04, S-3.4.05, S-3.6.01, S-3.6.02, W3-FIX-SEC-001, W3-FIX-SEC-002, W3-FIX-CODE-001, W3-FIX-CODE-002, W3-FIX-CODE-004, W3-FIX-SEC-004, W3-FIX-CODE-005, W3-FIX-SEC-005, W3-FIX-CODE-006 |
| BC-3.5.002 | S-3.3.04, S-3.3.05, S-3.4.01, S-3.4.02, S-3.4.03, S-3.4.04, S-3.6.02, W3-FIX-SEC-001, W3-FIX-SEC-002, W3-FIX-CODE-001, W3-FIX-CODE-002, W3-FIX-CODE-004, W3-FIX-SEC-004, W3-FIX-CODE-005, W3-FIX-SEC-005 |
| BC-3.6.001 | S-3.3.03, S-3.3.05, S-3.4.04, S-3.6.01, S-3.6.02, W3-FIX-CODE-001, W3-FIX-CODE-004, W3-FIX-CODE-005 |
| BC-3.6.002 | S-3.3.03, S-3.6.01, S-3.6.02 |
| BC-3.7.001 | S-3.5.01 |

---

## VP Assignment Matrix

| VP | Story | Method | Property (from verification-architecture.md) |
|----|-------|--------|----------------------------------------------|
| VP-001 | S-1.01 | kani | OrgSlug rejects invalid characters |
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
| v1.42 | 2026-04-21 | Wave-0a Red Gate complete — S-0.02 spec patched v1.3→v1.4 (task 10: removed invalid Cargo workspace [features] table; documented per-crate dtu=[] feature pattern). No story count change; no BC/VP changes. |
| v1.55 | 2026-04-27 | Wave 3 Multi-Tenant story registration (pre-compact handoff). Added 16 new stories (S-3.0.01/02, S-3.1.01–07, S-3.2.01–07, S-3.3.01–05, S-3.4.01–05, S-3.5.01, S-3.6.01/02, S-3.7.00–05) all at status: draft. 21 new BCs (BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–003, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001) at v0.2 PROPOSED. 2 new CAPs (CAP-036, CAP-037). Story count 76 → 92; BC count 200 → 221. All Wave 3 stories NOT ready — pending Phase 3.A convergence + human approval (D-045). |
| v1.56 | 2026-04-27 | BLOCK-2 + BLOCK-4 + BC-3.3.001→BC-3.3.004 propagation (consistency-validator Phase 3.A pass). BLOCK-2: total_stories corrected 92→111 (35 MT stories, not 16); Full Story List +35 rows; BC Traceability Matrix +27 Wave 3 BC rows (BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001). BLOCK-4: BC-3.4.003 added to S-3.7.04 and S-3.7.05 frontmatter behavioral_contracts + anchor_bcs + body BC tables + token budget count. BC-3.3.001→BC-3.3.004 propagation (ADR-010 customer config validation contract rename): S-3.3.01 inputs/frontmatter/body/ACs updated; S-3.3.02 inputs/frontmatter/body/ACs updated; E-3.3 wave table updated; BC Traceability Matrix rows added for BC-3.3.001 (ADR-007, S-3.4.05 only) and BC-3.3.004 (ADR-010, S-3.3.01 + S-3.3.02). BC-INDEX version pin v4.14→v4.15; total_bcs_covered 221→230; unique active BCs 200→222. |
| v1.60 | 2026-04-27 | Adversary Pass 1 story-side fixes. M-004: BC-3.3.001 anchored to S-3.3.01 — E-3.3 table row BCs updated (BC-3.3.004,BC-3.3.002,BC-3.3.003 → BC-3.3.001,BC-3.3.002,BC-3.3.003,BC-3.3.004); BC Traceability Matrix BC-3.3.001 story corrected S-3.4.05→S-3.3.01 (BC-3.3.001 is the unconditional ST guard implemented by S-3.3.01 startup validator, not by S-3.4.05 test migration); S-3.3.01 story file updated (BC-3.3.001 added to behavioral_contracts, anchor_bcs, inputs, body BC table; AC-016/AC-017 already traced to BC-3.3.001-startup; token budget count 3→4 BCs). m-001: overview line 18 story count 76→113. m-002: BC-INDEX pin v4.16→v4.17 at lines 24 and 93. m-003: v0.2 PROPOSED assertion updated to "v0.2 or v0.3 PROPOSED" in line 24. m-004: frontmatter total_bcs_covered 230→total_active_bcs 222 with comment clarifying 230 total registered. Wave 3 VP citation propagation: Full Story List VP columns updated to flat form — S-3.2.08 VP-3.2.003-01→VP-084, S-3.3.06 VP-3.2.005-04→VP-094. Story files S-3.2.08 and S-3.3.06 verification_properties frontmatter updated to flat form. S-3.2.01–07 verification_properties frontmatter updated to flat VP-NNN form (VP-3.2.001-01–04→VP-077–080, VP-3.2.003-01–03→VP-084–086, VP-3.2.004-01–04→VP-087–090, VP-3.2.005-01/02/04→VP-091/092/094). S-3.7.00–05 verification_properties frontmatter updated to flat form (VP-3.4.x-letter→VP-108–121). |
| v1.61 | 2026-04-27 | Adversary Pass 2 story-side fixes. M-001: total_vps_assigned 62→136; overview VP breakdown updated to "136 (30 Kani proofs, 77 proptests, 4 unit_tests, 6 fuzz targets, 19 integration tests)" per VP-INDEX v1.13. m-004: S-3.0.02 title + crate column corrected — ADR-007 §2.3 specifies a single centralized registry in prism-core (not per-crate constants); title now "prism-core: register DTU_DEFAULT_MODE registry (10-entry DtuRegistryEntry slice) per ADR-007 §2.3"; crate column "prism-dtu-* (7 crates)"→"prism-core"; S-3.0.02 story file rewritten to reflect centralized registry scope (DtuRegistryEntry struct + 10-entry static in prism-core). |
| v1.62 | 2026-04-27 | Pass 16 story-side fixes. M-16-002: S-1.01 Full Story List title updated "TenantId" → "OrgSlug [TenantId legacy alias]" per ADR-006. M-16-003: S-3.1.01 + S-3.1.03 subsystems SS-06→SS-21 (prism-core owns OrgId/OrgRegistry per D-047/ARCH-INDEX SS-21). m-16-001: BC body table titles corrected to Title Case per BC-INDEX canonical form across all affected Wave 3 stories (S-3.1.01–07, S-3.2.01–07, S-3.3.01–02). |
| v1.63 | 2026-04-28 | S-3.0.02 v0.3 → v0.4 (M-32-001 subsystems [SS-01,SS-06]→[SS-21]) |
| v1.64 | 2026-04-28 | M-33-001 fix — VP Assignment Matrix VP-001 Property column corrected `TenantId rejects invalid characters` → `OrgSlug rejects invalid characters` per verification-architecture.md v1.21 source-of-truth. Residual M-14-002 OrgSlug rename propagation. |
| v1.65 | 2026-04-28 | M-34-001 fix — Prose changelog backfill: append missing v1.63 → v1.64 entry for M-33-001 fix that was added to tabular changelog only. Bookkeeping audit-trail completeness. |
| v1.66 | 2026-04-28 | m-38-001 fix — S-3.5.01 v1.2→v1.3: line 228 "all 6 subsystems"→"all 7 subsystems" sibling-fix gap from P27 changelog over-claim. |
| v1.67 | 2026-04-28 | m-41-001 fix — S-3.5.01 v1.3→v1.4 lines 57+228 stale paraphrase corrected to BC-3.7.001 v0.8 canonical framing. COMPREHENSIVE 6-class BC-drift sweep zero residues. |
| v1.68 | 2026-04-28 | m-42-001 fix — S-3.0.01 + S-3.0.02 frontmatter epic_id "E-Quick"→"E-3.0" matching STORY-INDEX canonical. NEW DEFECT CLASS: frontmatter-vs-index drift. |
| v1.69 | 2026-04-28 | m-43-001 fix — S-3.0.01 v0.2→v0.3 line 146 body E-Quick→E-3.0 sibling propagation. |
| v1.71 | 2026-04-28 | Phase 3.A APPROVED — ADR-006..ADR-012 ACCEPTED; 3 Wave 4+ TDs filed; impl cleared to begin. |
| v1.70 | 2026-04-28 | Pass 44 fixes — L-44-001 wave-state.yaml legacy block removed + O-44-001 STORY-INDEX changelog block reordered ascending. |
| v1.78 | 2026-05-02 | W3.3 fix wave CLOSED — W3-FIX-SEC-004 (PR #122 4e053105) + W3-FIX-CODE-005 (PR #123 e4be29ae) MERGED. E-3.5 epic header (12→14). BC Traceability Matrix: BC-3.2.001/BC-3.3.004/BC-3.5.001/BC-3.5.002/BC-3.6.001 += new stories. total_stories 125→127. Pass-51 queued. D-188. |
| v1.79 | 2026-05-02 | W3.4 fix wave story authoring — W3-FIX-SEC-005 (5-DTU admin-token, 10 sites, P1) + W3-FIX-CODE-006 (CR-023 test coverage, P3) registered. E-3.5 epic header (14→16); Wave 3.1–3.4. BC Traceability Matrix: BC-3.5.001 += SEC-005 + CODE-006; BC-3.5.002 += SEC-005. total_stories 127→129. |
| v1.81 | 2026-05-02 | W4 Phase 4.A story remediation complete — all 8 W4 stories updated; 43 drift findings + 5 spec-quality HIGH findings addressed; ADR refs added per story (ADR-013/015/016/017/018/019); library pins updated per research-findings.md; S-4.03 5→8 pts, S-4.05 1→4 pts, S-4.06 5→9 pts, S-4.08 5→9 pts (stories already at new points in index); v1.80 story versions bumped per story frontmatter; pre-flight re-run queued. |
| v1.82 | 2026-05-02 | Wave 4 Phase 4.A iter-2 fixes — S-4.04/4.05/4.06 version bumps; NEW-004 ADR-018→019 annotation correction; NEW-003 S-4.02 points reconciliation 5→3. |
| v1.83 | 2026-05-02 | Wave 4 Phase 4.A Pass 1 remediation — 8 W4 story version bumps (S-4.01..S-4.08); CF discriminator collision RESOLVED (S-4.05 rate limits moved to action_state CF); UNION merge model adopted; UDF Volatility=Stable; ADR alignments per architect v0.2. |
| v1.93 | 2026-05-04 | Wave 4 Phase 4.A Pass 12 remediation — S-4.05 v1.9→v1.10 (SS-14 body sweep confirmed clean; no residual SS-14 references; closes F-P12-L-001). BC-2.12.004 v1.5→v1.6 (fire-loop model aligned to ADR-013 §2.5/§2.6; closes F-P12-M-001). |
| v1.92 | 2026-05-04 | Wave 4 Phase 4.A Pass 11 remediation — STRUCTURAL PREVENTION: dropped vN.M version pins from story-body ADR/BC cross-references (7 pins removed). S-4.08 v1.18→v1.19 (4 pins removed; dead-letter prose extended F-P11-M-002; AC-18 re-anchored F-P11-L-002). S-4.05 v1.8→v1.9 (3 pins removed; ADR-016 v0.2 stale ref removed F-P11-M-001). |
| v1.91 | 2026-05-03 | Wave 4 Phase 4.A Pass 10 remediation — S-4.08 v1.17→v1.18 (Task 7 line 222 alignment; closes P10-M-002 sister-row sweep analog). |
| v1.90 | 2026-05-03 | Wave 4 Phase 4.A Pass 9 remediation — S-4.08 v1.16→v1.17 (retry CF key sibling sweep: dead-letter CF key unified to {org_id}:{client_id}:{action_id}; idempotency_key moved to value field; alert_id canonicalization; SMTP auth Dev Notes → Task 7a; closes P9-H-001+H-002+M-003). |
| v1.89 | 2026-05-03 | Wave 4 Phase 4.A Pass 8 remediation — S-4.08 v1.15→v1.16 (AC-6 SMTP auth order XOAUTH2→PLAIN→E-AD-018 per ADR-016 §2.3; §4 tick 1s→60s default per ADR-013 §3.2; closes P8-H-001+H-003). |
| v1.88 | 2026-05-03 | Wave 4 Phase 4.A Pass 7 remediation — S-4.08 v1.14→v1.15 (BC-2.18.004 title sync at line 88 BC table; partial-fix regression of Pass 6 BC-INDEX H1 sync; consumer-table sweep gap; closes P7-HIGH-001). |
| v1.87 | 2026-05-03 | Wave 4 Phase 4.A Pass 5 remediation — S-4.08 v1.13→v1.14 (+VP-137, VP-144 in frontmatter; closes P5-S408-A-H-003/004). |
| v1.86 | 2026-05-03 | Wave 4 Phase 4.A Pass 4 remediation — S-4.06 v1.12→v1.13 (UUID v7 + dedup_window Duration alignment per ADR-015 §5). |
| v1.85 | 2026-05-02 | Wave 4 Phase 4.A Pass 3 remediation — 4 story VP frontmatter sweeps: S-4.01 v1.9→v1.10 (+VP-137); S-4.02 v1.6→v1.7 (+VP-141, VP-142); S-4.03 v1.8→v1.9 (+VP-139, VP-140); S-4.04 v1.7→v1.8 (+VP-140). Closes P3-XSTORY-A-H-002 (5 VP frontmatter omissions). |
| v1.84 | 2026-05-02 | Wave 4 Phase 4.A Pass 2 remediation — 5 story version bumps: S-4.03 v1.7→v1.8 (duplicate YAML key, BC anchor, comment); S-4.05 v1.7→v1.8 (§5/Annex VP-028 dedup); S-4.06 v1.11→v1.12 (VP-138 frontmatter); S-4.07 v1.7→v1.8 (mttd_approx formula); S-4.08 v1.12→v1.13 (subsystem SS-12 removed, SS-20 added). S-4.01/4.02/4.04 unchanged. |
