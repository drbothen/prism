---
document_type: adr
adr_id: "ADR-040"
title: "Dual-Path Infusion Architecture — HttpLookup (Declarative TOML) vs WASM Plugin, Host-Decode Path for ThreatIntel .prx Plugin, NVD via HttpLookup"
status: ACCEPTED
date: "2026-06-17"
modified: "2026-06-17"
version: "2.0"
producer: architect
subsystems_affected: [SS-17, SS-19]
supersedes: []
superseded_by: null
amends: null
anchor_stories: [S-DEMO-ENRICHMENT-PIVOT-002]
related_adrs: [ADR-023, ADR-028, ADR-031]
related_bcs: [BC-2.19.001]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-040: Dual-Path Infusion Architecture — HttpLookup vs WASM Plugin

## Status

ACCEPTED v2.0 (2026-06-17). Amendment adds §D7 (dual-path selection principle),
§D8 (HttpLookup source contract), §D9 (NVD plugin crate disposal), and narrows
the scope of D1–D6 to the ThreatIntel WASM Plugin path only. NVD moves from WASM
to the new `InfusionType::HttpLookup` permanent built-in.

v1.0 (2026-06-17): Closed F-001 CRIT (`PluginRuntime::enrich_single` unconditional
`Ok(None)`); specified Val-lift contract, guest binding strategy, error codes, and
`validate_plugin_ref_path` wiring point.

---

## Amendment Summary (v1.0 → v2.0)

The senior architect ratified a DUAL-PATH enrichment architecture on 2026-06-17:

| Enrichment Source | InfusionType | Rationale |
|-------------------|-------------|-----------|
| NVD CVSS Lookup | `HttpLookup` (NEW, permanent built-in) | Single stateless GET → JSONPath → field map; zero custom logic; no WASM toolchain needed |
| ThreatIntel IOC Lookup | `Plugin` (WASM, existing) | Polymorphic IOC classification (`is_ip_address` / `is_domain` / `is_hash`) requires code that cannot be expressed declaratively |

Consequence for PIVOT-002 scope:
- `prism-nvd-infusion` WASM plugin crate is **NOT BUILT and NOT NEEDED**; must be
  removed from the Cargo workspace and from all PIVOT-002 Justfile recipes.
- `nvd.infusion.toml` is updated to `type = "http_lookup"` (see §D8).
- `prism-threatintel-infusion` WASM plugin crate proceeds unchanged per D3.
- D1–D6 below are narrowed to the **ThreatIntel WASM Plugin path only**.

---

## Context

### The Bug

`PluginRuntime::enrich_single` (in `crates/prism-spec-engine/src/plugin/mod.rs`,
Component Model path, lines 903–984) contains two critical defects:

1. **Wrong params.** The function passes four `Val::S32(0)` values to `enrich-single`,
   which is declared in the WIT as `func(input-value: string, input-type: string)`.
   The Component Model Canonical ABI represents a WIT `string` argument as a (ptr, len)
   pair of `i32` values. The current code passes `(0, len)` for each string — meaning
   it always passes a zero pointer (pointing to WASM linear memory address 0, not the
   actual string data).

2. **Discarded result.** After `func.call`, the method checks `call_result` for `Ok`/`Err`
   but immediately returns `Ok(None)` in the `Ok` arm — it never reads `results[0]`.
   The WIT declares `enrich-single` as returning `enrichment` which is `option<string>`
   (a JSON-encoded enrichment object or null). The `results` buffer is pre-populated
   with `Val::S32(0)` (a sentinel, not the lifted value) and then discarded.

The result: every `.prx` infusion plugin — including the ThreatIntel and NVD plugins
required for PIVOT-002 — silently returns `None` for every enrichment call in production.

### Why the Current Code Cannot Work

The Component Model Canonical ABI does NOT allow raw-pointer `Val::S32` to pass or
receive WIT `string` types. In the Component Model (post-`wasm-tools` adapt step),
strings are lifted into host-side `Val::String(String)` values by the wasmtime runtime
before the host sees them. Likewise, the guest's returned `option<string>` is lifted into
`Val::Option(Some(Box<Val::String(...)>))` or `Val::Option(None)`.

The `dispatch_plugin_acquire_token` method in the SAME file (lines 621–869) demonstrates
the correct pattern: it passes `params: [wasmtime::component::Val; 0] = []` (zero params
because credentials flow through `host::get-config`, not WIT params), then reads the
result by inspecting the KV store after dispatch. This approach side-steps the string-
passing problem by routing data through the host state rather than WIT arguments.

For `enrich-single`, however, the input data (input_value and input_type) MUST transit
the WIT boundary as WIT strings — they come from query engine row values, not from
pre-loaded config. The correct host-side representation is `Val::String(...)`.

### The WIT Contract (Existing, in `prism-infusion-plugin.wit`)

The existing WIT file is correct and complete. The `infusion-plugin` interface already
declares:

```wit
type enrichment = option<string>; // JSON-encoded enrichment object or null

enrich-single: func(
    input-value: string,
    input-type: string,
) -> enrichment;
```

The WIT `string` type lifts to `wasmtime::component::Val::String(String)` on both
ingress (host → guest) and egress (guest → host). The return type `option<string>`
lifts to `Val::Option(OptionVal)` where `OptionVal` is:
- `Some(Box<Val::String(json_string)>)` when the plugin found enrichment data
- `None` (represented as `Val::Option(None)` internally) when no data is available

The WIT `world infusion-plugin-world` already includes `import host` and
`export infusion-plugin`, which is the correct structure for the guest to call
`host::http-request` and export `enrich-single` / `enrich-batch`.

**No WIT changes are required for PIVOT-002.** The existing WIT is correct.

---

## Decision

### D1: WIT Contract for `enrich-single` — No Changes Required

The existing `prism-infusion-plugin.wit` WIT contract is correct as-is:

