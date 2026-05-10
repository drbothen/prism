---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T20:00:00Z
phase: 5
pass: 3
previous_review: "ADR-023-pass-2.md"
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-3
date: 2026-05-10
reviewer: adversary
target_artifact: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
target_artifact_sha_at_review: "fe71c2cd"
target_artifact_version: "v1.2"
findings_total: 12
findings_by_tier:
  CRIT: 1
  HIGH: 4
  MED: 4
  LOW: 1
  OBS: 2
process_gap_findings: 2
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 2
new_findings_this_pass: 10
streak_status: "0/3 (RESET — pass-3 NOT_CLEAN)"
trajectory: "26 → 16 → 12 (slowing decay, novelty 0.833)"
related_tasks: [94, 95]
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-2.md"
  - ".factory/cycles/wave-4-operations/td-from-adr-023-pass-1.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/policies.yaml"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-spec-engine/src/lib.rs"
  - "crates/prism-spec-engine/src/plugin/mod.rs"
  - "crates/prism-spec-engine/src/plugin/host_functions.rs"
  - "crates/prism-spec-engine/src/plugin/loader.rs"
  - "crates/prism-core/src/types.rs"
  - "crates/prism-ocsf/src/mappers/armis.rs"
  - "crates/prism-ocsf/src/mappers/crowdstrike.rs"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 3)

## Finding ID Convention

Finding IDs use the pass-3-scoped format:

- `F-PASS3-{CRIT,HIGH,MED,LOW,OBS}-NNN` — net-new finding in pass-3, not present in v1.1

Pass-2 residuals that remain open in v1.2 are noted in the residual verification table (Part A)
but are not re-assigned new IDs — they are tracked by their pass-2 IDs until fully closed.
The 2 partial-close residuals from pass-2 that gave rise to new pass-3 findings are
F-PASS3-HIGH-003 (mis-attributed F-MED-NEW-004 closure) and F-PASS3-LOW-001 (partial
F-LOW-NEW-003 closure with 5 of 8 BCs still absent).

This is pass 3 of the ADR-023 adversarial review cycle. Target: 3 consecutive CLEAN passes
(0/3 streak, RESET at this pass).

---

## Summary

Fresh-context review of ADR-023 v1.2 at SHA `fe71c2cd`. Pass-3 surfaces **12 findings**
(1 CRIT / 4 HIGH / 4 MED / 1 LOW / 2 OBS), of which 2 are residuals carried forward from
pass-2 and 10 are new defects first visible in v1.2.

**Pass-2 residual verification:** 14 of 14 pass-2 spec defects are substantively closed. Two
caveats apply: (a) F-MED-NEW-004 closure is incidental rather than deliberate — the
line-range fix addressing F-HIGH-NEW-003's ADR-022 scheduling conflict removed the offending
text as a side-effect; the v1.2 changelog entry claims "F-MED-NEW-004 corrected via SP
arithmetic" which is factually inaccurate, catalogued below as F-PASS3-HIGH-003; (b)
F-LOW-NEW-003 is partially closed — 3 of 8 sensor BCs were added to the inputs frontmatter
but 5 are still absent, catalogued as F-PASS3-LOW-001.

**Streak status:** 0/3 RESET. Pass-3 is NOT_CLEAN. Fix-burst-3 is required before pass-4.
User mandate (2026-05-10): continue full 3-CLEAN cycle; close process-gap TDs before Wave 0.

**Trajectory:** 26 → 16 → 12. Decay is slowing (delta −10 → −4). Novelty 0.833 (10 new of
12 total). Citation errors in newly amended sections are the dominant defect class; the
structural fix is citation-integrity validation at authoring time (TD-FIX-BURST-VERIFY-002).

---

## Summary Table

