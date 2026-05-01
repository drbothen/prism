Output docs/demo-evidence/W3-FIX-SEC-001/AC-003-auth-model-per-clone.gif
Output docs/demo-evidence/W3-FIX-SEC-001/AC-003-auth-model-per-clone.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-001 && clear"
Enter
Sleep 3s
Show

# AC-003: Auth model split
# Claroty/CrowdStrike: model A (missing header=401)
# Cyberint: model B (missing header=200, falls back to instance session)
# Armis: validate-on-presence (missing header=200, backcompat)
Type "# AC-003: auth model split — model A strict (claroty/cs) | model B (cyberint) | validate-on-presence (armis)"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-cyberint -p prism-dtu-armis --features dtu --test x_org_id_auth -- test_AC_003 --test-threads=1"
Enter
Sleep 15s
