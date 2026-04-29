// S-1.02 — UUID v7 ID newtypes.
//
// All IDs use UUID v7 (time-ordered) so that RocksDB iteration is monotonically
// increasing by creation time.  UUID v4 (random) is explicitly prohibited per
// Architecture Compliance Rules.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! uuid_v7_newtype {
    (
        $(#[$attr:meta])*
        $name:ident
    ) => {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Allocate a new ID with a UUID v7 timestamp-ordered value.
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Wrap an existing `Uuid` (used during deserialization from storage).
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            /// Return the inner `Uuid`.
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

uuid_v7_newtype!(
    /// Identifier for a scheduled job (SS-12).
    ScheduleId
);

uuid_v7_newtype!(
    /// Identifier for a detection rule (SS-14).
    RuleId
);

uuid_v7_newtype!(
    /// Identifier for a security case (SS-14).
    CaseId
);

uuid_v7_newtype!(
    /// Identifier for an alert (SS-14).
    AlertId
);

uuid_v7_newtype!(
    /// Stable canonical organisation identity (SS-21 / S-3.1.01).
    ///
    /// Backed by UUID v7 (time-ordered). Use `OrgId::new()` to mint a fresh
    /// identifier or `OrgId::from_uuid(u)` to wrap a value obtained from
    /// persistent storage. All downstream crates (prism-credentials,
    /// prism-sensors, prism-audit) key their stores on this type rather than
    /// the mutable `OrgSlug` display string.
    OrgId
);

impl OrgId {
    /// Wrap a `Uuid`, enforcing that it is UUID v7.
    ///
    /// # Panics
    ///
    /// Panics with `"not a UUID v7"` if `uuid.get_version_num() != 7`.
    /// Use this when the version contract must be enforced at construction time
    /// (BC-3.1.001 precondition 3).
    pub fn from_uuid_v7(uuid: Uuid) -> Self {
        assert_eq!(
            uuid.get_version_num(),
            7,
            "not a UUID v7: received version {}",
            uuid.get_version_num()
        );
        Self(uuid)
    }
}

impl std::fmt::Display for OrgId {
    /// Formats as the bare hyphenated lowercase UUID string,
    /// e.g. `"018e3f71-5c6d-7a8b-9c0d-1e2f3a4b5c6d"` (BC-3.1.001 invariant 3).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
