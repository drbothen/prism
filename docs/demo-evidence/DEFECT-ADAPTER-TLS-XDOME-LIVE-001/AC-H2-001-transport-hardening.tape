# AC-H2-001 — reqwest http2 feature active on reqwest dependency node
# AC-UA-001 — build_http_client_with_custom_timeout sets User-Agent: prism/<version>
#
# Proves the ADR-050 transport hardening is in place:
#   - reqwest's http2 feature is active (h2 dependency inside [[package]]/name="reqwest" block)
#   - build_http_client_with_custom_timeout sends User-Agent: prism/<version> on outbound requests
#
# Two tests are shown:
#   test_reqwest_http2_feature_active   (RG-008, AC-H2-001)
#     Asserts h2 appears inside the reqwest package block of Cargo.lock
#     NOTE: whole-file grep is invalid (h2 was already transitive via hyper);
#     this test scopes the check to the reqwest package dependency block only
#   test_build_http_client_sends_user_agent_header  (RG-006, AC-UA-001)
#     Starts a wiremock server, sends a request via build_http_client_with_custom_timeout,
#     asserts User-Agent header starts with "prism/"
#
# Demo vehicle: cargo nextest run against prism-bin integration tests.
# Traces to: BC-2.16.002 HTTP Client Compliance postcondition (ADR-050 §D5/§D6)
#   ADR-050 §D5: http2 feature MUST be in production [dependencies] reqwest entries
#   ADR-050 §D6: ALL outbound third-party HTTP client builders MUST call .user_agent(...)

Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-H2-001-transport-hardening.gif"
Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-H2-001-transport-hardening.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
Enter
Sleep 1s
Show

Type "# AC-H2-001/UA-001: ADR-050 transport hardening — http2 feature active + User-Agent set"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-bin --test defect_adapter_tls_xdome_live_001 -E 'test(reqwest_http2_feature_active)'"
Enter
Sleep 30s

Type "cargo nextest run -p prism-bin -E 'test(test_build_http_client_sends_user_agent_header)'"
Enter
Sleep 30s

Sleep 2s
