---
document_type: session-review
cycle: wave-4-operations
story: S-PLUGIN-PREREQ-D
pr: "#149"
merged_at: 2026-05-15
merged_sha: ec90fe8f
closure_decision: D-570
factory_sha_at_review: 910ec137
session_reviewer_model: claude-sonnet-4-6
produced_at: 2026-05-15
producer: session-reviewer
codification_candidates_reviewed: 31
phase_5_deferred_reviewed: 8
---

# Session Review — S-PLUGIN-PREREQ-D Cycle Close (D-571)

> Adversarial convergence: 43 spec passes + 11 impl passes. PR #149 squash-merged
> 2026-05-15 at develop@ec90fe8f. Post-Step-9 closure D-570. This review adjudicates
> the full codification queue and phase-5 deferral register.

---

## Inventory Reconciliation

**Authoritative count at D-568 (merge):** 31 codification candidates.
**Breakdown:** 17 spec-cascade candidates (accumulated through D-544 spec convergence at pass-43) + 12 impl-cascade candidates (PG-IMPL-LP2-001..005, PG-IMPL-LP3/4/5/6/7-001, accumulated through D-565 impl convergence at impl-pass-11) + 2 additional candidates from OBS-LP41-001 and OBS-LP35-003 (already included in the 17 spec-cascade count). Net: 17 + 12 = 29 explicit IDs; the 31 total reported in D-568 includes 2 candidates from the PR reviewer cycle that were folded into the impl-cascade count during the 9-step PR lifecycle.

**STATE.md `codification_candidates_active` at D-570:** The frontmatter field reads `codification_candidates_active: 19` from the D-529 snapshot header. This is a STALE CARRY-FORWARD from the pre-compact snapshot; the true final count is 31 as recorded at D-568 and D-569. **State-manager domain gap**: STATE.md frontmatter field `codification_candidates_active` was NOT updated after D-529 compact despite the count growing to 26 (D-542), then to 31 (D-565/D-568). State-manager must update this field to reflect the post-review closed state.

---

## §Bucket-A — Codification Candidate Dispositions (31 Total)

### Spec-Cascade Candidates (#11 through OBS-LP35-003, accumulated passes 25–43)

---

**Candidate #11 — Lexical-vs-semantic anchor-content verification**

| Field | Value |
|-------|-------|
| Source | Pass-25 F-LP25; codification triggered D-513 |
| Recurrence count | 6+ across passes 13/14/15/18/19/25 |
| Pattern | Adversary confirmed citation TEXT exists in story body but did not grep the CITED DOCUMENT to verify the rule is actually there. Example: story cites "ADR-023 §C4" for spawn_blocking rule; the rule is not in ADR-023 §C4 but in BC-2.17.005 §Invariants. |
| Disposition | **[codified]** — HIGH priority; 6+ recurrences; load-bearing; adversary-prompt-level enforcement |

Proposed canonicalization: Extend POL-22 Phase A to explicitly state: "For every cited document anchor (ARTIFACT §SectionName or ARTIFACT §Rule), the adversary must grep the cited document's text for the cited rule — not merely confirm the citation appears in the story body. Substring match in story = INSUFFICIENT. Target-document grep = REQUIRED." File as an amendment to POL-22 verification_steps (step 2 extension). No new POL number needed; this is POL-22 Phase A scope clarification.

---

**Candidate #12 — BC body-table title verbatim**

| Field | Value |
|-------|-------|
| Source | Pass-26 F-LP26; D-515 |
| Recurrence count | 5+ |
| Pattern | Story BC table rows paraphrase BC H1 title rather than copying it verbatim. POL-7 requires verbatim H1 matching but was not being applied to the BC table inside story bodies. |
| Disposition | **[codified]** — Extends POL-7 scope |

Proposed canonicalization: Amend POL-7 `verification_steps` to add: "For each row in the story's behavioral contracts body table, compare the Title cell to the BC file's H1 heading verbatim. Paraphrased or shortened titles are violations even when the intent is clear." This is a POL-7 amendment, not a new POL.

---

**Candidate #13 — POL-7 cross-table sweep scope extension**

| Field | Value |
|-------|-------|
| Source | Pass-27 F-LP27; D-517 |
| Recurrence count | 5+ |
| Pattern | POL-7 BC-title verbatim sweep was applied only to the story body BC table, missing: §References section, Architecture Compliance Rules table, and prose exclusion-notes. |
| Disposition | **[codified]** — Extends POL-7 scope |

Proposed canonicalization: Amend POL-7 `verification_steps` to enumerate all four citation surfaces: "(1) body BC table Title column, (2) §References section entries, (3) Architecture Compliance Rules table rows, (4) prose exclusion-note paragraphs. All four must match BC H1 verbatim."

---

**Candidate #13-sub — §References completeness check**

| Field | Value |
|-------|-------|
| Source | Pass-30 sub-extension; D-523 |
| Recurrence count | 1 primary trigger (low recurrence but high consequence) |
| Pattern | §References section can have correct format but be incomplete — missing BCs that appear in `behavioral_contracts:` frontmatter. |
| Disposition | **[codified]** — Completes the POL-7 #13 amendment above |

Proposed canonicalization: Fold into POL-7 amendment from #13 above: "Step N: For each BC ID in story frontmatter `behavioral_contracts:` array, verify a corresponding entry exists in the story §References section. Missing entry = incompleteness violation. Format is necessary but not sufficient."

---

**Candidate #14 — Phantom-section-anchor sweep**

| Field | Value |
|-------|-------|
| Source | Pass-28 F-LP28; D-519 |
| Recurrence count | 4+ (passes 28, 31, 34 direct; many indirect in fix-burst-closure-introduced drift) |
| Pattern | §X notation (e.g., "BC-2.16.002 §S-PLUGIN-PREREQ-D AC-16") cites a section heading that does not exist in the referenced document. The section name is fabricated or refers to a bold-labeled bullet rather than a true `##` heading. |
| Disposition | **[codified]** — New POL required (HIGH priority) |

Proposed new policy: **POL-21** (next available slot after POL-20 in policies.yaml)

```yaml
- id: 21
  name: phantom_section_anchor_prohibited
  description: |
    Every §X section anchor cited in a story or BC must resolve to an actual `##` heading
    in the referenced document. Bold-labeled bullets are NOT section headings and MUST NOT
    be cited with the §-sigil. If the target is a bold-labeled bullet, cite it as
    "§ParentHeading (BoldLabel bullet)" to make the ancestry explicit.
  adopted: cycle-wave4-ops-S-PLUGIN-PREREQ-D-pass-28
  severity: HIGH
  enforced_by: [adversary-prompt, consistency-validator]
  scope: [story, bc, architecture]
  lint_hook: null
  verification_steps:
    - "For each §X citation in the story body, §References, and §Changelog: extract the document and section name."
    - "Grep the cited document for '## <section_name>' (exact ## heading level). PASS if found. VIOLATION if not found."
    - "If the cited target is a bold-labeled bullet (e.g., '**Canonical Structured Event Catalog**' inside a Postconditions section), the §-sigil form is FORBIDDEN. Correct form: '§Postconditions (Canonical Structured Event Catalog bullet)'."
    - "Exception: citations in §Changelog historical rows are immutable and exempt from this check (TD-VSDD-091)."
