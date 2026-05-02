use std::collections::HashMap;
use std::path::Path;

use crate::credential_check::scan_for_credentials;
use crate::error::ConfigError;
use crate::schema::CustomerConfig;

// ---------------------------------------------------------------------------
// DTU type classification — inlined from prism-core::dtu to avoid crate dep.
// Source of truth: crates/prism-core/src/dtu.rs (DTU_DEFAULT_MODE).
// Wave 3 types: 4 Security Telemetry (Client), 5 MSSP Coordination (Shared),
// 1 test-only (demo-server, Client, test_only=true).
// ---------------------------------------------------------------------------

/// A minimal registry entry for DTU type classification.
struct DtuEntry {
    type_name: &'static str,
    /// True if the default mode is Client (Security Telemetry); false = Shared (MSSP).
    is_security_telemetry: bool,
    /// True if only valid in test/demo contexts, not production customer config.
    test_only: bool,
}

const DTU_REGISTRY: &[DtuEntry] = &[
    // Security Telemetry (Client, production)
    DtuEntry {
        type_name: "claroty",
        is_security_telemetry: true,
        test_only: false,
    },
    DtuEntry {
        type_name: "armis",
        is_security_telemetry: true,
        test_only: false,
    },
    DtuEntry {
        type_name: "crowdstrike",
        is_security_telemetry: true,
        test_only: false,
    },
    DtuEntry {
        type_name: "cyberint",
        is_security_telemetry: true,
        test_only: false,
    },
    // Test-only Security Telemetry (D-051)
    DtuEntry {
        type_name: "demo-server",
        is_security_telemetry: true,
        test_only: true,
    },
    // MSSP Coordination (Shared, production)
    DtuEntry {
        type_name: "slack",
        is_security_telemetry: false,
        test_only: false,
    },
    DtuEntry {
        type_name: "pagerduty",
        is_security_telemetry: false,
        test_only: false,
    },
    DtuEntry {
        type_name: "jira",
        is_security_telemetry: false,
        test_only: false,
    },
    DtuEntry {
        type_name: "nvd",
        is_security_telemetry: false,
        test_only: false,
    },
    DtuEntry {
        type_name: "threatintel",
        is_security_telemetry: false,
        test_only: false,
    },
];

/// Archetype catalog — valid values for `data.archetype` (ADR-009 §2.2 / ADR-010 §2.3).
const VALID_ARCHETYPES: &[&str] = &[
    "HealthyOtEnvironment",
    "CompromisedEndpoint",
    "AuthOutage",
    "LargeScale",
    "PaginationEdgeCases",
    "SchemaDrift",
    "HighChurn",
    "DormantTenant",
];

