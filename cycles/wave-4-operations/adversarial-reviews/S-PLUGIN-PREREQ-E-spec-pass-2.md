---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 2
scope: spec
verdict: BLOCKED
total_findings: 9
severity_breakdown:
  critical: 0
  high: 3
  medium: 4
  low: 1
  observation: 1
in_scope_findings: 8
observations_queued: 1
produced_by: adversary
reviewed_at: 2026-05-15
fix_burst: fix-burst-2
fix_burst_closed_at: D-576
streak_after_fix: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 2

**Verdict: BLOCKED — 9 findings (0C + 3H + 4M + 1L + 1OBS)**

Fix-burst-2 closed 8 in-scope findings. 3 findings were closure regressions from fix-burst-1
(FB1 regressions caught by pass-2 adversary — paper-fix + 2 sibling-sweep gaps). 4 new medium
findings closed in-scope. 1 OBS process-gap queued for cycle-close. Streak reset to 0/3.
Adversary pass-3 NEXT.

---

## Key Discovery: FB1 Regression Detections (TD-VSDD-059 + TD-VSDD-060 active)

Pass-2 adversary found 3 regressions from fix-burst-1 closures:

1. **F-LP2-HIGH-001 (paper-fix):** BC-2.16.012 EC-016-012-004 was claimed closed in FB1 but the
   body still read "Implementer chooses; last-writer-wins, OR an error is returned" — directly
   contradicting ADR-026 D7 v1.2. TD-VSDD-059 paper-fix detection active and confirmed working.

2. **F-LP2-HIGH-002 (sibling-sweep gap):** 5 TD-A-003 alias sites were missed by FB1's sweep:
   HS-PREREQ-E-003 ×2, VP-156, ADR-027, and forward-task-map. FB1 F-LP1-MED-004 had closed 11
   sites but 5 siblings were not swept. TD-VSDD-060 sibling-site sweep discipline triggered.

3. **F-LP2-HIGH-003 (sibling-sweep gap):** 2 additional §C5 phantom-heading citations in
   ADR-027 (D5 narrative + §Source/Origin convention note) were not caught by FB1's sweep. FB1
   F-LP1-HIGH-003 had closed 18 sites across 3 BCs + story but missed 2 ADR-027 sites.

---

## Finding Summary Table

| ID | Severity | Status | Closed By | Description |
|----|----------|--------|-----------|-------------|
| F-LP2-HIGH-001 | HIGH | CLOSED | product-owner | BC-2.16.012 EC-016-012-004 paper-fix: FB1 claimed closure but body still specified "Implementer chooses; last-writer-wins, OR an error is returned" — direct contradiction of ADR-026 D7 mandate. Body rewritten to specify `Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))` as the sole outcome. EC-016-012-005 companion fix: AfterBoot error code updated to E-PLUGIN-020 per PO taxonomy v1.27 allocation. BC-2.16.012 v1.2→v1.3. |
| F-LP2-HIGH-002 | HIGH | CLOSED | product-owner + architect | TD-A-003 alias sibling-sweep: 5 sites missed by FB1 sweep. HS-PREREQ-E-003 §Scenario Description body + §HS-PREREQ-E-003-03 heading (×2 — product-owner); VP-156 §Property Statement (×1 — architect); ADR-027 §Consequences trade-off row (×1 — architect); forward-task-map.md footnote (×1 — state-manager). All 5 sites canonicalized to TD-S-PLUGIN-PREREQ-A-003. |
| F-LP2-HIGH-003 | HIGH | CLOSED | architect | §C5 phantom-heading sibling-sweep: 2 sites in ADR-027 missed by FB1 sweep. D5 narrative (line 124) and §Source/Origin convention note (line 228) both cited `ADR-023 §C5` — corrected to `ADR-023 §Architectural Constraints (C5 bullet, Rule N)` per POL-21. |
| F-LP2-MED-001 | MEDIUM | CLOSED | product-owner + architect | E-PLUGIN error-code namespace collision chain: Initial D7 routing in ADR-026 v1.2 used E-PLUGIN-001 (occupied — umbrella boot runtime-panic). Architect reassigned to E-PLUGIN-012 (DuplicateWriteToolRegistration) + E-PLUGIN-013 (WriteToolRegistrationAfterBoot) in ADR-026 v1.3. PO discovered E-PLUGIN-013 also occupied (allowed_urls manifest validation, taxonomy v1.19, BC-2.17.007). PO allocated E-PLUGIN-020 (next free after E-PLUGIN-019/FormatVersionMissing) for WriteToolRegistrationAfterBoot. Architect updated ADR-026 v1.3→v1.4. error-taxonomy v1.26→v1.27 (E-PLUGIN-012 + E-PLUGIN-020 rows authored). 2-collision discovery chain demonstrates POL-25 grep-before-write discipline. |
| F-LP2-MED-002 | MEDIUM | CLOSED | architect | VP-156 happens-before claim: §Property Statement title and body claimed a happens-before memory-model invariant that proptest cannot verify. Option (b) chosen: reworded to uniqueness-only. Structural visibility guarantee documented as structural (std::sync::RwLock contract + ADR-022 boot ordering cover) not proptest-verified. VP-156 v0.1→v0.2. |
| F-LP2-MED-003 | MEDIUM | CLOSED | architect | VP-156 source_invariant schema break: VP-156 frontmatter listed `source_invariant: INV-INVALIDATION-EXT-001` — a forward-reference to an invariant not yet codified. Field changed to `null`; invariant trace preserved in §Source Contract body via existing BC-2.16.012 INV-INVALIDATION-EXT-001 cite per VP template. VP-156 v0.1→v0.2 (combined with MED-002). |
| F-LP2-MED-004 | MEDIUM | CLOSED | product-owner | Story Red Gate test ordering: Red Gate test table rows were not grouped by BC, making it harder to validate coverage. Rows reordered by BC group (BC-2.16.011 rows first, then BC-2.16.012 rows). Story v1.2→v1.3. |
| F-LP2-LOW-001 | LOW | CLOSED | architect | ADR-027 §Source/Origin convention: §Source/Origin section lacked a convention note explaining why the CustomAdapter symbol originated in prism-sensors rather than prism-core. Convention note added per F-LP2-HIGH-003 sibling-sweep visit. ADR-027 v1.1→v1.2. |
| OBS-LP2-001 | OBSERVATION | QUEUED-CYCLE-CLOSE | — | [process-gap] POL-25 sweep enforcement gap: FB1 TD-A-003 sweep was manually executed but missed 5 sites across HS + VP + ADR + forward-task-map. POL-25 multi-cite propagation sweep requires a complete grep pattern across ALL artifact types. Codification candidate: extend POL-25 to include a mandatory pre-commit grep template covering BCs + VPs + ADRs + HS + forward-task-map + CYCLE-SNAPSHOT. Queued for cycle-close session-review. |

