//! `cursor` — Ephemeral internal pagination cursor for multi-page sensor API fetches.
//!
//! Implements BC-2.07.001 and BC-2.07.002. Pagination is entirely **internal** to
//! the query engine's sensor fetch layer. Cursor tokens are never exposed to the
//! MCP agent — the agent sees only `limit` and `total_available` in the `query`
//! tool response (BC-2.07.001 v3.0 note).
//!
//! # Key types
//! - [`CursorToken`] — opaque UUID v4 string identifying an in-flight fetch cursor.
//! - [`CursorEntry`] — in-memory result-set slice with lifecycle metadata.
//! - [`QueryCursorRegistry`] — process-singleton registry wrapping the prism-core
//!   `CursorRegistry` for cap enforcement.
//!
//! # BC References
//! - BC-2.07.001 — Internal Ephemeral Pagination Token Structure
//! - BC-2.07.002 — Internal Pagination Token Lifecycle
//!
//! Story: S-3.05

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use prism_core::cursor::{CursorId, CursorRegistry};
use prism_core::error::PrismError;
use prism_core::OrgSlug;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use uuid::Uuid;

/// Cursor expiry timeout (BC-2.07.002 — 60s from creation).
pub const CURSOR_EXPIRY_SECS: u64 = 60;

/// Background cleanup interval (BC-2.07.002 — wake every 30s).
pub const CLEANUP_INTERVAL_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// CursorToken
// ---------------------------------------------------------------------------

/// Opaque cursor identifier — a UUID v4 string.
///
/// Returned to the internal fetch loop; never exposed to the MCP agent
/// (BC-2.07.001 postconditions — tokens are internal only).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CursorToken(pub String);

impl CursorToken {
    /// Generate a fresh random `CursorToken` (UUID v4).
    pub fn new_random() -> Self {
        CursorToken(Uuid::new_v4().to_string())
    }
}

// ---------------------------------------------------------------------------
// CursorEntry
// ---------------------------------------------------------------------------

/// In-memory entry for one in-flight pagination cursor.
///
/// Holds the complete un-paged result set and the current read offset.
/// Entry lives only in process memory — never serialized to disk (BC-2.07.001
/// "Tokens are never persisted to disk").
///
/// # I-3 / SEC-006: Field visibility
/// All fields are `pub(crate)` — external crates cannot construct a `CursorEntry`
/// directly, which would bypass the cap registry enforced by `QueryCursorRegistry`.
/// Only `QueryCursorRegistry` (same crate) constructs entries during `create()`.
///
/// # SEC-002: Manual Debug implementation
/// `query_str` is redacted in the Debug output because it may contain
/// sensitive filter values (e.g., IP addresses, host names, user identifiers).
/// Using `#[derive(Debug)]` would leak `query_str` into logs and traces.
pub struct CursorEntry {
    /// Complete result rows for this fetch, stored until fully consumed or expired.
    /// Wrapped in Arc so that token rotation in `next_page()` is O(1) (pointer
    /// increment) rather than O(N) (full Vec clone) — CR-008.
    pub(crate) result_rows: Arc<Vec<serde_json::Value>>,
    /// Current page offset (monotonically increasing — BC-2.07.002 forward-only).
    pub(crate) offset: usize,
    /// Timestamp at which this entry was created, for expiry checking.
    pub(crate) created_at: Instant,
    /// The originating PrismQL query string (for diagnostics).
    /// Redacted in Debug output (SEC-002).
    pub(crate) query_str: String,
    /// The client (tenant) that owns this cursor.
    pub(crate) client_id: OrgSlug,
    /// The prism-core CursorId allocated for this entry (used for release).
    pub(crate) core_id: CursorId,
}

impl std::fmt::Debug for CursorEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CursorEntry")
            .field("result_rows_count", &self.result_rows.len())
            .field("offset", &self.offset)
            .field("created_at", &self.created_at)
            .field("query_str", &"<redacted>") // SEC-002: query string may contain sensitive data
            .field("client_id", &self.client_id)
            .field("core_id", &self.core_id)
            .finish()
    }
}

impl CursorEntry {
    /// Returns `true` if this cursor has expired (more than CURSOR_EXPIRY_SECS elapsed).
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > Duration::from_secs(CURSOR_EXPIRY_SECS)
    }
}

