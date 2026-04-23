// VP-005: Exactly 12 valid CaseStatus transitions.
// VP-006: No CaseStatus self-transitions.
//
// These proofs run only under `cargo kani` (cfg(kani)).
// Under normal `cargo test` they compile but do not execute.

#[cfg(kani)]
mod kani_proofs {
    use crate::case::{CaseStatus, VALID_TRANSITIONS};

    /// VP-005 — Exhaustive 5×5 check: exactly 12 pairs return true.
    ///
    /// Kani enumerates all symbolic (current, target) pairs; the assertion
    /// ensures `can_transition_to` returns `true` iff the pair is in
    /// `VALID_TRANSITIONS`.
    #[kani::proof]
    fn proof_exactly_12_transitions() {
        let current: CaseStatus = kani::any();
        let target: CaseStatus = kani::any();
        let allowed = current.can_transition_to(target);
        let expected = VALID_TRANSITIONS.contains(&(current, target));
        kani::assert(
            allowed == expected,
            "can_transition_to must return true iff pair is in VALID_TRANSITIONS",
        );
    }

    /// VP-006 — Every variant rejects a self-transition.
    #[kani::proof]
    fn proof_no_self_transitions() {
        for status in CaseStatus::ALL {
            kani::assert(
                !status.can_transition_to(status),
                "self-transition must always return false",
            );
        }
    }
}
