---
document_type: remediation-manifest
pass: 61
track: A
scope: HIGH-001 scope expansion — stem-only BC/VP path references in story bodies
produced_by: story-writer
date: 2026-04-20
---

# Remediation Pass-61 Track A
## HIGH-001 Scope Expansion: Stem-only BC/VP paths in story body tables

### Background

Pass-60 HIGH-001 sweep covered only `inputs:` frontmatter blocks. Pass-61 expands
coverage to all story body sections: `## File Structure Requirements` tables, `## Task`
prose, `## Traceability` tables, `## Previous Story Intelligence`, `## Verification
Properties` tables, `## Dev Notes`, and any other body section.

### Corpus Sweep Results

**Total stories scanned:** 75

**Patterns searched:**
- `BC-[0-9]+\.[0-9]+\.[0-9]+\.md` — stem-only BC filename (without slug)
- `VP-[0-9]+\.md` — stem-only VP filename (uppercase, without slug)

**Grep command (BC):** `grep -rn "BC-[0-9]+\.[0-9]+\.[0-9]+\.md" .factory/stories/`
**Grep command (VP):** `grep -rn "VP-[0-9]+\.md" .factory/stories/`

---

## Per-File Fix List

### S-4.07-case-metrics.md

| # | Location | Line | Old Value | New Value |
|---|----------|------|-----------|-----------|
| 1 | `## File Structure Requirements` table | 248 | `` `.factory/specs/behavioral-contracts/BC-2.14.012.md` `` | `` `.factory/specs/behavioral-contracts/BC-2.14.012-acknowledge-alert.md` `` |

**BC slug verified via Glob:** `BC-2.14.012-acknowledge-alert.md` confirmed at
`.factory/specs/behavioral-contracts/BC-2.14.012-acknowledge-alert.md`.

**Version bumped:** 1.4 → 1.5

**Changelog row added:**
```
| 1.5 | pass-61-fix | 2026-04-20 | story-writer | Fixed stem-only BC/VP path references in story body (HIGH-001 scope expansion from pass-60). |
```

---

## Summary

| Metric | Count |
|--------|-------|
| Stories scanned | 75 |
| BC stem-only findings (body) | 1 |
| VP stem-only findings (body) | 0 |
| Total references fixed | 1 |
| Stories touched | 1 |
| Anomalies | 0 |

---

## Anomalies

None. The single finding (`BC-2.14.012.md` in S-4.07 line 248) resolved cleanly
to the confirmed slug `BC-2.14.012-acknowledge-alert.md`. No BC or VP IDs were
found that lacked a corresponding file on disk.

---

## Stories Touched

- `/Users/jmagady/Dev/prism/.factory/stories/S-4.07-case-metrics.md` (version 1.4 → 1.5)

---

## Scope Confirmation

Pass-61 confirms the HIGH-001 class of defect (stem-only BC/VP filenames) is now
fully remediated across both:
1. `inputs:` frontmatter blocks (remediated pass-60)
2. Story body sections — all table types and prose (remediated pass-61)

No further stem-only BC/VP path references exist in the 75-story corpus.
