//! Maps OCSF attribute types to Protocol Buffer type strings.
//!
//! # Type Mapping Table
//!
//! | OCSF type | Proto type | Notes |
//! |-----------|-----------|-------|
//! | OCSF type | Proto type | OCSF base type | Notes |
//! |-----------|-----------|----------------|-------|
//! | `boolean_t` | `bool` | primitive | |
//! | `integer_t` | `int32` | primitive | Signed 32-bit |
//! | `long_t` | `int64` | primitive | Signed 64-bit |
//! | `float_t` | `double` | primitive | 64-bit float |
//! | `string_t` | `string` | primitive | UTF-8 |
//! | `json_t` | `string` | primitive | NOT `google.protobuf.Struct` |
//! | `timestamp_t` | `int64` | `long_t` | Epoch milliseconds |
//! | `port_t` | `int32` | `integer_t` | Range 0-65535 |
//! | `datetime_t` | `string` | `string_t` | RFC 3339 format |
//! | `hostname_t` .. `reg_key_path_t` | `string` | `string_t` | All string-derived types |
//! | `object_t` | message ref | — | Handled by codegen module |
//! | Unknown types | `string` | — | Fallback |

/// Map an OCSF type name to a proto3 scalar type string.
///
/// Returns `None` for `object_t` — object references must be resolved
/// by the caller using the attribute's `object_type` field.
///
/// Returns `"string"` as a fallback for unrecognized types.
pub fn ocsf_to_proto_type(type_name: &str) -> Option<&'static str> {
    let proto = match type_name {
        // String family — all string-like OCSF types (base type: string_t).
        "string_t" | "hostname_t" | "ip_t" | "mac_t" | "url_t" | "email_t" | "file_path_t"
        | "file_name_t" | "file_hash_t" | "subnet_t" | "uuid_t" | "username_t"
        | "process_name_t" | "resource_uid_t" | "datetime_t" | "bytestring_t"
        | "reg_key_path_t" => "string",

        // json_t maps to string, NOT google.protobuf.Struct.
        // prost_types::Struct does not implement serde traits, breaking
        // #[derive(Serialize, Deserialize)] on generated Rust types.
        "json_t" => "string",

        // Integer family.
        "integer_t" | "port_t" => "int32",

        // Long / timestamp family.
        "long_t" | "timestamp_t" => "int64",

        // Float.
        "float_t" => "double",

        // Boolean.
        "boolean_t" => "bool",

        // Object references — the caller must handle these.
        "object_t" => return None,

        // Fallback: unknown types emit as string.
        _ => "string",
    };
    Some(proto)
}

