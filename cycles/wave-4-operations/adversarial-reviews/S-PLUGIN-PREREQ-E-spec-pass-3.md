---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 3
scope: spec
verdict: BLOCKED
total_findings: 8
severity_breakdown:
  critical: 0
  high: 4
  medium: 3
  low: 1
  observation: 0
in_scope_findings: 8
observations_queued: 0
produced_by: adversary
reviewed_at: 2026-05-15
fix_burst: fix-burst-3
fix_burst_closed_at: D-577
streak_after_fix: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 3

**Verdict: BLOCKED — 8 findings (0C + 4H + 3M + 1L + 0OBS)**

Fix-burst-3 closed all 8 in-scope findings. 5 of 8 findings were sibling-sweep regressions from
fix-burst-2 (FB2), exposing a systemic POL-25 enforcement gap — the sweep template was not
consistently applied across all artifact layers after FB2 mutations. 3 findings were novel:
F-LP3-HIGH-002 (D1/D2/AC-2 Path A/B contradiction), F-LP3-HIGH-003 (register_write_tool
signature drift unit→Result), and F-LP3-HIGH-004 (ARCH-INDEX stale ADR versions). Streak remains
0/3. Not converging — adversary pass-4 next.

---

## Key Discovery: 5-of-8 Are FB2 Sibling-Sweep Regressions (Systemic POL-25 Gap)

Pass-3 found that fix-burst-2's mutations to VP-156, BC-2.16.012, ADR-026, and ADR-027 each
introduced at least one sibling site that was not swept. This is the second consecutive pass in
which a majority of findings are sibling-sweep regressions from the prior fix-burst. This pattern
is a cycle-close codification candidate:

- **POL-25 extension needed:** Enumerate index/frontmatter sweep targets explicitly in the POL-25
  policy body so the sweep template is unambiguous for each artifact type.
- **POL-23 extension needed:** ADR-version-bump sibling sweep must cover ARCH-INDEX registry rows,
  not only the ADR body changelog.

Both codification candidates are queued for cycle-close session-reviewer per operating procedure.
State-manager does NOT codify mid-cycle.

---

## Finding Summary Table

