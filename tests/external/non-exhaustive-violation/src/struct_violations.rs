//! Struct literal violations (E0639) for #[non_exhaustive] enforcement.
//!
//! Each function exercises one #[non_exhaustive] struct by attempting external
//! struct-literal construction. After `#[non_exhaustive]` is applied, each
//! literal MUST fail with E0639 (cannot create non-exhaustive struct expression).
//!
//! Violations 1-6, 9-12, 16-17, 20-24, 26, 32-36, 37-43, 45, 47, 49-59, 61-64, 66-76, 77-78, 80-84, 86-88, 92 (69 total E0639 in this file).
//! v51 (ScenarioEntityCatalog) is a LIVE E0639 violation — the type is public
//! (prism_dtu_common::scenario; lib.rs pub use) and #[non_exhaustive]; counted in
//! ci.yml EXPECTED=60 (ADR-036 §2.2, S-DEMO-DTU-LIVE-SCENARIO-001-A AC-014,
//! S-DEMO-DTU-LIVE-SCENARIO-001-B BPRL-P3-01 sibling sweep; gate was 52 when v51
//! was first added, subsequently bumped to 60 by S-DEMO-MULTI-TENANT-DTU-001).
//! v52 (IncidentTimeline) and v53 (IncidentStage) are LIVE E0639 violations added
//! by S-DEMO-DTU-LIVE-SCENARIO-001-B. ci.yml EXPECTED bumped from 50 to 52.
//! v61 (MultiInstanceServers) added by S-DEMO-MULTI-TENANT-DTU-001 (D-1075-API-GAP-001).
//! ci.yml EXPECTED bumped from 59 to 60.
//! v62 (StructuredErrorFields) added by S-5.02 (HIGH-3 fix).
//! ci.yml EXPECTED bumped from 60 to 61.
//! v63 (CapabilityEntry) and v64 (ResolutionStep) added by S-5.02 follow-up fix-burst
//! (CRIT-1/HIGH-1 non-exhaustive gate sibling-sweep). ci.yml EXPECTED bumped from 61 to 64
//! (together with v65 CapabilityStatus enum violation in enum_violations.rs).
//! v66 (TableNotAvailableDetails) and v67 (TableRegistry) added by S-3.13 (LOW-1 + CR-002).
//! ci.yml EXPECTED bumped from 64 to 66. Note: v65 is reserved for CapabilityStatus in
//! enum_violations.rs; S-3.13 struct violations use v66/v67 to maintain the global monotonic
//! ladder across both files with no collision.
//! v68 (Tier3CacheEntry) added by S-1.14-REDO burst-2 (MED-1-RESIDUAL — missing
//! #[non_exhaustive] on pub cache wire-format type). ci.yml EXPECTED bumped from 66 to 67.
//! v69 (InfusionUdfDescriptor) and v70 (EnrichStageDescriptor) added by S-1.14-REDO
//! fix-burst (architect-ruled FIX-IN-SCOPE — descriptor types exported to prism-query
//! lacked #[non_exhaustive]). ci.yml EXPECTED bumped from 67 to 69.
//! v71-v76 (6 prism-mcp resources types) added by S-5.03 (F-007 process-gap).
//! ci.yml EXPECTED bumped from 70 to 76.
//!   71. prism_mcp::ClientInventoryEntry  — struct, resources.rs
//!   72. prism_mcp::SensorConfigEntry     — struct, resources.rs
//!   73. prism_mcp::SensorHealthResult    — struct, resources.rs
//!   74. prism_mcp::RateLimitInfo         — struct, resources.rs
//!   75. prism_mcp::ResourcePressure      — struct, resources.rs
//!   76. prism_mcp::SensorHealthStructuredContent — struct, resources.rs
//!
//! S-SPEC-TYPE-UNIFICATION-001: Violation 30 (types::SensorSpec) removed.
//! `types::SensorSpec` was deleted (ADR-030 Approach D — unified on spec_parser::SensorSpec).
//!
//! S-5.01-FOLLOWUP-MCP-BOOT additions (prism-mcp pub API types):
//!   37. prism_mcp::safety_envelope::ResponseMeta       — struct, safety_envelope.rs
//!   38. prism_mcp::safety_envelope::ContentEntry       — struct, safety_envelope.rs
//!   39. prism_mcp::safety_envelope::StructuredContent  — struct, safety_envelope.rs
//!   40. prism_mcp::safety_envelope::ResponseEnvelope   — struct, safety_envelope.rs
//!   41. prism_mcp::safety_envelope::SafetyFlagSchema   — struct, safety_envelope.rs
//!   42. prism_mcp::safety_envelope::MetaEnvelopeSchemaType — struct, safety_envelope.rs
//!   43. prism_mcp::safety_envelope::ResponseEnvelopeSchema — struct, safety_envelope.rs
//!   45. prism_mcp::tool_registry::ToolRegistration     — struct, tool_registry.rs
//!
//! PLUGIN-MIGRATION-001-E addition:
//!   33. plugin::LoadedPlugin            — struct, plugin/loader.rs
//!
//! S-CONFIG-MULTI-TENANT-OVERRIDE-001 additions (ADR-029):
//!   34. overlay::SensorInstanceOverlay  — struct, overlay.rs
//!   35. overlay::OverlayProvenance      — struct, overlay.rs
//!   36. overlay::ResolvedSensorSpec     — struct, overlay.rs
//!
//! S-DEMO-001 additions (prism-bin pub struct types):
//!   49. prism_bin::spec_driven_adapter::BearerStaticAuthProvider — struct, spec_driven_adapter.rs
//!   50. prism_bin::spec_driven_adapter::SpecDrivenSensorAdapter  — struct, spec_driven_adapter.rs

use prism_core::{ColumnType, RiskTier, SensorId, TableType};
use prism_core::error::TableNotAvailableDetails;
use prism_spec_engine::overlay::{OverlayProvenance, ResolvedSensorSpec, SensorInstanceOverlay};
use prism_query::invalidation::WriteToolInvalidationMap;
use prism_spec_engine::infusion::{
    BuiltInSourceType, CredentialRef as InfusionCredentialRef, InfusionField, InfusionSourceConfig,
    InfusionSpec, InfusionType, PipeStageConfig, PluginConfig,
};
use prism_spec_engine::spec_parser::{
    AuthType, ColumnSpec, CredentialRef, FetchStep, RateLimitHints, SensorSpec,
    SensorTableDescriptor, TableSpec,
};
use prism_spec_engine::types::{
    ColumnDef, ColumnType as TypesColumnType, CredentialRef as TypesCredentialRef,
    SensorTableDescriptor as TypesSensorTableDescriptor,
};
use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec, WriteStep};

/// Violation 1: CredentialRef (spec_parser) struct literal (E0639).
pub fn v01_credential_ref() {
    let _cred = CredentialRef {
        name: "api_key".to_string(),
    };
    let _ = _cred;
}

/// Violation 2: SensorSpec (spec_parser) struct literal (E0639).
pub fn v02_sensor_spec() {
    let _sensor = SensorSpec {
        sensor_id: "crowdstrike".to_string(),
        name: "CrowdStrike".to_string(),
        auth_type: AuthType::Oauth2ClientCredentials,
        base_url: "https://api.crowdstrike.com".to_string(),
        tables: vec![],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };
    let _ = _sensor;
}

/// Violation 3: SensorTableDescriptor (spec_parser) struct literal (E0639).
pub fn v03_sensor_table_descriptor() {
    let _descriptor = SensorTableDescriptor {
        table_name: "crowdstrike.devices".to_string(),
        columns: vec![],
        sensor_id: "crowdstrike".to_string(),
        has_credentials: false,
    };
    let _ = _descriptor;
}

/// Violation 4: FetchStep (spec_parser) struct literal (E0639).
pub fn v04_fetch_step() {
    let _step = FetchStep {
        name: "fetch_devices".to_string(),
        method: "GET".to_string(),
        path_template: "/devices/v1".to_string(),
        body_template: None,
        response_path: "$.resources".to_string(),
        pagination_cursor_path: None,
        variables_produced: vec![],
        fan_out_batch_size: None,
        pagination: None,
    };
    let _ = _step;
}

/// Violation 5: ColumnSpec (spec_parser) struct literal (E0639).
pub fn v05_column_spec() {
    let _col = ColumnSpec {
        name: "device_id".to_string(),
        column_type: ColumnType::String,
        ocsf_field: None,
        options: vec![],
    };
    let _ = _col;
}

