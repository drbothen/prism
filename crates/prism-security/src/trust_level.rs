//! TrustLevelExt — extension methods for TrustLevel in the security context (BC-2.09.005).
//!
//! Provides wire-format strings and safety predicates for use in MCP response assembly.

use prism_core::TrustLevel;

/// Extension trait providing security-context helpers for `TrustLevel`.
pub trait TrustLevelExt {
    /// Returns the canonical wire-format string for `_meta.trust_level`.
    ///
    /// BC-2.09.005: exactly `"untrusted_external"` or `"internal"`.
    fn wire_str(&self) -> &'static str;

    /// Returns `true` if this level permits the LLM to treat values as safe prose.
    fn is_safe_for_prose(&self) -> bool;
}

impl TrustLevelExt for TrustLevel {
    fn wire_str(&self) -> &'static str {
        self.as_str()
    }

    fn is_safe_for_prose(&self) -> bool {
        matches!(self, TrustLevel::Internal)
    }
}

/// Determine the `TrustLevel` for a tool based on its origin.
///
/// BC-2.09.005: sensor tools => `UntrustedExternal`; health/capabilities/cred mgmt => `Internal`.
///
/// Convention: internal tool names contain one of the known internal prefixes
/// (`check_`, `list_capabilities`, `__error__`). Everything else is sensor data.
pub fn trust_level_for_tool(tool_name: &str) -> TrustLevel {
    // Internal tool name patterns
    let internal_patterns = [
        "check_",
        "list_capabilities",
        "__error__",
        "list_credential",
        "store_credential",
        "delete_credential",
        "list_sensors",
        "get_capabilities",
    ];
    for pat in &internal_patterns {
        if tool_name.starts_with(pat) || tool_name == *pat {
            return TrustLevel::Internal;
        }
    }
    TrustLevel::UntrustedExternal
}
