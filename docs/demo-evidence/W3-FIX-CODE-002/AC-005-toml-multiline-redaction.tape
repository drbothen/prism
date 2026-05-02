# AC-005: Multi-line TOML credential redaction in sanitize_error_message
#
# BC-3.3.001 Invariant — no credential value in error messages (SEC-006)
# password/bearer_token/api_secret multi-line values redacted; display_name not redacted
# Story: W3-FIX-CODE-002, branch: feature/W3-FIX-CODE-002, SHA: 85a453d4

Output AC-005-toml-multiline-redaction.gif
Output AC-005-toml-multiline-redaction.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 120s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-002"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- Demo begins ---
Type "# AC-005: Multi-line TOML credential redaction (SEC-006)"
Enter
Sleep 500ms

Type "# password/bearer_token/api_secret multi-line values must not appear in errors"
Enter
Sleep 500ms

Type "# non-credential fields (display_name) must remain visible for diagnostics"
Enter
Sleep 1s

Type "cargo test -p prism-customer-config --test sec006_toml_multiline_redaction 2>&1 | tail -20"
Enter
Wait+Screen /test result:/
Sleep 2s
