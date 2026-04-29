Output AC-002-concurrent-orgid-tagging.gif
Output AC-002-concurrent-orgid-tagging.webm

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

# --- AC-002: concurrent OrgId tagging (BC-3.2.004 postcondition 4) ---
Type "cargo test -p prism-dtu-slack --features dtu --test org_tagging test_BC_3_2_004_concurrent_sends_distinguished -- --nocapture"
Enter
Sleep 30s
