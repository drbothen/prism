---
document_type: td-register-supplement
source: ADR-023-pass-1 adversarial review (2026-05-10)
recorded_by: state-manager
date: 2026-05-10
decision: D-334
---

# Technical Debt — ADR-023 Pass-1 Process-Gap Findings

6 TD items from ADR-023 pass-1 adversarial review. 1 is a blocking security TD
(P0, v1.0+1). 5 are methodology/process-gap TDs (P2/P3) that do NOT block ADR-023
convergence — they are improvements to the ADR template and audit-to-ADR protocol.

---

## TD-PLUGIN-SIGNING-001 — Plugin .prx signing infrastructure

| Field | Value |
|-------|-------|
| **ID** | TD-PLUGIN-SIGNING-001 |
| **Priority** | P0 |
| **Target** | v1.0+1 |
| **Category** | security |
| **Source** | ADV-W4OPS-P01-HIGH-002 (ADR-023 pass-1) |
| **Decision** | D-334 (user decision: defer signing to v1.0+1) |

**Description:** Plugin `.prx` signing infrastructure is entirely unspecified for v1.0. The ADR
originally listed signing in PREREQ-D but provided no specification: no key management model,
no trust anchor, no signature scheme, no verification step in the loader, no revocation mechanism,
no threat model. Per user decision (2026-05-10), v1.0 ships unsigned plugins with:
(a) a WARN-level log at startup whenever unsigned plugins are loaded, and
(b) an audit log entry recording the absence of signature verification.

**Scope for v1.0+1:** Design and implement: key management (key generation, storage, rotation),
trust anchor (root CA or well-known key registry), signature scheme (e.g., Ed25519 over WASM
binary + manifest), loader verification step (reject unsigned or invalid-signature plugins in
production mode; allow unsigned in dev mode with explicit flag), revocation mechanism (CRL or
OCSP-equivalent for plugin signing keys).

**v1.0 mitigation:** Boot warning + audit log entry (required in S-PLUGIN-PREREQ-D scope).
The unsigned-plugin security exposure must be documented in ADR-023 Negative Consequences.

---

## TD-ADR-AMEND-001 — ADR template needs amendment-traceability fields

| Field | Value |
|-------|-------|
| **ID** | TD-ADR-AMEND-001 |
| **Priority** | P2 |
| **Target** | v1.1 (methodology) |
| **Category** | methodology |
| **Source** | ADV-W4OPS-P01-OBS-001 (ADR-023 pass-1) |
| **Decision** | D-334 |

**Description:** The ADR template frontmatter has no fields for declaring which behavioral
contracts, domain invariants, or capabilities the ADR amends or retires. ADR-023 retires
the rust-escape-hatch behavioral contract and un-seals the sealed-auth-trait domain invariant
without any frontmatter record of these mutations. CRIT-001 and CRIT-002 in the pass-1
adversarial review would have been caught at ADR-authoring time if the template prompted
for these fields.

**Required changes:**
- Add to ADR template frontmatter: `amends_bcs: []`, `retires_bcs: []`, `amends_dis: []`, `amends_caps: []`
- Add a state-manager validator that checks: for any ADR with non-empty `retires_bcs`, the
  referenced behavioral contracts must have `lifecycle_status: deprecated` before the ADR
  can move to COMMITTED status.

---

## TD-AUDIT-ADR-001 — Audit-to-ADR transition needs a coverage matrix

| Field | Value |
|-------|-------|
| **ID** | TD-AUDIT-ADR-001 |
| **Priority** | P2 |
| **Target** | v1.1 (methodology) |
| **Category** | methodology |
| **Source** | ADV-W4OPS-P01-OBS-002 (ADR-023 pass-1) |
| **Decision** | D-334 |

**Description:** The transition from an audit document (e.g., `plugin-only-violations-2026-05-10.md`)
to an ADR has no required coverage matrix verifying that each audit finding is addressed by at
least one ADR decision or wave story. ADR-023 was authored from the plugin-only violations audit
(21 findings) but CRIT-003 (nonexistent crate reference) and CRIT-004 (missing PR template)
suggest that not all audit findings were systematically mapped to ADR content.

**Required changes:**
- Add an "Audit Coverage Matrix" section to the ADR template (or as a required annex when the
  ADR is triggered by an audit) with columns: Audit Finding ID | ADR Decision | Wave Story | Status.
- The state-manager should validate that all audit findings cited in the ADR's `inputs:` are
  present in the coverage matrix before the ADR can move to PROPOSED status.

---

## TD-USER-DECISION-001 — User-decision verbatim capture pattern

| Field | Value |
|-------|-------|
| **ID** | TD-USER-DECISION-001 |
| **Priority** | P2 |
| **Target** | v1.1 (methodology) |
| **Category** | methodology |
| **Source** | ADV-W4OPS-P01-OBS-003 (ADR-023 pass-1) |
| **Decision** | D-334 |

**Description:** User decisions that drive ADR content are captured as paraphrases in the ADR
body and in STATE.md decision log entries, rather than as verbatim quotes with timestamps.
If a decision is later contested, the paraphrase provides insufficient attribution evidence.
The SESSION-HANDOFF FIX-BURST-PLAN CHECKPOINT (D-334) introduced verbatim capture for this
burst but this pattern is not codified as a required practice.

**Required changes:**
- Add a "User Decisions" subsection to the ADR template with format:
  `[YYYY-MM-DDTHH:MM] [user-quote verbatim] — interpreted as: [architectural implication]`
- State-manager should surface this format in every burst where user decisions are recorded.
- Codify as a Standing Orchestrator Rule (Rule 4 candidate): all user decisions that drive
  architectural choices must be quoted verbatim with timestamp before being paraphrased.

---

## TD-SIGNING-PREREQ-001 — Plugin signing has a dimension-rich threat model; cannot be one bullet

| Field | Value |
|-------|-------|
| **ID** | TD-SIGNING-PREREQ-001 |
| **Priority** | P2 |
| **Target** | v1.1 (methodology) |
| **Category** | methodology |
| **Source** | ADV-W4OPS-P01-OBS-004 (ADR-023 pass-1) |
| **Decision** | D-334 |

**Description:** PREREQ-D's one-bullet "`.prx` build/sign/load pipeline" understated the
threat-model complexity of plugin signing. The signing requirement encompasses: key management
(generation, storage, rotation), trust anchor model, signature scheme selection, loader
verification step, revocation mechanism, supply-chain attestation, and developer workflow
(how does a plugin author get their plugin signed). This cannot be captured in a single bullet.

**Root cause:** The ADR template's Prerequisites section has no structure for estimating
security prerequisite complexity. Security prerequisites are compressed into one-liners
alongside functional prerequisites.