/// Allowed scheme prefixes for `credential_ref` (ADR-010 §2.3.1).
const ALLOWED_CRED_SCHEMES: &[&str] = &["vault://", "env://", "file://", "keyring://"];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Validate all `*.toml` files in `dir`.
///
/// Files are processed in lexicographic filename order (BC-3.3.004 Invariant 4).
/// Non-`.toml` files are silently skipped (EC-3.3.001-07).
///
/// Validation pass order per file (BC-3.3.003 Invariant 4, ADR-010 §2.6):
/// 1. TOML parse → `E-CFG-000` on failure
/// 2. `schema_version` check first — absent → `E-CFG-030`; ≠ 1 → `E-CFG-031`
/// 3. Credential heuristic pass (BC-3.3.002) on raw TOML value tree
/// 4. Structural validation (R-CUST-001 through R-CUST-017)
///
/// Cross-file after all per-file passes:
/// - Duplicate `org_id` → `E-CFG-011`
/// - Duplicate `org_slug` → `E-CFG-012`
///
/// Returns ALL errors collected across ALL files (multi-error, not fail-fast).
/// Returns empty vec on success.
/// # Visibility (CR-005 / W3-FIX-CODE-002)
///
/// Changed from `pub` to `pub(crate)`. The only public entry point is
/// `load_and_validate` (which calls this function). External crates must not
/// call `validate_all` directly — the partial-config behaviour on duplicate-id
/// errors is a usability trap for external callers (AC-002; BC-3.3.004 Invariant 1).
pub(crate) fn validate_all(dir: &Path) -> (Vec<CustomerConfig>, Vec<ConfigError>) {
    // Collect *.toml files in lexicographic order.
    let mut toml_files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
                .collect()
        })
        .unwrap_or_default();
    toml_files.sort();

    let mut all_errors: Vec<ConfigError> = Vec::new();
    let mut valid_configs: Vec<CustomerConfig> = Vec::new();

    // Per-file tracking for cross-file duplicate detection.
    // Maps org_id string -> filename
    let mut org_id_map: HashMap<String, String> = HashMap::new();
    // Maps org_slug string -> filename
    let mut org_slug_map: HashMap<String, String> = HashMap::new();

    for path in &toml_files {
        let (maybe_config, file_errors) = validate_file(path);
        all_errors.extend(file_errors);

        if let Some(config) = maybe_config {
            valid_configs.push(config);
        }
    }

    // Cross-file: duplicate org_id / org_slug detection.
    // We re-parse valid configs only — configs with errors may not have a valid struct.
    // Re-scan all files for org_id/org_slug (even those that produced errors, to detect
    // collisions that would cause partial registration issues).
    for path in &toml_files {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Try to extract org_id and org_slug from the raw TOML for duplicate detection.
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let raw: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue, // parse errors already reported
        };

        if let toml::Value::Table(table) = &raw {
            // org_id duplicate check
            if let Some(toml::Value::String(oid)) = table.get("org_id") {
                if let Some(prev_file) = org_id_map.get(oid) {
                    all_errors.push(ConfigError::DuplicateOrgId {
                        file1: prev_file.clone(),
                        file2: file_name.clone(),
                        org_id: oid.clone(),
                    });
                } else {
                    org_id_map.insert(oid.clone(), file_name.clone());
                }
            }

            // org_slug duplicate check
            if let Some(toml::Value::String(slug)) = table.get("org_slug") {
                if let Some(prev_file) = org_slug_map.get(slug) {
                    all_errors.push(ConfigError::DuplicateOrgSlug {
                        file1: prev_file.clone(),
                        file2: file_name.clone(),
                        slug: slug.clone(),
                    });
                } else {
                    org_slug_map.insert(slug.clone(), file_name.clone());
                }
            }
        }
    }

    (valid_configs, all_errors)
}

