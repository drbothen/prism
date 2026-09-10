# Evidence Report: S-REL-CHANGELOG-CHANNEL-SCOPE-001

**Story:** Channel-Scoped Release Notes — Per-Channel git-cliff Tag-Pattern Filter  
**Worktree HEAD:** `0583581ce`  
**Branch:** `feature/S-REL-CHANGELOG-CHANNEL-SCOPE-001`  
**Captured:** 2026-09-09  
**tdd_mode:** facade — no runnable product surface; verification is terminal evidence of structural grep/inspection checks and git-cliff dry-runs

---

## Per-AC Coverage Table

| AC | Description | Command(s) | Result |
|----|-------------|-----------|--------|
| AC-001 | `--tag-pattern` flag present in all three workflow invocations | grep in release.yml, release-prep.yml, release-tag.yml | PASS |
| AC-002 | Nightly pattern covers nightly + stable; excludes beta/rc | grep-E spot-checks; grep for nightly line in release.yml | PASS |
| AC-003 | Pre-release pattern (release-tag.yml) covers channel + stable; excludes nightly and other channels | grep TAG_PATTERN/tag-pattern in release-tag.yml; beta regex spot-checks | PASS |
| AC-004 | Stable pattern (release-prep.yml) covers stable-only; excludes all pre-releases | grep TAG_PATTERN/stable in release-prep.yml; stable regex spot-checks | PASS |
| AC-005 | Stable-floor fallback implicit in pattern design | regex spot-check (v1.0.0 matches beta optional-group pattern); git-cliff dry-run with beta pattern | PASS |
| AC-006 | Nightly bypass-guard regex in release.yml unchanged (^/$ anchors + optional same-day suffix) | grep TAG.*=~.*nightly in release.yml; verified ^, $, and optional group present | PASS |
| AC-007 | Curated-CHANGELOG hard-fail (RELEASE-NOTES-MISSING) present exactly once | `grep -c 'echo "RELEASE-NOTES-MISSING'` returns 1; awk extraction present | PASS |
| AC-008 | actionlint exits 0 on all three modified workflow files | `actionlint` 1.7.12 on release.yml, release-prep.yml, release-tag.yml | PASS (exit 0 all three) |
| AC-009 | Docs updated — RELEASE-CHANNELS.md §5 + RELEASING.md §5 with ADR-063 §D7 reference | grep channel-scope/tag-pattern/D7 in both docs; grep ad-hoc-lane/release-tag behavior in RELEASING.md | PASS |
| AC-010 Part A | release-tag.yml structural: SHA pin, tool spec, no cargo install, step ordering | grep for taiki-e SHA + git-cliff@2.14.1; confirm step 5b push precedes step 6 tag creation | PASS |
| AC-010 Part B | Live per-channel git-cliff dry-runs (nightly + stable) | `git cliff --latest --tag-pattern '...'`; `git cliff --unreleased --tag v1.0.0 --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+$'` | PASS |

---

## AC-001: `--tag-pattern` Flag Present in All Three Workflow Invocations

```
$ grep 'tag-pattern' .github/workflows/release.yml
              --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$' \

$ grep 'TAG_PATTERN' .github/workflows/release-prep.yml
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-alpha\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+$'
            --tag-pattern "${TAG_PATTERN}" \

$ grep 'tag-pattern' .github/workflows/release-tag.yml
            --tag-pattern "${TAG_PATTERN}" \
```

All three invocations carry `--tag-pattern`. **PASS.**

---

## AC-002: Nightly Pattern Covers Nightly + Stable; Excludes Beta/RC

Pattern under test: `^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$`

```
$ echo "v1.0.0-nightly.20260909" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
v1.0.0-nightly.20260909
MATCHED

$ echo "v1.0.0-nightly.20260909.2" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
v1.0.0-nightly.20260909.2
MATCHED

$ echo "v1.0.0" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
v1.0.0
MATCHED   (stable-floor)

$ echo "v1.0.0-beta.1" | grep -E '...'
NO MATCH - PASS

$ echo "v1.0.0-rc.1" | grep -E '...'
NO MATCH - PASS

$ grep 'nightly' .github/workflows/release.yml | grep 'tag-pattern'
              --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$' \
```

