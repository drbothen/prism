#!/usr/bin/env bash
# S-REL-001 AC-004: DEF-REL-004 closed — crates-io-publish job removed.
#
# AC: "grep -n 'crates.io|crates-io|cargo publish' release.yml" returns zero
# functional matches. Comment-only references are acceptable.
#
# Red Gate: All assertions below FAIL on the unimplemented release.yml because:
#   - 'crates-io-publish:' is a live job definition (line 188)
#   - 'cargo publish -p prism-spec-engine --no-verify' is a live run: step (line 205)
#   - 'CARGO_REGISTRY_TOKEN' is scoped to the live publish step (line 202)
#
# Traces to: delta-analysis.md §3 DEF-REL-004
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-004"

# 1. crates-io-publish: job definition must not appear in functional lines.
# After implementation: removed (comment OK).
# At Red: 'crates-io-publish:' is a live YAML job key (line 188).
assert_not_in_functional_lines "$REL_YML" "crates-io-publish:" "AC-004"

# 2. cargo publish must not appear in functional lines.
# All workspace crates carry publish = false. cargo publish is incorrect here.
# After implementation: removed.
# At Red: 'cargo publish -p prism-spec-engine --no-verify' in run: block (line 205).
assert_not_in_functional_lines "$REL_YML" "cargo publish" "AC-004"

# 3. CARGO_REGISTRY_TOKEN must not appear in functional lines.
# Token was used only in the crates-io-publish job. After removal, no functional reference.
# Comment-only references are acceptable (e.g., in a deferral note).
# At Red: 'CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}' in env block (line 202).
assert_not_in_functional_lines "$REL_YML" "CARGO_REGISTRY_TOKEN" "AC-004"

tap_done
