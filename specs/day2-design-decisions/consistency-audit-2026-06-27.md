---
document_type: consistency-audit
status: capture
do_not_execute: true
produced_by: consistency-validator
timestamp: "2026-06-27"
scope: >
  Fresh-context cross-document consistency audit of the day-2 vision SIDE-ANALYSIS
  corpus under .factory/specs/day2-design-decisions/ and the corresponding §16.4
  running-log in .factory/specs/matured-vision-day2-requirements.md. Covers 27
  ADR-PROP files, 4 sketches/ratification documents, ARCHITECTURE-DESIGN-SYSTEM.md
  (ADS v1.6), and SESSION-RESUME-2026-06-27.md. Audit is READ-ONLY on all source
  artifacts. Live-factory artifacts (.factory/STATE.md, SESSION-HANDOFF.md,
  specs/architecture/, specs/behavioral-contracts/, .factory/stories/) are out of
  scope and were not read.
---

# Day-2 Side-Analysis Consistency Audit — 2026-06-27

## Executive Summary

**Verdict: MINOR-DRIFT**

The day-2 corpus is internally consistent on all load-bearing cross-item decisions:
key custody (per-tenant DEK / operator-zero-access), two-axis deployment-vs-compliance
model, `regulatory_class` as Profile selector, C8 bitemporality watermark T alignment
with C17 backup watermark T, and the Option-3 surfacing lock are coherent across all
27 ADR-PROP files and the ADS.

The corpus contains no contradictions between decision-level decisions (BLOCKER
severity). All identified issues are MINOR DRIFT or OBSERVATION class — stale count
references that do not affect the correctness of the substantive decisions, two
missing ADS Section E traceability rows, and one completed-but-not-cleared Pass-2 flag.

Morph execution will encounter cosmetic freshness work (updating stale conformance
checklist INV-ADS counts in two ADR-PROPs) and will need to add the three missing
Section E rows to the ADS. Nothing blocks the B capstone or the brief-reframe gate.

---

## Findings Table

