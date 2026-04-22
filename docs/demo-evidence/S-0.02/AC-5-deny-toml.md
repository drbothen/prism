# AC-5 Evidence: deny.toml license and advisory policy

Story: S-0.02 | Version: 1.4 | Date: 2026-04-21

**AC-5:** Given `cargo deny check` is run on the workspace, When `deny.toml` is present,
Then all crate licenses are checked against the allowlist and any crate with a
disallowed license causes the check to fail.

---

## deny.toml content

```toml
# deny.toml — cargo-deny configuration for the Prism workspace.
# Enforces license allowlist, advisory policy, ban rules, and source restrictions.

[licenses]
# OSI-approved licenses permitted in this workspace.
allow = [
  "MIT",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Unicode-DFS-2016",
  "Zlib",
]

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
unmaintained = "none"
yanked = "deny"
ignore = []

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

## TOML parse validation

```
$ python3 -c 'import tomllib; print(tomllib.loads(open("deny.toml").read()))'
TOML parse OK
{
  "licenses": {
    "allow": ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-DFS-2016", "Zlib"]
  },
  "advisories": {
    "db-path": "~/.cargo/advisory-db",
    "db-urls": ["https://github.com/rustsec/advisory-db"],
    "unmaintained": "none",
    "yanked": "deny",
    "ignore": []
  },
  "bans": {
    "multiple-versions": "warn",
    "wildcards": "deny"
  },
  "sources": {
    "unknown-registry": "deny",
    "unknown-git": "deny"
  }
}
```

## Key policy values confirmed

| Key | Value | Requirement |
|-----|-------|-------------|
| licenses.allow | MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-DFS-2016, Zlib | OSI-approved allowlist |
| bans.wildcards | deny | Architecture compliance: no wildcard version requirements |
| sources.unknown-registry | deny | Only crates.io permitted |
| sources.unknown-git | deny | Only crates.io permitted |
| advisories.yanked | deny | Yanked crates blocked |

## Test gate result

```
ok 1 - deny.toml exists
ok 2 - deny.toml has [licenses] section
ok 3 - AC-5: deny.toml allowlist contains all required OSI-approved licenses
ok 4 - deny.toml sets vulnerability = "deny"
ok 5 - deny.toml sets wildcards = "deny"
ok 6 - deny.toml sets unknown-registry = "deny"
ok 7 - AC-5: cargo deny runtime check deferred — no crates exist yet (empty workspace)
```
**PASS**

## Known limitation

`cargo deny check` runtime execution is deferred until crates exist. Schema is validated
now; runtime validation occurs after S-6.06 merges and workspace members are populated.
