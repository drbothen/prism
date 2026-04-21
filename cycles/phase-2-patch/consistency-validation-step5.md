---
document_type: consistency-report
level: ops
version: "1.0"
status: fail
producer: consistency-validator
timestamp: 2026-04-20T00:00:00Z
phase: 2
inputs:
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
  - ".factory/stories/STORY-INDEX.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/specs/architecture/ARCH-INDEX.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/stories/"
  - ".factory/specs/behavioral-contracts/"
  - ".factory/specs/verification-properties/"
input-hash: "8b61e8a"
traces_to: ".factory/STATE.md"
---

# Consistency Validation Report: Prism

## Report Metadata

| Field | Value |
|-------|-------|
| **Product** | Prism |
| **Generated** | 2026-04-20T00:00:00Z |
| **Generator** | consistency-validator |
| **Artifacts Scanned** | 322 (203 BCs + 75 stories + 39 VPs + 4 PRD supplements + 1 BC-INDEX + 1 STORY-INDEX + 1 VP-INDEX) |
| **Cycle** | phase-2-patch |
| **Step** | 5 (post-Wave-1-8 corpus sweep, post-Step-4 input-hash recompute) |
| **Gate Result** | FAIL — 5 blocking findings |

---

## Summary

| # | Check | Result |
|---|-------|--------|
| 1 | L2 to L3 Requirement Coverage | pass |
| 2 | L3 to L4 Verification Property Coverage | pass (with anchoring violations) |
| 3 | Dependency Acyclicity | pass (with asymmetry findings) |
| 4 | Architecture Alignment | pass |
| 5 | Acceptance Criteria Quality | not audited (out of scope for Step 5) |
| 6 | Story Sizing (all <= 13 points) | pass |
| 7 | Priority Consistency | pass |
| 8 | L1 to L2 to L3 to L4 Chain Completeness | pass |
| 9 | AC Completeness Coverage | not audited (out of scope for Step 5) |
| 10 | ASM/R Traceability | not audited (out of scope for Step 5) |

**Step 5 focus:** Cross-reference integrity (BC/VP dangling refs, level consistency, input path validity, dependency bidirectionality, anchor consistency, count reconciliation). Criteria 5, 9, and 10 require full AC/holdout scans and are deferred to the adversarial pass-59.

---

## Count Reconciliation

| Artifact | Index Declared | Filesystem Actual | Match |
|----------|---------------|-------------------|-------|
| Active BCs | 195 (BC-INDEX) | 195 (lifecycle scan) | PASS |
| Removed BCs | 6 (BC-INDEX) | 6 | PASS |
| Retired BCs | 2 (BC-INDEX) | 2 | PASS |
| Total BC files | 203 (BC-INDEX) | 203 | PASS |
| Stories | 75 (STORY-INDEX) | 75 | PASS |
| VPs | 39 (VP-INDEX) | 39 | PASS |
| Unique BCs covered by stories | 195 (STORY-INDEX) | 195 | PASS |
| Orphaned active BCs | 0 expected | 0 found | PASS |
| STATE.md `removed_bc_count` | 13 (field) | 8 (actual) | STALE — see IMP-004 |

BC-INDEX self-consistency: `195 + 6 + 2 = 203`. PASS.
VP-INDEX self-consistency: `Kani 20 + Proptest 11 + Fuzz 6 + Integration 2 = 39`. PASS.

---

## 1. L2 to L3 Requirement Coverage

### 1.1 Domain Capabilities to Behavioral Contracts

All 34 capabilities defined in `capabilities.md` (CAP-001 through CAP-034, no CAP-013) are covered by at least one active BC. The `capability:` field in BC frontmatter was scanned corpus-wide. No CAP ID appears in any BC that is not defined in capabilities.md. No capability is orphaned (every CAP has at least one non-removed BC).

| CAP range | BCs present | Gap? |
|-----------|-------------|------|
| CAP-001..CAP-012 | yes | none |
| CAP-013 | intentionally omitted from both capabilities.md and all BCs | consistent |
| CAP-014..CAP-034 | yes | none |

**Check 1 result: PASS**

**Note — YAML format defect in 4 BCs:** Four BC files use unquoted YAML array syntax for `capability:` instead of the corpus-wide string scalar convention. This does not affect coverage but is flagged as IMP-006.

---