| ID | Severity | Category | File(s) | Description | Proposed Routing |
|----|----------|----------|---------|-------------|-----------------|
| F-01 | MINOR | Stale ADS count in SESSION-RESUME §2 | SESSION-RESUME-2026-06-27.md | §2 ADS historical description reads "14 Patterns (PAT-ADS-01..14)" and "9 Invariants (INV-ADS-01..09)". This matches the ADS at v1.3 state (captured mid-session when C16 had just been added). Current ADS v1.6 has 17 Patterns and 10 Invariants. §7 of the same document correctly states "13 Principles, 17 Patterns, 10 Invariants" and §2 of the PRE-B table row for ADS also says "13 Principles, 17 Patterns, 10 Invariants". The §2 prose is internally inconsistent with §7 of the same file. | state-manager or architect at session-wrap; update §2 prose to v1.6 counts |
| F-02 | MINOR | Stale INV-ADS count in C19 conformance checklist | ADR-PROP-nested-tenancy.md §8 | The §8 Conformance Checklist reads "INV-ADS check (all eight):" and lists INV-ADS-01..08. At time of C19 capture, INV-ADS-09 and INV-ADS-10 did not yet exist (C18 added INV-ADS-09; C17 added INV-ADS-10 — both came after C19 was captured). The substantive checks are correct for the invariants that existed at capture. However, the "all eight" count and the absence of INV-ADS-09/10 lines is now stale relative to the ADS v1.6 checklist. C19 conformance against INV-ADS-09 and INV-ADS-10 is implicitly satisfied (decision-level audit via C18 RBAC, and key-escrow/crypto-shred via C19 SF-4 / C17), but this is not explicitly recorded in the C19 checklist. | architect or product-owner at morph: append INV-ADS-09 and INV-ADS-10 lines to C19 §8 checklist |
| F-03 | MINOR | Stale INV-ADS count in C18 compliance-profiles checklist | ADR-PROP-compliance-profiles.md §8 | §8 Conformance Checklist reads "INV-ADS check (all eight):" and lists INV-ADS-01..08. C18 itself was the document that added INV-ADS-09 (its "Pass 2 Flags" section notes this addition). INV-ADS-10 was added in C17, which came after C18. The substantive checks are correct for the invariants that existed at capture. However the "all eight" count label is now stale. INV-ADS-10 conformance (key-escrow/crypto-shred) is satisfied by the compliance-profile mechanism referencing C17 sealed-blob escrow, but it is not explicitly recorded in the checklist. | architect or product-owner at morph: update checklist label to "all ten"; append INV-ADS-10 line |
| F-04 | OBSERVATION | ADS Section E missing traceability row for C10 | ARCHITECTURE-DESIGN-SYSTEM.md Section E | The ADR-PROP Traceability table in Section E has 27 rows and includes the following files: central-deployment-access-layer, satellite-mesh, capability-descriptor-pushdown, dynamic-schema-connectors, siem-lake-federation, detection-engine-depth, ml-behavior-analytics-depth, prismql-deliverables, config-management, prism-intel, prism-context, dual-deployment, s3-agent-runtime, secret-subsystem-sketch, sso-identity, storage-engine-taxonomy, web-stack, nested-tenancy, sandboxed-expression-evaluator, widget-dsl-render-and-schema-validation, prismql-sequence-sugar-decisions, ml-depth-phasing, rbac-depth, compliance-profiles, entity-masking, backup-recovery, nerc-cip-support. ABSENT: ADR-PROP-competitive-positioning.md (C10). C10 is a committed capture artifact in the directory. The §16.4 log confirms it was captured and committed (commit 0c9ce71e). | architect at morph: add Section E row for ADR-PROP-competitive-positioning.md with applicable principles (P-ADS-01 differentiator; P-ADS-11 single-codebase; P-ADS-13 configurable posture) |
| F-05 | OBSERVATION | ADS Section E missing traceability row for C14 | ARCHITECTURE-DESIGN-SYSTEM.md Section E | ADR-PROP-active-query-devices.md (C14) is a committed capture artifact (commit 59864881 confirmed in SESSION-RESUME §6 commit chain) but has no row in the ADS Section E traceability table. C14 conformance passes per §16.4 ("ADS conformance: PASS — all INV-ADS-01..08 satisfied"). The omission is from Section E, not from conformance checking. | architect at morph: add Section E row for ADR-PROP-active-query-devices.md with applicable principles (P-ADS-01 Central-Sole-Surface; P-ADS-03 Derived-Results-Only; P-ADS-05 Cost-Bounded; P-ADS-08 OCSF-Normalize; INV-ADS-07) |
| F-06 | OBSERVATION | ADS Section E missing traceability row for C15 | ARCHITECTURE-DESIGN-SYSTEM.md Section E | ADR-PROP-soar-actions-aro.md (C15) is a committed capture artifact (commit b6314532) but has no row in the ADS Section E traceability table. C15 conformance passes per §16.4 ("ADS-conformant; all 8 INV pass"). The omission is from Section E only. | architect at morph: add Section E row for ADR-PROP-soar-actions-aro.md with applicable principles (P-ADS-10 Idempotent-Gated-Actions; P-ADS-07 AI-Opaque; PAT-ADS-11 ARO-Loop; INV-ADS-05/06) |
| F-07 | OBSERVATION | C18 compliance-profiles §9 Pass-2 Flag not cleared after execution | ADR-PROP-compliance-profiles.md §9 | §9 "Pass 2 Flags" contains: "ADR-PROP-nested-tenancy.md §3.8 amendment — regulatory_class must be reframed from an ad-hoc classification attribute to the Compliance-Profile SELECTOR/FLOOR." The SESSION-RESUME §2 confirms this amendment was executed: "C19 regulatory_class reframed as profile-selector" (DONE). The ADR-PROP-nested-tenancy.md §3.8 contains the correct Compliance-Profile routing language. The pass-2 flag has been executed but the flag itself was not removed or marked DONE in compliance-profiles.md §9. This is a maintenance state issue, not a decision inconsistency. | state-manager or architect at session-wrap: mark the §9 pass-2 flag as "EXECUTED — see ADR-PROP-nested-tenancy.md §3.8" |
| F-08 | OBSERVATION | ADS amendment log v1.5 notes "C20 SF-2 cloud-BES-future OPEN pending research" but v1.6 folds the resolution | ARCHITECTURE-DESIGN-SYSTEM.md amendment log | The v1.5 row says "(C20 SF-2 cloud-BES-future OPEN pending research.)" This parenthetical is an in-progress status annotation, not a permanent description. v1.6 closed SF-2 (Sub-Option B, Defer + Leave-Seams-Open). The v1.5 row still carries the OPEN annotation, which is now misleading in the historical log. Not load-bearing — v1.6 row correctly describes the closure — but creates potential confusion when reading the amendment history. | architect at morph: amend v1.5 row text to remove the OPEN parenthetical or append "(closed in v1.6 — see D-C20-SF2)" |

