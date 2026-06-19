# S-DEMO-ENRICHMENT-PIVOT-002 — AC-003 + AC-019
# Demonstrates: ThreatIntel WASM plugin val-lift fix + scenario IOC resolution
# AC-019: Val-lift fix tests (option-some, option-none, unexpected-val — F-001 CRIT closure)
# AC-003: plugin resolves scenario IOC as malicious (in-process canned fixture, no live DTU)
#
# Evidence form: test-execution capture

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-003-019-threatintel-val-lift.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-003-019-threatintel-val-lift.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-ENRICHMENT-PIVOT-002"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-019: Val-lift fix — option-some returns json_value, option-none returns ok_none,
# unexpected_val returns enrich_call_failed (all 3 sub-cases — F-003 rigor: real PluginRuntime path)
Type "cargo nextest run -p prism-spec-engine -E 'test(val_lift_fix)'"
Enter
Sleep 15s

# AC-003: ThreatIntel plugin resolves scenario IOC (in-process canned fixture)
Type "cargo nextest run -p prism-spec-engine -E 'test(threatintel_plugin_resolves_scenario_ioc_as_malicious)'"
Enter
Sleep 10s

Sleep 2s
