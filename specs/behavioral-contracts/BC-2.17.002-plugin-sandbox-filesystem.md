---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-17"
capability: "CAP-032"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to: ["CAP-032"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.17.002: Plugin Sandbox — No Direct Filesystem or Network Access

## Description

WASM plugins have NO direct access to the host filesystem, network stack, or process
spawning capabilities. The only interfaces available to plugins are those explicitly
defined in the Prism WIT host interface (`host::http_request`, `host::log`,
`host::get_config`, `host::kv_get`, `host::kv_set`). WASI filesystem and network
interfaces are deliberately NOT linked to plugin instances. This is INV-PLUGIN-002.

## Preconditions

- A WASM plugin is loaded by `PluginRuntime` (compiled and registered in the plugin registry)
- The plugin's WASM binary attempts to access the host filesystem (e.g., via WASI
  `path_open` syscall) or make a direct network socket call (e.g., via WASI `sock_open`)

## Postconditions

- **Filesystem access attempt:** The WASM call fails with a WASM trap because the
  WASI filesystem interface is not linked. `wasmtime` returns a link error at
  instantiation time or a trap at call time (import not satisfied). The plugin call
  returns `Err(PluginError::Trapped { ... })` to the host.
- **Direct network access attempt:** The WASM call fails identically — WASI network
  interfaces (`sock_*`) are not linked. No network packet is emitted from the host.
- **Allowed HTTP access:** When a plugin calls `host::http_request(method, url, headers, body)`,
  the request is executed via the host's `reqwest::Client` (not directly from WASM),
  subject to URL allowlist validation (see EC-17-006), and audit-logged with
  `(plugin_id, method, url, status, latency_ms)`.
- The `PluginRuntime` `wasmtime::component::Linker` is configured at construction time
  with ONLY the Prism host interface bindings — no WASI imports are registered.

## Invariants

- INV-PLUGIN-002: Plugins have NO direct filesystem access, NO direct network access, NO process spawning
- The WASI preview2 `wasi:filesystem`, `wasi:sockets`, `wasi:process`, and
  `wasi:environment` interfaces MUST NOT be linked into plugin instances
- All plugin outbound HTTP calls route exclusively through `host::http_request` on
  the host — plugins cannot bypass this proxy
- URL allowlist enforcement applies to every `host::http_request` call: if
  `[plugin.allowed_urls]` is configured, requests to non-allowlisted domains are
  rejected with an HTTP 403 equivalent returned to the plugin

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PLUGIN-004` | Plugin attempts WASI filesystem/network call (import not linked) | WASM trap → `Err(PluginError::Trapped)` at call boundary |
| `E-PLUGIN-005` | `host::http_request` URL not in configured allowlist | Plugin receives HTTP 403 response; host logs `WARN "Plugin '{plugin_id}' attempted HTTP to non-allowlisted URL: {url}"` |
| `E-PLUGIN-005` | `host::http_request` times out (10s per request limit) | Plugin receives HTTP 408/timeout response; host audit-logs failure with latency |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-17-005 | Plugin binary compiled with WASI imports present | At `load_plugin` time, `wasmtime::component::Linker::instantiate_pre` fails because WASI imports are unsatisfied → `E-PLUGIN-001` rejection; plugin not registered |
| EC-17-006 | Plugin calls `host::http_request` with URL in allowlist | Request executed via `HostState.http_client` (reqwest); response returned to plugin; audit log entry created |
| EC-17-007 | Plugin calls `host::http_request` when no allowlist is configured | Request allowed to any URL (open by default); audit log entry created |
| EC-17-008 | Plugin calls `host::kv_get` / `host::kv_set` | KV operations execute against `HostState.kv_store`, scoped to `"{plugin_id}:{key}"`. No cross-plugin KV access. |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-17-002-happy | Plugin calls `host::http_request` with allowlisted URL | Request proxied via reqwest; response returned to plugin | EC-17-006 |
| TV-17-002-blocked | Plugin binary with WASI `path_open` import | Load rejected at `instantiate_pre` → `E-PLUGIN-001` | EC-17-005 |
| TV-17-002-allowlist | Plugin calls `host::http_request` with non-allowlisted URL | HTTP 403 returned to plugin; WARN logged | Error row 2 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-040 | `PluginRuntime::build_linker()` produces a Linker whose import namespace set does not contain any `wasi:` prefixed interface name | Kani |
| (none) | HTTP proxy routes through host reqwest client — integration behavior; integration test with mock HTTP server | — |

## Related BCs

- BC-2.17.001 — Plugin Panic Isolation (filesystem/network call failure traps are caught here)
- BC-2.17.003 — Memory Limit Enforcement (orthogonal sandbox dimension)
- BC-2.17.004 — CPU Time Limit Enforcement (orthogonal sandbox dimension)
- BC-2.17.006 — WIT Validation (validates that plugin does not import unsupported interfaces)

## Architecture Anchors

- AD-019: WASM plugins — sandbox constraints
- `specs/architecture/sensor-adapters.md` — host functions, URL allowlist, KV store scoping
- S-1.15 Task 4: `plugin/host_functions.rs` — HTTP proxy, KV operations
- S-1.15 Architecture Compliance: "Do NOT enable WASI for plugin instances"

## Story Anchor

S-1.15 — prism-spec-engine: WASM Plugin Runtime (AC-4, AC-5 cover this behavior)

## VP Anchors

Integration test: `tests/plugin_tests.rs` — "Verify `host::http_request` proxy: mock HTTP server, plugin calls `http_request` → verify request goes through host proxy, not direct from plugin WASM."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-032 |
| Story Invariant | INV-PLUGIN-002 |
| ADR | AD-019 |
| Story | S-1.15 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-040); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