**PASS.**

---

## AC-003: Pre-Release Pattern (release-tag.yml) Covers Channel + Stable; Excludes Nightly and Other Channels

```
$ grep 'TAG_PATTERN\|tag-pattern' .github/workflows/release-tag.yml
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-alpha\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+$'
            --tag-pattern "${TAG_PATTERN}" \
```

Beta regex spot-checks (`^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$`):

```
$ echo "v1.0.0-beta.2" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'
v1.0.0-beta.2
MATCHED

$ echo "v1.0.0" | grep -E '...'
v1.0.0
MATCHED   (stable-floor)

$ echo "v1.0.0-nightly.20260909" | grep -E '...'
NO MATCH - PASS

$ echo "v1.0.0-rc.1" | grep -E '...'
NO MATCH - PASS
```

if/elif/else block present for all five channels (nightly/alpha/beta/rc/stable). **PASS.**

---

## AC-004: Stable Pattern (release-prep.yml) Covers Stable-Only; Excludes All Pre-Releases

```
$ grep 'stable\|TAG_PATTERN' .github/workflows/release-prep.yml | grep -v '#'
      - name: Install Rust toolchain (stable, for cargo update)
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-alpha\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$'
            TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+$'
            --tag-pattern "${TAG_PATTERN}" \
```

Stable-exclusion invariant — stable pattern `^v[0-9]+\.[0-9]+\.[0-9]+$` (no optional suffix group):

```
$ echo "v1.0.0" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$'
v1.0.0
MATCHED

$ echo "v1.0.0-nightly.20260909" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$'
NO MATCH - PASS

$ echo "v1.0.0-beta.1" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$'
NO MATCH - PASS
```

**PASS.**

---

## AC-005: Stable-Floor Fallback Implicit in Pattern Design

Regex check — `v1.0.0` (a stable tag) matches the beta optional-group pattern:

```
$ echo "v1.0.0" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'
v1.0.0
MATCHED - stable floor is present
```

git-cliff dry-run with beta pattern on this repo (has `v1.0.0-beta.1` as the only beta tag; no
prior stable tag exists):

```
$ git cliff --unreleased --tag v1.0.0-beta.1 \
    --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$' \
    --output /dev/stdout

INFO  git_cliff > Using configuration from: cliff.toml
# Changelog
...
## [1.0.0-beta.1] - 2026-09-10

### Added
- git-cliff categorized notes for nightly tags (S-REL-NIGHTLY-NOTES-001) (#277)
- ship specs tarball + install-script spec placement (S-REL-SPECS-TARBALL-001) (#279)
- per-channel git-cliff tag-pattern scoping (S-REL-CHANGELOG-CHANNEL-SCOPE-001)

### Fixed
- use git cliff --latest for nightly notes (AC-008, S-REL-NIGHTLY-NOTES-001) (#278)
- harden release-tag.yml step 5b against three fail-open defects
- replace pipefail-fragile echo|grep with here-string idiom
```

Output includes commits from PRs #277, #278 (which are also under nightly tags). Stable-floor fallback
behavior confirmed: with no prior beta tag, git-cliff walks from origin and includes all qualifying
commits. **PASS.**

---

## AC-006: Nightly Bypass-Guard Regex in release.yml Unchanged

```
$ grep -n 'TAG.*nightly' .github/workflows/release.yml | head -5
326:          if [[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}(\.[0-9]+)?$ ]]; then

$ grep 'TAG.*=~.*nightly' .github/workflows/release.yml
          if [[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}(\.[0-9]+)?$ ]]; then
```

Verified:
- `^` start anchor: present
- `$` end anchor: present
- `(\.[0-9]+)?` optional same-day suffix group: present
- `--tag-pattern` change is inside the nightly BODY only (after the if branch is selected);
  the bypass-guard itself is NOT modified

Note: The story's suggested grep `grep 'nightly\.\[0-9\]' .github/workflows/release.yml | grep -c '\\$'`
returns 0 because the YAML file contains `nightly\.[0-9]` (with a backslash before `.`), while the grep
pattern (after shell quoting resolution) looks for `nightly.[0-9]` without a preceding backslash. The
bypass-guard is present and correct; the alternate greps above confirm it. **PASS.**

