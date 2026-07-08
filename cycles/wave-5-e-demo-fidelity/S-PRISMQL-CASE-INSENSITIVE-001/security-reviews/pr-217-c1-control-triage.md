---
document_type: security-triage
story: S-PRISMQL-CASE-INSENSITIVE-001
pr: 217
finding_id: ADV-PR-P5-OBS-001
original_severity: OBS
triaged_severity: MEDIUM
cwe: CWE-117
date: 2026-07-08
authored_by: security-reviewer via orchestrator-relay
status: CLOSED
closure_commit: 36a094d6
---
# Security Triage — PR #217 C1 Control Character Log-Injection Gap

## Finding

**ADV-PR-P5-OBS-001** (pass-5 on frozen fab7df00): `prism_core::sanitize_for_log` strips ASCII control characters (0x00–0x1F + 0x7F) but does not strip:

- Unicode C1 control range (U+0080–U+009F), including CSI (U+009B) which enables ANSI escape sequence injection
- Unicode line separator U+2028 and paragraph separator U+2029, which function as newline equivalents in JavaScript engines, some log ingestion pipelines, and NEL-aware log parsers

## Severity Rationale: MEDIUM (upgraded from OBS)

| Factor | Assessment |
|--------|------------|
| CWE | CWE-117 Improper Output Neutralization for Logs |
| Affected surface | `ocsf.enum_label_unrecognized` warn event `value` + `sensor_type` fields at PRIMARY (`spec_driven_adapter.rs`) and SECONDARY (`normalizer.rs`); `infusion.coercion_failed` `truncated_value` field |
| Attack vector | Malicious sensor data containing Unicode C1 bytes or U+2028/29 in OCSF enum label fields; relevant when prism logs are consumed by journald, structured log parsers, or LLM-agent context feeds |
| Log-spoofing risk | U+2028/29 in a `value` or `sensor_type` field creates a false newline in log consumers that split on NEL/LS/PS, enabling log injection. CSI (U+009B) in C1 range enables ANSI terminal escape sequence injection in consumers rendering raw Unicode log output |
| LLM-agent vector | Prism is deployed as a per-analyst MCP server with LLM agents consuming structured log output. A U+2028/29 in an `ocsf.enum_label_unrecognized` `value` field could introduce a false context boundary when log content is fed as structured text to an LLM agent (prompt injection class). CWE-117 severity is elevated in LLM-agent consumption contexts per prism AD-017 / project deployment model |
| Connectivity.rs precedent | `connectivity.rs::sanitize_error` already strips C1 + U+2028/29, establishing the widened scope as the project standard for sensitive log emission. The gap is an asymmetry in `sanitize_for_log` vs the established codebase standard |
| Exploitability | Requires attacker-controlled OCSF enum label data reaching `build_column_array` or `normalize_with_mappers`. In the prism deployment model (sensor APIs served via DTU clones), this means the DTU clone's fixture data or a compromised sensor endpoint would need to contain C1/U+2028/29 bytes in an enum label column |

**Upgrade rationale:** CWE-117 combined with LLM-agent consumption vector and the connectivity.rs parity gap justifies MEDIUM. The finding is not CRIT/HIGH because exploitability requires sensor-side data manipulation (not direct user input to prism), and the current test suite (RG-079, RG-080) confirms correct ASCII-range sanitization is in place.

## Blast-Radius Inventory (TD-VSDD-060)

| Site | File | Field | Affected |
|------|------|-------|---------|
| PRIMARY warn | `crates/prism-bin/src/spec_driven_adapter.rs` | `ocsf.enum_label_unrecognized` `value`, `sensor_type` | yes |
| SECONDARY warn | `crates/prism-ocsf/src/normalizer.rs` | `ocsf.enum_label_unrecognized` `value`, `sensor_type` | yes |
| infusion coercion | `crates/prism-spec-engine/src/...` | `infusion.coercion_failed` `truncated_value` | yes |
| connectivity error | `crates/prism-core/src/connectivity.rs` | `sanitize_error` (already strips C1 + U+2028/29) | parity check — CLEAN |

## Fix Recommendation

Widen `prism_core::sanitize_for_log` to:

```rust
// was: c.is_ascii_control()
// now: strip ASCII controls + Unicode Cc (includes C1) + line/paragraph separators
!ch.is_control() && ch != '\u{2028}' && ch != '\u{2029}'
```

`char::is_control()` in Rust covers both the ASCII control range (0x00–0x1F + 0x7F) and the Unicode Cc general category (which includes C1 U+0080–U+009F). The two explicit additional exclusions cover U+2028 and U+2029, which are Unicode Zl/Zp category (not Cc) but have newline semantics in several consumers.

## BC and Taxonomy Amendment Guidance

**BC-2.16.002:** Row 91 `ocsf.enum_label_unrecognized` field descriptions for `value` and `sensor_type` must be updated to state: "`prism_core::sanitize_for_log` (strips ASCII control chars 0x00–0x1F + 0x7F, Unicode Cc control range U+0080–U+009F, and line/paragraph separators U+2028/U+2029) is applied BEFORE the 50-codepoint truncation cap." The `infusion.coercion_failed` `truncated_value` field description should be extended identically. POL-30 Fork B (description extension; catalog count and row count unchanged at 91).

**error-taxonomy.md:** E-INFUSE-013 (TruncatedEnumLabel) and E-INFUSE-014 (TypeCoercionFailed) rendering notes should cite `prism_core::sanitize_for_log` with the widened scope, so implementers deriving new coercion-error emission sites have the correct sanitization contract.

## Closure

@36a094d6 (implementer): `sanitize_for_log` widened as specified. RG-082 `test_rg082_sanitize_for_log_strips_unicode_cc_and_line_separators` RED→GREEN (vectors: U+0085 NEXT LINE, U+0091 PRIVATE USE ONE, U+2028 LINE SEPARATOR, U+2029 PARAGRAPH SEPARATOR — all stripped; ASCII letters/digits preserved). BC-2.16.002 v2.05→v2.06 (product-owner). error-taxonomy v2.19→v2.20 (product-owner). just check 5319/5319 GREEN; non-exhaustive 89/89. **CLOSED.**