```wit
// Return type for enrich-single / enrich-batch elements.
// JSON-encoded enrichment object (as UTF-8 string) or null (no data found).
// The JSON object's keys MUST correspond to the [[infusion.fields]] names
// declared in the .infusion.toml spec that loads this plugin.
type enrichment = option<string>;

enrich-single: func(
    input-value: string,   // The value to enrich (IOC, CVE ID, IP, etc.)
    input-type: string,    // The type discriminant ("ip", "domain", "hash", "cve_id", etc.)
) -> enrichment;
```

The convention for the returned JSON string is:
- ThreatIntel plugin returns:
  `{"threat_score": 85, "threat_is_known_malicious": true, "threat_sources": ["GreyNoise", "AbuseIPDB"]}`
- NVD plugin returns:
  `{"cvss_base_score": 8.1, "cvss_severity": "HIGH", "cvss_vector": "CVSS:3.1/AV:N/AC:H/PR:N/UI:N/S:U/C:H/I:H/A:H"}`

The JSON keys MUST exactly match the `[[infusion.fields]]` `name` entries in the
corresponding `.infusion.toml` spec. The host (`PluginInfusionSource`) parses the
JSON string and presents the fields as columns in the DataFusion result set.

The `input-type` discriminant values for ThreatIntel are: `"ip"`, `"domain"`, `"hash"`.
The `input-type` discriminant for NVD is: `"cve_id"`.
These values are declared in the `.infusion.toml` spec's `[[infusion.fields]]` `input_type`
attribute and passed through by `PluginInfusionSource` unchanged.

### D2: Host-Side Canonical ABI LIFT Strategy in `PluginRuntime::enrich_single`

The implementer MUST replace the Component Model path of `enrich_single` with the
following contract. This is the ONLY acceptable host-decode approach — it uses
`Val::String` for string arguments and `Val::Option` for the return value, which is
how wasmtime's Component Model lifts these types.

#### Params

Pass input strings as `Val::String(String)` — NOT as `Val::S32` ptr/len pairs:

```rust
let params = [
    wasmtime::component::Val::String(input_value.to_string()),
    wasmtime::component::Val::String(input_type.to_string()),
];
```

The wasmtime Component Model runtime lowers `Val::String(s)` into the guest's linear
memory automatically: it allocates memory in the guest's `cabi_realloc` region, copies
the UTF-8 bytes, and passes the (ptr, len) pair as the canonical ABI lowering. The host
does NOT manage guest linear memory for string arguments.

#### Results buffer

Pre-populate `results` with the correct `Val` shape to match the WIT return type
`option<string>`:

```rust
let mut results = vec![
    wasmtime::component::Val::Option(None),
];
```

The `None` is the default; wasmtime will overwrite it with the actual return value.

#### Return-value lift

After `func.call` returns `Ok(_)`, lift `results[0]` as follows:

```rust
match call_result {
    Ok(_) => {
        match results.into_iter().next() {
            Some(wasmtime::component::Val::Option(Some(boxed_val))) => {
                match *boxed_val {
                    wasmtime::component::Val::String(json_str) => {
                        // Parse the JSON string returned by the plugin.
                        match serde_json::from_str::<serde_json::Value>(&json_str) {
                            Ok(v) => Ok(Some(v)),
                            Err(e) => {
                                // Plugin returned non-JSON — treat as E-INFUSE-008.
                                error!(
                                    event_type = "plugin_enrich_json_parse_error",
                                    plugin_id = %plugin_id,
                                    error = %e,
                                    "plugin enrich-single returned non-JSON string"
                                );
                                Err(classify_enrich_call_failed(plugin_id, &e.to_string()))
                            }
                        }
                    }
                    other => {
                        // Plugin returned unexpected Val variant inside Option<Some>.
                        error!(
                            event_type = "plugin_enrich_unexpected_val",
                            plugin_id = %plugin_id,
                            "plugin enrich-single returned unexpected Val type inside Option<Some>"
                        );
                        Err(classify_enrich_call_failed(plugin_id, &format!("unexpected Val: {:?}", other)))
                    }
                }
            }
            Some(wasmtime::component::Val::Option(None)) | None => {
                // Plugin returned option::none — no enrichment data available.
                Ok(None)
            }
            Some(other) => {
                // Plugin returned wrong type entirely — protocol error.
                error!(
                    event_type = "plugin_enrich_unexpected_val",
                    plugin_id = %plugin_id,
                    "plugin enrich-single returned unexpected Val variant (expected option<string>)"
                );
                Err(classify_enrich_call_failed(plugin_id, &format!("unexpected result Val: {:?}", other)))
            }
        }
    }
    Err(e) => Err(classify_wasm_error(
        plugin_id,
        e.into(),
        DEFAULT_MEMORY_LIMIT_MB,
        elapsed_ms,
        DEFAULT_TIMEOUT_SECONDS * 1000,
    )),
}
```

The helper `classify_enrich_call_failed(plugin_id, reason)` maps to
`PluginError::EnrichCallFailed` (see D5 below).

**The `enrich_batch` method MUST receive the same treatment** — replace the
`Val::S32` params and discarded results with the same `Val::String` / `Val::Option`
pattern, returning `Vec<Option<Value>>` by calling the inner `enrich-single` WIT
function per item OR by implementing a true batch lift if the plugin supports it
(the WIT `enrich-batch` exists; follow the same lift pattern with `Val::List`
for the input and `Val::List` for the output).

#### Function resolution for the interface export

The `enrich-single` function is exported inside the `infusion-plugin` WIT interface
(not as a bare top-level export). The Component Model wraps it in an interface instance.
The interface name in the component binary follows the WIT package name pattern:
`"prism:infusion-plugin/infusion-plugin@0.1.0"`.

Use the same two-phase lookup pattern proven in `dispatch_plugin_acquire_token`:

```rust
// Phase 1: try bare name (WAT test fixtures).
let func = instance.get_func(&mut store, "enrich-single");

// Phase 2: if not found, scan for the interface instance export.
let func = if func.is_none() {
    let component = plugin.pre_instance.component();
    let interface_candidates = {
        let known = "prism:infusion-plugin/infusion-plugin@0.1.0".to_string();
        let mut candidates = vec![known];
        for (name, _) in component.component_type().exports(&self.engine) {
            if name.contains("/infusion-plugin@") && !candidates.contains(&name.to_string()) {
                candidates.push(name.to_string());
            }
        }
        candidates
    };
    let mut found = None;
    'outer: for iface_name in &interface_candidates {
        if let Some(iface_idx) = component.get_export_index(None, iface_name.as_str())
            && let Some(fn_idx) = component.get_export_index(Some(&iface_idx), "enrich-single")
            && let Some(f) = instance.get_func(&mut store, fn_idx)
        {
            found = Some(f);
            break 'outer;
        }
    }
    found
} else {
    func
};
```

This mirrors the battle-tested `dispatch_plugin_acquire_token` pattern; it handles
both test fixtures (bare name) and real Component Model binaries (interface-scoped).

### D3: Guest `wit_bindgen::generate!` Binding Strategy (ThreatIntel WASM Plugin Only)

**Scope note (v2.0):** This section applies to `prism-threatintel-infusion` ONLY.
`prism-nvd-infusion` is retired per D9; these directives do NOT apply to it.

The `prism-threatintel-infusion` plugin crate
MUST declare a `wit_bindgen::generate!` block that references the WIT world from
`prism-infusion-plugin.wit`:

```rust
// In src/lib.rs of each plugin crate.
wit_bindgen::generate!({
    world: "infusion-plugin-world",
    // Path to the WIT file, relative to the plugin crate's Cargo.toml.
    // The WIT file is in prism-spec-engine/wit/; since these are out-of-workspace
    // standalone crates, the path must be absolute or supplied via Cargo.toml
    // [package.metadata.wit] or a copied/symlinked WIT file.
    // RECOMMENDED: copy prism-spec-engine/wit/prism-infusion-plugin.wit into
    // each plugin's wit/ directory and reference it as:
    path: "wit",
    // Generate bindings for the host import and the plugin export.
    exports: {
        "prism:infusion-plugin/infusion-plugin": Plugin,
    },
});
```

The `generate!` macro produces:
- `bindings::exports::prism::infusion_plugin::infusion_plugin::Guest` trait
  (the interface the plugin implements)
- `bindings::prism::infusion_plugin::host::{HttpResponse, LogLevel, http_request, log, get_config, kv_get, kv_set}`
  (the host import functions the plugin calls)

The plugin struct and trait implementation:

```rust
struct Plugin;

impl Guest for Plugin {
    fn name() -> String {
        "threatintel-lookup".to_string()
    }

    fn version() -> String {
        "1.0.0".to_string()
    }

    fn enrich_single(input_value: String, input_type: String) -> Option<String> {
        // Dispatch HTTP via host::http_request (imported from WIT).
        // Parse response, serialize result fields as JSON, return Some(json).
        // Return None if the lookup finds no data.
        // ...
    }

    fn enrich_batch(inputs: Vec<String>, input_type: String) -> Vec<Option<String>> {
        inputs.iter()
            .map(|v| Self::enrich_single(v.clone(), input_type.clone()))
            .collect()
    }
}

export!(Plugin);
```

**WIT file placement for out-of-workspace plugin crates:** Since
`crates/plugins/prism-threatintel-infusion/` is a standalone `[workspace]` crate
(not a workspace member), it cannot reference `../../prism-spec-engine/wit/` with a
simple relative path unless the `wit_bindgen::generate!` macro supports absolute paths
(it does via `path = "<abs>"`) OR the WIT file is copied into the plugin's own `wit/` directory.

**Decision:** Copy `prism-spec-engine/wit/prism-infusion-plugin.wit` into
`prism-threatintel-infusion/wit/prism-infusion-plugin.wit` and use `path = "wit"` in `generate!`.
This avoids cross-workspace path coupling and makes the plugin crate self-contained.
The implementer MUST ensure the copy is kept in sync with the canonical source via a
CI check or Justfile recipe (see D4).

**Cargo.toml guest dependencies for `wit_bindgen`:**

```toml
# In crates/plugins/prism-threatintel-infusion/Cargo.toml
[dependencies]
wit-bindgen = "0.51"       # matches toolchain confirmed in story frontmatter
serde_json = "1"           # for serializing the enrichment response

[lib]
crate-type = ["cdylib", "lib"]

# MUST NOT include reqwest or tokio — HTTP goes through host::http-request WIT import.
```

### D4: Justfile Build Pipeline for Plugin Crates (ThreatIntel Only — v2.0)

**Scope note (v2.0):** Only ONE recipe is needed. `build-plugin-nvd-infusion` is retired
per D9 — do NOT add it. The implementer adds ONE new recipe:

```makefile
build-plugin-threatintel-infusion:
    cargo build --manifest-path crates/plugins/prism-threatintel-infusion/Cargo.toml \
        --target wasm32-wasip1 --release
    wasm-tools component new \
        target/wasm32-wasip1/release/prism_threatintel_infusion.wasm \
        --adapt wasi_snapshot_preview1.wasm \
        -o .prism/plugins/threatintel-lookup.prx
```

The `wasi_snapshot_preview1.wasm` adapter path must match the project-wide convention
from the `build-plugin-crowdstrike-oauth2` recipe.

NVD CVSS enrichment is served by `InfusionType::HttpLookup` — no `.prx` build step,
no WASM toolchain dependency, no `nvd-lookup.prx` artifact.

### D5: New `PluginError::EnrichCallFailed` Variant

The existing `PluginError` enum in `prism-core` does NOT have a variant for
enrichment-specific failures (e.g., the plugin returned non-JSON, or an unexpected Val
type). The implementer MUST add:

```rust
/// E-PLUGIN-023: Plugin `enrich-single` call completed but returned an invalid or
/// unparseable result — the JSON string returned by the guest could not be deserialized,
/// OR the Val type in `results[0]` was not the expected `Val::Option<Val::String>`.
/// Distinct from `Trapped` (WASM trap) and `NotLoaded` (registry miss).
/// Mapped to `InfusionError::PluginCallFailed` (E-INFUSE-008) at the `plugin_bridge.rs`
/// boundary.
#[error("plugin '{plugin_id}' enrich-single call failed: {reason}")]
EnrichCallFailed { plugin_id: String, reason: String },
```