## 2. L3 to L4 Verification Property Coverage

### 2.1 Behavioral Contracts to Verification Properties

All 39 VPs in VP-INDEX resolve to existing files on disk. All VP `source_bc` fields reference BC IDs that exist on disk and are active (not removed/retired). VP-030 uses array form `source_bc: [BC-2.12.001, BC-2.13.006]`; both BCs exist and are active.

All 39 VPs appear in both `verification-architecture.md` and `verification-coverage-matrix.md`.

VP-INDEX summary arithmetic is internally consistent: 20 Kani + 11 Proptest + 6 Fuzz + 2 Integration = 39 total, 32 P0 + 7 P1 = 39 total.

**VP anchor mismatches (3 violations — see BLK-004):**

| Story (claims VP) | VP | VP-INDEX anchor |
|-------------------|----|-----------------|
| S-4.03 (line 24) | VP-030 | S-4.01 |
| S-6.06 (line 27) | VP-033 | S-6.07 |
| S-6.06 (line 27) | VP-036 | S-6.07 |

These are blocking because VP-INDEX is the source-of-truth for anchor story (Criterion 72). The S-6.06 body correctly states both VPs execute in S-6.07, making this a frontmatter/body inconsistency as well.

**Check 2 result: PASS with blocking VP anchor violations (BLK-004)**

---

## 3. Dependency Acyclicity

### 3.1 Topological Order

The STORY-INDEX v1.28 documents a validated topological sort across 7 waves. No cycles were introduced by the Wave 1-8 sweep (the sweep added frontmatter fields and sections; it did not modify `depends_on` or `blocks` edges). The topological layers (Layer 0 through Layer 6) remain acyclic.

**Check 3 result: PASS** (no cycles detected)

### 3.2 Dependency Bidirectionality Violations

33 edges are asymmetric between `depends_on` and `blocks`. These fall into two categories:

**Category A — Core product graph (9 asymmetric edges, genuine bugs):**

| Fix required on | Field | Missing value |
|-----------------|-------|---------------|
| S-4.01 | `blocks` | add S-5.01 |
| S-2.04 | `blocks` | add S-5.10 |
| S-5.10 | `depends_on` | add S-5.09 |
| S-2.01 | `blocks` | add S-6.01 |
| S-5.01 | `blocks` | add S-6.01 |
| S-6.01 | `blocks` | add S-6.04, S-6.05 |
| S-3.02 | `blocks` | add S-3.10, S-3.11, S-3.12, S-3.13 |
| S-2.06 | `blocks` | add S-3.12 |
| S-0.01 or S-1.01 | `blocks`/`depends_on` | remove S-1.01 from S-0.01 blocks, or add S-0.01 to S-1.01 depends_on |

**Category B — DTU-related (24 edges, requires human decision):**

DTU stories S-6.07 through S-6.19 declare `blocks:` edges into the product graph (S-6.07 blocks S-3.06, S-3.07; S-6.08/09/10 block S-3.02; S-6.11/12/13 block S-4.08, S-5.06; S-6.14/15 block S-1.14, S-5.06; S-6.16/17/18/19 block S-5.09). The corresponding product stories do not list DTU stories in `depends_on`. This was acknowledged as "Option B, human-approved" per Phase-3-Patch Burst 5b but the reverse-direction was never completed. Recommended resolution: remove DTU `blocks` entries pointing into the product graph (DTU stories are test infrastructure, not schedule prerequisites).

See IMP-001 for full detail.

---

## 4. Architecture Alignment

### 4.1 Module Coverage

All 20 subsystems (SS-01 through SS-20) defined in the ARCH-INDEX Subsystem Registry are referenced by at least one story. Subsystem SS-20 (Observability / Log Forwarding) is used in stories S-5.08, S-5.09, and S-5.10 but has no BC-2.20.* series. This is a known architectural decision: S-5.09's single BC (BC-2.10.001) is in SS-10 (MCP Interface); SS-20 is implemented without a dedicated BC subsection. No orphaned subsystem IDs found.

All BC `subsystem:` values (SS-01 through SS-19) match canonical names in the ARCH-INDEX Subsystem Registry.

### 4.2 Component Consistency

Story `anchor_subsystem` and `subsystems` fields were not exhaustively audited against ARCH-INDEX for every story (out of scope for this step). The semantic anchoring integrity audit was performed in prior adversarial passes (p48-p58). No new drift introduced by the Wave 1-8 sweep.

