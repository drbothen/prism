---
document_type: adversarial-review
scope: LOCAL
passes: [30]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: 669080f5
fix_burst_head: null
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts: {MED: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 30 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 30 (frozen 669080f5; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 2/3 — NOT CLEAN)

**Pass result:** CLEAN(strict)=NO (1 MED), CLEAN(PR-merge)=NO (MED finding blocks)
**Findings:** 1 (F-P30-MED-001 MED — CLOSED product-owner: error-taxonomy.md v2.19 provenance row appended)
**Code HEAD at review:** 669080f5 (frozen; UNCHANGED from pass-29)
**Fix-burst HEAD:** n/a (no code change; error-taxonomy.md spec-hygiene fix only; feature HEAD UNCHANGED 669080f5)
**LOCAL 3-CLEAN(strict) streak after pass-30:** 0/3 (RESET by MED finding; streak was 1/3 after pass-29)

---

## Finding Inventory

### F-P30-MED-001 (MED) — error-taxonomy.md frontmatter v2.19 (modified 2026-07-07) has no v2.19 changelog row; newest changelog entry was v2.18 (2026-07-06); tombstone-gap pattern

**Severity:** MED — POL-26 (spec-file provenance completeness) + POL-32 (changelog_monotonic_descending / no-tombstone-gap). The frontmatter declares `version: "2.19"` but the changelog table's most recent row is `| 2.18 | ... | 2026-07-06 | ...`. A reader auditing the v2.18→v2.19 bump has no changelog evidence of what changed. This is the same tombstone-gap pattern that surfaced as F-LP10-MED-001 (D-870 v1.61 gap) in the wave-0 plugin prereqs cascade — that finding produced the tombstone convention for reconstructed rows. The same convention applies here.

**Root cause:** The error-taxonomy.md frontmatter was bumped from v2.18 to v2.19 during the S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-3 fix-burst (D-1573, commit a1bcf100) when E-QUERY-001's BC anchors list was extended to include BC-2.11.024 (mode-boundary rejection). The bump was substantive — E-QUERY-001's Description column now cites `BC-2.11.024 (SQL-mode rejection of IEQ/IIN/INE — mode-boundary enforcement; parse-time trap closure for operators that are filter/pipe-only)`. However, no changelog row was appended to record this bump.

**POL-26 trigger:** All version-bearing spec files must carry a changelog row for every version increment. A v2.18→v2.19 bump without a changelog row violates POL-26 provenance completeness.

**POL-32 trigger:** The changelog is required to be monotonic descending (newest row first); a missing row at the v2.19 position breaks the continuous sequence.

**D-870 precedent (tombstone convention):** When a changelog row is genuinely absent for a past version bump (as opposed to a bump that never happened), the correct closure is a SUBSTANTIVE row documenting what actually changed — not a bare tombstone. This is distinguished from pure-tombstone cases where the content of the bump must be reconstructed from context. Here the content is clear: E-QUERY-001 BC anchor extension to BC-2.11.024 in commit a1bcf100 (pass-3 fix-burst D-1573).

**No code change required:** The feature code at 669080f5 is correct. E-QUERY-001's error code, MCP mapping, Display message, and retryability classification are unchanged. The BC-2.11.024 anchor is the behavioral link that was missing from the taxonomy's BC anchors list for E-QUERY-001. The fix is a spec-document provenance row only.

**Closure:** CLOSED — product-owner: substantive v2.19 changelog row appended to `.factory/specs/prd-supplements/error-taxonomy.md` (the already-edited file in this burst). The row documents: E-QUERY-001 BC anchor addition of BC-2.11.024 (SQL-mode rejection of IEQ/IIN/INE at parse time; mode-boundary enforcement invariant); the commit anchor a1bcf100; the burst context D-1573 (pass-3 fix-burst). Frontmatter `version: "2.19"` is UNCHANGED (it was already v2.19; only the provenance row was missing). Feature HEAD UNCHANGED 669080f5. No code change.

---

## Observations (non-finding)

### OBS-P30-001 — No code change needed; spec-hygiene finding only

**Classification:** Process observation; NOT a new finding class.

**Observation:** F-P30-MED-001 is a spec-hygiene finding (missing changelog provenance row) not a code defect. The feature code at 669080f5 is correct and all behavioral contracts are satisfied. The LOCAL 3-CLEAN(strict) streak resets to 0/3 per BC-5.39.001 because the finding occurred before the streak completed, even though the fix is spec-only and the feature HEAD is unchanged (per DRIFT-ORCH-PRLEVEL-PUSH-001 the streak resets only on branch pushes, not on .factory/-only commits). Per BC-5.39.001 the streak criterion is three consecutive CLEAN(strict) adversary passes on the feature code — a spec artifact finding that is closed in-burst still counts as a finding for streak purposes.

**Note on streak reset mechanics:** The streak resets to 0/3 because this pass is NOT CLEAN(strict). DRIFT-ORCH-PRLEVEL-PUSH-001 (frozen-HEAD streak rule) says "the 3-CLEAN streak only counts consecutive CLEAN(strict) passes taken against an UNCHANGED feature/PR HEAD." The feature HEAD (669080f5) is UNCHANGED. However, BC-5.39.001 requires zero findings of ANY severity for CLEAN(strict) status. F-P30-MED-001 is a genuine finding that was present in this pass. Streak = 0/3.

### OBS-P30-002 — SAP-1 consistency with pass-29 (92 emission sites)

No new tracing emission sites were added between pass-29 and pass-30 (both review the same frozen HEAD 669080f5). SAP-1 result is identical to pass-29.

---

## SAP Probe Results (Pass 30, verified against 669080f5)

**SAP-1 (tracing emission catalog completeness):** PASS — same frozen HEAD as pass-29; 92 emission sites confirmed; all catalogued in BC-2.16.002 §Postconditions. No change.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU clone changes in the 44-file delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests. Unchanged from pass-29.

**POL-22 Phase A (ID/anchor integrity):** PASS-with-1-MED — all BC anchors, E-QUERY-NNN codes, and RG-NNN test names verified present. The 1 MED finding (F-P30-MED-001) is a spec-hygiene provenance gap, not an anchor integrity issue. Phase A structural integrity PASS; provenance gap CLOSED product-owner.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 74 RGT names (RG-001..RG-074) verified present in story v1.31. Unchanged from pass-29.

**Novelty:** LOW-MEDIUM — F-P30-MED-001 is a familiar tombstone-gap pattern (D-870 precedent; same class as v1.61 gap in wave-0 cascade). The spec-hygiene finding class for missing changelog provenance rows is established and well-precedented. Low novelty.

---

## Fix Summary

| Finding | Fix | Files | Commit |
|---------|-----|-------|--------|
| F-P30-MED-001 | Substantive v2.19 changelog row appended to error-taxonomy.md documenting E-QUERY-001 BC-2.11.024 anchor addition (commit a1bcf100, D-1573 pass-3 fix-burst) | `.factory/specs/prd-supplements/error-taxonomy.md` | product-owner (this burst; frontmatter version UNCHANGED v2.19) |

---

## Post-Fix State

- Feature HEAD: **669080f5** (UNCHANGED — no code change; spec-hygiene fix only)
- error-taxonomy.md: **v2.19** (frontmatter version unchanged; v2.19 provenance row NOW PRESENT)
- 1407/1407 prism-query tests GREEN (UNCHANGED)
- 447/447 prism-mcp tests GREEN (UNCHANGED)
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN (UNCHANGED)
- LOCAL 3-CLEAN(strict) streak: **0/3** (RESET by 1 MED finding in this pass)
- Novelty: LOW-MEDIUM (tombstone-gap class; D-870 precedent)
- NEXT ACTION: LOCAL adversary pass-31 on frozen 669080f5 with story v1.31 (streak candidate 1/3)
