---
document_type: adversarial-review
cycle: phase-2-patch
pass: 24
status: findings-open
novelty: MEDIUM
findings: 3
critical: 0
high: 2
medium: 1
low: 0
previous_pass: 23 (7 findings: 4 HIGH, 1 MED, 2 LOW — all closed Burst 24)
convergence_counter: 0 of 3
---

# Pass 24 — Policy 9 arithmetic drift + S-5.10 AC trace mis-anchoring

Note: current-cycle passes 12–23 logged inside /Users/jmagady/dev/prism/.factory/STATE.md under Burst-N summaries; this is the first per-pass file of the Phase-3 patch cycle.

Review scope: verified Burst-24 closures (SS-07 rename, VP-039 propagation, DI-026 + BC-2.05.011 traces, PRD §5 regen, StorageDomain 16 variants); independently audited VP-INDEX v1.3 (39 VPs) against verification-architecture.md and verification-coverage-matrix.md; sampled BC frontmatter/body consistency for BC-2.05.011, BC-2.07.*, BC-2.17.*, BC-2.18.*; verified STORY-INDEX Wave 5 arithmetic (48 = sum of S-5.01..S-5.10); verified PRD §7 Coverage Summary grand total (201 ✓); verified PRD §5 error taxonomy (33 active namespaces ✓); verified subsystem label propagation across 50+ BCs. Burst-24 closures all confirmed in-spec. Three new findings surfaced — two from Policy 9 arithmetic self-check and one new AC-trace mis-anchoring class in the newest story (S-5.10) where BC-2.05.011 was added to frontmatter in Burst 2.75 but no AC trace was ever wired to it.

---

## CRITICAL

None.

---

## HIGH

### P3P24-A-H-001 — S-5.10 AC traces mis-anchored; BC-2.05.011 has zero AC citations despite owning the story's core postconditions

**Policy violated:** 4 (`semantic_anchoring_integrity`) + 8 (`bc_array_changes_propagate_to_body_and_acs`)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** MEDIUM (new instance of a known class — AC trace drift after BC frontmatter additions; last seen in Burst 22 on S-5.01/S-5.05)
**File:** `/Users/jmagady/dev/prism/.factory/stories/S-5.10-audit-trail-forwarding.md`

**Evidence:**
- Line 8 `bcs:` frontmatter includes `BC-2.05.011`.
- Lines 44–50 body "Behavioral Contracts" table includes BC-2.05.011 with `precondition + postconditions 1–2 + invariant` coverage (the authoritative at-least-once forwarding contract; owns retry, watermark, eviction, and permanent-failure semantics).
- AC trace citations (lines 187, 192, 198, 203, 212) reference only BC-2.05.001/002/003/004. **No AC traces to BC-2.05.011.**
- AC traces are semantically wrong for four ACs:
  - AC-2 (retry backoff sequence 2s/4s/8s for 503s, line 192) → traces to `BC-2.05.003` which is "Credential Values Are Never Present in Audit Entries" (redaction, not retry). Retry backoff is BC-2.05.011 postcondition "Forwarding failure with backoff".
  - AC-3 (FIFO eviction at `buffer_cap_mb`, line 198) → traces to `BC-2.05.004` which is "Write Operations Log Capability Check and Execution Outcome". FIFO eviction is BC-2.05.011 postcondition "Buffer cap / FIFO eviction".
  - AC-4 (independent per-destination retry when destination A is in backoff, line 203) → traces to `BC-2.05.003` (redaction). Independent-destination retry is BC-2.05.011 postcondition + INV-AUDIT-FWD-003.
  - AC-6 (permanent error skip + ERROR log + watermark advance, line 212) → traces to `BC-2.05.002` which is "Audit Entries Use Structured JSON Format with Complete Fields". Permanent-error handling is BC-2.05.011 error case `E-AUDIT-005`.

**Why it fails:** Policy 8 requires `bc_array_changes_propagate_to_body_and_acs`. BC-2.05.011 was added to frontmatter in Burst 2.75 (STORY-INDEX line 29) and to the body BC table, but the ACs were never rewired to cite it. Simultaneously Policy 4 is violated because four ACs cite BCs whose postconditions do not semantically cover the AC's behavior (a test-writer reading AC-2 and fetching BC-2.05.003 would find redaction rules, not retry/backoff specifications).

