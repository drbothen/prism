---
document_type: uncertainty-map
story_id: S-3.02
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.02 Uncertainty Map — Query Tool and Materialization

## Summary verdict

**YELLOW** — Several DataFusion 53.x API surfaces are referenced without
exact-version pin verification, and the story itself flags two of them
("API has changed significantly since earlier versions"). DataFusion releases
monthly and 53.1 was pinned in ADR-015 around early Wave 4 architecture work.
The story body says `datafusion | 53` without a minor pin (53.1 in arch).
Implementation-blocking work is small but verification of cited APIs is required.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | version-pin | Story dependency table cites `datafusion 53` and `arrow 53` without minor pin (line 327–328); ADR-015 pins `datafusion = "53.1"` for Wave 4. 14-day spec drift means 53.x has likely cut new patch releases. | RESEARCH-NEEDED: confirm latest 53.x patch and that `MemTable`, `SessionContext::new_with_config_rt`, and `GreedyMemoryPool` API shapes still match. Pin to `=53.x.y` in `crates/prism-query/Cargo.toml`. |
| Important | api-assumption | Lines 385–387 explicitly call out `SessionContext::new_with_config_rt()` and warn "verify DataFusion 53 API docs before implementation (API has changed significantly since earlier versions)". This is a self-flagged uncertainty. | RESEARCH-NEEDED: validate the exact constructor name and `GreedyMemoryPool` integration pattern in latest 53.x. The story already requests this. |
| Important | feature-claim | Line 137–139 claims "DataFusion uses i128 intermediate accumulators … handles this natively" for integer overflow prevention (BC-2.11.006). Aggregation overflow behavior in DataFusion has historically varied by datatype/aggregator. | RESEARCH-NEEDED: confirm DataFusion 53.x SUM/COUNT i128 promotion semantics and whether the claim covers all aggregators used (SUM, AVG, COUNT). |
| Important | api-assumption | Line 391 notes "emit backtick-quoted table names when generating DataFusion SQL from its AST" — implies SQL generator backticks are syntactically accepted by DataFusion's parser. | RESEARCH-NEEDED: confirm DataFusion 53.x SQL parser accepts backtick-quoted identifiers (default ANSI mode uses double-quotes). May need parser dialect config. |
| Suggestion | architecture-pattern | `TableProvider` trait usage (line 180) is stable but its method signatures (`scan`, `supports_filter_pushdown`) have shifted across major versions. | Confirm trait signature against 53.x rustdoc before implementing. |
| Suggestion | unpinned-version | `arrow 53` line 327 is mentioned but `arrow` and `arrow-schema` are usually a matched pair with DataFusion. | Add explicit `arrow-array`, `arrow-schema` pins matching the DataFusion transitive minor. |
| Tech Debt | architecture-pattern | "scopeguard" mentioned line 86 for SessionContext lifecycle. Not pinned anywhere. | Add `scopeguard = "1"` to dep table when implementation begins. |

## Cross-references

- ADR-015 pins `datafusion = "53.1"` (caret-compatible, locks major 53). Story should match.
- Workspace `crates/prism-query/Cargo.toml` currently has NO `datafusion` dep — S-3.02 is the first to introduce it. Verify ADR-015 pin is replicated here.

## RESEARCH-NEEDED queries

1. "Latest stable datafusion 53.x patch as of 2026-05-04. Is `SessionContext::new_with_config_rt` still the constructor for custom RuntimeEnv? Has the GreedyMemoryPool API changed between 53.1 and current 53.x?"
2. "In datafusion 53.x, does the SQL parser accept backtick-quoted identifiers by default? What dialect config is required?"
3. "DataFusion 53.x integer overflow semantics for SUM/AVG/COUNT aggregators. Is i128 intermediate promotion automatic for all numeric aggregators?"
4. "DataFusion 53.x TableProvider trait — current signatures of scan(), supports_filters_pushdown(), schema()."