/// Violation 6: TableSpec (spec_parser) struct literal (E0639).
pub fn v06_table_spec() {
    let _table = TableSpec {
        table_name: "devices".to_string(),
        ocsf_class: "security_finding".to_string(),
        columns: vec![],
        steps: vec![],
        table_type: TableType::PointInTime,
        poll_interval_secs: None,
        retention_secs: None,
    };
    let _ = _table;
}

/// Violation 9: RateLimitHints (spec_parser) struct literal (E0639).
pub fn v09_rate_limit_hints() {
    let _rate_hints = RateLimitHints {
        requests_per_second: Some(10.0),
        burst_size: Some(100),
    };
    let _ = _rate_hints;
}

/// Violation 10: types::SensorTableDescriptor struct literal (E0639).
pub fn v10_types_sensor_table_descriptor() {
    let _types_descriptor = TypesSensorTableDescriptor {
        table_name: "crowdstrike.devices".to_string(),
        columns: vec![],
        steps_count: 1,
        pagination_type: prism_spec_engine::types::PaginationType::Cursor,
    };
    let _ = _types_descriptor;
}

/// Violation 11: types::CredentialRef struct literal (E0639).
pub fn v11_types_credential_ref() {
    let _types_cred = TypesCredentialRef {
        name: "api_key".to_string(),
    };
    let _ = _types_cred;
}

/// Violation 12: infusion::CredentialRef struct literal (E0639).
pub fn v12_infusion_credential_ref() {
    let _infusion_cred = InfusionCredentialRef {
        field_name: "api_key".to_string(),
        env_var: "MY_API_KEY".to_string(),
    };
    let _ = _infusion_cred;
}

/// Violation 16: WriteStep struct literal (E0639).
pub fn v16_write_step() {
    let _write_step = WriteStep {
        method: "POST".to_string(),
        url: "https://api.example.com/action".to_string(),
        body_template: None,
        response_path: None,
    };
    let _ = _write_step;
}

/// Violation 17: WriteEndpointSpec struct literal (E0639).
pub fn v17_write_endpoint_spec() {
    let _write_endpoint = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contain".to_string(),
        risk_tier: RiskTier::Reversible,
        capability_path: "crowdstrike.hosts.write".to_string(),
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        record_id_field: "device_id".to_string(),
        steps: vec![],
    };
    let _ = _write_endpoint;
}

/// Violation 20: InfusionSourceConfig struct literal (E0639).
pub fn v20_infusion_source_config() {
    let _infusion_source_config = InfusionSourceConfig {
        source_type: BuiltInSourceType::Csv,
        file_path: "/data/lookup.csv".to_string(),
        key_column: None,
        refresh_interval_secs: None,
    };
    let _ = _infusion_source_config;
}

/// Violation 21: InfusionField struct literal (E0639).
pub fn v21_infusion_field() {
    let _infusion_field = InfusionField {
        name: "geo_country".to_string(),
        input_field: "src_ip".to_string(),
        input_type: "ip".to_string(),
        output_type: "string".to_string(),
        description: None,
        source_column: None,
    };
    let _ = _infusion_field;
}

/// Violation 22: PipeStageConfig struct literal (E0639).
pub fn v22_pipe_stage_config() {
    let _pipe_stage_config = PipeStageConfig {
        adds_columns: vec!["geo_country".to_string()],
    };
    let _ = _pipe_stage_config;
}

/// Violation 23: PluginConfig struct literal (E0639).
pub fn v23_plugin_config() {
    let _plugin_config = PluginConfig {
        plugin_path: "/plugins/geo.prx".to_string(),
    };
    let _ = _plugin_config;
}

/// Violation 24: InfusionSpec struct literal (E0639).
pub fn v24_infusion_spec() {
    let _infusion_spec = InfusionSpec {
        infusion_id: "geoip".to_string(),
        name: "GeoIP".to_string(),
        infusion_type: InfusionType::LocalLookup,
        source: None,
        fields: vec![],
        pipe_stage: None,
        plugin_config: None,
        credentials: vec![],
        source_path: "/specs/geoip.infusion.toml".to_string(),
        cache_ttl_secs: None,
    };
    let _ = _infusion_spec;
}

/// Violation 26: types::ColumnDef struct literal (E0639).
pub fn v26_column_def() {
    let _column_def = ColumnDef {
        name: "device_id".to_string(),
        column_type: TypesColumnType::String,
        ocsf_field: None,
        nullable: false,
    };
    let _ = _column_def;
}

/// Violation 33: LoadedPlugin (prism_spec_engine) struct literal (E0639).
///
/// `LoadedPlugin` is the runtime handle for a compiled `.prx` plugin. It is a pub-API
/// surface type returned by `PluginRuntime::load_plugin()`. `#[non_exhaustive]` was added
/// in PLUGIN-MIGRATION-001-E (kv_store field addition) per CLAUDE.md §Conventions — external
/// crates must not struct-literal-construct it; fields may expand as the plugin runtime evolves.
///
/// Note: `LoadedPlugin` contains `wasmtime::component::Component` and
/// `wasmtime::component::InstancePre` which cannot be constructed externally. The struct
/// literal here uses `todo!()` for all fields because E0639 fires at the struct-expression
/// level (before field type-checking) once `#[non_exhaustive]` is applied.
#[allow(dead_code)]
pub fn v33_loaded_plugin() {
    let _plugin = prism_spec_engine::LoadedPlugin {
        metadata: todo!(),
        component: todo!(),
        pre_instance: todo!(),
        core_module: todo!(),
        raw_bytes: todo!(),
        allowed_urls: todo!(),
        kv_store: todo!(),
    };
    let _ = _plugin;
}

/// Violation 32: WriteToolInvalidationMap (prism_query::invalidation) struct literal (E0639).
///
/// `WriteToolInvalidationMap` is a pub-API surface type used by plugins to register write
/// tools at boot time (BC-2.16.012 / ADR-026 §D7). `#[non_exhaustive]` ensures external
/// crates cannot struct-literal-construct it — future fields (e.g., priority, rate_limit)
/// can be added without breaking downstream callers. External callers MUST use
/// `WriteToolInvalidationMap::new(...)`.
///
/// Added: post-PREREQ-E cleanup (pr-pass-4 OBS). Closes the perimeter-gate gap.
#[allow(dead_code)]
pub fn v32_write_tool_invalidation_map() {
    let _entry = WriteToolInvalidationMap {
        tool_name: "crowdstrike_contain_host".to_string(),
        source_ids: vec!["crowdstrike_hosts".to_string()],
        sensor_id: SensorId::from("crowdstrike"),
        plugin_name: "".to_string(),
    };
    let _ = _entry;
}

/// Violation 34: overlay::SensorInstanceOverlay struct literal (E0639).
///
/// `SensorInstanceOverlay` is a TOML-deserialized pub-API type for per-org overlay
/// files (ADR-029 §C). `#[non_exhaustive]` ensures external crates cannot construct
/// it via struct literal — future tunable scalar fields (e.g., `retry_policy`) may
/// be added as the multi-tenant overlay grammar evolves (BC-2.06.012).
/// External callers MUST use the TOML deserialization path.
///
/// Added: S-CONFIG-MULTI-TENANT-OVERRIDE-001 stubs.
#[allow(dead_code)]
pub fn v34_sensor_instance_overlay() {
    let _overlay = SensorInstanceOverlay {
        extends: "armis".to_string(),
        instance_id: "armis@acme".to_string(),
        base_url: Some("https://armis.acme-corp.io".to_string()),
        timeout_secs: None,
        rate_limit_hints: None,
    };
    let _ = _overlay;
}

/// Violation 35: overlay::OverlayProvenance struct literal (E0639).
///
/// `OverlayProvenance` tracks which scalar fields in a `ResolvedSensorSpec` came
/// from an overlay vs the TYPE spec (ADR-029 §D). `#[non_exhaustive]` ensures new
/// provenance fields can be added as tunable scalars expand (BC-2.06.012).
/// External callers MUST use `..Default::default()` for initialization.
///
/// Added: S-CONFIG-MULTI-TENANT-OVERRIDE-001 stubs.
#[allow(dead_code)]
pub fn v35_overlay_provenance() {
    let _provenance = OverlayProvenance {
        base_url_from_overlay: true,
        rps_from_overlay: false,
        burst_size_from_overlay: false,
        timeout_secs_from_overlay: false,
    };
    let _ = _provenance;
}

