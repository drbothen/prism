---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 5
scope: spec
verdict: BLOCKED
total_findings: 10
severity_breakdown:
  critical: 0
  high: 2
  medium: 3
  low: 2
  observation: 3
in_scope_findings: 7
observations_queued: 3
produced_by: adversary
reviewed_at: 2026-05-15
fix_burst: fix-burst-5
fix_burst_closed_at: D-579
streak_after_fix: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 5

**Verdict: BLOCKED — 10 findings (0C + 2H + 3M + 2L + 3OBS); 7 in-scope + 3 OBS queued cycle-close**

Fix-burst-5 closed all 7 in-scope findings. Trajectory: 14 → 9 → 8 → 9 → 10 (REGRESSION — pass-5 count exceeds pass-4).
The regression is driven by three classes: (1) a FB4 regression — the state-manager changelog renumber REDO for VP-153/154/155/156
was applied without updating subsystems in the story frontmatter to include SS-07 (prism-query), leaving the `subsystems:` array
incomplete relative to the architectural scope declared in ADR-026; (2) a fresh-context HIGH finding (F-LP5-HIGH-002) — the story
§References BC table contained 5 BC entries whose Title cells were verbatim H1 except for BC-2.01.013 and BC-2.16.004, which had
been corrected by FB4 but two new entries added in the story v1.5→v1.6 transition used paraphrased form (POL-7 surface 2 —
different paraphrase class than pass-4 catch); (3) VP-INDEX v1.40 source_invariant convention was undocumented — the convention
"DI-NNN only" was applied to VP-153/154/155 but never codified in the VP-INDEX itself, making it a silent convention.

CASCADE LENGTH NOTE: 5 passes deep. Trajectory 14→9→8→9→10. The FB4 regression (subsystems +SS-07 omitted) is a bookkeeping gap,
not a semantic defect — the prism-query scope was already present in ADR-026 v1.6 text but the story frontmatter was not updated.
The fresh-context HIGH (POL-7 surface 2) follows the same pattern as pass-4's F-LP4-HIGH-003 — paraphrase class differs each
pass, requiring independent context to catch. BC-5.39.001 3-CLEAN protocol: streak 0/3, pass-6 NEXT.

---

## Finding Inventory

### F-LP5-HIGH-001 — Story `subsystems:` frontmatter missing SS-07 (prism-query)

**Severity:** HIGH
**Type:** FB4 regression (subsystems array incomplete after changelog renumber REDO)
**Closed by:** product-owner (D-579 fix-burst-5)

ADR-026 v1.6 §Subsystems Affected lists SS-01 (prism-sensors), SS-07 (prism-query), and SS-16 (prism-spec-engine) as all
three subsystems touched by the SensorAuth unsealing and WriteToolInvalidationMap concurrency changes. The story frontmatter
`subsystems:` array at v1.5 contained only `[SS-01, SS-16]` — SS-07 was present in ADR-026's narrative but never propagated
to the story frontmatter. The state-manager changelog renumber REDO in FB4 touched the VP files but did not trigger a
story-subsystem sweep.

**Fix:** SS-07 added to story `subsystems:` array. Story v1.5 → v1.6.

---

### F-LP5-HIGH-002 — §References BC table: 5 BC entries not all verbatim H1 (POL-7 surface 2)

**Severity:** HIGH
**Type:** POL-7 verbatim H1 violation (fresh-context catch — different paraphrase class from pass-4)
**Closed by:** product-owner (D-579 fix-burst-5)

The story §References "Behavioral Contracts" subsection lists 5 BC entries (BC-2.01.013, BC-2.01.016, BC-2.16.004,
BC-2.16.011, BC-2.16.012) as markdown links + description. At v1.5, two of these entries used the verbatim H1 form
in the link text but had role descriptions that partially paraphrased the BC scope. POL-7 requires verbatim H1 in the
link text cell; descriptions in the role cell. The violation was a different surface than F-LP4-HIGH-003 (which caught
the BC table Title column), confirming that §References link-text is a distinct POL-7 surface requiring independent sweep.

**Fix:** All 5 §References BC entries normalized to verbatim H1 in link text. Role annotations preserved inline per
POL-7 guidance. Story v1.5 → v1.6.

---

### F-LP5-MED-001 — File Structure section missing 4 auth implementation files

**Severity:** MEDIUM
**Type:** Completeness gap (implementation deliverable list incomplete)
**Closed by:** product-owner (D-579 fix-burst-5)