// ---------------------------------------------------------------------------
// QueryCursorRegistry
// ---------------------------------------------------------------------------

/// Process-level registry of all active in-flight pagination cursors.
///
/// Wraps [`prism_core::cursor::CursorRegistry`] for cap enforcement (VP-029)
/// and stores per-cursor result sets internally (BC-2.07.001 / BC-2.07.002).
///
/// Must be constructed once during `QueryEngine` initialization and shared via
/// `Arc<Mutex<_>>`.
pub struct QueryCursorRegistry {
    /// prism-core registry: enforces the 200-cursor cap (VP-029 / S-1.02).
    core_registry: CursorRegistry,
    /// Active cursor entries keyed by token.
    entries: HashMap<CursorToken, CursorEntry>,
}

impl QueryCursorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        QueryCursorRegistry {
            core_registry: CursorRegistry::new(),
            entries: HashMap::new(),
        }
    }

    /// Store a complete result set and return the first page plus an optional
    /// continuation token.
    ///
    /// If `rows.len() <= page_size`, all rows are returned and the token is
    /// `None` (single-page result). Otherwise the first `page_size` rows are
    /// returned and a token is issued for the remaining rows (BC-2.07.001).
    ///
    /// Returns `Err(PrismError::CursorCapExceeded)` (via the prism-core
    /// `CursorRegistry::allocate()` call) when 200 cursors are already active
    /// (BC-2.07.002 — 200-cursor cap).
    ///
    /// # BC References
    /// - BC-2.07.001 postconditions
    /// - BC-2.07.002 §Concurrent Fetch Limits
    pub fn create(
        &mut self,
        rows: Vec<serde_json::Value>,
        page_size: usize,
        query_str: String,
        client_id: OrgSlug,
    ) -> Result<(Vec<serde_json::Value>, Option<CursorToken>), PrismError> {
        // CR-011: guard against page_size=0 which would cause an infinite loop.
        if page_size == 0 {
            return Err(PrismError::QueryExecutionFailed {
                detail: "E-QUERY-005: page_size must be greater than 0".to_string(),
            });
        }

        // Dedup rows by "id" field (EC-07-020: adapter-level deduplication).
        let rows = deduplicate_by_id(rows);

        // SEC-004: Check single-page BEFORE allocating from the cap registry.
        // Allocating and immediately releasing for single-page results wastes a slot
        // and inflates the cursor cap counter transiently.
        if rows.len() <= page_size {
            // Single-page result — return without allocating.
            return Ok((rows, None));
        }

        // Only allocate from the prism-core cap registry when a cursor will actually
        // be created (multi-page result). This enforces the 200-cursor cap for
        // cursors that persist (BC-2.07.002, VP-029, SEC-004).
        let core_id = self.core_registry.allocate()?;

        let token = CursorToken::new_random();
        let first_page = rows[..page_size].to_vec();
        let total_rows = rows.len();
        let rows = Arc::new(rows);

        // I-1: log cursor creation with diagnostic fields (no query string — SEC-002).
        // client_id is the org-slug scope for this cursor; there is no sensor_id at
        // cursor-creation time (pre-fetch), so we emit client_id only.
        debug!(
            client_id = %client_id,
            total_rows,
            page_size,
            active_cursors = self.core_registry.active_count(),
            "cursor created"
        );

        let entry = CursorEntry {
            result_rows: rows,
            offset: page_size,
            created_at: Instant::now(),
            query_str,
            client_id,
            core_id,
        };

        self.entries.insert(token.clone(), entry);
        Ok((first_page, Some(token)))
    }

    /// Advance the cursor by one page and return the next slice plus an
    /// optional new token.
    ///
    /// On expiry (`created_at.elapsed() > 60s`), removes the entry, releases
    /// the prism-core allocation, and returns `Err(PrismError::QueryExecutionFailed)`
    /// with an E-QUERY-004 message (BC-2.07.002 §Fetch Timeout semantics).
    ///
    /// Returns `None` token when the last page is returned (cursor exhausted).
    /// On exhaustion, the prism-core allocation is released.
    ///
    /// # BC References
    /// - BC-2.07.001 postconditions — forward-only offset
    /// - BC-2.07.002 §Forward-Only Progress
    pub fn next_page(
        &mut self,
        token: CursorToken,
        page_size: usize,
    ) -> Result<(Vec<serde_json::Value>, Option<CursorToken>), PrismError> {
        let entry = self
            .entries
            .get(&token)
            .ok_or_else(|| PrismError::QueryExecutionFailed {
                detail: E_QUERY_004_CURSOR_EXPIRED.to_string(),
            })?;

        // Check expiry (BC-2.07.002).
        if entry.is_expired() {
            let core_id = entry.core_id;
            self.entries.remove(&token);
            self.core_registry.release(core_id);
            // I-1: log cursor expiry lifecycle event after release so `remaining`
            // reflects the post-removal count (matches evict_expired pattern).
            debug!(
                remaining = self.core_registry.active_count(),
                "cursor expired on next_page"
            );
            return Err(PrismError::QueryExecutionFailed {
                detail: E_QUERY_004_CURSOR_EXPIRED.to_string(),
            });
        }

        let offset = entry.offset;
        let total = entry.result_rows.len();

        if offset >= total {
            // Already exhausted.
            let core_id = entry.core_id;
            self.entries.remove(&token);
            self.core_registry.release(core_id);
            return Ok((Vec::new(), None));
        }

        let end = (offset + page_size).min(total);
        let page = entry.result_rows[offset..end].to_vec();
        let is_last = end >= total;

        if is_last {
            // Cursor exhausted — release and return None token.
            let core_id = entry.core_id;
            self.entries.remove(&token);
            self.core_registry.release(core_id);
            Ok((page, None))
        } else {
            // Update offset and issue a new token.
            let new_token = CursorToken::new_random();
            let core_id = entry.core_id;
            let query_str = entry.query_str.clone();
            let client_id = entry.client_id.clone();
            let result_rows = Arc::clone(&entry.result_rows); // CR-008: O(1) pointer increment
            let created_at = entry.created_at;

            // Remove old token, insert with new token.
            self.entries.remove(&token);
            self.entries.insert(
                new_token.clone(),
                CursorEntry {
                    result_rows,
                    offset: end,
                    created_at,
                    query_str,
                    client_id,
                    core_id,
                },
            );

            Ok((page, Some(new_token)))
        }
    }

    /// Evict all entries whose `created_at.elapsed() > 60s`.
    ///
    /// Called by the background cleanup task every 30 seconds to prevent
    /// unbounded memory growth (BC-2.07.002 §Background Cleanup).
    ///
    /// Releases each expired entry's prism-core allocation.
    pub fn evict_expired(&mut self) {
        let expired_tokens: Vec<CursorToken> = self
            .entries
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(t, _)| t.clone())
            .collect();

        let count = expired_tokens.len();
        for token in expired_tokens {
            if let Some(entry) = self.entries.remove(&token) {
                self.core_registry.release(entry.core_id);
            }
        }

        // I-1: log background eviction counts (diagnostic, no PII).
        if count > 0 {
            debug!(
                evicted = count,
                remaining = self.core_registry.active_count(),
                "cursor background eviction complete"
            );
        }
    }

    /// Returns the count of currently active cursors (for health / metrics).
    pub fn active_count(&self) -> usize {
        self.core_registry.active_count()
    }
}

