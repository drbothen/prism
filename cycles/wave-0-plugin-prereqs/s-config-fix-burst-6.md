---
document_type: fix-burst-closure
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-05-24T00:00:00Z
cycle: "wave-0-plugin-prereqs"
story: "S-CONFIG-MULTI-TENANT-OVERRIDE-001"
fix_burst: 6
closes_pass: 5
traces_to: convergence-trajectory.md
---

# S-CONFIG-MULTI-TENANT-OVERRIDE-001 — Fix-Burst-6 Closure Record

**Date:** 2026-05-24
**Closes:** Pass-5 findings (F-LP5-MED-001 + F-LP5-LOW-001 + F-LP5-LOW-002)
**Also performs:** OBS-LP5-001 cycle narrative correction
**State burst:** D-814 (TD-VSDD-053 single-commit)
**Feature HEAD before:** `5c11fc7b`
**Feature HEAD after:** `3416eea6`

---

## Findings Closed

### F-LP5-MED-001 — BC-2.06.016 E-SPEC-020 placeholder drift

**Closed by:** PO dispatch commit `513ee6b8`
**BC bump:** BC-2.06.016 v1.1 → v1.2
**Fix per BC-2.06.016 v1.2 changelog (byte-quoted):**
> "Fix E-SPEC-020 Message template placeholder drift: `{sensor_id}@{org_slug}` → `{expected}` in line 108 to match canonical error-taxonomy.md line 392 authority text. Same drift class as F-LP4-MED-002 (BC-2.06.013 E-SPEC-023 `{field}` → `{field_name}`); F-LP5-MED-001 was a sibling-sweep gap in fix-burst 5 — burst swept BC-2.06.013 and BC-2.06.015 but missed BC-2.06.016 line 108 for the E-SPEC-020 row (POL-25 sibling-sweep gap). Scope decision: line 109 Suggestion field also uses `{sensor_id}@{org_slug}` but is deferred to architect adjudication under F-LP5-LOW-002 (whether Suggestion is BC-authoritative or taxonomy-derived); left untouched this burst. POL-29 sweep result: 9 matches of `{sensor_id}@{org_slug}` found across .factory/; all non-target matches are LEGITIMATE LITERAL USE (convention description in table cells, log span field doc, story body documentation, taxonomy changelog narrative) — no additional sibling drifts. 4-way alignment after fix: BC-2.06.016 line 108 ↔ taxonomy line 392 MATCH."

**BC-INDEX bump:** v5.49 → v5.50

---

### F-LP5-LOW-001 — overlay.rs doc-comment forward-pointer corrections

**Closed by:** Implementer commit `3416eea6`
**Feature HEAD:** `5c11fc7b` → `3416eea6`
**Fix:** 3 doc-comment sites in overlay.rs corrected (byte-sourced from 3416eea6 commit message and overlay.rs; TD-VSDD-091 function-name anchors):
1. `make_e_spec_019_unknown_extends` — doc-comment replaced paraphrased E-SPEC-019 template with forward-pointer: "Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md` row E-SPEC-019. The `format!` body below produces the exact emission text."
2. `make_e_spec_020_instance_id_mismatch` — doc-comment replaced paraphrased E-SPEC-020 template (separator + semantic drift: `instance_id '{actual}' does not match expected '{expected}' ({sensor_id}@{org_slug})` vs canonical `declares instance_id='{actual}' but expected '{expected}' (derived from filename and parent directory)`) with forward-pointer.
3. `make_e_spec_021_tables_in_overlay` — doc-comment replaced paraphrased E-SPEC-021 template (omission drift: missing "Table schema must be declared in the TYPE spec only." sentence) with forward-pointer.

**Not included in fix-burst-6 (sibling-sweep gap):** `e_spec_022_unknown_org_slug` and `make_e_spec_023_unrecognized_field` retained paraphrased docs; these 2 remaining sites closed by fix-burst-7 implementer commit `d600f7f4` (F-LP6-LOW-001).

