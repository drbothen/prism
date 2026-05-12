# AC-5 Evidence — `#[non_exhaustive]` on 30 Types + CI EXPECTED=30 Enforcement

**Story:** S-PLUGIN-PREREQ-C v1.3
**Status: SATISFIED**
**Resolves:** TD-S-PLUGIN-PREREQ-B-016 (P2)
**BC anchor:** BC-2.01.013 postcondition — the spec-driven adapter surface is explicitly
extensible; `#[non_exhaustive]` is the compile-time enforcement that external crates cannot
struct-literal-construct TOML spec types.

---

## AC Summary (quoted from story v1.3)

> `#[non_exhaustive]` is applied to ALL pub TOML-deserialized types in `crates/prism-spec-engine/`
> and `crates/prism-core/`. The full target set is 30 types total.
>
> **Compile-fail test (Red Gate anchor):** A compile-fail test under `tests/external/non-exhaustive-violation/`
> attempts struct-literal construction and match-without-wildcard of all audited types from
> outside the crate. With `#[non_exhaustive]` in place, each attempt MUST produce a compile error.
> CI enforces `EXPECTED=30` violations.

---

## Red Gate Tests

### Just recipe (primary CI gate):

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && just check-non-exhaustive

Verifying #[non_exhaustive] forward-compat enforcement (expected: 30 violations)...
PASS: 30 types correctly reject external construction (expected: 30)
```

### Direct compile-fail count (secondary verification):

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && \
  cargo check --manifest-path tests/external/non-exhaustive-violation/Cargo.toml 2>&1 \
  | grep -E "^error\[E0639\]|^error\[E0004\]" | wc -l

30

# Breakdown:
# E0639 (cannot create non-exhaustive struct using struct expression): 19
# E0004 (non-exhaustive patterns in match without wildcard arm):       11
# Total:                                                                30
```

---

## 30 Audited Types

### `crates/prism-spec-engine/src/spec_parser.rs` (9 types)

| Type | Kind | `#[non_exhaustive]` Added |
|------|------|---------------------------|
| `CredentialRef` | struct | yes |
| `SensorSpec` | struct | yes |
| `SensorTableDescriptor` | struct | yes |
| `FetchStep` | struct | yes |
| `ColumnSpec` | struct | yes |
| `TableSpec` | struct | yes |
| `PaginationConfig` | enum | yes |
| `AuthType` | enum | yes |
| `RateLimitHints` | struct | yes |

### `crates/prism-spec-engine/src/write_endpoint.rs` (3 types)

| Type | Kind | `#[non_exhaustive]` Added |
|------|------|---------------------------|
| `BatchMode` | enum | yes |
| `WriteStep` | struct | yes |
| `WriteEndpointSpec` | struct | yes |

### `crates/prism-spec-engine/src/infusion/mod.rs` (8 types)

| Type | Kind | `#[non_exhaustive]` Added |
|------|------|---------------------------|
| `InfusionType` | enum | yes |
| `BuiltInSourceType` | enum | yes |
| `InfusionSourceConfig` | struct | yes |
| `CredentialRef` | struct | yes |
| `InfusionField` | struct | yes |
| `PipeStageConfig` | struct | yes |
| `PluginConfig` | struct | yes |
| `InfusionSpec` | struct | yes |

### `crates/prism-spec-engine/src/types.rs` (8 types)

| Type | Kind | `#[non_exhaustive]` Added |
|------|------|---------------------------|
| `SensorTableDescriptor` | struct | yes |
| `CredentialRef` | struct | yes |
| `SensorSpec` | struct | yes |
| `ColumnType` | enum | yes |
| `ColumnDef` | struct | yes |
| `PaginationType` | enum | yes |
| `SpecStatus` | enum | yes |
| `ClientStatus` | enum | yes (edge case — config-input / wire boundary) |

### `crates/prism-core/src/column.rs` (2 types)

| Type | Kind | `#[non_exhaustive]` Added |
|------|------|---------------------------|
| `ColumnType` | enum | yes |
| `ColumnOptions` | struct | yes |

**Total: 30 types.**

---

## DtuMode Footnote

`crates/prism-spec-engine/src/types.rs` also contains `pub enum DtuMode` annotated with
`#[non_exhaustive]`. This is a pre-existing annotation governed by BC-3.2.005 (different audit
lifecycle). It is intentionally NOT enumerated in AC-5's audit table and NOT counted toward the
30-type total. The annotation remains in place. Its exclusion from AC-5 audit scope is
documented per F-LP4-LOW-002 adjudication and the story v1.3 footnote.

---

## MCP-Wire Types Excluded from AC-5 (for awareness)

11 types are deliberately excluded: `SensorSpecEntry`, `ConfigSnapshot`, `ValidationError`,
`ModeChange`, `ReloadResult`, `ReloadStatus`, `ModifiedSpec`, `AddSensorSpecResult`,
`ListSensorSpecsResult`, `AddSensorSpecArgs`, `ListSensorSpecsArgs`. Their stability is
governed by the MCP protocol version; adding a variant requires an MCP version bump, not a
Rust `#[non_exhaustive]` annotation. This exclusion is documented per F-LP3-MED-001 and
F-LP3-MED-002 adjudication (story v1.2 + v1.3 changelog).

---

## Production Code Reference

Each `#[non_exhaustive]` annotation was applied alongside a doc-comment explaining to external
consumers why struct-literal construction is not supported. Example from `spec_parser.rs`:

> "Fields may be added in future releases without a semver bump; use the `Default` impl
> or builder pattern."

For enums, the doc-comment explains that new variants may be added without a semver bump and
that external `match` expressions must include a `_ => {}` wildcard arm.

**CI enforcement:** The `.github/workflows/ci.yml` `non-exhaustive-violation-compile-fail` job
runs `just check-non-exhaustive` with `EXPECTED=30`. Any annotation regression (type loses
`#[non_exhaustive]`) causes the violator crate to produce fewer than 30 errors, and the count
check fails CI. Any new annotated type not in the violator crate is invisible to the check —
the violator crate serves as an explicit census, not an exhaustive scan.

---

## Cross-References

- Fix-burst-2 (F-LP2-HIGH-001): initial sibling sweep applying `#[non_exhaustive]` to
  `write_endpoint.rs`, `infusion/mod.rs`, and `types.rs` (15 types, commit 8908bf27)
- Fix-burst-4 (F-LP4-MED-001 + MED-002): added `types::SensorSpec` violation to violator crate,
  bumped EXPECTED from 29 to 30 (commit 651bbb64)
- F-LP5-LOW-001 cleanup: synced `main.rs` doc-header in violator crate from 29 to 30 types
  (commit c9bb9d26, HEAD)
- BC-2.01.013 v1.6: spec-driven adapter surface forward-compatibility constraint