impl Default for QueryCursorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Deduplication helper (EC-07-020)
// ---------------------------------------------------------------------------

/// Deduplicate rows by their `"id"` field value.
///
/// BC-2.07.002 EC-07-020: the adapter deduplicates at the Prism level.
/// Rows without an `"id"` field are kept as-is (no dedup key).
/// Preserves first occurrence (stable ordering).
///
/// # SEC-005: Polymorphic ID handling (Claroty and similar sensors)
/// Some sensors (e.g., Claroty) return numeric or boolean `"id"` values.
/// Using `v.as_str()` would silently ignore numeric IDs, allowing duplicates
/// through. We use `v.to_string()` for all non-null `"id"` values to ensure
/// deduplication works regardless of JSON type.
fn deduplicate_by_id(rows: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(id_val) = row.get("id") {
            if id_val.is_null() {
                // Null "id" — keep without dedup (no meaningful key).
                result.push(row);
            } else {
                // SEC-005: use to_string() to handle numeric, boolean, and string IDs.
                let id_str = id_val.to_string();
                if seen_ids.insert(id_str) {
                    result.push(row);
                }
            }
        } else {
            // No "id" field — keep without dedup.
            result.push(row);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Background cleanup task
// ---------------------------------------------------------------------------

/// Spawn a background tokio task that wakes every [`CLEANUP_INTERVAL_SECS`]
/// seconds and evicts expired cursors from `registry`.
///
/// # Shutdown semantics (CR-002 / CR-008)
/// The task exits when `shutdown` is cancelled (i.e., `shutdown.cancel()` is
/// called). This is typically done from `Drop` of the owning `QueryEngine`.
///
/// Without a cancellation token the task would hold an `Arc` clone of
/// `registry`, preventing deallocation even after all external references are
/// dropped. The previous doc comment claiming "exits when Arc is dropped" was
/// incorrect — the task itself held a strong reference.
///
/// If the `registry` mutex is poisoned (a thread panicked while holding it),
/// the task exits cleanly rather than propagating the poison.
///
/// The task MUST be started once during `QueryEngine` initialization
/// (BC-2.07.002 §Background Cleanup).
///
/// # Returns
/// A `JoinHandle` that resolves when the task exits. The caller should store
/// this handle and await it on shutdown for clean teardown.
pub fn spawn_cursor_cleanup_task(
    registry: Arc<Mutex<QueryCursorRegistry>>,
    shutdown: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));
        // CR-017: Tokio's default `MissedTickBehavior::Burst` would also fire the
        // first tick immediately at t=0. Setting `Skip` ensures the first cleanup
        // runs after the full interval, not at spawn time.
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    // Clean shutdown — stop the cleanup loop.
                    break;
                }
                _ = interval.tick() => {
                    let mut reg = match registry.lock() {
                        Ok(r) => r,
                        Err(_poisoned) => {
                            // Mutex poisoned — exit cleanly rather than propagating.
                            // I-1 / O-5: log the poison event so operators can detect it.
                            error!("cursor registry mutex poisoned; background cleanup task exiting");
                            break;
                        }
                    };
                    reg.evict_expired();
                }
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Error code constants (BC-2.07.002)
// ---------------------------------------------------------------------------

