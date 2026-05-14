---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 31
target_sha: bc9a9d2a
story_content_sha: 5e37c6cf
error_taxonomy_content_sha: 2e6af699
bc_content_sha: 84f58565
base_sha: 95d46be2
verdict: BLOCKED
streak: "0/3 HOLD (pass-31 BLOCKED: 2 HIGH + 1 MED — three new finding classes; trajectory increased 1→3)"
finding_summary: {CRITICAL: 0, HIGH: 2, MEDIUM: 1, LOW: 0, OBS: 0}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7, pass-8, pass-9, pass-10, pass-11, pass-12, pass-13, pass-14, pass-15, pass-16, pass-17, pass-18, pass-19, pass-20, pass-21, pass-22, pass-23, pass-24, pass-25, pass-26, pass-27, pass-28, pass-29, pass-30]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6, fix-burst-7, fix-burst-8, fix-burst-9, fix-burst-10, fix-burst-11, fix-burst-12, fix-burst-13, fix-burst-14, fix-burst-15, fix-burst-16, fix-burst-17, fix-burst-18, fix-burst-19, fix-burst-20, fix-burst-21, fix-burst-22, fix-burst-23, fix-burst-24, fix-burst-25, fix-burst-26, fix-burst-27, fix-burst-28]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → 5 → 1 → 1 → 3"
idempotency_check: false
post_fix_check: true
post_fix_target: "fix-burst-28 (F-LP30-MED-001 §References BC-2.16.002 cross-table completeness)"
trajectory_note: "Pass-31 broke the decreasing trend (1→3) — fresh-context found NEW content drifts not covered by codifications #11-#15 + #13 sub-extension (which targeted format symmetry and completeness, not non-title content text). Three distinct finding classes surfaced: (1) error message template three-site drift for E-PLUGIN-013/014 [codification #16 candidate]; (2) BC-2.17.002 EC-17-007 cross-spec security-semantic conflict with AC-7 default-deny; (3) AC-15 AuthToken Debug-format mismatch with existing code."
producer: "adversary (vsdd-factory; reified by state-manager due to read-only tool profile)"
---

# Adversarial Pass 31 — S-PLUGIN-PREREQ-D

**Verdict: BLOCKED (2 HIGH + 1 MEDIUM)**

**Context:** This is a post-fix-burst-28 fresh-context pass. Fix-burst-28 closed 1 MED
(F-LP30-MED-001: §References section appended BC-2.16.002 verbatim H1 entry between ADR-023
§C4 and BC-2.17.001, closing the cross-table completeness gap). The expected outcome was CLEAN
(0/3 → 1/3). Actual: BLOCKED by 2 HIGH + 1 MED — three new finding classes. Net actionable:
3 findings (2 HIGH + 1 MED). Streak holds at 0/3 per BC-5.39.001.

Trajectory pass-25..pass-31: 4 → 1 → 4 → 5 → 1 → 1 → **3** — decreasing trend broken.
Pass-31 surfaced fresh content drifts not covered by any prior codification. Codifications
#11-#15 + #13 sub-extension all HELD (format symmetry + completeness disciplines applied
correctly). The new findings are in a different axis: non-title verbatim text (error message
templates) and cross-spec semantic drift (BC behavior description vs AC contract).

F-LP31-HIGH-001 [process-gap] tagged as codification candidate #16: verbatim cross-table
sweep must extend to error message template text, not only BC title strings.

F-LP31-HIGH-002 [cross-spec security-semantic]: BC-2.17.002 v1.5 EC-17-007 describes
"allowed to any URL (open by default)" but story AC-7 + AC-17 establish `Vec<String>`
default-deny. Source-of-Truth Precedence Rule 1 applies — BC supersedes story when conflict
is contract semantics. BC must be amended to align with AC-7 default-deny in the same
fix-burst as AC-7 lands in implementation.

F-LP31-MED-001: AC-15 prescribes `AuthToken("[REDACTED]")` format (uppercase brackets, double
quotes inside parens) but existing code at `auth_provider.rs:68` implements
`AuthToken(<redacted>)` (lowercase, angle brackets). Story prescription diverges from
existing code — implementer ambiguity.

---

## Codification Regression Checks (#11–#15 + #13 sub-extension)

