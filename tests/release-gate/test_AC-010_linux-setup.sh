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
# §15 ratified: cargo-zigbuild closes DEFECT-REL001-MUSL-LIBSTDCXX-001 by replacing
# the CXX_x86_64_unknown_linux_musl=clang++ env export with Zig's own musl-aware
# C++ toolchain. clang is still installed for build.rs cc-rs usage on the glibc host.
# Assertions 9-16: assert §15 design (pip/zigbuild) and guard against CXX reintroduction.
#
# 9. 'libdbus-1-dev clang' — composed end of the apt-get install command.
#    Asserting both packages together distinguishes the functional install line
#    from comment-only mentions of clang (which lack the libdbus-1-dev prefix).
assert_contains "$REL_YML" \
  "libdbus-1-dev clang" \
  "AC-010"

# §15 cargo-zigbuild assertions (DEFECT-REL001-MUSL-LIBSTDCXX-001, Delta 15-1/15-2/15-3):

# 10. pip3 install with --require-hashes and the requirements file path (Delta 15-2, §15).
#     SID-2 composed: hash-pinning flag + exact requirements file path co-appear on one line.
assert_contains "$REL_YML" \
  "pip3 install --require-hashes -r .github/workflows/requirements-musl-ci.txt" \
  "AC-010"

# 11. cargo-zigbuild installed at the exact pinned version with --locked (Delta 15-1, §15).
#     SID-2 composed: --locked + exact version string must co-appear.
assert_contains "$REL_YML" \
  "cargo install --locked cargo-zigbuild --version 0.23.0" \
  "AC-010"

# 12. musl build uses cargo zigbuild with --locked AND both -p package specs (SID-2, §15).
#     Composed: zigbuild + locked + both package names in one shell line.
#     Single quotes prevent bash from expanding ${{ matrix.target }} — passed as literal.
MUSL_BUILD_CMD='cargo zigbuild --release --locked --target ${{ matrix.target }} -p prism-bin -p prism-dtu-demo-server'
assert_contains "$REL_YML" "$MUSL_BUILD_CMD" "AC-010"

# 13. requirements-musl-ci.txt file must exist adjacent to release.yml (Delta 15-2, §15).
REQUIREMENTS_TXT="${WORKTREE}/.github/workflows/requirements-musl-ci.txt"
assert_file_exists "$REQUIREMENTS_TXT" "AC-010"

# 14. Requirements file pins ziglang at the exact version (not a range or inequality).
assert_contains "$REQUIREMENTS_TXT" \
  "ziglang==0.16.0" \
  "AC-010"

# 15. Requirements file includes the sha256 hash (CWE-494 discipline, Delta 15-2, §15).
#     SID-2 composed: the full --hash=sha256:<digest> string must be present.
assert_contains "$REQUIREMENTS_TXT" \
  "--hash=sha256:9fcda73f62b851dd72a54b710ad40a209896db14cfb13649e62191243556342b" \
  "AC-010"

# 16. CXX_x86_64_unknown_linux_musl=clang++ MUST BE ABSENT from release.yml.
#     Negative regression guard: the superseded ABI-unsound export (clang++ linked against
#     glibc's libstdc++.a produces 117+ undefined-ref errors against musl libc) must not
#     be reintroduced. Full composed form guards both the key and the value.
assert_not_contains "$REL_YML" \
  "CXX_x86_64_unknown_linux_musl=clang++" \
  "AC-010"

tap_done