/// Violation 36: overlay::ResolvedSensorSpec struct literal (E0639).
///
/// `ResolvedSensorSpec` is the boot-time merge of a TYPE spec and per-org overlay
/// scalars (ADR-029 §E). `#[non_exhaustive]` ensures future fields (e.g., applied
/// rate limiter state, resolved credential refs) can be added without breaking
/// downstream callers (BC-2.06.014, INV-OVL-006).
/// External callers MUST use `OverlayLoader::merge_overlay_onto_type_spec(...)`.
///
/// Uses `SensorSpec::default()` for the inner `spec` field to avoid a second E0639
/// from a nested SensorSpec struct literal — only the outer `ResolvedSensorSpec`
/// literal is the target of this violation.
///
/// Added: S-CONFIG-MULTI-TENANT-OVERRIDE-001 stubs.
#[allow(dead_code)]
pub fn v36_resolved_sensor_spec() {
    use prism_core::OrgSlug;
    let _resolved = ResolvedSensorSpec {
        spec: SensorSpec::default(),
        provenance: OverlayProvenance::default(),
        // PRR-013 fix: use OrgSlug::new("acme").unwrap() instead of new_unchecked.
        // "acme" is a known-valid literal — unwrap() is safe here and avoids the
        // forbidden-pattern (OrgSlug::new_unchecked outside test-helpers feature gate).
        org_slug: OrgSlug::new("acme"),
        instance_id: "armis@acme".to_string(),
    };
    let _ = _resolved;
}

// ─── S-5.01-FOLLOWUP-MCP-BOOT: prism-mcp safety_envelope pub types ─────────────
//
// These 7 structs are the public API surface of prism-mcp's ResponseEnvelope stack.
// `#[non_exhaustive]` ensures future field additions (pagination cursors, new trust
// metadata, additional schema annotations) do not break downstream callers.
// External crates MUST construct via SafetyEnvelopeBuilder::wrap() (ResponseMeta,
// ContentEntry, StructuredContent, ResponseEnvelope) or schemars generation
// (SafetyFlagSchema, MetaEnvelopeSchemaType, ResponseEnvelopeSchema).

use prism_mcp::safety_envelope::{
    ContentEntry, MetaEnvelopeSchemaType, ResponseEnvelope, ResponseEnvelopeSchema, ResponseMeta,
    SafetyFlagSchema, StructuredContent,
};
// S-5.03 F-007: 6 new #[non_exhaustive] pub types from prism_mcp::resources
// S-5.04 F-S504-P5-002: HealthSummary added (summary_counts shape for check_sensor_health)
use prism_mcp::{
    ClientInventoryEntry, HealthSummary, RateLimitInfo, ResourcePressure, SensorConfigEntry,
    SensorHealthResult, SensorHealthStructuredContent,
};

/// Violation 37: prism_mcp::safety_envelope::ResponseMeta struct literal (E0639).
///
/// `ResponseMeta` is the `_meta` section of every MCP response envelope (BC-2.09.008).
/// `#[non_exhaustive]` ensures new metadata fields (e.g., query_duration, warning_count)
/// can be added without breaking external serialization deserializers.
/// External callers MUST construct via `SafetyEnvelopeBuilder::wrap(...)`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v37_response_meta() {
    let _meta = ResponseMeta {
        tool: "query".to_string(),
        data_source: todo!(),
        query_time: "2026-01-01T00:00:00Z".to_string(),
        trust_level: todo!(),
        safety_flags: vec![],
        total_results: 0,
        page: 1,
        has_more: false,
        next_cursor: None,
    };
    let _ = _meta;
}

/// Violation 38: prism_mcp::safety_envelope::ContentEntry struct literal (E0639).
///
/// `ContentEntry` is a prose summary line in the MCP response (BC-2.09.001).
/// `#[non_exhaustive]` ensures future content fields can be added without breaking
/// external deserializers. External callers MUST use `SafetyEnvelopeBuilder::wrap(...)`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v38_content_entry() {
    let _entry = ContentEntry {
        content_type: "text".to_string(),
        text: "3 results found".to_string(),
    };
    let _ = _entry;
}

/// Violation 39: prism_mcp::safety_envelope::StructuredContent struct literal (E0639).
///
/// `StructuredContent` wraps sensor data in the structured portion of the envelope
/// (BC-2.09.001 postcondition 3). `#[non_exhaustive]` ensures future content fields
/// can be added. External callers MUST use `SafetyEnvelopeBuilder::wrap(...)`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v39_structured_content() {
    let _content = StructuredContent {
        results: serde_json::json!([]),
    };
    let _ = _content;
}

/// Violation 40: prism_mcp::safety_envelope::ResponseEnvelope struct literal (E0639).
///
/// `ResponseEnvelope` is the full MCP response shape (BC-2.09.008). `#[non_exhaustive]`
/// ensures future top-level fields can be added without breaking external consumers.
/// External callers MUST use `SafetyEnvelopeBuilder::wrap(...)`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v40_response_envelope() {
    let _envelope = ResponseEnvelope {
        meta: todo!(),
        results: serde_json::json!([]),
        content: vec![],
        structured_content: todo!(),
    };
    let _ = _envelope;
}

/// Violation 41: prism_mcp::safety_envelope::SafetyFlagSchema struct literal (E0639).
///
/// `SafetyFlagSchema` is the schema-only type for `_meta.safety_flags` entries (BC-2.09.007).
/// `#[non_exhaustive]` ensures future flag annotation fields can be added.
/// External callers use only for outputSchema generation (read-only).
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v41_safety_flag_schema() {
    let _flag = SafetyFlagSchema {
        field: "hostname".to_string(),
        index: 0,
        pattern: "role impersonation".to_string(),
        category: "prompt_injection".to_string(),
    };
    let _ = _flag;
}

/// Violation 42: prism_mcp::safety_envelope::MetaEnvelopeSchemaType struct literal (E0639).
///
/// `MetaEnvelopeSchemaType` is the schema-only `_meta` mirror (BC-2.09.007, BC-2.09.008).
/// `#[non_exhaustive]` ensures future metadata schema fields can be added.
/// External callers use only for outputSchema generation (read-only).
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
/// IMP-10: data_source field updated from serde_json::Value to DataSource.
#[allow(dead_code)]
pub fn v42_meta_envelope_schema_type() {
    use prism_mcp::safety_envelope::DataSource;
    let _meta = MetaEnvelopeSchemaType {
        tool: "query".to_string(),
        data_source: DataSource::Single("crowdstrike".to_string()),
        query_time: "2026-01-01T00:00:00Z".to_string(),
        trust_level: "untrusted_external".to_string(),
        safety_flags: vec![],
        total_results: 0,
        page: 1,
        has_more: false,
        next_cursor: None,
    };
    let _ = _meta;
}

/// Violation 43: prism_mcp::safety_envelope::ResponseEnvelopeSchema struct literal (E0639).
///
/// `ResponseEnvelopeSchema` is the full schema-only response envelope mirror (BC-2.09.007).
/// `#[non_exhaustive]` ensures future top-level schema fields can be added.
/// External callers use only for outputSchema generation (read-only).
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v43_response_envelope_schema() {
    let _schema = ResponseEnvelopeSchema {
        meta: todo!(),
        results: vec![],
        content: vec![],
        structured_content: serde_json::json!({}),
    };
    let _ = _schema;
}

/// Violation 47: prism_security::confirmation_token::BoundingMetadata struct literal (E0639).
///
/// `BoundingMetadata` stores per-token bounding constraint signals (CRIT-1 fix,
/// confirmation_token.rs). `#[non_exhaustive]` ensures external crates cannot
/// struct-literal-construct it — future bounding fields (e.g., `row_count_estimate`,
/// `is_time_bounded`) can be added without breaking downstream callers (F-PR163-IMP-1).
/// External callers MUST use `BoundingMetadata::new(...)` or `BoundingMetadata::default()`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v47_bounding_metadata() {
    use prism_security::confirmation_token::BoundingMetadata;
    let _meta = BoundingMetadata {
        has_where_clause: true,
        has_explicit_limit: false,
        explicit_limit: None,
        dml_operation: None,
    };
    let _ = _meta;
}