/// E-QUERY-004: cursor expired — returned by `next_page()` when the cursor has
/// lived longer than [`CURSOR_EXPIRY_SECS`].
///
/// Note: `PrismError::QueryExecutionFailed` carries this code in its `detail`
/// field; there is no dedicated enum variant.
pub const E_QUERY_004_CURSOR_EXPIRED: &str =
    "E-QUERY-004: pagination cursor expired (>60s); re-execute the query";

/// E-QUERY-002: cursor cap exceeded — forwarded from `PrismError::CursorCapExceeded`
/// (prism-core, S-1.02 / VP-029).
///
/// Note: The prism-core variant string is "E-STORE-020: cursor cap exceeded"; this
/// constant documents the story-level semantic alias used in AC-4.
pub const E_QUERY_002_CURSOR_CAP: &str = "E-QUERY-002: query planning failed: cursor cap exceeded";

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify CursorExpiry constant matches BC-2.07.002 (60 seconds).
    ///
    /// GREEN-BY-DESIGN: pure constant comparison, zero branching, no I/O, 1 line.
    #[test]
    fn test_cursor_expiry_constant_is_60_seconds() {
        assert_eq!(CURSOR_EXPIRY_SECS, 60);
    }

    /// Verify cleanup interval constant matches BC-2.07.002 (30 seconds).
    ///
    /// GREEN-BY-DESIGN: pure constant comparison, zero branching, no I/O, 1 line.
    #[test]
    fn test_cleanup_interval_constant_is_30_seconds() {
        assert_eq!(CLEANUP_INTERVAL_SECS, 30);
    }

    /// Verify registry starts with zero active cursors.
    ///
    /// GREEN-BY-DESIGN: constructor + active_count(); zero branching, no I/O,
    /// 2 lines. The `active_count()` delegates to `CursorRegistry::active_count()`
    /// which is a pure len() call.
    #[test]
    fn test_registry_starts_empty() {
        let reg = QueryCursorRegistry::new();
        assert_eq!(reg.active_count(), 0);
    }
}
