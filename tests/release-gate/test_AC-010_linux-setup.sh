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

# DEFECT-REL001-PROTOC-MISSING-001 + F-REL001-P10-001:
# protoc toolchain is required by prost-build (prism-ocsf build.rs) on all 5 matrix legs.
# The fix-burst that added the setup-protoc step must have a load-bearing suite assertion
# per the F-REL001-P10-001 codified discipline (any fix-burst adding load-bearing workflow
# logic must add a suite assertion in the same burst).
#
# 6. SID-2 composed assertion: full pinned SHA + version comment must co-appear on the same
# uses: line. Asserting just the action name or just the SHA would leave either the SHA-pin
# or the human-readable version comment unverified.
# At Red: step absent → full string absent → FAILS.
assert_contains "$REL_YML" \
  "arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b # v3.0.0" \
  "AC-010"

# 7. setup-protoc must run on ALL 5 matrix legs — no if: gate allowed.
# Extract the step block from '- name: Install protoc' through the next step boundary
# (a line starting with '      - ') and verify no 'if:' condition appears in the block.
# At Red: step absent → block is empty → first branch fires → FAILS.
protoc_block=$(awk '
  /- name: Install protoc.*prost-build/ { capture=1 }
  capture && /^      - / && !/Install protoc/ { exit }
  capture { print }
' "$REL_YML")
if [ -z "$protoc_block" ]; then
  tap_fail "AC-010: setup-protoc step block not found in release.yml" \
    "AC-010 FAIL: expected '- name: Install protoc (required by prost-build' step — absent entirely"
elif echo "$protoc_block" | grep -qF 'if:' 2>/dev/null; then
  tap_fail "AC-010: setup-protoc step is gated by if: (must run on all 5 matrix legs)" \
    "AC-010 FAIL: setup-protoc step must be unconditional — 'if:' found in step block"
else
  tap_pass "AC-010: setup-protoc step runs unconditionally on all 5 matrix legs (no if: gate)"
fi

# DEFECT-REL001-MUSL-CXX-001 + F-REL001-P10-001 (SID-2 composed assertions):
# The musl C++ toolchain fix requires two load-bearing changes to co-exist:
# (1) clang added to apt-get install, and (2) CXX_x86_64_unknown_linux_musl
# env var exported with value clang++. Assert the composed forms, not fragments.
#
# 9. 'libdbus-1-dev clang' — composed end of the apt-get install command.
#    Asserting both packages together distinguishes the functional install line
#    from comment-only mentions of clang (which lack the libdbus-1-dev prefix).
assert_contains "$REL_YML" \
  "libdbus-1-dev clang" \
  "AC-010"

# 10. Full composed CXX env assignment (SID-2: key=value form — not the variable
#     name alone and not 'clang++' alone; the pairing is what the fix requires).
assert_contains "$REL_YML" \
  "CXX_x86_64_unknown_linux_musl=clang++" \
  "AC-010"

tap_done
