#!/usr/bin/env bash
# S-REL-001 AC-001: DEF-REL-001 closed — binary_exists guard and check_binary step removed.
#
# AC: "grep -n 'binary_exists|check_binary' .github/workflows/release.yml" returns zero
# matches. The build-release job has no outputs: block. No job has an if: condition
# referencing binary_exists.
#
# Red Gate: All assertions below FAIL on the unimplemented release.yml because:
#   - 'binary_exists' appears in build-release outputs: block and four if: conditions
#   - 'check_binary' appears as a step id
#   - 'steps.check_binary.outputs.binary_exists' appears in if: conditions
#
# Traces to: delta-analysis.md §3 DEF-REL-001
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-001"

# 1. binary_exists must be completely absent (covers outputs: block + if: conditions).
# At Red: present at job-level outputs block and multiple if: conditions.
assert_not_contains "$REL_YML" "binary_exists" "AC-001"

# 2. check_binary step id must be absent.
# At Red: present as 'id: check_binary' in the guard step.
assert_not_contains "$REL_YML" "check_binary" "AC-001"

# 3. Full composed assertion (SID-2): the exact if: condition string.
# At Red: 'steps.check_binary.outputs.binary_exists == '\''true'\''' present in multiple jobs.
assert_not_contains "$REL_YML" "steps.check_binary.outputs.binary_exists == 'true'" "AC-001"

# 4. Confirm no job needs build-release via the binary_exists output gate.
# After removal: jobs may still need build-release but not via the binary_exists output.
assert_not_contains "$REL_YML" "needs.build-release.outputs.binary_exists" "AC-001"

tap_done
