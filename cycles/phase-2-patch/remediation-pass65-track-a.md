---
pass: 65
track: A
remediation: MED-001 frontmatter version drift
approach: frontmatter-sync-only (no new changelog rows)
date: "2026-04-20"
---

# Pass-65 Track A — Frontmatter Version Sync

## Issue

Pass-64 appended `pass-64-fix` changelog rows to 8 stories but did NOT bump
the frontmatter `version:` field. The existing changelog row already describes
the change; this pass syncs the frontmatter field to match.

## Fix Applied

Single-line `version:` edit per file. No new changelog rows added.

## Files Changed

| File | version: before | version: after |
|------|-----------------|----------------|
| `.factory/stories/S-1.07-credential-crud.md` | `"1.5"` | `"1.6"` |
| `.factory/stories/S-1.08-feature-flags.md` | `"1.3"` | `"1.4"` |
| `.factory/stories/S-1.09-confirmation-tokens.md` | `"1.3"` | `"1.4"` |
| `.factory/stories/S-1.10-prompt-injection-defense.md` | `"1.3"` | `"1.4"` |
| `.factory/stories/S-1.11-spec-loading.md` | `"1.3"` | `"1.4"` |
| `.factory/stories/S-1.12-hot-reload.md` | `"1.3"` | `"1.4"` |
| `.factory/stories/S-1.13-sensor-write-specs.md` | `"1.3"` | `"1.4"` |
| `.factory/stories/S-4.08-action-delivery.md` | `"1.5"` | `"1.6"` |

## Status

COMPLETE — 8 files patched, no commit requested.
