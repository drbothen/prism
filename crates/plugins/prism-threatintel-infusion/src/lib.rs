//! ThreatIntel enrichment WASM plugin — S-DEMO-ENRICHMENT-PIVOT-002.
//!
//! Dispatches an IOC value to the correct prism-dtu-threatintel HTTP route based on
//! input type:
//! - IP: `GET /v3/ip/{input_value}?key={api_key}`
//! - domain: `GET /v3/domain/{input_value}?key={api_key}`
//! - hash: `GET /v3/hash/{input_value}?key={api_key}`
//!
//! Auth: `?key=<api_key>` query param; `api_key` resolved via `host::get_config("api_key")`.
//!
//! Response: 2xx JSON body → serialize as JSON string → return `Some(json_string)`.
//! Non-2xx or parse failure → return `None`.
//!
//! HTTP goes through host WIT import `host.http-request` (U9: WASM guests have no sockets).
//! The 30s timeout applies to the HOST reqwest client in host_functions.rs, not this crate.
//!
//! IOC classification:
//! - `is_ip_address`: 4 dot-separated octets parseable as u8, OR contains ':'
//! - `is_domain`: contains '.' but not an IP, length > 3
//! - `is_hash`: all hex chars, length 32 (MD5), 40 (SHA1), or 64 (SHA256)
//!
//! ## Architecture (WASM vs native)
//!
//! WASM target (`wasm32-wasip1`):
//!   - `host_impl` module contains the `wit_bindgen::generate!` call and the `Guest` impl.
//!   - WIT-generated host function wrappers (`http_request`, `get_config`) are used.
//!   - `export!(Plugin)` wires the Component Model ABI exports.
//!
//! Native target (`cargo check` / `cargo test` on host):
//!   - `host_impl` is gated `#[cfg(target_arch = "wasm32")]` and absent on native.
//!   - Classification helpers (`is_ip_address`, `is_domain`, `is_hash`) are `pub` and
//!     unit-testable on native without any WIT dependency.
//!
//! # Build
//! ```sh
//! cargo build --manifest-path crates/plugins/prism-threatintel-infusion/Cargo.toml \
//!   --target wasm32-wasip1 --release
//! ```
//! See Justfile recipe `build-plugin-threatintel-infusion`.

// ---------------------------------------------------------------------------
// WASM target: WIT bindings + Guest implementation + Component Model export wiring
// ---------------------------------------------------------------------------

/// WASM target: wit-bindgen-generated host function bindings AND Component Model export wiring.
///
/// The `wit_bindgen::generate!` macro reads `wit/prism-infusion-plugin.wit` and generates:
/// - Safe Rust wrappers for all host-imported functions (http-request, get-config)
/// - The `exports::prism::infusion_plugin::infusion_plugin::Guest` trait
///
/// `export!(Plugin)` wires the Component Model ABI for the infusion-plugin-world.
#[cfg(target_arch = "wasm32")]
mod host_impl {
    // wit-bindgen generates host function wrappers AND export trait from the WIT spec.
    wit_bindgen::generate!({
        world: "infusion-plugin-world",
        path: "wit",
    });

    /// The plugin implementation struct — implements the WIT Guest trait.
    pub struct Plugin;

    impl exports::prism::infusion_plugin::infusion_plugin::Guest for Plugin {
        fn name() -> String {
            // HIGH-1 fix (S-DEMO-ENRICHMENT-PIVOT-002): canonical plugin identity uses underscore
            // to match infusion_id in threatintel.infusion.toml → "threat_intel".
            // PluginRuntime keys loaded plugins by metadata.plugin_id (derived from name()),
            // so this MUST match the infusion_id that InfusionRegistry::load_spec_with_runtime
            // passes as plugin_id to PluginInfusionSource::new.
            "threat_intel".to_string()
        }

        fn version() -> String {
            // 1.0.2: Cargo.lock tracked for full-graph reproducibility, F-MCPRS-PRL10-OBS-001 human ruling
            "1.0.2".to_string()
        }