/// Parse and validate a single TOML file.
///
/// Returns `(Some(config), errors)` when the TOML parses successfully and the
/// config has no schema_version/parse failures (structural errors are still
/// collected). Returns `(None, errors)` when parse or schema_version check fails.
fn validate_file(path: &Path) -> (Option<CustomerConfig>, Vec<ConfigError>) {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let file_stem = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return (
                None,
                vec![ConfigError::TomlParseError {
                    file: file_name,
                    inner: e.to_string(),
                }],
            );
        }
    };

    // Step 1: Parse as raw TOML::Value (for schema_version check and credential scan).
    let raw: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            let inner = e.to_string();
            // SEC-006: include source context in parse errors so that non-credential
            // field values (e.g. display_name) are visible in diagnostic output.
            // sanitize_error_message redacts credential-pattern fields from both the
            // TOML parser snippet and the appended raw source lines.
            let full = format!("{inner}\n\nSource context:\n{content}");
            let sanitized = sanitize_error_message(&full);
            return (
                None,
                vec![ConfigError::TomlParseError {
                    file: file_name,
                    inner: sanitized,
                }],
            );
        }
    };

    let mut errors: Vec<ConfigError> = Vec::new();

    // Step 2: schema_version check FIRST (BC-3.3.003 Invariant 4).
    let schema_version_ok = check_schema_version(&file_name, &raw, &mut errors);

    // Step 3: Credential heuristic pass on the raw TOML value tree (BC-3.3.002).
    let cred_errors = scan_for_credentials(&file_name, &raw);
    errors.extend(cred_errors);

    // Step 4: Deserialize into typed struct.
    // This catches unknown fields (deny_unknown_fields), type mismatches, etc.
    let config: CustomerConfig = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            let inner = e.to_string();
            // Attempt to map structured serde errors to typed ConfigError variants.
            // "missing field `<field>`" → MissingField (R-CUST-001).
            if let Some(field) = extract_missing_field(&inner) {
                errors.push(ConfigError::MissingField {
                    file: file_name.clone(),
                    field,
                });
            } else {
                // Generic parse error — redact any credential values from the message.
                let sanitized = sanitize_error_message(&inner);
                errors.push(ConfigError::TomlParseError {
                    file: file_name,
                    inner: sanitized,
                });
            }
            // Cannot do structural validation without a parsed struct.
            return (None, errors);
        }
    };

    if !schema_version_ok {
        // schema_version is wrong; skip structural validation but continue credential
        // scan errors — return None for the config since it's not loadable.
        return (None, errors);
    }

    // Step 5: Structural validation (R-CUST-001 through R-CUST-017).
    validate_structural(&file_name, &file_stem, &config, path, &mut errors);

    let is_valid = errors.is_empty();
    (if is_valid { Some(config) } else { None }, errors)
}

// ---------------------------------------------------------------------------
// Error message helpers
// ---------------------------------------------------------------------------

