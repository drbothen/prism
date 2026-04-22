---
document_type: remediation-manifest
title: "Step 5 Remediation — Track A (Story-Writer Owned)"
cycle: "v1.0.0-greenfield"
phase: "pre-build-sweep"
burst: "step5-remediation"
date: "2026-04-20"
producer: story-writer
source_report: ".factory/cycles/phase-2-patch/consistency-validation-step5.md"
status: complete
---

# Step 5 Remediation — Track A

## Scope

Track A covers all story-writer-owned findings from
`consistency-validation-step5.md`:

| Finding | Category | Sub-tracks |
|---------|----------|------------|
| BLK-001 | VSDD `level` field wrong on DTU stories (L2/L3 instead of L4) | A1 |
| BLK-002 | `inputs:` dict format (should be YAML string-list) | A2 |
| BLK-003 | Wrong VP filename slugs in `inputs:` string-list | A2/A3 |
| BLK-004 | Stories claiming VP anchor they do not own | A1 |
| BLK-005 | BC paths in `inputs:` use wrong directory prefix or missing slug | A2 |
| IMP-001-A | Core product graph dependency bidirectionality gaps (9 edges) | A3 |
| IMP-001-B | DTU stories cross-graph `blocks:` into product graph (24 edges) | A3 |
| IMP-003 | Covered by BLK-004 fix; no separate action needed | — |

---

## Sub-track A1: Level + VP Anchor Fixes

**Findings addressed:** BLK-001, BLK-004

### BLK-001: DTU Story Level Field

All 6 Wave-6 DTU stories had `level: "L2"` or `level: "L3"` (the DTU *fidelity
tier* label) in the VSDD `level:` field, which must always be `"L4"` (the VSDD
hierarchy level for all stories). Fixed by setting `level: "L4"` on all 6 files.

| File | Old Version | New Version | Change |
|------|-------------|-------------|--------|
| `S-6.14-dtu-threatintel.md` | 1.1 | 1.2 | `level: "L2"` → `"L4"`; `blocks: [S-1.14, S-5.06]` → `[]` (IMP-001-B combined) |
| `S-6.15-dtu-nvd.md` | 1.1 | 1.2 | `level: "L2"` → `"L4"`; `blocks: [S-1.14, S-5.06]` → `[]` |
| `S-6.16-dtu-datadog.md` | 1.1 | 1.2 | `level: "L3"` → `"L4"`; `blocks: [S-5.09]` → `[]` |
| `S-6.17-dtu-splunk-hec.md` | 1.1 | 1.2 | `level: "L3"` → `"L4"`; `blocks: [S-5.09]` → `[]` |
| `S-6.18-dtu-elasticsearch.md` | 1.1 | 1.2 | `level: "L3"` → `"L4"`; `blocks: [S-5.09]` → `[]` |
| `S-6.19-dtu-otlp.md` | 1.1 | 1.2 | `level: "L3"` → `"L4"`; `blocks: [S-5.09]` → `[]` |

### BLK-004: VP Anchor Ownership Errors

Stories claimed VPs whose `anchor_story` in VP-INDEX points elsewhere.

| File | Old Version | New Version | Change |
|------|-------------|-------------|--------|
| `S-4.03-detection-rules.md` | 1.2 | 1.3 | Removed VP-030 from `verification_properties:` (anchor in S-4.01). Updated Task 9 VP-030 bullet + Verification Properties table row to note "proved in S-4.01." |
| `S-6.06-dtu-common.md` | 1.1 | 1.2 | Removed VP-033 and VP-036 from `verification_properties:` (anchors in S-6.07). Body already correctly noted "No VPs directly owned by this story." |

---

## Sub-track A2: Inputs Format + Path Fixes

**Findings addressed:** BLK-002, BLK-003, BLK-005

### Conversion Rule

Dict format (`- path: ... input-hash: null`) → YAML string-list (`- ".factory/specs/..."`).  
BC paths: `architecture/behavioral-contracts/BC-X.XX.NNN.md` → `.factory/specs/behavioral-contracts/BC-X.XX.NNN-<slug>.md`.  
VP paths: `specs/vp-NNN.md` → `.factory/specs/verification-properties/vp-NNN-<slug>.md`.  
`input-hash:` field set to `"[pending-recompute]"` sentinel on all modified files.