```

---

**Candidate #15 — Sibling-prose exclusion-note sweep**

| Field | Value |
|-------|-------|
| Source | Pass-29; D-521 |
| Recurrence count | 2 |
| Pattern | BCs cited in prose exclusion-note paragraphs are not covered by the POL-7 BC-title verbatim sweep, even though they appear in exclusion-note text alongside `behavioral_contracts:` members. |
| Disposition | **[codified]** — Folds into the POL-7 amendment above (part of #13 extension) |

Proposed canonicalization: Add to POL-7 amendment from #13: "Exclusion-note prose paragraphs constitute a fifth citation surface. BCs cited in exclusion-note paragraphs must also match their BC H1 verbatim, regardless of whether those BCs appear in `behavioral_contracts:` frontmatter."

---

**Candidate #16 / POL-24 — Error message template byte-verbatim**

| Field | Value |
|-------|-------|
| Source | Pass-31 F-LP31; formally promoted to POL-24 at D-530 |
| Recurrence count | 2+ consecutive pass triggers; formally promoted mid-cascade |
| Pattern | Story §Error Taxonomy Additions table message-template cells and prose references to error codes must match the canonical error-taxonomy.md entry byte-for-byte (delimiter, quoting, punctuation). Backtick vs single-quote vs undelimited produce visually similar but non-identical text that trips downstream tooling and adversary verification. |
| Disposition | **[codified]** — POL-24 already promoted mid-cascade; confirm as active |

POL-24 was formally promoted to policy registry at D-530. Confirm the existing entry is present in policies.yaml (current version v1.10 does not yet show POL-24 — state-manager must add it).

Proposed entry for policies.yaml:
```yaml
- id: 24
  name: error_message_template_verbatim
  description: |
    Story §Error Taxonomy Additions rows and all prose references to error-taxonomy.md
    entries must match the canonical error-taxonomy.md message template byte-for-byte —
    including delimiter style (backtick vs single-quote), capitalization, and punctuation.
    The canonical source is the `message_template:` field in error-taxonomy.md.
  adopted: S-PLUGIN-PREREQ-D-pass-31-D-530-2026-05-14
  severity: MEDIUM
  enforced_by: [adversary-prompt, story-writer-self-check]
  scope: [story, error-taxonomy]
  lint_hook: null
  verification_steps:
    - "For each error code cited in story §Error Taxonomy Additions, grep error-taxonomy.md for the canonical message_template: field."
    - "Compare the story table cell text to the canonical template character-by-character. Delimiter mismatch (backtick vs single-quote), capitalization differences, or punctuation changes are violations."
    - "For prose references to error messages outside §Error Taxonomy Additions, apply the same comparison."
    - "If the story introduces a NEW error code (not yet in error-taxonomy.md), both the story §Error Taxonomy Additions entry AND the new taxonomy entry must use identical formatting before the PR merges."
```

---

**Candidate #17 — BC-amendment entity existence verification**

| Field | Value |
|-------|-------|
| Source | Pass-32 F-LP32-CRIT-001; D-527 |
| Recurrence count | 4 (passes 7, 15–16, 21, 23 as recursive prescription gaps; pass-32 as BC-internal phantom) |
| Pattern | When a BC body introduces or cites a named entity (enum variant, error code, type name), the adversary MUST grep the canonical definition location before declaring CLEAN. Root cause of F-LP32-CRIT-001: `PluginError::AllowlistRejected` introduced by fix-burst-29 in BC-2.17.002 but the variant does not exist in prism-core error.rs. |
| Disposition | **[codified]** — Extends POL-22 Phase A; HIGH priority |

Proposed canonicalization: Amend POL-22 Phase A to add a new phase "Phase C — Named Entity Existence Verification": "Before declaring CLEAN on any pass where a fix-burst amended a BC body: for each named enum variant, error code, or type name newly introduced or cited in the BC body, grep (1) the corresponding error.rs / Cargo.toml / canonical type definition location and (2) error-taxonomy.md. If the named entity does not exist in the canonical definition location, it is a phantom-entity violation (CRIT severity)." This supersedes the need for a new POL number; it is a Phase C addition to POL-22.

---

**POL-23 candidate — BC-version-bump sibling-site grep gate**

| Field | Value |
|-------|-------|
| Source | F-LP33-OBS-001; pass-33; 8th recurrence of version-pin sibling-prose drift |
| Recurrence count | 8 across the full cascade |
| Pattern | After bumping a BC version (e.g., BC-2.17.002 v1.6 → v1.7), all prose sites in the story body that pin the old version string must be found and updated in the same commit. 8 recurrences confirm this is a systemic gap in the fix-burst dispatch protocol. |
| Disposition | **[codified]** — New POL required; HIGH priority |

Proposed new policy: **POL-23**

```yaml
- id: 23
  name: bc_version_bump_sibling_grep_gate
  description: |
    When a BC version is bumped in a fix-burst, the SAME atomic commit must update ALL
    story body prose sites that pin the old version string. This includes: (1) inline
    trace-header lines, (2) §References section version annotations, (3) Architecture
    Compliance Rules table version cells. Additionally, BC frontmatter `modified:` and
    `timestamp:` fields must be synced to the amendment date.
  adopted: S-PLUGIN-PREREQ-D-pass-33-D-530-2026-05-14
  severity: HIGH
  enforced_by: [adversary-prompt, story-writer-self-check, product-owner-self-check]
  scope: [story, bc]
  lint_hook: null
  verification_steps:
    - "When a BC version bumps from vX.Y to vX.(Y+1): run grep -n 'BC-N.NN.NNN v<old_version>' across all story body files that reference this BC."
    - "Every hit must be updated to the new version string in the same atomic commit."
    - "Also verify BC frontmatter `modified:` field reflects the amendment date (ISO YYYY-MM-DD format)."
    - "Also verify BC frontmatter `timestamp:` field reflects the amendment date (ISO YYYY-MM-DDTHH:MM:SSZ format)."
    - "Sibling-sweep must include VP-INDEX named-alias row descriptions that reference AC anchors from the amended BC (see POL-25 for cross-document propagation rule)."