**Check 4 result: PASS**

---

## 5. Acceptance Criteria Quality

Not audited in Step 5. This check requires full AC text analysis. Deferred to adversarial pass-59.

**Check 5 result: NOT AUDITED**

---

## 6. Story Sizing

All 75 stories have `points:` values of 13 or fewer. The corpus was swept in Waves 1-8 and no `points:` values were changed; the previous adversarial loop (passes 48-58) confirmed no over-sized stories. No stories have missing `points:` fields post-sweep.

**Check 6 result: PASS**

---

## 7. Priority Consistency

P0 stories depend only on other P0 stories. P1 stories (S-1.09, S-3.04, S-3.06, S-3.07) depend on P0 stories exclusively. No P0 story has an unresolved P1/P2 blocking dependency. This was validated during the adversarial loop and was not changed by the Wave 1-8 sweep.

**Check 7 result: PASS**

---

## 8. L1 to L2 to L3 to L4 Chain Completeness

### L1 to L2 to L3 to L4 Chain Overview

| Level | Artifact | Count | Traced Forward | Traced Backward | Coverage |
|-------|----------|-------|---------------|----------------|----------|
| L1 | Product Brief | 1 file | to L2 domain-spec | N/A | 100% |
| L2 | Domain Capabilities (CAP-NNN) | 34 (no CAP-013) | 34 to L3 BCs | 34 to L1 | 100% |
| L3 | Active BCs (BC-S.SS.NNN) | 195 | 195 to L4 stories | 195 to L2 CAP | 100% |
| L4 | Stories | 75 | N/A | all via behavioral_contracts | 100% |
| L4 | VPs (VP-NNN) | 39 | N/A | 39 via source_bc | 100% |

### Broken Chains

No broken chains detected. All 195 active BCs are covered by at least one story. All 34 capabilities have BC coverage. The 7 osquery-inspired stories (S-2.08, S-3.08 through S-3.13) have 0 BCs by design (enhancements from osquery synthesis, not traced to formal BCs) — this is documented in STORY-INDEX v1.28.

### Orphaned Artifacts

No orphaned active BCs. No VP without a `source_bc`. The 6 removed and 2 retired BC files are tombstones — they are not orphans; they have lifecycle records.

**Check 8 result: PASS**

---

## 9. AC Completeness Coverage

Not audited in Step 5. This requires per-BC clause extraction and AC cross-reference. Deferred to adversarial pass-59.

**Check 9 result: NOT AUDITED**

---

## 10. ASM/R Traceability

Not audited in Step 5. ASM/R analysis requires reading assumptions.md and risks.md and cross-referencing holdout scenarios. Deferred to adversarial pass-59.

**Check 10 result: NOT AUDITED**

---

## Cross-Reference Validation

### ID Consistency

| Check | Status | Issues |
|-------|--------|--------|
| BC IDs unique in BC-INDEX | pass | 0 duplicates |
| BC files match BC-INDEX exactly | pass | 203 IDs in index = 203 files on disk |
| VP IDs unique | pass | 0 duplicates |
| CAP IDs in BCs exist in capabilities.md | pass | CAP-001..034 (no CAP-013), all valid |
| VP source_bc → valid active BC | pass | 0 dangling, 0 pointing to removed/retired |
| Story behavioral_contracts → existing BC | pass | 0 dangling |
| Story behavioral_contracts → not removed/retired BC | pass | 0 violations |
| Story anchor_bcs → existing BC | pass | 0 dangling |
| Story verification_properties → existing VP | pass | 0 dangling |
| Story verification_properties anchor match (VP-INDEX SoT) | FAIL | 3 mismatches (BLK-004) |

### Naming Convention Compliance

| Convention | Expected Pattern | Violations |
|-----------|-----------------|------------|
| BC naming | BC-S.SS.NNN | 0 |
| VP naming | VP-NNN | 0 |
| CAP naming | CAP-NNN | 0 |
| Story naming | S-N.NN | 0 |
| VP filenames | `vp-NNN-slug.md` | 0 (all 39 match) |

### Canonical Frontmatter Validation

All 203 BC files, 75 story files, and 39 VP files have all 6 required fields present: `document_type`, `level`, `version`, `producer`, `traces_to`, `timestamp`. Zero missing field violations.

