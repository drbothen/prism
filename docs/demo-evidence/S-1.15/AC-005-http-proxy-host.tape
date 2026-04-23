Output docs/demo-evidence/S-1.15/AC-005-http-proxy-host.gif
Output docs/demo-evidence/S-1.15/AC-005-http-proxy-host.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-5: HTTP proxy host function — known-issue: test has hardcoded panic! (test-writer bug).
# Traces to: BC-2.17.002 / INV-PLUGIN-002
# NOTE: This test is the 1/23 intentional-failure flagged by the implementer.

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-5: HTTP proxy routed via host reqwest::Client"
Enter
Sleep 300ms
Type "# NOTE: Test has hardcoded panic! (test-writer bug — triaged to PR manager)"
Enter
Sleep 500ms
Type "# Demonstrating the host_http_request function via EC-17-007 allowlist tests:"
Enter
Sleep 300ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed --nocapture 2>&1"
Enter
Sleep 3s
Sleep 1s
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_002_ec17_url_not_in_allowlist_returns_403 --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
