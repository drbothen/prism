// VP-029: CursorRegistry enforces the 200-cursor cap.
//
// Proves:
//   1. Exactly 200 allocations succeed.
//   2. The 201st returns Err(CursorCapExceeded).
//   3. After releasing one, the next allocation succeeds (active-count cap, not
//      lifetime-count cap).

#[cfg(kani)]
mod kani_proofs {
    use crate::cursor::{CursorRegistry, CURSOR_CAP};
    use crate::error::PrismError;

    /// VP-029 — cap of 200 is enforced; release+allocate works.
    #[kani::proof]
    fn proof_cursor_cap_200() {
        let mut registry = CursorRegistry::new();
        let mut last_id = None;

        // Allocate exactly CURSOR_CAP (200) cursors — all must succeed.
        for _ in 0..CURSOR_CAP {
            let id = registry.allocate();
            kani::assert(id.is_ok(), "first 200 allocations must succeed");
            last_id = id.ok();
        }

        kani::assert(
            registry.active_count() == CURSOR_CAP,
            "active count must be exactly 200 after 200 allocations",
        );

        // 201st allocation must fail with CursorCapExceeded.
        let over_cap = registry.allocate();
        kani::assert(
            matches!(over_cap, Err(PrismError::CursorCapExceeded)),
            "201st allocation must return Err(CursorCapExceeded)",
        );

        // Release one cursor, then re-allocate — must succeed (active-count cap).
        if let Some(id) = last_id {
            registry.release(id);
            kani::assert(
                registry.active_count() == CURSOR_CAP - 1,
                "active count must decrease after release",
            );

            let realloc = registry.allocate();
            kani::assert(
                realloc.is_ok(),
                "allocation after release must succeed (cap is active-count, not lifetime-count)",
            );
        }
    }
}
