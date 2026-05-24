---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-24T00:00:00Z
cycle: "wave-0-plugin-prereqs"
story: "S-CONFIG-MULTI-TENANT-OVERRIDE-001"
pass: 5
traces_to: convergence-trajectory.md
---

# S-CONFIG-MULTI-TENANT-OVERRIDE-001 — LOCAL Adversary Pass-5

**Pass Date:** 2026-05-24
**Feature HEAD:** `5c11fc7b` (unchanged — pass-5 is read-only; implementer feature commits pending for fix-burst-6)
**Streak Before Pass:** 0/3
**Result:** 3 findings (1 MED + 1 LOW + 1 LOW) — OBS-LP5-001 also surfaced as an observation

---

## Part A — Fix-Burst-5 Closure Verification

All 4 pass-4 findings (F-LP4-MED-001 through F-LP4-MED-004) verified CLOSED:

- **F-LP4-MED-001 DURABLE:** BC-2.06.013 §Postconditions E-SPEC-021 message — canonical period-separated form confirmed at line 73; no semicolon-separated paraphrase remains.
- **F-LP4-MED-002 DURABLE:** BC-2.06.013 §Error Cases E-SPEC-023 message — `{field_name}` placeholder confirmed at line 82; `{field}` paraphrase form removed.
- **F-LP4-MED-003 DURABLE:** BC-2.06.015 §Postconditions E-SPEC-022 — canonical form "Check for typos or register the org in prism.toml [[orgs]]." confirmed at line 69; paraphrase removed.
- **F-LP4-MED-004 DURABLE:** S-CONFIG story body E-SPEC-020 — full canonical template with `overlay_fields` enumeration confirmed in task descriptions; shortened form removed.

---

## Part B — New Findings

### Finding Summary

| ID | Severity | Category | Description |
|----|----------|----------|-------------|
| F-LP5-MED-001 | MEDIUM | [process-gap] sibling-sweep | BC-2.06.016 line 108 E-SPEC-020 row `expected_instance_id` message template — paraphrase placeholder `{sensor_id}@{org_slug}` vs canonical `{expected}` per error-taxonomy.md line 392 |
| F-LP5-LOW-001 | LOW | doc-correctness | overlay.rs doc-comment forward-pointers cite wrong function names (3 sites): `make_e_spec_019_instance_id_mismatch`, `make_e_spec_021_tables_in_overlay`, `make_e_spec_022_unknown_org_slug` forward-pointer descriptions drift from actual function semantics |
| F-LP5-LOW-002 | LOW | [pending intent verification] | BC-2.06.016 Suggestion field content vs taxonomy description-prose sub-clauses — adversary surfaced as ambiguity: taxonomy description embeds suggestion-like guidance as prose sub-clauses inside the `description` column; BC-2.06.016 has first-class `Suggestion` rows per error code; source-of-truth authority unclear if they diverge |

---

## OBS-LP5-001 — Cycle Artifact Narrative Drift

**Observation:** s-config-pass-4.md §Finding Summary described F-LP4-MED-001 as "separator drift (colon instead of em-dash) from the E-SPEC-020 canonical template" and F-LP4-MED-002 as "E-SPEC-020 `{overlay_path}` vs canonical `{file}`." BC-2.06.013 v1.1 changelog (the authoritative fix record) states:

- F-LP4-MED-001: E-SPEC-021 message at line 73 — semicolon-separated paraphrase vs period-separated canonical
- F-LP4-MED-002: E-SPEC-023 message at line 82 — `{field}` placeholder vs canonical `{field_name}`

The error codes (E-SPEC-021, E-SPEC-023) and the separator descriptions (semicolon→period, not colon→em-dash) were both incorrect in the pass-4 report. Same drift appeared in convergence-trajectory.md pass-4 row and S-POL-29 §Originating Findings table.

**Source-of-Truth Precedence Rule #1** (BC supersedes report when conflict is about contract semantics): the BC changelogs are authoritative. Route to state-manager for narrative correction in same fix-burst-6 commit.

**Codification:** State-manager must byte-quote from BC changelog entries when authoring pass report finding descriptions.

---

## Verdict

**CLEAN (strict):** no
**CLEAN (PR-merge):** no
**Streak:** 0/3 → 0/3 (BLOCKED)
**Findings:** 1 MED + 2 LOW = 3 total
**Root cause (F-LP5-MED-001):** Fix-burst-5 swept BC-2.06.013 and BC-2.06.015 for E-SPEC-020/021/022/023 canonical template restoration but missed BC-2.06.016 line 108 — BC-2.06.016 owns the E-SPEC-020 message_template in its own §Error Cases table (sibling-sweep gap: 3-BC sweep not 1-BC sweep required).

## Fix-burst Routing

Fix-burst 6 dispatched:
- **PO** (513ee6b8): BC-2.06.016 v1.1→v1.2 — E-SPEC-020 message at line 108: `{sensor_id}@{org_slug}` → `{expected}` to match canonical error-taxonomy.md line 392 (F-LP5-MED-001 closed). Scope: line 109 Suggestion field `{sensor_id}@{org_slug}` left untouched pending F-LP5-LOW-002 architect adjudication.
- **Implementer** (3416eea6): overlay.rs doc-comment forward-pointer corrections at 3 sites (F-LP5-LOW-001 closed).
- **Architect** (4ef6c650): S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001 v0.1→v0.2 — AC-006 added adjudicating Suggestion field source-of-truth: BC-2.06.016 `Suggestion` rows canonical; taxonomy description-prose informative-only (F-LP5-LOW-002 closed Option B).
- **State-manager (this burst):** OBS-LP5-001 narrative correction across s-config-pass-4.md + convergence-trajectory.md + S-POL-29 §Originating Findings + lessons.md entry 42.
