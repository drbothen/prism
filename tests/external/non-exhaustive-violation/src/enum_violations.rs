//! Enum match violations (E0004) for #[non_exhaustive] enforcement.
//!
//! Each function exercises one #[non_exhaustive] enum by attempting an exhaustive match
//! without a wildcard arm. After `#[non_exhaustive]` is applied, each match MUST fail
//! with E0004 (non-exhaustive patterns).
//!
//! Violations 7-8, 13-15, 18-19, 25, 27-29, 31, 44, 46, 48, 60, 65, 70, 79, 85-86, 90 (22 total E0004 expected).
//!
//! S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 additions:
//!   85. prism_mcp::resources::ExampleKind — enum, resources.rs (ADR-045 reference example classification)
//!
//! DEFECT-CSDEVICES-EMPTY-PIPELINE-001 F-CSD-P28-OBS-001:
//!   90. prism_core::virtual_fields::VirtualField — enum, virtual_fields.rs (pre-DataFusion queryable metadata columns)
//!
//! S-5.01-FOLLOWUP-MCP-BOOT additions (prism-mcp pub enum types):
//!   44. prism_mcp::safety_envelope::DataSource — enum, safety_envelope.rs
//!
//! S-DEMO-001 additions (prism-bin pub enum types):
//!   48. prism_bin::spec_driven_adapter::AdapterAuthStrategy — enum, spec_driven_adapter.rs
//!
//! S-5.02 follow-up fix-burst (CRIT-1/HIGH-1 non-exhaustive gate sibling-sweep):
//!   65. prism_mcp::CapabilityStatus — enum, server.rs (re-exported from lib.rs)
//!
//! S-1.14-REDO adversarial OBS-1 FIX-IN-SCOPE:
//!   70. prism_core::InfusionError — enum, error.rs (re-exported from prism_core)

use prism_core::{ColumnOptions, ColumnType, InfusionError, PluginError};
use prism_spec_engine::infusion::{BuiltInSourceType, InfusionType};
use prism_spec_engine::spec_parser::{AuthType, PaginationConfig};
use prism_spec_engine::types::{
    ClientStatus, ColumnType as TypesColumnType, PaginationType, SpecStatus,
};
use prism_spec_engine::write_endpoint::BatchMode;

