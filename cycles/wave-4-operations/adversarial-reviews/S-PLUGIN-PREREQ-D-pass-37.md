---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_number: 37
burst_label: "Burst W-reify"
verdict: BLOCKED
streak_before: 0/3
streak_after: 0/3
finding_summary: "0 CRIT / 0 HIGH / 1 MED / 0 LOW / 1 OBS"
findings_total: 1
combined_burst_note: "pass-37 reify + fix-burst-35 COMBINED in single commit per TD-VSDD-053 state-manager-domain consolidation (D-538)"
timestamp: 2026-05-14T00:00:00Z
producer: state-manager
---

# S-PLUGIN-PREREQ-D Adversarial Pass-37 Report

> **VERDICT: BLOCKED** — 0 CRIT / 0 HIGH / 1 MED / 0 LOW / 1 OBS
>
> **Combined-burst note:** This pass-37 report and fix-burst-35 closure report are co-committed in a single
> D-538 commit per TD-VSDD-053 single-commit discipline. The finding (VP-INDEX:190 mis-anchor) is
> state-manager-domain (VP-INDEX is state-manager responsibility per CLAUDE.md routing table), so
> consolidating reify + fix + closure into one atomic commit is context-budget-conserving and
> operationally correct.

---

## Verification Trail: Fix-Burst-34 Closures HELD

Before recording pass-37 findings, the adversary verified that fix-burst-34 (D-537) closures all held:

| Closure | Evidence | Status |
|---------|----------|--------|
| BC-2.17.007 frontmatter `modified: 2026-05-14` | Line 14 of BC-2.17.007 — `modified: 2026-05-14` (synced to fix-burst-33 edit date) | **HELD** |
| BC-2.17.007 frontmatter `timestamp: 2026-05-14T00:00:00Z` | Line 7 of BC-2.17.007 — `timestamp: 2026-05-14T00:00:00Z` | **HELD** |
| BC-2.17.007:138 AC-7→AC-5 anchor | Line 138 now reads "per AC-5 manifest gate; default-deny consumer is AC-7" | **HELD** |
| BC-2.17.007:161 sibling-catch AC-7→AC-5 anchor | Line 161 (VP Anchors section) now reads "per AC-5 manifest gate; default-deny consumer is AC-7" | **HELD** |
| BC-2.17.007 v1.4 changelog row | §Changelog row for v1.4 present and correctly dated 2026-05-14 | **HELD** |
| BC-INDEX v4.75 with v1.4 row | BC-INDEX BC-2.17.007 row updated to v1.4 annotation | **HELD** |

All fix-burst-34 closures hold. No regression introduced.

---

## Findings

### F-LP37-MED-001 — VP-INDEX:190 carries same `per AC-7 default-deny` mis-anchor corrected in BC-2.17.007:138/161 (sibling-document propagation gap)

**Severity:** MED
**Policy:** Codification #11 (lexical-vs-semantic anchor) + POL-25-candidate (multi-cite propagation sweep) + TD-VSDD-060 (sibling-site sweep)
**Location:** `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/VP-INDEX.md:190`

**Current text (VP-PLUGIN-007 named-alias row):**
```
| VP-PLUGIN-007 | VP-152 | Plugin manifest allowlist explicit Vec<String> after PREREQ-D: manifest without allowed_urls field rejected at load time per AC-7 default-deny; allowed_urls=[] blocks all HTTP; non-empty list enforces host-only allowlist | prism-spec-engine | integration_test | P0 | draft | PLUGIN-PREREQ-D |
```

**Defect:** The description phrase "rejected at load time **per AC-7 default-deny**" anchors the load-time rejection rationale to AC-7 (the downstream `host_http_request` allowlist enforcement gate). The correct anchor is AC-5 (manifest schema validation gate, which produces E-PLUGIN-013 on manifest-without-allowed_urls-field). Fix-burst-34 (D-537) correctly restored the canonical AC-5 anchor in BC-2.17.007 at lines 138 and 161. VP-INDEX:190 is the 4th-cascade propagation gap carrying the same mis-anchor string class:

