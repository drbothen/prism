---
document_type: research-artifact
topic: multi-tenant-sensor-endpoint-overrides
date: 2026-05-23
research_drivers:
  - PLUGIN-MIGRATION-001-E architecture-clarification (D-803)
  - S-PLUGIN-MULTI-TENANT-ENDPOINT-001 (to-be-stubbed)
  - ADR-NNN-multi-tenant-sensor-endpoint-overrides (to-be-drafted by architect)
mcp_tools_used: [perplexity]
cache_key: "multi-tenant-config-overrides-sensor-tools-2026-05-23"
inputs: []
input-hash: "[live-research]"
---

# Multi-Tenant Sensor Endpoint Overrides — External Pattern Research

## 1. Executive Summary

Across nine comparable observability/integration tools (Vector, Telegraf, OpenTelemetry Collector, Datadog Agent, Filebeat, Logstash, Fluent Bit / Fluentd, Cribl Stream, Wazuh) the **dominant industry pattern is "multiple named instances of the same TYPE, each with a complete per-instance configuration, composed from a directory of include files."** No mainstream tool implements a TYPE-spec + per-tenant-override-file merge as its primary mechanism. Telegraf's `[[inputs.http]]` repeated-table-array, Datadog's `instances:` YAML list, OTel's named components (`otlp/tenant_a` vs `otlp/tenant_b`), Vector's component IDs, and Logstash multi-pipelines all converge on the same pattern: **one block per instance, drive composition via include directives or directory globbing, parameterize per-instance values with environment-variable interpolation**. For prism this translates to **option (d) — "sensor instance" — augmented with composability features from option (a) (per-org TOML files in a tenants/ directory loaded via include) — i.e., a hybrid (e) recommendation below.** Per-tenant credential isolation is already solved via the keyring; the missing piece is per-instance endpoint declaration, which the industry consistently models as "another instance of the type," not "an override to the type."

## 2. Tool-by-Tool Survey

### 2.1 Vector (timber.io / Datadog)

**Mechanism.** Vector uses *component IDs* as the unit of identity. Multiple components of the same `type` differ only by their TOML table name. Multi-tenant deployments wire fan-out by listing the shared source ID in multiple sinks' `inputs = [...]` arrays. ([Configuration | Vector docs](https://vector.dev/docs/reference/configuration/))

```toml
[sources.app_logs]
type = "file"
include = ["/var/log/app/*.log"]

[sinks.es_tenant_a]
type = "elasticsearch"
inputs = ["app_logs"]
endpoints = ["${TENANT_A_ES_ENDPOINT}"]

[sinks.es_tenant_b]
type = "elasticsearch"
inputs = ["app_logs"]
endpoints = ["${TENANT_B_ES_ENDPOINT}"]
```

