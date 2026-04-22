// Unit tests for CursorId and CursorRegistry.
//
// AC coverage: AC-6, AC-7
// VP coverage: VP-029 (cursor cap 200)

#[cfg(test)]
mod tests {
    use crate::cursor::{CursorRegistry, CURSOR_CAP};
    use crate::error::PrismError;

    // ── AC-6: 201st allocation fails ─────────────────────────────────────────

    #[test]
    fn test_BC_S_02_004_ac6_201st_allocation_fails() {
        let mut registry = CursorRegistry::new();
        // Allocate exactly 200 cursors.
        for _ in 0..CURSOR_CAP {
            registry.allocate().expect("allocation under cap must succeed");
        }
        // 201st must fail.
        let result = registry.allocate();
        assert_eq!(
            result,
            Err(PrismError::CursorCapExceeded),
            "201st allocation must return CursorCapExceeded"
        );
    }

    // ── AC-7: release + re-allocate succeeds ──────────────────────────────────

    #[test]
    fn test_BC_S_02_004_ac7_release_then_allocate_succeeds() {
        let mut registry = CursorRegistry::new();
        // Fill to cap.
        let mut ids = Vec::with_capacity(CURSOR_CAP);
        for _ in 0..CURSOR_CAP {
            ids.push(registry.allocate().expect("allocation under cap must succeed"));
        }
        // Release one.
        let released = ids.pop().unwrap();
        registry.release(released);
        assert_eq!(registry.active_count(), CURSOR_CAP - 1);

        // Now allocation must succeed.
        let result = registry.allocate();
        assert!(
            result.is_ok(),
            "allocation after release must succeed — cap is active count, not lifetime count"
        );
    }

    // ── VP-029: boundary at exactly 200 ──────────────────────────────────────

    #[test]
    fn test_BC_S_02_004_vp029_active_count_at_cap() {
        let mut registry = CursorRegistry::new();
        for _ in 0..CURSOR_CAP {
            registry.allocate().unwrap();
        }
        assert_eq!(registry.active_count(), CURSOR_CAP);
    }

    // ── VP-029: 199 allocations succeed ──────────────────────────────────────

    #[test]
    fn test_BC_S_02_004_vp029_199_allocations_succeed() {
        let mut registry = CursorRegistry::new();
        for i in 0..CURSOR_CAP - 1 {
            let r = registry.allocate();
            assert!(r.is_ok(), "allocation {i} must succeed");
        }
        assert_eq!(registry.active_count(), CURSOR_CAP - 1);
    }

    // ── Empty registry has zero active cursors ────────────────────────────────

    #[test]
    fn test_BC_S_02_004_empty_registry_has_zero_active_count() {
        let registry = CursorRegistry::new();
        assert_eq!(registry.active_count(), 0);
    }

    // ── Release of unknown cursor is safe (no panic) ─────────────────────────

    #[test]
    fn test_BC_S_02_004_release_unknown_cursor_does_not_panic() {
        let mut registry = CursorRegistry::new();
        let id = registry.allocate().unwrap();
        registry.release(id);
        // Releasing again should not panic.
        registry.release(id);
    }

    // ── IDs from successive allocations are distinct ──────────────────────────

    #[test]
    fn test_BC_S_02_004_allocated_ids_are_distinct() {
        let mut registry = CursorRegistry::new();
        let a = registry.allocate().unwrap();
        let b = registry.allocate().unwrap();
        assert_ne!(a, b, "two successive cursor IDs must be distinct");
    }
}
