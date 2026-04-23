#!/usr/bin/env bash
# Helper: run a pre-built plugin_tests binary test by name.
# Usage: run-test.sh <test_name>
BINARY="/Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime/target/debug/deps/plugin_tests-4f46a65c03c2d0f0"
exec "$BINARY" "$1" --nocapture 2>&1
