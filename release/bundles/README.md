# Release bundle source metadata

This directory contains reviewed source metadata for the release-bundle
assembler. Its normative format, installation boundary, and state machine are
defined by [`develop/specs/RELEASE_BUNDLES_V0.md`](../../develop/specs/RELEASE_BUNDLES_V0.md).

`bundle-registry.json` is the canonical registered Go/Rust
`mpk.release.bundle_registry.v0` transport: one compact RFC 8785 object plus
one LF. `scripts/build-release-bundles.sh --update all` derives the complete
registered Go/Rust descriptors and inventories from digest-pinned build
inputs; `--check all` performs the same network-disabled build without
repository writes. Its `registry_sha256` is recomputed from the canonical
object with only that field removed; it is not a hand-maintained build
constant.

The active Go tuple is `frontend.go.go2vir.v0` plus
`toolchain.go.go1.25.0.linux-amd64.v0` for `mpk.go.fixed.v0` on
`linux/amd64`. Production policy commands select those IDs and assert the
installed registry ID and digest; raw executable, toolchain, or registry paths
are rejected.

The registered Rust pair is
`frontend.rust.rust2vir.candidate.v0` plus
`toolchain.rust.nightly-2025-06-01.candidate.v0` for
`mpk.rust.checked.v0` on `i686-unknown-linux-gnu` and
`x86_64-unknown-linux-gnu`. The `.candidate.v0` suffix is a frozen historical
identifier; both descriptors are registered registry members. It does not
authorize the removed unregistered candidate publication path or candidate
CLI modes.

The following repository paths have distinct roles and are never copied as an
installed bundle root:

- `release/bundles/candidates/rust` is the removed historical staging path. Its
  presence is invalid after Rust registration, and candidate commands reject.
- `release/build-inputs/rust/build-inputs.json` is a tracked build-only input
  descriptor.
- `release/build-input-cache/rust/<build_inputs_sha256>` is an ignored,
  content-addressed build-only materialization.

An installed release instead contains `bin/mpk`,
`share/mpk/bundle-registry.json`, and exact bundle inventories beneath
`libexec/mpk/bundles/<bundle_id>`. This README, candidates, build-input
descriptors, and build-input caches are excluded from every installation
source. Do not add a registry override, local bundle path, or manual candidate
copy procedure here.

CI may restore the ignored build-input cache, but always validates it against
the tracked descriptor before use. Upgrade and rollback authority is documented
in `develop/docs/rust-frontend-toolchain-upgrade.md`.