BC levels: all 203 files have `level: L3`. PASS.
Story levels: 69 of 75 have `level: "L4"`. 6 files (S-6.14 through S-6.19) have `level: "L2"` — blocking violation BLK-001.
VP levels: all 39 have `level: L4`. PASS.

---

## Spec vs Implementation Drift

| Artifact | Spec Version | Drift Detected | Notes |
|----------|-------------|----------------|-------|
| BC-INDEX | v4.10 | no | 203 files, counts consistent |
| STORY-INDEX | v1.28 | no | 75 stories, counts consistent |
| VP-INDEX | v1.5 | no | 39 VPs, counts consistent |
| verification-architecture.md | — | no | all 39 VPs present |
| verification-coverage-matrix.md | — | no | all 39 VPs present |
| STATE.md `removed_bc_count` | 13 | YES | stale — actual is 8 (IMP-004) |
| Story `inputs:` paths (12 stories) | — | YES | legacy `architecture/` prefix + wrong VP slugs (BLK-002, BLK-003, BLK-005) |
| Story `level:` field (6 stories) | — | YES | `"L2"` should be `"L4"` (BLK-001) |
| BC-2.10.008 changelog ordering | — | YES | row 1.3 inserted out of sequence (IMP-002) |
| BC `capability:` YAML format (4 BCs) | — | YES | unquoted array instead of string scalar (IMP-006) |

---

## Findings

### Blocking

All blocking findings must be resolved before Phase 3 dispatch.

---

**BLK-001: Wave 8 DTU Stories Have Wrong VSDD Level (`"L2"` instead of `"L4"`)**

Files: `S-6.14-dtu-threatintel.md`, `S-6.15-dtu-nvd.md`, `S-6.16-dtu-datadog.md`, `S-6.17-dtu-splunk-hec.md`, `S-6.18-dtu-elasticsearch.md`, `S-6.19-dtu-otlp.md` (all line 5)

Wave 8 story-writer classified DTU clones as `level: "L2"` conflating the DTU fidelity tier label ("L2 stateful") with the VSDD hierarchy level. Per the VSDD hierarchy, all stories are L4 by definition. Wave 7 DTU stories (same category) correctly use `level: "L4"`.

Remediation: Change `level: "L2"` → `level: "L4"` in all 6 files. Bump `version` and add changelog row.
Owner: story-writer

---

**BLK-002: 6 Stories Reference Legacy `architecture/behavioral-contracts/` Input Paths (Do Not Exist)**

Files: `S-3.03`, `S-3.04`, `S-3.05`, `S-3.06`, `S-3.07`, `S-4.01`

29 `path:` entries reference `architecture/behavioral-contracts/BC-*.md` and `architecture/verification-properties/VP-*.md`. This path prefix does not exist. The correct prefix is `.factory/specs/behavioral-contracts/` and `.factory/specs/verification-properties/`. All referenced BC and VP IDs exist at the correct location. The VP entries also use wrong filenames (e.g., `VP-012.md` instead of `vp-012-alias-depth-limit.md`).

Complete bad-path inventory:

