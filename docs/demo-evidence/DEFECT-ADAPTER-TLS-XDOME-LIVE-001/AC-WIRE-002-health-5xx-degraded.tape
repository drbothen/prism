# AC-WIRE-002 — Health-wire 5xx returns reachable:true, auth_valid:true, error:"service_unavailable"
# AC-WIRE-003 / AC-WIRE-004 — Fleet envelope overall_status:"partial", healthy_count:0
#
# Proves that when a sensor returns HTTP 503, the check_sensor_health MCP wire contains
# the Degraded (not Down) classification:
#   "reachable":true              — sensor IS network-reachable
#   "auth_valid":true             — credentials appear valid (HTTP responded)
#   "error":"service_unavailable" — service-level failure, not network failure
#   "overall_status":"partial"    — fleet view reflects degraded, not unhealthy
#   "healthy_count":0             — Degraded sensor not counted as fully healthy
#
# Two wire-shape tests are run:
#   test_BC_2_08_002_degraded_reachable_wire_shape     (RG-019, AC-WIRE-002)
#   test_BC_2_08_002_degraded_envelope_summary_matches_overall_status  (RG-020, AC-WIRE-003+004)
#
# Demo vehicle: cargo nextest run against prism-bin integration tests.
# Traces to: BC-2.08.002 §Postconditions EC-08-009 (HS-007 re-gate)
#   check_one MUST serialize Degraded as reachable:true + auth_valid:true + error:"service_unavailable"
#   aggregate MUST use is_fully_healthy() predicate (includes && error.is_none())

Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-WIRE-002-health-5xx-degraded.gif"
Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-WIRE-002-health-5xx-degraded.webm"

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

Type "# AC-WIRE-002/003/004: 503 sensor → Degraded wire (reachable:true, healthy_count:0)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-bin --test defect_adapter_tls_xdome_live_001 -E 'test(BC_2_08_002_degraded)'"
Enter
Sleep 30s

Sleep 2s