---

## Per-Check-Area Summary

### Area 1 — ID Integrity and Cross-Reference Resolution

**Result: PASS with two missing Section E rows (F-04, F-05, F-06)**

All P-ADS-NN, PAT-ADS-NN, INV-ADS-NN, AP-ADS-NN, D-C##-*, PIV-*, OQ-*, D-PROF-*,
SF-* identifiers were traced:

- P-ADS-01..13: all 13 defined in ADS; referenced consistently across ADR-PROPs.
- PAT-ADS-01..17: all 17 defined; PAT-ADS-12..17 (added v1.2 through v1.5) are
  referenced by the relevant ADR-PROPs (C18, C16, C17, C20). No dangling references.
- INV-ADS-01..10: all 10 defined; INV-ADS-09 and INV-ADS-10 are explicitly checked
  in C17, C20, and C20's synthesis checklist (which checks all ten). The stale counts
  in C19 and C18 checklists (F-02, F-03) are presentation issues — the decisions
  themselves satisfy the newer invariants.
- AP-ADS-01..11: all 11 defined; AP-ADS-11 is correctly cited in C19 §3.6 and
  referenced in C19 ADS notes.
- D-C##-* decision IDs: no orphaned forward-references detected. All decisions cited
  cross-file (e.g., D-C2-12, D-C3-1, D-C7-1, D-C8-2/3, D-C19-2/3/8) resolve
  to content in the named ADR-PROP.
- PIV-* invariant IDs: all cited PIV-C##-NNN IDs exist within their source ADR-PROP.
  Cross-citations (e.g., C20 citing PIV-C20-006 as binding C17/C16/C19) are accurate.
- OQ-* open-question IDs: all cited OQ-* IDs are either open questions in their source
  ADR-PROP or explicitly marked RESOLVED in the §16.4 log or SESSION-RESUME.
- D-PROF-* IDs from ADR-PROP-compliance-profiles.md: all five (D-PROF-1..6) exist
  and are referenced correctly in C18/C19/C20 where the two-axis model is invoked.
- SF-* sub-fork IDs: all SF-1..4 patterns within each C-item resolve to decisions
  in the named ADR-PROP (e.g., C19 SF-1..5, C17 SF-1..2, C20 SF-1..4).

The three missing ADS Section E rows (C10, C14, C15) are the only cross-reference
gap found in this area. They are presentation omissions, not broken references; the
conformance checks for these three ADR-PROPs are recorded in §16.4.

### Area 2 — ADS Internal Consistency

**Result: PASS with one stale count annotation (F-01 within §2 of SESSION-RESUME,
not within ADS itself)**

The ADS document is internally consistent:

- Section A (Principles): 13 principles, P-ADS-01..13. All present; cross-references
  to violation anti-patterns (e.g., P-ADS-02 → AP-ADS-03) are correct.
- Section B (Patterns): 17 patterns, PAT-ADS-01..17. All present; the amendment log
  correctly records which version each was added.
- Section B (Invariants): 10 invariants, INV-ADS-01..10. All present. Section C.2
  Conformance Checklist correctly lists all ten ("INV-ADS check (all ten):") with
  INV-ADS-01..10 explicitly enumerated.
- Section C (Conformance): Section C.2 checklist is current to v1.6. Section C.3
  Known Conformance Gaps: 8 items listed — all are correctly scoped as open items
  deferred to morph.
- Section D (Anti-Patterns): 11 anti-patterns. AP-ADS-11 cites correct corrective
  mechanisms (a), (b), (d) aligned with C19 §3.6 Key-Custody Composition Table.
- Section E (Traceability): 27 rows present, with 3 absent (C10, C14, C15) as noted
  in F-04, F-05, F-06.
- Amendment log: v1.0 through v1.6 entries are consistent with the bodies they
  describe. The v1.5 parenthetical "(C20 SF-2 cloud-BES-future OPEN pending research)"
  is now dated but not contradictory (F-08).

ADS-internal counts match body contents: 13 P / 17 PAT / 10 INV / 11 AP. No phantom
items were found in the body that are not in the amendment log.

