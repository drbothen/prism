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
# C++ toolchain. §15/F-REL001-P16-001 further removes clang from the apt-get install
# line (gnu dry-run passed pre-clang; cc-rs uses gcc/g++; musl uses zig's bundled toolchain).
# Assertions 9a, 9b, 10-16: assert §15 design (pip/zigbuild) and guard against regression.
#
# 9a. SID-2 positive composed assertion: full apt-get install command as shipped after
#     §15/F-REL001-P16-001 (clang removed). Asserting the complete composed string ensures
#     all three packages co-appear on the functional install line — not merely individually
#     somewhere in the file.
assert_contains "$REL_YML" \
  "sudo apt-get install -y musl-tools pkg-config libdbus-1-dev" \
  "AC-010"

# 9b. Negative regression guard (§15/F-REL001-P16-001): clang must NOT appear on the
#     functional apt-get install line. The removal rationale comment directly above the
#     install command and the DEFECT-REL001-MUSL-LIBSTDCXX-001 comment inside the musl
#     block both contain the word "clang"; a whole-file grep would match those comment
#     lines and false-pass when clang is re-added to the install command.
#     Scope: strip YAML comment lines ('^[[:space:]]*#'), then filter to apt-get install
#     lines, then assert ' clang' is absent from that functional-line-only set.
apt_install_line=$(grep -v '^[[:space:]]*#' "$REL_YML" 2>/dev/null | grep 'apt-get install' || true)
if echo "$apt_install_line" | grep -qF ' clang' 2>/dev/null; then
  tap_fail "AC-010: clang found on apt-get install line (must not be reintroduced per §15/F-REL001-P16-001)" \
    "AC-010 FAIL: ' clang' found in functional apt-get install line — §15 requires clang absent (musl uses zig toolchain; gnu uses cc-rs/gcc)"
else
  tap_pass "AC-010: clang correctly absent from apt-get install line (§15/F-REL001-P16-001 regression guard)"
fi

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

# F-REL001-P14-001 — §14 Option B linux-gnu persistence invariant regression guard.
# Target: crates/prism-credentials/Cargo.toml (Cargo manifest, NOT release.yml).
# The release-gate suite is the story's authoritative regression net per F-REL001-P10-001;
# this file hosts Cargo.toml assertions because no separate Cargo-manifest test file exists.
#
# §14 rationale: linux-gnu requires keyring-linux-native-sync-persistent
# (dbus-secret-service + crypto-rust) for persistent credentials across reboots. This
# MUST be activated via a target-specific
#   [target.'cfg(all(target_os = "linux", not(target_env = "musl")))'.dependencies]
# block — NOT via the default = [...] feature array — because musl cannot link libdbus
# (C-linked via glibc). The compile_error! guard in lib.rs cannot catch accidental block
# removal (the guard only fires when the feature KEY is absent from the crate feature
# registry, not when the dep-activation block is removed while the key declaration
# remains). This automated regression guard closes that gap.
# Source: F-REL001-P14-001 (LOCAL adversary pass-14); ADR-034/BC-2.06.003.
CREDS_CARGO="${WORKTREE}/crates/prism-credentials/Cargo.toml"

# 17. Target-cfg header must be present as a functional (non-comment) line in the
#     Cargo manifest. Filtering comments first prevents the matching comment inside
#     the default = [...] block (which references the header for documentation
#     purposes) from producing a false pass.
#     At regression (block removed): header absent from functional lines → tap_fail.
TARGET_CFG_HDR="[target.'cfg(all(target_os = \"linux\", not(target_env = \"musl\")))'.dependencies]"
if grep -v '^[[:space:]]*#' "$CREDS_CARGO" | grep -qF "$TARGET_CFG_HDR" 2>/dev/null; then
  tap_pass "AC-010: target-cfg(linux-gnu) header present as functional line in Cargo.toml"
else
  tap_fail "AC-010: target-cfg(linux-gnu) header ABSENT from functional lines of Cargo.toml" \
    "AC-010 FAIL: expected [target.'cfg(all(target_os = \"linux\", not(target_env = \"musl\")))'.dependencies] — absent or only in comments"
fi

# 18. SID-2 composed block-scoped assertion: keyring with linux-native-sync-persistent
#     must appear inside the target-cfg(linux-gnu) block.
#     awk anchors capture to actual TOML section headers (/^\[target\./) to avoid false
#     match on the comment line inside default = [...] that references the header string.
#     Then greps for the feature name inside the captured block.
#     At regression (block removed): captured block is empty → grep fails → tap_fail.
target_gnu_block=$(awk '
  /^\[target\./ && /target_os.*linux.*not.*target_env.*musl/ { capture=1 }
  capture && /^\[/ && !/target_os/ { exit }
  capture { print }
' "$CREDS_CARGO")
if echo "$target_gnu_block" | grep -qF "linux-native-sync-persistent" 2>/dev/null; then
  tap_pass "AC-010: keyring linux-native-sync-persistent present in target-cfg(linux-gnu) block (Cargo.toml)"
else
  tap_fail "AC-010: keyring linux-native-sync-persistent ABSENT from target-cfg(linux-gnu) block (Cargo.toml)" \
    "AC-010 FAIL: expected keyring dep with linux-native-sync-persistent in [target.cfg(linux,not(musl)).dependencies] — block absent or feature removed"
fi

# 19. Negative guard: keyring-linux-native-sync-persistent must NOT be an active
#     (non-comment, quoted) entry in the default = [...] feature array.
#     Re-adding "keyring-linux-native-sync-persistent" to default would activate dbus on
#     musl targets, which cannot link libdbus at runtime (ADR-034/BC-2.06.003).
#     Comment references (# keyring-linux-native-sync-persistent ...) are acceptable.
#     awk extracts the default = [...] block; grep -v removes comment lines; the inner grep
#     checks for the quoted feature name form that would only appear as an active entry.
default_feat_block=$(awk '/^default = \[/{capture=1} capture{print} /^\]/{if(capture){capture=0}}' "$CREDS_CARGO")
if echo "$default_feat_block" | grep -v '^[[:space:]]*#' | grep -qF '"keyring-linux-native-sync-persistent"' 2>/dev/null; then
  tap_fail "AC-010: forbidden \"keyring-linux-native-sync-persistent\" as active default feature (Cargo.toml)" \
    "AC-010 FAIL: 'keyring-linux-native-sync-persistent' must not be in default = [...] — reactivation breaks musl builds (§14 Option B regression)"
else
  tap_pass "AC-010: 'keyring-linux-native-sync-persistent' correctly absent from default = [...] active entries (Cargo.toml)"
fi

tap_done
