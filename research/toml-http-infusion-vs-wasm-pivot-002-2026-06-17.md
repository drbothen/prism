---
document_type: design-investigation
title: "Can ThreatIntel/NVD Enrichment Be Served by Pure Declarative TOML (No WASM)?"
date: "2026-06-17"
author: architect
related_story: S-DEMO-ENRICHMENT-PIVOT-002
related_adrs: [ADR-040, ADR-023, ADR-028]
status: FINAL
---

# Investigation: Pure TOML `http_lookup` vs WASM Plugin for ThreatIntel/NVD Enrichment

## 1. Executive Answer

**PARTIAL — YES for NVD, PARTIAL for ThreatIntel with constraints.**

A declarative TOML `http_lookup` infusion source type could cover the NVD enrichment path with
zero custom code. ThreatIntel is PARTIAL: the structured HTTP call and response extraction are
fully declarative, but the IOC-type routing logic (`input_type = "ioc"` dispatching to `/v3/ip/:ip`,
`/v3/domain/:domain`, or `/v3/hash/:hash`) requires a decision function that a pure static TOML
spec cannot express without a new `InfusionType::HttpLookup` variant that supports a `url_template`
with an `input_type_dispatch` map.

Critically: **neither source TODAY has an `http_lookup` infusion type.** The current `InfusionType`
enum has exactly two variants:
- `LocalLookup` — backed by a local file (MMDB, CSV, or JSON). PROHIBITED from HTTP.
- `Plugin` — WASM `.prx` plugin dispatched via `PluginRuntime`. This is what PIVOT-002 uses.

The sensor `PipelineExecutor` does have a mature, battle-hardened declarative HTTP executor with
field mapping, authentication, pagination, fan-out, and JSONPath extraction — but it is wired only
to `SensorSpec`, not to `InfusionSpec`. Reusing it for enrichment would require either:
(a) a new `InfusionType::HttpLookup` variant that wraps a `SensorSpec`-equivalent config and
    drives `PipelineExecutor`, or
(b) keeping WASM for the general plugin path and only adding a lightweight
    `InfusionType::HttpLookup` variant for the simple lookup cases.

---

## 2. What the Code Actually Shows

### 2.1 Existing InfusionType variants (infusion/mod.rs)

```rust
pub enum InfusionType {
    LocalLookup,   // MMDB / CSV / JSON — file-backed, no HTTP
    Plugin,        // WASM .prx — may make HTTP calls via host::http-request WIT
}
```

`BuiltInSourceType` variants are `MaxmindMmdb`, `Csv`, `JsonLookup` — all file paths. The
`InfusionSourceConfig` struct has a `file_path` field and no URL or auth fields. There is no
`http_lookup` source type in any part of the codebase. The `grep -rn "http_lookup"` search
returned zero results.

### 2.2 The sensor fetch pipeline (pipeline.rs) IS a declarative HTTP executor

`PipelineExecutor::execute` in `crates/prism-spec-engine/src/pipeline.rs` is a full declarative
HTTP pipeline: it takes a `SensorSpec` (which carries `base_url`, `auth_type`) and a `TableSpec`
(which carries `steps: Vec<FetchStep>` each with `path_template`, `response_path`, `method`,
`body_template`, pagination, fan-out). It supports:
- Multiple auth types (Bearer, ApiKey, OAuth2, Cookie)
- Variable interpolation (`${step.field}`)
- Pagination (cursor, offset/limit)
- JSONPath extraction
- 401 retry with token refresh
- Rate-limit hints

This is exactly the machinery that would power an `http_lookup` infusion source. The execution
model is already proven for production sensor workloads (CrowdStrike, Armis, Cyberint, Claroty).

### 2.3 What ThreatIntel enrichment actually needs

The `threatintel.infusion.toml` spec declares `input_type = "ioc"` for all three fields.
The prism-dtu-threatintel DTU serves three distinct routes:
- `GET /v3/ip/:ip` (lookup.rs:162)
- `GET /v3/domain/:domain` (lookup.rs:187)
- `GET /v3/hash/:hash` (lookup.rs:214)

