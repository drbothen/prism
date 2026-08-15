# AC-WIRE-001 — Health-wire 4xx/401 returns reachable:true, auth_valid:false
#
# Proves that when a sensor returns HTTP 4xx (401/403), the check_sensor_health
# MCP wire JSON contains:
#   "reachable":true      — sensor is network-reachable
#   "auth_valid":false    — sensor rejected authentication
#   suggestion non-empty  — operator gets actionable guidance
#
# Two tests are shown:
#   test_sensor_health_wire_shape_403_reachable_auth_invalid  (RG-007, AC-WIRE-001)
#   test_sensor_health_wire_shape_401_auth_invalid_production_path  (production-path variant)
#
# Demo vehicle: cargo nextest run against prism-bin integration tests.
# Traces to: BC-2.08.002 §Postconditions EC-08-006 (HTTP 4xx → auth_valid:false)

Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-WIRE-001-health-401-auth-invalid.gif"
Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-WIRE-001-health-401-auth-invalid.webm"

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

Type "# AC-WIRE-001: 4xx sensor → reachable:true + auth_valid:false on MCP wire"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-bin --test defect_adapter_tls_xdome_live_001 -E 'test(wire_shape_403_reachable_auth_invalid) | test(wire_shape_401_auth_invalid_production_path)'"
Enter
Sleep 30s

Sleep 2s