| Burst | Location Fixed | Pattern |
|-------|---------------|---------|
| fix-burst-34 (D-537) | BC-2.17.007:138 | `per AC-7 default-deny` → `per AC-5 manifest gate; default-deny consumer is AC-7` |
| fix-burst-34 (D-537) | BC-2.17.007:161 (sibling-catch) | same |
| pass-37 (this pass) | VP-INDEX:190 | same mis-anchor — NOT caught by fix-burst-34 multi-cite sweep |

**Root cause:** Fix-burst-34 ran the TD-VSDD-060 sibling-sweep within BC-2.17.007 (correct) and noted zero body hits after fix. However, the sweep did not extend to VP-INDEX rows referencing the same VP-PLUGIN-007 property. The `grep -rn 'per AC-7 default-deny' .factory/specs/` would have caught VP-INDEX:190, but the fix-burst-34 sweep was scoped to BC-2.17.007 body only. This is the POL-25-candidate gap: when a BC's VP-row description is edited, the SAME grep MUST be executed against VP-INDEX in the same burst.

**Canonical form established by fix-burst-34:**
`rejected at load time per AC-5 manifest gate (default-deny consumer is AC-7)`

**Routing:** State-manager (VP-INDEX is state-manager domain per CLAUDE.md routing table; all prior VP-INDEX version bumps in this cascade handled by state-manager — v1.33→v1.35 chain). In-perimeter per story §References:1034 citing VP-PLUGIN-007.

---

### OBS-LP37-001 — [process-gap] POL-25-candidate strengthens to HIGH-priority codification candidate (4-burst recurrence on same anchor-string class)

**Severity:** OBS (process-gap)
**Routing:** Cycle-close session-reviewer adjudication

**Pattern documentation:** Bursts 32→33→34→37 each caught incremental sibling-sites of the same canonical-anchor mis-formulation class:
- fix-burst-30 (D-528): BC-2.17.002 EC-17-007 anchor fix (Path A)
- fix-burst-32 (D-533): VP-INDEX VP-152+VP-PLUGIN-007 "not-None" Option-semantics fix
- fix-burst-34 (D-537): BC-2.17.007:138+161 "per AC-7 default-deny" → "per AC-5 manifest gate" fix
- pass-37 (this pass): VP-INDEX:190 same "per AC-7 default-deny" mis-anchor — 4th cascade propagation gap

**Proposed codification (POL-25):** When a BC's VP-row description is edited (any BC that has a VP-INDEX named-alias row citing it), the SAME grep pattern used to find the BC body sites MUST be executed against VP-INDEX rows in the same burst. Also: extend Codification #11's sweep target list to explicitly enumerate VP-INDEX rows referencing edited BCs as mandatory sweep targets.

**Strengthening rationale:** Two prior sessions raised OBS items about multi-cite propagation sweep gaps. OBS-LP35-002 established POL-25 as candidate #22. OBS-LP37-001 strengthens this candidate to HIGH-priority — 4 burst recurrences of the SAME anchor-string class is evidence that the current sweep protocol has a structural gap at the BC-body ↔ VP-INDEX boundary.

---

## Convergence Trajectory Note

- Trajectory pass-25..37: 4→1→4→5→1→1→3→4→5→5→5→2→1
- The 5→2→1 pattern (passes 35→36→37) shows continued convergence-favorable decrease.
- Pass-37 finding was a sibling-cascade closure of an already-remediated class, NOT a novel finding class.
- Pass-38 has a realistic CLEAN chance — POL-25 sweep gap is the only remaining open vector in this anchor-string class.

---

## Combined-Burst Variant Note

This pass-37 report was co-created with fix-burst-35 closure in a single D-538 commit per TD-VSDD-053. The single-state-manager-domain nature of the finding (VP-INDEX:190 is state-manager's sole responsibility per CLAUDE.md routing; no product-owner or story-writer involvement required) makes the combined-burst variant the operationally correct choice for context-budget conservation. All 4 cascade recurrences of this anchor-string class are now closed:

| Burst | Artifact | Status |
|-------|----------|--------|
| fix-burst-34 (D-537) | BC-2.17.007:138 | CLOSED |
| fix-burst-34 (D-537) | BC-2.17.007:161 | CLOSED (sibling-catch) |
| fix-burst-35 (D-538) | VP-INDEX:190 | CLOSED (this burst) |

OBS-LP37-001 codification candidate POL-25 strengthened HIGH-priority — dispatched to cycle-close session-reviewer queue.