The WASM plugin stub (`prism-threatintel-infusion/src/lib.rs`) explicitly contains three functions:
`is_ip_address()`, `is_domain()`, `is_hash()` — all currently `todo!()` stubs — that classify
the IOC value before routing to the correct DTU endpoint. This is custom logic: given an arbitrary
string value (e.g., `"192.168.1.1"`, `"evil.com"`, `"d41d8cd98f00b204e9800998ecf8427e"`), the
plugin must determine whether it is an IP, domain, or hash to select the correct URL path segment.

A pure TOML spec cannot express: "if the input looks like an IPv4/IPv6, call `/v3/ip/...`;
if it looks like a domain, call `/v3/domain/...`; otherwise call `/v3/hash/...`". That conditional
routing requires either:
1. A Rust `InfusionSource` implementation with a `classify_ioc()` function, or
2. A WASM plugin that implements the classification.

**However**, there is an alternative design that avoids the problem entirely: the sensor TOML
(specifically the Armis or CrowdStrike spec) could project separate columns for different IOC
types BEFORE enrichment — e.g., `device_ip`, `external_domain`, `file_hash`. Then three
separate `http_lookup` infusion sources, each with a single static URL template and a known
`input_type`, would be sufficient. This is a data schema decision that happens UPSTREAM of
enrichment.

For the PIVOT-002 DEMO specifically: the current threatintel.infusion.toml uses `input_type =
"ioc"` (a single polymorphic type discriminant). This requires routing code. The NVD spec uses
`input_type = "cve_id"` (a single uniform type) and maps to a single URL template
`/rest/json/cves/2.0?cveId=${input}`. NVD requires NO type routing.

### 2.4 What NVD enrichment actually needs

The `nvd.infusion.toml` spec has `input_type = "cve_id"` and all three output fields come from
the same HTTP call shape: `GET /rest/json/cves/2.0?cveId=<input>&apiKey=<key>`. The response
is a nested JSON envelope that requires a JSONPath traversal:
`$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseScore`

The prism-dtu-nvd response is camelCase (`serde rename_all = "camelCase"`). The `PipelineExecutor`
already handles nested JSONPath extraction with bracket notation and wildcards
(`extract_at_path` supports `$.array[0].nested.field`). The NVD enrichment is exactly the
sensor pipeline pattern: one HTTP GET, one JSON response, JSONPath extraction of multiple fields.

**NVD could be served by a declarative `http_lookup` InfusionSource without any WASM.**

---

## 3. What a Declarative `http_lookup` InfusionSource Would Require

A new `InfusionType::HttpLookup` variant with the following TOML config block:

```toml
[infusion]
infusion_id = "nvd"
name = "NVD CVSS Lookup"
type = "http_lookup"           # NEW variant — not "plugin"

[source.http]
base_url = "https://services.nvd.nist.gov"   # or DTU override
url_template = "/rest/json/cves/2.0?cveId=${input}&apiKey=${credential.nvd_api_key}"
method = "GET"
response_path = "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData"

# Credential reference (AI-opaque, AD-017)
[source.credential]
ref = "nvd.api_key"

[[infusion.fields]]
name = "cvss_base_score"
input_field = "device_cves_first"
input_type = "cve_id"
output_type = "float"
source_column = "baseScore"      # JSONPath within the response_path object

[[infusion.fields]]
name = "cvss_severity"
input_field = "device_cves_first"
input_type = "cve_id"
output_type = "string"
source_column = "baseSeverity"

[[infusion.fields]]
name = "cvss_vector"
input_field = "device_cves_first"
input_type = "cve_id"
output_type = "string"
source_column = "vectorString"
```

This is exactly "eat our own dog food" — the same TOML pattern sensors use. The Rust implementation
would be a new `HttpLookupSource` struct implementing `InfusionSource` that:
1. Holds a `reqwest::Client` (with 30s timeout per CLAUDE.md), a `base_url`, a `url_template`, and a `response_path`.
2. In `enrich_single(input, input_type)`: resolves the credential from env var (AD-017), interpolates `${input}` and `${credential.*}` in `url_template` (reusing `Interpolator`), issues a GET, extracts the `response_path` subtree using `extract_at_path`, and returns `Some(json_subtree)`.
3. The UDF machinery then reads `source_column` fields from the returned JSON object.

New code needed: ~150-200 lines of Rust (`HttpLookupSource` + config structs + a new `BuiltInSourceType::HttpLookup` or `InfusionType::HttpLookup` variant). No WASM toolchain, no `wit_bindgen`, no `wasm-tools` build step.