---

### P3P24-A-H-002 — verification-coverage-matrix.md fuzz column arithmetic diverges from VP-INDEX (row-sum=7, Totals=6)

**Policy violated:** 9 (`vp_index_is_vp_catalog_source_of_truth`)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** HIGH (newly-adopted Policy 9 surfacing arithmetic drift that survived Burst 24's Pass-23 P3P23-A-H-002 "triple-fix" because that fix touched totals + two VP rows but not the per-module fuzz column counts)
**File:** `/Users/jmagady/dev/prism/.factory/specs/architecture/verification-coverage-matrix.md`

**Evidence:**
- Line 21 (prism-security row): `| prism-security | CRITICAL | 5 | 1 | 2 | 90% | VP-007, VP-008, VP-009, VP-010, VP-020 (Kani); VP-024 (proptest); VP-038 (fuzz — injection scanner) |`
- Fuzz Targets column value is `2` but only VP-038 is listed in the VPs column for prism-security.
- VP-INDEX v1.3 line 59 confirms only one fuzz VP exists for `prism-security`: VP-038.
- Fuzz column row sum: prism-core 0 + prism-security 2 + prism-query 2 + prism-ocsf 1 + prism-operations 1 + prism-spec-engine 1 + prism-sensors 0 + prism-credentials 0 + prism-storage 0 + prism-audit 0 + prism-dtu-crowdstrike 0 + prism-mcp 0 + prism-bin 0 = **7**.
- Line 41 Totals row: `| Fuzz targets | 6 | 5 | 1 |`.
- VP-INDEX summary (line 68): `| Fuzz | 6 | 5 | 1 |` — Totals row agrees with VP-INDEX; the per-module column is the drifting artifact.

**Why it fails:** Policy 9 requires VP-INDEX be the source of truth and per-module column sums in verification-coverage-matrix.md equal VP-INDEX per-tool totals exactly. Here the prism-security Fuzz Targets cell is 2 but only one fuzz VP exists for that module. Either the `2` should be `1` (restoring row-sum=6=VP-INDEX), or a second security fuzz VP is missing from the VPs list cell — in either case the document is internally inconsistent and externally mis-aligned with VP-INDEX.

---

## MEDIUM

### P3P24-A-M-001 — verification-coverage-matrix.md Invariant-to-VP table puts BC-2.05.011 in the "Invariant" column

**Policy violated:** 4 (`semantic_anchoring_integrity`)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** MEDIUM (BC cited-as-source-invariant pattern also exists in verification-architecture.md for VP-027/028/039 — accepted precedent there — but verification-coverage-matrix.md's table is explicitly titled "Invariant-to-VP Traceability" making the label mis-anchoring more consequential)
**File:** `/Users/jmagady/dev/prism/.factory/specs/architecture/verification-coverage-matrix.md`

**Evidence:**
- Line 54 heading: `## Invariant-to-VP Traceability`.
- Line 56 column header: `| Invariant | Verified By | Status |`.
- Line 77: `| BC-2.05.011 (Audit forward watermark monotonicity) | VP-039 (module: prism-audit) | P0 |`.
- BC-2.05.011 is a behavioral contract, not a DI-NNN invariant. The table's first column is named "Invariant" and every other row in the table (lines 58–83) references a `DI-NNN` identifier (DI-001 through DI-032). BC-2.05.011 is the lone BC row.
- BC-2.05.011's own `Traceability` section (file lines 159–166) cites `L2 Invariants | DI-026`, suggesting DI-026 is the correct invariant anchor; the watermark-monotonicity property is arguably INV-AUDIT-FWD-001 (defined in BC-2.05.011 body lines 79–83) which is not an L2 domain invariant but a BC-level invariant.

**Why it fails:** Policy 4 requires anchors be semantically correct. A BC in a column labeled "Invariant" conflates two different artifact classes. An implementer scanning the Invariant-to-VP traceability table to find domain-level invariants they must enforce will either (a) miss DI-026 entirely (it is listed separately on line 76 as verified by VP-033) or (b) conclude BC-2.05.011 is a "domain invariant" and try to lift it into `domain-spec/invariants.md`.

---

## Observations

None raised as blocking — noted for awareness only:

- **O-1** (not a finding): The error-taxonomy supplement has two H2 headers named `STATE` (line 78 "Pagination State Errors", line 303 "Pagination State Errors (additional)") and two H2 headers that surface RULE codes (line 198 "RULE: Detection Rule Errors", line 341 "RULE: Detection Rule Extended Errors"). Distinct unique namespaces still = 33 so PRD §5 claim holds; the supplement's split-section formatting is cosmetic.
- **O-2** (not a finding): STATE.md frontmatter `cap_count: 34` and task spec `CAPs: 34 (CAP-001..034)` include the retired CAP-013 (REMOVED per capabilities.md line 33) in the total. PRD §7 Capability Coverage Summary uses 33 active rows. No internal contradiction given the explicit "(active)" qualifier in PRD §7, but the 34-vs-33 distinction merits a one-line note somewhere central if anyone ever starts attributing CAP-013 coverage.

## Novelty Assessment

- Finding P3P24-A-H-001: MEDIUM novelty — class (AC-trace drift after BC additions) is known from Burst 22, but the instance in S-5.10 is new; fresh context catches it because S-5.10's full-file read reveals the "no AC cites BC-2.05.011" gap that summary-level checks miss.
- Finding P3P24-A-H-002: HIGH novelty — first Policy-9 enforcement surfacing a non-obvious arithmetic divergence. Pass-23 triple-fix focused on totals + two specific VP rows (prism-audit, prism-dtu-crowdstrike); the fuzz-column per-module arithmetic was not part of that fix and has silently drifted.
- Finding P3P24-A-M-001: MEDIUM novelty — extends Policy 4 semantic-anchoring audit into table-column naming (not just BC↔subsystem or VP↔module).

Overall novelty: **MEDIUM**. Trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 → **3**. Decay back toward asymptote after Pass-23 uptick. CRIT=0 for 13th consecutive pass. Pass 24 BLOCKS convergence at 0/3 due to two HIGH findings, but the mix is consistent with late-cycle refinement (no new drift classes beyond Policy-9 arithmetic, which was the expected Policy-9 surfacing).

## Policy 9 Plugin Integration Canary

Not yet integrated — no enforcement observable. The only relevant plugin asset is `/Users/jmagady/.claude/plugins/marketplaces/vsdd-factory/plugins/vsdd-factory/hooks/protect-vp.sh` which is a PreToolUse hook that denies edits to green-status VP files; it does not validate or even read VP-INDEX.md, verification-architecture.md, or verification-coverage-matrix.md cross-consistency. Policy 9 (VP-INDEX ↔ verification-architecture.md Provable Properties Catalog + P0 list ↔ verification-coverage-matrix.md VP-to-Module table + Totals row) is currently adversary-enforced only. Finding P3P24-A-H-002 is exactly the class of drift an automated Policy-9 lint hook would catch at edit-time (row-sum vs Totals-row arithmetic), suggesting the integration has measurable value if built.

---

Relevant files referenced:
- `/Users/jmagady/dev/prism/.factory/stories/S-5.10-audit-trail-forwarding.md` (finding H-001)
- `/Users/jmagady/dev/prism/.factory/specs/architecture/verification-coverage-matrix.md` (findings H-002, M-001)
- `/Users/jmagady/dev/prism/.factory/specs/verification-properties/VP-INDEX.md` (authoritative for H-002)
- `/Users/jmagady/dev/prism/.factory/specs/behavioral-contracts/BC-2.05.011-audit-forwarding-at-least-once.md` (authoritative for H-001 semantic check)
- `/Users/jmagady/dev/prism/.factory/specs/architecture/verification-architecture.md` (cross-checked clean)
- `/Users/jmagady/dev/prism/.factory/specs/architecture/ARCH-INDEX.md` (cross-checked clean)
- `/Users/jmagady/dev/prism/.factory/specs/behavioral-contracts/BC-INDEX.md` (cross-checked clean)
- `/Users/jmagady/dev/prism/.factory/specs/prd.md` (cross-checked clean — 33 namespaces, 201 coverage total, 195 BCs, SS-07 label)
- `/Users/jmagady/dev/prism/.factory/stories/STORY-INDEX.md` (cross-checked clean — v1.16, 75 stories, 195 BCs, Wave 5=48)
