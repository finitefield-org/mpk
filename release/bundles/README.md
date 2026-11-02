# Go and Rust release metadata

`semantic-profile-registry.json` is the canonical revision-4
`mpk.semantic_profile.registry.v1` registry. It contains exactly
`mpk.go.fixed.v0` and `mpk.rust.checked.v0`, preserving both entry hashes.
`bundle-registry.json` binds two frontend bundles, two toolchain bundles,
and three target tuples:

- Go on `linux/amd64`;
- Rust on `i686-unknown-linux-gnu`;
- Rust on `x86_64-unknown-linux-gnu`.

The registered pairs are `frontend.go.go2vir.candidate.v2` with
`toolchain.go.go1.25.0.linux-amd64.candidate.v1`, and
`frontend.rust.rust2vir.candidate.v3` with
`toolchain.rust.nightly-2025-06-01.candidate.v1`. Candidate suffixes are
immutable identifiers. Projections under `candidates/` reproduce the combined
registry; installed consumers never load that directory.

On a reviewed native x86-64 Linux host with writable cgroup v2 and frozen
build-input caches, run `sudo ./scripts/check-go-rust-frontend.sh`.
It rebuilds the bundles and runs both frontends through descriptor-relative
installed paths twice. The new registry requires fresh native acceptance;
older receipts cannot attest to these metadata bytes.

The installed image contains `bin/mpk`, both registries in `share/mpk`,
and exactly four inventories under `libexec/mpk/bundles/<bundle_id>`.
Unknown languages, old registry identities, caller-supplied executables and
alternate registry paths are rejected. All release metadata is untrusted;
proof acceptance still requires checked certificate bytes.
