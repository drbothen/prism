# S-DEMO-ENRICHMENT-PIVOT-002 — Security Gate ACs
# AC-007: UDF name rejects SQL injection chars + leading digit + accepts valid identifiers
# AC-008: PluginInfusionSource.config is pub(crate) not pub (CWE-200)
# AC-009: SandboxViolation URL NOT in WARN log (CWE-209)
# AC-011: plugin_ref path traversal rejected (CWE-22); within-dir accepted
# AC-012: load_all error does not leak absolute filesystem path (CWE-209)
#
# Evidence form: test-execution capture (all unit tests, no external deps)

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-007-008-009-011-012-security-gates.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-007-008-009-011-012-security-gates.webm

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

# AC-007: UDF name identifier validation (SQL injection + leading digit + valid)
Type "cargo nextest run -p prism-spec-engine -E 'test(sec001_udf_name)'"
Enter
Sleep 12s

# AC-008 + AC-009: config encapsulation (CWE-200) + SandboxViolation URL redaction (CWE-209)
Type "cargo nextest run -p prism-spec-engine -E 'test(sec002_plugin_infusion_source_config_not_pub) | test(sec003_sandbox_violation_url_not_in_warn_log)'"
Enter
Sleep 12s

# AC-011: path traversal rejected; within-dir accepted
Type "cargo nextest run -p prism-spec-engine -E 'test(sec003_path_traversal_rejected) | test(sec003_path_within_plugin_dir_accepted) | test(sec003_load_all_rejects_traversal)'"
Enter
Sleep 12s

# AC-012: load_all error path disclosure prevention
Type "cargo nextest run -p prism-spec-engine -E 'test(sec002_load_all_error_does_not_leak_absolute_path)'"
Enter
Sleep 12s

Sleep 2s
