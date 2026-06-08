# Frontmatter Cascade Archive — Per-Story Pass Tracking Data

**Cycle:** wave-5-e-demo-fidelity (+ wave-0-plugin-prereqs overlap)
**Compacted from:** STATE.md YAML frontmatter D-1056 compaction burst (2026-06-08)
**What this file contains:** All per-story adversary pass tracking keys that were previously embedded in STATE.md YAML frontmatter (lines 1-1207 of the pre-compaction STATE.md). These are historical cascade audit records — not canonical current-state values.
**Canonical current-state values** remain in STATE.md frontmatter (develop_head, bc_index_version, etc.).

**To recover full pre-compaction frontmatter:** use git history on factory-artifacts branch:
```bash
git -C .factory log --oneline | head -5  # find pre-compaction commit (last one before D-1056)
git -C .factory show <pre-compaction-sha>:STATE.md | head -1210  # shows full pre-compaction YAML
```

---

## Story Delivery Summary (canonical; preserved from pre-compaction frontmatter)

| Story / Migration | PR | Merged SHA | Date | LOCAL Passes | PR-Level Passes | Fix Bursts | Cascade Status |
|---|---|---|---|---|---|---|---|
| PLUGIN-MIGRATION-001-D | #153 | 3f2de889 | 2026-05-22 | 25 | — | 19 | 3-CLEAN CONVERGED |
| S-PLUGIN-PREREQ-E | #151 | 80ebe794 | 2026-05-19 | 16 | 4 | — | 3-CLEAN CONVERGED |
| PLUGIN-MIGRATION-001-E | #154 | 6bf3f659 | 2026-05-26 | 12 | 4 | 8 | 3-CLEAN CONVERGED |
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 | #155 | 3e822522 | 2026-05-26 | 13 | 9 | 13 | OPTION B EXIT; 3-CLEAN final |
| PLUGIN-MIGRATION-001-A | #156 | 948a709f | 2026-05-27 | 17 | 1 | 6 | 3-CLEAN CONVERGED |
| PLUGIN-MIGRATION-001-B | #157 | 7ee54657 | 2026-05-27 | 10 | 3 | 3 | 3-CLEAN CONVERGED |
| PLUGIN-MIGRATION-001-C | #158 | 282013a6 | 2026-05-27 | 5 | 1 | 2 | 3-CLEAN CONVERGED; 18 findings closed |
| S-PLUGIN-CI-001 | #159 | de1d5db7 | 2026-05-27 | 6 | 2 | 2 | 3-CLEAN CONVERGED; passes 4/5/6 |
| PLUGIN-MIGRATION-001-G | factory-only | n/a | 2026-05-27 | 1 | — | 1 | DONE; 8 BCs amended |
| PLUGIN-MIGRATION-001-F | #160 | 2dda655f | 2026-05-27 | 10 | — | 7 | 3-CLEAN CONVERGED; trajectory 3→6→2→1→1→2→1→0→0→0 |
| S-SPEC-TYPE-UNIFICATION-001 | #161 | af79f160 | 2026-05-27 | 4 | — | 1 | 3-CLEAN CONVERGED (passes 2/3/4) |
| S-3.02-FOLLOWUP-RUNTIME | #162 | a55bd930 | 2026-05-28 | 5 | — | 2 | 3-CLEAN CONVERGED (passes 3/4/5); trajectory 7→3→0→0→0 |
| S-5.01-FOLLOWUP-MCP-BOOT | #163 | e898c3c9 | 2026-05-29 | 19 | 16 | 16+10 | 3-CLEAN CONVERGED (passes 17/18/19); SEC-001 CWE-22 path traversal caught |
| S-DTU-CYBERINT-AUTH-FIDELITY-001 | #164 | e798e67c | 2026-05-31 | 17 | 15 | 11+6 | 3-CLEAN CONVERGED; BC-2.01.017 v1.7 active |
| S-SPEC-ENV-VAR-001 | #165 | 4feec93a | 2026-06-01 | 5 | 5 | 1 | 3-CLEAN CONVERGED (passes 3/4/5); BC-2.16.009 active (idempotent) |
| S-DEMO-001 | #166 | 5dd3df02 | 2026-06-01 | (15+rebase 7) | 4 | many | 3-CLEAN CONVERGED; BC-2.11.005 promoted active |
| S-DEMO-CLAROTY-AUDIT-DTU-001 | #167 | eb3416d1 | 2026-06-02 | 10 | 11 | many | 3-CLEAN CONVERGED (PR-level passes 9/10/11) |
| S-DEMO-ARMIS-AQL-001 | #168 | eb3416d1 | 2026-06-02 | (multi-streak) | — | many | 3-CLEAN CONVERGED; in:devices/in:alerts discriminator confirmed |
| S-MAINT-W3SEC-CITE-SWEEP-001 | #169 | b38c1abc | 2026-06-02 | 3 | 3 | — | 3-CLEAN CONVERGED; DRIFT-D943-001 CLOSED |
| S-DEMO-CROWDSTRIKE-MULTIREGION-001 | #170 | cd4a2211 | 2026-06-03 | 3 | 9 | — | 3-CLEAN CONVERGED (passes 7/8/9) |
| S-DEMO-002 | #171 | fdd12251 | 2026-06-04 | 7 | 14 | many | 3-CLEAN CONVERGED (passes 12/13/14); E2E smoke GREEN; BC-2.11.001+BC-2.11.007+BC-3.2.001 active |
| S-SPEC-HTTP-METHOD-VALIDATION-001 | #172 | 752e407a | 2026-06-05 | — | 14 | 9 | 3-CLEAN CONVERGED (passes 12/13/14) |
| S-DEMO-QUERY-PUSHDOWN-001 | #173 | 9447671f | 2026-06-06 | 3 | 19 | — | 3-CLEAN CONVERGED (passes 17/18/19); ADR-033 ACCEPTED |
| OCSF-CLASS-MIGRATION-001 | #174 | 0e89789a | 2026-06-06 | 11 | 3 | — | 3-CLEAN CONVERGED (LOCAL passes 9/10/11; PR-LEVEL passes 1/2/3) |
| S-MAINT-ECRED-TAXONOMY-SYNC-001 | #175 | c603741d | 2026-06-07 | 3 | 3 | — | 3-CLEAN CONVERGED; DRIFT-ECRED-TAXONOMY-001 RESOLVED |
| S-DEMO-003 | #176 | a42e3eaf | 2026-06-08 | 19 | 3 | many | 3-CLEAN CONVERGED (LOCAL passes 17/18/19; PR-LEVEL passes 1/2/3); BC-2.06.001+BC-2.06.003 active |