**Code allocation:** E-PLUGIN-023 is the next-free code after E-PLUGIN-022
(`AuthTokenNotCached`, added in PLUGIN-MIGRATION-001-E). The implementer MUST add
the E-PLUGIN-023 row to the error taxonomy in `prd-supplements/error-taxonomy.md` as
part of the same commit that adds the variant.

The `map_plugin_error_to_infusion_error` function in `plugin_bridge.rs` MUST be updated
to map `PluginError::EnrichCallFailed` to `InfusionError::PluginCallFailed`
(E-INFUSE-008). The `InfusionError::PluginCallFailed` variant (also new, per the
existing E-INFUSE-008 taxonomy entry in `error-taxonomy.md`) MUST be added to
`prism-core`'s `InfusionError` enum in the same commit.

**BC-2.16.002 obligation:** The two new `event_type` values —
`"plugin_enrich_json_parse_error"` and `"plugin_enrich_unexpected_val"` — MUST be added
as rows in the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions (SAP-1
obligation). Recurrence policy for both: one event per failing enrichment invocation
(not de-duplicated).

### D6: `validate_plugin_path` Wiring Point (CWE-22 / AC-011)

AC-011 of S-DEMO-ENRICHMENT-PIVOT-002 requires that `plugin_ref` paths from
`.infusion.toml` specs are canonicalized and verified against the configured plugin
directory before any file I/O.

The correct wiring point is in `InfusionRegistry::load_spec_with_runtime` (or wherever
the `PluginConfig::plugin_path` string is resolved to a filesystem path before
`PluginRuntime::load_plugin` is called). This is NOT in `load_plugin` itself
(which operates on already-resolved paths) and NOT in `load_all_plugins`
(which scans a directory — no user-controlled path input).

The validation function the implementer MUST write and call is:

```rust
/// Validate that `plugin_ref` resolves to a path within `plugin_dir`.
///
/// Steps:
/// 1. Construct candidate = plugin_dir.join(plugin_ref)
/// 2. std::fs::canonicalize(candidate) — resolves symlinks and `..` traversals.
///    Returns Err if the path does not exist; treat as InfusionError::InvalidFieldSpec.
/// 3. Assert canonicalized.starts_with(canonical_plugin_dir) where
///    canonical_plugin_dir = std::fs::canonicalize(plugin_dir)?
///    If false: return Err(InfusionError::InvalidFieldSpec {
///        field: "plugin_ref".to_string(),
///        spec_path: spec_path.to_string(),
///        message: "plugin path escapes plugin directory".to_string(),
///    })
///    Do NOT include the attempted path or the plugin_dir path in the error
///    message (CWE-209 / DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001).
fn validate_plugin_ref_path(
    plugin_dir: &Path,
    plugin_ref: &str,
    spec_path: &str,
) -> Result<std::path::PathBuf, InfusionError>
```

**Why not in `PluginRuntime::load_plugin`:** `load_plugin` is called from multiple
sites including hot-reload. Placing path validation there would require passing
`plugin_dir` into a function that currently takes only a `&Path` to the specific file.
The correct separation is: infusion-layer validates infusion-declared paths;
plugin-runtime loads from already-trusted paths. The infusion loader is the correct
trust boundary.

**Note on `load_all_plugins`:** The F-002 (CWE-22) finding in the adversary report
referenced the fact that `load_all_plugins` calls `std::fs::read` without path
validation. This path is safe: `load_all_plugins` enumerates `.prx` files from an
operator-configured `plugin_dir` using `std::fs::read_dir` — not from user-controlled
input. The CWE-22 risk is in the `plugin_ref` code path (user-controlled string from
a TOML spec), not in the directory scan. Do NOT add canonicalize checks to
`load_all_plugins`.

---

### D7: Dual-Path Infusion Source Selection Principle (Permanent)

`InfusionType::HttpLookup` is a **permanent production built-in** added in PIVOT-002,
not a demo shortcut. The selection principle for all current and future enrichment sources:

**Use `InfusionType::HttpLookup` when ALL of the following hold:**
1. The enrichment is a single stateless HTTP GET (or POST with a static body template).
2. The response is a JSON object from which one or more fields are extracted via JSONPath.
3. No custom code is required: no multi-hop calls, no branching on response content,
   no regex/classification of the input value, no aggregation across multiple calls.
4. The auth mechanism is one of: query parameter, Bearer header, or API key header
   (all supported by the `Interpolator` + credential reference model).

**Use `InfusionType::Plugin` (WASM) when ANY of the following hold:**
1. The enrichment requires classifying the input value to select a URL (e.g., IOC routing).
2. The enrichment makes multiple HTTP calls (multi-hop, fan-out, or conditional).
3. The enrichment aggregates or transforms results in a way that requires code logic.
4. The enrichment is provided by a third party as a compiled WASM binary.

**Applied to current PIVOT-002 enrichment sources:**

| Source | Type | Decisive reason |
|--------|------|----------------|
| NVD CVSS Lookup | `HttpLookup` | Single GET `/rest/json/cves/2.0?cveId=${input}` → JSONPath; zero logic |
| ThreatIntel IOC Lookup | `Plugin` | `classify_ioc(input)` dispatch to `/v3/ip/`, `/v3/domain/`, or `/v3/hash/` requires code |

**`is_api_backed` classification:** Both `HttpLookup` and `Plugin` sources make external
HTTP calls and MUST be classified as API-backed. The `InfusionRegistry::is_api_backed`
method MUST return `true` for UDF names from `HttpLookup` specs (BC-2.19.003 /
INV-INFUSE-003 / E-RULE-012 prohibition on detection rule filters). The implementer MUST
update `is_api_backed` to check for `InfusionType::HttpLookup` in addition to
`InfusionType::Plugin`.

---

### D8: `InfusionType::HttpLookup` Source Contract (Implementation-Ready Directives)