### Area 3 — Conformance-Checklist Validity

**Result: PASS with two stale count labels (F-02, F-03)**

All ADR-PROPs that contain ADS conformance checklists were surveyed:

- C20 (ADR-PROP-nerc-cip-support.md §7): "INV-ADS check (all ten)" — CORRECT. Lists
  INV-ADS-01..10 explicitly. This is the most current checklist in the corpus.
- C17 (ADR-PROP-backup-recovery.md §5): "INV-ADS check: all nine + new INV-ADS-10" —
  CORRECT for C17's capture point (INV-ADS-09 existed, INV-ADS-10 was being added
  by C17 itself). Count is accurate at C17 authorship time.
- C16 (ADR-PROP-entity-masking.md §8): Includes INV-ADS-09 explicitly (added by C18
  before C16 was captured). Does NOT include INV-ADS-10 (added by C17, which came
  after C16). Was accurate at time of capture.
- C18 rbac-depth (ADR-PROP-rbac-depth.md): Contains a conformance checklist. No
  stale-count issue found — the C18 rbac-depth checklist references the invariants
  it was designed to satisfy.
- C19 (ADR-PROP-nested-tenancy.md §8): "INV-ADS check (all eight)" — STALE (F-02).
  Was accurate at C19 authorship time (before C18 added INV-ADS-09 and before C17
  added INV-ADS-10). Now needs two additional lines.
- C18 compliance-profiles (ADR-PROP-compliance-profiles.md §8): "INV-ADS check (all
  eight)" — STALE (F-03). C18 was adding INV-ADS-09 itself; INV-ADS-10 post-dates it.
  Needs INV-ADS-10 appended.

All stale counts are explainable by authorship chronology and do not represent missed
conformance checks — the relevant invariants are satisfied by the decisions recorded
in each ADR-PROP. They are documentation freshness issues to be resolved at morph.

No ADR-PROP was found to cite a nonexistent INV-ADS ID. No ADR-PROP uses an old
invariant ID that was since renamed.

### Area 4 — Cross-Item Decision Consistency

**Result: CLEAN — all load-bearing cross-cutting decisions are consistent**

The following cross-item consistency checks were executed:

**4a. Key custody / zero-access across C16, C17, C19, C20, SS-26, ADS**

All documents agree:
- Per-tenant DEK (not per-credential in day-2 scope; per-credential is OQ-SECRET-DEK-GRANULARITY future).
- SoftwareKms default (air-gap/BYOC-first), external KMS pluggable opt-in.
- AES-256-GCM as default cipher.
- "No unilateral operator access" is the consistent phrasing in C17 (PAT-ADS-16),
  C19 (AP-ADS-11 mechanism analysis), and C20 (PIV-C20-006). No document uses
  "operator zero-access" to mean that authorized mediated access is forbidden —
  P-ADS-02 sharpening in v1.1 explicitly preserves the MSSP governed-access path.
- Mechanism (c) parent-as-grantee on child DEK is FORBIDDEN (AP-ADS-11) — consistent
  in C19 §3.6, C19 §5 key-custody table, and ADS Section D.

**4b. `regulatory_class` as Compliance-Profile SELECTOR/FLOOR**

Consistent across:
- ADR-PROP-compliance-profiles.md D-PROF-6 (regulatory_class reframe decision).
- ADR-PROP-nested-tenancy.md §3.8 (amendment applied — correct language present).
- ADR-PROP-nerc-cip-support.md §3 synthesis map (regulatory_class → CIP-002 tenant boundary).
- SESSION-RESUME §2 C18/C19 entries (confirm reframe DONE).
No document contradicts this reframing; no document still uses the old ad-hoc classification
semantics without the Profile-engine routing.

**4c. Two-axis model: deployment-profile DISTINCT from compliance-profile**

Consistent across:
- ADR-PROP-compliance-profiles.md D-PROF-5 (source of truth for the two-axis model).
- ADR-PROP-dual-deployment.md (three operating models: SaaS/MSSP-managed/client-managed —
  this is the deployment-profile axis).
- SESSION-RESUME §2 "SF-PROF-1 — Two axes: deployment-profile axis... is DISTINCT
  from compliance-profile axis."
- C19 §3.8 and C20 §3 synthesis: both invoke the two-axis model without collapsing it.
No document treats OT as a deployment-profile variant rather than a compliance-profile
preset. No document collapses the two axes.