---

## Error Code Resolution Chain (F-LP2-MED-001)

Demonstrates POL-25 grep-before-write discipline — chain of 2 collision discoveries:

```
ADR-026 D7 v1.2 (FB1) → E-PLUGIN-001 (OCCUPIED: umbrella runtime-panic)
                       ↓ architect re-routes
ADR-026 v1.3       → E-PLUGIN-012 (free: DuplicateWriteToolRegistration) ✓
                       + E-PLUGIN-013 (OCCUPIED: allowed_urls manifest validation, v1.19)
                       ↓ PO discovers collision, allocates next free
error-taxonomy v1.27   → E-PLUGIN-012 (DuplicateWriteToolRegistration) ✓
ADR-026 v1.4           + E-PLUGIN-020 (WriteToolRegistrationAfterBoot) ✓
```

---

## Key Decisions from Fix-Burst-2

| Decision | Chosen Option | Rationale |
|----------|---------------|-----------|
| EC-016-012-004 fix | Rewrite body to error-on-duplicate | Paper-fix detection (TD-VSDD-059); last-writer-wins explicitly forbidden per ADR-026 D7 |
| E-PLUGIN error codes | E-PLUGIN-012 (Duplicate) + E-PLUGIN-020 (AfterBoot) | 2-collision chain resolved; E-PLUGIN-013 occupied since v1.19 |
| VP-156 happens-before | Option b: uniqueness-only reframe | std::sync::RwLock + ADR-022 boot ordering cover happens-before structurally; proptest covers uniqueness |
| VP-156 source_invariant | → null + body cite | INV-INVALIDATION-EXT-001 not yet codified; null per VP template; trace preserved in body |
| Story Red Gate ordering | Group by BC | Readability + coverage auditability per adversary rubric |
| §Source/Origin convention | Added convention note in ADR-027 | Found during §C5 sibling-sweep visit (F-LP2-HIGH-003) |

---

## Artifact Versions After Fix-Burst-2

| Artifact | Before FB2 | After FB2 |
|----------|------------|-----------|
| ADR-026 | v1.2 | v1.4 (v1.2→v1.3 E-PLUGIN routing; v1.3→v1.4 E-PLUGIN-013→E-PLUGIN-020) |
| ADR-027 | v1.1 | v1.2 (§C5 sibling-sweep + §Source/Origin + TD-A-003 sweep) |
| BC-2.16.012 | v1.2 | v1.3 (EC-016-012-004 paper-fix + EC-016-012-005 code update) |
| VP-156 | v0.1 | v0.2 (happens-before → uniqueness-only + source_invariant → null + TD-A-003 sweep) |
| HS-PREREQ-E-003 | v1.1 | v1.2 (TD-A-003 ×2 canonicalized) |
| S-PLUGIN-PREREQ-E story | v1.2 | v1.3 (Red Gate test ordering grouped by BC) |
| error-taxonomy | v1.26 | v1.27 (E-PLUGIN-012 + E-PLUGIN-020 rows authored) |
| forward-task-map.md | — | amended (TD-A-003 footnote canonicalized) |
| STATE + HANDOFF | v7.280 | v7.281 |

---

## Closure Regression Analysis (TD-VSDD-059 / TD-VSDD-060)

**3 of 9 findings were FB1 closure regressions:**

| Finding | Type | FB1 Claim | Pass-2 Discovery | Root Cause |
|---------|------|-----------|------------------|------------|
| F-LP2-HIGH-001 | Paper-fix | EC-016-012-004 closed | Body still contradicted ADR-026 D7 | Changelog updated but body not rewritten |
| F-LP2-HIGH-002 | Sibling-sweep gap | 11 TD-A-003 sites closed | 5 additional sites in HS + VP + ADR + map | Grep pattern did not cover all file types |
| F-LP2-HIGH-003 | Sibling-sweep gap | 18 §C5 sites in BCs + story | 2 additional sites in ADR-027 | ADRs not in sweep target list for FB1 |

**Implication:** BC-5.39.001 3-CLEAN streak reset to 0/3. Pass-3 must confirm all regressions
closed and find 0 new findings to advance streak to 1/3.

---

## Streak Status

Pass 2 BLOCKED → Fix-burst-2 CLOSED 8/9 in-scope + 1 OBS queued → Streak 0/3.
Adversary pass-3 dispatch NEXT (fresh-context, BC-5.39.001 3-CLEAN protocol).

_Full adversary output is in the orchestrator dispatch log for this session. This file is the
structured audit-trail record of pass-2 findings and fix-burst-2 closures per the Single-Commit
Burst Protocol (TD-VSDD-053 / D-576)._
