# AC-5: OAuth2 token endpoint returns access_token: "dtu-fake-cs-token"
#
# Given POST /oauth2/token is called with any client_credentials body,
# Then the response is HTTP 200 with access_token: "dtu-fake-cs-token".
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-5.tape

Output docs/demo-evidence/S-6.07/AC-5.gif
Output docs/demo-evidence/S-6.07/AC-5.webm

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

Type "# AC-5: OAuth2 token endpoint returns dtu-fake-cs-token with HTTP 200"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_5_oauth"
Enter
Sleep 15s