        fn enrich_single(input_value: String, input_type: String) -> Option<String> {
            let api_key = prism::infusion_plugin::host::get_config("api_key").unwrap_or_default();

            // Determine the DTU route based on input_type or IOC auto-classification.
            let route = if input_type == "ip" || super::is_ip_address(&input_value) {
                format!("/v3/ip/{}?key={}", input_value, api_key)
            } else if input_type == "domain" || super::is_domain(&input_value) {
                format!("/v3/domain/{}?key={}", input_value, api_key)
            } else if input_type == "hash" || super::is_hash(&input_value) {
                format!("/v3/hash/{}?key={}", input_value, api_key)
            } else {
                // Unclassifiable IOC type — no enrichment available.
                return None;
            };

            // Build the full URL using get_config("base_url") for the DTU endpoint.
            // Falls back to empty string if not configured (will fail HTTP call gracefully).
            let base_url = prism::infusion_plugin::host::get_config("base_url").unwrap_or_default();
            let url = format!("{}{}", base_url.trim_end_matches('/'), route);

            // Issue HTTP request via host WIT import (U9: WASM guests have no sockets).
            // wit-bindgen 0.51 generates borrowed-slice signatures: (&str, &str, &[(String,String)], Option<...>).
            // Pass empty headers as &[] (empty slice ref), NOT vec![] (owned Vec).
            let response =
                prism::infusion_plugin::host::http_request("GET", &url, &[], None);

            // Non-2xx → no enrichment.
            if response.status < 200 || response.status >= 300 {
                return None;
            }

            // Parse body as JSON and return as serialized string.
            match serde_json::from_slice::<serde_json::Value>(&response.body) {
                Ok(json_val) => match serde_json::to_string(&json_val) {
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }

        fn enrich_batch(inputs: Vec<String>, input_type: String) -> Vec<Option<String>> {
            inputs
                .iter()
                .map(|input| Self::enrich_single(input.clone(), input_type.clone()))
                .collect()
        }
    }

    export!(Plugin);
}

// ---------------------------------------------------------------------------
// IOC classification helpers — pub for native unit testing
// ---------------------------------------------------------------------------

/// Returns `true` if the value looks like an IPv4 or IPv6 address.
///
/// IPv4: exactly 4 dot-separated segments, each parseable as u8 (0-255).
/// IPv6 heuristic: contains ':' (all valid IPv6 addresses contain colons).
/// These functions are `pub` for native unit testing.
pub fn is_ip_address(value: &str) -> bool {
    // IPv6 heuristic: any address with a colon is IPv6.
    if value.contains(':') {
        return true;
    }
    // IPv4: exactly 4 dot-separated segments, each 0-255.
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() == 4 {
        return parts.iter().all(|p| p.parse::<u8>().is_ok());
    }
    false
}

/// Returns `true` if the value looks like a domain name.
///
/// Must contain '.', must NOT be an IP address, and must be longer than 3 characters.
pub fn is_domain(value: &str) -> bool {
    value.contains('.') && !is_ip_address(value) && value.len() > 3
}

/// Returns `true` if the value looks like an MD5 (32), SHA1 (40), or SHA256 (64) hash.
///
/// All characters must be hex digits (0-9, a-f, A-F).
pub fn is_hash(value: &str) -> bool {
    let len = value.len();
    (len == 32 || len == 40 || len == 64) && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ip_address_ipv4_valid() {
        assert!(is_ip_address("192.168.1.1"));
        assert!(is_ip_address("10.0.0.1"));
        assert!(is_ip_address("45.55.100.1"));
        assert!(is_ip_address("255.255.255.255"));
        assert!(is_ip_address("0.0.0.0"));
    }

    #[test]
    fn test_is_ip_address_ipv6_heuristic() {
        assert!(is_ip_address("::1"));
        assert!(is_ip_address("2001:db8::1"));
        assert!(is_ip_address("fe80::1%eth0"));
    }

    #[test]
    fn test_is_ip_address_rejects_non_ip() {
        assert!(!is_ip_address("evil.example.com"));
        assert!(!is_ip_address("google.com"));
        assert!(!is_ip_address("not-an-ip"));
        assert!(!is_ip_address("256.256.256.256")); // out of u8 range
        assert!(!is_ip_address("1.2.3")); // only 3 segments
    }

    #[test]
    fn test_is_domain_valid() {
        assert!(is_domain("evil.example.com"));
        assert!(is_domain("google.com"));
        assert!(is_domain("sub.domain.org"));
    }

    #[test]
    fn test_is_domain_rejects_ip() {
        assert!(!is_domain("192.168.1.1"));
        assert!(!is_domain("10.0.0.1"));
    }

    #[test]
    fn test_is_domain_rejects_short() {
        assert!(!is_domain("a.b")); // too short (3 chars)
        assert!(!is_domain("ab")); // no dot
    }

    #[test]
    fn test_is_hash_md5() {
        assert!(is_hash("d41d8cd98f00b204e9800998ecf8427e")); // 32 chars, all hex
    }

    #[test]
    fn test_is_hash_sha1() {
        assert!(is_hash("da39a3ee5e6b4b0d3255bfef95601890afd80709")); // 40 chars
    }

    #[test]
    fn test_is_hash_sha256() {
        assert!(is_hash(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        )); // 64 chars
    }

    #[test]
    fn test_is_hash_rejects_non_hex() {
        assert!(!is_hash("d41d8cd98f00b204e9800998ecf8427z")); // 'z' not hex
    }

    #[test]
    fn test_is_hash_rejects_wrong_length() {
        assert!(!is_hash("d41d8cd98f00b204")); // 16 chars — not MD5/SHA1/SHA256
    }
}