For ThreatIntel: an `http_lookup` source could work IF the TOML includes an `input_type_dispatch`
map (routes each IOC type to a different URL template). This is an extension to the config schema
but involves no custom code execution logic — it is declarative routing:

```toml
[source.http]
input_type_dispatch = true
url_templates = [
  { input_type = "ip",     template = "/v3/ip/${input}?key=${credential.threatintel_api_key}" },
  { input_type = "domain", template = "/v3/domain/${input}?key=${credential.threatintel_api_key}" },
  { input_type = "hash",   template = "/v3/hash/${input}?key=${credential.threatintel_api_key}" },
]
```

But this requires: (a) the upstream sensor projecting the IOC type as a separate column alongside
the IOC value, and (b) the infusion UDF receiving `(input_value, input_type)` as separate
arguments. The current `InfusionSource::enrich_single(input: &str, input_type: &str)` signature
already has `input_type` as a parameter. So the routing logic IS expressible declaratively.

---

## 4. Why PIVOT-001 Chose WASM

Reading ADR-019 and ADR-023 in context (they address SIEM output and plugin-only sensor
architecture respectively) plus the PIVOT-002 story context:

The WASM plugin path was chosen for infusion enrichment for the following reasons:
1. **General extensibility / third-party custom enrichment** (per `InfusionType` doc comment:
   "WASM plugin delegation (may make external HTTP calls)"). The design explicitly permits
   arbitrary code in the plugin.
2. **IOC type classification logic** for ThreatIntel — the `is_ip_address()`, `is_domain()`,
   `is_hash()` stubs in `prism-threatintel-infusion/src/lib.rs` are custom code that the
   original design assumed would live in WASM.
3. **Symmetry with the sensor plugin architecture** (ADR-023 establishes a plugin-only sensor
   model; infusion plugins followed the same pattern).

The rationale was NOT specific to ThreatIntel or NVD. It was the general extensibility case
plus the assumption that IOC classification would need custom code. Both assumptions are
partially correct, but neither mandates WASM for the PIVOT-002 demo path.

---

## 5. Feasibility Assessment

### NVD: `http_lookup` is fully viable

| Requirement | Declarative TOML covers it? |
|-------------|----------------------------|
| Single HTTP GET with query param | YES — `url_template` |
| API key auth via query param | YES — `${credential.nvd_api_key}` interpolation |
| Nested JSONPath extraction | YES — `PipelineExecutor::extract_at_path` already handles `$.array[0].nested.field` |
| Multiple output columns from same response | YES — `source_column` per field |
| Rate limiting | YES — `rate_limit_hints` already in `SensorSpec` model |
| Credential handling (AI-opaque) | YES — same `CredentialRef` model |

**NVD is 100% feasible as a pure declarative `http_lookup` infusion with ~150 lines of new Rust
in `sources/http_lookup.rs` plus 2 new enum variants (`InfusionType::HttpLookup`,
`BuiltInSourceType::HttpLookup`) and a TOML config extension. No WASM. No `wasm-tools`.**

### ThreatIntel: `http_lookup` is viable with one design choice

| Requirement | Declarative TOML covers it? |
|-------------|----------------------------|
| HTTP GET to one of three endpoint patterns | YES — `input_type_dispatch` map |
| API key auth via query param or Bearer | YES — credential interpolation |
| Response JSON field extraction | YES — JSONPath |
| IOC type routing (ip/domain/hash) | REQUIRES: either (a) upstream sensor projects separate columns with known types, OR (b) `input_type_dispatch` map in `http_lookup` config |
| IOC type classification from raw value | NOT DECLARATIVE — requires `is_ip_address()` / `is_domain()` / `is_hash()` Rust functions |

The key question is whether the IOC type classification happens in the enrichment source or upstream.

**Option A (upstream classification):** The Armis/CrowdStrike sensor TOML projects separate
`alert_source_ip`, `alert_source_domain`, and `alert_source_hash` columns from the sensor data.
Each gets its own `http_lookup` infusion spec with a fixed URL pattern. Zero routing code
needed. This is the cleanest design and aligns with the dogfooding pattern.

**Option B (dispatch map):** Add `input_type_dispatch` to `HttpLookupSourceConfig` so the TOML
can declare multiple URL templates keyed by `input_type`. The `input_type` value (`"ip"`,
`"domain"`, `"hash"`) must be passed to the UDF as a second argument from the query — this
requires a UDF signature change (currently UDFs take one input column). Feasible but adds
complexity.

