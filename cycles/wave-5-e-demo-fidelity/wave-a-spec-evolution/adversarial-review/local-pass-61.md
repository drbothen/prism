---
document_type: adversarial-review
review_id: wave-a-spec-pass-61
pass_number: 61
reviewer: vsdd-factory:adversary
review_type: spec
artifact_scope:
  amended:
    - .factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md (v1.24)
    - .factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md (v0.33)
    - .factory/specs/prd-supplements/error-taxonomy.md (v2.67)
  new:
    - .factory/stories/S-WAVE-A-ENGINE-001-header-scheme-field-rule9-validation-auth-dispatch.md (v2.0)
  re-derived:
    - BC-2.16.014 (v1.18), BC-2.01.018 (v1.4), BC-2.01.006 (v1.8), BC-2.01.016 (v1.14),
      BC-2.01.017 (v1.10), BC-2.16.008, vp-159 (v1.26), vp-153 (v0.28),
      ADR-054 (v0.52), ADR-053 (v0.33), .factory/proposals/security-triage-rule9-cookie-name-charset.md
  indexes:
    - BC-INDEX (v8.70), VP-INDEX (v2.12), ARCH-INDEX (v2.273), STORY-INDEX (v2.723)
streak_pre_pass: "0/3"
verdict: NOT CLEAN
findings_count: 16
severity_breakdown:
  critical: 1
  high: 2
  medium: 7
  low: 6
novelty: HIGH
date: 2026-07-24
---

# Wave-A Spec-Evolution Adversarial Review — Local Pass 61

**Scope:** BC-2.16.009 v1.24 (SEC-001 amendment) + ADR-053 v0.33 (de-normativization) + error-taxonomy v2.67 (template (b)/(c) tchar gate) + S-WAVE-A-ENGINE-001 v2.0 (new story). Full re-derivation of all prior-converged axes plus fresh-context probe of SEC-001 amendment correctness, SAP-3 reachability, and index consistency.

---

## F-WASE-P61-CRIT-001 — BC-INDEX + ARCH-INDEX normative-source inversion: both indexes mis-describe what D-2013 SEC-001 did

**Severity:** CRITICAL  
**Category:** Index consistency / normative-source mis-attribution (CRIT per ADR-053 de-normativization)  
**POL:** POL-24 (byte-identity across all POL-24 carriers), POL-29 (no phantom constructs in normative text)

**Finding:**

The BC-INDEX v8.70 frontmatter NOTE (line 8) and the ARCH-INDEX v2.273 Changelog row state that D-2013 SEC-001 added the tchar charset constraint to "Rule 9 templates (b)/(c)" and that "template (a) bearer path unchanged." Both statements are wrong in both documents.

Reading BC-2.16.009 v1.24 §Error Conditions, E-SPEC-027 rows EC-009-043..046 (four edge cases: `;=` injection, bare `=`, SP, CTL) are under template (a) — the `"cookie:<name>"` syntactic path. Template (a) is the bearer-adjacent single-value path where the name is the cookie credential name. The SEC-001 tchar constraint applies to template (a) because it is the `"cookie:<name>"` auth_type variant where the name could be injected.

Templates (b) and (c) concern `header_scheme` coherence and absent-`header_scheme`+`cookie_roundtrip` — not the tchar charset. Those were added in v1.22/v1.23 and v1.22 respectively and are UNCHANGED in v1.24.

The ARCH-INDEX v2.273 row additionally says "ADR-053 §D2 is normative source for templates (b)/(c) cookie-name charset rule." Reading ADR-053 §D2: §D2 is the de-normativization clause that delegates the tchar constraint TO BC-2.16.009 and error-taxonomy. ADR-053 §D2 is NOT a POL-24 carrier; it retains templates (b) and (c) normatively (auth_type coherence and absent-header_scheme+cookie_roundtrip), and delegates template (a) to BC-2.16.009. Calling ADR-053 "normative source" inverts the causality.

Additionally, both indexes say "EC-009-046 added" (singular). Reading BC-2.16.009 v1.24 §Error Conditions: EC-009-043, EC-009-044, EC-009-045, and EC-009-046 were all added (four edge cases).

**Required fix:**

