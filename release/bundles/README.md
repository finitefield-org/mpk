# Active successor release metadata

This directory contains the source metadata for the sole active Go/Rust/C#
helper release. The release remains untrusted: only canonical certificate
bytes and checked theory certificates accepted by the configured source-free
checkers support proof acceptance.

`semantic-profile-registry.json` is the frozen revision-2
`mpk.semantic_profile.registry.v1` registry. It contains exactly the Go fixed,
Rust checked, and C# scalar profiles. `bundle-registry.json` is the canonical
`mpk.release.bundle_registry.v1` transport bound to that semantic registry. It
contains three frontend bundles, three toolchain bundles, and four tuples:

- Go on `linux/amd64`;
- Rust on `i686-unknown-linux-gnu`;
- Rust on `x86_64-unknown-linux-gnu`; and
- C# on `linux-x64`.

The registered bundle pairs are:

- `frontend.go.go2vir.candidate.v1` and
  `toolchain.go.go1.25.0.linux-amd64.candidate.v1`;
- `frontend.rust.rust2vir.candidate.v2` and
  `toolchain.rust.nightly-2025-06-01.candidate.v1`; and
- `frontend.csharp.csharp2vir.candidate.v1` and
  `toolchain.csharp.roslyn-5_6_0.dotnet-10_0_11.candidate.v1`.

Each `.candidate.vN` suffix is part of its frozen reviewed identifier; Rust v2
supersedes the unreproducible v1 descriptor without mutating that identity. The
suffix does not create a second publication state. The JSON files under
`candidates/` are source-only projections used to reproduce the combined active
registry; an installed release never copies that directory or resolves it at
runtime.

Verify the complete release deterministically and without repository writes:

```sh
./scripts/build-release-bundles.sh --check successor
./scripts/check-release-bundles.sh --fixture successor
```

The first command rebuilds every bundle twice from pinned inputs and compares
the results. The second materializes one installed image and runs its Go,
Rust, and C# frontends through the registered descriptor-relative paths. Both
commands are offline and fail closed when the required frozen cache, host
tools, Linux isolation features, or root-owned cgroup boundary are absent.

An installed image contains exactly:

- `bin/mpk`;
- `share/mpk/semantic-profile-registry.json`;
- `share/mpk/bundle-registry.json`; and
- the six registered inventories below
  `libexec/mpk/bundles/<bundle_id>`.

There is no registry override, raw executable path, compatibility flag, or
staging-tree lookup. Rollback replaces the whole release image; it never mixes
predecessor and successor identities.