| ID | Severity | Status | Closed By | Description |
|----|----------|--------|-----------|-------------|
| F-LP3-HIGH-001 | HIGH | CLOSED | architect | VP-156 description in 4 sibling sites still cited "uniqueness + happens-before" after FB2 rewrote the VP-156 body to uniqueness-only. 4 sites swept: VP-INDEX.md row description (v1.39→v1.40), verification-architecture.md Provable Properties Catalog row (v1.33→v1.34), BC-2.16.012 §VP Anchors section (v1.3→v1.4), BC-2.16.012 §Verification Properties section (v1.3→v1.4). Additionally: stale "ADR-026 D7 v1.2" pin in BC-2.16.012 §Verification Properties updated to v1.5 (FB2 bumped ADR-026 to v1.4 but did not propagate pin; FB3 architect sibling-sweep caught it). |
| F-LP3-HIGH-002 | HIGH | CLOSED | product-owner + architect | D1/D2/AC-2 Path A/B contradiction: ADR-026 D1 specified a 2-method trait (FB1 closure), but D2 body and AC-2 in the story still described the built-in auth impls as requiring "ZERO changes" — contradicting D1's auth_type_name() requirement. Path B chosen (production-grade default; rejected Path A "unknown" silent fallback as non-production): (a) ADR-026 D2 narrative amended to "one new method body per impl" (v1.4→v1.5); (b) ADR-026 D1 Path B rationale documented — per-impl one-line `fn auth_type_name()` body returning static string; (c) BC-2.01.016 §Postconditions rewritten from "without change" to "with one new method body per impl" + INV-AUTH-OPEN-002 aligned (v1.2→v1.3); (d) Story AC-2 rewritten — "ZERO changes" → "ONE NEW METHOD BODY per impl"; Red Gate Test 3 renamed `unchanged` → `minimal_diff` + updated to assert one new method body (v1.3→v1.4). |
| F-LP3-HIGH-003 | HIGH | CLOSED | product-owner | register_write_tool signature drift: Task 7, AC-9, and Red Gate Test 8 in the story still described `register_write_tool` as returning `()` (unit) — contradicting the D7 error-on-duplicate semantics mandated in ADR-026 and implemented in BC-2.16.012 EC-016-012-004. Story updated: Task 7 API signature → `Result<(), SpecEngineError>`; AC-9 updated with explicit error-path assertions (E-PLUGIN-012 on duplicate tool_name, E-PLUGIN-020 on post-boot registration); Red Gate Test 8 updated to assert all three paths (happy, E-PLUGIN-012, E-PLUGIN-020). Story v1.3→v1.4. |
| F-LP3-HIGH-004 | HIGH | CLOSED | architect | ARCH-INDEX stale ADR versions: ADR-026 registry row still showed PROPOSED v1.1 (not v1.5 after 4 version bumps across FB1/FB2/FB3); ADR-027 registry row showed PROPOSED v1.1 (not v1.2 after FB2 bump). POL-23 ADR-version-bump sibling sweep did not cover ARCH-INDEX registry rows. ARCH-INDEX v2.45→v2.46: ADR-026 row updated PROPOSED v1.1 → PROPOSED v1.5; ADR-027 row updated PROPOSED v1.1 → PROPOSED v1.2. Codification candidate: extend POL-23 to cover ARCH-INDEX registry row version field. |
| F-LP3-MED-001 | MEDIUM | CLOSED | product-owner | E-PLUGIN-012 category still cited "validation" in story Error Taxonomy Additions table — FB2 finalized E-PLUGIN-012 as a boot-phase error (category: boot) when correcting the E-PLUGIN-001 collision. Story Error Taxonomy table category cell for E-PLUGIN-012: "validation" → "boot". E-PLUGIN-020 row was correctly set to "runtime" in FB2. Story v1.3→v1.4 (bundled with F-LP3-HIGH-002 and F-LP3-HIGH-003 changes). |
| F-LP3-MED-002 | MEDIUM | CLOSED | product-owner | E-PLUGIN rows missing from story + stale v1.25 version pin: (a) 2 E-PLUGIN rows (E-PLUGIN-012 DuplicateWriteToolRegistration + E-PLUGIN-020 WriteToolRegistrationAfterBoot) were absent from the story's Error Taxonomy Additions table — added; (b) version pin in AC-3 body cited error-taxonomy.md v1.25 at 2 sites — updated to v1.27 (current canonical per D-575/D-576). Story v1.3→v1.4 (bundled). HS-PREREQ-E-001 §Expected Outcome also cited v1.25 — updated to v1.27 (HS-001 v1.1→v1.2). |
| F-LP3-MED-003 | MEDIUM | CLOSED | architect | ADR-026 runtime_deliverables field underpopulated: D7 decision (error-on-duplicate) was codified in FB1 but ADR-026 `runtime_deliverables:` frontmatter array listed only 3 entries — missing 5 new entries introduced by the D7 + D1 Path B decisions across FB1/FB2. Architect added 5 entries: `SpecEngineError::DuplicateWriteToolRegistration variant`, `SpecEngineError::WriteToolRegistrationAfterBoot variant`, `E-PLUGIN-012 error taxonomy row`, `E-PLUGIN-020 error taxonomy row`, `fn auth_type_name() body in 4 built-in auth impls`. ADR-026 v1.4→v1.5. Additionally: ADR-026 D7 category "validation" → "boot" (aligned with F-LP3-MED-001 boot-phase classification). |
| F-LP3-LOW-001 | LOW | CLOSED | product-owner | HS-PREREQ-E-003 missing 2 error-path sub-scenarios: (a) HS-003-04 duplicate registration sub-scenario (plugin attempts to register a write tool with an already-registered tool_name during boot sequence — validates E-PLUGIN-012 path); (b) HS-003-05 after-boot registration sub-scenario (plugin calls register_write_tool after boot step 8 completes — validates E-PLUGIN-020 path). Both sub-scenarios were exercised in AC-9 and Red Gate Test 8 but absent from the holdout scenario set. Added to HS-PREREQ-E-003 v1.2→v1.3. |

