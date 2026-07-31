---
document_type: adversarial-review
pass: 72
scope: local
perimeter: wave-a-spec-evolution
frozen_head: "9091fa16c"
develop_head_noted: "aa2a5fe6e"
timestamp: 2026-07-30T00:00:00Z
producer: adversary
CLEAN_strict: "no"
CLEAN_PR_merge: "no"
finding_counts: {HIGH: 4, MED: 5, LOW: 2, OBS: 3, total: 14}
novelty: HIGH
---

# Adversarial Review — LOCAL Pass 72 (Wave-A spec-evolution)

```
CLEAN (strict):   no      # 14 findings of any severity
CLEAN (PR-merge): no      # 4 HIGH + 5 MED present
```

**BC-5.39.001 streak effect:** RESET to 0/3.

> **TD-VSDD-091 transcription note:** The adversary's process-gap recommendation referenced a proposed records-lint check by a two-digit label not yet in `L9_CHECK_NAME_EXEMPT`. Per TD-VSDD-091 binding on staged `.factory/` additions, that bare label is replaced below with a function description carrying identical semantic content. No finding content, severity, routing, or count is altered.

---

## Critical Findings

None.

---

## High Findings

### F-WASE-P72-HIGH-001 — Two parallel, unreconciled required-filter mechanisms; `armis_device_activity.device_id` is annotated with the option BC-2.11.007 defines as "not mandatory"

- **Severity:** HIGH
- **Artifacts / anchors:** `ADR-057 §D7` ("Decision: `required_filters` field on `[[tables.steps]]`", "Why this mechanism over the alternatives"), `ADR-057 §D5` ("Step block (MUST)" + `device_id` column block); `BC-2.02.014 §Preconditions`, `§TOML Contract`; `S-WAVE-A-ARMIS-ACTIVITY-001 §AC-002`; `BC-2.11.007 §Column Options table`, `§REQUIRED Column Runtime Mechanism`, `§Invariants (INV-REQUIRED-SPECDRIVEN)`, `§Error Cases (E-QUERY-009)`.
- **Source-of-truth checked against:** `BC-2.11.007` (the artifact ADR-057 §D5 and BC-2.02.014 §Preconditions both name as the taxonomy authority) and `crates/prism-query/src/pushdown.rs §column push-down option priority block`.
- **LIVE vs changelog occurrences:** LIVE — `ADR-057 §D7` alternatives block (1), `ADR-057 §D5` `device_id` column block (1), `BC-2.02.014 §Preconditions` (1), `BC-2.02.014 §TOML Contract` TOML block `options = ["INDEX"]` (1), `S-WAVE-A-ARMIS-ACTIVITY-001 §AC-002` + `§T-IMPL-01` TOML block + `§Architecture Compliance Rules` item 4 (3). Changelog/immutable: not counted.
- **Why it is a defect:**
  `BC-2.11.007 §Column Options table` defines a three-value taxonomy that ADR-057 §D5, BC-2.02.014 §Preconditions, and story AC-002 all cite verbatim as "(REQUIRED / INDEX / ADDITIONAL)". In that same table:
  - `REQUIRED` = "The sensor API **requires** this parameter; queries cannot execute without it … Query rejected with `E-QUERY-009` if column is not constrained in WHERE clause. **Rejection occurs before any API calls.**"
  - `INDEX` = "supports this as a native filter parameter … Adapter SHOULD use this constraint … **Improves performance but is not mandatory.**"

  `BC-2.11.007 §REQUIRED Column Runtime Mechanism` further binds REQUIRED to DI-021, to `ColumnOptions::Required` in `prism_core::column` as the "single source of truth for REQUIRED classification — not a hardcoded column-name list", and to `INV-REQUIRED-SPECDRIVEN`. `ColumnOptions::Required` is live in `crates/prism-query/src/pushdown.rs` (priority-ordered `Required > Index > Additional > Optimized > Default` branch).

  Against that ground truth, ADR-057 §D7 ratifies a **second, new, parallel** required-filter mechanism — a step-level `required_filters: Vec<String>` on `FetchStep`, gated in `execute_impl §execute_impl` inside `prism-spec-engine`, with a **different error code** (`E-SPEC-029`) and a **different variant** (`SpecEngineError::HttpRequestFailed`, `status_code = 0`) — for behaviour BC-2.11.007 already contracts. §D7's "Why this mechanism over the alternatives" analyses exactly three rejected alternatives; **none of them is the existing `options = ["REQUIRED"]` / DI-021 / `E-QUERY-009` path**. The one alternative it does analyse rejects column-`options`-based detection on the grounds that "`INDEX` means 'push-down eligible,' not 'required'" — which is correct about INDEX and silent about the fact that the same enum carries a value that means precisely "required".

  Sharpest consequence: the `armis_device_activity` `device_id` column is now specified — in ADR-057 §D5, BC-2.02.014 §TOML Contract, and story T-IMPL-01 — as `options = ["INDEX"]`, i.e. carrying the annotation BC-2.11.007 defines as "**not mandatory**", on a surface whose own `BC-2.02.014 §Invariants` filter-required invariant states "queries without this predicate are errors, not empty result sets". The TOML annotation contradicts the surface's own invariant, and no artifact reconciles the two mechanisms or specifies precedence when both could fire.
- **Proposed routing:** `architect` (ADR-057 §D7 amendment: adjudicate `options = ["REQUIRED"]` / DI-021 / E-QUERY-009 as an alternative, and state precedence + error-surface unification), then `product-owner` (BC-2.02.014 / BC-2.11.007 reconciliation) and `story-writer` (AC-002 / T-IMPL-01).

### F-WASE-P72-HIGH-002 — Empty-value required-filter arm exists in ADR-057 §D7 and error-taxonomy E-SPEC-029 but is absent from every BC-2.02.014 and story site; no AC or Red Gate test covers `WHERE device_id = ''`

