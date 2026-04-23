// Unit tests for AlertSeverity.
//
// AC coverage: AC-8
// VP coverage: (none directly — types are preconditions for downstream BCs)

#[cfg(test)]
mod tests {
    use crate::alert::AlertSeverity;

    // ── AC-8: Critical maps to OCSF severity_id 5 ────────────────────────────

    #[test]
    fn test_BC_S_02_002_ac8_critical_ocsf_severity_id_is_5() {
        // AC-8: AlertSeverity::Critical → 5
        assert_eq!(AlertSeverity::Critical.as_ocsf_severity_id(), 5);
    }

    // ── All 5 variants have correct OCSF mappings ─────────────────────────────

    #[test]
    fn test_BC_S_02_002_high_ocsf_severity_id_is_4() {
        assert_eq!(AlertSeverity::High.as_ocsf_severity_id(), 4);
    }

    #[test]
    fn test_BC_S_02_002_medium_ocsf_severity_id_is_3() {
        assert_eq!(AlertSeverity::Medium.as_ocsf_severity_id(), 3);
    }

    #[test]
    fn test_BC_S_02_002_low_ocsf_severity_id_is_2() {
        assert_eq!(AlertSeverity::Low.as_ocsf_severity_id(), 2);
    }

    #[test]
    fn test_BC_S_02_002_informational_ocsf_severity_id_is_1() {
        assert_eq!(AlertSeverity::Informational.as_ocsf_severity_id(), 1);
    }

    // ── All values are distinct ───────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_002_all_severity_ids_are_distinct() {
        let ids: Vec<u32> = [
            AlertSeverity::Critical,
            AlertSeverity::High,
            AlertSeverity::Medium,
            AlertSeverity::Low,
            AlertSeverity::Informational,
        ]
        .iter()
        .map(|s| s.as_ocsf_severity_id())
        .collect();

        // Check uniqueness
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 5, "all OCSF severity IDs must be distinct");
    }

    // ── Ordering: higher severity = higher id ─────────────────────────────────

    #[test]
    fn test_BC_S_02_002_severity_id_ordering_critical_gt_informational() {
        assert!(
            AlertSeverity::Critical.as_ocsf_severity_id()
                > AlertSeverity::Informational.as_ocsf_severity_id()
        );
    }
}