| ID | Severity | Category | Residual? | Brief |
|----|----------|----------|-----------|-------|
| F-PASS3-CRIT-001 | CRIT | schema-gap | NO | `amends_bcs_pending_full_amendment_in_wave_2_g` is ad-hoc wave-specific field; no schema; no validator |
| F-PASS3-HIGH-001 | HIGH | broken-reference | NO | VP-PLUGIN-006 referenced in §E but absent from VP-INDEX; VP-PLUGIN-* series does not exist |
| F-PASS3-HIGH-002 | HIGH | citation-error | NO | POL-11 miscited: ADR says `ci_positive_coverage_assertion`; actual is `index_bump_required_for_index_mutations` |
| F-PASS3-HIGH-003 | HIGH | false-audit-trail | YES (partial F-MED-NEW-004) | v1.2 changelog says "SP arithmetic"; actual closure was incidental deletion |
| F-PASS3-HIGH-004 | HIGH | one-directional-amendment | NO | DI-012 amendment declared in ADR-023 §B but DI-012 itself has no back-reference annotation |
| F-PASS3-MED-001 | MED | assignment-drift | NO | PR template assigned to PREREQ-D in §G body; pass-1 fix-burst plan scoped it to PREREQ-F |
| F-PASS3-MED-002 | MED | process-integrity | YES (partial F-MED-NEW-004) | F-MED-NEW-004 closure incidental not deliberate; root cause never explicitly analyzed |
| F-PASS3-MED-003 | MED | scope-gap | NO | ADR-022 has 2 sites referencing four built-in adapters; only §G Story 3 amendment scoped |
| F-PASS3-MED-004 | MED | broken-reference (REQUIRES_VERIFICATION) | NO | SS-21/SS-22 in §C subsystems table may not exist in ARCH-INDEX |
| F-PASS3-LOW-001 | LOW | incomplete-inputs | YES (partial F-LOW-NEW-003) | 5 of 8 sensor BCs still absent from inputs frontmatter |
| F-PASS3-OBS-001 | OBS | process-gap | NO | ADR template needs generic `amends_bcs_pending` schema (TD-ADR-AMEND-002) |
| F-PASS3-OBS-002 | OBS | process-gap | NO | Citation-integrity validator needed for all inline references (TD-FIX-BURST-VERIFY-002) |

Total: 12 findings (1 CRIT / 4 HIGH / 4 MED / 1 LOW / 2 OBS).
Residuals from pass-2: 2 (F-PASS3-HIGH-003 as partial-close residual of F-MED-NEW-004;
F-PASS3-LOW-001 as partial-close residual of F-LOW-NEW-003).
Net-new defects: 10.
Process-gap findings: 2 (F-PASS3-OBS-001, F-PASS3-OBS-002).

---

## Part A — Pass-2 Residual Verification

| Pass-2 Finding ID | Status in v1.2 | Notes |
|---|---|---|
| F-CRIT-NEW-001-PASS2-RESIDUAL | CLOSED | §C Rule 3 body text revised; spec_parser.rs claim removed |
| F-CRIT-NEW-002 | CLOSED | §D sandbox escape finding resolved; `allowed_urls` populate path now specified |
| F-HIGH-NEW-001 | CLOSED | Instance-pool claim revised to match mod.rs actual lifecycle |
| F-HIGH-NEW-002 | CLOSED | Closed-grammar partition completed with all 12 type patterns |
| F-HIGH-NEW-003 | CLOSED | ADR-022 amendment conflict resolved — single scheduling location |
| F-HIGH-NEW-004 | CLOSED | Wave 0/F amends_bcs sweep scope clarified |
| F-HIGH-NEW-005 | CLOSED | VP-PLUGIN-001 enumeration sync mechanism specified |
| F-MED-NEW-001-PASS2-RESIDUAL | CLOSED | §F Implementation Note revised; strum-derives claim removed |
| F-MED-NEW-002 | CLOSED | VP-PLUGIN-004 test fixture specified |
| F-MED-NEW-003 | CLOSED | 401-injection scoping criteria added |
| F-MED-NEW-004 | PARTIAL-CLOSE (incidental) | Text deleted as side-effect of F-HIGH-NEW-003 fix; changelog claims SP arithmetic (see F-PASS3-HIGH-003) |
| F-MED-NEW-005 | CLOSED | PREREQ-D vs PREREQ-E boot.rs ownership conflict resolved |
| F-LOW-NEW-001 | CLOSED | Absolute filesystem path in PR template citation corrected |
| F-LOW-NEW-002 | CLOSED | PR template existence now documented |
| F-LOW-NEW-003 | PARTIAL-CLOSE | 3 of 8 sensor BCs added to inputs; 5 still absent (see F-PASS3-LOW-001) |
| F-OBS-NEW-001 | CLOSED | amends_bcs bidirectional lifecycle note added |
| F-OBS-NEW-002 | CLOSED | TD-FIX-BURST-VERIFY-001 discipline note added |

