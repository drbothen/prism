//! AC-4: StorageDomain::all() returns all variants (16 S-1.01 + 3 S-1.02 = 19)
//! with distinct column_family_name() strings.
//!
//! Updated in S-1.02: 3 new domains (Credentials, FeatureFlags, Scheduler)
//! added for VP-055 domain isolation testing.

use std::collections::HashSet;

use prism_core::StorageDomain;

/// AC-4: all() returns exactly 19 elements (16 S-1.01 + 3 S-1.02).
#[test]
fn test_ac4_storage_domain_all_returns_19_variants() {
    let all = StorageDomain::all();
    assert_eq!(
        all.len(),
        19,
        "StorageDomain::all() must return exactly 19 variants, got {}",
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
        all.len(),
        "all column_family_name() values must be distinct"
    );
}

/// AC-4: spot-check individual variant names.
#[test]
fn test_ac4_storage_domain_spot_check_names() {
    assert_eq!(StorageDomain::Default.column_family_name(), "default");
    assert_eq!(StorageDomain::Schedules.column_family_name(), "schedules");
    assert_eq!(
        StorageDomain::DiffResults.column_family_name(),
        "diff_results"
    );
    assert_eq!(
        StorageDomain::DetectionRules.column_family_name(),
        "detection_rules"
    );
    assert_eq!(
        StorageDomain::DetectionState.column_family_name(),
        "detection_state"
    );
    assert_eq!(StorageDomain::Alerts.column_family_name(), "alerts");
    assert_eq!(StorageDomain::Cases.column_family_name(), "cases");
    assert_eq!(
        StorageDomain::AuditBuffer.column_family_name(),
        "audit_buffer"
    );
    assert_eq!(StorageDomain::DirtyBits.column_family_name(), "dirty_bits");
    assert_eq!(StorageDomain::Watchdog.column_family_name(), "watchdog");
    assert_eq!(StorageDomain::Aliases.column_family_name(), "aliases");
    assert_eq!(StorageDomain::Decorators.column_family_name(), "decorators");
    assert_eq!(
        StorageDomain::InfusionCache.column_family_name(),
        "infusion_cache"
    );
    assert_eq!(
        StorageDomain::ActionState.column_family_name(),
        "action_state"
    );
    assert_eq!(
        StorageDomain::PluginState.column_family_name(),
        "plugin_state"
    );
    assert_eq!(
        StorageDomain::EventBuffer.column_family_name(),
        "event_buffer"
    );
    // S-1.02 additions
    assert_eq!(
        StorageDomain::Credentials.column_family_name(),
        "credentials"
    );
    assert_eq!(
        StorageDomain::FeatureFlags.column_family_name(),
        "feature_flags"
    );
    assert_eq!(StorageDomain::Scheduler.column_family_name(), "scheduler");
}

/// AC-4: all() contains all expected variants (no variant omitted).
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
        StorageDomain::Credentials,
        StorageDomain::FeatureFlags,
        StorageDomain::Scheduler,
    ];
    for variant in &expected {
        assert!(
            all.contains(variant),
            "StorageDomain::all() is missing variant: {variant:?}"
        );
    }
}
