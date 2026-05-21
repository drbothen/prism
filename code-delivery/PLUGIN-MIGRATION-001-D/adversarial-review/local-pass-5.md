---
document_type: adversarial-review-pass
story_id: PLUGIN-MIGRATION-001-D
pass_number: 5
pass_scope: LOCAL-SPEC-LEVEL
pass_date: 2026-05-20
adversary_model: claude-opus-4-7 (1M context); fresh-context
streak_before: 0/3
streak_after: 0/3
findings_summary: "1 HIGH + 2 MED + 2 LOW + 1 OBS"
checkpoint_status: BLOCKED-soft
---

# Pass-5 Adversarial Review

## Scope

LOCAL spec-level fresh-context verification of FB-IMPL-P4 closures (D-738) and durability sweep of all 45 cumulative pass-1..4 closures. Phases A (lexical-vs-semantic anchor), B (production-grade default), C (named-entity), D (cumulative durability), E (ADR-028 grounding), F (STORY-INDEX/frontmatter consistency), G (BC array propagation), H (token budget/AC count).

## Findings

### F-LP5-HIGH-001 ADR-028 §D2 Armis line internally contradicts BC, story, HS — DTU-grounded value is `bearer_static`, not `api_key`

**Confidence:** HIGH

**Issue:** ADR-028 §D2 line 87 declared: "**Armis:** API key header per DTU enforcement → spec declares `auth_type = "api_key"`". This contradicts every other artifact authored in FB-IMPL-P4:
- BC-2.16.013 v1.4 line 175: `armis.sensor.toml — auth_type: "bearer_static"`
- Story v1.4 AC-004, AC-011, Task 6 all say `bearer_static`
- HS-016 v1.1 says `bearer_static`
- STORY-INDEX v2.161 + BC-INDEX v5.25 say `bearer_static`

**Code ground-truth:** `crates/prism-dtu-armis/src/lib.rs:16-17` declares BearerStatic auth. The legacy `ArmisAuth::auth_type_name()` returns `"api_key"` — the latent label bug ADR-028 §D2 was supposed to immunize against.

The ADR's own §D2 table contains the bug the §D2 rule forbids. POL-22 Phase C named-entity verification surfaces it.

**Routing:** architect.

### F-LP5-MED-001 STORY-INDEX vs story frontmatter `crates_touched` divergence — POL-13 violation

Story frontmatter line 27: `crates_touched: [prism-sensors, prism-spec-engine]` (2 crates)
STORY-INDEX v2.161 row 399: `prism-sensors,prism-spec-engine,prism-core` (3 crates)
Story body §Files to MODIFY line 1033 lists `crates/prism-core/src/error.rs` (Task 11 — D-737 Decision 3 scope expansion)

POL-13 requires STORY-INDEX ↔ story-file frontmatter agreement. Sibling-sweep missed frontmatter when Task 11 was added during FB-IMPL-P4.

**Routing:** story-writer.

### F-LP5-MED-002 BC-2.16.001 v1.5 `modified:` frontmatter is null while Changelog declares v1.5 dated 2026-05-20 — POL-27 violation

BC-2.16.001 v1.5 frontmatter has `modified: null` but Changelog line 129 documents v1.5 dated 2026-05-20. POL-27 requires `modified:` to use ISO 8601 scalar matching latest Changelog entry.

**Routing:** product-owner.

### F-LP5-LOW-001 Cyberint cookie line citation off-by-1: BC + ADR-028 + HS-015 cite "lines 43-46" but `extract_session_token` is at lines 44-53

Citations across BC-2.16.013, ADR-028 §D2, HS-015, story AC-011 use `prism-dtu-cyberint/src/routes/alerts.rs:43-46`. Function `extract_session_token` actually spans lines 44-53. TD-VSDD-091 anti-volatile-pin: replace numeric range with symbol anchor `::extract_session_token()`.

**Routing:** product-owner + architect (POL-25 sibling sweep across BC, ADR, HS, story).

### F-LP5-LOW-002 SpecErrorCode enum lacks `#[non_exhaustive]`; Task 11 instruction is conditional but answerable in spec scope

Story Task 11 (lines 832-834) uses conditional phrasing ("check before… if not marked, add `#[non_exhaustive]`") even though the story already verified the enum lacks `#[non_exhaustive]`. Per CLAUDE.md production-grade default Rule 6, instructions answerable in scope must be unconditional.

**Routing:** story-writer.

### F-LP5-OBS-001 [process-gap] ADR-028 §Decision body internally contradicts its anchor rule on Armis row — single-pass authoring of architectural source-of-truth lacks cross-row self-consistency check

F-LP5-HIGH-001 is the same class of mis-grounding bug ADR-028 was supposed to permanently codify against. The ADR's own §D2 table contains the bug the §D2 rule forbids. Yet 4 fix-bursts plus a fresh architect-authored ADR all failed to catch it.

Process gap: ADR authoring of source-of-truth grounding rules requires an internal cross-row consistency check before promotion. Suggested codification: add checklist item to architect prompts or consistency-validator dispatch criteria for ADRs anchoring parallel rows (sensor rows, error-code rows, BC rows).

**Routing:** orchestrator (defer to S-7.02 cycle-close codification).

## Cumulative-Closure Durability Verification

12 closures verified DURABLE across passes 1–4. F-LP4-HIGH-004 auth_type propagation REGRESSED in ADR-028 §D2 only — all other artifacts (BC-2.16.013, story, HS-013..016, STORY-INDEX, BC-INDEX) remain consistent with bearer_static for Armis. The regression is isolated to the ADR §D2 Armis row.

## Phase Verification Summary

A PASS / B PASS / C FAIL / D FAIL / E content FAIL / F FAIL / G PASS / H PASS

## Verdict

**BLOCKED-soft** — 1 HIGH (F-LP5-HIGH-001) + 2 MED (F-LP5-MED-001/002) require closure. Streak resets to 0/3. The HIGH is notable because the ADR's own §D2 table contained the exact bug the rule was authored to codify against.

## Streak Update
- streak_before: 0/3
- streak_after: 0/3 (reset — HIGH finding blocks)
- next_action: architect fixes ADR-028 §D2 Armis row; PO fixes BC-2.16.001 modified field + cyberint symbol anchor sweep; story-writer fixes story crates_touched + Task 11 phrasing + cyberint cite in story body; then state-manager FB-IMPL-P5 closure.