/// Violation 45: prism_mcp::tool_registry::ToolRegistration struct literal (E0639).
///
/// `ToolRegistration` is the pub MCP tool definition record passed to
/// `ToolDescriptionRegistrar::register()` (BC-2.09.006). `#[non_exhaustive]` ensures
/// future fields (e.g., `capability_required`, `rate_limit`) can be added without
/// requiring external callers to update struct literals. External callers MUST use
/// `ToolRegistration::new(...)`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
#[allow(dead_code)]
pub fn v45_tool_registration() {
    let _reg = prism_mcp::tool_registry::ToolRegistration {
        name: "query".to_string(),
        description: "Execute a query.".to_string(),
        is_sensor_tool: true,
        output_schema: None,
    };
    let _ = _reg;
}

// ─── S-DEMO-001: prism-bin pub struct types ───────────────────────────────────
//
// `BearerStaticAuthProvider` and `SpecDrivenSensorAdapter` are public struct types
// in prism-bin. `#[non_exhaustive]` ensures future fields (e.g., per-adapter telemetry,
// token expiry hint) can be added without requiring external struct-literal updates.
// External callers MUST use the provided constructors (CR-006, S-DEMO-001 PR review).

/// Violation 49: prism_bin::spec_driven_adapter::BearerStaticAuthProvider struct literal (E0639).
///
/// `BearerStaticAuthProvider` is the per-fetch `AuthProvider` wrapper for `bearer_static`
/// sensors (Armis, Claroty). `#[non_exhaustive]` ensures future fields can be added without
/// breaking external callers. External callers MUST use `BearerStaticAuthProvider::new(token)`.
/// AD-017: the `token` field holds a bearer token and must never be constructed externally.
///
/// Added: S-DEMO-001 (CR-006).
#[allow(dead_code)]
pub fn v49_bearer_static_auth_provider() {
    let _provider = prism_bin::spec_driven_adapter::BearerStaticAuthProvider {
        token: "test-token".to_string(),
    };
    let _ = _provider;
}

/// Violation 50: prism_bin::spec_driven_adapter::SpecDrivenSensorAdapter struct literal (E0639).
///
/// `SpecDrivenSensorAdapter` is the spec-driven `SensorAdapter` that bridges TOML specs to
/// live Arrow data from DTU clones (GAP-002-A closure, S-DEMO-001). `#[non_exhaustive]`
/// ensures future fields (e.g., per-adapter rate-limiter handle, telemetry sink) can be
/// added without requiring external callers to update struct literals.
/// External callers MUST use `SpecDrivenSensorAdapter::new(spec, auth, client)`.
///
/// Note: uses `todo!()` for all fields because `ResolvedSensorSpec`, `AdapterAuthStrategy`,
/// and `reqwest::Client` cannot be constructed externally — E0639 fires at the
/// struct-expression level (before field type-checking) once `#[non_exhaustive]` is applied.
///
/// Added: S-DEMO-001 (CR-006).
#[allow(dead_code)]
pub fn v50_spec_driven_sensor_adapter() {
    let _adapter = prism_bin::spec_driven_adapter::SpecDrivenSensorAdapter {
        sensor_spec: todo!(),
        auth_strategy: todo!(),
        http_client: todo!(),
    };
    let _ = _adapter;
}

/// Violation 51: prism_dtu_common::scenario::ScenarioEntityCatalog struct literal (E0639).
///
/// `ScenarioEntityCatalog` is the shared entity namespace for one client's incident scenario
/// (ADR-036 §2.2, S-DEMO-DTU-LIVE-SCENARIO-001-A). `#[non_exhaustive]` ensures future
/// catalog fields (e.g., `lateral_device_ids_cs`, `lateral_device_ids_armis`) can be added
/// without breaking external match arms or struct literals.
///
/// External callers MUST use `build_scenario_entity_catalog(seed, &org_id)` — direct
/// struct literal construction MUST NOT compile (E0639).
///
/// This function constructs `ScenarioEntityCatalog` with all currently-known fields from a
/// foreign crate (this violation crate uses `features=["fixture-gen"]`). Because
/// `ScenarioEntityCatalog` is `#[non_exhaustive]`, the struct literal fails with E0639
/// (non-exhaustive type constructed outside its defining crate), which is the intended gate
/// violation counted in `ci.yml EXPECTED=60` (gate was 52 when v51 was first added by
/// S-DEMO-DTU-LIVE-SCENARIO-001-A; subsequently bumped to 60 by S-DEMO-MULTI-TENANT-DTU-001).
///
/// Added: S-DEMO-DTU-LIVE-SCENARIO-001-A (AC-014).
#[allow(dead_code)]
pub fn v51_scenario_entity_catalog() {
    // ScenarioEntityCatalog is publicly exported from prism_dtu_common::scenario
    // (lib.rs `pub use scenario::{..., ScenarioEntityCatalog}`). The struct is
    // #[non_exhaustive], so this struct literal in a foreign crate fails with E0639
    // (the correct intended violation), not E0432.
    let _catalog = prism_dtu_common::scenario::ScenarioEntityCatalog {
        org_slug: "deadbeef".to_string(),
        primary_device_id_cs: "dev-deadbeef-42-0".to_string(),
        primary_device_id_armis: "dev-deadbeef-42-0".to_string(),
        primary_hostname: "host-0".to_string(),
        lateral_device_ids_cs: vec![],
        lateral_device_ids_armis: vec![],
        ioc_ips: vec![],
        ioc_domains: vec![],
        ioc_hashes: vec![],
        device_cves: vec![],
    };
    let _ = _catalog;
}

/// Violation 52: prism_dtu_common::scenario::IncidentTimeline struct literal (E0639).
///
/// `IncidentTimeline` is the shared temporal timeline for one demo client's incident
/// scenario (ADR-036 v2.3 §2.2, S-DEMO-DTU-LIVE-SCENARIO-001-B). `#[non_exhaustive]`
/// ensures future timeline fields can be added without breaking external struct literals.
///
/// External callers MUST use `build_default_incident_timeline` — direct struct literal
/// construction MUST NOT compile (E0639).
///
/// Added: S-DEMO-DTU-LIVE-SCENARIO-001-B. ci.yml EXPECTED bumped from 50 to 52.
#[allow(dead_code)]
pub fn v52_incident_timeline() {
    let _timeline = prism_dtu_common::scenario::IncidentTimeline {
        entities: todo!(),
        stages: vec![],
        scenario_start_epoch_secs: 0,
    };
    let _ = _timeline;
}

/// Violation 53: prism_dtu_common::scenario::IncidentStage struct literal (E0639).
///
/// `IncidentStage` is a single stage entry in an `IncidentTimeline`. `#[non_exhaustive]`
/// ensures future stage fields (e.g., description text, severity level) can be added
/// without breaking external struct literals. ADR-036 v2.3 §2.2.
///
/// External callers MUST use `build_default_incident_timeline` to create stages —
/// direct struct literal construction MUST NOT compile (E0639).
///
/// Added: S-DEMO-DTU-LIVE-SCENARIO-001-B. ci.yml EXPECTED bumped from 50 to 52.
#[allow(dead_code)]
pub fn v53_incident_stage() {
    let _stage = prism_dtu_common::scenario::IncidentStage {
        name: "Baseline",
        activates_after_secs: 0,
        visible_entity_mask: todo!(),
    };
    let _ = _stage;
}

/// Violation 54: prism_dtu_demo_server::multi_instance::MultiInstanceConfig struct literal (E0639).
///
/// `MultiInstanceConfig` is the multi-instance bind configuration for
/// `prism-dtu-demo-server` (BC-2.06.017 Postcondition 1). `#[non_exhaustive]`
/// ensures future fields (e.g., bind_timeout, tls_config) can be added without
/// breaking external struct literals. External callers MUST use `MultiInstanceConfig::new(...)`.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (U-006). ci.yml EXPECTED bumped from 52 to 60.
/// (52 base + 7 E0639 struct violations [v54–v59: MultiInstanceConfig, InstanceEntry,
/// DemoBindError, MultiInstanceHarness, HarnessEntry, BindError; v61: MultiInstanceServers]
/// + 1 E0004 enum violation [v60: MultiInstanceBindError in enum_violations.rs] = 60 total.)
#[allow(dead_code)]
pub fn v54_multi_instance_config() {
    let _cfg = prism_dtu_demo_server::MultiInstanceConfig {
        instances: vec![],
    };
    let _ = _cfg;
}

