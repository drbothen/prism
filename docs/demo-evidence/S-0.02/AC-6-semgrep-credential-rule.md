# AC-6 Evidence: semgrep credential rule fires

Story: S-0.02 | Version: 1.4 | Date: 2026-04-21

**AC-6:** Given a developer writes `let api_key: String = get_secret()`, When the
Semgrep SAST rule runs via `semgrep --config .semgrep/`, Then the rule
`prism-no-string-credentials` flags this as an ERROR.

---

## .semgrep/credential-handling.yml content

```yaml
rules:
  - id: prism-no-string-credentials
    message: >
      Credentials must not be stored in String types. Use SecretString or
      reference-based patterns per project AI-Opaque Credentials policy.
    languages: [rust]
    severity: ERROR
    patterns:
      - pattern: "let $X: String = ..."
      - metavariable-regex:
          metavariable: "$X"
          regex: "(?i)(password|secret|token|api_?key|credential|private_?key)"

  - id: prism-no-log-secret
    message: >
      Do not log potential credential values. Credentials must remain
      AI-opaque and must not appear in log output.
    languages: [rust]
    severity: WARNING
    pattern: |
      println!($FMT, ...)
```

## semgrep version

```
1.156.0
```

## Firing trigger test

Trigger source (`let api_key: String = get_secret();`):

```
Findings: 2 (2 blocking)

❯❯❱ semgrep.prism-no-string-credentials  [ERROR / Blocking]
     Credentials must not be stored in String types. Use SecretString or
     reference-based patterns per project AI-Opaque Credentials policy.
       2┆ let api_key: String = get_secret();

❯❱ semgrep.prism-no-log-secret  [Blocking]
     Do not log potential credential values.
       3┆ println!("{}", api_key);
```

See `AC-6-semgrep-fires.txt` for full verbatim output.

## Test gate result

```
ok 4 - prism-no-string-credentials rule declared
ok 5 - prism-no-string-credentials has severity: ERROR
ok 6 - AC-6: credential-handling.yml has real patterns (no TODO placeholders)
ok 7 - prism-no-log-secret rule declared
ok 8 - prism-unsafe-block rule declared
ok 9 - AC-6: prism-no-string-credentials fires on String credential code
```
**PASS**