1. BC-INDEX frontmatter: add v8.71 NOTE above v8.70 NOTE that records the corrected account of D-2013 SEC-001 (template (a) changed; EC-009-043..046 four edge cases; ADR-053 §D2 is not the sole POL-24 carrier for template (a)). Bump BC-INDEX version 8.70→8.71.

2. ARCH-INDEX Changelog: add new v2.274 row above v2.273 correcting the record (template (a) changed; ADR-053 §D2 de-normativized template (a) to BC-2.16.009+taxonomy, not the other way; four edge cases EC-009-043..046; v2.273 row NOT modified — historical-row integrity). ADR-053 registry cell: prepend v0.34 with the corrected normative-source statement. Bump ARCH-INDEX version 2.273→2.274.

**Routing:** state-manager (index owners: BC-INDEX + ARCH-INDEX)

---

## F-WASE-P61-HIGH-001 — BC-2.16.009 v1.24 + error-taxonomy v2.67: backtick (U+0060) missing from tchar charset in template (a), 14-char list produces 76-char permitted set vs RFC 9110 §5.6.2 correct 77

**Severity:** HIGH  
**Category:** POL-24 byte-identity gap / RFC compliance defect  
**POL:** POL-24 (byte-identity across all POL-24 carriers), SEC-001 vector integrity

**Finding:**

BC-2.16.009 v1.24 §Error message (syntactic) and §Error Conditions E-SPEC-027 template (a) list the special char set as: `! # $ % & ' * + - . ^ _ ~ |` — 14 special chars, ordering ends `~ |`.

RFC 9110 §5.6.2 defines tchar as: `"!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"` — 15 special chars, ordering ends `` ^ _ ` | ~ ``.

Three independent sources confirm the backtick is required:

1. RFC 9110 §5.6.2 explicitly lists `"^" / "_" / "\`" / "\|" / "~"` — backtick before pipe before tilde.
2. security-triage-rule9-cookie-name-charset.md §5 item 2: "permitted set: 26+26+10+**15** = 77 characters." The 14-char special list yields only 76.
3. S-WAVE-A-ENGINE-001 `is_valid_cookie_name_tchar` match arm includes `b'\`'` per §Implementation Notes.

The same defect propagates to error-taxonomy.md v2.67 E-SPEC-027 template (a) message, which must be byte-identical per POL-24. The omission of backtick means the error message would tell an operator that a cookie name like `` foo`bar `` is invalid when it is actually RFC-compliant.

**Required fix:**

Restore backtick (U+0060) to the special-char enumeration in both:
- BC-2.16.009 §Error message (syntactic): `` ! # $ % & ' * + - . ^ _ ` | ~ `` (15 chars, RFC-ordered)
- BC-2.16.009 §Error Conditions E-SPEC-027 template (a) message: same enumeration  
- error-taxonomy.md E-SPEC-027 template (a) message: same enumeration (POL-24 atomicity)

Also fix §Syntactic check constraint bullet in BC-2.16.009 Rule 9 to cite the correct 15-char set.

**Routing:** product-owner (BC-2.16.009 + error-taxonomy content)

---

## F-WASE-P61-HIGH-002 — S-WAVE-A-ENGINE-001 + BC-2.16.009: no AC or Red Gate test covers Rule 9 on the `add_sensor_spec` MCP surface (SAP-3 violation)

**Severity:** HIGH  
**Category:** SAP-3 spec-arm reachability gap  
**SAP:** SAP-3 (every BC postcondition arm needs end-to-end test from public surface)

**Finding:**

S-WAVE-A-ENGINE-001 §Acceptance Criteria + §Red Gate Tests cover:
- AC-001..AC-014: validate_sensor_spec() path (called at `load_spec` / boot)
- AC-015: compile-time non_exhaustive constraint
- None explicitly assert `add_sensor_spec` → Rule 9 coverage

BC-2.16.009 §Validation Rules 9 says Rule 9 header_scheme charset validation fires "at spec load time." There are two spec-load paths: `SpecLoader::parse()` (called by BOTH `load_spec`/boot AND `add_sensor_spec`) and `validate_sensor_spec()` (called at boot only; VP-059 scope excludes Rules 8-10).