/// Violation 55: prism_dtu_demo_server::multi_instance::InstanceEntry struct literal (E0639).
///
/// `InstanceEntry` is a single named DTU clone bind entry within a `MultiInstanceConfig`
/// (BC-2.06.017 Postcondition 1). `#[non_exhaustive]` ensures future fields (e.g.,
/// per-instance TLS config) can be added without breaking external struct literals.
/// External callers MUST use `InstanceEntry::new(name, bind)`.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (U-006).
#[allow(dead_code)]
pub fn v55_instance_entry() {
    let _entry = prism_dtu_demo_server::InstanceEntry {
        name: "test".to_string(),
        bind: "127.0.0.1:0".parse().unwrap(),
    };
    let _ = _entry;
}

/// Violation 56: prism_dtu_demo_server::multi_instance::DemoBindError struct literal (E0639).
///
/// `DemoBindError` is a single bind failure within `MultiInstanceBindError::BindFailure`
/// (BC-2.06.017 Postcondition 6). `#[non_exhaustive]` ensures future diagnostic fields
/// can be added. External callers construct instances via the error-path of `start_instances`.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (U-006).
#[allow(dead_code)]
pub fn v56_demo_bind_error() {
    let _err = prism_dtu_demo_server::DemoBindError {
        instance_name: "test".to_string(),
        source: std::io::Error::other("test"),
    };
    let _ = _err;
}

/// Violation 57: prism_dtu_harness::multi_instance::MultiInstanceHarness struct literal (E0639).
///
/// `MultiInstanceHarness` is the multi-instance test harness managing N DTU clone instances
/// at distinct socket addresses (BC-2.06.017 Postcondition 2). `#[non_exhaustive]` ensures
/// future fields can be added. External callers MUST use `MultiInstanceHarness::start(entries)`.
///
/// Struct fields are private; the struct literal below triggers BOTH E0616 (private fields)
/// and E0639 (#[non_exhaustive] — cannot create non-exhaustive struct expression).
/// The CI gate counts E0639; both errors fire when a non-exhaustive struct with private
/// fields is constructed via struct literal from an external crate.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (U-006).
#[allow(dead_code)]
pub fn v57_multi_instance_harness() {
    // Triggers E0639 (#[non_exhaustive]) + E0616 (private fields socket_map, shutdown_tx, task_handles).
    let _h = prism_dtu_harness::MultiInstanceHarness {
        socket_map: todo!(),
        shutdown_tx: todo!(),
        task_handles: todo!(),
    };
    let _ = _h;
}

/// Violation 58: prism_dtu_harness::multi_instance::HarnessEntry struct literal (E0639).
///
/// `HarnessEntry` pairs an `(org_slug, sensor_id)` identity with a `Box<dyn BehavioralClone>`
/// (BC-2.06.017 Postcondition 2). `#[non_exhaustive]` ensures future fields (e.g., bind_addr)
/// can be added. External callers MUST use `HarnessEntry::new(org_slug, sensor_id, clone)`.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (U-006).
#[allow(dead_code)]
pub fn v58_harness_entry() {
    use prism_dtu_harness::HarnessEntry;
    use prism_dtu_common::BehavioralClone;
    let _entry = HarnessEntry {
        org_slug: "acme".to_string(),
        sensor_id: "armis".to_string(),
        clone: todo!() as Box<dyn BehavioralClone>,
    };
    let _ = _entry;
}

/// Violation 59: prism_dtu_harness::error::BindError struct literal (E0639).
///
/// `BindError` is a single clone bind failure within `HarnessError::BindFailure`
/// (BC-2.06.017 Postcondition 6). `#[non_exhaustive]` ensures future diagnostic fields
/// (e.g., instance address, retry count) can be added without breaking external consumers.
/// External callers receive `BindError` via the error-path of `MultiInstanceHarness::start`.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (U-006).
#[allow(dead_code)]
pub fn v59_bind_error() {
    use prism_dtu_harness::BindError;
    let _err = BindError {
        org_slug: "acme".to_string(),
        sensor_id: "armis".to_string(),
        source: std::io::Error::other("test"),
    };
    let _ = _err;
}

/// Violation 61 (60th type): prism_dtu_demo_server::multi_instance::MultiInstanceServers
/// struct literal (E0639).
///
/// `MultiInstanceServers` is the lifecycle handle returned by `start_instances`, owning
/// the shared `shutdown_tx: broadcast::Sender<()>` and all N watcher task handles
/// (BC-2.06.017 Postcondition 1 v1.2, D-1075-API-GAP-001). `#[non_exhaustive]` ensures
/// future fields (e.g., per-instance health monitors, metrics handles) can be added
/// without breaking external struct literals. Fields are private; external callers
/// MUST use `start_instances(cfg, factory).await` — direct struct literal construction
/// MUST NOT compile (E0639 + E0616 for private fields).
///
/// Struct fields are private; the struct literal below triggers BOTH E0616 (private fields)
/// and E0639 (#[non_exhaustive]). The CI gate counts E0639.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (D-1075-API-GAP-001). ci.yml EXPECTED bumped
/// from 59 to 60.
#[allow(dead_code)]
pub fn v61_multi_instance_servers() {
    // Triggers E0639 (#[non_exhaustive]) + E0616 (private fields socket_map, shutdown_tx, task_handles).
    let _servers = prism_dtu_demo_server::MultiInstanceServers {
        socket_map: todo!(),
        shutdown_tx: todo!(),
        task_handles: todo!(),
    };
    let _ = _servers;
}

/// Violation 62: StructuredErrorFields (prism-mcp error_mapping) struct literal (E0639).
///
/// BC-2.10.007 / S-5.02 HC-3: `StructuredErrorFields` carries `#[non_exhaustive]`
/// so that adding a field (e.g. a future `request_id` or `trace_id`) is backward-compatible.
/// External callers MUST use `StructuredErrorFields::new(...)` (the 9-argument constructor).
///
/// Added: S-5.02 (HIGH-3 fix). ci.yml EXPECTED bumped from 60 to 61.
#[allow(dead_code)]
pub fn v62_structured_error_fields() {
    // Triggers E0639 (#[non_exhaustive]).
    let _fields = prism_mcp::error_mapping::StructuredErrorFields {
        code: "E-MCP-001".to_string(),
        message: "invalid".to_string(),
        category: "validation".to_string(),
        retryable: false,
        retry_after_seconds: None,
        suggestion: "Fix it".to_string(),
        source: "prism_mcp".to_string(),
        original_params_valid: false,
        upstream_message: None,
    };
    let _ = _fields;
}

/// Violation 63: prism_mcp::CapabilityEntry struct literal (E0639).
///
/// `CapabilityEntry` is the per-capability-path response entry in the `list_capabilities`
/// tool response (BC-2.10.011, S-5.02 R4). `#[non_exhaustive]` ensures that future
/// fields (e.g., `last_evaluated_at`, `policy_source`) can be added without breaking
/// external consumers. External callers MUST NOT construct this directly — it is produced
/// exclusively by `PrismServer::list_capabilities`.
///
/// Added: S-5.02 follow-up fix-burst (CRIT-1/HIGH-1 non-exhaustive gate sibling-sweep).
/// ci.yml EXPECTED bumped from 61 to 64 (together with v64 ResolutionStep + v65 CapabilityStatus).
#[allow(dead_code)]
pub fn v63_capability_entry() {
    // Triggers E0639 (#[non_exhaustive]).
    let _entry = prism_mcp::CapabilityEntry {
        status: todo!(),
        resolution_chain: vec![],
    };
    let _ = _entry;
}

/// Violation 64: prism_mcp::ResolutionStep struct literal (E0639).
///
/// `ResolutionStep` is a single tier step in the capability resolution chain returned
/// by `list_capabilities` (BC-2.10.011, S-5.02 R4). `#[non_exhaustive]` ensures
/// that new step fields (e.g., `timestamp`, `policy_id`) can be added without breaking
/// external consumers. External callers MUST NOT construct this directly.
///
/// Added: S-5.02 follow-up fix-burst (CRIT-1/HIGH-1 non-exhaustive gate sibling-sweep).
#[allow(dead_code)]
pub fn v64_resolution_step() {
    // Triggers E0639 (#[non_exhaustive]).
    let _step = prism_mcp::ResolutionStep {
        level: "compile_tier".to_string(),
        result: "permit".to_string(),
        source: "WriteEndpointRegistry".to_string(),
    };
    let _ = _step;
}