**Option C (keep WASM for ThreatIntel, use http_lookup for NVD):** This is the minimum viable
simplification for PIVOT-002: NVD drops WASM entirely (saving the build toolchain setup, the
WIT binding, and the ADR-040 Component Model Val-lift fix for NVD). ThreatIntel continues on
WASM (since the IOC routing logic is genuinely code-like). This is the lowest-risk choice for
the current demo timeline.

---

## 6. Trade-offs Table

| Dimension | Pure TOML `http_lookup` | WASM Plugin (`type = "plugin"`) |
|-----------|------------------------|--------------------------------|
| **Implementation complexity** | LOW — ~150-200 lines Rust, reuses `Interpolator` + `extract_at_path`. No new toolchain. | HIGH — Requires: `wasm32-wasip1` cross-compile, `wasm-tools` adapt step, `wit_bindgen` codegen, `Val::String` lift fix (ADR-040 D2), `PluginError::EnrichCallFailed` variant. |
| **PIVOT-002 demo timeline** | FASTER — NVD `http_lookup` can be written and tested in ~1 day. No new toolchain dependencies to provision. | SLOWER — ADR-040 D2 (Val lift fix) + WIT binding + plugin build pipeline must all be completed and tested. Per ADR-040, 10-step implementation order. |
| **Extensibility (third-party enrichment)** | LOWER — custom logic (regex transforms, multi-hop calls, state aggregation) is not expressible. | HIGHER — arbitrary Rust code in the guest. Genuine extensibility for future third-party enrichment cases. |
| **Security / sandbox** | SIMPLER — HTTP client is the same `reqwest::Client` used by the sensor pipeline (proven, 30s timeout, no escape surface). | SAFER for arbitrary code — WASM sandbox prevents runaway guest code. But adds `wasmtime` surface area. |
| **Dogfooding consistency with sensors** | MAXIMUM — uses the exact same TOML declarative pattern. "Eat our own dog food." | INCONSISTENT — sensors are TOML-driven; enrichment sources are WASM-driven. Two mental models. |
| **Detection rule filter prohibition** | SAME — an `HttpLookup` type would still be API-backed and must be classified as `is_api_backed = true`, prohibiting it in detection rule filters per INV-INFUSE-003. | Same prohibition. |
| **Test infrastructure** | SIMPLER — unit tests use `wiremock`/mock HTTP server (same as sensor pipeline tests). | COMPLEX — requires Component Model `.prx` binary or WAT fixture; test toolchain mirrors build toolchain. |
| **ADR-040 applicability** | ADR-040 NOT NEEDED for `http_lookup` sources — only needed for the Plugin path. | ADR-040 is fully required and in-scope for the Plugin path. |
| **Long-term maintenance** | LOWER cost for simple lookup patterns. Higher cost for adding conditional logic later. | HIGHER initial cost; lower marginal cost for complex enrichment logic. |

---

## 7. Recommendation

### For the PIVOT-002 Demo (immediate)

**Hybrid approach:**

1. **NVD: Replace WASM with a declarative `http_lookup` InfusionSource.** Add `InfusionType::HttpLookup`
   variant and `HttpLookupSource` (~150 lines). This eliminates the WASM build toolchain dependency
   for NVD entirely and bypasses ADR-040's Val-lift fix for the NVD plugin. Deliver a `nvd.infusion.toml`
   using `type = "http_lookup"` instead of `type = "plugin"`. Demo-unblocked without WASM.

2. **ThreatIntel: Proceed with WASM per ADR-040.** The IOC routing logic (`is_ip_address()`,
   `is_domain()`, `is_hash()`) genuinely requires code — it is the single non-declarative requirement.
   WASM is the correct tool for enrichment plugins that need custom classification logic. Implement
   the ADR-040 D2 Val-lift fix, the `PluginError::EnrichCallFailed` variant, and the
   `wit_bindgen::generate!` binding as specified.

   **Alternative for ThreatIntel if timeline is tight:** If the PIVOT-002 demo only needs IP
   enrichment (not domain/hash routing), a single `http_lookup` with a fixed URL template
   `/v3/ip/${input}?key=${credential.threatintel_api_key}` can serve the IP-only case. The
   demo sensor fixtures likely emit IP IOCs from CrowdStrike/Armis. This defers WASM for
   ThreatIntel until a story that needs polymorphic IOC routing.