---

## AC-007: Curated-CHANGELOG Hard-Fail (RELEASE-NOTES-MISSING) Unchanged

```
$ grep -c 'echo "RELEASE-NOTES-MISSING' .github/workflows/release.yml
1
```

Value is 1 (the gate echo line only; the file also contains a descriptive comment referencing
the term, as expected and noted in the story).

```
$ grep 'RELEASE-NOTES-MISSING' .github/workflows/release.yml
          # Hard-fail (RELEASE-NOTES-MISSING) if the section is absent — a     #
              echo "RELEASE-NOTES-MISSING: CHANGELOG.md has no section for version '${VERSION}' (tag '${TAG}')." >&2
```

awk extraction present:

```
$ grep 'awk' .github/workflows/release.yml | head -4
            # awk: skip the "## [VERSION] - DATE" header line itself; print all subsequent
            # sed '/./,$!d' strips leading blank lines; the trailing awk strips trailing
            awk -v ver="$VERSION" '
              | awk '{buf[NR]=$0} /[^[:blank:]]/{last=NR} END{for(i=1;i<=last;i++) print buf[i]}' \
```

**PASS.**

---

## AC-008: `actionlint` Exits 0 on All Three Modified Workflow Files

```
$ actionlint --version
1.7.12

$ actionlint .github/workflows/release.yml
Exit code: 0

$ actionlint .github/workflows/release-prep.yml
Exit code: 0

$ actionlint .github/workflows/release-tag.yml
Exit code: 0
```

All three files pass with zero findings. **PASS.**

---

## AC-009: Docs Updated to Describe Channel-Scoped Model

RELEASE-CHANNELS.md:

```
$ grep 'channel.scope\|tag.pattern\|D7' docs/RELEASE-CHANNELS.md
generates a channel-scoped CHANGELOG section using the same three-substep mechanism
as `release-prep.yml` (pre-strip, `git cliff --unreleased --tag-pattern
§D7 Site 3.)
**Channel-scoped tag pattern (ADR-063 §D7):** git-cliff's global `tag_pattern`
this, each channel uses a per-invocation `--tag-pattern` flag that restricts the
| Channel | `--tag-pattern` |
overrides; `--tag-pattern` replaces the configured value for that single run only.
This channel scope logic is implemented at three sites per ADR-063 §D7:
| 0.3 | 2026-09-09 | Added §5 channel-scoped `--tag-pattern` (ADR-063 §D7): ...
```

RELEASING.md — ADR-063 §D7 reference present:

```
$ grep 'channel.scope\|tag.pattern\|D7' RELEASING.md
   # in release-prep.yml Step 7 (ADR-063 §D7). See docs/RELEASE-CHANNELS.md §5
     --tag-pattern "${TAG_PATTERN}" \
   **Channel-scoped `--tag-pattern` (ADR-063 §D7):** `release-prep.yml` Step 7
   ...
```

RELEASING.md — ad-hoc-lane develop-push behavior documented:

```
$ grep -i 'release-tag\|ad.hoc\|step 5b\|Site 3\|CHANGELOG.*develop\|develop.*CHANGELOG' RELEASING.md
pre-release tags use `release-tag.yml` which applies BASE-MATCH ...
**Ad-hoc pre-release lane (`release-tag.yml`):** When `release-tag.yml` is
... step 5b checks whether
directly to `develop`. Step 6 creates the annotated tag on that post-CHANGELOG
Site 3.)
```

Both documentation files updated with ADR-063 §D7 reference and ad-hoc-lane develop-push behavior. **PASS.**

---

## AC-010 Part A: release-tag.yml Structural Checks

**1. Install SHA match:**
```
$ grep 'taiki-e/install-action@d438492cf8a250514fa2d34b30bc3c0dc37c65ff' .github/workflows/release-tag.yml
        uses: taiki-e/install-action@d438492cf8a250514fa2d34b30bc3c0dc37c65ff # v2