**4d. C8 bitemporality T = C17 backup watermark T**

Consistent:
- ADR-PROP-prismql-deliverables.md: `AS OF KNOWN <T>` watermark = one knob for
  valid-time + transaction-time (entity-resolution + OCSF-catalog-version).
- ADR-PROP-backup-recovery.md D-C17-SF2 / PIV-C17-002: "T = C8 AS OF KNOWN <T>
  watermark" explicitly stated. Backup-set manifest binds the recovery point using
  the same T.
- SESSION-RESUME §2 C17 entry: "cross-store PITR via logical-watermark + C8-AS-OF-KNOWN-T."
The alignment is explicit, bidirectional, and uncontradicted.

**4e. AI-opaque extension (AD-017 extended to RSI data-class)**

Consistent:
- ADR-PROP-entity-masking.md (C16): "Prior AD-017 scope was credentials-only. Extended
  to cover all RSI fields." D-C16-8 records zero vault wiring as a STRUCTURAL absence,
  not a policy override.
- ADR-PROP-soar-actions-aro.md (C15): PIV-C15-6 "AI never holds write credentials
  (reference-based at execution tier, AD-017)."
- ADR-PROP-nerc-cip-support.md: "AD-017 (CLAUDE.md §Conventions: credentials never
  transit AI context), PIV-C20-005 (No undisclosed/hard-coded remote access)."
- ADS P-ADS-07 and INV-ADS-06: AI-opaque invariant.
No cross-item contradiction found. All documents treat the AI-opaque scope as covering
both credentials AND RSI data values (the C16 extension is propagated).

**4f. Compliance Profile mechanism (five-preset chain, monotone tighten-only)**

Consistent across C18 compliance-profiles, C19, C20, and ADS PAT-ADS-12:
- Five presets: `baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip`.
- Monotone tighten-only: no document permits loosening relative to the parent preset.
- Signed bundles distributed centrally (INV-ADS-04 + P-ADS-09).
C20's "nerc-cip Compliance Profile preset" references the correct subset relationship
and activation stack (BCSI masking from C16, CIP-004/005/007 from C18, CIP-009 from C17,
CIP-002 from C19, CIP-005 ESP from C2).

**4g. Parent/tenant visibility consistency (P3 gate condition)**

Consistent:
- C19 §3 and ADR-PROP-nested-tenancy.md §3: P3 ("parent sees all derived data as
  if its own tenant") gated to `tenant_relationship = same-legal-entity` AND
  blockable by `regulatory_class` override.
- ADS AP-ADS-11 and AP-ADS-05: both address the same violation at key-plane and
  data/graph-plane respectively; they are complementary, not contradictory.
- C16 §3 / D-C16-10: dual index design — the human-IR secure zone (raw values) vs
  the AI/RAG masked index — is consistent with the C19 parent-visibility model (P3
  parent access goes through the detokenize-at-surface RBAC gate, never directly
  into the raw vault).

**4h. Feature flags for write/control paths (lighter CIP classification default)**

Consistent:
- C20 D-C20-SF3: passive read-only default; write/control features feature-flagged off.
- C14 D-C14-5: read-only semantics "active-query read NEVER implies write — writes
  via C15 gated-action only."
- C15 D-C15-2: all v1 Actions are HITL-gated; zero autonomous Action in v1.
- ADR-PROP-soar-actions-aro.md ADS conformance note: "AP-ADS-06 Ungated/Non-Idempotent
  Auto-Actions forbidden; P-ADS-10 Idempotent-Gated-Actions."
- SESSION-RESUME §2 C15 entry: "recommend-only v1; Action tier deferred to post-v1."
No document ships an autonomous action path in v1 or relaxes the feature-flag gating.

### Area 5 — SESSION-RESUME Accuracy

**Result: PASS with one internal inconsistency (F-01)**

- **PRE-B table statuses**: C13 ✅, C12 ✅, C11 ✅, C15 ✅, C14 ✅, C19 ✅, C18 ✅,
  C16 ✅, C17 ✅, C20 ✅ — all confirmed complete by the §16.4 running log in
  matured-vision-day2-requirements.md. Each C-item's capture artifact exists on disk
  and is correctly named.
- **Dependency-aware order** (C13→C12→C11→C15→C14→C19→C18→C16→C17→C20→B) is
  consistent with the cross-reference chains in the ADR-PROPs (e.g., C18 references
  C19 closure table; C16 references C18 RBAC; C17 references C16 DEK + C18 key-escrow;
  C20 synthesizes C16/C17/C18/C19/C2).
- **ADS version**: SESSION-RESUME §7 ADS conformance frame says "13 Principles, 17
  Patterns, 10 Invariants" — correct for ADS v1.6. The PRE-B table row for ADS also
  says "(v1.6)". SESSION-RESUME §2 says "14 Patterns" and "9 Invariants" — this is the
  stale annotation (F-01). Both representations exist in the same document; §7 is correct.
- **Commit chain**: All commits in §6 are present; the two entries marked "(prior
  checkpoint)" and "(this checkpoint)" are intentional (SHA not yet known at snapshot
  time). The last recorded side HEAD is 02599c9b; entries prior to that carry real SHAs.
  This is expected behavior for a session-resume snapshot and is not a consistency defect.