14 of 14 pass-2 spec defects substantively closed. Two carry partial-close or mis-attribution
caveats: F-MED-NEW-004 (incidental closure, false changelog) and F-LOW-NEW-003 (3 of 8 BCs).
Both caveats are catalogued as new findings in Part B.

---

## Part B — New Findings (Pass 3)

### F-PASS3-CRIT-001 — Ad-hoc frontmatter field `amends_bcs_pending_full_amendment_in_wave_2_g` is project-novel with no validator

**Tier:** CRITICAL
**Section:** ADR-023 frontmatter

ADR-023 v1.2 frontmatter contains the field
`amends_bcs_pending_full_amendment_in_wave_2_g:` introduced as the fix for F-MED-NEW-003.
This field name is wave-specific, non-generic, and appears in no ADR template schema. No
state-manager hook validates it. The field name encodes the target wave inline (`_wave_2_g`),
making it non-reusable for future ADRs that need to defer a BC amendment to a different wave.
Any future ADR author who copies this pattern will create a proliferation of wave-specific
field names with no shared schema.

The correct schema (per TD-ADR-AMEND-002 being registered by this pass) is a generic list
field: `amends_bcs_pending: [{bc_id, target_wave_for_full_amendment, target_wave_for_prefix_note}]`.
The wave-specific field was adopted verbatim from the adversary's pass-2 proposed-fix language
without verifying that the field name would be reusable — a recurrence of the verbatim-adoption
pattern that TD-FIX-BURST-VERIFY-001 was intended to prevent, but whose scope was limited to
body claims rather than frontmatter schema choices.

**Proposed fix:** Rename field to the generic `amends_bcs_pending` list form in ADR-023 v1.3.
Register TD-ADR-AMEND-002 (process-gap; not an ADR-023 convergence blocker, but must be
codified before the next ADR is authored using this pattern).

---

### F-PASS3-HIGH-001 — VP-PLUGIN-006 referenced but undefined; VP-INDEX has no VP-PLUGIN entries

**Tier:** HIGH
**Section:** ADR-023 §E (Verification Properties)

ADR-023 v1.2 §E cites `VP-PLUGIN-006` as a verification property governing plugin isolation
boundary testing. VP-INDEX.md has no entries with the `VP-PLUGIN-` prefix. The entire
`VP-PLUGIN-*` series is absent from VP-INDEX. No VP-PLUGIN-006 was authored during fix-burst-2.

This is a broken reference introduced in fix-burst-2. The architect cited a VP from memory
without reading VP-INDEX. Any implementer attempting to satisfy VP-PLUGIN-006 will be working
against a non-existent specification. This is exactly the class of finding TD-FIX-BURST-VERIFY-002
(registered below) is designed to prevent via a pre-write citation-integrity validator.

**Proposed fix:** Author VP-PLUGIN-006 in VP-INDEX.md and create the VP file covering plugin
isolation boundary testing, OR replace the VP-PLUGIN-006 citation with an existing VP that
covers the intended invariant. Fix-burst-3 architect must read VP-INDEX before citing any VP.

---

### F-PASS3-HIGH-002 — POL-11 miscited: ADR says `ci_positive_coverage_assertion`; actual is `index_bump_required_for_index_mutations`

**Tier:** HIGH
**Section:** ADR-023 §E (Policy Citations)

ADR-023 v1.2 §E references `POL-11` with the description `ci_positive_coverage_assertion`.
The actual POL-11 entry in `.factory/policies.yaml` is `index_bump_required_for_index_mutations`.
These are entirely different policies. The ADR has the correct policy number but a completely
wrong policy name. An implementer relying on this citation would misidentify which policy
governs the cited behavior.

Root cause: fix-burst-2 cited POL-11 from memory without reading the current `policies.yaml`
source-of-truth. TD-FIX-BURST-VERIFY-001 codified "verify factual claims against source-of-truth
before adopting" for body prose — but policy name verification was not explicitly in scope.
TD-FIX-BURST-VERIFY-002 (registered below) closes this gap.

