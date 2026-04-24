# Wave 1 Integration Gate — Adversarial Pass 8

**Verdict:** BLOCKED (1H + 1M)
**Trajectory:** 11 → 11 → 4 → 3 → 3 → 3 (CLEAN) → 2 (BLOCKED) → 2 (BLOCKED)
**Convergence window:** 0 of 3 clean passes (window stays at 0; Pass 8 BLOCKED)

---

## Part A — Prior-Pass Verification

### Pass 7 findings

| ID | Severity | Status |
|----|----------|--------|
| P3WV1G-A-H-001 | HIGH | RESOLVED — S-6.06 `level: "L4"` → `level: null`; ADR-002 addendum sub-rule added |
| P3WV1G-A-M-001 | MEDIUM | RESOLVED — STATE.md `dtu_critical_path` "8 points" → "7 points" |

### Spot-checks (all prior-pass regressions)

Passes 1–7 findings spot-checked. No regressions detected. All previously remediated findings remain resolved.

---

## Part B — New Findings

### P3WV1H-A-H-001 (HIGH): S-6.20 `level: "harness"` — invalid value per ADR-002 addendum

**File:** `.factory/stories/S-6.20-dtu-demo-server.md` (frontmatter, line 5)
**Observed:** `level: "harness"`
**Expected:** `level: null` (or field omitted)

**Rationale:** ADR-002 addendum (Pass 5, extended Pass 7) defines the exclusive valid value set for `level:` in DTU stories: `{L0, L1, L2, L3, L4}` for fidelity-tier clone stories, or `null`/omitted for shared-infrastructure stories with no fidelity tier. The value `"harness"` is not in this set.

S-6.20 (prism-dtu-demo-server) is a demo harness / orchestration binary — it has no behavioral clone fidelity tier, no BCs, no VPs, and its profile is structurally identical to S-6.06 (shared-infrastructure, harness-only). The ADR-002 addendum sub-rule added in Pass 7 explicitly governs this profile: `level: null`.

Pass 7 remediated S-6.06 but did not sweep S-6.20, even though the ADR-002 addendum scope note named "S-6.06 through S-6.20" as the covered range for the shared-infrastructure sub-rule. S-6.20 was missed because the scope statement was added prophylactically but the pass-7 remediation loop did not verify S-6.20's existing value.

**Required fix:** `level: "harness"` → `level: null` in S-6.20 frontmatter.

---

### P3WV1H-A-M-001 (MEDIUM): S-6.06 `blocks:` list missing S-6.20 — bidirectional dependency-graph drift

**File:** `.factory/stories/S-6.06-dtu-common.md` (frontmatter, line 10)
**Observed:** `blocks: [S-6.07, S-6.08, S-6.09, S-6.10, S-6.11, S-6.12, S-6.13, S-6.14, S-6.15, S-6.16, S-6.17, S-6.18, S-6.19]` (13 entries)
**Expected:** 14 entries including S-6.20

**Rationale:** S-6.20 frontmatter correctly lists `depends_on: [S-6.06, ...]` — the reverse edge is present. However, S-6.06's forward edge (`blocks:`) was not updated when S-6.20 was added to the story corpus on 2026-04-22. S-6.20 depends on S-6.06 (it imports `BehavioralClone` and the DTU fleet management infrastructure from prism-dtu-common), so S-6.06 blocks S-6.20.

This is a bidirectional dependency-graph drift: one direction of the edge is present, the other is absent. Both directions must be represented for dependency-graph tooling and consistency validators to traverse the graph correctly.

**Required fix:** Append `S-6.20` to S-6.06 `blocks:` list (14 entries total).

---

### P3WV1H-A-OBS-001 (OBSERVATION): ADR-002 sub-rule added in Pass 7 lacks provenance annotation

**File:** `.factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md`
**Section:** `### Sub-rule: Shared-Infrastructure DTU Stories`

**Observation:** The original Pass 5 addendum block carries a date + PR marker for traceability. The Pass 7 sub-rule was appended without a provenance annotation (date, burst reference, finding ID). This is not a policy violation — the ADR-002 addendum body still correctly states the rules — but the lack of annotation makes it harder to trace when and why the sub-rule was introduced.

**Informational only.** No remediation required before Pass 9. Opportunistic fix acceptable.

---

## Remediation Required Before Pass 9

| Finding | Fix | Owner |
|---------|-----|-------|
| P3WV1H-A-H-001 | S-6.20 `level: "harness"` → `level: null`; bump v1.7 → v1.8 | state-manager |
| P3WV1H-A-M-001 | S-6.06 `blocks:` append S-6.20 (13 → 14 entries); bump v1.7 → v1.8 | state-manager |
| P3WV1H-A-OBS-001 | Add provenance annotation to ADR-002 Pass 7 sub-rule (opportunistic) | state-manager |

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 8 |
| **New findings** | 2 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 2 / (2 + 0) = 1.0 |
| **Median severity** | 3.5 (HIGH=4 + MEDIUM=3 / 2) |
| **Trajectory** | 11 → 11 → 4 → 3 → 3 → 3 (CLEAN) → 2 (BLOCKED) → 2 (BLOCKED) |
| **Verdict** | FINDINGS_REMAIN |
