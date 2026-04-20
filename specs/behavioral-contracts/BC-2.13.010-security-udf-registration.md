---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-13"
capability: "CAP-027"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "[pending-recompute]"
traces_to: ["CAP-027"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.010: Security UDF Registration — Register Domain-Specific Functions with DataFusion

## Description

Five security-domain UDFs are registered with every DataFusion SessionContext: `subnet_contains` (CIDR IP matching), `ioc_match` (named pattern set lookup), `time_window` (timestamp-within-duration test), `json_extract_string` (JSONPath extraction from raw_extensions blobs), and `severity_gte` (ordinal OCSF severity comparison). All UDFs are deterministic, stateless, and NULL-safe. Registration is idempotent. The `ioc_match` UDF reads pattern sets from configuration (refreshable via config reload); the backing store is specified in BC-2.13.014.

## Preconditions
- A DataFusion SessionContext is being initialized (either for a query execution or for rule compilation)

## Postconditions
- The following security UDFs are registered with the SessionContext:
  - **`subnet_contains(cidr: Utf8, ip: Utf8) -> Boolean`**: returns true if the IP address falls within the CIDR range; supports IPv4 and IPv6; invalid inputs return false (not error)
  - **`ioc_match(field: Utf8, pattern_set: Utf8) -> Boolean`**: matches field value against a named pattern set (loaded from configuration); pattern sets include IP lists, domain lists, hash lists; matching uses appropriate algorithm per type (CIDR for IPs, suffix for domains, exact for hashes)
  - **`time_window(field: Timestamp, duration: Utf8) -> Boolean`**: returns true if the timestamp field is within `duration` of now; duration uses the standard syntax: `30s`, `5m`, `1h`, `24h`, `7d`
  - **`json_extract_string(json: Utf8, path: Utf8) -> Utf8`**: extracts a string value from a JSON blob using JSONPath-like dotted notation; returns NULL if path not found or JSON invalid
  - **`severity_gte(severity: Utf8, threshold: Utf8) -> Boolean`**: compares OCSF severity enum values ordinally (info < low < medium < high < critical)
- All UDFs are deterministic and stateless (safe for DataFusion predicate pushdown optimization)
- UDFs are registered once per SessionContext creation; no per-query overhead

## Invariants
- UDF registration is idempotent: re-registering the same UDF is a no-op
- All UDFs handle NULL inputs gracefully (return NULL or false, never error)
- `ioc_match` pattern sets are loaded from configuration at startup and refreshable via config reload; stale pattern sets do not cause errors

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-UDF-001` | `ioc_match` references unknown pattern set name | Returns false; warning logged |
| `E-UDF-002` | `time_window` receives invalid duration string | Returns false; warning logged |
| `E-UDF-003` | `subnet_contains` receives malformed CIDR | Returns false; warning logged |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-033 | `subnet_contains("10.0.0.0/8", "10.0.0.1")` | Returns true |
| EC-13-034 | `subnet_contains("::ffff:10.0.0.0/104", "10.0.0.1")` | Returns true (IPv4-mapped IPv6) |
| EC-13-035 | `ioc_match(hostname, "known_bad_domains")` with empty pattern set | Returns false for all inputs |
| EC-13-036 | `time_window(event_time, "5m")` with event_time 4m59s ago | Returns true |
| EC-13-037 | `json_extract_string('{"a": {"b": 42}}', 'a.b')` | Returns "42" (stringified) |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `subnet_contains("192.168.0.0/16", "192.168.1.5")` | true | happy-path |
| `severity_gte("high", "medium")` | true | happy-path |
| `ioc_match("hostname", "nonexistent_list")` | false; E-UDF-001 warning | error |
| `subnet_contains("not_a_cidr", "10.0.0.1")` | false; E-UDF-003 warning | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-024 | Injection scanner: detects known injection patterns | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-027 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog
| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 1.0 | 2026-04-13 | cycle-1 | Initial contract |
| 1.1 | 2026-04-20 | pre-build-sweep | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