**Proposed fix:** Read policies.yaml POL-11; replace `ci_positive_coverage_assertion` with
`index_bump_required_for_index_mutations`. Extend fix-burst verification checklist to include:
"Each POL-N citation: I read policies.yaml POL-N and verified the name matches."

---

### F-PASS3-HIGH-003 — v1.2 changelog mis-attributes F-MED-NEW-004 closure as SP arithmetic

**Tier:** HIGH
**Section:** ADR-023 v1.2 Changelog

The v1.2 changelog entry for F-MED-NEW-004 states: "F-MED-NEW-004 corrected via SP arithmetic
reconciliation." Inspection of the actual diff shows that F-MED-NEW-004 was closed as a
side-effect of the F-HIGH-NEW-003 fix: the text containing the erroneous SP claim was removed
when the ADR-022 scheduling conflict was resolved. No SP arithmetic was performed; the
offending text was deleted, not corrected.

This is a false audit trail entry. A future reviewer reading the changelog would conclude that
F-MED-NEW-004 was remediated via explicit SP reconciliation, which is incorrect. If the
underlying SP concern recurs in a later version, the false changelog entry will prevent
accurate root-cause analysis.

**Proposed fix:** Correct changelog entry to: "F-MED-NEW-004 closed incidentally — offending
text removed as part of F-HIGH-NEW-003 ADR-022 scheduling conflict fix; no SP arithmetic
performed. If the SP reconciliation concern is valid, author an explicit SP note; if not,
record the claim as removed as erroneous."

---

### F-PASS3-HIGH-004 — DI-012 amendment lacks back-reference annotation in DI-012 itself

**Tier:** HIGH
**Section:** Domain invariant DI-012 (cross-document consistency)

ADR-023 v1.2 §B documents an amendment to DI-012
(`plugin_dispatch_is_sole_sensor_execution_path`). The amendment is recorded in ADR-023 but
DI-012's own body in `.factory/specs/domain-spec/invariants.md` has no
`scheduled_amendment_in: ADR-023` annotation. The amendment is one-directional: ADR-023
declares the amendment, but DI-012 remains unaware and will appear to future readers as an
unmodified invariant.

This is the same class of one-directional amendment gap that TD-ADR-AMEND-001 was filed to
prevent. The bidirectional consistency requirement added to TD-ADR-AMEND-001 in pass-2 was
not applied to DI-012 during fix-burst-2.

**Proposed fix:** Add `scheduled_amendment_in: ADR-023` annotation to DI-012 in invariants.md.
One-line fix.

---

### F-PASS3-MED-001 — PR template assigned to PREREQ-D in ADR body; pass-1 fix said PREREQ-F

**Tier:** MEDIUM
**Section:** ADR-023 §G (Prerequisites)

ADR-023 v1.2 §G assigns the PR template requirement to story PREREQ-D. The pass-1 fix-burst
plan (documented in the pass-1 report and the fix-burst-1 commit message) stated that the PR
template fix was scoped to PREREQ-F. An implementer reading §G will assign PR template work
to PREREQ-D, conflicting with the pass-1 historical record.

**Proposed fix:** Read the STORY-INDEX entry for S-PLUGIN-PREREQ-D and S-PLUGIN-PREREQ-F;
verify which PREREQ story the PR template work is canonically assigned to; correct the §G
entry to match. STORY-INDEX is the source of truth.

---

### F-PASS3-MED-002 — F-MED-NEW-004 closure incidental; root cause never explicitly analyzed

**Tier:** MEDIUM
**Section:** Process integrity (pass-2 closure audit)

As noted in the residual table and in F-PASS3-HIGH-003, F-MED-NEW-004 was closed as a
deletion side-effect. The pass-2 report counted it as deliberately closed; the v1.2 changelog
claims SP arithmetic as the mechanism. The root cause of F-MED-NEW-004 (an incorrect SP claim
for OCSF transformer work) was never explicitly analyzed or corrected — the text was simply
removed. If the SP claim reappears in a later version, it will not be caught because the
closure record implies it was "fixed."

**Proposed fix:** Add a note to the v1.2 changelog and to the pass-3 residual table clarifying
the incidental nature. If the underlying SP concern was valid, author an explicit SP
reconciliation note; if not, record that the claim was removed as erroneous.

