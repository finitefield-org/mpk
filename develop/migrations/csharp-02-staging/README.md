# Inactive C# successor release staging

This directory contains the reviewed, deterministic successor release
descriptors for `CSHARP-02-T12`. They are source-only staging inputs and are
not searched, copied, or accepted by the active Go/Rust release resolver.

- `semantic-profile-registry.json` is the frozen inactive revision-2 semantic
  registry transport.
- `csharp-bundle-candidate.json` is the source-only C# candidate projection.
- `bundle-registry.json` is the byte-exact registry used only by the private
  staged installed-tree fixture.

`scripts/build-release-bundles.sh --check csharp` rebuilds the frontend and
toolchain projections twice from pinned caches and byte-compares all three
JSON transports. `scripts/check-release-bundles.sh --fixture csharp` builds an
ephemeral installed tree and exercises it through the private staged runner.
The selected glibc 2.27 closure includes the hash-pinned `libc.so`
compatibility bridge built from `release/build-inputs/csharp/libc-compat.c`;
it exposes only the two stable stat entry points required by the frozen .NET
runtime and links them to the selected `libc.so.6`.
Only `CSHARP-02-T20` may install these schemas into the active release.
