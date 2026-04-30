Output AC-002-validate-before-register.gif
Output AC-002-validate-before-register.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 24
Set Shell "bash"

# --- setup (hidden): set PATH, move to worktree ---
Hide
Type "export PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
Enter
Sleep 0.5s
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.02"
Enter
Sleep 0.5s
Show

# --- AC-002: validate-before-register (no partial state on validation failure) ---
Type "cargo test -p prism-customer-config --test startup_boot_test test_BC_3_3_004_validate_before_register_no_partial_state -- --nocapture"
Enter
Sleep 20s
