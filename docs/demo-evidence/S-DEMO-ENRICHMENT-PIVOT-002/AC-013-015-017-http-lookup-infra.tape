# S-DEMO-ENRICHMENT-PIVOT-002 — HTTP Lookup Infrastructure ACs
# AC-013: InfusionType::HttpLookup parses NVD spec; rejects missing ${input}; rejects invalid method
# AC-015: E-INFUSE-009/010/011 error variants exclude credentials/env-var-name/resolved-IP (CWE-209)
# AC-017: SSRF guard rejects private IP without DTU_MODE; accepts with DTU_MODE (CWE-918)
#
# Evidence form: test-execution capture (all unit tests, no external deps)

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-013-015-017-http-lookup-infra.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-013-015-017-http-lookup-infra.webm

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

# AC-013: HttpLookup type parses NVD spec; rejects bad url_template and method
Type "cargo nextest run -p prism-spec-engine -E 'test(http_lookup_infusion_type_parses_nvd_spec) | test(http_lookup_parse_rejects_missing_input_placeholder) | test(http_lookup_parse_rejects_invalid_method)'"
Enter
Sleep 12s

# AC-015: Error format excludes credentials / env-var-names / resolved IPs (AD-017 / CWE-209)
Type "cargo nextest run -p prism-spec-engine -E 'test(http_lookup_failed_error_format_excludes_credentials) | test(credential_resolution_failed_excludes_env_var_name) | test(ssrf_rejected_error_excludes_resolved_ip)'"
Enter
Sleep 12s

# AC-017: SSRF guard — private IP rejected without DTU_MODE; accepted with DTU_MODE
Type "cargo nextest run -p prism-spec-engine -E 'test(ssrf_rejects_private_base_url_without_dtu_mode) | test(ssrf_accepts_private_base_url_with_dtu_mode)'"
Enter
Sleep 12s

Sleep 2s
