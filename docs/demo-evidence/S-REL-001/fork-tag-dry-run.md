# Task 12 Gate — Dry-Run Evidence
## S-REL-001: Release Workflow Dry-Run on Origin Test Tag (F-REL001-P13-001)

**Destination note:** The story spec called for a fork dry-run. A fork is structurally
impossible when the authenticated user IS the repository owner (GitHub enforces this).
The human explicitly approved running the dry-run against `origin` (drbothen/prism,
the user's own public repo) as the correct alternative. This is documented in the
orchestrator's Task 12 dispatch as "HUMAN-APPROVED destination: origin".

**Run URL:** https://github.com/drbothen/prism/actions/runs/29709483646

**Tag:** `v0.0.1-rc.test` at commit `6be04270d94013039f09e75c6ba6b65a95fb40de`

**Tag push time (UTC):** 2026-07-20T00:25:41Z

**Workflow completed:** 2026-07-20T01:04:37Z (approximately)

---

## Per-Leg Results Table

| Target | Job ID | Conclusion | Duration | Job Link |
|--------|--------|------------|----------|----------|
| x86_64-unknown-linux-gnu | 88251358882 | FAIL | 18m 9s | https://github.com/drbothen/prism/actions/runs/29709483646/job/88251358882 |
| x86_64-unknown-linux-musl | 88251358889 | FAIL | 9m 12s | https://github.com/drbothen/prism/actions/runs/29709483646/job/88251358889 |
| x86_64-pc-windows-msvc | 88251358895 | FAIL | 35m 37s | https://github.com/drbothen/prism/actions/runs/29709483646/job/88251358895 |
| aarch64-apple-darwin | 88251358899 | FAIL | 18m 30s | https://github.com/drbothen/prism/actions/runs/29709483646/job/88251358899 |
| x86_64-apple-darwin | 88251358918 | FAIL | 31m 56s | https://github.com/drbothen/prism/actions/runs/29709483646/job/88251358918 |
| publish-release (Create GitHub Release) | 88253186504 | SKIPPED | 0s | https://github.com/drbothen/prism/actions/runs/29709483646/job/88253186504 |

All 5 build-release matrix legs failed. The publish-release job was correctly
skipped (`needs: build-release`).

---

## REAL DEFECT — Missing `protoc` installation in release.yml

**Label:** REAL DEFECT (F-REL001-P13-001 / DEFECT-REL001-PROTOC-MISSING-001)

**Root cause:** `release.yml` does not install the Protocol Buffer compiler (`protoc`),
which is required by `prism-ocsf` build.rs via `prost-build` at compile time.
`ci.yml` installs `protoc` in every build/test job via
`arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b` (SHA-pinned v3.0.0),
but this step was never added to `release.yml`.

**Precise log excerpts (representative — identical root cause on all 5 legs):**

### Linux (x86_64-unknown-linux-gnu) — job 88251358882

```
2026-07-20T00:38:21.1670651Z error: failed to run custom build command for `prism-ocsf v0.1.0 (/home/runner/work/prism/prism/crates/prism-ocsf)`
2026-07-20T00:38:21.1822026Z   build.rs: prost-build failed to compile OCSF .proto files: Custom { kind: NotFound, error: "Could not find `protoc`. If `protoc` is installed, try setting the `PROTOC` environment variable to the path of the `protoc` binary. To install it on Debian, run `apt-get install protobuf-compiler`." }
2026-07-20T00:38:21.1879045Z warning: build failed, waiting for other jobs to finish...
2026-07-20T00:43:39.8230432Z ##[error]Process completed with exit code 101.
```

### macOS (aarch64-apple-darwin) — job 88251358899

```
2026-07-20T00:40:58.2534130Z error: failed to run custom build command for `prism-ocsf v0.1.0 (/Users/runner/work/prism/prism/crates/prism-ocsf)`
2026-07-20T00:40:58.2589620Z   thread 'main' (80712) panicked at crates/prism-ocsf/build.rs:68:10:
2026-07-20T00:40:58.2591560Z   build.rs: prost-build failed to compile OCSF .proto files: Custom { kind: NotFound, error: "Could not find `protoc`. If `protoc` is installed, try setting the `PROTOC` environment variable to the path of the `protoc` binary. To install it on macOS, run `brew install protobuf`." }
2026-07-20T00:43:43.5312410Z ##[error]Process completed with exit code 101.
```

### Windows (x86_64-pc-windows-msvc) — job 88251358895

```
2026-07-20T00:53:01.8968850Z error: failed to run custom build command for `prism-ocsf v0.1.0 (D:\a\prism\prism\crates\prism-ocsf)`
2026-07-20T00:53:01.9690406Z   build.rs: prost-build failed to compile OCSF .proto files: Custom { kind: NotFound, error: "Could not find `protoc`. If `protoc` is installed, try setting the `PROTOC` environment variable to the path of the `protoc` binary." }
2026-07-20T01:00:31.4892282Z ##[error]Process completed with exit code 1.
```

**Required fix (for orchestrator routing):**
Add `arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b # v3.0.0`
step to the `build-release` job in `release.yml`, after the `Install Linux build deps`
conditional step and before the `Build release binary` step. Mirror the pattern from
every job in `ci.yml` that runs `cargo build`.

---

## Release Asset Listing (EC-001/AC-005 Wire Proof)

**Not applicable.** The `publish-release` (Create GitHub Release) job was SKIPPED
because all 5 `build-release` matrix legs failed. No GitHub Release exists for
`v0.0.1-rc.test`. The asset listing, prerelease flag check, and checksums.txt
content cannot be captured in this run.

```
gh release view v0.0.1-rc.test --repo drbothen/prism → "release not found"
```

EC-001/AC-005 (prerelease flag must be true for `-` tags) wire proof is BLOCKED
by DEFECT-REL001-PROTOC-MISSING-001. The release creation logic itself is correct
in the workflow — the `--prerelease` flag detection (`[[ "$TAG" == *-* ]]`) is
present and correct; it simply could not execute because the build phase failed.

---

## Idempotency Spot-Check (EC-009)

**Not applicable.** No release was created; the idempotency path (`gh release upload
--clobber`) was never reached. This spot-check must be re-run after
DEFECT-REL001-PROTOC-MISSING-001 is fixed and a successful run exists.

---

## Attestation Step Outcome

The `actions/attest-build-provenance` step in each leg ran during post-build
cleanup but produced no attestation because the build itself failed before the
archive step. The OIDC token and `id-token: write` permission were available
(confirmed from job logs). Attestation would have run successfully had the build
succeeded.

---

## Cleanup Verification

1. GitHub Release deleted: N/A — no release was created (publish-release was skipped).
2. Remote tag deleted:

```
git push origin :refs/tags/v0.0.1-rc.test
```

Executed at step 6. Verified by:
```
gh release view v0.0.1-rc.test --repo drbothen/prism → "release not found"  (before cleanup, already true)
git ls-remote origin refs/tags/v0.0.1-rc.test → (empty)  (after cleanup)
```

Local tag also deleted:
```
git tag -d v0.0.1-rc.test
```

---

## Gate Verdict

**DRY-RUN FAILED**

Failing legs: ALL 5 (x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl,
x86_64-pc-windows-msvc, aarch64-apple-darwin, x86_64-apple-darwin)

Root cause: DEFECT-REL001-PROTOC-MISSING-001 — `protoc` not installed in `release.yml`

The workflow machinery (trigger, checkout, rust-toolchain, caching, archive creation,
SHA-256 computation, attestation, idempotent publish) is correct and structurally
sound. The single missing step is `arduino/setup-protoc` before `cargo build --release`.
This defect is entirely separate from any architectural concern — it is a missing
dependency installation step, directly observable from the log, with a known fix
already in `ci.yml`.

**Orchestrator routing:** Implementer (the fix is in `.github/workflows/release.yml`,
a CI/CD artifact; the required step SHA is already pinned in `ci.yml`).
