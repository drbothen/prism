// VP-051: Case State Machine — Exhaustive 5×5 Transition Table.
//
// Enumerates all 25 (from, to) pairs.  For each pair asserts `advance_case_state`
// returns Ok for the 12 valid pairs and the correct Err variant for the 13 invalid
// pairs (E-CASE-005 for self-transitions, E-CASE-004 for other invalids).
//
// Traces to: BC-2.14.002 postconditions.

#[cfg(kani)]
mod kani_proofs {
    use crate::case::{advance_case_state, CaseStatus, CaseTransitionError, VALID_TRANSITIONS};

    /// VP-051 — all 25 pairs produce the correct Ok/Err outcome.
    #[kani::proof]
    fn proof_exhaustive_transition_table() {
        let from: CaseStatus = kani::any();
        let to: CaseStatus = kani::any();

        let result = advance_case_state(from, to);

        if from == to {
            // Self-transition: must return E-CASE-005
            kani::assert(
                matches!(result, Err(CaseTransitionError::SelfTransition)),
                "self-transition must return Err(SelfTransition) E-CASE-005",
            );
        } else if VALID_TRANSITIONS.contains(&(from, to)) {
            // Valid transition: must return Ok(to)
            kani::assert(
                matches!(result, Ok(s) if s == to),
                "valid transition must return Ok(target_state)",
            );
        } else {
            // Invalid non-self transition: must return E-CASE-004
            kani::assert(
                matches!(result, Err(CaseTransitionError::InvalidTransition)),
                "invalid non-self transition must return Err(InvalidTransition) E-CASE-004",
            );
        }
    }
}