/// Violation 66: prism_core::error::TableNotAvailableDetails struct literal (E0639).
///
/// Added: S-3.13 (LOW-1 — #[non_exhaustive] on `TableNotAvailableDetails`).
/// ci.yml EXPECTED bumped from 65 to 66 (v65 is CapabilityStatus enum violation in
/// enum_violations.rs; S-3.13 struct violations use v66/v67 on the global ladder).
/// Callers outside prism-core MUST use `TableNotAvailableDetails::new(...)` instead
/// of struct literal construction.
#[allow(dead_code)]
pub fn v66_table_not_available_details() {
    // Triggers E0639 (#[non_exhaustive]).
    let _details = TableNotAvailableDetails {
        table: "crowdstrike_alerts".to_string(),
        sensor: "crowdstrike".to_string(),
        available_sensors: "armis".to_string(),
        available_tables: "armis_alerts".to_string(),
        did_you_mean: "".to_string(),
    };
    let _ = _details;
}

/// Violation 67: prism_query::table_registry::TableRegistry struct literal (E0639).
///
/// `TableRegistry` is a new pub struct in `prism-query` that manages the dynamic
/// catalog of available sensor tables. `#[non_exhaustive]` ensures external crates
/// cannot use struct-literal construction — callers MUST use `TableRegistry::new()`
/// or `TableRegistry::from_snapshot()`. Future fields (e.g., metrics counters,
/// TTL expiry timestamps) can be added without breaking external callers.
///
/// Added: S-3.13 (CR-002 — `#[non_exhaustive]` on new pub prism-query types).
/// S-3.13 bumped EXPECTED from 64 to 66 jointly: v66 (TableNotAvailableDetails, LOW-1)
/// and v67 (TableRegistry, CR-002) are both new #[non_exhaustive] types (+1 each).
#[allow(dead_code)]
pub fn v67_table_registry() {
    // Triggers E0639 (#[non_exhaustive]).
    // Note: TableRegistry fields are private; the E0639 check fires before E0603.
    let _reg = prism_query::table_registry::TableRegistry {};
    let _ = _reg;
}

/// Violation 68: prism_spec_engine::infusion::cache::Tier3CacheEntry struct literal (E0639).
///
/// `Tier3CacheEntry` is the wire-format struct for the Tier-3 RocksDB infusion cache.
/// `#[non_exhaustive]` ensures external crates cannot use struct-literal construction
/// — callers must rely on `InfusionTier3Cache::set()` to write entries. Future fields
/// (e.g., compression flag, schema_version, source_ttl_hint) can be added without
/// breaking external callers.
///
/// Added: S-1.14-REDO burst-2 (MED-1-RESIDUAL).
/// ci.yml EXPECTED bumped from 66 to 67.
#[allow(dead_code)]
pub fn v68_tier3_cache_entry() {
    // Triggers E0639 (#[non_exhaustive]).
    let _entry = prism_spec_engine::infusion::cache::Tier3CacheEntry {
        value_json: None,
        expiry_unix_secs: 0u64,
    };
    let _ = _entry;
}

/// Violation 69: prism_spec_engine::InfusionUdfDescriptor struct literal (E0639).
///
/// `InfusionUdfDescriptor` is the exported UDF descriptor produced by
/// `InfusionRegistry` and consumed by prism-query to register DataFusion
/// scalar UDFs (BC-2.19.001). `#[non_exhaustive]` was added in S-1.14-REDO
/// fix-burst because the story added `cache_ttl_secs` as a new field, which
/// was a cross-crate breaking change without the attribute. Future fields
/// (e.g., per-UDF rate-limit hints, description string) can now be added
/// without breaking external callers. External callers MUST use
/// `InfusionUdfDescriptor::new(...)`.
///
/// Added: S-1.14-REDO fix-burst (architect-ruled FIX-IN-SCOPE, together with v70).
/// ci.yml EXPECTED bumped from 67 to 69 (both v69 and v70 added in the same burst).
#[allow(dead_code)]
pub fn v69_infusion_udf_descriptor() {
    use prism_spec_engine::{InfusionSource, InfusionUdfDescriptor};
    use std::sync::Arc;
    // Triggers E0639 (#[non_exhaustive]).
    let _descriptor = InfusionUdfDescriptor {
        name: "geoip_country".to_string(),
        input_type: "ip".to_string(),
        output_type: "string".to_string(),
        infusion_id: "geoip".to_string(),
        source: todo!() as Arc<dyn InfusionSource>,
        source_column: None,
        cache_ttl_secs: 3600,
    };
    let _ = _descriptor;
}

/// Violation 70: prism_spec_engine::EnrichStageDescriptor struct literal (E0639).
///
/// `EnrichStageDescriptor` is the exported descriptor for the `| enrich` PrismQL
/// pipe stage, produced by `InfusionRegistry::enrich_descriptor` and consumed by
/// prism-query (BC-2.19.001 / AC-3). `#[non_exhaustive]` ensures that future
/// fields (e.g., per-stage timeout hints, batch-size metadata) can be added
/// without a semver-breaking change. External callers MUST use
/// `EnrichStageDescriptor::new(...)`.
///
/// Added: S-1.14-REDO fix-burst (architect-ruled FIX-IN-SCOPE, together with v69).
/// ci.yml EXPECTED bumped from 67 to 69 (both v69 and v70 added in the same burst).
#[allow(dead_code)]
pub fn v70_enrich_stage_descriptor() {
    use prism_spec_engine::EnrichStageDescriptor;
    // Triggers E0639 (#[non_exhaustive]).
    let _descriptor = EnrichStageDescriptor {
        infusion_name: "geoip".to_string(),
        input_field: "src_ip".to_string(),
        output_columns: vec![],
        infusion_id: "geoip".to_string(),
    };
    let _ = _descriptor;
}

// ─── S-5.03 F-007: prism-mcp resources pub types ────────────────────────────
//
// These 6 structs are the public API surface of prism-mcp's resource handlers.
// `#[non_exhaustive]` ensures future field additions can be made without
// breaking downstream callers.

/// Violation 71: prism_mcp::ClientInventoryEntry struct literal (E0639).
///
/// `ClientInventoryEntry` is the per-client summary row returned by the
/// `prism://config/clients` resource (BC-2.10.008). `#[non_exhaustive]` ensures
/// future fields (e.g., `last_reload_at`, `error_count`) can be added without
/// breaking external struct-literal construction.
///
/// Added: S-5.03 (F-007 process-gap — register new prism-mcp pub types).
/// ci.yml EXPECTED bumped from 70 to 76 (6 new types, v71-v76).
#[allow(dead_code)]
pub fn v71_client_inventory_entry() {
    // Triggers E0639 (#[non_exhaustive]).
    let _entry = ClientInventoryEntry {
        client_id: "acme".to_string(),
        display_name: None,
        sensor_count: 3,
        enabled_sensors: vec![],
    };
    let _ = _entry;
}

/// Violation 72: prism_mcp::SensorConfigEntry struct literal (E0639).
///
/// `SensorConfigEntry` is the per-sensor config row returned by the
/// `prism://config/clients/{client_id}/sensors` resource (BC-2.10.008 postcondition 2).
/// `#[non_exhaustive]` ensures `api_base_url` and future fields can be added without
/// breaking external callers (VP-050: URL redaction invariant).
///
/// Added: S-5.03 (F-007 process-gap).
#[allow(dead_code)]
pub fn v72_sensor_config_entry() {
    // Triggers E0639 (#[non_exhaustive]).
    let _entry = SensorConfigEntry {
        sensor_type: "crowdstrike".to_string(),
        status: "active".to_string(),
        credential_ref: "cs-api-key".to_string(),
        sources: vec![],
        api_base_url: "https://api.crowdstrike.com".to_string(),
    };
    let _ = _entry;
}

/// Violation 73: prism_mcp::SensorHealthResult struct literal (E0639).
///
/// `SensorHealthResult` holds per-sensor health status for the `check_sensor_health`
/// tool (BC-2.08.005). `#[non_exhaustive]` ensures S-5.04 live-probe fields can be
/// added without breaking external callers that only pattern-match on spec-only fields.
///
/// Added: S-5.03 (F-007 process-gap).
#[allow(dead_code)]
pub fn v73_sensor_health_result() {
    // Triggers E0639 (#[non_exhaustive]).
    let _result = SensorHealthResult {
        sensor_id: "crowdstrike".to_string(),
        client_id: "acme".to_string(),
        probe_level: "spec-only".to_string(),
        reachable: None,
        auth_valid: None,
        rate_limit: None,
        last_successful_query_at: None,
        error: None,
    };
    let _ = _result;
}

