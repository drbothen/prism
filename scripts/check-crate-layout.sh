#!/usr/bin/env bash
# check-crate-layout.sh — Workspace crate layout validator (ADR-012 §2.4)
#
# Validates that every crate under crates/ conforms to the canonical layout:
#   Rule 1: src/lib.rs OR src/main.rs must exist
#   Rule 2: No .rs files at crate root except build.rs
#   Rule 3: No fixture files under tests/fixtures/ (use fixtures/ instead)
#   Rule 4: Inline #[cfg(test)] in src/ is permitted — not checked
#   Rule 5: src/tests/ submodule is permitted — not checked
#
# Exceptions:
#   - tests/ directory is optional (prism-ocsf has none — conformant)
#   - build.rs at crate root is explicitly excluded from Rule 2
#
# Exit codes:
#   0  — all crates conform; no violation output
#   1  — one or more violations; per-crate lines printed to stdout:
#         "crates/<name>: <rule description>"
#
# Usage:
#   scripts/check-crate-layout.sh            # plain output
#   scripts/check-crate-layout.sh --markdown # markdown table (for just layout-report)
#
# Environment:
#   WORKSPACE_ROOT — override workspace root (default: directory containing this script's parent)
#
# Traces to: BC-3.7.001 (VP-134, VP-135, VP-136), ADR-012 §2.4, S-3.5.01

set -uo pipefail

# ---------------------------------------------------------------------------
# Resolve workspace root
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_ROOT="${WORKSPACE_ROOT:-"$(cd "${SCRIPT_DIR}/.." && pwd)"}"
CRATES_DIR="${WORKSPACE_ROOT}/crates"

MARKDOWN=0
if [ "${1:-}" = "--markdown" ]; then
    MARKDOWN=1
fi

# ---------------------------------------------------------------------------
# Scan crates and collect violations
# ---------------------------------------------------------------------------
violations=()
checked=0

if [ ! -d "${CRATES_DIR}" ]; then
    echo "check-crate-layout.sh: crates/ directory not found at ${CRATES_DIR}" >&2
    exit 1
fi

for crate_dir in "${CRATES_DIR}"/*/; do
    [ -d "${crate_dir}" ] || continue
    # Only directories with Cargo.toml are crates
    [ -f "${crate_dir}Cargo.toml" ] || continue

    crate_name="$(basename "${crate_dir}")"
    crate_path="crates/${crate_name}"
    checked=$((checked + 1))

    crate_violations=()

    # ── Rule 1: src/lib.rs OR src/main.rs must exist ─────────────────────────
    if [ ! -f "${crate_dir}src/lib.rs" ] && [ ! -f "${crate_dir}src/main.rs" ]; then
        crate_violations+=("${crate_path}: missing src/lib.rs or src/main.rs (Rule 1)")
    fi

    # ── Rule 2: No .rs files at crate root except build.rs ──────────────────
    while IFS= read -r -d '' rs_file; do
        basename_rs="$(basename "${rs_file}")"
        if [ "${basename_rs}" != "build.rs" ]; then
            crate_violations+=("${crate_path}: loose .rs file at crate root: ${basename_rs} (Rule 2 — move to src/)")
        fi
    done < <(find "${crate_dir}" -maxdepth 1 -name "*.rs" -print0 2>/dev/null)

    # ── Rule 3: No fixture files under tests/fixtures/ ───────────────────────
    if [ -d "${crate_dir}tests/fixtures" ]; then
        crate_violations+=("${crate_path}: fixtures should be in fixtures/, not tests/fixtures/ (Rule 3)")
    fi

    violations+=("${crate_violations[@]+"${crate_violations[@]}"}")
done

# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------
if [ "${MARKDOWN}" -eq 1 ]; then
    echo "| Crate | src/lib.rs | No loose .rs | fixtures/ clean | Status |"
    echo "|-------|-----------|--------------|-----------------|--------|"
    for crate_dir in "${CRATES_DIR}"/*/; do
        [ -d "${crate_dir}" ] || continue
        [ -f "${crate_dir}Cargo.toml" ] || continue
        crate_name="$(basename "${crate_dir}")"
        crate_path="crates/${crate_name}"

        rule1="OK"
        rule2="OK"
        rule3="OK"
        status="PASS"

        if [ ! -f "${crate_dir}src/lib.rs" ] && [ ! -f "${crate_dir}src/main.rs" ]; then
            rule1="FAIL"
            status="FAIL"
        fi

        # Check for loose .rs (excluding build.rs)
        loose_count=0
        while IFS= read -r -d '' rs_file; do
            bn="$(basename "${rs_file}")"
            if [ "${bn}" != "build.rs" ]; then
                loose_count=$((loose_count + 1))
            fi
        done < <(find "${crate_dir}" -maxdepth 1 -name "*.rs" -print0 2>/dev/null)
        if [ "${loose_count}" -gt 0 ]; then
            rule2="FAIL"
            status="FAIL"
        fi

        if [ -d "${crate_dir}tests/fixtures" ]; then
            rule3="FAIL"
            status="FAIL"
        fi

        echo "| ${crate_name} | ${rule1} | ${rule2} | ${rule3} | ${status} |"
    done
    exit 0
fi

# Plain output
if [ "${#violations[@]}" -gt 0 ]; then
    for v in "${violations[@]}"; do
        echo "${v}"
    done
    echo ""
    echo "check-crate-layout: ${#violations[@]} violation(s) in ${checked} crate(s) checked." >&2
    exit 1
fi

# All conformant — silent success (exit 0)
exit 0