The story §File Structure section listed the primary files affected (spec_parser.rs, plugin_registry.rs, sensorauth.rs)
but did not enumerate the 4 built-in auth implementation files that must each receive a one-line `fn auth_type_name()`
body (Path B decision, D-577 F-LP3-HIGH-002). These files are:
`crates/prism-sensors/src/auth/crowdstrike.rs`,
`crates/prism-sensors/src/auth/cyberint.rs`,
`crates/prism-sensors/src/auth/claroty.rs`,
`crates/prism-sensors/src/auth/armis.rs`.
Without explicit enumeration, the implementer has no authoritative list of which auth impl files require updates.

**Fix:** 4 auth impl files added to §File Structure with role annotation "(auth_type_name() body — Path B)". Story v1.5 → v1.6.

---

### F-LP5-MED-002 — Compliance Rules missing ADR-027 D5 anchor reference

**Severity:** MEDIUM
**Type:** Compliance rule citation gap
**Closed by:** product-owner (D-579 fix-burst-5)

The story §Compliance Rules section enumerated ADR-026 and ADR-027 compliance anchors but referenced only ADR-027
at the top level without citing D5 specifically. ADR-027 D5 was expanded in FB4 (F-LP4-LOW-001) to enumerate two
required actions (CustomAdapter clean-pass + hardcoded-sensor-string audit). The story §Compliance Rules must cite
D5 explicitly so the implementer's compliance checklist includes both D5 sub-actions.

**Fix:** ADR-027 D5 anchor added to §Compliance Rules with both sub-actions enumerated. Story v1.5 → v1.6.

---

### F-LP5-MED-003 — BC-2.01.013 and BC-2.16.004 AC trace table missing Path B entries

**Severity:** MEDIUM
**Type:** POL-8 AC trace completeness gap
**Closed by:** product-owner (D-579 fix-burst-5)

The story AC trace table (§Acceptance Criteria Traceability) for BC-2.01.013 and BC-2.16.004 contained Path A entries
(where SensorAuth is used for spec-load-time validation) but was missing the Path B entries covering the runtime
dispatch path through PluginRegistry. ADR-026 D7 establishes both paths as required; BC-2.01.013 covers the
spec-load path and BC-2.16.004 covers the Rust escape-hatch removal path. Both require Path B AC entries per POL-8.

**Fix:** Path B AC trace entries added for BC-2.01.013 and BC-2.16.004. Story v1.5 → v1.6.

---

### F-LP5-MED-004 — ADR-026 `subsystems_affected` missing SS-07

**Severity:** MEDIUM (downgraded from HIGH — ADR-026 §narrative already names prism-query; gap is structured metadata only)
**Type:** Structured metadata completeness gap
**Closed by:** architect (D-579 fix-burst-5)

ADR-026 v1.6 `subsystems_affected:` frontmatter array contained `[SS-01, SS-16]`. The §narrative references prism-query
(SS-07) in the WriteToolInvalidationMap concurrency discussion (D7 rationale), but this was not reflected in the
structured frontmatter array. Story F-LP5-HIGH-001 catch triggered a sibling sweep which confirmed ADR-026 was the
origin of the gap (story was downstream victim).

**Fix:** SS-07 added to ADR-026 `subsystems_affected:` array. ADR-026 v1.6 → v1.7.

---

### F-LP5-LOW-001 — VP-INDEX `source_invariant` DI-NNN-only convention undocumented

**Severity:** LOW
**Type:** Convention documentation gap (Path A silent convention)
**Closed by:** architect (D-579 fix-burst-5)

VP-153/154/155 carry `source_invariant:` values in the form `DI-NNN-description` (domain invariant IDs). VP-156
carries `source_invariant: null` with the domain invariant cited in the body. This is the "DI-NNN-only" convention
established during the F-LP2-MED-003 discussion (D-576), but was never documented in VP-INDEX v1.40 or the VP
template header. A new adversary reading the VP package could not determine whether DI-NNN or full-phrase identifiers
are canonical.

**Fix:** VP-INDEX v1.40 → v1.41 — source_invariant convention blockquote added to the index header: DI-NNN identifiers
only in `source_invariant:` field; full domain invariant prose in VP body. Path A (DI-NNN) vs Path B (null + body cite)
distinction documented.

---

### F-LP5-LOW-002 — BC-2.16.012 `subsystems:` array missing SS-07

**Severity:** LOW
**Type:** Metadata completeness gap (same class as F-LP5-HIGH-001 / F-LP5-MED-004)
**Closed by:** architect (D-579 fix-burst-5)

