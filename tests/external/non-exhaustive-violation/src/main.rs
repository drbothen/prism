//! AC-5 (S-PLUGIN-PREREQ-C) — #[non_exhaustive] compile-fail test.
//!
//! Named: `test_BC_2_01_013_non_exhaustive_sensor_spec_no_external_literal`
//!
//! This file attempts struct-literal construction of all 8 TOML-deserialized types in
//! `prism-spec-engine` from OUTSIDE the crate. Once `#[non_exhaustive]` is applied
//! to each type (AC-5), every struct-literal expression here must fail with:
//!   E0639: cannot create non-exhaustive struct with a struct expression
//!   E0639: cannot create non-exhaustive variant with a struct expression
//!
//! RED GATE: Before AC-5, none of the 8 types carry `#[non_exhaustive]`.
//! Struct-literal construction succeeds and THIS CRATE COMPILES (cargo check exits 0).
//! The Red Gate is: running `cargo check -p non-exhaustive-violation` exits 0 BEFORE
//! AC-5 is implemented, but the expected behaviour is exit non-zero.
//!
//! GREEN: After AC-5, `#[non_exhaustive]` is applied to all 8 types.
//! `cargo check -p non-exhaustive-violation` exits non-zero with 8 E0639 errors.
//!
//! Target types (all 8 from AC-5 story table):
//!   1. CredentialRef          — struct, spec_parser.rs
//!   2. SensorSpec             — struct, spec_parser.rs
//!   3. SensorTableDescriptor  — struct, spec_parser.rs
//!   4. FetchStep              — struct, spec_parser.rs
//!   5. ColumnSpec             — struct, spec_parser.rs
//!   6. TableSpec              — struct, spec_parser.rs (via new_point_in_time constructor
//!                               — but direct struct-literal is the issue)
//!   7. PaginationConfig       — enum, spec_parser.rs (variant construction)
//!   8. AuthType               — enum, spec_parser.rs (variant construction)
//!
//! CI run: `cargo check -p non-exhaustive-violation`
//! Expected: FAIL (non-zero) after AC-5 implementation.
//! Currently (Red Gate): PASS (zero) = Red Gate condition met.

use prism_core::{ColumnOptions, ColumnType, TableType};
use prism_spec_engine::infusion::CredentialRef as InfusionCredentialRef;
use prism_spec_engine::spec_parser::{
    AuthType, ColumnSpec, CredentialRef, FetchStep, PaginationConfig, RateLimitHints, SensorSpec,
    SensorTableDescriptor, TableSpec,
};
use prism_spec_engine::types::{
    CredentialRef as TypesCredentialRef, SensorTableDescriptor as TypesSensorTableDescriptor,
};

