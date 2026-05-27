//! Struct literal violations (E0639) for #[non_exhaustive] enforcement.
//!
//! Each function exercises one #[non_exhaustive] struct by attempting external
//! struct-literal construction. After `#[non_exhaustive]` is applied, each
//! literal MUST fail with E0639 (cannot create non-exhaustive struct expression).
//!
//! Violations 1-6, 9-12, 16-17, 20-24, 26, 32-36 (23 total E0639 expected).
//!
//! S-SPEC-TYPE-UNIFICATION-001: Violation 30 (types::SensorSpec) removed.
//! `types::SensorSpec` was deleted (ADR-030 Approach D — unified on spec_parser::SensorSpec).
//!
//! PLUGIN-MIGRATION-001-E addition:
//!   33. plugin::LoadedPlugin            — struct, plugin/loader.rs
//!
//! S-CONFIG-MULTI-TENANT-OVERRIDE-001 additions (ADR-029):
//!   34. overlay::SensorInstanceOverlay  — struct, overlay.rs
//!   35. overlay::OverlayProvenance      — struct, overlay.rs
//!   36. overlay::ResolvedSensorSpec     — struct, overlay.rs

use prism_core::{ColumnType, RiskTier, SensorId, TableType};
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