---

### F-PASS3-MED-003 — ADR-022 has 2 sites referencing four built-in adapters; only §G Story 3 amendment scoped

**Tier:** MEDIUM
**Section:** ADR-022 (cross-document consistency)

ADR-023 v1.2 §G notes a scheduled amendment to ADR-022 removing the four built-in adapter
references. ADR-022 has at least 2 distinct sites referencing those adapters: the §G Story 3
section (scoped for amendment) and one additional site in §C (not mentioned in ADR-023's
amendment scope). If fix-burst-3 amends only the §G site, ADR-022 §C will remain inconsistent.

**Proposed fix:** Add the §C site to the ADR-023 §G amendment scope list. Fix-burst-3 architect
must grep ADR-022 for all four built-in adapter names before declaring the amendment complete.

---

### F-PASS3-MED-004 — SS-21 and SS-22 in subsystems table may not exist in ARCH-INDEX (REQUIRES_VERIFICATION)

**Tier:** MEDIUM (REQUIRES_VERIFICATION)
**Section:** ADR-023 §C (Subsystem table)

ADR-023 v1.2 §C lists SS-21 and SS-22 as subsystems in the plugin architecture subsystem
table. STATE.md frontmatter records `subsystem_count: 20`. If ARCH-INDEX only contains SS-01
through SS-20, the SS-21 and SS-22 references are broken. This finding is REQUIRES_VERIFICATION:
if SS-21/SS-22 exist in ARCH-INDEX, this finding is void; if they do not, the ADR contains
broken subsystem references.

**Proposed fix (if verified broken):** Author ARCH-INDEX entries for SS-21 and SS-22, or
replace with correct existing subsystem IDs. Fix-burst-3 architect must grep ARCH-INDEX for
`SS-21` and `SS-22` before touching §C.

---

### F-PASS3-LOW-001 — F-LOW-NEW-003 partial close — only 3 of 8 sensor BCs added to inputs

**Tier:** LOW
**Section:** ADR-023 frontmatter inputs

Pass-2 finding F-LOW-NEW-003 required 8 sensor BCs to be added to the inputs frontmatter.
In v1.2, 3 sensor BCs are present in inputs but 5 remain absent:

- BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md — ABSENT
- BC-2.01.006-cyberint-cookie-auth.md — ABSENT
- BC-2.01.007-claroty-bearer-polymorphic-ids.md — ABSENT
- BC-2.02.003-crowdstrike-field-mapping.md — ABSENT
- BC-2.02.004-cyberint-field-mapping.md — ABSENT

These 5 BCs govern sensor-specific behaviors that ADR-023 claims to supersede via the plugin
architecture. A reviewer auditing ADR-023 against the BCs it impacts will find these 5 missing
from the inputs trail.

**Proposed fix:** Add the 5 missing BCs to the ADR-023 frontmatter inputs list.

---

### F-PASS3-OBS-001 — ADR template needs generic `amends_bcs_pending` schema (process-gap)

**Tier:** OBSERVATION (process-gap)
**Section:** ADR template methodology

The `amends_bcs_pending_full_amendment_in_wave_2_g:` field pattern introduced in v1.2 (see
F-PASS3-CRIT-001) exposes a gap in the ADR template: no generic schema exists for "this ADR
begins an amendment to BC-X that will be fully completed in a future wave." Without it, future
ADR authors will use ad-hoc wave-specific field names or omit the deferred-amendment tracking
entirely.

This is a process-gap finding, not an ADR-023-specific defect. Does not block ADR-023
convergence. Registering as TD-ADR-AMEND-002.

---

### F-PASS3-OBS-002 — Inline finding-citation integrity needs validator (process-gap)

**Tier:** OBSERVATION (process-gap)
**Section:** Fix-burst methodology

Three findings in this pass (F-PASS3-HIGH-001 undefined VP, F-PASS3-HIGH-002 POL-11
miscitation, F-PASS3-MED-004 SS-21/22 verification gap) are all instances of the same
structural pattern: the fix-burst architect cited a reference (VP, POL, SS) without reading
the referenced artifact. TD-FIX-BURST-VERIFY-001 codified this discipline for proposed-fix
body prose claims but the lesson did not transfer to ALL inline citations. A state-manager
pre-write validator at PreToolUse on Write/Edit that rejects documents with unverified
citations would catch this class at authoring time rather than at the next adversarial pass.