/// Attempt to extract a field name from a serde "missing field `<name>`" error message.
/// Returns `Some(field_name)` if the pattern matches, else `None`.
fn extract_missing_field(error_str: &str) -> Option<String> {
    // toml 0.8 formats: "missing field `field_name`"
    let pattern = "missing field `";
    if let Some(start) = error_str.find(pattern) {
        let rest = &error_str[start + pattern.len()..];
        if let Some(end) = rest.find('`') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Sanitize a TOML parse error message to remove credential values.
///
/// TOML 0.8 error messages include multi-line snippets that show the offending
/// line including its value. For credential-named fields, the value MUST NOT
/// appear in the error (BC-3.3.002 Invariant 3).
///
/// The TOML 0.8 snippet format is:
/// ```text
///    |
/// 12 | bearer_token = "abc123"
///    | ^^^^^^^^^^^^
/// ```
/// This function redacts the value part of assignment lines whose field name
/// matches credential naming patterns.
///
/// ## Multi-line (triple-quoted) credential handling (SEC-006)
///
/// For triple-quoted values like:
/// ```toml
/// password = """
/// my-secret-value
/// """
/// ```
/// The opening line `password = """` is a credential field assignment. Subsequent
/// snippet lines contain the secret content and MUST also be redacted, until the
/// closing `"""` line is seen. Non-credential fields (e.g. `display_name`) use
/// the same triple-quote syntax but their continuation lines must NOT be redacted.
fn sanitize_error_message(error_str: &str) -> String {
    // Process line-by-line.
    let lines: Vec<&str> = error_str.lines().collect();
    let mut result = Vec::with_capacity(lines.len());
    // Tracks whether we are inside a multi-line (triple-quoted) credential value.
    let mut in_multiline_cred = false;

    for line in &lines {
        // TOML 0.8 snippet lines look like: "  N | content" or "   | ^^^^^"
        // Find the " | " separator that separates the line number from content.
        if let Some(pipe_pos) = find_snippet_pipe(line) {
            let content = &line[pipe_pos + 3..]; // skip " | "

            // If we are inside a multi-line credential block, redact until closing `"""`.
            if in_multiline_cred {
                let trimmed = content.trim();
                if trimmed == "\"\"\"" || trimmed.ends_with("\"\"\"") {
                    // Closing triple-quote — redact this line too, then exit the block.
                    let prefix = &line[..pipe_pos + 3];
                    result.push(format!("{prefix}[redacted]"));
                    in_multiline_cred = false;
                } else {
                    // Continuation line of a multi-line credential — redact it.
                    let prefix = &line[..pipe_pos + 3];
                    result.push(format!("{prefix}[redacted]"));
                }
                continue;
            }

            // Check if this content is a TOML assignment with a credential field.
            if let Some(eq_pos) = content.find(" = ") {
                let field_name = content[..eq_pos].trim();
                if is_credential_pattern(field_name) {
                    let value_part = content[eq_pos + 3..].trim();
                    // Replace value with [redacted].
                    let prefix = &line[..pipe_pos + 3];
                    result.push(format!("{prefix}{field_name} = [redacted]"));
                    // If the opening value is `"""`, enter multi-line mode.
                    if value_part == "\"\"\"" {
                        in_multiline_cred = true;
                    }
                    continue;
                }
            }
        } else {
            // Non-snippet line (e.g., raw source context appended for diagnostics).
            // If inside a multi-line credential block, redact continuation lines.
            if in_multiline_cred {
                let trimmed = line.trim();
                if trimmed == "\"\"\"" || trimmed.ends_with("\"\"\"") {
                    result.push("[redacted]".to_string());
                    in_multiline_cred = false;
                } else {
                    result.push("[redacted]".to_string());
                }
                continue;
            }

            // Also redact credential-pattern assignments in raw source lines
            // (no pipe format). This handles the case where raw file content is
            // appended to the error message for diagnostic context.
            if let Some(eq_pos) = line.find(" = ") {
                let field_name = line[..eq_pos].trim();
                if is_credential_pattern(field_name) {
                    let value_part = line[eq_pos + 3..].trim();
                    result.push(format!("{field_name} = [redacted]"));
                    if value_part == "\"\"\"" {
                        in_multiline_cred = true;
                    }
                    continue;
                }
            }
        }
        result.push(line.to_string());
    }
    result.join("\n")
}

/// Find the position of ` | ` in a TOML snippet line.
/// Returns the byte offset of the space before `|`.
fn find_snippet_pipe(line: &str) -> Option<usize> {
    // Pattern: optional whitespace, digits or whitespace, then " | "
    // We look for " | " after stripping leading content.
    line.find(" | ")
}

/// Returns true if a field name matches credential naming patterns.
fn is_credential_pattern(name: &str) -> bool {
    const SUFFIXES: &[&str] = &["_token", "_secret", "_key", "_password", "_pass"];
    const EXACT: &[&str] = &["password"];
    if EXACT.contains(&name) {
        return true;
    }
    SUFFIXES.iter().any(|s| name.ends_with(s))
}

/// Validate `schema_version`: absent → E-CFG-030, ≠ 1 → E-CFG-031.
/// Returns `true` if schema_version is present and equals 1.
fn check_schema_version(file: &str, raw: &toml::Value, errors: &mut Vec<ConfigError>) -> bool {
    let table = match raw.as_table() {
        Some(t) => t,
        None => return false,
    };

    match table.get("schema_version") {
        None => {
            errors.push(ConfigError::MissingSchemaVersion {
                file: file.to_string(),
            });
            false
        }
        Some(toml::Value::Integer(v)) => {
            let v = *v as u64;
            if v == 1 {
                true
            } else {
                errors.push(ConfigError::UnsupportedSchemaVersion {
                    file: file.to_string(),
                    found: v,
                    migration_hint: v > 1,
                });
                false
            }
        }
        Some(_) => {
            // Wrong type — the serde struct deserialization will catch this as a parse
            // error; don't double-report. Return false so structural validation is skipped.
            false
        }
    }
}

/// Run all structural validation rules (R-CUST-001 through R-CUST-017).
fn validate_structural(
    file: &str,
    file_stem: &str,
    config: &CustomerConfig,
    config_path: &Path,
    errors: &mut Vec<ConfigError>,
) {
    // R-CUST-001: Required fields present and non-empty.
    // org_id deserialized as Uuid so serde handles presence; check display_name.
    if config.display_name.is_empty() {
        errors.push(ConfigError::MissingField {
            file: file.to_string(),
            field: "display_name".to_string(),
        });
    }
    if config.org_slug.is_empty() {
        errors.push(ConfigError::MissingField {
            file: file.to_string(),
            field: "org_slug".to_string(),
        });
    }

    // R-CUST-002: org_slug must match filename stem (case-sensitive).
    if !config.org_slug.is_empty() && config.org_slug != file_stem {
        errors.push(ConfigError::SlugMismatch {
            file: file.to_string(),
            slug: config.org_slug.clone(),
            stem: file_stem.to_string(),
        });
    }

    // E-CFG-019 / CR-003 (W3-FIX-CODE-002): validate org_slug against OrgSlug pattern.
    //
    // Only check non-empty slugs (empty slugs are already reported as MissingField).
    // Uses `prism_core::tenant::OrgSlug::new` which enforces `^[a-zA-Z0-9_-]{1,64}$`.
    if !config.org_slug.is_empty() && prism_core::tenant::OrgSlug::new(&config.org_slug).is_err() {
        errors.push(ConfigError::InvalidOrgSlugPattern {
            file: file.to_string(),
            slug: config.org_slug.clone(),
        });
    }

    // R-CUST-003: org_id must be UUID v7 (version nibble = 7).
    let version = config.org_id.get_version_num();
    if version != 7 {
        errors.push(ConfigError::InvalidOrgIdVersion {
            file: file.to_string(),
            value: config.org_id.to_string(),
            found_version: version,
        });
    }

    // Validate each [[dtu]] block.
    for (idx, dtu) in config.dtu.iter().enumerate() {
        validate_dtu_block(file, config_path, idx, dtu, errors);
    }
}

/// Validate a single `[[dtu]]` block (R-CUST-004 through R-CUST-017).
fn validate_dtu_block(
    file: &str,
    config_path: &Path,
    idx: usize,
    dtu: &crate::schema::DtuBlock,
    errors: &mut Vec<ConfigError>,
) {
    let dtu_type = dtu.r#type.as_str();

    // Look up the type in the registry.
    let registry_entry = DTU_REGISTRY.iter().find(|e| e.type_name == dtu_type);

    match registry_entry {
        None => {
            // R-CUST-004: truly unknown type.
            errors.push(ConfigError::UnknownDtuType {
                file: file.to_string(),
                dtu_type: dtu_type.to_string(),
            });
            // Still validate mode and other fields below where possible.
        }
        Some(entry) if entry.test_only => {
            // R-CUST-013: test-only type in production config.
            errors.push(ConfigError::TestOnlyTypeInProduction {
                file: file.to_string(),
                dtu_type: dtu_type.to_string(),
            });
            // Fall through to also check BC-3.3.001 (E-CFG-017) for demo-server+shared (EC-008).
        }
        Some(_) => {}
    }

    // R-CUST-009: mode must be "shared" or "client".
    let mode = dtu.mode.as_str();
    if mode != "shared" && mode != "client" {
        errors.push(ConfigError::InvalidMode {
            file: file.to_string(),
            index: idx,
            value: mode.to_string(),
        });
        // Cannot do mode-dependent checks if mode is invalid.
        return;
    }

    // BC-3.3.001 (R-CUST-017): Security Telemetry type + mode=shared → E-CFG-017.
    if mode == "shared" {
        if let Some(entry) = registry_entry {
            if entry.is_security_telemetry {
                errors.push(ConfigError::SecurityTelemetrySharedMode {
                    file: file.to_string(),
                    dtu_type: dtu_type.to_string(),
                });
            }
        }
    }

    // R-CUST-005: credential_ref must have an allowed scheme prefix.
    if !ALLOWED_CRED_SCHEMES
        .iter()
        .any(|s| dtu.credential_ref.starts_with(s))
    {
        errors.push(ConfigError::InvalidCredentialRef {
            file: file.to_string(),
            field: "credential_ref".to_string(),
        });
    }

    // R-CUST-016: mode=shared with spec present → E-CFG-016.
    if mode == "shared" && dtu.spec.is_some() {
        errors.push(ConfigError::SharedModeWithSpec {
            file: file.to_string(),
            dtu_index: idx,
        });
    }

    // R-CUST-014 + R-CUST-015: mode=client requires spec; spec file must exist.
    if mode == "client" {
        match &dtu.spec {
            None => {
                errors.push(ConfigError::MissingClientSpec {
                    file: file.to_string(),
                    dtu_index: idx,
                });
            }
            Some(spec_path) => {
                // SEC-P2-002 (CWE-22): I/O-free pre-join checks MUST fire before
                // the existence check (resolved.exists()) so that a traversal attempt
                // targeting a non-existent path still emits E-CFG-018, not E-CFG-015.
                //
                // Ordering (BC-3.3.004 CWE-22 invariant):
                //   1. Pre-join: reject `..` components (no filesystem I/O)
                //   2. Pre-join: reject absolute paths (no filesystem I/O)
                //   3. Post-join: resolved.exists() check
                //   4. Post-join: canonicalize() + prefix comparison (in validate_spec_path)
                let spec_as_path = Path::new(spec_path.as_str());

                // Step 1: reject `..` components (no I/O).
                use std::path::Component;
                for component in spec_as_path.components() {
                    if matches!(component, Component::ParentDir) {
                        errors.push(ConfigError::SpecPathTraversal {
                            file: config_path.to_path_buf(),
                            spec_path: spec_path.clone(),
                            message: "parent directory traversal (`..`) is not permitted"
                                .to_string(),
                        });
                        return; // skip R-CUST-015 to avoid double-reporting
                    }
                }

                // Step 2: reject absolute paths (no I/O).
                if spec_as_path.is_absolute() {
                    errors.push(ConfigError::SpecPathTraversal {
                        file: config_path.to_path_buf(),
                        spec_path: spec_path.clone(),
                        message: "absolute paths are not permitted".to_string(),
                    });
                    return; // skip R-CUST-015 to avoid double-reporting
                }

                // Steps 3 + 4: existence check, then symlink-escape check.
                let parent = config_path.parent().unwrap_or(Path::new("."));
                let resolved = parent.join(spec_as_path);

                if resolved.exists() {
                    match validate_spec_path(config_path, spec_path) {
                        Ok(_canonical) => {
                            // File is within the customers directory — all good.
                            // R-CUST-015 existence check is implicitly satisfied.
                        }
                        Err(e) => {
                            // Symlink escape or other post-join boundary violation.
                            errors.push(e);
                            return; // skip R-CUST-015 to avoid double-reporting
                        }
                    }
                } else {
                    // R-CUST-015: spec file does not exist.
                    errors.push(ConfigError::SpecFileNotFound {
                        file: file.to_string(),
                        spec_path: spec_path.clone(),
                    });
                }
            }
        }
    }

    // R-CUST-006, R-CUST-007, R-CUST-008: [dtu.data] sub-table validation.
    if let Some(data) = &dtu.data {
        // R-CUST-006: archetype must be in catalog.
        if let Some(archetype) = &data.archetype {
            if !VALID_ARCHETYPES.contains(&archetype.as_str()) {
                errors.push(ConfigError::UnknownArchetype {
                    file: file.to_string(),
                    value: archetype.clone(),
                });
            }
        }

        // R-CUST-008: scale must be a positive finite float.
        if let Some(scale) = data.scale {
            if !scale.is_finite() || scale <= 0.0 {
                let value_str = if scale.is_nan() {
                    "NaN".to_string()
                } else if scale.is_infinite() {
                    "infinite".to_string()
                } else {
                    format!("{scale}")
                };
                errors.push(ConfigError::InvalidScale {
                    file: file.to_string(),
                    value: value_str,
                });
            }
        }

        // R-CUST-007: seed is u64 so TOML parse already validates range; no further check needed.
        // (Negative values cause a TOML parse error before reaching here.)
    }
}

// ---------------------------------------------------------------------------
// AC-001..AC-004 (W3-FIX-SEC-003): path traversal rejection helper (CWE-22)
//
// Pre-join checks (no filesystem I/O):
//   AC-002 — absolute paths rejected via Path::is_absolute()
//   AC-001 — `..` components rejected via Path::components()
//
// Post-join checks (filesystem I/O):
//   AC-003 — relative within-tree paths accepted; returns canonical PathBuf
//   AC-004 — symlink escapes rejected via prefix check on canonicalized paths
//
// Returns `Ok(canonical_path)` when the path is safe.
// Returns `Err(ConfigError::SpecPathTraversal { .. })` on any violation.
// ---------------------------------------------------------------------------
/// CR-014: implementation detail; callers outside this crate should prefer the higher-level
/// `validate_dtu_block` path. `#[doc(hidden)]` prevents accidental stable-API coupling;
/// integration tests in `tests/path_traversal.rs` access it directly for unit coverage.
#[doc(hidden)]
pub fn validate_spec_path(
    config_path: &Path,
    spec_path: &str,
) -> Result<std::path::PathBuf, ConfigError> {
    use std::path::Component;

    let spec_as_path = Path::new(spec_path);

    // AC-002: reject absolute paths (platform-aware: handles `/` on Unix, drive
    // letters and UNC paths on Windows).
    if spec_as_path.is_absolute() {
        return Err(ConfigError::SpecPathTraversal {
            file: config_path.to_path_buf(),
            spec_path: spec_path.to_string(),
            message: "absolute paths are not permitted".to_string(),
        });
    }

    // AC-001: reject any path containing a `..` component.
    // Use Component iterator so this catches both literal `..` and `..` embedded
    // in longer paths, regardless of separator style.
    for component in spec_as_path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(ConfigError::SpecPathTraversal {
                file: config_path.to_path_buf(),
                spec_path: spec_path.to_string(),
                message: "parent directory traversal (`..`) is not permitted".to_string(),
            });
        }
    }

    // Resolve the spec path relative to the directory containing the config file.
    let parent = config_path.parent().unwrap_or(Path::new("."));
    let candidate = parent.join(spec_as_path);

    // Canonicalize both paths to resolve symlinks.  canonicalize() requires the
    // path to exist on disk; if it doesn't, map the IO error to SpecPathTraversal
    // (the caller is responsible for checking existence separately via R-CUST-015,
    // but a non-existent path cannot be safely validated for symlink escapes).
    let canonical_candidate =
        candidate
            .canonicalize()
            .map_err(|e| ConfigError::SpecPathTraversal {
                file: config_path.to_path_buf(),
                spec_path: spec_path.to_string(),
                message: format!("could not canonicalize path: {e}"),
            })?;

    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| ConfigError::SpecPathTraversal {
            file: config_path.to_path_buf(),
            spec_path: spec_path.to_string(),
            message: format!("could not canonicalize parent directory: {e}"),
        })?;

    // AC-004: reject symlink escapes — the resolved canonical path must still be
    // a descendant of the canonical parent directory.
    if !canonical_candidate.starts_with(&canonical_parent) {
        return Err(ConfigError::SpecPathTraversal {
            file: config_path.to_path_buf(),
            spec_path: spec_path.to_string(),
            message: "path resolves outside the allowed directory (symlink escape)".to_string(),
        });
    }

    Ok(canonical_candidate)
}