security-triage-rule9-cookie-name-charset.md §1.3 identifies `add_sensor_spec` as the sole runtime-exploitable injection surface (it accepts sensor_spec from MCP input during live operation; the SEC-001 attack vector is live). If Rule 9 validation only reaches `SpecLoader::parse()` during boot (cold path) but NOT during `add_sensor_spec` (hot path), SEC-001 is not closed at runtime.

The story has zero ACs or RGTs that verify `add_sensor_spec` dispatches through `SpecLoader::parse()` and that a non-tchar cookie name is rejected with E-SPEC-027 template (a) at the MCP surface. SAP-3 requires at least one end-to-end test from the public surface (`add_sensor_spec` tool call) not merely a unit test that constructs synthetic spec structs.

**Required fix:**

Add to S-WAVE-A-ENGINE-001:
- AC-019: `add_sensor_spec` with a cookie auth spec containing a non-tchar cookie name (e.g., `cookie:foo;bar`) MUST return E-SPEC-027 template (a) error (not E-SPEC-027 template (b) or (c)) with the tchar rejection message via the MCP wire.
- AC-020: `add_sensor_spec` with a cookie auth spec containing a valid tchar cookie name MUST succeed.
- RG-024: Red Gate test invoking `add_sensor_spec` on the MCP stdio surface and asserting the serialized JSON error envelope matches E-SPEC-027 template (a) wire shape.

BC-2.16.009 §Validation Rules 9: add explicit `**Entry points and function coverage (normative):**` statement naming both `validate_sensor_spec()` (Rules 1–7 at boot) and `SpecLoader::parse()` (Rules 1–10 including add_sensor_spec path) as the two enforcement points.

**Routing:** story-writer (AC/RGT additions); product-owner (BC-2.16.009 entry-points sub-section)

---

## F-WASE-P61-MED-001 — BC-2.16.009 §Description says "nine categories" — Rule 8 probe_table omitted; correct count is ten

**Severity:** MEDIUM  
**Category:** Stale count / §Description drift  

**Finding:**

BC-2.16.009 v1.24 §Description states "The validator enforces nine categories of constraints..." Reading the §Validation Rules section: Rule 1 (auth_type), Rule 2 (base_url), Rule 3 (tables), Rule 4 (columns), Rule 5 (env-var), Rule 6 (OCSF), Rule 7 (step template), Rule 8 (probe_table reference — added in v1.11 per D-1946 burst-4), Rule 9 (header_scheme), Rule 10 (auth_acquisition coherence). That is ten rules/categories, not nine.

Rule 8 was added in v1.11 and §Description was not updated. The v1.12 bump corrected "7→9" (adding Rules 9+10 added in v1.12) but started from the list that already excluded Rule 8, so the count ended at 9 when it should have been 10.

**Required fix:** BC-2.16.009 §Description: "nine categories" → "ten categories"; add Rule 8 (probe_table reference validation, E-SPEC-026) to the category enumeration list in the §Description, between HTTP method whitelist and `header_scheme`.

**Routing:** product-owner (BC-2.16.009 §Description)

---

## F-WASE-P61-MED-002 — S-WAVE-A-ENGINE-001 RG-020..RG-023 have no AC anchor; SEC-001 security gate is AC-uncovered

**Severity:** MEDIUM  
**Category:** Story completeness / Red Gate test traceability gap

**Finding:**

S-WAVE-A-ENGINE-001 §Red Gate Tests lists RG-020, RG-021, RG-022, RG-023 (four tests covering EC-009-043..046, the SEC-001 edge cases). However, none of these four RGTs is cited in any AC row under §Acceptance Criteria (AC-001..AC-018 in the current story).

VSDD convention per BC-5.38.001: every Red Gate test must trace to an AC. RGTs without AC anchors indicate either:
(a) The AC was omitted (story gap), or
(b) The RGT was added without updating the AC list (authoring omission).

Given that RG-020..023 directly cover the SEC-001 injection vectors (`;=` injection, bare `=`, SP, CTL in cookie name), the AC gap means the SEC-001 security gate has no formal acceptance criterion — it only has Red Gate tests. The acceptance gate for "cookie name with `;=` injection character is rejected at add_sensor_spec" has no AC to hang the story-level holdout on.

