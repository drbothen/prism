#!/usr/bin/env bash
# scripts/records-lint.sh — Mechanical records-lint gate (TD-VSDD-092).
#
# Deterministic pre-commit check the orchestrator runs before every
# state-manager/factory commit. Exit 1 blocks.
#
# Checks implemented:
#   L1  — Frontmatter/changelog version parity: document's version: frontmatter
#         must match the top (latest) changelog row version.
#         (Prism convention: changelog rows are descending, top = latest.)
#   L7  — Changelog monotonic descending order: changelog version numbers must
#         decrease top-to-bottom. A row whose version exceeds the row above it
#         is a violation.
#   L9  — New-text line-cite ban: staged additions (git diff --cached added
#         lines) to .factory/ record files must not contain file:NNN line-number
#         cite patterns. Pre-existing (unchanged) lines are grandfathered.
#         Enforces the TD-VSDD-091 amendment (2026-07-24): the original
#         "pass-report changelogs" exception is retired; ALL record-tier text
#         must use symbol/anchor cites only.
#         WORKTREE NOTE: .factory/ is a git worktree on the factory-artifacts
#         orphan branch with its own separate index. L9 must query that index
#         directly (git -C .factory diff --cached), not the main project index,
#         which is blind to .factory/ additions due to .gitignore and the
#         separate worktree index. Failure to do so means L9 checks staged
#         additions to the main project only (effectively a no-op for all
#         .factory/ commits). Fixed in F-WASE-P61-LOW-001 (2026-07-24).
#   L10 — Cross-document index↔artifact version consistency gate: for each
#         artifact referenced by a registry row in BC-INDEX.md, ARCH-INDEX.md,
#         or VP-INDEX.md, the maximum version pin in that row must equal the
#         artifact file's frontmatter version:. Detects two defect classes:
#           STALE   — row's max pin < artifact version (index omitted a recent
#                     amendment; the CRIT-001/MED-006 omission direction)
#           PHANTOM — row's max pin > artifact version (index claims a version
#                     that does not exist; the fabrication direction)
#         BC-INDEX uses opening-paren "(v" extraction only: companion artifact
#         version transitions ("BC-INDEX v7.62→v7.63", "error-taxonomy v2.47→v2.48")
#         cannot be distinguished from BC own-history "→v" arrows syntactically;
#         "(v" is safe and covers the dominant active-format convention.
#         ARCH-INDEX uses first-v per row to avoid false PHANTOMs from sibling
#         artifact version mentions in the history cell. VP-INDEX uses max-of-all-v
#         per row but SKIPS draft rows and rows with no explicit version pin
#         (draft VPs have no history file; description text may carry v-prefixed
#         scope descriptors that are not version pins).
#         Always emits a positive-coverage line so the check cannot go silently
#         inert (lesson from L9's prior worktree-bypass blind spot).
#         Origin: pass-63 CRIT-001 (BC-2.16.009 v1.25 index vs v1.26 file,
#         BC-2.16.008 v0 index vs v1.6 file) + MED-006 (ADR-053 v0.34 index vs
#         v0.35 file); three consecutive adversary passes found the same class.
#
# Ratchet scoping (deliberate): L1 and L7 run only against STAGED versioned
# artifact files (the files being committed right now), not the full artifact
# corpus. Rationale: prism has pre-existing L1/L7 violations across BC and ADR
# files that predate this gate. Running full-scan on every commit would
# permanently block all factory commits until every historical violation is
# resolved — that's remediation work for a dedicated story, not a pre-commit tax.
# The ratchet ensures every file you TOUCH is left clean; the corpus improves
# incrementally with each commit that modifies a file. Use --full-scan for a
# one-time audit to discover the full backlog.
#
# Self-probe discipline (lesson MECHANICAL-GATE-COVERAGE-PARITY): every check
# claimed by this gate is demonstrated against a synthetic violation.
# Run:  scripts/records-lint.sh --self-probe
#
# Usage:
#   scripts/records-lint.sh              # L1+L7 on staged versioned files; L9 on staged diff
#   scripts/records-lint.sh --self-probe # verify each check catches a violation (exit 0/2)
#   scripts/records-lint.sh --full-scan  # L1+L7 on ALL versioned artifacts (periodic audit)
#   scripts/records-lint.sh --l9-only    # L9 staged-diff check only (fast path)
#   scripts/records-lint.sh --l1-l7-only # L1+L7 only (no staged-diff check)
#
# Exit codes:
#   0 — all checks pass
#   1 — one or more violations found; details printed to stdout
#   2 — self-probe detected a false-green check (gate is not trustworthy)
#
# Cross-applied from the CLIP email-notifications Stage-3 cascade
# (trend-gate #4 structural intervention + S3-39..S3-42 evidence),
# human-directed 2026-07-24.
# Traces to: TD-VSDD-092, TD-VSDD-091 (amendment), CLAUDE.md §Operational Discipline TDs

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ════════════════════════════════════════════════════════════════════════════════
# CONFIG BLOCK — prism artifact map
# Adjust these paths when the factory structure changes.
# All VERSIONED_ARTIFACT_DIRS paths are relative to WORKSPACE_ROOT.
# ════════════════════════════════════════════════════════════════════════════════

# L1 + L7 targets: directories whose *.md files carry YAML frontmatter with a
# version: field and a Changelog table. Script scans all *.md files in each dir
# (non-recursive: factory files are flat within their directory).
VERSIONED_ARTIFACT_DIRS=(
    ".factory/specs/behavioral-contracts"       # BC-*.md files
    ".factory/specs/architecture/decisions"     # ADR-*.md files
    ".factory/specs/verification-properties"    # VP-*.md files
    ".factory/specs/prd-supplements"            # error-taxonomy, interface-definitions,
                                                # nfr-catalog, test-vectors
                                                # (added: F-WASE-P61-MED-006 — these files
                                                # carry version: frontmatter + standard 5-col
                                                # Changelog table and are ABOVE PRD prose in
                                                # the Source-of-Truth precedence hierarchy)
    ".factory/stories"                          # per-story spec files + STORY-INDEX.md
                                                # (enabled after STORY-INDEX.md v-prefix was
                                                # normalized v2.723→2.724 in D-2015/FB45).
                                                # 13 files use v-prefixed versions (e.g. v1.1)
                                                # but all 13 have no standard Changelog table
                                                # and skip cleanly. 1 file has no YAML
                                                # frontmatter and also skips cleanly.
                                                # extract_frontmatter_version strips the
                                                # v-prefix for robustness (future-proofing).
                                                # Pre-existing: 7 L1 + 3 L7 violations
                                                # (grandfathered by ratchet).
    ".factory/specs/architecture"               # flat section docs (non-recursive; decisions/
                                                # is covered above as a separate entry and is
                                                # NOT re-scanned here — find -maxdepth 1 only).
                                                # 25 files: 1 no-version (skip), 5 no-changelog
                                                # (skip), 4 pre-existing L1+L7 violations
                                                # (grandfathered), 15 clean L1+L7.
                                                # Uses identical enable-with-ratchet rationale
                                                # as .factory/stories above.
)

# Directories deliberately excluded from L1/L7 scanning. These are NOT silent
# exclusions — they are printed at runtime so readers know what the gate does
# not cover. A silent exclusion is a false-green vector (origin of F-WASE-P61-MED-006).
#
# Currently empty: all known versioned artifact directories are covered above.
# Keep this array in place — a future exclusion must go here with a real path
# and a precise, truthful reason. Never advertise a path that does not exist on disk.
SKIPPED_ARTIFACT_DIRS_NOTICE=()

# L9 scope: staged additions under these paths trigger the line-cite ban.
# Covers all factory artifacts: BCs, ADRs, VPs, STATE.md, burst logs, etc.
# NOTE: .factory/ is a git worktree with its own index; run_l9 handles this
# automatically by querying git -C .factory diff --cached when applicable.
RECORD_DIRS_L9=(
    ".factory"
)

# ════════════════════════════════════════════════════════════════════════════════
# END CONFIG BLOCK
# ════════════════════════════════════════════════════════════════════════════════

SELF_PROBE=0
L9_ONLY=0
L1_L7_ONLY=0
FULL_SCAN=0

for arg in "${@:-}"; do
    case "${arg}" in
        --self-probe)   SELF_PROBE=1 ;;
        --l9-only)      L9_ONLY=1 ;;
        --l1-l7-only)   L1_L7_ONLY=1 ;;
        --full-scan)    FULL_SCAN=1 ;;
    esac
done

# ── Helpers ───────────────────────────────────────────────────────────────────