| Story | Bad path |
|-------|----------|
| S-3.03 | `architecture/behavioral-contracts/BC-2.11.010.md` |
| S-3.04 | `architecture/behavioral-contracts/BC-2.11.008.md` |
| S-3.04 | `architecture/behavioral-contracts/BC-2.11.009.md` |
| S-3.04 | `architecture/behavioral-contracts/BC-2.11.013.md` |
| S-3.04 | `architecture/behavioral-contracts/BC-2.11.014.md` |
| S-3.04 | `architecture/behavioral-contracts/BC-2.11.015.md` |
| S-3.04 | `architecture/verification-properties/VP-012.md` → actual: `vp-012-alias-depth-limit.md` |
| S-3.04 | `architecture/verification-properties/VP-013.md` → actual: `vp-013-alias-cycle-detection.md` |
| S-3.04 | `architecture/verification-properties/VP-025.md` → actual: `vp-025-cache-key-deterministic.md` |
| S-3.04 | `architecture/verification-properties/VP-037.md` → actual: `vp-037-alias-expansion-no-panic.md` |
| S-3.05 | `architecture/behavioral-contracts/BC-2.07.001.md` |
| S-3.05 | `architecture/behavioral-contracts/BC-2.07.002.md` |
| S-3.05 | `architecture/behavioral-contracts/BC-2.07.003.md` |
| S-3.05 | `architecture/behavioral-contracts/BC-2.07.004.md` |
| S-3.05 | `architecture/behavioral-contracts/BC-2.07.005.md` |
| S-3.05 | `architecture/behavioral-contracts/BC-2.07.006.md` |
| S-3.06 | `architecture/behavioral-contracts/BC-2.11.004.md` |
| S-3.07 | `architecture/behavioral-contracts/BC-2.04.001.md` |
| S-3.07 | `architecture/behavioral-contracts/BC-2.04.005.md` |
| S-3.07 | `architecture/behavioral-contracts/BC-2.04.007.md` |
| S-3.07 | `architecture/behavioral-contracts/BC-2.04.008.md` |
| S-3.07 | `architecture/behavioral-contracts/BC-2.05.009.md` |
| S-4.01 | `architecture/behavioral-contracts/BC-2.12.001.md` |
| S-4.01 | `architecture/behavioral-contracts/BC-2.12.002.md` |
| S-4.01 | `architecture/behavioral-contracts/BC-2.12.003.md` |
| S-4.01 | `architecture/behavioral-contracts/BC-2.12.004.md` |
| S-4.01 | `architecture/behavioral-contracts/BC-2.12.010.md` |
| S-4.01 | `architecture/verification-properties/VP-026.md` → actual: `vp-026-splay-deterministic.md` |
| S-4.01 | `architecture/verification-properties/VP-030.md` → actual: `vp-030-schedule-rule-caps.md` |

Remediation: Fix as part of BLK-005 (combined inputs format conversion). Recompute `input-hash`.
Owner: story-writer

---

**BLK-003: 4 Stories Reference VP Input Files With Wrong Filename Slugs**

Files: `S-1.02`, `S-1.03`, `S-1.04`, `S-1.05`

These stories use the correct `.factory/specs/verification-properties/` prefix but the filename slugs do not match any file on disk:

| Story | Bad filename | Actual filename |
|-------|-------------|-----------------|
| S-1.02 | `vp-005-case-status-12-transitions.md` | `vp-005-case-state-machine.md` |
| S-1.02 | `vp-006-case-status-no-self-transitions.md` | `vp-006-case-state-no-self-transitions.md` |
| S-1.03 | `vp-002-deny-by-default.md` | `vp-002-capability-deny-by-default.md` |
| S-1.03 | `vp-003-most-specific-wins.md` | `vp-003-capability-most-specific-wins.md` |
| S-1.03 | `vp-004-exact-match-explanation.md` | `vp-004-capability-deny-overrides-allow.md` |
| S-1.04 | `vp-016-ocsf-normalize-valid-protobuf.md` | `vp-016-ocsf-output-valid-protobuf.md` |
| S-1.04 | `vp-022-normalize-no-panic.md` | `vp-022-ocsf-normalizer-no-panic.md` |
| S-1.05 | `vp-017-no-fields-silently-dropped.md` | `vp-017-ocsf-unmapped-fields-preserved.md` |

Remediation: Correct each slug to the actual canonical filename. Recompute `input-hash`. Bump version and add changelog row.
Owner: story-writer

---

**BLK-004: 3 VP Anchor Story Mismatches Between Story Frontmatter and VP-INDEX**

Files: `S-4.03-detection-rules.md` (line 24), `S-6.06-dtu-common.md` (line 27)

VP-INDEX is the source-of-truth for anchor story assignment (Criterion 72). Three story frontmatter entries contradict VP-INDEX:

| Story | VP claimed | VP-INDEX anchor | Body consistent? |
|-------|-----------|-----------------|-----------------|
| S-4.03 | VP-030 | S-4.01 | No — body has a VP-030 proof task |
| S-6.06 | VP-033 | S-6.07 | No — body line 255 says "VP-033 executes in S-6.07" |
| S-6.06 | VP-036 | S-6.07 | No — body says same |

For S-4.03/VP-030: VP-030 covers schedule AND rule count caps; S-4.03 has a natural claim on rule caps. Resolution: remove VP-030 from S-4.03 frontmatter and update the body task to reference VP-030 as a cross-story dependency proven in S-4.01.

