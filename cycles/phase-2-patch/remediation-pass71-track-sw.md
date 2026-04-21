---
pass: 71
track: story-writer
finding: CRIT-002
date: 2026-04-20
author: story-writer
---

# Pass-71 Story-Writer Track — CRIT-002 Manifest

## Finding

CRIT-002: Chronological date inversion in changelog tables of two stories.
Parallel scope to pass-70 MED-003 fix (which corrected S-4.08).

## Files Edited

### 1. S-1.14-infusion-specs.md

**Path:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.14-infusion-specs.md`

**Version bump:** 1.5 → 1.6

**Date corrections:**
- v1.0 (B-36): `2026-04-19` → `2026-04-17`
  - Was dated AFTER v1.1 (2026-04-18); now precedes it.

**Resulting chronological order:**
| Version | Date |
|---------|------|
| 1.0 | 2026-04-17 |
| 1.1 | 2026-04-18 |
| 1.2 | 2026-04-20 |
| 1.3 | 2026-04-20 |
| 1.4 | 2026-04-20 |
| 1.5 | 2026-04-20 |
| 1.6 | 2026-04-20 |

### 2. S-1.15-wasm-runtime.md

**Path:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.15-wasm-runtime.md`

**Version bump:** 1.5 → 1.6

**Date corrections:**
- v1.1 (B-36): `2026-04-19` → `2026-04-17`
  - Was dated AFTER v1.2 (2026-04-18); now precedes it.
- v1.0 (B-37): `2026-04-19` → `2026-04-16`
  - Was co-dated with v1.1 (both 2026-04-19); now strictly precedes v1.1.

**Resulting chronological order:**
| Version | Date |
|---------|------|
| 1.0 | 2026-04-16 |
| 1.1 | 2026-04-17 |
| 1.2 | 2026-04-18 |
| 1.3 | 2026-04-20 |
| 1.4 | 2026-04-20 |
| 1.5 | 2026-04-20 |
| 1.6 | 2026-04-20 |

## Constraints Applied

- No commit performed.
- No input-hash recomputed.
- All body content preserved; only changelog table rows and frontmatter version field touched.
- Absolute paths used throughout.
