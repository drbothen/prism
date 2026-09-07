# VHS Demo — S-REL-AGENT-VERSION-001 AC-002
# Surface A: MCP serverInfo.version on the wire equals "1.0.0-beta.1" (NOT "0.1.0")
# Acceptance Criteria:
#   AC-002: Wire-level MCP initialize response shows serverInfo.version = "1.0.0-beta.1"
#   AC-003: PrismServer::new uses "0.0.0-test" default (proven via RG-001/RG-002 tests)
#   AC-004: get_info uses self.product_version (this demo shows the runtime result)
#   AC-005: boot.rs step-9 passes env!("PRISM_VERSION") as final arg
# Authority: ADR-064 §D4 §Surface A; CLAUDE.md §Conventions §Wire-shape assertion discipline
#
# The binary is pre-built with PRISM_BUILD_VERSION=1.0.0-beta.1 override.
# The helper script (mcp-wire-query.sh) starts prism stdio MCP server, sends
# an MCP initialize request (newline-delimited JSON-RPC), and captures the response.

Output docs/demo-evidence/S-REL-AGENT-VERSION-001/AC-002-mcp-serverinfo-wire.gif
Output docs/demo-evidence/S-REL-AGENT-VERSION-001/AC-002-mcp-serverinfo-wire.webm

Require python3

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

# === Demo: MCP initialize wire shape ===
# Start prism stdio server, send initialize request, show serverInfo.version
Type "python3 docs/demo-evidence/S-REL-AGENT-VERSION-001/mcp-wire-query.sh \\"
Enter
Type "  ./target/debug/prism /tmp/prism-demo-config"
Enter
Sleep 10s

# === Hold final frame ===
Sleep 2s