For S-6.06/VP-033/036: the body is already correct; only the frontmatter needs correction.

Remediation: (1) S-4.03: remove VP-030 from `verification_properties`. (2) S-6.06: remove VP-033 and VP-036 from `verification_properties`. Bump version on each file.
Owner: story-writer

---

**BLK-005: 12 Stories Use Non-Canonical Variant-B Dict-Format `inputs:` Block**

Files: `S-3.03`, `S-3.04`, `S-3.05`, `S-3.06`, `S-3.07`, `S-3.08`, `S-3.09`, `S-3.10`, `S-3.11`, `S-3.12`, `S-3.13`, `S-4.01`

These 12 stories (S-3.03 through S-3.13 plus S-4.01) use a per-entry `path: / input-hash:` dict structure instead of the canonical YAML string-list format. Example of non-canonical form:
```yaml
inputs:
  - path: prd.md
    input-hash: null
  - path: architecture/behavioral-contracts/BC-2.11.010.md
    input-hash: null
```
Expected canonical form:
```yaml
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.010-explain-query-tool.md"
```

Six of the 12 also have legacy BC/VP path prefixes (BLK-002). The remaining six (S-3.08 through S-3.13) have only `prd.md` in the dict.

Remediation: Convert all 12 to YAML string-list format, simultaneously fixing legacy paths (BLK-002) and wrong VP slugs (BLK-003) where applicable. Recompute `input-hash` for each file.
Owner: story-writer

Note: BLK-002, BLK-003, and BLK-005 should be fixed in a single combined burst to avoid redundant file edits. Total files requiring inputs remediation: 16 stories (12 dict-format + 4 wrong-slug).

---

### Major

**IMP-001: 33 Dependency Graph Edges Are Asymmetric**

Category A (9 genuine asymmetries in the core product graph):

| Fix on | Missing in | Value to add |
|--------|-----------|--------------|
| S-4.01 `blocks` | — | S-5.01 |
| S-2.04 `blocks` | — | S-5.10 |
| S-5.10 `depends_on` | — | S-5.09 |
| S-2.01 `blocks` | — | S-6.01 |
| S-5.01 `blocks` | — | S-6.01 |
| S-6.01 `blocks` | — | S-6.04, S-6.05 |
| S-3.02 `blocks` | — | S-3.10, S-3.11, S-3.12, S-3.13 |
| S-2.06 `blocks` | — | S-3.12 |
| S-0.01 `blocks` has S-1.01 but S-1.01 `depends_on` does not have S-0.01 | — | either remove or add reverse |

Category B (24 DTU-related edges): DTU stories S-6.07 through S-6.19 declare `blocks:` pointing into the product graph; the product stories do not declare reciprocal `depends_on`. This was approved as "Option B" per Burst 5b but the forward direction was never reconciled. Requires human decision: remove DTU `blocks` edges (recommended — DTU is test infrastructure) or add reciprocal `depends_on` in product stories.

Remediation: story-writer fixes Category A immediately. Human decision needed on Category B before story-writer handles the DTU edges.
Owner: story-writer (Category A); human + story-writer (Category B)

---

**IMP-002: BC-2.10.008 Changelog Row 1.3 Inserted Out of Order**

File: `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.10.008-mcp-resources.md`

Changelog table rows: `1.2, 1.1, 1.3, 1.0`. The pre-build sweep row (`1.3`) was appended after `1.2` and `1.1` but before `1.0`. Correct descending order: `1.3, 1.2, 1.1, 1.0`. Frontmatter `version: "1.3"` is correct.

Remediation: Reorder the four changelog rows to descending version order. No version bump required.
Owner: product-owner

---

**IMP-003: S-6.06 `verification_properties` Frontmatter Contradicts Story Body**

File: `/Users/jmagady/Dev/prism/.factory/stories/S-6.06-dtu-common.md` (line 27)

Frontmatter has `verification_properties: [VP-033, VP-036]`. Body line 255 states: "No VPs are directly owned by this story. VP-033 and VP-036 execute in S-6.07." VP-INDEX confirms S-6.07 is the anchor. This is the frontmatter-body inconsistency underlying BLK-004 and will be resolved as part of that fix.
Owner: story-writer (covered by BLK-004)

---

**IMP-004: STATE.md `removed_bc_count` Field Is Stale**