/// Violation 7: PaginationConfig exhaustive match (E0004).
pub fn v07_pagination_config_match() {
    let p: PaginationConfig = PaginationConfig::None;
    match p {
        PaginationConfig::None => {}
        PaginationConfig::CursorToken { .. } => {}
        // After AC-5: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 8: AuthType exhaustive match (E0004).
pub fn v08_auth_type_match() {
    let auth: AuthType = AuthType::BearerStatic;
    match auth {
        AuthType::Oauth2ClientCredentials => {}
        AuthType::BearerStatic => {}
        AuthType::CookieRoundtrip => {}
        AuthType::ApiKey => {}
        // After AC-5: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 13: prism_core::ColumnType exhaustive match (E0004).
pub fn v13_core_column_type_match() {
    let col_type: ColumnType = ColumnType::String;
    match col_type {
        ColumnType::String => {}
        ColumnType::Integer => {}
        ColumnType::Float => {}
        ColumnType::Boolean => {}
        ColumnType::Datetime => {}
        ColumnType::Json => {}
        // After HIGH-004: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 14: prism_core::ColumnOptions exhaustive match (E0004).
pub fn v14_core_column_options_match() {
    let col_opt: ColumnOptions = ColumnOptions::Required;
    match col_opt {
        ColumnOptions::Required => {}
        ColumnOptions::Index => {}
        ColumnOptions::Additional => {}
        ColumnOptions::Hidden => {}
        ColumnOptions::Optimized => {}
        // After HIGH-004: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 15: BatchMode exhaustive match (E0004).
pub fn v15_batch_mode_match() {
    let batch_mode: BatchMode = BatchMode::Serial;
    match batch_mode {
        BatchMode::Serial => {}
        BatchMode::Parallel => {}
        // After fix-burst-2: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 18: InfusionType exhaustive match (E0004).
pub fn v18_infusion_type_match() {
    let infusion_type: InfusionType = InfusionType::LocalLookup;
    match infusion_type {
        InfusionType::LocalLookup => {}
        InfusionType::Plugin => {}
        // After fix-burst-2: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 19: BuiltInSourceType exhaustive match (E0004).
pub fn v19_built_in_source_type_match() {
    let source_type: BuiltInSourceType = BuiltInSourceType::Csv;
    match source_type {
        BuiltInSourceType::MaxmindMmdb => {}
        BuiltInSourceType::Csv => {}
        BuiltInSourceType::JsonLookup => {}
        // After fix-burst-2: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 25: types::ColumnType exhaustive match (E0004).
/// ADR-024: types::ColumnType is now re-exported from prism_core::column::ColumnType
/// (domain-level names Integer/Float/Datetime; #[non_exhaustive] preserved via re-export).
pub fn v25_types_column_type_match() {
    let types_col_type: TypesColumnType = TypesColumnType::String;
    match types_col_type {
        TypesColumnType::String => {}
        TypesColumnType::Integer => {}
        TypesColumnType::Float => {}
        TypesColumnType::Boolean => {}
        TypesColumnType::Datetime => {}
        TypesColumnType::Json => {}
        // After ADR-024: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 27: types::PaginationType exhaustive match (E0004).
pub fn v27_pagination_type_match() {
    let pagination_type: PaginationType = PaginationType::Cursor;
    match pagination_type {
        PaginationType::Cursor => {}
        PaginationType::Offset => {}
        PaginationType::None => {}
        // After fix-burst-2: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 28: types::SpecStatus exhaustive match (E0004).
pub fn v28_spec_status_match() {
    let spec_status: SpecStatus = SpecStatus::Loaded;
    match spec_status {
        SpecStatus::Loaded => {}
        SpecStatus::FailedValidation => {}
        SpecStatus::PendingReload => {}
        SpecStatus::NoCredentials => {}
        SpecStatus::ValidationWarnings { .. } => {}
        // After fix-burst-2: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 29: types::ClientStatus exhaustive match (E0004).
pub fn v29_client_status_match() {
    let client_status: ClientStatus = ClientStatus::Configured;
    match client_status {
        ClientStatus::Configured => {}
        ClientStatus::NotConfigured => {}
        // After fix-burst-2: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 31: prism_core::PluginError exhaustive match (E0004).
/// F-LP22 closure (D-572): PluginError is a pub API in prism-core and requires
/// #[non_exhaustive] per the project's non-exhaustive discipline (CLAUDE.md).
pub fn v31_plugin_error_match() {
    let plugin_err: PluginError = PluginError::NotLoaded {
        plugin_id: "test".to_string(),
    };
    match plugin_err {
        PluginError::Trapped { .. } => {}
        PluginError::Timeout { .. } => {}
        PluginError::MemoryExceeded { .. } => {}
        PluginError::NotLoaded { .. } => {}
        PluginError::InvalidInterface { .. } => {}
        PluginError::SandboxViolation { .. } => {}
        PluginError::CompilationFailed { .. } => {}
        PluginError::EmptyPluginId { .. } => {}
        // After F-LP22: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 46: prism_security::confirmation_token::BoundingDmlOperation exhaustive match (E0004).
///
/// `BoundingDmlOperation` is the mirrored DML kind stored in confirmation tokens
/// (OBS-1 fix, confirmation_token.rs). `#[non_exhaustive]` ensures external match arms
/// include a wildcard so new DML kinds (e.g., `Truncate`, `Upsert`) can be added
/// without requiring all downstream match consumers to update immediately
/// (F-PR163-IMP-1). External callers MUST include `_ => {}`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
pub fn v46_bounding_dml_operation_match() {
    use prism_security::confirmation_token::BoundingDmlOperation;
    let op: BoundingDmlOperation = BoundingDmlOperation::InsertInto;
    match op {
        BoundingDmlOperation::InsertInto => {}
        BoundingDmlOperation::Update => {}
        BoundingDmlOperation::Delete => {}
        // After F-PR163-IMP-1: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 44: prism_mcp::safety_envelope::DataSource exhaustive match (E0004).
///
/// `DataSource` is the `_meta.data_source` field of the MCP response envelope
/// (BC-2.09.008 EC-09-019). `#[non_exhaustive]` ensures new data source types
/// (e.g., `Stream`, `Cache`, `Federated`) can be added without requiring all
/// downstream match arms to be updated immediately.
/// External callers MUST include a wildcard arm: `_ => { /* unknown source */ }`.
///
/// Added: S-5.01-FOLLOWUP-MCP-BOOT.
pub fn v44_data_source_match() {
    use prism_mcp::safety_envelope::DataSource;
    let ds: DataSource = DataSource::Single("crowdstrike".to_string());
    match ds {
        DataSource::Single(_) => {}
        DataSource::Multiple(_) => {}
        // After S-5.01-FOLLOWUP-MCP-BOOT: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 48: prism_bin::spec_driven_adapter::AdapterAuthStrategy exhaustive match (E0004).
///
/// `AdapterAuthStrategy` is the auth dispatch enum held by `SpecDrivenSensorAdapter`
/// (OQ-1 Resolution, S-DEMO-001). `#[non_exhaustive]` ensures new strategies
/// (e.g., `ApiKey`, `Oauth2`) can be added without requiring all external match arms
/// to include a wildcard immediately (CR-001, S-DEMO-001 PR review).
/// External callers MUST include `_ => {}`.
///
/// Added: S-DEMO-001.
#[allow(dead_code)]
pub fn v48_adapter_auth_strategy_match() {
    use prism_bin::spec_driven_adapter::AdapterAuthStrategy;
    // Construct a representative value without triggering E0639 on an inner type.
    // BearerStatic is a unit variant — safe to construct without any Arc.
    let strategy: AdapterAuthStrategy = AdapterAuthStrategy::BearerStatic;
    match strategy {
        AdapterAuthStrategy::Plugin(_) => {}
        AdapterAuthStrategy::BearerStatic => {}
        AdapterAuthStrategy::StaticCookie(_) => {}
        // After S-DEMO-001 CR-001: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 60: prism_dtu_demo_server::MultiInstanceBindError exhaustive match (E0004).
///
/// `MultiInstanceBindError` is the error type returned by `start_instances`
/// (BC-2.06.017 Postconditions 6–7). `#[non_exhaustive]` ensures future error variants
/// (e.g., `TimeoutError`, `TlsError`) can be added without requiring all external
/// match arms to be updated immediately.
/// External callers MUST include `_ => {}` or handle via
/// `match e { DuplicateName{..} => .., BindFailure(..) => .., _ => {} }`.
///
/// Added: S-DEMO-MULTI-TENANT-DTU-001 (U-006). ci.yml EXPECTED bumped from 52 to 60.
/// This is violation 60 of 60: 7 E0639 struct violations (v54–v59 + v61 in struct_violations.rs;
/// v54–v59 are MultiInstanceConfig, InstanceEntry, DemoBindError, MultiInstanceHarness,
/// HarnessEntry, BindError; v61 is MultiInstanceServers, added by D-1075-API-GAP-001)
/// + this 1 E0004 enum violation (v60) = 8 new violations total, bringing the gate from 52 → 60.
#[allow(dead_code)]
pub fn v60_multi_instance_bind_error_match() {
    use prism_dtu_demo_server::MultiInstanceBindError;
    // Construct a representative DuplicateName variant.
    let err: MultiInstanceBindError = MultiInstanceBindError::DuplicateName {
        name: "test".to_string(),
    };
    match err {
        MultiInstanceBindError::DuplicateName { .. } => {}
        MultiInstanceBindError::BindFailure(_) => {}
        // After S-DEMO-MULTI-TENANT-DTU-001: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 65: prism_mcp::CapabilityStatus exhaustive match (E0004).
///
/// `CapabilityStatus` is the tri-state capability status in the `list_capabilities` response
/// (BC-2.10.011 v1.5, S-5.02 R4). `#[non_exhaustive]` ensures that new status variants
/// (e.g., `TemporarilyDisabled`, `RequiresElevation`) can be added without requiring all
/// external match arms to be updated immediately.
/// External callers MUST include `_ => {}`.
///
/// Added: S-5.02 follow-up fix-burst (CRIT-1/HIGH-1 non-exhaustive gate sibling-sweep).
/// ci.yml EXPECTED bumped from 61 to 64 (together with v63 CapabilityEntry + v64 ResolutionStep
/// struct violations).
#[allow(dead_code)]
pub fn v65_capability_status_match() {
    use prism_mcp::CapabilityStatus;
    let status: CapabilityStatus = CapabilityStatus::Enabled;
    match status {
        CapabilityStatus::Enabled => {}
        CapabilityStatus::RuntimeDisabled => {}
        CapabilityStatus::CompileTimeDisabled => {}
        // After S-5.02: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 70: prism_core::InfusionError exhaustive match (E0004).
///
/// `InfusionError` is the pub-API error type for the infusion enrichment framework
/// (BC-2.19.001 through BC-2.19.005, S-1.14-REDO). `#[non_exhaustive]` ensures
/// that new error variants (e.g., future E-INFUSE-* codes) can be added without
/// requiring all external match arms to be updated immediately.
/// External callers MUST include `_ => {}`.
///
/// Added: S-1.14-REDO adversarial OBS-1 FIX-IN-SCOPE. ci.yml EXPECTED bumped 69 → 70.
/// Updated: S-DEMO-ENRICHMENT-PIVOT-002 adds InvalidFieldSpec, HttpLookupFailed,
/// CredentialResolutionFailed, SsrfRejected variants.
#[allow(dead_code)]
pub fn v70_infusion_error_match() {
    let err: InfusionError = InfusionError::UnknownInfusion {
        name: "test".to_string(),
    };
    match err {
        InfusionError::UnknownInfusion { .. } => {}
        InfusionError::DuplicateUdfName { .. } => {}
        InfusionError::MissingRequiredField { .. } => {}
        InfusionError::UnknownSourceType { .. } => {}
        InfusionError::CredentialUnresolved { .. } => {}
        InfusionError::ApiBackedUdfInDetectionRule { .. } => {}
        InfusionError::InvalidFieldSpec { .. } => {}
        InfusionError::PluginCallFailed { .. } => {}
        InfusionError::HttpLookupFailed { .. } => {}
        InfusionError::CredentialResolutionFailed { .. } => {}
        InfusionError::SsrfRejected { .. } => {}
        InfusionError::SourceFileTooLarge { .. } => {}
        // After S-1.14-REDO OBS-1 + S-DEMO-ENRICHMENT-PIVOT-002: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 79: prism_spec_engine::infusion::HttpLookupAuthType exhaustive match (E0004).
///
/// `HttpLookupAuthType` is the authentication mechanism enum for http_lookup-type
/// infusions (ADR-040 v2.0 D8.2). `#[non_exhaustive]` ensures new auth mechanisms
/// (e.g., `MutualTls`, `Oauth2ClientCredentials`) can be added without requiring all
/// external match arms to be updated immediately.
/// External callers MUST include `_ => {}`.
///
/// Added: S-DEMO-ENRICHMENT-PIVOT-002 v1.3. Renumbered v79 (was v71 pre-rebase;
/// S-5.03 claimed v71-v76 on develop@85ac7b06; struct violations v77-v78 are also
/// S-DEMO-ENRICHMENT-PIVOT-002 in struct_violations.rs).
#[allow(dead_code)]
pub fn v79_http_lookup_auth_type_match() {
    use prism_spec_engine::infusion::HttpLookupAuthType;
    let auth: HttpLookupAuthType = HttpLookupAuthType::BearerHeader;
    match auth {
        HttpLookupAuthType::QueryParam { .. } => {}
        HttpLookupAuthType::BearerHeader => {}
        HttpLookupAuthType::ApiKeyHeader { .. } => {}
        // After S-DEMO-ENRICHMENT-PIVOT-002: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 86: prism_core::TemporalLiteralPosition exhaustive match (E0004).
///
/// `TemporalLiteralPosition` is the position enum for E-QUERY-042
/// (`PrismError::TemporalLiteralInvalidPosition`). `#[non_exhaustive]` ensures new
/// position variants (e.g., `HavingClause`, `SetAssignment`) can be added in future
/// ADR-052 revisions without requiring all external match arms to be updated.
/// External callers MUST include `_ => {}`.
///
/// Added: S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 E-QUERY-042 implementation.
/// ci.yml EXPECTED bumped from 88 to 89.
#[allow(dead_code)]
pub fn v86_temporal_literal_position_match() {
    use prism_core::TemporalLiteralPosition;
    let pos: TemporalLiteralPosition = TemporalLiteralPosition::GroupBy;
    match pos {
        TemporalLiteralPosition::GroupBy => {}
        TemporalLiteralPosition::OrderBy => {}
        TemporalLiteralPosition::NonColumnLhsComparison => {}
        // After S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 90: prism_core::virtual_fields::VirtualField exhaustive match (E0004).
///
/// `VirtualField` is the pre-DataFusion queryable metadata column enum in `prism-core`
/// (BC-2.15.009 — virtual field category, S-2.03). `#[non_exhaustive]` ensures that future
/// virtual columns (e.g., a tenant-partition field or a pipeline-version sentinel) can be
/// added without requiring all external match arms to be updated immediately.
/// External callers MUST include `_ => {}`.
///
/// Added: DEFECT-CSDEVICES-EMPTY-PIPELINE-001 F-CSD-P28-OBS-001. ci.yml EXPECTED bumped 89 → 90.
#[allow(dead_code)]
pub fn v90_virtual_field_match() {
    use prism_core::VirtualField;
    let vf: VirtualField = VirtualField::Sensor;
    match vf {
        VirtualField::Sensor => {}
        VirtualField::Client => {}
        VirtualField::SourceTable => {}
        // After DEFECT-CSDEVICES-EMPTY-PIPELINE-001 F-CSD-P28-OBS-001: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}

/// Violation 85: prism_mcp::resources::ExampleKind exhaustive match (E0004).
///
/// `ExampleKind` classifies PQL usage examples for the 4-tier CI gate and for
/// `build_reference_content` (ADR-045 §B, BC-2.11.022). `#[non_exhaustive]` ensures new
/// example tiers can be added without requiring external `match` arms to be updated.
/// External callers MUST include `_ => {}`.
///
/// Variants renamed Positive/NegativeE040/NegativeOther per BC-2.11.022 / ADR-045 D3
/// (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 CRIT-003 fix-burst).
/// NegativeE043 added: FIX-CSDEVICES-EMPTY-PIPELINE F-CSD-P26-OBS-001.
///
/// Added: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (resources.rs).
#[allow(dead_code)]
pub fn v85_example_kind_match() {
    use prism_mcp::resources::ExampleKind;
    let kind: ExampleKind = ExampleKind::Positive;
    match kind {
        ExampleKind::Positive => {}
        ExampleKind::NegativeE040 => {}
        ExampleKind::NegativeOther => {}
        ExampleKind::NegativeE043 => {}
        // After FIX-CSDEVICES-EMPTY-PIPELINE: E0004 — `_` arm required for #[non_exhaustive] enum
    }
}
