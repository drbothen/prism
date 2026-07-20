#!/usr/bin/env bash
# S-REL-001 AC-009: build-release builds prism-bin AND prism-dtu-demo-server together.
#
# AC: '-p prism-bin -p prism-dtu-demo-server' appears in one cargo command.
# prism-dtu-demo-server binary is wrapped before upload-artifact using per-OS
# conditional logic: .tar.gz (tar, preserves +x) on Unix; .zip (7z, .exe) on Windows.
# Uploaded as artifact 'prism-dtu-demo-server-${{ matrix.target }}'. Strip on Unix.
#
# Red Gate: All assertions FAIL on the unimplemented release.yml because:
#   - cargo build has no -p flags (builds whole workspace default)
#   - prism-dtu-demo-server does not appear anywhere in the file
#   - No wrap/archive step for demo-server exists
#   - No strip step for prism-dtu-demo-server exists
#
# Traces to: architect U13 adjudication; ADJ-003 per-OS wrap + strip
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-009"

# 1. prism-dtu-demo-server must be referenced in the build matrix.
# At Red: completely absent → FAILS.
assert_contains "$REL_YML" "prism-dtu-demo-server" "AC-009"

# 2. SID-2: full composed cargo invocation with both -p flags in one command.
# The COMPLETE string '-p prism-bin -p prism-dtu-demo-server' must appear together
# (not split across two separate cargo invocations).
# At Red: current cargo build has no -p flags at all → FAILS.
assert_contains "$REL_YML" "-p prism-bin -p prism-dtu-demo-server" "AC-009"

# 3. Demo-server archive/wrap step: per-OS conditional using matrix.archive_ext.
# After implementation: 'prism-dtu-demo-server-${{ matrix.target }}' in archive path.
# At Red: absent → FAILS.
assert_contains "$REL_YML" "prism-dtu-demo-server-\${{ matrix.target }}" "AC-009"

# 4. Demo-server artifact upload name must match 'prism-dtu-demo-server-${{ matrix.target }}'.
# At Red: absent → FAILS.
if grep -qF 'name: prism-dtu-demo-server-${{ matrix.target }}' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-009: demo-server artifact named 'prism-dtu-demo-server-\${{ matrix.target }}'"
else
  tap_fail "AC-009: demo-server artifact name missing" \
    "AC-009 FAIL: expected 'name: prism-dtu-demo-server-\${{ matrix.target }}' upload-artifact step"
fi

# 5. Strip step for prism-dtu-demo-server on Unix legs.
# After implementation: 'strip target/${{ matrix.target }}/release/prism-dtu-demo-server'
# At Red: absent → FAILS.
assert_contains "$REL_YML" "strip target/\${{ matrix.target }}/release/prism-dtu-demo-server" "AC-009"

# 6. Per-OS archive_ext conditional must wrap demo-server (zip on Windows, tar.gz on Unix).
# After implementation: archive_ext conditional in the Wrap step.
# At Red: absent → FAILS.
if grep -qF 'prism-dtu-demo-server-${{ matrix.target }}.zip' "$REL_YML" 2>/dev/null || \
   grep -qF 'prism-dtu-demo-server-${{ matrix.target }}.tar.gz' "$REL_YML" 2>/dev/null || \
   grep -qF 'prism-dtu-demo-server-${{ matrix.target }}.${{ matrix.archive_ext }}' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-009: demo-server archive path uses per-OS extension (archive_ext conditional)"
else
  tap_fail "AC-009: demo-server archive per-OS extension handling absent" \
    "AC-009 FAIL: expected demo-server archive with .zip/.tar.gz or archive_ext variable"
fi

tap_done
