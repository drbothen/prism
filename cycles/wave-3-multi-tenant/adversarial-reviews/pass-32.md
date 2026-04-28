---
document_type: adversarial-review-pass
phase: 3
wave: 3
sub_phase: 3.A
pass: 32
verdict: FINDINGS_OPEN
findings_critical: 0
findings_major: 1
findings_minor: 0
findings_process_gap: 0
window_position: "0/3 → 0/3"
predecessor_sha: df1b96e8
date: 2026-04-28
producer: adversary
reviewers: [adversary]
inputs: [".factory/specs/wave-3/*", ".factory/stories/S-3.*.md", ".factory/specs/architecture/*", ".factory/specs/domain-spec/*", ".factory/specs/behavioral-contracts/BC-3.*"]
---

# Wave 3 Phase 3.A — Adversarial Pass 32

**Verdict:** FINDINGS_OPEN
**Counts:** 0 critical · 1 major · 0 minor · 0 process-gap
**Window position:** 0/3 → 0/3 (no advance — findings non-zero)
**Predecessor SHA:** df1b96e8
**26th consecutive 0-critical pass (P7-P32).**

## Pass 31 fix verification (all confirmed)

- M-31-001 (L2-INDEX CAP-040 SS-21 annotation): CONFIRMED at L2-INDEX.md:94 — "Multi-Tenant Adapter Dispatch Mode (SS-21 registry / SS-06 config parsing / SS-01 enforcement, internal)". Changelog v1.10 at line 124.
- M-31-002 (BC-3.3.004 R-CUST-013 cross-ref removed): CONFIRMED at BC-3.3.004:75 — note row now reads only `schema_version absent → E-CFG-030 (BC-3.3.003); schema_version unsupported value → E-CFG-031 (BC-3.3.003).` Changelog v0.9 at line 178.
- m-31-002 (ADR-012 "seven subsystems"): CONFIRMED at ADR-012:43 — "22 crates across seven subsystems (SS-01..SS-06 plus SS-21)".
- m-31-001 (BC-3.7.001 Open Questions phrasing) and m-31-003 (verification-coverage-matrix BC-3.1.001 exception): file states clean at v0.8 / v1.22 respectively.

VP-INDEX arithmetic verified: 30 kani + 77 proptest + 4 unit_test + 6 fuzz + 19 integration = 136 total; 113 P0 + 23 P1 = 136.

Sample BC frontmatter SS-21/SS-01/SS-06/SS-05 alignment per focus area 3 verified for BC-3.1.001/003/004 → SS-21 / CAP-038, BC-3.1.002 → SS-05 / CAP-007, BC-3.4.001-004 → SS-01 / CAP-039, BC-3.3.001-004 → SS-06 / CAP-009, BC-3.7.001 → SS-01 / CAP-037, BC-3.2.004 → SS-01 / CAP-040, BC-3.2.005 → SS-06 / CAP-040.

## Critical Findings

(none)

## Major Findings

### Finding M-32-001 (severity: Major) — S-3.0.02 frontmatter `subsystems:` missing SS-21 — sibling-fix gap from M-31-001 propagation

**File:** `/Users/jmagady/Dev/prism/.factory/stories/S-3.0.02-dtu-mode-metadata.md`
**Lines:** 7

**Evidence (verbatim):**
- Line 7 (pre-fix): `subsystems: [SS-01, SS-06]`
- Line 4 title: "prism-core: register DTU_DEFAULT_MODE registry (10-entry DtuRegistryEntry slice) per ADR-007 §2.3"
- Line 6: `target_module: prism-core`
- Line 250 file structure: `crates/prism-core/src/dtu.rs | Create | DtuMode enum + DtuRegistryEntry struct + DTU_DEFAULT_MODE static (10 entries)`
- Lines 54-61 narrative: "ADR-007 §2.3 specifies a centralized compile-time registry in prism-core (co-located with OrgRegistry per D-047), not per-crate declarations."