```
PASS — same SHA as release.yml's "Install git-cliff (nightly path)" step.

**2. Tool spec:**
```
$ grep 'git-cliff@2.14.1' .github/workflows/release-tag.yml
          tool: git-cliff@2.14.1
```
PASS.

**3. No `cargo install` for git-cliff:**
```
$ grep 'cargo install.*git-cliff' .github/workflows/release-tag.yml
NOT FOUND - PASS
```
PASS.

**4. Tag-ordering invariant — step order inspection:**
```
$ grep -n 'Install git-cliff\|git push origin develop\|Create annotated tag\|git tag' .github/workflows/release-tag.yml
186:      # 5a. Install git-cliff (ADR-063 §D7 Site 3)
193:      - name: Install git-cliff (release-tag path)
201:      #     step 6 (Create annotated tag) so the annotated tag points at
369:          git push origin develop
372:      # 6. Create annotated tag on develop HEAD
374:      - name: Create annotated tag on develop HEAD
378:          git tag -a "${TAG}" \
```

Ordering: step 5a (line 193) → step 5b body with `git push origin develop` (line 369) → step 6 "Create annotated tag" (line 374). The `git push origin develop` precedes the `git tag -a` invocation. Tag-ordering invariant satisfied. **PASS.**

---

## AC-010 Part B: Live Per-Channel git-cliff Dry-Runs

Current tags in repo at HEAD `0583581ce`:
```
edge-nightly
v1.0.0-beta.1
v1.0.0-nightly.20260908
v1.0.0-nightly.20260909
v1.0.0-nightly.20260909.2
v1.0.0-nightly.20260909.3
vp-verified-VP-014-f5212641
vp-verified-VP-015-f5212641
```

**Nightly dry-run** — `--latest` with nightly pattern (no CHANGELOG mutation):

```
$ git cliff --latest \
    --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$' \
    --strip=header --output /dev/stdout

INFO  git_cliff > Using configuration from: cliff.toml
## [1.0.0-nightly.20260909.3] - 2026-09-09

### Added
- ship specs tarball + install-script spec placement (S-REL-SPECS-TARBALL-001) (#279)
```

Output shows only the delta between `v1.0.0-nightly.20260909.2` and `v1.0.0-nightly.20260909.3`.
Does NOT include commits already under `v1.0.0-beta.1` or earlier nightly tags. **PASS.**

**Stable dry-run** — `--unreleased --tag v1.0.0` with stable-only pattern (no CHANGELOG mutation):

```
$ git cliff --unreleased --tag v1.0.0 \
    --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+$' \
    --output /dev/stdout

INFO  git_cliff > Using configuration from: cliff.toml
WARN  git_cliff_core::changelog > 4 commit(s) skipped due to parse error(s)
# Changelog
...
## [1.0.0] - 2026-09-10

### Added
- CI/CD pipeline and release workflow (#1)
- developer toolchain bootstrap (#2)
...
```

Includes commits from PRs #277, #278 (which are also under nightly tags) and PR #279:

```
- git-cliff categorized notes for nightly tags (S-REL-NIGHTLY-NOTES-001) (#277)
- ship specs tarball + install-script spec placement (S-REL-SPECS-TARBALL-001) (#279)
- use git cliff --latest for nightly notes (AC-008, S-REL-NIGHTLY-NOTES-001) (#278)
```

Stable `--tag-pattern` correctly excludes ALL pre-release tags from the tag universe, so
nightly/beta-tagged commits appear as "unreleased" relative to the stable line. Channel-isolation
invariant confirmed. No prior stable tag exists in this repo; output walks from repo origin (EC-001
behavior). **PASS.**

---

## No Git-State Mutation

All commands above were read-only. Verified:

```
$ git status --short
(empty — no modified files, no new files outside docs/demo-evidence/)

$ git diff --stat
(empty)
```

No tags were created, no CHANGELOG.md was modified, no pushes were performed.

---

## Summary

All 10 acceptance criteria (AC-001 through AC-010 Part A and Part B) verified as **PASS** against
worktree HEAD `0583581ce` on branch `feature/S-REL-CHANGELOG-CHANNEL-SCOPE-001`.
