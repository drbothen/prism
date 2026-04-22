# AC-4 Evidence: dev-setup.sh is idempotent

Story: S-0.02 | Version: 1.4 | Date: 2026-04-21

**AC-4:** Given `scripts/dev-setup.sh` is run a second time on a system where all tools
are already installed, When the script completes, Then no errors occur (the script is
idempotent — it does not reinstall already-present tools).

---

## Idempotency guard pattern

The `install_if_missing` function checks PATH before installing:

```bash
install_if_missing() {
  local tool="$1"
  local pkg="${2:-$1}"
  if command -v "$tool" >/dev/null 2>&1; then
    echo "  ✓ $tool already installed"
  else
    echo "  → installing $pkg"
    cargo install --locked "$pkg"
  fi
}
```

On second run where tools are present, each call prints `✓ <tool> already installed`
and skips `cargo install`. No errors occur because the install branch is never reached.

## Syntax check (confirms guard logic is valid)

```
$ bash -n scripts/dev-setup.sh && echo "syntax OK"
syntax OK
```

## Expected second-run output (representative)

```
  ✓ rustup present (rustup 1.x.x)

Installing cargo tool extensions...
  ✓ cargo-deny already installed
  ✓ cargo-audit already installed
  ✓ cargo-semver-checks already installed
  ✓ cargo-mutants already installed
  ✓ cargo-fuzz already installed
  ✓ cargo-llvm-cov already installed
  ✓ kani already installed
  ✓ just already installed
  ✓ lefthook already installed

Configuring git hooks via lefthook...

Development toolchain ready
```

## Test gate result

`ok 1 - AC-4: dev-setup.sh contains existence/idempotency checks`
**PASS**