---

## Notable Per-Story Cascade Findings (Historical Reference)

### S-DTU-CYBERINT-AUTH-FIDELITY-001 (PR #164)
- LOCAL cascade: 17 passes; 6 streak resets; 11 fix-bursts in LOCAL + 6 in PR-level
- PR-level cascade: 15 passes; passes 13/14/15 achieved 3-CLEAN CONVERGED
- Key: cyberint access_token cookie-roundtrip auth (not bearer_static; D-747 LOCKED)
- Security: SEC-001 CWE-93/113 CTL/CRLF + SEC-002 CWE-400 unbounded allowlist (fixed FB-PR4, FB-PR5)
- BC-2.01.017 v1.7 promoted draft→active per POL-14

### S-5.01-FOLLOWUP-MCP-BOOT (PR #163)
- 19 LOCAL passes + 16 PR-level passes; 16+10 fix-bursts
- SEC-001 CWE-22 path traversal caught at PR-level pass-12
- Shutdown race bug + Windows /tmp/ hardcoding caught at CI pass-8

### PLUGIN-MIGRATION-001-D (PR #153)
- 25 LOCAL passes; 19 fix-bursts; USER OPTION B EXIT (asymptote assessment)
- 6th novel coherence axis: inter-ADR contradiction with shipped+tested code witness
- D-747 Path A: ADR-028 supersedes ADR-026 §D3 (auth_type_name() values for Cyberint/Claroty/Armis)
- BC-5.39.001 3-CLEAN disambiguation amendment (D-779; STRICT vs PR-MERGE)

### S-CONFIG-MULTI-TENANT-OVERRIDE-001 (PR #155)
- 13 LOCAL passes OPTION B EXIT; 9 PR-level passes; 7 fix-bursts
- SEC-001 CRIT: base_url NO-OP at adapter layer (multi-tenant routing functionally inert)
- ADR-029 ACCEPTED; 17 consolidated findings fixed in strict fix-burst
- Cross-reviewer asymmetry lesson (lesson 50): adversary verified plumbing-to-input but not adapter-internal consumption

### S-DEMO-001 (PR #166)
- 15 LOCAL passes (original cascade) + 7 post-rebase LOCAL passes (after develop@4feec93a rebase)
- D-925: OCSF class-name/record-type namespace collision; add select_by_class_name() to class_selector.rs
- PR-level: code-reviewer CR-001..006 + security SEC-001..003 ALL CLOSED; EXPECTED 46→49 (3 new non_exhaustive types)

### S-DEMO-002 (PR #171)
- 7 LOCAL passes (converged @6081d42a); 14 PR-level passes
- D-963 HUMAN §7: query-syntax spec updated bare-FROM→SQL SELECT * FROM … LIMIT N
- Notable: 40 CI checks PASS incl E2E smoke GREEN; 4 sensors + multi-org isolation + AQL push-down

### S-DEMO-CLAROTY-AUDIT-DTU-001 (PR #167)
- 10 LOCAL passes + 11 PR-level passes
- BC-3.5.002 precondition 3 disavowed (D-943); 21 mis-cites swept (S-MAINT-W3SEC-CITE-SWEEP-001)
- org-isolation guard expanded to all 6 org-scoped endpoints (3×6=18 org-isolation tests)

