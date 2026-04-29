/// Configuration error codes for the prism-customer-config crate.
///
/// Each variant maps 1:1 to an error code (E-CFG-NNN) defined in BC-3.3.001,
/// BC-3.3.002, BC-3.3.003, and BC-3.3.004.
///
/// Display is implemented manually so that `UnsupportedSchemaVersion` (E-CFG-031)
/// can conditionally append the migration hint only when `migration_hint == true`
/// (i.e., `found > 1`). BC-3.3.003 postcondition 3 requires the hint only for
/// future-version configs, not for past versions like `schema_version = 0`.
#[derive(Debug)]
pub enum ConfigError {
    // E-CFG-000: TOML parse error
    TomlParseError {
        file: String,
        inner: String,
    },

    // E-CFG-001: Required field absent (R-CUST-001)
    MissingField {
        file: String,
        field: String,
    },

    // E-CFG-002: org_slug does not match filename stem (R-CUST-002)
    SlugMismatch {
        file: String,
        slug: String,
        stem: String,
    },

    // E-CFG-003: org_id is not a UUID v7 (R-CUST-003)
    InvalidOrgIdVersion {
        file: String,
        value: String,
        found_version: usize,
    },

    // E-CFG-004: DTU type absent from registry (R-CUST-004)
    UnknownDtuType {
        file: String,
        dtu_type: String,
    },

    // E-CFG-005: credential_ref lacks a recognized scheme prefix (R-CUST-005)
    InvalidCredentialRef {
        file: String,
        field: String,
    },

    // E-CFG-006: unknown archetype value (R-CUST-006)
    UnknownArchetype {
        file: String,
        value: String,
    },

    // E-CFG-007: invalid seed value (R-CUST-007)
    InvalidSeed {
        file: String,
        value: String,
    },

    // E-CFG-008: invalid scale value (R-CUST-008)
    InvalidScale {
        file: String,
        value: String,
    },

    // E-CFG-009: invalid mode value (R-CUST-009)
    InvalidMode {
        file: String,
        index: usize,
        value: String,
    },

    // E-CFG-010: unknown field (R-CUST-010, via deny_unknown_fields)
    UnknownField {
        file: String,
        field: String,
    },

    // E-CFG-011: duplicate org_id across files (R-CUST-011)
    DuplicateOrgId {
        file1: String,
        file2: String,
        org_id: String,
    },

    // E-CFG-012: duplicate org_slug across files (R-CUST-012)
    DuplicateOrgSlug {
        file1: String,
        file2: String,
        slug: String,
    },

    // E-CFG-013: test-only DTU type in production config (R-CUST-013)
    TestOnlyTypeInProduction {
        file: String,
        dtu_type: String,
    },

    // E-CFG-014: mode=client missing spec field (R-CUST-014)
    MissingClientSpec {
        file: String,
        dtu_index: usize,
    },

    // E-CFG-015: spec file path not found on disk (R-CUST-015)
    SpecFileNotFound {
        file: String,
        spec_path: String,
    },

    // E-CFG-016: shared mode with spec field present (R-CUST-016)
    SharedModeWithSpec {
        file: String,
        dtu_index: usize,
    },

    // E-CFG-017: Security Telemetry DTU type declared with shared mode (BC-3.3.001)
    SecurityTelemetrySharedMode {
        file: String,
        dtu_type: String,
    },

    // E-CFG-020: suspected credential value in config (BC-3.3.002)
    // NOTE: MUST NOT include the field value in the message (BC-3.3.002 Invariant 3)
    SuspectedCredentialValue {
        file: String,
        field_name: String,
    },

    // E-CFG-030: schema_version field absent (BC-3.3.003)
    MissingSchemaVersion {
        file: String,
    },