All six active codification disciplines verified against story v1.28 (SHA 5e37c6cf).

### Codification #11 — Lexical-vs-Semantic Anchor-Content Verification

**Target:** Every POL-22 Phase A anchor citation must be confirmed by opening and grepping
the cited document, not by story-body substring matching alone.

Applied to all 30+ cited anchors in this pass. BC-2.16.002 §Canonical Structured Event
Catalog verified by opening BC file: section heading present. ADR-023 §C4 verified present.
VP-PLUGIN-004/VP-PLUGIN-007 entries verified in VP-INDEX. BC-2.17.001..007 H1 titles verified
by opening each BC file. BC-2.22.001 §Boot Sequence Steps verified present.
BC-2.17.002 §Error Conditions E-PLUGIN-005 verified present (AC-9 trace header).

**Codification #11: HELD — all anchors semantically verified in cited documents.**

### Codification #12 — BC Body-Table Title Verbatim Verification (POL-22 Phase B)

**Target:** Every BC body-table Title cell must match BC H1 verbatim (whitespace-normalized).

9 BC rows in body BC table verified:

| BC | Body-Table Title | BC H1 (from file) | Result |
|----|-----------------|-------------------|--------|
| BC-2.16.002 | "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" | verbatim | PASS |
| BC-2.17.001 | verbatim BC H1 | verified | PASS |
| BC-2.17.002 | verbatim BC H1 | verified | PASS |
| BC-2.17.003 | verbatim BC H1 | verified | PASS |
| BC-2.17.004 | verbatim BC H1 | verified | PASS |
| BC-2.17.006 | verbatim BC H1 | verified | PASS |
| BC-2.17.007 | verbatim BC H1 (with parenthetical annotation preserved) | verified | PASS |
| BC-2.22.001 | verbatim BC H1 | verified | PASS |

**Codification #12: HELD — 8/8 BC body-table Title cells verbatim.**

### Codification #13 — POL-7 Cross-Table Sweep (BC Title Verbatim at ALL Citation Sites)

**Target:** Every BC-NNN.NNN citation in the story (regardless of site: body BC table,
§References, Architecture Compliance Rules, frontmatter comments, prose, exclusion-note
paragraphs) must have verbatim BC H1 title.

Phase B extended verification — 5-chain sample:

| Chain | BC | Body BC Table | §References | Architecture Compliance Rules | Exclusion-Note / Prose | Result |
|-------|-----|--------------|-------------|-------------------------------|------------------------|--------|
| 1 | BC-2.16.002 | PASS (verbatim) | PASS (fix-burst-28 applied) | N/A | N/A | **PASS** |
| 2 | BC-2.17.001 | PASS | PASS (verbatim) | PASS | N/A | PASS |
| 3 | BC-2.17.005 | N/A (not in body table) | PASS (verbatim — line 1016 fixed by fix-burst-25) | N/A | PASS (line 269 fixed by fix-burst-27) | PASS |
| 4 | BC-2.17.002 | PASS | PASS | PASS | N/A | PASS |
| 5 | BC-2.22.001 | PASS | PASS | PASS | N/A | PASS |

**Codification #13: HELD — all BC title citation sites verbatim. fix-burst-28 closure of BC-2.16.002 §References gap CONFIRMED.**

### Codification #13 Sub-Extension — §References Completeness Check

**Target:** All members of `behavioral_contracts:` frontmatter array must appear in §References.

`behavioral_contracts:` frontmatter members: [BC-2.16.002, BC-2.17.001, BC-2.17.002,
BC-2.17.003, BC-2.17.004, BC-2.17.006, BC-2.17.007, BC-2.22.001] — 8 entries.

§References BC entries (post fix-burst-28): BC-2.16.002 + BC-2.17.001 + BC-2.17.002 +
BC-2.17.003 + BC-2.17.004 + BC-2.17.005 + BC-2.17.006 + BC-2.17.007 + BC-2.22.001 — 9 entries
(BC-2.17.005 is in §References but not in frontmatter array; this is correct per codification
#15 design — BC-2.17.005 is cited in exclusion-note paragraph, its §References entry is the
verbatim title anchor).

All 8 frontmatter members present in §References. **Codification #13 sub-extension: HELD.**

### Codification #14 — Phantom-Section-Anchor Sweep

