# AC-001: SEC-P3-001 — inline-table TOML credential value redacted (CWE-209)
#
# Traces to: BC-3.3.004 postcondition 2
# Story: W3-FIX-SEC-004
#
# Demonstrates that a snippet line containing
#   credentials = { bearer_token = "my-secret" }
# is fully redacted by sanitize_error_message.
# Both success path (credential line → REDACTED) and error path
# (non-credential inline field → not redacted) are shown.

Output docs/demo-evidence/W3-FIX-SEC-004/AC-001-inline-table-credential-redacted.gif
Output docs/demo-evidence/W3-FIX-SEC-004/AC-001-inline-table-credential-redacted.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-004"
Enter
Sleep 1s
Show

# Success path: inline-table bearer_token → REDACTED
Type "# AC-001 SEC-P3-001: inline-table credentials redacted (CWE-209 fix)"
Enter
Sleep 500ms
Type "cargo test -p prism-customer-config --test sec_p3_001_inline_table_redaction -- test_AC_001_inline_table_credentials_redacted 2>&1 | tail -4"
Enter
Sleep 15s