**Required fix:** Add AC-019 (non-tchar cookie name rejected at `add_sensor_spec` MCP surface per SAP-3, closing HIGH-002 simultaneously) and AC-020 (valid tchar cookie name accepted) to S-WAVE-A-ENGINE-001, anchoring RG-020..023 and RG-024 (new from HIGH-002 fix).

**Routing:** story-writer

---

## F-WASE-P61-MED-003 — S-WAVE-A-ENGINE-001 carries three stale frozen-perimeter claims

**Severity:** MEDIUM  
**Category:** Stale state / story freshness

**Finding:**

S-WAVE-A-ENGINE-001 v2.0 §Context or §Implementation Notes contains these stale claims:

1. "BC-5.39.001 streak = 3/3 CLEAN (passes 58/59/60)" — The streak was RESET 3/3→0/3 by D-2013 (SEC-001 reopening). The story was authored in D-2013 immediately after the reset, but this line reflects the pre-reset converged state.

2. "error-taxonomy v2.67 — FROZEN per D-2013 convergence" — error-taxonomy v2.67 is NOT frozen; this pass found HIGH-001 which requires an edit to error-taxonomy.md E-SPEC-027 template (a).

3. "Primary authoring source: BC-2.16.009 v1.23" — The current version is v1.24 (SEC-001 amendment). A story that names a superseded version as its primary source causes implementer confusion about which version's contracts they are implementing.

**Required fix:** story-writer removes/corrects these three stale claims in S-WAVE-A-ENGINE-001:
1. Remove or correct the "streak = 3/3" line to "BC-5.39.001 streak 0/3 (reset D-2013; pass 61+ required)"
2. Remove "FROZEN" claim for error-taxonomy (or correct to "v2.67 + v2.68 pending HIGH-001 fix")
3. Update "Primary authoring source" from BC-2.16.009 v1.23 to v1.24 (and note v1.25 pending HIGH-001+HIGH-002 fixes)

**Routing:** story-writer

---

## F-WASE-P61-MED-004 — Four contracted BCs missing S-WAVE-A-ENGINE-001 in their §Traceability Stories rows

**Severity:** MEDIUM  
**Category:** Traceability gap

**Finding:**

S-WAVE-A-ENGINE-001 frontmatter `behavioral_contracts: [BC-2.16.009, BC-2.01.017, BC-2.16.014, BC-2.01.016]`. Convention per BC-2.16.009 §Traceability: every story that contracts a BC should appear in that BC's `| Stories |` row. Reading each BC's §Traceability:

- BC-2.16.009 v1.24 `| Stories |` row: lists S-1.11, S-1.13, PLUGIN-MIGRATION-001-F, S-SPEC-ENV-VAR-001, S-DEMO-CROWDSTRIKE-MULTIREGION-001, S-SPEC-HTTP-METHOD-VALIDATION-001, S-5.04 — **missing S-WAVE-A-ENGINE-001**
- BC-2.01.017 v1.10 §Traceability: has `| Story | S-DTU-CYBERINT-AUTH-FIDELITY-001 |` (singular) — **missing S-WAVE-A-ENGINE-001 entirely**
- BC-2.16.014 v1.18 §Traceability: **missing S-WAVE-A-ENGINE-001**
- BC-2.01.016 v1.14 §Traceability: **missing S-WAVE-A-ENGINE-001**

STORY-INDEX v2.723 §BC Traceability Matrix rows for these four BCs already include S-WAVE-A-ENGINE-001 (added in D-2013 v2.723 burst). The gap is in the individual BC files themselves.

**Required fix:** Add S-WAVE-A-ENGINE-001 to `| Stories |` row in each of the four BC files. For BC-2.01.017 which only has a singular `| Story |` row, add a new `| Stories | S-WAVE-A-ENGINE-001 |` row.

**Routing:** state-manager (index/traceability row updates in BC files) — product-owner for BC-2.16.014 + BC-2.01.016 if they require content amendments; state-manager for the BC-2.16.009 and BC-2.01.017 row-append operations

---