**Target:** Every §X notation in the story that cites a BC or ADR must resolve to an actual
section heading in the cited document.

All §X notations verified:
- Story line 918: BC-2.16.002 §Canonical Structured Event Catalog row pipeline_max_requests_exceeded — section exists: PASS
- Story line 260: same anchor — PASS
- Story line 466: BC-2.16.002 §Canonical Structured Event Catalog row pipeline_max_requests_exceeded — PASS
- ADR-023 §C4 citations: section C4 exists in ADR-023: PASS
- BC-2.17.002 §Error Conditions E-PLUGIN-005: section present — PASS

**Codification #14: HELD — zero phantom-section anchors found.**

### Codification #15 — Sibling-Prose-Not-Swept Exclusion-Note (POL-7 Extension)

**Target:** BCs cited in exclusion-note paragraphs must also have verbatim titles.

Story line 269 (exclusion-note):
> "Note: BC-2.17.005 (Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version) is NOT anchored to this story."

Title verbatim against BC H1: "Plugin Hot Reload — Atomic Module Swap, In-Flight Calls
Complete Against Old Version" — VERBATIM MATCH (fix-burst-27 applied).

**Codification #15: HELD — exclusion-note title verbatim.**

---

## POL-22 Phase A — Anchor Verification (40+ samples)