- **Severity:** HIGH
- **Artifacts / anchors:** `ADR-057 §D7` Rule 2; `error-taxonomy.md §SPEC table` E-SPEC-029 row; `BC-2.02.014 §Postconditions` (absent-filter bullet), `§Error Cases` row 1, `§Edge Cases EC-014-001`, `§Canonical Test Vectors TV-BC-2.02.014-002`, `§TOML Contract` required-filter obligation paragraph; `S-WAVE-A-ARMIS-ACTIVITY-001 §AC-004`, `§Edge Cases EC-001`, `§Tasks RG-004`, `§T-IMPL-02`.
- **Source-of-truth checked against:** `ADR-057 §D7` Rule 2 (architecture authority for the gate) and `error-taxonomy.md §SPEC table` E-SPEC-029 (POL-24 message-template carrier).
- **LIVE vs changelog occurrences:** Arm PRESENT at 2 LIVE upstream sites — ADR-057 §D7 Rule 2 ("if `FetchContext.query_filters` does NOT contain that key **or contains it as an empty string**") and E-SPEC-029 Description ("the caller's query **omits or provides an empty value** for one of the required filter keys"). Arm ABSENT at 4 LIVE BC-2.02.014 sites and 4 LIVE story sites, all of which condition solely on absence ("filter absent from query (no `WHERE device_id = '...'` predicate)", "has no matching entry", "not provided"). Changelog rows not counted.
- **Why it is a defect:** The empty-value arm is not decorative — it is the arm that closes the actual defect. A query `SELECT * FROM armis_device_activity WHERE device_id = ''` pushes an empty string into `FetchContext.query_filters["device_id"]`, so the absence-only gate specified in BC-2.02.014 and in story T-IMPL-02 does **not** fire; interpolation then yields the same malformed path `/api/v1/devices//activity` and the same HTTP 200 + `activities: [], total: 0` silent empty result that BC-2.02.014 §Error Cases exists to eliminate (Standing Rule 3 §2 / SOUL.md #4). Because `tdd_mode: strict` binds the test-writer to the enumerated AC/RG list, and neither AC-004 nor RG-004 mentions the empty-value input, the arm will receive no Red Gate coverage and the defect ships behind a green gate. This is a POL-29 9b failure (downstream copy target not swept in the burst that authored §D7 Rule 2) compounded by a POL-38-class obligation gap (a normative arm with no AC + RGT anchor).
- **Proposed routing:** `product-owner` (add the empty-value arm to BC-2.02.014 §Postconditions / §Error Cases row 1 / EC-014-001 / a new TV; POL-24 byte-identity against E-SPEC-029), then `story-writer` (extend AC-004 + RG-004, or add AC-009/RG-009).

### F-WASE-P72-HIGH-003 — `S-MAINT-ADR-ANCHOR-GATE-001` targets a phantom ADR directory at 3 LIVE sites; a gate built to that path is functionally inert (false-green generator)

- **Severity:** HIGH
- **Artifacts / anchors:** `S-MAINT-ADR-ANCHOR-GATE-001 §Narrative`, `§Acceptance Criteria AC-001`, `§Architecture Mapping` (ADR-files row).
- **Source-of-truth checked against:** filesystem — `Glob('/Users/jmagady/Dev/prism/.factory/specs/architecture/adr/**')` returns **no files**; every ADR resides under `.factory/specs/architecture/decisions/`. `scripts/records-lint.sh` (the sibling mechanical gate) correctly uses `decisions/` in both its L10 ADR resolution message and its L10 self-probe fixture path.
- **LIVE vs changelog occurrences:** 3 LIVE sites, all in this story's body; 0 changelog occurrences. A `.factory`-wide grep for `specs/architecture/adr/` returns 5 files, of which 3 are append-only historical pass reports / session-task records and 1 is an out-of-perimeter day2-UI doc — this story is the **only live normative carrier**.
- **Why it is a defect:** AC-001 is the story's primary normative requirement: "MUST emit a hard-block error for any ADR file in `.factory/specs/architecture/adr/` whose frontmatter does NOT contain an `anchor_stories:` key". An implementer following the AC literally builds a scanner over an empty (non-existent) directory. The scanner finds zero ADRs, emits zero violations, and **exits 0** — reported as "gate passes". This is exactly the CI-as-code anti-pattern the positive-coverage axis exists to catch: exit code 0 with no runtime-computed non-zero item count is indistinguishable from a functionally-inert check. The gate's whole purpose is to prevent SAC-2 drift; built to this path it would certify the corpus clean forever. Per the mis-anchoring rubric this class of anchor error "would mislead an implementer into building the wrong thing"; it blocks convergence regardless of the story's `draft` status (POL-4).
- **Additional obligation:** AC-001/AC-002 as written also have no positive-coverage assertion requirement. The corrective edit should require the gate to emit a runtime-computed count of ADR files actually scanned (e.g. `Check passed: N ADR files validated`, N non-zero), so a future path regression fails loud rather than green.
- **Proposed routing:** `story-writer` (correct all 3 sites to `.factory/specs/architecture/decisions/`; add a positive-coverage-assertion clause to AC-001).

### F-WASE-P72-HIGH-004 — BC-INDEX attributes finding `F-WASE-P71-HIGH-005` (two-`ArmisState` construction for AC-016/RG-016) to BC-2.02.014 at 2 LIVE sites; BC-2.02.014's own changelog records no such change

- **Severity:** HIGH
- **Artifacts / anchors:** `BC-INDEX.md` frontmatter most-recent `# NOTE:` row (D-2069, FB100/FB102 closure); `BC-INDEX.md` BC-2.02.014 registry row, trailing provenance cell.
- **Source-of-truth checked against:** `BC-2.02.014 §Changelog` top row (FB100) — enumerates exactly `F-WASE-P71-HIGH-001` (12 live pin sites), `F-WASE-P71-HIGH-001-site-11` (content falsification), `F-WASE-P71-MED-004` (blockquote supersession marker), `F-WASE-P71-MED-006` (§Architecture Anchors §D7). No HIGH-005, no `ArmisState`, no AC-016/RG-016. The `BC-INDEX` BC-2.02.006 registry row's own newest entry correctly records HIGH-005 two-`ArmisState` construction for AC-016/RG-016 — that is the artifact the finding landed in.
- **LIVE vs changelog occurrences:** 2 LIVE sites (both in BC-INDEX: the frontmatter NOTE and the registry-row provenance cell). Both are current-state ledger text, not archived rows.
- **Why it is a defect:** POL-37 (`ledger_from_frontmatter`) requires state-manager to generate every ledger row by reading each touched file's on-disk changelog after specialist edits land — never from a dispatch brief or an author's description. Here the FB100 dispatch touched BC-2.02.006 **and** BC-2.02.014, and the BC-2.02.006 finding was written into both rows. The result is a falsified audit trail: BC-2.02.014 is recorded as having received a contract change (an AC-016/RG-016 two-`ArmisState` construction obligation) that does not exist in it. AC-016/RG-016 belong to `S-WAVE-A-ARMIS-SPEC-001`, which is not even in BC-2.02.014's traceability. This is a recurrence of a pattern the index itself already documents once — the BC-2.01.016 registry row carries an inline `[CORRECTION …: the above … description is PHANTOM … a falsified ledger entry]` note for the same class — which makes this the second recorded instance of cross-artifact ledger attribution drift in the same index. It is also the precise defect class that records-lint L10 is documented as unable to detect (version-number half only; "a 0-mismatch result is NOT equivalent to every index row verified").
- **Proposed routing:** `state-manager` (correct both LIVE sites; do not rewrite the immutable BC-2.02.006 row).

---

## Medium Findings

### F-WASE-P72-MED-001 — BC-INDEX BC-2.02.014 row cites a phantom `ADR-057 §D8` and misstates the FB100 sweep target

- **Severity:** MEDIUM
- **Artifacts / anchors:** `BC-INDEX.md` BC-2.02.014 registry row, trailing provenance cell — "12 stale ADR-057 v0.7 pins swept to **§D7/§D8** section anchors per POL-39".
- **Source-of-truth checked against:** `ADR-057` body — section headings are `§D1` … `§D7` plus `§Consequences`; there is no `§D8`. `BC-2.02.014 §Changelog` FB100 row enumerates the actual sweep targets: `ADR-057 §D6`, `§D5` (×3), `§D4` (×4), `§Consequences (C1)`, and an added `§D7` citation in §Architecture Anchors.
- **LIVE vs changelog occurrences:** 1 LIVE site. The BC-INDEX frontmatter NOTE for the same burst does not carry the `§D7/§D8` phrasing, so the error is confined to the registry cell.
- **Why it is a defect:** POL-21 (`phantom_section_anchor_prohibited`) — every `§X` anchor must resolve to an actual heading. `§D8` resolves to nothing. Independently, the description is wrong on the merits: the FB100 sweep replaced version-pinned cites with **version-free** `§D4`/`§D5`/`§D6`/`§Consequences (C1)` anchors; characterising it as a sweep "to §D7/§D8" misdirects any future reader auditing the POL-39 discharge.
- **Proposed routing:** `state-manager`.

### F-WASE-P72-MED-002 — ARCH-INDEX ADR-057 row attributes the E-SPEC-029 registration to FB94 at 2 LIVE sites; ground truth is FB95

- **Severity:** MEDIUM
- **Artifacts / anchors:** `ARCH-INDEX.md` ADR-057 registry row — the `v0.8 FB94` clause ("canonical error E-SPEC-029 registered") and the `v1.0 FB99` clause ("E-SPEC-029 registration obligation completed in FB94 same commit").
- **Source-of-truth checked against:** `ADR-057 §D7 §Canonical error variant` — "**E-SPEC-029 REGISTERED (FB95 2026-07-30)**"; `ADR-057 §Consequences C7` — "DISCHARGED FB95 2026-07-30"; `ADR-057 §Changelog` v0.8/FB94 row — "**C7 added: product-owner MUST register E-SPEC-NNN** in `error-taxonomy.md`" (i.e. at FB94 the code was an open obligation, not registered); `error-taxonomy.md §Changelog` top row — "FB95 … E-SPEC-029 registered".
- **LIVE vs changelog occurrences:** 2 LIVE sites, both inside the ARCH-INDEX registry row (current-state ledger). ARCH-INDEX `§Changelog` row for D-2069 correctly says only "ADR-057 pin v0.9→v1.0" and does not repeat the mis-attribution.
- **Why it is a defect:** POL-37. The row records the error-code registration as having happened one burst earlier than it did, and the `v0.8 FB94` clause additionally contradicts the ADR's own v0.8 changelog row, which is the primary evidence that FB94 *created* the obligation. Any future audit of the C7 discharge chain reading ARCH-INDEX first will conclude FB94 was self-contained when in fact the obligation spanned FB94 → FB95 → FB99.
- **Proposed routing:** `state-manager`.

### F-WASE-P72-MED-003 — `error-taxonomy.md` E-SPEC-029 asserts present-tense emission for code that does not exist, while its two immediate siblings carry explicit `[PLANNED]` markers

- **Severity:** MEDIUM
- **Artifacts / anchors:** `error-taxonomy.md §SPEC table` E-SPEC-029 row (Description opening: "Emitted by `execute_impl §execute_impl` in `prism-spec-engine` when a `[[tables.steps]]` block declares `required_filters = ["..."]` … and the caller's query omits or provides an empty value"); compare `E-SPEC-027` and `E-SPEC-028` rows, both of which open "Emitted by `SpecLoader::parse()` Rule 9/10 **[PLANNED — Wave-A engine story per ADR-053 D2/ADR-054 D10]**".
- **Source-of-truth checked against:** codebase — `Grep('required_filters', crates/)` returns **0 occurrences**; `Grep('E-SPEC-029|ESpec029', crates/)` returns **0 occurrences**. `ADR-057 §D7` Rule 1 confirms the field is still to be *added* ("Add `#[serde(default)] pub required_filters: Vec<String>` to `FetchStep §FetchStep`"), and `S-WAVE-A-ARMIS-ACTIVITY-001 §T-IMPL-02` is unimplemented.
- **LIVE vs changelog occurrences:** 1 LIVE site (the E-SPEC-029 row). The `error-taxonomy.md §Changelog` FB95 row is immutable and not counted.
- **Why it is a defect:** POL-29 9a sibling-pair asymmetry. E-SPEC-027 and E-SPEC-028 sit in the same table, in the same Wave-A spec-first cohort, describing the same class of not-yet-shipped validation, and both received `[PLANNED]` markers specifically to stop readers treating spec-first rows as shipped behaviour. E-SPEC-029 was authored after that convention was established and did not receive it. The absence of the string in the sibling is the failure mode, not evidence of cleanliness. Concretely: a reader or a live-audit script consuming the E-SPEC-029 row will believe `required_filters` is a supported TOML key today.
- **Proposed routing:** `product-owner`.

### F-WASE-P72-MED-004 — Partial-fix regression: `S-WAVE-A-ARMIS-ACTIVITY-001 §AC-005` still carries `"dev-001"` inside an AC whose query literal is `'d-001'`

- **Severity:** MEDIUM
- **Artifacts / anchors:** `S-WAVE-A-ARMIS-ACTIVITY-001 §AC-005` — the AC statement uses `WHERE device_id = 'd-001'`, and enumerated step 1 of the same AC reads `Pre-seed step_vars["query.filter.device_id"] = "dev-001"`.
- **Source-of-truth checked against:** `crates/prism-dtu-armis/fixtures/device-activity.json` — the fixture device IDs are `d-001, d-002, d-005, d-013, d-015, d-020, d-023, d-024`; there is no `dev-001`. `BC-2.02.014 §Canonical Test Vectors TV-BC-2.02.014-001` uses `d-001`. `S-WAVE-A-ARMIS-ACTIVITY-001 §Changelog` FB96 row claims closure of `F-WASE-P70-MED-003` as "AC-005 `dev-001`→`d-001`; RG-005 `"dev-001"`→`"d-001"`; RG-006 `"dev-001"`→`"d-001"`, **3 sites**".
- **LIVE vs changelog occurrences:** 1 residual LIVE site (AC-005 step 1). RG-005 and RG-006 are correctly on `d-001`. The FB96 changelog row is immutable.
- **Why it is a defect:** This is the S-7.01 partial-fix class in its purest form: the burst that claimed a 3-site sweep landed 2 of 3, and the surviving site is inside the AC body itself, creating a self-contradicting acceptance criterion (query for `d-001`, assert the pre-seed equals `dev-001`). A test-writer implementing RG-005 from AC-005 verbatim writes an assertion that cannot pass. `dev-001` is additionally a generated-style identifier prefix, so the residue actively points at the wrong fixture universe.
- **Proposed routing:** `story-writer`.

### F-WASE-P72-MED-005 — Two BC-INDEX registry rows carry 7 cells against a 6-column header; GFM discards the excess, making the entire cell-7 provenance history unrenderable

- **Severity:** MEDIUM
- **Artifacts / anchors:** `BC-INDEX.md §Behavioral Contract Index` table header (`| BC ID | Title | Subsystem | CAP | Priority | Status |` — 6 columns); the `BC-2.02.013` row and the `BC-2.02.014` row, each terminating with an additional `| … |` cell after the Status cell.
- **Source-of-truth checked against:** the table's own header row (6 columns), and `BC-INDEX.md` frontmatter `# NOTE:` row for D-2065/FB88, which records the 7-cell shape as **deliberate**: "BC-2.02.013 was 6-cell, now 7-cell with correct history".
- **LIVE vs changelog occurrences:** 2 LIVE rows, verified by direct read of the contiguous registry block spanning `BC-2.01.001` through `BC-2.03.002`. I did **not** enumerate cell counts across the remaining ~230 registry rows — see Counts NOT Verified.
- **Why it is a defect:** GitHub-Flavored Markdown specifies that when a body row has more cells than the header row, **the excess cells are ignored**. The FB88 remediation therefore relocated BC-2.02.013's D-1795 provenance history, and appended BC-2.02.014's full version-history narrative, into a cell that no renderer displays. The content is present in the raw file and invisible in every rendered view, including the GitHub view a reviewer or auditor uses. A repair intended to restore an audit trail instead placed it beyond reach; and because the row still parses without error, no gate flags it. Either the header gains a documented 7th column (`History`) for all rows, or the history text moves inside the Status cell as the other 200+ rows do.
- **Proposed routing:** `state-manager`.

---

## Low Findings

### F-WASE-P72-LOW-001 — `S-MAINT-ADR-ANCHOR-GATE-001` waives its SAC-1 Red Gate enumeration on a rationale its own sibling refutes *(pending intent verification)*

- **Severity:** LOW
- **Artifacts / anchors:** `S-MAINT-ADR-ANCHOR-GATE-001 §Tasks §Red Gate tests` ("N/A — this story's deliverables are a validator script/hook and an upstream issue … The story is `tdd_mode: strict` but its implementation is a shell/WASM script, not a Rust crate") and its `**Red Gate density check** (BC-5.38.001)` paragraph ("**0 pre-written named tests** … Density check **deferred** to implementation-time pre-pass (standard pattern for tooling stories where the test vehicle is not yet determined)").
- **Source-of-truth checked against:** sibling `S-MAINT-L11-GATE-001` — same track (Platform Engineering), same `status: draft`, same `tdd_mode: strict`, same `behavioral_contracts: []`, same shell-script deliverable (`scripts/records-lint.sh`) — which enumerates `RG-001` … `RG-007` with named test functions (`test_l11_violation_artifact_id_vN`, `test_l11_clean_no_version_pin`, `test_l11_exempt_changelog_section`, `test_l11_exempt_frontmatter_version_field`, `test_l11_exempt_index_tier_filename`, `test_l11_fires_real_factory_worktree`, `test_l11_full_scan_includes_claude_md`) plus a "7 Red Gate tests" density paragraph. `S-MAINT-ANTIPIN-SWEEP-001` and `-002` and `S-MAINT-CAPREF-SWEEP-001` likewise all carry density-check paragraphs.
- **LIVE vs changelog occurrences:** 1 LIVE site (the §Tasks Red Gate block). SAC-1's obligation attaches "before it reaches `status: ready`", so this is not yet a hard SAC-1 breach — hence LOW.
- **Why it is a defect:** The stated rationale — "the test vehicle is not yet determined" for tooling stories — is empirically refuted by four sibling stories in the same maintenance cohort that determined their test vehicle at draft time. This is a defer-pattern rationalization of the kind the Canonical Principle names explicitly ("standard pattern for …" as a substitute for doing the work). Because the adversary cannot adjudicate authorial intent, tagging LOW and pending intent verification; the orchestrator or human should decide whether this story is genuinely exempt or whether the four siblings set the binding precedent.
- **Proposed routing:** `story-writer` (after orchestrator/human adjudication).

### F-WASE-P72-LOW-002 — `T-IMPL-04(b)` directs an edit to `new_with_seed`, which contains no `activity_fixture` assignment; and "all three construction paths" understates the constructor set in `clone.rs`

- **Severity:** LOW
- **Artifacts / anchors:** `S-WAVE-A-ARMIS-ACTIVITY-001 §T-IMPL-04(b)` ("In ALL THREE construction paths in `clone.rs` — `new_with_seed §new_with_seed` (4-line delegation …), `new_with_seed_anchored §new_with_seed_anchored`, and `new_with_scenario §new_with_scenario` — populate `state.activity_fixture` from BOTH …"); `§File Structure Requirements` `clone.rs` row; `BC-2.02.014 §Description` ("all three construction paths reside in `clone.rs`") and `§TOML Contract` generated-records obligation.
- **Source-of-truth checked against:** `crates/prism-dtu-armis/src/clone.rs` — **five** constructors load the `device-activity` fixture via `prism_dtu_common::load_fixture_as(crate_dir, "device-activity")`: `ArmisClone::new §new`, `ArmisClone::new_with_org §new_with_org`, `new_with_seed_anchored §new_with_seed_anchored`, `new_with_scenario §new_with_scenario`, and `new_with_seed §new_with_seed` only transitively (its body is a single delegating call to `new_with_seed_anchored §new_with_seed_anchored` and contains no fixture load and no `activity_fixture` assignment).
- **LIVE vs changelog occurrences:** 1 LIVE site in the story (T-IMPL-04(b)) plus 1 supporting LIVE site in the §FSR `clone.rs` row; 2 LIVE sites in BC-2.02.014 (§Description, §TOML Contract obligation). Changelog rows in both artifacts not counted.
- **Why it is a defect:** Instructing the implementer to populate `state.activity_fixture` "in ALL THREE" paths including `new_with_seed` is unactionable at that site — there is nothing there to modify, and an implementer who "completes" the instruction has either edited nothing or duplicated the delegation. Separately, the unqualified claim "all three construction paths reside in `clone.rs`" is an exhaustiveness assertion that is false of `clone.rs` as a whole; it is true only under the unstated scope restriction "the `#[cfg(feature = "fixture-gen")]` seeded paths". Since `new §new` and `new_with_org §new_with_org` set `fixture_gen_seeded = false`, the seeded-mode scoping is defensible on the merits — hence LOW rather than MED — but the wording should state the scope rather than assert exhaustiveness.
- **Proposed routing:** `story-writer` (T-IMPL-04(b) / §FSR), `product-owner` (BC-2.02.014 §Description scope qualifier).

---

## Observations

### F-WASE-P72-OBS-001 — Neither BC-2.02.014 nor `AC-008`/`RG-008` says which generated device-identity key the activity builder must key on; `build_asset` emits three, one of which is an integer for exactly the index RG-008 selects

- **Artifacts / anchors:** `BC-2.02.014 §Edge Cases EC-014-006`, `§Error Cases` data-gap row, `§TOML Contract` generated-records obligation; `S-WAVE-A-ARMIS-ACTIVITY-001 §AC-008`, `§Tasks RG-008`, `§T-IMPL-04(a)`; `BC-2.02.014 §Canonical Test Vectors TV-BC-2.02.014-005` (`device_id = 'dev-slug-1-0'`).
- **Source-of-truth checked against:** `crates/prism-dtu-armis/src/generator.rs §build_asset` — its doc comment and emitted object show `id` is an **integer** when `id_index.is_multiple_of(5)` (per EC-001 / BC-3.4.002) and the `dev-{slug}-{seed}-{i}` string otherwise, while `asset_id` and `device_id` are **always** the string form.
- **Why worth noting:** All spec sites say only "keyed to generated device IDs (`dev-{org_slug}-{seed}-{i}` from `generator.rs §build_asset`)". That phrase is true of `asset_id` and `device_id` and false of `id` for one record in five. RG-008 and TV-BC-2.02.014-005 both select index `0`, which **is** a multiple of 5 — the single index in five where the two identity families diverge. An implementer keying the new activity builder on `device_id` or `asset_id` satisfies RG-008; one keying on `id` produces a record whose `device_id` is an integer-derived value and RG-008's `dev-<slug>-<seed>-0` literal never matches. Naming the source field explicitly in `§T-IMPL-04(a)` removes the ambiguity at zero cost. Not filed as MED because the field the activity surface actually joins on (`ActivityRecord.device_id`) has an unambiguous string counterpart in `build_asset`'s emission.
- **Proposed routing:** `product-owner` (BC-2.02.014 EC-014-006 / obligation paragraph), `story-writer` (T-IMPL-04(a)).

### F-WASE-P72-OBS-002 — `E-SPEC-029` message template says "not provided" but the same row fires it for a present-but-empty value

- **Artifacts / anchors:** `error-taxonomy.md §SPEC table` E-SPEC-029 — `message_template: "required filter '{key}' not provided; query requires WHERE {key} = '...' predicate"` against the same row's Description ("omits **or provides an empty value**") and `ADR-057 §D7` Rule 2 ("does NOT contain that key (**or contains it as an empty string**)").
- **Why worth noting:** For the empty-value arm the agent-facing message asserts something false — the filter *was* provided, with an empty value — and the remediation hint ("query requires `WHERE {key} = '...'` predicate") does not tell the agent that its existing predicate is the problem. Since these outputs are consumed by an LLM agent, a message that misdescribes the input state is a reasoning hazard, not a cosmetic one. A second template arm (or a `{reason}` substitution distinguishing absent from empty) resolves it. POL-24 byte-identity means any change must land atomically across `error-taxonomy.md §SPEC table`, `ADR-057 §D7`, and the BC-2.02.014 sites named in F-WASE-P72-HIGH-002.
- **Proposed routing:** `product-owner`.

### F-WASE-P72-OBS-003 — `[process-gap]` The ADR-anchor gate story has no edge case for `document_type: adr-amendment`, and the one such file in the ADR directory carries no `anchor_stories` key

- **Artifacts / anchors:** `S-MAINT-ADR-ANCHOR-GATE-001 §Edge Cases` (EC-001 … EC-005 — covers `status: superseded`, multi-story citation, malformed YAML, changelog-only citation, `behavioral_contracts: []` + §Authority; **no** case for non-`adr` document types); `ADR-026-AMENDMENT-rule-c-keyring-scope.md` frontmatter (`document_type: adr-amendment`, `amends: ADR-026`, no `anchor_stories:` key).
- **Source-of-truth checked against:** `Grep('^anchor_stories:', .factory/specs/architecture/decisions)` matched 39 files; `ADR-026-AMENDMENT-rule-c-keyring-scope.md` is not among them. Its `document_type` is `adr-amendment`, distinct from the `adr` type SAC-2 addresses.
- **Why worth noting:** As specified, AC-001 hard-blocks "any ADR file … whose frontmatter does NOT contain an `anchor_stories:` key". The amendment file lives in the ADR directory and would be hard-blocked, even though SAC-2's obligation is written for ADRs and an amendment inherits its parent's anchor set. Conversely, if the implementer filters on `document_type: adr`, the file is silently skipped and no artifact records that decision. The gate story must decide: amendments inherit the parent's `anchor_stories` and are skipped (documented as an EC), or they carry their own key. Tagged `[process-gap]` because the gap is in the gate specification, not in any single artifact's content. I make **no** corpus-wide SAC-2 claim — see Counts NOT Verified.
- **Proposed routing:** `story-writer` (add the EC), with `architect` adjudicating the inherit-vs-own-key question if the answer is not mechanical.

---

## Probe Verdicts

| Probe | Verdict | Basis and scope of the claim |
|---|---|---|
| **SAP-1** — tracing emission catalog completeness | **SCOPED — NOT RUN as a corpus sweep.** No finding. | The Wave-A spec-evolution perimeter contains **zero merged `crates/` deliverables**: every perimeter story is `draft` except `S-WAVE-A-ARMIS-ACTIVITY-001` (`ready`), and none has merged. I read `BC-2.02.014` and `ADR-057` in full and confirm **neither contracts any new `event_type` emission**, so the perimeter introduces no new catalog obligation. I did **NOT** run `rg 'event_type\s*=' crates/ --type rust`, did **NOT** enumerate the Canonical Structured Event Catalog in `BC-2.16.002 §Postconditions`, and make **no** assertion about pre-existing emission/catalog parity. |
| **SAP-2** — DTU↔TOML schema parity | **PASS for the `armis_device_activity` surface, with one reachability defect already contracted.** | Read the wire-emission site directly: `crates/prism-dtu-armis/src/routes/devices.rs §get_device_activity` returns `(StatusCode::OK, Json(body)).into_response()` with `body = ActivityResponse { data: ActivityData { activities, total } }`. Read `crates/prism-dtu-armis/src/types.rs §ActivityRecord` — five fields (`activity_id: String`, `device_id: String`, `activity_type: String`, `timestamp: String`, `details: serde_json::Value`). **All five are emitted** (whole-struct `Json` serialization; no envelope-level field omission) → **zero P1 CRITICAL** at the emission site. **Rule 6 dual-path check:** `§get_device_activity` is single-path — it filters `state.activity_fixture` only and has no generated-records branch; the branch's *absence* is the MED-005 defect already contracted at `BC-2.02.014 §Edge Cases EC-014-006` / `AC-008` / `RG-008`, not a clearance. **Rule 2 datetime pairing:** `timestamp` is declared `column_type = "string"` mapping to a `String` wire field — no `Datetime` column is declared on this table, so the chrono-vs-`timestamp_formats` pairing rule does not apply; no false finding minted. **Independently verified:** fixture device IDs in `crates/prism-dtu-armis/fixtures/device-activity.json` are exactly `d-001, d-002, d-005, d-013, d-015, d-020, d-023, d-024` (8, matching the BC and story enumerations verbatim); generated IDs from `generator.rs §build_asset` are `dev-{org_slug}-{seed}-{i}` on `asset_id`/`device_id` — **disjoint from the fixture set, zero overlap, confirmed**; `generator.rs` has **no** activity-record builder. **Also independently confirmed** the five `BC-2.02.006 §Generated-Records Path Coverage` absence claims against `generator.rs §build_asset`'s emitted object: `site` PRESENT (via `format!`), `os_version` ABSENT, `risk_factors` ABSENT, `network_id` ABSENT (only `zone: null`), `tags` ABSENT, `device_cves` ABSENT. **Scope limit:** I did not audit the Cyberint or CrowdStrike TOML/DTU pairs, nor the `armis_devices`/`armis_alerts` column sets beyond those six fields. |
| **SAP-3** — spec-arm reachability | **FAIL — one uncovered arm.** | `BC-2.02.014` claims six EC rows (EC-014-001 … EC-014-006) and five TVs; `S-WAVE-A-ARMIS-ACTIVITY-001` maps AC-001…AC-008 one-to-one onto RG-001…RG-008, and RG-004/RG-005/RG-007/RG-008 are specified as end-to-end pipeline/PrismQL-surface tests (not synthetic-AST), which satisfies the probe for the arms they cover. **The gap:** the required-filter **empty-value** arm — normative in `ADR-057 §D7` Rule 2 and in `error-taxonomy.md §SPEC table` E-SPEC-029 — has **no EC row, no TV, no AC, and no RG test**, and is reachable from the public surface via `WHERE device_id = ''`. Filed as F-WASE-P72-HIGH-002. |
| **SAC-1** — enumerated Red Gate list on `tdd_mode: strict` stories | **CONDITIONAL PASS, one draft outlier.** | Verified per-file: `S-WAVE-A-ARMIS-ACTIVITY-001` — RG-001…RG-008 named, "8 failing tests" density paragraph present, all Red Gate tasks precede all `T-IMPL-*` tasks under an explicit "to be executed ONLY AFTER all RG tests are authored and failing" heading → **compliant**. `S-MAINT-L11-GATE-001` (RG-001…RG-007), `S-MAINT-CAPREF-SWEEP-001` (3 RGs), `S-MAINT-ANTIPIN-SWEEP-001` (2 RGs), `S-MAINT-ANTIPIN-SWEEP-002` (2 RGs) — all carry named RGs + density paragraphs. `S-MAINT-ADR-ANCHOR-GATE-001` — **0 named tests, density check explicitly deferred** (F-WASE-P72-LOW-001); not a hard breach because SAC-1 attaches at `status: ready` and the story is `draft`. **Scope limit:** I did not open the bodies of `S-WAVE-A-ENGINE-001`, `-MCP-001`, `-CYBERINT-SPEC-001`, `-CYBERINT-PATCH-001`, `-ARMIS-REMEDIATION-001`, `-ARMIS-SPEC-001`, `S-ADR054-WAVE-A-001`, or `S-ADR055-WAVE-A-001`, so I make no SAC-1 claim for those eight. |
| **SAC-2** — ADR `anchor_stories` from `§Authority` ground truth | **PASS for ADR-057, the only ADR whose body I read in full; corpus claim NOT made.** | `ADR-057` frontmatter carries `anchor_stories: [S-WAVE-A-ARMIS-ACTIVITY-001]` with a per-entry verification annotation, and I confirmed the ground truth bidirectionally: `S-WAVE-A-ARMIS-ACTIVITY-001 §Authority` opens "**ADR-057** (accepted 2026-07-27) is the authoritative design decision …". The ARCH-INDEX rows for ADR-050…ADR-056 each record SAC-2 verification/promotion states, but I did **not** independently re-derive them from the eight ADR bodies plus the cited stories' `§Authority` sections. A grep for `^anchor_stories:` matched 39 files under `specs/architecture/decisions/`; I did not classify the non-matching files by `document_type`, so I make **no** claim about how many are ADRs missing the key. One concrete non-match verified: `ADR-026-AMENDMENT-rule-c-keyring-scope.md` (`document_type: adr-amendment`) has no key → F-WASE-P72-OBS-003. |

### Supplementary axis results (verified clean — recorded so a later pass need not re-derive)

- **POL-13 story-frontmatter ↔ STORY-INDEX status/version consistency:** CLEAN across the 14 perimeter registry rows. Spot-verified frontmatter against index for `S-WAVE-A-ARMIS-ACTIVITY-001` (ready v1.9 ↔ ready v1.9), `S-WAVE-A-ARMIS-SPEC-001` (draft v1.9 ↔ draft v1.9), `S-WAVE-A-ENGINE-001` (draft v3.0 ↔ draft v3.0), `S-WAVE-A-CYBERINT-SPEC-001` (draft v1.8 ↔ draft v1.8), `S-WAVE-A-MCP-001` (draft v1.5 ↔ draft v1.5), `S-MAINT-CAPREF-SWEEP-001` (draft v1.0 ↔ draft v1.0), `S-MAINT-ADR-ANCHOR-GATE-001` (draft v0.1 ↔ draft v0.1), `S-MAINT-L11-GATE-001` / `-ANTIPIN-SWEEP-001` / `-002` (draft v1.1 ↔ draft v1.1 each). No mismatch found.
- **Dependency-graph bidirectional consistency (maintenance cohort):** CLEAN and acyclic. `S-MAINT-L11-GATE-001` `depends_on: []`, `blocks: [CAPREF-SWEEP-001, ANTIPIN-SWEEP-001, ANTIPIN-SWEEP-002]`; `CAPREF-SWEEP-001` `depends_on: [L11-GATE-001]`, `blocks: [ANTIPIN-SWEEP-001]`; `ANTIPIN-SWEEP-001` `depends_on: [L11-GATE-001, CAPREF-SWEEP-001]`, `blocks: [ANTIPIN-SWEEP-002]`; `ANTIPIN-SWEEP-002` `depends_on: [L11-GATE-001, ANTIPIN-SWEEP-001]`, `blocks: []`. Every edge is declared from both ends. I specifically probed for a false-green risk here — both ANTIPIN stories make `records-lint.sh --full-scan` L11 their primary mechanical gate, and L11 is **not implemented** (`scripts/records-lint.sh` implements L1/L7/L9/L10; `L11` appears only in the `L9_CHECK_NAME_EXEMPT` token list and as self-probe *input text*). The correctly-declared `depends_on: [S-MAINT-L11-GATE-001]` on all three consumers closes that vector. **No finding.**
- **POL-39 status claim in the supplied rubric:** ACCURATE. `records-lint-L11` is genuinely proposed-not-deployed; I confirmed no `run_l11` function exists in `scripts/records-lint.sh` after PR #231. No stale-policy-status finding.
- **POL-9 VP propagation (VP-153 / VP-159 / VP-160 / VP-161):** CLEAN for these four. Each appears in `VP-INDEX.md`, in `verification-architecture.md §Provable Properties Catalog`, and in `verification-coverage-matrix.md §Coverage by Module` (`prism-spec-engine` row) with matching module, tool, and priority. L10 pin sync verified against on-disk frontmatter: VP-153 index `v0.28` ↔ file `0.28`; VP-159 `v1.27` ↔ `1.27`; VP-160 `v1.3` ↔ `1.3`; VP-161 `v1.3` ↔ `1.3`. **Scope limit:** per-tool and total VP arithmetic NOT recomputed — see below.
- **Named-entity existence checks (POL-22 Phase C) that passed:** `ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md` exists at the path `BC-2.02.014 §Architecture Anchors` cites. `BC-2.11.007` and `BC-2.11.001` both exist, and `BC-2.11.007 §Column Options table` genuinely declares the `REQUIRED / INDEX / ADDITIONAL` taxonomy that ADR-057 §D5 and BC-2.02.014 §Preconditions attribute to it — the anchor is semantically correct (it is what the taxonomy *says* that produces F-WASE-P72-HIGH-001, not a bad anchor). `ADR-057 §D7` Rule 1's two load-bearing claims are both true: `FetchStep` carries `#[non_exhaustive]` in `crates/prism-spec-engine/src/spec_parser.rs`, and `"FetchStep"` is present in `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` — so the "no new `EXPECTED_SYMBOLS` entry required" instruction repeated in `S-WAVE-A-ARMIS-ACTIVITY-001 §T-IMPL-02` and `§File Structure Requirements` is **correct**. `error-taxonomy.md §SPEC table` E-SPEC-029 exists and is reachable from `ADR-057 §D7`/`§C7`, `BC-2.02.014`, and the story.
- **POL-39 candidate that is NOT a violation:** `S-WAVE-A-ARMIS-SPEC-001 §Previous Story Intelligence` retains one artifact version pin in past-tense prose ("this distinction **was corrected in** BC-2.02.006 v1.8"). `S-MAINT-ANTIPIN-SWEEP-001 §Acceptance Criteria AC-003`, `§Edge Cases EC-001`, and `EC-005` name this exact site as the canonical **retained** case — correctly-scoped historical prose per the FB86 adjudication. Correctly retained; **no finding**. Recorded here so a future pass does not re-mint it.
- **Grandfathered, not filed:** `BC-2.01.006` and `BC-2.01.008` carry `timestamp:` values without the `Z` suffix. Both were introduced in `cycle-1` (2026-04-14), predating the POL-23 step-4 P2-02 Direction A adjudication, which binds **new** BCs. Not a defect.

---

## Counts NOT Verified (do not treat as established)

Every number below appears in a perimeter artifact and I did **not** independently recompute it from the corpus. Where I checked only internal consistency, I say so explicitly.

1. `BC-INDEX.md` frontmatter `total_contracts: 269`, `active_contracts: 251`, `draft_contracts: 5`, `removed_contracts: 7`, `retired_contracts: 6`. I verified only that the stated identity `251 + 5 + 0 + 7 + 6 = 269` is **internally arithmetically consistent**. I did **not** enumerate BC files or read their `lifecycle_status` frontmatter.
2. `VP-INDEX.md` total VP count and the per-tool sums (kani / proptest / fuzz / integration). Not recomputed; the VP-INDEX self-consistency axis (total = sum of per-tool = actual row count) was **NOT** exercised this pass.
3. `verification-coverage-matrix.md §Coverage by Module` per-module counts for `prism-spec-engine` (the row I read shows `4 | 15 | 4 | 1 | 11 | 85%`) and the per-tool column sums across all module rows. Not recomputed; the "sum module rows per tool column must equal VP-INDEX per-tool totals" check was **NOT** performed.
4. The Canonical Structured Event Catalog row count in `BC-2.16.002 §Postconditions` (recorded elsewhere as 90 after a 92→90 removal). Not counted; catalog not read.
5. The number of BC-INDEX registry rows carrying more cells than the 6-column header. I verified 2 (BC-2.02.013, BC-2.02.014) by direct read of the contiguous block from `BC-2.01.001` to `BC-2.03.002`; the remaining ~230 rows were **not** cell-counted. F-WASE-P72-MED-005 claims exactly 2 confirmed, not 2 total.
6. The "~2,813 pins" corpus figure in `S-MAINT-ANTIPIN-SWEEP-001 §Work-List Methodology`, and the "83 files" figure in its `§Architecture Mapping`. Neither recomputed.
7. The `records-lint.sh` self-probe pass count (reported as 38/38 after PR #231) and the L10 "434 of 496 BC-INDEX rows unverifiable" capability-boundary figure. I read the script's structure and confirmed which checks exist (L1/L7/L9/L10, no L11); I did **not** execute it — I have no Bash access — and I did not count L10-parsable rows.
8. Corpus-wide SAC-2 coverage. A grep for `^anchor_stories:` matched **39 files** under `specs/architecture/decisions/`. I did **not** classify the non-matching files by `document_type`, so I assert **no** count of ADRs missing the key. The single non-match I confirmed and classified is `ADR-026-AMENDMENT-rule-c-keyring-scope.md` (`document_type: adr-amendment`).
9. Total `E-SPEC-NNN` code count, and whether any code besides E-SPEC-029 lacks a `[PLANNED]` marker while being unimplemented. I verified the marker state of exactly three rows (E-SPEC-027, E-SPEC-028, E-SPEC-029) and the codebase absence of exactly two symbols (`required_filters`, `E-SPEC-029`/`ESpec029`).
10. `S-WAVE-A-ENGINE-001` AC/RGT counts (28 ACs / 40 RGTs per the STORY-INDEX row). Not verified against the story body — I read only that story's frontmatter.

### Perimeter artifacts NOT reviewed this pass (Level-2 partial-output disclosure)

Bodies not opened: `BC-2.16.009`, `BC-2.16.008`, `BC-2.16.014`, `BC-2.16.002`, `BC-2.01.016`, `BC-2.01.017`, `BC-2.02.004`, and `BC-2.02.006` (for BC-2.02.006 I read only the `BC-INDEX` row plus verified its five generated-path absence claims against `generator.rs`); `ADR-026`, `ADR-028`, `ADR-051`, `ADR-052`, `ADR-053`, `ADR-054`, `ADR-055`, `ADR-056`; `VP-153`, `VP-159`, `VP-160`, `VP-161` bodies (frontmatter + three index rows only); `domain-spec/invariants.md` (so the **Invariant-to-BC Orphan Detection axis was NOT run** — no DI-NNN extraction, no orphan sweep); stories `S-WAVE-A-ENGINE-001`, `-MCP-001`, `-CYBERINT-SPEC-001`, `-CYBERINT-PATCH-001`, `-ARMIS-REMEDIATION-001`, `S-ADR054-WAVE-A-001`, `S-ADR055-WAVE-A-001`; and `policies.yaml` — I applied the policy rubric as supplied in the dispatch and did **not** read the file for full `verification_steps`. The **BC H1 ↔ BC-INDEX title sync** axis was exercised on only 4 BCs (BC-2.01.006, BC-2.01.008, BC-2.01.018, BC-2.02.014 — all four match their registry titles verbatim), well short of the 10-BC sample the axis specifies. The **Story Frontmatter-Body Coherence** axis was fully exercised on 1 story (`S-WAVE-A-ARMIS-ACTIVITY-001`: `behavioral_contracts: [BC-2.02.006, BC-2.02.014]` — both appear as rows in the body `§Behavioral Contracts` table with titles matching their BC H1 headings, and both are referenced in AC trace lines; bidirectionally clean), short of the 5-story sample.

---

## Novelty Assessment

**Novelty: HIGH.**

Two findings open **structurally new axes** that no amount of re-reading the same artifacts against the same checklist would surface:

- **F-WASE-P72-HIGH-001** required reading `BC-2.11.007` — an artifact that `ADR-057 §D5` and `BC-2.02.014 §Preconditions` both *cite by name* while neither ADR-057's alternatives analysis nor any BC reconciles what it actually says. The finding is not that an anchor is wrong; it is that the anchor is **right** and its content contradicts the decision that cites it. A citation-resolution check passes here; only reading the cited table's semantics finds it.
- **F-WASE-P72-HIGH-002** came from diffing an arm across four artifacts in the authority chain (ADR → error-taxonomy → BC → story) rather than validating each artifact against its own upstream. The arm is present at both ends of the chain's head and absent through the entire tail — a shape that per-artifact review is structurally blind to.

Two more are **new instances of established patterns, in new locations**: the ledger-falsification class (F-WASE-P72-HIGH-004, MED-001, MED-002) has a documented prior instance inside BC-INDEX itself — the BC-2.01.016 row carries an inline `falsified ledger entry` correction — yet three fresh instances landed in the two most recent index bursts across two different index files. That recurrence pattern, not any single row, is the signal.

**F-WASE-P72-MED-005** (7-cell rows in a 6-column table) is novel in a different sense: it is a defect *created by a prior remediation*, invisible to every version-pin and content-text gate, and detectable only by counting delimiters against the header. The FB88 note shows the 7-cell shape was authored deliberately, which means no reviewer to date has evaluated it against Markdown's excess-cell rule.

Findings are **not** refinements of wording, formatting, or style. The perimeter has **not** converged.

### Process-gap follow-ups for the Cycle-Closing Checklist

- `[process-gap]` **F-WASE-P72-OBS-003** — the ADR-anchor gate specification has no disposition for `document_type: adr-amendment` files residing in the ADR directory.
- `[process-gap]` **Recurrence flag on F-WASE-P72-HIGH-004 / MED-001 / MED-002.** Three ledger-content falsifications in `BC-INDEX.md` and `ARCH-INDEX.md` landed in the two most recent index bursts, against a prior documented instance in the same BC-INDEX file. POL-37 states the correct procedure (generate every ledger row from on-disk frontmatter and changelog after specialist edits land) and is being satisfied in the version-number dimension while failing in the **content** dimension — precisely the boundary `TD-VSDD-092 §L10 capability boundary` documents as mechanically undetectable ("L10 cannot detect content falsification — a row describing a change that does not exist in the target artifact, which is the defect class that originally triggered this check"). This meets the 3-recurrence codification threshold. Recommend a mechanical records-lint ledger-citation cross-reference check: for each index ledger row asserting `<ARTIFACT> vX→vY (<finding-ids>)`, assert each cited finding ID appears in that artifact's own `§Changelog` row for version `Y`. That check would have caught all three findings above and the prior BC-2.01.016 instance.
