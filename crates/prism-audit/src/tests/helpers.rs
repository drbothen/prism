//! Shared test helpers for prism-audit tests (S-2.04).
//!
//! Provides:
//! - `MemBackend` — a `Clone + RocksStorageBackend` wrapping `Arc<InMemoryBackend>`.
//! - `FailingBackend` — a `Clone + RocksStorageBackend` whose `put()` always fails.
//! - `count_audit_entries()` — count entries in the audit_buffer CF.
//! - `make_request()` — build a minimal `AuditedRequest` for tests.
//! - `AlwaysSucceedService` / `AlwaysFailService` — minimal Tower services.

use std::sync::Arc;
use std::task::{Context, Poll};

use prism_core::{PrismError, StorageDomain};
use prism_storage::backend::RocksStorageBackend;
use prism_storage::memory_backend::InMemoryBackend;

use crate::audit_emitter::{AuditedRequest, AuditedResponse};
use crate::audit_entry::{AuditOutcome, DataClassification};

// ── MemBackend — Clone-able wrapper over InMemoryBackend ─────────────────────

/// A `Clone`-able, `RocksStorageBackend`-implementing wrapper over `InMemoryBackend`.
///
/// `AuditEmitterService` requires `B: Clone`; `InMemoryBackend` doesn't implement
/// `Clone` directly (contains `RwLock`), so we wrap it in `Arc<>`.
///
/// All clones share the same underlying storage, which is what we want for tests.
#[derive(Clone)]
pub struct MemBackend {
    inner: Arc<InMemoryBackend>,
}

impl Default for MemBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MemBackend {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InMemoryBackend::new()),
        }
    }
}

unsafe impl Send for MemBackend {}
unsafe impl Sync for MemBackend {}

impl RocksStorageBackend for MemBackend {
    fn get(&self, domain: StorageDomain, key: &[u8]) -> Result<Option<Vec<u8>>, PrismError> {
        self.inner.get(domain, key)
    }

    fn put(&self, domain: StorageDomain, key: &[u8], value: &[u8]) -> Result<(), PrismError> {
        self.inner.put(domain, key, value)
    }

    fn put_batch(
        &self,
        domain: StorageDomain,
        entries: &[(&[u8], &[u8])],
    ) -> Result<(), PrismError> {
        self.inner.put_batch(domain, entries)
    }

    fn remove(&self, domain: StorageDomain, key: &[u8]) -> Result<(), PrismError> {
        self.inner.remove(domain, key)
    }

    fn scan(
        &self,
        domain: StorageDomain,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        self.inner.scan(domain, prefix)
    }

    fn scan_range(
        &self,
        domain: StorageDomain,
        start: &[u8],
        end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        self.inner.scan_range(domain, start, end)
    }
}

/// Count entries in the `audit_buffer` CF.
pub fn count_audit_entries(backend: &MemBackend) -> usize {
    backend
        .scan(StorageDomain::AuditBuffer, b"audit:")
        .unwrap_or_default()
        .len()
}

// ── FailingBackend ────────────────────────────────────────────────────────────

/// A storage backend whose `put()` always returns
/// `PrismError::StorageWriteFailed`. Used to simulate audit emission failures.
#[derive(Clone)]
pub struct FailingBackend;

impl Default for FailingBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FailingBackend {
    pub fn new() -> Self {
        Self
    }
}

unsafe impl Send for FailingBackend {}
unsafe impl Sync for FailingBackend {}

impl RocksStorageBackend for FailingBackend {
    fn get(&self, _domain: StorageDomain, _key: &[u8]) -> Result<Option<Vec<u8>>, PrismError> {
        Ok(None)
    }

    fn put(&self, domain: StorageDomain, _key: &[u8], _value: &[u8]) -> Result<(), PrismError> {
        Err(PrismError::StorageWriteFailed {
            domain: domain.column_family_name().to_owned(),
            detail: "FailingBackend: write always fails".to_owned(),
        })
    }

    fn put_batch(
        &self,
        domain: StorageDomain,
        _entries: &[(&[u8], &[u8])],
    ) -> Result<(), PrismError> {
        Err(PrismError::StorageWriteFailed {
            domain: domain.column_family_name().to_owned(),
            detail: "FailingBackend: write always fails".to_owned(),
        })
    }

    fn remove(&self, domain: StorageDomain, _key: &[u8]) -> Result<(), PrismError> {
        Err(PrismError::StorageWriteFailed {
            domain: domain.column_family_name().to_owned(),
            detail: "FailingBackend: remove always fails".to_owned(),
        })
    }

    fn scan(
        &self,
        _domain: StorageDomain,
        _prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        Ok(vec![])
    }

    fn scan_range(
        &self,
        _domain: StorageDomain,
        _start: &[u8],
        _end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        Ok(vec![])
    }
}

// ── make_request ──────────────────────────────────────────────────────────────

/// Build a minimal `AuditedRequest` for the given tool name.
pub fn make_request(tool_name: &str) -> AuditedRequest {
    AuditedRequest {
        tool_name: tool_name.to_owned(),
        client_id: "test_client".to_owned(),
        user_identity: "analyst@example.com".to_owned(),
        parameters: serde_json::json!({"query": "test"}),
        data_classification: DataClassification::Internal,
        capability_checks: vec![],
        safety_flags: vec![],
    }
}

// ── AlwaysSucceedService ─────────────────────────────────────────────────────

/// A minimal Tower service that always responds with `AuditOutcome::Success`.
#[derive(Clone)]
pub struct AlwaysSucceedService;

impl tower::Service<AuditedRequest> for AlwaysSucceedService {
    type Response = AuditedResponse;
    type Error = PrismError;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<AuditedResponse, PrismError>> + Send>,
    >;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: AuditedRequest) -> Self::Future {
        Box::pin(async {
            Ok(AuditedResponse {
                outcome: AuditOutcome::Success,
                result_summary: "ok".to_owned(),
                error_code: None,
            })
        })
    }
}

// ── AlwaysFailService ─────────────────────────────────────────────────────────

/// A minimal Tower service that always returns an inner error.
#[derive(Clone)]
pub struct AlwaysFailService;

impl tower::Service<AuditedRequest> for AlwaysFailService {
    type Response = AuditedResponse;
    type Error = PrismError;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<AuditedResponse, PrismError>> + Send>,
    >;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: AuditedRequest) -> Self::Future {
        Box::pin(async {
            Err(PrismError::Internal {
                detail: "inner handler failed".to_owned(),
            })
        })
    }
}
