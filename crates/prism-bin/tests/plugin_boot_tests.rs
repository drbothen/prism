//! S-PLUGIN-PREREQ-D Red Gate tests — prism-bin plugin boot wiring.
//!
//! All 7 tests stub with `todo!()` and MUST FAIL until the implementer wires
//! `PluginRuntime` into `crates/prism-bin/src/boot.rs` (BC-5.38.001 Red Gate).
//!
//! Traces to: S-PLUGIN-PREREQ-D (v1.32)
//! BCs: BC-2.22.001 (boot orchestration)
//! VPs: VP-PLUGIN-004, VP-PLUGIN-007
//!
//! # Test → AC / BC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_22_001_boot_step_plugin_load_placement | AC-1 | BC-2.22.001 §Sequencing Invariant (step 7.5) |
//! | test_BC_2_22_001_plugin_load_failure_exits_code_4 | AC-2 | BC-2.22.001 §Pre-Traffic Gate Invariant condition 6 |
//! | test_BC_2_22_001_plugin_load_disabled_env | AC-3 | BC-2.22.001; EC-D-004 |
//! | test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config | AC-18 | BC-2.22.001; EC-D-011 |
//! | test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit | AC-4 | VP-PLUGIN-004 (unsigned-plugin boot warning + audit) |
//! | test_VP_PLUGIN_007_plugin_load_rejected_no_allowlist | AC-5 | VP-PLUGIN-007; E-PLUGIN-013 |
//! | test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded | AC-5 | VP-PLUGIN-007; E-PLUGIN-014 |

#![allow(dead_code, unused_imports)]

/// AC-1 (S-PLUGIN-PREREQ-D) — Plugin-load step inserted between storage init (step 7)
/// and query-engine init (step 8) per BC-2.22.001 §Sequencing Invariant (step 7.5).
///
/// Verifies that boot sequencing calls `PluginRuntime::load_all_plugins` AFTER storage
/// is initialised and BEFORE the QueryEngine is started. A future injected-failure harness
/// will confirm that a plugin-load failure blocks step 8 (query-engine init) per the
/// §Pre-Traffic Gate Invariant.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires boot.rs step 7.5.
#[test]
fn test_BC_2_22_001_boot_step_plugin_load_placement() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-1)")
}

/// AC-2 (S-PLUGIN-PREREQ-D) — Plugin-load step failure causes boot to exit with code 4.
///
/// BC-2.22.001 §Pre-Traffic Gate Invariant condition 6: if the plugin-load step (7.5) fails,
/// the MCP server MUST NOT bind. The process exits with code 4 (ADR-022 §A boot-failure map
/// for plugin/runtime init failure).
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires boot.rs.
#[test]
fn test_BC_2_22_001_plugin_load_failure_exits_code_4() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-2)")
}

/// AC-3 (S-PLUGIN-PREREQ-D) — `PRISM_DISABLE_PLUGIN_LOAD=1` skips all plugin loading.
///
/// When the environment variable `PRISM_DISABLE_PLUGIN_LOAD` is set to the exact string `"1"`,
/// `boot::plugin_load_step` emits a single `tracing::warn!(event_type = "plugin_load_disabled_via_envvar", ...)`
/// and returns `Ok(0)` without scanning the plugin directory. The MCP server still binds.
/// (EC-D-004; BC-2.16.002 catalog row.)
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires boot.rs env-var check.
#[test]
fn test_BC_2_22_001_plugin_load_disabled_env() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-3)")
}

/// AC-18 (S-PLUGIN-PREREQ-D) — `PRISM_DISABLE_PLUGIN_LOAD=1` takes precedence over plugin
/// directory configuration; non-`"1"` values (e.g. `"true"`, `"yes"`, `"0"`) are treated as
/// unset and do NOT disable loading.
///
/// Only the exact string `"1"` activates the escape valve (EC-D-011). All other values
/// fall through to the normal plugin-load path.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires env-var exact-match check.
#[test]
fn test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-18)")
}

/// AC-4 (S-PLUGIN-PREREQ-D) — Every successfully loaded plugin generates a
/// `tracing::warn!(event_type = "plugin_load_unsigned", plugin_path = ..., plugin_hash = ...)` audit
/// entry and a boot-level WARN log per VP-PLUGIN-004.
///
/// Plugin signing is deferred to TD-PLUGIN-SIGNING-001; v1.0 treats all plugins as unsigned.
/// The audit entry is mandatory so operators are aware that loaded plugins are not
/// cryptographically verified.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires audit emission in
/// `PluginRuntime::load_all_plugins`.
#[test]
fn test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-4)")
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin whose manifest omits the `allowed_urls` field is
/// rejected with `E-PLUGIN-013`; remaining N-1 plugins continue loading (n-1 survivor rule).
///
/// VP-PLUGIN-007: the `allowed_urls` field is REQUIRED in the manifest (not optional).
/// An absent or `null` value is rejected. An explicitly empty list `[]` is accepted (default-deny).
/// BC-2.16.002 catalog row `plugin_load_failed_manifest_no_allowed_urls` must be emitted.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires manifest validation.
#[test]
fn test_VP_PLUGIN_007_plugin_load_rejected_no_allowlist() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-5, E-PLUGIN-013)")
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin whose manifest `format_version` exceeds
/// `CURRENT_SUPPORTED_VERSION` is rejected with `E-PLUGIN-014`.
///
/// VP-PLUGIN-007: `format_version > CURRENT_SUPPORTED_VERSION` (currently 1) must cause
/// `load_plugin` to return an error; the plugin is skipped and the n-1 survivor rule applies.
/// BC-2.16.002 catalog row `plugin_load_failed_format_version_exceeded` (ERROR, fields:
/// `plugin_path`, `format_version`, `max_supported`) must be emitted.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires format_version check.
#[test]
fn test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-5, E-PLUGIN-014)")
}
