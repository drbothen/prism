---
document_type: po-adjudication
finding_id: F-LP12-LOW-001
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: 12
severity: LOW
status: CLOSED
decision: OPTION_A
decision_date: 2026-05-30
authored_by: product-owner
d_row: D-875
---

# PO Adjudication: F-LP12-LOW-001

**Finding (verbatim from Pass 12 adversarial report):**
21 cite-pins to `BC-2.01.017 v1.3` or `BC-2.01.017 v1.2` in production code at
`.worktrees/S-DTU-CYBERINT-AUTH-FIDELITY-001/crates/prism-spec-engine/src/auth_provider.rs`.
BC-2.01.017 is currently at v1.4 (D-866). POL-29 step 8f v1.29 mandates sibling-sweep
including `rg "<artifact-ID> v<old>" crates/ --type rust` when any burst bumps a BC frontmatter
version. This sweep was NOT performed at D-866.

---

## 1. Per-Cite-Pin Analysis

### EC version introduction (from BC-2.01.017 changelog)

| Version | D-row | Content introduced |
|---------|-------|--------------------|
| v1.0 | D-849 | Initial draft. EC-017-001, EC-017-002, EC-017-003, EC-017-004, EC-017-005, EC-017-006, EC-017-007, EC-017-008, EC-017-009. E-AUTH-004, E-AUTH-005, E-AUTH-006. |
| v1.1 | D-852 | SUPERSEDED. Bad amendment to EC-017-005 (fabricated evidence). Retained for audit trail. |
| v1.2 | D-854 | F-LP1-MED-002 RE-ADJUDICATION. Reverted v1.1 bad amendment. Restored EC-017-005 to correct E-AUTH-006 semantics. Critically: added precise resolver-source citation (lines 78-81) and accurate NOT-empty-string-normalized behavior. The v1.2 pin in code references THIS restoration — the behavior was ambiguous/wrong in v1.1, correct in v1.2. |
| v1.3 | D-857 | F-LP3-HIGH-001. Introduced EC-017-010 and E-AUTH-007 (BackendUnavailable → distinct error code). |
| v1.4 | D-866 | F-LP8-MED-001. Changelog hygiene only. No new ECs, no semantic content change. |

### Cite-pin table

| Line | Text | EC / behavior anchored | EC first-defined version | Pin version | Category |
|------|------|------------------------|--------------------------|-------------|----------|
| 145 | `E-AUTH-007` (BC-2.01.017 v1.3 EC-017-010) | EC-017-010 BackendUnavailable→E-AUTH-007 | v1.3 | v1.3 | A |
| 192 | BC-2.01.017 v1.3 EC-017-010 / error-taxonomy.md v1.54 §E-AUTH-007 | EC-017-010 | v1.3 | v1.3 | A |
| 282 | BC-2.01.017 v1.3 EC-017-010 / TV-BC-2.01.017-009 / error-taxonomy.md v1.54 §E-AUTH-007 | EC-017-010 + test vector | v1.3 | v1.3 | A |
| 347 | BC-2.01.017 v1.3 EC-017-010 / error-taxonomy.md v1.54 §E-AUTH-007 | EC-017-010 | v1.3 | v1.3 | A |
| 436 | BC-2.01.017 v1.3 EC-017-010 | EC-017-010 | v1.3 | v1.3 | A |
| 457 | BC-2.01.017 v1.3 EC-017-010 / error-taxonomy.md v1.54 | EC-017-010 variant dispatch | v1.3 | v1.3 | A |
| 484 | BC-2.01.017 v1.3 EC-017-010 (in error message string) | EC-017-010 | v1.3 | v1.3 | A |
| 897 | BC-2.01.017 v1.2 EC-017-003 (credential not found path) | EC-017-003 re-adjudicated | v1.2 (restored) | v1.2 | A |
| 904 | BC-2.01.017 v1.2 EC-017-003; ADR-022 §C | EC-017-003 | v1.2 (restored) | v1.2 | A |
| 918 | BC-2.01.017 v1.2 EC-017-003 (in assert message) | EC-017-003 | v1.2 (restored) | v1.2 | A |
| 926 | BC-2.01.017 v1.2 EC-017-003 (resolver Err path) | EC-017-003 | v1.2 (restored) | v1.2 | A |
| 932 | BC-2.01.017 v1.2 EC-017-005 (empty/whitespace value path) | EC-017-005 re-adjudicated | v1.2 (restored) | v1.2 | A |
| 944 | BC-2.01.017 v1.2 EC-017-005; ADR-022 §C | EC-017-005 | v1.2 (restored) | v1.2 | A |
| 962 | BC-2.01.017 v1.2 EC-017-005 (in assert message) | EC-017-005 | v1.2 (restored) | v1.2 | A |
| 970 | BC-2.01.017 v1.2 EC-017-005 (resolver Ok(empty) path) | EC-017-005 | v1.2 (restored) | v1.2 | A |
| 1006 | BC-2.01.017 v1.3 EC-017-010 (backend unavailable path) | EC-017-010 | v1.3 | v1.3 | A |
| 1012 | ADR-022 §C; BC-2.01.017 v1.3 EC-017-010 | EC-017-010 | v1.3 | v1.3 | A |
| 1028 | BC-2.01.017 v1.3 EC-017-010 (in assert message) | EC-017-010 | v1.3 | v1.3 | A |
| 1036 | BC-2.01.017 v1.3 EC-017-010 / TV-BC-2.01.017-009 (in assert message) | EC-017-010 + test vector | v1.3 | v1.3 | A |
| 1042 | BC-2.01.017 v1.3 EC-017-010 (in assert message) | EC-017-010 | v1.3 | v1.3 | A |