# Extract version string from YAML frontmatter (between first and second ---).
# Prints the bare version number (e.g. 1.2) or empty string if not found.
# Uses grep+sed to avoid macOS BSD awk 3-arg match() incompatibility.
extract_frontmatter_version() {
    local file="$1"
    # Extract the version: line from within the first frontmatter block,
    # then strip everything except the version number itself.
    awk 'BEGIN{in_fm=0; done=0}
         /^---$/{
             if(!in_fm && !done){ in_fm=1; next }
             if(in_fm){ done=1; in_fm=0; exit }
         }
         in_fm && /^version:/{
             line=$0
             gsub(/^version:[[:space:]]*/, "", line)
             gsub(/"/, "", line)
             gsub(/^v/, "", line)   # strip optional v-prefix (story files use e.g. "v1.1")
             gsub(/[[:space:]].*/, "", line)
             print line
             exit
         }
    ' "${file}" 2>/dev/null || true
}

# Extract the top (first) changelog version from a document's changelog table.
# Matches only the first column cell: | N.M[.P] | — NOT numbers embedded in the
# burst/change text columns. The closing \| anchor is critical to limit matching
# to the version cell only.
extract_changelog_versions() {
    local file="$1"
    grep -oE '^\|[[:space:]]*[0-9]+\.[0-9]+(\.[0-9]+)?[[:space:]]*\|' "${file}" 2>/dev/null \
        | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' \
        | head -1  # top row only (latest version); L7 uses extract_all_changelog_versions
}

# Extract ALL changelog version strings from the version column, top-to-bottom.
# Uses the same first-cell anchor to avoid matching version numbers embedded
# in burst IDs, change descriptions, or other columns.
extract_all_changelog_versions() {
    local file="$1"
    grep -oE '^\|[[:space:]]*[0-9]+\.[0-9]+(\.[0-9]+)?[[:space:]]*\|' "${file}" 2>/dev/null \
        | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' \
        || true
}

# Semantic version comparison: prints "gt", "lt", or "eq" for $1 vs $2.
semver_compare() {
    local a="$1" b="$2"
    local a_maj a_min a_pat b_maj b_min b_pat
    # Append ".0.0" so that versions with fewer components (e.g. "1.2") are
    # treated as "1.2.0" without introducing unset-variable errors.
    IFS='.' read -r a_maj a_min a_pat _rest <<< "${a}.0.0"
    IFS='.' read -r b_maj b_min b_pat _rest <<< "${b}.0.0"
    a_maj="${a_maj:-0}"; a_min="${a_min:-0}"; a_pat="${a_pat:-0}"
    b_maj="${b_maj:-0}"; b_min="${b_min:-0}"; b_pat="${b_pat:-0}"
    if   [ "${a_maj}" -gt "${b_maj}" ]; then echo "gt"
    elif [ "${a_maj}" -lt "${b_maj}" ]; then echo "lt"
    elif [ "${a_min}" -gt "${b_min}" ]; then echo "gt"
    elif [ "${a_min}" -lt "${b_min}" ]; then echo "lt"
    elif [ "${a_pat}" -gt "${b_pat}" ]; then echo "gt"
    elif [ "${a_pat}" -lt "${b_pat}" ]; then echo "lt"
    else echo "eq"
    fi
}

# ── L1 — Frontmatter/changelog version parity ────────────────────────────────
# Prism convention: top row of changelog = latest version; must match frontmatter.
run_l1() {
    local file="$1"
    local fm_ver first_cl_ver

    fm_ver="$(extract_frontmatter_version "${file}")"
    [ -z "${fm_ver}" ] && return 0   # no frontmatter version — skip

    first_cl_ver="$(extract_changelog_versions "${file}")"
    [ -z "${first_cl_ver}" ] && return 0   # no changelog table — skip

    if [ "${fm_ver}" != "${first_cl_ver}" ]; then
        echo "L1 FAIL [${file}]: frontmatter version=${fm_ver} does not match" \
             "top changelog row version=${first_cl_ver} (top = latest)"
        return 1
    fi
    return 0
}

# ── L7 — Changelog monotonic descending order ─────────────────────────────────
# Prism convention: top = latest. Each row's version must be <= the row above it.
# A row whose version EXCEEDS the preceding row is a violation.
run_l7() {
    local file="$1"
    local prev_ver="" row_num=0 exit_code=0

    # Collect versions into an array to avoid subshell variable-scoping issues
    local versions=()
    while IFS= read -r ver; do
        versions+=("${ver}")
    done < <(extract_all_changelog_versions "${file}")

    for ver in "${versions[@]+"${versions[@]}"}"; do
        row_num=$((row_num + 1))
        if [ -n "${prev_ver}" ]; then
            local cmp
            cmp="$(semver_compare "${ver}" "${prev_ver}")"
            if [ "${cmp}" = "gt" ]; then
                echo "L7 FAIL [${file}]: changelog row ${row_num} version ${ver}" \
                     "exceeds preceding row ${prev_ver} (must be descending, top=latest)"
                exit_code=1
            fi
        fi
        prev_ver="${ver}"
    done

    return "${exit_code}"
}

# ── L9 pattern construction ───────────────────────────────────────────────────
# Four arms; each targets a distinct live form of volatile line-cite in the
# prism .factory/ corpus. Enforces the TD-VSDD-091 amendment (2026-07-24):
# ALL record-tier text must use symbol/anchor cites; line-number cites banned.
#
# Arm 1 — filename.ext:NNN  (e.g. pipeline.rs:142, bc-2.01.001.md:50)
#   Original pattern. Excludes URL ports by requiring a known file extension
#   before the colon.
_L9_ARM1='\b[A-Za-z0-9_][A-Za-z0-9_.-]*\.(rs|md|toml|yaml|yml|py|sh|txt|json|ts|tsx|js|jsx):[0-9]+'
#
# Arm 2 — `path/to/file.ext` line(s) ~?NNN
#   Backtick-quoted filename/path + "line" or "lines" keyword + optional ~ + number.
#   Matches: `pipeline.rs` line ~975, `crates/prism-mcp/src/server.rs` lines 1742
#   Path chars: letters, digits, underscore, dot, slash, hyphen (hyphen last = literal).
_L9_ARM2='`[-A-Za-z0-9_./]+\.(rs|md|toml|yaml|yml|py|sh|txt|json|ts|tsx|js|jsx)`[[:space:]]+lines?[[:space:]]+~?[0-9]+'
#
# Arm 3 — Line(s) ~NNN  (e.g. Line ~975:, Lines ~410-412)
#   Case-insensitive "line"/"Line"/"lines"/"Lines" + tilde + number.
#   [Ll] covers the capitalised form found in ADR changelogs. Tilde required to
#   distinguish from casual English use ("line 3 of the agenda", "the bottom line:").
_L9_ARM3='\b[Ll]ines?[[:space:]]+~[0-9]+'
#
# Arm 4 — DOCNAME vX.Y:NNN  (e.g. ARCH-INDEX v2.193:154)
#   All-caps document identifier + version + colon + line number (no file extension).
#   Does NOT match: BC-2.16.009 v1.24 (no :NNN), v2.193:154 (no all-caps prefix),
#   RFC 9110 §5.6.2 (no version component).
_L9_ARM4='\b[A-Z][A-Z0-9_-]+[[:space:]]+v[0-9]+\.[0-9]+:[0-9]+'
#
# Arm 5 — ~L<NN+> and bare L<NN+>  (e.g. ~L215, L864)
#   Covers two forms of positional line cite found in prism record-tier text:
#   (a) ~L<NN+>: tilde + uppercase L + 2+ digits. Requires 2+ digits to exclude
#       single-digit network-layer references (~L2, ~L3) and single-digit
#       check-names (~L7, ~L9 from this script's own documentation). Matches
#       the VP-INDEX v2.12 form: ~L215/~L957/~L985 (D-2007 2026-07-24).
#   (b) \bL<NN+>\b: bare uppercase L + 2+ digits at word boundaries. Captures
#       the burst-log/adversary-review form: L864, L850, L80, etc. Requires 2+
#       digits to exclude L1/L7/L9 (records-lint check names) and L2/L3 (network
#       layer references without tilde). Does NOT catch single-digit ~L7 (arm-5
#       requires 2+ digits), nor bare slash-continuation digits like
#       ~L500/535/558/662 where only the first segment has the L prefix.
#   NOTE: arm-3 is complementary — it catches "Lines ~NNN" (keyword+tilde+digits);
#   arm-5 catches ~L<NNN> and bare L<NNN> without any preceding keyword.
_L9_ARM5='(~L[0-9]{2,}|\bL[0-9]{2,}\b)'

L9_CITE_PATTERN="(${_L9_ARM1}|${_L9_ARM2}|${_L9_ARM3}|${_L9_ARM4}|${_L9_ARM5})"

# ── L9 — New-text line-cite ban ───────────────────────────────────────────────
# Staged additions to .factory/ files must not contain any of the L9_CITE_PATTERN
# line-cite forms. Uses a git dir parameter so the self-probe can override
# WORKSPACE_ROOT.
#
# WORKTREE BYPASS FIX (F-WASE-P61-LOW-001): .factory/ is a git worktree mounted
# at WORKSPACE_ROOT/.factory on the factory-artifacts orphan branch. It has its
# own separate git index (.git/worktrees/-factory/index). The main project has
# .factory/ in its .gitignore, so `git -C WORKSPACE_ROOT diff --cached -- .factory`
# always returns empty — the main index has no knowledge of .factory/ additions.
# This caused a complete bypass: L9 returned 0 immediately on every commit since
# the gate was introduced (TD-VSDD-092 / 2026-07-24). The fix: when git_root is
# WORKSPACE_ROOT and WORKSPACE_ROOT/.factory/.git is a file (worktree link), query
# the .factory/ worktree's own index directly via git -C .factory diff --cached.
# The self-probe skips this branch (its temp repos have no nested worktree).
run_l9() {
    local git_root="${1:-${WORKSPACE_ROOT}}"
    shift || true
    local record_dirs=("${@:-${RECORD_DIRS_L9[@]}}")

    # Accumulate diff output from all relevant git indexes.
    local combined_diff=""

    # 1. Standard diff from the specified git_root (main project or self-probe repo).
    local main_diff
    main_diff="$(
        git -C "${git_root}" diff --cached --unified=0 -- "${record_dirs[@]}" 2>/dev/null \
        || true
    )"
    if [ -n "${main_diff}" ]; then
        combined_diff="${main_diff}"
    fi

    # 2. .factory/ worktree diff (production only).
    #    Skip when: self-probe has overridden git_root (temp repos have no nested
    #    worktree), or when running on a repo where .factory/ is not a worktree.
    if [ "${git_root}" = "${WORKSPACE_ROOT}" ] && \
       [ -f "${WORKSPACE_ROOT}/.factory/.git" ]; then
        local factory_diff
        factory_diff="$(
            git -C "${WORKSPACE_ROOT}/.factory" diff --cached --unified=0 2>/dev/null \
            || true
        )"
        if [ -n "${factory_diff}" ]; then
            # Prefix path headers with .factory/ so violation messages name the
            # full project-relative path (e.g. .factory/specs/.../ADR-NNN.md).
            factory_diff="$(
                printf '%s\n' "${factory_diff}" \
                | sed -e 's|^+++ b/|+++ b/.factory/|' \
                      -e 's|^--- a/|--- a/.factory/|'
            )"
            if [ -n "${combined_diff}" ]; then
                combined_diff="${combined_diff}"$'\n'"${factory_diff}"
            else
                combined_diff="${factory_diff}"
            fi
        fi
    fi

    # Nothing staged in any relevant index — nothing to check.
    [ -z "${combined_diff}" ] && return 0

    local violations=() file_ctx=""

    while IFS= read -r line; do
        # Track current file context from diff headers
        if [[ "${line}" =~ ^\+\+\+[[:space:]]b/(.+)$ ]]; then
            file_ctx="${BASH_REMATCH[1]}"
            continue
        fi
        [[ "${line}" =~ ^\+\+\+ ]] && continue
        [[ "${line}" =~ ^---     ]] && continue
        [[ "${line}" =~ ^\+      ]] || continue

        local content="${line:1}"  # strip leading +

        if echo "${content}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
            local cites
            cites="$(echo "${content}" | grep -oE "${L9_CITE_PATTERN}" 2>/dev/null \
                | head -3 | paste -sd ' ')"
            violations+=("L9 FAIL [${file_ctx:-unknown}]: staged addition contains line-cite: ${cites}")
            violations+=("  Line: ${content:0:120}")
        fi
    done <<< "${combined_diff}"

    if [ "${#violations[@]}" -gt 0 ]; then
        for v in "${violations[@]}"; do
            echo "${v}"
        done
        return 1
    fi
    return 0
}