/// Convert a snake_case OCSF name to PascalCase for proto message names.
///
/// Handles extension-prefixed names by stripping the prefix:
/// - `"network_endpoint"` → `"NetworkEndpoint"`
/// - `"win/win_service"` → `"WinService"` (prefix stripped)
pub fn to_pascal_case(s: &str) -> String {
    // Strip extension prefix (e.g., "win/win_service" → "win_service").
    let name = s.rsplit('/').next().unwrap_or(s);
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

/// Convert a name to SCREAMING_SNAKE_CASE for proto enum type names.
///
/// - `"authentication"` → `"AUTHENTICATION"`
/// - `"security_finding"` → `"SECURITY_FINDING"`
pub fn to_screaming_snake(s: &str) -> String {
    s.to_uppercase()
}

/// Sanitize an OCSF object name for use as a lookup key.
///
/// Strips extension prefixes: `"win/win_service"` → `"win_service"`.
pub fn sanitize_object_name(s: &str) -> String {
    s.rsplit('/').next().unwrap_or(s).to_string()
}

/// Convert a human-readable caption to a SCREAMING_SNAKE enum variant name.
///
/// - `"Logon"` → `"LOGON"`
/// - `"Service Ticket Request"` → `"SERVICE_TICKET_REQUEST"`
/// - `"TLP:AMBER+STRICT"` → `"TLP_AMBER_STRICT"`
///
/// Non-alphanumeric characters are replaced with `_`, consecutive
/// underscores are collapsed, and leading/trailing underscores are trimmed.
pub fn to_enum_variant_name(caption: &str) -> String {
    caption
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_uppercase().next().unwrap_or(c)
            } else {
                '_'
            }
        })
        .collect::<String>()
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_type_mapping() {
        assert_eq!(ocsf_to_proto_type("string_t"), Some("string"));
        assert_eq!(ocsf_to_proto_type("integer_t"), Some("int32"));
        assert_eq!(ocsf_to_proto_type("long_t"), Some("int64"));
        assert_eq!(ocsf_to_proto_type("timestamp_t"), Some("int64"));
        assert_eq!(ocsf_to_proto_type("float_t"), Some("double"));
        assert_eq!(ocsf_to_proto_type("boolean_t"), Some("bool"));
        assert_eq!(ocsf_to_proto_type("port_t"), Some("int32"));
    }

    #[test]
    fn all_string_derived_types() {
        // Every type derived from string_t in the OCSF spec.
        for t in &[
            "hostname_t",
            "ip_t",
            "mac_t",
            "url_t",
            "email_t",
            "uuid_t",
            "file_path_t",
            "file_name_t",
            "file_hash_t",
            "subnet_t",
            "username_t",
            "process_name_t",
            "resource_uid_t",
            "datetime_t",
            "bytestring_t",
            "reg_key_path_t",
        ] {
            assert_eq!(
                ocsf_to_proto_type(t),
                Some("string"),
                "expected string for {t}"
            );
        }
    }

    #[test]
    fn timestamp_is_int64_not_string() {
        // OCSF timestamp_t is epoch milliseconds (base type: long_t).
        // Must be int64, NOT string and NOT google.protobuf.Timestamp.
        assert_eq!(ocsf_to_proto_type("timestamp_t"), Some("int64"));
    }

    #[test]
    fn datetime_is_string() {
        // OCSF datetime_t is RFC 3339 string (base type: string_t).
        // Must be string, NOT int64. This is different from timestamp_t.
        assert_eq!(ocsf_to_proto_type("datetime_t"), Some("string"));
    }

    #[test]
    fn json_t_maps_to_string() {
        assert_eq!(ocsf_to_proto_type("json_t"), Some("string"));
    }

    #[test]
    fn object_t_returns_none() {
        assert_eq!(ocsf_to_proto_type("object_t"), None);
    }

    #[test]
    fn unknown_type_falls_back_to_string() {
        assert_eq!(ocsf_to_proto_type("some_future_type"), Some("string"));
    }

    #[test]
    fn pascal_case_conversion() {
        assert_eq!(to_pascal_case("network_endpoint"), "NetworkEndpoint");
        assert_eq!(to_pascal_case("user"), "User");
        assert_eq!(to_pascal_case("auth_factor"), "AuthFactor");
        assert_eq!(to_pascal_case("cis_csc"), "CisCsc");
    }

    #[test]
    fn pascal_case_strips_extension_prefix() {
        assert_eq!(to_pascal_case("win/win_service"), "WinService");
        assert_eq!(to_pascal_case("win/reg_key"), "RegKey");
    }

    #[test]
    fn screaming_snake_conversion() {
        assert_eq!(to_screaming_snake("authentication"), "AUTHENTICATION");
        assert_eq!(to_screaming_snake("security_finding"), "SECURITY_FINDING");
    }

    #[test]
    fn enum_variant_name_conversion() {
        assert_eq!(to_enum_variant_name("Logon"), "LOGON");
        assert_eq!(
            to_enum_variant_name("Service Ticket Request"),
            "SERVICE_TICKET_REQUEST"
        );
        assert_eq!(to_enum_variant_name("TLP:AMBER+STRICT"), "TLP_AMBER_STRICT");
        assert_eq!(to_enum_variant_name("Unknown"), "UNKNOWN");
        assert_eq!(to_enum_variant_name("Other"), "OTHER");
    }

    #[test]
    fn sanitize_object_name_strips_prefix() {
        assert_eq!(sanitize_object_name("win/win_service"), "win_service");
        assert_eq!(sanitize_object_name("user"), "user");
    }
}