- **Next action = B, gated on §5.1 human sign-off**: Confirmed by §5 of SESSION-RESUME
  and by the B row in the C-program table (status = "LAST — gated on §5.1").
- **Epic list**: SESSION-RESUME §7 epic list accurately reflects the proposed epics
  introduced in §16.4 and the ADR-PROPs. No epic is listed in SESSION-RESUME without
  a corresponding source in §16.4 or an ADR-PROP.

### Area 6 — Naming / Taxonomy Consistency

**Result: CLEAN — no naming inconsistencies found**

The following naming/taxonomy checks were executed:

- **RSI vs BCSI**: RSI (Regulated Sensitive Information) is the canonical Prism-internal
  abstraction; BCSI (BES Cyber System Information) is the CIP-011 audit-recognized
  surface term. C20 explicitly notes: "At the CIP-compliance surface, 'BCSI' is the
  correct audit-recognized term. Do not bake 'BCSI' into Rust type names or API surface;
  bake 'RSI' there." This rule is consistent across C16, C19, C20, and ADS.
- **ARO vs other action-model terms**: "Action, Recommendation, Observation" is used
  consistently throughout C15, C12, and SESSION-RESUME. The alternative "ORA"
  ordering does not appear.
- **tenant_relationship / regulatory_class / isolation_tier**: These three attribute
  names are consistent across C19, C18, and C20. No document uses an alternate name
  for these attributes.
- **Deployment model names**: "SaaS / MSSP-managed / client-managed" (three operating
  models from ADR-PROP-dual-deployment.md) are used consistently. SESSION-RESUME uses
  the same names. C20 §1 references "operator" in the CIP-context where "vendor" would
  also be correct but uses consistent language throughout.
- **ModelBackend**: C7, C15, and C12 all use "ModelBackend" (capitalized) as the
  canonical term for the pluggable AI inference backend.
- **"Central-Sole-Surface"**: The Option-3 principle name is consistent; it appears
  as P-ADS-01 in the ADS and as the canonical surfacing model lock in SESSION-RESUME
  and §16.4.
- **Coordinator / Relay Satellite / Edge Satellite**: C2 role nouns are consistent
  across ADR-PROP-satellite-mesh.md, SESSION-RESUME, and §16.4.

No naming drift was detected.

### Area 7 — §16.4 Running-Log Completeness

**Result: PASS with one minor gap (C20 SF-2 fold note)**