## F-WASE-P61-MED-005 — STORY-INDEX v2.723 §BC Traceability Matrix BC-2.16.009 row missing S-DEMO-CROWDSTRIKE-MULTIREGION-001 and S-5.04

**Severity:** MEDIUM  
**Category:** STORY-INDEX vs BC-file drift

**Finding:**

STORY-INDEX v2.723 §BC Traceability Matrix BC-2.16.009 row lists: `S-1.11, S-1.13, PLUGIN-MIGRATION-001-F, S-SPEC-ENV-VAR-001, S-SPEC-HTTP-METHOD-VALIDATION-001, S-WAVE-A-ENGINE-001`.

BC-2.16.009 v1.24 §Traceability Stories row lists: `S-1.11, S-1.13, PLUGIN-MIGRATION-001-F, S-SPEC-ENV-VAR-001, S-DEMO-CROWDSTRIKE-MULTIREGION-001, S-SPEC-HTTP-METHOD-VALIDATION-001, S-5.04`.

Two stories present in the BC-2.16.009 file are absent from the STORY-INDEX matrix row: S-DEMO-CROWDSTRIKE-MULTIREGION-001 and S-5.04.

**Required fix:** STORY-INDEX §BC Traceability Matrix BC-2.16.009 row += S-DEMO-CROWDSTRIKE-MULTIREGION-001 + S-5.04.

**Routing:** state-manager

---

## F-WASE-P61-MED-006 — records-lint.sh CONFIG block excludes `prd-supplements/` from VERSIONED_ARTIFACT_DIRS — error-taxonomy.md version/changelog changes are not L1/L7 gated

**Severity:** MEDIUM  
**Category:** Process-gap / mechanical gate coverage gap (TD-VSDD-092)

**Finding:**

scripts/records-lint.sh defines `VERSIONED_ARTIFACT_DIRS` (the directories scanned for L1 frontmatter-version == top-changelog-row and L7 descending-order checks). Reading the CONFIG block: the array contains `.factory/specs/behavioral-contracts/`, `.factory/specs/architecture/decisions/`, `.factory/specs/verification-properties/` — but NOT `.factory/specs/prd-supplements/`.

error-taxonomy.md lives in `.factory/specs/prd-supplements/` and is a versioned artifact with `version:` frontmatter and a `## Changelog` section. It is a POL-24 primary carrier and changes in almost every spec-evolution burst. If L1/L7 gates do not cover prd-supplements/, then error-taxonomy.md (and any other versioned artifact in that directory) can have:
- Frontmatter `version:` not matching its top changelog row (L1 miss)
- Changelog rows in non-descending order (L7 miss)

This was specifically relevant in D-1993 where BC-INDEX.md frontmatter was at v8.63 while the changelog showed v8.64 — a class of drift that L1 would have caught if prd-supplements/ were included.

**Required fix:** Add `.factory/specs/prd-supplements/` to `VERSIONED_ARTIFACT_DIRS` in scripts/records-lint.sh CONFIG block.

**Routing:** devops-engineer

---

## F-WASE-P61-MED-007 — BC-INDEX BC-2.16.009 registry-cell version history is NOT newest-first: v1.24 was appended at the end after v1.13

**Severity:** MEDIUM  
**Category:** BC-INDEX formatting convention violation

**Finding:**

BC-INDEX v8.70 §BC Registry BC-2.16.009 row §Status cell: the amendment history inside the cell reads v1.23 → v1.22 → ... → v1.13 → **v1.24** at the end. The v1.24 amendment was appended at the tail rather than prepended at the head.

Convention per other BC rows in BC-INDEX: newest-first ordering (v1.N → v1.N-1 → ... → v1.1). A reader scanning the cell reads v1.23 first and concludes the current version is v1.23, missing v1.24 entirely. This is a usability and consistency defect; the index is the first place a reader looks to find the current state.

**Required fix:** Reorder the BC-2.16.009 §Status cell in BC-INDEX to newest-first: v1.25 (after HIGH-001/HIGH-002/MED-001 fixes) prepended at head, followed by corrected v1.24, then v1.23, v1.22, etc.

**Routing:** state-manager (BC-INDEX index cell ordering)

---