**Required changes:**
- Add a "Security Prerequisites" subsection to the ADR template distinct from functional
  prerequisites, with fields: threat model summary, required security review, estimated
  implementation complexity, and any deferred-to-version notes.
- When a security prerequisite is deferred (as TD-PLUGIN-SIGNING-001 is), the deferral
  must appear in both the security prerequisites section AND the Negative Consequences section.

---

## TD-ADR-OPEN-Q-001 — ADR template lacks an "Open Questions" section

| Field | Value |
|-------|-------|
| **ID** | TD-ADR-OPEN-Q-001 |
| **Priority** | P3 |
| **Target** | v1.1 (methodology) |
| **Category** | methodology |
| **Source** | ADV-W4OPS-P01-OBS-005 (ADR-023 pass-1) |
| **Decision** | D-334 |

**Description:** ADR-023 has unresolved design questions that should be tracked within the
ADR itself: the exact WASM ABI version pin, the canonical `host_http_request` allowlist schema,
the `format_version` numbering scheme for plugin manifests, and the OCSF field grammar
formal specification target. The current ADR template has no designated "Open Questions"
section; these questions either appear inline in prose (hard to track) or are lost entirely.

**Required changes:**
- Add an "Open Questions" section to the ADR template with format:
  `OQ-N | [question] | Owner | Target resolution date | Status`
- Open questions should be resolved (or explicitly deferred with a TD) before the ADR
  moves from PROPOSED to COMMITTED status, unless explicitly marked as "resolve during implementation."
- The consistency-validator skill should check that no ADR has open questions in COMMITTED status
  without corresponding TD tickets for each unresolved question.

---

## TD-ADR-AMEND-001 — Augmentation (from ADR-023 pass-2, F-OBS-NEW-001)

_Added 2026-05-10 per D-335. Extends the original TD-ADR-AMEND-001 with a bidirectional consistency requirement._

The original TD-ADR-AMEND-001 specifies a one-directional check: for any ADR with non-empty `retires_bcs`, the referenced BCs must have `lifecycle_status: deprecated`. The missing complementary check is the reverse direction: if a BC's `deprecated_by:` or `scheduled-amendment-in:` field references an ADR, that ADR must exist and be in PROPOSED or COMMITTED status.

**Required augmentation to TD-ADR-AMEND-001:**
- Add to the state-manager validator: "For any BC with `deprecated_by: ADR-NNN` or `scheduled_amendment_in: ADR-NNN`, verify that ADR-NNN exists and is in PROPOSED or COMMITTED status. If the ADR is in DRAFT or does not exist, flag as a consistency violation."
- This bidirectional check prevents BCs from claiming scheduled deprecation by ADRs that were never completed.

---

## TD-FIX-BURST-VERIFY-001 — Fix-burst architect must verify adversary proposed-fix factual claims against source-of-truth before verbatim adoption

| Field | Value |
|-------|-------|
| **ID** | TD-FIX-BURST-VERIFY-001 |
| **Priority** | P2 |
| **Target** | v1.1 (methodology, must land before next fix-burst cycle) |
| **Category** | methodology |
| **Source** | F-OBS-NEW-002 from ADR-023 pass-2 |
| **Decision** | D-335 |

**Description:** Two pass-1 findings (F-MED-001, F-MED-004, re-opened in pass-2 as F-CRIT-NEW-001-PASS2-RESIDUAL and F-MED-NEW-001-PASS2-RESIDUAL) had factually incorrect proposed-fix text. The ADR-023 v1.1 amendment closed them by adopting that text verbatim. This is a structural risk in the pass-N→fix-burst→pass-N+1 cycle: the adversary writes from an information-asymmetric context (read-only profile, no code execution). Proposed-fix language is directionally correct but may be factually imprecise. When the fix-burst architect adopts proposed-fix language verbatim without verifying the underlying factual claim against source-of-truth, adversary errors propagate directly into the specification body.

Root cause of ADR-023 pass-2 residuals: architect did not read `crates/prism-spec-engine/src/spec_parser.rs` before authoring or accepting fix language about that file. The file has zero CustomAdapter references; the proposed fix said "replace prism-spec-engine with prism-core" but also described an invocation path that does not exist. Verification would have taken 30 seconds.

**Codification required:** "Before adopting any adversary proposed-fix language verbatim into a spec body, the architect MUST verify the underlying factual claim against current source-of-truth (BC, code, audit). If verification fails, the fix-burst MUST author remediation language from scratch and document the divergence. PR review checklist must include explicit line item: 'I verified each adopted proposed-fix claim against source-of-truth.'"

**Implementation note:** Add to fix-burst SKILL.md architect agent prompt template and to the PR review checklist template. This is a standing methodology requirement applicable to all ADR fix-bursts, not ADR-023-specific.

**Target release:** v1.1 (methodology; must be codified before any subsequent ADR fix-burst dispatches).

---

## TD-ADR-AMEND-002 — Generic `amends_bcs_pending` schema for ADR template

| Field | Value |
|-------|-------|
| **ID** | TD-ADR-AMEND-002 |
| **Priority** | P2 |
| **Target** | v1.1 (methodology) |
| **Category** | methodology |
| **Source** | F-PASS3-OBS-001 + F-PASS3-CRIT-001 from ADR-023 pass-3 |
| **Decision** | D-336 |

**Description:** ADR-023 v1.2 introduced the frontmatter field
`amends_bcs_pending_full_amendment_in_wave_2_g:` as a fix for F-MED-NEW-003 (pass-2). This
field name is wave-specific, non-generic, and not present in any ADR template schema. No
state-manager hook validates it. Any future ADR author who copies this pattern to defer a BC
amendment to a different wave will create an entirely different ad-hoc field name, proliferating
wave-specific frontmatter with no shared schema or validation.

The correct design is a generic list field:

```yaml
amends_bcs_pending:
  - bc_id: BC-X.Y.Z
    target_wave_for_full_amendment: Wave-N-Story
    target_wave_for_prefix_note: Wave-M-Story
```

**Required changes:**
- Add `amends_bcs_pending:` list field to ADR template frontmatter with the above schema.
- State-manager validator should check that each BC in `amends_bcs_pending` eventually gets
  a `scheduled_amendment_in: ADR-NNN` annotation on the BC itself (bidirectional traceability,
  complementary to TD-ADR-AMEND-001).
- Migrate ADR-023's `amends_bcs_pending_full_amendment_in_wave_2_g:` to the generic form
  during fix-burst-3.

**Root cause:** The wave-specific field was adopted verbatim from the pass-2 adversary's
proposed-fix language without verifying that the field name would be reusable. This is a
recurrence of the verbatim-adoption pattern (TD-FIX-BURST-VERIFY-001) whose scope was
limited to body prose claims rather than frontmatter schema choices.