# ════════════════════════════════════════════════════════════════════════════════
# L10 — Cross-document index↔artifact version consistency gate
# ════════════════════════════════════════════════════════════════════════════════

# Helper: extract the BC's own version pin from a BC-INDEX table row.
# Extracts using two structural patterns that identify the BC's own version:
#   (vX.Y SPACE ...  (active-format point: opening-paren version with space after)
#   (vX.Y-vX.Z      (active-format range: opening-paren range; upper endpoint used)
# The space-after-version requirement excludes content parentheticals like
# "(v1.2)" or "(v1.2," that appear in row description text (output-descriptor
# references, spec-section references). The "(v" prefix excludes companion artifact
# references which always appear as "ARTIFACT_NAME v..." (no leading paren).
# Returns the maximum of all extracted pins as a bare number (no 'v').
# Returns "0" (sentinel) when no structural pin is found.
#
# DESIGN NOTE — why "(v" only, not "→v" or max-of-all-v:
# BC rows embed companion artifact version transitions in their history cells:
# "BC-INDEX v7.62→v7.63", "error-taxonomy v2.47→v2.48", "ADR-054 v0.35". These
# companion transitions use the same "→v" arrow syntax as a BC's own draft-format
# history ("v1.0→v1.1"). There is no reliable syntactic way to distinguish the
# BC's own arrow transitions from companion artifact transitions in a single row
# without full field-position parsing. The "→v" approach therefore produces false
# PHANTOMs (e.g., BC-INDEX transition v7.62→v7.63 in a BC-2.16.002 row inflates
# the pin to 7.63 > BC artifact v2.10). For safety, only the "(v" opening-paren
# pattern is used; it matches the dominant active-format convention and produces
# zero false positives. Rows with only "→v"-style history (draft-format only,
# no active "(" entry) are counted as no-pin and skipped rather than guessed.
_l10_bc_pin_from_row() {
    local row="$1"
    local max="0"
    local ver
    while IFS= read -r ver; do
        [ -z "${ver}" ] && continue
        if [ "$(semver_compare "${ver}" "${max}")" = "gt" ]; then
            max="${ver}"
        fi
    done < <(
        # Point-version active-format: "(v1.25 — description" → requires [space]
        # after version number to exclude content parentheticals like "(v1.2)".
        printf '%s\n' "${row}" \
            | grep -oE '\(v[0-9]+\.[0-9]+(\.[0-9]+)?[[:space:]]' \
            | sed 's/^.v//; s/[[:space:]]$//' 2>/dev/null
        # Range active-format: "(v1.4-v1.6 — ..." → take upper (max) endpoint.
        # Covers ASCII hyphen; en-dash variant handled below if present.
        printf '%s\n' "${row}" \
            | grep -oE '\(v[0-9]+\.[0-9]+(\.[0-9]+)?-v[0-9]+\.[0-9]+(\.[0-9]+)?' \
            | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?$' 2>/dev/null
    )
    printf '%s\n' "${max}"
}

# Helper: extract the FIRST v\d+\.\d+ pin from a line, return bare (no 'v').
# Returns empty string when no pin found.
# Used for ARCH-INDEX rows where the first v-prefixed version is always the ADR's
# current STATUS version (due to descending history order in the Status cell).
# Using max-of-all would produce false PHANTOMs when companion artifact version
# mentions appear later in the Status+History cell (e.g., "BC-2.16.009 v1.25→v1.26"
# appearing inside an ADR-053 row would inflate max above the ADR's own version).
_l10_first_pin_from_line() {
    local line="$1"
    printf '%s\n' "${line}" \
        | grep -oE '\bv[0-9]+\.[0-9]+(\.[0-9]+)?\b' \
        | head -1 \
        | sed 's/^v//'
}

# Helper: extract all v\d+\.\d+ pins from a line, return the maximum (bare).
# Returns "0" when no pin found. Used for VP-INDEX rows.
# VP rows are simple (e.g. "active — v0.28") with no companion version references,
# making max-of-all safe.
_l10_max_pin_from_line() {
    local line="$1"
    local max="0"
    local ver
    while IFS= read -r ver; do
        [ -z "${ver}" ] && continue
        if [ "$(semver_compare "${ver}" "${max}")" = "gt" ]; then
            max="${ver}"
        fi
    done < <(printf '%s\n' "${line}" \
             | grep -oE '\bv[0-9]+\.[0-9]+(\.[0-9]+)?\b' \
             | sed 's/^v//' 2>/dev/null)
    printf '%s\n' "${max}"
}

# Helper: resolve an artifact ID to a file path in a directory via glob.
# Returns the first match (full absolute path) or empty string.
# Glob form: "${glob_pfx}-*.md" (e.g. "BC-2.16.009-*.md", "vp-153-*.md").
#
# AMENDMENT DISAMBIGUATION: when the ID itself does not contain "AMENDMENT",
# AMENDMENT-suffixed files are excluded from the glob results. This prevents
# alphabetical sort from selecting ADR-026-AMENDMENT-*.md before ADR-026-*.md
# when both exist (uppercase 'A' sorts before lowercase 's'). For AMENDMENT IDs
# (e.g. ADR-026-AMENDMENT), the filter is not applied — the AMENDMENT file
# is the correct target.
_l10_resolve_artifact() {
    local abs_dir="$1"
    local glob_pfx="$2"
    if [[ "${glob_pfx}" != *AMENDMENT* ]]; then
        find "${abs_dir}" -maxdepth 1 -name "${glob_pfx}-*.md" -print 2>/dev/null \
            | grep -v 'AMENDMENT' | sort | head -1
    else
        find "${abs_dir}" -maxdepth 1 -name "${glob_pfx}-*.md" -print 2>/dev/null \
            | sort | head -1
    fi
}