## F-WASE-P61-LOW-001 — records-lint.sh L9 regex does not catch all prose forms of volatile line citations (space-separated, backtick-wrapped)

**Severity:** LOW  
**Category:** Gate coverage gap (TD-VSDD-092 L9)

**Finding:**

The L9 check in scripts/records-lint.sh uses a regex to detect staged additions containing `file.rs:NNN` patterns. Reading the regex: it matches `\w+\.rs:\d+` (word-chars, .rs, colon, digits). It would miss:

- `` `spec_parser.rs` at line 459 `` (backtick-wrapped filename + prose "at line NNN")
- `(spec_parser.rs line 459)` (parenthetical prose form)
- `spec_parser.rs ~line 459` (tilde prefix)

These prose forms are equally volatile (line numbers decay on diffs) and must be caught. TD-VSDD-091 amendment 2026-07-24 REVOKED the "changelogs exception" — ALL record-tier text must use section/symbol/anchor cites only.

**Required fix:** Extend L9 regex to also match `` `\w+\.rs`[^:] `` prose + `line \d+` combos, and the parenthetical form `\w+\.rs line \d+`. Alternative: broaden the existing match to `\w+\.rs[: ]+\d+` which catches both colon and space separators.

**Routing:** devops-engineer

---

## F-WASE-P61-LOW-002 — ADR-053 §D2 still assigns E-SPEC-027 template registration to the engine story as [PLANNED]

**Severity:** LOW  
**Category:** Stale [PLANNED] marker

**Finding:**

ADR-053 v0.33 §D2 contains a row in the closed-value-set table or the normative E-SPEC-027 registration row marked `[PLANNED — engine story delivers template (a)]`. Reading error-taxonomy.md v2.67: E-SPEC-027 templates (a), (b), and (c) are ALL registered in the taxonomy (added in D-1948 burst-4 at error-taxonomy v2.58). The engine story is NOT required to register E-SPEC-027 — it is already registered.

The [PLANNED] marker was correct in v0.33 at the time of SEC-001 (D-2013 burst) — the marker predated D-1948. In v0.33 authored 2026-07-24 (after D-1948), this marker is stale.

**Required fix:** ADR-053 §D2: remove or correct the `[PLANNED — engine story delivers template (a)]` marker on E-SPEC-027 registration. The registration was executed at error-taxonomy.md v2.58 (D-1948 burst-4, 2026-07-22).

**Routing:** architect (ADR content)

---

## F-WASE-P61-LOW-003 — S-WAVE-A-ENGINE-001 §Error Conditions EC ID scheme is discontinuous: EC-001..EC-013, then EC-043..EC-046

**Severity:** LOW  
**Category:** Story internal consistency

**Finding:**

S-WAVE-A-ENGINE-001 §Error Conditions or §Test Vectors section assigns story-internal EC IDs. The IDs EC-001 through EC-013 cover the main Rule 9/10 error paths (matching BC-2.16.009 EC-009-030..042). Then EC-043..EC-046 appear for the SEC-001 edge cases (matching BC-2.16.009 EC-009-043..046). The story-internal scheme jumps from EC-013 to EC-043, leaving a 29-ID gap that readers may interpret as missing coverage.

The likely cause: the story-writer used the BC's EC IDs (EC-009-043..046) directly as story EC IDs rather than continuing the story's own numbering (EC-014..EC-017).

**Required fix:** Either: (a) renumber story-internal EC-043..046 → EC-014..EC-017 and add a cross-reference note to BC-2.16.009 EC-009-043..046; or (b) add a comment explaining the gap as intentional alignment with BC-2.16.009's EC numbering.

**Routing:** story-writer

---

## F-WASE-P61-LOW-004 — S-WAVE-A-ENGINE-001 AC-015 states a compile-time property with no runtime assertion planned

**Severity:** LOW  
**Category:** Acceptance criterion completeness / SID-2

**Finding:**

S-WAVE-A-ENGINE-001 AC-015 (per §Acceptance Criteria): states that the tchar validation match arm should be compile-time exhaustive (non_exhaustive gate or similar). This is a compile-time property.

However, there is no corresponding runtime assertion or Red Gate test that verifies: (a) a byte value outside the tchar set returns `false` from the validation function, and (b) a byte value inside the set returns `true`. The compile-time gate prevents adding new arms without updating the match, but does not assert the correctness of the current charset.