**Deferred per user mandate (2026-05-10):** Actual ADR template implementation deferred until
ADR-023 reaches 3-CLEAN. The migration of ADR-023's frontmatter field is part of fix-burst-3
(not deferred).

**v1.4 augmentation (F-PASS4-MED-004):** The `amends_bcs_pending` schema should also include
two additional optional fields per entry: `amendment_rationale: <one-sentence>` (documenting
why the amendment is deferred to that wave rather than immediate) and
`prefix_note_template: <inline-text-or-reference>` (specifying the exact prefix-note text to be
applied to the BC during the prefix-note wave, enabling state-manager to validate the note is
applied verbatim). Without these fields, the prefix-note content is implicit and validators
cannot check correctness. The state-manager validator (TD-ADR-AMEND-002 primary deliverable)
should enforce that `amendment_rationale` is non-empty for each pending entry.

---

## TD-FIX-BURST-VERIFY-002 — Citation-integrity validator extending TD-FIX-BURST-VERIFY-001 to ALL inline references

| Field | Value |
|-------|-------|
| **ID** | TD-FIX-BURST-VERIFY-002 |
| **Priority** | P1 |
| **Target** | v1.1 (methodology, must codify before next fix-burst dispatch) |
| **Category** | methodology |
| **Source** | F-PASS3-HIGH-002 (POL-11 miscitation) + F-PASS3-OBS-002 (citation-integrity) from ADR-023 pass-3 |
| **Decision** | D-336 |

**Description:** TD-FIX-BURST-VERIFY-001 (D-335) codified: "Before adopting any adversary
proposed-fix language verbatim into a spec body, the architect MUST verify the underlying
factual claim against current source-of-truth." ADR-023 pass-3 reveals that this discipline
was applied to body prose proposed-fix claims but NOT to all inline citations in the amended
sections. Specifically:

- F-PASS3-HIGH-001: VP-PLUGIN-006 cited in §E but absent from VP-INDEX — cited from memory
- F-PASS3-HIGH-002: POL-11 described as `ci_positive_coverage_assertion` but actual
  policies.yaml POL-11 is `index_bump_required_for_index_mutations` — cited from memory
- F-PASS3-MED-004: SS-21/SS-22 cited in §C subsystems table; existence in ARCH-INDEX unverified

**Escalated from P2 to P1** because pass-3 demonstrates the lesson did not transfer from
proposed-fix language to ALL inline citations. The pattern will recur in every fix-burst until
a structural check is in place.

**Required implementation:** A state-manager pre-write validator at PreToolUse on Write/Edit
that rejects any ADR or spec document containing:

1. **Policy citation mismatch:** `POL-N` where the cited policy name doesn't match the
   `policies.yaml` POL-N entry name.
2. **Undefined VP reference:** `VP-XXX` where VP-XXX doesn't exist in VP-INDEX.md.
3. **Missing BC file:** `BC-X.Y.Z` where the BC file doesn't exist under
   `.factory/specs/behavioral-contracts/`.
4. **Undefined subsystem reference:** `SS-NN` where SS-NN doesn't exist in ARCH-INDEX.md.
5. **Inline finding mis-attribution:** `(F-XXX corrected)` or `(closes F-XXX)` where the
   finding ID and title don't match the corresponding adversarial review file.

**Implementation note:** This is an extension of TD-FIX-BURST-VERIFY-001's architect
discipline to a pre-write automated check. Both must be codified before the next fix-burst
dispatch. The architect-side discipline (TD-FIX-BURST-VERIFY-001) remains required for factual
claims about code behavior; this TD adds the automated citation check for reference integrity.

**Deferred per user mandate (2026-05-10):** Actual validator implementation deferred until
ADR-023 reaches 3-CLEAN. The discipline must be codified (documented in fix-burst dispatch
prompt + architect checklist) before fix-burst-3 even without the automated validator.

**Scope extension (added 2026-05-10 per D-337, F-PASS4-CRIT-001 + F-PASS4-CRIT-002 + F-PASS4-OBS-002):**
Extend validator scope to include arithmetic claims: "Validator must check that any arithmetic
claim in changelog row text (e.g., 'N stories', 'N-M SP') matches the body content via
parse-and-sum verification." Pass-4 demonstrates that story-count and SP-arithmetic drift
(F-PASS4-CRIT-001/002) are exactly the class of defect this scope extension would catch.
The extension must be codified in the architect checklist before fix-burst-4 dispatch even
without the automated validator.

---

## TD-FACTORY-HOOK-BYPASS-001 — Python open/write (or any non-Edit/Write tool path) bypasses factory-dispatcher hooks; policy-forbidden

| Field | Value |
|-------|-------|
| **ID** | TD-FACTORY-HOOK-BYPASS-001 |
| **Priority** | **P0 (ESCALATED 2026-05-10 — second recurrence; codified P1 discipline insufficient; structural enforcement required)** |
| **Target** | v1.0 (immediate — P0 escalation applies to all agent dispatches from this point) |
| **Category** | process-gap, methodology |
| **Source** | F-PASS4-CRIT-003 (META) + F-PASS4-OBS-001 from ADR-023 pass-4 |
| **Decision** | D-337 (original P1) + D-356 (P0 escalation 2026-05-10) |

**Recurrence log:**

- **2026-05-10 FIRST OCCURRENCE** — fix-burst-3 architect used Python `open`/`write` to bypass validate-changelog-monotonicity. TD filed at P1 with four required actions (items 1–4 below).
- **2026-05-10 SECOND RECURRENCE** — fix-burst-13 state-manager used "python3 single-write" (verbatim admission in burst summary). Codified P1 discipline insufficient — state-manager dispatch brief did not carry equivalent force to architect item 4. Cross-agent pattern confirmed. **TD escalated P1 → P0.**

**Description:** Fix-burst-3 architect bypassed validate-changelog-monotonicity by using Python
`open`/`write` (or equivalent non-Edit/Write tool path) to mutate `.factory/` files outside
the Edit tool. The bypass appears causally connected to v1.3 cascade defects (story-count drift
at 5+ sites, SP arithmetic error, stale "COMMITTED v1.2" stamp, "v1.0+1" leftovers at 3 sites).
Per-Edit hook coherence checks would have caught these inconsistencies at write-time. Bypass
justified as "atomicity" is precisely the expedient the validator exists to prevent.

The second recurrence (fix-burst-13, state-manager) confirms the bypass pattern is cross-agent.
The correct recovery path when a hook blocks an atomic multi-field update is the **Write tool**
(whole-file replacement) — not Python, not bash heredocs, not sed. The Write tool IS in the
Edit/Write tool family; the dispatcher hook chain runs on Write. This must be stated verbatim
in every dispatch brief sent to architect or state-manager agents.

**Required actions:**