**Cross-references confirming SS-21 ownership:**
- ARCH-INDEX.md:133 — `SS-21 | Identity & Core Types | system-overview.md, module-decomposition.md | prism-core | Phase 3`
- L2-INDEX.md:94 (post-M-31-001) — `CAP-040 | ... (SS-21 registry / SS-06 config parsing / SS-01 enforcement, internal)`
- ADR-007:171-172 — "The default mode registry is a compile-time constant in prism-core"
- ADR-007:15 frontmatter — `subsystems_affected: [SS-01, SS-03, SS-05, SS-06, SS-21]`
- capabilities.md:65 (CAP-040) — "SS-21 (Identity & Core Types) for the DTU_DEFAULT_MODE compile-time registry"

**Convention check:** S-3.1.01-org-id-newtype.md (prism-core): `subsystems: [SS-21]`. S-3.1.03-org-registry.md (prism-core): `subsystems: [SS-21]`. S-3.0.02 also implements a prism-core artifact (DTU_DEFAULT_MODE registry, co-located with OrgRegistry per D-047) but lists `[SS-01, SS-06]` reflecting the pre-M-30-001 / M-31-001 CAP-040 annotation.

**Issue:** Pass 30 + Pass 31 corrected CAP-040's subsystem annotation in capabilities.md and L2-INDEX.md to name SS-21 as the registry owner. The implementing story (S-3.0.02) was not updated in lockstep — it still listed `[SS-01, SS-06]` describing consumers, not the implementation site. POL-6 (Architecture is subsystem-name source of truth) + POL-4 (semantic anchoring integrity) require the `subsystems:` field on a story to reflect the architectural layer the story implements.

**Fix applied (this pass):** S-3.0.02 line 7 changed from `subsystems: [SS-01, SS-06]` → `subsystems: [SS-21]`. Version bumped 0.3 → 0.4. Changelog entry added documenting M-32-001 fix.

**Sibling-fix risk evaluated:** No prose in S-3.0.02 body referenced "SS-01 and SS-06" in subsystem-context (only in consumer-context, which remains correct). Other prism-core Wave 3 stories (S-3.1.01, S-3.1.03) already follow `[SS-21]` convention.

## Minor Findings

(none)

## Process-Gap Findings

(none)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 32 |
| **New findings** | 1 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (1 / (1 + 0)) |
| **Median severity** | 3.0 (major) |
| **Trajectory** | …→0(P28)→0(P29)→1M+3m+1PG(P30)→2M+3m(P31)→1M(P32) |
| **Verdict** | FINDINGS_REMAIN |

26 consecutive 0-critical passes (P7-P32) preserved. CLEAN passes total: P12, P26, P28, P29.

The finding is a NEW sibling-fix propagation gap not surfaced in prior passes — the third in the documented D-115/D-116/D-117 family. Pass 30 surfaced CAP-040 SS-21 in capabilities.md (D-116). Pass 31 surfaced the L2-INDEX CAP-040 sibling annotation (M-31-001). Pass 32 now surfaces the implementing story (S-3.0.02). Each pass, the fix horizon expands by one artifact.

Recommend after M-32-001 fix burst: Pass 33 looking for the next sibling — likely either (a) another Wave 3 capability/BC artifact still saying "SS-06 registry" instead of "SS-21 registry", or (b) BC-3.2.005 (currently `subsystem: SS-06`, anchored to CAP-040) — may be intentional since BC-3.2.005 covers config-time mode immutability (an SS-06 concern), but warrants a check.

## Files reviewed

Source-of-truth indexes:
- L2-INDEX.md (v1.10), capabilities.md (v1.14), invariants.md (v1.2)
- ARCH-INDEX.md (v1.8), BC-INDEX.md (v4.26), STORY-INDEX.md (v1.62)
- verification-architecture.md (v1.21), verification-coverage-matrix.md (v1.22)

Wave 3 BCs sampled (12 of 22): BC-3.1.001-004, BC-3.2.001/004/005, BC-3.3.001-004, BC-3.4.001-004, BC-3.7.001.

ADRs sampled: ADR-007 (v0.12), ADR-012 (v0.13).

Wave 3 stories sampled (4 of 37): S-3.0.01, S-3.0.02 (finding), S-3.1.03, S-3.1.07.