Verified 40+ story body anchor citations against their target documents (semantic open-and-grep
per codification #11 discipline):

- BC-2.16.002 §Canonical Structured Event Catalog: section present — PASS
- BC-2.17.001..007 H1 titles: all verified present in respective BC files — PASS (8 anchors)
- BC-2.22.001 §Boot Sequence Steps: section present — PASS
- ADR-023 §C4: section C4 present — PASS
- ADR-022 §A (exit codes), §C (runtime wiring), §D (concurrency permits): all sections present — PASS
- VP-PLUGIN-004 (VP-INDEX §VP-149): entry present — PASS
- VP-PLUGIN-007 (VP-INDEX §VP-152): entry present — PASS
- SS-22, SS-17, SS-16 in ARCH-INDEX: all present — PASS
- E-PLUGIN-001..016 in error-taxonomy.md: all present — PASS
- E-PIPELINE-001 in error-taxonomy.md: present — PASS
- All 25 story §References entries: all target files verifiable — PASS
- All BC-2.16.002 catalog row names (pipeline_max_requests_exceeded, etc.) verified in BC-2.16.002 §Canonical Structured Event Catalog — PASS

**POL-22 Phase A: PASS — 40+ anchors semantically verified. Zero phantom or fabricated anchors.**

---

## POL-22 Phase B — BC-Title Chain Verification (8 chains)

Full 8-chain verification: all BCs in `behavioral_contracts:` frontmatter array.

| Chain | BC | Body BC Table Title | §References Title | Verbatim BC H1 | Result |
|-------|----|--------------------|--------------------|----------------|--------|
| 1 | BC-2.16.002 | PASS (verbatim) | PASS (fix-burst-28 applied verbatim H1) | BC H1 confirmed | **PASS** |
| 2 | BC-2.17.001 | PASS | PASS (verbatim) | verified | PASS |
| 3 | BC-2.17.002 | PASS | PASS (verbatim) | verified | PASS |
| 4 | BC-2.17.003 | PASS | PASS (verbatim) | verified | PASS |
| 5 | BC-2.17.004 | PASS | PASS (verbatim) | verified | PASS |
| 6 | BC-2.17.006 | PASS | PASS (verbatim) | verified | PASS |
| 7 | BC-2.17.007 | PASS | PASS (verbatim, parenthetical annotation preserved) | verified | PASS |
| 8 | BC-2.22.001 | PASS | PASS (verbatim) | verified | PASS |

**POL-22 Phase B: 8/8 chains PASS. fix-burst-28 closure CONFIRMED — BC-2.16.002 §References completeness gap closed.**

---

## POL-22 Phase C — Carry-Forward Regression (17+ samples)

Prior fix-burst closures 1..28 spot-checked:

| Prior Finding | Fix Applied At | Regression Check |
|---------------|---------------|-----------------|
| F-LP30-MED-001 (§References BC-2.16.002 completeness) | fix-burst-28 | PASS — §References now contains BC-2.16.002 verbatim H1 entry between ADR-023 §C4 and BC-2.17.001 |
| F-LP29-MED-001 (line 269 BC-2.17.005 exclusion-note title verbatim) | fix-burst-27 | PASS — line 269 reads "Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version" verbatim |
| F-LP28-MED-001 (phantom §-section story:918) | fix-burst-26 | PASS — canonical §Catalog row anchor present |
| F-LP28-MED-002 (AC-16 trace header canonical) | fix-burst-26 | PASS — line 466 verbatim canonical anchor |
| F-LP28-LOW-001 (Token Budget 8→9 BCs) | fix-burst-26 | PASS — Token Budget row shows 9 BCs |
| F-LP28-LOW-003 (ADR-022 in inputs) | fix-burst-26 | PASS — ADR-022 present in inputs frontmatter |
| F-LP27-MED-001 (subsystems [SS-22, SS-17, SS-16]) | fix-burst-25 | PASS — subsystems: [SS-22, SS-17, SS-16] |
| F-LP27-MED-002 (PluginError #[non_exhaustive] unconditional) | fix-burst-25 | PASS — prescription unconditional in §non_exhaustive Requirements |
| F-LP27-MED-003 (§References 7/8 BC titles verbatim) | fix-burst-25 | PASS — 8 anchored BCs verbatim (BC-2.16.002 completeness closed fix-burst-28) |
| F-LP26-MED-001 (BC-2.16.002 body-table title verbatim) | fix-burst-24 | PASS — "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" |
| F-LP25-HIGH-001 (spawn_blocking anchor BC-2.17.005 §Invariants) | fix-burst-23 | PASS — canonical BC-2.17.005 §Invariants anchor |
| F-LP23-HIGH-001 (Vec<String> contract chain) | fix-burst-22 | PASS — AC-17 + Match-Site rows all Vec<String> |
| F-LP22-MED-001 (AC-17 Match-Site Inventory 6 test sites) | fix-burst-21 | PASS — 6 test-crate sites present |
| F-LP21-HIGH-001 (SpecEngineError::TooManyRequests canonical) | fix-burst-20 | PASS — canonical type used throughout |
| F-LP19-MED-001 (§Scope multi-line rejection bullets) | fix-burst-18 | PASS — canonical event_type names present |
| F-LP13-MED-001 (version-pin removed from story) | fix-burst-12 | PASS — no version-pin active-body references |
| F-LP7-HIGH-001 (path mis-anchors) | fix-burst-6 | PASS — canonical /src/ path anchors throughout |

**Phase C: 17/17 PASS — zero regressions from prior fix-bursts.**

---

## POL-22 Phase D — Novel Finding Sweep

Three findings identified in novel search. All three are substantive and actionable.

---

### F-LP31-HIGH-001 — E-PLUGIN-013/014 Error Message Template Three-Site Drift [process-gap]

**Severity:** HIGH
**Tag:** `[process-gap]` — codification candidate #16

**Finding:**

§Error Taxonomy Additions table (story lines 906-907) contains different error message
templates for E-PLUGIN-013 and E-PLUGIN-014 than the canonical sources at AC-5 body
(story lines 322-323) and `error-taxonomy.md` (lines 455-456). The divergence is in
non-title verbatim text — error message strings — not BC title strings. Codifications
#11-#15 sweep BC titles; they do NOT sweep error message template text.

**E-PLUGIN-013 evidence:**

§Error Taxonomy Additions table (DIVERGENT):
> `"Plugin '{path}' manifest missing required allowed_urls field. All plugins must declare their permitted outbound URL hostnames."`

AC-5 body (canonical):
> `"Plugin manifest at '{path}' missing required field 'allowed_urls'; field must be an explicit list (use 'allowed_urls = []' for no URLs)"`

error-taxonomy.md lines 455-456 (canonical):
> `"Plugin manifest at '{path}' missing required field 'allowed_urls'; field must be an explicit list (use 'allowed_urls = []' for no URLs)"`

AC-5 and error-taxonomy.md are VERBATIM with each other. §Error Taxonomy Additions table
DIVERGES at both the message structure (field name quoting, list suggestion) and wording.

**E-PLUGIN-014 evidence:**

§Error Taxonomy Additions table (DIVERGENT):
> `"Plugin '{path}' manifest format_version {n} exceeds supported version {CURRENT_SUPPORTED_VERSION}. Update Prism to load this plugin."`

AC-5 body (canonical):
> `"Plugin manifest at '{path}' format_version {actual} exceeds maximum supported version {supported}"`

error-taxonomy.md lines 455-456 (canonical):
> `"Plugin manifest at '{path}' format_version {actual} exceeds maximum supported version {supported}"`

Again AC-5 and error-taxonomy.md are VERBATIM. §Error Taxonomy Additions table uses different
placeholder names (`{n}` vs `{actual}`; `{CURRENT_SUPPORTED_VERSION}` vs `{supported}`) and
adds an unsolicited second sentence not present in canonical form.

**Consistent entries (PASS):**

E-PLUGIN-015 and E-PLUGIN-016 messages are consistent across all three sites. Only 013 and
014 drift. This is not a systematic format issue — it is specific content drift in two rows
of the §Error Taxonomy Additions table that were apparently not copied verbatim from
error-taxonomy.md.

**Impact:**

The §Error Taxonomy Additions table is the prescriptive reference for what the implementer
adds to error-taxonomy.md. If the implementer follows the story, they will add divergent
message text that conflicts with the canonical AC-5 body text and breaks the consistent
error-message contract. HIGH because it directly affects implementation correctness.

**Routing:** story-writer — align §Error Taxonomy Additions table E-PLUGIN-013/014 message
text to canonical form matching AC-5 body + error-taxonomy.md.

**Codification candidate #16 [process-gap]:** POL-7 verbatim cross-table sweep must extend
to error message template text (non-title content) in addition to BC title strings. The
existing codifications (#12 BC body-table titles, #13 all-citation-site BC titles) cover
title strings only. §Error Taxonomy Additions table is a third category of cross-table
prescriptive text that must be verified verbatim against its canonical source (AC-5 body +
error-taxonomy.md). This represents the same verbatim-drift axis as #12 and #13 but applied
to a different content class.

---

### F-LP31-HIGH-002 — BC-2.17.002 EC-17-007 Cross-Spec Security-Semantic Conflict

**Severity:** HIGH (security-semantic)

**Finding:**

BC-2.17.002 v1.5 Expected Columns table EC-17-007 (line 85 of the BC file) still describes
the pre-AC-7 "open by default" behavior for HTTP requests when no allowlist is configured.
Story AC-7 and AC-17 establish the new `Vec<String>` default-deny semantics. Source-of-Truth
Precedence Rule 1 states: BC supersedes story when the conflict is about contract semantics.
The BC therefore represents the authoritative contract — but the BC text is stale relative
to the implementation contract established in this story.

**BC-2.17.002 v1.5 EC-17-007 (line 85):**
> `Plugin calls 'host::http_request' when no allowlist is configured | Request allowed to any URL (open by default); audit log entry created`

**Story AC-7 contract (canonical):**
`allowed_urls: Vec<String>` — empty `vec![]` means DENY ALL URLs. No implicit allow. The
"open by default" framing is the exact opposite of AC-7's contract.

**Story AC-17 contract (canonical):**
AC-17 Match-Site Inventory confirms 6 test sites use `Vec<String>` (closed by fix-burst-22).
The `HostState::test_default()` signature uses `allowed_urls: Vec<String>`.

**§Obsolete Tests table:**
The story designates `test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed` as
obsolete and provides Option A.ii (rename to `test_BC_2_17_002_ec17_007_http_request_empty_allowlist_blocked`
with inverted assertion) as the migration path. This further confirms that EC-17-007 "allowed
to any URL" has been superseded by default-deny.

**Risk:**

If BC-2.17.002 is not amended before implementation, a security reviewer reading EC-17-007
as the authoritative contract source (per Source-of-Truth Precedence Rule 1) would conclude
that the correct behavior for `allowed_urls: []` is to allow all URLs. This directly inverts
the security property. BC-2.17.002 must be amended to EC-17-007 default-deny semantics in
the same fix-burst as AC-7 lands.

**Routing (two specialists):**
- story-writer: add §BC Amendments directive (if not present) noting BC-2.17.002 EC-17-007
  must be amended in the same fix-burst as AC-7 implementation lands.
- product-owner: amend BC-2.17.002 v1.5 → v1.6 — revise EC-17-007 Expected Columns row
  to read: `Plugin calls 'host::http_request' when no allowlist is configured | Request
  DENIED (default-deny when allowed_urls is empty Vec); error E-PLUGIN-012 returned` (or
  equivalent canonical phrasing consistent with AC-7/AC-17). Update BC-INDEX BC-2.17.002
  row version v1.5 → v1.6.

---

### F-LP31-MED-001 — AC-15 AuthToken Debug-Format Mismatch with Existing Code

**Severity:** MEDIUM (implementer ambiguity, regression risk)

**Finding:**

Story AC-15 (lines 925-927) prescribes the `Debug` format for `AuthToken` as:
> `AuthToken("[REDACTED]")`

The existing production code at
`crates/prism-spec-engine/src/auth_provider.rs:68` implements:
> `f.write_str("AuthToken(<redacted>)")`

The differences are:
1. Case: `[REDACTED]` (uppercase) vs `<redacted>` (lowercase)
2. Bracket style: square brackets `[...]` vs angle brackets `<...>`
3. Inner quotes: story prescription wraps the placeholder in double quotes
   (yielding `"[REDACTED]"` inside the outer format string) while code uses angle brackets
   directly (yielding `<redacted>` without inner quotes)

**Impact:**

The existing code already implements redaction correctly (the security property holds).
The story's prescriptive example would cause an implementer to either:
(a) Change the existing `auth_provider.rs:68` implementation to match the story's form
    `"[REDACTED]"` — which is a REGRESSION (the security property is preserved but the
    format string changes), or
(b) Recognize the mismatch and skip aligning to the prescription — in which case the
    story prescription is misleading.

This is MEDIUM (not HIGH) because the existing code is already compliant with the security
intent (redacted Debug output). The prescription divergence creates implementer ambiguity
but not a compile-breaking or security-breaking defect. However, the prescription must
match the existing code to prevent a regression when an implementer follows it literally.

**Routing:** story-writer — align story AC-15 prescriptive example (lines 925-927) to
match existing code form: `AuthToken(<redacted>)` (lowercase, angle brackets, no inner
double quotes around the placeholder).

---

## Summary and Trajectory Analysis

Pass-31 broke the six-pass decreasing trend (4→1→4→5→1→1→**3**). Three new finding classes
surfaced that were not covered by any of the 15 active codification disciplines:

1. **F-LP31-HIGH-001 [process-gap]**: Error message template text in §Error Taxonomy Additions
   table diverges from canonical sources at AC-5 body and error-taxonomy.md. Codifications
   #12 and #13 sweep BC title strings only — they do NOT sweep non-title prescriptive text.
   Codification candidate #16: extend verbatim cross-table sweep to error message templates.

2. **F-LP31-HIGH-002**: BC-2.17.002 EC-17-007 describes pre-AC-7 "open by default" HTTP
   behavior. AC-7 established default-deny via `Vec<String>`. Stale BC text creates
   security-semantic contradiction under Source-of-Truth Precedence Rule 1. This drift
   survived 30 passes because it is in a BC file (external document), not in the story body
   itself — the story's POL-22 Phase A sweep checked anchor existence, not semantic alignment
   of the referenced content.

3. **F-LP31-MED-001**: AC-15 Debug prescription uses `[REDACTED]` form; existing code uses
   `<redacted>` form. New-axis finding — prior passes verified no fabricated types/anchors
   but did not compare story prescriptive examples against existing code implementations.

**Fix-burst-29 routing:**
- story-writer (parallel): (1) §Error Taxonomy Additions E-PLUGIN-013/014 message text
  corrected to match canonical; (2) AC-15 Debug prescription corrected to match existing
  code; (3) §BC Amendments directive added noting BC-2.17.002 EC-17-007 must be amended.
- product-owner (parallel): BC-2.17.002 v1.5 → v1.6 EC-17-007 default-deny amendment +
  BC-INDEX version bump.
- state-manager: closure burst after both story-writer and product-owner commit.

**Codification candidate #16 [process-gap]**: Tagged F-LP31-HIGH-001. Verbatim cross-table
sweep must extend to error message template text in §Error Taxonomy Additions table and
similar prescriptive tables, not only BC title strings. Session-reviewer adjudicates at
cycle-close along with candidates #11-#15.