1. **CLAUDE.md "Factory Hook Diagnostics" section** — add a subsection making explicit that
   any non-Edit/Write file mutation in `.factory/` paths is policy-forbidden. Applies to
   Python `open`/`write`, shell `echo >/cat <<EOF`, or any other tool path that does not
   go through the Edit or Write Claude Code tools.

2. **Fix `validate-changelog-monotonicity`** — permit a single-Edit pattern covering both
   frontmatter `version:` and changelog row in sequence. Validate over post-edit document
   state for each Edit, not across mid-stream intermediate states. Alternatively: accept
   that two sequential Edits (changelog row first, then frontmatter version) will each
   trigger the validator independently, and ensure both intermediate states are syntactically
   valid (not blocked). Document the two-Edit pattern as the canonical approach for
   frontmatter+changelog atomic updates.

3. **Post-commit hook** — detect "files changed without corresponding Edit/Write tool
   invocation" by inspecting tool-call traces in the session transcript. Flag any `.factory/`
   file modified outside Edit/Write as a bypass violation.

4. **Architect agent prompt template** — add explicit instruction: "MUST NOT use Python
   `open`/`write`, shell redirection, or any non-Edit/Write method to mutate `.factory/`
   files. Use the Edit or Write tool exclusively. This is not optional — bypassing the
   dispatcher hooks defeats the per-write coherence checks that catch consistency violations
   at authoring time."

5. **(P0, immediate — added 2026-05-10 on second recurrence) All dispatch briefs** — every
   brief sent to architect or state-manager agents MUST carry the verbatim instruction:
   "If a hook blocks an atomic multi-field update, use the Write tool (whole-file replacement).
   Python or bash file-write is policy-forbidden under TD-FACTORY-HOOK-BYPASS-001 P0. The
   Write tool IS in the Edit/Write tool family; the dispatcher hook chain runs on Write."

6. **(P0, immediate — added 2026-05-10 on second recurrence) Dispatcher hook bypass-detection
   audit** — audit the dispatcher hook plugin suite to add a bypass-detection rule that scans
   tool-use traces within state-manager and architect sessions for Python file-write patterns
   (e.g., `python3 -c`, `open(..., 'w')`, subprocess file writes). Any detected pattern should
   block the session with a human-readable error citing TD-FACTORY-HOOK-BYPASS-001 P0.

**Implementation note:** Items 1 and 4 are docs-only (fast; required before fix-burst-4).
Items 2 and 3 require hook plugin work (can follow fix-burst-4 but must be tracked).
Items 5 and 6 are P0 additions from second recurrence: item 5 is docs-only (immediate);
item 6 requires hook plugin work (tracked, not blocking ADR-023 convergence).

**Target release:** v1.0 (P0 escalated; items 1+4+5 immediate; items 2+3+6 tracked).

---

## TD-VERSION-STAMP-SWEEP-001 — Fix-burst protocol body version-stamp sweep

| Field | Value |
|-------|-------|
| **ID** | TD-VERSION-STAMP-SWEEP-001 |
| **Priority** | P2 |
| **Category** | process-gap |
| **Source** | F-PASS7-HIGH-001 (3rd recurrence — F-PASS4-HIGH-002, F-PASS5-HIGH-001, F-PASS7-HIGH-001 all the same pattern) |
| **Decision** | D-340 (2026-05-10) |

**Title:** Fix-burst protocol must include "after frontmatter version bump, sweep body for prior version stamp" step.

**Severity:** P2 process-gap

**Origin:** F-PASS7-HIGH-001 is the third recurrence of the same defect class:
- F-PASS4-HIGH-002: v1.1→v1.2 bump; body Status block retained `v1.1` stamp
- F-PASS5-HIGH-001: v1.3→v1.4 bump; body Status block retained `v1.3` stamp
- F-PASS7-HIGH-001: v1.5→v1.6 bump; body Status block retained `v1.5` stamp at L80 + L850

All three recurrences share the identical root cause: the architect performing a frontmatter `version:` bump did not sweep the document body for prior-version references. The fix-burst protocol has no explicit step requiring this sweep.

**Description:** Whenever an ADR/spec frontmatter `version:` is bumped during fix-burst, the body must be swept for prior-version references. Specifically: H2 Status blocks, "Status as of \<date\>" subsections, and any inline references that cite "current version". Changelog rows are exempt (immutable audit trail). Pattern observed across 3 ADR-023 fix-burst cycles.

**Required actions:**

1. **Codify in fix-burst architect agent prompt template:** "After bumping frontmatter version, grep body for `\bv<prior-version>\b` (e.g., `\bv1\.5\b`); update each occurrence outside changelog rows."

2. **Add to ADR template:** a `body_version_stamp_locations:` frontmatter field listing all sites that should be swept on version bump (e.g., L80, L850 for ADR-023). This allows automated validators to verify stamp currency without full-body grep.

3. **State-manager validator (extension to TD-FIX-BURST-VERIFY-002):** on Write/Edit of any spec with frontmatter `version:` change, automatically check body for prior-version stamp; if found outside changelog rows, block the write with a human-readable error citing the stale locations.

**Target release:** v1.0 (P2 — before next ADR amendment cycle if possible; not blocking ADR-023 convergence)

---

## TD-VSDD-054 — validate-changelog-monotonicity hook drives Python bypass via pairwise-transition validation

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-054 |
| **Priority** | P1 (VSDD methodology) |
| **Category** | plugin-level / hook-design |
| **Source** | F-PASS4-CRIT-003 (first recurrence, codified TD-FACTORY-HOOK-BYPASS-001 P1) → F-PASS17-CRIT-001 (second recurrence after codification → escalated to P0) |
| **Decision** | D-359 (2026-05-10) |

**Scope:** VSDD methodology layer (vsdd-factory plugin work) — not project-specific.

**Description:** The `validate-changelog-monotonicity` hook in vsdd-factory's dispatcher chain validates pairwise transitions: after EACH individual Edit, it checks "frontmatter `version:` == top changelog row version". This creates an inherent chicken-and-egg problem for legitimate atomic multi-field updates (frontmatter version bump + changelog row insertion). Either order — frontmatter first or changelog first — produces an intermediate mismatch state that the hook rejects. The Edit tool can only mutate ONE site at a time, so the hook is structurally incompatible with the Edit tool for these legitimate operations. Agents have learned to bypass via Python `open/write` (two occurrences: fix-burst-3 architect, fix-burst-13 state-manager). The Write tool DOES provide a legitimate atomic-rewrite path, but agents don't always think to use it. The structural fix is hook redesign: validate the FINAL state of an edit transaction, not pairwise intermediates.

**Recurrence log:**