#### D8.1: New TOML schema for `type = "http_lookup"` specs

The `nvd.infusion.toml` MUST be updated to use the following schema.
This replaces the existing `type = "plugin"` / `[source].plugin_ref = "nvd-lookup.prx"` block.

```toml
[infusion]
infusion_id = "nvd"
name        = "NVD CVSS Lookup"
type        = "http_lookup"

[source.http]
base_url      = "https://services.nvd.nist.gov"
url_template  = "/rest/json/cves/2.0?cveId=${input}"
method        = "GET"
response_path = "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData"

# Credential reference (AI-opaque, AD-017 / ADR-032).
# Auth mechanism: ?apiKey= query param appended by HttpLookupSource
# at call time using the resolved credential value.
# DTU override: when PRISM_NVD_BASE_URL is set, base_url is overridden.
[source.credential]
ref     = "nvd.api_key"
env_var = "PRISM_NVD_API_KEY"
auth    = "query_param"
param_name = "apiKey"

# DTU-grounded fields (SAP-2, confirmed from prism-dtu-nvd types.rs camelCase):
[[infusion.fields]]
name          = "cvss_base_score"
input_field   = "device_cves_first"
input_type    = "cve_id"
output_type   = "float"
source_column = "baseScore"
description   = "CVSS v3.1 base score (0.0-10.0)"

[[infusion.fields]]
name          = "cvss_severity"
input_field   = "device_cves_first"
input_type    = "cve_id"
output_type   = "string"
source_column = "baseSeverity"
description   = "CVSS v3.1 base severity (LOW/MEDIUM/HIGH/CRITICAL)"

[[infusion.fields]]
name          = "cvss_vector"
input_field   = "device_cves_first"
input_type    = "cve_id"
output_type   = "string"
source_column = "vectorString"
description   = "CVSS v3.1 vector string"

[infusion.pipe_stage]
adds_columns = ["cvss_base_score", "cvss_severity", "cvss_vector"]
```

**Schema notes:**
- `[source.http]` is the new TOML block; `[source.credential]` carries the auth ref.
- `source_column` per `[[infusion.fields]]` is the JSONPath key within the subtree
  addressed by `response_path`. The UDF extracts `response_path_object[source_column]`.
- `auth = "query_param"` / `param_name = "apiKey"` directs the HttpLookupSource
  to append `?apiKey=<resolved_value>` to the URL at call time. Credential value
  is resolved from `env_var` at runtime; never stored in the spec.

#### D8.2: New Rust types

The implementer MUST add the following to `crates/prism-spec-engine/src/infusion/mod.rs`:

1. **`InfusionType::HttpLookup` variant** — add alongside `LocalLookup` and `Plugin`.
   Doc comment MUST include: "HTTP lookup (single GET → JSONPath extraction). PROHIBITED
   in detection rule filters (E-RULE-012) — API-backed."

2. **`HttpLookupAuthType` enum** (new, `#[non_exhaustive]`):
   ```
   QueryParam { param_name: String }   // appends ?name=<value>
   BearerHeader                        // Authorization: Bearer <value>
   ApiKeyHeader { header_name: String } // X-Api-Key: <value> or custom
   ```

3. **`HttpLookupCredentialConfig` struct** (new, `#[non_exhaustive]`):
   - `ref_name: String` — the logical credential ref (for diagnostics, safe to log).
   - `env_var: String` — resolved at call time from env var (AD-017).
   - `auth: HttpLookupAuthType` — how the resolved value is passed to the HTTP request.

4. **`HttpLookupConfig` struct** (new, `#[non_exhaustive]`):
   - `base_url: String` — e.g., `"https://services.nvd.nist.gov"`.
   - `url_template: String` — e.g., `"/rest/json/cves/2.0?cveId=${input}"`.
   - `method: String` — `"GET"` or `"POST"`.
   - `response_path: String` — JSONPath to the subtree containing output fields.
   - `credential: Option<HttpLookupCredentialConfig>` — auth config; `None` = unauthenticated.

5. **`InfusionSpec.http_lookup_config: Option<HttpLookupConfig>`** — new field on
   `InfusionSpec`. `None` for `LocalLookup` and `Plugin` types. Present and required
   for `HttpLookup` type.

All new public types require `#[non_exhaustive]` (CLAUDE.md §Conventions). The
compile-fail gate `EXPECTED=66` in `ci.yml` MUST be incremented by the count of
new public non-exhaustive types added (implementer counts and updates BOTH `ci.yml`
and the CLAUDE.md sentence that tracks the count).

#### D8.3: Loader changes (`loader.rs`)

The `InfusionLoader::parse` function MUST be extended to handle `source_type_str == "http_lookup"`:

```rust
"http_lookup" => InfusionType::HttpLookup,
```

When `InfusionType::HttpLookup` is resolved, the loader MUST parse the `[source.http]`
block into an `HttpLookupConfig` and `[source.credential]` into an
`HttpLookupCredentialConfig`, and populate `spec.http_lookup_config`.

Validation for `HttpLookup` type:
- `base_url` must be non-empty.
- `url_template` must be non-empty and contain `${input}` (validate at parse time;
  return `InfusionError::InvalidFieldSpec` if absent — prevents silent mis-enrichment).
- `method` must be `"GET"` or `"POST"`.
- `response_path` must be non-empty.
- Every `[[infusion.fields]]` entry that uses `source_column` must have a non-empty
  `source_column` value.

**No change to the `plugin_ref` validation path** — that remains for `InfusionType::Plugin`.

#### D8.4: `HttpLookupSource` implementation

New file: `crates/prism-spec-engine/src/infusion/sources/http_lookup.rs`.

`HttpLookupSource` implements `InfusionSource`. Construction:

```rust
pub struct HttpLookupSource {
    client: reqwest::Client,          // build_http_client_with_timeout(30) — CLAUDE.md §Conventions
    config: HttpLookupConfig,
    spec_path: String,                // for error messages (no credential values — CWE-209)
}
```

