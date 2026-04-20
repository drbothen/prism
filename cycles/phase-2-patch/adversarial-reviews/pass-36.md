---
document_type: adversarial-review
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
cycle: phase-2-patch
pass: 36
previous_review: pass-35.md
status: findings-open
novelty: MEDIUM — fresh-context scan found 1 missed-line in Burst 36 C-002 sweep + 1 Burst 36 arithmetic regression + 1 inventory labeling issue + 1 observational
findings_total: 4
findings_crit: 0
findings_high: 2
findings_med: 1
findings_low: 1
findings_observational: 0
previous_pass: 35
convergence_counter: 0
date: 2026-04-19
---

# Adversarial Review: Prism (Pass 36)

## Finding ID Convention

`P3P36-A-{SEV}-NNN` where SEV is CRIT / HIGH / MED / LOW.

## Part A — Methodology

### Dimensions Scanned (11)

1. Semantic anchoring integrity (Policy 4) — BC-ID / error-code / tool-name alignment across story bodies
2. Changelog discipline (Policy 2) — version bumps, changelog completeness, inventory currency
3. Arithmetic consistency (Policy 6 adjacent) — count claims in Mermaid labels, frontmatter totals
4. Policy 8 bidirectional AC-to-BC trace — acceptance-criteria ↔ BC-INDEX cross-reference integrity
5. BC-INDEX ↔ story-body title sync — canonical H1 titles propagated to story BC tables
6. Error taxonomy propagation — new error codes from Burst 36 reflected in story bodies + BCs
7. Capability enumeration completeness — CAP-NNN tool lists vs api-surface.md + story ACs
8. SS-anchor correctness — SS-ID assignments match architecture subsystem names
9. VP-INDEX ↔ architecture traceability — VP references in stories trace to declared VPs
10. Test-vector consistency — test-vectors.md scenario error codes / tool names match canonical sources
11. Cross-document version pin integrity — supplement pinned versions match current file versions

### Policies Applied

- policies.yaml rubric (full 9-policy set)
- Semantic anchoring integrity policy elevated all near-miss error-code fits to explicit review

### Corpus

| Artifact | Version | Lines |
|----------|---------|-------|
| BC-INDEX | v4.10 | 203 BCs (195 active + 6 dual-anchor + 2 removed) |
| STORY-INDEX | v1.25 | 75 stories |
| ARCH-INDEX | current | — |
| capabilities.md | v1.2 | — |
| api-surface.md | v1.2 | — |
| error-taxonomy.md | v1.2 | — |
| VP-INDEX | v1.3 | 39 VPs (20+11+6+2) |
| test-vectors.md | v2.2 | — |

---

## Part B — New Findings

### P3P36-A-HIGH-001 — S-5.06:199 residual error-code drift