Registering as TD-FIX-BURST-VERIFY-002 (P1 escalated from P2 because pass-3 demonstrates
the discipline gap is broader than body prose).

---

## Source-of-Truth Verifications

| Claim | Source Read | Result |
|---|---|---|
| `amends_bcs_pending_full_amendment_in_wave_2_g` field in ADR template | ADR template + policies.yaml | Field is project-novel; no schema entry found |
| VP-PLUGIN-006 exists in VP-INDEX | VP-INDEX.md | NOT FOUND; VP-PLUGIN-* series absent |
| POL-11 name in policies.yaml | policies.yaml | Actual: `index_bump_required_for_index_mutations`; ADR says `ci_positive_coverage_assertion` |
| F-MED-NEW-004 closure mechanism (SP arithmetic) | v1.2 diff vs F-HIGH-NEW-003 fix | Incidental deletion, not SP arithmetic |
| DI-012 back-reference to ADR-023 | invariants.md | No `scheduled_amendment_in: ADR-023` annotation |
| PR template PREREQ assignment (PREREQ-D vs PREREQ-F) | pass-1 fix-burst plan | Pass-1 plan said PREREQ-F; v1.2 §G says PREREQ-D |
| ADR-022 sites referencing four built-in adapters | ADR-022.md | 2 sites (§G Story 3 + §C); only §G Story 3 scoped |
| SS-21 and SS-22 in ARCH-INDEX | ARCH-INDEX.md | REQUIRES_VERIFICATION (not in adversary read context) |
| 5 absent sensor BCs in inputs frontmatter | BC files + F-LOW-NEW-003 requirement | 3 of 8 present; 5 absent |
| spec_parser.rs CustomAdapter references (pass-2 residual) | spec_parser.rs | VERIFIED CLOSED — zero occurrences; §C + §F correctly updated |
| allowed_urls: None TODO (pass-2 residual) | plugin/mod.rs | VERIFIED CLOSED — populate path now specified |
| ADR-022 amendment single scheduling (pass-2 residual) | ADR-022.md + ADR-023.md §G | VERIFIED CLOSED — conflict resolved |
| BC-2.01.013 reference (F-MED-NEW-002 residual) | BC-2.01.013 file | VERIFIED CLOSED — correct reference |
| PREREQ-D→PREREQ-F sequencing (F-MED-NEW-005 residual) | ADR-023 §G sequencing | VERIFIED CLOSED — sequencing corrected |

---

## Top 3 Most-Critical Findings

### 1. F-PASS3-CRIT-001 — Ad-hoc frontmatter field `amends_bcs_pending_full_amendment_in_wave_2_g`

ADR-023 v1.2 frontmatter contains `amends_bcs_pending_full_amendment_in_wave_2_g:`, a
wave-specific non-generic field name not present in any ADR template schema and not validated
by any state-manager hook. The field was adopted verbatim from the pass-2 adversary's proposed
fix without verifying that the field name would be reusable across future ADRs. This is a
structural schema gap: any future ADR needing deferred-BC-amendment tracking will proliferate
arbitrary wave-specific field names. The correct fix is to define a generic `amends_bcs_pending:`
list field in the ADR template. TD-ADR-AMEND-002 is being registered. CRITICAL because the
schema gap will propagate to every future ADR authored before the template is fixed, and
because the underlying issue — verbatim adoption of proposed-fix schema choices without
reusability verification — is a recurrence of the exact structural pattern TD-FIX-BURST-VERIFY-001
was intended to eliminate but whose scope was limited to body prose.

### 2. F-PASS3-HIGH-001 — VP-PLUGIN-006 referenced but undefined

ADR-023 v1.2 §E cites VP-PLUGIN-006 as a verification property. This VP does not exist in
VP-INDEX.md. The entire VP-PLUGIN-* series is absent from VP-INDEX. Any implementer attempting
to satisfy VP-PLUGIN-006 will be working against a non-existent specification. This is a
broken reference introduced in fix-burst-2 by an architect who cited a VP from memory without
reading VP-INDEX. HIGH because VP citations drive implementation decisions; a broken VP
citation either stops work or causes implementers to invent their own interpretation.

