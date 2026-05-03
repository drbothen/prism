---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 4
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-03T00:00:00Z
predecessor: pass-3.md (BLOCKED 8 findings; remediated 2026-05-02)
verdict: BLOCKED
findings_count: 7
severity_breakdown: { CRITICAL: 0, HIGH: 2, MEDIUM: 3, LOW: 2, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-03
remediation_commits: [15fa97e6]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 4

**Verdict:** BLOCKED (7 findings: 0C/2H/3M/2L/0OBS)
**Trajectory:** 38→17→8→7 (continued descent; HIGH-class issues are partial-fix-regression patterns)
**Convergence window reset** — pass-3 was BLOCKED; window remains 0/3.

---

## HIGH Findings

### P4-VPINDEX-A-H-001 — VP-053 module and anchor drift in VP-INDEX

**Severity:** HIGH
**File:** `.factory/specs/behavioral-contracts/VP-INDEX.md`

VP-INDEX lists VP-053 with a stale module annotation and an incorrect section anchor. The `module` field reads `prism-core` but per the verification-coverage-matrix (v1.24) VP-053 belongs to `prism-core` (kani). The anchor `#vp-053-resolved-case-disposition-non-null` does not match the canonical heading format established by the VP-INDEX anchor convention. Additionally, the `bc_anchor` field for VP-053 references `BC-2.14.006` without the required `[§anchor]` suffix introduced in VP-INDEX v1.20 for all post-Wave-3 entries.

**Required fix:** Update VP-053 entry: add `[§resolved-case-disposition]` anchor suffix to `bc_anchor`; confirm module = `prism-core`; update section anchor to match canonical heading slug.

---

### P4-XADR-A-H-001 — 4 ADRs body Status field stale at v0.3 (should be v0.4)

**Severity:** HIGH
**Files:** `ADR-013.md`, `ADR-015.md`, `ADR-016.md`, `ADR-018.md`

Pass-3 remediation elevated ADR-013, ADR-015, ADR-016, and ADR-018 to `status: v0.4` in their YAML frontmatter. However, the **body** of each ADR contains a `## Status` section with a `Status:` prose line still reading `v0.3 — PROPOSED`. The frontmatter and body are now inconsistent. This is a class of finding that has recurred across pass-1, pass-2, and pass-3 (partial-fix pattern: frontmatter updated, body Status section missed). The body Status line is load-bearing for LLM-agent consumers that scan the body rather than parsing YAML.

**Required fix:** In each of ADR-013, ADR-015, ADR-016, ADR-018: update the `## Status` section body line from `Status: v0.3 — PROPOSED` to `Status: v0.4 — PROPOSED`.

---

## MEDIUM Findings

### P4-VPINDEX-A-M-001 — VP-138 still listed as P1 in VP-INDEX summary table

**Severity:** MEDIUM
**File:** `.factory/specs/behavioral-contracts/VP-INDEX.md`

VP-138 (Cross-org case access denied, INV-CASE-003) was elevated from P1 to P0 in this pass. The VP-INDEX summary table and the VP-138 row's `priority` field still show `P1`. The verification-architecture.md and verification-coverage-matrix.md reflect the P0 elevation (via this remediation), but VP-INDEX is the authoritative index and its stale P1 designation will mislead the adversary on the next pass if not corrected.

**Required fix:** In VP-INDEX, update VP-138 `priority: P1` → `priority: P0`; update summary table P0/P1 counts accordingly.

---

### P4-STATE-A-M-002 — STATE.md total_vps_added field stale (8 not 9)

**Severity:** MEDIUM
**File:** `.factory/STATE.md`

`wave_4_phase_4_a_preflight.total_vps_added` reads `"8 [VP-137..VP-144]"`. VP-145 was added during pass-1 remediation (ADR-017 §3.3 reopen_count monotonicity) and is listed in `phase_1_vps_added: [VP-137, VP-138]` — wait, VP-145 was added in pass-1 remediation alongside ADR-017 upgrade. The total should be 9 (VP-137..VP-145). The VP range string `[VP-137..VP-144]` excludes VP-145.

**Required fix:** Update `total_vps_added` to `"9 [VP-137..VP-145]"`.

---

### P4-S406-A-M-003 — S-4.06 dedup_window_secs anchoring ambiguity

**Severity:** MEDIUM
**File:** `.factory/stories/S-4.06.md`

S-4.06 v1.12 references `dedup_window_secs` as the canonical field name for the case deduplication window configuration. ADR-015 §5 and ADR-017 §4 use `dedup_window` (duration type, no `_secs` suffix) as the canonical field name. The story body uses `dedup_window_secs: u64` (seconds integer), which conflicts with the ADR-015/017 canonical `dedup_window: Duration`. This is not a new divergence (it was present in v1.11) but pass-3 remediation did not address it, and with VP-140 (dedup window scheduling-time resolution) now at P1, the field name disagreement creates a spec-impl ambiguity that will surface during story delivery.

**Required fix:** Align S-4.06 `dedup_window_secs` → `dedup_window: Duration` to match ADR-015 §5 canonical field name; add a note that serde deserialization accepts `dedup_window_secs` integer as a Duration migration path.

---

## LOW Findings

### P4-VPINDEX-A-L-001 — UUID v4 typo in VP-047 entry

**Severity:** LOW
**File:** `.factory/specs/behavioral-contracts/VP-INDEX.md`

VP-047 entry description reads "UUID v4 validation: non-v4 always rejected, v4 always accepted". The correct UUID version is v7 (per BC-2.18.009 and ADR-013). This is a copy-paste typo from an earlier draft that survived through three remediation passes. The verification-architecture.md Provable Properties Catalog correctly reads "UUID v7 validation" (line VP-047 entry).

**Required fix:** VP-INDEX VP-047 description: `v4` → `v7` in both the title field and the description body.

---

### P4-ADR017-A-L-002 — ADR-017 §5 stub not expanded

**Severity:** LOW
**File:** `.factory/specs/architecture/ADR-017.md`

ADR-017 §5 (Implementation Notes) contains a two-line stub: `_Implementation notes TBD during story delivery._`. Given that ADR-017 is at v0.4 and covers the case lifecycle invariants with VP-051/VP-138/VP-145 now anchored, the §5 stub is the last unresolved deferred section. It does not block convergence but is a quality gap that the spec-reviewer flagged in SR-401-001 (deferred to Phase 4.B). Noted here for tracking completeness; not blocking.

**Required fix (deferred):** Expand ADR-017 §5 with implementation guidance during Phase 4.B polish. Not blocking Pass 5.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 4 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 2 |
| **Novelty score** | 5/7 = 0.71 |
| **Median severity** | 2.5 |
| **Trajectory** | 38→17→8→7 |
| **Verdict** | FINDINGS_REMAIN |

## Convergence Assessment

**Trajectory:** 38 → 17 → 8 → 7

The descent continues. No regressions between pass-3 and pass-4. Both HIGH findings are instances of known partial-fix patterns (body/frontmatter sync gap for ADR Status fields; VP-INDEX anchor/priority gap). The MEDIUM findings are propagation issues. With these 7 findings remediated, pass-5 has a clear path to CLEAN: 2H partial-fix patterns are the dominant risk class; if body Status sync is complete across all 4 ADRs and VP-INDEX VP-138/VP-053 are corrected, no further blockers are expected.

**Pass 5 target:** CLEAN (0H/0M/0L) — opens convergence window 1/3.
