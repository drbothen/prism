---
id: TD-W3-S-3.0.02-DOC-001
type: tech_debt
severity: suggestion
story: S-3.0.02
wave: 3
filed: 2026-04-28
status: open
---

# TD-W3-S-3.0.02-DOC-001 — Story v0.6: update marker comment text in prism-dtu-* crates

## Problem

Story S-3.0.02 §Tasks item 3 and §Architecture-Compliance-Rules instruct adding a marker
comment containing the literal string `DTU_DEFAULT_MODE` to each `prism-dtu-*` crate's
`lib.rs`. Example from the story:

```
// Classification lives in prism-core::DTU_DEFAULT_MODE (ADR-007 §2.3)
```

That literal `DTU_DEFAULT_MODE` substring would trip AC-8's grep test
(`grep -RIn 'DTU_DEFAULT_MODE|dtu_default_mode' crates/prism-dtu-*/`), causing a false
positive CI failure.

## Decision Made

Implementer correctly omitted the marker comments under TDD discipline: tests are
authoritative. The grep test passes with exit code 1 (no matches) as required by AC-8.

## Resolution

Update story S-3.0.02 to v0.6. Change the suggested marker comment in §Tasks item 3 from:

```
// Classification lives in prism-core::DTU_DEFAULT_MODE (ADR-007 §2.3)
```

to a form that does not contain the literal grep target, e.g.:

```
// Mode classification lives in prism_core::dtu (ADR-007 §2.3)
```

This is a documentation-polish fix only. No code changes needed; AC-8 already passes.

## Impact

- Non-blocking: merge completed at 373baf785fdf91d2f8ecfa9aa701716568bc7fcd
- Story writer task only
- Affects: S-3.0.02 story spec v0.6
