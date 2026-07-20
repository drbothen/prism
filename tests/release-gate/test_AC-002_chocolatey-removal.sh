#!/usr/bin/env bash
# S-REL-001 AC-002: DEF-REL-002 closed — chocolatey-publish job removed.
#
# AC: "grep -n 'chocolatey|choco|nuspec' release.yml" returns zero functional
# matches. Comment-only references are acceptable.
#
# Red Gate: All assertions below FAIL on the unimplemented release.yml because:
#   - 'chocolatey-publish:' is a live job definition (line 167)
#   - 'choco pack' and 'choco push' are live run: steps (lines 185-186)
#   - 'nuspec' is in the choco pack invocation (line 185)
#
# Traces to: delta-analysis.md §3 DEF-REL-002
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-002"

# 1. chocolatey-publish: job definition must not appear in functional lines.
# After implementation: removed (comment acceptable).
# At Red: 'chocolatey-publish:' is a live YAML job key.
assert_not_in_functional_lines "$REL_YML" "chocolatey-publish:" "AC-002"

# 2. choco pack must not appear in functional lines.
# At Red: 'choco pack packaging/chocolatey/prism.nuspec' in run: block.
assert_not_in_functional_lines "$REL_YML" "choco pack" "AC-002"

# 3. choco push must not appear in functional lines.
# At Red: 'choco push --source ...' in run: block.
assert_not_in_functional_lines "$REL_YML" "choco push" "AC-002"

# 4. nuspec must not appear in functional lines.
# At Red: 'packaging/chocolatey/prism.nuspec' in run: block.
assert_not_in_functional_lines "$REL_YML" "nuspec" "AC-002"

tap_done