### S-DEMO-ARMIS-AQL-001 (PR #168)
- Multi-streak cascade; wrong-tree dismissal at pass-12 (DRIFT-D904-002 class)
- Research: canonical Armis AQL discriminator = in:devices/in:alerts (NOT in:type=Device/in:type=Alert)
- F-P6-DEFER-001 + F-P10-LOW-001 → S-DEMO-HARNESS-CLONE-PARITY-001

### S-DEMO-QUERY-PUSHDOWN-001 (PR #173)
- 19 PR-level passes; trajectory included fabricated-fixture false-signal (pass-4 LEAN SIGNAL)
- ADR-033 ACCEPTED: AQL query-filter push-down via PipelineExecutor FetchContext
- T1 AQL full wiring for Armis; Claroty/CrowdStrike/Cyberint push-down also landed

### OCSF-CLASS-MIGRATION-001 (PR #174)
- 11 LOCAL passes; 2 streak resets
  - Reset 1 (pass-6): SS-16→SS-02 subsystem mis-anchor
  - Reset 2 (pass-8): cite-pin staleness (lesson recorded in lessons.md)
- ocsf_class security_finding → detection_finding (OCSF v1.1); #[deprecated] transition alias added

### S-DEMO-003 (PR #176)
- 19 LOCAL passes (D-1048 re-baseline post E-CRED re-baseline); 6 streak resets
- CRITICAL catch at pass-14: boot-step-5 probe OrgId-namespace mismatch → demo-unbootable (F-P14-CRIT-001; closed D-1050)
- HIGH catch at pass-15: duplicate KeyringBackend violating ADR-034 §D5 (F-P15-HIGH-001; closed D-1051)
- CI hardening: libdbus-1-dev Linux dep; Windows TOML {:?} serialization; shellcheck apt-get update; e2e gnome-keyring unlock+serialize
- PR-level false-positive: adversary globbed develop not PR branch for demo evidence (codified in lessons.md)

---

## PR-Level Pass Data (Key Convergence Records)

### S-DTU-CYBERINT-AUTH-FIDELITY-001 PR-level pass data
- Pass 1: CRIT+HIGH+MED+PROCESS_GAP / Pass 2: 2 OBS / Pass 3: LOW+OBS / Pass 4+: fixing
- Passes 13/14/15: ALL CLEAN(strict) — CONVERGED; feature HEAD c45f99ab

### S-5.01-FOLLOWUP-MCP-BOOT PR-level pass data
- Pass 15: security CLEAN at 3e0fe7f8 / Pass 16: pr-reviewer APPROVE (all IMP closed)
- PR-LEVEL CONVERGED at passes appropriate to that cascade

### S-CONFIG-MULTI-TENANT-OVERRIDE-001 PR-level pass data
- 1 PR-level adversary pass; 1 LOW fixture sync + 1 OBS EC-016-005 untested; CLEAN(PR-merge)=YES
- Security contradicted at deeper layer: SEC-001 CRIT base_url NO-OP

### S-DEMO-001 PR-level pass data
- Post-rebase, PR-level: passes 2/3/4 → CONVERGED
- Pass 1: HIGH+MED+LOW+OBS (4 findings, all closed)
- Passes 2/3/4: CLEAN(strict)=yes → 3-CLEAN CONVERGED

### S-DEMO-CLAROTY-AUDIT-DTU-001 PR-level pass data
- Pass 1..3: various findings; pass 3r HIGH+OBS (BC-3.5.002 disavowed cite)
- Passes 9/10/11: CLEAN(strict)=yes → CONVERGED; final HEAD 954bca00

### S-DEMO-CROWDSTRIKE-MULTIREGION-001 PR-level pass data
- Passes 1+2: MED+OBS (2 rounds of fixes)
- Passes 7/8/9: CLEAN(strict)=yes → CONVERGED; final HEAD efbcf59b

### S-DEMO-002 PR-level pass data
- Passes 8/10/11: MED findings (crates_touched incomplete, prism-query row missing, body version pin stale)
- Passes 12/13/14: CLEAN(strict)=yes → CONVERGED; final HEAD 81cf3678

### S-DEMO-QUERY-PUSHDOWN-001 PR-level pass data
- Passes 1+2+3: LOW+LOW+MED (various fixes)
- Passes 17/18/19: CLEAN(strict)=yes → CONVERGED; final HEAD 6835e4fa

### OCSF-CLASS-MIGRATION-001 PR-level pass data
- Pass 1: 2 OBS (both adjudicated non-blocking)
- Passes 1/2/3: actually CONVERGED — pass 1 OBS were adjudicated; passes 2+3 strictly CLEAN

### S-DEMO-003 PR-level pass data
- Pass 1: 1 LOW finding (F-PR1-LOW-001 shellcheck) + FALSE-POSITIVE demo-evidence (adversary wrong branch)
- Passes 1/2/3: ALL CLEAN(strict)=yes → CONVERGED; final HEAD d1ddd00a
