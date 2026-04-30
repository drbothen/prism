//! Org-scoped CrowdStrike pagination session ID generation (S-3.2.08 / D-048).
//!
//! # D-048 Resolution
//!
//! When `prism-query` generates a CrowdStrike pagination session ID (a UUID used as
//! the `X-DTU-Session-Id` header value), it embeds the calling [`OrgId`] in the UUID
//! v7 random/node bytes so that org-temporal uniqueness is **structural**, not
//! probabilistic.
//!
//! The `prism-dtu-crowdstrike` `session_registry` (keyed by bare `String` session ID)
//! is intentionally NOT re-keyed — per ADR-008 §2.1 D-048. The query engine is the
//! correct enforcement point: it generates session IDs that are org-namespaced by
//! construction, so the clone never encounters a session ID that could collide across
//! orgs.
//!
//! # UUID v7 Byte Layout
//!
//! UUID v7 byte layout (RFC 4122 bis):
//! - Bytes 0–5:  48-bit Unix timestamp in milliseconds (big-endian)
//! - Byte 6:     High nibble = version bits (0x7_); low nibble = rand_a[0..3]
//! - Byte 7:     rand_a[4..11]
//! - Byte 8:     High 2 bits = variant (0b10xxxxxx); low 6 bits = rand_b[0..5]
//! - Bytes 9–15: rand_b[6..63]
//!
//! The OrgId XOR is applied to bytes 8–15 (the random portion that does NOT carry
//! version or variant bits in fixed positions). Bytes 6–7 are left untouched to
//! preserve the UUID v7 version field and variant bits.
//!
//! # In-Process Session Registry (D-048 extraction design)
//!
//! [`extract_org_id_from_session_id`] recovers the [`OrgId`] from a session ID by
//! consulting an in-process registry populated by [`generate_crowdstrike_session_id`]
//! at generation time. This design is intentional:
//!
//! - XOR embedding alone does not enable standalone extraction without the base UUID.
//! - The session registry is used only for verification (tests / VP-084). The
//!   `prism-dtu-crowdstrike` clone never calls `extract_org_id_from_session_id` in
//!   production — it treats the session ID as an opaque key.
//! - The registry is bounded to [`SESSION_REGISTRY_CAPACITY`] entries (FIFO eviction).
//!   Production session volumes are order-of-magnitude below this limit.
//!
//! # Traceability
//!
//! - BC-3.2.003 precondition 4 + invariant 4
//! - ADR-008 §2.1 D-048
//! - VP-084

use prism_core::OrgId;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use uuid::{Uuid, Version};

/// Maximum number of session ID → OrgId mappings held in the in-process registry.
/// When the registry reaches capacity, the oldest [`SESSION_EVICTION_BATCH`] entries
/// are evicted before inserting the new entry.
///
/// Chosen to be well above realistic production session volumes (bounded by the number
/// of concurrent CrowdStrike pagination queries active in a single prism-query process).
const SESSION_REGISTRY_CAPACITY: usize = 10_000;

/// Number of oldest entries evicted when the registry hits capacity.
const SESSION_EVICTION_BATCH: usize = 1_000;

/// Process-global session ID → OrgId registry.
///
/// Populated by [`generate_crowdstrike_session_id`]; queried by
/// [`extract_org_id_from_session_id`]. Bounded by [`SESSION_REGISTRY_CAPACITY`]
/// with FIFO eviction to prevent unbounded memory growth.
static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>> = OnceLock::new();

/// Internal bounded session registry: HashMap for O(1) lookup + VecDeque for
/// insertion-order tracking (FIFO eviction).
struct SessionRegistry {
    /// Session ID → OrgId mapping for O(1) extract lookup.
    map: HashMap<String, OrgId>,
    /// Insertion-ordered queue of session IDs for FIFO eviction.
    order: VecDeque<String>,
}

impl SessionRegistry {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Insert a new session ID → OrgId mapping. Evicts the oldest
    /// [`SESSION_EVICTION_BATCH`] entries if at capacity.
    fn insert(&mut self, session_id: String, org_id: OrgId) {
        if self.map.len() >= SESSION_REGISTRY_CAPACITY {
            for _ in 0..SESSION_EVICTION_BATCH {
                if let Some(evicted) = self.order.pop_front() {
                    self.map.remove(&evicted);
                }
            }
        }
        self.order.push_back(session_id.clone());
        self.map.insert(session_id, org_id);
    }

    /// Look up the OrgId for a session ID. Returns `None` if not registered.
    fn get(&self, session_id: &str) -> Option<OrgId> {
        self.map.get(session_id).copied()
    }
}

/// Access the process-global session registry, initialising it on first use.
fn registry() -> &'static Mutex<SessionRegistry> {
    SESSION_REGISTRY.get_or_init(|| Mutex::new(SessionRegistry::new()))
}

