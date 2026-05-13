//! Enum match violations (E0004) for #[non_exhaustive] enforcement.
//!
//! Each function exercises one #[non_exhaustive] enum by attempting an exhaustive match
//! without a wildcard arm. After `#[non_exhaustive]` is applied, each match MUST fail
//! with E0004 (non-exhaustive patterns).
//!
//! Violations 7-8, 13-15, 18-19, 25, 27-29 (11 total E0004 expected).

use prism_core::{ColumnOptions, ColumnType};
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
