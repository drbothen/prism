---
document_type: remediation-manifest
phase: phase-2-patch
step: 5
track: B
producer: product-owner
timestamp: 2026-04-20T00:00:00Z
findings: [IMP-002, IMP-005, IMP-006]
---

# Remediation Manifest — Step 5 Track B

## Summary

Track B remediation of consistency-validation-step5.md findings IMP-002, IMP-005, and IMP-006. All three findings resolved. Files touched: 4 BC files + 1 new epics.md = 5 files written. No commits made. No input-hash values modified.

---

## IMP-002: BC-2.10.008 Changelog Row Ordering

**Status:** RESOLVED

**File:** `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.10.008-mcp-resources.md`

**Action taken:** Reordered Changelog rows to descending version order: `1.3, 1.2, 1.1, 1.0`. The pre-build-sweep row (1.3) had been appended out of sequence after 1.1; it now correctly precedes 1.2.

**Combined with IMP-006:** This file also received the IMP-006 `capability:` normalization (array `["CAP-008", "CAP-009"]` → scalar `"CAP-008, CAP-009"`) and version bump 1.3 → 1.4, with a new Changelog row 1.4 appended. Both fixes applied in a single Write.

**No version bump for IMP-002 alone:** The version bump (1.3 → 1.4) is attributed to IMP-006; IMP-002 is purely cosmetic reordering within the existing 1.3 content.

---

## IMP-005: Create `.factory/specs/epics.md`

**Status:** RESOLVED

**File created:** `/Users/jmagady/Dev/prism/.factory/specs/epics.md`

**Story counts per epic (derived from `epic_id:` frontmatter scan of all S-*.md files):**

| Epic ID | Title | Wave | Story Count | Stories |
|---------|-------|------|-------------|---------|
| E-0 | Foundation (CI/CD + Toolchain) | 0 | 2 | S-0.01, S-0.02 |
| E-1 | Core Primitives & Domain Types | 1 | 15 | S-1.01 – S-1.15 |
| E-2 | Storage & Sensor Foundation | 2 | 8 | S-2.01 – S-2.08 |
| E-3 | Query Engine & PrismQL | 3 | 13 | S-3.01 – S-3.13 |
| E-4 | Scheduler, Detection, Cases | 4 | 8 | S-4.01 – S-4.08 |
| E-5 | MCP Interface & Tool Surface | 5 | 10 | S-5.01 – S-5.10 |
| E-6 | DTU, Packaging, Integration Tests | 6 | 19 | S-6.01 – S-6.19 |
| **Total** | | | **75** | |

**Anomalies:** None. Every `epic_id:` value in the story corpus falls into E-0 through E-6 with no gaps and no unrecognized values. Total 75 stories matches STORY-INDEX expected count.

**Frontmatter:** Canonical `epic-registry` document_type with `input-hash: "[pending-recompute]"` per constraint (state-manager computes hash).

---

## IMP-006: Normalize 4 BC `capability:` Field YAML Arrays

**Status:** RESOLVED

All four BC files had `capability:` expressed as a YAML array. Each was normalized to a string scalar `"CAP-NNN, CAP-MMM"` format, version bumped (minor), and a Changelog row appended.

| File | Old Value | New Value | Old Version | New Version |
|------|-----------|-----------|-------------|-------------|
| BC-2.01.010-partial-failure-handling.md | `[CAP-001, CAP-002]` | `"CAP-001, CAP-002"` | 1.1 | 1.2 |
| BC-2.10.002-tool-registration-via-tool-router.md | `[CAP-005, CAP-015]` | `"CAP-005, CAP-015"` | 2.4 | 2.5 |
| BC-2.10.005-notifications-tools-list-changed.md | `[CAP-005, CAP-009]` | `"CAP-005, CAP-009"` | 1.2 | 1.3 |
| BC-2.10.008-mcp-resources.md | `["CAP-008", "CAP-009"]` | `"CAP-008, CAP-009"` | 1.3 | 1.4 |

**Note:** BC-2.10.008 received both IMP-002 (Changelog reorder) and IMP-006 (capability normalization) in a single Write to minimize hook fires per instructions.

---

## Files Touched

1. `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.01.010-partial-failure-handling.md` — IMP-006
2. `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.10.002-tool-registration-via-tool-router.md` — IMP-006
3. `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.10.005-notifications-tools-list-changed.md` — IMP-006
4. `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.10.008-mcp-resources.md` — IMP-002 + IMP-006
5. `/Users/jmagady/Dev/prism/.factory/specs/epics.md` — IMP-005 (created)

## Anomalies

None. All findings resolved cleanly. No ambiguous epic assignments. No BC content changes beyond frontmatter normalization and Changelog reordering.