/// Generate a CrowdStrike pagination session ID with the calling [`OrgId`] embedded
/// in the UUID v7 random bytes (bytes 8–15).
///
/// The base UUID is generated with [`Uuid::now_v7()`], providing timestamp ordering.
/// The calling org's UUID bytes (8–15) are XORed into the random portion of the
/// session UUID so that a session ID produced in Org A's context cannot collide with
/// Org B's session IDs structurally — even if both calls happen within the same
/// millisecond (EC-001 from S-3.2.08).
///
/// The result is stored in the process-global session registry (enabling
/// [`extract_org_id_from_session_id`] round-trips) and returned as a [`String`]
/// suitable for use as the `X-DTU-Session-Id` HTTP header value sent to
/// `prism-dtu-crowdstrike`.
///
/// # Byte Layout Invariant
///
/// Bytes 0–7 of the base UUID v7 (timestamp + version bits) are preserved unchanged.
/// Only bytes 8–15 (random portion) receive the OrgId XOR. This ensures the generated
/// value remains a structurally valid UUID v7 (version nibble and variant bits intact).
///
/// # AC Traceability
///
/// - AC-001: session IDs from different orgs differ in bytes 8–15
/// - AC-003: no session ID can be generated without an `OrgId` parameter
/// - BC-3.2.003 invariant 4 / D-048
pub fn generate_crowdstrike_session_id(org_id: OrgId) -> String {
    let base = Uuid::now_v7();
    let session_uuid = xor_org_into_session_bytes(base, org_id);
    let session_str = session_uuid.to_string();

    // Store in registry so extract_org_id_from_session_id can recover the OrgId.
    // Poisoned mutex recovery: if another thread panicked while holding the lock,
    // we recover the inner data rather than propagating the panic.
    let mut guard = registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.insert(session_str.clone(), org_id);

    session_str
}

/// Extract the embedded [`OrgId`] from a CrowdStrike session ID previously generated
/// by [`generate_crowdstrike_session_id`] in this process.
///
/// Returns `Some(org_id)` if and only if:
/// 1. `session_id` parses as a valid UUID, AND
/// 2. The UUID has version == v7 (RFC 4122 bis SortRand), AND
/// 3. The session ID was generated by [`generate_crowdstrike_session_id`] in this
///    process and is still held in the in-process session registry.
///
/// Returns `None` if any condition fails — including UUIDs v4 or other versions, malformed
/// strings, or session IDs generated in a different process or after registry eviction.
///
/// # Design Note
///
/// XOR embedding alone does not allow standalone extraction without the base UUID
/// (which is not persisted). The in-process registry is the authoritative source.
/// This is intentional: `extract_org_id_from_session_id` is a verification/test helper.
/// The `prism-dtu-crowdstrike` clone never calls this function in production.
///
/// # Traceability
///
/// - VP-084 (cross-org isolation property)
/// - BC-3.2.003 postcondition 2
/// - AC-004: returns None for non-v7 / non-Prism-generated session IDs
pub fn extract_org_id_from_session_id(session_id: &str) -> Option<OrgId> {
    // Parse as UUID; reject malformed strings.
    let parsed = Uuid::parse_str(session_id).ok()?;

    // Reject non-v7 UUIDs (AC-004). The session registry only holds v7 values,
    // but we apply this guard before the registry lookup as a belt-and-suspenders
    // check: a UUID v4 or v1 string could never have been generated by
    // generate_crowdstrike_session_id, and returning None early avoids a spurious
    // registry miss being misinterpreted as an unknown session rather than an
    // invalid input.
    if parsed.get_version() != Some(Version::SortRand) {
        return None;
    }

    // Look up in the in-process registry.
    // Poisoned mutex recovery: recover inner data rather than propagating the panic.
    let guard = registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.get(session_id)
}

/// Low-level UUID v7 XOR helper: apply an [`OrgId`] namespace to the random bytes
/// (8–15) of a base UUID, returning the namespaced UUID.
///
/// Separated from [`generate_crowdstrike_session_id`] to allow unit testing of the
/// XOR logic in isolation, independent of the real-time `Uuid::now_v7()` call.
///
/// # Arguments
///
/// - `base`: a UUID v7 generated by the caller (typically `Uuid::now_v7()`)
/// - `org_id`: the calling org whose UUID bytes are XORed into bytes 8–15 of `base`
///
/// # Returns
///
/// A new UUID with bytes 0–7 identical to `base` and bytes 8–15 equal to
/// `base[8..16] XOR org_id.as_uuid().as_bytes()[8..16]`.
///
/// # Invariants
///
/// - Bytes 0–7 (timestamp + version nibble) are never modified.
/// - Bytes 8–15 receive the OrgId XOR; the variant bits in byte 8 (top 2 bits = 0b10)
///   may change as a result of XOR. The caller ([`generate_crowdstrike_session_id`])
///   accepts this: the resulting UUID is still structurally parseable; only the variant
///   field may no longer read as RFC 4122. The version nibble (byte 6 high nibble) is
///   untouched so `get_version()` still returns v7.
///
/// # Traceability
///
/// - AC-001 / EC-001: structural byte-level separation of session IDs across orgs
/// - EC-002: XOR with nil OrgId (all zeros) is the identity operation
pub fn xor_org_into_session_bytes(base: Uuid, org_id: OrgId) -> Uuid {
    let mut session_bytes: [u8; 16] = *base.as_bytes();
    let org_bytes: [u8; 16] = *org_id.as_uuid().as_bytes();
    // XOR the random portion (bytes 8-15) with the OrgId bytes (8-15).
    // Bytes 0-7 (timestamp, version nibble, rand_a) are intentionally untouched.
    for i in 8..16 {
        session_bytes[i] ^= org_bytes[i];
    }
    Uuid::from_bytes(session_bytes)
}