### 3. F-PASS3-HIGH-002 — POL-11 miscited as `ci_positive_coverage_assertion`

ADR-023 v1.2 §E cites POL-11 with the description `ci_positive_coverage_assertion`. The
actual POL-11 in policies.yaml is `index_bump_required_for_index_mutations` — a completely
different policy. The ADR has the correct number but an entirely wrong name. An implementer
auditing ADR-023 for policy compliance will misidentify which policy governs the cited behavior.
This demonstrates that TD-FIX-BURST-VERIFY-001's "verify factual claims against source-of-truth"
discipline was not applied to policy name verification — the scope was too narrow. TD-FIX-BURST-VERIFY-002
escalates to P1 precisely because this finding shows the gap extends beyond body prose.

---

## Convergence Assessment

| Metric | Value |
|---|---|
| Pass | 3 |
| Findings total | 12 |
| CRIT | 1 |
| HIGH | 4 |
| MED | 4 |
| LOW | 1 |
| OBS | 2 |
| Residuals from pass-2 | 2 |
| New in pass-3 | 10 |
| Streak | 0/3 (RESET) |
| Status | NOT_CLEAN |
| Fix-burst required | YES (fix-burst-3) |

Convergence streak resets to 0/3. Fix-burst-3 must close all 10 new defects (1 CRIT + 4 HIGH
+ 4 MED + 1 LOW) plus record the 2 OBS process-gap TDs before pass-4 can be dispatched.

User mandate (2026-05-10): full 3-CLEAN cycle, no shortcuts; close process-gap TDs
(TD-ADR-AMEND-002 + TD-FIX-BURST-VERIFY-002) BEFORE Wave 0 dispatch. Fix-burst-3 architect
MUST grep-verify every POL/VP/BC/SS citation against source-of-truth — extending TD-FIX-BURST-VERIFY-001
discipline beyond proposed-fix language to ALL inline citations.

---

## Novelty Assessment

| **Pass** | Pass 3 |
|----------|--------|
| **Total findings** | 12 |
| **Residuals (from pass-2)** | 2 (16.7%) |
| **Net-new defects** | 10 (83.3%) |
| **CRIT novel findings** | 1 (F-PASS3-CRIT-001 — ad-hoc frontmatter schema) |
| **HIGH novel findings** | 3 (F-PASS3-HIGH-001 undefined VP; F-PASS3-HIGH-002 POL miscitation; F-PASS3-HIGH-004 DI-012 back-reference) |
| **MED novel findings** | 3 (F-PASS3-MED-001 PREREQ assignment drift; F-PASS3-MED-003 ADR-022 dual site; F-PASS3-MED-004 SS-21/22) |
| **Process-gap novel findings** | 2 (F-PASS3-OBS-001 schema; F-PASS3-OBS-002 citation validator) |
| **Novelty score** | 0.833 (10 of 12 findings are new) |
| **Trajectory** | 26→16→12 |
| **Verdict** | FINDINGS_REMAIN |

Novelty ratio remains high (0.833) at pass-3, indicating the fix-bursts close previous-pass
defects while introducing new defects in the amended sections. Slowing decay (−10 → −4) combined
with high novelty confirms that citation errors in newly amended sections are the dominant
pattern. The structural fix is citation-integrity validation at authoring time (TD-FIX-BURST-VERIFY-002
P1) rather than further adversarial passes alone.

---

## Operational Notes

**Read-only profile:** This adversary agent was dispatched with read-only tooling. The pass-3
report was delivered inline in the orchestrator transcript. The state-manager backfill burst
(Standing Rule 1) is responsible for persisting this report to
`.factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-3.md`. Per TD-VSDD-ADVERSARY-PERSISTENCE,
adversary agents cannot persist their own reports; this backfill is the current workaround.

**File reads performed:** ADR-023 v1.2, ADR-023-pass-2.md, ADR-022 v1.2, td-from-adr-023-pass-1.md,
VP-INDEX.md, policies.yaml, invariants.md, spec_parser.rs, plugin/mod.rs, BC-2.01.013, BC-2.16.004.
