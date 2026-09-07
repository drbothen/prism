# Demo Evidence Report — S-REL-WRITER-001

**Story:** S-REL-WRITER-001 — technical-writer Layer-1 dispatch contract  
**Branch:** feature/E-REL-NOTES-changelog  
**Recorded:** 2026-09-07  
**Tool:** VHS terminal recording (grep/git commands, FiraCode Nerd Font Mono)

---

## Coverage Summary

| AC | Description | Recording | Status |
|----|-------------|-----------|--------|
| AC-001 | Technical-writer dispatch step exists in release-prep.yml AFTER git-cliff step | AC-001-005-structural-verification | PASS |
| AC-002 | RELEASING.md §5 documents the two-layer model | AC-001-005-structural-verification | PASS |
| AC-003 | RELEASING.md §5 states Layer-1 placement is INSIDE the `## [VERSION]` block | AC-001-005-structural-verification | PASS |
| AC-004 | release-prep.yml PR checklist includes Layer-1 review item | AC-001-005-structural-verification | PASS |
| AC-005 | release.yml is NOT modified | AC-001-005-structural-verification | PASS |

---

## Recordings

### AC-001 through AC-005: Structural verification

**Artifact:** `AC-001-005-structural-verification.gif` / `AC-001-005-structural-verification.webm`  
**Tape:** `AC-001-005-structural-verification.tape`

This recording runs the five AC verification greps in sequence against the live worktree files.

---

#### AC-001: Technical-writer dispatch step exists after git-cliff step

Command: `grep -n 'git cliff\|Layer-1\|technical-writer' .github/workflows/release-prep.yml`

Key evidence from output:
```
173:  cargo install git-cliff --version 2.14.1 --locked --quiet
211:  git cliff --tag "v${VERSION}" --unreleased --prepend CHANGELOG.md
253:  # 7a. Draft Layer-1 release notes (technical-writer)
254:  #     Dispatch notice: vsdd-factory:technical-writer should insert
259:  #     See RELEASING.md §5 for the Layer-1 contract.
261:- name: Draft Layer-1 release notes (technical-writer)
264:  echo "::notice::MANUAL STEP REQUIRED: dispatch vsdd-factory:technical-writer..."
271:  echo "See RELEASING.md §5 for the Layer-1 contract."
```

Line 261 (`Draft Layer-1 release notes (technical-writer)`) appears after line 211 (git-cliff invocation). Step 7a is ordered AFTER Step 7.

Traces to: ADR-063 D5 — "technical-writer step ordered AFTER git-cliff".

---

#### AC-002 + AC-003: RELEASING.md §5 two-layer model + INSIDE placement

Command: `grep -n 'Layer 1\|Layer-1\|Highlights\|INSIDE' RELEASING.md | head -10`

Key evidence from output:
```
262:1. Review the git-cliff generated CHANGELOG entry (Layer 2):
265:2. Dispatch `vsdd-factory:technical-writer` to draft the Layer-1 top-block
266:   (`### Highlights` + ...
368:  (Layer-1 Highlights + Layer-2 git-cliff entries) extracted via `--notes-file`
394:Every release uses a two-layer CHANGELOG model. Layer 2 (git-cliff categorized body)
395:is auto-generated. Layer 1 (Highlights / Breaking Changes / Upgrade Notes) is...
450:#### Layer 1 — Curated top-block (human-authored, agent-assisted)
455:- `### Highlights` — 5-8 bullets summarizing the platform-level value...
460:**Placement invariant:** Layer-1 sections are placed INSIDE the `## [VERSION]` block,
```

Line 460 explicitly states the INSIDE placement invariant: `Layer-1 sections are placed INSIDE the ## [VERSION] block`. Both Layer 1 and Layer 2 terms are documented.

Traces to: ADR-063 D4 — two-layer CHANGELOG model; ADR-063 D4 v1.1 — placement inside `## [VERSION]` block.

---

#### AC-004: PR checklist includes Layer-1 review items

Command: `grep -n 'Layer-1\|Highlights.*reviewed' .github/workflows/release-prep.yml`

Key evidence from output:
```
312:  f'git-cliff (ADR-063 D5). The Layer-1 `### Highlights` / `### Breaking Changes`',
319:  '2. Dispatch `vsdd-factory:technical-writer` to draft the Layer-1 top-block',
323:  '3. Review and curate the drafted Layer-1 block — current behavior only;',
340:  '- [ ] Dispatch `vsdd-factory:technical-writer` to DRAFT the Layer-1 `### Highlights`',
341:  '- [ ] Review and curate the drafted Layer-1 block; confirm it is present and accurate',
342:  '- [ ] Layer-1 Breaking Changes (narrative) section reviewed (or confirmed empty)',
```

Lines 340-342 are the PR checklist items: `[ ] Dispatch vsdd-factory:technical-writer to DRAFT the Layer-1 ### Highlights` and `[ ] Layer-1 Breaking Changes (narrative) section reviewed (or confirmed empty)`.

Traces to: ADR-063 D4 — "human reviews and curates the draft in the release-prep PR".

---

#### AC-005: release.yml NOT modified

Command: `git diff develop -- .github/workflows/release.yml | wc -l`

Expected output: `0`  
Result: `0` — empty diff, release.yml is unmodified.

Traces to: ADR-063 D5 — "release.yml `--notes-file` mechanism unchanged".
