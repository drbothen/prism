Output AC-002-error-codes.gif
Output AC-002-error-codes.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.01"
Enter
Sleep 0.5s
Show

# --- AC-002: E-CFG-XXX error code coverage (18 tests, all GREEN) ---
Type "cargo test -p prism-customer-config test_e_cfg -- --nocapture"
Enter
Sleep 30s