File: `/Users/jmagady/Dev/prism/.factory/STATE.md`

`removed_bc_count: 13` has not been updated since earlier project states. BC-INDEX and filesystem both show 8 non-active BCs (6 removed + 2 retired). The historical count of 13 included BCs that were later un-retired or were index-only entries never backed by files.

Remediation: state-manager updates `removed_bc_count: 13` → `removed_bc_count: 6` and adds `retired_bc_count: 2` alongside it.
Owner: state-manager

---

**IMP-005: No `epics.md` File Exists; `epic_id` References Are Unverifiable**

All 75 stories carry `epic_id: "E-N"` (E-0 through E-6). Criterion 53 requires `epic_id` to reference a valid epic in `epics.md`. No such file exists anywhere under `.factory/`. The IDs appear to map to implementation waves but this is undocumented.

Remediation: Create `.factory/specs/epics.md` with a minimal table defining E-0 through E-6 with wave mappings and descriptions. Alternatively, document explicitly in STATE.md that `epic_id` is a wave shorthand not validated against a file.
Owner: product-owner or state-manager

---

**IMP-006: 4 BC Files Have Non-Canonical YAML Array `capability:` Field**

Files and current values:

| File | Current `capability:` value |
|------|-----------------------------|
| `BC-2.01.010-partial-failure-handling.md` | `[CAP-001, CAP-002]` (unquoted) |
| `BC-2.10.002-tool-registration-via-tool-router.md` | `[CAP-005, CAP-015]` (unquoted) |
| `BC-2.10.005-notifications-tools-list-changed.md` | `[CAP-005, CAP-009]` (unquoted) |
| `BC-2.10.008-mcp-resources.md` | `["CAP-008", "CAP-009"]` (quoted but still array) |

The corpus convention is `capability: "CAP-NNN"` (string scalar). Multi-capability BCs should use `capability: "CAP-NNN, CAP-MMM"` to match the BC-INDEX table format. Wave 5 fixed BC-2.16.008 and Wave 6 fixed BC-2.19.004 for the same issue; these four were missed.

Remediation: Normalize to `capability: "CAP-NNN, CAP-MMM"` string format. Bump version on each file.
Owner: product-owner

---

### Minor

**MIN-001: Story `cycle:` Field Is `v1.0.0-greenfield`; STATE.md `current_cycle` Is `phase-2-patch`**

All 75 stories have `cycle: "v1.0.0-greenfield"`. This is intentional — stories use the product implementation cycle (v1.0.0 greenfield), not the pipeline patch cycle name. No action required unless policy changes. Recommend documenting the distinction in STATE.md.

---

**MIN-002: BC-2.10.008 Changelog Row 1.3 Out of Sequence**

Same file as IMP-002. The row ordering within the changelog table is malformed regardless of whether ascending or descending convention is used. Resolved with IMP-002.

---

## Validation Gate Result

**FAIL** — 5 blocking findings:

1. **BLK-001**: 6 stories with `level: "L2"` (must be `"L4"`)
2. **BLK-002**: 29 non-existent legacy `architecture/` input paths across 6 stories
3. **BLK-003**: 8 wrong VP filename slugs across 4 stories
4. **BLK-004**: 3 VP anchor story mismatches (story frontmatter contradicts VP-INDEX)
5. **BLK-005**: 12 stories with non-canonical dict-format `inputs:` block

**Remediation dispatch:** BLK-001, BLK-002, BLK-003, BLK-004, BLK-005 all owned by story-writer and can be dispatched as two parallel sub-tracks (A1: level + VP anchor fixes on 8 files; A2: inputs format conversion on 16 files). IMP-002 and IMP-006 can run in parallel as a product-owner track. IMP-004 is a state-manager one-liner. IMP-001 Category A can also run in the story-writer burst; Category B awaits human decision.

---

## Overall Metrics

| Metric | Value |
|--------|-------|
| **Total Checks** | 10 (criteria 1-10) |
| **Fully Passed** | 5 (checks 1, 3, 6, 7, 8) |
| **Pass with violations** | 2 (checks 2, 4) |
| **Not audited** | 3 (checks 5, 9, 10 — deferred to adversarial pass-59) |
| **Blocking findings** | 5 |
| **Important findings** | 6 |
| **Minor findings** | 2 |
| **Dangling BC refs in stories** | 0 |
| **Dangling VP refs in stories** | 0 |
| **Orphaned active BCs** | 0 |
| **Count mismatches (index vs filesystem)** | 0 |
| **Required frontmatter fields missing** | 0 |
| **Level field violations** | 6 stories (BLK-001) |
| **Input path violations** | 37 paths across 16 stories (BLK-002, BLK-003, BLK-005) |
| **Overall Status** | inconsistencies-found — blocking |

