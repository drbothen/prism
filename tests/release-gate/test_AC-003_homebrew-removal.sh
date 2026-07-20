#!/usr/bin/env bash
# S-REL-001 AC-003: DEF-REL-003 closed — homebrew-update job removed; S-REL-008 comment present.
#
# AC: "grep -n 'homebrew' release.yml" returns zero functional matches. Comment
# documenting the deferral is acceptable. Comment must reference 'S-REL-008'.
#
# Red Gate:
#   - 'homebrew-update:' as functional job def → FAIL (present)
#   - '1898co/homebrew-tap' as functional checkout → FAIL (present)
#   - 'S-REL-008' as comment reference → FAIL (absent from current file)
#
# Traces to: delta-analysis.md §3 DEF-REL-003
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-003"

# 1. homebrew-update: job definition must not appear in functional lines.
# After implementation: removed (comment OK).
# At Red: 'homebrew-update:' is a live YAML job key (line 125).
assert_not_in_functional_lines "$REL_YML" "homebrew-update:" "AC-003"

# 2. 1898co/homebrew-tap must not appear in functional lines.
# The tap org does not exist; functional checkout would fail at runtime.
# After implementation: removed.
# At Red: present in repository: checkout step (line 133).
assert_not_in_functional_lines "$REL_YML" "1898co/homebrew-tap" "AC-003"

# 3. Formula/prism.rb sed update must not appear in functional lines.
# At Red: functional sed -i lines update Formula/prism.rb (lines 148-149).
assert_not_in_functional_lines "$REL_YML" "Formula/prism.rb" "AC-003"

# 4. Deferral comment must reference S-REL-008 (the story that will re-enable homebrew).
# SID-2: assert the full composed reference string 'S-REL-008', not just 'REL-008'.
# After implementation: comment like "# homebrew-update removed (DEF-REL-003)... S-REL-008."
# At Red: absent from file entirely → FAILS.
assert_contains "$REL_YML" "S-REL-008" "AC-003"

tap_done