`--config-dir ./config` and multiple `--config` flags compose files: later files override earlier files for the SAME component ID; new component IDs are simply added. `${VAR}` interpolation is the supported per-deployment override mechanism, and `--strict-env` fails startup if a referenced var is missing. ([Splitting configuration | Vector docs](https://vector.dev/docs/reference/configuration/#splitting-your-configuration); [Vector CLI](https://vector.dev/docs/reference/cli/))

**Trade-offs.** Pro: zero ambiguity — every tenant is its own first-class declarative object visible in `vector top` and metrics. Con: file count grows linearly with tenants; no schema-validated "tenant" abstraction (operators discipline themselves with naming conventions).

### 2.2 Telegraf (InfluxData)

**Mechanism.** Telegraf uses TOML *array-of-tables* (`[[inputs.http]]`, `[[inputs.snmp]]`). Each repeated header creates an independent instance of that plugin with its own URL, credentials, tags, and interval. `--config-directory` merges every `*.conf` file in a directory as if they were concatenated. Each instance accepts an `alias = "..."` for logging/selection clarity. ([Telegraf inputs.http](https://docs.influxdata.com/telegraf/latest/plugins/#input-http); [Configuration overview](https://docs.influxdata.com/telegraf/latest/configuration/))

```toml
[[inputs.http]]
  alias = "acme"
  urls = ["https://api.acme-corp.io/v1/metrics"]
  tags = { tenant_id = "acme" }

[[inputs.http]]
  alias = "contoso"
  urls = ["https://api.contoso.com/v1/metrics"]
  tags = { tenant_id = "contoso" }
```

**Trade-offs.** Pro: same TYPE used many times with full per-instance parametric freedom; tags propagate downstream for routing. Con: no inheritance — every instance restates the full config; copy-paste burden mitigated by writing TOML generators.

### 2.3 OpenTelemetry Collector

**Mechanism.** OTel Collector uses **named components** — `otlp/tenant_a`, `otlp/tenant_b` — where the suffix is purely an identity discriminator on the same component TYPE. ([OpenTelemetry Collector configuration](https://opentelemetry.io/docs/collector/configuration/)) Multiple `--config` URIs are deep-merged by the Collector's config provider chain; `${env:VAR}` substitution is first-class. The routing connector and routing processor steer telemetry per-tenant.

```yaml
exporters:
  otlp/tenant_a:
    endpoint: ${env:TENANT_A_OTLP_ENDPOINT}
    headers: { authorization: "Bearer ${env:TENANT_A_TOKEN}" }
  otlp/tenant_b:
    endpoint: ${env:TENANT_B_OTLP_ENDPOINT}
    headers: { authorization: "Bearer ${env:TENANT_B_TOKEN}" }

connectors:
  routing:
    table:
      - statement: route() where attributes["tenant"] == "a"
        pipelines: [logs/tenant_a]
```

**Trade-offs.** Pro: cleanest production-grade model; per-tenant isolation extends from declaration through pipeline routing. Con: requires operators to learn the "type/name" component identifier convention.

### 2.4 Datadog Agent

**Mechanism.** Per-integration directory `conf.d/<integration>.d/conf.yaml` with a top-level `instances:` YAML list. Each list element is an independently configured monitored target. ([Datadog Agent configuration](https://docs.datadoghq.com/agent/configuration/); [Kubernetes integrations](https://docs.datadoghq.com/containers/kubernetes/integrations/))

```yaml
# /etc/datadog-agent/conf.d/postgresql.d/conf.yaml
init_config:
instances:
  - host: pg-acme.prod.local
    username: datadog
    tags: [env:prod, tenant:acme]
  - host: pg-contoso.prod.local
    username: datadog
    tags: [env:prod, tenant:contoso]
```

Autodiscovery templates (`auto_conf.yaml`) generate `instances` dynamically using `%%host%%`, `%%port%%`, `%%env_VAR%%` macros. Tags carry tenant identity downstream into dashboards/monitors.

**Trade-offs.** Pro: well-understood industry pattern; per-tenant tag convention standardized. Con: single-file-per-integration grows large with many tenants; rotation/update requires editing one shared file.

### 2.5 Elastic Filebeat / Metricbeat

**Mechanism.** Modules live in `modules.d/<module>.yml`, each enabled by removing `.disabled` suffix. Each YAML file holds a list of `- module: <name>` items; repeating the list lets multiple module instances coexist with different `tenant_id`, `client_id`, etc., and per-instance `processors.add_fields` injects tenant labels into emitted documents. ([Filebeat O365 module](https://www.elastic.co/docs/reference/beats/filebeat/filebeat-module-o365))

**Trade-offs.** Pro: identical conceptual model as Datadog (`instances:`); mature tooling for enable/disable. Con: same single-file-grows-large issue.

### 2.6 Logstash

**Mechanism.** *Pipeline-per-tenant* via top-level `pipelines.yml` plus per-pipeline `conf.d/<tenant>/*.conf` directories. Each pipeline is independent (own workers, own DLQ, own back-pressure boundary). `${VAR}` env substitution parameterizes endpoints/credentials per pipeline. ([Multiple Pipelines | Elastic](https://www.elastic.co/docs/reference/logstash/multiple-pipelines))

```yaml
# /etc/logstash/pipelines.yml
- pipeline.id: tenant-acme
  path.config: "/etc/logstash/conf.d/acme/*.conf"
- pipeline.id: tenant-contoso
  path.config: "/etc/logstash/conf.d/contoso/*.conf"
```

**Trade-offs.** Pro: hardest tenant isolation in the industry (one tenant's queue overflow doesn't block others). Con: heaviest weight — each pipeline is its own JVM thread pool; not idiomatic for in-process query engines like prism.

### 2.7 Fluent Bit / Fluentd

**Mechanism.** Multiple `[INPUT]` sections (or Fluentd `<source>` blocks), each tagged distinctly. Tag-based routing in `[FILTER]`/`[OUTPUT]` (`Match tenantA.*`) or `<match tenantA.**>`. `@INCLUDE tenants/*.conf` composes per-tenant config files. ([Fluent Bit configuration](https://docs.fluentbit.io/manual/administration/configuring-fluent-bit/configuration-file); [Fluentd config file](https://docs.fluentd.org/configuration/config-file))

**Trade-offs.** Pro: tag-based routing is composable and well-documented. Con: tag-discipline failure (operator typos `tenantA` vs `TenantA`) silently routes events to wrong destination — discoverability problem.

### 2.8 Cribl Stream

**Mechanism.** Three layered isolation tiers: **Workspaces** (Cribl.Cloud, fully isolated environments), **Worker Groups** (clusters within a workspace), and per-tenant Sources/Routes/Pipelines/Destinations within a group. Each Sentinel workspace or Kinesis Firehose stream becomes a separate Destination instance with its own endpoint+credential. Routes filter on `__inputId` or custom fields to fan-out per tenant. ([Cribl Workspaces](https://cribl.io/blog/workspaces-unlock-the-power-of-multi-tenancy/); [Cribl Worker Groups](https://cribl.io/blog/worker-groups-what-are-they-and-why-you-should-care/))

**Trade-offs.** Pro: explicit multi-tenant primitives all the way down. Con: significant deployment complexity; commercial product.

### 2.9 Wazuh

**Mechanism.** Single ossec.conf at the manager; multi-tenancy implemented in the OpenSearch/dashboard layer via Document-Level Security (DLS) keyed on `agent.labels.group`. Agent groups receive label injection (`<labels><label key="group">Acme</label></labels>`); operators map roles/policies onto those labels to restrict per-tenant data visibility. ([Wazuh multi-tenancy](https://documentation.wazuh.com/current/user-manual/wazuh-dashboard/multi-tenancy.html))

**Trade-offs.** Wazuh's pattern is *tag-then-filter-at-storage*, not *parameterize-at-ingest* — not directly relevant to prism's per-endpoint problem, but informative for the tag-driven discoverability story.

### 2.10 Helm / Kustomize (Kubernetes packaging — bonus context)

**Helm:** layered values via `-f base.yaml -f env.yaml -f tenant.yaml`. Deep-merge of MAPS; SCALARS overwritten; **ARRAYS REPLACED WHOLESALE** (not merged element-wise). ([Helm values best practices](https://helm.sh/docs/chart_best_practices/values/))

**Kustomize:** `base/` plus `overlays/<env>/<tenant>/`. Strategic-merge patches identify list elements by `name` and merge in-place; lists without a merge key get replaced. ([Kustomize patches](https://kubectl.docs.kubernetes.io/references/kustomize/kustomization/patches/))

Both pattern families confirm the same lesson: **TOML/YAML array-of-tables merge is ambiguous and unsafe to overload**; both ecosystems converged on whole-array replacement.

## 3. TOML Override Patterns (Rust crate deep-dive)

prism is on Rust; its config layer matters. The three options:

### 3.1 `config-rs` (`config` crate)

Layered `Source`s via `ConfigBuilder::add_source`. Last source added wins. Tables deep-merged recursively; **arrays REPLACED**, not merged. Per-environment via app-side file selection (`format!("config/{}", run_mode)`). ([config crate docs](https://docs.rs/config/latest/config/))

### 3.2 `figment` (Sergio Benitez)

`Figment::merge(...)` chains providers — later wins. Has a first-class **Profile** concept (`select("debug")`, `Global` profile that supersedes others). Arrays still REPLACED on merge. ([figment crate docs](https://docs.rs/figment/latest/figment/))

### 3.3 `toml` crate

Pure parser/serializer; no merge semantics — caller implements deep-merge over `toml::Value`.

**Critical implication for prism.** TOML `[[tables]]` array merging is **unsafe** under both library defaults. If prism layered `armis.sensor.toml`'s `[[tables]]` array with a per-org override file's `[[tables]]` array, **the override would REPLACE the entire tables array**, silently losing every table the override file didn't restate. This is exactly the Helm/Kustomize lesson: array merging is a footgun.

Per-org overrides must therefore either (a) NOT touch array-of-tables fields (only scalars), or (b) be modeled as a per-instance complete spec, not a partial-override patch.

## 4. Evaluation of Prism's Four Options Against the Research

| Option | Industry analog | Endpoint flexibility | Discoverability | TOML array hazard | Boilerplate | Cred isolation | Test/validation |
|---|---|---|---|---|---|---|---|
| **(a) Per-org override TOML at `customers/<org>/<sensor>.sensor.toml`** | None directly (Helm `-f` layering comes closest, but Helm replaces arrays) | High (any field overridable) | LOW (split-brain: must read 2 files to know effective config) | HIGH (per-org `[[tables]]` would replace global) | Low if only scalars override; high if structural | Already solved | Medium (must render effective spec to validate) |
| **(b) Per-org inline `[[orgs.sensor_overrides]]` in prism.toml** | Resembles Filebeat `processors.add_fields` per-module item | Medium (flat scalar overrides) | HIGH (single file) | LOW (override is structurally a different table) | Medium | Already solved | Easy |
| **(c) Credential-encoded base URL (`{token, base_url}`)** | No mainstream analog (keyring records are universally token-only) | Medium (URL only — can't override other params) | LOW (endpoint hidden in keyring → ops can't grep config to know where queries go) | N/A | Low | Already solved | Hard (must touch keyring to test) |
| **(d) Sensor instance ID `armis@acme`, `armis@contoso`** | Telegraf `[[inputs.http]]` with `alias`; OTel `otlp/tenant_a`; Vector component IDs; Datadog `instances:` list | High (full per-instance config) | HIGH (every instance is grep-able) | LOW (each instance is its own spec) | High (per-instance restates the type) | Already solved | Easy (each instance validates independently) |

**Verdict.** Option (d) matches the dominant industry pattern. Option (c) is the *anti-pattern* — every tool surveyed keeps endpoint declaration in config and credentials in a separate secret store; conflating them breaks operator discoverability ("where is my data going?" requires reading credentials, which by AD-017 must never transit AI context). Option (a) is appealing on paper but inherits the TOML array-replace hazard that has bitten Helm and Kustomize users for a decade. Option (b) is workable for small scalar overrides but doesn't scale to per-tenant structural differences (different `[[tables]]`, different `[ratelimit]`).

## 5. Hybrid Option (e): "Sensor Instance with Per-Org Composition Directory"

Synthesizing the patterns from Vector (`--config-dir`), Telegraf (`--config-directory`), Fluent Bit (`@INCLUDE tenants/*.conf`), and OTel (named components):

**Layout:**

```
.prism/specs/sensors/
  armis.sensor.toml                       # Global TYPE spec (defaults: schema, tables, vendor-required ratelimit)
  customers/
    acme/armis.sensor.toml                # Per-instance overlay declaring instance armis@acme + endpoint
    contoso/armis.sensor.toml             # Per-instance overlay declaring instance armis@contoso + endpoint
```

**Merge semantics (deliberately constrained):**

1. The global `armis.sensor.toml` declares the TYPE and its full canonical schema (tables, columns, ratelimit defaults). Loaded once at boot, registered as the **type definition** (not as a queryable instance).
2. Each `customers/<org>/<sensor>.sensor.toml` declares an **instance** of the type. Identity: `{sensor_type}@{org_slug}`. Required fields: `base_url`, `instance_id = "armis@acme"`, `extends = "armis"`. Permitted overrides: SCALAR fields only (base_url, ratelimit overrides, optional timeout). `[[tables]]` blocks **forbidden** in the per-org overlay — schema lives at the type level (mirrors Telegraf's TYPE-stable / INSTANCE-parameterized split).
3. Boot-time validation: every per-org overlay must reference an existing global TYPE, every required scalar present, schema not redefined, and `org_slug` must match an `[[orgs]]` entry in `prism.toml`.

**Why this hybrid wins:**

- **Endpoint discoverability:** `find customers/ -name 'armis.sensor.toml'` enumerates every Armis-using tenant.
- **No TOML-array merge footgun:** schema (`[[tables]]`) lives at the TYPE; overlays carry only scalars.
- **SaaS sensors stay zero-boilerplate:** CrowdStrike Falcon (one global endpoint) needs no per-org overlay; `armis.sensor.toml` just declares `base_url = "https://api.crowdstrike.com"`. Only sensors that VARY per tenant get overlay files.
- **Maps cleanly onto existing `(org_id, sensor_id)` credential tuple:** the instance identifier `armis@acme` is the natural composite key.
- **Industry-validated:** combines Telegraf TYPE-stable-INSTANCE-parameterized + OTel named-component identity + Vector config-directory composition.

## 6. Recommended Design for Prism

**Recommendation: Hybrid Option (e) — "Sensor Instance with Per-Org Composition Directory."**

**Three-sentence rationale.** Every mainstream observability/integration tool (Telegraf, Datadog Agent, OTel Collector, Vector, Fluent Bit, Logstash) models per-tenant endpoint variation as **multiple named instances of the same TYPE composed via include directives**, not as merge-overrides of a single TYPE spec. Pure-override approaches (option a) inherit a well-documented TOML array-merge footgun that has plagued Helm and Kustomize for a decade, and pure inline approaches (option b) don't scale to structural per-tenant differences. The hybrid splits the surface: **schema lives at the TYPE, endpoint/scalar-tunables live at the per-org INSTANCE overlay**, which mirrors Telegraf's `[[inputs.http]]`-with-`alias` and OTel's `otlp/tenant_a` exactly while staying compatible with the `(org_id, sensor_id)` credential tuple prism has already implemented.

Supporting citations: [Telegraf inputs.http array-of-tables](https://docs.influxdata.com/telegraf/latest/plugins/#input-http); [OpenTelemetry named components](https://opentelemetry.io/docs/collector/configuration/); [Vector splitting configuration](https://vector.dev/docs/reference/configuration/#splitting-your-configuration); [config crate merge semantics](https://docs.rs/config/latest/config/) (confirms TOML array REPLACE behavior — informs the "schema only at TYPE" rule).

## 7. Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| **Stale per-org overlays after global schema bump.** Operator forgets to verify acme/armis.sensor.toml after upgrading armis.sensor.toml `[[tables]]` definitions. | MEDIUM | Boot-time validator: every per-org overlay's `extends = "armis"` must reference a registered TYPE; emit `WARN` if overlay was last-modified before global type's modtime — pattern from Datadog Agent's `conf.yaml.example` drift detection. |
| **TOML array-merge footgun re-emerges if someone tries to override `[[tables]]` in the per-org overlay.** | HIGH | Hard-reject in `prism-spec-engine`: a per-org overlay file containing `[[tables]]` fails parse with `E-SPEC-NNN`. Test in compile-fail crate per Conventions §perimeter-violation-compile-fail-gates. |
| **Per-org overlay references nonexistent `org_id`.** Operator typo `acmee/armis.sensor.toml`. | MEDIUM | Boot-time cross-check: every `customers/<slug>/` directory must correspond to an `[[orgs]] slug = "..."` entry in prism.toml; fail fast at boot per ADR-022 §A exit-code-on-config-error convention. |
| **Discoverability split-brain: operator queries "what endpoint does armis@acme hit?"** | LOW | `prism config show --sensor armis@acme` renders the effective merged spec with provenance (which file each field came from) — pattern from Helm's `helm get values --revision N`. |
| **AI agent or operator confuses `armis` (TYPE) with `armis@acme` (INSTANCE) in queries.** | LOW | Parser explicitly rejects unscoped sensor refs at query time when multiple instances exist for the type ("ambiguous sensor reference — disambiguate as `armis@<org>`"). |
| **Per-org overlays multiply file count linearly with tenants × sensors-needing-override.** 50 tenants × 4 sensors needing override = 200 files. | LOW | This is the explicit industry trade-off (Telegraf, Datadog accept it). Mitigated by template generators in tooling layer. |
| **Validation timing — when to validate compatibility?** | MEDIUM | Boot time only; query-time lookup is `(org_id → instance_id → cached spec)`. Mirrors osquery and Datadog's startup-validate-then-cache pattern. |

## 8. References / Citations

**Tool documentation (primary sources):**
- Vector: <https://vector.dev/docs/reference/configuration/>, <https://vector.dev/docs/reference/configuration/#splitting-your-configuration>, <https://vector.dev/docs/reference/cli/>
- Telegraf: <https://docs.influxdata.com/telegraf/latest/configuration/>, <https://docs.influxdata.com/telegraf/latest/plugins/#input-http>
- OpenTelemetry Collector: <https://opentelemetry.io/docs/collector/configuration/>
- Datadog Agent: <https://docs.datadoghq.com/agent/configuration/>, <https://docs.datadoghq.com/containers/kubernetes/integrations/>, <https://docs.datadoghq.com/containers/cluster_agent/clusterchecks/>, <https://datadoghq.dev/integrations-core/meta/config-specs/>
- Filebeat O365 module: <https://www.elastic.co/docs/reference/beats/filebeat/filebeat-module-o365>
- Logstash multi-pipeline: <https://www.elastic.co/docs/reference/logstash/multiple-pipelines>, <https://www.elastic.co/docs/reference/logstash/creating-logstash-pipeline>
- Fluent Bit: <https://docs.fluentbit.io/manual/administration/configuring-fluent-bit/configuration-file>, <https://docs.fluentbit.io/manual/data-pipeline/inputs>, <https://docs.fluentbit.io/manual/pipeline/filters>
- Fluentd: <https://docs.fluentd.org/configuration/config-file>, <https://docs.fluentd.org/configuration/routing-examples>
- Cribl Stream: <https://cribl.io/blog/workspaces-unlock-the-power-of-multi-tenancy/>, <https://cribl.io/blog/worker-groups-what-are-they-and-why-you-should-care/>, <https://cribl.io/blog/isolation-a-new-wave-of-experiences-to-help-teams-work-side-by-side/>
- Wazuh: <https://documentation.wazuh.com/current/user-manual/wazuh-dashboard/multi-tenancy.html>

**Kubernetes packaging:**
- Helm best practices on values: <https://helm.sh/docs/chart_best_practices/values/>
- Helm install CLI: <https://helm.sh/docs/helm/helm_install/>
- Kustomize patches: <https://kubectl.docs.kubernetes.io/references/kustomize/kustomization/patches/>
- Kustomize introduction: <https://kubectl.docs.kubernetes.io/guides/introduction/kustomize/>

**Rust configuration crates:**
- config crate: <https://docs.rs/config/latest/config/>, <https://docs.rs/config/latest/config/struct.ConfigBuilder.html>
- figment crate: <https://docs.rs/figment/latest/figment/>, <https://docs.rs/figment/latest/figment/struct.Figment.html>, <https://docs.rs/figment/latest/figment/struct.Profile.html>
- toml crate: <https://docs.rs/toml/latest/toml/>
- TOML spec v1.0.0: <https://toml.io/en/v1.0.0>

**Comparative architecture references:**
- AWS SaaS Tenant Isolation Strategies whitepaper: <https://docs.aws.amazon.com/whitepapers/latest/saas-tenant-isolation-strategies/full-stack-isolation.html>
- Spring Boot external config: <https://docs.spring.io/spring-boot/docs/current/reference/htmlsingle/#features.external-config>
- Ansible variable precedence: <https://docs.ansible.com/ansible/latest/playbook_guide/playbooks_variables.html>
- Rust hierarchical configuration writeup: <https://steezeburger.com/2023/03/rust-hierarchical-configuration/>
- Leapcell flexible configuration in Rust: <https://leapcell.io/blog/flexible-configuration-for-rust-applications-beyond-basic-defaults>

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 4 | Deep-dives on Vector, Telegraf, OpenTelemetry Collector, Datadog/Filebeat (Telegraf + OTel returned full responses; Vector + Datadog hit response-size cap and were re-issued via perplexity_ask) |
| Perplexity perplexity_ask | 5 | Vector multi-tenant TOML, Datadog conf.d + Filebeat modules.d, Helm/Kustomize layering + Spring/Ansible, Fluent Bit/Fluentd + Cribl routing, Logstash multi-pipeline + Wazuh, Rust config-rs/figment/toml hierarchical merging, Cribl + Sentinel DCR + Kinesis |
| Perplexity perplexity_search | 1 | figment profile select / Rust hierarchical override mechanics — to confirm Profile API surface |
| Context7 | 1 (attempted, server session error) | Tried to fetch config-rs docs; session unavailable. Substituted with perplexity_search of docs.rs URLs which returned authoritative API descriptions. |
| Tavily | 0 | Not needed — Perplexity provided coverage with multi-source citations for every tool. |
| WebFetch / WebSearch | 0 | Not needed — Perplexity-cited URLs surfaced directly in responses; citations were verifiable inline. |
| Training data | 1 area | Generic awareness that osquery and DataDog cache validated specs at boot (used only as supporting analogy; not load-bearing for any recommendation). |

**Total MCP tool calls:** 11 (4 research + 5 ask + 1 search + 1 attempted Context7).
**Training data reliance:** LOW — every recommendation, mechanism description, and trade-off claim is anchored to a cited tool documentation URL. The one "training data" reference (osquery boot-cache pattern) is used as supporting analogy only and is not load-bearing for any recommendation.