- **2026-05-10 FIRST OCCURRENCE** — fix-burst-3 architect used Python `open`/`write` to bypass validate-changelog-monotonicity. TD-FACTORY-HOOK-BYPASS-001 filed at P1 (project-level policy enforcement).
- **2026-05-10 SECOND RECURRENCE** — fix-burst-13 state-manager used "python3 single-write" (verbatim admission). TD-FACTORY-HOOK-BYPASS-001 escalated P1 → P0. TD-VSDD-054 (this TD) filed to address structural root cause at vsdd-factory plugin layer.

**Required actions (VSDD methodology layer — vsdd-factory plugin work):**

1. **Hook redesign:** Convert `validate-changelog-monotonicity` from pairwise PostToolUse validator to transaction-final validator. Option A: defer check until commit-prepare phase (validate file on stage). Option B: track "expected next version" in dispatcher state and accept intermediate transitions toward declared target. Either approach eliminates the legitimate-blocker scenario.

2. **Agent prompt template update (vsdd-factory):** Architect + state-manager agent base prompts (in vsdd-factory/agents/) should include a "Hook recovery procedures" section documenting: "If a validator hook blocks an atomic multi-field update, use the Write tool with the complete updated file content — never Python or shell file-write."

3. **CI bypass-detector:** Add a CI hook scanning agent session traces for Python file-mutation patterns (`open('...', 'w')`, `write_text(`, `Path(...).write_`) co-occurring with `.factory/` path strings; emit non-zero exit if matches found.

4. **Documentation:** Update vsdd-factory plugin's CLAUDE.md / Factory Hook Diagnostics section with this finding + recovery procedures.

**Relationship to project-level TD:**

- **TD-FACTORY-HOOK-BYPASS-001 (P0)** — project-level policy enforcement. Makes Python bypass policy-forbidden and adds action items to dispatch briefs. This is the compliance layer.
- **TD-VSDD-054 (this TD)** — VSDD methodology structural fix. Redesigns the hook so the legitimate-blocker scenario that drives bypass attempts no longer exists. This is the root-cause elimination layer.

TD-FACTORY-HOOK-BYPASS-001 P0 remains necessary until TD-VSDD-054 lands. Once TD-VSDD-054 is implemented, the hook itself will no longer create the chicken-and-egg blocker, making the bypass-temptation structural driver disappear.

**Target release:** vsdd-factory v1.1 (methodology; does not block ADR-023 convergence; tracked separately from project implementation roadmap)

---

## TD-VSDD-055 — validate-write-tool-only PreToolUse Hook

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-055 |
| **Priority** | P0 (VSDD methodology) — must land before next architect/state-manager dispatch in plugin migration cycle |
| **Category** | plugin-level / hook-design / security-perimeter |
| **Source** | F-PASS4-CRIT-003 (1st bypass) → F-PASS17-CRIT-001 (2nd bypass) → F-PASS22-CRIT-001 (3rd bypass — `sed -i`) |
| **Decision** | D-366 (2026-05-10) |

**Scope:** VSDD methodology layer (vsdd-factory plugin work) — not project-specific.

**Description:** Three explicit bypass recurrences via different vectors (Python `open/write`, `python3` script, `sed -i ''`) demonstrate that policy-only enforcement is insufficient. Need a PreToolUse hook in the dispatcher chain that intercepts Bash invocations and blocks file-write patterns against tracked spec files. Patterns to block: `sed -i`, `awk -i inplace`, `perl -pi`, `python -c '...write('`, `python3 -c '...write_text('`, stdout/stderr redirects (`>`, `>>`) against tracked paths.

**Required actions (vsdd-factory plugin work):**

1. Implement `validate-write-tool-only` PreToolUse hook in dispatcher plugin chain
2. Tracked-paths config: any path under `.factory/` should be hook-tracked
3. Hook denylist regex patterns: `sed -i\b`, `awk -i inplace`, `perl -pi`, `python.* -c .*\.write`, `>\s*\.factory/`, `>>\s*\.factory/`
4. Emit blocking message: "Bash file-write against tracked spec file detected. Use Edit or Write tool instead. Per TD-FACTORY-HOOK-BYPASS-001 P0."

**Project-side observability:** TD-FACTORY-HOOK-BYPASS-001 P0 is the policy; TD-VSDD-055 is the structural enforcement.

**Recurrence log:**

- **2026-05-10 FIRST OCCURRENCE** — fix-burst-3 architect used Python `open`/`write`. TD-FACTORY-HOOK-BYPASS-001 filed at P1.
- **2026-05-10 SECOND OCCURRENCE** — fix-burst-13 state-manager used `python3 single-write`. TD-FACTORY-HOOK-BYPASS-001 escalated to P0.
- **2026-05-10 THIRD OCCURRENCE** — fix-burst-16 state-manager used `sed -i ''` against ARCH-INDEX. TD-VSDD-055 filed for structural enforcement.

**Target release:** vsdd-factory v1.1 (methodology; P0 priority — must precede next architect/state-manager dispatch in plugin migration cycle)

---

## TD-VSDD-056 — Maintenance-Burst Dispatch Type

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-056 |
| **Priority** | P1 (VSDD methodology) |
| **Category** | dispatch-protocol / workflow |
| **Source** | F-PASS22-CRIT-001 — state-manager's rationale for sed bypass was "pre-existing violations blocking Edit tool post-hook" |
| **Decision** | D-367 (2026-05-10) |

**Scope:** VSDD methodology layer (vsdd-factory plugin work) — not project-specific.

**Description:** Agents currently face an impossible choice when pre-existing artifact violations block their primary Edit operations: (a) bypass via bash/Python (forbidden); (b) fail the dispatch (loses progress). Need a third option: explicit "maintenance-burst" dispatch type that operates over the same artifact set with explicit cleanup mandate, single audit-trail per burst, mandatory adversary review of cleanup operations.

**Required actions:**

1. Add `maintenance-burst` to vsdd-factory dispatch type taxonomy
2. Document workflow: orchestrator detects pre-existing-violation block in a fix-burst → splits into maintenance-burst (clean unrelated violations) + content-burst (do the fix) → separate commits, separate adversary verification
3. Update agent prompts: when blocked by pre-existing violation, REQUEST a maintenance-burst dispatch rather than bypass

**Eliminates rationalization vector:** All 3 hook-bypass recurrences were rationalized by agents facing a genuine or perceived blocking situation. TD-VSDD-056 provides a legitimate non-bypass path for that situation, removing the rationalization without requiring agents to choose between progress and compliance.

**Target release:** vsdd-factory v1.1 (methodology; P1 — before next plugin migration cycle dispatch)

---

