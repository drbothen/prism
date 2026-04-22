//! TrustLevelExt — extension methods for TrustLevel in the security context (BC-2.09.005).
//!
//! Stub: `unimplemented!()` bodies. Red Gate — tests must fail.

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
        unimplemented!("TrustLevel::wire_str — stub (Red Gate)")
    }

    fn is_safe_for_prose(&self) -> bool {
        unimplemented!("TrustLevel::is_safe_for_prose — stub (Red Gate)")
    }
}

/// Determine the `TrustLevel` for a tool based on its origin.
///
/// BC-2.09.005: sensor tools => `UntrustedExternal`; health/capabilities/cred mgmt => `Internal`.
pub fn trust_level_for_tool(tool_name: &str) -> TrustLevel {
    unimplemented!("trust_level_for_tool — stub (Red Gate)")
}