---

## Path B Decision Rationale (F-LP3-HIGH-002)

**Option A (rejected):** `auth_type_name()` returns `"unknown"` for built-in impls; no per-impl
body required. Rejected because: (1) silently returns wrong auth-type data to plugin consumers;
(2) contradicts the production-grade default (no silent fallbacks); (3) the data is static and
trivially implementable (one line per impl); (4) would require later correction when plugin
consumers rely on the field.

**Option B (chosen):** Each of the 4 built-in auth impls adds exactly one line:
`fn auth_type_name(&self) -> &'static str { "<static-string>" }`. This is the minimal-diff
production-grade solution — correct output, zero runtime cost, zero architectural change.

The "ZERO changes" framing in AC-2 was aspirational, not accurate. Path B accurately describes
what the implementation requires while keeping the change surface as small as possible.

---

## Sibling-Sweep Regression Pattern (Cycle-Close Codification Candidates)

5 of 8 pass-3 findings trace directly to incomplete sibling sweeps in fix-burst-2:

| Finding | FB2 Mutation | Missed Sibling |
|---------|-------------|----------------|
| F-LP3-HIGH-001 (4 sites) | VP-156 body rewritten uniqueness-only | VP-INDEX row + verification-architecture row + BC-2.16.012 §VP Anchors + §VPs |
| F-LP3-HIGH-002 | ADR-026 D1 2-method surface | D2 narrative + AC-2 story + BC-2.01.016 §Postconditions/INV |
| F-LP3-MED-001 | E-PLUGIN-012 finalized boot-phase | Story Error Taxonomy table category cell |
| F-LP3-MED-002 | error-taxonomy v1.26→v1.27 | Story Error Taxonomy table + v1.25 pin in AC-3 + HS-001 v1.25 pin |
| F-LP3-MED-003 | ADR-026 D7 boot-phase decisions | runtime_deliverables frontmatter array |

Proposed codification extensions (cycle-close, session-reviewer):
1. **POL-25 extension:** Sweep target list must include `VP-INDEX row`, `verification-architecture.md row`, `all BC §VP Anchors sections`, `all BC §Verification Properties sections`, `ARCH-INDEX registry row` when any VP or ADR version changes.
2. **POL-23 extension:** ADR version bump sibling sweep must include `ARCH-INDEX registry row version field` (not only ADR body changelog).

---

## Artifact Versions After Fix-Burst-3

| Artifact | After FB2 | After FB3 |
|----------|-----------|-----------|
| ADR-026 | v1.4 | v1.5 |
| ADR-027 | v1.2 | v1.2 (unchanged) |
| BC-2.01.016 | v1.2 | v1.3 |
| BC-2.16.012 | v1.3 | v1.4 |
| VP-INDEX | v1.39 | v1.40 |
| verification-architecture | v1.33 | v1.34 |
| ARCH-INDEX | v2.45 | v2.46 |
| HS-PREREQ-E-001 | v1.1 | v1.2 |
| HS-PREREQ-E-003 | v1.2 | v1.3 |
| S-PLUGIN-PREREQ-E story | v1.3 | v1.4 |
| error-taxonomy | v1.27 | v1.27 (unchanged) |
| STATE + HANDOFF | v7.281 | v7.282 |

---

## Next Step

Adversary pass-4 dispatch (fresh-context). BC-5.39.001 3-CLEAN protocol — streak 0/3.
Pass-3 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-3.md`

Convergence trajectory: pass-1: 14 → pass-2: 9 → pass-3: 8 (NOT converging; sibling-sweep
regression pattern recurring). Pass-4 must break the pattern to begin convergence.