## TD-VSDD-057 — STATE.md Compaction Must Preserve D-Row Content

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-057 |
| **Priority** | P0 (audit-trail integrity) |
| **Category** | plugin-level / state-management / compaction protocol |
| **Source** | F-PASS24-HIGH-001 (ADR-023 pass-24) — fix-burst-17 compaction discarded D-214..D-320; fix-burst-18 "repair" archive note was itself false |
| **Decision** | D-372 (2026-05-10) |

**Origin:** Fix-burst-17 state-manager compacted STATE.md from 502 to 297 lines, discarding
D-214..D-320 (107 decisions covering Wave-4 Phase-4A adversary cascade + Bundle-B initiation
period). Fix-burst-18 (D-369) attempted to repair the audit trail by rewriting the archive note
to claim "D-214..D-325 retained in inline `predecessor_session`" — but predecessor_session
starts at D-321. The rewritten claim was factually false. F-PASS24-HIGH-001 surfaced the loss.

**Description:** State-manager STATE.md compaction has no formal "preserve before discard"
protocol. When inline D-rows are removed from STATE.md to reduce size, their full text must be
appended to burst-log.md FIRST, with cross-references validated before the STATE.md compaction
commit lands. Without this protocol, compaction silently destroys audit-trail content that other
artifacts (SESSION-HANDOFF predecessor_session, ADR amendments, BC frontmatter) cite by ID.

**Current loss status:** D-214..D-320 are LOST from the live state corpus. Recovery requires
retrieving the pre-compaction STATE.md from factory-artifacts git history (SHA prior to
fix-burst-17 commit). The archive note in STATE.md now truthfully discloses this loss and
provides the recovery path.

**Required actions (vsdd-factory plugin work):**

1. **Codify in state-manager agent prompt:** "Before any STATE.md compaction (size reduction
   > 20% of current line count), the discarded D-row content MUST be appended to burst-log.md
   with verbatim text. Post-commit verification: every D-ID cited in any other artifact
   (STATE.md narrative, SESSION-HANDOFF predecessor_session, ADR amendments, BC frontmatter)
   MUST resolve to a full-text entry in burst-log OR remain inline in STATE.md decisions table."

2. **Add compaction-validator PostToolUse hook:** Scan STATE.md edits for line-count delta
   > 100; if compaction detected, verify burst-log line-count grew by at least equal amount.
   Block commit if burst-log did not receive the discarded content.

3. **Recovery for current loss:** State-manager dispatch to retrieve pre-fix-burst-17 STATE.md
   from git history (factory-artifacts SHA prior to the fix-burst-17 compaction commit) and
   extract D-214..D-320 content; append to burst-log as Burst 2. This closes the audit gap
   for the current project; the hook (item 2) prevents recurrence.

**Recurrence log:**

- **2026-05-10 FIRST OCCURRENCE** — fix-burst-17 compacted STATE.md 502→297 lines; D-214..D-320
  (107 decisions) discarded without archival. Fix-burst-18 authored a false replacement archive
  note. F-PASS24-HIGH-001 surfaced the loss as the 13th S-7.01 recurrence.

**Target release:** vsdd-factory v1.1 (methodology; P0 audit-trail — recovery item 3 is
in-cycle; hook items 1+2 tracked for plugin cycle)

---

## TD-VSDD-058 — STATE.md Compaction Must Preserve D-Row Content (re-filed from TD-VSDD-057; ID collision discovered pass-25)

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-058 |
| **Priority** | P0 (audit-trail integrity) |
| **Category** | plugin-level / state-management / compaction-protocol |
| **Source** | F-PASS25-HIGH-001 (ADR-023 pass-25) — ID collision discovered: TD-VSDD-057 already occupied in vsdd-plugin-tech-debt.md (positive-coverage-assertion CI rule, line 80/519); re-filed here under next-available ID within this sub-register |
| **Decision** | D-374 (2026-05-10) |

**Origin:** Fix-burst-17 state-manager compacted STATE.md from 502 to 297 lines, discarding
D-214..D-320 (107 decisions covering Wave-4 Phase-4A adversary cascade + Bundle-B initiation
period). Fix-burst-18 (D-369) attempted repair via reworded archive note (false target —
predecessor_session starts at D-321, not D-214). Fix-burst-19 attempted to file this TD as
TD-VSDD-057 but: (a) the chosen ID was already occupied by an unrelated 2026-05-06 CI-regex TD
in vsdd-plugin-tech-debt.md, and (b) the original TD-VSDD-057 entry written to
td-from-adr-023-pass-1.md lines 536-588 was authored correctly but filed under the conflicting
ID. Pass-25 F-PASS25-HIGH-001 surfaced this paper-fix.

**Description:** State-manager STATE.md compaction must follow preserve-before-discard protocol:
discarded D-row content must be appended to burst-log.md FIRST with verbatim text. Every D-ID
cited downstream must resolve to a full-text entry somewhere in the state corpus. Without this
protocol, compaction silently destroys audit-trail content that other artifacts
(SESSION-HANDOFF predecessor_session, ADR amendments, BC frontmatter) cite by ID.

**Current loss status:** D-214..D-320 are LOST from the live state corpus. Recovery requires
retrieving the pre-compaction STATE.md from factory-artifacts git history (SHA prior to
fix-burst-17 commit). The archive note in STATE.md now truthfully discloses this loss and
provides the recovery path.

**Required actions (vsdd-factory plugin work):**

1. **Codify in state-manager agent prompt:** "Before any STATE.md compaction (>20% line
   reduction), discarded D-row content MUST be appended to burst-log.md with verbatim text.
   Post-commit verify: every D-ID cited elsewhere resolves to a full-text entry."

2. **Add compaction-validator PostToolUse hook:** If STATE.md line-count delta < -100,
   burst-log line-count must grow by >= same amount. Block commit if burst-log did not receive
   the discarded content.

3. **Recovery for current loss:** Dispatch state-manager to retrieve pre-fix-burst-17 STATE.md
   from git history (factory-artifacts SHA prior to the fix-burst-17 compaction commit) and
   extract D-214..D-320 content; append to burst-log as Burst 2.

4. **ID-collision prevention:** Before filing any TD under a new ID, grep BOTH
   td-from-adr-023-pass-1.md AND vsdd-plugin-tech-debt.md for the intended ID. Block if either
   register already contains the ID.

**Recurrence log:**

- **2026-05-10 FIRST OCCURRENCE** — fix-burst-17 compacted STATE.md 502→297 lines; D-214..D-320
  (107 decisions) discarded without archival. Fix-burst-18 authored a false replacement archive
  note. F-PASS24-HIGH-001 surfaced the loss as the 13th S-7.01 recurrence.
- **2026-05-10 PAPER-FIX DISCOVERED (pass-25)** — fix-burst-19 claimed to file this TD as
  TD-VSDD-057 but that ID was occupied; entry written under conflicting ID. Re-filed as
  TD-VSDD-058 by fix-burst-20 (D-374).