### ADR-040 Status

**ADR-040 is NOT superseded** for the general plugin path or for ThreatIntel IOC routing.
It remains the correct architecture for plugin-type enrichment sources. However, for NVD
specifically, ADR-040 is NOT the right tool: NVD's enrichment pattern is a simple HTTP lookup
with JSON extraction and has no custom logic requiring a WASM sandbox. If the human approves
the `http_lookup` path for NVD, the NVD plugin crate (`prism-nvd-infusion`) and its
corresponding TOML `type = "plugin"` spec become unnecessary for the demo.

**ADR-040 would need a scoped amendment** (not supersession) stating: "For infusion sources
whose entire behavior is a single stateless HTTP GET → JSONPath extraction → field mapping,
`InfusionType::HttpLookup` is preferred over `InfusionType::Plugin`. The Plugin path remains
correct for sources requiring custom logic (IOC classification, multi-hop calls, aggregation)."

---

## 8. Decision Required from Human

The human must decide:

**Q1: NVD path.** Approve `InfusionType::HttpLookup` for NVD enrichment (adds ~150-200 lines
of new Rust, eliminates WASM build for NVD, is cleaner and faster to deliver). If approved,
scope PIVOT-002 to implement `HttpLookupSource`, update `nvd.infusion.toml` to
`type = "http_lookup"`, and remove `prism-nvd-infusion` from the story scope.

**Q2: ThreatIntel path.** Either:
- (a) Proceed with WASM per ADR-040 (full IOC routing, needs `is_ip_address()` etc.), or
- (b) Limit PIVOT-002 demo to IP-only ThreatIntel lookup via `http_lookup` (simpler, faster,
  defers polymorphic IOC routing to a follow-on story), or
- (c) Keep WASM for ThreatIntel as designed in ADR-040 but unblock the demo by doing NVD
  via `http_lookup` first.

**Q3: Long-term direction.** Is `InfusionType::HttpLookup` the canonical pattern for
"simple external lookup" infusion sources (i.e., is this a permanent architecture addition,
not just a demo shortcut)? If yes, this should be captured in an ADR amendment (ADR-040
scoped amendment or a new ADR-041). If it is a "demo shortcut for S-1.14-REDO refactor",
that changes the production-grade obligation (Canonical Principle: demo shortcuts that create
cleanup debt require human direction per Rule 3).

---

## 9. Files Cited

| File | Finding |
|------|---------|
| `crates/prism-spec-engine/src/infusion/mod.rs:47-54` | `InfusionType` has only `LocalLookup` and `Plugin` variants — no `HttpLookup` |
| `crates/prism-spec-engine/src/infusion/sources/mod.rs:23` | `load_source` is `unimplemented!()` stub — S-1.14 implementation pending |
| `crates/prism-spec-engine/src/pipeline.rs:138-574` | Full declarative HTTP pipeline with auth, interpolation, pagination, JSONPath extraction |
| `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002/specs/infusions/nvd.infusion.toml` | NVD uses `type = "plugin"`, single URL pattern, pure JSONPath extraction — no custom logic |
| `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002/specs/infusions/threatintel.infusion.toml` | ThreatIntel uses `type = "plugin"`, `input_type = "ioc"` (polymorphic routing needed) |
| `crates/prism-dtu-threatintel/src/routes/lookup.rs:162,187,214` | Three distinct route handlers: ip_lookup, domain_lookup, hash_lookup |
| `crates/prism-dtu-nvd/src/routes/cves.rs:37-46` | Single endpoint `CveQueryParams` with `cveId` param — pure declarative |
| `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002/crates/plugins/prism-threatintel-infusion/src/lib.rs:102-120` | `is_ip_address()`, `is_domain()`, `is_hash()` stubs — IOC classification requires code |
| `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002/crates/plugins/prism-nvd-infusion/src/lib.rs:117-136` | `enrich_cve()` stub — purely: call URL, parse camelCase JSON, return fields |
| `.factory/specs/architecture/decisions/ADR-040-wasm-infusion-plugin-host-decode-path.md` | ADR-040 rationale: fixes WASM Val-lift bug for Plugin path; NVD not specifically required to be WASM |
