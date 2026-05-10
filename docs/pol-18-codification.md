# POL-18 codification

POL-18 (test_injection_feature_pairing) was codified in `.factory/policies.yaml` v1.7.
The .factory/ directory is gitignored (tracked on factory-artifacts branch).

The policy is now enforced in prism-bin via the `required-features` declarations
added to Cargo.toml by F-PASS2-MED-1 fix in this branch.

See: crates/prism-bin/Cargo.toml [[test]] sections.
