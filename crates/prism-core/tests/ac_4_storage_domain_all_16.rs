//! AC-4: StorageDomain::all() returns exactly 16 variants with distinct column_family_name() strings.

use std::collections::HashSet;

use prism_core::StorageDomain;

/// AC-4: all() returns exactly 16 elements.
#[test]
fn test_ac4_storage_domain_all_returns_16_variants() {
    let all = StorageDomain::all();
    assert_eq!(
        all.len(),
        16,
        "StorageDomain::all() must return exactly 16 variants, got {}",
        all.len()
    );
}

/// AC-4: all column_family_name() strings are distinct (no duplicates).
#[test]
fn test_ac4_storage_domain_column_family_names_are_distinct() {
    let all = StorageDomain::all();
    let names: HashSet<&'static str> = all.iter().map(|d| d.column_family_name()).collect();
    assert_eq!(
        names.len(),
        16,
        "all 16 column_family_name() values must be distinct"
    );
}

/// AC-4: spot-check individual variant names.
#[test]
fn test_ac4_storage_domain_spot_check_names() {
    assert_eq!(StorageDomain::Default.column_family_name(), "default");
    assert_eq!(StorageDomain::Schedules.column_family_name(), "schedules");
    assert_eq!(StorageDomain::DiffResults.column_family_name(), "diff_results");
    assert_eq!(StorageDomain::DetectionRules.column_family_name(), "detection_rules");
    assert_eq!(StorageDomain::DetectionState.column_family_name(), "detection_state");
    assert_eq!(StorageDomain::Alerts.column_family_name(), "alerts");
    assert_eq!(StorageDomain::Cases.column_family_name(), "cases");
    assert_eq!(StorageDomain::AuditBuffer.column_family_name(), "audit_buffer");
    assert_eq!(StorageDomain::DirtyBits.column_family_name(), "dirty_bits");
    assert_eq!(StorageDomain::Watchdog.column_family_name(), "watchdog");
    assert_eq!(StorageDomain::Aliases.column_family_name(), "aliases");
    assert_eq!(StorageDomain::Decorators.column_family_name(), "decorators");
    assert_eq!(StorageDomain::InfusionCache.column_family_name(), "infusion_cache");
    assert_eq!(StorageDomain::ActionState.column_family_name(), "action_state");
    assert_eq!(StorageDomain::PluginState.column_family_name(), "plugin_state");
    assert_eq!(StorageDomain::EventBuffer.column_family_name(), "event_buffer");
}

/// AC-4: all() contains all 16 expected variants (no variant omitted).
#[test]
fn test_ac4_storage_domain_all_contains_expected_variants() {
    let all: HashSet<StorageDomain> = StorageDomain::all().iter().cloned().collect();
    let expected = [
        StorageDomain::Default,
        StorageDomain::Schedules,
        StorageDomain::DiffResults,
        StorageDomain::DetectionRules,
        StorageDomain::DetectionState,
        StorageDomain::Alerts,
        StorageDomain::Cases,
        StorageDomain::AuditBuffer,
        StorageDomain::DirtyBits,
        StorageDomain::Watchdog,
        StorageDomain::Aliases,
        StorageDomain::Decorators,
        StorageDomain::InfusionCache,
        StorageDomain::ActionState,
        StorageDomain::PluginState,
        StorageDomain::EventBuffer,
    ];
    for variant in &expected {
        assert!(
            all.contains(variant),
            "StorageDomain::all() is missing variant: {variant:?}"
        );
    }
}