`enrich_single(input: &str, input_type: &str) -> Option<serde_json::Value>`:

1. Resolve the credential value from `env::var(&config.credential.env_var)`.
   On `Err` (env var not set): return `Err(InfusionError::CredentialResolutionFailed)`.
   Credential value MUST NOT appear in any log or error message (AD-017 / INV-INFUSE-005).

2. Build the full URL:
   ```
   url = config.base_url + interpolate(config.url_template, [("input", input)])
   ```
   Use the existing `Interpolator::interpolate` from `crates/prism-spec-engine/src/pipeline.rs`
   (the `${var}` substitution logic). Do NOT re-implement template interpolation.

3. Apply auth per `HttpLookupAuthType`:
   - `QueryParam { param_name }`: append `?{param_name}={credential_value}` to the URL.
   - `BearerHeader`: add `Authorization: Bearer {credential_value}` header.
   - `ApiKeyHeader { header_name }`: add `{header_name}: {credential_value}` header.
   The credential value is used for the HTTP call and MUST NOT be stored in any
   struct field, logged, or included in error messages after the call completes.

4. Issue the HTTP call via `self.client.get(url)` (or `.post(url)` for POST).
   30-second timeout is already set on the client at construction time.
   On non-2xx status: return `Err(InfusionError::HttpLookupFailed)` (see §D8.5).
   On network error: return `Err(InfusionError::HttpLookupFailed)`.

5. Parse the response body as `serde_json::Value`.
   On parse failure: return `Err(InfusionError::HttpLookupFailed)`.

6. Extract the `response_path` subtree using the existing `extract_at_path` function
   from `crates/prism-spec-engine/src/pipeline.rs`. This function already handles
   `$.array[0].nested.field` notation — do NOT re-implement JSONPath.

7. If `extract_at_path` returns `None` (path not found in response): return `Ok(None)`.
   This is the "no enrichment data for this input" case (e.g., CVE not found in NVD).

8. Return `Ok(Some(subtree_value))` where `subtree_value` is the JSON object at
   `response_path`. The UDF layer reads `subtree_value[source_column]` per field.

**SSRF / URL allowlist validation (security):**

The implementer MUST validate that `base_url` is not a private/loopback address in
production mode. The validation rule: reject `base_url` values whose hostname resolves
to RFC-1918 ranges (10.x, 172.16-31.x, 192.168.x), loopback (127.x, ::1), or
link-local (169.254.x) unless `PRISM_DTU_MODE=true` is set in the environment (DTU
override for test isolation). This check MUST happen at `HttpLookupSource` construction
time (not at call time) so it is caught at registry load, not at query execution.

On SSRF rejection: return `Err(InfusionError::InvalidFieldSpec { field: "base_url",
spec_path, message: "base_url resolves to a private/loopback address" })`.

Do NOT include the resolved IP address in the error message (CWE-209).

**Caching:** `HttpLookupSource::enrich_single` is called from the same UDF machinery
that already wraps `PluginInfusionSource`. The three-tier infusion cache (S-1.14-REDO)
wraps the `InfusionSource` trait — `HttpLookupSource` does not need to implement
caching itself. Cache TTL uses `spec.cache_ttl_secs` (default 3600s, same as Plugin).

