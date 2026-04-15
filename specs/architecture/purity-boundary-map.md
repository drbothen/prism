---
document_type: architecture-section
level: L3
section: "purity-boundary-map"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, domain-spec/invariants.md]
traces_to: ARCH-INDEX.md
---

# Purity Boundary Map

## Decision: Pure Core / Effectful Shell Separation (AD-008)

**Status:** accepted
**Context:** Formal verification (Kani, proptest) works best on pure functions without I/O dependencies. The more logic we can push into the pure core, the higher our verification coverage.
**Decision:** Strict separation of pure domain logic from effectful I/O. All crates are classified as pure-core, effectful-shell, or mixed (with documented I/O boundary).
**Rationale:** Pure functions are deterministic — same input always produces same output. This makes them ideal targets for property-based testing and formal proofs. The effectful shell is thin and tested via integration tests.
**Consequences:** Domain logic must not import I/O crates (reqwest, rocksdb, keyring, tokio::fs). Effectful code calls pure functions, not the other way around.

## Classification

| Crate | Classification | I/O Boundary | Verification Strategy |
|-------|---------------|-------------|----------------------|
| prism-core | **pure-core** | None — no I/O imports | Kani proofs for invariants, proptest for all types |
| prism-ocsf | **pure-core** | None — normalization is pure transformation | Kani proofs for field mapping, proptest for DynamicMessage |
| prism-security | **mixed** | Token store uses SystemTime for expiry; injection scanner is pure | Pure: flag evaluation, path resolution, pattern detection. Effect: clock access for token expiry |
| prism-query | **mixed** | Parser is pure; DataFusion execution has internal state | Pure: AxiQL parser, alias resolution, push-down classification, UDF implementations. Effect: DataFusion SessionContext lifecycle |
| prism-spec-engine | **mixed** | Spec parsing is pure; pipeline execution makes HTTP calls | Pure: TOML parsing, validation, variable resolution. Effect: HTTP calls via reqwest, arc-swap config swap |
| prism-operations | **mixed** | State machines and diff logic are pure; RocksDB access is effectful | Pure: case state transitions, diff computation, detection rule evaluation, template interpolation. Effect: RocksDB reads/writes, timer ticks |
| prism-sensors | **effectful-shell** | HTTP client, auth token management | Pure: none (thin adapter orchestration). Effect: HTTP calls, auth flows |
| prism-credentials | **effectful-shell** | OS keyring, filesystem encryption/decryption | Pure: credential name validation (moved to prism-core). Effect: keyring access, file I/O, AES encryption |
| prism-storage | **effectful-shell** | RocksDB (in production), BTreeMap (in tests) | Pure: key construction, domain enum mapping. Effect: RocksDB operations |
| prism-audit | **effectful-shell** | tracing subscriber, RocksDB buffer writes | Pure: audit entry construction. Effect: tracing emission, RocksDB writes |
| prism-mcp | **effectful-shell** | MCP transport (stdio), tool dispatch | Pure: response construction, error formatting. Effect: MCP transport I/O |
| prism-bin | **effectful-shell** | CLI parsing, signal handling, process lifecycle | Pure: none. Effect: everything |

## Pure Core Catalog

Functions in the pure core are the primary targets for formal verification:

| Function / Module | Crate | Inputs | Outputs | Properties |
|-------------------|-------|--------|---------|-----------|
| `TenantId::new(s)` | prism-core | raw string | Result<TenantId, Error> | Validates `[a-zA-Z0-9_-]+`, non-empty |
| `ClientCapabilities::is_allowed(path)` | prism-core | capability path | bool + CapabilityExplanation | DI-003: deny-by-default, most-specific wins |
| `CaseStatus::can_transition_to(target)` | prism-core | (current, target) status pair | bool | DI-025: exactly 12 valid transitions |
| `AxiqlParser::parse(input)` | prism-query | &str | Result<Ast, Vec<Error>> | DI-019: security limits enforced |
| `AliasResolver::expand(query, aliases)` | prism-query | query + alias map | expanded query | DI-020: depth 3, no cycles |
| `PushDownClassifier::classify(predicate, schema)` | prism-query | predicate + column options | PushDown or PostFilter | DI-021: REQUIRED columns enforced |
| `OcsfNormalizer::normalize(record, mapping)` | prism-ocsf | raw record + field map | OcsfEvent | DI-005: valid OCSF message |
| `FieldResolver::resolve(field_path)` | prism-ocsf | dot-notation path | resolved value | Four-tier resolution |
| `DetectionRule::validate(rule)` | prism-operations | rule definition | Result<(), Vec<Error>> | DI-024: all validation rules |
| `DiffEngine::compute_diff(prev, curr)` | prism-operations | two Arrow RecordBatches | DiffResult (added, removed) | Deterministic, order-independent |
| `TemplateInterpolator::render(template, vars)` | prism-operations | template + variable map | rendered string | 4-level resolution, no panics |
| `InjectionScanner::scan(text)` | prism-security | string | Vec<SafetyFlag> | Regex-based, never modifies input |
| `FeatureFlagEvaluator::evaluate(path, caps)` | prism-security | path + BTreeMap | Effect (Allow/Deny) + trace | DI-003: hierarchical resolution |
| `SpecValidator::validate(spec)` | prism-spec-engine | SensorSpec | Result<(), Vec<Error>> | DI-030: all validation rules |

## Purity Strategy

The boundary between pure and effectful code follows this pattern:

1. **Parse at the boundary.** Raw inputs (MCP tool parameters, TOML config, sensor API responses) are parsed into validated domain types at the I/O boundary.
2. **Transform in the core.** All logic operates on validated domain types. No raw strings, no unvalidated IDs, no optional fields that should be required.
3. **Effect at the edge.** I/O operations (HTTP, RocksDB, keyring, MCP transport) are thin wrappers that call pure functions for all decision-making.

This pattern maximizes the surface area available for formal verification while keeping the effectful shell minimal and testable via integration tests.