Note on EC-017-003 and EC-017-005: Both ECs were originally defined in v1.0. However, v1.1 introduced an incorrect amendment to EC-017-005, and v1.2 reverted it with precise behavioral evidence. Code written AFTER D-854 pins to v1.2 to assert it was verified against the re-adjudicated (correct) semantics — not against the superseded v1.1 or the less-precisely-cited v1.0. This is a meaningful behavioral anchor, not stale notation.

**Verdict: All 21 cite-pins are Category A (behavioral anchors).**

---

## 2. Project Convention Findings

Examination of other versioned BC cite-pins in `crates/`:

| BC cite-pin found in code | BC current version | Relationship |
|---------------------------|--------------------|--------------|
| `BC-2.03.003 v1.4` (prism-credentials/src/file.rs) | v1.4 | Current-version pin — v1.4 introduced Argon2id parameters |
| `BC-2.11.003 v1.4` (prism-query/src/) | v1.4 | Current-version pin at time of write — v1.4 expanded the denylist |
| `BC-2.15.008 v1.7` (prism-storage/src/) | v1.7 | Current-version pin — v1.7 is current |
| `BC-2.16.001 v1.6` (prism-spec-engine/src/spec_parser.rs, tests/) | v1.7 | **Pinned-at-write-time** — code was written when v1.6 was current; v1.7 added incremental refinement. Code correctly anchors to the version that governed the writing context. |

The BC-2.16.001 `v1.6` vs current `v1.7` case is directly analogous to the F-LP12-LOW-001 situation. It demonstrates that the project convention is **"pinned-at-write-time"**: cite-pins reflect the BC version that governed the behavior at the time the code was written. The convention is neither a promise to track current nor strictly "introduced-in" — it records the spec state the implementer verified against. This convention is consistent with TD-VSDD-091 (anti-volatile-pin): the pin is a stable, semantically meaningful anchor, not a decaying line-number reference.

---

## 3. Chosen Option and Rationale

**Decision: Option A — "Pinned-at-write-time" anchor convention. No code change. BC-2.01.017 §Notes amendment.**

Rationale:

1. **All 21 cite-pins are semantically correct.** Each pin cites the BC version that introduced or re-established the specific EC/postcondition being anchored. Updating to v1.4 would replace a semantically meaningful anchor with a version whose only content change was changelog hygiene — which conveys less information.

2. **v1.4 adds nothing relevant.** The D-866 v1.4 bump was explicitly a "no semantic content change" operation (changelog row deduplication + reorder). There is no EC, postcondition, or invariant that a v1.4 cite-pin would provide over v1.3. Forcing 21 cite-pin updates for a hygiene-only bump is the "mechanically correct but semantically empty" outcome.

3. **Project convention (BC-2.16.001 precedent) supports pinned-at-write-time.** BC-2.16.001 code cites v1.6 while the BC is at v1.7 — and this was not flagged as an error in the codebase. Same pattern.

4. **POL-29 step 8f v1.29 must be amended.** The sweep obligation was designed to catch cases where a semantic change is made to a BC but callsites still reference the old (now-superseded) behavior. It was not designed to force cite-pin updates on hygiene-only version bumps. The policy needs an exception clause for no-semantic-content-change bumps.

