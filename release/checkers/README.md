# Embedded Reference Checker Asset

`mpk-checker-ref-linux-amd64` is the deterministic, statically linked build of
the independent Go reference checker embedded into `bin/mpk`. It is not an
installed sibling executable, release-bundle inventory member, registry
executable, plugin, or proof-evidence input.

The asset is built from `go-tools/mpk-checker-ref` with the digest-pinned Go
image and closed build environment in `scripts/release_bundles.py`. The
`check-all` release gate rebuilds it and requires byte equality. Runtime code
copies only these embedded bytes into a sealed anonymous executable, passes the
candidate certificate on standard input, and never resolves Go, checker source,
or a checker path from the host.

Regenerate the asset only after a reviewed checker change:

```text
/usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
  /usr/bin/python3 -B scripts/release_bundles.py update-reference-checker
```
