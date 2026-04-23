// S-1.02 — CursorId and CursorRegistry enforcing the 200-cursor cap (VP-029).
//
// `CursorRegistry` is a plain (non-async) struct.  The query engine (S-3.05)
// wraps it in `Arc<Mutex<CursorRegistry>>` for concurrent access.

use std::collections::BTreeSet;

use crate::error::PrismError;

/// Maximum number of simultaneously active cursors across all pagination consumers.
/// Owned by SS-07 (Adapter Pagination); enforced here at the allocation boundary.
pub const CURSOR_CAP: usize = 200;

/// Opaque monotonically-incrementing cursor identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CursorId(pub u64);

/// Registry that tracks all live cursors and enforces the 200-cap policy.
///
/// Not async-safe at this layer — wrap in `Arc<Mutex<_>>` in the query engine.
pub struct CursorRegistry {
    active: BTreeSet<CursorId>,
    next_id: u64,
}

impl CursorRegistry {
    /// Create a new, empty registry.
    pub fn new() -> Self {
        CursorRegistry {
            active: BTreeSet::new(),
            next_id: 0,
        }
    }

    /// Allocate a new cursor.
    ///
    /// Returns `Err(PrismError::CursorCapExceeded)` if `active.len() >= 200`.
    /// Returns `Ok(CursorId)` otherwise.
    ///
    /// AC-6: 201st allocation with 200 already active → `Err`.
    /// AC-7: release one, then allocate → `Ok` (cap is active count, not lifetime).
    pub fn allocate(&mut self) -> Result<CursorId, PrismError> {
        if self.active.len() >= CURSOR_CAP {
            return Err(PrismError::CursorCapExceeded);
        }
        let id = CursorId(self.next_id);
        self.next_id += 1;
        self.active.insert(id);
        Ok(id)
    }

    /// Release an active cursor, making room for future allocations.
    pub fn release(&mut self, id: CursorId) {
        self.active.remove(&id);
    }

    /// Returns the current number of active cursors (for health reporting).
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

impl Default for CursorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
