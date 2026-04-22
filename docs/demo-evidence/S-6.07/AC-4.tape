# AC-4: FailureMode::RateLimit returns HTTP 429 with Retry-After header
#
# Given FailureMode::RateLimit { after_n_requests: 3, retry_after_secs: 60 } is configured,
# When the 4th request to any endpoint arrives,
# Then the response is HTTP 429 with Retry-After: 60 header.
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-4.tape

Output docs/demo-evidence/S-6.07/AC-4.gif
Output docs/demo-evidence/S-6.07/AC-4.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 24
Set WindowBar Colorful

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.07-crowdstrike"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-4: RateLimit failure mode — 4th request returns 429 + Retry-After: 60"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_4_rate_limit"
Enter
Sleep 15s