# ── L10 ───────────────────────────────────────────────────────────────────────
# Scans BC-INDEX.md, ARCH-INDEX.md, VP-INDEX.md for index rows that contain
# version pins. For each pinned artifact, compares the index's version pin
# against the artifact file's frontmatter version:
#
#   STALE     : pin < artifact version  (index omitted a recent amendment)
#   PHANTOM   : pin > artifact version  (index claims a non-existent version)
#   PASS      : pin == artifact version
#   UNRESOLVED: artifact file not found on disk (cannot compare)
#
# NO-PIN rows (BC rows where no structural version pin is found) are SKIPPED
# and counted separately in the coverage line. "No version claim made" is a
# distinct category from "claim is outdated" — reporting no-pin as STALE would
# generate hundreds of false positives for legitimately unpinned BC rows.
# L10 can only catch the version-number half of index drift; content-falsification
# (a row describing changes that don't exist in the artifact) is NOT detectable
# by this check and must not be claimed as a capability.
#
# Always emits a positive-coverage line even when there are no violations,
# so a silently-inert run is distinguishable from a clean run.
#
# Accepts an optional workspace root argument for self-probe isolation:
#   run_l10 [workspace_root]
# When omitted, uses WORKSPACE_ROOT from the CONFIG BLOCK.
run_l10() {
    local workspace="${1:-${WORKSPACE_ROOT}}"

    local total_cells=0
    local pass_count=0
    local nopin_count=0
    local stale_count=0
    local phantom_count=0
    local unresolved_count=0
    local violations=()

    # ── BC-INDEX ──────────────────────────────────────────────────────────────
    # Pin extraction strategy: structural patterns only.
    # _l10_bc_pin_from_row extracts versions using only the "(vX.Y" opening-paren
    # pattern (active-format). The "→v" arrow pattern was found to produce false
    # PHANTOMs from companion artifact transitions embedded in BC history descriptions
    # (e.g., "BC-INDEX v7.62→v7.63", "error-taxonomy v2.47→v2.48") and was removed.
    # Rows with no "(v" structural pin → nopin_count (skip, no violation);
    # "no claim made" ≠ "claim is outdated."
    local bc_index="${workspace}/.factory/specs/behavioral-contracts/BC-INDEX.md"
    if [ -f "${bc_index}" ]; then
        local bc_dir="${workspace}/.factory/specs/behavioral-contracts"
        while IFS= read -r row; do
            # Match table data rows: line starts with | followed by BC-N.NN.NNN
            [[ "${row}" =~ ^[|][[:space:]]*(BC-[0-9]+\.[0-9]+\.[0-9]+)[[:space:]]*[|] ]] \
                || continue
            local aid="${BASH_REMATCH[1]}"
            total_cells=$((total_cells + 1))

            local bc_pin
            bc_pin="$(_l10_bc_pin_from_row "${row}")"

            # No structural pin found → skip this row (no version claim to verify)
            if [ "${bc_pin}" = "0" ]; then
                nopin_count=$((nopin_count + 1))
                continue
            fi

            local af
            af="$(_l10_resolve_artifact "${bc_dir}" "${aid}")"
            if [ -z "${af}" ]; then
                unresolved_count=$((unresolved_count + 1))
                violations+=("L10 UNRESOLVED [${aid}]: no file matches ${aid}-*.md in behavioral-contracts/")
                continue
            fi

            local fm
            fm="$(extract_frontmatter_version "${af}")"
            if [ -z "${fm}" ]; then
                pass_count=$((pass_count + 1))
                continue
            fi

            local cmp
            cmp="$(semver_compare "${bc_pin}" "${fm}")"
            if [ "${cmp}" = "lt" ]; then
                stale_count=$((stale_count + 1))
                violations+=("L10 STALE [${aid}]: index pin=${bc_pin} < artifact version=${fm}")
            elif [ "${cmp}" = "gt" ]; then
                phantom_count=$((phantom_count + 1))
                violations+=("L10 PHANTOM [${aid}]: index pin=${bc_pin} > artifact version=${fm}")
            else
                pass_count=$((pass_count + 1))
            fi
        done < "${bc_index}"
    fi

    # ── ARCH-INDEX ────────────────────────────────────────────────────────────
    # Pin extraction strategy: first-v-in-row.
    # The Status+History cell opens with "ACCEPTED vX.Y (...)" so the first
    # v-prefixed token in the full row is always the ADR's current version.
    # Using first rather than max prevents false PHANTOMs from companion artifact
    # version mentions that appear later in the history cell.
    local arch_index="${workspace}/.factory/specs/architecture/ARCH-INDEX.md"
    if [ -f "${arch_index}" ]; then
        local adr_dir="${workspace}/.factory/specs/architecture/decisions"
        while IFS= read -r row; do
            # Match table data rows: line starts with | followed by ADR-NNN
            # (including AMENDMENT suffixes like ADR-026-AMENDMENT)
            [[ "${row}" =~ ^[|][[:space:]]*(ADR-[A-Z0-9_-]+)[[:space:]]*[|] ]] \
                || continue
            local aid="${BASH_REMATCH[1]}"
            # Guard: skip the header row if it somehow matched (it shouldn't)
            [[ "${aid}" = "ID" ]] && continue
            total_cells=$((total_cells + 1))

            local first_pin
            first_pin="$(_l10_first_pin_from_line "${row}")"
            if [ -z "${first_pin}" ]; then
                unresolved_count=$((unresolved_count + 1))
                violations+=("L10 UNRESOLVED [${aid}]: no version pin found in ARCH-INDEX row")
                continue
            fi

            local af
            af="$(_l10_resolve_artifact "${adr_dir}" "${aid}")"
            if [ -z "${af}" ]; then
                unresolved_count=$((unresolved_count + 1))
                violations+=("L10 UNRESOLVED [${aid}]: no file matches ${aid}-*.md in decisions/")
                continue
            fi

            local fm
            fm="$(extract_frontmatter_version "${af}")"
            if [ -z "${fm}" ]; then
                pass_count=$((pass_count + 1))
                continue
            fi

            local cmp
            cmp="$(semver_compare "${first_pin}" "${fm}")"
            if [ "${cmp}" = "lt" ]; then
                stale_count=$((stale_count + 1))
                violations+=("L10 STALE [${aid}]: index first_pin=${first_pin} < artifact version=${fm}")
            elif [ "${cmp}" = "gt" ]; then
                phantom_count=$((phantom_count + 1))
                violations+=("L10 PHANTOM [${aid}]: index first_pin=${first_pin} > artifact version=${fm}")
            else
                pass_count=$((pass_count + 1))
            fi
        done < "${arch_index}"
    fi

    # ── VP-INDEX ──────────────────────────────────────────────────────────────
    # Pin extraction strategy: max-of-all-v (safe for VP rows).
    # VP rows are simple (e.g. "active — v0.28") with no companion references.
    # Rows with no explicit version pin → skip (VP convention; most VP rows carry
    # no version history — only actively-amended VPs include version pins).
    # VP files use a lowercase prefix: VP-153 → vp-153-*.md
    local vp_index="${workspace}/.factory/specs/verification-properties/VP-INDEX.md"
    if [ -f "${vp_index}" ]; then
        local vp_dir="${workspace}/.factory/specs/verification-properties"
        while IFS= read -r row; do
            [[ "${row}" =~ ^[|][[:space:]]*(VP-[0-9]+)[[:space:]]*[|] ]] || continue
            local aid="${BASH_REMATCH[1]}"
            total_cells=$((total_cells + 1))

            # Skip draft VP rows: draft VPs have no version history file yet.
            # Description text may contain v-prefixed scope descriptors (e.g.,
            # "v1.0 scope") that are not version pins; skip before extraction
            # to prevent false UNRESOLVEDs from scope-descriptor text.
            if [[ "${row}" =~ \|[[:space:]]*draft[[:space:]]*\| ]]; then
                nopin_count=$((nopin_count + 1))
                continue
            fi

            local max_pin
            max_pin="$(_l10_max_pin_from_line "${row}")"

            # Skip rows with no explicit version pin (VP convention)
            if [ "${max_pin}" = "0" ]; then
                nopin_count=$((nopin_count + 1))
                continue
            fi

            # VP files use lowercase prefix: VP-153 → vp-153-*.md
            local num="${aid#VP-}"
            local af
            af="$(_l10_resolve_artifact "${vp_dir}" "vp-${num}")"
            if [ -z "${af}" ]; then
                unresolved_count=$((unresolved_count + 1))
                violations+=("L10 UNRESOLVED [${aid}]: no file matches vp-${num}-*.md in verification-properties/")
                continue
            fi

            local fm
            fm="$(extract_frontmatter_version "${af}")"
            if [ -z "${fm}" ]; then
                pass_count=$((pass_count + 1))
                continue
            fi

            local cmp
            cmp="$(semver_compare "${max_pin}" "${fm}")"
            if [ "${cmp}" = "lt" ]; then
                stale_count=$((stale_count + 1))
                violations+=("L10 STALE [${aid}]: index max_pin=${max_pin} < artifact version=${fm}")
            elif [ "${cmp}" = "gt" ]; then
                phantom_count=$((phantom_count + 1))
                violations+=("L10 PHANTOM [${aid}]: index max_pin=${max_pin} > artifact version=${fm}")
            else
                pass_count=$((pass_count + 1))
            fi
        done < "${vp_index}"
    fi

    # STORY-INDEX: version comparison not implemented. STORY-INDEX is a narrative
    # changelog list rather than a version-pinned registry table; there is no
    # reliable per-story version pin structure to compare against story file
    # frontmatter.

    local mismatches=$((stale_count + phantom_count))
    # Always emit coverage line — never let the check go silently inert.
    echo "L10: ${total_cells} index cells checked, ${mismatches} mismatches" \
         "(${stale_count} STALE, ${phantom_count} PHANTOM, ${unresolved_count} UNRESOLVED)"
    # Advisory: always print no-pin count separately so "0 mismatches" is never
    # mistaken for "every row verified." 433+ BC rows use formats L10 cannot parse
    # (e.g. "vX.Y (D-NNN ...)" history); L10 reports them as unverifiable, not clean.
    echo "L10-advisory: ${nopin_count} registry rows carry no version pin (cannot be verified by L10)"

    if [ "${#violations[@]}" -gt 0 ]; then
        for v in "${violations[@]}"; do
            echo "  ${v}"
        done
        return 1
    fi
    return 0
}

