# Review Findings — S-1.15 WASM Plugin Runtime

## Convergence Table

| Cycle | Total Findings | Blocking | Fixed | Remaining |
|-------|---------------|----------|-------|-----------|
| 1 | 5 | 2 | 0 | 5 |
| 2 | 0 | 0 | 5 | 0 → APPROVE |

## Cycle 1 Findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| F1 | MEDIUM | `create_store()` ignores `memory_limit_mb` — StoreLimits not enforced in production enrich calls | FIXED (commit c575f09) |
| F2 | MEDIUM | URL allowlist uses substring match — bypassable via query params | FIXED (commit c575f09) |
| F3 | LOW | `PluginRuntime` fields `engine`/`linker`/`registry` are `pub` instead of `pub(crate)` | TECH-DEBT — integration tests in tests/ require pub access; tracked below |
| F4 | LOW | `fire_alert`/`fire_case`/`fire_report` stubs silently return Ok without TODO comment | FIXED (commit c575f09) |
| F5 | SUGGESTION | `allowed_urls: None` hardcoded in `make_host_state` — no doc comment on deferral | FIXED (commit c575f09) |

## Tech Debt Register Entry (F3)

- **Item:** `PluginRuntime` fields `engine`, `linker`, `registry` are `pub` (should be `pub(crate)` or behind accessor methods)
- **Reason blocked:** Integration test file (`tests/plugin_tests.rs`) is outside the crate and directly accesses these fields for hot_reload and VP-042 tests. Making them `pub(crate)` breaks compilation of the test binary.
- **Remediation path:** Refactor VP-042 and hot_reload tests to use only the public API methods (`load_plugin`, `get_plugin`, `list_plugins`), wrapping the `hot_reload` and `hot_unload` functions behind a `PluginRuntime::reload_plugin()` / `PluginRuntime::unload_plugin()` method pair. Target story: **S-4.08**.
- **Risk:** Low — the fields are only accessible to code that explicitly imports `PluginRuntime`; no injection risk.
