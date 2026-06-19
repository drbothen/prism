# S-DEMO-ENRICHMENT-PIVOT-002 — AC-001 + AC-002
# Demonstrates: threatintel.infusion.toml (plugin-type) and nvd.infusion.toml (http_lookup)
# each parse and register their expected UDF descriptors.
#
# Evidence form: test-execution capture (cargo nextest)
# Tests: test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs (AC-001)
#        test_enrichment_pivot_002_nvd_toml_loads_as_http_lookup_and_registers_3_udfs (AC-002)
# These run in-process against fixture infusion TOML files — no live services needed.

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-001-002-toml-loading.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-001-002-toml-loading.webm

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

# AC-001: threatintel.infusion.toml loads as plugin-type — 3 UDF descriptors registered
Type "cargo nextest run -p prism-spec-engine -E 'test(threatintel_toml_loads_and_registers_3_udfs)'"
Enter
Sleep 15s

# AC-002: nvd.infusion.toml loads as http_lookup-type — 3 UDF descriptors registered
Type "cargo nextest run -p prism-spec-engine -E 'test(nvd_toml_loads_as_http_lookup_and_registers_3_udfs)'"
Enter
Sleep 15s

Sleep 2s
