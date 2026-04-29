Output AC-003-credential-heuristics.gif
Output AC-003-credential-heuristics.webm

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

# --- AC-003: credential heuristics (BC-3.3.002) — literals rejected, scheme prefixes accepted ---
Type "cargo test -p prism-customer-config test_bc_3_3_002 -- --nocapture"
Enter
Sleep 30s
