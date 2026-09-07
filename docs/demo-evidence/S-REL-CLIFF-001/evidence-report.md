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
1. A throwaway empty commit with a `feat(api)!:` subject AND a separate `BREAKING CHANGE:` footer body was created in the `Hide` section using two `-m` flags (multi-line commit, not single-line):
   ```
   git commit --allow-empty -m 'feat(api)!: THROWAWAY remove legacy sensor API' -m 'BREAKING CHANGE: /v0/sensors removed, migrate to /v1/sensors'
   ```
   This produces `commit.message = "THROWAWAY remove legacy sensor API"` and `commit.breaking_description = "/v0/sensors removed, migrate to /v1/sensors"` — two distinct values, exercising the `commit.breaking_description != commit.message` conditional in cliff.toml.
2. `git cliff --unreleased --tag v1.0.0-beta.1 --output /dev/stdout 2>/dev/null` was run in the `Show` (recorded) section.
3. The throwaway commit was dropped via `git reset --soft HEAD~1 && git reset HEAD` in the `Hide` cleanup section.
4. `git log --oneline -3` confirmed the throwaway commit is absent from history after recording.

Key evidence from output:

```
## [1.0.0-beta.1] - 2026-09-07

### Breaking Changes

- **BREAKING** THROWAWAY remove legacy sensor API — /v0/sensors removed, migrate to /v1/sensors

### Added

- version identity — 1.0.0-dev reset + PRISM_VERSION injection + version-agnostic docs ([#262](https://github.com/drbothen/prism/pull/262))

- agent-facing version identity — prism-mcp serverInfo + prism-spec-engine UA (S-REL-AGENT-VERSION-001) ([#263](https://github.com/drbothen/prism/pull/263))

- git-cliff setup + technical-writer Layer-1 dispatch (S-REL-CLIFF-001, S-REL-WRITER-001)

### Fixed

- musl rustup race + CHANGELOG beta.1 section + Release-notes wiring (DEFECT-REL001-MUSL-RUSTUP-COMPONENT-RACE-001) ([#261](https://github.com/drbothen/prism/pull/261))
```

`### Breaking Changes` appears BEFORE `### Added`, confirming the Tera filter-based two-part body mechanism (ADR-063 v1.7 D3 Deviation-2). The breaking entry shows ` — /v0/sensors removed, migrate to /v1/sensors` after the subject (em dash separator from the fixed template), confirming the `commit.breaking_description != commit.message` path is exercised correctly with no duplication.

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

The throwaway `feat(api)!:` commit used for the AC-006 demo was created inside the `Hide` section of `AC-006-breaking-changes-first.tape` (using two `-m` flags: subject + `BREAKING CHANGE:` footer) and dropped inside the `Hide` cleanup section via `git reset --soft HEAD~1 && git reset HEAD`. Post-recording `git log --oneline -3` shows:

```
7de4718a0 fix(E-REL-NOTES): template duplication guard + footer protection + breaking-changes sweep (review cycle 4)
84de9ac66 fix(E-REL-NOTES): catch-all skip rule + tag guard + pre-strip narrative + heading consistency (review cycle 3)
a77cc2f25 fix(E-REL-NOTES): correct awk version-anchor + .rel-artifacts comment (review cycle 2)
```

The throwaway commit (`feat(api)!: THROWAWAY remove legacy sensor API`) is absent. History is clean.