/// test_BC_2_01_013_non_exhaustive_sensor_spec_no_external_literal
///
/// Attempt external struct-literal construction of all 8 AC-5 types.
/// After AC-5, each of these must fail with E0639 (non-exhaustive struct expression).
fn main() {
    // ── 1. CredentialRef ─────────────────────────────────────────────────────
    // Expected error after AC-5: E0639 — cannot create non-exhaustive struct
    // `CredentialRef` with a struct expression
    let _cred = CredentialRef {
        name: "api_key".to_string(),
    };

    // ── 2. SensorSpec ─────────────────────────────────────────────────────────
    // Expected error after AC-5: E0639 — cannot create non-exhaustive struct
    // `SensorSpec` with a struct expression
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

    // ── 3. SensorTableDescriptor ──────────────────────────────────────────────
    // Expected error after AC-5: E0639 — cannot create non-exhaustive struct
    // `SensorTableDescriptor` with a struct expression
    let _descriptor = SensorTableDescriptor {
        table_name: "crowdstrike.devices".to_string(),
        columns: vec![],
        sensor_id: "crowdstrike".to_string(),
        has_credentials: false,
    };

    // ── 4. FetchStep ─────────────────────────────────────────────────────────
    // Expected error after AC-5: E0639 — cannot create non-exhaustive struct
    // `FetchStep` with a struct expression
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

    // ── 5. ColumnSpec ─────────────────────────────────────────────────────────
    // Expected error after AC-5: E0639 — cannot create non-exhaustive struct
    // `ColumnSpec` with a struct expression
    let _col = ColumnSpec {
        name: "device_id".to_string(),
        column_type: ColumnType::String,
        ocsf_field: None,
        options: vec![],
    };

    // ── 6. TableSpec ─────────────────────────────────────────────────────────
    // Expected error after AC-5: E0639 — cannot create non-exhaustive struct
    // `TableSpec` with a struct expression (direct literal, not via constructor).
    let _table = TableSpec {
        table_name: "devices".to_string(),
        ocsf_class: "security_finding".to_string(),
        columns: vec![],
        steps: vec![],
        table_type: TableType::PointInTime,
        poll_interval_secs: None,
        retention_secs: None,
    };

    // ── 7. PaginationConfig (CursorToken variant) ──────────────────────────────
    // Expected error after AC-5: E0639 — cannot create non-exhaustive variant
    // `PaginationConfig::CursorToken` with a struct expression.
    // (Note: enum variant struct-literal construction is also blocked by #[non_exhaustive]
    //  on the enum when the variant uses struct syntax.)
    let _pagination = PaginationConfig::CursorToken {
        cursor_response_path: "$.next_cursor".to_string(),
        page_size: None,
    };

    // ── 8. AuthType (enum variant) ────────────────────────────────────────────
    // Expected error after AC-5: E0639 on pattern matching without wildcard arm.
    // For unit-variant enums, #[non_exhaustive] blocks exhaustive match from external crates.
    // We exercise this via a match statement that lacks a wildcard:
    let auth: AuthType = AuthType::BearerStatic;
    match auth {
        AuthType::Oauth2ClientCredentials => {}
        AuthType::BearerStatic => {}
        AuthType::CookieRoundtrip => {}
        AuthType::ApiKey => {}
        // After AC-5: E0004 — non-exhaustive patterns: `_` not covered.
        // Before AC-5: this compiles fine (no #[non_exhaustive]).
    }

    // ── 9. RateLimitHints ────────────────────────────────────────────────────
    // Expected error after HIGH-004 fix: E0639 — cannot create non-exhaustive struct
    // `RateLimitHints` with a struct expression
    let _rate_hints = RateLimitHints {
        requests_per_second: Some(10.0),
        burst_size: Some(100),
    };

    // ── 10. types::SensorTableDescriptor ─────────────────────────────────────
    // Expected error after HIGH-004 fix: E0639 — cannot create non-exhaustive struct
    // `types::SensorTableDescriptor` with a struct expression
    let _types_descriptor = TypesSensorTableDescriptor {
        table_name: "crowdstrike.devices".to_string(),
        columns: vec![],
        steps_count: 1,
        pagination_type: prism_spec_engine::types::PaginationType::Cursor,
    };

    // ── 11. types::CredentialRef ──────────────────────────────────────────────
    // Expected error after HIGH-004 fix: E0639 — cannot create non-exhaustive struct
    // `types::CredentialRef` with a struct expression
    let _types_cred = TypesCredentialRef {
        name: "api_key".to_string(),
    };

    // ── 12. infusion::CredentialRef ───────────────────────────────────────────
    // Expected error after HIGH-004 fix: E0639 — cannot create non-exhaustive struct
    // `infusion::CredentialRef` with a struct expression
    let _infusion_cred = InfusionCredentialRef {
        field_name: "api_key".to_string(),
        env_var: "MY_API_KEY".to_string(),
    };

    // ── 13. prism_core::ColumnType (exhaustive match without wildcard) ────────
    // Expected error after HIGH-004 fix: E0004 — non-exhaustive patterns without wildcard
    let col_type: ColumnType = ColumnType::String;
    match col_type {
        ColumnType::String => {}
        ColumnType::Integer => {}
        ColumnType::Float => {}
        ColumnType::Boolean => {}
        ColumnType::Datetime => {}
        ColumnType::Json => {}
        // After HIGH-004: E0004 — `_` arm required for non-exhaustive enum
    }

    // ── 14. prism_core::ColumnOptions (exhaustive match without wildcard) ─────
    // Expected error after HIGH-004 fix: E0004 — non-exhaustive patterns without wildcard
    let col_opt: ColumnOptions = ColumnOptions::Required;
    match col_opt {
        ColumnOptions::Required => {}
        ColumnOptions::Index => {}
        ColumnOptions::Additional => {}
        ColumnOptions::Hidden => {}
        ColumnOptions::Optimized => {}
        // After HIGH-004: E0004 — `_` arm required for non-exhaustive enum
    }

    // Force all bindings to be used (suppress dead-code warnings).
    let _ = (
        _cred,
        _sensor,
        _descriptor,
        _step,
        _col,
        _table,
        _pagination,
        _rate_hints,
        _types_descriptor,
        _types_cred,
        _infusion_cred,
    );
}
