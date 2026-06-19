# S-DEMO-ENRICHMENT-PIVOT-002 — HttpLookupSource operations + cleanup + spawn_blocking
# AC-016: HttpLookupSource enrich_single calls url_template; extracts response_path;
#         returns None on path not found; returns Err on non-2xx (wiremock mock — no real NVD)
# AC-018: prism-nvd-infusion crate removed (filesystem assertion)
# AC-020: spawn_blocking wraps real InfusionAsyncUdf path (tokio runtime test — F-004 rigor)
#
# Evidence form: test-execution capture

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-016-018-020-http-lookup-ops.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-016-018-020-http-lookup-ops.webm

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

# AC-016: HttpLookupSource enrich_single — wiremock tests (in-process mock HTTP, no real NVD)
Type "cargo nextest run -p prism-spec-engine -E 'test(http_lookup_source_returns_none_on_path_not_found) | test(http_lookup_source_returns_err_on_non_2xx)'"
Enter
Sleep 40s

# AC-018: prism-nvd-infusion crate removed (fs assertion — ADR-040 D9)
Type "cargo nextest run -p prism-spec-engine -E 'test(nvd_plugin_crate_removed)'"
Enter
Sleep 10s

# AC-020: spawn_blocking wraps real InfusionAsyncUdf path (prism-query tokio runtime — F-004)
Type "cargo nextest run -p prism-query -E 'test(enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking)'"
Enter
Sleep 15s

Sleep 2s