| File | Old Version | New Version | Findings | Changes |
|------|-------------|-------------|----------|---------|
| `S-3.03-explain-query.md` | 1.1 | 1.2 | BLK-002, BLK-005 | Dict → string-list; BC-2.11.010 path fixed with slug `-explain-query-tool` |
| `S-3.04-alias-system.md` | 1.1 | 1.2 | BLK-002, BLK-003, BLK-005 | Dict → string-list; 5 BC slugs fixed; 4 VP slugs fixed (vp-012 alias-depth-limit, vp-013 alias-cycle-detection, vp-025 cache-key-deterministic, vp-037 alias-expansion-no-panic) |
| `S-3.05-pagination-caching.md` | 1.2 | 1.3 | BLK-002, BLK-005 | Dict → string-list; 6 BC slugs fixed (BC-2.07.001 through .006) |
| `S-3.06-prismql-write-parser.md` | 1.1 | 1.2 | BLK-002, BLK-005 | Dict → string-list; BC-2.11.004 path fixed with slug `-prismql-pipe-mode` |
| `S-3.07-write-execution.md` | 1.1 | 1.2 | BLK-002, BLK-005 | Dict → string-list; 5 BC slugs fixed (BC-2.04.001/005/007/008, BC-2.05.009) |
| `S-3.08-hidden-columns.md` | 1.1 | 1.2 | BLK-002 | Dict → string-list; prd.md only |
| `S-3.09-progress-streaming.md` | 1.1 | 1.2 | BLK-002 | Dict → string-list; prd.md only |
| `S-3.10-multi-table-joins.md` | 1.1 | 1.2 | BLK-002 | Dict → string-list; prd.md only |
| `S-3.11-query-timeout.md` | 1.1 | 1.2 | BLK-002 | Dict → string-list; prd.md only |
| `S-3.12-schema-introspection.md` | 1.1 | 1.2 | BLK-002 | Dict → string-list; prd.md only |
| `S-3.13-dynamic-table-availability.md` | 1.3 | 1.4 | BLK-002 | Dict → string-list; prd.md only |
| `S-4.01-schedule-crud.md` | 1.3 | 1.4 | BLK-002, BLK-003, BLK-005 | Dict → string-list; 5 BC slugs fixed; 2 VP slugs fixed (vp-026-splay-deterministic, vp-030-schedule-rule-caps) |
| `S-1.02-entity-types.md` | 1.2 | 1.3 | BLK-003 | VP slug corrections: vp-005 → `-case-state-machine`; vp-006 → `-case-state-no-self-transitions` |
| `S-1.03-capability-resolution.md` | 1.1 | 1.2 | BLK-003 | VP slug corrections: vp-002 → `-capability-deny-by-default`; vp-003 → `-capability-most-specific-wins`; vp-004 → `-capability-deny-overrides-allow` |
| `S-1.04-ocsf-schema-loading.md` | 1.1 | 1.2 | BLK-003 | VP slug corrections: vp-016 → `-ocsf-output-valid-protobuf`; vp-022 → `-ocsf-normalizer-no-panic` |
| `S-1.05-ocsf-field-mapping.md` | 1.1 | 1.2 | BLK-003 | VP slug correction: vp-017 → `-ocsf-unmapped-fields-preserved` |

---

## Sub-track A3: Dependency Graph Bidirectionality Fixes

**Findings addressed:** IMP-001-A (9 core product graph edges), IMP-001-B (24 DTU
cross-graph blocks edges)

### IMP-001-B: DTU Cross-Graph blocks Removal

DTU stories model *test infrastructure*. They must not have `blocks:` edges into the
product graph. Product stories that need DTU clones should list them in their own
`depends_on:` if needed (they do not, per current design). Removed all cross-graph
`blocks:` edges from 7 DTU stories. (S-6.14 through S-6.19 were handled in A1 above
since those files required level fixes simultaneously.)

| File | Old Version | New Version | Blocks Removed |
|------|-------------|-------------|----------------|
| `S-6.07-dtu-crowdstrike.md` | 1.1 | 1.2 | `[S-3.06, S-3.07]` → `[]` |
| `S-6.08-dtu-claroty.md` | 1.1 | 1.2 | `[S-3.02]` → `[]` |
| `S-6.09-dtu-cyberint.md` | 1.1 | 1.2 | `[S-3.02]` → `[]` |
| `S-6.10-dtu-armis.md` | 1.1 | 1.2 | `[S-3.02]` → `[]` |
| `S-6.11-dtu-slack.md` | 1.2 | 1.3 | `[S-4.08, S-5.06]` → `[]` |
| `S-6.12-dtu-pagerduty.md` | 1.2 | 1.3 | `[S-4.08, S-5.06]` → `[]` |
| `S-6.13-dtu-jira.md` | 1.2 | 1.3 | `[S-4.08, S-5.06]` → `[]` |

### IMP-001-A: Core Product Graph Bidirectionality

The 9 missing edges identified by the validator. For each edge A→B, either added B to
A's `blocks:` or added A to B's `depends_on:`. Chose the canonical form per the
relationship direction.