**Root cause:** initial doc-comment authoring cited function names without reading actual function bodies; forward-pointers drifted from the function they named. Closure record originally misstated the 3 fixed sites as `make_e_spec_019_instance_id_mismatch` + `make_e_spec_021_tables_in_overlay` + `make_e_spec_022_unknown_org_slug` — the first and third names do not exist in overlay.rs (F-LP6-MED-001; corrected by D-815 burst).

---

### F-LP5-LOW-002 — Suggestion field source-of-truth adjudication (Option B)

**Closed by:** Architect commit `4ef6c650` via S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001 v0.1 → v0.2
**Option adjudicated:** B — BC-2.06.016 `Suggestion` rows are canonical for operator-facing remediation guidance; taxonomy description-prose sub-clauses are informative-only and do not constitute a competing `Suggestion` authority.
**Rationale per §Source of Truth Precedence in S-POL-29 story:**
> "BC-2.06.016 has a first-class, structured `Suggestion` row per error code containing fuller, operator-facing remediation detail. These are not competing representations of the same field — they are different fields in different schemas. CLAUDE.md Rule #3 (PRD supplements supersede PRD prose 'for the same surface area') applies to message_template, severity, category, exit code, and retryable flag — fields the taxonomy explicitly columns out. It does NOT extend to Suggestion text, which has no counterpart column in the taxonomy."
**Story bump:** S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001 v0.1 → v0.2; AC-006 added to enumerate `suggestion_authority` field in POL-29 class (d) registry schema.
**STORY-INDEX bump:** v2.187 → v2.188

---

## OBS-LP5-001 Narrative Correction

**Performed by:** State-manager (this burst, D-814)
**Root cause per lessons entry 42:** State-manager authored pass-4 report narratives from memory/summary rather than reading BC changelogs for line-level fix descriptions.

### Files corrected

| File | Lines | Old narrative (paraphrase) | Corrected narrative (byte-sourced from BC changelog) |
|------|-------|---------------------------|------------------------------------------------------|
| `s-config-pass-4.md` | lines 27-28 | F-LP4-MED-001: "separator drift (colon instead of em-dash) from the E-SPEC-020 canonical template" | F-LP4-MED-001: "E-SPEC-021 message §Postconditions — semicolon-separated paraphrase vs canonical period-separated form per BC-2.06.013 v1.1 changelog" |
| `s-config-pass-4.md` | lines 27-28 | F-LP4-MED-002: "E-SPEC-020 `{overlay_path}` vs canonical `{file}`" | F-LP4-MED-002: "E-SPEC-023 §Error Cases placeholder `{field}` vs canonical `{field_name}` per BC-2.06.013 v1.1 changelog" |
| `convergence-trajectory.md` | pass-4 row | "F-LP4-MED-001/002 BC-2.06.013 canonical template separator/placeholder drift" | "F-LP4-MED-001 BC-2.06.013 E-SPEC-021 paraphrase: semicolon-separated vs canonical period-separated; F-LP4-MED-002 BC-2.06.013 E-SPEC-023 placeholder `{field}` vs canonical `{field_name}`" |
| `S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001-*.md` | §Originating Findings table F-LP4-MED-001/002 rows | "separator drift (colon vs em-dash) for E-SPEC-020" / "placeholder name drift (`{overlay_path}` vs `{file}`)" | "E-SPEC-021 paraphrase drift: semicolon vs period per BC-2.06.013 v1.1 changelog" / "E-SPEC-023 placeholder `{field}` vs `{field_name}` per BC-2.06.013 v1.1 changelog" |

**Source of truth used:** BC-2.06.013 v1.1 changelog line 200 (byte-quoted in s-config-pass-5.md Part B OBS-LP5-001 section).

---

## Streak Status

- **Before fix-burst-6:** 0/3
- **After fix-burst-6:** 0/3 (fix-burst does not advance streak per BC-5.39.001)
- **Next:** pass-6 adversary dispatch (first streak attempt 0/3 → 1/3)

## Version Bumps Summary

| Artifact | Before | After |
|----------|--------|-------|
| BC-2.06.016 | v1.1 | v1.2 |
| BC-INDEX | v5.49 | v5.50 |
| S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001 | v0.1 | v0.2 |
| STORY-INDEX | v2.187 | v2.188 |
| STATE.md | v7.500 | v7.501 |