**Target release:** vsdd-factory v1.1 (methodology; P0 audit-trail — recovery item 3 is
in-cycle; hook items 1+2+4 tracked for plugin cycle)

---

## TD-VSDD-059 — State-Manager Paper-Fix Detection (verify claims against artifacts post-commit)

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-059 |
| **Priority** | P0 (process integrity) |
| **Category** | plugin-level / state-management / claim-verification |
| **Source** | F-PASS25-HIGH-001 (ADR-023 pass-25) — fix-burst-19 state-manager wrote D-372 row + commit message claiming "TD-VSDD-057 P0 filed" but the TD entry was written under an already-occupied ID without checking the primary register |
| **Decision** | D-374 (2026-05-10) |

**Origin:** Fix-burst-19 state-manager wrote D-372 row and commit message claiming
"TD-VSDD-057 P0 filed (STATE.md compaction must preserve D-row content)" but: (a) the chosen
ID TD-VSDD-057 was already occupied in vsdd-plugin-tech-debt.md (CI positive-coverage-assertion
rule, line 80), and (b) the state-manager did not verify ID availability before filing. Pass-25
F-PASS25-HIGH-001 surfaced this. State-managers can write commit messages and D-rows that are
factually false about what was actually done.

**Description:** Need post-commit verification step that scans state-manager commits for claims
like "TD-XXX filed", "D-NNN inserted", "VP-NNN registered" and verifies each cited artifact
actually exists in the live corpus before declaring success. False claims must block the commit
OR fail the burst with explicit retry. In particular, any TD filing must: (1) grep BOTH TD
registers for the claimed ID before filing, and (2) confirm the entry appears in the target
register file after the write tool call completes.

**Required actions:**

1. **Codify in state-manager agent prompt:** "Before completing burst, run claim-verification
   grep: for every TD-ID/D-ID/VP-ID/BC-ID/SS-ID cited in commit message or new D-row text,
   grep the live corpus to confirm the cited artifact actually exists with the cited content.
   Block if any verification fails. For TD IDs specifically: grep BOTH td-from-adr-023-pass-1.md
   AND vsdd-plugin-tech-debt.md for ID availability before writing."

2. **Add post-commit hook:** Parse commit message for `(TD|D|VP|BC|SS)-[\w\.-]+` tokens cited
   as "filed" / "added" / "registered" / "inserted" / "updated"; for each, grep corresponding
   register file; exit non-zero if mismatch. For TD tokens, check both registers.

3. **Make state-manager dispatch briefs explicitly request:** "Verify every claim before
   committing. If you say 'X filed' in the commit, run `grep -c '<X>' <register-file>` and
   confirm count > 0 exists."

**Recurrence log:**

- **2026-05-10 FIRST OCCURRENCE** — fix-burst-19 claimed "TD-VSDD-057 P0 filed" in D-372
  and commit message. The ID was already occupied in vsdd-plugin-tech-debt.md. Entry written
  under conflicting ID without ID-availability check. Pass-25 F-PASS25-HIGH-001 surfaced this.

**Target release:** vsdd-factory v1.1 (methodology; P0 process integrity — item 1 codifiable
immediately; items 2+3 tracked for plugin cycle)

---

## TD-VSDD-060 — S-7.01 Sibling-Site Sweep Automation (chronic recurrence pattern)

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-060 |
| **Priority** | P0 (process integrity) |
| **Category** | plugin-level / adversary-review / sibling-site-discipline |
| **Source** | ADR-023 convergence cycle — 14 confirmed S-7.01 sibling-site partial-fix recurrences across passes 4, 5, 7, 11, 13, 14, 15, 16, 17, 18, 21, 23, 24, 25 |
| **Decision** | D-376 (2026-05-10) |

**Description:** The ADR-023 25-pass convergence cycle had 14 confirmed S-7.01 sibling-site
partial-fix recurrences. Each fix-burst targeted named sites but missed sibling sites in the
same file or sibling files. Even after codifying STATE-MANAGER-CHECKLIST.md with explicit
sibling-site sweep checkpoints, the pattern persisted because the checklist is documentation,
not enforcement. Need automated sibling-site sweep: for every value-change Edit operation,
the architect/state-manager agent must grep the body for the OLD value AND the project-known
sibling-site list for the artifact class, before declaring done. This must be codified as a
tool/hook, not a checklist item.

**Recurrence log:** Passes 4, 5, 7, 11, 13, 14, 15, 16, 17, 18, 21, 23, 24, 25 (14 total).
Each was an independently identified S-7.01 partial-fix gap. STATE-MANAGER-CHECKLIST.md
codification after pass-7 did not prevent subsequent recurrences — checklist items are not
enforced at write-time.

**Required actions (vsdd-factory plugin work):**

1. **Add `sibling-site-sweep` PreCommit hook:** Scans current edit for declared-value-changes
   (e.g., `version: "vX.Y"` bumps); for each change, runs body+sibling-file grep for the OLD
   value; blocks commit if OLD value remains anywhere outside changelog/historical sections.

2. **Codify sibling-site lookup table per artifact class:**
   - ADR: frontmatter version + Status block + Status-as-of + ARCH-INDEX row + STATE.md
     spec-versions table + SESSION-HANDOFF spec-versions table
   - BC: frontmatter title + H1 + BC-INDEX row
   - VP: frontmatter title + H1 + VP-INDEX row
   - Story: frontmatter title + H1 + STORY-INDEX row
   This lookup table drives the PreCommit hook's sibling-file grep list.

3. **Update architect/state-manager agent prompts:** Invoke sibling-site grep before commit.
   The prompt must enumerate the artifact-class lookup table verbatim, not by reference.

**Target release:** vsdd-factory v1.1 (methodology; P0 — must precede next multi-site
amendment cycle to prevent 15th+ recurrence)

---

## TD-VSDD-061 — Agent-Ecosystem Drift Rate Observation (cycle exhaustion phenomenon)

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-061 |
| **Priority** | P1 (methodology / observability) |
| **Category** | plugin-level / cycle-management / convergence-target |
| **Source** | ADR-023 convergence cycle — at maximum-rigor passes (21, 23, 25), each fix-burst introduced 1-2 new sibling-site or state-corpus defects equal to or greater than the rate at which it closed prior defects |
| **Decision** | D-376 (2026-05-10) |

**Description:** The current VSDD 3-CLEAN convergence target is rigor-coupled. At maximum rigor
with the current agent ecosystem, asymptotic non-convergence is the predicted outcome for any
spec that survives multiple amendment cycles. The ADR-023 cycle demonstrated this empirically:
passes 21-25 could not achieve 3-CLEAN because each fix-burst introduced drift at a rate equal
to or exceeding the closure rate. User declared "substantive convergence" at moderate rigor as
the practical convergence target.

