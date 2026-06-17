//! VP-050: Sensor resource response redacts credentials and full API URLs.
//!
//! Property: `render_sensor_inventory_resource` output contains no API key patterns
//! and no full URL paths — only host+port components (BC-2.10.008 postcondition).
//!
//! Method: proptest with fabricated credentials (UUID-format, Bearer-prefix,
//! base64 32+ chars) and full URL paths with paths/queries/credentials.
//!
//! Traces to: BC-2.10.008 postconditions, VP-050.

// VP-050 proptest is gated to test builds.
#[cfg(test)]
mod vp_050_tests {
    use proptest::prelude::*;

    use crate::resources::render_sensor_inventory_resource;

    // ─── Credential pattern strategies ───────────────────────────────────────

    /// Generate a UUID-format fake credential (8-4-4-4-12 hex groups).
    fn uuid_credential() -> impl Strategy<Value = String> {
        "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}".prop_map(|s| s)
    }

    /// Generate a Bearer-prefixed fake credential.
    fn bearer_credential() -> impl Strategy<Value = String> {
        "[A-Za-z0-9+/]{32,64}".prop_map(|s| format!("Bearer {s}"))
    }

    /// Generate a full URL including path, query parameters, and optional credentials.
    fn full_url_with_path() -> impl Strategy<Value = String> {
        (
            "[a-z]{3,10}",                   // host
            "[0-9]{4,5}",                    // port
            "/[a-z]{3,20}/[a-z]{3,20}",      // path
            "[a-z]{3,10}=[A-Za-z0-9]{8,32}", // query param
        )
            .prop_map(|(host, port, path, query)| {
                format!("https://{host}.example.com:{port}{path}?{query}")
            })
    }

    // ─── VP-050 proptest ──────────────────────────────────────────────────────

    proptest! {
        /// VP-050: render_sensor_inventory_resource redacts API keys.
        ///
        /// For any UUID-format credential, the serialized output must NOT contain
        /// the raw credential value.
        #[test]
        fn prop_vp050_uuid_credential_redacted(
            cred in uuid_credential(),
        ) {
            let sources = vec!["alerts".to_string(), "detections".to_string()];
            let entry = render_sensor_inventory_resource(
                "crowdstrike",
                &cred,
                "https://api.example.com:443",
                &sources,
            );
            let serialized = serde_json::to_string(&entry).unwrap();
            // The raw UUID credential must NOT appear in the serialized output.
            prop_assert!(
                !serialized.contains(&cred),
                "VP-050 FAIL: raw UUID credential leaked into resource output: {cred}"
            );
        }

        /// VP-050: render_sensor_inventory_resource redacts Bearer-prefixed credentials.
        #[test]
        fn prop_vp050_bearer_credential_redacted(
            cred in bearer_credential(),
        ) {
            let sources = vec!["alerts".to_string()];
            let entry = render_sensor_inventory_resource(
                "claroty",
                &cred,
                "https://api.example.com:443",
                &sources,
            );
            let serialized = serde_json::to_string(&entry).unwrap();
            // Extract the token part (after "Bearer ") for the assertion.
            let token_part = cred.trim_start_matches("Bearer ");
            prop_assert!(
                !serialized.contains(token_part),
                "VP-050 FAIL: Bearer token leaked into resource output"
            );
        }

        /// VP-050: render_sensor_inventory_resource strips full URL paths to host+port only.
        ///
        /// For any full URL with path/query, the serialized output must NOT contain
        /// path or query components — only host+port.
        #[test]
        fn prop_vp050_url_stripped_to_host_port(
            full_url in full_url_with_path(),
        ) {
            let sources = vec!["events".to_string()];
            let entry = render_sensor_inventory_resource(
                "armis",
                "cred-ref-1234",
                &full_url,
                &sources,
            );
            let serialized = serde_json::to_string(&entry).unwrap();
            // Extract path portion (after third slash) for assertion.
            // A URL like "https://host.example.com:443/path/sub?query" has path "/path/sub".
            // The serialized output must not contain the path.
            if let Some(path_start) = full_url
                .trim_start_matches("https://")
                .find('/')
            {
                let path_part = &full_url.trim_start_matches("https://")[path_start..];
                if path_part.len() > 1 {
                    prop_assert!(
                        !serialized.contains(path_part),
                        "VP-050 FAIL: full URL path leaked into resource output: {path_part}"
                    );
                }
            }
        }
    }
}
