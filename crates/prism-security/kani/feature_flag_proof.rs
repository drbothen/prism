// VP-020: Kani Proof — Compile-Time Feature Disabled → check_permission Always Returns Deny
//
// Story:  S-1.08 — prism-security: Feature Flags (P0 Core)
// VP:     VP-020
// Source: BC-2.04.004 — Two-Tier Gate — Both Compile-Time and Runtime Must Permit
//
// Property statement (VP-020):
//   For every capability path `p`, `is_allowed(p, &ctx)` returns `true` if and only if
//   both gates pass: the compile-time Cargo feature for the enclosing code family is
//   enabled, AND the runtime capability evaluation for the tenant returns `Allow`.
//   Either gate alone denying forces the combined result to `false`.
//
// Feasibility (from VP-020 spec):
//   "Bounded inputs: 4-combination truth table. Kani trivially handles bool logic.
//    Compile-time gate modeled as runtime bool in test; separate build-matrix test
//    covers the real cfg gate."
//
// This harness models the compile-time gate as a symbolic bool per VP-020's
// feasibility assessment. The 2×2 truth table exhaustively covers all combinations.

#[cfg(kani)]
mod proofs {
    use std::collections::BTreeMap;

    use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
    use prism_security::feature_flag::{
        CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator,
    };

    // ─────────────────────────────────────────────────────────────
    // VP-020 Proof Harness
    // ─────────────────────────────────────────────────────────────

    /// VP-020: Two-tier gate truth table.
    ///
    /// Symbolically checks all four combinations of (compile_ok, runtime_allow)
    /// and asserts that `check_permission` returns `Allowed` iff BOTH are true.
    #[kani::proof]
    fn proof_vp020_two_tier_gate_truth_table() {
        // Symbolic inputs: the two boolean gates.
        let compile_ok: bool = kani::any();
        let runtime_allow: bool = kani::any();

        // Construct a minimal evaluator for this proof.
        // The capability path is fixed — VP-020 is about gate logic, not path resolution.
        let capability = "sensor.crowdstrike.containment";
        let client_id = "acme";

        // Build runtime capability map: either Allow or Deny for the test path.
        let mut caps = ClientCapabilities::new();
        if runtime_allow {
            let path = CapabilityPath::new(capability).expect("valid path");
            caps.grant(path, CapabilityEffect::Allow);
        }
        // (if !runtime_allow, deny-by-default applies — no explicit entry needed)

        let mut client_map = BTreeMap::new();
        client_map.insert(client_id.to_string(), caps);

        let evaluator = FeatureFlagEvaluator::new(client_map);

        let compile_gate = if compile_ok {
            CompileTimeGate::Present
        } else {
            CompileTimeGate::Absent
        };

        let result = evaluator.check_permission(compile_gate, client_id, capability);

        let allowed = matches!(result, CapabilityCheckResult::Allowed);

        // VP-020 core assertion: result == (compile_ok AND runtime_allow)
        kani::assert(
            allowed == (compile_ok && runtime_allow),
            "VP-020: check_permission must return Allowed iff BOTH gates pass",
        );

        // VP-020 corollary A: compile_ok == false → always Denied (regardless of runtime)
        if !compile_ok {
            kani::assert(
                matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
                "VP-020 corollary A: compile gate absent → DeniedCompileTime regardless of runtime",
            );
        }

        // VP-020 corollary B: compile_ok == true && runtime_allow == false → DeniedRuntime
        if compile_ok && !runtime_allow {
            kani::assert(
                matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
                "VP-020 corollary B: compile gate present, runtime deny → DeniedRuntime",
            );
        }
    }

    /// VP-020 supplementary: compile-time absent + runtime explicitly Allow still denies.
    ///
    /// This is the critical security property: runtime configuration CANNOT override
    /// a missing compile-time feature (BC-2.04.001, BC-2.04.004 invariant).
    #[kani::proof]
    fn proof_vp020_compile_absent_runtime_allow_still_denies() {
        let capability = "sensor.crowdstrike.containment";
        let client_id = "acme";

        // Runtime config explicitly allows — compile gate is absent.
        let mut caps = ClientCapabilities::new();
        let path = CapabilityPath::new(capability).expect("valid path");
        caps.grant(path, CapabilityEffect::Allow);

        let mut client_map = BTreeMap::new();
        client_map.insert(client_id.to_string(), caps);

        let evaluator = FeatureFlagEvaluator::new(client_map);

        let result = evaluator.check_permission(
            CompileTimeGate::Absent, // feature NOT compiled in
            client_id,
            capability,
        );

        kani::assert(
            matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
            "VP-020: compile-absent with runtime Allow must still return DeniedCompileTime",
        );
    }
}
