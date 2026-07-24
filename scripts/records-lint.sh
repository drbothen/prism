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
    # TODO-parameterize: ".factory/specs/architecture" — add when ARCH-INDEX.md
    #   and section files adopt the standard 5-col Changelog table format.
    #   Currently those files use inline prose rows, not the table convention.
    # TODO-parameterize: ".factory/stories" — add when stories adopt frontmatter
    #   version: + standard Changelog table (currently use version: but no table).
)

# L9 scope: staged additions under these paths trigger the line-cite ban.
# Covers all factory artifacts: BCs, ADRs, VPs, STATE.md, burst logs, etc.
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

# ── L9 — New-text line-cite ban ───────────────────────────────────────────────
# Staged additions to .factory/ files must not contain file:NNN line-cite patterns.
# Pattern targets: word.extension:digits — e.g. pipeline.rs:142, bc-2.01.001.md:50
# Excludes URL ports by requiring a specific file extension (not arbitrary hostname).
# Uses a git dir parameter so the self-probe can override WORKSPACE_ROOT.
L9_CITE_PATTERN='\b[A-Za-z0-9_][A-Za-z0-9_.-]*\.(rs|md|toml|yaml|yml|py|sh|txt|json|ts|tsx|js|jsx):[0-9]+'

run_l9() {
    local git_root="${1:-${WORKSPACE_ROOT}}"
    shift || true
    local record_dirs=("${@:-${RECORD_DIRS_L9[@]}}")

    # Nothing staged in the record dirs → nothing to check
    if git -C "${git_root}" diff --cached --quiet -- "${record_dirs[@]}" 2>/dev/null; then
        return 0
    fi

    local violations=() file_ctx=""
    local diff_output
    diff_output="$(
        git -C "${git_root}" diff --cached --unified=0 -- "${record_dirs[@]}" 2>/dev/null
    )"

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
    done <<< "${diff_output}"

    if [ "${#violations[@]}" -gt 0 ]; then
        for v in "${violations[@]}"; do
            echo "${v}"
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

    # ── L9 violation: staged addition with file:NNN line-cite ────────────────
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
    # Stage file containing a line-cite violation
    printf '# Adversary pass\nThe issue is at pipeline.rs:142 under load.\n' \
        > "${probe_factory}/violation.md"
    git -C "${probe_repo}" add fdir/violation.md 2>/dev/null

    if run_l9 "${probe_repo}" "fdir" >/dev/null 2>&1; then
        probe_failures+=("L9: MISSED file:NNN line-cite in staged addition (pipeline.rs:142) — false-green")
    else
        echo "L9 probe PASS: line-cite pattern correctly flagged"
        pass_count=$((pass_count+1))
    fi

    # ── L9 clean: staged addition with symbol/anchor cite only ───────────────
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
        echo "L9 probe PASS: symbol-anchor cite correctly cleared"
        pass_count=$((pass_count+1))
    else
        probe_failures+=("L9: INCORRECTLY flagged symbol-anchor cite (no file:NNN) — false-red")
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
    echo "TODO-parameterize — edge cases not yet self-probed:"
    echo "  L1: files with no frontmatter version (should skip)"
    echo "  L1: files with no changelog table (should skip)"
    echo "  L7: single-row changelog (should pass — no ordering comparison possible)"
    echo "  L9: URL port http://host:8080 should NOT trigger — validate manually"
    echo "  L9: unchanged line with file:NNN should NOT trigger — validate manually"
    exit 0
}

# ── Main ──────────────────────────────────────────────────────────────────────
if [ "${SELF_PROBE}" -eq 1 ]; then
    run_self_probe
fi

checks_failed=0

if [ "${L9_ONLY}" -eq 0 ]; then
    if [ "${FULL_SCAN}" -eq 1 ]; then
        # --full-scan: check every versioned artifact file (periodic audit mode)
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
fi

if [ "${checks_failed}" -ne 0 ]; then
    echo ""
    echo "records-lint: FAIL — one or more violations found (see above)."
    exit 1
fi

echo "records-lint: PASS"
exit 0