**`enrich_batch`:** Default implementation calls `enrich_single` per item (same as
`PluginInfusionSource`'s batch default). No dedicated batch HTTP endpoint for NVD.

#### D8.5: New error codes required (product-owner to register)

The following new error codes are required by `HttpLookupSource`. The implementer MUST
add these variant shells to the `InfusionError` enum in `prism-core`; the product-owner
MUST register the corresponding taxonomy rows in `prd-supplements/error-taxonomy.md`
in the same story:

| Code | Variant | Condition |
|------|---------|-----------|
| **E-INFUSE-009** | `HttpLookupFailed { spec_path, status_code: Option<u16>, message }` | HTTP call failed (non-2xx, network error, or JSON parse failure). `message` MUST NOT contain credential values. |
| **E-INFUSE-010** | `CredentialResolutionFailed { spec_path, credential_ref }` | Env var for credential not set at call time. `credential_ref` is the logical name (safe to log); the env var name MUST NOT appear in the message. |
| **E-INFUSE-011** | `SsrfRejected { spec_path }` | `base_url` resolves to a private/loopback address and `PRISM_DTU_MODE` is not set. No IP address in the message (CWE-209). |

**Note to product-owner:** E-INFUSE-009 through E-INFUSE-011 are the next-free codes
after E-INFUSE-008 (`PluginCallFailed`, allocated in D5 of this ADR v1.0). Register all
three rows in `error-taxonomy.md` as part of the PIVOT-002 story, same commit that adds
the enum variants.

#### D8.6: Registry wiring for `HttpLookup` in `load_spec_with_runtime`

The `InfusionRegistry::load_spec_with_runtime` method currently branches on
`InfusionType::Plugin` to construct a `PluginInfusionSource`. The implementer MUST
add a second branch:

```rust
InfusionType::HttpLookup => {
    let http_config = spec.http_lookup_config.as_ref()
        .ok_or_else(|| InfusionError::MissingRequiredField {
            field: "http_lookup_config".to_string(),
            spec_path: spec.source_path.clone(),
        })?;
    Arc::new(HttpLookupSource::new(
        build_http_client_with_timeout(30),
        http_config.clone(),
        spec.source_path.clone(),
    )?)  // construction validates SSRF
}
```

The `build_http_client_with_timeout` function is already in `pipeline.rs` — import and
reuse it. Do NOT call `reqwest::Client::new()` (CLAUDE.md §Conventions forbidden pattern).

The `InfusionRegistry::is_api_backed` method MUST be updated to return `true` for
`InfusionType::HttpLookup` (same as `Plugin`):

```rust
return spec.infusion_type == InfusionType::Plugin
    || spec.infusion_type == InfusionType::HttpLookup;
```

#### D8.7: BC-2.16.002 tracing obligation (SAP-1)

Two new `event_type` values introduced by `HttpLookupSource` MUST be registered in
the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions:

| event_type | Condition | Recurrence |
|------------|-----------|------------|
| `"http_lookup_enrich_failed"` | `InfusionError::HttpLookupFailed` returned | Once per failing call |
| `"http_lookup_ssrf_rejected"` | `InfusionError::SsrfRejected` returned | Once per registry-load rejection |

Fields for `http_lookup_enrich_failed`: `infusion_id`, `spec_path`, `status_code` (u16 or absent).
Fields for `http_lookup_ssrf_rejected`: `infusion_id`, `spec_path`.
Neither field set may include credential values or resolved IP addresses.

---

### D9: NVD Plugin Crate Disposition

**`prism-nvd-infusion` WASM plugin crate is retired.**

With NVD moving to `InfusionType::HttpLookup`, the `crates/plugins/prism-nvd-infusion/`
crate serves no purpose. The implementer MUST:

1. Remove `crates/plugins/prism-nvd-infusion/` from the filesystem.
2. Remove any reference to `prism-nvd-infusion` from:
   - Root `Cargo.toml` `members` array (if present — plugin crates may be standalone).
   - `Justfile`: remove the `build-plugin-nvd-infusion` recipe (per D4 v1.0 — that
     recipe was defined in this ADR and must now be retracted).
   - Any CI workflow step that builds `nvd-lookup.prx`.
   - Any `.prism/plugins/` reference to `nvd-lookup.prx`.
3. Update `nvd.infusion.toml` to use `type = "http_lookup"` per §D8.1.

**`prism-threatintel-infusion` WASM plugin crate is retained** and proceeds exactly
as specified in D3–D4 of this ADR (v1.0 decisions, unchanged). The `build-plugin-threatintel-infusion`
Justfile recipe and `threatintel-lookup.prx` artifact remain in scope.

---

## Scope Boundary — PIVOT-002 vs Deferred Work (v2.0 update)

All items below are in-scope for PIVOT-002 under the production-grade default:

| Item | Scope decision | Rationale |
|------|---------------|-----------|
| `InfusionType::HttpLookup` variant + new config types | IN-SCOPE (PIVOT-002) | NVD path requires it; permanent built-in per D7 |
| `HttpLookupSource` implementation (~150-200 lines) | IN-SCOPE (PIVOT-002) | The entire NVD enrichment path; demo-blocking |
| SSRF `base_url` validation at construction | IN-SCOPE (PIVOT-002) | Security gate; not a deferred hardening item |
| `is_api_backed` update for `HttpLookup` | IN-SCOPE (PIVOT-002) | BC-2.19.003 / INV-INFUSE-003 correctness; E-RULE-012 filter prohibition |
| E-INFUSE-009 / E-INFUSE-010 / E-INFUSE-011 error variants | IN-SCOPE (PIVOT-002) | Required for type-safe error handling in `HttpLookupSource` |
| `nvd.infusion.toml` updated to `type = "http_lookup"` | IN-SCOPE (PIVOT-002) | Spec change driven by D8.1 |
| Remove `prism-nvd-infusion` crate + `build-plugin-nvd-infusion` recipe | IN-SCOPE (PIVOT-002) | Per D9; crate is dead code once NVD moves to HttpLookup |
| `compile-fail gate EXPECTED` count update for new non-exhaustive types | IN-SCOPE (PIVOT-002) | CLAUDE.md §Conventions; CI gate enforces count |
| BC-2.16.002 catalog rows for two new event_type values (SAP-1) | IN-SCOPE (PIVOT-002) | SAP-1 same-commit obligation |
| `enrich_single` Component Model Val lift fix (ThreatIntel WASM) | IN-SCOPE (PIVOT-002) | Closes F-001 CRIT for ThreatIntel path |
| `enrich_batch` Component Model Val lift fix (ThreatIntel WASM) | IN-SCOPE (PIVOT-002) | Same code path as `enrich_single` |
| `PluginError::EnrichCallFailed` + `InfusionError::PluginCallFailed` variants | IN-SCOPE (PIVOT-002) | E-INFUSE-008 taxonomy row already allocated |
| `validate_plugin_ref_path` (AC-011 CWE-22) | IN-SCOPE (PIVOT-002) | Security gate for Plugin path |
| Two new BC-2.16.002 catalog rows for WASM enrich events (SAP-1) | IN-SCOPE (PIVOT-002) | SAP-1 requires same-commit catalog registration |

The following items are explicitly out of scope for PIVOT-002:

| Item | Out-of-scope rationale |
|------|----------------------|
| `wit_bindgen` typed bindings via `bindgen!` macro | Raw `Val` approach and `wit_bindgen` typed approach both work; typed-binding migration is a refactor. S-1.14-REDO scope. |
| `fire_alert` / `fire_case` / `fire_report` WASM dispatch (stub) | S-4.08; unrelated to infusion path |
| `InfusionSource::enrich_single` → `async fn` migration | `spawn_blocking` wrapping is acceptable (implementation (a)); async trait migration is S-1.14-REDO scope |
| `HttpLookup` POST body template or pagination | Not needed for NVD (GET only); extend in S-1.14-REDO if a future enrichment source requires it |
| `input_type_dispatch` map for multi-URL HttpLookup sources | Not needed for NVD; ThreatIntel uses Plugin path which handles IOC routing in WASM |

---

## Consequences

### Positive

- **ThreatIntel** `.prx` infusion plugin returns enrichment data to the query engine
  via the corrected Val-lift path (F-001 CRIT closed for ThreatIntel).
- **NVD** enrichment is delivered without any WASM toolchain dependency — no
  `wasm-tools`, no `wasm32-wasip1` cross-compile, no `wit_bindgen` for NVD.
  ~150-200 lines of Rust replaces an entire plugin crate build pipeline.
- The `HttpLookupSource` reuses `Interpolator` and `extract_at_path` from `pipeline.rs`
  — proven on production sensor workloads (CrowdStrike, Armis, Cyberint, Claroty).
- `is_api_backed` correctly classifies both `Plugin` and `HttpLookup` sources,
  enforcing the E-RULE-012 detection rule filter prohibition consistently.
- `validate_plugin_ref_path` closes the CWE-22 path traversal risk at the correct
  trust boundary (infusion loader, not plugin runtime) — applies to Plugin path only.
- E-PLUGIN-023, E-INFUSE-008 (Plugin path), E-INFUSE-009/010/011 (HttpLookup path)
  give operators distinct, triage-able error codes for each failure mode.
- The `prism-nvd-infusion` crate removal reduces the workspace by one dead-code crate.

### Negative / Risks

- The ThreatIntel `Val::String` / `Val::Option` round-trip relies on the guest plugin
  having been compiled with `wasm-tools component new --adapt wasi_snapshot_preview1.wasm`.
  WAT core-module test fixtures do NOT go through this path. Test fixtures that exercise
  enrichment data return MUST be Component Model binaries.
- The WIT `option<string>` return for ThreatIntel requires that both `Val::Option(Some(...))`
  and `Val::Option(None)` arms are handled. A guest compiled against the wrong WIT version
  could produce `Val::Result(...)` instead of `Val::Option(...)`. The
  `plugin_enrich_unexpected_val` event provides runtime-observable failure; unit tests
  MUST cover both arms.
- `HttpLookupSource` SSRF validation uses a hostname-based RFC-1918 check at construction
  time. This check requires a DNS lookup for non-literal hostnames; if DNS is unavailable
  at registry-load time, the check must fail closed (reject the spec, not accept it).

### Migration / Implementation Order

The implementer MUST follow this order to satisfy TDD discipline (Red Gate first):

**Step 1 — Error taxonomy (foundation for all error paths):**
1. Add `PluginError::EnrichCallFailed` to `prism-core/src/error.rs`.
2. Add `InfusionError::PluginCallFailed` (E-INFUSE-008) to same file.
3. Add `InfusionError::HttpLookupFailed` (E-INFUSE-009), `CredentialResolutionFailed`
   (E-INFUSE-010), `SsrfRejected` (E-INFUSE-011) to same file.
4. Add E-PLUGIN-023, E-INFUSE-008, E-INFUSE-009, E-INFUSE-010, E-INFUSE-011 taxonomy
   rows to `prd-supplements/error-taxonomy.md` in the same commit.
5. Update `map_plugin_error_to_infusion_error` in `plugin_bridge.rs`.

**Step 2 — HttpLookup type infrastructure:**
6. Add `InfusionType::HttpLookup`, `HttpLookupAuthType`, `HttpLookupCredentialConfig`,
   `HttpLookupConfig` to `infusion/mod.rs`; add `http_lookup_config` field to `InfusionSpec`.
7. Update `InfusionLoader::parse` to handle `"http_lookup"` source type (D8.3).
8. Write failing Red Gate tests for `HttpLookupSource` construction and `enrich_single`
   (unit tests with a `wiremock` or equivalent mock HTTP server; no real NVD calls in CI).
9. Implement `HttpLookupSource` in `sources/http_lookup.rs` per D8.4.
   Reuse `Interpolator` and `extract_at_path` from `pipeline.rs`.
   Reuse `build_http_client_with_timeout(30)` from `pipeline.rs`.
10. Update `InfusionRegistry::load_spec_with_runtime` to branch on `HttpLookup` (D8.6).
11. Update `InfusionRegistry::is_api_backed` to include `HttpLookup` (D8.6).
12. Update `nvd.infusion.toml` to `type = "http_lookup"` per D8.1.
13. Remove `prism-nvd-infusion` crate and `build-plugin-nvd-infusion` Justfile recipe (D9).
14. Update `EXPECTED=66` in `ci.yml` and the CLAUDE.md count sentence for new
    `#[non_exhaustive]` types.

**Step 3 — WASM Plugin path for ThreatIntel (D2–D6, now ThreatIntel-only):**
15. Write failing Red Gate tests for `enrich_single` Val-lift (unit, WAT or minimal `.prx`).
16. Fix `PluginRuntime::enrich_single` per D2.
17. Fix `PluginRuntime::enrich_batch` per D2.
18. Write `validate_plugin_ref_path` and its Red Gate tests (AC-011, D6).
19. Copy WIT file into `prism-threatintel-infusion/wit/` (D3).
20. Implement `prism-threatintel-infusion/src/lib.rs` with `wit_bindgen::generate!` (D3).
21. Add `build-plugin-threatintel-infusion` Justfile recipe (D4).

**Step 4 — BC-2.16.002 catalog rows (SAP-1, same commit as tracing sites):**
22. Add four BC-2.16.002 catalog rows:
    - `plugin_enrich_json_parse_error` (WASM path, D5)
    - `plugin_enrich_unexpected_val` (WASM path, D5)
    - `http_lookup_enrich_failed` (HttpLookup path, D8.7)
    - `http_lookup_ssrf_rejected` (HttpLookup path, D8.7)

**Step 5 — Final gate:**
23. Run `just check` — all Red Gate tests pass.

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-06-17 | Initial ACCEPTED (architect, S-DEMO-ENRICHMENT-PIVOT-002). Closes F-001 CRIT (enrich_single unconditional Ok(None)) and F-002 (CWE-22 validate_plugin_path wiring). Specifies Val lift contract, guest binding strategy, error codes, scope boundary. NVD and ThreatIntel both on Plugin path in v1.0. |
| 2.0 | 2026-06-17 | Amendment: Dual-path architecture ratified by senior architect. NVD moves from Plugin (WASM) to new permanent built-in InfusionType::HttpLookup (§D7–D9). ThreatIntel stays on Plugin path (IOC classification requires code). prism-nvd-infusion crate retired. HttpLookup source contract specified: HttpLookupConfig, HttpLookupSource, SSRF validation, credential handling (AD-017), pipeline.rs reuse (Interpolator, extract_at_path, build_http_client_with_timeout). E-INFUSE-009/010/011 error codes specified (product-owner to register taxonomy rows). Implementation order updated to 23-step TDD sequence. D1–D6 narrowed to ThreatIntel WASM path only. |
