Output AC-001-all-7-org-tagging-tests-green.gif
Output AC-001-all-7-org-tagging-tests-green.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.05"
Enter
Sleep 0.5s
Show

# --- AC-001: all 7 org_tagging tests GREEN (BC-3.2.004 + BC-3.2.005) ---
Type "cargo test -p prism-dtu-slack --features dtu --test org_tagging"
Enter
Sleep 30s