All 195 active BCs are covered by stories. All 39 VPs resolve to existing files with valid source_bc references. BC-INDEX, STORY-INDEX, and VP-INDEX are internally consistent and match their respective filesystems. The blocking findings are all localized to story frontmatter and inputs formatting — they do not represent semantic traceability failures but must be resolved before the input-hash values can be correct and before the corpus is fully self-consistent.

---

## Remediation Dispatch Plan

### Track A: story-writer (BLK-001, BLK-002, BLK-003, BLK-004, BLK-005, IMP-001-A, IMP-003)

**Sub-track A1 — Level and VP anchor fixes (8 files, small changes):**
- S-6.14, S-6.15, S-6.16, S-6.17, S-6.18, S-6.19: `level: "L2"` → `level: "L4"`, bump version
- S-4.03: remove VP-030 from `verification_properties`, update body task, bump version
- S-6.06: remove VP-033 and VP-036 from `verification_properties`, bump version

**Sub-track A2 — Inputs format conversion (16 files):**
- S-3.03 through S-3.13 and S-4.01: convert dict-format `inputs:` to YAML string-list; fix legacy BC/VP paths; use canonical VP filenames (see BLK-002 table)
- S-1.02, S-1.03, S-1.04, S-1.05: fix VP filename slugs in existing string-list inputs
- After all path fixes: recompute `input-hash` for each file, bump version, add changelog row

**Sub-track A3 — Dependency graph Category A (9 edge fixes):**
- See IMP-001 Category A table above. Add missing `blocks` or `depends_on` values. Bump version on each modified story.
- Category B awaits human decision.

### Track B: product-owner (IMP-002, IMP-005, IMP-006)
- BC-2.10.008: reorder changelog rows to descending order (`1.3, 1.2, 1.1, 1.0`)
- Create `.factory/specs/epics.md` defining E-0 through E-6
- BC-2.01.010, BC-2.10.002, BC-2.10.005, BC-2.10.008: normalize `capability:` to string scalar format, bump version

### Track C: state-manager (IMP-004)
- Update STATE.md: `removed_bc_count: 13` → `removed_bc_count: 6`, add `retired_bc_count: 2`
- After Tracks A and B complete: atomic commit of all changes, recompute input-hashes, update STATE.md step status

**Sequencing:** Tracks A1, A2, B, C can run in parallel. Track A3 Category B awaits human decision. state-manager commit runs last.

---

## Appendix: Validation Methodology

Step 5 performed corpus-wide cross-reference validation via direct file inspection using filesystem scans, Python-based cross-reference scripts, and YAML field extraction. All checks were performed read-only against the live `.factory/` directory.

**Tools used:**
- Python 3 scripts for bidirectionality analysis, BC/VP cross-reference, path existence, level field extraction
- `grep`/`ls` for file counts, subsystem values, capability values, and section structure
- Direct file reads for changelog ordering, VP anchor verification, and YAML format inspection

**Key reference files consulted:**
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-INDEX.md` (v4.10)
- `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` (v1.28)
- `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/VP-INDEX.md` (v1.5)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/ARCH-INDEX.md`
- `/Users/jmagady/Dev/prism/.factory/STATE.md`
- `/Users/jmagady/Dev/prism/.factory/cycles/phase-2-patch/input-hash-recompute-report.md` (Step 4)

**Scope limitations:** AC completeness (Check 5, 9), ASM/R traceability (Check 10), and semantic anchor audits were deferred to adversarial pass-59. These were covered thoroughly in adversarial passes 48-58 and were not modified by the Wave 1-8 template-compliance sweep.

**Validation criteria applied:** Criteria 1-23, 29-31, 51-53, 64, 66, 67-69, 71-73, 75-80 from the consistency-validator agent's 80-criterion validation list. Criteria 24-28, 32-50, 54-63, 65, 70, 74 were spot-checked; no violations found in the subset examined.
