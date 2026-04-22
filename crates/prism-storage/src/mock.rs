// S-1.02 — MockStorageEngine for VP-055 testing.
//
// Implements `StorageBackend` with:
//   1. In-memory storage per domain (BTreeMap<StorageDomain, BTreeMap<Key, Value>>)
//   2. Failure injection: `fail_at` causes `put_batch` to fail at entry index N,
//      triggering rollback so zero entries from the batch are visible.

use std::collections::BTreeMap;

use prism_core::StorageDomain;

use crate::backend::StorageBackend;

/// In-memory storage error type.
#[derive(Debug, PartialEq, Eq)]
pub enum MockStorageError {
    /// Injected failure at batch position N.
    InjectedFailure { position: usize },
}

impl std::fmt::Display for MockStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MockStorageError::InjectedFailure { position } => {
                write!(f, "injected failure at batch position {position}")
            }
        }
    }
}

impl std::error::Error for MockStorageError {}

/// In-memory `StorageBackend` for testing.  Supports failure injection for
/// VP-055 atomicity proof.
pub struct MockStorageEngine {
    /// Per-domain in-memory store.
    data: BTreeMap<StorageDomain, BTreeMap<Vec<u8>, Vec<u8>>>,
    /// If `Some(n)`, `put_batch` will fail after writing `n` entries and roll back.
    fail_at: Option<usize>,
}

impl MockStorageEngine {
    /// Create a normal (no-failure) engine.
    pub fn new() -> Self {
        MockStorageEngine {
            data: BTreeMap::new(),
            fail_at: None,
        }
    }

    /// Create an engine that fails at position `n` in every `put_batch` call.
    pub fn new_with_failure_at(n: usize) -> Self {
        MockStorageEngine {
            data: BTreeMap::new(),
            fail_at: Some(n),
        }
    }
}

impl Default for MockStorageEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for MockStorageEngine {
    type Error = MockStorageError;

    fn get(&self, domain: StorageDomain, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(&domain).and_then(|d| d.get(key)).cloned()
    }

    fn put(
        &mut self,
        domain: StorageDomain,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), Self::Error> {
        self.data.entry(domain).or_default().insert(key, value);
        Ok(())
    }

    fn put_batch(
        &mut self,
        domain: StorageDomain,
        entries: &[(Vec<u8>, Vec<u8>)],
    ) -> Result<(), Self::Error> {
        // Check if injection is configured.
        if let Some(fail_pos) = self.fail_at {
            if fail_pos < entries.len() {
                // Fail at `fail_pos` — do not commit any entries (atomicity).
                return Err(MockStorageError::InjectedFailure { position: fail_pos });
            }
        }
        // Commit all entries.
        let domain_map = self.data.entry(domain).or_default();
        for (key, value) in entries {
            domain_map.insert(key.clone(), value.clone());
        }
        Ok(())
    }

    fn remove(&mut self, domain: StorageDomain, key: &[u8]) -> Result<(), Self::Error> {
        if let Some(d) = self.data.get_mut(&domain) {
            d.remove(key);
        }
        Ok(())
    }
}
