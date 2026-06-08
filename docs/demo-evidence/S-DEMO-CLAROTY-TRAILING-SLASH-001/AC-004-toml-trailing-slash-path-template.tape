# AC-004 — NormalizePathLayer in clone.rs + spec-engine parity test (Gap-CL-001 CLOSED)
#
# Demonstrates that AC-004 is satisfied:
#   (a) NormalizePathLayer::trim_trailing_slash() is applied at the outer service level
#       in crates/prism-dtu-claroty/src/clone.rs (both TLS and plain-HTTP serve paths).
#   (b) The spec-engine parity test confirms claroty.sensor.toml round-trips without error.
#
# Traces to: BC-2.16.013 v1.25 §Postconditions §1; Gap-CL-001 CLOSED.

Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-004-toml-trailing-slash-path-template.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-004-toml-trailing-slash-path-template.webm"

Require cargo
Require grep

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-TRAILING-SLASH-001"
Enter
Sleep 1s
Show

Type "# AC-004: NormalizePathLayer in clone.rs (trailing-slash fix site)"
Enter
Sleep 400ms

Type "grep -n 'NormalizePathLayer\\|trim_trailing_slash' crates/prism-dtu-claroty/src/clone.rs"
Enter
Sleep 2s

Type "# AC-004: spec-engine parity — claroty.sensor.toml loads without error"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-spec-engine -E 'test(claroty)' 2>&1 | tail -8"
Enter
Sleep 60s

Sleep 2s
