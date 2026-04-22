//! OCSF schema types, loading, and downloading.
//!
//! The schema is sourced from the OCSF export API at
//! `https://schema.ocsf.io/export/schema`. The full export contains all event
//! classes and objects with inheritance fully resolved, eliminating the need to
//! implement OCSF's `extends` + `$include` + profile merging logic.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

/// The full OCSF schema export from `schema.ocsf.io/export/schema`.
///
/// Contains all event classes, objects, and type definitions for a specific
/// OCSF version. Classes are fully resolved — inherited attributes from
/// base classes and profiles are already merged.
#[derive(Debug, Deserialize)]
pub struct OcsfSchema {
    /// OCSF version string (e.g., `"1.7.0"`).
    pub version: String,

    /// All event classes keyed by name (e.g., `"authentication"`, `"security_finding"`).
    pub classes: BTreeMap<String, OcsfClass>,

    /// All object types keyed by name (e.g., `"user"`, `"network_endpoint"`).
    pub objects: BTreeMap<String, OcsfObject>,

    /// Primitive type definitions (e.g., `"string_t"`, `"integer_t"`).
    #[serde(default)]
    pub types: BTreeMap<String, serde_json::Value>,

    /// Base event definition (common to all event classes).
    #[serde(default)]
    pub base_event: serde_json::Value,
}

/// An OCSF event class (e.g., Authentication, Security Finding).
///
/// In the export schema, all inherited attributes from base classes and
/// profiles are already merged into the `attributes` map.
#[derive(Debug, Deserialize)]
pub struct OcsfClass {
    /// Snake_case class name (e.g., `"authentication"`).
    pub name: String,

    /// Unique class identifier (e.g., `3002` for Authentication).
    pub uid: u32,

    /// Human-readable class name (e.g., `"Authentication"`).
    pub caption: String,

    /// Class description.
    #[serde(default)]
    pub description: String,

    /// Parent class name (e.g., `"iam"`).
    #[serde(default)]
    pub extends: String,

    /// Category name (e.g., `"iam"`, `"findings"`, `"network"`).
    #[serde(default)]
    pub category: String,

    /// Category UID (e.g., `3` for IAM).
    #[serde(default)]
    pub category_uid: u32,

    /// Category display name.
    #[serde(default)]
    pub category_name: String,

    /// Active profiles (e.g., `["cloud", "host", "security_control"]`).
    #[serde(default)]
    pub profiles: Vec<String>,

    /// Fully-resolved attributes keyed by name. Sorted by `BTreeMap`.
    pub attributes: BTreeMap<String, OcsfAttribute>,
}

/// An OCSF object type (e.g., User, Network Endpoint).
#[derive(Debug, Deserialize)]
pub struct OcsfObject {
    /// Snake_case object name (e.g., `"user"`, `"network_endpoint"`).
    pub name: String,

    /// Human-readable name (e.g., `"User"`).
    pub caption: String,

    /// Object description.
    #[serde(default)]
    pub description: String,

    /// Parent object name (e.g., `"_entity"`).
    #[serde(default)]
    pub extends: Option<String>,

    /// Object attributes keyed by name. Sorted by `BTreeMap`.
    pub attributes: BTreeMap<String, OcsfAttribute>,

    /// Observable type number (e.g., `20` for Endpoint, `21` for User).
    #[serde(default)]
    pub observable: Option<u32>,
}

/// A single attribute in an event class or object.
#[derive(Debug, Deserialize)]
pub struct OcsfAttribute {
    /// OCSF type name (e.g., `"string_t"`, `"integer_t"`, `"object_t"`).
    #[serde(rename = "type")]
    pub type_name: String,

    /// Human-readable name (e.g., `"Activity ID"`).
    #[serde(default)]
    pub caption: String,

    /// Attribute description.
    #[serde(default)]
    pub description: String,

    /// Requirement level: `"required"`, `"recommended"`, or `"optional"`.
    #[serde(default)]
    pub requirement: Option<String>,

    /// Whether this attribute is an array (proto `repeated`).
    #[serde(default)]
    pub is_array: bool,

    /// For `object_t` attributes, the referenced object type name.
    /// May include extension prefix (e.g., `"win/win_service"`).
    #[serde(default)]
    pub object_type: Option<String>,

    /// Attribute group: `"primary"`, `"context"`, `"classification"`, `"occurrence"`.
    #[serde(default)]
    pub group: Option<String>,

    /// Sibling attribute name (e.g., `activity_id` has sibling `activity_name`).
    #[serde(default)]
    pub sibling: Option<String>,

    /// Profile that contributed this attribute (e.g., `"cloud"`, `"security_control"`).
    #[serde(default)]
    pub profile: Option<String>,

    /// Enum value definitions. Keys are either integer strings (`"0"`, `"1"`, `"99"`)
    /// for true proto enums, or string labels (`"GET"`, `"POST"`) for documented
    /// string values that should not become proto enums.
    #[serde(rename = "enum", default)]
    pub enum_values: Option<BTreeMap<String, OcsfEnumValue>>,

    /// Deprecation information.
    #[serde(rename = "@deprecated", default)]
    pub deprecated: Option<OcsfDeprecated>,
}

/// A single value in an OCSF enum definition.
#[derive(Debug, Deserialize)]
pub struct OcsfEnumValue {
    /// Human-readable enum variant name (e.g., `"Logon"`, `"Unknown"`).
    pub caption: String,

    /// Variant description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Deprecation metadata for an attribute.
#[derive(Debug, Deserialize)]
pub struct OcsfDeprecated {
    /// Deprecation message (e.g., `"Use the ancestry attribute instead."`).
    pub message: String,

