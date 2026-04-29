//! OrgRegistry — bijective BiMap between `OrgSlug` and `OrgId`.
//!
//! Implements BC-3.1.001, BC-3.1.003, BC-3.1.004 (S-3.1.03).
//!
//! The registry is built once at startup from `customers/*.toml` files and is
//! thereafter read-only on the hot path.  `resolve` and `slug_for` perform no
//! filesystem I/O (BC-3.1.001 invariant 4).  `register` is the sole write path
//! (BC-3.1.004 invariant 1); the BiMap inner field is private (BC-3.1.003
//! precondition 3).

use std::sync::RwLock;

use bimap::BiMap;

use crate::ids::OrgId;
use crate::tenant::OrgSlug;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors returned by [`OrgRegistry::register`].
///
/// BC-3.1.004 postconditions 2, 3, 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationError {
    /// A different `OrgId` is already bound to `slug`.
    ///
    /// BC-3.1.004 postcondition 2.
    SlugConflict {
        slug: OrgSlug,
        existing_id: OrgId,
        attempted_id: OrgId,
    },
    /// A different `OrgSlug` is already bound to `id`.
    ///
    /// BC-3.1.004 postcondition 3.
    IdConflict {
        id: OrgId,
        existing_slug: OrgSlug,
        attempted_slug: OrgSlug,
    },
}

impl std::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrationError::SlugConflict {
                slug,
                existing_id,
                attempted_id,
            } => write!(
                f,
                "slug '{slug}' is already bound to org {existing_id}; \
                 cannot rebind to {attempted_id}"
            ),
            RegistrationError::IdConflict {
                id,
                existing_slug,
                attempted_slug,
            } => write!(
                f,
                "org {id} is already bound to slug '{existing_slug}'; \
                 cannot rebind to '{attempted_slug}'"
            ),
        }
    }
}

impl std::error::Error for RegistrationError {}

// ---------------------------------------------------------------------------
// OrgRegistry
// ---------------------------------------------------------------------------

/// Bijective registry mapping `OrgSlug` <-> `OrgId`.
///
/// Backed by a `BiMap` wrapped in an `RwLock` for concurrent read access.
/// Concurrent reads are safe and contention-free; writes (at startup only) are
/// serialised by the lock.
///
/// The BiMap field is intentionally private — all mutations go through
/// [`register`](OrgRegistry::register) (BC-3.1.004 invariant 1, BC-3.1.003
/// precondition 3).
pub struct OrgRegistry {
    inner: RwLock<BiMap<OrgSlug, OrgId>>,
}

impl OrgRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(BiMap::new()),
        }
    }

    /// Resolve a slug to its canonical `OrgId`, or `None` if not registered.
    ///
    /// BC-3.1.001 postconditions 1–4. Pure read; no I/O.
    pub fn resolve(&self, slug: &OrgSlug) -> Option<OrgId> {
        self.inner
            .read()
            .expect("OrgRegistry RwLock poisoned")
            .get_by_left(slug)
            .copied()
    }

    /// Return the `OrgSlug` bound to `id`, or `None` if not registered.
    ///
    /// BC-3.1.001 postcondition 4. Pure read; no I/O.
    pub fn slug_for(&self, id: &OrgId) -> Option<OrgSlug> {
        self.inner
            .read()
            .expect("OrgRegistry RwLock poisoned")
            .get_by_right(id)
            .cloned()
    }

    /// Register a `(slug, id)` pair.
    ///
    /// - Returns `Ok(())` if the pair is new or already identical (idempotent,
    ///   BC-3.1.004 postcondition 4 / D-050).
    /// - Returns `Err(RegistrationError::SlugConflict { .. })` if `slug` is
    ///   already bound to a *different* `OrgId`.
    /// - Returns `Err(RegistrationError::IdConflict { .. })` if `id` is already
    ///   bound to a *different* `OrgSlug`.
    ///
    /// On error the registry is left unchanged (BC-3.1.004 postconditions 2–3).
    pub fn register(&self, slug: OrgSlug, id: OrgId) -> Result<(), RegistrationError> {
        let mut map = self.inner.write().expect("OrgRegistry RwLock poisoned");

        // Check for slug conflict: slug already bound to a different id.
        if let Some(&existing_id) = map.get_by_left(&slug) {
            if existing_id == id {
                // Idempotent re-registration of the exact same pair — Ok per D-050.
                return Ok(());
            }
            return Err(RegistrationError::SlugConflict {
                slug,
                existing_id,
                attempted_id: id,
            });
        }

        // Check for id conflict: id already bound to a different slug.
        if let Some(existing_slug) = map.get_by_right(&id).cloned() {
            return Err(RegistrationError::IdConflict {
                id,
                existing_slug,
                attempted_slug: slug,
            });
        }

        // No conflict — insert.
        map.insert(slug, id);
        Ok(())
    }

    /// Number of registered (slug, id) pairs (forward map length).
    ///
    /// The reverse map length is always equal (bijection invariant,
    /// BC-3.1.003 invariant 1).  Used in tests to verify the invariant.
    pub fn len(&self) -> usize {
        self.inner
            .read()
            .expect("OrgRegistry RwLock poisoned")
            .len()
    }

    /// Returns `true` when no pairs are registered.
    pub fn is_empty(&self) -> bool {
        self.inner
            .read()
            .expect("OrgRegistry RwLock poisoned")
            .is_empty()
    }
}

impl Default for OrgRegistry {
    fn default() -> Self {
        Self::new()
    }
}