# ── Self-probe ────────────────────────────────────────────────────────────────
# Verify each check catches a synthetic violation and passes a clean case.
# Per lesson MECHANICAL-GATE-COVERAGE-PARITY: every check a gate CLAIMS must be
# demonstrated against a synthetic violation before the gate is trusted.
run_self_probe() {
    echo "Running records-lint self-probe (TD-VSDD-092 / MECHANICAL-GATE-COVERAGE-PARITY)..."
    echo ""

    local probe_failures=() pass_count=0
    local tmpdir
    tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/records-lint-probe.XXXXXX")"
    # Note: cleanup deferred to caller; self-probe exits before return

    # ── L1 violation: frontmatter says 1.5, top changelog row says 1.4 ────────
    cat > "${tmpdir}/probe-l1-fail.md" <<'PROBE'
---
version: "1.5"
---
# Test Document

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.4 | 2026-01-01 | Last entry |
| 1.3 | 2026-01-01 | Earlier |
PROBE
    if run_l1 "${tmpdir}/probe-l1-fail.md" >/dev/null 2>&1; then
        probe_failures+=("L1: MISSED version mismatch (fm=1.5, top-changelog=1.4) — false-green")
    else
        echo "L1 probe PASS: version mismatch correctly flagged"
        pass_count=$((pass_count+1))
    fi

    # ── L1 clean: frontmatter matches top changelog row ─────────────────────
    cat > "${tmpdir}/probe-l1-ok.md" <<'PROBE'
---
version: "1.5"
---
# Test Document

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.5 | 2026-01-02 | Latest |
| 1.4 | 2026-01-01 | Earlier |
PROBE
    if run_l1 "${tmpdir}/probe-l1-ok.md" >/dev/null 2>&1; then
        echo "L1 probe PASS: matching versions correctly cleared"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L1: INCORRECTLY flagged matching versions (fm=1.5, top-changelog=1.5) — false-red")
    fi

    # ── L7 violation: version increases from row to row (ascending = wrong) ───
    # Prism expects descending. Row order 1.3 → 1.5 → 1.4 has an ascending step.
    cat > "${tmpdir}/probe-l7-fail.md" <<'PROBE'
---
version: "1.5"
---
# Test Document

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.3 | 2026-01-01 | First |
| 1.5 | 2026-01-02 | Out of order — higher than above |
| 1.4 | 2026-01-03 | Wrong position |
PROBE
    if run_l7 "${tmpdir}/probe-l7-fail.md" >/dev/null 2>&1; then
        probe_failures+=("L7: MISSED ascending step in changelog (1.3→1.5→1.4) — false-green")
    else
        echo "L7 probe PASS: out-of-order changelog correctly flagged"
        pass_count=$((pass_count+1))
    fi

    # ── L7 clean: descending order (top = latest = prism convention) ──────────
    cat > "${tmpdir}/probe-l7-ok.md" <<'PROBE'
---
version: "1.5"
---
# Test Document

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.5 | 2026-01-03 | Latest |
| 1.4 | 2026-01-02 | Previous |
| 1.3 | 2026-01-01 | Oldest |
PROBE
    if run_l7 "${tmpdir}/probe-l7-ok.md" >/dev/null 2>&1; then
        echo "L7 probe PASS: descending order correctly cleared"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L7: INCORRECTLY flagged valid descending changelog — false-red")
    fi

    # ── no-frontmatter file → L1 clean skip ──────────────────────────────────
    # Story files without YAML frontmatter delimiters (e.g. S-3.04-FOLLOWUP-MCP-001.md)
    # should skip cleanly: extract_frontmatter_version returns "" → return 0.
    cat > "${tmpdir}/probe-no-fm.md" <<'PROBE'
# Story: S-EXAMPLE-001
**Version:** v1.0

Content without YAML frontmatter delimiters (no opening ---).
PROBE
    if run_l1 "${tmpdir}/probe-no-fm.md" >/dev/null 2>&1; then
        echo "L1 probe PASS: no-frontmatter file correctly skipped"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L1: INCORRECTLY flagged no-frontmatter file (no --- delimiters) — false-red")
    fi

    # ── v-prefixed version + no changelog → L1 clean skip ────────────────────
    # 13 story files use v-prefixed versions (e.g. "v1.1") with no standard
    # changelog table. extract_frontmatter_version strips the v-prefix →
    # fm_ver="1.1"; no changelog → first_cl_ver="" → return 0 (skip cleanly).
    cat > "${tmpdir}/probe-v-prefix-no-cl.md" <<'PROBE'
---
version: "v1.5"
---
# Story with v-prefixed version, no standard changelog table.

## Notes

This file has no Changelog section with the standard | N.M | table format.
Should skip cleanly (no changelog → no L1 comparison possible).
PROBE
    if run_l1 "${tmpdir}/probe-v-prefix-no-cl.md" >/dev/null 2>&1; then
        echo "L1 probe PASS: v-prefix version + no changelog correctly skipped"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L1: INCORRECTLY flagged v-prefix version with no changelog (should skip) — false-red")
    fi

    # ── L9 arm-1 violation: staged addition with file.ext:NNN line-cite ──────
    # Creates an isolated git repo in a temp subdir (not touching prism's .factory/).
    local probe_repo="${tmpdir}/l9probe"
    local probe_factory="${probe_repo}/fdir"
    mkdir -p "${probe_factory}"
    git -C "${probe_repo}" init -q 2>/dev/null
    git -C "${probe_repo}" config user.email "probe@test" 2>/dev/null
    git -C "${probe_repo}" config user.name "probe" 2>/dev/null
    # Seed commit so staging is possible
    printf '# seed\n' > "${probe_factory}/seed.md"
    git -C "${probe_repo}" add fdir/seed.md 2>/dev/null
    git -C "${probe_repo}" -c commit.gpgsign=false commit -q -m "seed" 2>/dev/null
    # Stage file containing an arm-1 line-cite violation (filename.ext:NNN)
    printf '# Adversary pass\nThe issue is at pipeline.rs:142 under load.\n' \
        > "${probe_factory}/violation.md"
    git -C "${probe_repo}" add fdir/violation.md 2>/dev/null

    if run_l9 "${probe_repo}" "fdir" >/dev/null 2>&1; then
        probe_failures+=("L9-arm1: MISSED file.ext:NNN line-cite in staged addition (pipeline.rs:142) — false-green")
    else
        echo "L9-arm1 probe PASS: file.ext:NNN line-cite correctly flagged"
        pass_count=$((pass_count+1))
    fi

    # ── L9 arm-1 clean: staged addition with symbol/anchor cite only ─────────
    local probe_repo2="${tmpdir}/l9probe2"
    local probe_factory2="${probe_repo2}/fdir"
    mkdir -p "${probe_factory2}"
    git -C "${probe_repo2}" init -q 2>/dev/null
    git -C "${probe_repo2}" config user.email "probe@test" 2>/dev/null
    git -C "${probe_repo2}" config user.name "probe" 2>/dev/null
    printf '# seed\n' > "${probe_factory2}/seed.md"
    git -C "${probe_repo2}" add fdir/seed.md 2>/dev/null
    git -C "${probe_repo2}" -c commit.gpgsign=false commit -q -m "seed" 2>/dev/null
    printf '# Adversary pass\nThe issue is in `pipeline_executor::build_request`.\n' \
        > "${probe_factory2}/clean.md"
    git -C "${probe_repo2}" add fdir/clean.md 2>/dev/null

    if run_l9 "${probe_repo2}" "fdir" >/dev/null 2>&1; then
        echo "L9-arm1 probe PASS: symbol-anchor cite correctly cleared"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L9-arm1: INCORRECTLY flagged symbol-anchor cite (no file:NNN) — false-red")
    fi

    # ── L9 arm-2 violation: backtick-quoted filename + "line" keyword + NNN ──
    # e.g. `pipeline.rs` line ~975 — a common form in proposals and ADR changelogs.
    local arm2_fail='The injection site is `pipeline.rs` line ~975 in the executor.'
    if echo "${arm2_fail}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        echo "L9-arm2 probe PASS: backtick+line-keyword cite correctly flagged"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L9-arm2: MISSED backtick+line-keyword cite (\`pipeline.rs\` line ~975) — false-green")
    fi

    # ── L9 arm-2 near-miss: backtick symbol ref with no .ext + no "line" ─────
    # `pipeline_executor::build_request` — function path, no file extension → safe
    local arm2_clean='See `pipeline_executor::build_request` for the implementation.'
    if echo "${arm2_clean}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        probe_failures+=("L9-arm2: INCORRECTLY flagged backtick function ref (no .ext + no line keyword) — false-red")
    else
        echo "L9-arm2 probe PASS: backtick function ref correctly cleared"
        pass_count=$((pass_count+1))
    fi

    # ── L9 arm-3 violation: standalone "Line ~NNN" / "Lines ~NNN-NNN" ────────
    # e.g. Line ~337: — from ADR-053 §Changelog v0.31 row
    local arm3_fail='(1) Line ~337: "with both templates" should read "with all three templates".'
    if echo "${arm3_fail}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        echo "L9-arm3 probe PASS: standalone Line ~NNN cite correctly flagged"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L9-arm3: MISSED standalone Line ~NNN cite (Line ~337) — false-green")
    fi

    # ── L9 arm-3 near-miss: "line" without tilde+number ─────────────────────
    # "The bottom line:" — English phrase; no tilde+digit → must NOT trigger
    local arm3_clean='The bottom line: implement the fix using symbol anchors only.'
    if echo "${arm3_clean}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        probe_failures+=("L9-arm3: INCORRECTLY flagged English 'line' phrase (no tilde+number) — false-red")
    else
        echo "L9-arm3 probe PASS: English 'line' phrase correctly cleared"
        pass_count=$((pass_count+1))
    fi

    # ── L9 arm-4 violation: DOCNAME vX.Y:NNN ─────────────────────────────────
    # e.g. ARCH-INDEX v2.193:154 — from STORY-INDEX.md corpus
    local arm4_fail='See ARCH-INDEX v2.193:154 for the full subsystem registry.'
    if echo "${arm4_fail}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        echo "L9-arm4 probe PASS: DOCNAME vX.Y:NNN cite correctly flagged"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L9-arm4: MISSED DOCNAME vX.Y:NNN cite (ARCH-INDEX v2.193:154) — false-green")
    fi

    # ── L9 arm-4 near-miss: version ref without :NNN suffix ──────────────────
    # "BC-2.16.009 v1.24" — version reference, no line number → must NOT trigger
    local arm4_clean='Companion: BC-2.16.009 v1.24 carries the updated error template.'
    if echo "${arm4_clean}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        probe_failures+=("L9-arm4: INCORRECTLY flagged version ref without :NNN (BC-2.16.009 v1.24) — false-red")
    else
        echo "L9-arm4 probe PASS: version-only ref correctly cleared"
        pass_count=$((pass_count+1))
    fi

    # ── L9 arm-5 violation: ~L<NNN> tilde+L form ─────────────────────────────
    # Covers the VP-INDEX v2.12 changelog row form (D-2007 2026-07-24):
    #   "AC-7d new_oauth2 constructor usage at ~L215/~L957/~L985"
    # The slash-delimited run has THREE separate arm-5 matches.
    # arm-3 does NOT catch this — arm-3 requires "line" keyword before the tilde.
    local arm5_tilde_fail='AC-7d new_oauth2 constructor usage at ~L215/~L957/~L985 was already correct.'
    if echo "${arm5_tilde_fail}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        echo "L9-arm5 probe PASS: ~L<NNN> tilde+L line-cite correctly flagged"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L9-arm5: MISSED ~L<NNN> tilde+L line-cite (~L215/~L957/~L985) — false-green")
    fi

    # ── L9 arm-5 near-miss 1: ~L single-digit must NOT trigger ───────────────
    # ~L2 / ~L3 appear as OSI/Purdue network layer references in research docs
    # (e.g. "Building OT ~L2–L3"). Arm-5 requires 2+ digits to exclude these.
    local arm5_tilde_near_miss='Building OT ~L2–L3 (Purdue network layers, not line cites).'
    if echo "${arm5_tilde_near_miss}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        probe_failures+=("L9-arm5: INCORRECTLY flagged ~L2 single-digit (OSI/Purdue layer ref) — false-red")
    else
        echo "L9-arm5 probe PASS: ~L2 single-digit (network layer) correctly cleared"
        pass_count=$((pass_count+1))
    fi

    # ── L9 arm-5 violation: bare L<NNN> form ─────────────────────────────────
    # Covers burst-log/adversary-review form: "L864", "L850", "L80", etc.
    # These appear as bare positional line cites without a tilde prefix.
    local arm5_bare_fail='Body Status block retained version stamp at L864 + L850 (F-PASS7-HIGH-001).'
    if echo "${arm5_bare_fail}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        echo "L9-arm5 probe PASS: bare L<NNN> line-cite correctly flagged"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L9-arm5: MISSED bare L<NNN> line-cite (L864, L850) — false-green")
    fi

    # ── L9 arm-5 near-miss 2: L1/L7/L9 check names and SS-01 ID must NOT match
    # L7, L1, L9 are records-lint check names (single digit → excluded by 2+ rule).
    # SS-01 is a finding ID format (no L prefix). None must trigger arm-5.
    local arm5_bare_near_miss='L7 FAIL and L1 mismatch are records-lint check names; see also SS-01.'
    if echo "${arm5_bare_near_miss}" | grep -qE "${L9_CITE_PATTERN}" 2>/dev/null; then
        probe_failures+=("L9-arm5: INCORRECTLY flagged L7/L1/SS-01 near-miss (check names + finding ID) — false-red")
    else
        echo "L9-arm5 probe PASS: L7/L1/SS-01 near-miss correctly cleared"
        pass_count=$((pass_count+1))
    fi

    # ── L10 self-probe ────────────────────────────────────────────────────────
    # Probe set covers: BC STALE, BC PHANTOM, BC EQUAL-PASS, BC no-pin skip,
    # BC UNRESOLVED, BC RANGE-PASS, BC RANGE-STALE, ADR STALE (first-v strategy),
    # ADR PHANTOM false-positive prevention (sibling v-mention must not inflate to
    # PHANTOM), VP EQUAL-PASS, VP no-pin skip, plus four near-miss patterns
    # (RFC/CWE/E-SPEC/date).
    local l10d="${tmpdir}/l10probe"
    mkdir -p "${l10d}/.factory/specs/behavioral-contracts"
    mkdir -p "${l10d}/.factory/specs/architecture/decisions"
    mkdir -p "${l10d}/.factory/specs/verification-properties"

    # BC artifacts at v1.6 (used for STALE, PHANTOM, PASS, NO-PIN, UNRESOLVED probes)
    cat > "${l10d}/.factory/specs/behavioral-contracts/BC-99.01.001-stale-probe.md" <<'PROBEEOF'
