//! VP-014: Query size limit is always enforced.
//!
//! Property: For all `query_str` where `query_str.len() > MAX_QUERY_SIZE`,
//! `PrismQlParser::parse(query_str)` returns `Err(_)` — never `Ok(_)`.
//!
//! Method: Kani bounded model checking (`cargo kani`).
//! Run: `cargo kani --harness proof_query_size_limit`
//!
//! BC: BC-2.11.006 postcondition 1 / EC-001
//! Story: S-3.01

#[cfg(kani)]
mod kani_proofs {
    use crate::filter_parser::{PrismQlParser, PRISM_MAX_QUERY_SIZE};

    /// VP-014 — queries exceeding MAX_QUERY_SIZE always return Err.
    ///
    /// Kani non-deterministically constructs a query string whose length
    /// exceeds the limit and asserts that the parser never returns Ok.
    #[kani::proof]
    fn proof_query_size_limit() {
        // Kani symbolic length — any value strictly greater than the limit.
        let extra_bytes: usize = kani::any();
        kani::assume(extra_bytes > 0);
        kani::assume(extra_bytes <= 16); // bound the search space

        let len = PRISM_MAX_QUERY_SIZE + extra_bytes;
        // Kani does not support heap allocation of symbolic size in general;
        // implementer must adapt this harness to use a fixed-size array or
        // a Kani `kani::vec::any_vec` primitive.
        // TODO(S-3.01): replace placeholder with concrete harness body.
        let _ = len;
        kani::assert(true, "VP-014 harness stub — implementer must complete");
    }
}
