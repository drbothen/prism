# Demo Evidence Report — S-REL-CLIFF-001

**Story:** S-REL-CLIFF-001 — git-cliff setup: cliff.toml + release-prep.yml Step 7 replacement  
**Branch:** feature/E-REL-NOTES-changelog  
**Recorded:** 2026-09-07  
**Tool:** VHS terminal recording (git-cliff 2.14.1, FiraCode Nerd Font Mono)

---

## Coverage Summary

| AC | Description | Recording | Status |
|----|-------------|-----------|--------|
| AC-001 | cliff.toml exists with required sections | AC-001-002-dry-run-categorized | PASS |
| AC-002 | Skip rules suppress docs/ci/test/chore/style/build/revert | AC-001-002-dry-run-categorized | PASS |
| AC-003 | release-prep.yml Step 7 replaced with 3-substep git-cliff flow | AC-001-002-dry-run-categorized (context), release-prep.yml grep | PASS |
| AC-004 | git-cliff 2.14.1 pinned | AC-004-git-cliff-version | PASS |
| AC-005 | `--prepend` used without `--output` | AC-001-002-dry-run-categorized (invocation visible) | PASS |
| AC-006 | Breaking Changes section precedes Added | AC-006-breaking-changes-first | PASS |

---

## Recordings

### AC-004: git-cliff 2.14.1 version pin

**Artifact:** `AC-004-git-cliff-version.gif` / `AC-004-git-cliff-version.webm`  
**Tape:** `AC-004-git-cliff-version.tape`

Command run: `git cliff --version`  
Expected output: `git-cliff 2.14.1`  
Result: Output confirmed `git-cliff 2.14.1`.

Traces to: ADR-063 D1 — version pin rationale.

---

### AC-001 + AC-002: Dry-run produces categorized sections, token-free PR links; skip types absent

**Artifact:** `AC-001-002-dry-run-categorized.gif` / `AC-001-002-dry-run-categorized.webm`  
**Tape:** `AC-001-002-dry-run-categorized.tape`

Command run: `git cliff --unreleased --tag v1.0.0-beta.1 --output /dev/stdout 2>/dev/null`

Key evidence from output:

- `### Added` section present with entries from `feat:` commits
- `### Fixed` section present with entries from `fix:` commits
- PR links injected token-free: e.g. `([#262](https://github.com/drbothen/prism/pull/262))`
- `<!-- N -->` group prefix markers stripped: only `### Added`, `### Fixed` visible (not `<!-- 1 -->Added`)
- No entries from `docs:`, `ci:`, `chore:`, `test:`, `style:`, `build:`, or `revert:` commits

AC-001 confirmation: `ls cliff.toml` exits 0; file contains `[changelog]`, `[git]`, `[remote.github]`, and `commit_parsers` with six typed entries (feat/fix/perf/refactor/security plus docs-skip group).  
AC-002 confirmation: Only `### Added` and `### Fixed` sections appear; no skip-type entries visible.

Traces to: ADR-063 D3 — cliff.toml categorization convention; ADR-063 v1.7 D3 Deviation-1 — token-free PR link injection.

---

### AC-006: Breaking Changes section precedes Added

**Artifact:** `AC-006-breaking-changes-first.gif` / `AC-006-breaking-changes-first.webm`  
**Tape:** `AC-006-breaking-changes-first.tape`

Procedure:
1. A throwaway empty commit with `feat!:` subject and `BREAKING CHANGE:` footer was created in the `Hide` section of the tape.
2. `git cliff --unreleased --tag v1.0.0-beta.1 --output /dev/stdout 2>/dev/null` was run in the `Show` (recorded) section.
3. The throwaway commit was dropped via `git reset --soft HEAD~1 && git reset HEAD` in the `Hide` cleanup section.
4. `git log --oneline -3` confirmed the throwaway commit is absent from history after recording.

Key evidence from output:

```
## [1.0.0-beta.1] - 2026-09-07

### Breaking Changes

- **BREAKING** THROWAWAY remove legacy sensor API — BREAKING CHANGE: /v0/sensors removed,
  migrate to /v1/sensors

### Added

- version identity — ...
- agent-facing version identity — ...

### Fixed

- musl rustup race — ...
```

`### Breaking Changes` appears BEFORE `### Added`, confirming the Tera filter-based two-part body mechanism (ADR-063 v1.7 D3 Deviation-2).

Additional verification: `grep 'group_order' cliff.toml` returns no match — `[git] group_order` is not present (it does not exist in git-cliff 2.14.1).

Traces to: ADR-063 v1.7 D3 — filter-based two-part body; `commits | filter(attribute="breaking", value=true)` renders Breaking Changes FIRST.

---

### AC-003 + AC-005 supplemental grep evidence

```
$ grep -n 'git cliff' .github/workflows/release-prep.yml
211:          git cliff --tag "v${VERSION}" --unreleased --prepend CHANGELOG.md

$ grep -n 'git log --merges' .github/workflows/release-prep.yml
(no output — scaffold removed)

$ grep -n 'git cliff' .github/workflows/release-prep.yml | grep -v output
211:          git cliff --tag "v${VERSION}" --unreleased --prepend CHANGELOG.md
```

---

## Synthetic Commit Disposal Confirmation

The throwaway `feat!:` commit used for the AC-006 demo was created inside the `Hide` section of `AC-006-breaking-changes-first.tape` and dropped inside the `Hide` cleanup section via `git reset --soft HEAD~1 && git reset HEAD`. Post-recording `git log --oneline -3` shows:

```
5ead5f1cb docs(ci): fix misleading comment in release-prep.yml Step 7 pre-strip edge case
1b0504b7d docs(releasing): holistic coherence audit — F-1..F-4, H-A..H-C, stale infusion term
7809f7e66 docs(RELEASING): fix --generate-notes stale claims + align archive contents to release.yml
```

The throwaway commit (`feat!: THROWAWAY ...`) is absent. History is clean.