---
version: "1.6"
lifecycle_status: active
status: active
---
## Changelog
| 1.6 | 2026-01-01 | Latest |
PROBEEOF
    cat > "${l10d}/.factory/specs/behavioral-contracts/BC-99.01.002-phantom-probe.md" <<'PROBEEOF'
---
version: "1.6"
lifecycle_status: active
status: active
---
## Changelog
| 1.6 | 2026-01-01 | Latest |
PROBEEOF
    cat > "${l10d}/.factory/specs/behavioral-contracts/BC-99.01.003-pass-probe.md" <<'PROBEEOF'
---
version: "1.6"
lifecycle_status: active
status: active
---
## Changelog
| 1.6 | 2026-01-01 | Latest |
PROBEEOF
    cat > "${l10d}/.factory/specs/behavioral-contracts/BC-99.01.004-nopin-probe.md" <<'PROBEEOF'
---
version: "1.6"
lifecycle_status: draft
status: draft
---
## Changelog
| 1.6 | 2026-01-01 | Latest |
PROBEEOF
    # BC-99.01.005: no file — UNRESOLVED (pin exists but artifact absent)
    # BC-99.01.006: companion-only-pin probe — verify companion v2.5 NOT extracted
    # BC-99.01.007: RANGE-PASS — range upper endpoint == artifact (must PASS)
    # BC-99.01.008: RANGE-STALE — range upper endpoint < artifact (must STALE)
    cat > "${l10d}/.factory/specs/behavioral-contracts/BC-99.01.007-range-pass-probe.md" <<'PROBEEOF'
---
version: "1.6"
lifecycle_status: active
status: active
---
## Changelog
| 1.6 | 2026-01-01 | Latest |
PROBEEOF
    cat > "${l10d}/.factory/specs/behavioral-contracts/BC-99.01.008-range-stale-probe.md" <<'PROBEEOF'
---
version: "1.6"
lifecycle_status: active
status: active
---
## Changelog
| 1.6 | 2026-01-01 | Latest |
PROBEEOF

    # BC-INDEX: structural pins to test each comparison outcome.
    #   001: active (v1.5 — ...)  → _l10_bc_pin_from_row extracts (v1.5 → pin=1.5 < artifact=1.6 → STALE
    #   002: active (v1.7 — ...; companion: some-doc.md v2.5)
    #         → extracts (v1.7 → pin=1.7 > artifact=1.6 → PHANTOM
    #         → companion v2.5 NOT extracted (no (v before it) — companion exclusion test
    #   003: active (v1.6 — ...; per ADR-100 v0.35)
    #         → extracts (v1.6 → pin=1.6 == artifact=1.6 → PASS
    #         → ADR-100 v0.35 NOT extracted
    #   004: draft (no parens)
    #         → no (v → pin=0 → no-pin skip (no violation)
    #   005: active (v1.0 — placeholder) → pin=1.0, but NO FILE → UNRESOLVED
    #   006: active (companion: some-doc.md v2.5) → no (v after ( (text follows, not v) → pin=0 → no-pin skip
    #   007: active (v1.4-v1.6 — ...) → range upper=1.6 == artifact=1.6 → PASS
    #   008: active (v1.4-v1.5 — ...) → range upper=1.5 < artifact=1.6 → STALE
    cat > "${l10d}/.factory/specs/behavioral-contracts/BC-INDEX.md" <<'PROBEEOF'
---
version: "1.0"
document_type: behavioral-contract-index
---
| BC-99.01.001 | STALE probe | SS-01 | CAP-001 | P0 | active (v1.5 — prior amendment; v1.4 initial) |
| BC-99.01.002 | PHANTOM probe | SS-01 | CAP-001 | P0 | active (v1.7 — claimed; v1.6 prior; companion: some-doc.md v2.5) |
| BC-99.01.003 | PASS probe | SS-01 | CAP-001 | P0 | active (v1.6 — latest; per ADR-100 v0.35) |
| BC-99.01.004 | NO-PIN probe | SS-01 | CAP-001 | P0 | draft |
| BC-99.01.005 | UNRESOLVED probe | SS-01 | CAP-001 | P0 | active (v1.0 — placeholder) |
| BC-99.01.006 | COMPANION-ONLY probe | SS-01 | CAP-001 | P0 | active (companion: some-doc.md v2.5) |
| BC-99.01.007 | RANGE-PASS probe | SS-01 | CAP-001 | P0 | active (v1.4-v1.6 — range upper equals artifact) |
| BC-99.01.008 | RANGE-STALE probe | SS-01 | CAP-001 | P0 | active (v1.4-v1.5 — range upper below artifact) |
PROBEEOF

    # ADR artifact at v0.35
    cat > "${l10d}/.factory/specs/architecture/decisions/ADR-999-arch-stale-probe.md" <<'PROBEEOF'
