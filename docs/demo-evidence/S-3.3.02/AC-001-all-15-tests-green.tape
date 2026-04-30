Output AC-001-all-15-tests-green.gif
Output AC-001-all-15-tests-green.webm

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

# --- AC-001: all 15 startup_boot_test tests GREEN ---
Type "cargo test -p prism-customer-config --test startup_boot_test 2>&1 | tail -25"
Enter
Sleep 20s
