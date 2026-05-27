# S-PLUGIN-CI-001 PR Review Findings

PR: #159 — feature/S-PLUGIN-CI-001 → develop
Branch head: 5dd627a2

---

## Convergence Table

| Cycle | Total Findings | Blocking | Fixed | Remaining |
|-------|----------------|----------|-------|-----------|
| 1 (security+code) | 0 | 0 | 0 | 0 → APPROVE |

---

## Cycle 1 — Security + Code Review

**Date:** 2026-05-27
**Verdict:** APPROVE — zero blocking findings

### Security Review Findings

Scope checked:
- WASM sandbox integrity (BC-2.17.002 / VP-040 / INV-PLUGIN-002)
- Credential handling (AD-017)
- CI supply-chain security (pinned actions, pinned tool version)
- OWASP top 10 applicable surface
- Input injection / command injection

| ID | Finding | Severity | Category | Route | Status |
|----|---------|----------|----------|-------|--------|
| SEC-1 | `curl \| tar` pipe installs wasm-tools 1.248.0 directly to /usr/local/bin with no checksum verification. GitHub Releases HTTPS is trusted but a compromised release or MITM could deliver a malicious binary. | suggestion | ci-supply-chain | N/A — see note | non-blocking |
| SEC-2 | `std::env::set_var` + `remove_var` used in async tests (AC-003, existing AC-006 test). Both are annotated with `// SAFETY:` claiming single-threaded tokio runtime. This is correct for `#[tokio::test]` default (single-thread), but if tests are ever run with multi-thread runtime the unsoundness would be silent. | nit | test-safety | N/A — see note | non-blocking |
| SEC-3 | Committed `.prx` binary (129KB) has no in-repo SHA/hash pin documented alongside it. README describes the rebuild procedure but does not record the expected hash for integrity verification. | nit | supply-chain | N/A — see note | non-blocking |

**Notes:**

SEC-1: The `curl | tar` pattern is standard GitHub Actions practice for binary tool install. Pinning to a specific version (1.248.0) and using `curl -fsSL` (fail on HTTP error) mitigates most risk. The production-grade fix would be adding `sha256sum --check` after download. However: (a) this is a CI-only gate, not a production code path; (b) Wasmtime / bytecodealliance have a strong provenance record; (c) the Justfile `build-plugin-crowdstrike-oauth2` recipe runs `wasm-tools validate` as a post-build gate which would catch a corrupt/wrong binary. This is a suggestion, not a blocker.

SEC-2: All three `set_var` call sites have explicit `// SAFETY:` comments. The existing pre-merge adversary 3-CLEAN protocol accepted this pattern. Non-blocking.

SEC-3: The README.md in `tests/fixtures/` describes the Wasmtime 44.0.1 source and wasm-tools 1.248.0 pin. The `.prx` is rebuilt by CI on every PR. Recording a hash would be an improvement but is not a CLAUDE.md or BC violation. Non-blocking.

**WASM Sandbox Integrity — PASS:**
- `build_component_linker` correctly uses `define_unknown_imports_as_traps` BEFORE registering real host functions, then calls `allow_shadowing(true)` only to overwrite the trap stub, then restores `allow_shadowing(false)`. The sequencing is correct: WASI stubs trap; real host functions are available only under the WIT namespace. BC-2.17.002 / INV-PLUGIN-002 satisfied.
- No `wasmtime_wasi::add_to_linker_*` call found in the diff. Confirmed.

**Credential Handling — PASS:**
- No real credentials in diff. Test env vars (`test-client-id-ac003`, `test-client-secret-ac003`) are synthetic sentinel values, not real credentials.
- No credential values in CI workflow. AD-017 satisfied.

**CI Action Pin — PASS:**
- All `uses:` lines in the new CI steps are SHA-pinned (e.g., `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2`, `taiki-e/install-action@cf525cb33f51aca27cd6fa02034117ab963ff9f1 # v2.75.22`). Matches project convention.

**OWASP Top 10 — PASS:**
- No injection surface in diff (no user-controlled strings flow into shell commands or WASM imports).
- `find_host_interface_namespace` reads component import names from wasmtime's reflection API — these are deserialized WIT names, not user input.
- `*/host@*` pattern match in `find_host_interface_namespace` is simple string containment; a maliciously crafted plugin that exports a name containing `/host@` would only affect its own namespace binding, not other plugins. Acceptable.

### Code Review Findings

| ID | Finding | Severity | Category | Route | Status |
|----|---------|----------|----------|-------|--------|
| CODE-1 | All three ACs (001, 002, 003) have tests. AC-001 test un-ignored. AC-002 new test covers n-1 survivor rule. AC-003 new test covers double-401 terminal path. Coverage is complete. | — | coverage | — | PASS |
| CODE-2 | `allow_shadowing(false)` restored after overwrite in `build_component_linker`. Correct. | — | coherence | — | PASS |
| CODE-3 | `extract_component_exports` flattening logic is correct: outer `ComponentInstance` names AND inner function names are both collected, enabling `validate_wit_interface` to match against either naming form. | — | coherence | — | PASS |
| CODE-4 | Justfile fallback path removed — correct. The bare `wasm-tools component new` without `--adapt` produces a non-functional core-module artifact for WASI reactors. Removing the silent fallback prevents shipping a broken .prx without error. | — | coherence | — | PASS |
| CODE-5 | `concat!(env!("CARGO_MANIFEST_DIR"), "/plugins/...")` path construction is correct for nextest. Previously used a cwd-relative path which is unreliable across cargo/nextest invocations. | — | coherence | — | PASS |

**SAP-1 check (tracing emission catalog):**
No new `tracing::*!(event_type=...)` sites found in the diff. Existing sites unchanged. SAP-1 probe: PASS.

**SAP-2 check (DTU↔TOML parity):**
No sensor TOML specs modified in this diff. SAP-2 probe: N/A.

---

## Verdict

**APPROVE** — Zero blocking findings. All three ACs have load-bearing tests. WASM sandbox invariants (BC-2.17.002) are correctly implemented. CI supply-chain follows project conventions. Three non-blocking suggestions (SEC-1 checksum, SEC-2 async env safety annotation, SEC-3 hash documentation) do not block merge under project policy.
