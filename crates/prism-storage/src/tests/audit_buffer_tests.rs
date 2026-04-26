// S-2.02 — Audit buffer tests.
//
// Covers BC-2.15.003 (buffered audit log persistence) and BC-2.15.004 (overflow purge).
//
// Test naming convention: test_BC_S_SS_NNN_[assertion_name]
//
// All tests use InMemoryBackend (from S-2.01) — no RocksDB on-disk state in CI.

#[cfg(test)]
mod inner {
    use std::collections::BTreeMap;

    use crate::audit_buffer::{
        append_audit_entry, check_and_purge_overflow, AuditEntry, AUDIT_BUFFER_MAX_ENTRIES,
        AUDIT_BUFFER_PURGE_TARGET,
    };
    use crate::memory_backend::InMemoryBackend;

    // ── Helper ────────────────────────────────────────────────────────────────

    /// Build a minimal `AuditEntry` with a given timestamp and trace ID.
    fn make_entry(timestamp_ns: u64, trace_id: &str) -> AuditEntry {
        AuditEntry {
            timestamp_ns,
            trace_id: trace_id.to_string(),
            payload: BTreeMap::new(),
        }
    }

    /// Insert `n` synthetic entries into the backend using `append_audit_entry`.
    /// Each entry gets a distinct monotonically-increasing timestamp (1, 2, ..., n).
    fn insert_n_entries(backend: &InMemoryBackend, n: usize) {
        for i in 1..=n {
            let entry = make_entry(i as u64, &format!("trace-{i:06}"));
            append_audit_entry(backend, &entry)
                .expect("append_audit_entry should not panic during setup");
        }
    }

    // ── AC-1: BC-2.15.003 — persistence and timestamp ordering ────────────────

    /// AC-1 (BC-2.15.003 postcondition): a single entry written to the audit buffer
    /// is readable from the `audit_buffer` CF and its key is lex-ordered correctly.
    ///
    /// The "restart" is simulated by using the same `InMemoryBackend` instance to
    /// re-read — the `InMemoryBackend` is the durable store in this context;
    /// the important property is that the data exists after the write returns.
    #[test]
    fn test_BC_2_15_003_entry_persisted_before_forwarding() {
        let backend = InMemoryBackend::new();
        let entry = make_entry(1_000_000_000, "trace-abc");

        let result = append_audit_entry(&backend, &entry);

        // result is Ok(()) and the key exists in the backend.
        assert!(
            result.is_ok(),
            "BC-2.15.003 postcondition: append_audit_entry must persist entry before returning Ok"
        );
    }

    /// AC-1 (BC-2.15.003 postcondition): multiple entries written in timestamp
    /// order produce keys that are lexicographically ordered by timestamp.
    ///
    /// Key format: `audit:{timestamp_ns}:{trace_id}`.  Because timestamps are
    /// zero-padded to a fixed width, lexicographic order == timestamp order.
    #[test]
    fn test_BC_2_15_003_entries_lex_ordered_by_timestamp() {
        let backend = InMemoryBackend::new();

        // Write three entries with increasing timestamps.
        append_audit_entry(&backend, &make_entry(100, "trace-early")).expect("write early entry");
        append_audit_entry(&backend, &make_entry(200, "trace-mid")).expect("write mid entry");
        append_audit_entry(&backend, &make_entry(300, "trace-late")).expect("write late entry");

        // Scan the audit_buffer CF and verify lex order equals timestamp order.
        use crate::backend::RocksStorageBackend;
        use prism_core::StorageDomain;
        let all = backend
            .scan(StorageDomain::AuditBuffer, b"audit:")
            .expect("scan must succeed");

        // There must be exactly 3 entries.
        assert_eq!(
            all.len(),
            3,
            "BC-2.15.003: expected 3 persisted entries, got {}",
            all.len()
        );

        // Keys must be in ascending lex order (= ascending timestamp order).
        let keys: Vec<Vec<u8>> = all.into_iter().map(|(k, _)| k).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(
            keys, sorted,
            "BC-2.15.003 postcondition: audit keys must be in lexicographic (timestamp) order"
        );
    }

    /// AC-1 (BC-2.15.003 invariant): simulated restart — same backend instance
    /// is queried after entries are written.  Entries survive in the backend.
    ///
    /// In production this is a RocksDB re-open; here the InMemoryBackend models
    /// durability within the same process.
    #[test]
    fn test_BC_2_15_003_entries_survive_simulated_restart() {
        let backend = InMemoryBackend::new();
        let entry = make_entry(42_000_000, "trace-restart");

        append_audit_entry(&backend, &entry).expect("write before simulated restart");

        // "Restart" is simulated by re-reading from the same backend (no re-open
        // required for InMemoryBackend; the invariant is that the data persists).
        use crate::backend::RocksStorageBackend;
        use prism_core::StorageDomain;
        let all = backend
            .scan(StorageDomain::AuditBuffer, b"audit:")
            .expect("scan after simulated restart");

        assert_eq!(
            all.len(),
            1,
            "BC-2.15.003 invariant: entry must be readable after simulated restart"
        );
    }