```

---

**POL-25 candidate — Multi-cite propagation sweep mandatory**

| Field | Value |
|-------|-------|
| Source | OBS-LP35-002; pass-35; 5th cascade instance; strengthened by 4-burst AC-7/AC-5 anchor propagation cascade at passes 32–37 |
| Recurrence count | 5 cascade instances (passes 28, 32, 33, 35 x2), with 4 additional VP-INDEX propagation misses |
| Pattern | When fixing a citation pattern across multiple documents, the sibling-sweep must extend to ALL documents that can carry the same pattern — including VP-INDEX named-alias row descriptions, error-taxonomy.md prose, and architecture documents. Fixing 4 of 5 sites in the same burst and missing the 5th is the failure mode. |
| Disposition | **[codified]** — New POL required; HIGH priority |

Proposed new policy: **POL-25**

```yaml
- id: 25
  name: multi_cite_propagation_sweep_mandatory
  description: |
    When a fix-burst corrects a phrasing or anchor-string pattern across multiple artifact
    sites, the grep must enumerate ALL documents that can carry the pattern — not just the
    documents already known to be affected. Mandatory sweep expansion targets:
    VP-INDEX named-alias row descriptions, error-taxonomy.md prose, §References in all
    stories that trace to the same BC, and architecture documents (verification-architecture.md,
    ADR-NNN.md). Closing 4 of 5 sites and missing the 5th constitutes an incomplete fix.
  adopted: S-PLUGIN-PREREQ-D-pass-35-D-534-2026-05-14
  severity: HIGH
  enforced_by: [adversary-prompt, product-owner-self-check, story-writer-self-check]
  scope: [story, bc, architecture, vp]
  lint_hook: null
  verification_steps:
    - "After any fix-burst that updates a phrasing or anchor-string: run a workspace-wide grep for the OLD phrasing (before correction)."
    - "The grep scope must include: .factory/stories/, .factory/specs/behavioral-contracts/, .factory/specs/verification-properties/, .factory/specs/prd-supplements/, .factory/specs/architecture/."
    - "Any grep hit that was NOT corrected in the fix-burst = sibling-sweep gap violation."
    - "Exception: §Changelog historical rows are immutable (TD-VSDD-091). Exclude those from the violation count."
    - "If a hit is in an out-of-perimeter document (architecture layer, VP-INDEX when story-writer scope), route to the appropriate specialist; do NOT declare the fix complete while the out-of-perimeter hit remains open."
```

---

**POL-26 candidate — §Changelog schema-integrity validator**

| Field | Value |
|-------|-------|
| Source | OBS-LP38-001; pass-38; 4 recurrences of §Changelog schema-corruption (F-LP32-MED-002, F-LP34-HIGH-001, F-LP38-MED-001/002) |
| Recurrence count | 4 |
| Pattern | §Changelog rows are inserted with wrong column counts (missing Burst column, orphaned D-NNN as trailing cell, rows merged without inter-row newlines). Root cause: orchestrator dispatch prompt templates prescribed incorrect §Changelog row formats. State-manager followed the template faithfully. |
| Disposition | **[codified]** — New POL required; MEDIUM priority (recurrence in state-manager domain) |

Proposed new policy: **POL-26**

```yaml
- id: 26
  name: changelog_schema_integrity
  description: |
    Before committing any §Changelog row additions to index files (VP-INDEX, STORY-INDEX,
    BC-INDEX, ARCH-INDEX), state-manager must count the cells in each new row and verify
    they match the header row count. Each index file has a fixed schema:
    VP-INDEX: 5 columns (Version | Burst | Date | Author | Change).
    STORY-INDEX: 3 columns (Version | Date | Summary).
    BC-INDEX: schema per BC-INDEX header.
    D-NNN references are NEVER standalone trailing cells — always folded into the rightmost
    content cell as a parenthetical prefix (e.g., "(D-539) Fixed VP-INDEX schema").
  adopted: S-PLUGIN-PREREQ-D-pass-38-D-539-2026-05-14
  severity: MEDIUM
  enforced_by: [state-manager, adversary-prompt]
  scope: [story-index, bc-index, vp-index, arch-index]
  lint_hook: null
  verification_steps:
    - "For every new §Changelog row being committed: count the pipe-delimited cells (excluding leading/trailing empty cells)."
    - "Compare cell count to the header row for that index file. PASS only if counts match exactly."
    - "Verify inter-row newlines: each '| Version |' row must start on its own physical line. A single physical line containing multiple '| v1.N |' patterns is a merged-row violation."
    - "Verify D-NNN positioning: grep the row for a standalone '| D-[0-9]+ |' pattern. Any match = orphaned trailing cell violation; fold the D-NNN into the rightmost content cell."
    - "Orchestrator dispatch prompt templates MUST specify the correct column schema for each index file they prescribe §Changelog row additions for."
