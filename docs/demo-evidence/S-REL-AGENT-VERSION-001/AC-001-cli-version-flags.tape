# VHS Demo — S-REL-AGENT-VERSION-001 AC-001 / AC-004
# Surface A: CLI version flags emit correct product version (1.0.0-beta.1)
# Acceptance Criteria:
#   AC-001 + AC-004: prism --version and prism version both print "prism 1.0.0-beta.1"
# Authority: ADR-064 §D4 §Surface A — env!("PRISM_VERSION") injected at build time by
#            prism-bin/build.rs (D2) → threaded via boot.rs step 9 → PrismServer::get_info
#
# The binary is pre-built with PRISM_BUILD_VERSION=1.0.0-beta.1 before recording.

Output docs/demo-evidence/S-REL-AGENT-VERSION-001/AC-001-cli-version-flags.gif
Output docs/demo-evidence/S-REL-AGENT-VERSION-001/AC-001-cli-version-flags.webm

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

# === Hidden Setup: navigate to worktree root ===
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-REL-AGENT-VERSION-001 && clear"
Enter
Sleep 2s
Show

# === Demo: prism --version ===
Type "./target/debug/prism --version"
Enter
Sleep 2s

# === Demo: prism version subcommand ===
Type "./target/debug/prism version"
Enter
Sleep 2s

# === Hold final frame ===
Sleep 2s
