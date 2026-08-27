# Inactive C#/Go successor release staging

This directory contains the reviewed, deterministic successor release
descriptors and generated artifacts for `CSHARP-02-T12` and
`CSHARP-02-T13`. They are source-only staging inputs and are not searched,
copied, or accepted by the active Go/Rust release resolver.

- `semantic-profile-registry.json` is the frozen inactive revision-2 semantic
  registry transport.
- `csharp-bundle-candidate.json` is the source-only C# candidate projection.
- `bundle-registry.json` is the byte-exact registry used only by the private
  staged installed-tree fixture.
- `go-bundle-candidate.json` and `go-bundle-registry.json` are the exact
  source-only successor Go candidate and standalone one-tuple registry.
- `go/` contains the complete deterministic successor Go corpus and the
  zero-change semantic-difference report against the active fixtures.

`scripts/build-release-bundles.sh --check csharp` rebuilds the frontend and
toolchain projections twice from pinned caches and byte-compares all three
JSON transports. `scripts/check-release-bundles.sh --fixture csharp` builds an
ephemeral installed tree and exercises it through the private staged runner.
The selected glibc 2.27 closure includes the hash-pinned `libc.so`
compatibility bridge built from `release/build-inputs/csharp/libc-compat.c`;
it exposes only the two stable stat entry points required by the frozen .NET
runtime and links them to the selected `libc.so.6`.

`scripts/build-release-bundles.sh --check go-successor` reproduces the static
staging-only `go2vir` twice under the pinned offline Go 1.25.0 image and checks
both Go descriptors. `scripts/check-release-bundles.sh --fixture go-successor`
regenerates all 13 positive and eight negative Go cases under the same image,
checks every staged byte, and proves the four semantic-difference counters
remain zero. The overlay verifies exact active-source hashes; it does not edit
or replace the active binary or fixture family.

Only `CSHARP-02-T20` may install these schemas into the active release.
