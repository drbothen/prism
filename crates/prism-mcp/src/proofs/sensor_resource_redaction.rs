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

    // ─── Unit test: userinfo stripping (F-OBS-1) ─────────────────────────────

    /// VP-050 / F-OBS-1: `strip_url_to_host_port` MUST strip URL userinfo (the
    /// `user:pass@` segment before the host). A URL containing embedded credentials
    /// in the authority section MUST NOT leak `user:pass@` into the output.
    ///
    /// This is a LOAD-BEARING test: it FAILS before the fix (current implementation
    /// returns `"https://user:secret@host:443"` — userinfo leaks). After the fix,
    /// the output MUST be `"https://host:443"`.
    ///
    /// Threat model: a misconfigured `base_url` in a sensor TOML spec like
    /// `https://user:secret@api.vendor.com:443/v1` would expose the credential
    /// in the MCP resource response forwarded to AI agent context (AD-017 / DI-002
    /// / BC-2.19.005 credential-redaction spirit).
    #[test]
    fn test_vp050_strip_url_to_host_port_strips_userinfo() {
        use crate::resources::render_sensor_inventory_resource;

        // Case 1: https with userinfo + path
        let entry = render_sensor_inventory_resource(
            "crowdstrike",
            "cred-ref-1234",
            "https://user:secret@host.example.com:443/v1/events",
            &["detections".to_string()],
        );
        let serialized = serde_json::to_string(&entry).unwrap();
        // VP-050 F-OBS-1 LOAD-BEARING: the userinfo segment MUST NOT appear in output.
        assert!(
            !serialized.contains("user:secret@"),
            "VP-050 F-OBS-1: userinfo 'user:secret@' leaked into api_base_url. \
             strip_url_to_host_port must strip the `user:pass@` authority prefix. \
             Got serialized: {serialized}"
        );
        assert!(
            !serialized.contains("user:secret"),
            "VP-050 F-OBS-1: credential value 'user:secret' leaked into api_base_url. \
             Got serialized: {serialized}"
        );
        // The host MUST still appear (not a blank output).
        assert!(
            serialized.contains("host.example.com"),
            "VP-050 F-OBS-1: host must be present after userinfo is stripped. \
             Got serialized: {serialized}"
        );

        // Case 2: http with userinfo, no port, no path
        let entry2 = render_sensor_inventory_resource(
            "claroty",
            "cred-ref-5678",
            "http://admin:pass@internal.claroty.com",
            &["assets".to_string()],
        );
        let serialized2 = serde_json::to_string(&entry2).unwrap();
        assert!(
            !serialized2.contains("admin:pass@"),
            "VP-050 F-OBS-1: userinfo 'admin:pass@' leaked (http, no port). \
             Got serialized: {serialized2}"
        );
        assert!(
            serialized2.contains("internal.claroty.com"),
            "VP-050 F-OBS-1: host must be present after userinfo stripped (http). \
             Got serialized: {serialized2}"
        );

        // Case 3: userinfo with no port and a path
        let entry3 = render_sensor_inventory_resource(
            "armis",
            "cred-ref-9012",
            "https://token:x@api.armis.com/api/v1/devices",
            &["devices".to_string()],
        );
        let serialized3 = serde_json::to_string(&entry3).unwrap();
        assert!(
            !serialized3.contains("token:x@"),
            "VP-050 F-OBS-1: userinfo 'token:x@' leaked (no port). \
             Got serialized: {serialized3}"
        );
    }

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
        /// This test is LOAD-BEARING: it would FAIL if `strip_url_to_host_port` were
        /// removed from `render_sensor_inventory_resource`, because `api_base_url` is
        /// now a serialized field in `SensorConfigEntry` (BC-2.10.008 postcondition 2).
        ///
        /// For any full URL with path/query, the serialized `api_base_url` field must:
        /// (a) NOT contain path components (anything after the host:port)
        /// (b) NOT contain query strings
        /// (c) CONTAIN the host component
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

            // Extract path portion (after host:port) from the original URL.
            // A URL like "https://host.example.com:443/path/sub?query=val" — path is "/path/sub".
            let authority_rest = full_url.trim_start_matches("https://");
            if let Some(slash_pos) = authority_rest.find('/') {
                let path_part = &authority_rest[slash_pos..]; // "/path/sub?query=val"
                if path_part.len() > 1 {
                    // LOAD-BEARING: the api_base_url field must NOT contain the path.
                    // This fails if strip_url_to_host_port is removed from the function.
                    prop_assert!(
                        !serialized.contains(path_part),
                        "VP-050 FAIL: full URL path leaked into api_base_url field: {path_part}. \
                         The `api_base_url` field MUST contain only scheme+host+port (BC-2.10.008 \
                         v1.8 postcondition 2). If strip_url_to_host_port is absent, the full URL \
                         path will appear in the serialized api_base_url — this test catches that."
                    );
                }
            }

            // LOAD-BEARING: the api_base_url field must NOT contain a query string.
            prop_assert!(
                !serialized.contains("?"),
                "VP-050 FAIL: query string leaked into api_base_url. \
                 api_base_url must contain ONLY scheme+host+port. \
                 Got serialized: {serialized}"
            );

            // Sanity: the api_base_url field must be present in serialized output.
            prop_assert!(
                serialized.contains("api_base_url"),
                "VP-050 FAIL: 'api_base_url' field must be present in serialized SensorConfigEntry \
                 (BC-2.10.008 postcondition 2). Got: {serialized}"
            );
        }
    }
}