    /// OCSF version since which the attribute is deprecated.
    #[serde(default)]
    pub since: String,
}

/// Load a cached OCSF schema from disk.
///
/// The file should contain the JSON output from `schema.ocsf.io/export/schema`.
pub fn load_schema(path: &Path) -> Result<OcsfSchema> {
    let content = std::fs::read_to_string(path).map_err(|e| Error::Read {
        path: path.to_path_buf(),
        source: e,
    })?;
    let schema: OcsfSchema = serde_json::from_str(&content)?;
    Ok(schema)
}

/// Download the OCSF schema export and save to disk.
///
/// Fetches from `{base_url}?version={version}` and validates the response
/// parses as a valid [`OcsfSchema`] before writing.
#[cfg(feature = "download")]
pub async fn download_schema(version: &str, output_path: &Path, base_url: &str) -> Result<()> {
    let url = format!("{base_url}?version={version}");
    eprintln!("Downloading OCSF schema v{version} from {url}");

    let response = reqwest::get(&url)
        .await
        .map_err(|e| Error::Download(format!("GET {url}: {e}")))?;

    if !response.status().is_success() {
        return Err(Error::Download(format!(
            "GET {url} returned {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| Error::Download(format!("reading response body: {e}")))?;

    // Validate before writing.
    let schema: OcsfSchema = serde_json::from_str(&body)
        .map_err(|e| Error::Schema(format!("downloaded schema is not valid OCSF JSON: {e}")))?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Write {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    std::fs::write(output_path, &body).map_err(|e| Error::Write {
        path: output_path.to_path_buf(),
        source: e,
    })?;

    eprintln!(
        "Saved OCSF v{} ({} classes, {} objects) to {}",
        schema.version,
        schema.classes.len(),
        schema.objects.len(),
        output_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal schema JSON for testing without network access.
    fn minimal_schema_json() -> String {
        r#"{
            "version": "1.7.0",
            "classes": {
                "authentication": {
                    "name": "authentication",
                    "uid": 3002,
                    "caption": "Authentication",
                    "category": "iam",
                    "category_uid": 3,
                    "attributes": {
                        "activity_id": {
                            "type": "integer_t",
                            "caption": "Activity ID",
                            "requirement": "required",
                            "enum": {
                                "0": {"caption": "Unknown"},
                                "1": {"caption": "Logon"},
                                "2": {"caption": "Logoff"}
                            }
                        },
                        "src_endpoint": {
                            "type": "object_t",
                            "caption": "Source Endpoint",
                            "object_type": "network_endpoint",
                            "requirement": "recommended"
                        },
                        "message": {
                            "type": "string_t",
                            "caption": "Message",
                            "requirement": "recommended"
                        },
                        "severity_id": {
                            "type": "integer_t",
                            "caption": "Severity ID",
                            "requirement": "required",
                            "enum": {
                                "0": {"caption": "Unknown"},
                                "1": {"caption": "Informational"},
                                "2": {"caption": "Low"},
                                "3": {"caption": "Medium"},
                                "4": {"caption": "High"},
                                "5": {"caption": "Critical"},
                                "6": {"caption": "Fatal"},
                                "99": {"caption": "Other"}
                            }
                        },
                        "time": {
                            "type": "timestamp_t",
                            "caption": "Event Time",
                            "requirement": "required"
                        }
                    }
                }
            },
            "objects": {
                "network_endpoint": {
                    "name": "network_endpoint",
                    "caption": "Network Endpoint",
                    "attributes": {
                        "ip": {
                            "type": "ip_t",
                            "caption": "IP Address"
                        },
                        "port": {
                            "type": "port_t",
                            "caption": "Port"
                        },
                        "hostname": {
                            "type": "hostname_t",
                            "caption": "Hostname"
                        }
                    }
                }
            },
            "types": {},
            "base_event": {}
        }"#
        .to_string()
    }

    #[test]
    fn parse_minimal_schema() {
        let schema: OcsfSchema = serde_json::from_str(&minimal_schema_json()).unwrap();
        assert_eq!(schema.version, "1.7.0");
        assert_eq!(schema.classes.len(), 1);
        assert_eq!(schema.objects.len(), 1);
    }

    #[test]
    fn parse_class_attributes() {
        let schema: OcsfSchema = serde_json::from_str(&minimal_schema_json()).unwrap();
        let auth = &schema.classes["authentication"];

        assert_eq!(auth.uid, 3002);
        assert_eq!(auth.category, "iam");
        assert_eq!(auth.attributes.len(), 5);

        let activity_id = &auth.attributes["activity_id"];
        assert_eq!(activity_id.type_name, "integer_t");
        assert!(activity_id.enum_values.is_some());

        let src_endpoint = &auth.attributes["src_endpoint"];
        assert_eq!(src_endpoint.type_name, "object_t");
        assert_eq!(
            src_endpoint.object_type.as_deref(),
            Some("network_endpoint")
        );
    }

    #[test]
    fn parse_deprecated_attributes() {
        let json = r#"{
            "version": "1.7.0",
            "classes": {},
            "objects": {
                "test": {
                    "name": "test",
                    "caption": "Test",
                    "attributes": {
                        "old_field": {
                            "type": "string_t",
                            "caption": "Old",
                            "@deprecated": {
                                "message": "Use new_field instead.",
                                "since": "1.4.0"
                            }
                        }
                    }
                }
            },
            "types": {},
            "base_event": {}
        }"#;
        let schema: OcsfSchema = serde_json::from_str(json).unwrap();
        let attr = &schema.objects["test"].attributes["old_field"];
        assert!(attr.deprecated.is_some());
        assert_eq!(attr.deprecated.as_ref().unwrap().since, "1.4.0");
    }
}
