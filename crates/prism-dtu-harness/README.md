# prism-dtu-harness

Multi-tenant DTU test harness for Prism integration tests.

Provides `HarnessBuilder` — a fluent builder that spins up in-process or
network-isolated DTU behavioral clones keyed by `(OrgId, DtuType)`.

Requires `feature = "dtu"` (or `cfg(test)`) to access any public items.

---

## Quick start

Single-customer logical-mode test (in-process clones, one TCP port per DTU):

```rust,ignore
#[tokio::test]
async fn single_customer_smoke_test() {
    let harness = prism_dtu_harness::Harness::builder()
        .with_customer("acme-corp")
        .build()
        .await
        .expect("harness build must succeed");

    // Access the Claroty clone endpoint.
    let addr = harness
        .endpoint_for("acme-corp", DtuType::Claroty)
        .expect("acme-corp/Claroty must be registered");

    let resp = reqwest::get(format!("http://{addr}/assets/v1/assets"))
        .await
        .expect("GET must succeed");

    assert_eq!(resp.status(), 200);
}
```

---

## Multi-tenant scenarios

Three-org test with `IsolationMode::Network` (each `(org, dtu)` gets its own TCP port):

```rust,ignore
#[tokio::test]
async fn multi_tenant_network_mode() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer("acme-corp")
        .with_customer("globex")
        .with_customer("initech")
        .build()
        .await
        .expect("network-mode harness build must succeed");

    // customer_endpoints() returns the (OrgId, DtuType) → SocketAddr table.
    let endpoints = harness.customer_endpoints();
    // 3 orgs × 4 DTU types = 12 distinct endpoints.
    assert_eq!(endpoints.len(), 12);

    // Cross-org credential mismatch: acme-corp token sent to globex endpoint returns 401.
    let globex_claroty = harness
        .endpoint_for("globex", DtuType::Claroty)
        .expect("globex/Claroty must exist");
    let acme_token = harness
        .admin_token_for("acme-corp", DtuType::Claroty)
        .expect("acme-corp token must exist");

    let status = reqwest::Client::new()
        .get(format!("http://{globex_claroty}/assets/v1/assets"))
        .header("authorization", format!("Bearer {acme_token}"))
        .send()
        .await
        .expect("GET must not fail at network level")
        .status();

    assert_eq!(status, 401, "wrong-org token must be rejected");
}
```

---

## Failure injection

`inject_failure` / `clear_failure` change a live clone's behavior at any point during a test:

```rust,ignore
#[tokio::test]
async fn failure_injection_example() {
    let harness = prism_dtu_harness::Harness::builder()
        .with_customer("acme-corp")
        .build()
        .await
        .expect("build must succeed");

    let addr = harness
        .endpoint_for("acme-corp", DtuType::Claroty)
        .expect("endpoint must exist");

    // Baseline: 200 OK.
    assert_eq!(
        reqwest::get(format!("http://{addr}/assets/v1/assets"))
            .await
            .unwrap()
            .status(),
        200
    );

    // Inject AuthReject.
    harness
        .inject_failure("acme-corp", DtuType::Claroty, FailureMode::AuthReject)
        .await
        .expect("inject must succeed");

    assert_eq!(
        reqwest::get(format!("http://{addr}/assets/v1/assets"))
            .await
            .unwrap()
            .status(),
        401
    );

    // Clear failure — clone returns 200 again.
    harness
        .clear_failure("acme-corp", DtuType::Claroty)
        .await
        .expect("clear must succeed");

    assert_eq!(
        reqwest::get(format!("http://{addr}/assets/v1/assets"))
            .await
            .unwrap()
            .status(),
        200
    );
}
```

---

## Per-test overrides

`with_customer_overrides` and `with_failure` allow per-test customisation of
archetype, scale, seed, and initial failure mode **at builder time** — no raw
`CustomerSpec` construction needed:

```rust,ignore
#[tokio::test]
async fn per_test_override_example() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        // Register acme-corp, then narrow it to Claroty-only with a custom seed.
        .with_customer("acme-corp")
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed_override = Some(42);
        })
        // Register globex with a pre-injected AuthReject on Claroty.
        .with_customer("globex")
        .with_failure("globex", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await
        .expect("harness build must succeed");

    // acme-corp: 1 endpoint (Claroty only).
    assert!(harness.endpoint_for("acme-corp", DtuType::Claroty).is_some());
    assert!(harness.endpoint_for("acme-corp", DtuType::Armis).is_none());

    // globex: Claroty returns 401 immediately — no separate inject_failure call needed.
    let globex_addr = harness
        .endpoint_for("globex", DtuType::Claroty)
        .expect("globex/Claroty must exist");

    assert_eq!(
        reqwest::get(format!("http://{globex_addr}/assets/v1/assets"))
            .await
            .unwrap()
            .status(),
        401
    );
}
```

### Override field reference

| Field | Type | Applied in `build()` |
|-------|------|----------------------|
| `archetype` | `Option<Archetype>` | Clone initialised with specified archetype |
| `scale` | `Option<f64>` | Overrides `GenOpts.scale` (fixture count multiplier) |
| `seed_override` | `Option<u64>` | Overrides `seed` for deterministic device ID generation |
| `initial_failure` | `Option<FailureMode>` | `inject_failure` called before `build()` returns |

All four fields default to `None` — existing tests using `CustomerSpec::new` are unaffected.
