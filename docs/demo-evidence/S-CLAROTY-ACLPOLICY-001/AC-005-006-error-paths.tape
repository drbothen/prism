# AC-005 + AC-006: E-QUERY-038 plan-time gates — Tier-2 column by raw name raises error;
# raw TOML Tier-1 name (policy_id) rejected in favour of Arrow name (metadata_uid).
#
# AC-005 (RG-005): SELECT policy_source FROM claroty.claroty_organization_acl_policies
#         raises E-QUERY-038 at plan time; available_columns contains raw_extensions,
#         metadata_uid, name, actor_user_name, comment, class_uid, _sensor but NOT
#         policy_source (Tier-2 column, only reachable via raw_extensions).
#         Same applies to all other Tier-2 names (policy_acl_type, policy_acl,
#         applied_models, matching_devices, policy_creation_date, policy_last_updated).
# AC-006 (RG-006 WIRE-SHAPE rename): SELECT policy_id FROM claroty.claroty_organization_acl_policies
#         raises E-QUERY-038; available_columns contains metadata_uid (Arrow form of
#         policy_id→metadata.uid) but NOT policy_id (raw TOML column name).
#         Asymmetric mapping: policy_id → metadata_uid (not just underscore substitution).
#
# Tests: RG-005 (test_BC_2_16_022_claroty_org_acl_policies_policy_source_tier2_e_query_038)
#        RG-006 (test_BC_2_16_022_claroty_org_acl_policies_policy_id_raw_name_not_projected_metadata_uid_is)
# Traces to: BC-2.16.022 §Invariants (Tier-2 not exposed as standalone Arrow column),
#            EC-016-022-003 (Tier-2 column-not-found), EC-016-022-004 (raw Tier-1 name rejected),
#            TV-BC-2.16.022-003

Output "docs/demo-evidence/S-CLAROTY-ACLPOLICY-001/AC-005-006-error-paths.gif"
Output "docs/demo-evidence/S-CLAROTY-ACLPOLICY-001/AC-005-006-error-paths.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 180s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-ACLPOLICY-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-005/006: E-QUERY-038 plan-time gates — Tier-2 + raw Tier-1 name rejection (BC-2.16.022)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-query -E 'test(BC_2_16_022)' 2>&1 | tail -10"
Enter
Sleep 60s
Sleep 2s