    // E-CFG-031: schema_version present but not equal to 1 (BC-3.3.003)
    // migration_hint is true only when found > 1 (future schema).
    // The Display impl conditionally appends the migration hint based on this flag.
    UnsupportedSchemaVersion {
        file: String,
        found: u64,
        /// True when found > 1: a future schema version that can be migrated down.
        /// False when found < 1: a past/unknown version with no migration path.
        migration_hint: bool,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::TomlParseError { file, inner } => {
                write!(f, "E-CFG-000 [{file}]: TOML parse error: {inner}")
            }
            ConfigError::MissingField { file, field } => {
                write!(f, "E-CFG-001 [{file}]: required field '{field}' is missing")
            }
            ConfigError::SlugMismatch { file, slug, stem } => {
                write!(
                    f,
                    "E-CFG-002 [{file}]: org_slug '{slug}' does not match filename stem '{stem}'"
                )
            }
            ConfigError::InvalidOrgIdVersion {
                file,
                value,
                found_version,
            } => {
                write!(
                    f,
                    "E-CFG-003 [{file}]: org_id '{value}' is UUID v{found_version}; must be UUID v7"
                )
            }
            ConfigError::UnknownDtuType { file, dtu_type } => {
                write!(
                    f,
                    "E-CFG-004 [{file}]: unknown DTU type '{dtu_type}'; not registered"
                )
            }
            ConfigError::InvalidCredentialRef { file, field } => {
                write!(
                    f,
                    "E-CFG-005 [{file}]: '{field}' must use a scheme prefix (vault://, env://, file://, keyring://)"
                )
            }
            ConfigError::UnknownArchetype { file, value } => {
                write!(f, "E-CFG-006 [{file}]: unknown archetype value '{value}'")
            }
            ConfigError::InvalidSeed { file, value } => {
                write!(
                    f,
                    "E-CFG-007 [{file}]: invalid seed value '{value}'; must be a non-negative integer"
                )
            }
            ConfigError::InvalidScale { file, value } => {
                write!(
                    f,
                    "E-CFG-008 [{file}]: invalid scale value '{value}'; must be a positive finite float"
                )
            }
            ConfigError::InvalidMode { file, index, value } => {
                write!(
                    f,
                    "E-CFG-009 [{file}]: [[dtu]][{index}] has invalid mode '{value}'; must be 'client' or 'shared'"
                )
            }
            ConfigError::UnknownField { file, field } => {
                write!(f, "E-CFG-010 [{file}]: unknown field '{field}'")
            }
            ConfigError::DuplicateOrgId {
                file1,
                file2,
                org_id,
            } => {
                write!(
                    f,
                    "E-CFG-011: org_id '{org_id}' is declared in both '{file1}' and '{file2}'"
                )
            }
            ConfigError::DuplicateOrgSlug { file1, file2, slug } => {
                write!(
                    f,
                    "E-CFG-012: org_slug '{slug}' is declared in both '{file1}' and '{file2}'"
                )
            }
            ConfigError::TestOnlyTypeInProduction { file, dtu_type } => {
                write!(
                    f,
                    "E-CFG-013 [{file}]: '{dtu_type}' is a test-only type and cannot appear in production customer config"
                )
            }
            ConfigError::MissingClientSpec { file, dtu_index } => {
                write!(
                    f,
                    "E-CFG-014 [{file}]: [[dtu]][{dtu_index}] mode='client' requires 'spec' field"
                )
            }
            ConfigError::SpecFileNotFound { file, spec_path } => {
                write!(f, "E-CFG-015 [{file}]: spec file not found: '{spec_path}'")
            }
            ConfigError::SharedModeWithSpec { file, dtu_index } => {
                write!(
                    f,
                    "E-CFG-016 [{file}]: [[dtu]][{dtu_index}] has 'spec' field but mode='shared'; 'spec' is only valid when mode='client'"
                )
            }
            ConfigError::SecurityTelemetrySharedMode { file, dtu_type } => {
                write!(
                    f,
                    "E-CFG-017 [{file}]: DTU type '{dtu_type}' is a Security Telemetry sensor and cannot use mode='shared'; set mode='client'"
                )
            }
            ConfigError::SuspectedCredentialValue { file, field_name } => {
                write!(
                    f,
                    "E-CFG-020 [{file}]: field '{field_name}' appears to contain a credential value; use a scheme-prefixed reference (vault://, env://, file://, keyring://) instead"
                )
            }
            ConfigError::MissingSchemaVersion { file } => {
                write!(
                    f,
                    "E-CFG-030 [{file}]: 'schema_version' field is missing; add 'schema_version = 1'"
                )
            }
            // E-CFG-031: conditionally append migration hint only for future versions (found > 1).
            // BC-3.3.003 postcondition 3: hint ONLY when found > 1.
            ConfigError::UnsupportedSchemaVersion {
                file,
                found,
                migration_hint,
            } => {
                write!(
                    f,
                    "E-CFG-031 [{file}]: unsupported schema_version {found}; only schema_version 1 is supported"
                )?;
                if *migration_hint {
                    write!(f, "; run 'prism config migrate' to upgrade")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigError {}
