Output docs/demo-evidence/S-6.08/AC-6-rate-limit.gif
Output docs/demo-evidence/S-6.08/AC-6-rate-limit.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# AC-6: RateLimit{after_n_requests:5} — 6th request returns HTTP 429 + Retry-After:30

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test ac_6_rate_limit"
Enter
Sleep 10s
