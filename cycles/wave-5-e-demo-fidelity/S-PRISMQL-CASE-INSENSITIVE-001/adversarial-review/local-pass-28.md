---
document_type: adversarial-review
scope: LOCAL
passes: [28]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: b341cdd7
fix_burst_head: "669080f5"
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts: {MED: 2, LOW: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 28 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 28 (frozen b341cdd7; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 1/3 — NOT CLEAN)

**Pass result:** CLEAN(strict)=NO (2 MED + 1 LOW), CLEAN(PR-merge)=NO (MED findings block)
**Findings:** 3 (F-P28-MED-001 MED — CLOSED product-owner BC-2.02.013 v1.8 + story-writer v1.31; F-P28-MED-002 MED — CLOSED implementer @669080f5; F-P28-LOW-001 LOW — CLOSED implementer @669080f5)
**Code HEAD at review:** b341cdd7 (frozen)
**Fix-burst HEAD:** 669080f5 (TD-VSDD-091 version-suffix strip + §Mode-Boundary Enforcement section-name fix; comment-only; feature HEAD advanced to 669080f5)
**LOCAL 3-CLEAN(strict) streak after pass-28:** 0/3 (findings present; streak reset)

---

## Finding Inventory

### F-P28-MED-001 (MED) — BC-2.02.013 v1.7 coverage column stale: status/disposition declared Absent + activity_name declared Partial, but all three landed

**Severity:** MED — POL-4 spec accuracy (implementation-guiding prose); a literal implementer reading v1.7 would add the missing enum_map entries and trip the collision panic.

**Finding:** BC-2.02.013 v1.7 §Coverage column (the "Status in enum_map.rs today" column of the §Canonical Test Vectors coverage table or equivalent tracking table) declared:
- `status` / `status_id` coverage: **Absent** — implementation had NOT yet added status_id 10 entries to the shared enum map.
- `activity_name` / `activity_id` coverage: **Partial** — only severity/disposition implemented; activity not yet wired.

However, the feature branch (b341cdd7 delta) already landed all three:
- `status_id: 10` entries (status vocabulary) — present in `enum_map.rs` as of the S-PRISMQL-CASE-INSENSITIVE-001 implementation bursts.
- `activity_id: 6` entries (activity vocabulary) — present.
- `disposition_id: 29` entries — present (was already tracked as Present in v1.7).

The BC v1.7 coverage column retained temporal-indexical language ("Status today", "currently Absent") that was accurate at an earlier implementation checkpoint but rotted as implementation bursts landed the missing entries. A literal re-implementer following v1.7 would attempt to add `status_id 10` and `activity_id 6` entries that already exist in `enum_map.rs`, triggering the `OnceLock` collision panic (no-duplicate invariant enforced by `OcsfEnumMap::new()`).

**Why MED (not HIGH):** The implementation is correct; the defect is in the BC prose coverage column only. No runtime regression path from simply reading the code — the existing tests would catch duplicate insertion attempts. However, a spec-first implementer (following BC as specification) would be misled, making this a genuine production-grade spec accuracy defect.

**POL-4 trigger:** BC prose that describes implementation state with temporal-indexical framing ("Status today", "currently Absent") decays when implementation advances. This is a recurrence-class finding for TD-VSDD-091/POL-4 extension — see OBS-P28-001 below.

**Closure:** CLOSED — product-owner BC-2.02.013 v1.7→v1.8 (this burst): coverage column updated to Present with verified counts for all three vocabulary dimensions (status_id 10, activity_id 6, disposition_id 29); temporal-indexical "Status today"/"currently Absent" framing removed; stative/directive language substituted. Story-writer v1.30→v1.31: BC pin sweep at all 18 live sites (18 present-tense BC-2.02.013 version citations confirmed v1.8; 0 stale). Pass-28 narrative + new frozen candidate HEAD 669080f5 recorded.

---

### F-P28-MED-002 (MED) — ~217 versioned BC pins in code comments across 25 delta files, incl. 47 stale BC-2.10.012 v1.8 pins in prism_describe.rs (TD-VSDD-091 / ACR-9)

**Severity:** MED — TD-VSDD-091 (anti-volatile-pin); code comments citing versioned BC pins are volatile and decay on each BC version bump; the 47 stale `BC-2.10.012 v1.8` pins in `crates/prism-bin/src/prism_describe.rs` are the highest-concentration stale cluster.

**Finding:** Grep across the 25 files in the 44-file delta that contain `BC-2.\d+\.\d+ v\d+\.\d+` patterns reveals approximately 217 versioned BC pin citations in code comments. Of these:

- **47 pins** in `crates/prism-bin/src/prism_describe.rs` cite `BC-2.10.012 v1.8` — stale since BC-2.10.012 advanced to v1.9 in D-1586 fix-burst. These are the test comment blocks on RG-028 and related tests that were added during the v1.8 implementation burst and never advanced when BC-2.10.012 moved to v1.9.
- **Remaining ~170 pins** across other delta files: BC-2.11.024, BC-2.02.013, BC-2.16.002 citations — version strings present in test module doc comments.

**TD-VSDD-091 applicability:** ADR-052 §D4 prohibits `file.rs:NNN` line-number pins (which decay on diffs). The same decay rationale applies to versioned BC pins in code comments: they require a sweep on every BC version bump, creating maintenance debt. The correct approach per TD-VSDD-091's spirit is to cite function names + behavioral anchors, not `BC-X.XX.XXX vY.ZZ` version strings in inline comments.

**Out-of-delta legacy note:** The adversary detected 7 additional files outside the 44-file delta (primarily in `crates/prism-query/src/`) that contain legacy versioned BC pins predating the S-PRISMQL-CASE-INSENSITIVE-001 branch. Those files are outside the fix-burst scope for this story's LOCAL cascade and are reported as informational only; they do not block CLEAN status.

**Closure:** CLOSED — implementer @669080f5 (this burst): version suffixes stripped from all in-delta code comment BC citations in the 25 affected files. Comment-only change; no behavioral code modified. 1407/1407 prism-query + 447/447 prism-mcp GREEN (UNCHANGED — comment-only). The 7 out-of-delta legacy files were intentionally left untouched; this is explicitly noted here as non-blocking informational.

---

### F-P28-LOW-001 (LOW) — sql_parser.rs section reference cited non-existent "§DML-Mode-Boundary Enforcement" (POL-4)

**Severity:** LOW — POL-4 spec accuracy (code comment cites non-existent section name); navigability defect.

**Finding:** `crates/prism-query/src/sql_parser.rs` line ~1182 contained a code comment reference to `"§DML-Mode-Boundary Enforcement"` — a section name that does not exist in BC-2.11.024 v1.3. The actual section in BC-2.11.024 is `"§Mode-Boundary Enforcement (DML scope)"`. The inverted naming (DML-Mode vs Mode-DML) would send a reader looking for a section that is not findable by that title.

**Closure:** CLOSED — same commit 669080f5 as F-P28-MED-002: comment corrected to `"§Mode-Boundary Enforcement (DML scope)"` matching BC-2.11.024 v1.3 verbatim section heading.

---

## Observations (non-finding)

### OBS-P28-001 — [process-gap candidate] BC temporal-indexical state language rots when implementation advances

**Classification:** Process-gap candidate; NOT a finding in this pass (the finding class is F-P28-MED-001 above; this OBS captures the root-cause pattern for codification).

**Observation:** BC-2.02.013 v1.7 used temporal-indexical language in its coverage column: "Status today: Absent", "currently Absent", "activity_name: Partial". This framing was accurate at BC authoring time (before the implementation bursts landed the missing entries) but decayed silently as implementation advanced. The defect was caught by pass-28 fresh-context adversary because the coverage column contradicted the actual `enum_map.rs` content.

**Pattern:** BCs that describe implementation state ("Status today", "currently X", "as of this writing") are specification anti-patterns under the VSDD model. BCs should use stative/directive phrasing ("MUST normalize", "the map MUST include entries for") rather than temporal present-tense claims about what the implementation currently does.

**Codification candidate:** Extend TD-VSDD-091 or POL-4 to explicitly forbid temporal-indexical present-tense state claims in BC coverage/postcondition language. Requires session-reviewer adjudication or follow-up story. See lessons.md entry for this observation.

**S-7.02 checklist flag:** Codification work (TD-VSDD-091/POL-4 extension, possible new POL rule) is a justified-deferred follow-up requiring session-reviewer adjudication. Does NOT block LOCAL cascade.

### OBS-P28-002 — shared_enum_map OnceLock design ratified (pass-16)

The `OnceLock`-based single-initialization design of `shared_enum_map()` was ratified in pass-16 (D-1586). No re-examination needed. The collision panic guard (`OcsfEnumMap::new()` duplicate-key check with `should_panic` test) remains in place and GREEN.

### OBS-P28-003 — No DTU test-vector changes needed in delta

SAP-2 scan: no sensor TOML or DTU clone changes in the 44-file delta. DTU parity check not applicable.

---

## SAP Probe Results (Pass 28, verified against b341cdd7)

**SAP-1 (tracing emission catalog completeness):** PASS — grep `event_type\s*=` across `crates/` workspace: 92 emission sites confirmed (91 existing catalog rows + 1 new entry added in the S-PRISMQL-CASE-INSENSITIVE-001 implementation; verified BC-2.16.002 §Postconditions catalog current). All `ocsf.enum_label_unrecognized` dual sites (PRIMARY `build_column_array` in `spec_driven_adapter.rs` + SECONDARY `normalize_with_mappers` in `normalizer.rs`) match BC-2.16.002 catalog row 91. No uncatalogued emission sites in delta.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU clone changes in this delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests in their respective crates; no external-dependency waivers present.

**POL-22 Phase A (ID/anchor integrity):** PASS — all BC anchors, E-QUERY-NNN codes, and RG-NNN test names verified present in story v1.30 (pre-fix) and v1.31 (post-fix). The BC-2.02.013 pin sweep (18 sites) and §Mode-Boundary Enforcement comment correction are both verified correct.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 74 RGT names (RG-001..RG-074) verified present in story v1.31. All domain entities present. No Red Gate test count change (comment-only fix-burst).

**Novelty:** LOW-MEDIUM — F-P28-MED-001 (BC temporal-indexical decay) is a new instance of an established finding class; its pattern is codification-candidate material (OBS-P28-001). F-P28-MED-002 (versioned BC pin sweep) is a known maintenance class (TD-VSDD-091). F-P28-LOW-001 (section name citation) is a routine prose-accuracy nit.

---

## Fix Summary

| Finding | Fix | Files | Commit |
|---------|-----|-------|--------|
| F-P28-MED-001 | BC-2.02.013 v1.7→v1.8 coverage column Absent/Partial→Present; temporal-indexical framing removed | `.factory/specs/behavioral-contracts/BC-2.02.013-*.md` | product-owner (this burst) |
| F-P28-MED-001 (companion) | Story v1.30→v1.31 BC-2.02.013 pin sweep 18 sites v1.7→v1.8 | `.factory/stories/S-PRISMQL-CASE-INSENSITIVE-001-*.md` | story-writer (this burst) |
| F-P28-MED-002 | Version suffixes stripped from in-delta code comment BC citations (~217 pins, 25 files); 7 out-of-delta legacy files left untouched (noted) | `crates/**/*.rs` (25 delta files) | implementer @669080f5 |
| F-P28-LOW-001 | `§DML-Mode-Boundary Enforcement` → `§Mode-Boundary Enforcement (DML scope)` in sql_parser.rs | `crates/prism-query/src/sql_parser.rs` | implementer @669080f5 (same commit) |

---

## Post-Fix-Burst State

- Feature HEAD: **669080f5** (comment-only pin strip + section-name fix; no behavioral code changed)
- 1407/1407 prism-query tests GREEN (UNCHANGED — comment-only)
- 447/447 prism-mcp tests GREEN (UNCHANGED — comment-only)
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN (UNCHANGED)
- LOCAL 3-CLEAN(strict) streak: **0/3** (RESET by 2 MED + 1 LOW findings in this pass)
- Novelty: LOW-MEDIUM (BC temporal-indexical decay — codification candidate; versioned pin maintenance — known class)
- NEXT ACTION: LOCAL adversary pass-29 on frozen 669080f5 with story v1.31 (streak candidate 1/3)
