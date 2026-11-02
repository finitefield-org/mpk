# Omitted generated review records

GitHub push protection rejected two optional generated review records because
the JSON encoding of a Go test fixture's ZIP checksum resembled a FullStory API
key. The value came from the synthetic module `example.com/a@v1.0.0`. For each
record, the retained ZIP had two synthetic entries, and independently applying
Go's `dirhash.Hash1` algorithm to those entries reproduced the recorded ZIP
checksum. The checksum is not an API credential.

The only files omitted from the unpushed history were the two
`a325bdb3cec922912aec54483a907f419daab7cafd492a7f4bfb51daa73463f2.graph-generated-originals.json`
records under `observation-json/owner-final/native-originals/versioned-owned/`
and `observation-json/owner-large/native-originals/versioned-owned/`. They
were generated review outputs rather than inputs directly read by the Go test
or the Rust graph-input test. Product source, test source, source fixtures,
proof inputs and their limits remain unchanged.

Their original bytes, paths and mode `0600` were retained locally in
`/private/tmp/mpk-rejected-evidence-20261102/original-generated-evidence.tar`
and restored to a separate directory with exact SHA-256 and mode checks. The
original owner-final SHA-256 is
`d1d76133463f1934a29e5a1390fa08aeba963000b2f552b78d4bd018592226c8`;
the owner-large SHA-256 is
`63680e02a600e942dbcac3d7c3756ddd15d19b850b36f140d35f455a4a2a5089`.
The complete pre-omission history is retained locally at
`refs/codex/go-practical-before-evidence-removal`, and the accompanying map
records the 31 rewritten commit IDs. These local backups are not part of a new
clone. Replaying the historical `observation-json/review.py` over the original
generated outputs requires restoring those records first.

Verification is scoped to the affected boundary: the Rust
`large_original_native_graph_and_export_streams_survive_full_handoff` test
reads the complete 5,697-object, 182-package original input; the Go
`TestPracticalModuleGraphPreservesUnselectedOldLeavesCyclesAndPrunedInputs`
test exercises the generator for the omitted records. The T02 full gate remains
deferred to T02-W04, and these checks do not establish native Linux release
acceptance or application proof.

Both selected tests passed after the omission: the Rust graph-input test used
the existing optimized test profile and retained all 5,697 objects, and the Go
test ran unchanged with Go 1.25.0 under local Linux x86-64 emulation. These
tests were selected because they consume the affected original graph input and
generate the affected observation respectively; unrelated tests were not
rerun. The older review's requirement to restore the omitted outputs remains
separate from those passing test results.