**Severity:** HIGH
**Policy:** 4 (semantic_anchoring_integrity)
**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-5.06-action-infusion-tools.md` line 199

**Description:**

Line 199 cites `E-ACTION-003` with "Action '{id}' not found" semantics for `delete_action` on unknown action_id. But error-taxonomy.md:364 defines E-ACTION-003 as:

> "Action '{action_id}' delivery to '{destination}' failed after 5 attempts. Dead-letter record written…" (category: retry_exhaustion / dead-letter)

The correct code for "action_id not registered" is E-ACTION-006 per error-taxonomy.md:367:

> "Action '{action_id}' is not registered." (category: not_found)

Burst 36's C-002 sweep (lines 121/304/329/495) missed this line.

**Fix:** S-5.06:199 — `E-ACTION-003` → `E-ACTION-006`.

---

### P3P36-A-HIGH-002 — api-surface.md Mermaid "(22 Write Tools)" is arithmetically wrong; actual count is 24

**Severity:** HIGH
**Policy:** Arithmetic / Policy 6 adjacent
**Location:** `/Users/jmagady/Dev/prism/.factory/specs/architecture/api-surface.md` line 51 + v1.2 changelog entry line 338

**Description:**

Burst 36 changelog claims Mermaid labels updated to "Always-Visible 24→28, Capability-Gated 20→22". The read-tool count of 28 reconciles. The write-tool count does NOT.

Lines 146–169 enumerate 24 capability-gated write tools:

1. configure_credential_source
2. delete_credential
3. crowdstrike_contain_host
4. crowdstrike_lift_containment
5. acknowledge_alert
6. create_case
7. update_case
8. create_alias
9. delete_alias
10. create_schedule
11. delete_schedule
12. create_rule
13. delete_rule
14. create_pack
15. delete_pack
16. confirm_action
17. reload_config
18. add_sensor_spec
19. fire_action
20. test_action
21. reload_infusion
22. reload_plugin
23. create_action
24. delete_action

Pre-Burst-35 count was 20. Burst 35 added 4 tools → correct total is 24, not 22.

**Fix:** Line 51 `(22 Write Tools)` → `(24 Write Tools)`; update v1.2 changelog entry to reflect `20→24` (not `20→22`).

---

### P3P36-A-MED-001 — test-vectors.md v2.2 has no Burst 36 changelog entry despite being listed in recent-versions inventory

**Severity:** MED
**Policy:** 2 (Changelog Discipline)
**Location:** `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` frontmatter + changelog

**Description:**

test-vectors.md v2.2 changelog (line 322) attributes the v2.2 bump to Burst 34 (pass-33 M-001 — 5 stale execute_action refs reconciled), not Burst 36. Two interpretations:

(a) File was NOT touched in Burst 36 — orchestrator inventory (which listed it in "recent versions") is stale.
(b) File WAS touched in Burst 36 without a version bump or changelog entry.

Verify via `git log .factory/specs/prd-supplements/test-vectors.md`.

**Fix:** If not touched in Burst 36, remove from recent-versions inventory (no spec edit needed). If touched without bump, bump to v2.3 + add changelog entry. MED-001 is tentatively an inventory labeling issue — no spec drift.

---

### P3P36-A-LOW-001 — S-1.15:365 cites E-PLUGIN-003 for KV-store size limit; semantically near-miss after Burst 35/36 narrowing

**Severity:** LOW
**Policy:** 4 (Observational)
**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.15-wasm-runtime.md` line 365

**Description:**

E-PLUGIN-003 template ("exceeded resource limit: {resource} ({limit})") can accommodate KV storage via `resource="kv_store"` but error-taxonomy.md does not explicitly enumerate KV storage as a named resource value. This is an implicit-fit rather than a true drift — the template is general-purpose and KV is a valid instantiation.

This was not introduced by Burst 35 or 36; it pre-dates the error-taxonomy expansion. Post-expansion, the taxonomy is now precise enough that explicit enumeration would be cleaner.

**Fix options:**
- (a) Add parenthetical clarification in S-1.15:365 that `{resource}` templates to `"kv_store"` — lower friction.
- (b) Add dedicated E-PLUGIN-012 row for KV size limit — more precise.

Option (a) is preferred given current convergence pressure.

---

## Summary

| Severity | Count | Finding IDs |
|----------|-------|-------------|
| CRIT | 0 | — |
| HIGH | 2 | P3P36-A-HIGH-001, P3P36-A-HIGH-002 |
| MED | 1 | P3P36-A-MED-001 |
| LOW | 1 | P3P36-A-LOW-001 |
| **Total** | **4** | |

**Verdict: NOT CLEAN.** 4 findings open. Convergence counter stays at 0/3.

Burst 37 must close:
- P3P36-A-HIGH-001 (S-5.06:199 E-ACTION-003 → E-ACTION-006)
- P3P36-A-HIGH-002 (api-surface.md line 51 + changelog: 22→24 write tools)
- P3P36-A-MED-001 (verify test-vectors.md Burst 36 touch via git log; correct inventory if not touched)
- P3P36-A-LOW-001 (S-1.15:365 parenthetical clarification — low priority, batch with HIGH fixes)

### Sweeps Run Clean

- BC-INDEX arithmetic (195+6+2=203) ✓
- STORY-INDEX 75 stories ✓
- VP-INDEX (20+11+6+2=39) ✓
- BC-2.17.005 v1.1 lines 54/70 E-PLUGIN-011 ✓
- error-taxonomy.md v1.2 all 5 Burst 36 additions present ✓
- capabilities.md CAP-031/032/033 enumerations ✓
- CAP-032 prose E-PLUGIN-006/007 ✓
- api-surface.md SS anchoring lines 133-138 all correct (SS-17/18/19) ✓
- S-5.06 frontmatter BC array + body BC table + AC traces bidirectional ✓
- S-1.14/1.15/4.08 Policy 8 AC-to-BC-ID sweep ✓
