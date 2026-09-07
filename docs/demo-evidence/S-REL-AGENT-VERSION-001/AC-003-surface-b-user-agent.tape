# VHS Demo — S-REL-AGENT-VERSION-001 AC-003 (Surface B)
# Surface B: prism-spec-engine build_http_client_with_timeout emits correct User-Agent
# Acceptance Criteria:
#   AC-006: prism-spec-engine/build.rs + src/version_resolver.rs exist with D2 fallback chain
#   AC-007: pipeline.rs build_http_client_with_timeout uses env!("PRISM_VERSION")
# Test: test_infusion_http_client_sends_prism_user_agent (wiremock assertion — wire-level UA)
# Authority: ADR-064 §D4 §Surface B; ADR-050 §D6 normative form: prism/{PRISM_VERSION}
#
# The wiremock integration test starts an actual HTTP server and verifies the exact
# User-Agent header sent by prism-spec-engine's outbound HTTP client (the Surface B
# wire-level assertion). PRISM_BUILD_VERSION=1.0.0-beta.1 at build time means
# the UA header is "prism/1.0.0-beta.1".

Output docs/demo-evidence/S-REL-AGENT-VERSION-001/AC-003-surface-b-user-agent.gif
Output docs/demo-evidence/S-REL-AGENT-VERSION-001/AC-003-surface-b-user-agent.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-REL-AGENT-VERSION-001 && clear"
Enter
Sleep 2s
Show

# === Demo: Surface B wiremock test (wire-level User-Agent assertion) ===
Type "PRISM_BUILD_VERSION=1.0.0-beta.1 cargo nextest run -p prism-spec-engine \\"
Enter
Type "  -E 'test(test_infusion_http_client_sends_prism_user_agent)'"
Enter
Sleep 60s

# === Hold final frame ===
Sleep 2s