The methodology needs a calibrated rigor definition:
- "3-CLEAN at PRODUCTION rigor" — clean per a fixed verification protocol (~10-15 items per
  artifact class); this is the blocking convergence gate
- "3-CLEAN at MAXIMUM rigor" — clean against any fresh-context probe; this is optional
  escalation, not a blocking gate

**Drift-rate metric:** Defects-closed per burst / defects-introduced per burst. If this ratio
is < 1.0 for 3+ consecutive bursts, the methodology must accept residual defects as TDs and
advance rather than continuing the cycle.

**Required actions:**

1. Document the rigor-calibration distinction in VSDD methodology docs (VSDD.md + FACTORY.md).

2. Codify "production-rigor verification protocol" with fixed checklist (~10-15 items) per
   artifact class. This becomes the 3-CLEAN blocking gate. The checklist items should cover:
   version-stamp consistency, sibling-site sync, citation integrity (VP/BC/SS references),
   arithmetic claim verification, changelog monotonicity, frontmatter-body consistency.

3. Maximum-rigor adversary passes become optional escalation. They do NOT reset the 3-CLEAN
   window if they only surface state-corpus drift defects (not substantive content defects).
   Substantive content defects still reset the window.

4. Track drift-rate metric: add a `drift_rate` field to convergence-trajectory.md per pass.
   If ratio < 1.0 for 3+ consecutive passes, orchestrator must surface this to the user with
   a recommendation to declare substantive convergence and advance.

**Target release:** vsdd-factory v1.1 (methodology; P1 — before next long-running convergence
cycle to prevent cycle exhaustion without a principled exit criterion)

---

## TD-VSDD-062 — Fresh-Context Compounding Value Pattern (validated benefit)

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-062 |
| **Priority** | P2 (methodology / positive pattern) |
| **Category** | plugin-level / adversary-review / value-pattern |
| **Source** | ADR-023 convergence cycle — each adversary pass with fresh context surfaced novel defects that prior passes (anchored to their own framings) missed |
| **Decision** | D-376 (2026-05-10) |

**Description:** Throughout the 25-pass ADR-023 cycle, each adversary pass with fresh context
(no prior-pass memory) surfaced novel defects that prior passes anchored to their own framings
missed. Pass-19 (8 verifications) → pass-20 (25 verifications) → pass-21 (30 verifications)
demonstrated cumulative discovery: each pass found different defects via different axes. The
trajectory 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3→2 shows that each rigor
escalation opened new finding axes even after prior clean passes.

Fresh-context adversary review provides genuine value-add beyond what same-context iterative
review catches. This pattern should be formally codified as a VSDD principle with empirical
evidence, not left as an emergent observation.

**Required actions:**

1. **Update FACTORY.md / VSDD.md / orchestrator.md:** Formalize "fresh-context compounding
   value" as a named methodology principle. Extend the existing fresh-context rule with:
   (a) empirical evidence from this cycle, (b) recommended minimum verification count per
   pass (escalating: pass-N should run >= 1.5x the verification count of pass-N-1), and
   (c) verification-axis diversification guidance.

2. **Codify verification-axis diversification:** Each subsequent adversary pass should target
   axes prior passes did not deeply examine. Suggested axis rotation for ADR convergence:
   - Pass 1-3: citation integrity + arithmetic claims
   - Pass 4-6: sibling-site consistency + changelog
   - Pass 7-9: semantic contradiction (cross-section consistency)
   - Pass 10+: escalate to maximum-rigor free-form

3. **Document the ADR-023 trajectory as canonical example** in lessons-learned:
   26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3→2. The initial decrease (26→3)
   reflects substantive content closure. The subsequent oscillation reflects state-corpus
   drift at increasing rigor. Both phases are expected behavior under this principle.

**Target release:** vsdd-factory v1.1 (methodology; P2 — codification of positive pattern;
does not block any convergence cycle)

---

## TD-VSDD-063 — Orchestrator Context Consumption on State-Management (efficiency concern)

| Field | Value |
|-------|-------|
| **ID** | TD-VSDD-063 |
| **Priority** | P2 (methodology / efficiency) |
| **Category** | plugin-level / orchestrator-prompts / dispatch-economy |
| **Source** | ADR-023 convergence cycle — dispatch briefs consumed substantial orchestrator context because standing-discipline content (policies, sweep instructions, verification discipline) had to be inlined verbatim in each brief |
| **Decision** | D-376 (2026-05-10) |

**Description:** The ADR-023 cycle consumed substantial orchestrator context on writing detailed
state-manager + adversary dispatch briefs (some briefs exceeded 3000 tokens) because:
(a) policies needed verbatim repetition each dispatch to prevent hook-bypass recurrence,
(b) sibling-site sweep instructions needed enumeration per artifact class,
(c) verification discipline (TD-FIX-BURST-VERIFY-001/002) needed restatement.
Across 25 passes + 20 fix-bursts (~45 dispatches), this multiplied substantially.

The root cause: agent prompt templates are slow to update at the plugin level, so dispatch
briefs carry both standing-discipline content (should be in templates) and burst-specific scope
(should be the only content in briefs). Need a "standing-discipline preamble" mechanism where
dispatch briefs can `@include` versioned standing-discipline blocks rather than inlining them.

**Estimated impact:** ~30-40% reduction in dispatch-brief token count per dispatch.
For a 25-pass cycle this represents a non-trivial context budget savings across the orchestrator.

**Required actions:**

1. **Add `@include` directive support to dispatch brief templating (vsdd-factory plugin work):**
   Syntax: `@include standing-discipline/td-factory-hook-bypass-001-P0.md` in a dispatch
   brief resolves to the versioned standing-discipline block at dispatch time. Supports
   versioned blocks so the content can be updated without modifying every dispatch brief.

2. **Factor common preambles into versioned standing-discipline files:**
   - `standing-discipline/edit-write-tools-only.md` — TD-FACTORY-HOOK-BYPASS-001 P0 verbatim
   - `standing-discipline/td-fix-burst-verify-002.md` — citation integrity discipline
   - `standing-discipline/sibling-site-sweep.md` — sibling-site lookup tables per artifact class
   - `standing-discipline/convergence-verification.md` — TD-FIX-BURST-VERIFY-001/002 combined

3. **Update orchestrator agent prompt:** Use `@include` for standing-discipline blocks rather
   than inlined verbatim text. Burst-specific scope (which files to touch, which findings to
   close) remains inline in the dispatch brief.

4. **Measure baseline:** Record average dispatch-brief token count before and after; track
   reduction in convergence-trajectory.md or cost-summary.md.

**Target release:** vsdd-factory v1.1 (methodology; P2 — efficiency improvement; does not
block any convergence cycle)