/// Violation 74: prism_mcp::RateLimitInfo struct literal (E0639).
///
/// `RateLimitInfo` holds per-sensor rate-limit state within `SensorHealthResult`
/// (BC-2.08.005 postcondition). `#[non_exhaustive]` ensures new rate-limit fields
/// (e.g., `retry_after`, `burst_remaining`) can be added without breaking callers.
///
/// Added: S-5.03 (F-007 process-gap).
#[allow(dead_code)]
pub fn v74_rate_limit_info() {
    // Triggers E0639 (#[non_exhaustive]).
    let _info = RateLimitInfo {
        remaining: None,
        limit: None,
        reset_at: None,
    };
    let _ = _info;
}

/// Violation 75: prism_mcp::ResourcePressure struct literal (E0639).
///
/// `ResourcePressure` holds the active cursor count and token count for the
/// `check_sensor_health` response (BC-2.08.005 postcondition). `#[non_exhaustive]`
/// ensures future pressure metrics (e.g., `pending_writes`) can be added.
///
/// Added: S-5.03 (F-007 process-gap).
#[allow(dead_code)]
pub fn v75_resource_pressure() {
    // Triggers E0639 (#[non_exhaustive]).
    // Fields are Option<usize>; use None to match the production contract
    // (BC-2.08.005: hardcoded 0 is FORBIDDEN in S-5.03 scope).
    let _pressure = ResourcePressure {
        active_cursor_count: None,
        active_token_count: None,
    };
    let _ = _pressure;
}

/// Violation 76: prism_mcp::SensorHealthStructuredContent struct literal (E0639).
///
/// `SensorHealthStructuredContent` is the top-level `structuredContent` shape for
/// the `check_sensor_health` tool response (BC-2.08.005). `#[non_exhaustive]`
/// ensures future fields (e.g., `query_ids`) can be added without breaking callers.
///
/// Added: S-5.03 (F-007 process-gap).
#[allow(dead_code)]
pub fn v76_sensor_health_structured_content() {
    // Triggers E0639 (#[non_exhaustive]).
    let _content = SensorHealthStructuredContent {
        sensors: vec![],
        resource_pressure: ResourcePressure::new(None, None),
        trust_level: "internal".to_string(),
        summary: "0 sensors".to_string(),
    };
    let _ = _content;
}

// ─── S-DEMO-ENRICHMENT-PIVOT-002 v1.3: http_lookup infusion types ─────────────
//
// These 2 structs are the configuration API surface for http_lookup-type infusions
// (ADR-040 v2.0 D8.2). `#[non_exhaustive]` ensures new configuration fields can be
// added without breaking downstream callers. Renumbered v77-v78 (S-5.03 used v71-v76).

/// Violation 77: prism_spec_engine::infusion::HttpLookupCredentialConfig struct literal (E0639).
///
/// `HttpLookupCredentialConfig` is the credential configuration for http_lookup-type
/// infusions (ADR-040 v2.0 D8.2). `#[non_exhaustive]` ensures external crates cannot
/// use struct-literal construction — callers MUST use `HttpLookupCredentialConfig::new()`.
/// Future fields (e.g., token_refresh_url, rotation_policy) can be added without breaking
/// external callers.
///
/// Added: S-DEMO-ENRICHMENT-PIVOT-002 v1.3. Renumbered v77 (was v71 pre-rebase;
/// S-5.03 claimed v71-v76 on develop@85ac7b06).
#[allow(dead_code)]
pub fn v77_http_lookup_credential_config() {
    use prism_spec_engine::infusion::HttpLookupAuthType;
    // Triggers E0639 (#[non_exhaustive]).
    let _cred = prism_spec_engine::infusion::HttpLookupCredentialConfig {
        ref_name: "nvd.api_key".to_string(),
        env_var: "PRISM_NVD_API_KEY".to_string(),
        auth: HttpLookupAuthType::BearerHeader,
    };
    let _ = _cred;
}

/// Violation 78: prism_spec_engine::infusion::HttpLookupConfig struct literal (E0639).
///
/// `HttpLookupConfig` is the HTTP endpoint configuration for http_lookup-type infusions
/// (ADR-040 v2.0 D8.2). `#[non_exhaustive]` ensures external crates cannot use
/// struct-literal construction — callers MUST use `HttpLookupConfig::new()`.
/// Future fields (e.g., timeout_secs, retry_count, custom_headers) can be added without
/// breaking external callers.
///
/// Added: S-DEMO-ENRICHMENT-PIVOT-002 v1.3. Renumbered v78 (was v72 pre-rebase;
/// S-5.03 claimed v71-v76 on develop@85ac7b06).
#[allow(dead_code)]
pub fn v78_http_lookup_config() {
    // Triggers E0639 (#[non_exhaustive]).
    let _cfg = prism_spec_engine::infusion::HttpLookupConfig {
        base_url: "https://services.nvd.nist.gov".to_string(),
        url_template: "/rest/json/cves/2.0?cveId=${input}".to_string(),
        method: "GET".to_string(),
        response_path: "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData".to_string(),
        credential: None,
    };
    let _ = _cfg;
}

// ─── S-DEMO-PRISMQL-ONBOARDING-001-A: prism_describe response types ──────────
//
// These 3 structs are the public response API for the `prism_describe` L2 schema
// discovery tool (BC-2.10.012). `#[non_exhaustive]` ensures new fields (e.g., table
// metadata, column constraints, deprecation info) can be added without breaking
// downstream callers. ci.yml EXPECTED bumped from 79 to 82 (+3).
//   80. prism_mcp::PrismDescribeResponse — top-level prism_describe response
//   81. prism_mcp::TableDescriptor       — per-table descriptor
//   82. prism_mcp::ColumnDescriptor      — per-column descriptor

/// Violation 80: prism_mcp::PrismDescribeResponse struct literal (E0639).
///
/// `PrismDescribeResponse` is the top-level response from `prism_describe` (BC-2.10.012).
/// `#[non_exhaustive]` ensures external crates cannot use struct-literal construction —
/// future fields (e.g., schema_version, last_updated_at) can be added without breaking
/// external callers.
///
/// Added: S-DEMO-PRISMQL-ONBOARDING-001-A (BC-2.10.012). ci.yml EXPECTED bumped 79→82.
#[allow(dead_code)]
pub fn v80_prism_describe_response() {
    // Triggers E0639 (#[non_exhaustive]).
    let _resp = prism_mcp::PrismDescribeResponse {
        client_id: "acme".to_string(),
        tables: vec![],
        pql_hints: vec![],
    };
    let _ = _resp;
}

/// Violation 81: prism_mcp::TableDescriptor struct literal (E0639).
///
/// `TableDescriptor` describes a single sensor table within a `PrismDescribeResponse`
/// (BC-2.10.012). `#[non_exhaustive]` ensures external crates cannot use struct-literal
/// construction — future fields (e.g., row_count_estimate, freshness_ttl) can be added
/// without breaking external callers.
///
/// Added: S-DEMO-PRISMQL-ONBOARDING-001-A (BC-2.10.012). ci.yml EXPECTED bumped 79→82.
#[allow(dead_code)]
pub fn v81_table_descriptor() {
    // Triggers E0639 (#[non_exhaustive]).
    let _td = prism_mcp::TableDescriptor {
        name: "crowdstrike.alerts".to_string(),
        sensor_type: "crowdstrike".to_string(),
        description: String::new(),
        columns: vec![],
        example_query: String::new(),
    };
    let _ = _td;
}

/// Violation 82: prism_mcp::ColumnDescriptor struct literal (E0639).
///
/// `ColumnDescriptor` describes a single column within a `TableDescriptor`
/// (BC-2.10.012). `#[non_exhaustive]` ensures external crates cannot use struct-literal
/// construction — future fields (e.g., is_indexed, ocsf_field_path) can be added
/// without breaking external callers.
///
/// Uses `prism_core::column::ColumnType` (ADR-024 canonical sensor-schema enum;
/// variants: String/Integer/Float/Boolean/Datetime/Json). Do NOT use
/// `prism_core::types::ColumnType` (internal table schemas).
///
/// Added: S-DEMO-PRISMQL-ONBOARDING-001-A (BC-2.10.012). ci.yml EXPECTED bumped 79→82.
#[allow(dead_code)]
pub fn v82_column_descriptor() {
    // Triggers E0639 (#[non_exhaustive]).
    let _cd = prism_mcp::ColumnDescriptor {
        name: "severity".to_string(),
        col_type: prism_core::column::ColumnType::String,
        description: None,
        nullable: true,
    };
    let _ = _cd;
}

