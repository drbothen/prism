//! Struct literal violations (E0639) for #[non_exhaustive] enforcement.
//!
//! Each function exercises one #[non_exhaustive] struct by attempting external
//! struct-literal construction. After `#[non_exhaustive]` is applied, each
//! literal MUST fail with E0639 (cannot create non-exhaustive struct expression).
//!
//! Violations 1-6, 9-12, 16-17, 20-24, 26, 30 (19 total E0639 expected).

use prism_core::{ColumnType, RiskTier, TableType};
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
    SensorSpec as TypesSensorSpec, SensorTableDescriptor as TypesSensorTableDescriptor,
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

/// Violation 30: types::SensorSpec struct literal (E0639).
///
/// `prism_spec_engine::types::SensorSpec` is annotated `#[non_exhaustive]`
/// (hot-reload infrastructure type, distinct from spec_parser::SensorSpec).
/// External crates MUST NOT construct it via struct literal — fields may expand
/// as ADR-023 grammar evolves. This violation exercises the annotation to ensure
/// it is never silently removed.
#[allow(dead_code)]
pub fn v30_types_sensor_spec() {
    let _spec = TypesSensorSpec {
        sensor_id: "crowdstrike".to_string(),
        name: "CrowdStrike".to_string(),
        version: "1.0.0".to_string(),
        auth_type: "oauth2_client_credentials".to_string(),
        base_url: "https://api.crowdstrike.com".to_string(),
        tables: vec![],
        file_hash: "abc123".to_string(),
        source_path: "/specs/crowdstrike.sensor.toml".to_string(),
        // Intentionally omitting `mode` and `credential_refs` — E0639 fires because
        // #[non_exhaustive] prevents external struct-literal construction regardless
        // of whether all fields are supplied.
    };
    let _ = _spec;
}