---
version: "0.35"
status: accepted
---
## Changelog
| 0.35 | 2026-01-01 | Latest |
PROBEEOF

    # ARCH-INDEX: ADR-999 first-v=0.34 but artifact is 0.35 → STALE.
    # The row also contains "companion ADR-100 v1.5" deeper in the history cell
    # to verify that first-v (not max-v) is used — if max-v were used, we'd get
    # PHANTOM (v1.5 > v0.35) instead of the correct STALE.
    cat > "${l10d}/.factory/specs/architecture/ARCH-INDEX.md" <<'PROBEEOF'
---
version: "1.0"
document_type: architecture-index
---
| ADR-999 | Arch STALE probe | ACCEPTED v0.34 (v0.33 prior; reference companion ADR-100 v1.5 noted for context) | 2026-01-01 | decisions/ADR-999-arch-stale-probe.md |
PROBEEOF

    # VP artifact at v0.28
    cat > "${l10d}/.factory/specs/verification-properties/vp-999-pass-probe.md" <<'PROBEEOF'
---
version: "0.28"
status: active
---
## Changelog
| 0.28 | 2026-01-01 | Latest |
PROBEEOF

    # VP-INDEX: VP-999 with pin v0.28 (PASS), VP-998 with no pin (should SKIP)
    cat > "${l10d}/.factory/specs/verification-properties/VP-INDEX.md" <<'PROBEEOF'
---
version: "1.0"
document_type: verification-property-index
---
| VP-999 | VP PASS probe | prism-test | kani | P0 | active — v0.28 | S-1.01 |
| VP-998 | VP no-pin probe | prism-test | kani | P0 | draft | S-1.02 |
PROBEEOF

    local l10_out
    l10_out="$(run_l10 "${l10d}" 2>&1)"

    # Case 1: BC STALE — structural pin v1.5 (from "(v1.5 — ...") < artifact v1.6
    if echo "${l10_out}" | grep -q "STALE \[BC-99\.01\.001\]"; then
        echo "L10 probe PASS: BC STALE correctly detected (structural pin=1.5 < artifact=1.6)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-STALE: MISSED BC STALE (structural pin=1.5 < artifact v1.6). Output: $(echo "${l10_out}" | head -3)")
    fi

    # Case 2: BC PHANTOM — structural pin v1.7 > artifact v1.6.
    # Row also embeds "companion: some-doc.md v2.5" — must NOT produce PHANTOM from v2.5.
    if echo "${l10_out}" | grep -q "PHANTOM \[BC-99\.01\.002\]"; then
        echo "L10 probe PASS: BC PHANTOM correctly detected (structural pin=1.7 > artifact=1.6)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-PHANTOM: MISSED BC PHANTOM (structural pin=1.7 > artifact v1.6). Output: $(echo "${l10_out}" | head -3)")
    fi

    # Case 3: BC EQUAL-PASS — structural pin v1.6 == artifact v1.6, must NOT appear.
    # Row embeds "per ADR-100 v0.35" — ADR reference must NOT be extracted.
    if ! echo "${l10_out}" | grep -q "\[BC-99\.01\.003\]"; then
        echo "L10 probe PASS: BC EQUAL-PASS correctly cleared (structural pin=1.6 == artifact=1.6)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-EQUAL-PASS: INCORRECTLY flagged matching versions for BC-99.01.003. Output: $(echo "${l10_out}" | grep 'BC-99.01.003')")
    fi

    # Case 4: BC NO-PIN SKIP — row has "draft" with no (v or →v → pin=0 → skipped.
    # "No version claim made" ≠ "claim is outdated"; must NOT appear in violations.
    if ! echo "${l10_out}" | grep -q "\[BC-99\.01\.004\]"; then
        echo "L10 probe PASS: BC no-pin row correctly skipped (no structural pin in 'draft' row)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-NO-PIN-SKIP: INCORRECTLY flagged BC-99.01.004 no-pin row. Output: $(echo "${l10_out}" | grep 'BC-99.01.004')")
    fi

    # Case 5: BC UNRESOLVED — structural pin v1.0 found but artifact file absent.
    if echo "${l10_out}" | grep -q "UNRESOLVED \[BC-99\.01\.005\]"; then
        echo "L10 probe PASS: BC UNRESOLVED correctly detected (pin=1.0, artifact file absent)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-UNRESOLVED: MISSED UNRESOLVED for BC-99.01.005 (pin=1.0, no artifact file). Output: $(echo "${l10_out}" | head -3)")
    fi

    # Case 5b: BC companion-only row (BC-99.01.006) — "active (companion: some-doc.md v2.5)".
    # "(companion:" has ( followed by "c" not "v", so no (v extraction; no →v arrows.
    # → pin=0 → no-pin skip; must NOT appear in violations.
    if ! echo "${l10_out}" | grep -q "\[BC-99\.01\.006\]"; then
        echo "L10 probe PASS: BC companion-only row correctly skipped (no structural (v or →v)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-COMPANION-ONLY: INCORRECTLY flagged BC-99.01.006 companion-only row. Output: $(echo "${l10_out}" | grep 'BC-99.01.006')")
    fi

    # Case 6: ADR STALE — first_pin=0.34 < artifact=0.35
    if echo "${l10_out}" | grep -q "STALE \[ADR-999\]"; then
        echo "L10 probe PASS: ADR STALE correctly detected (first_pin=0.34 < artifact=0.35)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-ADR-STALE: MISSED ADR STALE (first_pin=0.34 < artifact v0.35). Output: $(echo "${l10_out}" | head -5)")
    fi

    # Case 7: ADR first-v PHANTOM false-positive prevention.
    # The ARCH-INDEX row for ADR-999 contains "ADR-100 v1.5" deep in the history
    # cell. If max-v were used instead of first-v, we'd get PHANTOM (1.5 > 0.35).
    # Verify PHANTOM is NOT reported for ADR-999.
    if echo "${l10_out}" | grep -q "PHANTOM \[ADR-999\]"; then
        probe_failures+=("L10-ADR-FP: INCORRECTLY reported PHANTOM for ADR-999 due to sibling v1.5 mention (should use first-v, not max-v). Output: $(echo "${l10_out}" | grep 'ADR-999')")
    else
        echo "L10 probe PASS: ADR first-v correctly avoids PHANTOM from sibling v1.5 mention"
        pass_count=$((pass_count+1))
    fi

    # Case 8: VP EQUAL-PASS — max_pin=0.28 == artifact=0.28, must NOT appear
    if ! echo "${l10_out}" | grep -q "\[VP-999\]"; then
        echo "L10 probe PASS: VP EQUAL-PASS correctly cleared (max_pin=0.28 == artifact=0.28)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-VP-PASS: INCORRECTLY flagged VP-999 matching versions. Output: $(echo "${l10_out}" | grep 'VP-999')")
    fi

    # Case 9: VP no-pin skip — VP-998 has only 'draft', no v-pin → must NOT appear
    if ! echo "${l10_out}" | grep -q "\[VP-998\]"; then
        echo "L10 probe PASS: VP no-pin row correctly skipped (VP-998 has no version pin)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-VP-NOPIN: INCORRECTLY flagged VP-998 no-pin row. Output: $(echo "${l10_out}" | grep 'VP-998')")
    fi

    # Case 10: BC RANGE-PASS — range upper endpoint v1.6 == artifact v1.6 → PASS.
    # Row: "active (v1.4-v1.6 — range upper equals artifact)"
    # Must NOT appear in violations (the upper endpoint matches the artifact).
    if ! echo "${l10_out}" | grep -q "\[BC-99\.01\.007\]"; then
        echo "L10 probe PASS: BC range PASS correctly cleared (range upper=1.6 == artifact=1.6)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-RANGE-PASS: INCORRECTLY flagged BC-99.01.007 range PASS. Output: $(echo "${l10_out}" | grep 'BC-99.01.007')")
    fi

    # Case 11: BC RANGE-STALE — range upper endpoint v1.5 < artifact v1.6 → STALE.
    # Row: "active (v1.4-v1.5 — range upper below artifact)"
    # Must appear as STALE (the index claims the BC stopped at v1.5 but file is v1.6).
    if echo "${l10_out}" | grep -q "STALE \[BC-99\.01\.008\]"; then
        echo "L10 probe PASS: BC range STALE correctly detected (range upper=1.5 < artifact=1.6)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-RANGE-STALE: MISSED BC range STALE (range upper=1.5 < artifact v1.6). Output: $(echo "${l10_out}" | head -3)")
    fi

    # ── L10 near-miss probes: false-positive prevention ───────────────────────
    # These test that the v-prefixed pattern correctly excludes tokens that look
    # like numbers but carry no 'v' prefix: RFC numbers, CWE IDs, error codes,
    # and ISO dates. All must return the "0" sentinel (no pin found).

    local rfc_line='This implements RFC 9110 section 5.6.2 for header validation compliance.'
    local rfc_pin
    rfc_pin="$(_l10_max_pin_from_line "${rfc_line}")"
    if [ "${rfc_pin}" = "0" ]; then
        echo "L10 near-miss PASS: RFC 9110 correctly yields no version pin (no v-prefix)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-RFC-FP: INCORRECTLY extracted version '${rfc_pin}' from RFC 9110 reference")
    fi

    local cwe_line='Mitigates CWE-208 constant-time comparison and CWE-20 input validation.'
    local cwe_pin
    cwe_pin="$(_l10_max_pin_from_line "${cwe_line}")"
    if [ "${cwe_pin}" = "0" ]; then
        echo "L10 near-miss PASS: CWE-208/CWE-20 correctly yield no version pin (no v-prefix)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-CWE-FP: INCORRECTLY extracted version '${cwe_pin}' from CWE ID references")
    fi

    local ecode_line='Emits E-SPEC-028(b) and E-SENSOR-014 when auth_type has wrong value.'
    local ecode_pin
    ecode_pin="$(_l10_max_pin_from_line "${ecode_line}")"
    if [ "${ecode_pin}" = "0" ]; then
        echo "L10 near-miss PASS: E-SPEC-028/E-SENSOR-014 correctly yield no version pin (no v-prefix)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-ECODE-FP: INCORRECTLY extracted version '${ecode_pin}' from error code references")
    fi

    local date_line='This was fixed on 2026-07-24 as part of decision D-2013.'
    local date_pin
    date_pin="$(_l10_max_pin_from_line "${date_line}")"
    if [ "${date_pin}" = "0" ]; then
        echo "L10 near-miss PASS: date 2026-07-24 correctly yields no version pin (no v-prefix)"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L10-DATE-FP: INCORRECTLY extracted version '${date_pin}' from date 2026-07-24")
    fi

    # Cleanup temp dir (safe: it's /tmp/records-lint-probe.*, not prism's .factory/)
    ( cd / && rm -rf "${tmpdir}" ) 2>/dev/null || true

    # ── Report ─────────────────────────────────────────────────────────────────
    echo ""
    echo "Self-probe results: ${pass_count} checks passed"
    if [ "${#probe_failures[@]}" -gt 0 ]; then
        echo ""
        echo "SELF-PROBE FAILURES (${#probe_failures[@]}):"
        for f in "${probe_failures[@]}"; do
            echo "  ${f}"
        done
        echo ""
        echo "FAIL (exit 2): gate has untrustworthy check(s) — see above."
        echo "  Per lesson MECHANICAL-GATE-COVERAGE-PARITY: fix the checks before"
        echo "  deploying this script as a blocking gate."
        exit 2
    fi

    echo ""
    echo "SELF-PROBE PASS: all ${pass_count} checks correctly detect/pass synthetic violations."
    echo ""
    echo "Coverage notes (not self-probed; verify manually):"
    echo "  L1: files with frontmatter but no version: key (should skip)"
    echo "  L7: single-row changelog (should pass — no ordering comparison possible)"
    echo "  L9: URL port http://host:8080 must NOT trigger arm-1 — validate manually"
    echo "  L9: unchanged line with file:NNN must NOT trigger — validate manually"
    echo "  L9: worktree bypass fix (git -C .factory diff --cached) is exercised at"
    echo "       runtime only; not replicable in self-probe temp repos (no nested worktree)."
    echo "  L9 arm-5 coverage gap: bare slash-continuation digits (e.g. ~L500/535/558/662 —"
    echo "       segments after first slash have no L prefix and are not caught by arm-5)."
    echo "Self-probed cases added 2026-07-24:"
    echo "  L1: no-frontmatter file (no --- delimiters) → clean skip"
    echo "  L1: v-prefixed version + no changelog table → clean skip"
    echo "  L9 arm-5: ~L<NNN> tilde+L form (~L215/~L957/~L985) → flagged"
    echo "  L9 arm-5: ~L2 single-digit (OSI layer) → cleared (2+ digit threshold)"
    echo "  L9 arm-5: bare L<NNN> form (L864, L850) → flagged"
    echo "  L9 arm-5: L7/L1/SS-01 near-miss (check names + finding ID) → cleared"
    echo "Self-probed cases added 2026-07-25 (L10):"
    echo "  L10 BC STALE: structural pin=1.5 (from '(v1.5 — ...') < artifact=1.6 → flagged"
    echo "  L10 BC PHANTOM: structural pin=1.7 > artifact=1.6; companion v2.5 excluded → flagged on 1.7"
    echo "  L10 BC EQUAL-PASS: structural pin=1.6 == artifact=1.6; ADR-100 v0.35 excluded → cleared"
    echo "  L10 BC no-pin SKIP: 'draft' row has no (v → pin=0 → skipped (not STALE)"
    echo "  L10 BC UNRESOLVED: pin=1.0 found but artifact file absent → UNRESOLVED reported"
    echo "  L10 BC companion-only: 'active (companion: doc v2.5)' → no (v pin → skipped"
    echo "  L10 ADR STALE: first_pin=0.34 < artifact=0.35 → flagged"
    echo "  L10 ADR first-v FP prevention: sibling v1.5 in row must not produce PHANTOM → cleared"
    echo "  L10 VP EQUAL-PASS: max_pin=0.28 == artifact=0.28 → cleared"
    echo "  L10 VP no-pin skip: row with only 'draft' (no v-pin) → skipped"
    echo "  L10 BC RANGE-PASS: range '(v1.4-v1.6 — ...)' upper=1.6 == artifact=1.6 → cleared"
    echo "  L10 BC RANGE-STALE: range '(v1.4-v1.5 — ...)' upper=1.5 < artifact=1.6 → flagged"
    echo "  L10 near-miss RFC 9110: no v-prefix → no pin extracted"
    echo "  L10 near-miss CWE-208/CWE-20: no v-prefix → no pin extracted"
    echo "  L10 near-miss E-SPEC-028/E-SENSOR-014: no v-prefix → no pin extracted"
    echo "  L10 near-miss date 2026-07-24: no v-prefix → no pin extracted"
    echo ""
    echo "Excluded directories (not L1/L7 checked):"
    if [ "${#SKIPPED_ARTIFACT_DIRS_NOTICE[@]}" -eq 0 ]; then
        echo "  (none — all known versioned artifact directories are covered)"
    else
        for notice in "${SKIPPED_ARTIFACT_DIRS_NOTICE[@]}"; do
            echo "  SKIPPED: ${notice}"
        done
    fi
    exit 0
}

