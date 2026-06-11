#!/usr/bin/env bash
# validate-error-taxonomy-retirement-annotations.sh
#
# SOURCE: Architect adjudication FB-PR-1-error-taxonomy-test-relocation.md
#         (Option 1 — spec-governance annotation invariant relocated from Rust test
#          to this hook per Layer 2 enforcement model)
#
# PURPOSE: Verifies that all error codes declared RETIRED in error-taxonomy.md carry
#          the required back-pointer fields: story ID, ADR reference.
#          This is the spec-governance enforcement layer (Layer 2).
#          The code-side construction-site gate is in the Rust test
#          test_BC_2_16_011_e_spec_008_retired_annotation (Layer 1).
#
# INVARIANTS ENFORCED:
#   E-SPEC-008 row must contain:
#     - "RETIRED in S-PLUGIN-PREREQ-E"  (BC-2.16.011 AC-11; ADR-027 §Decision)
#     - "ADR-027"                        (BC-2.16.011 AC-11 back-pointer)
#
# EXTENSIBILITY: When future stories retire additional error codes, add a new
#   check_retirement_annotation call following the E-SPEC-008 pattern below.
#   Each call documents the error code, the story that retired it, and the ADR
#   that mandates the operational deletion. The function accumulates invariants
#   over time — this script is NOT story-scoped.
#
# EXIT CODES:
#   0 — PASS (all required annotations present)
#   1 — FAIL (any annotation missing; error message names the missing annotation)
#
# USAGE:
#   bash .factory/hooks/validate-error-taxonomy-retirement-annotations.sh
#
# TRIGGER: Called from the .factory/ pre-commit chain when
#   specs/prd-supplements/error-taxonomy.md is staged, AND as a standalone
#   invocable gate for wave-0-plugin-prereqs wave-gate verification.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FACTORY_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
TAXONOMY="${FACTORY_DIR}/specs/prd-supplements/error-taxonomy.md"

if [ ! -f "${TAXONOMY}" ]; then
    echo "FAIL: error-taxonomy.md not found at ${TAXONOMY}"
    exit 1
fi

FAIL=0

# check_retirement_annotation ERROR_CODE STORY_ID ADR_ID
# Greps error-taxonomy.md for the two required annotation strings on a retired row.
# Args:
#   $1 — error code (e.g. "E-SPEC-008") — used in error messages only
#   $2 — story annotation string (e.g. "RETIRED in S-PLUGIN-PREREQ-E")
#   $3 — ADR back-pointer string (e.g. "ADR-027")
check_retirement_annotation() {
    local error_code="${1}"
    local story_annotation="${2}"
    local adr_annotation="${3}"

    if ! grep -q "${story_annotation}" "${TAXONOMY}"; then
        echo "FAIL: BC-2.16.011 AC-11 — error-taxonomy.md missing '${story_annotation}' in ${error_code} retirement row"
        FAIL=1
    fi

    if ! grep -q "${adr_annotation}" "${TAXONOMY}"; then
        echo "FAIL: BC-2.16.011 AC-11 — error-taxonomy.md missing '${adr_annotation}' back-pointer in ${error_code} retirement annotation"
        FAIL=1
    fi
}

# E-SPEC-008: retired in S-PLUGIN-PREREQ-E per BC-2.16.011 AC-11 + ADR-027
check_retirement_annotation "E-SPEC-008" "RETIRED in S-PLUGIN-PREREQ-E" "ADR-027"

# check_row_retirement ERROR_CODE ANNOTATION_REGEX
# Row-anchored variant for batch retirements: verifies the annotation appears on the
# SAME taxonomy table row as the error code (taxonomy rows are single markdown lines,
# so a line-anchored BRE pattern gives per-row enforcement even when one ADR retires
# many codes and the ADR string alone would not discriminate between rows).
# Args:
#   $1 — error code (must be the first cell of its table row, e.g. "E-CFG-017")
#   $2 — annotation BRE fragment that must appear later on the same line
check_row_retirement() {
    local error_code="${1}"
    local annotation="${2}"

    if ! grep -q "^| ${error_code} | .*${annotation}" "${TAXONOMY}"; then
        echo "FAIL: ADR-037 — error-taxonomy.md row ${error_code} missing '${annotation}' annotation on its table row"
        FAIL=1
    fi
}

# E-CFG customer-config namespace: 23 codes retired per ADR-037 (2026-06-10,
# prism-customer-config crate retirement; supersedes ADR-010). Rule tables preserved
# in retired BC-3.3.001..004 bodies; taxonomy v1.65. E-CFG-100..103 remain ACTIVE.
for code in E-CFG-000 E-CFG-001 E-CFG-002 E-CFG-003 E-CFG-004 E-CFG-005 E-CFG-006 \
            E-CFG-007 E-CFG-008 E-CFG-009 E-CFG-010 E-CFG-011 E-CFG-012 E-CFG-013 \
            E-CFG-014 E-CFG-015 E-CFG-016 E-CFG-017 E-CFG-018 E-CFG-019 E-CFG-020 \
            E-CFG-030 E-CFG-031; do
    check_row_retirement "${code}" "RETIRED per ADR-037 (2026-06-10)"
done

# Future retired codes: add new check_retirement_annotation calls here, e.g.:
# check_retirement_annotation "E-SPEC-NNN" "RETIRED in S-STORY-ID" "ADR-NNN"
# Batch retirements (one ADR, many codes): use check_row_retirement per code, as in
# the ADR-037 E-CFG block above.

if [ "${FAIL}" -eq 0 ]; then
    echo "PASS: error-taxonomy.md retirement annotations verified"
fi

exit "${FAIL}"
