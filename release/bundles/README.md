# Release bundle source metadata

This directory contains reviewed source metadata for the release-bundle
assembler. Its normative format, installation boundary, and state machine are
defined by [`develop/specs/RELEASE_BUNDLES_V0.md`](../../develop/specs/RELEASE_BUNDLES_V0.md).

`bundle-registry.json` is the canonical bootstrap
`mpk.release.bundle_registry.v0` transport: one compact RFC 8785 object plus
one LF. It intentionally contains no registered descriptors or tuples until
the first Go release registration milestone. Its `registry_sha256` must be
recomputed from the canonical object with only that field removed; it is not a
hand-maintained build constant.

The following repository paths have distinct roles and are never copied as an
installed bundle root:

- `release/bundles/candidates/rust` is a temporary, tracked, unregistered Rust
  projection during the staged migration. It is never selectable or installed.
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