/// Violation 83: prism_core::error::ColumnNotFoundDetails struct literal (E0639).
///
/// `ColumnNotFoundDetails` carries the E-QUERY-038 column-not-found error context:
/// the missing column name, table, client, available columns, and an optional
/// did-you-mean suggestion (strsim levenshtein).  `#[non_exhaustive]` ensures future
/// fields (e.g., `column_type`, `query_position`) can be added without breaking
/// callers that pattern-match or construct via struct literal.
/// External callers MUST use `ColumnNotFoundDetails::new(...)`.
///
/// Added: S-DEMO-PRISMQL-ONBOARDING-001-B (BC-2.11.016 / E-QUERY-038).
#[allow(dead_code)]
pub fn v83_column_not_found_details() {
    // Triggers E0639 (#[non_exhaustive]).
    let _details = prism_core::error::ColumnNotFoundDetails {
        column: "severity".to_string(),
        table: "crowdstrike_alerts".to_string(),
        client_id: "acme-corp".to_string(),
        available_columns: vec![],
        did_you_mean: None,
    };
    let _ = _details;
}

// ─── S-5.04 F-S504-P5-002: HealthSummary (summary_counts shape) ──────────────
//
// `HealthSummary` is the structured `summary_counts` object embedded in
// `SensorHealthStructuredContent` (BC-2.08.007 postcondition). It is the
// wire-format shape for `healthy_count`, `unhealthy_count`, `total_count`, and
// `rate_limited_count` returned in the `check_sensor_health` response.
// `#[non_exhaustive]` ensures future summary fields can be added without
// breaking downstream callers. ci.yml EXPECTED bumped from 83 to 84.

/// Violation 84: prism_mcp::HealthSummary struct literal (E0639).
///
/// `HealthSummary` is the `summary_counts` shape in `SensorHealthStructuredContent`
/// (BC-2.08.007). `#[non_exhaustive]` ensures external callers cannot use
/// struct-literal construction — callers MUST use `HealthSummary::from_results(...)`.
/// Future fields (e.g., auth_invalid_count, degraded_count) can be added without
/// breaking downstream callers.
///
/// Added: S-5.04 (F-S504-P5-002 fix — BC-2.08.007 summary_counts postcondition).
#[allow(dead_code)]
pub fn v84_health_summary() {
    // Triggers E0639 (#[non_exhaustive]).
    let _summary = HealthSummary {
        healthy_count: 0,
        unhealthy_count: 2,
        total_count: 2,
        rate_limited_count: 2,
    };
    let _ = _summary;
}

// ─── S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: SqlPipeQuery ────────────────────
//
// `SqlPipeQuery` is the SQL→Pipe composition AST node (BC-2.11.020, ADR-043).
// `#[non_exhaustive]` ensures future fields (e.g., `with` CTEs, query-level hints)
// can be added without breaking external struct literals.
// ci.yml EXPECTED bumped from 85 to 86.

/// Violation 86: prism_query::ast::SqlPipeQuery struct literal (E0639).
///
/// `SqlPipeQuery` holds the SQL head and pipe stages for `SELECT … | stage …`
/// composition queries (BC-2.11.020 / ADR-043). `#[non_exhaustive]` prevents
/// external crates from constructing it via struct literal — callers must use
/// the parser. Future fields (e.g., `with`, `hints`) can be added safely.
///
/// Added: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (AC-001).
#[allow(dead_code)]
pub fn v86_sql_pipe_query() {
    use prism_query::ast::{FromClause, SelectClause, SourceRef, SqlPipeQuery, SqlQuery};
    // Triggers E0639 (#[non_exhaustive]).
    let _spq = SqlPipeQuery {
        head: SqlQuery::new(
            SelectClause::new(vec![]),
            FromClause::new(SourceRef::from_raw("t")),
        ),
        stages: vec![],
    };
    let _ = _spq;
}

// ─── S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 AC-021: UnknownSourceTableDetails ──
//
// `UnknownSourceTableDetails` is the inner-boxed fields struct for
// `PrismError::UnknownSourceTable` (E-QUERY-036). It carries `available_tables`
// and `did_you_mean` for actionable diagnostics (AC-021 / GRAMMAR-004).
// `#[non_exhaustive]` ensures future fields can be added without breaking
// callers that construct or pattern-match. ci.yml EXPECTED bumped from 86 to 87.

/// Violation 87: prism_core::error::UnknownSourceTableDetails struct literal (E0639).
///
/// `UnknownSourceTableDetails` carries the E-QUERY-036 error context: the
/// source name, available sensor IDs, and an optional Levenshtein suggestion.
/// `#[non_exhaustive]` ensures external callers cannot use struct-literal
/// construction — callers MUST use `UnknownSourceTableDetails::new(...)`.
/// Future fields (e.g., `available_prefixes`, `query_position`) can be added
/// without breaking downstream callers.
///
/// Added: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 AC-021 (GRAMMAR-004 E-QUERY-036).
#[allow(dead_code)]
pub fn v87_unknown_source_table_details() {
    use prism_core::error::UnknownSourceTableDetails;
    // Triggers E0639 (#[non_exhaustive]).
    let _details = UnknownSourceTableDetails {
        source_name: "ghost.table".to_string(),
        available_tables: vec![],
        did_you_mean: None,
    };
    let _ = _details;
}

// ─── S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B: EnrichUdfNotFoundDetails ─────────
//
// `EnrichUdfNotFoundDetails` is the inner-boxed fields struct for
// `PrismError::EnrichUdfNotFound` (E-QUERY-039). It carries the requested
// infusion name, available infusion names, and an optional Levenshtein suggestion
// for actionable diagnostics. `#[non_exhaustive]` ensures future fields (e.g.,
// `column_context`, `query_position`) can be added without breaking callers.
// ci.yml EXPECTED bumped from 87 to 88.

/// Violation 88: prism_core::error::EnrichUdfNotFoundDetails struct literal (E0639).
///
/// `EnrichUdfNotFoundDetails` carries the E-QUERY-039 error context: the requested
/// enrichment UDF name, available infusion names, and an optional Levenshtein
/// suggestion. `#[non_exhaustive]` ensures external callers cannot use struct-literal
/// construction — callers MUST use `EnrichUdfNotFoundDetails::new(...)`.
/// Future fields (e.g., `column_context`, `query_position`) can be added
/// without breaking downstream callers.
///
/// Added: S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B (E-QUERY-039 enrich-UDF gate).
#[allow(dead_code)]
pub fn v88_enrich_udf_not_found_details() {
    use prism_core::error::EnrichUdfNotFoundDetails;
    // Triggers E0639 (#[non_exhaustive]).
    let _details = EnrichUdfNotFoundDetails {
        infusion: "threat_intel".to_string(),
        available_infusions: vec![],
        did_you_mean: None,
    };
    let _ = _details;
}

// ─── DEFECT-PQL-FNCALL-LHS-001 BC-2.11.019 v1.26 §OBS-005: ParseError ─────────
//
// `ParseError` is the pub-API surface type for PrismQL parse failures returned
// by `parse_query()` and `parse_sql_with_limits()`. `#[non_exhaustive]` ensures
// that future fields (e.g., `recovery_suggestion`, `severity`, `related_spans`)
// can be added without requiring external callers to update struct literals.
// ci.yml EXPECTED bumped from 91 to 92.

/// Violation 92: prism_query::error::ParseError struct literal (E0639).
///
/// `ParseError` is the canonical PrismQL parse error type (prism-query pub API).
/// `#[non_exhaustive]` was added by DEFECT-PQL-FNCALL-LHS-001 fix-burst-42
/// (BC-2.11.019 v1.26 §OBS-005). External callers MUST use `ParseError::new(...)`
/// or `ParseError::semantic(...)` — direct struct literal construction MUST NOT
/// compile (E0639). Future fields (e.g., `recovery_suggestion`, `related_spans`)
/// can be added without breaking downstream callers.
///
/// Added: DEFECT-PQL-FNCALL-LHS-001 fix-burst-42 (BC-2.11.019 v1.26 §OBS-005).
/// ci.yml EXPECTED bumped from 91 to 92.
#[allow(dead_code)]
pub fn v92_parse_error() {
    use prism_query::error::ParseError;
    // Triggers E0639 (#[non_exhaustive]).
    let _err = ParseError {
        offset: 0,
        message: "E-QUERY-003: test".to_string(),
        recovery_label: None,
        semantic: true,
    };
    let _ = _err;
}
