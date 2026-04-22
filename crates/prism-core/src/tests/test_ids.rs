// Unit tests for UUID v7 ID newtypes.
//
// AC coverage: (none direct — types are preconditions for downstream BCs)
// VP coverage: (none direct — verifies structural correctness)

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::ids::{AlertId, CaseId, RuleId, ScheduleId};

    // ── new() returns a ScheduleId ────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_005_schedule_id_new_returns_value() {
        let id = ScheduleId::new();
        // The inner UUID must be non-nil.
        assert_ne!(id.as_uuid(), Uuid::nil());
    }

    // ── Two successive ScheduleId::new() calls produce distinct values ────────

    #[test]
    fn test_BC_S_02_005_schedule_id_successive_calls_distinct() {
        let a = ScheduleId::new();
        let b = ScheduleId::new();
        assert_ne!(a, b, "two successive ScheduleId::new() must be distinct");
    }

    // ── from_uuid round-trips ─────────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_005_case_id_from_uuid_round_trip() {
        // Use a known UUID for deterministic round-trip.
        let raw = Uuid::parse_str("01875e4e-9f00-7abc-8def-123456789abc")
            .expect("valid UUID literal");
        let case_id = CaseId::from_uuid(raw);
        assert_eq!(case_id.as_uuid(), raw);
    }

    // ── from_uuid is the identity for all ID types ────────────────────────────

    #[test]
    fn test_BC_S_02_005_rule_id_from_uuid_identity() {
        let raw = Uuid::now_v7();
        let id = RuleId::from_uuid(raw);
        assert_eq!(id.as_uuid(), raw);
    }

    #[test]
    fn test_BC_S_02_005_alert_id_from_uuid_identity() {
        let raw = Uuid::now_v7();
        let id = AlertId::from_uuid(raw);
        assert_eq!(id.as_uuid(), raw);
    }

    // ── Serde round-trip for CaseId ───────────────────────────────────────────

    #[test]
    fn test_BC_S_02_005_case_id_serde_round_trip() {
        let id = CaseId::new();
        let serialized = serde_json::to_string(&id).expect("serialize must succeed");
        let deserialized: CaseId = serde_json::from_str(&serialized).expect("deserialize must succeed");
        assert_eq!(id, deserialized, "CaseId serde round-trip must be identity");
    }

    // ── Serde round-trip for ScheduleId ──────────────────────────────────────

    #[test]
    fn test_BC_S_02_005_schedule_id_serde_round_trip() {
        let id = ScheduleId::new();
        let serialized = serde_json::to_string(&id).expect("serialize must succeed");
        let deserialized: ScheduleId =
            serde_json::from_str(&serialized).expect("deserialize must succeed");
        assert_eq!(id, deserialized, "ScheduleId serde round-trip must be identity");
    }

    // ── Hash implementation works ─────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_005_case_id_hash_works() {
        use std::collections::HashSet;
        let a = CaseId::new();
        let b = CaseId::new();
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 2, "two distinct CaseIds must hash to different buckets");
    }
}
