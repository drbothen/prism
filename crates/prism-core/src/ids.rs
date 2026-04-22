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