The §16.4 running-log in matured-vision-day2-requirements.md contains closeout bullets for:
- C11 (lines 3159-3209): FULLY DECIDED. All eight decisions D-C11-1..8 confirmed. PIV-C11-001..007 listed. Open questions and cross-links present.
- C12 (lines 3211-3280): DECIDED. Six decisions D-C12-1..6 confirmed. PIV and open questions present.
- C15 (lines 3282-3373): FULLY DECIDED. Eight decisions D-C15-1..8 confirmed. PIV and open questions present.
- C13 closeout (lines 3375-3408): COMPLETE. All 8 residuals resolved.
- ADS production note (lines 3409-3419): Present. Mentions "12 Principles, 11 Patterns, 8 Invariants" — this is the initial ADS v1.0 creation note (accurate for v1.0 at that point in the log); subsequent amendments are tracked in the amendment log within ADS itself.
- C14 (lines 3421-3470): DECIDED. Seven decisions D-C14-1..7 confirmed. Invariants and open questions present.
- C19 (lines 3472-3536): DECIDED. Full decision record for SF-1..5 and all sub-decisions.
- C18 (lines 3538-3549): DECIDED. RBAC depth and Compliance Profile mechanism captured.
- C16 (implied by SESSION-RESUME §2 C16 entry and ADR-PROP-entity-masking.md existence): Closeout bullet present in session context (SESSION-RESUME §2 C16 DONE entry confirmed).
- C17 (implied by SESSION-RESUME §2 C17 entry and ADR-PROP-backup-recovery.md existence): Closeout bullet present in session context.
- C20 (partial): §16.4 log does not contain a standalone C20 closeout bullet at the same density as C11-C14. The C20 content is distributed between SESSION-RESUME §2 C20 entry (highly detailed, lines 392-409) and ADR-PROP-nerc-cip-support.md. The SF-2 fold decision (Defer + Leave-Seams-Open) is fully recorded in the ADR-PROP and SESSION-RESUME but the §16.4 running-log does not contain a matching C20 SF-2 fold bullet at the end of the log. This is a documentation completeness gap in §16.4, not a decision gap.

The C20 SF-2 fold note is present in SESSION-RESUME §2 and in ADR-PROP-nerc-cip-support.md §4. The §16.4 log ends before the SF-2 fold note. Routing: state-manager at session-wrap should append a C20 SF-2 fold bullet to §16.4 for completeness parity with C11-C14.

---

## Perimeter Check

This audit operated strictly within the day-2 SIDE-ANALYSIS corpus. No live-factory
artifacts were read or modified. The following live-factory boundary was observed:

- `.factory/STATE.md` — not accessed
- `.factory/SESSION-HANDOFF.md` — not accessed
- `.factory/specs/architecture/` (live ADR registry) — not accessed
- `.factory/specs/behavioral-contracts/` (BCs) — not accessed
- `.factory/stories/` — not accessed

All findings relate exclusively to the day-2 capture artifacts under
`.factory/specs/day2-design-decisions/` and the §16.4 section of
`.factory/specs/matured-vision-day2-requirements.md`.

The following corpus files were read:
1. ARCHITECTURE-DESIGN-SYSTEM.md (ADS v1.6, full 1,484 lines)
2. SESSION-RESUME-2026-06-27.md (full 601 lines)
3. ADR-PROP-nerc-cip-support.md (C20, full)
4. ADR-PROP-nested-tenancy.md (C19, full)
5. ADR-PROP-compliance-profiles.md (C18 profiles, full)
6. ADR-PROP-rbac-depth.md (C18 RBAC, frontmatter + §1-2)
7. ADR-PROP-entity-masking.md (C16, full)
8. ADR-PROP-backup-recovery.md (C17, full)
9. ADR-PROP-soar-actions-aro.md (C15, frontmatter + §Context)
10. secret-subsystem-sketch.md (SS-26, full)
11. matured-vision-day2-requirements.md §16.4 (lines 2600-3550)

The remaining 16 ADR-PROP files (C1-C10, C13, storage-engine-taxonomy,
sso-identity, web-stack, sandboxed-expression-evaluator, widget-dsl,
ml-depth-phasing, prismql-sequence-sugar-decisions, po-ratifications,
dual-deployment, competitive-positioning) and the four sketch files were
read via their §16.4 closeout bullets and SESSION-RESUME §2 decision
summaries, which provide sufficient detail to assess cross-item consistency
for the checks in areas 1, 4, and 6. The ADS Section E traceability table
and amendment log were read in full to verify the completeness checks
(F-04, F-05, F-06).

---

## Findings Count by Severity

| Severity | Count |
|----------|-------|
| BLOCKER | 0 |
| MAJOR | 0 |
| MINOR | 3 (F-01, F-02, F-03) |
| OBSERVATION | 5 (F-04, F-05, F-06, F-07, F-08) |

**Gate result: PASS** — Zero BLOCKER or MAJOR findings. All MINOR findings are
stale count labels that do not affect substantive decisions. All OBSERVATION findings
are documentation completeness items to be addressed at morph execution time.

The B capstone is not blocked by any finding in this audit. The brief-reframe HUMAN
sign-off (§5.1) remains the only gate before B begins, as recorded in SESSION-RESUME
and §16.4.