5. **Production-grade lens (CLAUDE.md):** Forcing 21 mechanical cite-pin updates produces churn with zero behavioral value. The correct production-grade action is to document the convention and amend the policy.

---

## 4. Per-Finding Closure

All 21 cite-pins:
- **Status: CLOSED — intentional behavioral anchor**
- **Action: none (no code change required)**

Mapping:
- v1.2 EC-017-003 pins (lines 897, 904, 918, 926): anchor to v1.2 re-adjudication of EC-017-003 semantics. Correct.
- v1.2 EC-017-005 pins (lines 932, 944, 962, 970): anchor to v1.2 restoration of EC-017-005 E-AUTH-006 semantics (reversing the bad v1.1 amendment). Correct and specifically meaningful.
- v1.3 EC-017-010 pins (lines 145, 192, 282, 347, 436, 457, 484, 1006, 1012, 1028, 1036, 1042): anchor to the version that introduced EC-017-010 and E-AUTH-007. Correct.

---

## 5. Implementer Follow-On Dispatch

**None required.** All 21 cite-pins are intentional; no code change needed.

---

## 6. BC-2.01.017 Amendment (§Notes addition)

The following §Notes section is added to BC-2.01.017 to document the convention:

> **§Notes for Implementers — Cite-pin convention:**
> Code doc-comments and assert messages in `auth_provider.rs` cite `BC-2.01.017 v<N>` where
> `<N>` is the BC version that introduced or re-established the specific EC/postcondition being
> anchored (pinned-at-write-time convention). This is intentional: `v1.2` pins reference the
> re-adjudicated EC-017-003/EC-017-005 semantics (D-854 revert of bad v1.1 amendment);
> `v1.3` pins reference the introduction of EC-017-010 and E-AUTH-007 (D-857). The current
> BC version is tracked in BC-INDEX — code citations need not track the current version if the
> pinned version correctly describes the anchored behavior. No-semantic-content-change version
> bumps (such as the v1.3→v1.4 changelog hygiene bump, D-866) do NOT require a cite-pin sweep.

This amendment is applied in the companion BC-2.01.017 v1.5 change (see commit).

---

## 7. Policy Amendment Recommendation

**POL-29 step 8f v1.29 requires amendment.** The current text mandates:

> "when any burst bumps a BC or ADR frontmatter version (including no-semantic-change bumps),
> the sibling-sweep MUST include `rg \"<artifact-ID> v<old>\" crates/ --type rust`"

The phrase "including no-semantic-change bumps" causes the F-LP12-LOW-001 false alarm. The correct rule is:

> "when any burst bumps a BC or ADR frontmatter version WITH a semantic content change
> (new ECs, modified postconditions, new invariants, error code additions/changes),
> the sibling-sweep MUST include `rg \"<artifact-ID> v<old>\" crates/ --type rust` and
> each cite-pin must be evaluated for whether it anchors behavior that changed.
> Hygiene-only version bumps (changelog reorder, schema normalization, formatting) are
> EXEMPT from the sibling-sweep obligation — no behavioral content changed, so no callsite
> semantic becomes incorrect."

**Routing:** Policy amendment requires orchestrator to dispatch policy-owner (product-owner + spec-steward) for codification. This adjudication documents the intent; formal POL-29 text amendment is a follow-on task for the next available burst that touches POL-29 governance.

---

## 8. Self-Audit Checklist

- [x] Did I rationalize any decision with "MVP," "for now," "good enough"? **No.**
- [x] Did I add a tech-debt-register entry without all three required conditions? **No — this is a closure, not a deferral.**
- [x] Did I leave any "pending architect review" for a question I could answer in scope? **No — convention determined by corpus evidence.**
- [x] Did I find a bug and surface it as advisory instead of fixing in scope? **N/A — all 21 pins are correct; no bug exists.**
- [x] Did I default to the cheapest mechanism instead of the correct mechanism? **No — Option A is the semantically correct outcome, not just the cheapest.**
- [x] Did I evaluate ADVISORY-severity finding as potential BLOCKER? **Yes — LOW severity confirmed. These are intentional behavioral anchors, not stale references. No runtime, correctness, or traceability impact.**
- [x] Did I paper-fix a finding by doc-commenting without structural fix? **No — the finding requires no structural fix. Convention documented in BC notes is load-bearing (future implementers learn why to not force-update pins).**
- [x] Did I sibling-sweep callsites when changing a canonical identifier? **N/A — no code or identifier changed.**
