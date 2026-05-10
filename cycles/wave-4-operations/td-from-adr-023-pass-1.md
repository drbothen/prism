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
