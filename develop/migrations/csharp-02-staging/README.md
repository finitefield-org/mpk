# Inactive C#/Go/Rust successor release staging

This directory contains the reviewed, deterministic successor release
descriptors and generated artifacts for `CSHARP-02-T12` through
`CSHARP-02-T15`. They are source-only staging inputs and are not searched,
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
- `rust-bundle-candidate.json` and `rust-bundle-registry.json` are the exact
  source-only two-target successor Rust candidate and standalone registry.
- `rust/` contains the complete deterministic successor Rust public/private
  corpus, 73-case negative diagnostic inventory, and zero-change semantic-
  difference report against the active fixtures.
- These validated Go/Rust artifacts and the C# candidate context feed only the
  explicitly injected T15 successor VC test boundary. Canonical successor VC
  and skeleton bytes are regenerated in tests and are not installed or
  published from this directory.

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

`scripts/build-release-bundles.sh --check rust-successor` reproduces the
staging-only `rust2vir` and `rust2vir-driver` binaries twice under the pinned
offline nightly and checks both Rust descriptors. The fixture route,
`scripts/check-release-bundles.sh --fixture rust-successor`, regenerates all
13 positive cases, runs the complete 73-case negative corpus plus private-
protocol identity gate, and checks every staged byte. The
`--fixture-update rust-successor` form is the explicit
fixture publication route. The eleven-file overlay verifies exact active-
source hashes and never edits or replaces the active Rust binaries or fixture
family.

Only `CSHARP-02-T20` may install these schemas into the active release.