| File | Old Version | New Version | Edge Fixed | Change |
|------|-------------|-------------|------------|--------|
| `S-2.04-audit-construction.md` | 1.1 | 1.2 | S-2.04 → S-5.10 | Added S-5.10 to blocks |
| `S-5.10-audit-trail-forwarding.md` | 1.2 | 1.3 | S-5.09 → S-5.10 | Added S-5.09 to depends_on |
| `S-2.01-rocksdb-init.md` | 1.1 | 1.2 | S-2.01 → S-6.01 | Added S-6.01 to blocks |
| `S-5.01-mcp-bootstrap.md` | 1.4 | 1.5 | S-5.01 → S-6.01 | Added S-6.01 to blocks |
| `S-6.01-cli-startup.md` | 1.1 | 1.2 | S-6.01 → S-6.04/S-6.05 | Added S-6.04, S-6.05 to blocks |
| `S-3.02-query-materialization.md` | 1.1 | 1.2 | S-3.02 → S-3.10/S-3.11/S-3.12/S-3.13 | Added S-3.10, S-3.11, S-3.12, S-3.13 to blocks |
| `S-2.06-datasource-trait.md` | 1.1 | 1.2 | S-2.06 → S-3.12 | Added S-3.12 to blocks |
| `S-4.01-schedule-crud.md` | 1.3 | 1.4 | S-4.01 → S-5.01 | Added S-5.01 to blocks (combined with A2 inputs fixes) |
| `S-0.01-ci-cd-pipeline.md` | 1.1 | 1.2 | Phantom S-0.01 → S-1.01 | Removed S-1.01 from blocks (S-1.01 does not list S-0.01 in depends_on; not a real prerequisite edge) |

---

## Notable Judgment Calls

### JC-001: S-4.03 VP-030 body edit scope
BLK-004 required removing VP-030 from S-4.03's `verification_properties:` frontmatter.
The body already had a Task 9 entry and a Verification Properties table row referencing
VP-030. Rather than deleting those references (which would lose context for the
implementer), they were updated to say "VP-030 Kani proof is anchored in S-4.01 —
verify that story passes before testing detection rule enforcement here." This preserves
the cross-story dependency signal without falsely claiming VP ownership.

### JC-002: S-0.01 phantom blocks edge
The consistency validator flagged S-0.01 `blocks: [S-1.01]` as asymmetric because
S-1.01's `depends_on:` does not include S-0.01. Two resolution options:
  (a) Add S-0.01 to S-1.01's `depends_on:` — forces S-1.01 into Wave 1 after Wave 0
  (b) Remove S-1.01 from S-0.01's `blocks:` — acknowledges CI pipeline is an enabler,
      not a strict build-order prerequisite for the entity-type implementation
Option (b) chosen. S-1.01 can build in parallel with or after S-0.01 since the
Rust tests run on the shared CI pipeline but S-1.01's code does not import S-0.01.
S-0.01's Dev Notes already document this relationship as an "indirect blocks" caveat.

### JC-003: DTU level field — VSDD level vs. DTU fidelity tier
The DTU stories used the `level:` frontmatter field to encode the DTU fidelity tier
(L2/L3/L4). However, in the VSDD schema, `level:` means the document hierarchy level
(L1=PRD, L2=arch, L3=spec, L4=story). All stories are L4 by definition. The DTU
fidelity tier is correctly encoded in the story body (e.g., "Fidelity: L2") and the
DTU spec filename. The `level:` frontmatter field was corrected to `"L4"` on all 6
affected stories; the body fidelity labels were not changed.

### JC-004: IMP-001-B — DTU blocks removal vs. product depends_on addition
Two options for resolving DTU cross-graph edges:
  (a) Remove `blocks:` from DTU stories (DTU is test infra, not a prerequisite)
  (b) Add DTU stories to product story `depends_on:` (product stories depend on clones)
Option (a) chosen for all 13 DTU stories. The DTU clones are test infrastructure; the
product implementation must not be gated on test infrastructure build order. The correct
testing model is: implementer builds the feature, test-writer references the DTU clone
in integration tests. The wave scheduler already puts DTU stories in Wave 1 (no
dependencies), so they will always be available before product stories that need them.

---

## File Count Summary

| Sub-track | Files Modified | Findings Closed |
|-----------|---------------|-----------------|
| A1 (level + VP anchor) | 8 | BLK-001, BLK-004 |
| A2 (inputs format + path) | 16 | BLK-002, BLK-003, BLK-005 |
| A3-IMP-001-B (DTU cross-graph) | 7 (+ 6 from A1) | IMP-001-B |
| A3-IMP-001-A (core product graph) | 9 | IMP-001-A |
| **Total unique files** | **33** | **BLK-001–005, IMP-001-A/B** |

IMP-003 (schema version drift) was confirmed covered by BLK-004 fix. No separate
action taken.

---

## input-hash Sentinel Status

All files converted from dict-format `inputs:` had their `input-hash:` top-level
field set to `"[pending-recompute]"`. State-manager must run
`compute-input-hash <file> --update` on each before the final build gate check.

Files requiring recompute (A2 sub-track, inputs content changed):
- S-3.03, S-3.04, S-3.05, S-3.06, S-3.07, S-3.08, S-3.09, S-3.10, S-3.11,
  S-3.12, S-3.13, S-4.01, S-1.02, S-1.03, S-1.04, S-1.05

Files with unchanged inputs (A1, A3 sub-tracks — `inputs:` list not modified):
- S-6.06, S-6.07, S-6.08, S-6.09, S-6.10, S-6.11, S-6.12, S-6.13, S-6.14,
  S-6.15, S-6.16, S-6.17, S-6.18, S-6.19, S-4.03, S-2.04, S-5.10, S-2.01,
  S-5.01, S-6.01, S-3.02, S-2.06, S-0.01
  (input-hash values on these files are unchanged and remain valid)