Per SID-2 (implementer discipline: composed-output assertions): at least one test must assert the full composed string as emitted. The tchar validation function's correctness should be asserted at the unit level with specific known-good and known-bad bytes including the backtick (once HIGH-001 is fixed) and at least one SEC-001 vector byte.

**Required fix:** story-writer (or product-owner): Add AC (or note in AC-015) that `is_valid_cookie_name_tchar(b'`')` must return `true` and `is_valid_cookie_name_tchar(b';')` must return `false`, with explicit test assertions. This is a runtime correctness gate, complementary to the compile-time exhaustiveness gate.

**Routing:** story-writer

---

## F-WASE-P61-LOW-005 — STORY-INDEX v2.723 frontmatter uses v-prefixed version string: "v2.723" should be "2.723"

**Severity:** LOW  
**Category:** Formatting convention inconsistency

**Finding:**

STORY-INDEX v2.723 frontmatter line 4: `version: "v2.723"`. All other index artifacts in `.factory/` (BC-INDEX, ARCH-INDEX, VP-INDEX) use bare numeric version strings without a `v`-prefix. The `v`-prefix is for git tags and semver release strings, not for document version fields in YAML frontmatter.

Mechanical impact: scripts/records-lint.sh L1 check compares `version:` frontmatter against the top changelog row's version. If the frontmatter says `"v2.723"` but the changelog row says `v2.723` (or vice versa), the L1 check may flag a false mismatch or miss a real one depending on normalization.

**Required fix:** STORY-INDEX frontmatter: strip v-prefix from version field. Bump to "2.724" (bump for this FB45 burst simultaneously). Add v2.724 changelog row.

**Routing:** state-manager

---

## F-WASE-P61-LOW-006 — ADR-053 §D2 closed-value-set table has no worked example for the SEC-001 cookie-name injection vector

**Severity:** LOW  
**Category:** Spec completeness / worked-example gap

**Finding:**

ADR-053 v0.33 §D2 defines the closed-value-set table for `header_scheme` values and their template dispatch rules. The table enumerates templates (a), (b), (c) with conditions. However, there is no worked row that demonstrates the SEC-001 injection scenario: `"cookie:foo;bar"` (name containing `;`, a non-tchar character) firing template (a) rejection.

The table was last amended in v0.33 for the (b)/(c) tchar cookie-name charset additions. The v0.33 amendment added EC-009-046 as an example for template (b)/(c) but did not add a parallel worked example for template (a) rejection of the same injection vector.

For a security-critical constraint, having no worked example of the attack vector makes it harder for implementers to verify they are handling the template (a) path correctly.

**Required fix:** ADR-053 §D2: add a worked row for template (a) showing `"cookie:foo;bar"` (`;` is non-tchar) → E-SPEC-027 template (a) rejection with reference to SEC-001 CWE-20/CWE-74 vector.

**Routing:** architect (ADR content)

---

## Verified-Clean Axes

All prior-converged axes re-derived clean at pass 61 under the amended perimeter:

- E-SPEC-028 co-fire truth table (six reachable pairs; (b)⊕(g) exclusion holds)
- EC-009-036 both branches + custom_via_plugin counterfactual
- DI-012 6-value + counting-unit parity (INV-014-006)
- E-SPEC-013 v2.64 byte-verbatim at VP-153 Rule B
- ADR-054 §D4 6-field coherence; VP-159 P3/P4 De Morgan totality
- VP-159 AC-4b pinned-clock kill condition (base_time+86_400; TTL provably FALSE)
- BC-INDEX four-representation coherence (268=251+4+0+7+6) pre-amendment
- VP-153 as-built 5-value/PLANNED-6-value framing honest vs spec_parser.rs VALID_AUTH_TYPES
- D11 [EXECUTED] markers for ADR-054 sampled accurate
- TV-9 NotFoundCredentialResolver code-verified at auth_provider.rs §P2 dispatch
- POL-7/9/11/23/27/32/36 all clean
- ADR-053 template (b)/(c) normative retention (BC-2.01.017 §P3 cookie-name tchar gate correct)
- Three-site E-SPEC-027 template (b)/(c) propagation: ADR-053 §D2 / BC-2.16.009 Rule 9 / error-taxonomy.md (template (b)/(c) added in v1.22/v1.23 / v2.67 respectively — independently verified)
- security-triage-rule9-cookie-name-charset.md §1.3 `add_sensor_spec` sole-runtime-exploitable-surface designation (HIGH-002 finding is fresh, not a re-raise of prior axis)

