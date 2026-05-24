---
document_type: fix-burst-closure
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-05-24T00:00:00Z
cycle: "wave-0-plugin-prereqs"
story: "S-CONFIG-MULTI-TENANT-OVERRIDE-001"
fix_burst: 7
closes_pass: 6
traces_to: convergence-trajectory.md
---

# S-CONFIG-MULTI-TENANT-OVERRIDE-001 — Fix-Burst-7 Closure Record

**Date:** 2026-05-24
**Closes:** Pass-6 findings (F-LP6-MED-001 + F-LP6-MED-002 + F-LP6-LOW-001 + F-LP6-LOW-002)
**State burst:** D-815 (TD-VSDD-053 single-commit)
**Feature HEAD before:** `3416eea6`
**Feature HEAD after:** `d600f7f4`

---

## Findings Closed

### F-LP6-LOW-002 — BC-2.06.016 EC-016-003 cross-file aggregation + EC-016-005 within-file structural-suppresses-semantic

**Closed by:** PO commit `455f9fbb`
**BC bump:** BC-2.06.016 v1.2 → v1.3
**Fix per BC-2.06.016 v1.3 changelog (byte-quoted):**
> "Clarify EC-016-003 cross-file aggregation scope; add EC-016-005 within-file structural-suppresses-semantic boundary; expand INV-ERR-003. F-LP6-LOW-002 identified ambiguity: EC-016-003 'all five error codes fire in the same boot' was silent on whether codes could originate from the same file or required multiple files. Code analysis of `validate_overlay_toml` structural-check early-return (lines 524-527, `prism-spec-engine/src/overlay.rs`) confirmed: structural errors (E-SPEC-021/E-SPEC-023) cause an early-return before deserialization, making E-SPEC-019/E-SPEC-020 unreachable for the same file. Early-return is correct-by-design (semantic checks require successful deserialization). Fix: EC-016-003 amended to specify 'each from a DIFFERENT overlay file or directory' and explain which codes are structural vs semantic. New EC-016-005 documents the within-file suppression case explicitly with operator guidance ('fix structural error first, re-run to surface semantic errors'). INV-ERR-003 expanded to describe cross-file aggregation vs within-file suppression. Option A chosen per fix decision: documentation alignment with sound code design; no impl change. POL-29 sibling-sweep: (1) EC-016-003 references in other .factory/ files all reference BC-2.01.016's EC-016-003 (a distinct BC); no conflicting narrative to update. (2) 'All five error codes' phrase appears only in this BC. (3) No prior 'structural-suppresses-semantic' documentation found; EC-016-005 is the canonical first definition."

**BC-INDEX bump:** v5.50 → v5.51

---

### F-LP6-LOW-001 — overlay.rs sibling doc-comment forward-pointer completion

**Closed by:** Implementer commit `d600f7f4`
**Feature HEAD:** `3416eea6` → `d600f7f4`
**Fix:** 2 remaining doc-comment sites in overlay.rs corrected (TD-VSDD-091 function-name anchors; byte-quoted from d600f7f4 commit message):
1. `e_spec_022_unknown_org_slug` (impl block method, `prism-spec-engine/src/overlay.rs`) — before: "Message template (BC-2.06.016 canonical form, INV-ERR-002): ..." (embedded template paraphrase); after: "Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md` row E-SPEC-022. The `format!` body below produces the exact emission text."
2. `make_e_spec_023_unrecognized_field` (free function, `prism-spec-engine/src/overlay.rs`) — before: "Canonical message template (BC-2.06.016 §Error Catalog line 148): ..." plus note about trailing suffix + POL-24 + F-LP2-MED-001 parenthetical (multiple paraphrase drift sources); after: "Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md` row E-SPEC-023. The `format!` body below produces the exact emission text."

**Sibling-sweep verification (per d600f7f4 commit message):**
- `grep "Canonical message template per" overlay.rs` → 5 occurrences (1 per builder): `make_e_spec_019_unknown_extends`, `make_e_spec_020_instance_id_mismatch`, `make_e_spec_021_tables_in_overlay`, `e_spec_022_unknown_org_slug`, `make_e_spec_023_unrecognized_field`
- `grep "BC-2.06.016 §" overlay.rs` → 0 occurrences
- `grep "(BC-2.06.016 canonical form" overlay.rs` → 0 occurrences

**Not a runtime change:** AC-005 byte-compare test (`test_BC_2_06_016_error_messages_match_canonical_templates`) unaffected — doc-comment changes do not alter runtime emission text.

---

### F-LP6-MED-001 — s-config-fix-burst-6.md F-LP5-LOW-001 closure section narrative correction

**Closed by:** State-manager D-815 burst (this record)
**Fix:** s-config-fix-burst-6.md F-LP5-LOW-001 section rewritten with byte-quoted function names from `prism-spec-engine/src/overlay.rs` source (TD-VSDD-091 anti-volatile-pin discipline):
- Old (WRONG): `make_e_spec_019_instance_id_mismatch` — this function does NOT exist in overlay.rs
- Old (MISSING): `make_e_spec_020_instance_id_mismatch` — omitted from original record
- Old (WRONG): `make_e_spec_022_unknown_org_slug` listed as 3rd fixed site — this was NOT fixed by fix-burst-6 (it is the method that remained un-fixed until fix-burst-7)
- Corrected sites (byte-quoted from overlay.rs and 3416eea6 commit message):
  1. `make_e_spec_019_unknown_extends` — free function (overlay.rs, `make_e_spec_019_unknown_extends` anchor per TD-VSDD-091)
  2. `make_e_spec_020_instance_id_mismatch` — free function (overlay.rs, `make_e_spec_020_instance_id_mismatch` anchor per TD-VSDD-091)
  3. `make_e_spec_021_tables_in_overlay` — free function (overlay.rs, `make_e_spec_021_tables_in_overlay` anchor per TD-VSDD-091)