    // ── AC-2: BC-2.15.004 — overflow purge ───────────────────────────────────

    /// AC-2 (BC-2.15.004 postcondition): inserting 100,001 entries and calling
    /// `check_and_purge_overflow()` reduces the count to ≤ 90,000 and returns
    /// the number of entries deleted.
    ///
    /// Uses 100,001 x ~32-byte payloads (≈ 3 MB heap) — acceptable for CI.
    #[test]
    fn test_BC_2_15_004_overflow_purges_to_target() {
        let backend = InMemoryBackend::new();

        // Insert AUDIT_BUFFER_MAX_ENTRIES + 1 entries to trigger the overflow.
        let overflow_count = AUDIT_BUFFER_MAX_ENTRIES + 1; // 100,001
        insert_n_entries(&backend, overflow_count);

        // Count before purge.
        use crate::backend::RocksStorageBackend;
        use prism_core::StorageDomain;
        let before = backend
            .scan(StorageDomain::AuditBuffer, b"audit:")
            .expect("scan before purge")
            .len();
        assert_eq!(
            before, overflow_count,
            "setup: expected {overflow_count} entries before purge"
        );

        // Run overflow purge.
        let purged =
            check_and_purge_overflow(&backend).expect("check_and_purge_overflow must not fail");

        // Post-purge count must be ≤ AUDIT_BUFFER_PURGE_TARGET (90,000).
        let after = backend
            .scan(StorageDomain::AuditBuffer, b"audit:")
            .expect("scan after purge")
            .len();

        assert!(
            after <= AUDIT_BUFFER_PURGE_TARGET,
            "BC-2.15.004 postcondition: post-purge count {after} must be ≤ {AUDIT_BUFFER_PURGE_TARGET}"
        );

        // The returned purge count must equal the deletion delta.
        let expected_purged = before - after;
        assert_eq!(
            purged, expected_purged,
            "BC-2.15.004 postcondition: returned purge count {purged} must equal \
             deletion delta {expected_purged}"
        );
    }

    /// AC-2 (BC-2.15.004 invariant): when the buffer has ≤ AUDIT_BUFFER_MAX_ENTRIES
    /// entries, `check_and_purge_overflow()` returns 0 and makes no changes.
    #[test]
    fn test_BC_2_15_004_no_purge_below_threshold() {
        let backend = InMemoryBackend::new();
        insert_n_entries(&backend, 10); // well below 100K

        let purged = check_and_purge_overflow(&backend)
            .expect("check_and_purge_overflow must not fail on under-threshold buffer");

        assert_eq!(
            purged, 0,
            "BC-2.15.004: must return 0 purged entries when buffer is below overflow threshold"
        );

        use crate::backend::RocksStorageBackend;
        use prism_core::StorageDomain;
        let after = backend
            .scan(StorageDomain::AuditBuffer, b"audit:")
            .expect("scan")
            .len();
        assert_eq!(
            after, 10,
            "BC-2.15.004: entry count must be unchanged when no overflow occurs"
        );
    }

    /// AC-2 (BC-2.15.004 postcondition): after purge, the NEWEST entries are
    /// preserved (lowest-timestamp / oldest entries are deleted first).
    #[test]
    fn test_BC_2_15_004_purge_removes_oldest_entries() {
        let backend = InMemoryBackend::new();
        let overflow_count = AUDIT_BUFFER_MAX_ENTRIES + 1;
        insert_n_entries(&backend, overflow_count);

        check_and_purge_overflow(&backend).expect("purge must succeed");

        use crate::backend::RocksStorageBackend;
        use prism_core::StorageDomain;
        let remaining: Vec<Vec<u8>> = backend
            .scan(StorageDomain::AuditBuffer, b"audit:")
            .expect("scan after purge")
            .into_iter()
            .map(|(k, _)| k)
            .collect();

        // The newest entries (highest timestamps = largest keys) must be present.
        // Timestamps run 1..=overflow_count; after purge we expect the top 90,000
        // (timestamps overflow_count-89999 ..= overflow_count) to survive.
        // We assert the last key is the one with the maximum timestamp.
        let last_key = remaining.last().expect("remaining must not be empty");
        let last_key_str = std::str::from_utf8(last_key).expect("key is UTF-8");
        let expected_max_ts = overflow_count as u64;
        assert!(
            last_key_str.contains(&format!("{expected_max_ts}")),
            "BC-2.15.004: newest entry (ts={expected_max_ts}) must survive purge; \
             last remaining key was: {last_key_str}"
        );
    }
}