**Novelty assessment:** HIGH — CRIT-001 (normative-source inversion not caught in 60 passes), HIGH-001 (backtick omission: HIGH-001 is a new defect introduced IN the SEC-001 amendment — v1.24 itself omitted the backtick while strengthening the charset; fresh fresh-context derivation needed to spot this class), HIGH-002 (SAP-3 add_sensor_spec end-to-end reachability gap: first pass on the new story), MED-006 (L9 gate operational gap: DRIFT-L9-VACUOUS-GATE-001 class).

---

## Routing Summary

| Finding | Routing |
|---------|---------|
| F-WASE-P61-CRIT-001 | state-manager: BC-INDEX v8.71 NOTE + ARCH-INDEX v2.274 row + ADR-053 v0.34 registry cell |
| F-WASE-P61-HIGH-001 | product-owner: BC-2.16.009 v1.25 (tchar backtick) + error-taxonomy v2.68 (POL-24 sync) |
| F-WASE-P61-HIGH-002 | story-writer: S-WAVE-A-ENGINE-001 AC-019/AC-020/RG-024; product-owner: BC-2.16.009 entry-points sub-section |
| F-WASE-P61-MED-001 | product-owner: BC-2.16.009 §Description count + Rule 8 category |
| F-WASE-P61-MED-002 | story-writer: S-WAVE-A-ENGINE-001 AC-019/AC-020 (addresses both HIGH-002 and MED-002 simultaneously) |
| F-WASE-P61-MED-003 | story-writer: remove 3 stale frozen-perimeter claims from S-WAVE-A-ENGINE-001 |
| F-WASE-P61-MED-004 | state-manager: BC-2.16.009 + BC-2.01.017 Stories rows; product-owner: BC-2.16.014 + BC-2.01.016 Stories rows |
| F-WASE-P61-MED-005 | state-manager: STORY-INDEX BC-2.16.009 reverse-map row += S-DEMO-CROWDSTRIKE-MULTIREGION-001 + S-5.04 |
| F-WASE-P61-MED-006 | devops-engineer: records-lint.sh CONFIG adds prd-supplements/ |
| F-WASE-P61-MED-007 | state-manager: BC-INDEX BC-2.16.009 row reorder newest-first; prepend v1.25 |
| F-WASE-P61-LOW-001 | devops-engineer: L9 regex enhancement |
| F-WASE-P61-LOW-002 | architect: ADR-053 §D2 E-SPEC-027 registration marker update |
| F-WASE-P61-LOW-003 | story-writer: EC ID discontinuity |
| F-WASE-P61-LOW-004 | story-writer: AC-015 runtime assertion |
| F-WASE-P61-LOW-005 | state-manager: STORY-INDEX version v-prefix strip + bump to 2.724 |
| F-WASE-P61-LOW-006 | architect: ADR-053 §D2 worked example row for SEC-001 vector |

---

## Verdict

```
CLEAN (strict): no   # 16 findings: 1 CRIT, 2 HIGH, 7 MED, 6 LOW
CLEAN (PR-merge): no
BC-5.39.001 streak: 0/3 → 0/3 (streak unchanged; new findings prevent advancement)
```

Novelty: HIGH — CRIT-001 normative-source inversion latent since D-2013; HIGH-001 backtick omission introduced by SEC-001 amendment itself; HIGH-002 first SAP-3 pass on new story; MED-006 L9 gate operational gap (DRIFT-L9-VACUOUS-GATE-001).

**Next:** FB45 fix-burst to close all 16 findings → adversary pass 62 on amended perimeter (BC-2.16.009 v1.25 + error-taxonomy v2.68 + S-WAVE-A-ENGINE-001 amended + ADR-053 v0.34 + all prior converged package).