# ── Main ──────────────────────────────────────────────────────────────────────
if [ "${SELF_PROBE}" -eq 1 ]; then
    run_self_probe
fi

checks_failed=0

if [ "${L9_ONLY}" -eq 0 ]; then
    if [ "${FULL_SCAN}" -eq 1 ]; then
        # --full-scan: check every versioned artifact file (periodic audit mode).
        # Print excluded dirs first so the audit report shows full coverage boundaries.
        echo "records-lint --full-scan: L1+L7 on all versioned artifact directories."
        echo "Covered:"
        for dir in "${VERSIONED_ARTIFACT_DIRS[@]}"; do
            echo "  ${dir}"
        done
        echo "Excluded (see CONFIG BLOCK for reasons):"
        if [ "${#SKIPPED_ARTIFACT_DIRS_NOTICE[@]}" -eq 0 ]; then
            echo "  (none)"
        else
            for notice in "${SKIPPED_ARTIFACT_DIRS_NOTICE[@]}"; do
                echo "  SKIPPED: ${notice}"
            done
        fi
        echo ""
        for dir in "${VERSIONED_ARTIFACT_DIRS[@]}"; do
            abs_dir="${WORKSPACE_ROOT}/${dir}"
            [ -d "${abs_dir}" ] || continue
            while IFS= read -r -d '' file; do
                if ! run_l1 "${file}"; then checks_failed=1; fi
                if ! run_l7 "${file}"; then checks_failed=1; fi
            done < <(find "${abs_dir}" -maxdepth 1 -name "*.md" -print0 2>/dev/null)
        done
    else
        # Default (ratchet mode): check only staged versioned artifact files.
        # Pre-existing violations in unmodified files are grandfathered until
        # a future commit touches them. See "Ratchet scoping" note in header.
        staged_md_files="$(
            git -C "${WORKSPACE_ROOT}" diff --cached --name-only --diff-filter=ACM 2>/dev/null \
                | grep -E '\.md$' \
                || true
        )"
        for rel_path in ${staged_md_files}; do
            abs_file="${WORKSPACE_ROOT}/${rel_path}"
            [ -f "${abs_file}" ] || continue
            # Only check files that fall under a VERSIONED_ARTIFACT_DIR
            in_scope=0
            for dir in "${VERSIONED_ARTIFACT_DIRS[@]}"; do
                if [[ "${rel_path}" == "${dir}/"* ]]; then
                    in_scope=1; break
                fi
            done
            [ "${in_scope}" -eq 1 ] || continue
            if ! run_l1 "${abs_file}"; then checks_failed=1; fi
            if ! run_l7 "${abs_file}"; then checks_failed=1; fi
        done
    fi
fi

if [ "${L1_L7_ONLY}" -eq 0 ]; then
    if ! run_l9; then checks_failed=1; fi
    # L10: run corpus-wide (not staged-only) — cross-document consistency checks
    # are always full-corpus because the index files themselves may not be staged.
    # Skip when --l9-only is set (l9-only means only run L9).
    if [ "${L9_ONLY}" -eq 0 ]; then
        if ! run_l10; then checks_failed=1; fi
    fi
fi

if [ "${checks_failed}" -ne 0 ]; then
    echo ""
    echo "records-lint: FAIL — one or more violations found (see above)."
    exit 1
fi

echo "records-lint: PASS [L1+L7 covered: behavioral-contracts, architecture/decisions, architecture (flat), verification-properties, prd-supplements, stories | L10 covered: BC-INDEX, ARCH-INDEX, VP-INDEX | excluded: none]"
exit 0