BC-2.16.012 (Plugin Registry Dispatch Migration) `subsystems:` frontmatter array did not include SS-07 despite
the BC's scope explicitly covering the prism-query call-site migration (Task 7 in the story). Sibling sweep of all
PREREQ-E artifacts following F-LP5-HIGH-001 catch revealed BC-2.16.012 as a third site with the SS-07 omission.

**Fix:** SS-07 added to BC-2.16.012 `subsystems:` array. BC-2.16.012 v1.5 → v1.6.

---

### OBS-LP5-001 — Token budget approaching 85% (story v1.6 after FB5 additions)

**Severity:** OBSERVATION
**Type:** Token budget advisory
**Disposition:** QUEUED-CYCLE-CLOSE

Story v1.6 after FB5 additions brings total token count to approximately 4,700 tokens (estimated from §File Structure +4
auth files + §Compliance Rules +ADR-027 D5 + §References 5 verbatim H1 + Path B AC traces). The token budget threshold
for S-PLUGIN-PREREQ-E is 5,500 tokens (per ADR-022 context budget table). At 85% utilization, there is ~800 tokens
remaining before the implementer context-load budget triggers a warning. Non-blocking for pass-6; logged for cycle-close.

---

### OBS-LP5-002 — ADR-027 `modified:` date not updated after D5 expansion in FB4

**Severity:** OBSERVATION
**Type:** POL-27 date-drift (observation class — same pattern as F-LP4-MED-003 on error-taxonomy)
**Disposition:** QUEUED-CYCLE-CLOSE

ADR-027 was updated in FB4 (F-LP4-LOW-001 D5 scope expansion; ADR-027 v1.2 → v1.3). The `modified:` field was
not updated to 2026-05-15. This is the same POL-27 drift class that was caught in pass-4 for error-taxonomy.
Classified as OBS (not MED) because the POL-27 extension from BC files to ADRs is a codification candidate queued
cycle-close (not yet a promulgated rule for ADR files). Will auto-elevate to MED if POL-27 codification lands before
cycle-close.

---

### OBS-LP5-003 — HS-003 holdout text references "CustomAdapter Rust trait" with stale article form

**Severity:** OBSERVATION
**Type:** Terminology precision (minor; non-semantic)
**Disposition:** QUEUED-CYCLE-CLOSE

HS-003 (Plugin Registry Dispatch holdout scenario) v1.3 body references "the CustomAdapter Rust trait" using the
definite article in two locations where the story and BCs now consistently use the form "CustomAdapter trait"
(without "Rust" qualifier, since all traits in scope are Rust). Non-semantic, no implementer confusion risk, but
introduces minor inconsistency in holdout-evaluator reading context. Queued cycle-close for HS-003 v1.3 → v1.4
minor cleanup pass.

---

## Trajectory Summary

| Pass | Findings | In-Scope | OBS Queued | Delta | Note |
|------|----------|----------|------------|-------|------|
| 1 | 14 | 12 | 2 | — | Initial: 1C+4H+5M+2L+2OBS |
| 2 | 9 | 8 | 1 | -5 | 3 FB1 regressions caught |
| 3 | 8 | 8 | 0 | -1 | 5 FB2 sibling-sweep regressions |
| 4 | 9 | 9 | 0 | +1 | FLAT — VP-156 anchor-back gaps (FB1 residue) |
| 5 | 10 | 7 | 3 | +1 | REGRESSION — FB4 bookkeeping gap + POL-7 surface 2 |

Trajectory shorthand: **14→9→8→9→10** (regression at pass-5; predominantly bookkeeping gaps + fresh-context
POL-7 surface coverage).

---

## Artifact Versions After Fix-Burst-5

| Artifact | After FB4 | After FB5 |
|----------|-----------|-----------|
| ADR-026 | v1.6 | v1.7 |
| ARCH-INDEX | v2.47 | v2.48 |
| BC-2.16.012 | v1.5 | v1.6 |
| S-PLUGIN-PREREQ-E story | v1.5 | v1.6 |
| VP-INDEX | v1.40 | v1.41 |
| VP-153 | v0.4 | v0.5 |
| VP-154 | v0.5 | v0.6 |
| VP-155 | v0.2 | v0.3 |
| VP-156 | v0.3 | v0.4 |
| STATE + HANDOFF | v7.283 | v7.284 |

## Next Step

Adversary pass-6 dispatch (fresh-context). BC-5.39.001 3-CLEAN protocol — streak 0/3.
Pass-5 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-5.md`
