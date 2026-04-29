#!/usr/bin/env bash
# check-crate-layout.sh — Workspace crate layout validator (ADR-012 §2.4)
#
# Validates that every crate under crates/ conforms to the canonical layout:
#   - src/lib.rs or src/main.rs must exist (no loose .rs at crate root except build.rs)
#   - fixture data lives in fixtures/ not tests/fixtures/
#   - tests/ directory is optional (prism-ocsf has none — conformant)
#   - build.rs at crate root is explicitly permitted
#
# Exit codes:
#   0  — all crates conform; no violation output (or optional summary line)
#   1  — one or more crates violate layout rules; per-crate lines printed:
#         "crates/<name>: <rule description>"
#
# Usage:
#   scripts/check-crate-layout.sh            # plain output
#   scripts/check-crate-layout.sh --markdown # markdown table (for just layout-report)
#
# Rules enforced (BC-3.7.001 / ADR-012 §2.1):
#   Rule 1: src/lib.rs OR src/main.rs must exist
#   Rule 2: No .rs files at crate root except build.rs
#   Rule 3: No fixture files under tests/fixtures/ (use fixtures/ instead)
#
# Exceptions:
#   - prism-ocsf: tests/ directory is optional (conformant without it)
#   - build.rs at crate root is explicitly excluded from Rule 2
#
# TODO(S-3.5.01 implementer): Replace this stub body with the real validation logic.
#   See BC-3.7.001 postconditions 1-7 and ADR-012 §2.4 for full requirements.
#   Test vectors are defined in tests/check_crate_layout_test.rs (shell tests TBD).

# RED GATE STUB — always fails until implementation phase.
# This ensures all tests fail before any implementation begins.
echo "check-crate-layout.sh: NOT YET IMPLEMENTED (Red Gate stub)" >&2
exit 1