- Now also documents the 2 sites NOT in fix-burst-6 and their fix-burst-7 closure

**Root cause:** D-814 state-manager burst authored from memory/summary rather than reading overlay.rs before writing function-name citations. Same failure mode as OBS-LP5-001 (lesson 42), occurring inside the burst that codified lesson 42.

---

### F-LP6-MED-002 — lessons.md entry 41 bullets (1)+(2) narrative correction

**Closed by:** State-manager D-815 burst (this record)
**Fix:** lessons.md entry 41 bullets (1) and (2) rewritten with byte-quoted text from BC-2.06.013 v1.1 changelog (the authoritative fix record):

**Bullet (1) — before (WRONG paraphrase):**
> "Separator drift — BC-2.06.013 §Postconditions used colon where canonical template uses em-dash (F-LP4-MED-001)"

**Bullet (1) — after (byte-quoted from BC-2.06.013 v1.1 changelog):**
> "Separator/form drift — BC-2.06.013 §Postconditions E-SPEC-021 message at line 73 used semicolon-separated paraphrase ('Remove [[tables]] and declare schema in the TYPE spec only') vs canonical period-separated form ('Table schema must be declared in the TYPE spec only') — per BC-2.06.013 v1.1 changelog: 'F-LP4-MED-001: E-SPEC-021 message at line 73 — replaced paraphrase (semicolon-separated, "Remove [[tables]] and declare schema in the TYPE spec only") with canonical (period-separated, "Table schema must be declared in the TYPE spec only")'"

**Bullet (2) — before (WRONG paraphrase):**
> "Placeholder name drift — BC-2.06.013 §Error Cases used `{overlay_path}` vs canonical `{file}` (F-LP4-MED-002)"

**Bullet (2) — after (byte-quoted from BC-2.06.013 v1.1 changelog):**
> "Placeholder name drift — BC-2.06.013 §Error Cases E-SPEC-023 message at line 82 used `{field}` placeholder, lowercase 'allowed fields are:', no sub-fields clause vs canonical `{field_name}` placeholder, 'Allowed overlay fields are:', '(with sub-fields: requests_per_second, burst_size)' appended — per BC-2.06.013 v1.1 changelog: 'F-LP4-MED-002: E-SPEC-023 message at line 82 — replaced paraphrase (`{field}` placeholder, lowercase "allowed fields are:", no sub-fields clause) with canonical (`{field_name}` placeholder, "Allowed overlay fields are:", "(with sub-fields: requests_per_second, burst_size)" appended)'"

**Meta-correction header removed:** Entry 41 header previously had "[CODIFIED — NOTE: original entry 41 text contained OBS-LP5-001 paraphrase drift...]" prefix. This meta-correction is now folded into the bullet corrections; header reverts to clean "[process-gap] [CODIFIED]" without a self-referential note.

**Source of truth used:** BC-2.06.013 v1.1 changelog row (byte-quoted above; read from `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.06.013-scalar-only-overlay-enforcement.md` §Changelog).

---

## Streak Status

- **Before fix-burst-7:** 0/3
- **After fix-burst-7:** 0/3 (fix-burst does not advance streak per BC-5.39.001)
- **Next:** pass-7 adversary dispatch (streak attempt 0/3→1/3)

## Version Bumps Summary

| Artifact | Before | After |
|----------|--------|-------|
| BC-2.06.016 | v1.2 | v1.3 |
| BC-INDEX | v5.50 | v5.51 |
| STATE.md | v7.501 | v7.502 |

## Self-Check Verification (lesson 42 compliance — byte-quote discipline)

Per lesson 42 mandatory pre-commit check:

1. `rg "make_e_spec_019_instance_id_mismatch|make_e_spec_022_unknown_org_slug" .factory/cycles/wave-0-plugin-prereqs/` — result: 2 hits, BOTH in historical-attribution context (s-config-fix-burst-6.md "Closure record originally misstated..." sentence + s-config-pass-5.md immutable adversary finding description). Zero forward-propagating errors.

2. `rg "colon where canonical template uses em-dash|\{overlay_path\}" .factory/cycles/wave-0-plugin-prereqs/lessons.md` — result: 1 hit in entry 42 historical-evidence narrative (documenting the OLD wrong text). Zero forward-propagating errors in entry 41 bullet bodies.

3. All 5 function names cited in this closure record byte-verified against overlay.rs source:
   - `make_e_spec_019_unknown_extends` — confirmed at grep line 726
   - `make_e_spec_020_instance_id_mismatch` — confirmed at grep line 704
   - `make_e_spec_021_tables_in_overlay` — confirmed at grep line 668
   - `e_spec_022_unknown_org_slug` — confirmed at grep line 645
   - `make_e_spec_023_unrecognized_field` — confirmed at grep line 686