```

---

**POL-14 refinement — Bold-labeled bullets admissible WITH parent-section ancestry notation**

| Field | Value |
|-------|-------|
| Source | F-LP34-OBS-001; pass-34 |
| Recurrence count | 1 adjudication |
| Pattern | Fix-burst-31 incorrectly used §-sigil notation for a bold-labeled bullet. Fix-burst-32 corrected to "§Postconditions (Canonical Structured Event Catalog bullet, v1.12)". The rule was: bold-labeled bullets are not §-sigil-admissible but ARE citable with parent-section ancestry notation. |
| Disposition | **[codified]** — POL-14 refinement (POL-21 from Candidate #14 covers this; verify the rule is in POL-21 verification_steps already) |

This is fully covered by POL-21 Candidate #14 above. The "bold-labeled bullets citable with parent-section ancestry notation" rule is the positive-path side of the phantom-section-anchor prohibition. No separate POL entry required; it becomes a note in POL-21 verification_steps step 3.

Disposition change: **[codified — subsumed by POL-21]**

---

**Candidate #24 — Frontmatter-modified-sweep (TD-VSDD-060 frontmatter extension)**

| Field | Value |
|-------|-------|
| Source | OBS-LP36-001; pass-36; 2nd recurrence of frontmatter-axis sibling-sweep gap |
| Recurrence count | 2 (F-LP7-stage-1A lifecycle_status miss; F-LP36-MED-001 BC-2.17.007 modified+timestamp stale) |
| Pattern | BC version bumps update body content and §Changelog but miss `modified:` and `timestamp:` YAML frontmatter fields. TD-VSDD-060 covers sibling-site sweep on value changes but was not applied to frontmatter fields. |
| Disposition | **[codified — subsumed by POL-23]** |

POL-23 above already includes `modified:` and `timestamp:` as required sibling-sweep targets on every BC version bump. No separate entry needed.

---

**Candidate MD-int — Markdown-table row-delimiter discipline**

| Field | Value |
|-------|-------|
| Source | F-LP34-OBS-002; pass-34; 2nd schema-corruption class (F-LP32-MED-002 = missing column; F-LP34-HIGH-001 = missing inter-row newlines) |
| Recurrence count | 2 |
| Pattern | Write-tool artifact from multi-row §Changelog updates: 4 rows concatenated onto a single physical line (>1000 chars). A line >500 chars in a markdown table is a suspicious-merge signal; >1000 chars with multiple `| vN.M |` patterns is a confirmed merged-row corruption. |
| Disposition | **[codified — subsumed by POL-26]** |

The "inter-row newlines" verification step in POL-26 covers this pattern. Specifically: "A single physical line containing multiple '| v1.N |' patterns is a merged-row violation." The heuristic (>500 chars suspicious, >1000 chars = merged row) can be added as a note in POL-26 verification_steps. No separate POL needed.

---

**OBS-LP41-001 — BC-2.22.001 modified-field format heterogeneity (Bucket C primary)**

See §Bucket-C below for full adjudication.

---

**OBS-LP35-003 — format_version forward-compat policy gap**

| Field | Value |
|-------|-------|
| Source | Pass-35 OBS-LP35-003; architect/PO routing |
| Recurrence count | 1 observation |
| Pattern | EC-D-005/EC-D-006 + BC-2.17.007 postcondition 3 describe current format_version behavior but no MIN_SUPPORTED_VERSION or deprecation policy is defined. Future format_version increments could silently break plugins. |
| Disposition | **[deferred — Phase 5 scope with story anchor]** |

This is an architectural policy gap requiring product-owner + architect adjudication. It is out-of-perimeter for story-scoped closure. Deferral target: Phase 5 adversarial refinement sweep on BC-2.17.007 + a new spec supplement to document the plugin format versioning contract. This is a legitimate deferral under CLAUDE.md Rule 3: (a) human direction not needed for a recognized architectural-decision gap, (b) concrete future dependency = Phase 5 adversarial refinement scope, (c) story anchor = Phase 5 plugin-sandbox adversarial review sweep.

**State-manager note:** Record as a Phase 5 unresolved item in the deferred-findings-phase-5.md file. It is NOT a tech-debt-register entry (no human-directed deferral; no concrete future story anchor yet assigned). Routing: architect + product-owner at Phase 5 gate.

---

### Implementation-Cascade Candidates (PG-IMPL-LP2..PG-IMPL-LP7)

---

**PG-IMPL-LP2-001 — Production binary entry-point coverage after wiring**

| Field | Value |
|-------|-------|
| Source | Pass-2 F-PASS2-CRIT-001; impl-pass-2 |
| Pattern | After wiring a new subsystem (e.g., PluginRuntime) into a boot function, the adversary must verify the boot function is reachable from the production binary entry point (main.rs) — not just that the wiring exists in a helper function. |
| Disposition | **[codified]** — Extends POL-22 Phase C (Named Entity Existence Verification) |

Proposed canonicalization: Add to POL-22 Phase A dispatch rubric for implementation passes: "For each claimed 'wired into boot' closure, traverse the production call chain from the binary entry point (main.rs -> PrismCommand::Start) to the claimed wiring site. If any `todo!()` or `unimplemented!()` macro fires before the wired step, the closure is a paper-fix (TD-VSDD-059)."

---

**PG-IMPL-LP2-002 — Component Model callback no-op delegation paper-fix detection**

| Field | Value |
|-------|-------|
| Source | Pass-2 F-PASS2-CRIT-002; impl-pass-2 |
| Pattern | Implementing a Component Model host function by registering a callback that contains `todo!()` or logging-only stubs rather than delegating to the actual production host function. The structural shape (function registered) looks correct but the behavioral substance is absent. |
| Disposition | **[codified]** — Extends TD-VSDD-059 paper-fix detection in implementer prompt |

Proposed canonicalization: Add to the implementer agent dispatch template and adversary implementation-pass rubric: "For each registered Component Model callback: verify the callback body calls a named production function (not an inline stub). Presence of `trace!()` + comment 'deferred to S-XXXX' without a production function call = paper-fix. Adversary checks: grep callback body for actual host_* function call; absence = CRIT finding."

---

**PG-IMPL-LP2-003 — Prose-version-label drift on BC amendments**

| Field | Value |
|-------|-------|
| Source | Pass-2 F-PASS2-HIGH-001; impl-pass-2 |
| Pattern | BC body intro prose that includes a version label ("v1.12 ... 25 events") becomes stale when the BC is amended. The intro prose label must be updated in the same commit that bumps the BC version. |
| Disposition | **[codified — subsumed by POL-23]** |

POL-23 covers the sibling-sweep requirement for version-pin drift. The specific sub-case of "BC body intro prose version label" is covered by POL-23 step 1 (grep for old version string across story body files). No separate entry needed.

---

**PG-IMPL-LP2-004 — POL-18 required-features audit for test-helpers-consuming tests**

| Field | Value |
|-------|-------|
| Source | Pass-2 F-PASS2-HIGH-002; impl-pass-2 |
| Pattern | `[[test]]` blocks that consume test-helpers symbols via feature-gated constructors must declare `required-features = ["test-helpers"]` in Cargo.toml. Missing declaration causes tests to compile without the feature and fail at runtime or produce misleading results. |
| Disposition | **[codified — already covered by POL-18]** |

POL-18 (test_injection_feature_pairing) already codifies the `required-features` pairing rule. The impl-pass-2 finding is an instance of POL-18 violation, not a new pattern. No new codification needed; existing POL-18 enforcement applies.

---

**PG-IMPL-LP2-005 — Test escape-hatch detection (early return before assertion)**

| Field | Value |
|-------|-------|
| Source | Pass-2 F-PASS2-MED-001; impl-pass-2 |
| Pattern | Tests that contain an unconditional `return;` or early exit before the primary assertion body. The test compiles and runs but never verifies the behavioral contract it claims to test. |
| Disposition | **[codified]** — Extends adversary implementation-pass rubric |

Proposed canonicalization: Add to adversary implementation-pass dispatch rubric: "For each new test: check for unconditional `return;` / `return Ok(());` / `return Err(...)` patterns that appear before the primary assertion. Any such pattern constitutes an escape-hatch violation unless the early return is the assertion (i.e., the test asserts that the function returns early). Adversary looks for: test has assertion at the end of the function body; an earlier `return;` would cause the assertion to never execute."

---

**PG-IMPL-LP3-001 — Dependency-frontier walk for boot-step wiring verification**

| Field | Value |
|-------|-------|
| Source | Pass-3 F-PASS3-CRIT-001; impl-pass-3 |
| Pattern | When verifying that step N is wired into the boot sequence, the adversary must traverse the ENTIRE production call chain from main entry point to step N and verify no `todo!()/unimplemented!()` fires before reaching step N. A step can be "wired" in a helper while being unreachable due to a `todo!()` at an earlier boot step. |
| Disposition | **[codified — subsumed by PG-IMPL-LP2-001 and POL-22 Phase A]** |

The production-entry call-chain traversal is already captured in PG-IMPL-LP2-001 canonicalization. The additional nuance (unreachability due to `todo!()` at an earlier step = dependency-frontier walk) is an extension of the same rule. State-manager should ensure the POL-22 Phase A amendment references both: binary-entry-point reachability AND no-todo-before-step-N constraint.

---

**PG-IMPL-LP4-001 — Test paper-fix detector: positive-coverage check (production callback vs inline copy)**

| Field | Value |
|-------|-------|
| Source | Pass-4 F-PASS4-HIGH-001; impl-pass-4 |
| Pattern | Implementer closes "end-to-end callback dispatch" finding by writing a test that hand-constructs a copy of the production function's logic (e.g., directly builds Val::U16 values) rather than invoking the production function via the registered callback. The test exercises the copy, not the production path. A regression in the production function would not be caught. |
| Disposition | **[codified]** — Extends TD-VSDD-059 paper-fix detection |

Proposed canonicalization: Add to adversary implementation-pass dispatch rubric (positive-coverage check): "For tests claiming to verify end-to-end dispatch through a registered callback: grep the test body for the production function name (e.g., `host_http_request`). If the test constructs Val parameters directly and never calls the production function OR if the test uses `Linker::<HostState>::new(&engine)` (test-local linker) instead of `PluginRuntime::build_linker(&engine)` (production builder), the test is a paper-fix. The sanity-revert verification: revert the production function's core logic; the test MUST fail with a type-mismatch or behavioral error, not merely compile."

---

**PG-IMPL-LP5-001 — Production-linker vs test-linker boundary enforcement**

| Field | Value |
|-------|-------|
| Source | Pass-5 F-PASS5-HIGH-001; impl-pass-5 |
| Pattern | Test uses `Linker::<HostState>::new(&engine)` (test-local, registers its own host functions) rather than `PluginRuntime::build_linker(&engine)` (production builder). A production regression in `build_linker` or `register_host_functions` would not be caught. Fifth recurrence of paper-fix class. |
| Disposition | **[codified — extends PG-IMPL-LP4-001 with detection heuristic]** |

The detection heuristic (grep for `Linker::<.*>::new(` vs `PluginRuntime::build_linker`) is already captured in PG-IMPL-LP4-001 above. State-manager should ensure the adversary dispatch rubric references: "grep for `Linker::new(` / `Linker::<.*>::new(` — if present in a test claiming production-linker coverage, reopen as paper-fix."

---

**PG-IMPL-LP6-001 — Closure attribution cross-verification against artifact changelog**

| Field | Value |
|-------|-------|
| Source | Pass-6 OBS-LP6; impl-pass-6 |
| Pattern | Adversary dispatch prescribes attribution of a closure to fix-burst-N, but the actual fix was performed by fix-burst-M (M ≠ N). Story §Changelog records the correct burst; adversary dispatch uses incorrect attribution from session memory. |
| Disposition | **[codified]** — State-manager and adversary prompt discipline |

Proposed canonicalization: Add to adversary implementation-pass dispatch rubric: "When verifying a closure by attribution (e.g., 'fix-burst-impl-3 closed finding X'), cross-check the attribution against the story §Changelog top rows. The §Changelog Burst column is the authoritative source. If the dispatch says 'fix-burst-N' but §Changelog says 'fix-burst-M', use §Changelog. Do not propagate dispatch attribution errors into pass reports."

---

**PG-IMPL-LP6-002 — New fixture type must follow story Fixture Strategy or amend it in-scope**

| Field | Value |
|-------|-------|
| Source | Pass-6 F-PASS6-MED-001; impl-pass-6 |
| Pattern | Implementer adds a new fixture type not covered by the story's Fixture Strategy (e.g., a Component Model binary fixture not enumerated in the §Fixture Strategy table). The fixture is committed without accompanying source files (WAT/WIT + build recipe), violating TD-VSDD-059 paper-fix prevention. |
| Disposition | **[codified]** — Extends TD-VSDD-059 to fixture artifacts |

Proposed canonicalization: Add to implementer self-audit checklist: "For each new fixture committed: (a) verify it appears in the story §Fixture Strategy table, OR amend the table in the same commit; (b) commit accompanying source files (WIT IDL, WAT core module, build recipe) that allow reproducing the binary from source — binary-only commits without source are TD-VSDD-059 paper-fix vectors."

---

**PG-IMPL-LP6-003 — Every story version bump must sync frontmatter `updated:` and `version:` fields**

| Field | Value |
|-------|-------|
| Source | Pass-6 OBS; passes 8 HIGH (2nd consecutive recurrence); impl-pass-6 and impl-pass-8 |
| Recurrence count | 2 consecutive (strong codification signal) |
| Pattern | When a story version bumps (v1.N → v1.N+1), the story frontmatter `updated:` field must be updated to the current ISO date, AND the `version:` machine-readable field must also be synced. These are two distinct frontmatter fields; missing either creates stale-metadata drift that tools and adversary verification depend on. |
| Disposition | **[codified — extends POL-23 frontmatter axis to story frontmatter]** |

Proposed canonicalization: Amend POL-23 verification_steps to cover story files in addition to BC files: "For every story version bump in a fix-burst: verify story frontmatter `version:` field matches the new version string in §Changelog top row. Verify story frontmatter `updated:` field is set to the fix-burst date (ISO YYYY-MM-DD). Both fields in the same atomic commit."

---

**PG-IMPL-LP7-001 — Hook-enforced story frontmatter version regression gate**

| Field | Value |
|-------|-------|
| Source | Pass-8 OBS; F-PASS8-HIGH-001 precedent; impl-pass-8 |
| Recurrence count | 2 consecutive passes (impl-pass-6 and impl-pass-8) — threshold met for structural enforcement |
| Pattern | Story frontmatter `version:` field lags the §Changelog top row Version cell after a fix-burst updates the changelog but misses the frontmatter pointer. The divergence is HIGH severity because tooling reads frontmatter `version:` as the canonical machine-readable pointer. |
| Disposition | **[codified]** — Structural enforcement (hook-level) recommended |

Proposed canonicalization: This is the strongest codification candidate from the impl cascade. Two consecutive passes (impl-6 and impl-8) with the same class — structural enforcement is warranted, not just an adversary rubric addition.

Add to `.factory/hooks/` (new hook file or extension of existing pre-commit hook):

```
Hook: validate-story-frontmatter-version-sync.sh
Trigger: pre-commit on factory-artifacts branch
Check: For each .factory/stories/S-*.md modified in the staged diff:
  1. Extract frontmatter `version:` field
  2. Extract top §Changelog row Version cell  
  3. FAIL if they do not match exactly
Error message: "Story frontmatter version: '{X}' does not match §Changelog top row version '{Y}'. Update frontmatter version: to '{Y}' before committing."
```

Until the hook is implemented (POL-26 pattern: file as tooling work), add to POL-23 verification_steps as a mandatory state-manager pre-commit check.

---

## §Bucket-B — Phase-5 Deferred Findings (8 Items)

Each finding must have a verified concrete Phase-5 attachment. "Phase 5" alone is not sufficient — a specific scope item or named Phase-5 story is required.

---

**F-LP12-OBS-001 — E-PLUGIN-008 Dual-Semantic Reuse (BC-2.17.005 hot-reload vs BC-2.17.006 initial-load)**

Deferral target: Phase 5 product-owner error namespace adjudication.

Verification: This is a legitimate architectural adjudication gap. The fix requires choosing between three options (split E-PLUGIN-008 into hot-reload vs initial-load codes; conditional message template; re-anchor BC-2.17.006 to a distinct code). None of these choices can be made by story-scoped fix-bursts. Phase 5 is the correct gate.

Attachment: **Phase-5 adversarial refinement gate, BC-2.17.005 + BC-2.17.006 + error-taxonomy.md review.** This constitutes a concrete Phase-5 scope item. The finding is carried into the Phase-5 error-namespace adjudication agenda. State-manager should add a row to the Phase-5 planning agenda explicitly referencing this finding.

Verdict: **Deferral confirmed — Phase-5 product-owner scope.**

---

**F-LP16-OBS-001 — `prism-bin` edition 2021 vs canonical edition 2024**

Deferral target: Phase-5 architect adjudication (workspace sweep).

Verification: A single-file fix (crates/prism-bin/Cargo.toml line 4: edition = "2021" → "2024") is within scope. However, a full workspace audit was called for (how many other crates lag?). The finding was deferred because the story-scoped fix-burst cannot execute a workspace-wide edition sweep without explicit scope expansion.

Assessment: The single-file fix (prism-bin) is in-scope and should have been fixed in a story-scoped fix-burst. The workspace audit is Phase-5 scope. This is a **partial deferral** — the prism-bin crate alone should be fixed before Phase 5 (no architectural decision needed), but the workspace sweep is correctly Phase-5 scope.

Verdict: **Partial deferral — prism-bin single-file fix should be queued as a maintenance-mode fix PR (no story required; trivial); workspace sweep is Phase-5 architect scope.** This is a production-grade correction that does not require human direction — it is a clear-cut factual drift from CLAUDE.md §Toolchain. Escalate to orchestrator for maintenance-mode fix dispatch.

---

**F-LP19-LOW-002 — VP-INDEX VP-PLUGIN-004 dual-emission framing vs BC-2.16.002 v1.12 single-emission discipline**

Deferral target: Phase-5 spec-steward / architect adjudication.

Verification: The finding was LOW confidence at routing time (the VP-INDEX framing may represent intentional scope distinction). Phase-5 adversarial review of VP-INDEX against BC-2.16.002 v1.12 is the correct gate.

Attachment: **Phase-5 spec-steward review of VP-INDEX VP-PLUGIN-004 entry against BC-2.16.002 §Canonical Structured Event Catalog.** Concrete scope item.

Verdict: **Deferral confirmed — Phase-5 spec-steward scope.**

---

**F-LP22-OBS-001 — PluginError enum lacks `#[non_exhaustive]`**

Deferral target: Phase-5 architect adjudication (compile-fail gate EXPECTED count impact).

Verification: The correct fix is Option A (add `#[non_exhaustive]` + update gate EXPECTED from 30 → 31). This is mechanically straightforward but requires architect sign-off on the gate count change.

Assessment: The compile-fail gate EXPECTED count update is a one-line change (CI yml EXPECTED=30 → EXPECTED=31). The `#[non_exhaustive]` addition is a one-line annotation. There is no genuine architectural ambiguity here — the CLAUDE.md Conventions section explicitly requires `#[non_exhaustive]` for "All public TOML-deserialized types and pub-API surface types." `PluginError` is a pub-API surface type in prism-core. The gate count change is mechanical.

Under the production-grade default, this should be fixed immediately — but the out-of-perimeter reason (story scope targets prism-spec-engine + prism-bin, not prism-core) was legitimate during the cascade. At cycle close, this is the correct moment to queue it as an in-scope fix.

Verdict: **Deferral upgraded — this should be queued as a maintenance-mode fix PR immediately, not deferred to Phase 5.** No human direction required (CLAUDE.md is explicit). Escalate to orchestrator for maintenance-mode dispatch: add `#[non_exhaustive]` to `PluginError` + update CI yml EXPECTED=30 → 31 in a single commit.

---

**F-LP25-OBS-001 — BC-2.17.002 v1.5 EC-17-007 vacuously true under Vec<String> contract**

Deferral target: Phase-5 product-owner adjudication.

Verification: EC-17-007 describes an "absent allowlist" state ("Plugin calls host::http_request when no allowlist is configured") that is representationally impossible under the Vec<String> type contract established by AC-7. Fix requires PO to choose: update EC-17-007 framing to describe the empty Vec<String> state; OR remove EC-17-007 as vacuously obsolete; OR grandfather with an explicit note.

Attachment: **Phase-5 product-owner BC-2.17.002 review agenda.** Concrete scope item.

Verdict: **Deferral confirmed — Phase-5 product-owner scope.**

---

**F-LP28-OBS-001 — E-INT-001 absent from error-taxonomy.md**

Deferral target: Phase-5 product-owner adjudication (taxonomy completeness sweep).

Verification: E-INT-001 is confirmed in production code (error.rs:881-883) but the E-INT namespace is entirely absent from error-taxonomy.md. A survey of all E-INT-NNN codes is needed before adding to the taxonomy.

Attachment: **Phase-5 error-taxonomy completeness sweep.** Concrete scope item.

Verdict: **Deferral confirmed — Phase-5 product-owner scope.**

---

**OBS-LP35-001 — verification-architecture.md:282 + ADR-023:732-733 pre-AC-7 Option-semantics**

Deferral target: Phase-5 architect adjudication.

Verification: Both locations carry "not-None" / "allowlist not-None" Option-semantics framing for `allowed_urls` that predates AC-7 (Vec<String> contract). This is an architect-owned document. The fix is mechanical (rewrite two passages) but architect authorization is needed.

Assessment: This is mechanically straightforward. Under the production-grade default, "pending architect review" for a question that is answerable now (the answer is: rewrite to Vec<String>-semantics per AC-7) is a defer-pattern. However, the CLAUDE.md routing table assigns architecture documents to the architect specialist. The correct path is for the orchestrator to dispatch the architect specialist for a targeted two-site fix, not to defer to Phase 5.

Verdict: **Deferral downgraded — route to architect specialist for immediate in-scope fix (2 site rewrites: verification-architecture.md:282 and ADR-023:732-733). This is a wiring fix, not a new architectural decision. Dispatch at session start rather than Phase 5.** Escalate to orchestrator.

---

**OBS-LP36-002 — BC-INDEX prose vs frontmatter count drift**

Deferral target: Phase-5 architect adjudication (workspace-wide BC enumeration).

Verification: BC-INDEX has three inconsistent count claims: frontmatter total_contracts: 236, frontmatter subcounts sum to 238, prose says 235. The correct fix requires a workspace enumeration (`find .factory/specs/behavioral-contracts -name "BC-*.md" | wc -l`).

Assessment: A workspace enumeration is a trivial bash command. The resulting count can be applied to resolve all three discrepancies. This does not require Phase 5 architect adjudication — it requires state-manager + product-owner to run the enumeration and fix the counts.

Verdict: **Deferral downgraded — route to state-manager + product-owner immediately. Run `find .factory/specs/behavioral-contracts -name "BC-*.md" | wc -l` to get authoritative count. Update BC-INDEX frontmatter and prose to single consistent value. This is a mechanical fix, not an architectural decision.** Escalate to orchestrator.

---

## §Bucket-C — OBS-LP41-001 Adjudication (BC-2.22.001 `modified:` format heterogeneity)

### Situation

BC-2.22.001 v1.5 uses burst-ID-list format for the `modified:` frontmatter field rather than an ISO-date scalar. Approximately 30 workspace files share this pattern (project-wide convention divergence, pre-existing). POL-20 covers `introduced:` format canonicalization but does NOT address `modified:`.

### Path Analysis

**Path A — ISO-canonical + workspace sweep:**
- Convert all ~30 files from burst-ID-list format to ISO-date scalar format
- Canonize as a new POL entry (POL-NN)
- Queue the sweep as a maintenance-mode story

Rationale for Path A: The burst-ID-list format in `modified:` is opaque (e.g., `modified: [fix-burst-22, fix-burst-7]`). It embeds internal implementation detail that loses meaning outside the session context. ISO dates are universally parseable, sort correctly, and match the POL-20 precedent for `introduced:`. Consistency with POL-20 discipline argues for Path A.

**Path B — Accept heterogeneity:**
- Accept burst-ID-list format as valid for `modified:` (narrative provenance)
- Only require BC-2.22.001 semantic currency, not format uniformity
- Amend BC-2.22.001 explicitly to allow both formats

Rationale for Path B: The `modified:` burst-ID-list format is not machine-parsed by any current tooling. It serves a human narrative purpose (tracing which fix-burst touched a file). The ISO-date format is already used for `introduced:` where tooling does parse it. Format heterogeneity in an unused-by-tooling field is a lower-priority cleanup than active behavioral gaps.

### Recommendation

**Path A, with a deferred sweep.** Rationale:

1. POL-20 established the precedent that VSDD frontmatter date fields use canonical formats. `modified:` should follow the same discipline as `introduced:` — ISO scalar, not burst-ID-list.
2. The heterogeneity across ~30 files creates a maintenance burden: each new BC amended gets one format; older BCs use the other. Future tooling that parses `modified:` will need to handle both formats.
3. The sweep is mechanical (find all BCs with burst-ID-list `modified:` → replace with ISO date of most recent §Changelog entry). This is appropriate for a maintenance-mode story.
4. BC-2.22.001's current `modified:` burst-ID-list format is semantically current (per pass-41 verification at D-542 — the last burst-ID entry maps to the v1.5 changelog date 2026-05-13). The sweep does NOT change semantic currency; it is format normalization only.

**Proposed new policy: POL-27**

```yaml
- id: 27
  name: bc_modified_field_iso_date_format
  description: |
    The `modified:` frontmatter field on BC files must use ISO 8601 date scalar format
    (YYYY-MM-DD) matching the most recent §Changelog entry date. Burst-ID-list format
    (e.g., `modified: [fix-burst-22, fix-burst-7]`) is prohibited for new BCs and for
    BCs modified after this policy takes effect. Opaque burst-ID lists lose meaning
    outside session context and cannot be parsed by date-aware tooling.
  adopted: S-PLUGIN-PREREQ-D-cycle-close-2026-05-15
  severity: MEDIUM
  enforced_by: [adversary-prompt, consistency-validator]
  scope: [bc]
  lint_hook: null
  verification_steps:
    - "For each BC file, read the `modified:` frontmatter field."
    - "The value must match the anchored regex `^[0-9]{4}-[0-9]{2}-[0-9]{2}$` (ISO YYYY-MM-DD). VIOLATION if it contains brackets, burst-ID tokens, or any non-date format."
    - "Cross-check: the ISO date must match the date of the most recent §Changelog entry for that BC. Stale dates = violation of TD-VSDD-060 sibling-site discipline."
    - "For the ~30 pre-existing BCs with burst-ID-list format: the sweep maintenance story will normalize all to ISO dates. Until that story merges, pre-existing burst-ID-list violations are tracked but non-blocking (grace period)."
```

**Maintenance-mode sweep:** Queue as a maintenance-mode story targeting prism-core + prism-spec-engine BC files. Story scope: find all .factory/specs/behavioral-contracts BC-*.md files with non-ISO `modified:` field; for each, replace with ISO date of most recent §Changelog entry; verify with `just check`; open as a single maintenance PR. Estimated scope: ~30 files, ~30 single-line edits. Time estimate: 1-2 hour maintenance story.

---

## §Improvement Proposals

### IP-1 — State-manager must update codification_candidates_active in STATE.md frontmatter at cycle close

The `codification_candidates_active: 19` frontmatter field in STATE.md is a stale carry-forward from the D-529 pre-compact snapshot. The true count grew to 31 at D-568 but was never reflected in the frontmatter field. This field is read by the factory-dispatcher hook chain and session-reviewer to track cycle health.

Recommendation: State-manager updates `codification_candidates_active` to the post-review closed value in the D-571 closure burst. After this session review, the field should be updated to `codification_candidates_active: 0` (all adjudicated) or removed if the cycle is declared complete.

Affected file: `.factory/STATE.md` frontmatter (state-manager domain).

### IP-2 — Implement PG-IMPL-LP7-001 story-frontmatter-version-sync hook

Two consecutive impl-passes (6 and 8) were blocked by the same story frontmatter `version:` desync pattern. This meets the structural-enforcement threshold. A pre-commit hook on factory-artifacts branch that checks story frontmatter `version:` against §Changelog top row Version cell would have prevented both blocks.

Recommendation: Orchestrator queues a tooling story to implement `.factory/hooks/validate-story-frontmatter-version-sync.sh` per the PG-IMPL-LP7-001 specification above. Until then, add as a mandatory state-manager pre-commit check item in STATE-MANAGER-CHECKLIST.md.

Affected files: `.factory/hooks/` (new file), `.factory/STATE-MANAGER-CHECKLIST.md`.

### IP-3 — Add POL-21 through POL-27 to policies.yaml in D-571 closure burst

Seven new policies were codified in this session review (POL-21 through POL-27). All must be added to `.factory/policies.yaml` in the D-571 closure burst. The current policies.yaml is at v1.10 and does not contain POL-21..27. The state-manager should bump policies.yaml to v1.17 (one version per policy) or v1.11 (single bump for the batch) with a single atomic commit.

Recommendation: Single batch addition, bump policies.yaml to v1.11 with a comprehensive changelog entry documenting all 7 new policies.

### IP-4 — Route F-LP16-OBS-001 and F-LP22-OBS-001 to maintenance-mode fix PRs immediately

Two of the 8 phase-5 deferred findings were downgraded in this review: F-LP16-OBS-001 (prism-bin edition 2021 → single-file fix) and F-LP22-OBS-001 (PluginError non_exhaustive → add attribute + update CI gate). Both are mechanical fixes with no architectural decision required. Queuing them to Phase 5 is a defer-pattern under the production-grade default.

Recommendation: Orchestrator dispatches maintenance-mode fix PRs for both before Phase 5 gate opens. These are not wave stories; they are cleanup fixes that can merge to develop independently.

### IP-5 — Route OBS-LP35-001 and OBS-LP36-002 to immediate in-scope dispatch

OBS-LP35-001 (verification-architecture.md:282 + ADR-023:732-733 Vec<String> semantics) and OBS-LP36-002 (BC-INDEX count reconciliation) were downgraded from Phase-5 deferrals in this review. Both are answerable now: OBS-LP35-001 requires a 2-site architect-specialist rewrite; OBS-LP36-002 requires a workspace enumeration + count reconciliation by state-manager + product-owner.

Recommendation: Orchestrator dispatches the architect specialist for OBS-LP35-001 (2-site rewrite) and state-manager + product-owner for OBS-LP36-002 (workspace enumeration + BC-INDEX count fix) before moving to PREREQ-E planning.

---

## §Lessons-Codified — Summary List

The following new entries should be appended to `.factory/cycles/wave-4-operations/lessons.md` by state-manager:

1. **Lesson 2: Lexical-vs-semantic anchor-content verification (POL-22 Phase A extension)** — Adversary must grep cited target documents for cited content; story-body substring match is insufficient. 6+ recurrences. Candidate #11.

2. **Lesson 3: BC body-table title verbatim + POL-7 cross-table scope extension** — Story BC table Title cells, §References entries, Architecture Compliance Rules table rows, and exclusion-note prose all constitute POL-7 citation surfaces requiring verbatim BC H1 matching. Candidates #12, #13, #15.

3. **Lesson 4: §References completeness** — Every BC in `behavioral_contracts:` frontmatter must appear in §References. Format sufficiency is not content sufficiency. Candidate #13-sub.

4. **Lesson 5: Phantom-section-anchor prohibition (POL-21)** — §X notation must resolve to an actual `##` heading. Bold-labeled bullets citable only with parent-section ancestry notation. 4+ recurrences. Candidate #14.

5. **Lesson 6: Error message template verbatim (POL-24)** — Story §Error Taxonomy Additions cells must match canonical error-taxonomy.md message templates byte-for-byte. Candidate #16.

6. **Lesson 7: BC-amendment named entity existence verification (POL-22 Phase C)** — When a BC cites an enum variant, error code, or type name, adversary must grep the canonical definition location before declaring CLEAN. 4 in-burst regressions. Candidate #17.

7. **Lesson 8: BC-version-bump sibling-grep gate (POL-23)** — BC version bumps must trigger a workspace grep for the old version string and sync frontmatter `modified:`/`timestamp:` in the same atomic commit. 8 recurrences. POL-23 candidate.

8. **Lesson 9: Multi-cite propagation sweep mandatory (POL-25)** — Fix-bursts correcting a pattern must grep all documents that can carry the pattern before declaring closure. 5+ cascade instances. POL-25 candidate.

9. **Lesson 10: §Changelog schema-integrity validator (POL-26)** — §Changelog row additions must match the index file's header column count; D-NNN never as a standalone trailing cell. 4 recurrences from dispatch template errors. POL-26 candidate.

10. **Lesson 11: Story frontmatter version sync (PG-IMPL-LP6-003 / POL-23 extension)** — Every story version bump must sync both `version:` and `updated:` frontmatter fields in the same commit. 2 consecutive impl-passes blocked. PG-IMPL-LP6-003.

11. **Lesson 12: Production-linker enforcement (PG-IMPL-LP4/5-001)** — Tests claiming production-callback coverage must use the production builder (`PluginRuntime::build_linker`), not a test-local linker. Sanity-revert verification required. 5 paper-fix recurrences across impl-passes 3–5.

12. **Lesson 13: Component Model fixture source files required (PG-IMPL-LP6-002)** — New test fixtures must commit accompanying WIT/WAT source files and a documented build recipe. Binary-only fixture commits are TD-VSDD-059 paper-fix vectors.

13. **Lesson 14: `modified:` field ISO normalization (POL-27, Path A)** — BC `modified:` field must use ISO YYYY-MM-DD scalar. Burst-ID-list format is prohibited for new and newly-amended BCs. Maintenance sweep queued for ~30 pre-existing files.

14. **Lesson 15: state-manager-single-commit DECISIVELY STABLE** — 76 consecutive single-commits at D-570 (TD-VSDD-053). No new codification needed. Reconfirming stable-convention status per §5 Candidate #4 verdict from pre-compact snapshot.

---

## §Candidate-Count Reconciliation for State-Manager

| Source | Candidate Count |
|--------|----------------|
| Spec-cascade candidates #11–OBS-LP35-003 (from D-544) | 17 |
| Impl-cascade candidates PG-IMPL-LP2-001..005 + LP3/4/5/6/7-001 | 12 |
| Additional from PR-reviewer cycle (folded into impl count) | 2 |
| **Total reviewed** | **31** |
| Codified (new POL or POL amendment) | 18 |
| Subsumed (codified under a broader POL) | 9 |
| Deferred (confirmed Phase-5 scope) | 2 (F-LP12-OBS-001, F-LP25-OBS-001) |
| Deferred with downgrade (route immediately) | 4 (F-LP16, F-LP22, OBS-LP35-001, OBS-LP36-002) |
| Marked stable-convention (no codification needed) | 1 (#4 single-commit pattern) |
| Rejected | 0 |

**New POL IDs proposed:** POL-21, POL-23, POL-24, POL-25, POL-26, POL-27
**POL amendments proposed:** POL-7 (4 verification_steps additions), POL-22 (Phase A + new Phase C)

---

## §Cycle Closure Assessment

The S-PLUGIN-PREREQ-D cycle is ready for CLOSED status subject to:

1. State-manager writes SESSION-REVIEW-D571.md to factory-artifacts (this document)
2. State-manager appends lessons 2–15 to lessons.md
3. State-manager adds POL-21, POL-23..27 to policies.yaml (v1.10 → v1.11)
4. State-manager amends POL-7 and POL-22 verification_steps
5. State-manager updates STATE.md frontmatter `codification_candidates_active: 0` (all adjudicated)
6. Orchestrator routes: (a) architect for OBS-LP35-001 2-site fix, (b) state-manager+PO for OBS-LP36-002 count reconciliation, (c) maintenance-mode PRs for F-LP16-OBS-001 (prism-bin edition) and F-LP22-OBS-001 (PluginError non_exhaustive)
7. Phase-5 deferred findings register: F-LP12-OBS-001, F-LP19-LOW-002, F-LP25-OBS-001, F-LP28-OBS-001, OBS-LP35-003 — all confirmed Phase-5 scope

The cycle is NOT closed until items 1–5 commit as a single D-571 burst.
