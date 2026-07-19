#!/usr/bin/env bash
# S-REL-001 AC-010: Linux setup step installs musl-tools, pkg-config, and libdbus-1-dev.
#
# AC: libdbus-1-dev is installed unconditionally alongside musl-tools and pkg-config on
# both Linux legs, with a comment citing ADR-034/BC-2.06.003 and the build.rs host-linkage
# rationale. The step is gated on contains(matrix.target, 'linux').
#
# Red Gate: All assertions FAIL on the unimplemented release.yml because:
#   - There is NO apt-get step at all in the current build-release job
#   - libdbus-1-dev, musl-tools, pkg-config are all absent
#   - ADR-034/BC-2.06.003 comment is absent
#   - The Linux target gate condition is absent
#
# Background: prism-credentials default features enable keyring-linux-native-sync-persistent
# which links dbus-secret-service (C-linked) via libdbus-sys. build.rs runs on the glibc host
# even for musl cross-targets. libdbus-1-dev MUST be installed on both Linux legs.
# Source: ADR-034/BC-2.06.003; research U2; story ADJ-001.
#
# Traces to: architect ADJ-001; ADR-034/BC-2.06.003; research U2
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-010"

# 1. libdbus-1-dev must be installed in the Linux setup step.
# At Red: no apt-get install in release.yml at all → FAILS.
assert_contains "$REL_YML" "libdbus-1-dev" "AC-010"

# 2. musl-tools must be in the apt-get install command.
# At Red: absent → FAILS.
assert_contains "$REL_YML" "musl-tools" "AC-010"

# 3. pkg-config must be in the apt-get install command.
# At Red: absent → FAILS.
assert_contains "$REL_YML" "pkg-config" "AC-010"

# 4. SID-2: full composed citation 'ADR-034/BC-2.06.003' must appear as a comment.
# This is the canonical reference for why libdbus-1-dev is required on Linux runners.
# At Red: absent → FAILS.
assert_contains "$REL_YML" "ADR-034/BC-2.06.003" "AC-010"

# 5. Linux gate condition: step must be conditional on Linux targets only.
# After implementation: 'contains(matrix.target, '\''linux'\'')'
# At Red: absent → FAILS.
if grep -qF "contains(matrix.target, 'linux')" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-010: Linux apt step gated on contains(matrix.target, 'linux')"
else
  tap_fail "AC-010: Linux apt step gate condition absent" \
    "AC-010 FAIL: expected \"if: contains(matrix.target, 'linux')\" on the apt-get install step"
fi

tap_done
